//! Port of `bot/knowledge/health.py`: everyday health questions. The entries are data in the
//! bundle; this is every guard and reply function they use, with the helpers those read.
//!
//! Build-time only (not ported): the outer body of `_not_a_denial_of` (it builds the guard's
//! pattern at import time; the pattern is the guard's captured value), and the `said(...)`,
//! `time_entries(...)` calls that build keys and entries.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d2_shared::{not_the_flat_you_live_in, py_repr_list};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::{Num, Number, Understood};
use std::collections::HashSet;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.knowledge.health.{n}")
}
fn s(n: &str) -> String {
    data::string(&name(n))
}
fn strs(n: &str) -> Vec<String> {
    data::strs(&name(n))
}
fn set(n: &str) -> HashSet<String> {
    data::set(&name(n))
}
fn rx(n: &str) -> &'static Re {
    data::re(&name(n))
}
fn float(n: &str) -> f64 {
    data::get(&name(n)).as_f64().expect("a number")
}
fn any_in(text: &str, words: &[String]) -> bool {
    words.iter().any(|w| text.contains(w.as_str()))
}
fn or<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.is_empty() { b } else { a }
}
fn hits<S: AsRef<str>>(words: &HashSet<String>, of: &[S]) -> bool {
    of.iter().any(|w| words.contains(w.as_ref()))
}
fn words_set(u: &Understood) -> HashSet<String> {
    u.words.iter().cloned().collect()
}

macro_rules! lazy_re {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| rx(stringify!($n)));)* };
}
macro_rules! lazy_strs {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<Vec<String>> = LazyLock::new(|| strs(stringify!($n)));)* };
}
macro_rules! lazy_set {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<HashSet<String>> = LazyLock::new(|| set(stringify!($n)));)* };
}
macro_rules! lazy_s {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<String> = LazyLock::new(|| s(stringify!($n)));)* };
}

lazy_re!(
    _A_MACHINE, _A_FAULT, _A_WHEN_ASK, _A_CHILDS_AGE, _ON_THE_CARD, _AN_ILLNESS, _PREGNANT_AT_ALL,
    _PREGNANT_DENIED, _A_PERSONS_OWN_EYE, _AN_EYE_ON_AN_ANIMAL, _AN_ANIMALS_OWN_EYE, _PROSTHESIS_GIVING_WAY,
    _PROSTHESIS_HURTS, _PROSTHESIS_CLEANING, BP_FIGURES, AGE_OF_A_CHILD, TEMPERATURE_NOUN, _LOW_QUESTION,
    _COLD_SORE_EYE, _COLD_SORE_PREGNANT, _COLD_SORE_BABY, _COLD_SORE_POP, _COLD_SORE_HELP, _COLD_SORE_CREAM,
    _COLD_SORE_SPREAD, _COLD_SORE_CAUSE, _COLD_SORE_EARLY, _CANNOT_WARM, _A_SESSION, _SIGNS_ASK, _SOMEONE_ELSE,
    _MORNING_AFTER_OLD, _MORNING_AFTER_SICKS, _MORNING_AFTER_OVULATION, _MORNING_AFTER_HURTS,
    _MORNING_AFTER_FITTED, _MORNING_AFTER_LEVO, _MORNING_AFTER_ULI, _MORNING_AFTER_COILS,
    _MORNING_AFTER_EFFECTIVE, _MORNING_AFTER_FREEISH, _MORNING_AFTER_WHERE, _MORNING_AFTER_LATE, _MMOL,
    _NAMES_A_DOCTOR, _SAYS_IT_OUT_LOUD, _NAMES_A_STROKE_SIGN, _NAMES_A_DENTAL_CAUSE, _SAYS_NUMB, _TURNED_DOWN,
    _HOW_IT_LEFT_THEM, _A_HOME_WORD, _A_COLD_OR_DAMP_WORD, _A_BABY_IN_THE_LINE, _A_STOMACH_SAID, _A_NIGHT,
    _UP_TO_THE_LOO, _SOMEBODY_NAMED, _NOT_A_CONDITION, _THE_WORD_IS_A_HEAP, _MEANS, _WASTE, _EYE_TROUBLE,
    _SKIN_ITSELF, _JAUNDICE_SKIN, _CHANGE_SUBJECT, _INTERACTION_WORD, _STATIN_WHETHER, _A_STOPPING_A_MEDICINE,
    _A_DRIVING_AGAIN, _A_SPORT_AGAIN, _A_JOB_AGAIN, _A_PERMISSION_ASK, _SOMETHING_TO_GO_BACK_TO,
    _RESULT_SAYS_CLEAR, _A_CHECKING_THE_RESULT, _RULES_IT_OUT, _A_TEST_NAME, _RESULT_SAYS_ABNORMAL,
    _A_FASTING_LENGTH, _A_TEST_THAT_TAKES, _ASKS_WHETHER_SOMEONE_HAS_IT, _A_GASP_FOR_A_THING, _AN_AGE,
    _PET_AS_SUBJECT, _PET_SYMPTOM, _THE_PERSON_IS_REACTING, _TEEN_SAID, _ADULT_AGE, _ADULT_NAMED,
);
lazy_strs!(
    NO_DRUGS, SHAKE_HANDS, MELATONIN_PREGNANT, MELATONIN_CHILD, GIVE_TO_A_GROWN_UP, GIVE_TO_SOMEONE, NOT_A_BODY,
    WHITLOW_WORDS, KEEPING_FLUID_DOWN, _GLASS_EYE_URGENT, _BLUES_SAID, _BLOOD_WORDS, _A_PRESSURE_IN_A_PIPE,
    _ANTACID_MIND_THE_GAP, _GRAPEFRUIT, _THINNER, _STATIN, _CRANBERRY, _DAIRY, _THYROID_PILL, _IRON_PILL,
    _TETRACYCLINE, _GREENS, _MEDICINE_WORDS, _MORE_THAN_ONE, _STATIN_STARTED, _STATIN_NAMES, _CHILD_WORD,
    _GROWN_RELATION, _FRUIT_ITSELF, _MOOD_THE_NOUN, _MOOD_A_HOME_FLAT, _MOOD_WORDS, _MOOD_ADVERB,
    _MOOD_THE_STATE, _MOOD_SAID_OF_A_PERSON,
);
lazy_set!(
    TABLET_DEVICE, NAMED_MEDICINES, TABLET_AS_MEDICINE, COLD_WARMTH_WORDS, HOT_WARMTH_WORDS, _RUNNING_WORDS,
    _A_RATE_NAMED_HIGH_OR_LOW, _MOOD_FEELING_SAID, _MOOD_JOIN, _MOOD_TIME, _MOOD_HELPERS,
    _MOOD_WHERE_THE_PERSON_WAS, _MOOD_THE_VERB_BEFORE,
);
lazy_s!(
    PREGNANT_TABLETS_NOTE, _TOOTHACHE_PLAIN, HEADACHE_NO_DRUGS, HEADACHE_PREGNANT, HEADACHE_PLAIN,
    _PROSTHESIS_GIVES_WAY, _PROSTHESIS_SORE, _PROSTHESIS_CLEAN, _GLASS_EYE_CLINIC, _GLASS_EYE_CLEAN,
    STOMACH_ANTIBIOTIC, STOMACH_PARACETAMOL, STOMACH_HARD, SHAKE_HANDS_TEXT, SHAKE_ALL_OVER, BP_NORMAL, BP_HIGH,
    SIDE_EFFECT_TEXT, TABLET_TEXT, MELATONIN_PREGNANT_TEXT, MELATONIN_CHILD_TEXT, MELATONIN_TEXT, FEVER_ADVICE,
    NORMAL_TEMPERATURE, LOW_READING, DEHYDRATION_SICK, DEHYDRATION_PLAIN, MORNING_AFTER_AGE, MORNING_AFTER_SICK,
    MORNING_AFTER_OVULATED, MORNING_AFTER_FITTED_HURT, MORNING_AFTER_FITTED, MORNING_AFTER_EFFECT,
    MORNING_AFTER_COIL, MORNING_AFTER_FREE, MORNING_AFTER_CLOCK, MORNING_AFTER_PREG, _JAUNDICE_ADULT,
    _JAUNDICE_NEWBORN, STATIN_WITH_OTHER_TABLETS, STATIN_TIMING, STATIN_FIRST_WEEKS, SCAN_MRI, SCAN_CT,
    SCAN_ULTRASOUND, SCAN_ANY, AFTER_A_CLEAR_RESULT_MEDICINE, AFTER_A_CLEAR_RESULT_DRIVING,
    AFTER_A_CLEAR_RESULT_EXERCISE, AFTER_A_CLEAR_RESULT_WORK, AFTER_A_CLEAR_RESULT_TEXT,
    CLEAR_RESULT_FOR_A_PET_TEXT, CLEAR_RESULT_TEXT, NEGATIVE_RESULT_TEXT, ABNORMAL_RESULT_FOR_A_PET_TEXT,
    ABNORMAL_RESULT_TEXT, CHASE_RESULT_TEXT, _A_FASTING_BEFORE_A_TEST_TEXT, _A_BLOOD_TEST_TEXT, _CATARACT_LENGTH,
    _CATARACT_WHAT, ADULT_COUGH_REPLY, _A_HERNIA,
);

static TABLETS_SAFE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&name("TABLETS_SAFE")).items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
static _HELP_WORDS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get(&name("_HELP_WORDS")).items().iter().map(|p| p.strs()).collect());
static DRINK_PILLS: LazyLock<std::collections::HashMap<String, String>> =
    LazyLock::new(|| data::str_map(&name("DRINK_PILLS")));
static BODY_RANGE: LazyLock<(f64, f64)> = LazyLock::new(|| {
    let r = data::get(&name("BODY_RANGE")).items();
    (r[0].as_f64().unwrap(), r[1].as_f64().unwrap())
});
static FEVER_FROM: LazyLock<f64> = LazyLock::new(|| float("FEVER_FROM"));
static COLD_BELOW: LazyLock<f64> = LazyLock::new(|| float("COLD_BELOW"));
static LOW_FROM: LazyLock<f64> = LazyLock::new(|| float("LOW_FROM"));
static RAISED_FROM: LazyLock<f64> = LazyLock::new(|| float("RAISED_FROM"));
static WHITLOW_RES: LazyLock<Vec<Re>> =
    LazyLock::new(|| WHITLOW_WORDS.iter().map(|w| Re::new(&format!(r"\b{}\b", escape(w)), 0)).collect());
static LETTERS_APOS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));
static NOT_LETTER_OR_SPACE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z ]", 0));
static POO_BLACK: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(black|tarry|tar\s*-?\s*like|very dark|dark red)\b", 0));
static POO_PALE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(pale|clay|whitish|white|greasy|yellowish)\b", 0));
static POO_URGENT: LazyLock<Re> = LazyLock::new(|| Re::new(POO_COLOUR_3, 0));
static COIL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bcoil\b|\biud\b", crate::py::re::I));
static RATE_SPREAD: LazyLock<Re> = LazyLock::new(|| Re::new("(\\d{2,3})\\s*(?:to|-|\u{2013})\\s*(\\d{2,3})", 0));
static SKIN_OR_EYES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:skin|eyes?|eyeb?all|whites|face)\b", 0));
static SCAN_MRI_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bmri\b|magnetic resonance", 0));
static SCAN_CT_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bct\b|\bcat scan\b|\bct scan\b", 0));
static VET: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:vet|vets)\b", 0));
static RESULT_CLEAR_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(A_RESULT_REPLY_5, 0));
static RESULT_NEGATIVE_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:negative|not detected|no trace)\b", 0));
static RESULT_ABNORMAL_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(A_RESULT_REPLY_6, 0));
static RESULT_FOLLOW_UP: LazyLock<Re> = LazyLock::new(|| Re::new(A_RESULT_REPLY_3, 0));
static RESULT_CHASE: LazyLock<Re> = LazyLock::new(|| Re::new(A_RESULT_REPLY_4, 0));
static RESULT_ANY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:result|results|follow up|follow-up|came back)\b", 0));
/// `_not_a_denial_of("diabetes", "diabetic")`, built once.
static NOT_A_DENIAL_OF_DIABETES: LazyLock<Re> = LazyLock::new(|| _not_a_denial_of_pattern(&["diabetes", "diabetic"]));

/// Python `str(understood.get(key) or "")` for a list field: its repr, or "" when empty.
fn list_or_empty(items: &[String]) -> String {
    if items.is_empty() { String::new() } else { py_repr_list(items) }
}

/// True unless the line is about a machine that broke (a boiler, a fridge...).
pub fn not_a_machine_that_broke(understood: &Understood) -> bool {
    // keys "raw" and "rewritten" are not in a reading: "" each
    let bits = [understood.clean.as_str(), understood.text.as_str(), "", "", understood.plain.as_str()];
    let mut parts: Vec<&str> = bits.to_vec();
    parts.extend(understood.words.iter().map(String::as_str));
    let t = parts.join(" ");
    !(_A_MACHINE.is_match(&t) && _A_FAULT.is_match(&t))
}

/// True unless the line asks when (how often, how long until).
pub fn not_a_when_ask(understood: &Understood) -> bool {
    let words_repr = list_or_empty(&understood.words);
    let mut parts: Vec<String> = vec![understood.clean.clone(), understood.text.clone(), String::new(), words_repr];
    if !understood.words.is_empty() {
        parts.push(understood.words.join(" "));
    }
    !_A_WHEN_ASK.is_match(&parts.join(" "))
}

/// Neither the flat you live in nor a machine that broke.
pub fn not_a_season_or_a_machine(understood: &Understood) -> bool {
    not_the_flat_you_live_in(understood) && not_a_machine_that_broke(understood)
}

/// The line's words, lower-cased (or the clean line when it has none).
pub fn _the_line(understood: &Understood) -> String {
    if !understood.words.is_empty() {
        return understood.words.join(" ").to_lowercase();
    }
    understood.clean.to_lowercase()
}

/// True unless the line names a child's age.
pub fn not_a_childs_age(understood: &Understood) -> bool {
    !_A_CHILDS_AGE.is_match(&_the_line(understood))
}

/// True unless the line names an illness the card does not cover.
pub fn _an_illness_the_card_covers(understood: &Understood) -> bool {
    let t = _the_line(understood);
    if _ON_THE_CARD.is_match(&t) {
        return true;
    }
    !_AN_ILLNESS.is_match(&t)
}

/// The reply with the pregnancy-safe painkiller swapped in when the line says they are pregnant.
pub fn tablets_for_a_pregnancy(reply: Option<String>, understood: &Understood) -> Option<String> {
    let reply = reply?;
    let t = text::squash(&or(&understood.clean, &understood.text).to_lowercase());
    if !_PREGNANT_AT_ALL.is_match(&t) || _PREGNANT_DENIED.is_match(&t) {
        return Some(reply);
    }
    let mut said = reply;
    let mut swapped = false;
    for (from, to) in TABLETS_SAFE.iter() {
        if said.contains(from.as_str()) {
            said = said.replace(from.as_str(), to);
            swapped = true;
        }
    }
    let low = said.to_lowercase();
    if low.contains("ibuprofen") || low.contains("nurofen") {
        swapped = true;
    }
    if swapped {
        said.push_str(&PREGNANT_TABLETS_NOTE);
    }
    Some(said)
}

/// Toothache: painkillers, salt water, and the dentist.
pub fn _toothache_help(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    tablets_for_a_pregnancy(Some(_TOOTHACHE_PLAIN.clone()), understood)
}

/// A headache: without tablets, in pregnancy, or the plain answer.
pub fn headache_help(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    if any_in(&t, &NO_DRUGS) {
        return Some(HEADACHE_NO_DRUGS.clone());
    }
    if _PREGNANT_AT_ALL.is_match(&t) && !_PREGNANT_DENIED.is_match(&t) {
        return Some(HEADACHE_PREGNANT.clone());
    }
    Some(HEADACHE_PLAIN.clone())
}

/// True unless the eye in the line is an animal's.
pub fn _a_persons_eye(understood: &Understood) -> bool {
    let said = &understood.clean;
    if _A_PERSONS_OWN_EYE.is_match(said) {
        return true;
    }
    if !_AN_EYE_ON_AN_ANIMAL.is_match(said) {
        return true;
    }
    !_AN_ANIMALS_OWN_EYE.is_match(said)
}

/// Whether the line says eye, eyelid or lash.
pub fn _the_eye_is_said(understood: &Understood) -> bool {
    let said = format!("{} {} {}", understood.clean, understood.text, understood.norm_text).to_lowercase();
    ["eye", "eyes", "eyelid", "eyelids", "lash", "lashes"].iter().any(|w| said.contains(w))
}

/// The pregnancy verdict on an anti-inflammatory the line names, or "".
pub fn _the_pregnancy_verdict(understood: &Understood) -> String {
    let words = words_set(understood);
    let named = ["ibuprofen", "aspirin", "nurofen", "voltarol", "naproxen", "anti-inflammatory", "anti-inflammatories"];
    if !hits(&words, &named) {
        return String::new();
    }
    THE_PREGNANCY_VERDICT_0.to_string()
}

/// Painkillers in pregnancy; a breastfeeding-only line is another row's.
pub fn _medicine_when_pregnant_or_feeding(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let words = words_set(reading);
    let feeding = hits(&words, &["breastfeeding", "breast", "feeding", "nursing", "chestfeeding"]);
    let pregnant = hits(&words, &["pregnant", "pregnancy", "expecting", "trimester", "weeks"]);
    if feeding && !pregnant {
        return None;
    }
    Some(MEDICINE_WHEN_PREGNANT_OR_FEEDING_0.to_string())
}

/// A prosthetic limb: giving way, sore, or cleaning.
pub fn _prosthesis_care(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = format!("{} {} {}", reading.text, reading.norm_text, reading.clean);
    if _PROSTHESIS_GIVING_WAY.is_match(&said) {
        return Some(_PROSTHESIS_GIVES_WAY.clone());
    }
    if _PROSTHESIS_HURTS.is_match(&said) {
        return Some(_PROSTHESIS_SORE.clone());
    }
    if _PROSTHESIS_CLEANING.is_match(&said) {
        return Some(_PROSTHESIS_CLEAN.clone());
    }
    Some(_PROSTHESIS_SORE.clone())
}

/// A glass eye: cleaning it, and the clinic first when the socket looks infected.
pub fn _glass_eye_out(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = format!(" {} ", format!("{} {} {}", reading.text, reading.norm_text, reading.clean).to_lowercase());
    if any_in(&said, &_GLASS_EYE_URGENT) {
        return Some(format!("{} {}", *_GLASS_EYE_CLINIC, *_GLASS_EYE_CLEAN));
    }
    Some(_GLASS_EYE_CLEAN.clone())
}

/// What the colour of poo means: black, pale, blood with warning signs, or blood.
pub fn _poo_colour(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = or(&understood.norm_text, &understood.clean).to_lowercase();
    if POO_BLACK.is_match(&t) {
        return Some(POO_COLOUR_1.into());
    }
    if POO_PALE.is_match(&t) {
        return Some(POO_COLOUR_2.into());
    }
    if POO_URGENT.is_match(&t) {
        return Some(POO_COLOUR_4.into());
    }
    Some(POO_COLOUR_0.into())
}

/// A sore eye: an allergy's itch, a stye, or conjunctivitis.
pub fn _eye_reply(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let clean = reading.clean.to_lowercase();
    let words_here: HashSet<String> = LETTERS.findall_str(&clean).into_iter().collect();
    let lid = ["stye", "sty", "styes", "eyelid", "eyelids", "lash", "lashes", "lid"];
    if ["itchy", "itch", "itches", "itching", "watery", "streaming"].iter().any(|w| clean.contains(w))
        && !["sticky", "crusty", "gunge", "gritty", "pus", "discharge", "glued", "infection"].iter().any(|w| clean.contains(w))
        && !hits(&words_here, &lid)
    {
        return Some(EYE_REPLY_1.into());
    }
    if hits(&words_here, &lid) {
        return Some(EYE_REPLY_2.into());
    }
    Some(EYE_REPLY_0.into())
}

/// Tablets and food: antibiotics, paracetamol, or the ones that want food.
pub fn _medicine_with_food(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let t = reading.clean.to_lowercase();
    if t.contains("antibiotic") {
        return Some(STOMACH_ANTIBIOTIC.clone());
    }
    let harsh = ["ibuprofen", "nurofen", "aspirin", "naproxen", "diclofenac", "steroid", "co-codamol", "painkiller"];
    if t.contains("paracetamol") && !harsh.iter().any(|w| t.contains(w)) {
        return Some(STOMACH_PARACETAMOL.clone());
    }
    Some(STOMACH_HARD.clone())
}

/// Shaky hands, or shaking all over.
pub fn shaky_hands_help(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    let words: HashSet<String> = LETTERS_APOS.findall_str(&t).into_iter().collect();
    Some(if hits(&words, &SHAKE_HANDS) { SHAKE_HANDS_TEXT.clone() } else { SHAKE_ALL_OVER.clone() })
}

/// A blood pressure reading read out: low, normal, a little high, or high.
pub fn bp_help(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let Some(found) = BP_FIGURES.search(&clean) else {
        if ["normal", "fine", "perfect", "healthy"].iter().any(|w| clean.contains(w)) {
            return Some(BP_NORMAL.clone());
        }
        return Some(BP_HIGH.clone());
    };
    let top: i64 = found.str(1).parse().expect("digits");
    let bottom: i64 = found.str(2).parse().expect("digits");
    let figures = format!("{top} over {bottom}");
    if top < 90 || bottom < 60 || (top <= 95 && bottom <= 65) {
        return Some(format!("{figures}{BP_HELP_1}"));
    }
    if top >= 140 || bottom >= 90 {
        return Some(BP_HIGH.clone());
    }
    if top >= 135 || bottom >= 85 {
        return Some(format!("{figures}{BP_HELP_2}"));
    }
    Some(format!("{figures}{BP_HELP_0}"))
}

/// Side effects of a new tablet (not a tablet computer).
pub fn _tablet_side_effects(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    let words: HashSet<String> = LETTERS_APOS.findall_str(&t).into_iter().collect();
    if words.iter().any(|w| TABLET_DEVICE.contains(w)) {
        return None;
    }
    Some(SIDE_EFFECT_TEXT.clone())
}

/// Whether a named medicine sits within five words of a "not helping" phrase.
pub fn _a_medicine_that_has_not_helped(toks: &[String]) -> bool {
    let marks: Vec<usize> = toks.iter().enumerate().filter(|(_, w)| NAMED_MEDICINES.contains(*w)).map(|(i, _)| i).collect();
    if marks.is_empty() {
        return false;
    }
    for phrase in _HELP_WORDS.iter() {
        let span = phrase.len();
        if span > toks.len() {
            continue;
        }
        for start in 0..=toks.len() - span {
            if toks[start..start + span] != phrase[..] {
                continue;
            }
            if marks.iter().any(|m| m.abs_diff(start) <= 5) {
                return true;
            }
        }
    }
    false
}

/// True unless the line names a medicine without saying it has not helped.
pub fn _the_named_medicine_has_not_helped(understood: &Understood) -> bool {
    let said = or(&understood.clean, &understood.text);
    let toks = LETTERS_APOS.findall_str(&said.to_lowercase());
    if !toks.iter().any(|t| NAMED_MEDICINES.contains(t)) {
        return true;
    }
    _a_medicine_that_has_not_helped(&toks)
}

/// A tablet that is not helping (a tablet computer is another row's).
pub fn _tablet_not_helping(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    let toks = LETTERS_APOS.findall_str(&t);
    let words: HashSet<String> = toks.iter().cloned().collect();
    if words.iter().any(|w| TABLET_DEVICE.contains(w)) {
        return None;
    }
    if words.iter().any(|w| NAMED_MEDICINES.contains(w)) {
        return if _a_medicine_that_has_not_helped(&toks) { Some(TABLET_TEXT.clone()) } else { None };
    }
    let mut plural = hits(&words, &["tablets", "pills", "painkillers", "capsules"])
        || t.contains("on tablets")
        || t.contains("my tablets")
        || t.contains("the tablets");
    if t.contains("a day") || t.contains("per day") || t.contains("daily") {
        plural = true;
    }
    if !plural && !words.iter().any(|w| TABLET_AS_MEDICINE.contains(w)) {
        return None;
    }
    Some(TABLET_TEXT.clone())
}

/// Melatonin: in pregnancy, for a child, or for a grown-up.
pub fn _melatonin(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    let words: HashSet<String> = LETTERS_APOS.findall_str(&t).into_iter().collect();
    if hits(&words, &MELATONIN_PREGNANT) {
        return Some(MELATONIN_PREGNANT_TEXT.clone());
    }
    let age = AGE_OF_A_CHILD.search(&t);
    if age.is_some_and(|m| m.str(1).parse::<i64>().unwrap_or(99) < 18) || t.contains("months old") {
        return Some(MELATONIN_CHILD_TEXT.clone());
    }
    let grown_up = hits(&words, &GIVE_TO_A_GROWN_UP);
    if !grown_up && (hits(&words, &MELATONIN_CHILD) || any_in(&t, &GIVE_TO_SOMEONE)) {
        return Some(MELATONIN_CHILD_TEXT.clone());
    }
    Some(MELATONIN_TEXT.clone())
}

/// A number from the reading in Celsius (Fahrenheit converted), or None when it has no value.
pub fn _in_celsius(number: &Number) -> Option<Num> {
    let value = number.value.as_ref()?;
    let unit = number.unit.as_deref().unwrap_or("").to_lowercase();
    if matches!(unit.as_str(), "fahrenheit" | "f" | "degrees f" | "deg f") || value.as_f64() >= 45.0 {
        return Some(Num::Float((value.as_f64() - 32.0) * 5.0 / 9.0));
    }
    Some(value.clone())
}

/// Python's `round(x, 1)` (correctly rounded, half to even on the exact value).
fn round1(x: f64) -> f64 {
    format!("{x:.1}").parse().unwrap_or(x)
}

/// A temperature as it is said back: "38C" or "37.5C".
pub fn _a_reading(value: &Num) -> String {
    match value {
        Num::Int(i) => format!("{i}C"),
        Num::Float(f) => {
            let rounded = round1(*f);
            if rounded == rounded.trunc() {
                format!("{}C", rounded.trunc() as i64)
            } else {
                format!("{rounded:.1}C")
            }
        }
    }
}

/// What a body temperature reading means.
pub fn _the_number_says(value: &Num) -> String {
    let said = _a_reading(value);
    let v = value.as_f64();
    if v >= *FEVER_FROM {
        if (v - *FEVER_FROM).abs() < 0.05 {
            return format!("{said} is a fever: 100.4F is the point where one starts. {}", *FEVER_ADVICE);
        }
        return format!("{said} is a fever: 38C (100.4F) is where one starts. {}", *FEVER_ADVICE);
    }
    if v < *COLD_BELOW {
        return THE_NUMBER_SAYS_1.replacen("%s", &said, 1);
    }
    if v < *LOW_FROM {
        return THE_NUMBER_SAYS_2.replacen("%s", &said, 1);
    }
    if v >= *RAISED_FROM {
        return THE_NUMBER_SAYS_3.replacen("%s", &said, 1);
    }
    THE_NUMBER_SAYS_0.replacen("%s", &said, 1)
}

/// Whether the line is about a body temperature (a temperature word, or a number in the range).
pub fn _a_body_temperature_number(understood: &Understood) -> bool {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    if TEMPERATURE_NOUN.is_match(&said) {
        return true;
    }
    if any_in(&said, &NOT_A_BODY) {
        return false;
    }
    for number in &understood.numbers {
        if let Some(v) = _in_celsius(number) {
            let v = v.as_f64();
            if BODY_RANGE.0 <= v && v <= BODY_RANGE.1 {
                return true;
            }
        }
    }
    false
}

/// What the temperature in the line means, or the normal range when it gives none.
pub fn temperature_card(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    for number in &reading.numbers {
        if let Some(v) = _in_celsius(number) {
            return Some(_the_number_says(&v));
        }
    }
    let said = or(&reading.clean, &reading.text).to_lowercase();
    if _LOW_QUESTION.is_match(&said) {
        return Some(LOW_READING.clone());
    }
    Some(NORMAL_TEMPERATURE.clone())
}

/// A cold sore: near the eye, in pregnancy, on a baby, popping it, creams, spreading, causes,
/// the first tingle, or the plain answer.
pub fn _cold_sore(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let clean = or(&reading.clean, &reading.text).to_lowercase();
    let out = if _COLD_SORE_EYE.is_match(&clean) {
        COLD_SORE_1
    } else if _COLD_SORE_PREGNANT.is_match(&clean) {
        COLD_SORE_2
    } else if _COLD_SORE_BABY.is_match(&clean) {
        COLD_SORE_3
    } else if _COLD_SORE_POP.is_match(&clean) {
        COLD_SORE_4
    } else if _COLD_SORE_HELP.is_match(&clean) || _COLD_SORE_CREAM.is_match(&clean) {
        COLD_SORE_5
    } else if _COLD_SORE_SPREAD.is_match(&clean) {
        COLD_SORE_6
    } else if _COLD_SORE_CAUSE.is_match(&clean) {
        COLD_SORE_7
    } else if _COLD_SORE_EARLY.is_match(&clean) {
        COLD_SORE_8
    } else {
        COLD_SORE_0
    };
    Some(out.into())
}

/// Alcohol with the medicine the line names.
pub fn _drink_and_the_drug(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let words = words_set(reading);
    let groups: [(&[&str], &str); 7] = [
        (&["metronidazole", "tinidazole", "antibiotic", "antibiotics", "penicillin", "amoxicillin", "trimethoprim", "nitrofurantoin"], "antibiotics"),
        (&["codeine", "co codamol", "co-codamol", "cocodamol", "codamol"], "codeine"),
        (
            &["blood pressure", "bp", "statin", "statins", "ramipril", "amlodipine", "lisinopril", "atorvastatin", "simvastatin", "rosuvastatin", "water tablet", "water tablets", "diuretic"],
            "blood-pressure",
        ),
        (&["antihistamine", "antihistamines", "hayfever", "hay fever", "piriton", "cetirizine", "loratadine", "chlorphenamine", "fexofenadine", "allergy"], "antihistamine"),
        (&["antidepressant", "antidepressants", "sertraline", "fluoxetine", "citalopram", "escitalopram", "paroxetine", "venlafaxine", "mirtazapine", "ssri"], "antidepressant"),
        (&["paracetamol", "acetaminophen", "calpol", "panadol"], "paracetamol"),
        (&["ibuprofen", "nurofen", "brufen"], "ibuprofen"),
    ];
    for (names, key) in groups {
        if hits(&words, names) {
            return Some(DRINK_PILLS[key].clone());
        }
    }
    Some(DRINK_AND_THE_DRUG_0.into())
}

/// Whether the line names a finger infection (a whitlow).
pub fn _names_a_finger(reading: &Understood) -> bool {
    let said = format!(" {} ", NOT_LETTER_OR_SPACE.sub(" ", &reading.clean.to_lowercase()));
    WHITLOW_RES.iter().any(|rx| rx.is_match(&said))
}

/// Dehydration, or getting fluid into someone who is being sick.
pub fn dehydration_help(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    if any_in(&t, &KEEPING_FLUID_DOWN) {
        return Some(DEHYDRATION_SICK.clone());
    }
    Some(DEHYDRATION_PLAIN.clone())
}

/// True unless the line is about a warm-up before a session (they cannot warm up is still ours).
pub fn _not_on_the_warm_up(understood: &Understood) -> bool {
    let said = _the_line(understood);
    if _CANNOT_WARM.is_match(&said) {
        return true;
    }
    !_A_SESSION.is_match(&said)
}

/// Whether "stitch" in the line is a wound's stitches rather than a runner's cramp.
pub fn _a_wound_not_a_cramp(understood: &Understood) -> bool {
    let mut words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if words.is_empty() {
        words = text::split(&understood.clean);
    }
    if words.iter().any(|w| _RUNNING_WORDS.contains(w)) {
        return false;
    }
    let said: HashSet<String> = words.into_iter().collect();
    if hits(&said, &["stitches", "sutures", "suture", "glued", "glue", "staple", "staples"]) {
        return true;
    }
    hits(&said, &["cut", "cuts", "wound", "wounds", "gash", "slashed", "bleeding"]) && said.contains("stitch")
}

/// True unless the line is about someone else's symptoms (signs-of asks stay ours).
pub fn _is_the_readers_own_line(understood: &Understood) -> bool {
    let said = &understood.clean;
    if _SIGNS_ASK.is_match(said) {
        return true;
    }
    !_SOMEONE_ELSE.is_match(said)
}

/// The morning-after pill: the sentence the line asks about first, then the ones it needs.
pub fn _morning_after(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let t = or(&reading.clean, &reading.text);
    // the sentences, by name, so identity (`is`) is the name
    let lead: Option<&str> = if _MORNING_AFTER_OLD.is_match(t) {
        Some("AGE")
    } else if _MORNING_AFTER_SICKS.is_match(t) {
        Some("SICK")
    } else if _MORNING_AFTER_OVULATION.is_match(t) {
        Some("OVULATED")
    } else if _MORNING_AFTER_HURTS.is_match(t) && COIL.is_match(t) {
        Some("FITTED_HURT")
    } else if _MORNING_AFTER_FITTED.is_match(t) && COIL.is_match(t) {
        Some("FITTED")
    } else if _MORNING_AFTER_LEVO.is_match(t) && _MORNING_AFTER_ULI.is_match(t) {
        Some("EFFECT")
    } else if _MORNING_AFTER_COILS.is_match(t) {
        Some("COIL")
    } else if _MORNING_AFTER_EFFECTIVE.is_match(t) {
        Some("EFFECT")
    } else if _MORNING_AFTER_FREEISH.is_match(t) || _MORNING_AFTER_WHERE.is_match(t) {
        Some("FREE")
    } else if _MORNING_AFTER_LATE.is_match(t) {
        Some("CLOCK")
    } else {
        None
    };
    let (rest, limit): (Vec<&str>, usize) = match lead {
        None => (vec!["CLOCK", "FREE", "COIL", "PREG"], 4),
        Some("EFFECT") => (vec!["EFFECT", "FREE", "PREG"], 3),
        Some("FITTED_HURT") => (vec!["FITTED_HURT", "FITTED", "CLOCK"], 3),
        Some("FITTED") => (vec!["FITTED", "FITTED_HURT", "CLOCK"], 3),
        Some("OVULATED") => (vec!["OVULATED", "COIL", "CLOCK"], 3),
        Some(lead) => {
            let base: Vec<&str> = if lead == "COIL" { vec!["CLOCK", "FREE", "PREG"] } else { vec!["CLOCK", "FREE", "COIL", "PREG"] };
            let mut r = vec![lead];
            r.extend(base.into_iter().filter(|s| *s != lead));
            (r, 3)
        }
    };
    let sentence = |n: &str| -> String {
        match n {
            "AGE" => MORNING_AFTER_AGE.clone(),
            "SICK" => MORNING_AFTER_SICK.clone(),
            "OVULATED" => MORNING_AFTER_OVULATED.clone(),
            "FITTED_HURT" => MORNING_AFTER_FITTED_HURT.clone(),
            "FITTED" => MORNING_AFTER_FITTED.clone(),
            "EFFECT" => MORNING_AFTER_EFFECT.clone(),
            "COIL" => MORNING_AFTER_COIL.clone(),
            "FREE" => MORNING_AFTER_FREE.clone(),
            "CLOCK" => MORNING_AFTER_CLOCK.clone(),
            _ => MORNING_AFTER_PREG.clone(),
        }
    };
    let mut said: Vec<String> = Vec::new();
    for n in rest {
        let one = sentence(n);
        if said.contains(&one) {
            continue;
        }
        said.push(one);
        if said.len() == limit {
            break;
        }
    }
    Some(said.join(" "))
}

/// Whether the text gives a blood sugar of 15 or more.
pub fn _a_high_reading(words: &str) -> bool {
    for m in _MMOL.finditer(&words.to_lowercase()) {
        if let Ok(v) = m.str(1).parse::<f64>() {
            if v >= 15.0 {
                return true;
            }
        }
    }
    false
}

/// True unless the line asks how to raise it with the doctor.
pub fn _not_a_line_about_how_to_tell_the_doctor(understood: &Understood) -> bool {
    let said = or(&understood.text, &understood.clean);
    !(_NAMES_A_DOCTOR.is_match(said) && _SAYS_IT_OUT_LOUD.is_match(said))
}

/// Whether a numb face is the dentist's anaesthetic (and names no stroke sign).
pub fn _the_dentist_numbed_it(understood: &Understood) -> bool {
    let said = or(&understood.text, &understood.clean);
    if _NAMES_A_STROKE_SIGN.is_match(said) {
        return false;
    }
    _NAMES_A_DENTAL_CAUSE.is_match(said) && _SAYS_NUMB.is_match(said)
}

/// Whether "turned me down" is the news, with no word of how it left them.
pub fn _a_refusal_is_not_a_mood(understood: &Understood) -> bool {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    _TURNED_DOWN.is_match(&said) && !_HOW_IT_LEFT_THEM.is_match(&said)
}

/// True unless the line is about a cold or damp home.
pub fn not_a_cold_or_damp_house(understood: &Understood) -> bool {
    let said = or(&understood.clean, &understood.text);
    !(_A_HOME_WORD.is_match(said) && _A_COLD_OR_DAMP_WORD.is_match(said))
}

/// Whether "low", "down", "flat" or "blue" in the line is the person's mood, not a thing's.
pub fn _the_mood_is_the_persons(understood: &Understood) -> bool {
    let line = _the_line(understood);
    let words: Vec<&str> = text::split_ref(&line);
    if words.is_empty() {
        return true;
    }
    let isin = |list: &[String], w: &str| list.iter().any(|x| x == w);
    let bare: Vec<&str> = words.iter().copied().filter(|w| !isin(&_MOOD_THE_NOUN, w)).collect();
    if bare.contains(&"flat") && !bare.iter().any(|w| _MOOD_FEELING_SAID.contains(*w)) {
        let said = bare.join(" ");
        if any_in(&said, &_MOOD_A_HOME_FLAT) {
            return false;
        }
    }
    let n = bare.len() as isize;
    let at_ = |i: isize| -> &str { bare[i as usize] };
    let mut thing_said = false;
    let mut person_said = false;
    for (i, word) in bare.iter().enumerate() {
        if !isin(&_MOOD_WORDS, word) {
            continue;
        }
        let i = i as isize;
        let mut j = i - 1;
        while j >= 0 && isin(&_MOOD_ADVERB, at_(j)) {
            j -= 1;
        }
        let mut found: Option<isize> = None;
        let stop = (j - 3).max(-1);
        let mut k = j;
        while k > stop {
            if isin(&_MOOD_THE_STATE, at_(k)) {
                found = Some(k);
                break;
            }
            k -= 1;
        }
        let Some(found) = found else {
            let nxt = if i + 1 < n { at_(i + 1) } else { "" };
            if _MOOD_JOIN.contains(nxt) {
                continue;
            }
            if !nxt.is_empty() && !_MOOD_TIME.contains(nxt) && !isin(&_MOOD_SAID_OF_A_PERSON, nxt) {
                thing_said = true;
            }
            continue;
        };
        let at: isize;
        if found == j {
            let mut k = found - 1;
            while k >= 0 && _MOOD_HELPERS.contains(at_(k)) {
                k -= 1;
            }
            at = k;
        } else {
            let between = &bare[(found + 1) as usize..i as usize];
            let mut kept = between.len();
            while kept > 0 && isin(&_MOOD_ADVERB, between[kept - 1]) {
                kept -= 1;
            }
            if between[..kept].iter().any(|w| _MOOD_JOIN.contains(*w)) {
                person_said = true;
                continue;
            }
            if between[..kept].iter().any(|w| isin(&_MOOD_SAID_OF_A_PERSON, w)) {
                person_said = true;
                continue;
            }
            if between.iter().any(|w| isin(&_MOOD_ADVERB, w)) {
                let mut k = found - 1;
                while k >= 0 && _MOOD_HELPERS.contains(at_(k)) {
                    k -= 1;
                }
                at = k;
            } else {
                at = found + 1;
            }
        }
        let subject = if 0 <= at && at < n { at_(at) } else { "" };
        if at > 0 && _MOOD_WHERE_THE_PERSON_WAS.contains(at_(at - 1)) {
            person_said = true;
            continue;
        }
        if isin(&_MOOD_SAID_OF_A_PERSON, subject)
            || subject.is_empty()
            || _MOOD_THE_VERB_BEFORE.contains(subject)
            || _MOOD_JOIN.contains(subject)
            || isin(&_MOOD_WORDS, subject)
        {
            person_said = true;
            continue;
        }
        thing_said = true;
    }
    person_said || !thing_said
}

/// Whether the line is about a baby (or a birth).
pub fn _about_a_baby(understood: &Understood) -> bool {
    let mut t = or(&understood.text, &understood.clean).to_string();
    if t.is_empty() {
        t = understood.words.join(" ");
    }
    _A_BABY_IN_THE_LINE.is_match(&t)
}

/// The baby blues, or postnatal depression when it has gone on.
pub fn _postnatal_mood_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = format!(" {} ", understood.clean.to_lowercase());
    if any_in(&clean, &_BLUES_SAID) && !clean.contains("depression") && !clean.contains("weeks") && !clean.contains("months") {
        return Some(POSTNATAL_MOOD_REPLY_1.into());
    }
    Some(POSTNATAL_MOOD_REPLY_0.into())
}

/// True unless the line is about getting up to the loo in the night (a stomach is still ours).
pub fn _not_a_night_of_going(understood: &Understood) -> bool {
    let t = format!("{} {} {} ", understood.clean, understood.text, understood.norm_text).to_lowercase();
    if _A_STOMACH_SAID.is_match(&t) {
        return true;
    }
    !(_A_NIGHT.is_match(&t) && _UP_TO_THE_LOO.is_match(&t))
}

/// True unless the appetite in the line is somebody else's.
pub fn _my_own_appetite(understood: &Understood) -> bool {
    !_SOMEBODY_NAMED.is_match(or(&understood.clean, &understood.text))
}

/// The pattern `_not_a_denial_of(*subject)` builds: "not diabetic", "never had asthma".
pub fn _not_a_denial_of_pattern(subject: &[&str]) -> Re {
    let mut sorted: Vec<&str> = subject.to_vec();
    sorted.sort_by_key(|w| std::cmp::Reverse(w.chars().count()));
    let words: Vec<String> = sorted.iter().map(|w| text::split(w).join(r"\s+")).collect();
    let pattern = _NOT_A_CONDITION.pattern().replacen("%s", &words.join("|"), 1);
    Re::new(&pattern, crate::py::re::U)
}

/// `_not_a_denial_of(*subject).<locals>.guard`: true unless the line denies the condition.
pub fn _not_a_denial_of_guard(pattern: &Re, understood: &Understood) -> bool {
    let mut said = or(&understood.text, &understood.clean).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    !pattern.is_match(&said.to_lowercase())
}

/// True unless "piles" in the line is a heap ("piling up").
pub fn _a_heap_not_the_condition(understood: &Understood) -> bool {
    let clean = format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase();
    !_THE_WORD_IS_A_HEAP.is_match(&clean)
}

fn split_apos(said: &str) -> Vec<String> {
    text::split(&said.replace('\'', " "))
}

/// A heart rate reading, or what BPM / HR means.
pub fn _a_rate_reading(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = or(&reading.text, &reading.clean).to_lowercase();
    let device = ["watch", "fitbit", "garmin", "tracker", "smartwatch", "smart watch", "band", " app"].iter().any(|w| said.contains(w));
    let mut given: Option<i64> = None;
    for number in &reading.numbers {
        if let Some(v) = &number.value {
            let f = v.as_f64();
            if (25.0..=250.0).contains(&f) {
                given = Some(match v {
                    Num::Int(i) => *i,
                    Num::Float(x) => x.trunc() as i64,
                });
                break;
            }
        }
    }
    let apos = split_apos(&said);
    let kids = [
        "kid", "kids", "son", "sons", "daughter", "daughters", "boy", "boys", "girl", "girls", "teen", "teens",
        "teenager", "teenagers", "children", "child", "baby",
    ];
    if kids.iter().any(|w| apos.iter().any(|a| a == w)) {
        return Some(A_RATE_READING_0.into());
    }
    let letters_asked = ["mean", "means", "stand for", "stands for", "what does", "whats", "short for", "abbreviation", "spell"]
        .iter()
        .any(|p| said.contains(p));
    let spread = RATE_SPREAD.search(&said);
    let too_slow = given.is_some_and(|g| g < 60);
    let plain_words = text::split(&said);
    let opening: String = if letters_asked && (said.contains("bpm") || said.contains("beats per minute")) {
        A_RATE_READING_1.into()
    } else if letters_asked && (plain_words.iter().any(|w| w == "hr") || apos.iter().any(|w| w == "hr")) {
        A_RATE_READING_3.into()
    } else if letters_asked && device {
        A_RATE_READING_5.into()
    } else if let Some(sp) = &spread {
        let low: i64 = sp.str(1).parse().expect("digits");
        let high: i64 = sp.str(2).parse().expect("digits");
        A_RATE_READING_6.replacen("%d", &low.to_string(), 1).replacen("%d", &high.to_string(), 1)
    } else if too_slow {
        A_RATE_READING_7.replacen("%d", &given.unwrap().to_string(), 1)
    } else if given.is_some() && device {
        A_RATE_READING_8.replacen("%d", &given.unwrap().to_string(), 1)
    } else if let Some(g) = given {
        A_RATE_READING_9.replacen("%d", &g.to_string(), 1)
    } else if ["high", "higher", "highs", "elevated", "raised"].iter().any(|w| apos.iter().any(|a| a == w)) {
        A_RATE_READING_10.into()
    } else if ["low", "lower", "slow"].iter().any(|w| plain_words.iter().any(|a| a == w)) {
        A_RATE_READING_11.into()
    } else {
        A_RATE_READING_12.into()
    };
    if too_slow && spread.is_none() && !letters_asked {
        return Some(opening + A_RATE_READING_4);
    }
    Some(opening + A_RATE_READING_2)
}

/// True unless the line asks what "high" or "low" means for a rate.
pub fn _not_the_meaning_of_high_or_low(understood: &Understood) -> bool {
    !(_MEANS.is_match(&understood.clean) && understood.words.iter().any(|w| _A_RATE_NAMED_HIGH_OR_LOW.contains(w)))
}

/// True unless the pressure in the line is water in a pipe (blood words keep it ours).
pub fn _not_a_pressure_in_a_pipe(understood: &Understood) -> bool {
    let mut said = understood.words.join(" ").to_lowercase();
    if said.is_empty() {
        said = or(&understood.clean, &understood.text).to_lowercase();
    }
    if said.is_empty() {
        return true;
    }
    if any_in(&said, &_BLOOD_WORDS) {
        return true;
    }
    !any_in(&said, &_A_PRESSURE_IN_A_PIPE)
}

/// An antacid with another tablet: the gap some of them need.
pub fn _an_antacid_with_another_tablet(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = text::squash(&reading.text).to_lowercase();
    if any_in(&line, &_ANTACID_MIND_THE_GAP) {
        return Some(AN_ANTACID_WITH_ANOTHER_TABLET_1.into());
    }
    Some(AN_ANTACID_WITH_ANOTHER_TABLET_0.into())
}

/// A painkiller with an antidepressant: codeine, paracetamol, or the general answer.
pub fn _a_painkiller_with_an_antidepressant(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = text::squash(&reading.text).to_lowercase();
    if ["codeine", "co-codamol", "co codamol", "tramadol"].iter().any(|w| line.contains(w)) {
        return Some(A_PAINKILLER_WITH_AN_ANTIDEPRESSANT_1.into());
    }
    if ["paracetamol", "painkiller", "painkillers", "pain relief", "headache"].iter().any(|w| line.contains(w)) {
        return Some(A_PAINKILLER_WITH_AN_ANTIDEPRESSANT_2.into());
    }
    Some(A_PAINKILLER_WITH_AN_ANTIDEPRESSANT_0.into())
}

/// Jaundice in a newborn or in an adult.
pub fn _jaundice_card(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = or(&understood.text, or(&understood.clean, &understood.norm_text)).to_lowercase();
    if ["baby", "babies", "newborn", "newborns", "infant", "infants"].iter().any(|w| t.contains(w)) {
        return Some(_JAUNDICE_NEWBORN.clone());
    }
    Some(_JAUNDICE_ADULT.clone())
}

/// Whether the yellow in the line is on the skin or the eyes (not in the loo, not an eye infection).
pub fn _the_colour_is_on_the_skin(understood: &Understood) -> bool {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    if _WASTE.is_match(&said) && !SKIN_OR_EYES.is_match(&said) {
        return false;
    }
    if _EYE_TROUBLE.is_match(&said) && !_SKIN_ITSELF.is_match(&said) {
        return false;
    }
    _JAUNDICE_SKIN.is_match(&said) || _CHANGE_SUBJECT.is_match(&said)
}

/// A food with a medicine: grapefruit, cranberry, dairy, greens, or the general answer.
pub fn _a_food_and_the_drug(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let words = words_set(reading);
    let out = if hits(&words, &_GRAPEFRUIT) {
        if hits(&words, &_THINNER) {
            A_FOOD_AND_THE_DRUG_5
        } else if hits(&words, &_STATIN) {
            A_FOOD_AND_THE_DRUG_6
        } else {
            A_FOOD_AND_THE_DRUG_1
        }
    } else if hits(&words, &_CRANBERRY) {
        A_FOOD_AND_THE_DRUG_2
    } else if hits(&words, &_DAIRY) {
        if hits(&words, &_THYROID_PILL) {
            A_FOOD_AND_THE_DRUG_7
        } else if hits(&words, &_IRON_PILL) {
            A_FOOD_AND_THE_DRUG_8
        } else if hits(&words, &_TETRACYCLINE) {
            A_FOOD_AND_THE_DRUG_9
        } else {
            A_FOOD_AND_THE_DRUG_3
        }
    } else if hits(&words, &_GREENS) {
        A_FOOD_AND_THE_DRUG_4
    } else {
        A_FOOD_AND_THE_DRUG_0
    };
    Some(out.into())
}

/// Whether the line names a second medicine (or an interaction).
pub fn _another_medicine_is_named(line: &str) -> bool {
    if _INTERACTION_WORD.is_match(line) {
        return true;
    }
    let medicines = _MEDICINE_WORDS.iter().filter(|w| line.contains(w.as_str())).count();
    if medicines >= 2 {
        return true;
    }
    medicines > 0 && any_in(line, &_MORE_THAN_ONE)
}

/// A statin: with other tablets, the time of day, or the first weeks.
pub fn _a_statin_and_the_day(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _the_line(understood);
    if _another_medicine_is_named(&line) {
        return Some(STATIN_WITH_OTHER_TABLETS.clone());
    }
    if ["night", "evening", "morning", "what time", "when", "same time", "best time", "time of day", "together"]
        .iter()
        .any(|w| line.contains(w))
    {
        return Some(STATIN_TIMING.clone());
    }
    if _STATIN_WHETHER.is_match(&line) && !any_in(&line, &_STATIN_STARTED) {
        return None;
    }
    Some(STATIN_FIRST_WEEKS.clone())
}

/// Whether the line the reading leans on named a statin.
// TYPE(understood.leans_on): Python reads a dict (its "words" / "text"); the reading here holds
// an Option<String>, which Python's `isinstance(lean, dict)` turns away, and None reads as {}.
pub fn _the_last_line_named_a_statin(understood: &Understood) -> bool {
    // None reads as {} (nothing said), and a string is not a dict: either way no statin is named.
    let _ = (&understood.leans_on, &*_STATIN_NAMES);
    false
}

/// What an MRI, a CT, an ultrasound or an X-ray is.
pub fn _a_scan_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = understood.clean.to_lowercase();
    if SCAN_MRI_SAID.is_match(&t) {
        return Some(SCAN_MRI.clone());
    }
    if SCAN_CT_SAID.is_match(&t) {
        return Some(SCAN_CT.clone());
    }
    if t.contains("ultrasound") || t.contains("sonogram") {
        return Some(SCAN_ULTRASOUND.clone());
    }
    Some(SCAN_ANY.clone())
}

/// After a clear result: the thing they ask permission for.
pub fn _the_permission_for(t: &str) -> String {
    if _A_STOPPING_A_MEDICINE.is_match(t) {
        return AFTER_A_CLEAR_RESULT_MEDICINE.clone();
    }
    if _A_DRIVING_AGAIN.is_match(t) {
        return AFTER_A_CLEAR_RESULT_DRIVING.clone();
    }
    if _A_SPORT_AGAIN.is_match(t) {
        return AFTER_A_CLEAR_RESULT_EXERCISE.clone();
    }
    if _A_JOB_AGAIN.is_match(t) {
        return AFTER_A_CLEAR_RESULT_WORK.clone();
    }
    AFTER_A_CLEAR_RESULT_TEXT.clone()
}

/// The last bot reply among the last two turns, lower-cased ("" when none).
fn last_bot_of_two(memory: &Memory) -> String {
    let start = memory.turns.len().saturating_sub(2);
    let mut last = String::new();
    for turn in &memory.turns[start..] {
        if !turn.bot.is_empty() {
            last = turn.bot.to_lowercase();
        }
    }
    last
}

/// "Can I go back to X?" after the last reply said the result was clear.
pub fn _after_a_clear_result(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = or(&understood.text, &understood.clean).to_lowercase();
    if !_A_PERMISSION_ASK.is_match(&t) || !_SOMETHING_TO_GO_BACK_TO.is_match(&t) {
        return None;
    }
    if !_RESULT_SAYS_CLEAR.is_match(&last_bot_of_two(memory)) {
        return None;
    }
    Some(_the_permission_for(&t))
}

/// "So that's good then?" after the last reply said the result was clear.
pub fn _a_check_on_a_clear_result(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = or(&understood.text, &understood.clean).to_lowercase();
    let t = text::strip(&t);
    if t.is_empty() || text::split_ref(t).len() > 6 || _A_CHECKING_THE_RESULT.match_(t).is_none() {
        return None;
    }
    if !_RESULT_SAYS_CLEAR.is_match(&last_bot_of_two(memory)) {
        return None;
    }
    Some(A_CHECK_ON_A_CLEAR_RESULT_0.into())
}

/// A test result: clear, negative, abnormal, a follow-up, or chasing it.
pub fn _a_result_reply(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let t = or(&understood.text, &understood.clean).to_lowercase();
    let pet = VET.is_match(&t);
    if _RULES_IT_OUT.is_match(&t) {
        return Some(if pet { CLEAR_RESULT_FOR_A_PET_TEXT.clone() } else { CLEAR_RESULT_TEXT.clone() });
    }
    if _A_TEST_NAME.is_match(&t) {
        if RESULT_CLEAR_SAID.is_match(&t) {
            return Some(if pet { CLEAR_RESULT_FOR_A_PET_TEXT.clone() } else { CLEAR_RESULT_TEXT.clone() });
        }
        if RESULT_NEGATIVE_SAID.is_match(&t) {
            return Some(NEGATIVE_RESULT_TEXT.clone());
        }
        if RESULT_ABNORMAL_SAID.is_match(&t) {
            return Some(if pet { ABNORMAL_RESULT_FOR_A_PET_TEXT.clone() } else { ABNORMAL_RESULT_TEXT.clone() });
        }
    }
    let last = last_bot_of_two(memory);
    if _RESULT_SAYS_ABNORMAL.is_match(&last) {
        return Some(A_RESULT_REPLY_1.into());
    }
    if _RESULT_SAYS_CLEAR.is_match(&last) {
        return Some(A_RESULT_REPLY_2.into());
    }
    if RESULT_FOLLOW_UP.is_match(&t) {
        return Some(ABNORMAL_RESULT_TEXT.clone());
    }
    if RESULT_CHASE.is_match(&t) {
        return Some(CHASE_RESULT_TEXT.clone());
    }
    if RESULT_ANY.is_match(&t) {
        return Some(ABNORMAL_RESULT_TEXT.clone());
    }
    Some(A_RESULT_REPLY_0.into())
}

/// A blood test: what it is, or how long to fast before one.
pub fn _a_blood_test(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = text::squash(or(&understood.clean, &understood.text));
    if _A_FASTING_LENGTH.is_match(&said) && !_A_TEST_THAT_TAKES.is_match(&said) {
        return Some(_A_FASTING_BEFORE_A_TEST_TEXT.clone());
    }
    Some(_A_BLOOD_TEST_TEXT.clone())
}

/// Cataracts: how long the operation takes, or what one is.
pub fn _the_cataract_answer(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.clean, &understood.text).to_lowercase();
    let asks_a_length = said.contains("how long") || said.contains("how many minutes") || said.contains("how soon");
    let is_the_operation = ["operation", "surgery", "operate", "procedure", "the op"].iter().any(|w| said.contains(w));
    let is_the_wait = said.contains("wait");
    let is_the_recovery = [
        "recover", "recovery", "heal", "healing", "back to work", "back to normal", "off work", "drive again", "after the op",
    ]
    .iter()
    .any(|w| said.contains(w));
    if asks_a_length && is_the_operation && !(is_the_wait || is_the_recovery) {
        return Some(_CATARACT_LENGTH.clone());
    }
    Some(_CATARACT_WHAT.clone())
}

/// True unless the line opens "is she ...?" (whether someone has it).
pub fn _not_a_yes_no_question(understood: &Understood) -> bool {
    _ASKS_WHETHER_SOMEONE_HAS_IT.match_(&understood.clean.to_lowercase()).is_none()
}

/// True unless "gasping for" a thing (a drink) is the line.
pub fn _a_gasp_for_a_thing_is_a_thirst(understood: &Understood) -> bool {
    !_A_GASP_FOR_A_THING.is_match(&understood.clean.to_lowercase())
}

/// A grown-up's lingering cough, when the line names a grown relation.
pub fn _grown_cough(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let joined = format!("{} {}", understood.clean, understood.words.join(" ")).to_lowercase();
    if !["cough", "coughing", "coughed"].iter().any(|w| joined.contains(w)) {
        return None;
    }
    if any_in(&joined, &_CHILD_WORD) {
        return None;
    }
    for age in _AN_AGE.findall_str(&joined) {
        if age.parse::<i64>().expect("digits") < 18 {
            return None;
        }
    }
    if any_in(&joined, &_GROWN_RELATION) {
        return Some(ADULT_COUGH_REPLY.clone());
    }
    None
}

/// Whether the line says the pet has the symptom (not that it sets the person off).
pub fn _the_pet_has_the_symptom(understood: &Understood) -> bool {
    let t = understood.clean.to_lowercase();
    let Some(start) = _PET_AS_SUBJECT.match_(&t) else { return false };
    let rest = &t[start.end()..];
    let Some(symptom) = _PET_SYMPTOM.search(rest) else { return false };
    !_THE_PERSON_IS_REACTING.is_match(&rest[..symptom.start()])
}

/// Whether the child in the line is younger than a teenager (and not a named grown-up).
pub fn _a_younger_child(understood: &Understood) -> bool {
    let said = text::squash(&understood.clean.to_lowercase());
    if _TEEN_SAID.is_match(&said) || _ADULT_AGE.is_match(&said) {
        return false;
    }
    !_ADULT_NAMED.is_match(&said)
}

/// A hernia, or a hiatus hernia.
pub fn _hernia(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = format!(" {} ", understood.clean.to_lowercase());
    if line.contains("hiatus") {
        return Some(HERNIA_0.into());
    }
    Some(_A_HERNIA.clone())
}

fn warmth_in(understood: &Understood, set: &HashSet<String>) -> bool {
    understood.warmth.as_deref().is_some_and(|w| set.contains(w))
}

fn _not_a_denial_of_cell(f: &FnRef, understood: &Understood) -> bool {
    let pattern = f.cell("pattern").and_then(Value::as_re).expect("captured pattern");
    _not_a_denial_of_guard(pattern, understood)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.health:<lambda>:2292", |_: &FnRef, u: &Understood| warmth_in(u, &COLD_WARMTH_WORDS)),
    ("bot.knowledge.health:<lambda>:2321", |_: &FnRef, u: &Understood| warmth_in(u, &HOT_WARMTH_WORDS)),
    ("bot.knowledge.health:<lambda>:5557", |_: &FnRef, u: &Understood| {
        _the_mood_is_the_persons(u) && !_a_refusal_is_not_a_mood(u) && not_a_cold_or_damp_house(u)
    }),
    ("bot.knowledge.health:<lambda>:5771", |_: &FnRef, u: &Understood| {
        _the_mood_is_the_persons(u) && _not_a_line_about_how_to_tell_the_doctor(u)
    }),
    ("bot.knowledge.health:<lambda>:5920", |_: &FnRef, u: &Understood| {
        let line = _the_line(u);
        [
            "again", "back into", "back to", "get back", "getting back", "got back", "restart", "re-start", "havent",
            "haven't", "not exercised in", "not done", "after a break", "after years", "after a long", "years off",
            "years since", "since school", "used to", "out of shape", "stopped exercising",
        ]
        .iter()
        .any(|cue| line.contains(cue))
    }),
    ("bot.knowledge.health:<lambda>:7393", |_: &FnRef, u: &Understood| !_the_dentist_numbed_it(u)),
    ("bot.knowledge.health:<lambda>:7747", |_: &FnRef, u: &Understood| !_a_high_reading(&u.clean)),
    ("bot.knowledge.health:<lambda>:10532", |_: &FnRef, u: &Understood| {
        !_a_high_reading(&format!("{} {}", u.clean, u.text)) && _not_a_denial_of_guard(&NOT_A_DENIAL_OF_DIABETES, u)
    }),
    ("bot.knowledge.health:<lambda>:11388", |_: &FnRef, u: &Understood| !_the_pet_has_the_symptom(u)),
    ("bot.knowledge.health:<lambda>:11440", |_: &FnRef, u: &Understood| {
        let t = format!(" {} ", u.text);
        _FRUIT_ITSELF.iter().any(|w| t.contains(w.as_str()))
    }),
    ("bot.knowledge.health:_a_body_temperature_number:1013", |_: &FnRef, u: &Understood| _a_body_temperature_number(u)),
    ("bot.knowledge.health:_a_gasp_for_a_thing_is_a_thirst:11111", |_: &FnRef, u: &Understood| _a_gasp_for_a_thing_is_a_thirst(u)),
    ("bot.knowledge.health:_a_heap_not_the_condition:2098", |_: &FnRef, u: &Understood| _a_heap_not_the_condition(u)),
    ("bot.knowledge.health:_a_persons_eye:297", |_: &FnRef, u: &Understood| _a_persons_eye(u)),
    ("bot.knowledge.health:_a_wound_not_a_cramp:1422", |_: &FnRef, u: &Understood| _a_wound_not_a_cramp(u)),
    ("bot.knowledge.health:_a_younger_child:11647", |_: &FnRef, u: &Understood| _a_younger_child(u)),
    ("bot.knowledge.health:_about_a_baby:1915", |_: &FnRef, u: &Understood| _about_a_baby(u)),
    ("bot.knowledge.health:_an_illness_the_card_covers:148", |_: &FnRef, u: &Understood| _an_illness_the_card_covers(u)),
    ("bot.knowledge.health:_is_the_readers_own_line:1466", |_: &FnRef, u: &Understood| _is_the_readers_own_line(u)),
    ("bot.knowledge.health:_my_own_appetite:2038", |_: &FnRef, u: &Understood| _my_own_appetite(u)),
    ("bot.knowledge.health:_names_a_finger:1228", |_: &FnRef, u: &Understood| _names_a_finger(u)),
    ("bot.knowledge.health:_not_a_denial_of.<locals>.guard:2059", _not_a_denial_of_cell),
    ("bot.knowledge.health:_not_a_night_of_going:2008", |_: &FnRef, u: &Understood| _not_a_night_of_going(u)),
    ("bot.knowledge.health:_not_a_pressure_in_a_pipe:8621", |_: &FnRef, u: &Understood| _not_a_pressure_in_a_pipe(u)),
    ("bot.knowledge.health:_not_a_yes_no_question:10585", |_: &FnRef, u: &Understood| _not_a_yes_no_question(u)),
    ("bot.knowledge.health:_not_on_the_warm_up:1414", |_: &FnRef, u: &Understood| _not_on_the_warm_up(u)),
    ("bot.knowledge.health:_not_the_meaning_of_high_or_low:8401", |_: &FnRef, u: &Understood| {
        _not_the_meaning_of_high_or_low(u)
    }),
    ("bot.knowledge.health:_the_colour_is_on_the_skin:9051", |_: &FnRef, u: &Understood| _the_colour_is_on_the_skin(u)),
    ("bot.knowledge.health:_the_dentist_numbed_it:1666", |_: &FnRef, u: &Understood| _the_dentist_numbed_it(u)),
    ("bot.knowledge.health:_the_eye_is_said:314", |_: &FnRef, u: &Understood| _the_eye_is_said(u)),
    ("bot.knowledge.health:_the_last_line_named_a_statin:9418", |_: &FnRef, u: &Understood| _the_last_line_named_a_statin(u)),
    ("bot.knowledge.health:_the_named_medicine_has_not_helped:839", |_: &FnRef, u: &Understood| {
        _the_named_medicine_has_not_helped(u)
    }),
    ("bot.knowledge.health:not_a_childs_age:122", |_: &FnRef, u: &Understood| not_a_childs_age(u)),
    ("bot.knowledge.health:not_a_season_or_a_machine:70", |_: &FnRef, u: &Understood| not_a_season_or_a_machine(u)),
    ("bot.knowledge.health:not_a_when_ask:61", |_: &FnRef, u: &Understood| not_a_when_ask(u)),
];

macro_rules! r {
    ($id:literal, $f:path) => {
        ($id, |_: &FnRef, m: &mut Memory, u: &Understood| $f(m, u))
    };
}

pub const REPLIES: &[(&str, ReplyFn)] = &[
    r!("bot.knowledge.health:_a_blood_test:9837", _a_blood_test),
    r!("bot.knowledge.health:_a_check_on_a_clear_result:9728", _a_check_on_a_clear_result),
    r!("bot.knowledge.health:_a_food_and_the_drug:9234", _a_food_and_the_drug),
    r!("bot.knowledge.health:_a_painkiller_with_an_antidepressant:8828", _a_painkiller_with_an_antidepressant),
    r!("bot.knowledge.health:_a_rate_reading:8292", _a_rate_reading),
    r!("bot.knowledge.health:_a_result_reply:9745", _a_result_reply),
    r!("bot.knowledge.health:_a_scan_reply:9554", _a_scan_reply),
    r!("bot.knowledge.health:_a_statin_and_the_day:9399", _a_statin_and_the_day),
    r!("bot.knowledge.health:_after_a_clear_result:9703", _after_a_clear_result),
    r!("bot.knowledge.health:_an_antacid_with_another_tablet:8807", _an_antacid_with_another_tablet),
    r!("bot.knowledge.health:_cold_sore:1087", _cold_sore),
    r!("bot.knowledge.health:_drink_and_the_drug:1181", _drink_and_the_drug),
    r!("bot.knowledge.health:_eye_reply:489", _eye_reply),
    r!("bot.knowledge.health:_glass_eye_out:446", _glass_eye_out),
    r!("bot.knowledge.health:_grown_cough:11270", _grown_cough),
    r!("bot.knowledge.health:_hernia:11907", _hernia),
    r!("bot.knowledge.health:_jaundice_card:9000", _jaundice_card),
    r!("bot.knowledge.health:_medicine_when_pregnant_or_feeding:345", _medicine_when_pregnant_or_feeding),
    r!("bot.knowledge.health:_medicine_with_food:548", _medicine_with_food),
    r!("bot.knowledge.health:_melatonin:878", _melatonin),
    r!("bot.knowledge.health:_morning_after:1550", _morning_after),
    r!("bot.knowledge.health:_poo_colour:460", _poo_colour),
    r!("bot.knowledge.health:_postnatal_mood_reply:1930", _postnatal_mood_reply),
    r!("bot.knowledge.health:_prosthesis_care:432", _prosthesis_care),
    r!("bot.knowledge.health:_tablet_not_helping:857", _tablet_not_helping),
    r!("bot.knowledge.health:_tablet_side_effects:805", _tablet_side_effects),
    r!("bot.knowledge.health:_the_cataract_answer:10310", _the_cataract_answer),
    r!("bot.knowledge.health:_toothache_help:264", _toothache_help),
    r!("bot.knowledge.health:bp_help:608", bp_help),
    r!("bot.knowledge.health:dehydration_help:1373", dehydration_help),
    r!("bot.knowledge.health:headache_help:268", headache_help),
    r!("bot.knowledge.health:shaky_hands_help:575", shaky_hands_help),
    r!("bot.knowledge.health:temperature_card:1044", temperature_card),
];

// ---- the reply text written inside the Python functions (verbatim)
const THE_PREGNANCY_VERDICT_0: &str = "Avoid ibuprofen and aspirin unless a doctor or midwife has told you otherwise";
const MEDICINE_WHEN_PREGNANT_OR_FEEDING_0: &str = "Paracetamol is the painkiller to reach for in pregnancy, at the lowest dose for the shortest time. Avoid ibuprofen and aspirin unless a doctor or midwife has told you otherwise, and ask a pharmacist before anything else - many cold and cough remedies have something in them you would not expect. While breastfeeding, paracetamol and ibuprofen are both fine at the normal dose; go easy on codeine, which can make a baby drowsy.";
const POO_COLOUR_0: &str = "Bright red blood on the paper or in the pan is usually piles or a small tear, and it settles in a few days with plenty of water, fibre and no straining. See the GP if it is more than a streak, keeps coming for more than a few weeks, or comes with weight loss, a change in your poo for weeks, pain or a fever: that is not piles, and black or tarry poo is 111 or A&E the same day.";
const POO_COLOUR_1: &str = "Black, tarry poo is blood from higher up the gut, and it is the same day: ring 111 or go to A&E if you are dizzy, pale, breathless or it keeps coming, and ring the GP today if you feel well in yourself. Do not take iron or bismuth for it before you are seen, because they blacken the stool too and hide the cause.";
const POO_COLOUR_2: &str = "Pale, putty-coloured poo usually means bile is not reaching the gut, so it is a GP question rather than a food one - especially with dark wee, yellow eyes or skin, an ache under the right ribs or itching, which is the same day. Otherwise make a routine appointment and mention any new medicine, because some cause it.";
const POO_COLOUR_3: &str = "\\b(faint\\w*|dizzy|light[- ]?headed|breathless|short of breath|vomit\\w*|sick|throwing up|passing out|passed out|collapsed|collapsing|shaking|clots?|pouring|loads of blood|a lot of blood|so much blood|badly|really bad|severe|unbearable)\\b";
const POO_COLOUR_4: &str = "Blood in your poo with feeling faint, dizzy, sick or in a lot of pain is not a wait and see: ring 111 now, or 999 or go to A&E if you are faint, breathless, vomiting, or the bleeding is heavy or has clots. Steady bright red on the paper alone is usually piles, but the faintness or the pain is what decides this one, so get it looked at today rather than working out the cause yourself.";
const EYE_REPLY_0: &str = "A sticky, gritty red eye with crusty lashes in the morning is usually conjunctivitis: bathe it with cooled boiled water and a fresh cotton wool ball for each eye, and stop wearing lenses until it is better. A stye is a blocked lash follicle - a warm compress for 10 minutes several times a day brings it to a head; never squeeze it. Pain, light sensitivity or blurred vision is a same-day GP or optician visit.";
const EYE_REPLY_1: &str = "An itchy eye is an allergy until proved otherwise - pollen, dust mites or a cat - and the itch is the tell, because an infection is sticky or gritty rather than itchy. Don't rub it, which makes it worse; use a cool compress and antihistamine or lubricating drops from a pharmacist, keep the windows shut in pollen season and shower before bed. An eye that is painful, that light hurts, whose sight has changed, or that is red with a contact lens in, is the same-day GP or optician.";
const EYE_REPLY_2: &str = "A stye is a blocked lash follicle: a warm compress for ten minutes, four times a day, brings it to a head in a few days, and never squeeze it. Leave lenses and eye make-up off until it has gone. A stye that is getting bigger, or an eyelid swelling shut, is a same-day GP; pain in the eye itself, light hurting or blurred vision is A&E.";
const BP_HELP_0: &str = " is a normal reading: at home, normal is anything under 135 over 85. Check it now and then at the same time of day rather than every hour, and keep off the salt if it has been high before.";
const BP_HELP_1: &str = " is a low reading, not a high one. If you feel well the usual causes are dehydration, a hot day and skipping meals: drink a glass of water, eat something and stand up slowly. See a GP if you faint or feel dizzy on standing, or if it keeps reading low.";
const BP_HELP_2: &str = " is a little above the home range, which stops at 135 over 85: it is not high enough to act on from one reading. Sit still five minutes before you take it, do it twice a day for a week at the same times and write the numbers down - if the week averages over 135 over 85, take the list to the GP.";
const THE_NUMBER_SAYS_0: &str = "%s is within the normal range of 36.1 to 37.2C, which runs half a degree higher in the evening, and a fever starts at 38C (100.4F), so there is nothing to treat. A stiff neck, a rash that does not fade under a glass, or trouble breathing with a fever is a same-day call.";
const THE_NUMBER_SAYS_1: &str = "%s is below 35C (95F), which is hypothermia territory rather than a fever: get them out of the cold and out of wet clothes, wrap them in blankets and warm them slowly with a warm drink if they can swallow. Shivering that stops, slurred speech, confusion or drowsiness at that reading is a 999 call.";
const THE_NUMBER_SAYS_2: &str = "%s is a low reading rather than a fever: normal is 36.1 to 37.2C, and a cold room or a thermometer that slipped reads low more often than an illness does. Take it again after ten minutes somewhere warm with the arm tucked in, and if it stays under 36C with shivering, confusion or drowsiness, ask a doctor the same day.";
const THE_NUMBER_SAYS_3: &str = "%s is a raised reading rather than a fever: normal is 36.1 to 37.2C and 38C (100.4F) is where a fever starts. A drink, an ordinary room and another reading in an hour, and a stiff neck, a rash that does not fade under a glass or trouble breathing is a same-day call.";
const COLD_SORE_0: &str = "A cold sore clears in about 7 to 10 days. Aciclovir cream from a pharmacist works best if you start it at the first tingle, and don't kiss anyone or share towels while it is weeping. See a GP if it is near your eye, if you are pregnant, or if a first outbreak comes with a fever.";
const COLD_SORE_1: &str = "Keep it off your eye: the virus spreads on your fingers, so wash your hands after touching the sore and don't rub your eyes. A cold sore that reaches the eye is seen by a GP or an optician the same day - it can damage the surface of the eye, and aciclovir eye ointment is a prescription rather than the cream.";
const COLD_SORE_2: &str = "A cold sore in pregnancy is common and not dangerous in itself, but mention it to your midwife, and keep it away from a newborn: a cold sore on a baby under four weeks is a same-day call to a doctor. Aciclovir cream is safe to use in pregnancy from a pharmacist, and it works best started at the first tingle.";
const COLD_SORE_3: &str = "Ring 111 or your surgery today: a cold sore on a baby under four weeks is seen the same day, because the virus is serious in a newborn. Until then, wash your hands before you pick the baby up, don't kiss the baby while a sore is weeping, and keep the sore itself away from the baby's face. Ring 999 if the baby is floppy, feeding badly, or has a temperature.";
const COLD_SORE_4: &str = "Don't pop, squeeze or pick it: the fluid spreads the virus to the rest of your lip and can leave a scar or an infection. Start aciclovir cream from a pharmacist now, keep it covered with a dab of cream rather than a plaster, wash your hands after touching it, and it should clear in 7 to 10 days.";
const COLD_SORE_5: &str = "Aciclovir cream from a pharmacist is the one that works: dab it on at the first tingle, five times a day for 4 to 5 days, and wash your hands after. It is the same medicine as Zovirax at a fraction of the price, and a numbing gel helps the soreness without changing how long it lasts. Keep it off your eye, and see a GP if it has not crusted over in 10 days.";
const COLD_SORE_6: &str = "A cold sore is contagious from the first tingle until the scab has healed over, so no kissing, and don't share cups, towels or lip balm while it is there. Swimming is fine, and so is work or school; wash your hands after touching it, keep it away from a newborn, and start aciclovir cream at the first tingle to shorten it.";
const COLD_SORE_7: &str = "Cold sores come from the herpes simplex virus, which most people catch as a child and which then sits dormant and flares up when you are run down - sun, cold wind, stress, illness, tiredness or a period. There is no cure, but aciclovir cream from a pharmacist at the first tingle shortens each one, and 7 to 10 days is the usual run.";
const COLD_SORE_8: &str = "Start aciclovir cream from a pharmacist now, at the first tingle: begun that early it can shorten the sore or stop it coming out at all. Don't kiss anyone or share towels until it has crusted over, and it clears in about 7 to 10 days.";
const DRINK_AND_THE_DRUG_0: &str = "One or two drinks is usually fine with an everyday medicine, taken with food. What decides it is the drug: ibuprofen and other anti-inflammatories irritate the stomach with alcohol, paracetamol's limit is the liver, codeine and antihistamines add to the drowsiness, and metronidazole must not meet alcohol at all. A pharmacist will check the one you actually take.";
const POSTNATAL_MOOD_REPLY_0: &str = "What you are describing is common - about one new parent in ten has postnatal depression - and it is treatable, so tell your GP or health visitor, who asks about this every week and will not be surprised. It is not the baby blues if it lasts past the first fortnight, starts months later, or comes with feeling numb about the baby, guilt, no appetite, or panic; it is not a failure of love and it does not mean anyone will take the baby away. Talking therapy works, and antidepressants can be taken while breastfeeding. Ring your GP today, and if you ever think about harming yourself or the baby, ring 111 or 999 or go to A&E.";
const POSTNATAL_MOOD_REPLY_1: &str = "The baby blues are the ordinary version: tearful, irritable and overwhelmed, starting around day three to five and gone within about ten days to a fortnight. Sleep when the baby sleeps, eat and drink something at every feed, and take the help that is offered. If it has not lifted by two weeks, or it is getting worse rather than better, tell your health visitor or GP - that is when it is called postnatal depression and it needs treating.";
const A_RATE_READING_0: &str = "A child's resting heart rate is higher than an adult's: about 100 to 160 in a baby, 90 to 150 up to three, 80 to 140 up to six, 70 to 120 up to nine, and 60 to 100 from about ten. One reading off a watch is usually the moment rather than the heart, so take it again sitting still; a fast rate that comes with fever, drowsiness or fast breathing needs a doctor the same day.";
const A_RATE_READING_1: &str = "BPM is beats per minute, the number of times your heart beats in a minute, and one reading is usually the moment rather than the heart";
const A_RATE_READING_2: &str = ": coffee, a hot shower, a late night or being unwell push it up, so sit still five minutes and take it again. Resting, 60 to 100 beats a minute is normal, and a rate that stays over 100 sitting still, one that stays under 50 and leaves you dizzy, or any reading with chest pain or breathlessness, is a doctor's call.";
const A_RATE_READING_3: &str = "HR is heart rate, and one reading is usually the moment rather than the heart";
const A_RATE_READING_4: &str = ": rest, 60 to 100 beats a minute is normal, and 50 is ordinary for a regular runner. Take it again sitting still for five minutes; under 50 with dizziness, fainting or unusual tiredness, or any reading with chest pain or breathlessness, is a doctor's call.";
const A_RATE_READING_5: &str = "On a watch, HR is heart rate, and one reading is usually the moment rather than the heart";
const A_RATE_READING_6: &str = "%d to %d beats a minute is the resting range, so it is where your own number sits inside that range that matters rather than one reading";
const A_RATE_READING_7: &str = "A heart rate of %d is at the slow end, and a slow resting pulse is normal in people who are fit and in sleep";
const A_RATE_READING_8: &str = "A heart rate of %d on a watch is one reading, and it is usually the moment rather than the heart";
const A_RATE_READING_9: &str = "A heart rate of %d is one reading, and it is usually the moment rather than the heart";
const A_RATE_READING_10: &str = "A high heart rate means over 100 beats a minute at rest, and one reading is usually the moment rather than the heart";
const A_RATE_READING_11: &str = "A low heart rate means under 60 beats a minute at rest, and one reading is usually the moment rather than the heart";
const A_RATE_READING_12: &str = "HR is heart rate, and one reading is usually the moment rather than the heart";
const AN_ANTACID_WITH_ANOTHER_TABLET_0: &str = "Most tablets are fine with an antacid. The ones that are not are iron, levothyroxine, alendronate and the antibiotics ciprofloxacin, norfloxacin, tetracycline and doxycycline: those want two hours between the antacid and the tablet. Paracetamol, ibuprofen, blood pressure tablets and antidepressants are not affected.";
const AN_ANTACID_WITH_ANOTHER_TABLET_1: &str = "An antacid can stop that tablet being taken up properly. Iron, levothyroxine and the antibiotics ciprofloxacin, norfloxacin, tetracycline and doxycycline are the ones that matter most, and so are alendronate and the other bone tablets. Leave two hours between the antacid and the other tablet - the leaflet in the box says for that medicine how long the gap should be.";
const A_PAINKILLER_WITH_AN_ANTIDEPRESSANT_0: &str = "An anti-inflammatory on top of an SSRI antidepressant raises the risk of bleeding in the stomach a little, because both thin the blood in different ways. Paracetamol is the painkiller to reach for first. If you do need ibuprofen, take it with food at the lowest dose that works, and get seen if you have a pain high in the stomach, black tarry poo, or you bruise and bleed more than you used to.";
const A_PAINKILLER_WITH_AN_ANTIDEPRESSANT_1: &str = "Codeine is usually fine with an SSRI for a few days: the things to watch are constipation and drowsiness, so no driving on it, and if you need more than a few days' worth ask about something else. Tramadol is the one not to mix with an SSRI - the two together can bring on too much serotonin - and none of them belong with an MAOI.";
const A_PAINKILLER_WITH_AN_ANTIDEPRESSANT_2: &str = "Paracetamol is the painkiller to use with an SSRI antidepressant - it is not the one the combination is about. Codeine is usually fine too, in the dose on the packet and for a few days rather than weeks. An anti-inflammatory is the one to ask about: ibuprofen, naproxen and aspirin add to the bleeding risk an SSRI already carries, so paracetamol first.";
const A_FOOD_AND_THE_DRUG_0: &str = "Most food is fine with most tablets. The pairs that matter are grapefruit with simvastatin and atorvastatin, dairy with doxycycline, ciprofloxacin, iron or levothyroxine, cranberry in quantity with warfarin, and a big change in leafy greens on warfarin. A pharmacist will check the two you actually have.";
const A_FOOD_AND_THE_DRUG_1: &str = "Grapefruit only matters with a handful of medicines, and statins are the common one: it blocks the enzyme that clears simvastatin and atorvastatin, so the drug builds up. Check the leaflet for 'avoid grapefruit', and if yours is one of those two ask about swapping to pravastatin or rosuvastatin rather than giving up the fruit. Paracetamol, antibiotics and blood pressure tablets are fine with it.";
const A_FOOD_AND_THE_DRUG_2: &str = "Cranberry in normal amounts is fine on warfarin, but a glass every day, or a lot of it, can push your INR up and make bruising and bleeding more likely. Keep it occasional, don't start a daily habit, and tell the anticoagulant clinic if you do, so they can check the INR. It doesn't stop the tablet working, so you can still drink it for a urine infection.";
const A_FOOD_AND_THE_DRUG_3: &str = "Milk, cheese and yoghurt don't stop most tablets working, so you can have dairy with them. The ones that care are the tetracycline antibiotics - doxycycline, lymecycline - and ciprofloxacin, which bind to dairy calcium, plus an iron tablet and levothyroxine, which dairy blocks: leave two hours around those and it makes no difference to the rest.";
const A_FOOD_AND_THE_DRUG_4: &str = "Leafy greens are good for you on warfarin: the vitamin K in them is what keeps the dose steady, so the rule is to keep the amount the same week to week rather than cut them out. What swings an INR is a change - a sudden daily kale or salad habit, or dropping greens you eat every day - so tell the clinic when that happens and carry on eating them.";
const A_FOOD_AND_THE_DRUG_5: &str = "Warfarin itself is not affected by grapefruit, but the newer blood thinners apixaban and rivaroxaban are partly cleared by the enzyme it blocks, so keep grapefruit to the odd glass and tell the clinic you eat it. Don't stop the tablets over it: what matters is a daily grapefruit habit, not one glass.";
const A_FOOD_AND_THE_DRUG_6: &str = "Grapefruit lifts the level of simvastatin and atorvastatin in the blood, which is what makes the muscles ache, so leave grapefruit and grapefruit juice off while you take it - one glass is not an emergency, a daily habit is. Pravastatin, rosuvastatin and fluvastatin are barely affected, so a swap is easy if you like grapefruit: ask the pharmacist.";
const A_FOOD_AND_THE_DRUG_7: &str = "Levothyroxine wants an empty stomach and no dairy for about four hours: take it with water when you wake, then leave the milk in your tea and cereal until later. Calcium, iron and antacids with calcium in them do the same thing, so the tablet goes first and the dairy after.";
const A_FOOD_AND_THE_DRUG_8: &str = "Milk is the thing to move, not the tablet: the calcium in milk, cheese and yoghurt blocks iron absorption, so take the iron with water or orange juice and leave two hours either side of dairy. Vitamin C helps iron in; tea, coffee and calcium hold it back.";
const A_FOOD_AND_THE_DRUG_9: &str = "That one does interact: doxycycline, lymecycline and ciprofloxacin bind to the calcium in milk, cheese and yoghurt and pass straight through, so take the tablet with water and leave two hours before or after dairy. Milk is fine with every other antibiotic, and with paracetamol.";
const A_CHECK_ON_A_CLEAR_RESULT_0: &str = "It is - nothing was found, so there is nothing to do about it. If the symptom that started all this is still there in a few weeks, say so to the GP even with that result behind you.";
const A_RESULT_REPLY_0: &str = "A blood test is a small sting and a vial or two, and there is nothing about having one to worry about - it is the number it comes back with that matters. If that comes back outside the range they will tell you, and it is usually repeated in a few weeks.";
const A_RESULT_REPLY_1: &str = "Same result, same question to put on the phone: what exactly it says, and what the range or the next step is. Most abnormal results are a little outside the range and are repeated in a few weeks, so ask rather than reading the worst into it.";
const A_RESULT_REPLY_2: &str = "Nothing in a clear result is a warning - it means nothing was found, so there is nothing to worry about and nothing to act on. If the symptom that started all this is still there in a few weeks, say so to the GP even with that result behind you.";
const A_RESULT_REPLY_3: &str = "\\b(?:need(?:s)? (?:a )?follow up|want(?:s)? to see|asked me to come|come back in|another appointment|appointment)\\b";
const A_RESULT_REPLY_4: &str = "\\b(?:follow up|follow-up|chase|chasing|ring|phone|call|ask|heard|hear)\\b";
const A_RESULT_REPLY_5: &str = "\\b(?:clear|all clear|normal|fine|nothing there|nothing found|no problems|nothing showed up|(?:she|he)'?s fine|(?:she|he) is fine)\\b";
const A_RESULT_REPLY_6: &str = "\\b(?:abnormal|borderline|flagged|outside the range|out of range|high|low|showed something|shows something|showed up)\\b";
const HERNIA_0: &str = "A hiatus hernia is the top of the stomach sliding up through the hole in the diaphragm that the food pipe comes through, so part of it sits above the muscle that keeps acid down. It is very common and it is not dangerous in itself: what it causes is heartburn and reflux, worse lying flat, bending or after a big meal. Smaller meals, nothing to eat for three hours before bed, the head of the bed up on a couple of books, and losing weight round the middle all help, and a GP can add a tablet that turns the acid down if those are not enough. An operation is only for the few people whose reflux still damages the food pipe on treatment. Trouble swallowing, food sticking, vomiting or weight loss without trying are the reasons to be seen quickly rather than treated at home.";
