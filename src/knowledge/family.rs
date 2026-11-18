//! Port of PyBot's `bot/knowledge/family.py`: children and babies, parents and partners,
//! friends and family rows - the guards that keep each card to the person and the question it
//! is written for, and the replies worded for the age, the person and the shape of the line.
//!
//! Import-time-only Python (no runtime port): the `lambda pair: -len(pair[0])` sort key that
//! builds `_GROWN_AGE_WORDS` (line 1269; the table is in the bundle), and the module-level
//! `for _tens_word ...` loop that fills `_YEAR_WORDS` (also in the bundle).

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g2 as dup;
use crate::knowledge::d1_shared::{self as sh, ShapeFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Bundle access and small Python idioms used throughout.

/// A module global of family.py by its Python name.
fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.family.{name}"))
}
fn g_re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("family.{name} is not a regex"))
}
/// A `{str: str}` dict global, in insertion order.
fn g_pairs(name: &str) -> Vec<(String, String)> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}
/// A `{str: int}` dict global, in insertion order.
fn g_str_ints(name: &str) -> Vec<(String, i64)> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
}
fn int_of(table: &[(String, i64)], key: &str) -> Option<i64> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

macro_rules! re_global {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<&'static Re> = LazyLock::new(|| g_re(stringify!($name)));)*
    };
}
macro_rules! str_global {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<String> = LazyLock::new(|| g(stringify!($name)).str().to_string());)*
    };
}
macro_rules! strs_global {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! set_global {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($name)).str_set());)*
    };
}
macro_rules! inline_re {
    ($name:ident, $pat:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, 0));
    };
    ($name:ident, $pat:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
    };
}

/// `re.findall(r"[a-z']+", s)`, in order.
fn az_list(s: &str) -> Vec<&str> {
    s.split(|c: char| !(c.is_ascii_lowercase() || c == '\'')).filter(|w| !w.is_empty()).collect()
}

/// `bool(words & set)`.
fn meets<'a, I: IntoIterator<Item = &'a str>>(words: I, set: &HashSet<String>) -> bool {
    words.into_iter().any(|w| set.contains(w))
}

/// `str(reading.get("text") or reading.get("clean") or "")`, falling back to the words joined
/// when both are empty - the way most guards here read the line.
fn text_or_words(u: &Understood) -> String {
    let t = sh::text_or_clean(u);
    if !t.is_empty() {
        return t.to_string();
    }
    u.words.join(" ")
}

/// `str(reading.get("clean") or reading.get("text") or "")`, falling back to the words joined.
fn clean_or_words(u: &Understood) -> String {
    let t = sh::clean_or_text(u);
    if !t.is_empty() {
        return t.to_string();
    }
    u.words.join(" ")
}

/// Python's `repr(str)`.
fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `str(list_of_str)`.
fn py_list_str(items: &[String]) -> String {
    format!("[{}]", items.iter().map(|s| py_repr(s)).collect::<Vec<_>>().join(", "))
}

/// `str(reading.get("clean") or reading.get("norm") or reading.get("text") or "")`: the norm is a
/// list, and `str()` of it is its repr.
fn clean_or_norm_or_text(u: &Understood) -> String {
    if !u.clean.is_empty() {
        return u.clean.clone();
    }
    if !u.norm.is_empty() {
        return py_list_str(&u.norm);
    }
    u.text.clone()
}

/// The person's lines on the earlier turns (`turn.get("user") or turn.get("text")`).
fn turn_users(memory: &Memory) -> impl DoubleEndedIterator<Item = &str> {
    memory.turns.iter().map(|t| t.user.as_str())
}

/// An anchored `pattern.match(text, pos)`.
fn match_at(re: &Re, text_: &str, pos: usize) -> bool {
    re.search_at(text_, pos).map(|m| m.start() == pos).unwrap_or(false)
}

// ---------------------------------------------------------------------------------------------
// Module data.

strs_global!(
    _TWO_MONTHS_UP, OFF_SCHOOL_ASKS, OFF_SICKNESS, OFF_POX, OFF_ANTIBIOTIC, OFF_FEVER, OFF_MILD,
    _TEEN_WORDS, _ADULT_WORDS, _SMALL_CHILD, _A_YEAR_OR_MORE, _MONTHS_AT_LEAST_SIX, _AT_HOME_WORDS,
    _OCCASION_WORDS, _A_CHILD_WORD, _FRIEND_DRIFT_WORDS, _FRIEND_WORRY_WORDS, _FRIEND_MOVED_WORDS,
    _STROKE_EATING, _HANDED_BACK, A_CHILD_MONEY_TROUBLE, _OTHER_DRINKS, _WATER_ONLY, _MONEY_ASKERS,
);
set_global!(
    _GIVING_UP_A_PERSON, _BABY_IN_THE_WATER, _THE_FRAME_OF_AN_AGE, _A_PERSON_OR_CHILD_WORD,
    _TEN_OR_OVER, _GROWN_QUIET_RELATIONS, _CHILD_QUIET_RELATIONS, _FRIEND_QUIET_RELATIONS,
    _HOUSEHOLD_QUIET_RELATIONS, _CAFFEINE_SAID, _MILK_SAID, _COOK_ASK, _LET_DOWN, _LET_DOWN_SKIPPED,
    _LET_DOWN_PEOPLE, _SOMEONE_ELSE,
);
str_global!(
    _DUMMY_BABY, _DUMMY_HABIT, _DUMMY_OLDER, _CHILD_CHART, _MILK_FROM_A_YEAR, _MILK_FROM_SIX_MONTHS,
    _FRIENDS_WORK, _FRIENDS_CLUBS, _DATE_AT_HOME, _DATE_OCCASION, _DATE_OUT, _THE_ASKING_OUT,
    _THE_NOT_WANTING, _HURT_FEELINGS, BIRTH_WEIGHT, _MILESTONE_TAIL, _MILESTONE_ORDER, _TALKER_HELP,
    _TALKER_NO_AGE, _TALKER_UNDER_TWO, _TALKER_TWO, _TALKER_THREE, _TALKER_SCHOOL,
    _QUIET_CHILD_REPLY, _QUIET_FRIEND_REPLY, _FRIEND_MOVED_REASON, _FRIEND_QUIET_REASON,
    _QUIET_GROWN_REPLY, PICKY_EATER, _STROKE_EATING_REPLY, _APPETITE_ASK, _APPETITE_FOOD,
    THE_REPAIR_AFTER_HITTING, THE_REPAIR_AFTER_SHOUTING, _DRINKS_ANSWER, _WATER_ANSWER,
    _FAMILY_ASKING_MONEY, _FRIEND_ASKING_MONEY, NAPPY_HOW_OFTEN, NAPPY_HOW_TO,
);
re_global!(
    _AN_EYE_IN_THE_LINE, _GLASS_IN_A_THING, _ASKS_THE_AGE, _A_BOY, _A_GIRL, _MONEY_IN_THE_LINE,
    _A_CAFFEINE_DRINK, _A_BARE_LEAK, _A_CLOTH_OR_A_WASH, _A_NO_CAFFEINE, OTHER_PUMP,
    _TAKING_SOMEBODY, _A_ROLE_IN_A_WEDDING, _A_NAME_IS_ASKED, _GIVING_UP_ON, _WAKING_UP, _A_DEATH,
    _CHILD_IN_THE_LINE, _AGE_DIGITS, _AGE_WORD_FORM, _A_CHILD_OF_AGE, _A_CHILD_OF_AGE_IN_WORDS,
    _A_CHILD_NAMED, _A_CHILD_IS_AGE, _AGE_TOO_YOUNG, _TEEN_SAID, _BED_CLOCK, _NOT_AN_AGE_AFTER,
    NOT_GAINING, GROWTH_SPURT, ANNOUNCEMENT_WEIGHT, QUESTION_WEIGHT, CONCERNED_WEIGHT, _AT_WORK,
    _FRIENDS_FEELING, _TEEN_AGE, _ADULT_AGE, _SMALL_CHILD_AGE, _A_PICKUP_NOT_A_WAKING,
    _A_FIGURE_IN_THE_LINE, _A_WEAR_STRETCH, _THE_JOB, _A_PERSON_OR_CHILD_PHRASE, _A_GROWN_AGE,
    _GRUNT, _GRUNT_WITH_BREATH, _AGE_IN_LINE, _GRUNT_AS_AN_ANSWER, _TEEN, _SAY_TO_SOMEONE,
    _NAMES_A_LIKING, _NOT_A_LIKING, _A_CHILD_AGE, _A_WHEN_ASK, _THE_YEAR_NAMED, NOT_A_BITE,
    _TALK_MONTHS, _TALK_YEARS, _TALK_WORD_AGE, _TALK_WORD_HALF, _ASKING_HOW_TO_TALK,
    _TALK_CHILD_WORD, _TALKER_SPEECH_NOUN, _TALKER_AIMED_AT_A_PERSON, A_TALK_WORD,
    A_A_SILENCE_WITH_A_PERSON, _APPETITE_AFTER, _APPETITE_WEIGHT, _APPETITE_TASTE,
    _APPETITE_ELDERLY, _A_PERSON_ALREADY_THERE, _GETTING_OVER, _AN_ENDING_OR_A_PERSON,
    _SOME_TIME_AGO, _WORDS_FOR_SOMEONE, _I_AM_IN_THE_ROW, _I_ASK_TO_STOP_THE_ROW,
    _A_GROWN_UP_IN_THE_ROW, _A_CHILD_IN_THE_ROW, _A_VISIT_I_AM_MAKING, _A_VISIT_AS_A_GRIEVANCE,
    _A_FOOD_LIMIT, AN_HOUR, A_LENGTH_OF_STAY, _SMACKED_A_CHILD, _TOLD_THEM_OFF,
    _ONE_PARENT_IS_TIRED, _A_HARD_PARENT, _TIRED_BECAUSE_OF_SOMEONE, ASKS_FOR_A_PRESENT,
    THEY_BOUGHT_IT, A_RATE_OR_A_GRIEVANCE, _A_TRIP_STILL_BEING_WEIGHED, A_NAMED_DAY_RATE,
    A_MEAL_NAMED, A_HOME_OR_A_VISIT, _TELLING_A_CHILD, WHO_AT_THE_FRONT, ASKING_FOR_THE_WORDS,
    _A_CHILD_IN_THE_LINE, _CHILD_ATE, _A_SWEET_THING, _BIRTH_IN_THE_CHAT, _A_HOSPITAL_OR_A_WARD,
    _ASKING_FOR_THE_WORDS, _THE_LENGTH_ASK, _IT_ITCHES, _OWN_APPETITE, _SOMEONE_NAMED,
    _A_WHO_CAN_HELP, _AN_ELDER_TO_LOOK_AFTER,
);

/// `_AGE_WORDS`: "one" -> 1 ... "seventeen" -> 17.
static _AGE_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| g_str_ints("_AGE_WORDS"));
/// `_CHILD_SAID`: 1 -> "one" ... (int keys).
static _CHILD_SAID: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| {
    g("_CHILD_SAID").pairs().iter().map(|(k, v)| (k.as_int().unwrap_or(-1), v.str().to_string())).collect()
});
fn child_said(age: i64) -> Option<String> {
    _CHILD_SAID.iter().find(|(k, _)| *k == age).map(|(_, v)| v.clone())
}
static _MONTH_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| g_str_ints("_MONTH_WORDS"));
static _YEAR_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| g_str_ints("_YEAR_WORDS"));
static _SMALL_NUMBERS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| g_str_ints("_SMALL_NUMBERS"));
static _SMALL_ENOUGH: LazyLock<i64> = LazyLock::new(|| g("_SMALL_ENOUGH").as_int().unwrap_or(9));
static _A_LATE_TALKER_IS_UNDER: LazyLock<f64> =
    LazyLock::new(|| g("_A_LATE_TALKER_IS_UNDER").as_f64().unwrap_or(10.0));

/// `pattern.match(text, pos)` for `_NOT_AN_AGE_AFTER`, anchored with `\G`.
static NOT_AN_AGE_AFTER_AT: LazyLock<Re> =
    LazyLock::new(|| Re::new(&format!(r"\G(?:{})", _NOT_AN_AGE_AFTER.pattern()), _NOT_AN_AGE_AFTER.flags()));

// ---------------------------------------------------------------------------------------------
// Glass, phones, and the child's pronouns.

/// False when the glass in the line belongs to a phone, a window or a screen.
pub fn _not_a_thing_made_of_glass(understood: &Understood) -> bool {
    // "raw" is not a key the reading has
    let said = text::squash(&clean_or_words(understood).to_lowercase());
    if !_GLASS_IN_A_THING.is_match(&said) {
        return true;
    }
    _AN_EYE_IN_THE_LINE.is_match(&said)
}

/// he/him or she/her for the child this card is about, they/them when nothing says which; the
/// fourth value is the verb that goes with the subject ("does", "do").
pub fn _the_child_pronouns(memory: &Memory, understood: &Understood) -> (&'static str, &'static str, &'static str, &'static str) {
    let mut said: Vec<&str> = vec![sh::text_or_clean(understood)];
    // LEANS_ON: the line this one leans on would be read here; always absent on one turn.
    said.extend(turn_users(memory));
    for line in said {
        let low = format!(" {} ", line.to_lowercase());
        let boy = _A_BOY.is_match(&low);
        let girl = _A_GIRL.is_match(&low);
        if boy && !girl {
            return ("he", "him", "his", "does");
        }
        if girl && !boy {
            return ("she", "her", "hers", "does");
        }
    }
    ("they", "them", "their", "do")
}

/// The dummy advice for the age the chat has named, and the baby advice when it has none.
pub fn _the_dummy_answer(memory: &Memory, understood: &Understood) -> Option<String> {
    let age = match _the_child_age_in_the_chat(memory, understood) {
        Some(a) if a >= 2 => a,
        _ => return Some(_DUMMY_BABY.clone()),
    };
    let word = child_said(age).filter(|w| !w.is_empty()).unwrap_or_else(|| age.to_string());
    if age <= 4 {
        return Some(_DUMMY_HABIT.replacen("%s", &word, 1).replacen("%s", &word, 1));
    }
    Some(_DUMMY_OLDER.clone())
}

/// The phone card, opening on the age only when the age is what was asked.
pub fn _a_phone_rule_and_an_age(memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = sh::text_or_clean(understood);
    let (he, him, _his, does) = _the_child_pronouns(memory, understood);
    if _ASKS_THE_AGE.is_match(&text_.to_lowercase()) {
        return Some(format!(
            "Eleven or twelve is when most children get their first phone, and the rules matter \
more than the model. Agree the amount in advance: about two hours a day of \
screens for fun at that age, and what happens when it runs out, because a rule \
made mid-argument is a fight. Charge it in the kitchen overnight with a cheap \
alarm clock in the bedroom, and set the phone's own downtime with a passcode {he} \
{does} not have. Taking it off {him} for a week over a bad mood only teaches {him} to \
hide it: keep confiscation for the things you said you would."
        ));
    }
    Some(format!(
        "Agree the rule before the argument: about two hours a day of screens for fun, what \
happens when the time is gone, and which hours are phone-free for the whole house so \
it is not a rule about {him} alone. Eleven or twelve is when most children get their \
first phone. Charge it in the kitchen overnight with a cheap alarm clock in the \
bedroom, and set the phone's own downtime with a passcode {he} {does} not have. Taking it \
off {him} for a week over a bad mood only teaches {him} to hide it: keep confiscation for \
the things you said you would."
    ))
}

// ---------------------------------------------------------------------------------------------
// Nappies, pumps, coffee, loans, weddings, names, giving up, wakes and debts.

/// True unless the line is only about a nappy that leaks.
pub fn _a_leak_is_not_a_strip(understood: &Understood) -> bool {
    let text_ = text_or_words(understood);
    if !_A_BARE_LEAK.is_match(&text_) {
        return true;
    }
    _A_CLOTH_OR_A_WASH.is_match(&text_)
}

/// The pump in the line is a machine rather than the milk one.
pub fn _not_a_machine_pump(understood: &Understood) -> bool {
    let clean = text::squash(&understood.clean.to_lowercase());
    !OTHER_PUMP.is_match(&clean)
}

/// True unless the line names a drink that carries caffeine.
pub fn _not_a_real_coffee(understood: &Understood) -> bool {
    let text_ = text_or_words(understood);
    if _A_NO_CAFFEINE.is_match(&text_) {
        return true;
    }
    !_A_CAFFEINE_DRINK.is_match(&text_)
}

/// True unless the line is taking somebody somewhere, which is a day out and not a loan.
pub fn _not_a_trip_out(understood: &Understood) -> bool {
    let said = sh::or(&understood.norm_text, &understood.text).to_lowercase();
    if _HANDED_BACK.iter().any(|w| said.contains(w.as_str())) {
        return true;
    }
    !_TAKING_SOMEBODY.is_match(&said)
}

/// The row's two fences: what was handed over is a thing, and the take-verb was not a trip.
pub fn _not_a_debt_or_a_trip(understood: &Understood) -> bool {
    _not_a_debt(understood) && _not_a_trip_out(understood)
}

/// True unless the line names money, which is `lent-money`'s question and not this one.
pub fn _not_a_debt(understood: &Understood) -> bool {
    !_the_debt_is_money(understood)
}

/// True when the line names the role or the wedding the money is for.
pub fn _a_role_in_a_wedding(understood: &Understood) -> bool {
    _A_ROLE_IN_A_WEDDING.is_match(&text_or_words(understood))
}

/// False when the line asks what something is called, which is a word question, not a text.
pub fn _the_question_is_a_name(understood: &Understood) -> bool {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    !_A_NAME_IS_ASKED.is_match(&words.join(" "))
}

/// False when the thing being given up on is a diet, a job or an extension.
pub fn _a_giving_up_on_a_person(understood: &Understood) -> bool {
    let text_ = text_or_words(understood).to_lowercase();
    if !_GIVING_UP_ON.is_match(&text_) {
        return true;
    }
    if ["marriage", "divorce", "husband", "wife", "partner"].iter().any(|w| text_.contains(w)) {
        return true;
    }
    for m in _GIVING_UP_ON.finditer(&text_) {
        if text::split_ref(m.str(1)).iter().any(|w| _GIVING_UP_A_PERSON.contains(*w)) {
            return true;
        }
    }
    false
}

/// True unless the line's "wake" is somebody waking up rather than a funeral gathering.
pub fn _a_waking_is_not_a_wake(understood: &Understood) -> bool {
    let text_ = text_or_words(understood);
    if !_WAKING_UP.is_match(&text_) {
        return true;
    }
    _A_DEATH.is_match(&text_)
}

/// True when the line names money or a debt, which is the only thing the money row answers.
pub fn _the_debt_is_money(understood: &Understood) -> bool {
    _MONEY_IN_THE_LINE.is_match(&text_or_words(understood))
}

/// `_not_a_line_about_a_bath` and an age in the line: the first dip is a baby's question.
pub fn _a_baby_in_the_water(understood: &Understood) -> bool {
    if !_not_a_line_about_a_bath(understood) {
        return false;
    }
    if understood.words.is_empty() {
        let lowered = understood.clean.to_lowercase();
        return text::split_ref(&lowered).iter().any(|w| _BABY_IN_THE_WATER.contains(w.to_lowercase().as_str()));
    }
    understood.words.iter().any(|w| _BABY_IN_THE_WATER.contains(w.to_lowercase().as_str()))
}

inline_re!(A_BATH, r"\bbath");

/// False when the line the person typed names a bath, however it is read.
pub fn _not_a_line_about_a_bath(understood: &Understood) -> bool {
    !A_BATH.is_match(&sh::text_or_clean(understood).to_lowercase())
}

/// Whether the line names a child (or the school) as well as the symptom and the morning.
pub fn _a_childs_morning(understood: &Understood) -> bool {
    _CHILD_IN_THE_LINE.is_match(&understood.clean)
}

/// No dose to read off a bottle under two months; from two months, the bottle.
pub fn _medicine_for_a_young_baby(_memory: &Memory, reading: &Understood) -> Option<String> {
    // set(words) joined: a set of single tokens, so only the phrases inside one word can match
    // across it; the port keeps the words' own order (set order noted, not reproduced)
    let mut words: Vec<&str> = Vec::new();
    for w in &reading.words {
        if !words.contains(&w.as_str()) {
            words.push(w);
        }
    }
    let mut parts: Vec<&str> = words;
    parts.extend(reading.norm.iter().map(String::as_str));
    let said = parts.join(" ");
    if _TWO_MONTHS_UP.iter().any(|p| said.contains(p.as_str())) {
        return Some(
            "From two months paracetamol is the one that is used, at the dose on the bottle \
for their age with the syringe that comes with it, no more than four doses in \
24 hours - and ibuprofen only if they are over 5kg. But a baby under three \
months with a temperature of 38C or more is seen the same day rather than \
treated at home, so ring 111 or the GP and say how old they are. Floppy, not \
feeding, hard to wake or a rash that does not fade is 999."
                .into(),
        );
    }
    Some(
        "Not without a doctor or 111 at that age. Paracetamol is only given to a baby under \
two months on medical advice, and ibuprofen is not used under three months or under \
5kg at all, so there is no dose to read off a bottle. Ring 111 or your GP, say how \
old they are and what the temperature is, and they will tell you what is safe. If a \
baby under three months has a temperature of 38C or more, or is floppy, not feeding, \
hard to wake or has a rash that does not fade, that is 999 rather than a dose."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// School: lost teeth and days off.

/// A word list compiled into one `\bword\b` test per word.
fn bounded_res(words: &[String]) -> Vec<Re> {
    words.iter().map(|w| Re::new(&format!(r"\b{}\b", sh::escape(w)), 0)).collect()
}

static OFF_SCHOOL_ASKS_RES: LazyLock<Vec<Re>> = LazyLock::new(|| bounded_res(&OFF_SCHOOL_ASKS));

/// True when the line asks how long, or whether, a child stays away from school.
pub fn _asks_the_exclusion_question(clean: &str) -> bool {
    OFF_SCHOOL_ASKS_RES.iter().any(|r| r.is_match(clean))
}

/// A child's first lost tooth, and the one case that is not that: an adult's loose tooth.
pub fn _milk_tooth(_memory: &Memory, reading: &Understood) -> Option<String> {
    let clean = reading.clean.as_str();
    const ADULT: [&str; 11] = [
        "adult tooth", "adult teeth", "my tooth", "permanent tooth", "i am 30", "i'm 30", "i am 40", "i'm 40",
        "grown up tooth", "my own tooth", "gum disease",
    ];
    const CHILD: [&str; 6] = ["my son", "my daughter", "my child", "my kid", "toddler", "baby"];
    let adult = ADULT.iter().any(|w| clean.contains(w));
    if adult && !CHILD.iter().any(|w| clean.contains(w)) {
        return Some(
            "A loose tooth in an adult is a dentist's job this week, not a wait-and-see: \
gum disease is the usual cause and it is treatable, but the tooth will not \
tighten up on its own. Do not wiggle it, and if the gum is sore or bleeding when \
you brush, say that at the appointment."
                .into(),
        );
    }
    Some(
        "That is normal: milk teeth start coming out at about six, usually the two at the \
front of the bottom, and the last ones go around twelve. A little blood is fine, and \
let a wobbly tooth come out by itself rather than pulling it. If a tooth is knocked \
out by a bang, it is a dentist the same day - an adult tooth goes back in the socket \
or in milk on the way."
            .into(),
    )
}

static OFF_SICKNESS_RES: LazyLock<Vec<Re>> = LazyLock::new(|| bounded_res(&OFF_SICKNESS));
static OFF_POX_RES: LazyLock<Vec<Re>> = LazyLock::new(|| bounded_res(&OFF_POX));
static OFF_ANTIBIOTIC_RES: LazyLock<Vec<Re>> = LazyLock::new(|| bounded_res(&OFF_ANTIBIOTIC));
static OFF_FEVER_RES: LazyLock<Vec<Re>> = LazyLock::new(|| bounded_res(&OFF_FEVER));
static OFF_MILD_RES: LazyLock<Vec<Re>> = LazyLock::new(|| bounded_res(&OFF_MILD));
/// `OFF_MILD_LEADS`: each mild illness's words (as `\bword\b` tests) and its lead.
static OFF_MILD_LEADS: LazyLock<Vec<(Vec<Re>, String)>> = LazyLock::new(|| {
    g("OFF_MILD_LEADS").items().iter().map(|p| (bounded_res(&p.items()[0].strs()), p.items()[1].str().to_string())).collect()
});

/// `_off_school.<locals>.names`: whether the line names one of the group's words.
#[allow(non_snake_case)]
pub fn _off_school__names(group: &[Re], clean: &str) -> bool {
    group.iter().any(|r| r.is_match(clean))
}

/// The school's exclusion rule, led by the symptom the parent named ("" when school is only
/// the background).
pub fn _off_school(_memory: &Memory, reading: &Understood) -> Option<String> {
    let clean = reading.clean.as_str();
    let names = |group: &[Re]| _off_school__names(group, clean);
    if !(names(&OFF_SICKNESS_RES) || names(&OFF_POX_RES) || names(&OFF_ANTIBIOTIC_RES) || names(&OFF_FEVER_RES) || names(&OFF_MILD_RES))
        && !_asks_the_exclusion_question(clean)
    {
        return Some(String::new());
    }
    let (lead, tail): (String, String) = if names(&OFF_SICKNESS_RES) {
        (
            "48 hours off after the last bout of sickness or diarrhoea, and until 24 hours \
after a temperature settles without medicine. "
                .into(),
            "Ring the school on the first morning and say what it is.".into(),
        )
    } else if names(&OFF_POX_RES) {
        (
            "Chickenpox needs every spot crusted over, usually about five days after the rash \
came out. "
                .into(),
            "Ring the school on the first morning and say what it is.".into(),
        )
    } else if names(&OFF_ANTIBIOTIC_RES) {
        (
            "Scarlet fever and whooping cough need 48 hours after the first antibiotic. ".into(),
            "Ring the school on the first morning and say what it is; if the throat is sore \
with a fever, the doctor decides that one."
                .into(),
        )
    } else if names(&OFF_FEVER_RES) {
        (
            "Twenty-four hours off after a temperature settles without medicine, and keep them \
home while it is still spiking. A fever with a rash, a stiff neck or a child who \
is hard to wake is a doctor's call the same day, not a school decision. "
                .into(),
            "If it turns out to be the sickness bug, it is 48 hours after the last bout; ring \
the school on the first morning and say what it is."
                .into(),
        )
    } else if names(&OFF_MILD_RES) {
        let lead = OFF_MILD_LEADS
            .iter()
            .find(|(words, _)| names(words))
            .map(|(_, said)| said.clone())
            .unwrap_or_else(|| {
                "A cold, a cough or a runny nose is not a reason to stay away if the child is \
well enough to join in: eating, drinking, no fever. "
                    .into()
            })
            + " ";
        let back = if lead.contains("24 hours") {
            ""
        } else {
            "They can go back once any fever has been gone for 24 hours without medicine. "
        };
        let tail = format!(
            "{back}If it is the sickness bug instead, it is 48 hours after the last bout; \
chickenpox is until the spots crust over, usually about five days; ring \
the school on the first morning and say what it is."
        );
        (lead, tail)
    } else {
        (
            "The symptom decides: 48 hours off after the last bout of sickness or diarrhoea, \
24 hours after a temperature settles without medicine, and a cold with no fever is \
usually a school day as long as the child is well enough to join in. "
                .into(),
            "Chickenpox needs every spot crusted over, usually about five days, and scarlet \
fever or whooping cough 48 hours after the first antibiotic. Ring the school on \
the first morning and say what it is."
                .into(),
        )
    };
    Some(lead + &tail)
}

// ---------------------------------------------------------------------------------------------
// A child's age and bedtime.

/// The age the line names, or None: "my 10 year old", "aged 7", "a seven year old".
pub fn _child_age(clean: &str) -> Option<i64> {
    if let Some(m) = _AGE_DIGITS.search(clean) {
        for group in m.groups().into_iter().flatten() {
            if !group.is_empty() {
                return group.parse().ok();
            }
        }
    }
    if let Some(m) = _AGE_WORD_FORM.search(clean) {
        return int_of(&_AGE_WORDS, m.str(1));
    }
    None
}

/// The age the line names in any of the shapes people write it, or None.
pub fn _any_age_in(clean: &str) -> Option<i64> {
    if let Some(age) = _child_age(clean) {
        return Some(age);
    }
    let mut found = _A_CHILD_OF_AGE.search(clean).or_else(|| _A_CHILD_OF_AGE_IN_WORDS.search(clean));
    if found.is_none() && _A_CHILD_NAMED.is_match(clean) {
        found = _A_CHILD_IS_AGE.search(clean);
        if let Some(m) = &found {
            let years: i64 = m.str(1).parse().unwrap_or(0);
            if !(1..=17).contains(&years) {
                found = None;
            } else if match_at(&NOT_AN_AGE_AFTER_AT, clean, m.end()) {
                return None;
            }
        }
    }
    let m = found?;
    if match_at(&NOT_AN_AGE_AFTER_AT, clean, m.end()) {
        return None;
    }
    let value = m.str(1);
    if text::is_digit(value) {
        return value.parse().ok();
    }
    int_of(&_AGE_WORDS, value)
}

/// The child's age, from the line, then from the line it leans on, then from the chat.
pub fn _the_child_age_in_the_chat(memory: &Memory, understood: &Understood) -> Option<i64> {
    // LEANS_ON: the leaned-on line's text would be read after these; absent on one turn.
    for said in [&understood.text, &understood.clean, &understood.norm_text] {
        if let Some(age) = _any_age_in(&said.to_lowercase()) {
            return Some(age);
        }
    }
    for user in turn_users(memory).rev() {
        if let Some(age) = _any_age_in(&user.to_lowercase()) {
            return Some(age);
        }
    }
    None
}

/// The bedtime the line names in the evening, in minutes after six, or None.
pub fn _said_bedtime(clean: &str) -> Option<i64> {
    let m = _BED_CLOCK.search(clean)?;
    let hour: i64 = sh::or(m.str(1), m.str(4)).parse().ok()?;
    let minute_text = sh::or(m.str(2), m.str(5));
    let minute: i64 = if minute_text.is_empty() { 0 } else { minute_text.parse().ok()? };
    if !m.str(3).is_empty() || (6..=11).contains(&hour) {
        return Some(hour * 60 + minute);
    }
    None
}

/// "A eight year old" -> "An eight year old": the article follows the sound.
pub fn _an(who: &str) -> String {
    let body = who.strip_prefix("A ").unwrap_or(who);
    match text::split_ref(body).first() {
        Some(first) if ["eight", "eleven", "eighteen"].contains(first) => format!("An {body}"),
        _ => who.to_string(),
    }
}

/// "A seven year old" -> "a seven year old", for the middle of a sentence.
pub fn _a(who: &str) -> String {
    format!("{}{}", text::slice(who, 0, Some(1)).to_lowercase(), text::slice(who, 1, None))
}

/// 630 -> "6.30pm", 540 -> "9pm": the way a bedtime is written.
pub fn _clock_words(minutes: i64) -> String {
    let (hour, minute) = (minutes.div_euclid(60), minutes.rem_euclid(60));
    if minute != 0 {
        return format!("{hour}.{minute:02}pm");
    }
    format!("{hour}pm")
}

/// `_CHILD_BANDS`: (the oldest age, the hours, the window, the earliest, the latest).
static _CHILD_BANDS: LazyLock<Vec<(i64, String, String, i64, i64)>> = LazyLock::new(|| {
    g("_CHILD_BANDS")
        .items()
        .iter()
        .map(|b| {
            let it = b.items();
            (
                it[0].as_int().unwrap_or(0),
                it[1].str().to_string(),
                it[2].str().to_string(),
                it[3].as_int().unwrap_or(0),
                it[4].as_int().unwrap_or(0),
            )
        })
        .collect()
});

/// The bedtime for the age the line names; the whole chart when it names none.
pub fn _child_bedtime(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = sh::clean_or_text(understood).to_lowercase();
    let mut age = _child_age(&clean);
    let teen = _TEEN_SAID.is_match(&clean) && age.is_none();
    if teen {
        age = Some(15);
    }
    let age = match age {
        Some(a) if !_AGE_TOO_YOUNG.is_match(&clean) && a <= 18 => a,
        _ => return Some(_CHILD_CHART.clone()),
    };
    let (_, hours, window, low, high) = _CHILD_BANDS.iter().find(|b| age <= b.0)?.clone();
    let who = if teen {
        "A teenager".to_string()
    } else {
        _an(&format!("A {} year old", child_said(age).unwrap_or_else(|| age.to_string())))
    };
    if let Some(said) = _said_bedtime(&clean) {
        let tots = _clock_words(said);
        if said > high {
            return Some(format!(
                "{tots} is past the window for {}, who is usually ready between {window} and needs {hours}. It \
works if the morning starts late enough; if it does not, move lights out \
fifteen minutes earlier for a fortnight rather than arguing tonight.",
                _a(&who)
            ));
        }
        if said < low {
            return Some(format!(
                "{tots} is early for {}, who is usually ready between {window}, not before. A child who is not \
tired fights it, so keep the last hour calm and dim and work back to {tots} over \
a week or two.",
                _a(&who)
            ));
        }
        return Some(format!(
            "{tots} is inside the window for {} - {window}, with {hours} of sleep - so it is fine as long as \
the mornings are not a fight. Keep the wake-up time fixed and the last hour calm \
and screen-free.",
            _a(&who)
        ));
    }
    if age >= 13 {
        return Some(format!(
            "Teenagers need eight to ten hours and their body clock runs late, so lights out \
between {window} is right and 10pm is not late. Fix the wake-up time and keep it at \
weekends, get daylight early, and keep the phone out of the last half hour."
        ));
    }
    Some(format!(
        "{who} needs {hours}, so lights out between {window}, earlier on a school morning. Work back from the \
wake-up time rather than the clock, keep the last hour calm and screen-free, and let \
it move half an hour either way as long as the mornings work."
    ))
}

/// True when the line is about a 13 to 17 year old, however the age is written.
pub fn _a_teenager_is_named(understood: &Understood) -> bool {
    let text_ = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    if _TEEN_WORDS.iter().any(|w| text_.contains(w.as_str())) {
        return true;
    }
    _TEEN_AGE.is_match(&text_)
}

/// True when the boy or girl on the line is grown: the small-child cards are not theirs.
pub fn _an_adult_child_is_named(understood: &Understood) -> bool {
    let text_ = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    if _ADULT_WORDS.iter().any(|p| text_.contains(p.as_str())) {
        return true;
    }
    _ADULT_AGE.is_match(&text_)
}

/// A `shape` on the bedtime row: true when the line says a small child's age.
pub fn a_small_childs_age(words: &str) -> bool {
    _SMALL_CHILD_AGE.is_match(&words.to_lowercase())
}

/// False when the child is a teenager or grown, or when the "up" is a pickup: not this card.
pub fn _not_a_toddler_in_the_morning(understood: &Understood) -> bool {
    if _a_teenager_is_named(understood) || _an_adult_child_is_named(understood) {
        return false;
    }
    !_A_PICKUP_NOT_A_WAKING.is_match(sh::clean_or_text(understood))
}

/// False when the line says how old a relation is and nothing else.
pub fn _not_just_the_age_of_a_relation(understood: &Understood) -> bool {
    // "clear" is not a key the reading has, so this is the clean line
    let text_ = text::squash(&understood.clean.to_lowercase());
    if text_.is_empty() {
        return true;
    }
    if text_.contains('?') || _ASKS_THE_AGE.is_match(&text_) {
        return true;
    }
    if !_A_FIGURE_IN_THE_LINE.is_match(&text_) {
        return true;
    }
    text::split_ref(&text_)
        .into_iter()
        .filter(|w| !_THE_FRAME_OF_AN_AGE.contains(*w))
        .any(|w| !w.is_empty() && _A_FIGURE_IN_THE_LINE.fullmatch(w).is_none())
}

/// False when the line is the wear idiom about a stretch of time or a job, not a bad dream.
pub fn _not_a_stretch_or_a_job(understood: &Understood) -> bool {
    let words: Vec<String> =
        understood.words.iter().map(|w| sh::strip_punct(&w.to_lowercase()).to_string()).collect();
    let mut text_ = words.iter().filter(|w| !w.is_empty()).cloned().collect::<Vec<_>>().join(" ");
    if text_.is_empty() {
        text_ = text::squash(&sh::text_or_clean(understood).to_lowercase());
    }
    if words.iter().any(|w| _A_PERSON_OR_CHILD_WORD.contains(w.as_str())) {
        return true;
    }
    if _A_PERSON_OR_CHILD_PHRASE.is_match(&text_) {
        return true;
    }
    !(_A_WEAR_STRETCH.is_match(&text_) || _THE_JOB.is_match(&text_))
}

/// True when the line names someone 19 or over: that person's cough is the adult one.
pub fn _a_grown_person_named(understood: &Understood) -> bool {
    let text_ = format!(" {} ", understood.clean.to_lowercase());
    for figure in _A_GROWN_AGE.findall_str(&text_) {
        if text::is_digit(&figure) {
            if figure.parse::<i64>().map(|n| n >= 19).unwrap_or(false) {
                return true;
            }
        } else if int_of(&_YEAR_WORDS, &figure).unwrap_or(0) >= 19 {
            return true;
        }
    }
    false
}

inline_re!(MONTHS_IN_DIGITS, r"\b(\d{1,2})\s*(?:month|months|mth|mo)\b");

/// The age in months the line names, or None: digits, or the words norm writes them as.
pub fn _months_old(text_: &str) -> Option<i64> {
    if let Some(m) = MONTHS_IN_DIGITS.search(text_) {
        return m.str(1).parse().ok();
    }
    _MONTH_WORDS.iter().find(|(name, _)| text_.contains(&format!(" {name} month"))).map(|(_, v)| *v)
}

/// The (digits, digits, word) groups of every `_AGE_IN_LINE` match.
fn ages_in_line(text_: &str) -> Vec<(String, String, String)> {
    _AGE_IN_LINE
        .finditer(text_)
        .iter()
        .map(|m| (m.str(1).to_string(), m.str(2).to_string(), m.str(3).to_string()))
        .collect()
}

/// True when a line says a person is ten or older, as digits or as the words norm keeps.
pub fn _an_age_ten_or_over(text_: &str, words: &[String]) -> bool {
    if _TEEN.is_match(text_) {
        return true;
    }
    for (first, second, word) in ages_in_line(text_) {
        let digits = sh::or(&first, &second);
        let age = if !digits.is_empty() { digits.parse().ok() } else { int_of(&_SMALL_NUMBERS, &word) };
        if let Some(age) = age {
            if age > *_SMALL_ENOUGH {
                return true;
            }
        }
    }
    for word in words {
        if _TEN_OR_OVER.contains(word.as_str()) {
            return true;
        }
        if text::is_digit(word) && word.parse::<i64>().map(|n| n > *_SMALL_ENOUGH).unwrap_or(false) {
            return true;
        }
    }
    false
}

/// True when the line this turn leans on is about a person of ten or over.
pub fn _the_chat_names_a_teen(_understood: &Understood) -> bool {
    // LEANS_ON: Python reads the leaned-on reading's `subject_type`, `text` and `words`; a
    // single-turn reading leans on nothing, which is False.
    false
}

/// False only for a grunt that belongs to a mouth rather than to a pair of lungs.
pub fn _a_grunt_is_a_breath_sign(understood: &Understood) -> bool {
    let text_ = format!(" {} ", understood.clean.to_lowercase());
    if !_GRUNT.is_match(&text_) {
        return true;
    }
    if _GRUNT_WITH_BREATH.is_match(&text_) {
        return true;
    }
    if _GRUNT_AS_AN_ANSWER.is_match(&text_) {
        return false;
    }
    if _SMALL_CHILD.iter().any(|w| text_.contains(w.as_str())) {
        return true;
    }
    for (first, second, word) in ages_in_line(&text_) {
        let age = if !first.is_empty() {
            first.parse().ok()
        } else if !second.is_empty() {
            second.parse().ok()
        } else if !word.is_empty() {
            int_of(&_SMALL_NUMBERS, &word)
        } else {
            None
        };
        if let Some(age) = age {
            return age <= *_SMALL_ENOUGH;
        }
    }
    !_the_chat_names_a_teen(understood)
}

/// True when the line puts the milk question on a baby over six months.
pub fn _an_older_baby_is_named(understood: &Understood) -> bool {
    let text_ = format!(" {} ", understood.clean.to_lowercase());
    if _A_YEAR_OR_MORE.iter().any(|p| text_.contains(p.as_str())) {
        return true;
    }
    if _MONTHS_AT_LEAST_SIX.iter().any(|p| text_.contains(p.as_str())) {
        return true;
    }
    matches!(_months_old(&text_), Some(m) if m >= 6)
}

/// The milk half of the answer belongs to the age on the line: solids, or cow's milk.
pub fn _the_feeds_for_the_age(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = format!(" {} ", understood.clean.to_lowercase());
    if _A_YEAR_OR_MORE.iter().any(|p| text_.contains(p.as_str())) {
        return Some(_MILK_FROM_A_YEAR.clone());
    }
    Some(_MILK_FROM_SIX_MONTHS.clone())
}

// ---------------------------------------------------------------------------------------------
// Friends, dates and hurt feelings.

/// The clubs answer, or the coffee-with-one-person answer when the line says work.
pub fn _making_friends(_memory: &Memory, reading: &Understood) -> Option<String> {
    let clean = sh::clean_or_text(reading).to_lowercase();
    let body = if _AT_WORK.is_match(&clean) { _FRIENDS_WORK.clone() } else { _FRIENDS_CLUBS.clone() };
    if _FRIENDS_FEELING.is_match(&clean) {
        return Some(format!("That's hard going, and it is not you being bad at it. {body}"));
    }
    Some(body)
}

/// The person's own lines on the turns before this one, as lower-case strings, newest first.
pub fn _the_talk_before(memory: &Memory) -> Vec<String> {
    let turns = &memory.turns;
    let mut lines: Vec<String> = turns[turns.len().saturating_sub(3)..].iter().map(|t| t.user.clone()).collect();
    let said = memory.extra.get("last_user").map(|v| v.str().to_string()).unwrap_or_default();
    if !said.is_empty() && lines.last().map(|l| l != &said).unwrap_or(true) {
        lines.push(said);
    }
    lines.iter().rev().map(|l| l.to_lowercase()).collect()
}

/// "likes", "doesnt" or "" - read from the newest line in the talk that says either.
pub fn _the_feeling_before(memory: &Memory) -> &'static str {
    for line in _the_talk_before(memory) {
        if _NOT_A_LIKING.is_match(&line) {
            return "doesnt";
        }
        if _NAMES_A_LIKING.is_match(&line) {
            return "likes";
        }
    }
    ""
}

/// The idea for a date out, the one for a night in, or the one for an occasion.
pub fn _date_idea(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if _AT_HOME_WORDS.iter().any(|w| clean.contains(w.as_str())) {
        return Some(_DATE_AT_HOME.clone());
    }
    if _OCCASION_WORDS.iter().any(|w| clean.contains(w.as_str())) {
        return Some(_DATE_OCCASION.clone());
    }
    Some(_DATE_OUT.clone())
}

/// The date script, or nothing when a say-to line has no one they like behind it.
pub fn _asking_out(memory: &Memory, reading: &Understood) -> Option<String> {
    let line = sh::text_or_clean(reading).to_lowercase();
    if !_SAY_TO_SOMEONE.is_match(&line) {
        return Some(_THE_ASKING_OUT.clone());
    }
    match _the_feeling_before(memory) {
        "likes" => Some(_THE_ASKING_OUT.clone()),
        "doesnt" => Some(_THE_NOT_WANTING.clone()),
        _ => None,
    }
}

/// The words to say, after taking the person's side when they only reported it.
pub fn _hurt_feelings(_memory: &Memory, reading: &Understood) -> Option<String> {
    if reading.kind == "statement" && reading.asks.is_empty() {
        return Some(format!("That's not on, and it is fair to feel stung. {}", *_HURT_FEELINGS));
    }
    Some(_HURT_FEELINGS.clone())
}

// ---------------------------------------------------------------------------------------------
// A child's weight, school years and milestones.

/// `_child_weight.<locals>.has`: whether the line names one of the phrases as whole words.
#[allow(non_snake_case)]
pub fn _child_weight__has(clean: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| sh::re_search(&format!(r"\b{}\b", sh::escape(p)), clean, 0))
}

inline_re!(
    WEIGHING_SCALES,
    r"\bscales?\b|\b(?:what|which|how do i) (?:should|do|can) i weigh\b|\bweigh (?:her|him|them|the baby|my baby) on\b"
);
inline_re!(A_BABY_WORD, r"\b(?:baby|babies|newborn|newborns|infant|infants)\b");
inline_re!(AN_AGE_IN_YEARS, r"\b(\d{1,2})\s*(?:year|years|yr|yrs|yo|y\.o)");

/// `CHILD_WEIGHT_BY_AGE`: (the oldest age, the usual range said).
static CHILD_WEIGHT_BY_AGE: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| {
    g("CHILD_WEIGHT_BY_AGE").items().iter().map(|p| (p.items()[0].as_int().unwrap_or(0), p.items()[1].str().to_string())).collect()
});

/// A child's weight: the GP plots the chart, and a diet is never the answer.
pub fn _child_weight(_memory: &Memory, reading: &Understood) -> Option<String> {
    let clean = reading.clean.to_lowercase();
    let has = |phrases: &[&str]| _child_weight__has(&clean, phrases);
    if GROWTH_SPURT.is_match(&clean) {
        return None;
    }
    if ANNOUNCEMENT_WEIGHT.is_match(&clean)
        && !CONCERNED_WEIGHT.is_match(&clean)
        && !has(&["too thin", "too skinny", "off their food", "not eating", "wont eat", "small for", "skinny"])
    {
        if QUESTION_WEIGHT.is_match(&clean) {
            return Some(BIRTH_WEIGHT.clone());
        }
        return None;
    }
    if WEIGHING_SCALES.is_match(&clean) {
        return Some(
            "Use one set of scales and keep to it: the clinic's or the health visitor's are the \
ones that count, because two scales can differ by a few hundred grams. At home, \
weigh them in a clean nappy or naked, at the same time of day - before a feed \
rather than after - and once a week is plenty, not daily. Write the weight and \
the date in the red book and let the person who weighs them read the line, \
rather than comparing this Tuesday with the last one."
                .into(),
        );
    }
    let not_gaining = NOT_GAINING.is_match(&clean);
    let baby = A_BABY_WORD.is_match(&clean);
    if baby
        && !not_gaining
        && !has(&["too", "overweight", "too heavy", "heavy for"])
        && has(&["putting on weight", "put on weight", "putting weight on", "gaining weight", "gained weight", "is gaining", "has gained"])
    {
        return Some(
            "A baby who is putting on weight is doing what a baby should, and the line in the \
red book settles it: weighed at the clinic or by the health visitor, the gain is \
read against their own curve rather than any number. Bring them in sooner if the \
gain has flattened off, if they have fewer than six wet nappies a day, or if they \
are too sleepy to feed properly."
                .into(),
        );
    }
    if has(&[
        "thinks she is fat", "thinks he is fat", "thinks they are fat", "thinks she is too fat", "thinks he is too fat",
        "says she is fat", "says he is fat", "says they are fat", "said she is fat", "said he is fat", "said they are fat",
        "says she is too fat", "says he is too fat", "says she feels fat", "says he feels fat", "says she looks fat",
        "says he looks fat", "calls her fat", "calls him fat", "called her fat", "called him fat", "kids call her fat",
        "kids call him fat", "kids call them fat", "kids call her fat", "bullied about her weight", "bullied about his weight",
        "teased about her weight", "teased about his weight", "hates her body", "hates his body", "worried about her weight",
        "worried about his weight", "body image", "anorexic", "anorexia", "bulimic", "bulimia", "eating disorder",
        "eating disorders", "skipping meals", "making herself sick", "making himself sick", "starving herself",
        "starving himself", "afraid of getting fat", "scared of getting fat",
    ]) {
        return Some(
            "Don't argue them out of the feeling and don't put them on a diet: say you are glad \
they told you, ask what they would like to change, and keep meals regular and eaten \
together. Talk about what food does rather than what it weighs, and keep the \
conversation off their body. Book the GP this week if they are skipping meals, \
cutting out whole food groups, making themselves sick or losing weight, because \
early help works far better and the GP can refer them to a specialist service."
                .into(),
        );
    }
    if has(&[
        "how much should", "how heavy", "should weigh", "weight for a", "normal weight", "average weight", "healthy weight",
        "what is a normal weight", "right weight", "good weight", "should a", "is a normal",
    ]) {
        let mut usual = String::new();
        if let Some(found) = AN_AGE_IN_YEARS.search(&clean) {
            let age: i64 = found.str(1).parse().unwrap_or(0);
            if let Some((_, range_said)) = CHILD_WEIGHT_BY_AGE.iter().find(|(top, _)| age <= *top) {
                usual = format!("A {age} year old is usually somewhere between {range_said}, ");
            }
        }
        let lead = if !usual.is_empty() {
            usual
        } else {
            "There is no single right number - a 7 year old is usually between about 18 and \
28 kilos, a 10 year old between 25 and 40 - "
                .to_string()
        };
        return Some(
            lead + "but the line on the growth chart matters far more than any figure, because \
children grow at different speeds. The GP or health visitor plots that line. If the \
weight has dropped or they are not following their own curve, that is worth an \
appointment rather than a change of food.",
        );
    }
    if !not_gaining
        && has(&[
            "overweight", "too fat", "too heavy", "heavy for his age", "heavy for her age", "heavy for their age",
            "putting on weight", "put on weight", "gaining weight", "gained weight", "piling on the pounds",
        ])
    {
        return Some(
            "Don't put a child on a diet: the aim is to grow into the weight rather than shrink \
it. Keep meals and two snacks on a routine, swap sugary drinks and juice for water \
or milk, keep portions child-sized (about a handful) and let them run about for an \
hour a day; weighing food and counting calories backfires at this age. Ask the GP or \
school nurse to plot the growth chart if the weight has jumped in a few months, \
because that is what shows whether there is anything to do."
                .into(),
        );
    }
    Some(
        "See the GP or health visitor rather than changing their food: a child who is losing \
weight or not growing needs the reason found, and the growth chart is what shows it. \
While you wait, add calories where they are cheap - full-fat milk, cheese, yoghurt, \
peanut butter on toast, eggs - and add an extra snack rather than bigger meals. Ask for \
an urgent appointment if they are also weeing a lot or very thirsty, or having loose \
stools, because that is not a phase."
            .into(),
    )
}

/// Whether a line names a child, young or grown, by a word, a pronoun or an age.
pub fn _a_child_named(text_: &str) -> bool {
    let said = format!(" {} ", text::squash(&text_.to_lowercase().replace("'s", " ")));
    if _A_CHILD_AGE.is_match(&said) {
        return true;
    }
    _A_CHILD_WORD.iter().any(|w| said.contains(&format!(" {w} ")))
}

/// Whether a user line in the last few turns names a child.
pub fn _a_child_in_the_chat(memory: &Memory) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(4)..].iter().any(|t| _a_child_named(&t.user))
}

inline_re!(SECONDARY_SCHOOL, r"secondary|high school");

/// `SCHOOL_YEARS`: year -> (low age, high age).
static SCHOOL_YEARS: LazyLock<Vec<(i64, (i64, i64))>> = LazyLock::new(|| {
    g("SCHOOL_YEARS")
        .pairs()
        .iter()
        .map(|(k, v)| (k.as_int().unwrap_or(0), (v.items()[0].as_int().unwrap_or(0), v.items()[1].as_int().unwrap_or(0))))
        .collect()
});
static SCHOOL_YEAR_NOTE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("SCHOOL_YEAR_NOTE"));

/// Going up a school year: the ages that year means and how to start it well.
pub fn _the_year_they_move_up_to(_memory: &Memory, understood: &Understood) -> Option<String> {
    let typed = understood.text.to_lowercase();
    let mut year: Option<i64> = _THE_YEAR_NAMED.search(&typed).and_then(|m| m.str(1).parse().ok());
    if year.is_none() && SECONDARY_SCHOOL.is_match(&typed) {
        year = Some(7);
    }
    let Some(year) = year else {
        return Some(
            "Going up a year is mostly about the run-up: walk the route once before term, \
check nothing has been outgrown, and move the sleep time back a week early. On \
the first morning tell them when you will see them again and be there on time - \
the wobble usually comes at the gate, not in the room."
                .into(),
        );
    };
    let (low, high) = SCHOOL_YEARS.iter().find(|(y, _)| *y == year).map(|(_, r)| *r).unwrap_or((0, 0));
    let key = if year <= 3 {
        "reading"
    } else if year <= 6 {
        "work"
    } else if year <= 9 {
        "building"
    } else {
        "exams"
    };
    let note = SCHOOL_YEAR_NOTE.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())?;
    Some(format!(
        "Year {year} is ages {low} and {high}. {note} Before term starts, walk the route once, check \
nothing has been outgrown, and move the sleep time back a week early. On the first \
morning tell them when you will see them again and be there on time: the wobble \
usually comes at the gate, not in the room. If they are nervous, ask what they \
think will be hard rather than telling them it will be fine."
    ))
}

/// `_MILESTONE_SKILLS`: each skill's clues and the sentence the card opens on.
static _MILESTONE_SKILLS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    g("_MILESTONE_SKILLS").items().iter().map(|p| (p.items()[0].strs(), p.items()[1].str().to_string())).collect()
});
static _MILESTONE_REST: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_MILESTONE_REST"));

/// The milestones card, opening on the skill the line asked about.
pub fn _milestones_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    let mut said = sh::clean_or_text(understood).to_lowercase();
    if said.is_empty() {
        said = memory.extra.get("last_user").map(|v| v.str().to_lowercase()).unwrap_or_default();
    }
    if !_a_child_named(&said) && !_A_WHEN_ASK.is_match(&said) && !_a_child_in_the_chat(memory) {
        return None;
    }
    for (clues, opening) in _MILESTONE_SKILLS.iter() {
        if clues.iter().any(|c| said.contains(c.as_str())) {
            let first = clues.first().map(String::as_str).unwrap_or("");
            for (key, rest) in _MILESTONE_REST.iter() {
                if first.contains(key.as_str()) || key.contains(first) {
                    if !rest.is_empty() {
                        return Some(format!("{opening}. {rest} {}", *_MILESTONE_TAIL));
                    }
                }
            }
            return Some(format!("{opening}. {}", *_MILESTONE_TAIL));
        }
    }
    Some(format!("{} {}", *_MILESTONE_ORDER, *_MILESTONE_TAIL))
}

/// `HURTING_WORDS`: each behaviour's spellings, its -ing form and the words for the moment.
static HURTING_WORDS: LazyLock<Vec<(Vec<String>, String, String)>> = LazyLock::new(|| {
    g("HURTING_WORDS")
        .items()
        .iter()
        .map(|p| (p.items()[0].strs(), p.items()[1].str().to_string(), p.items()[2].str().to_string()))
        .collect()
});

/// The behaviour the parent named, as (its -ing form, the words to name it in the moment).
pub fn _the_hurting_word(understood: &Understood) -> (String, String) {
    let mut words: HashSet<String> = HashSet::new();
    for one in [&understood.clean, &understood.text, &understood.norm_text] {
        if !one.is_empty() {
            let cleaned = NOT_A_BITE.sub(" ", &one.to_lowercase());
            words.extend(az_list(&cleaned).into_iter().map(str::to_string));
        }
    }
    for (spellings, naming, in_the_moment) in HURTING_WORDS.iter() {
        if spellings.iter().any(|s| words.contains(s)) {
            return (naming.clone(), in_the_moment.clone());
        }
    }
    ("hitting".into(), "'No - hitting hurts'".into())
}

/// The one card for a small child who hurts somebody, worded for what they did.
pub fn _hitting_or_biting(_memory: &Memory, understood: &Understood) -> Option<String> {
    let (naming, in_the_moment) = _the_hurting_word(understood);
    Some(format!(
        "Stop it in the moment, every time, and name it: {in_the_moment}, then take them \
away from the other child for a minute and help them say what they wanted ('you \
wanted the truck'). The {naming} is a shortcut for a word they don't have yet, \
so give them the sentence and praise it the first time. Never do it back: it teaches \
that the biggest person wins."
    ))
}

// ---------------------------------------------------------------------------------------------
// A late talker, and a person who has gone quiet.

/// The child's age in years as a float, or None: '18 months', '2 and a half', 'my son is 2'.
/// `strict` is for a line on an earlier turn: it wants the age said as an age.
pub fn _the_talker_age(text_: &str, strict: bool) -> Option<f64> {
    let said = format!(" {} ", text_.to_lowercase());
    if strict && !_TALK_CHILD_WORD.is_match(&said) {
        return None;
    }
    if let Some(m) = _TALK_MONTHS.search(&said) {
        return Some(m.str(1).parse::<f64>().ok()? / 12.0);
    }
    if let Some(m) = _TALK_WORD_HALF.search(&said) {
        return Some(int_of(&_AGE_WORDS, m.str(1))? as f64 + 0.5);
    }
    if let Some(m) = _TALK_WORD_AGE.search(&said) {
        return Some(int_of(&_AGE_WORDS, m.str(1))? as f64);
    }
    if let Some(m) = _TALK_YEARS.search(&said) {
        for group in m.groups().into_iter().flatten() {
            if !group.is_empty() {
                let mut value: f64 = group.parse().ok()?;
                if said.contains("and a half") || said.contains("half") {
                    value += 0.5;
                }
                return Some(value);
            }
        }
    }
    None
}

/// The milestones card answers a child's speech, not a person who is not speaking to anyone.
pub fn _a_childs_missing_speech_is_the_question(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { understood.words.join(" ") } else { understood.clean.clone() };
    let said = format!(" {} ", line.to_lowercase());
    if !A_TALK_WORD.is_match(&said) {
        return false;
    }
    if _TALKER_SPEECH_NOUN.is_match(&said) {
        return true;
    }
    if _the_talker_age(&understood.text, true).is_some() {
        return true;
    }
    !_TALKER_AIMED_AT_A_PERSON.is_match(&said)
}

/// True when the line, or the turn it leans on, is about a child who stopped talking to them.
pub fn _a_silence_with_a_person(understood: &Understood, memory: &Memory) -> bool {
    if A_A_SILENCE_WITH_A_PERSON.is_match(&understood.text) {
        return true;
    }
    _the_talk_before(memory).iter().any(|line| A_A_SILENCE_WITH_A_PERSON.is_match(line))
}

/// The late-talker card, worded for the age the parent named, here or on the turn before.
pub fn _late_talker_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    if dup::a_repair_is_open(memory) {
        return Some(String::new());
    }
    let said = sh::clean_or_text(understood).to_lowercase();
    if !said.is_empty() && _ASKING_HOW_TO_TALK.is_match(&said) {
        return Some(format!("{} {}", *_TALKER_HELP, _the_talker_ages(memory, understood)));
    }
    Some(_the_talker_ages(memory, understood))
}

/// The ages and the referral, worded for the age named here or on the turn before.
pub fn _the_talker_ages(memory: &Memory, understood: &Understood) -> String {
    let mut age = _the_talker_age(&understood.text, false);
    if age.is_none() {
        for line in _the_talk_before(memory) {
            age = _the_talker_age(&line, true);
            if age.is_some() {
                break;
            }
        }
    }
    let Some(age) = age else {
        return _TALKER_NO_AGE.clone();
    };
    if age >= *_A_LATE_TALKER_IS_UNDER && _a_silence_with_a_person(understood, memory) {
        return String::new();
    }
    if age < 2.0 {
        return _TALKER_UNDER_TWO.clone();
    }
    if age < 3.0 {
        return _TALKER_TWO.clone();
    }
    if age < 4.0 {
        return _TALKER_THREE.clone();
    }
    _TALKER_SCHOOL.clone()
}

/// The quiet card, worded for the grown-up or the child the chat is about.
pub fn _wont_talk_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    let mut said: Vec<String> = vec![understood.text.clone()];
    // LEANS_ON: the leaned-on reading's subject and text would be added here.
    if let Some(Value::Dict(_)) = memory.extra.get("user") {
        if let Some(Value::List(people)) = memory.extra.get("user").and_then(|u| u.get("people")) {
            said.extend(people.iter().filter_map(|p| p.as_str().map(str::to_string)));
        }
    }
    let mut words: HashSet<String> = HashSet::new();
    for one in &said {
        words.extend(az_list(&one.to_lowercase()).into_iter().map(str::to_string));
    }
    let any = |set: &HashSet<String>| words.iter().any(|w| set.contains(w));
    if any(&_CHILD_QUIET_RELATIONS) {
        return Some(_QUIET_CHILD_REPLY.clone());
    }
    if any(&_FRIEND_QUIET_RELATIONS) && !any(&_HOUSEHOLD_QUIET_RELATIONS) {
        let said_text = said.join(" ").to_lowercase();
        let drifted = _FRIEND_DRIFT_WORDS.iter().any(|w| said_text.contains(w.as_str()));
        let worried = _FRIEND_WORRY_WORDS.iter().any(|w| said_text.contains(w.as_str()));
        if drifted && !worried {
            let moved = _FRIEND_MOVED_WORDS.iter().any(|w| said_text.contains(w.as_str()));
            let reason = if moved { &*_FRIEND_MOVED_REASON } else { &*_FRIEND_QUIET_REASON };
            return Some(_QUIET_FRIEND_REPLY.replace("{reason}", reason));
        }
    }
    if any(&_GROWN_QUIET_RELATIONS) {
        return Some(_QUIET_GROWN_REPLY.clone());
    }
    Some(_QUIET_CHILD_REPLY.clone())
}

// ---------------------------------------------------------------------------------------------
// Feeding: solids, milk, caffeine and the fussy eater.

/// True unless the age in the line is a baby still on milk alone.
pub fn _a_child_old_enough_to_eat(understood: &Understood) -> bool {
    for number in &understood.numbers {
        let Some(crate::understood::Num::Int(value)) = &number.value else {
            continue;
        };
        let unit = number.unit.as_deref().unwrap_or("").to_lowercase();
        if unit.starts_with("week") {
            return false;
        }
        if unit.starts_with("month") && *value < 6 {
            return false;
        }
    }
    true
}

/// The pumping room's own question, with the machine pump kept out of it.
pub fn _a_milk_pump_question(understood: &Understood) -> bool {
    _not_a_machine_pump(understood) && _a_milk_question(understood)
}

/// False when the line says nothing about milk: "back at work" is then not this answer.
pub fn _a_milk_question(understood: &Understood) -> bool {
    let mut words: HashSet<String> = HashSet::new();
    for part in [&understood.clean, &understood.text] {
        words.extend(az_list(&part.to_lowercase()).into_iter().map(str::to_string));
    }
    for list in [&understood.norm, &understood.words] {
        words.extend(list.iter().map(|w| w.to_lowercase()));
    }
    meets(words.iter().map(String::as_str), &_MILK_SAID)
}

/// The line names a drink, so the caffeine limit is its answer and a plate of food is not.
pub fn _a_caffeine_drink_was_named(understood: &Understood) -> bool {
    let norm = if understood.norm.is_empty() { String::new() } else { py_list_str(&understood.norm) };
    let mut words: HashSet<String> = HashSet::new();
    for part in [&understood.clean, &understood.text, &norm] {
        words.extend(az_list(&part.to_lowercase()).into_iter().map(str::to_string));
    }
    meets(words.iter().map(String::as_str), &_CAFFEINE_SAID)
}

/// The fussy-eating answer, or None when the line is a dinner with one food ruled out.
pub fn _picky_eater(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = sh::or(&understood.clean, &understood.norm_text);
    let cook = understood.words.iter().any(|w| _COOK_ASK.contains(w.as_str()));
    if cook && !dup::cooking_refused_foods(text_, false).is_empty() {
        return None;
    }
    Some(PICKY_EATER.clone())
}

// ---------------------------------------------------------------------------------------------
// An older parent: a stroke, and an appetite that has gone.

static _STROKE_EATING_SET: LazyLock<HashSet<String>> = LazyLock::new(|| _STROKE_EATING.iter().cloned().collect());

/// The map of stroke care, or the eating plan when the line is about food.
pub fn _stroke_reply(memory: &Memory, reading: &Understood) -> Option<String> {
    let words: HashSet<String> = reading.words.iter().map(|w| w.to_lowercase()).collect();
    let mut text_ = sh::clean_or_text(reading).to_lowercase();
    if text_.is_empty() {
        text_ = memory.understood.as_ref().map(|u| u.clean.to_lowercase()).unwrap_or_default();
    }
    let said = text_
        .split(|c: char| !c.is_ascii_lowercase())
        .filter(|w| !w.is_empty())
        .any(|w| _STROKE_EATING_SET.contains(w))
        || words.iter().any(|w| _STROKE_EATING_SET.contains(w));
    if said {
        return Some(_STROKE_EATING_REPLY.clone());
    }
    Some(
        "Frightening to watch, and the first day or two is tests rather than answers: a scan to \
say what kind of stroke it was, and a check that they can swallow before anything to \
eat or drink. Ask which stroke unit they are on and when the physio, speech and \
occupational therapist will see them. Week one \
tells you very little about the year, so take it a day at a time and talk to them \
normally, as an adult."
            .into(),
    )
}

inline_re!(
    GONE_OFF_FOOD,
    r"\b(?:gone|went)\s+off\s+(?:her|his|their|its)?\s*(?:own\s+)?(?:food|eating|meals)\b|\b(?:off|gone off)\s+(?:her|his|their)\s+food\b"
);

/// A parent who has stopped eating: the reason names the answer, and one branch per reason.
pub fn _elderly_appetite(_memory: &Memory, reading: &Understood) -> Option<String> {
    let clean = sh::clean_or_text(reading).to_lowercase();
    if _APPETITE_AFTER.is_match(&clean) {
        return Some(format!(
            "An appetite that goes after an operation or a stay in hospital usually comes \
back over two to four weeks, so feed it in small amounts meanwhile: small \
plates every couple of hours, full-fat milk and cheese and butter rather than \
'healthy' options, and whatever they actually fancy rather than what they ought \
to eat. {} Ring the ward or the GP if they are also drinking \
very little, are sleepy or confused, or the weight is still going down after a \
month - that is a check-up, not a phase.",
            *_APPETITE_ASK
        ));
    }
    if _APPETITE_WEIGHT.is_match(&clean) {
        return Some(format!(
            "Losing weight without trying needs looking at rather than persuading - in an \
older person it is worth a GP appointment for a blood test and a review of the \
medicines, because the cause is often something fixable like a thyroid problem, \
a urine infection or a drug that kills the appetite. While that is happening, \
feed them the way you would build someone up: little and often, full-fat \
everything, and a pudding made with cream. {}",
            *_APPETITE_ASK
        ));
    }
    if _APPETITE_TASTE.is_match(&clean) {
        return Some(format!(
            "Food that has stopped tasting of anything is usually the medicines or a dry \
mouth rather than age, so ask the GP or pharmacist to review what they are on - \
some drugs dull taste and some cause a metallic taste that settles after a few \
weeks. Meanwhile season harder than you would for yourself, use cold or \
room-temperature food (cold dulls flavour less), sharp things like lemon, \
vinegar, pickles and herbs, and check that dentures fit, since sore gums are a \
common reason for going off food quietly. {}",
            *_APPETITE_ASK
        ));
    }
    if _APPETITE_ELDERLY.is_match(&clean) {
        return Some(format!(
            "An older person who has gone off their food needs tempting more than \
instructing: small plates of whatever they actually like, offered often, with \
no comment about how much is left. {} {}",
            *_APPETITE_FOOD, *_APPETITE_ASK
        ));
    }
    if GONE_OFF_FOOD.is_match(&clean) {
        return Some(format!(
            "Going off food like that is usually the illness or the worry rather than the \
food itself, and it comes back as that settles: offer small amounts of whatever \
they fancy every couple of hours rather than a plate to finish, and keep the \
drinks for between meals so they do not fill the space a meal would have taken. \
Ask the GP if it lasts more than a fortnight, or if they are also losing weight \
or drinking very little. {}",
            *_APPETITE_ASK
        ));
    }
    None
}

// ---------------------------------------------------------------------------------------------
// Endings, let-downs, visits and rows.

/// False when the line says the ending was a while back, so the first-hours card stands down.
pub fn _not_some_time_ago(understood: &Understood) -> bool {
    // "raw" is not a key the reading has
    !_SOME_TIME_AGO.is_match(&text::squash(&clean_or_words(understood).to_lowercase()))
}

inline_re!(
    THE_FRONT_OF_WHAT_IS_GOT_OVER,
    r"^[\s,.!?]*(?:(?:a|an|the|my|your|his|her|their|our|this|that|these|those|being|having|all|it|about|with|from|over|of)\b[\s,.]*)*",
    crate::py::re::I
);

/// Whether the thing being got over is a person, or a bug.
pub fn _a_person_is_what_they_get_over(understood: &Understood) -> bool {
    let line = sh::clean_or_text(understood);
    let words = understood.words.join(" ");
    let text_ = if line.is_empty() { words.as_str() } else { line };
    let Some(over) = _GETTING_OVER.search(text_) else {
        return true;
    };
    if _AN_ENDING_OR_A_PERSON.is_match(text_) || _A_PERSON_ALREADY_THERE.is_match(text_) {
        return true;
    }
    let rest = THE_FRONT_OF_WHAT_IS_GOT_OVER.subn("", &text_[over.end()..], 1);
    let rest = text::strip_chars(&rest, " ?!.,");
    let lowered = rest.to_lowercase();
    let head = az_list(&lowered).first().copied().unwrap_or("");
    const KEEPS: [&str; 30] = [
        "", "it", "this", "that", "them", "him", "her", "what", "everything", "anything", "all", "which", "whom",
        "people", "someone", "somebody", "badly", "now", "situation", "past", "behaved", "guilt", "guilty", "shame",
        "regret", "myself", "how", "why", "whole", "the",
    ];
    if KEEPS.contains(&head) {
        return true;
    }
    let refs = &understood.reference;
    for who in [&refs.it, &refs.that, &refs.she, &refs.he, &refs.they] {
        let who = who.as_deref().unwrap_or("");
        if !who.is_empty() && (_AN_ENDING_OR_A_PERSON.is_match(who) || _A_PERSON_ALREADY_THERE.is_match(who)) {
            return true;
        }
    }
    false
}

/// Whether somebody, rather than a thing, is the one doing the letting down.
pub fn _a_person_is_the_one_letting_you_down(understood: &Understood) -> bool {
    let lowered;
    let words: Vec<&str> = if !understood.norm.is_empty() {
        understood.norm.iter().map(String::as_str).collect()
    } else {
        lowered = sh::clean_or_text(understood).to_lowercase();
        az_list(&lowered)
    };
    for (at, word) in words.iter().enumerate() {
        if !_LET_DOWN.contains(*word) {
            continue;
        }
        let after = &words[(at + 1).min(words.len())..(at + 4).min(words.len())];
        if after.is_empty() || !["me", "us"].contains(&after[0]) || !after.contains(&"down") {
            continue;
        }
        let mut back = at as i64 - 1;
        while back >= 0 && _LET_DOWN_SKIPPED.contains(words[back as usize]) {
            back -= 1;
        }
        if back >= 0 && !_LET_DOWN_PEOPLE.contains(words[back as usize]) {
            return false;
        }
    }
    true
}

/// True when the visit is somebody coming to them, which is what the hosting card answers.
pub fn _the_visit_card_fits(understood: &Understood) -> bool {
    let line = clean_or_norm_or_text(understood);
    let typed = understood.text.as_str();
    if _A_FOOD_LIMIT.is_match(&line) {
        return false;
    }
    if _A_VISIT_AS_A_GRIEVANCE.is_match(&line) {
        return false;
    }
    if AN_HOUR.is_match(&line) && !A_LENGTH_OF_STAY.is_match(&line) {
        return false;
    }
    !(_A_VISIT_I_AM_MAKING.is_match(&line) || _A_VISIT_I_AM_MAKING.is_match(typed))
}

/// False when the speaker is one of the two people in the row.
pub fn _i_am_not_one_of_the_two(understood: &Understood) -> bool {
    let line = clean_or_norm_or_text(understood);
    if line.is_empty() {
        return true;
    }
    if _I_ASK_TO_STOP_THE_ROW.is_match(&line) {
        return !(_A_GROWN_UP_IN_THE_ROW.is_match(&line) && !_A_CHILD_IN_THE_ROW.is_match(&line));
    }
    !_I_AM_IN_THE_ROW.is_match(&line)
}

/// Which of the two repairs: the shout's, or the hitting one that has its own opening.
pub fn _the_repair_after_it(_memory: &Memory, understood: &Understood) -> Option<String> {
    if _SMACKED_A_CHILD.is_match(sh::clean_or_text(understood)) {
        return Some(THE_REPAIR_AFTER_HITTING.clone());
    }
    Some(THE_REPAIR_AFTER_SHOUTING.clone())
}

/// True when the line says what a parent did to a child, after the row, rather than asking.
pub fn _told_them_off(understood: &Understood) -> bool {
    _TOLD_THEM_OFF.is_match(&clean_or_norm_or_text(understood))
}

/// True unless the line says one parent is tired and nothing else names the hard parent.
pub fn _the_parent_is_not_only_tired(understood: &Understood) -> bool {
    let line = text::squash(&sh::clean_or_text(understood).to_lowercase());
    if _A_HARD_PARENT.is_match(&line) {
        return true;
    }
    if _TIRED_BECAUSE_OF_SOMEONE.is_match(&line) {
        return true;
    }
    !_ONE_PARENT_IS_TIRED.is_match(&line)
}

/// Whether a "what do i say to her" is an asking rather than a hard conversation.
pub fn _not_a_hard_conversation(understood: &Understood) -> bool {
    let line = sh::clean_or_text(understood);
    let words = understood.words.join(" ");
    let said = if line.is_empty() { words.as_str() } else { line };
    if !_WORDS_FOR_SOMEONE.is_match(said) {
        return true;
    }
    if _A_PERSON_ALREADY_THERE.is_match(said) {
        return false;
    }
    let refs = &understood.reference;
    for who in [&refs.she, &refs.he, &refs.they, &refs.it] {
        let who = who.as_deref().unwrap_or("");
        if !who.is_empty() && _A_PERSON_ALREADY_THERE.is_match(who) {
            return false;
        }
    }
    true
}

/// True when the line is a visit the speaker is making, which is a trip to plan.
pub fn _the_trip_i_am_making(understood: &Understood) -> bool {
    let line = clean_or_norm_or_text(understood);
    let typed = understood.text.as_str();
    if !(_A_VISIT_I_AM_MAKING.is_match(&line) || _A_VISIT_I_AM_MAKING.is_match(typed)) {
        return false;
    }
    !_A_TRIP_STILL_BEING_WEIGHED.is_match(&line)
}

/// True unless the line is a visit somebody is cooking for, which is the kitchen's question.
pub fn _not_a_meal_the_visit_is_for(understood: &Understood) -> bool {
    let text_ = sh::text_or_clean(understood);
    if text_.is_empty() {
        return true;
    }
    if !A_MEAL_NAMED.is_match(text_) {
        return true;
    }
    A_RATE_OR_A_GRIEVANCE.is_match(text_) || A_NAMED_DAY_RATE.is_match(text_)
}

/// True unless the line is a trip the speaker is taking, which is not a boundary.
pub fn _not_a_visit_i_am_making(understood: &Understood) -> bool {
    let text_ = sh::text_or_clean(understood);
    if text_.is_empty() {
        return true;
    }
    if !_A_VISIT_I_AM_MAKING.is_match(text_) {
        return true;
    }
    A_RATE_OR_A_GRIEVANCE.is_match(text_)
}

/// True unless the line is a present to buy for a parent, which this row does not answer.
pub fn _not_asking_for_a_present_for_them(understood: &Understood) -> bool {
    let text_ = sh::text_or_clean(understood);
    if text_.is_empty() {
        return false;
    }
    !(ASKS_FOR_A_PRESENT.is_match(text_) && !THEY_BOUGHT_IT.is_match(text_))
}

/// True when the line (or the line it follows) names the home or a visit to it.
pub fn _the_home_or_a_visit_is_named(understood: &Understood) -> bool {
    let text_ = sh::text_or_clean(understood);
    // LEANS_ON: the leaned-on line's text would be read beside this one; absent on one turn.
    let above = "";
    if text_.is_empty() && above.is_empty() {
        return true;
    }
    A_HOME_OR_A_VISIT.is_match(&format!("{text_} {above}"))
}

/// True only for a friend's birthday framed as a date, not as a slight.
pub fn _a_friend_birthday_in_the_diary(understood: &Understood) -> bool {
    let words = &understood.norm;
    let has = |w: &str| words.iter().any(|x| x == w);
    if !has("birthday") {
        return false;
    }
    if ["forgot", "forgotten", "missed"].iter().any(|w| has(w)) {
        return false;
    }
    if ["was", "were", "yesterday", "last"].iter().any(|w| has(w)) {
        return false;
    }
    words
        .windows(2)
        .any(|p| ["my", "our"].contains(&p[0].as_str()) && ["mate", "mates", "friend", "friends"].contains(&p[1].as_str()))
}

inline_re!(
    SOMEONE_ELSE_FORGETS,
    r"\b(?:he|she|they|hubby|husband|wife|partner|boyfriend|girlfriend|mum|mom|dad|mother|father|son|daughter|brother|sister|mate|friend|boss|workmate)\s+(?:keeps?|always|never|does not|doesn't|did not|didn't|never ever)\s+(?:remember|forget|forgot|forgotten)"
);

/// True unless the line says somebody else is the one who forgets the date.
pub fn _the_person_forgets_it_themselves(understood: &Understood) -> bool {
    let mut text_ = text::squash(&sh::clean_or_text(understood).to_lowercase());
    if text_.is_empty() {
        text_ = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    }
    !SOMEONE_ELSE_FORGETS.is_match(&text_)
}

/// False when the line is not somebody working out what to say to a child about money.
pub fn _a_child_is_being_told(understood: &Understood) -> bool {
    let mut low = format!(" {} ", understood.words.join(" ")).to_lowercase();
    if text::strip(&low).is_empty() {
        low = format!(" {} ", understood.text.to_lowercase());
    }
    if low.contains(" tell if ") || low.contains(" tell whether ") || low.contains(" tell when ") {
        return false;
    }
    if !_TELLING_A_CHILD.is_match(&low) {
        return false;
    }
    A_CHILD_MONEY_TROUBLE.iter().any(|w| low.contains(&format!(" {w} ")) || low.contains(&format!(" {w}")))
}

/// True when the line is about another person's drinking rather than the speaker's own.
pub fn _names_someone_else(understood: &Understood) -> bool {
    understood.words.iter().any(|w| _SOMEONE_ELSE.contains(sh::strip_punct(&w.to_lowercase())))
}

/// False for a line asking who to go to about something, which is not small talk.
pub fn _a_route_ask_not_an_opener(understood: &Understood) -> bool {
    !WHO_AT_THE_FRONT.is_match(&text::squash(&understood.clean))
}

/// `" ".join(str(norm_text or plain or text or "").lower().split())`.
fn norm_plain_text(understood: &Understood) -> String {
    let t = sh::or(sh::or(&understood.norm_text, &understood.plain), &understood.text);
    text::squash(&t.to_lowercase())
}

/// True when the line reports a child having eaten the sweet things, not asking what to give.
pub fn _a_child_ate_the_lot(understood: &Understood) -> bool {
    let text_ = norm_plain_text(understood);
    _A_CHILD_IN_THE_LINE.is_match(&text_) && _CHILD_ATE.is_match(&text_) && _A_SWEET_THING.is_match(&text_)
}

/// False when the line is asking how to say something, so the bed-and-shopping card stands down.
pub fn _not_an_ask_for_the_words_to_say(understood: &Understood) -> bool {
    !ASKING_FOR_THE_WORDS.is_match(&norm_plain_text(understood))
}

// ---------------------------------------------------------------------------------------------
// Drinks while feeding, visiting hours, bottles, money and readings.

/// The drinks card, with the caffeine sentence only when the line named a drink of its own.
pub fn _a_drink_while_feeding(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words: Vec<String> = understood.words.iter().map(|w| sh::strip_punct(&w.to_lowercase()).to_string()).collect();
    let said = format!(" {} ", words.join(" "));
    let named = |list: &[String]| list.iter().any(|d| said.contains(&format!(" {d} ")) || said.contains(&format!(" {d}s ")));
    if named(&_OTHER_DRINKS) {
        return Some(_DRINKS_ANSWER.clone());
    }
    if named(&_WATER_ONLY) {
        return Some(_WATER_ANSWER.clone());
    }
    Some(_DRINKS_ANSWER.clone())
}

/// Visiting hours for a ward, or when to visit a baby the chat has just welcomed.
pub fn _the_visiting_hours(memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = sh::text_or_clean(understood);
    if !_A_HOSPITAL_OR_A_WARD.is_match(&text_.to_lowercase()) {
        let mut said: Vec<&str> = Vec::new();
        for turn in &memory.turns {
            said.push(&turn.user);
            said.push(&turn.bot);
        }
        if _BIRTH_IN_THE_CHAT.is_match(&said.join(" ")) {
            return Some(
                "Ask them when they are ready for visitors rather than turning up: the first \
fortnight is for feeding and sleep, and a short visit at a time they choose \
is the one they can say yes to. If the baby is still in hospital, ring the \
ward for its visiting hours."
                    .into(),
            );
        }
    }
    Some(
        "Ring the ward before you set off: visiting hours are set ward by ward and change, \
many open late morning or early afternoon until about 8pm. Ward rounds, meals and \
handovers are usually closed, and two at the bedside at a time is the common limit. \
Don't visit with a cough, cold or stomach bug, use the hand gel both ways, and leave \
flowers at home: most wards ban them."
            .into(),
    )
}

/// Whether the line wants the visit hosted rather than the sentence for refusing it.
pub fn _not_asking_for_the_words(understood: &Understood) -> bool {
    !_ASKING_FOR_THE_WORDS.is_match(&sh::clean_or_text(understood).to_lowercase())
}

/// The bottle answer, or the night-feed answer when the line asks about the night.
pub fn _the_bottle_answer(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = understood.words.join(" ").to_lowercase();
    if said.contains("night")
        && (said.contains("feed") || said.contains("feeds") || said.contains("feeding") || said.contains("bottle") || said.contains("milk"))
    {
        return Some(
            "One night feed at a time, a few nights apart, starting with the easiest. For \
a bottle, water it down over two or three nights, then offer water only in a \
cup; for the breast, feed a minute less each night and settle them another \
way - a hand on the back, the same words, no light. Last milk at bedtime. \
Over six months and gaining well, a baby can go all night without a feed; \
younger, or not gaining, ask the health visitor first."
                .into(),
        );
    }
    Some(
        "Move from bottles to a free-flow cup at about 12 months, and have them off bottles \
by two: a bottle held for hours, or taken to bed, keeps milk on the teeth and is the \
usual cause of decay in a child this age. Drop one bottle at a time over a fortnight, \
starting with the daytime ones, and keep milk to meals and water in between. Nothing \
but water in a bottle in bed."
            .into(),
    )
}

/// The line to hold, worded for the person doing the asking.
pub fn _the_money_asked_off_you(_memory: &Memory, reading: &Understood) -> Option<String> {
    if reading.words.iter().any(|w| _MONEY_ASKERS.contains(w)) {
        return Some(_FAMILY_ASKING_MONEY.clone());
    }
    Some(_FRIEND_ASKING_MONEY.clone())
}

/// True when the line asks how long a reading should be, not what to read.
pub fn _the_length_of_a_reading(understood: &Understood) -> bool {
    _THE_LENGTH_ASK.is_match(&clean_or_norm_or_text(understood))
}

/// True unless the line says the broken heart is somebody else's.
pub fn _the_heart_is_the_persons_own(reading: &Understood) -> bool {
    reading.state_owner.as_deref() != Some("other")
}

/// True unless the line is the reader's own appetite with nobody else named in it.
pub fn _not_my_own_appetite(reading: &Understood) -> bool {
    let text_ = sh::clean_or_text(reading);
    if !_OWN_APPETITE.is_match(text_) {
        return true;
    }
    _SOMEONE_NAMED.is_match(text_)
}

/// True unless the line asks who can help and names an older relative to look after.
pub fn _a_break_for_the_carer(understood: &Understood) -> bool {
    let said = text_or_words(understood).to_lowercase();
    if said.contains("single parent") || said.contains("single mum") || said.contains("single dad") {
        return true;
    }
    if !_A_WHO_CAN_HELP.is_match(&said) {
        return true;
    }
    !_AN_ELDER_TO_LOOK_AFTER.is_match(&said)
}

/// The changing sequence, or how often to change first for a line that asks how often.
pub fn _change_a_nappy_reply(_memory: &Memory, understood: &Understood) -> Option<String> {
    if dup::asks_how_often(understood) {
        return Some(NAPPY_HOW_OFTEN.clone());
    }
    Some(NAPPY_HOW_TO.clone())
}

// ---------------------------------------------------------------------------------------------
// The lambdas written inline in the entries.

/// The rainy-day plan, or the weekend one when the line names the weekend (line 4665).
pub fn lambda_4665(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if ["weekend", "saturday", "sunday"].iter().any(|w| clean.contains(w)) {
        return Some(
            "One outing and one thing at home each day is enough: a park, a swim or a bus \
ride in the morning, then a den, a bake or a film in the afternoon. Keep the \
outing small enough that nobody is overtired by teatime, and let them pick it, \
because choosing is half the entertainment."
                .into(),
        );
    }
    Some(
        "Build a den out of chairs and a sheet, then run three short challenges \
inside it: a ten-minute toy treasure hunt, paper aeroplanes off the stairs \
and a kitchen disco. Changing rooms every half hour keeps the 'I'm bored' \
away until teatime."
            .into(),
    )
}
/// `lambda u: _the_visit_card_fits(u) and _not_asking_for_the_words(u)` (line 4842).
pub fn lambda_4842(u: &Understood) -> bool {
    _the_visit_card_fits(u) && _not_asking_for_the_words(u)
}
/// `lambda reading: _i_am_not_one_of_the_two(reading) and not_a_fight_about_clothes(reading)`.
pub fn lambda_6214(reading: &Understood) -> bool {
    _i_am_not_one_of_the_two(reading) && dup::not_a_fight_about_clothes(reading)
}
/// `lambda u: bool(u.get("norm")) and not _a_friend_birthday_in_the_diary(u)` (line 7488).
pub fn lambda_7488(u: &Understood) -> bool {
    !u.norm.is_empty() && !_a_friend_birthday_in_the_diary(u)
}
/// `lambda reading: not owns_the_deed(reading) and _a_person_is_the_one_letting_you_down(...)`.
pub fn lambda_7817(reading: &Understood) -> bool {
    !dup::owns_the_deed(reading) && _a_person_is_the_one_letting_you_down(reading)
}
/// `lambda reading: not owns_the_deed(reading)` (lines 7973, 11168, 11619).
pub fn lambda_7973(reading: &Understood) -> bool {
    !dup::owns_the_deed(reading)
}
/// `lambda reading: not owns_the_deed(reading)` (line 11168; same body as 7973).
pub fn lambda_11168(reading: &Understood) -> bool {
    lambda_7973(reading)
}
/// `lambda reading: not owns_the_deed(reading)` (line 11619; same body as 7973).
pub fn lambda_11619(reading: &Understood) -> bool {
    lambda_7973(reading)
}
/// `lambda reading: not _a_grown_person_named(reading)` (line 9690).
pub fn lambda_9690(reading: &Understood) -> bool {
    !_a_grown_person_named(reading)
}
/// `lambda understood: not _IT_ITCHES.search(clean.lower())` (line 9766).
pub fn lambda_9766(understood: &Understood) -> bool {
    !_IT_ITCHES.is_match(&understood.clean.to_lowercase())
}
/// `lambda u: _not_asking_for_a_present_for_them(u) and _not_a_visit_i_am_making(u)`.
pub fn lambda_12267(u: &Understood) -> bool {
    _not_asking_for_a_present_for_them(u) && _not_a_visit_i_am_making(u)
}
/// `lambda u: _not_a_visit_i_am_making(u) and _not_a_meal_the_visit_is_for(u)` (line 12357).
pub fn lambda_12357(u: &Understood) -> bool {
    _not_a_visit_i_am_making(u) && _not_a_meal_the_visit_is_for(u)
}
/// `lambda reading: _a_childs_missing_speech_is_the_question(reading) and not owns_the_deed(...)`.
pub fn lambda_13078(reading: &Understood) -> bool {
    _a_childs_missing_speech_is_the_question(reading) && !dup::owns_the_deed(reading)
}

// ---------------------------------------------------------------------------------------------
// Registry tables.

macro_rules! guard_fn {
    ($wrapper:ident, $f:path) => {
        fn $wrapper(_f: &FnRef, u: &Understood) -> bool {
            $f(u)
        }
    };
}
macro_rules! reply_fn {
    ($wrapper:ident, $f:path) => {
        fn $wrapper(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
            $f(m, u)
        }
    };
}

guard_fn!(g_lambda_4842, lambda_4842);
guard_fn!(g_lambda_6214, lambda_6214);
guard_fn!(g_lambda_7488, lambda_7488);
guard_fn!(g_lambda_7817, lambda_7817);
guard_fn!(g_lambda_7973, lambda_7973);
guard_fn!(g_lambda_11168, lambda_11168);
guard_fn!(g_lambda_11619, lambda_11619);
guard_fn!(g_lambda_9690, lambda_9690);
guard_fn!(g_lambda_9766, lambda_9766);
guard_fn!(g_lambda_12267, lambda_12267);
guard_fn!(g_lambda_12357, lambda_12357);
guard_fn!(g_lambda_13078, lambda_13078);
guard_fn!(g_a_baby_in_the_water, _a_baby_in_the_water);
guard_fn!(g_a_break_for_the_carer, _a_break_for_the_carer);
guard_fn!(g_a_caffeine_drink_was_named, _a_caffeine_drink_was_named);
guard_fn!(g_a_child_ate_the_lot, _a_child_ate_the_lot);
guard_fn!(g_a_child_is_being_told, _a_child_is_being_told);
guard_fn!(g_a_child_old_enough_to_eat, _a_child_old_enough_to_eat);
guard_fn!(g_a_childs_morning, _a_childs_morning);
guard_fn!(g_a_giving_up_on_a_person, _a_giving_up_on_a_person);
guard_fn!(g_a_grunt_is_a_breath_sign, _a_grunt_is_a_breath_sign);
guard_fn!(g_a_leak_is_not_a_strip, _a_leak_is_not_a_strip);
guard_fn!(g_a_milk_pump_question, _a_milk_pump_question);
guard_fn!(g_a_person_is_what_they_get_over, _a_person_is_what_they_get_over);
guard_fn!(g_a_role_in_a_wedding, _a_role_in_a_wedding);
guard_fn!(g_a_route_ask_not_an_opener, _a_route_ask_not_an_opener);
guard_fn!(g_a_teenager_is_named, _a_teenager_is_named);
guard_fn!(g_a_waking_is_not_a_wake, _a_waking_is_not_a_wake);
guard_fn!(g_an_older_baby_is_named, _an_older_baby_is_named);
guard_fn!(g_names_someone_else, _names_someone_else);
guard_fn!(g_not_a_debt_or_a_trip, _not_a_debt_or_a_trip);
guard_fn!(g_not_a_hard_conversation, _not_a_hard_conversation);
guard_fn!(g_not_a_real_coffee, _not_a_real_coffee);
guard_fn!(g_not_a_stretch_or_a_job, _not_a_stretch_or_a_job);
guard_fn!(g_not_a_thing_made_of_glass, _not_a_thing_made_of_glass);
guard_fn!(g_not_a_toddler_in_the_morning, _not_a_toddler_in_the_morning);
guard_fn!(g_not_an_ask_for_the_words_to_say, _not_an_ask_for_the_words_to_say);
guard_fn!(g_not_just_the_age_of_a_relation, _not_just_the_age_of_a_relation);
guard_fn!(g_not_my_own_appetite, _not_my_own_appetite);
guard_fn!(g_not_some_time_ago, _not_some_time_ago);
guard_fn!(g_the_debt_is_money, _the_debt_is_money);
guard_fn!(g_the_heart_is_the_persons_own, _the_heart_is_the_persons_own);
guard_fn!(g_the_home_or_a_visit_is_named, _the_home_or_a_visit_is_named);
guard_fn!(g_the_length_of_a_reading, _the_length_of_a_reading);
guard_fn!(g_the_parent_is_not_only_tired, _the_parent_is_not_only_tired);
guard_fn!(g_the_person_forgets_it_themselves, _the_person_forgets_it_themselves);
guard_fn!(g_the_question_is_a_name, _the_question_is_a_name);
guard_fn!(g_the_trip_i_am_making, _the_trip_i_am_making);
guard_fn!(g_told_them_off, _told_them_off);

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.family:<lambda>:11168", g_lambda_11168),
    ("bot.knowledge.family:<lambda>:11619", g_lambda_11619),
    ("bot.knowledge.family:<lambda>:12267", g_lambda_12267),
    ("bot.knowledge.family:<lambda>:12357", g_lambda_12357),
    ("bot.knowledge.family:<lambda>:13078", g_lambda_13078),
    ("bot.knowledge.family:<lambda>:4842", g_lambda_4842),
    ("bot.knowledge.family:<lambda>:6214", g_lambda_6214),
    ("bot.knowledge.family:<lambda>:7488", g_lambda_7488),
    ("bot.knowledge.family:<lambda>:7817", g_lambda_7817),
    ("bot.knowledge.family:<lambda>:7973", g_lambda_7973),
    ("bot.knowledge.family:<lambda>:9690", g_lambda_9690),
    ("bot.knowledge.family:<lambda>:9766", g_lambda_9766),
    ("bot.knowledge.family:_a_baby_in_the_water:459", g_a_baby_in_the_water),
    ("bot.knowledge.family:_a_break_for_the_carer:3540", g_a_break_for_the_carer),
    ("bot.knowledge.family:_a_caffeine_drink_was_named:2337", g_a_caffeine_drink_was_named),
    ("bot.knowledge.family:_a_child_ate_the_lot:3306", g_a_child_ate_the_lot),
    ("bot.knowledge.family:_a_child_is_being_told:3207", g_a_child_is_being_told),
    ("bot.knowledge.family:_a_child_old_enough_to_eat:2276", g_a_child_old_enough_to_eat),
    ("bot.knowledge.family:_a_childs_morning:494", g_a_childs_morning),
    ("bot.knowledge.family:_a_giving_up_on_a_person:380", g_a_giving_up_on_a_person),
    ("bot.knowledge.family:_a_grunt_is_a_breath_sign:1363", g_a_grunt_is_a_breath_sign),
    ("bot.knowledge.family:_a_leak_is_not_a_strip:218", g_a_leak_is_not_a_strip),
    ("bot.knowledge.family:_a_milk_pump_question:2318", g_a_milk_pump_question),
    ("bot.knowledge.family:_a_person_is_what_they_get_over:2638", g_a_person_is_what_they_get_over),
    ("bot.knowledge.family:_a_role_in_a_wedding:343", g_a_role_in_a_wedding),
    ("bot.knowledge.family:_a_route_ask_not_an_opener:3267", g_a_route_ask_not_an_opener),
    ("bot.knowledge.family:_a_teenager_is_named:1046", g_a_teenager_is_named),
    ("bot.knowledge.family:_a_waking_is_not_a_wake:421", g_a_waking_is_not_a_wake),
    ("bot.knowledge.family:_an_older_baby_is_named:1392", g_an_older_baby_is_named),
    ("bot.knowledge.family:_names_someone_else:3248", g_names_someone_else),
    ("bot.knowledge.family:_not_a_debt_or_a_trip:315", g_not_a_debt_or_a_trip),
    ("bot.knowledge.family:_not_a_hard_conversation:2979", g_not_a_hard_conversation),
    ("bot.knowledge.family:_not_a_real_coffee:257", g_not_a_real_coffee),
    ("bot.knowledge.family:_not_a_stretch_or_a_job:1204", g_not_a_stretch_or_a_job),
    ("bot.knowledge.family:_not_a_thing_made_of_glass:43", g_not_a_thing_made_of_glass),
    ("bot.knowledge.family:_not_a_toddler_in_the_morning:1111", g_not_a_toddler_in_the_morning),
    ("bot.knowledge.family:_not_an_ask_for_the_words_to_say:3315", g_not_an_ask_for_the_words_to_say),
    ("bot.knowledge.family:_not_just_the_age_of_a_relation:1145", g_not_just_the_age_of_a_relation),
    ("bot.knowledge.family:_not_my_own_appetite:3517", g_not_my_own_appetite),
    ("bot.knowledge.family:_not_some_time_ago:2629", g_not_some_time_ago),
    ("bot.knowledge.family:_the_debt_is_money:432", g_the_debt_is_money),
    ("bot.knowledge.family:_the_heart_is_the_persons_own:3506", g_the_heart_is_the_persons_own),
    ("bot.knowledge.family:_the_home_or_a_visit_is_named:3144", g_the_home_or_a_visit_is_named),
    ("bot.knowledge.family:_the_length_of_a_reading:3478", g_the_length_of_a_reading),
    ("bot.knowledge.family:_the_parent_is_not_only_tired:2962", g_the_parent_is_not_only_tired),
    ("bot.knowledge.family:_the_person_forgets_it_themselves:3175", g_the_person_forgets_it_themselves),
    ("bot.knowledge.family:_the_question_is_a_name:361", g_the_question_is_a_name),
    ("bot.knowledge.family:_the_trip_i_am_making:3048", g_the_trip_i_am_making),
    ("bot.knowledge.family:_told_them_off:2923", g_told_them_off),
];

reply_fn!(r_lambda_4665, lambda_4665);
reply_fn!(r_a_drink_while_feeding, _a_drink_while_feeding);
reply_fn!(r_a_phone_rule_and_an_age, _a_phone_rule_and_an_age);
reply_fn!(r_asking_out, _asking_out);
reply_fn!(r_change_a_nappy_reply, _change_a_nappy_reply);
reply_fn!(r_child_bedtime, _child_bedtime);
reply_fn!(r_child_weight, _child_weight);
reply_fn!(r_date_idea, _date_idea);
reply_fn!(r_elderly_appetite, _elderly_appetite);
reply_fn!(r_hitting_or_biting, _hitting_or_biting);
reply_fn!(r_hurt_feelings, _hurt_feelings);
reply_fn!(r_late_talker_reply, _late_talker_reply);
reply_fn!(r_making_friends, _making_friends);
reply_fn!(r_medicine_for_a_young_baby, _medicine_for_a_young_baby);
reply_fn!(r_milestones_reply, _milestones_reply);
reply_fn!(r_milk_tooth, _milk_tooth);
reply_fn!(r_off_school, _off_school);
reply_fn!(r_picky_eater, _picky_eater);
reply_fn!(r_stroke_reply, _stroke_reply);
reply_fn!(r_the_bottle_answer, _the_bottle_answer);
reply_fn!(r_the_dummy_answer, _the_dummy_answer);
reply_fn!(r_the_feeds_for_the_age, _the_feeds_for_the_age);
reply_fn!(r_the_money_asked_off_you, _the_money_asked_off_you);
reply_fn!(r_the_repair_after_it, _the_repair_after_it);
reply_fn!(r_the_visiting_hours, _the_visiting_hours);
reply_fn!(r_the_year_they_move_up_to, _the_year_they_move_up_to);
reply_fn!(r_wont_talk_reply, _wont_talk_reply);

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.family:<lambda>:4665", r_lambda_4665),
    ("bot.knowledge.family:_a_drink_while_feeding:3350", r_a_drink_while_feeding),
    ("bot.knowledge.family:_a_phone_rule_and_an_age:140", r_a_phone_rule_and_an_age),
    ("bot.knowledge.family:_asking_out:1543", r_asking_out),
    ("bot.knowledge.family:_change_a_nappy_reply:13508", r_change_a_nappy_reply),
    ("bot.knowledge.family:_child_bedtime:881", r_child_bedtime),
    ("bot.knowledge.family:_child_weight:1570", r_child_weight),
    ("bot.knowledge.family:_date_idea:1533", r_date_idea),
    ("bot.knowledge.family:_elderly_appetite:2530", r_elderly_appetite),
    ("bot.knowledge.family:_hitting_or_biting:1963", r_hitting_or_biting),
    ("bot.knowledge.family:_hurt_feelings:1557", r_hurt_feelings),
    ("bot.knowledge.family:_late_talker_reply:2144", r_late_talker_reply),
    ("bot.knowledge.family:_making_friends:1412", r_making_friends),
    ("bot.knowledge.family:_medicine_for_a_young_baby:540", r_medicine_for_a_young_baby),
    ("bot.knowledge.family:_milestones_reply:1896", r_milestones_reply),
    ("bot.knowledge.family:_milk_tooth:640", r_milk_tooth),
    ("bot.knowledge.family:_off_school:658", r_off_school),
    ("bot.knowledge.family:_picky_eater:2374", r_picky_eater),
    ("bot.knowledge.family:_stroke_reply:2510", r_stroke_reply),
    ("bot.knowledge.family:_the_bottle_answer:3420", r_the_bottle_answer),
    ("bot.knowledge.family:_the_dummy_answer:125", r_the_dummy_answer),
    ("bot.knowledge.family:_the_feeds_for_the_age:1403", r_the_feeds_for_the_age),
    ("bot.knowledge.family:_the_money_asked_off_you:3462", r_the_money_asked_off_you),
    ("bot.knowledge.family:_the_repair_after_it:2905", r_the_repair_after_it),
    ("bot.knowledge.family:_the_visiting_hours:3370", r_the_visiting_hours),
    ("bot.knowledge.family:_the_year_they_move_up_to:1857", r_the_year_they_move_up_to),
    ("bot.knowledge.family:_wont_talk_reply:2189", r_wont_talk_reply),
];

fn s_a_small_childs_age(_f: &FnRef, words: &str) -> bool {
    a_small_childs_age(words)
}

pub const SHAPES: &[(&str, ShapeFn)] = &[("bot.knowledge.family:a_small_childs_age:1093", s_a_small_childs_age)];
