//! Port of `bot/understanding/topics.py`: what the line is about (topics with their scores,
//! the entities it names, the numbers it gives).

#![allow(non_upper_case_globals)]

use crate::py::re::Re;
use crate::py::text;
use crate::understanding::kinds::{_asks_back, a_bad_verdict, small_talk_only};
use crate::understanding::normalize::{background_positions, reason_positions, Line};
use crate::understanding::{match_at, pyuni, rx, t};
use crate::understood::{Entity, Num, Number};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.understanding.topics.";

// ---- PyBot's tables (bot.understanding.topics.*) ----
static ADDRESS_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ADDRESS_WORDS"));
static AGE_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AGE_AFTER"));
static AGE_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AGE_NOUNS"));
static AGE_OF_A_CHILD_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AGE_OF_A_CHILD_WORDS"));
static ANIMALS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ANIMALS"));
static ARTICLES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ARTICLES"));
static ASK_PHRASES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_PHRASES"));
static ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASK_WORDS"));
static A_AMOUNT_NOUN: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| t::map_int(M, "A_AMOUNT_NOUN"));
static A_AMOUNT_SYMBOL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_AMOUNT_SYMBOL"));
const A_CHILD_TILL: i64 = 19;
static A_CHILD_TO_READ_FOR: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_CHILD_TO_READ_FOR"));
static A_GUT_ONLY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_GUT_ONLY"));
static A_MONEY_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_MONEY_WORD"));
static A_PERSON_WHO_WANTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PERSON_WHO_WANTS"));
static A_PORTION_NEEDS_OF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PORTION_NEEDS_OF"));
static A_PORTION_UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_PORTION_UNITS"));
static A_THING_WANTED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_THING_WANTED"));
static A_WAIT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_WAIT"));
static A_WAIT_TO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_WAIT_TO"));
static BEHAVIOUR: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BEHAVIOUR"));
const BIG_PURSE: i64 = 500;
static BODILY_OUTPUT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODILY_OUTPUT"));
static BODILY_OUTPUT_NOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODILY_OUTPUT_NOUN"));
static BODY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODY"));
static BODY_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODY_CUES"));
static BODY_OR_SELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODY_OR_SELF"));
static BODY_TWO_LIFE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BODY_TWO_LIFE"));
static BOOK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BOOK_WORDS"));
static BROKE_AS_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BROKE_AS_SUBJECT"));
static BROKE_AS_TIME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "BROKE_AS_TIME"));
static CANCELS: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| t::re_str_pairs(M, "CANCELS"));
static CANCEL_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CANCEL_AFTER"));
const CARRY: i64 = 2;
static CAR_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CAR_NAMES"));
static CLIPPED_AFTER_A_SLASH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CLIPPED_AFTER_A_SLASH"));
static CLIPPED_UNIT_WORDS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "CLIPPED_UNIT_WORDS"));
static COLD_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COLD_PEOPLE"));
static COMMON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMMON_WORDS"));
static COMMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMMS"));
static COMPARE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMPARE_WORDS"));
static COMPLAINT_OF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMPLAINT_OF"));
static COMPLAINT_OF_NEXT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMPLAINT_OF_NEXT"));
static CONDITIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONDITIONS"));
const CONDITION_BOOST: i64 = 4;
static CONDITION_NEEDS_PART: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CONDITION_NEEDS_PART"));
const CONDITION_REACH: i64 = 3;
const COUNTRY_BOOST: i64 = 6;
static COUNTRY_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COUNTRY_CUES"));
static COUNTRY_HOMOGRAPHS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COUNTRY_HOMOGRAPHS"));
static CROPS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CROPS"));
const CROP_BOOST: i64 = 3;
static CROP_KITCHEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CROP_KITCHEN"));
static CROP_STATES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "CROP_STATES"));  // t, 22 items
static CUE_CARRIERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CUE_CARRIERS"));
static DEGREE_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DEGREE_NOUNS"));
static DERIVED_ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "DERIVED_ENDINGS"));  // t, 26 items
static DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DETERMINERS"));
static DRIVEN_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DRIVEN_JOINERS"));
static DRIVEN_MOOD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DRIVEN_MOOD"));
static DRIVEN_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DRIVEN_PERSON"));
static DRIVEN_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DRIVEN_TAIL"));
static DRIVES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DRIVES"));
static EXERCISE_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EXERCISE_AFTER"));
static EXERCISE_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EXERCISE_NOUNS"));
static EXTRA_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "EXTRA_PEOPLE"));
static FAMILY_PAIRS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "FAMILY_PAIRS"));
const FAMILY_STEM: i64 = 5;
const FAMILY_WORD: i64 = 5;
static FIGURE_SIZE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FIGURE_SIZE_WORDS"));
const FRIEND_BOOST: i64 = 1;
static FRIEND_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FRIEND_WORDS"));
static GENERIC: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GENERIC"));
static GIFTS_OR_OCCASIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIFTS_OR_OCCASIONS"));
static GIFT_ALWAYS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIFT_ALWAYS"));
static GIFT_ASKING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIFT_ASKING"));
const GIFT_BOOST: i64 = 5;
static GIFT_OCCASION: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIFT_OCCASION"));
static GIFT_WANTING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIFT_WANTING"));
static GIFT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIFT_WORDS"));
static GIVEN_UP_FOOD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVEN_UP_FOOD"));
static GIVE_DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVE_DETERMINERS"));
static GIVE_RELATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVE_RELATIVE"));
static GIVE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVE_THING"));
static GIVE_THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVE_THINGS"));
static GIVE_TO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVE_TO"));
static GIVE_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GIVE_VERB"));
static GIVING_UP: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "GIVING_UP"));  // t, 14 items
const GIVING_UP_BOOST: i64 = 3;
static GROWING_PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "GROWING_PLACES"));
static HEAD_FOG: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HEAD_FOG"));
const HEAD_FOG_BOOST: i64 = 6;
const HEAD_FOG_REACH: i64 = 4;
static HEAD_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HEAD_WORDS"));
static HEARERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HEARERS"));
static HURTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "HURTS"));
const HURT_BOOST: i64 = 5;
static IDIOM_PHRASES: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "IDIOM_PHRASES"));
static JUDGEMENT_OF_RECORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JUDGEMENT_OF_RECORD"));
static LEADING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LEADING"));
static MACHINE_BIN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MACHINE_BIN_WORDS"));
static MACHINE_WORD_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "MACHINE_WORD_PHRASES"));  // t, 12 items
static MECHANISM: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "MECHANISM"));  // t, 2 items
static MECHANISM_HOW: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MECHANISM_HOW"));
static MIXED_FRACTION: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MIXED_FRACTION"));
static MIXED_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MIXED_JOINERS"));
static MONEY_TIGHT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "MONEY_TIGHT"));  // t, 24 items
const MONEY_TIGHT_MAX: i64 = 10;
static NAME_BREAKS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NAME_BREAKS"));
static NAME_FRAMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NAME_FRAMES"));
static NATURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NATURE_WORDS"));
static NOSE_BLOCKED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOSE_BLOCKED"));
const NOSE_BLOCKED_BOOST: i64 = 6;
static NOT_CUT_OUT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_CUT_OUT"));
static NOT_PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_PLACES"));
static NOT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_WORDS"));
static NOUNY_SIGNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOUNY_SIGNS"));
static NUMBER_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NUMBER_FILLER"));
static NUMBER_RE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NUMBER_RE"));
static NUMBER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NUMBER_WORDS"));
static ONE_IS_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ONE_IS_A_THING"));
static ONLY_A_STOOL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ONLY_A_STOOL"));
static ORDINARY_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ORDINARY_VERBS"));
static OTHER_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OTHER_PERSON"));
static OWNED_BY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWNED_BY"));
static OWNERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWNERS"));
static OWNING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWNING_WORDS"));
static OWNS_THE_UNIT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWNS_THE_UNIT"));
static OWN_SELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OWN_SELF"));
const PART_UP_BOOST: i64 = 4;
static PEOPLE_TOPICS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "PEOPLE_TOPICS"));  // t, 3 items
static PERSON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PERSON_WORDS"));
static PETS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PETS"));
const PET_BOOST: i64 = 3;
static PHRASES: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| t::map_strs(M, "PHRASES"));
static PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACES"));
static PLACE_HOMOGRAPHS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_HOMOGRAPHS"));
const PLACE_OVER: i64 = 2;
static PLACE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_THING"));
static PLACE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_WORDS"));
static PLACE_WORDS_WITH_CUE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_WORDS_WITH_CUE"));
static PLAYS_UP_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLAYS_UP_BEFORE"));
const PLAY_UP_REACH: i64 = 3;
static POLY_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POLY_FORMS"));
static PORTION_UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PORTION_UNITS"));
static PORTION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PORTION_WORDS"));
static RELATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RELATIONS"));
static RUN_OUT_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "RUN_OUT_BEFORE"));
static SAMPLE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SAMPLE_WORDS"));
const SCIENCE_BOOST: i64 = 2;
static SEASONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SEASONS"));
const SENSE_WEIGHT: i64 = 6;
static SIGN_FOLLOWS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SIGN_FOLLOWS"));
static SIGN_NOT_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SIGN_NOT_AFTER"));
const SKIN_BOOST: i64 = 4;
static SKIN_PARTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SKIN_PARTS"));
const SKIN_REACH: i64 = 3;
static SKIN_STATES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SKIN_STATES"));
static STONE_COMPOUNDS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "STONE_COMPOUNDS"));  // t, 3 items
static STONE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STONE_VERBS"));
static STOOL_AS_FURNITURE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STOOL_AS_FURNITURE"));
static STRONG: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "STRONG"));
static STUDY_OBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STUDY_OBJECTS"));
static SUMS_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUMS_AFTER"));
static SUM_MONEY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SUM_MONEY"));
static TEETHING_AND_A_CHILD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TEETHING_AND_A_CHILD"));
static TEETHING_IS_A_FAULT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TEETHING_IS_A_FAULT"));
static THE_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THE_PEOPLE"));
static THE_PERSONS_OWN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THE_PERSONS_OWN"));
static THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "THINGS"));
static TRAVEL_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TRAVEL_CUES"));
static TRAVEL_STRONG: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TRAVEL_STRONG"));
static TWO_HOMES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TWO_HOMES"));
const TWO_LIVES_REACH: i64 = 99;
static UNIT_WORDS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "UNIT_WORDS"));
const UNWELL_BOOST: i64 = 3;
static UNWELL_IDIOMS: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "UNWELL_IDIOMS"));  // t, 8 items
static UNWELL_NOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "UNWELL_NOT"));
static UNWELL_SIGNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "UNWELL_SIGNS"));
static WANTED_BEFORE_A_STOP: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WANTED_BEFORE_A_STOP"));
static WANT_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WANT_VERBS"));
static WARMTH_ARE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WARMTH_ARE"));
static WARMTH_DEGREE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WARMTH_DEGREE"));
static WARMTH_HABITUAL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WARMTH_HABITUAL"));
static WARMTH_THEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WARMTH_THEN"));
static WARMTH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WARMTH_WORDS"));
static WATCHING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WATCHING"));
static WATCH_OUT_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WATCH_OUT_AFTER"));
static WEAK: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "WEAK"));
static WEATHER_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WEATHER_BEFORE"));
static WEATHER_ITSELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WEATHER_ITSELF"));
static WORD_DIGITS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WORD_DIGITS"));
static WORK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WORK_WORDS"));
static WORLD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WORLD"));
static WORLD_NOT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WORLD_NOT"));
static WORN_OUT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WORN_OUT"));
const WORN_OUT_BOOST: i64 = 4;
static WORN_OUT_CUE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WORN_OUT_CUE"));
const WORN_OUT_LOOKBACK: i64 = 3;
static WORN_OUT_SAID: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "WORN_OUT_SAID"));  // t, 5 items
static _A_STATE_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_STATE_VERB"));
static _BREAKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_BREAKS"));
static _NOT_THE_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_NOT_THE_SUBJECT"));
static _POSSESSIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_POSSESSIVE"));
static _THE_OWNER_SETTLES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_THE_OWNER_SETTLES"));

// ---- tables that are not plain lists ----
type Tiers = Vec<(i64, HashSet<String>)>;
fn tiers_of(v: &crate::data::Value) -> Tiers {
    v.items().iter().map(|t| (t.items()[0].as_int().unwrap_or(0), t.items()[1].str_set())).collect()
}
/// `TABLE`: word or phrase -> [(topic, weight)], in Python's order.
static TABLE: LazyLock<IndexMap<String, Vec<(String, i64)>>> = LazyLock::new(|| {
    t::value(M, "TABLE").pairs().iter().map(|(k, v)| (k.str().to_string(), v.items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_int().unwrap_or(0))).collect())).collect()
});
/// The phrases of TABLE (keys with a space), in TABLE's order.
static TABLE_PHRASES: LazyLock<Vec<(&'static String, &'static Vec<(String, i64)>)>> = LazyLock::new(|| TABLE.iter().filter(|(k, _)| k.contains(' ')).collect());
/// The phrases of TABLE that say the weather, without a season word in them... (for
/// `_the_weather_is_just_a_season`: the keys whose entries name nature-and-weather).
static WEATHER_PHRASES: LazyLock<Vec<&'static String>> = LazyLock::new(|| {
    TABLE_PHRASES.iter().filter(|(_, e)| e.iter().any(|(topic, _)| topic == "nature-and-weather")).map(|(k, _)| *k).collect()
});
/// `SENSE_PAIRS`: name -> (forms, the sense's topic, tiers).
static SENSE_PAIRS: LazyLock<Vec<(Vec<String>, String, Tiers)>> = LazyLock::new(|| {
    t::value(M, "SENSE_PAIRS").pairs().iter().map(|(_, v)| (v.items()[0].strs(), v.items()[1].str().to_string(), tiers_of(&v.items()[2]))).collect()
});
static FORMS_OF: LazyLock<HashSet<String>> = LazyLock::new(|| SENSE_PAIRS.iter().flat_map(|(f, _, _)| f.iter().cloned()).collect());
static SENSE_ELSEWHERE_FORMS: LazyLock<HashMap<String, Tiers>> =
    LazyLock::new(|| t::value(M, "SENSE_ELSEWHERE_FORMS").pairs().iter().map(|(k, v)| (k.str().to_string(), tiers_of(v))).collect());
static _SENSE_TIERS: LazyLock<HashMap<String, Vec<(String, Tiers)>>> = LazyLock::new(|| {
    t::value(M, "_SENSE_TIERS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.items().iter().map(|p| (p.items()[0].str().to_string(), tiers_of(&p.items()[1]))).collect())).collect()
});
/// `TWO_LIVES_FORMS`: word -> [(cues, the topics of that sense)].
static TWO_LIVES_FORMS: LazyLock<HashMap<String, Vec<(HashSet<String>, HashSet<String>)>>> = LazyLock::new(|| {
    t::value(M, "TWO_LIVES_FORMS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.items().iter().map(|p| (p.items()[0].str_set(), p.items()[1].str_set())).collect())).collect()
});
static LIVES: LazyLock<HashMap<String, Vec<HashSet<String>>>> =
    LazyLock::new(|| t::value(M, "LIVES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.items().iter().map(|s| s.str_set()).collect())).collect());
/// `WORD_NUMBERS`: number words and their values (ints and floats kept apart).
static WORD_NUMBERS: LazyLock<HashMap<String, Num>> = LazyLock::new(|| {
    t::value(M, "WORD_NUMBERS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), match v { crate::data::Value::Int(i) => Num::Int(*i), other => Num::Float(other.as_f64().unwrap_or(0.0)) }))
        .collect()
});
static AGE_IN_A_YEAR: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| t::map_int(M, "AGE_IN_A_YEAR"));
static CONDITION_NOT_NEXT: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| t::map_set(M, "CONDITION_NOT_NEXT"));
static CONDITION_OTHER_LIFE: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| t::map_set(M, "CONDITION_OTHER_LIFE"));
static CONDITION_NEEDS_WORD: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| t::map_set(M, "CONDITION_NEEDS_WORD"));
static VERB_CUES: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| t::map_set(M, "VERB_CUES"));
static MACHINE_WORD_CUES: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| t::map_set(M, "MACHINE_WORD_CUES"));
static LX_CHAT: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map("bot.understanding.lexicon.", "CHAT"));
static LX_CORE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.lexicon.", "CORE_WORDS"));
/// `frozenset(STOPWORDS)` of the understanding package (`_function_words`).
static _FUNCTION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.", "STOPWORDS"));

// ---- helpers ----
fn has(set: &HashSet<String>, w: Option<&str>) -> bool {
    w.is_some_and(|w| set.contains(w))
}
fn sv(norm: &[String]) -> HashSet<&str> {
    norm.iter().map(String::as_str).collect()
}
fn any_in<'a>(words: impl IntoIterator<Item = &'a str>, set: &HashSet<String>) -> bool {
    words.into_iter().any(|w| set.contains(w))
}
fn as_f64(n: &Num) -> f64 {
    n.as_f64()
}

/// `_work_out_sums(norm)`: "work out 3 times 4" is a sum, not exercise.
fn _work_out_sums(norm: &[String]) -> BTreeSet<usize> {
    let mut hits = BTreeSet::new();
    let training = norm.iter().any(|w| EXERCISE_NOUNS.contains(w));
    for (index, word) in norm.iter().enumerate() {
        if !matches!(word.as_str(), "work" | "works" | "working" | "worked") || index + 1 >= norm.len() {
            continue;
        }
        if norm[index + 1] != "out" {
            continue;
        }
        let after = &norm[(index + 2).min(norm.len())..(index + 8).min(norm.len())];
        if after.len() >= 3 && NUMBER_WORDS.contains(&after[0]) && after[1] == "times" && NUMBER_WORDS.contains(&after[2]) {
            hits.insert(index);
            continue;
        }
        if training || after.iter().take(3).any(|w| EXERCISE_AFTER.contains(w)) {
            continue;
        }
        if after.iter().any(|w| SUMS_AFTER.contains(w) || NUMBER_WORDS.contains(w)) || after.iter().any(|w| pyuni::is_digit(w)) {
            hits.insert(index);
        }
    }
    hits
}

/// `_head_fog(norm)`: "my head feels foggy".
fn _head_fog(norm: &[String]) -> bool {
    let foggy: Vec<usize> = norm.iter().enumerate().filter(|(_, w)| HEAD_FOG.contains(*w)).map(|(k, _)| k).collect();
    if foggy.is_empty() {
        return false;
    }
    for (at, word) in norm.iter().enumerate() {
        if HEAD_FOG.contains(word) || !HEAD_WORDS.contains(word) {
            continue;
        }
        if foggy.iter().any(|&o| o > at && (o - at) as i64 <= HEAD_FOG_REACH) {
            return true;
        }
    }
    false
}

/// `_skin_state(norm, skip)`: a skin state beside a part of the skin.
fn _skin_state(norm: &[String], skip: &BTreeSet<usize>) -> Option<usize> {
    let states: Vec<usize> = norm.iter().enumerate().filter(|(k, w)| SKIN_STATES.contains(*w) && !skip.contains(k)).map(|(k, _)| k).collect();
    if states.is_empty() {
        return None;
    }
    for (at, word) in norm.iter().enumerate() {
        if !SKIN_PARTS.contains(word) || skip.contains(&at) {
            continue;
        }
        if states.iter().any(|&s| (s as i64 - at as i64).abs() <= SKIN_REACH) {
            return Some(at);
        }
    }
    None
}

/// `_a_part_plays_up(norm, skip)`: "my knee is playing up".
fn _a_part_plays_up(norm: &[String], skip: &BTreeSet<usize>) -> Option<usize> {
    let ups: Vec<usize> = (0..norm.len())
        .filter(|&at| {
            norm[at] == "up"
                && at > 0
                && !skip.contains(&at)
                && (PLAYS_UP_BEFORE.contains(&norm[at - 1]) || (at >= 2 && matches!(norm[at - 1].as_str(), "me" | "my") && PLAYS_UP_BEFORE.contains(&norm[at - 2])))
        })
        .collect();
    if ups.is_empty() {
        return None;
    }
    for (at, word) in norm.iter().enumerate() {
        if !BODY.contains(word) || skip.contains(&at) {
            continue;
        }
        if ups.iter().any(|&u| u > at && (u - at) as i64 <= PLAY_UP_REACH) {
            return Some(at);
        }
    }
    None
}

/// `_named_condition(norm, words, skip)`: a named health condition ("thrush", "shingles").
fn _named_condition(norm: &[String], words: &HashSet<&str>, skip: &BTreeSet<usize>) -> bool {
    if words.iter().any(|w| PETS.contains(*w)) && !words.iter().any(|w| _person_words().contains(*w) && !PETS.contains(*w)) {
        return false;
    }
    for (at, word) in norm.iter().enumerate() {
        if !CONDITIONS.contains(word) || skip.contains(&at) {
            continue;
        }
        if let Some(nexts) = CONDITION_NOT_NEXT.get(word) {
            if !nexts.is_empty() && at + 1 < norm.len() && nexts.contains(&norm[at + 1]) {
                continue;
            }
        }
        if let Some(other) = CONDITION_OTHER_LIFE.get(word) {
            if !other.is_empty() && words.iter().any(|w| other.contains(*w)) {
                continue;
            }
        }
        let near = |pred: &dyn Fn(&str) -> bool| norm.iter().enumerate().any(|(o, w)| !skip.contains(&o) && (o as i64 - at as i64).abs() <= CONDITION_REACH && pred(w));
        if CONDITION_NEEDS_PART.contains(word) && !near(&|w| BODY.contains(w)) {
            continue;
        }
        if let Some(wanted) = CONDITION_NEEDS_WORD.get(word) {
            if !wanted.is_empty() && !near(&|w| wanted.contains(w)) {
                continue;
            }
        }
        return true;
    }
    false
}

/// `_person_words()`: every word for a person (relations, extra people, pets, and the strong
/// family and friends words).
fn _person_words() -> &'static HashSet<String> {
    static WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
        let mut words: HashSet<String> = RELATIONS.iter().cloned().collect();
        words.extend(EXTRA_PEOPLE.iter().cloned());
        words.extend(PETS.iter().cloned());
        for topic in ["family-and-parenting", "friends-and-relationships"] {
            words.extend(text::split(STRONG.get(topic).map(String::as_str).unwrap_or_else(|| panic!("KeyError: {topic}"))));
        }
        words
    });
    &WORDS
}

/// `reads_as_unfit(text)`: "i'm not cut out for this" -> where it is said.
pub fn reads_as_unfit(text_: &str) -> Option<usize> {
    for m in NOT_CUT_OUT.finditer(text_) {
        let all = text::split_ref(&text_[..m.start()]);
        let before = &all[all.len().saturating_sub(6)..];
        if !before.iter().any(|w| NOT_WORDS.contains(*w)) {
            continue;
        }
        let person: Vec<&&str> = before.iter().filter(|w| OWN_SELF.contains(**w) || OTHER_PERSON.contains(**w)).collect();
        if person.last().is_none_or(|p| !OWN_SELF.contains(**p)) {
            continue;
        }
        return Some(m.start());
    }
    None
}

/// `_a_habit_given_up(norm)`: "i've given up sugar".
fn _a_habit_given_up(norm: &[String]) -> bool {
    let joined = norm.join(" ");
    if !norm.iter().any(|w| GIVEN_UP_FOOD.contains(w)) {
        return false;
    }
    GIVING_UP.iter().any(|p| joined.contains(p.as_str()))
}

/// `the_bin_is_on_a_machine(understood)`.
pub fn the_bin_is_on_a_machine(understood: &crate::understood::Understood) -> bool {
    if understood.words.iter().any(|w| MACHINE_BIN_WORDS.contains(w)) {
        return true;
    }
    understood.topics.iter().any(|t| t == "technology")
}

/// `unclear_subject(norm, inherited=None, previous=None)`: a word with two lives ("bank") said
/// with nothing to tell which.
pub fn unclear_subject(norm: &[String], inherited: Option<&str>, previous: Option<&Line>) -> Option<String> {
    let content: Vec<usize> = norm.iter().enumerate().filter(|(_, w)| !_NOT_THE_SUBJECT.contains(*w)).map(|(k, _)| k).collect();
    let &first = content.first()?;
    let word = &norm[first];
    let tiers = TWO_LIVES_FORMS.get(word).filter(|t| !t.is_empty())?;
    if _THE_OWNER_SETTLES.contains(word) && first > 0 && _POSSESSIVE.contains(&norm[first - 1]) {
        return None;
    }
    if let Some(inh) = inherited.filter(|i| !i.is_empty()) {
        if tiers.iter().any(|(_, topics)| topics.contains(inh)) {
            return None;
        }
    }
    if let Some(prev) = previous {
        let before: HashSet<&str> = sv(&prev.norm);
        if tiers.iter().any(|(cues, _)| before.iter().any(|w| cues.contains(*w))) {
            return None;
        }
    }
    let after: Vec<&String> = content.iter().skip(1).take(8).map(|&k| &norm[k]).collect();
    if !after.iter().any(|w| _A_STATE_VERB.contains(*w)) {
        return None;
    }
    if tiers.iter().any(|(cues, _)| _sense_cue(norm, &[first], &[(TWO_LIVES_REACH, cues.clone())])) {
        return None;
    }
    Some(word.clone())
}

/// `_teething_is_a_fault(norm)`.
fn _teething_is_a_fault(norm: &[String]) -> bool {
    norm.iter().any(|w| TEETHING_IS_A_FAULT.contains(w)) && !norm.iter().any(|w| TEETHING_AND_A_CHILD.contains(w))
}

/// `the_warmth_said(norm)`: "she is always cold" -> the warmth word said of a person.
pub fn the_warmth_said(norm: &[String]) -> Option<String> {
    for (at, word) in norm.iter().enumerate() {
        if !WARMTH_WORDS.contains(word) {
            continue;
        }
        let mut back = at as isize - 1;
        let mut habit = false;
        while back >= 0 && WARMTH_DEGREE.contains(&norm[back as usize]) {
            if WARMTH_HABITUAL.contains(&norm[back as usize]) {
                habit = true;
            }
            back -= 1;
        }
        if back < 1 || !WARMTH_ARE.contains(&norm[back as usize]) {
            continue;
        }
        if habit {
            continue;
        }
        let who = &norm[back as usize - 1];
        let person = COLD_PEOPLE.contains(who) || PERSON_WORDS.contains(who) || (who.ends_with('s') && PERSON_WORDS.contains(&who[..who.len() - 1]));
        if !person {
            continue;
        }
        let rest: Vec<&String> = norm[at + 1..].iter().filter(|w| !WARMTH_THEN.contains(*w)).collect();
        if rest.first().is_some_and(|r| !matches!(r.as_str(), "." | "!" | "?" | ",")) {
            continue;
        }
        return Some(word.clone());
    }
    None
}

/// `_means_something_else(norm, position)`: the word is in another sense here.
fn _means_something_else(norm: &[String], position: usize) -> bool {
    if norm[position] == "teething" && _teething_is_a_fault(norm) {
        return true;
    }
    let Some(tiers) = SENSE_ELSEWHERE_FORMS.get(&norm[position]).filter(|t| !t.is_empty()) else { return false };
    if BODY_TWO_LIFE.contains(&norm[position]) && position > 0 && OWNING_WORDS.contains(&norm[position - 1]) {
        return true;
    }
    _sense_cue(norm, &[position], tiers)
}

/// `says_another_thing(text)`.
pub fn says_another_thing(text_: &str) -> HashSet<String> {
    let norm: Vec<String> = rx("[^a-z0-9]+").split(&text_.to_lowercase()).into_iter().filter(|w| !w.is_empty()).collect();
    names_another_thing(&norm).into_iter().map(|k| norm[k].clone()).collect()
}

/// `names_another_thing(norm)`.
pub fn names_another_thing(norm: &[String]) -> BTreeSet<usize> {
    (0..norm.len()).filter(|&k| SENSE_ELSEWHERE_FORMS.contains_key(&norm[k]) && _means_something_else(norm, k)).collect()
}

/// `is_a_machine_word(text)`: "mouse" said of a computer.
pub fn is_a_machine_word(text_: &str) -> bool {
    let said = text_.to_lowercase().replace('\'', "");
    let words: HashSet<String> = rx("[^a-z]+").split(&said).into_iter().collect();
    if !words.contains("mouse") && !words.contains("mice") && !words.contains("mousepad") {
        return false;
    }
    if words.contains("mice") {
        return false;
    }
    if MACHINE_WORD_PHRASES.iter().any(|p| said.contains(p.as_str())) {
        return false;
    }
    let cues = MACHINE_WORD_CUES.get("mouse").unwrap_or_else(|| panic!("KeyError: mouse"));
    !words.iter().any(|w| cues.contains(w))
}

/// `_the_sense_the_cues_name(norm, position, word)`.
fn _the_sense_the_cues_name(norm: &[String], position: usize, word: &str) -> bool {
    _SENSE_TIERS.get(word).is_some_and(|v| v.iter().any(|(_, tiers)| _sense_cue(norm, &[position], tiers)))
}

/// `_sense_cue(norm, positions, tiers)`: a cue word within reach of one of the positions.
fn _sense_cue(norm: &[String], positions: &[usize], tiers: &[(i64, HashSet<String>)]) -> bool {
    for (cue_at, word) in norm.iter().enumerate() {
        for (reach, cues) in tiers {
            if !cues.contains(word) {
                continue;
            }
            for &spot in positions {
                let (s, c) = (spot as i64, cue_at as i64);
                if s - reach <= c && c <= s + reach {
                    return true;
                }
            }
        }
    }
    false
}

/// `_takes_an_object(norm, position)`.
fn _takes_an_object(norm: &[String], position: usize) -> bool {
    position + 1 < norm.len() && DETERMINERS.contains(&norm[position + 1])
}

/// `_acts_on_a_thing(topic, words, norm, start)`.
fn _acts_on_a_thing(topic: &str, words: &[String], norm: &[String], start: usize) -> bool {
    for (offset, word) in words.iter().enumerate() {
        if VERB_CUES.get(topic).is_some_and(|c| c.contains(word)) {
            return _takes_an_object(norm, start + offset);
        }
    }
    false
}

/// `_thing_a_cue_acts_on(norm, position)`.
fn _thing_a_cue_acts_on(norm: &[String], position: usize) -> String {
    if !_takes_an_object(norm, position) {
        return String::new();
    }
    let mut index = position + 2;
    let mut noun = norm.get(index).cloned().unwrap_or_default();
    if DEGREE_NOUNS.contains(&noun) && !(index + 1 < norm.len() && norm[index + 1] == "of") {
        return String::new();
    }
    while index + 1 < norm.len() && norm[index + 1] == "of" {
        index += 2;
        if index < norm.len() && DETERMINERS.contains(&norm[index]) {
            index += 1;
        }
        if index < norm.len() {
            noun = norm[index].clone();
        }
    }
    noun
}

/// `_acts_on_the_body(norm, position)`.
fn _acts_on_the_body(norm: &[String], position: usize) -> bool {
    let thing = _thing_a_cue_acts_on(norm, position);
    if !thing.is_empty() {
        return BODY_OR_SELF.contains(&thing);
    }
    if let Some(after) = norm.get(position + 1) {
        if !BODY_OR_SELF.contains(after) && !CUE_CARRIERS.contains(after) {
            let others = norm.iter().enumerate().any(|(k, w)| {
                k != position && !POLY_FORMS.contains(w) && _lookup(w).iter().any(|(topic, weight)| *weight >= 2 && topic != "health" && topic != "small-talk")
            });
            return !others;
        }
    }
    true
}

/// `_is_run_out(norm, position)`.
fn _is_run_out(norm: &[String], position: usize) -> bool {
    RUN_OUT_BEFORE.contains(&norm[position]) && position + 1 < norm.len() && norm[position + 1] == "out"
}

/// `_watches_out_for(words, norm, end)`.
fn _watches_out_for(words: &[String], norm: &[String], end: usize) -> bool {
    words.last().is_some_and(|w| WATCHING.contains(w)) && end < norm.len() && WATCH_OUT_AFTER.contains(&norm[end])
}

/// `_says_nothing_here(topic, words, norm, start, study)`: the topic word does not say the
/// topic here ("watch out for", "health visitor", "treat the cough" vs "treat yourself").
fn _says_nothing_here(topic: &str, words: &[String], norm: &[String], start: usize, study: bool) -> bool {
    if _watches_out_for(words, norm, start + words.len()) {
        return true;
    }
    if topic == "family-and-parenting" && words.join(" ").contains("health visitor") {
        let rest = norm.join(" ").replace("health visitor", " ").replace("health visitors", " ");
        for word in text::split_ref(&rest) {
            if _lookup(word).iter().any(|(t, w)| t == "health" && *w >= 2) {
                return true;
            }
        }
    }
    if words.len() == 1 && BODY_CUES.contains(&words[0]) {
        return !_acts_on_the_body(norm, start);
    }
    if words.iter().any(|w| VERB_CUES.get(topic).is_some_and(|c| c.contains(w))) {
        return _acts_on_a_thing(topic, words, norm, start) && !study;
    }
    false
}

/// `_age_value(word)`.
fn _age_value(word: &str) -> f64 {
    if let Some(v) = WORD_NUMBERS.get(word) {
        return as_f64(v);
    }
    match WORD_DIGITS.match_(word) {
        Some(d) => d.as_str().replace(',', ".").parse().unwrap_or(99.0),
        None => 99.0,
    }
}

/// `_ages_of_a_child(norm)`: "my 5 year old" -> the position of the figure.
fn _ages_of_a_child(norm: &[String]) -> Vec<usize> {
    let mut ages = Vec::new();
    for (position, word) in norm.iter().enumerate() {
        if !AGE_NOUNS.contains(word) || position == 0 || position + 1 >= norm.len() {
            continue;
        }
        if norm[position + 1] != "old" {
            continue;
        }
        let number_at = position - 1;
        if AGE_AFTER.contains(&norm[number_at]) {
            continue;
        }
        if number_at == 0 || !DETERMINERS.contains(&norm[number_at - 1]) {
            continue;
        }
        let per_year = *AGE_IN_A_YEAR.get(word).unwrap_or_else(|| panic!("KeyError: {word}")) as f64;
        if _age_value(&norm[number_at]) / per_year > A_CHILD_TILL as f64 {
            continue;
        }
        ages.push(number_at);
    }
    ages
}

/// `_a_book_for_a_child(norm, child_ages)`.
fn _a_book_for_a_child(norm: &[String], child_ages: &[usize]) -> Option<usize> {
    for (position, word) in norm.iter().enumerate() {
        if !BOOK_WORDS.contains(word) || norm.get(position + 1).map(String::as_str) != Some("for") {
            continue;
        }
        let after = position + 2;
        let tail = &norm[after.min(norm.len())..(after + 5).min(norm.len())];
        if tail.iter().any(|w| A_CHILD_TO_READ_FOR.contains(w)) {
            return Some(position);
        }
        if child_ages.iter().any(|&age| after <= age && age <= after + 3) {
            return Some(position);
        }
        if tail.iter().any(|w| w == "year") && tail.iter().any(|w| w == "old" || w == "olds") {
            return Some(position);
        }
    }
    None
}

/// `_drives_a_person(norm, position)`: "it drives me mad".
fn _drives_a_person(norm: &[String], position: usize) -> bool {
    if !DRIVES.contains(&norm[position]) {
        return false;
    }
    let Some(next) = norm.get(position + 1) else { return false };
    if !DRIVEN_PERSON.contains(next) {
        return false;
    }
    let tail: HashSet<&str> = norm[(position + 2).min(norm.len())..(position + 7).min(norm.len())].iter().map(String::as_str).collect();
    any_in(tail.iter().copied(), &DRIVEN_MOOD) || (any_in(tail.iter().copied(), &DRIVEN_TAIL) && any_in(tail.iter().copied(), &DRIVEN_JOINERS))
}

/// `_idiom_positions(norm)`: the words of idioms ("run off my feet").
fn _idiom_positions(norm: &[String]) -> BTreeSet<usize> {
    let mut spots = BTreeSet::new();
    let joined = format!(" {} ", norm.join(" "));
    for phrase in IDIOM_PHRASES.iter() {
        let needle = format!(" {} ", phrase.join(" "));
        let mut start = 0;
        while let Some(found) = joined[start..].find(&needle) {
            let at = start + found;
            let before = text::split_ref(&joined[..at]).len();
            spots.extend(before..before + phrase.len());
            start = at + 1;
        }
    }
    spots
}

/// `_bad_verdict_positions(norm)`.
fn _bad_verdict_positions(norm: &[String]) -> BTreeSet<usize> {
    (0..norm.len()).filter(|&k| a_bad_verdict(norm, k)).collect()
}

/// `_drive_idiom(norm)`.
fn _drive_idiom(norm: &[String]) -> BTreeSet<usize> {
    let mut idiom = BTreeSet::new();
    for position in 0..norm.len() {
        if !_drives_a_person(norm, position) {
            continue;
        }
        idiom.insert(position);
        idiom.insert(position + 1);
        for offset in position + 2..(position + 7).min(norm.len()) {
            if DRIVEN_MOOD.contains(&norm[offset]) || DRIVEN_TAIL.contains(&norm[offset]) || DRIVEN_JOINERS.contains(&norm[offset]) {
                idiom.insert(offset);
            }
        }
    }
    idiom
}

/// `_is_a_complaint(norm, position)`.
fn _is_a_complaint(norm: &[String], position: usize) -> bool {
    COMPLAINT_OF.contains(&norm[position]) && position + 1 < norm.len() && COMPLAINT_OF_NEXT.contains(&norm[position + 1])
}

/// `TABLE.get(key)` when it is a non-empty list.
fn table(key: &str) -> Option<&'static [(String, i64)]> {
    TABLE.get(key).map(Vec::as_slice).filter(|v| !v.is_empty())
}

/// `_family(word)`: the topics of a word by its derived family ("gardening" -> "garden").
fn _family(word: &str) -> Option<&'static [(String, i64)]> {
    for ending in DERIVED_ENDINGS.iter() {
        if word.ends_with(ending.as_str()) {
            let stem = &word[..word.len() - ending.len()];
            if text::len(stem) as i64 >= FAMILY_STEM {
                if let Some(hit) = table(stem).or_else(|| table(&format!("{stem}e"))) {
                    return Some(hit);
                }
            }
        }
        if text::len(word) as i64 >= FAMILY_WORD && !_FUNCTION_WORDS.contains(word) {
            if let Some(hit) = table(&format!("{word}{ending}")) {
                return Some(hit);
            }
        }
    }
    for pair in FAMILY_PAIRS.iter() {
        let (here, there) = (&pair[0], &pair[1]);
        if word.ends_with(here.as_str()) {
            if let Some(hit) = table(&format!("{}{}", &word[..word.len() - here.len()], there)) {
                return Some(hit);
            }
        }
        if word.ends_with(there.as_str()) {
            if let Some(hit) = table(&format!("{}{}", &word[..word.len() - there.len()], here)) {
                return Some(hit);
            }
        }
    }
    None
}

/// `_lookup(word)`: the topics a word names, with their weights.
pub fn _lookup(word: &str) -> Cow<'static, [(String, i64)]> {
    if let Some(hit) = table(word) {
        return Cow::Borrowed(hit);
    }
    for (suffix, replacement) in [("ies", "y"), ("es", ""), ("s", ""), ("ing", ""), ("ing", "e"), ("ed", ""), ("ed", "e")] {
        if word.ends_with(suffix) && text::len(word) > suffix.len() + 2 {
            if let Some(hit) = table(&format!("{}{}", &word[..word.len() - suffix.len()], replacement)) {
                return Cow::Borrowed(hit);
            }
        }
    }
    match _family(word) {
        Some(family) => Cow::Owned(family.iter().filter(|(t, _)| t != "about-the-bot").cloned().collect()),
        None => Cow::Borrowed(&[]),
    }
}

/// `_is_a_portion(norm, position)`: "a slice of", "a bit of".
fn _is_a_portion(norm: &[String], position: usize) -> bool {
    if !PORTION_WORDS.contains(&norm[position]) {
        return false;
    }
    let after = &norm[(position + 1).min(norm.len())..(position + 3).min(norm.len())];
    if after.is_empty() {
        return false;
    }
    if matches!(after[0].as_str(), "a" | "an" | "the") {
        if after.len() > 1 && PORTION_UNITS.contains(&after[1]) {
            return false;
        }
        return true;
    }
    after[0] == "of" && after.len() > 1 && matches!(after[1].as_str(), "a" | "an" | "the")
}

/// `_the_weather_is_just_a_season(norm, text)`: "in the summer" sets the time, not the weather.
fn _the_weather_is_just_a_season(norm: &[String], text_: &str) -> bool {
    let spots: Vec<usize> = norm.iter().enumerate().filter(|(_, w)| SEASONS.contains(*w)).map(|(k, _)| k).collect();
    let Some(&lowest) = spots.iter().min() else { return false };
    if lowest <= 2 {
        return false;
    }
    if spots.iter().any(|&at| at > 0 && WEATHER_BEFORE.contains(&norm[at - 1])) {
        return false;
    }
    if norm.iter().any(|w| WEATHER_ITSELF.contains(w)) {
        return false;
    }
    if norm.iter().any(|w| !SEASONS.contains(w) && _lookup(w).iter().any(|(t, weight)| t == "nature-and-weather" && *weight >= 2)) {
        return false;
    }
    !WEATHER_PHRASES.iter().any(|key| text_.contains(key.as_str()) && !text::split_ref(key).iter().any(|w| SEASONS.contains(*w)))
}

/// `score_topics(line, inherited=None)`: `[(topic, score, top)]`, best first.
pub fn score_topics(line: &Line) -> Vec<(String, i64, i64)> {
    let norm = &line.norm;
    let text_ = format!(" {} ", line.norm_text);
    let sums = _work_out_sums(norm);
    let study = norm.iter().any(|w| STUDY_OBJECTS.contains(w));
    let background = background_positions(norm);
    let mut found: IndexMap<String, (i64, i64, usize)> = IndexMap::new();
    let mut own: HashSet<String> = HashSet::new();
    let mut note = |found: &mut IndexMap<String, (i64, i64, usize)>, topic: &str, weight: i64, position: usize, polysemous: bool| {
        let (score, top, first) = found.get(topic).copied().unwrap_or((0, 0, position));
        found.insert(topic.to_string(), (score + weight, top.max(weight), first));
        if !polysemous {
            own.insert(topic.to_string());
        }
    };
    let mut idiom = _drive_idiom(norm);
    idiom.extend(_idiom_positions(norm));
    idiom.extend(_bad_verdict_positions(norm));
    let child_ages = _ages_of_a_child(norm);
    let warmth = the_warmth_said(norm).is_some();
    for (position, word) in norm.iter().enumerate() {
        if sums.contains(&position) || background.contains(&position) || idiom.contains(&position) {
            continue;
        }
        if _is_run_out(norm, position) || _is_a_complaint(norm, position) {
            continue;
        }
        if PLACES.contains(word) && _names_a_person(norm, position) {
            continue;
        }
        let mut elsewhere = _means_something_else(norm, position);
        if WARMTH_WORDS.contains(word) && warmth {
            elsewhere = true;
        }
        if FORMS_OF.contains(word) && _the_sense_the_cues_name(norm, position, word) {
            continue;
        }
        for (topic, weight) in _lookup(word).iter() {
            if elsewhere {
                continue;
            }
            if _says_nothing_here(topic, std::slice::from_ref(word), norm, position, study) {
                continue;
            }
            let poly = POLY_FORMS.contains(word) || _is_a_portion(norm, position);
            note(&mut found, topic, *weight, position, poly);
        }
    }
    for (forms, sense_topic, tiers) in SENSE_PAIRS.iter() {
        let spots: Vec<usize> = norm.iter().enumerate().filter(|(_, w)| FORMS_OF.contains(*w) && forms.contains(w)).map(|(k, _)| k).collect();
        if !spots.is_empty() && _sense_cue(norm, &spots, tiers) {
            note(&mut found, sense_topic, SENSE_WEIGHT, spots[0], false);
        }
    }
    let family_score = found.get("family-and-parenting").map(|e| e.0).unwrap_or(0);
    if !child_ages.is_empty() && (family_score >= 2 || !found.iter().any(|(t, (s, _, _))| *s >= 2 && t != "family-and-parenting" && t != "feelings")) {
        note(&mut found, "family-and-parenting", 2, child_ages[0], false);
    }
    if let Some(book_at) = _a_book_for_a_child(norm, &child_ages) {
        note(&mut found, "entertainment", 3, book_at, false);
    }
    if !sums.is_empty() && !found.values().any(|(s, _, _)| *s >= 2) {
        note(&mut found, "maths-and-numbers", 2, *sums.iter().next().unwrap(), false);
    }
    if let Some(output_at) = _the_persons_own_output(norm) {
        note(&mut found, "health", 2, output_at, false);
    }
    if let Some(stone_at) = stone_weight_at(norm) {
        note(&mut found, "fitness", 3, stone_at, false);
    }
    if let Some(tight_at) = tight_money_at(norm, &line.text) {
        if !found.iter().any(|(t, (s, _, _))| *s >= 2 && t != "money") {
            note(&mut found, "money", 3, tight_at, false);
        }
    }
    if !sums.is_empty() && norm.iter().any(|w| SUM_MONEY.contains(w)) {
        note(&mut found, "money", 3, *sums.iter().next().unwrap(), false);
    }
    let named = norm.iter().any(|w| !ADDRESS_WORDS.contains(w) && _lookup(w).iter().any(|(t, weight)| *weight >= 2 && t != "small-talk"));
    for (key, entries) in TABLE_PHRASES.iter() {
        if !text_.contains(key.as_str()) {
            continue;
        }
        let words = text::split(key);
        if words.len() > norm.len() {
            continue;
        }
        for start in 0..=(norm.len() - words.len()) {
            if norm[start..start + words.len()] != words[..] {
                continue;
            }
            if background.contains(&start) {
                continue;
            }
            if !sums.is_empty() && key.as_str() == "work out" {
                continue;
            }
            if named && entries.iter().all(|(t, _)| t == "small-talk") {
                continue;
            }
            for (topic, weight) in entries.iter() {
                if _says_nothing_here(topic, &words, norm, start, study) {
                    continue;
                }
                note(&mut found, topic, *weight, start, false);
            }
            break;
        }
    }
    let text_norm = format!(" {} ", norm.join(" "));
    if let Some(m) = JUDGEMENT_OF_RECORD.search(&text_norm) {
        if !found.values().any(|(s, _, _)| *s >= 3) {
            let at = text_norm[..m.start()].matches(' ').count();
            note(&mut found, "feelings", 3, at, false);
        }
    }
    if let Some(unfit) = reads_as_unfit(&text_norm) {
        if !found.values().any(|(s, _, _)| *s >= 3) {
            let at = text_norm[..unfit].matches(' ').count();
            note(&mut found, "feelings", 3, at, false);
        }
    }
    if _the_weather_is_just_a_season(norm, &text_) && found.iter().any(|(t, (s, _, _))| t != "nature-and-weather" && *s >= 2) {
        own.remove("nature-and-weather");
    }
    for (pattern, topic) in CANCELS.iter() {
        if !found.contains_key(topic) {
            continue;
        }
        if !found.iter().any(|(t, (s, _, _))| t != topic && *s >= 2) {
            continue;
        }
        let Some(m) = pattern.search(&text_norm) else { continue };
        let after = text::split(text::strip(&text_norm[m.end()..]));
        if let Some(first) = after.first() {
            if !CANCEL_AFTER.contains(first) && !matches!(first.as_str(), "." | "," | ";" | ":") {
                continue;
            }
        }
        let e = found.get_mut(topic).unwrap();
        e.0 = (e.0 - 2).max(0);
    }
    let mut ranked: Vec<(String, (i64, i64, usize))> = found.into_iter().collect();
    ranked.sort_by(|a, b| (b.1 .0, b.1 .1, -(b.1 .2 as i64)).cmp(&(a.1 .0, a.1 .1, -(a.1 .2 as i64))));
    ranked
        .into_iter()
        .map(|(topic, (score, top, _))| {
            let mine = own.contains(&topic);
            let s = if mine { score } else { score.min(1) };
            let t = if mine { top } else { top.min(1) };
            (topic, s, t)
        })
        .collect()
}

/// `unsure_words(norm, topic)`: words with two homes the line does not settle.
pub fn unsure_words(norm: &[String], topic: Option<&str>) -> Vec<String> {
    if topic.is_none_or(str::is_empty) {
        return vec![];
    }
    let words = sv(norm);
    let mut in_doubt: Vec<String> = Vec::new();
    for word in &words {
        if !TWO_HOMES.contains(*word) {
            continue;
        }
        if LIVES.get(*word).is_some_and(|homes| homes.iter().any(|home| words.iter().any(|w| home.contains(*w)))) {
            continue;
        }
        in_doubt.push(word.to_string());
    }
    in_doubt.sort();
    in_doubt
}

/// `_place_points(line, entities, scores, asks=None)`.
fn _place_points(line: &Line, entities: &[Entity], scores: &IndexMap<String, i64>, asks: Option<bool>) -> i64 {
    if line.norm.iter().any(|w| TRAVEL_STRONG.contains(w)) {
        return 4;
    }
    if entities.iter().any(|e| e.kind == "person") {
        return 0;
    }
    let others: Vec<i64> = scores.iter().filter(|(t, _)| *t != "geography-and-travel" && *t != "small-talk").map(|(_, s)| *s).collect();
    if others.iter().max().is_some_and(|&m| m >= PLACE_OVER) {
        return 0;
    }
    let asked = asks.unwrap_or_else(|| _looks_like_an_ask(line));
    if asked { 3 } else { 0 }
}

/// `_what_someone_else_wants(norm)`: "my son wants a bike" -> (shopping, where).
fn _what_someone_else_wants(norm: &[String]) -> Option<(String, usize)> {
    for (at, word) in norm.iter().enumerate() {
        if !WANT_VERBS.contains(word) {
            continue;
        }
        if !norm[at.saturating_sub(3)..at].iter().any(|w| A_PERSON_WHO_WANTS.contains(w)) {
            continue;
        }
        for spot in at + 1..norm.len().min(at + 5) {
            if WANTED_BEFORE_A_STOP.contains(&norm[spot]) {
                break;
            }
            if !A_THING_WANTED.contains(&norm[spot]) {
                continue;
            }
            for candidate in &norm[spot + 1..norm.len().min(spot + 5)] {
                if A_THING_WANTED.contains(candidate) {
                    continue;
                }
                let topics: Vec<String> = _lookup(candidate).iter().filter(|(t, w)| !PEOPLE_TOPICS.contains(t) && t != "small-talk" && *w >= 2).map(|(t, _)| t.clone()).collect();
                if let Some(first) = topics.into_iter().next() {
                    return Some((first, spot + 1));
                }
            }
            break;
        }
    }
    None
}

/// `_looks_like_an_ask(line)`.
fn _looks_like_an_ask(line: &Line) -> bool {
    if line.text.contains('?') || line.norm.is_empty() {
        return true;
    }
    ASK_WORDS.contains(&line.norm[0])
}

/// `_names_a_friend_of_their_own(norm)`: "my friend".
fn _names_a_friend_of_their_own(norm: &[String]) -> bool {
    norm.windows(2).any(|p| FRIEND_WORDS.contains(&p[1]) && OWNED_BY.contains(&p[0]))
}

/// `_the_persons_own_output(norm)`: the person's own bodily output, by position.
fn _the_persons_own_output(norm: &[String]) -> Option<usize> {
    let words = sv(norm);
    if !any_in(words.iter().copied(), &BODILY_OUTPUT) || any_in(words.iter().copied(), &SAMPLE_WORDS) {
        return None;
    }
    if any_in(words.iter().copied(), &ONLY_A_STOOL) && !words.iter().any(|w| BODILY_OUTPUT.contains(*w) && !ONLY_A_STOOL.contains(*w)) {
        if any_in(words.iter().copied(), &STOOL_AS_FURNITURE) {
            return None;
        }
        if !A_GUT_ONLY.is_match(&norm.join(" ")) {
            return None;
        }
    }
    if any_in(words.iter().copied(), &ANIMALS) || any_in(words.iter().copied(), &PETS) {
        return None;
    }
    let person = words.iter().any(|w| _person_words().contains(*w) && !PETS.contains(*w) && !ANIMALS.contains(*w));
    let speaker = words.iter().any(|w| matches!(*w, "i" | "im" | "ive" | "we" | "weve"));
    for (at, word) in norm.iter().enumerate() {
        if !BODILY_OUTPUT.contains(word) {
            continue;
        }
        if BODILY_OUTPUT_NOUN.contains(word) {
            if at > 0 && OWNERS.contains(&norm[at - 1]) {
                return Some(at);
            }
            continue;
        }
        if person || speaker {
            return Some(at);
        }
    }
    None
}

/// `own_output_word(norm)`: the position of the word, or None.
pub fn own_output_word(norm: &[String]) -> Option<usize> {
    _the_persons_own_output(norm)
}

/// `_owns_an_animal(norm)`: "my dog".
fn _owns_an_animal(norm: &[String]) -> bool {
    norm.windows(2).any(|p| OWNERS.contains(&p[0]) && ANIMALS.contains(&p[1]))
}

/// `_a_crop_is_failing(norm)`.
fn _a_crop_is_failing(norm: &[String]) -> bool {
    let words = sv(norm);
    if any_in(words.iter().copied(), &CROP_KITCHEN) {
        return false;
    }
    let joined = norm.join(" ");
    if !CROP_STATES.iter().any(|s| joined.contains(s.as_str())) {
        return false;
    }
    if any_in(words.iter().copied(), &GROWING_PLACES) {
        return true;
    }
    norm.windows(2).any(|p| OWNERS.contains(&p[0]) && CROPS.contains(&p[1]))
}

/// `_asks_mechanism(norm)`: "how does a rainbow form".
fn _asks_mechanism(norm: &[String]) -> bool {
    if norm.is_empty() || !MECHANISM.contains(&norm[0]) {
        return false;
    }
    if norm[0] == "how" {
        return norm.len() > 1 && MECHANISM_HOW.contains(&norm[1]);
    }
    true
}

/// `_sign_said_of_a_person(norm, words)`.
fn _sign_said_of_a_person(norm: &[String], words: &HashSet<&str>) -> bool {
    let joined = norm.join(" ");
    if UNWELL_IDIOMS.iter().any(|p| joined.contains(p.as_str())) {
        return false;
    }
    for (at, word) in norm.iter().enumerate() {
        let after = norm.get(at + 1).map(String::as_str).unwrap_or("");
        let before = if at > 0 { norm[at - 1].as_str() } else { "" };
        if UNWELL_SIGNS.contains(word) {
            if BEHAVIOUR.contains(after) || SIGN_NOT_AFTER.contains(after) {
                continue;
            }
            if NOUNY_SIGNS.contains(word) && !after.is_empty() && !SIGN_FOLLOWS.contains(after) && !pyuni::is_digit(after) {
                continue;
            }
            if word == "broke" && !any_in(words.iter().copied(), &BODY) {
                continue;
            }
            return true;
        }
        if before == "not" && UNWELL_NOT.contains(word) {
            if word != "responding" {
                return true;
            }
            for ahead in &norm[(at + 1).min(norm.len())..(at + 4).min(norm.len())] {
                if COMMS.contains(ahead) {
                    return false;
                }
                if HEARERS.contains(ahead) {
                    return true;
                }
            }
            return true;
        }
    }
    false
}

/// `_a_person_is_unwell(norm, words)`.
fn _a_person_is_unwell(norm: &[String], words: &HashSet<&str>) -> bool {
    if !any_in(words.iter().copied(), &UNWELL_SIGNS) && !(words.contains("not") && any_in(words.iter().copied(), &UNWELL_NOT)) {
        return false;
    }
    if !norm.iter().any(|w| THE_PEOPLE.contains(w)) {
        return false;
    }
    _sign_said_of_a_person(norm, words)
}

/// `broke_at(norm)`: "i'm broke" -> where.
pub fn broke_at(norm: &[String]) -> Option<usize> {
    for (at, word) in norm.iter().enumerate() {
        if word != "broke" {
            continue;
        }
        if norm[..at].iter().any(|b| !BROKE_AS_SUBJECT.contains(b)) {
            continue;
        }
        if norm[at + 1..].iter().any(|a| !BROKE_AS_TIME.contains(a)) {
            continue;
        }
        return Some(at);
    }
    None
}

/// `_purse_amount(norm, at)`.
fn _purse_amount(norm: &[String], at: usize) -> Option<f64> {
    let amount = &norm[at - 1];
    if pyuni::is_digit(&amount.replace('.', "")) {
        return Some(amount.parse::<f64>().unwrap_or_else(|_| panic!("ValueError: {amount}")));
    }
    let value = as_f64(WORD_NUMBERS.get(amount)?);
    if (amount == "hundred" || amount == "thousand") && at >= 2 {
        if let Some(before) = WORD_NUMBERS.get(&norm[at - 2]) {
            return Some(as_f64(before) * value);
        }
    }
    Some(value)
}

/// `_a_wait_follows(words)`: "... till friday".
fn _a_wait_follows(words: &[String]) -> bool {
    for (spot, word) in words.iter().take(3).enumerate() {
        if !A_WAIT.contains(word) {
            continue;
        }
        let rest: Vec<&String> = words[(spot + 1).min(words.len())..(spot + 3).min(words.len())].iter().filter(|o| !matches!(o.as_str(), "the" | "a" | "i")).collect();
        if rest.first().is_some_and(|r| A_WAIT_TO.contains(*r)) {
            return true;
        }
    }
    false
}

/// `purse_till_at(norm, text='')`: "£20 till payday" -> where.
pub fn purse_till_at(norm: &[String], text_: &str) -> Option<usize> {
    for (at, word) in norm.iter().enumerate() {
        if at == 0 {
            continue;
        }
        if A_AMOUNT_NOUN.contains_key(word) {
            if _a_wait_follows(&norm[at + 1..]) {
                return Some(at - 1);
            }
            continue;
        }
        if !A_MONEY_WORD.contains(word) {
            continue;
        }
        let amount = &norm[at - 1];
        if !NUMBER_WORDS.contains(amount) && !pyuni::is_digit(&amount.replace('.', "")) {
            continue;
        }
        if _purse_amount(norm, at).unwrap_or(0.0) >= BIG_PURSE as f64 {
            continue;
        }
        if _a_wait_follows(&norm[at + 1..]) {
            return Some(at - 1);
        }
    }
    if !text_.is_empty() {
        let low = text_.to_lowercase();
        for m in A_AMOUNT_SYMBOL.finditer(&low) {
            let value = match m.get(1).filter(|p| !p.is_empty()) {
                Some(pounds) => pounds.replace(',', "").parse::<f64>().unwrap_or_else(|_| panic!("ValueError")),
                None => m.str(2).parse::<f64>().unwrap_or_else(|_| panic!("ValueError")) / 100.0,
            };
            if value >= BIG_PURSE as f64 {
                continue;
            }
            // Python slices the line as typed at the offset found in its lower-cased copy
            let end_char = text::char_index(&low, m.end());
            let after: Vec<String> = rx("[a-z']+").findall_str(&text::slice(text_, end_char as isize, None).to_lowercase());
            if _a_wait_follows(&after) {
                return Some(0);
            }
        }
    }
    None
}

/// `tight_money_at(norm, text='')`: "money is tight" -> where it is said.
pub fn tight_money_at(norm: &[String], text_: &str) -> Option<usize> {
    let joined = format!(" {} ", norm.join(" "));
    let mut at = broke_at(norm);
    if let Some(purse) = purse_till_at(norm, text_) {
        at = Some(at.map_or(purse, |a| a.min(purse)));
    }
    for phrase in MONEY_TIGHT.iter() {
        let Some(spot) = joined.find(&format!(" {phrase} ")) else { continue };
        let index = text::split_ref(&joined[..spot]).len();
        at = Some(at.map_or(index, |a| a.min(index)));
    }
    let at = at?;
    for word in norm {
        if _lookup(word).iter().any(|(t, w)| *w >= 2 && t != "money" && t != "small-talk") {
            return None;
        }
    }
    Some(at)
}

/// `stone_weight_at(norm)`: "lost 2 stone".
pub fn stone_weight_at(norm: &[String]) -> Option<usize> {
    let spots: Vec<usize> = norm.iter().enumerate().filter(|(_, w)| *w == "stone" || *w == "stones").map(|(k, _)| k).collect();
    if spots.is_empty() {
        return None;
    }
    let joined = norm.join(" ");
    let mut cues: BTreeSet<usize> = norm.iter().enumerate().filter(|(_, w)| STONE_VERBS.contains(*w)).map(|(k, _)| k).collect();
    for phrase in STONE_COMPOUNDS.iter() {
        let mut start = 0;
        while let Some(found) = joined[start..].find(phrase.as_str()) {
            let spot = start + found;
            cues.insert(text::split_ref(&joined[..spot]).len());
            start = spot + 1;
        }
    }
    spots.into_iter().find(|&at| cues.iter().any(|&c| (at as i64 - c as i64).abs() <= 4))
}

/// `tight_on_money(norm, kind=None, text='')`.
pub fn tight_on_money(norm: &[String], kind: Option<&str>, text_: &str) -> bool {
    if kind == Some("question") || norm.is_empty() || norm.len() as i64 > MONEY_TIGHT_MAX {
        return false;
    }
    tight_money_at(norm, text_).is_some()
}

/// `_worn_out(norm)`: "i'm shattered after work".
fn _worn_out(norm: &[String]) -> bool {
    if norm.is_empty() {
        return false;
    }
    let reach = WORN_OUT_LOOKBACK as usize;
    for (at, word) in norm.iter().enumerate() {
        if WORN_OUT.contains(word) && norm[at.saturating_sub(reach)..at].iter().any(|i| WORN_OUT_CUE.contains(i)) {
            return true;
        }
    }
    let joined = format!(" {} ", norm.join(" "));
    for phrase in WORN_OUT_SAID.iter() {
        let Some(spot) = joined.find(&format!(" {phrase} ")) else { continue };
        let all = text::split_ref(&joined[..spot]);
        if all[all.len().saturating_sub(reach)..].iter().any(|i| WORN_OUT_CUE.contains(*i)) {
            return true;
        }
    }
    false
}

/// `rank_topics(line, inherited=None, entities=None, asks=None)`: `[(topic, score)]` with a
/// score of 2 or more, best first.
pub fn rank_topics(line: &Line, inherited: Option<&str>, entities: Option<&[Entity]>, asks: Option<bool>) -> Vec<(String, i64)> {
    let mut found: IndexMap<String, (i64, i64)> = score_topics(line).into_iter().map(|(t, s, top)| (t, (s, top))).collect();
    fn boost(found: &mut IndexMap<String, (i64, i64)>, topic: &str, points: i64) {
        let entry = found.entry(topic.to_string()).or_insert((0, 0));
        entry.0 += points;
        entry.1 = entry.1.max(points);
    }
    let norm = &line.norm;
    let words = sv(norm);
    let entities = entities.unwrap_or(&[]);
    if entities.iter().any(|e| e.kind == "place") {
        let scores: IndexMap<String, i64> = found.iter().map(|(t, (s, _))| (t.clone(), *s)).collect();
        let points = _place_points(line, entities, &scores, asks);
        if points != 0 {
            boost(&mut found, "geography-and-travel", points);
        }
    }
    if entities.iter().any(_a_country_named) {
        boost(&mut found, "geography-and-travel", COUNTRY_BOOST);
    }
    if entities.iter().any(|e| text::split_ref(&e.text.to_lowercase()).last().map(|w| PETS.contains(*w)).unwrap_or_else(|| panic!("IndexError"))) {
        boost(&mut found, "pets", PET_BOOST);
    }
    let reason = reason_positions(norm);
    let outside: HashSet<&str> = norm.iter().enumerate().filter(|(k, _)| !reason.contains(k)).map(|(_, w)| w.as_str()).collect();
    if any_in(outside.iter().copied(), &BODY) && any_in(outside.iter().copied(), &HURTS) {
        boost(&mut found, "health", HURT_BOOST);
    }
    if _head_fog(norm) {
        boost(&mut found, "health", HEAD_FOG_BOOST);
    }
    if _skin_state(norm, &reason).is_some() && !PEOPLE_TOPICS.iter().any(|t| found.get(t).map(|e| e.0).unwrap_or(0) >= 2) {
        boost(&mut found, "health", SKIN_BOOST);
    }
    if _named_condition(norm, &words, &reason) {
        boost(&mut found, "health", CONDITION_BOOST);
    }
    if _a_part_plays_up(norm, &reason).is_some() {
        boost(&mut found, "health", PART_UP_BOOST);
    }
    let warmth = the_warmth_said(norm);
    let sign_words: Vec<String> = match &warmth {
        Some(w) => norm.iter().filter(|x| *x != w).cloned().collect(),
        None => norm.clone(),
    };
    if _a_person_is_unwell(&sign_words, &sv(&sign_words)) {
        boost(&mut found, "health", UNWELL_BOOST);
    }
    if NOSE_BLOCKED.is_match(&format!(" {} ", norm.join(" "))) {
        boost(&mut found, "health", NOSE_BLOCKED_BOOST);
    }
    let animals = _owns_an_animal(norm);
    if animals {
        boost(&mut found, "pets", PET_BOOST);
    }
    if _names_a_friend_of_their_own(norm) {
        boost(&mut found, "friends-and-relationships", FRIEND_BOOST);
    }
    if let Some((topic, _at)) = _what_someone_else_wants(norm) {
        if PEOPLE_TOPICS.iter().any(|t| found.get(t).map(|e| e.0).unwrap_or(0) >= 2) {
            let points = found.get(&topic).map(|e| e.0).unwrap_or(0).max(2) + 1;
            boost(&mut found, &topic, points);
        }
    }
    if _a_crop_is_failing(norm) {
        boost(&mut found, "garden-and-plants", CROP_BOOST);
    }
    if _a_habit_given_up(norm) {
        boost(&mut found, "food-and-drink", GIVING_UP_BOOST);
    }
    if _worn_out(norm) && !_asks_back(norm) {
        boost(&mut found, "sleep", WORN_OUT_BOOST);
    }
    if any_in(words.iter().copied(), &WORLD)
        && !any_in(words.iter().copied(), &WORLD_NOT)
        && !any_in(words.iter().copied(), &NATURE_WORDS)
        && !any_in(words.iter().copied(), &HURTS)
        && !animals
        && _asks_mechanism(norm)
    {
        boost(&mut found, "science", SCIENCE_BOOST);
    }
    for number in detect_numbers(line) {
        if number.unit.as_deref() == Some("percent") {
            boost(&mut found, "maths-and-numbers", 3);
        }
    }
    if _gift_ask(&words, norm) {
        boost(&mut found, "shopping", GIFT_BOOST);
    }
    if found.get("health").map(|e| e.0).unwrap_or(0) >= 3 && found.iter().filter(|(_, e)| e.0 >= 2).all(|(t, _)| t == "health" || t == "small-talk") {
        found.shift_remove("small-talk");
    }
    if let Some(inh) = inherited.filter(|i| !i.is_empty()) {
        for (topic, entry) in found.iter_mut() {
            if topic != inh && entry.1 < 2 {
                entry.0 -= 1;
            }
        }
        boost(&mut found, inh, CARRY);
    }
    let mut scores: IndexMap<String, i64> = found.into_iter().map(|(t, (s, _))| (t, s)).collect();
    if _the_weather_is_just_a_season(norm, &format!(" {} ", line.norm_text)) {
        if let Some(&entry) = scores.get("nature-and-weather") {
            if scores.iter().any(|(t, s)| t != "nature-and-weather" && *s >= 2) {
                scores.insert("nature-and-weather".to_string(), entry.min(1));
            }
        }
    }
    let mut ranked: Vec<(String, i64)> = scores.into_iter().filter(|(_, s)| *s >= 2).collect();
    let inh = inherited.unwrap_or("\u{0}");
    ranked.sort_by(|a, b| (b.1, b.0 == inh).cmp(&(a.1, a.0 == inh)));
    ranked
}

/// `gift_ask(understood)`.
pub fn gift_ask(understood: &crate::understood::Understood) -> bool {
    if understood.norm.is_empty() {
        return false;
    }
    _gift_ask(&sv(&understood.norm), &understood.norm)
}

/// `_gift_ask(words, norm)`: "what should i get my mum for her birthday".
fn _gift_ask(words: &HashSet<&str>, norm: &[String]) -> bool {
    let w = |set: &HashSet<String>| any_in(words.iter().copied(), set);
    if w(&GIFT_ALWAYS) {
        return true;
    }
    if w(&GIFT_WORDS) && (w(&GIFT_ASKING) || w(&GIVE_VERB) || w(&GIFT_OCCASION) || words.contains("for")) {
        return true;
    }
    if w(&GIFT_WANTING) && _gift_still_to_get(words, norm) {
        return true;
    }
    if !(w(&GIFT_ASKING) || w(&GIFT_OCCASION)) {
        return false;
    }
    if w(&GIFT_OCCASION) {
        for position in 0..norm.len().saturating_sub(1) {
            if GIVE_TO.contains(&norm[position]) && norm[position + 1..(position + 3).min(norm.len())].iter().any(|x| GIVE_THINGS.contains(x)) {
                return true;
            }
        }
    }
    for (position, word) in norm.iter().enumerate() {
        if !GIVE_VERB.contains(word) {
            continue;
        }
        let rest = &norm[position + 1..];
        if rest.is_empty() {
            continue;
        }
        let head = &rest[0];
        if GIVE_THING.contains(head) || GIFT_WORDS.contains(head) || GIFT_OCCASION.contains(head) {
            return true;
        }
        let mut tail: Option<&[String]> = None;
        for (index, one) in rest.iter().take(2).enumerate() {
            if GIVE_TO.contains(one) || (GIVE_DETERMINERS.contains(one) && rest.get(index + 1).is_some_and(|n| GIVE_TO.contains(n))) {
                tail = Some(if GIVE_TO.contains(one) { &rest[index + 1..] } else { &rest[(index + 2).min(rest.len())..] });
                break;
            }
        }
        let Some(tail) = tail else { continue };
        if tail.is_empty() || GIVE_RELATIVE.contains(&tail[0]) {
            return true;
        }
        if tail.iter().take(3).any(|o| GIFTS_OR_OCCASIONS.contains(o)) {
            return true;
        }
        if tail[0] == "for" && tail[1..(4).min(tail.len())].iter().any(|o| o.chars().any(pyuni::ch_digit)) {
            return true;
        }
    }
    false
}

/// `_gift_still_to_get(words, norm)`.
fn _gift_still_to_get(words: &HashSet<&str>, norm: &[String]) -> bool {
    if any_in(words.iter().copied(), &GIVE_TO) && words.contains("for") && (words.contains("nothing") || words.contains("anything")) {
        return true;
    }
    for (position, word) in norm.iter().enumerate() {
        if !GIVE_VERB.contains(word) {
            continue;
        }
        let rest = &norm[position + 1..];
        let mut tail: Option<&[String]> = None;
        for (index, one) in rest.iter().take(2).enumerate() {
            if GIVE_TO.contains(one) {
                tail = Some(&rest[index + 1..]);
                break;
            }
            if GIVE_DETERMINERS.contains(one) && rest.get(index + 1).is_some_and(|n| GIVE_TO.contains(n)) {
                tail = Some(&rest[(index + 2).min(rest.len())..]);
                break;
            }
        }
        let Some(tail) = tail else { continue };
        if tail.is_empty() {
            continue;
        }
        if GIVE_THING.contains(&tail[0]) || GIFT_WORDS.contains(&tail[0]) {
            return true;
        }
        if tail.iter().any(|t| t == "for") && (words.contains("nothing") || words.contains("anything")) {
            return true;
        }
        if tail.iter().take(3).any(|o| GIFT_OCCASION.contains(o)) {
            return true;
        }
    }
    false
}

/// `subject_topic(line, entities=None)`: the first topic a strong word says.
pub fn subject_topic(line: &Line, _entities: Option<&[Entity]>) -> Option<String> {
    score_topics(line).into_iter().find(|(_, _, top)| *top >= 2).map(|(t, _, _)| t)
}

/// `the_words_that_said(line, topic, entities=None)`: `[(word, weight)]` that said the topic.
pub fn the_words_that_said(line: &Line, topic: Option<&str>, _entities: Option<&[Entity]>) -> Vec<(String, i64)> {
    let Some(topic) = topic.filter(|t| !t.is_empty()) else { return vec![] };
    let norm = &line.norm;
    let mut found: Vec<(String, i64)> = Vec::new();
    let mut covered: HashSet<&str> = HashSet::new();
    for size in 2..5 {
        if size > norm.len() {
            continue;
        }
        for start in 0..=(norm.len() - size) {
            let key = norm[start..start + size].join(" ");
            for (name, weight) in TABLE.get(&key).map(Vec::as_slice).unwrap_or(&[]) {
                if name == topic {
                    found.push((key.clone(), *weight));
                    covered.extend(norm[start..start + size].iter().map(String::as_str));
                }
            }
        }
    }
    for word in norm {
        if covered.contains(word.as_str()) {
            continue;
        }
        for (name, weight) in _lookup(word).iter() {
            if name == topic {
                found.push((word.clone(), *weight));
            }
        }
    }
    found.sort_by(|a, b| (-a.1, &a.0).cmp(&(-b.1, &b.0)));
    found
}

/// `detect_topic(line, inherited=None, entities=None, asks=None)`: `(topic, score)`.
pub fn detect_topic(line: &Line, inherited: Option<&str>, entities: Option<&[Entity]>, asks: Option<bool>) -> (Option<String>, i64) {
    let ranked = rank_topics(line, inherited, entities, asks);
    if let Some((t, s)) = ranked.into_iter().next() {
        return (Some(t), s);
    }
    match inherited.filter(|i| !i.is_empty()) {
        Some(i) => (Some(i.to_string()), 0),
        None => (None, 0),
    }
}

/// `_is_name(word)`: a capitalised word that is a name, not an ordinary word.
fn _is_name(word: &str) -> bool {
    let lower = word.to_lowercase();
    if COMMON_WORDS.contains(&lower) && lower != "i" {
        return false;
    }
    let stem = lower.split('\'').next().unwrap_or("");
    if stem != lower && (LX_CORE_WORDS.contains(stem) || (stem.ends_with('n') && LX_CORE_WORDS.contains(&stem[..stem.len() - 1]))) {
        return false;
    }
    !LX_CHAT.contains_key(&lower) || lower == "i"
}

/// `_capitalised(text)`: runs of capitalised words that read as names.
fn _capitalised(text_: &str) -> Vec<String> {
    let mut runs: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for token in text::split_ref(text_) {
        let bare = text::strip_chars(token, ".,!?;:\"'()[]");
        if bare.chars().next().is_some_and(pyuni::ch_upper) && pyuni::is_alpha(&bare.replace(['-', '\''], "")) {
            current.push(bare.to_string());
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    let mut names = Vec::new();
    for run in runs {
        if run.len() == 1 {
            if _is_name(&run[0]) {
                names.push(run[0].clone());
            }
            continue;
        }
        let mut run: &[String] = &run;
        let first = run[0].to_lowercase();
        if ARTICLES.contains(&first) || LEADING.contains(&first) {
            run = &run[1..];
        }
        if run.is_empty() {
            continue;
        }
        let Some(start) = run.iter().position(|w| _is_name(w) || PLACES.contains(&w.to_lowercase())) else { continue };
        let mut part: Vec<&str> = Vec::new();
        for word in &run[start..] {
            if NAME_BREAKS.contains(&word.to_lowercase()) {
                break;
            }
            part.push(word);
        }
        if !part.is_empty() {
            names.push(part.join(" "));
        }
    }
    names
}

/// `detect_entities(line, text=None)`: the things, places, names and people the line names.
pub fn detect_entities(line: &Line, text_: Option<&str>) -> Vec<Entity> {
    let text_ = text_.unwrap_or(&line.text);
    let norm = &line.norm;
    if small_talk_only(norm) {
        return vec![];
    }
    let mut entities: Vec<Entity> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let add = |entities: &mut Vec<Entity>, seen: &mut HashSet<String>, name: &str, kind: &str, at: Option<usize>| {
        let key = name.to_lowercase();
        if !key.is_empty() && !seen.contains(&key) && text::len(&key) > 1 {
            seen.insert(key.clone());
            let at = at.unwrap_or_else(|| _position(line, &key));
            entities.push(Entity { text: text::strip(name).to_string(), kind: kind.to_string(), at: at as i64 });
        }
    };
    for m in rx(r#"\*([^*]{2,60})\*|\"([^\"]{2,60})\""#).finditer(text_) {
        let name = m.get(1).filter(|g| !g.is_empty()).or(m.get(2)).unwrap_or("");
        add(&mut entities, &mut seen, name, "title", None);
    }
    for person in rx(r"\bmy ([a-z]+(?:\s+[a-z]+)?)").finditer(&line.clean) {
        let word = text::split_ref(person.str(1))[0].to_string();
        if RELATIONS.contains(&word) {
            add(&mut entities, &mut seen, &format!("my {word}"), "person", None);
            seen.insert(word);
        }
    }
    for at in _ages_of_a_child(norm) {
        let after = norm.get(at + 3).map(String::as_str).unwrap_or("");
        if !THE_PERSONS_OWN.contains(&norm[at - 1]) {
            continue;
        }
        if !after.is_empty() && !AGE_OF_A_CHILD_WORDS.contains(after) && (PETS.contains(after) || _thing(after).is_some()) {
            continue;
        }
        let name = norm[at - 1..(at + 3).min(norm.len())].join(" ");
        add(&mut entities, &mut seen, &name, "person", Some(at - 1));
    }
    for run in _capitalised(text_) {
        let low = run.to_lowercase();
        if matches!(low.as_str(), "i" | "i'm" | "im") {
            continue;
        }
        let first_word = text::split_ref(&low)[0].to_string();
        let mut kind = if PLACES.contains(&low) || PLACES.contains(&first_word) { "place" } else { "name" };
        if kind == "name" && _said_as_a_place(&run, norm) {
            kind = "place";
        }
        add(&mut entities, &mut seen, &run, kind, None);
    }
    let verdicts = _bad_verdict_positions(norm);
    let clean_words = text::split_ref(&line.clean);
    for (index, word) in norm.iter().enumerate() {
        if verdicts.contains(&index) {
            continue;
        }
        let asked = _asked_as_a_country(norm, index);
        if PLACES.contains(word) && _names_a_person(norm, index) {
            continue;
        }
        if PLACES.contains(word) && (_names_a_place(word, text_, asked) || _placed_by_a_word(norm, index)) {
            add(&mut entities, &mut seen, word, "place", None);
            continue;
        }
        if RELATIONS.contains(word) {
            if index > 0 && !clean_words.contains(&word.as_str()) && OWNERS.contains(&norm[index - 1]) {
                add(&mut entities, &mut seen, &format!("{} {}", norm[index - 1], word), "person", Some(index - 1));
                seen.insert(word.clone());
                continue;
            }
            add(&mut entities, &mut seen, word, "person", None);
            continue;
        }
        if let Some(thing) = _thing(word) {
            add(&mut entities, &mut seen, &thing, "thing", None);
        }
    }
    entities.sort_by_key(|e| e.at);
    entities
}

/// `_said_as_a_place(run, norm)`: "in Paris".
fn _said_as_a_place(run: &str, norm: &[String]) -> bool {
    let first = text::strip_chars(&text::split_ref(run)[0].to_lowercase(), ".,!?;:\"'()[]").to_string();
    if NOT_PLACES.contains(&first) {
        return false;
    }
    for (index, word) in norm.iter().enumerate() {
        if *word != first {
            continue;
        }
        if _places_a_name(norm, index) {
            return true;
        }
        if index >= 2 && PLACE_THING.contains(&norm[index - 1]) && PLACE_WORDS.contains(&norm[index - 2]) {
            if norm[index - 2] == "at" && norm.iter().any(|w| WORK_WORDS.contains(w)) {
                return false;
            }
            return true;
        }
        return false;
    }
    false
}

/// `_placed_by_a_word(norm, index)`.
fn _placed_by_a_word(norm: &[String], index: usize) -> bool {
    _places_a_name(norm, index)
}

/// `_has_travel_word(norm)`.
fn _has_travel_word(norm: &[String]) -> bool {
    if norm.iter().any(|w| TRAVEL_CUES.contains(w)) {
        return true;
    }
    norm.iter().any(|w| w.ends_with('s') && TRAVEL_CUES.contains(&w[..w.len() - 1]))
}

/// `_places_a_name(norm, index)`: "in", "to", "from" in front.
fn _places_a_name(norm: &[String], index: usize) -> bool {
    if index < 1 || index >= norm.len() {
        return false;
    }
    let before = &norm[index - 1];
    if PLACE_WORDS_WITH_CUE.contains(before) {
        return _has_travel_word(norm);
    }
    if !PLACE_WORDS.contains(before) {
        return false;
    }
    if before == "at" && norm.iter().any(|w| WORK_WORDS.contains(w)) {
        return false;
    }
    true
}

/// `_a_country_named(entity)`.
fn _a_country_named(entity: &Entity) -> bool {
    if entity.kind != "place" {
        return false;
    }
    text::split_ref(&entity.text.to_lowercase()).last().is_some_and(|w| PLACE_HOMOGRAPHS.contains(*w))
}

/// `_set_in_a_place(norm, index)`.
fn _set_in_a_place(norm: &[String], index: usize) -> bool {
    if index + 2 < norm.len() && norm[index + 1] == "in" && PLACES.contains(&norm[index + 2]) && !PLACE_HOMOGRAPHS.contains(&norm[index + 2]) {
        return true;
    }
    _compared_with_a_place(norm, index, 3)
}

fn _compared_with_a_place(norm: &[String], index: usize, reach: usize) -> bool {
    for near in index.saturating_sub(reach)..norm.len().min(index + reach + 1) {
        if near != index && PLACES.contains(&norm[near]) && !PLACE_HOMOGRAPHS.contains(&norm[near]) {
            let between = &norm[index.min(near)..index.max(near)];
            if between.iter().any(|w| COMPARE_WORDS.contains(w)) {
                return true;
            }
        }
    }
    false
}

/// `_asked_as_a_country(norm, index)`.
fn _asked_as_a_country(norm: &[String], index: usize) -> bool {
    if index > 0 && ARTICLES.contains(&norm[index - 1]) {
        return false;
    }
    if norm.iter().any(|w| COUNTRY_CUES.contains(w)) {
        return true;
    }
    if _set_in_a_place(norm, index) {
        return true;
    }
    _compared_with_a_place(norm, index, 2)
}

/// `_names_a_person(norm, index)`: "jordan is my friend".
fn _names_a_person(norm: &[String], index: usize) -> bool {
    let before = if index > 0 { norm[index - 1].as_str() } else { "" };
    if PERSON_WORDS.contains(before) || NAME_FRAMES.contains(before) {
        return true;
    }
    index + 3 < norm.len() && norm[index + 1] == "is" && norm[index + 2] == "my" && PERSON_WORDS.contains(&norm[index + 3])
}

/// `person_names(line)`: place words used as people's names.
pub fn person_names(line: &Line) -> Vec<String> {
    let norm = &line.norm;
    let mut seen: HashSet<&str> = HashSet::new();
    let mut names = Vec::new();
    for (index, word) in norm.iter().enumerate() {
        if PLACES.contains(word) && !seen.contains(word.as_str()) && _names_a_person(norm, index) {
            seen.insert(word);
            names.push(word.clone());
        }
    }
    names
}

/// `_names_a_place(word, text, asked=False)`.
fn _names_a_place(word: &str, text_: &str, asked: bool) -> bool {
    if !PLACE_HOMOGRAPHS.contains(word) {
        return true;
    }
    if asked && COUNTRY_HOMOGRAPHS.contains(word) {
        return true;
    }
    rx(&format!(r"\b{}\b", text::capitalize(word))).is_match(text_)
}

/// `_position(line, name)`: where in `norm` the name starts.
fn _position(line: &Line, name: &str) -> usize {
    let first = text::split_ref(name).first().copied().unwrap_or_else(|| panic!("IndexError: list index out of range"));
    line.norm.iter().position(|w| w == first || w.starts_with(first)).unwrap_or(line.norm.len())
}

/// `_thing(word)`: the word names a thing.
fn _thing(word: &str) -> Option<String> {
    if THINGS.contains(word) {
        return Some(word.to_string());
    }
    if text::len(word) <= 3 {
        return None;
    }
    let candidates = [
        Some(word.trim_end_matches('s').to_string()),
        if word.ends_with("es") { Some(word[..word.len() - 2].to_string()) } else { None },
        if word.ends_with("ies") { Some(word[..word.len() - 3].to_string()) } else { None },
    ];
    for c in candidates.into_iter().flatten() {
        if !c.is_empty() && THINGS.contains(&c) {
            return Some(word.to_string());
        }
    }
    None
}

/// `_spoken_numbers(norm)`: the values of every number said (as words or figures).
fn _spoken_numbers(norm: &[String]) -> Vec<f64> {
    let mut values = Vec::new();
    for word in norm {
        if let Some(v) = WORD_NUMBERS.get(word) {
            values.push(as_f64(v));
            continue;
        }
        if let Some(d) = WORD_DIGITS.match_(word) {
            if let Ok(v) = d.as_str().replace(',', ".").parse::<f64>() {
                values.push(v);
            }
        }
    }
    values
}

/// `_mixed_number(norm, position, value)`: "two and a half".
fn _mixed_number(norm: &[String], position: usize, value: Num) -> (Num, Vec<usize>) {
    let words = &norm[(position + 1).min(norm.len())..(position + 4).min(norm.len())];
    if words.first().map(String::as_str) != Some("and") {
        return (value, vec![]);
    }
    let step = if matches!(words.get(1).map(String::as_str), Some("a" | "an")) { 2 } else { 1 };
    let fraction = words.get(step).map(String::as_str);
    if !matches!(fraction, Some("half" | "quarter")) {
        return (value, vec![]);
    }
    let add = if fraction == Some("half") { 0.5 } else { 0.25 };
    (Num::Float(as_f64(&value) + add), (position + 1..position + 1 + step + 1).collect())
}

/// `_the_figure_and_its_unit(digits, unit, raw='')`.
fn _the_figure_and_its_unit(digits: &str, unit: &str, raw: &str) -> String {
    if unit.is_empty() {
        return digits.to_string();
    }
    if unit == "%" {
        return format!("{digits}%");
    }
    if UNIT_WORDS.contains_key(unit) || CLIPPED_UNIT_WORDS.contains_key(unit) || FIGURE_SIZE_WORDS.contains(unit) {
        let next = text::slice(raw, text::len(digits) as isize, Some(text::len(digits) as isize + 1));
        let gap = if !next.is_empty() && next.chars().all(text::is_space) { " " } else { "" };
        return format!("{digits}{gap}{unit}");
    }
    digits.to_string()
}

/// A Python number from a parsed figure (an int when whole, as `int(value)` there).
fn figure_num(value: f64) -> Num {
    if value.fract() == 0.0 && value.abs() < 9.0e18 { Num::Int(value as i64) } else { Num::Float(value) }
}

/// `detect_numbers(line)`: every number the line gives, with its unit.
pub fn detect_numbers(line: &Line) -> Vec<Number> {
    let norm = &line.norm;
    let mut out: Vec<Number> = Vec::new();
    // seen keys: (tuple length, value, unit, word)
    let mut seen: Vec<(u8, f64, Option<String>, Option<String>)> = Vec::new();
    let mut typed: Vec<(usize, f64)> = Vec::new();
    let mut merged: BTreeSet<usize> = BTreeSet::new();
    let spoken = _spoken_numbers(norm);
    let clean = &line.clean;
    for m in NUMBER_RE.finditer(clean) {
        let raw = text::strip(m.as_str());
        let digits = m.str(1);
        let unit = m.str(2).to_lowercase();
        let Ok(parsed) = digits.replace(',', ".").parse::<f64>() else { continue };
        let value = figure_num(parsed);
        if !spoken.iter().any(|s| *s == parsed) {
            continue;
        }
        if MIXED_JOINERS.contains(&unit) && match_at(&MIXED_FRACTION, clean, m.end()).is_some() {
            continue;
        }
        let mut canonical = UNIT_WORDS.get(&unit).filter(|u| !u.is_empty()).or_else(|| UNIT_WORDS.get(unit.trim_end_matches('s')).filter(|u| !u.is_empty())).cloned();
        if canonical.is_none() {
            canonical = CLIPPED_UNIT_WORDS.get(&unit).cloned();
        }
        if canonical.is_none() && unit.is_empty() {
            if let Some(clipped) = match_at(&CLIPPED_AFTER_A_SLASH, clean, m.end()) {
                canonical = Some(CLIPPED_UNIT_WORDS.get(&text::strip(clipped.as_str()).to_lowercase()).cloned().unwrap_or_else(|| "years".to_string()));
            }
        }
        typed.push((clean[..m.start()].matches(' ').count(), parsed));
        if !seen.iter().any(|(n, v, u, _)| *n == 2 && *v == parsed && *u == canonical) {
            seen.push((2, parsed, canonical.clone(), None));
            out.push(Number { value: Some(value), unit: canonical, text: _the_figure_and_its_unit(digits, &unit, raw) });
        }
    }
    for (position, word) in norm.iter().enumerate() {
        if word == "a" || word == "an" {
            if position + 1 < norm.len() && A_PORTION_UNITS.contains(&norm[position + 1]) {
                let before = if position > 0 { norm[position - 1].as_str() } else { "" };
                let after = norm.get(position + 2).map(String::as_str).unwrap_or("");
                let unit = UNIT_WORDS.get(&norm[position + 1]).cloned();
                if A_PORTION_NEEDS_OF.contains(&norm[position + 1]) && after != "of" {
                    continue;
                }
                let dup = seen.iter().any(|(n, v, u, _)| *n == 2 && *v == 1.0 && *u == unit);
                if !OWNS_THE_UNIT.contains(before) && unit.as_ref().is_some_and(|u| !u.is_empty()) && !dup {
                    seen.push((2, 1.0, unit.clone(), None));
                    out.push(Number { value: Some(Num::Int(1)), unit, text: format!("{} {}", word, norm[position + 1]) });
                }
            }
            continue;
        }
        let Some(value) = WORD_NUMBERS.get(word).cloned() else { continue };
        if _counts_as_number(norm, position) {
            continue;
        }
        if merged.contains(&position) {
            continue;
        }
        if typed.iter().any(|(at, got)| *got == as_f64(&value) && (*at as i64 - position as i64).abs() <= 1) {
            continue;
        }
        let (value, folded) = _mixed_number(norm, position, value);
        merged.extend(folded.iter().copied());
        let mut unit: Option<String> = None;
        let mut step = 1 + folded.len();
        while position + step < norm.len() && step <= 5 {
            let after_word = &norm[position + step];
            if let Some(u) = UNIT_WORDS.get(after_word) {
                unit = Some(u.clone());
                break;
            }
            if !NUMBER_FILLER.contains(after_word) {
                break;
            }
            step += 1;
        }
        let v = as_f64(&value);
        if !seen.iter().any(|(n, sv_, u, w)| *n == 3 && *sv_ == v && *u == unit && w.as_deref() == Some(word.as_str())) {
            seen.push((3, v, unit.clone(), Some(word.clone())));
            out.push(Number { value: Some(value), unit, text: word.clone() });
        }
    }
    out
}

/// `_counts_as_number(norm, position)`: "one" said as a thing ("the red one"), not a number.
fn _counts_as_number(norm: &[String], position: usize) -> bool {
    if norm[position] != "one" {
        return false;
    }
    let following: Vec<&str> = norm[(position + 1).min(norm.len())..(position + 4).min(norm.len())].iter().map(String::as_str).collect();
    if following.len() >= 3 && (following[..3] == ["and", "a", "half"] || following[..3] == ["and", "an", "half"]) {
        return false;
    }
    if following.len() >= 2 && (following[..2] == ["and", "half"] || following[..2] == ["and", "quarter"]) {
        return false;
    }
    let before = if position > 0 { Some(norm[position - 1].as_str()) } else { None };
    let after = norm.get(position + 1).map(String::as_str);
    if after.is_some_and(|a| !a.is_empty() && UNIT_WORDS.contains_key(a)) {
        return false;
    }
    before.is_none() || has(&ONE_IS_A_THING, before) || after.is_none()
}

/// `_index()`: TABLE built from STRONG, WEAK, CAR_NAMES and PHRASES (PyBot builds it once at
/// import; the port reads the built TABLE from the bundle and keeps this for reference).
pub fn _index() -> IndexMap<String, Vec<(String, i64)>> {
    static _BREAKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_BREAKS"));
    let mut table: IndexMap<String, Vec<(String, i64)>> = IndexMap::new();
    let mut add = |key: &str, topic: &str, weight: i64| {
        let entries = table.entry(key.to_string()).or_default();
        for e in entries.iter_mut() {
            if e.0 == topic {
                if weight > e.1 {
                    e.1 = weight;
                }
                return;
            }
        }
        entries.push((topic.to_string(), weight));
    };
    for (topic, words) in STRONG.iter() {
        for word in text::split_ref(words) {
            if !GENERIC.contains(word) {
                add(word, topic, 2);
            }
        }
    }
    for (topic, words) in WEAK.iter() {
        for word in text::split_ref(words) {
            if !GENERIC.contains(word) {
                add(word, topic, 1);
            }
        }
    }
    for name in CAR_NAMES.iter() {
        add(name, "cars", 2);
    }
    for (topic, phrases) in PHRASES.iter() {
        for phrase in phrases {
            let key = text::squash(&_BREAKS.sub(" ", phrase));
            if !key.is_empty() && !GENERIC.contains(&key) {
                let ordinary = text::split_ref(&key).iter().all(|w| ORDINARY_VERBS.contains(*w));
                add(&key, topic, if ASK_PHRASES.contains(&key) || ordinary { 1 } else { 3 });
            }
        }
    }
    table
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

    /// Python: bot/understanding/topics.py:2338 `_function_words()`
    pub fn _function_words() -> Vec<String> {
        crate::unported!("bot.understanding.topics:_function_words")
    }

}

