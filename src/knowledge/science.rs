//! Port of PyBot's `bot/knowledge/science.py`: everyday science and the body's everyday
//! questions (memory and forgetting, sleep, falls, medicines, caffeine, headaches, steps,
//! heartburn, baby sleep, the body's records, planets, tall buildings and bridges, physics
//! pairs), with the guards that keep somebody else's memory, a thing that falls or a sum's
//! "factor" off the rows.

use crate::data::{self, FnRef};
use crate::knowledge::body;
use crate::knowledge::d1_dup_g4 as dup;
use crate::knowledge::d1_shared::{self as sh, LeadFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as re, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Module globals from the bundle, by their Python name.
macro_rules! global {
    ($name:ident: re) => {
        static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.knowledge.science.", stringify!($name))));
    };
    ($name:ident: set) => {
        static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.knowledge.science.", stringify!($name))));
    };
    ($name:ident: strs) => {
        static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.knowledge.science.", stringify!($name))));
    };
    ($name:ident: string) => {
        static $name: LazyLock<String> =
            LazyLock::new(|| data::string(concat!("bot.knowledge.science.", stringify!($name))));
    };
    ($name:ident: pairs) => {
        static $name: LazyLock<Vec<(Vec<String>, String)>> =
            LazyLock::new(|| names_table(concat!("bot.knowledge.science.", stringify!($name))));
    };
    ($name:ident: str_int) => {
        static $name: LazyLock<HashMap<String, i64>> =
            LazyLock::new(|| str_int_map(concat!("bot.knowledge.science.", stringify!($name))));
    };
}

/// A table of (names, answer) rows, in order.
fn names_table(name: &str) -> Vec<(Vec<String>, String)> {
    data::get(name).items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str().to_string())).collect()
}

/// A `{str: int}` dict.
fn str_int_map(name: &str) -> HashMap<String, i64> {
    data::get(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
}

/// A `{str: str}` dict, in insertion order.
fn str_pairs(name: &str) -> Vec<(String, String)> {
    data::get(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}

global!(_ANOTHER_ADULTS_MEMORY: re);
global!(_SOMEBODY_ELSE_IS_THE_SUBJECT: re);
global!(_NAMES_THE_ILLNESS: re);
global!(_MY_OWN_ERRAND: re);
global!(_MONEY_OWED: re);
global!(_A_HABIT_THING: re);
global!(_TIRED: re);
global!(FELL_OUT: re);
global!(STANDS_UP_THINGS: re);
global!(PEOPLE_WHO_FALL: re);
global!(NOT_A_FALL_PHRASE: re);
global!(BEFORE_THE_FALL: re);
global!(PEOPLE_WHO_ARE_THE_SUBJECT: string);
global!(NOT_THE_SUBJECT: set);
global!(GRINDING_BABY: strs);
global!(GRINDING_ADULT: string);
global!(_SAME_OR_TWICE: re);
global!(_TWICE: re);
global!(BABY_GRINDING: string);
global!(_HANGOVER_LENGTH: string);
global!(_HANGOVER_CURE: string);
global!(_HANGOVER_LENGTH_ASK: re);
global!(GRINDING_WHAT: string);
global!(_WHAT_IS_IT: strs);
global!(NIGHT_CAUSE_TOPICS: strs);
global!(NIGHT_SYMPTOM: strs);
global!(EARLY_HOUR: strs);
global!(STILL_ASLEEP: strs);
global!(LOW_MORNING: strs);
global!(WAKE_AT_NIGHT: string);
global!(SLEEP_SAID: re);
global!(SLEEP_NUMBER: str_int);
global!(SLEEP_TALK: re);
global!(EYES_COST: re);
global!(_MED_MISSED: re);
global!(_MED_DOUBLE: re);
global!(_MED_REPEAT: re);
global!(_MED_SUGAR: re);
global!(_MED_CONFUSED: re);
global!(_MED_TABLET_MUDDLE: re);
global!(_MED_CONFUSED_NEW: re);
global!(_MED_TOO_MUCH: re);
global!(_MED_DOUBLE_ANSWER: string);
global!(_MED_BOX: string);
global!(_MED_ROUTINE: string);
global!(_MED_CONFUSED_ANSWER: string);
global!(_MED_TOOK_TWO: string);
global!(_PILL_MINI: re);
global!(_MED_RAN_OUT: re);
global!(_MED_CRITICAL: re);
global!(_MED_RAN_OUT_ANSWER: string);
global!(_MED_CRITICAL_ANSWER: string);
global!(_MED_THIRD: re);
global!(_MED_THIRD_PERSON: re);
global!(_MED_PATTERN: string);
global!(_MED_ANTIBIOTIC: re);
global!(_MED_ANTIBIOTIC_ANSWER: string);
global!(_MED_INSULIN: string);
global!(_MED_SUGAR_LINE: string);
global!(_KEEP_ANY: re);
global!(_KEEP_LONG: re);
global!(_CAFFEINE_SYMPTOM: strs);
global!(_CAFFEINE_COUNT: re);
global!(_KIDS_LINE: string);
global!(_KID_WORDS: strs);
global!(_PRONOUN_SUBJECT: re);
global!(_CAFFEINE_TAIL: string);
global!(_LIMIT_ASKS: set);
global!(_TEEN_AGE_PHRASES: strs);
global!(_COLD_PREVENT: strs);
global!(_COLD_HELP: string);
global!(_COLD_STOP: string);
global!(_COLD_NAMED: strs);
global!(_NUMBER_WORDS: str_int);
global!(_UNIT_DAYS: str_int);
global!(_BARE_LENGTHS: strs);
global!(_EVERY_DAY: strs);
global!(HEADACHE_DAYS: string);
global!(_HEADACHE_AFTERNOON: string);
global!(_HEAVY_PERIOD_WORDS: strs);
global!(_PERIOD_CHANGE_WORDS: strs);
global!(PERIOD_CHANGE_REPLY: string);
global!(NOT_THIS_PERSONS_PERIOD: strs);
global!(HICCUPS_BABY: string);
global!(HICCUPS_BABY_AGAIN: string);
global!(_BABY_WORDS: strs);
global!(STEPS_PISA: string);
global!(STEPS_EMPIRE: string);
global!(STEPS_PAULS: string);
global!(STEPS_FLIGHT: string);
global!(STEPS_TAIL: string);
global!(STEPS_A_MILE: string);
global!(STEPS_REPORT_WORDS: strs);
global!(STEPS_COUNT: re);
global!(_HEARTBURN_REPORT_BEFORE: re);
global!(_HEARTBURN_REPORT_AFTER: re);
global!(_HEARTBURN_NOT_A_FOOD: set);
global!(_HEARTBURN_ANSWER: string);
global!(_HEARTBURN_BLAMED: string);
global!(_HEARTBURN_PREGNANT: string);
global!(_HEARTBURN_PILLS: string);
global!(_HEARTBURN_EAT: string);
global!(_HEARTBURN_EAT_WORDS: strs);
global!(_HEARTBURN_PREGNANT_WORDS: strs);
global!(_HEARTBURN_PILL_WORDS: strs);
global!(_HEARTBURN_TABLETS_NOT_WORKING: string);
global!(_HEARTBURN_TREATMENT_WORDS: strs);
global!(_HEARTBURN_TABLETS_WORDS: strs);
global!(_HEARTBURN_NOT_WORKING_WORDS: strs);
global!(_HEARTBURN_TEA: string);
global!(_HEARTBURN_TRIGGERS: pairs);
global!(_HEARTBURN_WHY: string);
global!(_HEARTBURN_WHICH: string);
global!(WATER_FREEZES: string);
global!(_SUN_FACTORS: strs);
global!(DIZZY_SPELL: string);
global!(POUNDS_THAT_ARE_MONEY: re);
global!(AN_AMOUNT_OF_IT: re);
global!(WHAT_IS_REMEMBERED: re);
global!(LOSING_SOMETHING_ELSE: re);
global!(WATER_IS_THE_SUBJECT: set);
global!(_COLD_FOOD: re);
global!(_A_AGE: re);
global!(_A_NUMBER_WORDS: str_int);
global!(_A_WORRY: re);
global!(_BABY_SLEEP_TAIL: string);
global!(_BABY_SLEEP_PLAIN: string);
global!(_GENERIC_PARTS: strs);
global!(_FINGER_WORDS: strs);
global!(_SAYS_A_BODY: strs);
global!(_AN_ANIMAL: strs);
global!(_A_A_COLD: re);
global!(HAYFEVER_ADVICE: string);
global!(_VOLCANO_CAUSE: re);
global!(_VOLCANO_BUILD: re);
global!(_SITTING_KIT: set);
global!(_A_PLACE_SHAPED: strs);
global!(_ICE_WHEN: re);
global!(_BRUISE_PLAIN: string);
global!(_BRUISE_ASKS: pairs);
global!(_CLOCK_AT: re);
global!(_BED_AT: re);
global!(PLANET_SIZES: pairs);
global!(BUILDING_STANDS: string);
global!(CRANE_STANDS: string);
global!(BRIDGE_STANDS: string);
global!(CHIMNEY_STANDS: string);
global!(TREE_STANDS: string);
global!(PLANT_STANDS: string);
global!(GARDEN_PLANT_STANDS: string);
global!(GARDEN_PLANTS: re);
global!(STRUCTURE_ANSWERS: pairs);
global!(PLACES_WITH_A_TALLEST: strs);
global!(PLACES_WITH_A_LONGEST: strs);
global!(TALL_BUILDINGS: pairs);
global!(_DEFINE: strs);
global!(MASS_OR_WEIGHT_WORDS: strs);
global!(MASS_OR_WEIGHT_UNITS: strs);
global!(_NEWTON_SECOND: string);
global!(_NEWTON_FIRST: string);
global!(_NEWTON_THIRD: string);
global!(_NEWTON_EACH: re);
global!(_NEWTON_WORD: str_int);
global!(THE_ROCK: set);
global!(THE_SKY: set);
global!(_SLEEP_MAGNESIUM: string);
global!(_SLEEP_VALERIAN: string);
global!(_SLEEP_GUMMIES: string);
global!(_SLEEP_HABIT: string);
global!(_SLEEP_CHILD: string);
global!(_SLEEP_CHILD_WORDS: strs);

/// `HEADACHE_NO_DRUGS` and `NO_DRUGS`, imported from health.
static HEADACHE_NO_DRUGS: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.health.HEADACHE_NO_DRUGS"));
static NO_DRUGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.health.NO_DRUGS"));

/// `NAMED_MOONS`: the moons worth naming, by planet.
static NAMED_MOONS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.knowledge.science.NAMED_MOONS"));
/// `MOONS`: the confirmed count, by planet.
static MOONS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| str_int_map("bot.knowledge.science.MOONS"));
/// `DISCOVERERS`: who found what, in table order.
static DISCOVERERS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("bot.knowledge.science.DISCOVERERS"));
/// `SLEEP_WORD`: the hours as a capitalised word, by number.
static SLEEP_WORD: LazyLock<HashMap<i64, String>> = LazyLock::new(|| {
    data::get("bot.knowledge.science.SLEEP_WORD")
        .pairs()
        .iter()
        .map(|(k, v)| (k.as_int().unwrap_or(0), v.str().to_string()))
        .collect()
});
/// `_CAFFEINE_DRINKS`: (drink, the phrases that name it), in order.
static _CAFFEINE_DRINKS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.science._CAFFEINE_DRINKS")
        .items()
        .iter()
        .map(|row| (row.items()[0].str().to_string(), row.items()[1].strs()))
        .collect()
});
/// `_CAFFEINE_LEAD`: the limit's opening, by drink.
static _CAFFEINE_LEAD: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.knowledge.science._CAFFEINE_LEAD"));
/// `_BABY_SLEEP_BANDS`: (from months, to months, the line), in order.
static _BABY_SLEEP_BANDS: LazyLock<Vec<(f64, f64, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.science._BABY_SLEEP_BANDS")
        .items()
        .iter()
        .map(|row| {
            let r = row.items();
            (r[0].as_f64().unwrap_or(0.0), r[1].as_f64().unwrap_or(0.0), r[2].str().to_string())
        })
        .collect()
});
static _BABY_SLEEP_TAIL_UNDER_MONTHS: LazyLock<f64> =
    LazyLock::new(|| data::get("bot.knowledge.science._BABY_SLEEP_TAIL_UNDER_MONTHS").as_f64().unwrap_or(12.0));
/// `BODY_RECORDS`: (the parts, the record words asked with, the reply), in order.
static BODY_RECORDS: LazyLock<Vec<(Vec<String>, Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.science.BODY_RECORDS")
        .items()
        .iter()
        .map(|row| {
            let r = row.items();
            (r[0].strs(), r[1].strs(), r[2].str().to_string())
        })
        .collect()
});
static STEPS_PER_MILE: LazyLock<f64> =
    LazyLock::new(|| data::get("bot.knowledge.science.STEPS_PER_MILE").as_f64().unwrap_or(2000.0));
static STEPS_PER_KM: LazyLock<f64> =
    LazyLock::new(|| data::get("bot.knowledge.science.STEPS_PER_KM").as_f64().unwrap_or(1250.0));

/// `re.search(r"\b" + re.escape(word) + r"\b", text)`.
fn says_word(text: &str, word: &str) -> bool {
    sh::re_search(&format!(r"\b{}\b", sh::escape(word)), text, 0)
}

/// `re.sub(r"[^a-z0-9' ]+", " ", text)`.
fn plain_chars(text: &str) -> String {
    static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
    NOT_PLAIN.sub(" ", text)
}

// ---------------------------------------------------------------------------------------------
// Memory and forgetting.

/// True when the line is not about another adult (or a child) forgetting.
pub fn _not_somebody_elses_memory(understood: &Understood) -> bool {
    let text = dup::habits_words_of(understood);
    if text.is_empty() {
        return true;
    }
    !_SOMEBODY_ELSE_IS_THE_SUBJECT.is_match(&text)
}

/// True when the line is the speaker forgetting to do a thing (their own errand).
pub fn _the_speaker_forgot_to_do_it(understood: &Understood) -> bool {
    let text = dup::habits_words_of(understood);
    !text.is_empty() && _MY_OWN_ERRAND.is_match(&text)
}

/// True when the chat is about a debt, not a memory.
pub fn _its_money_that_has_not_come_back(understood: &Understood) -> bool {
    let text = dup::habits_words_of(understood);
    !text.is_empty() && _MONEY_OWED.is_match(&text)
}

/// True when the line is the speaker's own habit that slipped (keys, names, tablets) and does
/// not name the illness.
pub fn _a_habit_without_the_illness(understood: &Understood) -> bool {
    let text = dup::habits_words_of(understood);
    if text.is_empty() {
        return false;
    }
    if _ANOTHER_ADULTS_MEMORY.is_match(&text) {
        return false;
    }
    _A_HABIT_THING.is_match(&text) && !_NAMES_THE_ILLNESS.is_match(&text)
}

/// What killed the dinosaurs, or when they lived.
pub fn _dinosaurs(_memory: &Memory, understood: &Understood) -> String {
    static WHEN: LazyLock<Re> = LazyLock::new(|| {
        Re::new(
            r"when|how long|how old|hoe long|what era|which era|what period|which period|million years",
            0,
        )
    });
    let clean = sh::or(&understood.clean, &understood.plain).to_lowercase();
    if WHEN.is_match(&clean) {
        return "The dinosaurs were around for about 165 million years, from roughly 230 million \
                years ago to 66 million years ago, in the Triassic, Jurassic and Cretaceous \
                periods. The first ones were small and two-legged, the biggest - the long-necked \
                sauropods and then T. rex - came late, and by the end they were on every \
                continent including Antarctica."
            .into();
    }
    "An asteroid about 10 km wide hit what is now Mexico 66 million years ago. The dust \
     it threw up blocked the sun for years, plants and the animals that ate them \
     collapsed, and the dinosaurs that could not fly died out; birds, which are their \
     descendants, survived."
        .into()
}

/// True when the line, the clean line or the words name being tired.
pub fn _names_the_feeling(understood: &Understood) -> bool {
    let words = understood.words.join(" ");
    [understood.text.as_str(), understood.clean.as_str(), words.as_str()]
        .iter()
        .filter(|p| !p.is_empty())
        .any(|p| _TIRED.is_match(&p.to_lowercase()))
}

/// Whether any of the patterns (regex source, no flags) is found in the text.
pub fn _any_hit<S: AsRef<str>>(patterns: &[S], text: &str) -> bool {
    patterns.iter().any(|p| sh::re_search(p.as_ref(), text, 0))
}

// ---------------------------------------------------------------------------------------------
// Falls.

/// False when the "fall" is a falling out (a row), not a fall.
pub fn _a_fall_and_not_a_row(understood: &Understood) -> bool {
    let words = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    // `raw` is not a key of the reading: it reads as "".
    let text = [understood.clean.as_str(), understood.text.as_str(), "", words.as_str()].join(" ").to_lowercase();
    !FELL_OUT.is_match(&plain_chars(&text))
}

/// True when a person who falls is named, or nothing that stands up (a building, a crane) is.
pub fn _a_person_who_falls(understood: &Understood) -> bool {
    let said = plain_chars(&format!("{} {}", understood.clean, understood.text).to_lowercase());
    if PEOPLE_WHO_FALL.is_match(&said) {
        return true;
    }
    !STANDS_UP_THINGS.is_match(&said)
}

/// False when what keeps falling is a thing (a picture, the shelf), not a person.
pub fn _a_thing_falls_not_a_person(understood: &Understood) -> bool {
    // `raw` is not a key of the reading: it reads as "".
    let said = plain_chars(&format!("{} {} ", understood.clean, understood.text).to_lowercase());
    if NOT_A_FALL_PHRASE.is_match(&said) {
        return false;
    }
    for subject in BEFORE_THE_FALL.findall_str(&said) {
        if subject == "i" || subject == "we" || subject == "you" {
            continue;
        }
        if sh::re_search(&PEOPLE_WHO_ARE_THE_SUBJECT, &subject, 0) {
            continue;
        }
        if NOT_THE_SUBJECT.contains(&subject) {
            continue;
        }
        return false;
    }
    true
}

/// A fall a person had: not a row, a person (not a building), and not a thing falling.
pub fn _a_fall_a_person_had(understood: &Understood) -> bool {
    _a_fall_and_not_a_row(understood) && _a_person_who_falls(understood) && _a_thing_falls_not_a_person(understood)
}

/// The cold-or-flu row's lead: "No" for the same thing / the difference, "Yes" for twice,
/// else none.
pub fn _cold_or_flu_verdict(understood: &Understood) -> String {
    let text = understood.text.to_lowercase();
    if _SAME_OR_TWICE.is_match(&text) {
        return "No".into();
    }
    if _TWICE.is_match(&text) {
        return "Yes".into();
    }
    String::new()
}

/// The hangover card, with how long one lasts when that is asked.
pub fn _hangover(_memory: &Memory, understood: &Understood) -> String {
    let said = format!("{} {}", understood.clean, understood.words.join(" "));
    if _HANGOVER_LENGTH_ASK.is_match(&said) {
        return format!("{}{}", *_HANGOVER_LENGTH, *_HANGOVER_CURE);
    }
    _HANGOVER_CURE.clone()
}

/// Teeth grinding: a child's, what bruxism is, or the adult's night guard.
pub fn _grinding(_memory: &Memory, understood: &Understood) -> String {
    let clean = plain_chars(&sh::padded(&sh::clean_or_text(understood).to_lowercase()));
    if sh::any_word_in(&clean, &GRINDING_BABY) {
        return BABY_GRINDING.clone();
    }
    if _asks_what_it_is(&clean) {
        return GRINDING_WHAT.clone();
    }
    GRINDING_ADULT.clone()
}

/// Whether the line opens by asking what something is.
pub fn _asks_what_it_is(clean: &str) -> bool {
    let said = text::strip(clean);
    _WHAT_IS_IT.iter().any(|o| said.starts_with(o.as_str()))
}

/// Waking in the night; None when the waking has a cause to name (a symptom, the health or
/// feelings topic), an early hour, or a low morning earlier in the chat.
pub fn _night_waking(memory: &Memory, understood: &Understood) -> Option<String> {
    // `str(understood.get("norm") or "")`: the list's repr when it has words.
    let norm = if understood.norm.is_empty() { String::new() } else { dup::py_repr_str_list(&understood.norm) };
    let said_so_far = [understood.clean.as_str(), norm.as_str(), understood.plain.as_str()].join(" ");
    if understood.topics.iter().any(|t| NIGHT_CAUSE_TOPICS.contains(t)) {
        return None;
    }
    if _any_hit(&NIGHT_SYMPTOM, &said_so_far) {
        return None;
    }
    if _any_hit(&EARLY_HOUR, &said_so_far) && !_any_hit(&STILL_ASLEEP, &said_so_far) {
        return None;
    }
    let said = dup::thread_user_lines(memory).iter().map(|t| t.to_lowercase()).collect::<Vec<_>>().join(" ");
    if sh::any_in(&said, &LOW_MORNING) {
        return None;
    }
    Some(WAKE_AT_NIGHT.clone())
}

/// How many moons a planet has, which have names, or which has the most.
pub fn _moons(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.as_str();
    if ["name", "called", "which moon", "which moons", "list"].iter().any(|w| clean.contains(w)) {
        if clean.contains("earth") {
            return "Just the Moon, the fifth largest in the solar system and the only other world \
                    people have stood on."
                .into();
        }
        for planet in ["jupiter", "saturn", "mars", "uranus", "neptune"] {
            if clean.contains(planet) {
                return format!("{}.", NAMED_MOONS[planet]);
            }
        }
        return "Earth's is the Moon, Mars has Phobos and Deimos, and the four big Jovian moons \
                are Io, Europa, Ganymede and Callisto. Most of the hundreds the counts refer to \
                are unnamed rocks a few kilometres across."
            .into();
    }
    if clean.contains("most") && !["earth", "mars", "jupiter"].iter().any(|n| clean.contains(n)) {
        return "Saturn has the most confirmed moons, more than 140, ahead of Jupiter at about 95.".into();
    }
    for planet in ["jupiter", "saturn", "uranus", "neptune", "mars", "earth", "venus", "mercury"] {
        if clean.contains(planet) {
            let count = MOONS[planet];
            let name = text::capitalize(planet);
            if count == 0 {
                return format!("{name} has no moons.");
            }
            if count == 1 {
                return format!("{name} has one moon.");
            }
            return format!("{name} has {count} confirmed moons.");
        }
    }
    "Mercury and Venus have none, Earth has one and Mars has two; the gas giants have \
     dozens, with Saturn topping the list at more than 140."
        .into()
}

/// Who discovered the thing the line names (the longest name found wins); None for none.
pub fn who_discovered(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = sh::clean_or_text(understood).to_lowercase().replace('\'', "");
    let mut best: Option<&(String, String)> = None;
    for row in DISCOVERERS.iter() {
        if says_word(&clean, &row.0) && best.is_none_or(|b| text::len(&row.0) > text::len(&b.0)) {
            best = Some(row);
        }
    }
    best.map(|(_, who)| who.clone())
}

// ---------------------------------------------------------------------------------------------
// Sleep.

/// The hours of sleep the line says it gets (1 to 14), if any.
pub fn _sleep_said(clean: &str) -> Option<i64> {
    let m = SLEEP_SAID.search(clean)?;
    if !SLEEP_TALK.is_match(clean) {
        return None;
    }
    let word = m.str(1).to_lowercase();
    let hours = match SLEEP_NUMBER.get(&word).copied().filter(|n| *n != 0) {
        Some(n) => n,
        // `int(word)`: the capture is one or two digits
        None => dup::py_int(&word)?,
    };
    if (1..=14).contains(&hours) { Some(hours) } else { None }
}

/// `SLEEP_WORD.get(said, said)` as Python's "%s" prints it.
fn sleep_word(said: i64) -> String {
    SLEEP_WORD.get(&said).cloned().unwrap_or_else(|| said.to_string())
}

/// How much sleep: teenagers, the over-65s, or the hours the line says against seven to nine.
pub fn _sleep_hours(_memory: &Memory, understood: &Understood) -> String {
    static ENOUGH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(enough|ok|okay|fine|normal)\b", 0));
    let clean = sh::clean_or_text(understood).to_lowercase();
    let said = _sleep_said(&clean);
    if ["teenager", "teenagers", "teen", "teens", "teenage", "15", "16", "17", "sixteen", "fifteen", "seventeen"]
        .iter()
        .any(|w| clean.contains(w))
    {
        return "Teenagers need eight to ten hours a night, and their body clock runs late, so an \
                11pm bedtime is not late for them. The fix that helps most is a light morning \
                and the same wake-up time at weekends."
            .into();
    }
    if ["over 65", "over-65", "older", "elderly", "pensioner", "70"].iter().any(|w| clean.contains(w)) {
        return "Seven to eight hours is usual after 65, and the sleep gets lighter and more \
                broken, which is ordinary. Twenty-minute naps are fine; a long afternoon sleep \
                is what makes the night worse."
            .into();
    }
    if let Some(said) = said {
        if said < 7 {
            return format!(
                "{} hours is on the short side: most adults need seven to nine, and the shortfall \
                 shows as yawning, coffee and losing the thread by mid-afternoon. Half an hour \
                 earlier to bed, and the same wake-up time every day, is the fix - a long weekend \
                 lie-in is not.",
                sleep_word(said)
            );
        }
        if (7..=9).contains(&said) {
            let yes = if ENOUGH.is_match(&clean) { "Yes - " } else { "" };
            let word = if yes.is_empty() { sleep_word(said) } else { sleep_word(said).to_lowercase() };
            return format!(
                "{yes}{word} hours is inside the seven to nine most adults need, so there is nothing \
                 to fix if your days are fine. If you are yawning by mid-afternoon anyway, half an \
                 hour earlier to bed does more than a weekend lie-in."
            );
        }
        if said > 9 {
            return format!(
                "{} hours is more than an adult needs, and the extra is not doing anything for \
                 you. Long nights after a short week are fine; sleeping that long every night and \
                 still waking unrefreshed is worth a word with the GP.",
                sleep_word(said)
            );
        }
    }
    "Most adults do best on seven to nine hours, but the number is a guide and the test \
     that matters is how you are by mid-afternoon: yawning, reaching for coffee and \
     losing the thread on six hours means you are short, whatever the chart says. The fix \
     is usually a 30-minute earlier bedtime rather than a lie-in at the weekend, and \
     keeping the wake-up time the same even after a bad night. Some people genuinely run \
     fine on six and it runs in families; if your days are fine, the number is not a \
     problem to solve."
        .into()
}

/// An eye test: what it costs, how often, or yes, get one.
pub fn _eyes_test(_memory: &Memory, understood: &Understood) -> String {
    static ASKS_WHEN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:how often|how frequently|when|how soon)\b", re::I));
    static SHOULD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:should|shall|ought)\b", re::I));
    let clean = sh::clean_or_text(understood);
    let money = "free if you are under 16, over 60, or on some benefits or a low income - otherwise \
                 about £25 to £45";
    let when = "every two years suits most adults, and every year from 70 or if you have diabetes, \
                glaucoma in the family or a big change in your vision";
    if EYES_COST.is_match(clean) {
        return format!("In the UK a sight test is {money}. It checks eye health as well as glasses, and {when}.");
    }
    let asks_when = ASKS_WHEN.is_match(clean);
    let says_yes = SHOULD.is_match(clean) && !asks_when;
    format!("{}{when}. It is {money}.", if says_yes { "Yes - a sight test " } else { "A sight test " })
}

// ---------------------------------------------------------------------------------------------
// Medicines.

/// A missed pill (the mini pill's window when it is named) and emergency contraception.
pub fn _missed_pill(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    let mut first = "Take the missed pill as soon as you remember, then carry on at your usual time, and \
                     use condoms for the next 7 days.";
    if _PILL_MINI.is_match(&clean) {
        first = "Take the missed mini pill as soon as you remember - the window is 12 hours (3 \
                 hours for the older progestogen-only pills), and a later one counts as missed - \
                 and use condoms for the next 7 days.";
    }
    format!(
        "{first} If you had sex in those days, get emergency contraception today: a pharmacy \
         gives it free without an appointment and it is licensed up to 5 days, but it works \
         best the sooner it is taken."
    )
}

/// Whether to keep taking a medicine: finish antibiotics, keep the long-haul tablets, or ask
/// for a review before stopping.
pub fn _keep_taking(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    if _MED_ANTIBIOTIC.is_match(&clean) || clean.contains("finish the course") || clean.contains("finish them") {
        return "Finish the course even if you feel better - a half-finished course is how \
                infections come back and how resistance starts - and take the missed dose when \
                you remember unless the next one is nearly due. If the side effects are why you \
                want to stop, ring the pharmacist: there is usually another antibiotic that suits \
                better, and they can swap it today."
            .into();
    }
    if _KEEP_LONG.is_match(&clean) || _KEEP_ANY.is_match(&clean) {
        return "Keep taking it: tablets for blood pressure, cholesterol, the thyroid and the heart \
                are for the long haul, and they work precisely because you do not feel them - so \
                stopping when you feel well is when the risk comes back. If the dose or the side \
                effects are a problem, ask for a review rather than stopping, and never stop one \
                of these suddenly on your own. Ring the pharmacist or the surgery if you have \
                already stopped."
            .into();
    }
    "Keep taking it until a pharmacist, nurse or GP tells you to stop, and ask for a review \
     rather than deciding on your own: tablets for the heart, blood pressure, thyroid, \
     epilepsy, steroids and antidepressants can be dangerous to stop suddenly. If it is not \
     suiting you, the fix is usually the dose, the brand or the time of day, so say what \
     has changed and ask. If you have already stopped and feel unwell, ring the pharmacist \
     today or 111."
        .into()
}

/// A missed or muddled dose: confusion, running out, the critical medicines, a routine that
/// keeps slipping, antibiotics, insulin, a double dose, or the plain missed dose; "" for a line
/// that only states a regimen.
pub fn _missed_medication(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    if _MED_CONFUSED.is_match(&clean) {
        if _MED_TABLET_MUDDLE.is_match(&clean) && !_MED_CONFUSED_NEW.is_match(&clean) {
            let pattern = if _MED_THIRD_PERSON.is_match(&clean) { format!(" {}", *_MED_PATTERN) } else { String::new() };
            return format!("{}{}", *_MED_BOX, pattern);
        }
        return _MED_CONFUSED_ANSWER.clone();
    }
    if _MED_RAN_OUT.is_match(&clean) {
        return _MED_RAN_OUT_ANSWER.clone();
    }
    if _MED_CRITICAL.is_match(&clean) {
        return _MED_CRITICAL_ANSWER.clone();
    }
    if _MED_REPEAT.is_match(&clean) && _MED_MISSED.is_match(&clean) {
        let sugar = if _MED_SUGAR.is_match(&clean) { format!(" {}", *_MED_SUGAR_LINE) } else { String::new() };
        return format!("{}{}", *_MED_ROUTINE, sugar);
    }
    if _MED_ANTIBIOTIC.is_match(&clean) {
        return _MED_ANTIBIOTIC_ANSWER.clone();
    }
    if _MED_MISSED.is_match(&clean) && _MED_SUGAR.is_match(&clean) && !_MED_REPEAT.is_match(&clean) {
        return _MED_INSULIN.clone();
    }
    if _MED_TOO_MUCH.is_match(&clean) && !_MED_MISSED.is_match(&clean) {
        return _MED_TOOK_TWO.clone();
    }
    if _MED_MISSED.is_match(&clean) && _MED_DOUBLE.is_match(&clean) {
        return format!("{} {}", *_MED_DOUBLE_ANSWER, *_MED_BOX);
    }
    if _MED_MISSED.is_match(&clean) && !_MED_REPEAT.is_match(&clean) {
        return format!(
            "Skip the missed dose and take the next one at its usual time, and do not take two \
             to catch up. {}",
            *_MED_BOX
        );
    }
    if _MED_THIRD.match_(&clean).is_some() {
        return format!("{} {}", *_MED_BOX, *_MED_PATTERN);
    }
    if !_MED_REPEAT.is_match(&clean) {
        return String::new();
    }
    let mut reply = _MED_ROUTINE.clone();
    if _MED_SUGAR.is_match(&clean) {
        reply.push(' ');
        reply.push_str(&_MED_SUGAR_LINE);
    }
    reply
}

// ---------------------------------------------------------------------------------------------
// Caffeine.

/// Caffeine in pregnancy or while breastfeeding: the 200mg figure.
pub fn _caffeine_while_expecting(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    let figure = "200mg of caffeine a day, about two mugs of coffee or four cups of tea: half the \
                  usual limit. It is also in cola, energy drinks, chocolate and some cold remedies, \
                  so those count too. Stay under the figure rather than at it";
    if ["breastfeeding", "breast feeding", "nursing", "feeding"].iter().any(|w| clean.contains(w)) {
        return format!(
            "While you are breastfeeding the advice is the same as in pregnancy: {figure}. A coffee \
             before a feed is fine, and the caffeine peaks in your milk an hour later, so the \
             evening ones are the ones to make decaf."
        );
    }
    format!(
        "In pregnancy the advice is {figure}, and the same 200mg is what to keep to while you are \
         trying or breastfeeding."
    )
}

/// Whether one of the last three user turns was about a child.
pub fn _child_in_chat(memory: &Memory) -> bool {
    let start = memory.turns.len().saturating_sub(3);
    for turn in &memory.turns[start..] {
        let text = text::squash(&turn.user.to_lowercase());
        if text.is_empty() {
            continue;
        }
        let words = text::split_ref(&text);
        if _KID_WORDS.iter().any(|w| words.contains(&w.as_str())) || text.contains('$') {
            return true;
        }
        if sh::any_in(&text, &_TEEN_AGE_PHRASES) || text.contains("under 12") {
            return true;
        }
    }
    false
}

/// A daily figure as people say it: a whole number, else one decimal place.
pub fn _per_day(daily: f64) -> String {
    let whole = daily.trunc();
    if (daily - whole).abs() < 0.05 {
        return format!("{}", whole as i64);
    }
    let s = format!("{daily:.1}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The caffeine limit: the jitters, a child's, a count over or inside the limit, a figure in
/// mg, or the 400mg limit; "" for a plain mention of coffee.
pub fn _caffeine_limit(memory: &Memory, understood: &Understood) -> String {
    static I_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi\b", 0));
    static A_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b\d+\b", 0));
    static MILLIGRAM: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{2,4})\s*(?:mg|milligrams?)\b", 0));
    let clean = understood.clean.to_lowercase();
    if sh::any_in(&clean, &_CAFFEINE_SYMPTOM) {
        return "Those are the usual signs of too much caffeine: cut back to a cup or two in the \
                morning and make the rest decaf, which keeps the habit without the jitters. Taper \
                over a week rather than stopping dead, or the headaches will be worse than the \
                shakes, and if your heart pounds for no reason, get it checked."
            .into();
    }
    if _child_in_chat(memory)
        && _PRONOUN_SUBJECT.is_match(&clean)
        && !I_WORD.is_match(&clean)
        && !A_NUMBER.is_match(&clean)
        && !_CAFFEINE_COUNT.is_match(&clean)
    {
        return _KIDS_LINE.clone();
    }
    // (per day, count, unit, period) of the heaviest count said
    let mut heaviest: Option<(f64, i64, String, String)> = None;
    for found in _CAFFEINE_COUNT.finditer(&clean) {
        let count = dup::py_int(found.str(1)).expect("int(count)");
        let unit = found.str(2).to_string();
        let period = found.get(3).filter(|p| !p.is_empty()).unwrap_or("day").to_string();
        let daily = if period == "week" { count as f64 / 7.0 } else { count as f64 };
        if heaviest.as_ref().is_none_or(|h| daily > h.0) {
            heaviest = Some((daily, count, unit, period));
        }
    }
    if let Some((daily, count, unit, period)) = &heaviest {
        if *daily >= 5.0 {
            return format!(
                "{count} {unit} a {period} is well over the 400mg adult limit, which is about four \
                 instant coffees or two large filter ones; a cup of tea is around 50mg against 80mg \
                 or more in a coffee. Come down a cup every few days and make the later ones decaf, \
                 and the sleep and the jitters both settle."
            );
        }
    }
    if let Some(m) = MILLIGRAM.search(&clean) {
        if ["a lot", "lot", "ok", "okay", "alright", "fine", "safe", "too much", "bad"].iter().any(|w| clean.contains(w)) {
            let amount = dup::py_int(m.str(1)).expect("int(mg)");
            if amount >= 400 {
                return format!(
                    "{amount}mg a day is at or over the 400mg adult limit - four instant coffees or \
                     two large filter ones - so it is the point to come down from, a cup every few \
                     days rather than all at once. Keep it under 200mg if you are pregnant, and if \
                     it comes with a pounding heart or bad sleep, get that looked at."
                );
            }
            return format!(
                "{amount}mg is under the 400mg adult limit, which is about four instant coffees or \
                 two large filter ones, so on the caffeine count alone it is fine. It is still worth \
                 keeping the last one before early afternoon if sleep is a problem, and the limit \
                 halves to 200mg in pregnancy."
            );
        }
    }
    if !["much", "too", "limit", "safe", "many", "a lot", "lot", "ok", "okay", "alright", "fine", "bad"]
        .iter()
        .any(|w| clean.contains(w))
    {
        if let Some((daily, count, unit, period)) = &heaviest {
            return format!(
                "{count} {unit} a {period} is about {} a day, which is inside the 400mg adult limit - \
                 that is four instant coffees or two large filter ones. Keep the last one before \
                 early afternoon if sleep is a problem, and stay under 200mg a day if you are \
                 pregnant.",
                _per_day(*daily)
            );
        }
        return String::new();
    }
    "Around 400mg of caffeine a day is the usual safe limit for an adult, which is about \
     four cups of instant coffee or two large filter coffees; pregnant women are advised to \
     keep to 200mg. Stop by early afternoon if your sleep is affected, and cut down slowly \
     over a week or two, since going cold turkey can bring headaches and irritability."
        .into()
}

/// The caffeine limit for the drink the line names (herbal tea has none); None when the line
/// asks nothing about how much.
pub fn _the_limit_for_the_drink_named(_memory: &Memory, reading: &Understood) -> Option<String> {
    let clean = reading.clean.to_lowercase();
    if !reading.words.iter().any(|w| _LIMIT_ASKS.contains(w)) {
        return None;
    }
    let herbal = &_CAFFEINE_DRINKS.iter().find(|(name, _)| name == "herbal").expect("herbal").1;
    if sh::any_in(&clean, herbal) {
        return Some(format!("{}.", _CAFFEINE_LEAD["herbal"]));
    }
    let found: Vec<&str> = _CAFFEINE_DRINKS
        .iter()
        .filter(|(name, phrases)| name != "herbal" && sh::any_in(&clean, phrases))
        .map(|(name, _)| name.as_str())
        .collect();
    if found.len() == 1 {
        if let Some(lead) = _CAFFEINE_LEAD.get(found[0]) {
            return Some(format!("{lead}{}", *_CAFFEINE_TAIL));
        }
    }
    if found.contains(&"tea") && found.contains(&"coffee") {
        return Some(format!("{}{}", _CAFFEINE_LEAD["coffee"], *_CAFFEINE_TAIL));
    }
    Some(format!("{}{}", _CAFFEINE_LEAD["coffee"], *_CAFFEINE_TAIL))
}

// ---------------------------------------------------------------------------------------------
// Colds, headaches, periods, hiccups.

/// A cold: what helps, or how to catch fewer when the line asks about preventing one.
pub fn _cold_help(_memory: &Memory, understood: &Understood) -> String {
    // `norm` is a list, not a str: only clean and norm_text are read.
    let said = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    if sh::any_in(&said, &_COLD_NAMED) {
        return _COLD_HELP.clone();
    }
    if sh::any_in(&said, &_COLD_PREVENT) {
        return _COLD_STOP.clone();
    }
    _COLD_HELP.clone()
}

/// How many days the words say a headache has gone on (3 for "every day" or a bare "days"),
/// or None when they do not say.
pub fn _headache_days<S: AsRef<str>>(words: &[S]) -> Option<i64> {
    let words: Vec<&str> = words.iter().map(|w| w.as_ref()).collect();
    let said = words.join(" ");
    if sh::any_in(&said, &_EVERY_DAY) {
        return Some(3);
    }
    let mut best: Option<i64> = None;
    for (at, word) in words.iter().enumerate() {
        let Some(&unit) = _UNIT_DAYS.get(*word) else { continue };
        let mut count: Option<i64> = None;
        for back in 1..=3usize {
            if at < back {
                break;
            }
            let before = words[at - back];
            if text::is_digit(before) {
                count = Some(dup::py_int(before).expect("int(before)"));
                break;
            }
            if before == "a" || before == "an" {
                count = Some(1);
                break;
            }
            if let Some(&n) = _NUMBER_WORDS.get(before) {
                count = Some(n);
                break;
            }
            if !["a", "an", "of", "the", "this", "last", "past", "few"].contains(&before) {
                break;
            }
        }
        if count.is_none() {
            if (*word == "week" || *word == "weeks")
                && ["all week", "past week", "last week", "over a week", "a week", "this past"]
                    .iter()
                    .any(|p| said.contains(p))
            {
                count = Some(7);
            } else if *word == "day" || *word == "days" {
                count = if said.contains("all day") { Some(1) } else { None };
            }
        }
        let Some(count) = count else { continue };
        let days = count.saturating_mul(unit);
        best = Some(best.map_or(days, |b| b.max(days)));
    }
    if best.is_some() {
        return best;
    }
    if sh::any_in(&said, &_BARE_LENGTHS) {
        return Some(3);
    }
    None
}

/// The see-a-GP card for a headache of three days or more; None for a shorter one.
pub fn _headache_days_reply(_memory: &Memory, understood: &Understood) -> Option<String> {
    match _headache_days(&understood.words) {
        Some(days) if days >= 3 => Some(HEADACHE_DAYS.clone()),
        _ => None,
    }
}

/// A headache: the no-tablets card when the line asks for that, else the afternoon one.
pub fn _headache_reply(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    if sh::any_in(&text, &NO_DRUGS) {
        return HEADACHE_NO_DRUGS.clone();
    }
    _HEADACHE_AFTERNOON.clone()
}

/// Periods: a change in pattern, painful periods, or heavy ones; None for a condition or
/// somebody else's period.
pub fn _period_answer(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<&str> = understood.words.iter().map(String::as_str).collect();
    let clean = sh::padded(&understood.clean.to_lowercase());
    let named = |list: &[String]| list.iter().any(|w| words.contains(w.as_str()) || clean.contains(&format!(" {w} ")));
    if named(&NOT_THIS_PERSONS_PERIOD) {
        return None;
    }
    if !named(&_HEAVY_PERIOD_WORDS) && named(&_PERIOD_CHANGE_WORDS) {
        return Some(PERIOD_CHANGE_REPLY.clone());
    }
    if !named(&_HEAVY_PERIOD_WORDS) {
        return Some(
            "Period pain that is bad enough to stop your day is worth taking seriously: \
             ibuprofen works better than paracetamol for it, started a day before, and heat \
             on your lower tummy helps. Pain that is getting worse month by month, or that \
             has started later in life, is the pattern to describe to a GP - it can be \
             endometriosis, which takes years to get diagnosed if nobody is told."
                .into(),
        );
    }
    Some(
        "Heavy periods are common and treatable: soaking a pad or tampon every hour, clots \
         bigger than a 10p coin, or bleeding past seven days are the things to describe to a \
         GP, who can check for fibroids, thyroid trouble or a copper coil and offer tablets, \
         the pill or a coil that thins the lining. Eat iron - red meat, beans, fortified \
         cereal - and go the same day if you are flooding or feel faint."
            .into(),
    )
}

/// Whether a bot turn in the chat already opened with these words.
pub fn _said_before(memory: &Memory, opening: &str) -> bool {
    memory.turns.iter().any(|t| t.bot.starts_with(opening))
}

/// Hiccups: a baby's (again, if the baby card was already given), or the adult remedies.
pub fn _hiccups_remedy(memory: &Memory, understood: &Understood) -> String {
    if understood.words.iter().any(|w| _BABY_WORDS.contains(w)) {
        if _said_before(memory, &text::slice(&HICCUPS_BABY, 0, Some(24)))
            || _said_before(memory, &text::slice(&HICCUPS_BABY_AGAIN, 0, Some(24)))
        {
            return HICCUPS_BABY_AGAIN.clone();
        }
        return HICCUPS_BABY.clone();
    }
    "Sip cold water slowly, hold your breath for a few seconds, swallow a few times or \
     breathe into a paper bag, and one of those stops most hiccups within a couple of \
     minutes. They start when a big meal, a fizzy drink or a laugh pushes the stomach up \
     against the diaphragm, which is why they pass on their own. A GP can help if they \
     last more than two days, or come with pain or trouble swallowing."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Steps.

/// How many steps up a landmark (or a flight of stairs), the one named first.
pub fn _steps_up_a_building(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    let (lead, rest): (&String, [&String; 3]) = if said.contains("pisa") || said.contains("leaning tower") {
        (&STEPS_PISA, [&STEPS_EMPIRE, &STEPS_PAULS, &STEPS_FLIGHT])
    } else if said.contains("empire") {
        (&STEPS_EMPIRE, [&STEPS_PISA, &STEPS_PAULS, &STEPS_FLIGHT])
    } else if said.contains("paul") || said.contains("dome") || said.contains("cathedral") {
        (&STEPS_PAULS, [&STEPS_PISA, &STEPS_EMPIRE, &STEPS_FLIGHT])
    } else if ["flight", "floor", "storey", "story", "riser", "stair"].iter().any(|w| said.contains(w)) {
        (&STEPS_FLIGHT, [&STEPS_PISA, &STEPS_EMPIRE, &STEPS_PAULS])
    } else {
        (&STEPS_PISA, [&STEPS_EMPIRE, &STEPS_PAULS, &STEPS_FLIGHT])
    };
    let mut parts: Vec<&str> = vec![lead.as_str()];
    parts.extend(rest.iter().map(|s| s.as_str()));
    parts.push(STEPS_TAIL.as_str());
    parts.join(" ")
}

/// The miles a reported step count comes to, else how many steps make a mile.
pub fn _steps_in_a_distance(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    if let Some(found) = STEPS_COUNT.search(&said) {
        if sh::any_in(&said, &STEPS_REPORT_WORDS) {
            let count = dup::py_int(&found.str(1).replace(',', "")).expect("int(count)");
            if (1000..=100000).contains(&count) {
                let miles: f64 = format!("{:.1}", count as f64 / 2000.0).parse().unwrap_or(0.0);
                let miles = if miles.fract() == 0.0 { format!("{}", miles as i64) } else { text::float_repr(miles) };
                return format!(
                    "That's about {miles} miles - {} steps at roughly 2,000 to a mile. A 5k is about \
                     6,500 steps and a marathon around 55,000, so that is a proper walk.",
                    text::thousands(count)
                );
            }
        }
    }
    STEPS_A_MILE.clone()
}

/// The food the line blames for the heartburn ("curry gives me heartburn"), "" for none.
pub fn _heartburn_blames(said: &str) -> String {
    for pattern in [&*_HEARTBURN_REPORT_BEFORE, &*_HEARTBURN_REPORT_AFTER] {
        let Some(found) = pattern.search(said) else { continue };
        let words: Vec<&str> = text::split_ref(found.str(1))
            .into_iter()
            .filter(|w| !text::strip_chars(w, "'").is_empty() && !_HEARTBURN_NOT_A_FOOD.contains(*w))
            .collect();
        if words.is_empty() {
            continue;
        }
        let phrase = words[words.len().saturating_sub(3)..].join(" ");
        return if dup::lookup_names_a_thing(&phrase) { phrase } else { String::new() };
    }
    String::new()
}

/// Whether any of the words is in the text, as whole words (`bounded`) or anywhere.
fn any_word_hit<S: AsRef<str>>(words: &[S], said: &str, bounded: bool) -> bool {
    words.iter().any(|w| {
        let e = sh::escape(w.as_ref());
        let pattern = if bounded { format!(r"\b{e}\b") } else { e };
        sh::re_search(&pattern, said, 0)
    })
}

/// Heartburn: in pregnancy, tablets not working, tablets that cause it, a remedy, tea, a named
/// trigger, what to eat, which foods, why, a blamed food, or the treatment card.
pub fn _heartburn(_memory: &Memory, understood: &Understood) -> String {
    static TEA: LazyLock<Re> = LazyLock::new(|| Re::new(r"\btea\b", 0));
    static MINT_TEA: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(mint|peppermint|spearmint) tea\b", 0));
    // `norm` is a list, not a str: only clean and norm_text are read.
    let said = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    if any_word_hit(&_HEARTBURN_PREGNANT_WORDS, &said, true) {
        return _HEARTBURN_PREGNANT.clone();
    }
    let tablets_or_treatment: Vec<&String> =
        _HEARTBURN_TABLETS_WORDS.iter().chain(_HEARTBURN_TREATMENT_WORDS.iter()).collect();
    if any_word_hit(&_HEARTBURN_NOT_WORKING_WORDS, &said, false) && any_word_hit(&tablets_or_treatment, &said, true) {
        return _HEARTBURN_TABLETS_NOT_WORKING.clone();
    }
    if any_word_hit(&_HEARTBURN_PILL_WORDS, &said, true) {
        return _HEARTBURN_PILLS.clone();
    }
    if any_word_hit(&_HEARTBURN_TABLETS_WORDS, &said, true) {
        return _HEARTBURN_PILLS.clone();
    }
    if any_word_hit(&_HEARTBURN_TREATMENT_WORDS, &said, true) {
        return _HEARTBURN_ANSWER.clone();
    }
    if TEA.is_match(&said) && !MINT_TEA.is_match(&said) {
        return _HEARTBURN_TEA.clone();
    }
    for (words, label) in _HEARTBURN_TRIGGERS.iter() {
        if any_word_hit(words, &said, true) {
            return label.clone();
        }
    }
    if any_word_hit(&_HEARTBURN_EAT_WORDS, &said, false) {
        return _HEARTBURN_EAT.clone();
    }
    let blamed = _heartburn_blames(&said);
    let asks_which = [
        "which", "what foods", "what food", "food", "foods", "diet", "trigger", "triggers", "causes",
        "cause", "avoid", "cut out", "what should", "list", "what can i eat",
    ];
    if any_word_hit(&asks_which, &said, true) && blamed.is_empty() {
        return _HEARTBURN_WHICH.clone();
    }
    if _any_hit(&[r"\bwhy\b", r"\bwhat causes\b"], &said) {
        return _HEARTBURN_WHY.clone();
    }
    if !blamed.is_empty() {
        return _HEARTBURN_BLAMED.replacen("{}", &blamed, 1);
    }
    _HEARTBURN_ANSWER.clone()
}

/// What water freezes at, or why ice is slippery.
pub fn _freezing_point(_memory: &Memory, understood: &Understood) -> String {
    static SLIPPERY: LazyLock<Re> = LazyLock::new(|| Re::new(r"slippery|slippy|slips? |slip on|skate|skating|glide", 0));
    let line = sh::clean_or_text(understood).to_lowercase();
    if SLIPPERY.is_match(&line) {
        return "Ice is slippery because a skate or a boot melts a thin film of water under \
                it, so you are sliding on water rather than on ice, and the film keeps \
                refreezing as you go over it. Water freezes at 0C (32F); salt lowers that, \
                which is why roads are gritted."
            .into();
    }
    WATER_FREEZES.clone()
}

/// The step count the line says (200 to 100,000), else the default.
pub fn _step_count_said(said: &str, default: f64) -> f64 {
    if let Some(found) = STEPS_COUNT.search(said) {
        let count = dup::py_int(&found.str(1).replace(',', "")).expect("int(count)");
        if (200..=100000).contains(&count) {
            return count as f64;
        }
    }
    default
}

/// The steps in a distance the line names (miles or km), or None.
pub fn _steps_from_a_distance(said: &str) -> Option<f64> {
    let rows: [(&str, f64); 8] = [
        ("mile", *STEPS_PER_MILE),
        ("miles", *STEPS_PER_MILE),
        ("km", *STEPS_PER_KM),
        ("kms", *STEPS_PER_KM),
        ("kilometre", *STEPS_PER_KM),
        ("kilometres", *STEPS_PER_KM),
        ("kilometer", *STEPS_PER_KM),
        ("kilometers", *STEPS_PER_KM),
    ];
    for (word, steps) in rows {
        if sh::re_search(&format!(r"\b{word}\b"), said, 0) {
            let count = sh::re_cached(&format!(r"(\d+(?:\.\d+)?)\s*{word}\b"), 0).search(said);
            let many = match count {
                Some(m) => m.str(1).parse::<f64>().expect("float(count)"),
                None => 1.0,
            };
            return Some(many * steps);
        }
    }
    None
}

/// The steps in a time the line names (hours, half an hour, minutes), or None.
pub fn _steps_from_a_time(said: &str) -> Option<f64> {
    static HOURS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d+(?:\.\d+)?)\s*(?:hours?|hrs?)\b", 0));
    static AN_HOUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:an|one|a single)\s+hour\b", 0));
    static HALF_HOUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhalf an hour\b", 0));
    static MINUTES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d+)\s*(?:minutes|minute|mins|min)\b", 0));
    let mut minutes: Option<f64> = None;
    if let Some(found) = HOURS.search(said) {
        minutes = Some(found.str(1).parse::<f64>().expect("float(hours)") * 60.0);
    } else if AN_HOUR.is_match(said) {
        minutes = Some(60.0);
    } else if HALF_HOUR.is_match(said) {
        minutes = Some(30.0);
    } else if let Some(found) = MINUTES.search(said) {
        minutes = Some(found.str(1).parse::<f64>().expect("float(minutes)"));
    }
    match minutes {
        Some(m) if m != 0.0 => Some(6250.0 * m / 60.0),
        _ => None,
    }
}

/// The calories in the steps (or distance, or time) the line names, 0.04 to 0.05 a step.
pub fn _steps_calories(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.clean.to_lowercase();
    let steps = _steps_from_a_distance(&said)
        .or_else(|| _steps_from_a_time(&said))
        .unwrap_or_else(|| _step_count_said(&said, 10000.0));
    let low = text::round_half_even(steps * 0.04 / 10.0) * 10;
    let high = text::round_half_even(steps * 0.05 / 10.0) * 10;
    format!(
        "About {} to {} calories in {} steps - roughly 0.04 to 0.05 of a calorie a step, which is \
         the figure for an average adult. It climbs with your weight and your pace, and it is a \
         small number next to what a snack costs: a biscuit is often a whole mile's worth, so the \
         walking is for the habit and the heart as much as the calories.",
        text::thousands(low),
        text::thousands(high),
        _said_steps(steps)
    )
}

/// A length of time as people say it ("an hour", "half an hour", "2 hours", "40 minutes").
pub fn _duration_phrase(minutes: f64) -> String {
    if (minutes - 60.0).abs() < 0.5 {
        return "an hour".into();
    }
    if (minutes - 30.0).abs() < 0.5 {
        return "half an hour".into();
    }
    if minutes.rem_euclid(60.0).abs() < 0.5 {
        let hours = (minutes / 60.0).floor() as i64;
        return format!("{} hour{}", hours, if hours == 1 { "" } else { "s" });
    }
    format!("{} minutes", text::round_half_even(minutes))
}

/// A step count rounded to a figure people say (to the 1,000, 500, 250 or 50).
pub fn _round_steps(value: f64) -> i64 {
    for step in [1000.0, 500.0, 250.0, 50.0] {
        if value >= step * 4.0 {
            return text::round_half_even(value / step) * step as i64;
        }
    }
    text::round_half_even(value)
}

/// A rounded step count with thousands commas.
pub fn _said_steps(value: f64) -> String {
    text::thousands(_round_steps(value))
}

/// The steps in the time the line names, at an ordinary and a brisk pace; or an hour's worth.
pub fn _steps_in_a_time(_memory: &Memory, understood: &Understood) -> String {
    static HOURS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d+(?:\.\d+)?)\s*(?:hours?|hrs?)\b", 0));
    static HALF_HOUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhalf an hour\b", 0));
    static MINUTES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d+)\s*(?:minutes|minute|mins|min)\b", 0));
    static BRISK: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(brisk|fast|quick|power|uphill|hilly)\b", 0));
    let said = understood.clean.to_lowercase();
    let mut minutes: Option<f64> = None;
    if let Some(found) = HOURS.search(&said) {
        minutes = Some(found.str(1).parse::<f64>().expect("float(hours)") * 60.0);
    } else if HALF_HOUR.is_match(&said) {
        minutes = Some(30.0);
    } else if let Some(found) = MINUTES.search(&said) {
        minutes = Some(found.str(1).parse::<f64>().expect("float(minutes)"));
    }
    let brisk = BRISK.is_match(&said);
    let given = minutes.filter(|m| *m != 0.0);
    let ordinary = 6250.0 * given.unwrap_or(60.0) / 60.0;
    let quick = 7500.0 * given.unwrap_or(60.0) / 60.0;
    if let Some(minutes) = given {
        return format!(
            "About {} steps in {} of walking at an ordinary pace, or {} if you are walking briskly \
             - that is 1,250 steps a kilometre at about 5km/h. Hills, a slow stroll and short legs \
             all move it.",
            _said_steps(ordinary),
            _duration_phrase(minutes),
            _said_steps(quick)
        );
    }
    if brisk {
        return format!(
            "About {} steps in an hour of brisk walking - 6.5km/h or so, which is 1,250 steps a \
             kilometre over six and a half of them. An ordinary-paced hour is about {}, and a slow \
             wander round the shops is well under that.",
            _said_steps(7500.0),
            _said_steps(6250.0)
        );
    }
    format!(
        "About {} to {} steps in an hour of walking: an ordinary pace is about 5km/h, which is \
         1,250 steps a kilometre, so an hour gets you five of them. A brisk walk nearer 6.5km/h is \
         the top of that range, and a slow wander round shops is well under it.",
        _said_steps(6250.0),
        _said_steps(7500.0)
    )
}

/// Forgetting: a word on the tip of the tongue, a blank mid-sentence, forgetting things at
/// work, or how memory fades.
pub fn _forgetting(_memory: &Memory, understood: &Understood) -> String {
    static WORD: LazyLock<Re> = LazyLock::new(|| {
        Re::new(
            r"\b(?:tip of my tongue|the word (?:i|for)|a word for|word for|can'?t find the word|cant find the word|find the words?)\b",
            0,
        )
    });
    static SAYING: LazyLock<Re> = LazyLock::new(|| {
        Re::new(r"\b(?:saying|say|said|train of thought|mid sentence|mid-sentence|what i was about to)\b", 0)
    });
    static AT_WORK: LazyLock<Re> = LazyLock::new(|| {
        Re::new(
            r"\b(?:at work|in work|at the office|work\b|tasks?|to-?do|deadlines?|meetings?|emails?|appointments?|errands?|shopping list|birthdays?)\b",
            0,
        )
    });
    let said = sh::or(&understood.clean, &understood.plain).to_lowercase();
    if WORD.is_match(&said) {
        return "Word-finding blanks are ordinary and commoner when you are tired, stressed or \
                under pressure: describe the word out loud, or let it go and get on with the \
                sentence, because it usually turns up a few minutes later once you stop \
                reaching for it. Run through the alphabet or the first letter if you want to \
                help it along. If you are losing everyday words often, or the wrong word comes \
                out and you do not notice, that is worth a word with a GP."
            .into();
    }
    if SAYING.is_match(&said) {
        return "That is usually not memory loss: a blank mid-sentence comes from being tired, \
                distracted or holding too much at once, and it passes. Say the last few words \
                you did get out, or retrace the point you were making, and the rest normally \
                follows; jotting the thing down before you go on stops it slipping again. If it \
                starts happening most days, or it comes with losing everyday words and getting \
                lost in familiar places, see a GP and say how long it has been going on."
            .into();
    }
    if AT_WORK.is_match(&said) {
        return "Keep one list instead of holding it in your head - a notebook or the notes app, \
                written down the moment it comes up rather than later, because the thing you \
                mean to remember is the thing that goes. Put a reminder on your phone for \
                anything with a time on it, and if a thing takes under two minutes do it now \
                rather than noting it. Keep a fixed place for keys, wallet and phone, and repeat \
                a name back once when you are introduced. Most of this is too much at once \
                rather than a fading memory, so sleep and one thing at a time help; if you are \
                forgetting recent conversations or the names of people you know well, get it \
                checked."
            .into();
    }
    "Most forgetting is not a lost file: new memories fade unless they are slept on and \
     retrieved, and the brain clears out details to keep the gist; being tired, stressed \
     or trying to hold too much at once makes it worse. Write things down and sleep on \
     it; if you are forgetting recent conversations or the names of people you know \
     well, get it checked."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Guards on the odd word.

/// True when the line's words ask about itching, picking or scratching.
pub fn _asks_about_itching(understood: &Understood) -> bool {
    let text = understood.words.join(" ");
    let words = text::split_ref(&text);
    ["itch", "itches", "itchy", "itching", "pick", "picking", "scratch", "scratching"]
        .iter()
        .any(|w| words.contains(w))
}

/// True unless the line asks about itching.
pub fn _not_about_itching(understood: &Understood) -> bool {
    !_asks_about_itching(understood)
}

/// False when the "factor" in the line is arithmetic (two numbers, "factors of 12", a number
/// that is not a sunscreen factor), not a sun factor.
pub fn _a_factor_question_without_the_sun(understood: &Understood) -> bool {
    let text = understood.clean.to_lowercase();
    if !text.contains("factor") {
        return true;
    }
    if ["sun", "spf", "suncream", "sun cream", "sunscreen", "sunblock", "sun factor"].iter().any(|w| text.contains(w)) {
        return true;
    }
    let spaced = text.replace('?', " ").replace(',', " ").replace('.', " ");
    let words = text::split_ref(&spaced);
    let numbers: Vec<&str> = words.iter().copied().filter(|w| text::is_digit(w)).collect();
    if numbers.len() > 1 || (words.contains(&"of") && !numbers.is_empty()) {
        return false;
    }
    if !numbers.is_empty() && (words.contains(&"divisible") || words.contains(&"have") || words.contains(&"has")) {
        return false;
    }
    if numbers.iter().any(|n| !_SUN_FACTORS.iter().any(|f| f == n)) {
        return false;
    }
    !["multiple", "multiples", "highest", "common"].iter().any(|w| text.contains(w))
}

/// False when the line names a burn and nothing about the sun (that is the burns card's).
pub fn _a_burn_without_the_sun(understood: &Understood) -> bool {
    let text = understood.clean.to_lowercase();
    if !["burn", "burns", "burning", "burnt"].iter().any(|w| text.contains(w)) {
        return true;
    }
    [
        "sun", "sunlight", "sunshine", "sun cream", "suncream", "sunscreen", "sunblock", "spf", "uv",
        "sunbed", "sunbed", "sun factor", "factor 30", "factor 50",
    ]
    .iter()
    .any(|w| text.contains(w))
}

/// The dizzy-spell card; "" when the safety module has an open case that is not safe.
pub fn _a_dizzy_spell(memory: &Memory, _understood: &Understood) -> String {
    if dup::memory_truthy(memory, "safety_case") && !dup::memory_truthy(memory, "safety_safe") {
        return String::new();
    }
    DIZZY_SPELL.clone()
}

/// False when the pounds in the line are money (a tip, a bill, a price), not weight.
pub fn _the_pounds_are_weight(understood: &Understood) -> bool {
    !POUNDS_THAT_ARE_MONEY.is_match(sh::clean_or_text(understood))
}

/// False when "how much / many / long" is about something other than what is remembered, or
/// the thing lost is a temper, a job or sleep.
pub fn _an_amount_of_losing(understood: &Understood) -> bool {
    let text = sh::clean_or_text(understood);
    if LOSING_SOMETHING_ELSE.is_match(text) {
        return false;
    }
    if !AN_AMOUNT_OF_IT.is_match(text) {
        return true;
    }
    WHAT_IS_REMEMBERED.is_match(text)
}

/// True when every word of the line is one a question about what water is uses.
pub fn _water_is_the_subject(understood: &Understood) -> bool {
    understood.words.iter().all(|w| WATER_IS_THE_SUBJECT.contains(&w.to_lowercase()))
}

/// False when the cold in the line is food (cold rice, leftovers) or the reading has a warmth.
pub fn _not_a_food_cold(understood: &Understood) -> bool {
    if understood.warmth.as_deref().is_some_and(|w| !w.is_empty()) {
        return false;
    }
    !_COLD_FOOD.is_match(sh::text_or_clean(understood))
}

// ---------------------------------------------------------------------------------------------
// Baby sleep.

/// The baby's age in months from "6 weeks old", "two years old", ...; None when not said.
pub fn _the_age_in_months(said: &str) -> Option<f64> {
    let found = _A_AGE.search(said)?;
    let number = found.name_str("n");
    let count = match _A_NUMBER_WORDS.get(number) {
        Some(&n) => n,
        None => dup::py_int(number)?,
    };
    if count <= 0 {
        return None;
    }
    let unit = found.name_str("unit").replace('-', "");
    if unit == "week" {
        return Some(count as f64 / 4.345);
    }
    if unit == "year" {
        return Some((count * 12) as f64);
    }
    Some(count as f64)
}

/// How much a baby of the age the line says sleeps, reassuring a worried parent.
pub fn _a_baby_sleep_reply(memory: &Memory, understood: &Understood) -> Option<String> {
    static TODDLER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\btoddlers?\b", 0));
    let mut said = sh::squash_lower(sh::text_or_clean(understood));
    if said.is_empty() {
        // `memory.get("turns", [{}])[-1].get("text", "")`: an empty list of turns raises, and a
        // turn has no "text".
        memory.turns.last()?;
        said = String::new();
    }
    let mut months = _the_age_in_months(&said);
    if months.is_none() && TODDLER.is_match(&said) {
        months = Some(18.0);
    }
    let Some(months) = months else {
        return Some(_BABY_SLEEP_PLAIN.clone());
    };
    let Some(band) = _BABY_SLEEP_BANDS.iter().find(|(low, high, _)| *low <= months && months < *high).map(|b| &b.2)
    else {
        return Some(_BABY_SLEEP_PLAIN.clone());
    };
    let tail = _the_baby_sleep_tail(Some(months));
    if _A_WORRY.is_match(&said) {
        let opening = if tail.is_empty() { "Sleeping a lot is usually fine: " } else { "A baby sleeping a lot is usually fine: " };
        return Some(format!("{opening}{band}.{tail}"));
    }
    Some(format!("{}.{tail}", text::upper_first(band)))
}

/// The feeding-and-nappies tail for a baby under a year; "" for an older child.
pub fn _the_baby_sleep_tail(months: Option<f64>) -> String {
    if months.is_some_and(|m| m >= *_BABY_SLEEP_TAIL_UNDER_MONTHS) {
        return String::new();
    }
    format!(" {}", *_BABY_SLEEP_TAIL)
}

/// The body's record the line asks about (the strongest bone, the smallest muscle...); None
/// for an animal's or when nothing matches.
pub fn _what_the_body_is_like(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = sh::padded(&text::strip(sh::clean_or_text(understood)).to_lowercase());
    let said = |word: &str| _what_the_body_is_like__said(&text, word);
    if _AN_ANIMAL.iter().any(|w| said(w)) && !said("human") {
        return None;
    }
    let body_said = _SAYS_A_BODY.iter().any(|w| said(w));
    for (parts, asks, reply) in BODY_RECORDS.iter() {
        let named: Vec<&String> = parts.iter().filter(|w| said(w)).collect();
        if named.is_empty() || (!named.iter().any(|w| !_GENERIC_PARTS.contains(w)) && !body_said) {
            continue;
        }
        if named.iter().any(|w| *w == "prints") && !_FINGER_WORDS.iter().any(|w| said(w)) {
            continue;
        }
        if asks.iter().any(|w| said(w)) {
            return Some(reply.clone());
        }
    }
    None
}

/// `_what_the_body_is_like`'s `said`: the word is in the text as a whole word.
#[allow(non_snake_case)]
pub fn _what_the_body_is_like__said(text: &str, word: &str) -> bool {
    says_word(text, word)
}

/// Every part word the body-record rows are keyed on.
pub fn _the_body_record_key() -> Vec<String> {
    BODY_RECORDS.iter().flat_map(|(parts, _, _)| parts.iter().cloned()).collect()
}

/// Hay fever or a cold (when the line names a cold), else the hay fever advice.
pub fn _hayfever_or_a_cold(_memory: &Memory, understood: &Understood) -> String {
    let said = [understood.clean.as_str(), understood.text.as_str(), understood.norm_text.as_str(), understood.plain.as_str()]
        .join(" ")
        .to_lowercase();
    if _A_A_COLD.is_match(&said) {
        return "Hay fever itches - eyes, nose, throat, the roof of your mouth - and comes in \
                sneezing fits with no fever, in the same weeks each year. A cold starts with a \
                sore throat, then a blocked nose, a cough and often a mild fever, and clears in \
                about a week. In June grass pollen is the usual one, so a week of antihistamine \
                is the test."
            .into();
    }
    HAYFEVER_ADVICE.clone()
}

/// False when the line is about building a model volcano, not what makes one erupt.
pub fn _not_a_volcano_build(understood: &Understood) -> bool {
    let words = understood.words.join(" ");
    let text = sh::padded(sh::or(sh::text_or_clean(understood), &words));
    if !_VOLCANO_BUILD.is_match(&text) {
        return true;
    }
    _VOLCANO_CAUSE.is_match(&text)
}

/// False when the line asks where or how high something goes and names no desk kit.
pub fn _a_place_for_the_kit(understood: &Understood) -> bool {
    let clean = text::squash(&understood.clean);
    if clean.is_empty() {
        return false;
    }
    if !sh::any_in(&clean, &_A_PLACE_SHAPED) {
        return true;
    }
    text::split_ref(&clean).iter().any(|w| _SITTING_KIT.contains(*w))
}

/// Why ice melts, with how long it takes when the line asks.
pub fn _how_long_ice_takes(understood: &Understood) -> String {
    let why = "Ice melts when its molecules take in enough heat from the room or the air to break \
               free of the fixed lattice they are locked in and slide around as liquid. It stays at \
               0C the whole time it is melting, because the heat goes into loosening the bonds \
               rather than raising the temperature.";
    let text = sh::padded(sh::clean_or_text(understood));
    if !_ICE_WHEN.is_match(&text) {
        return why.into();
    }
    format!(
        "An ice cube on the counter is gone in about half an hour in a warm kitchen, and a whole \
         tray left out takes a few hours. In a drink it is quicker still, a few minutes, because \
         the drink is warmer than the freezer. {why}"
    )
}

/// The bruise answer for what the line asks (spreading, faster, staying, hurting), else the
/// plain one.
pub fn _the_bruise_answer(_memory: &Memory, reading: &Understood) -> String {
    let text = sh::padded(&sh::text_or_clean(reading).to_lowercase());
    let spaced = text.replace('?', " ").replace('.', " ").replace(',', " ");
    for (words, answer) in _BRUISE_ASKS.iter() {
        if words.iter().any(|w| spaced.contains(&format!(" {w} "))) {
            return answer.clone();
        }
    }
    _BRUISE_PLAIN.clone()
}

/// True unless every "back" in the line is a direction (body's own test).
pub fn _the_backs_are_a_body(understood: &Understood) -> bool {
    !body::_back_is_a_direction(understood)
}

/// The ice-melt row's reply (`<lambda>:5221`).
pub fn lambda_5221(_memory: &Memory, understood: &Understood) -> String {
    _how_long_ice_takes(understood)
}

/// The memory-assessment row's guard (`<lambda>:6619`): no deed, no habit, no errand, no debt.
pub fn lambda_6619(reading: &Understood) -> bool {
    !dup::repair_owns_the_deed(reading)
        && !_a_habit_without_the_illness(reading)
        && !_the_speaker_forgot_to_do_it(reading)
        && !_its_money_that_has_not_come_back(reading)
}

/// The own-memory row's guard (`<lambda>:6694`): the speaker's own memory, and no deed, habit,
/// errand or debt.
pub fn lambda_6694(reading: &Understood) -> bool {
    _not_somebody_elses_memory(reading)
        && !dup::repair_owns_the_deed(reading)
        && !_a_habit_without_the_illness(reading)
        && !_the_speaker_forgot_to_do_it(reading)
        && !_its_money_that_has_not_come_back(reading)
}

/// The sunscreen row's guard (`<lambda>:7693`): not a maths factor, not a burn without sun.
pub fn lambda_7693(understood: &Understood) -> bool {
    _a_factor_question_without_the_sun(understood) && _a_burn_without_the_sun(understood)
}

// ---------------------------------------------------------------------------------------------
// The sleep clock.

/// A clock time as people write it ("7am", "10.30pm").
pub fn _clock_time(hour: i64, minute: i64) -> String {
    let suffix = if hour < 12 { "am" } else { "pm" };
    let on_clock = match hour.rem_euclid(12) {
        0 => 12,
        h => h,
    };
    if minute == 0 { format!("{on_clock}{suffix}") } else { format!("{on_clock}.{minute:02}{suffix}") }
}

/// The bedtime that many hours (and twenty minutes to drop off) before a wake-up time.
pub fn _back_from(hour: i64, minute: i64, hours: i64) -> (i64, i64) {
    let total = (hour * 60 + minute - hours * 60 - 20).rem_euclid(24 * 60);
    (total / 60, total % 60)
}

/// The wake-up time that many hours (and twenty minutes to drop off) after a bedtime.
pub fn _forward_from(hour: i64, minute: i64, hours: i64) -> (i64, i64) {
    let total = (hour * 60 + minute + 20 + hours * 60).rem_euclid(24 * 60);
    (total / 60, total % 60)
}

/// When to go to bed for the wake-up time the line names (or when to get up for its bedtime),
/// else how to work it back from the morning.
pub fn _sleep_clock(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.as_str();
    let found = _CLOCK_AT.search(text);
    let in_bed = if found.is_some() { None } else { _BED_AT.search(text) };
    let (mut hour, mut minute) = (0i64, 0i64);
    if let Some(m) = found.as_ref().or(in_bed.as_ref()) {
        let minutes = m.get(3).filter(|s| !s.is_empty()).map(dup::py_int).unwrap_or(Some(0));
        match (dup::py_int(m.str(2)), minutes) {
            (Some(h), Some(mi)) => (hour, minute) = (h, mi),
            _ => (hour, minute) = (0, 0),
        }
        if m.get(4) == Some("pm") && hour < 12 {
            hour += 12;
        }
        if in_bed.is_some() && hour < 12 && m.get(4).is_none_or(str::is_empty) {
            hour += 12;
        }
    }
    let in_bed = in_bed.is_some();
    if in_bed && (20..=23).contains(&hour) || in_bed && hour <= 3 {
        let (eh, em) = _forward_from(hour.rem_euclid(24), minute, 8);
        let (nh, nm) = _forward_from(hour.rem_euclid(24), minute, 9);
        return format!(
            "In bed at {}, asleep about twenty minutes later, so up at {} on eight hours or {} on \
             nine - and that alarm is worth keeping the same every day, weekend included, because \
             the fixed morning is what sets the body clock and the bedtime then follows it.",
            _clock_time(hour.rem_euclid(24), minute),
            _clock_time(eh, em),
            _clock_time(nh, nm)
        );
    }
    if (4..=11).contains(&hour) {
        let (eh, em) = _back_from(hour, minute, 8);
        let (nh, nm) = _back_from(hour, minute, 9);
        return format!(
            "To be up at {}, count back eight hours and leave twenty minutes to drop off, so lights \
             out about {} - and about {} if you are a nine-hour sleeper. What matters more than the \
             exact minute is keeping the morning the same every day, weekend included, because that \
             is the end of the night the body clock sets itself by, and the bedtime then falls into \
             place on its own.",
            _clock_time(hour, minute),
            _clock_time(eh, em),
            _clock_time(nh, nm)
        );
    }
    "Work it back from the morning: pick the time you have to be up, count back eight \
     hours and allow twenty minutes to drop off - a half six alarm means lights out about \
     ten, a seven o'clock one about half ten. Nine-hour sleepers go an hour earlier. Keep \
     the wake-up time the same every day, weekends included: a bedtime held while the \
     mornings shift is the wrong way round. If you lie awake an hour, the bedtime is too \
     early, not too late."
        .into()
}

// ---------------------------------------------------------------------------------------------
// Planets, tall things, bridges and buildings.

/// The size of the planet the line names first; None when another body with a size row of its
/// own (sun, moon, earth) is named before it.
pub fn _the_planet_size(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = [understood.clean.as_str(), understood.text.as_str()]
        .iter()
        .filter(|p| !p.is_empty())
        .map(|p| p.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let at = |name: &str| _the_planet_size__at(&said, name);
    let mut found: Vec<(usize, &String)> = Vec::new();
    for (names, answer) in PLANET_SIZES.iter() {
        for name in names {
            if let Some(pos) = at(name) {
                found.push((pos, answer));
            }
        }
    }
    let (first, answer) = found.into_iter().min()?;
    for other in ["sun", "moon", "earth"] {
        if let Some(pos) = at(other) {
            if pos < first {
                return None;
            }
        }
    }
    Some(answer.clone())
}

/// `_the_planet_size`'s `at`: where the name starts as a word of letters, if it is there.
#[allow(non_snake_case)]
pub fn _the_planet_size__at(said: &str, name: &str) -> Option<usize> {
    sh::re_cached(&format!("(?<![a-z]){name}(?![a-z])"), 0).search(said).map(|m| m.start())
}

/// How a tall thing (crane, bridge, chimney, tree, plant) stays up, else a building.
pub fn _tall_thing(_memory: &Memory, understood: &Understood) -> String {
    static CRANES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bcranes?\b", 0));
    static BRIDGES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bbridges?\b", 0));
    static CHIMNEYS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bchimneys?\b", 0));
    static TREES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\btrees?\b|\bforest\b|\bwoods\b", 0));
    static PLANTS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bplants?\b|\bhouseplants?\b|\bflowers?\b|\bhanging basket\b", 0));
    let said = [understood.clean.as_str(), understood.text.as_str()]
        .iter()
        .filter(|p| !p.is_empty())
        .map(|p| p.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    if CRANES.is_match(&said) {
        return CRANE_STANDS.clone();
    }
    if BRIDGES.is_match(&said) {
        return BRIDGE_STANDS.clone();
    }
    if CHIMNEYS.is_match(&said) {
        return CHIMNEY_STANDS.clone();
    }
    if TREES.is_match(&said) {
        return TREE_STANDS.clone();
    }
    if GARDEN_PLANTS.is_match(&said) {
        return GARDEN_PLANT_STANDS.clone();
    }
    if PLANTS.is_match(&said) {
        return PLANT_STANDS.clone();
    }
    BUILDING_STANDS.clone()
}

/// The Channel Tunnel or the longest tunnels, when the line names a tunnel; else None.
pub fn _a_tunnel_named(said: &str) -> Option<String> {
    if !said.contains(" tunnel") && !said.contains(" tunnels") {
        return None;
    }
    if said.contains("channel tunnel") || (said.contains(" channel ") && (said.contains("how long") || said.contains("long"))) {
        return Some(
            "The Channel Tunnel is 50.5 km from end to end, with about 38 km of it under the \
             sea, and a train takes about 35 minutes from Folkestone to Calais. Three tubes \
             run side by side: two for trains and a smaller service tunnel between them, \
             bored through chalk that is soft enough to cut and stiff enough to stand while \
             you do it."
                .into(),
        );
    }
    if said.contains("longest") || said.contains("how long") {
        return Some(
            "The longest railway tunnel in the world is the Gotthard Base Tunnel in \
             Switzerland at 57 km, opened in 2016; the longest road tunnel is the Laerdal \
             Tunnel in Norway at 24.5 km. The longest undersea one is still the Channel \
             Tunnel at 38 km under the water."
                .into(),
        );
    }
    None
}

/// The place after " in " that the line asks about (a known place of up to three words, else
/// the first word); None when the line has no " in ".
pub fn _the_place_asked<S: AsRef<str>>(said: &str, known: &[S]) -> Option<String> {
    let parts = text::split_on(said, " in ");
    if parts.len() < 2 {
        return None;
    }
    let words = text::split_ref(parts.last().expect("parts"));
    for count in [3usize, 2, 1] {
        let guess = words[..count.min(words.len())].join(" ");
        if known.iter().any(|k| k.as_ref() == guess) {
            return Some(guess);
        }
    }
    words.first().map(|w| w.to_string())
}

/// The longest bridge in Europe or in Britain; None for anything else.
pub fn _the_longest_bridge(said: &str) -> Option<String> {
    if !said.contains(" bridge") && !said.contains(" bridges") {
        return None;
    }
    if !said.contains("longest") && !said.contains("biggest") && !said.contains("largest") {
        return None;
    }
    if let Some(place) = _the_place_asked(said, &PLACES_WITH_A_LONGEST) {
        if !PLACES_WITH_A_LONGEST.contains(&place) {
            return None;
        }
    }
    if said.contains(" europe ") && !said.contains(" world ") {
        return Some(
            "The longest bridge in Europe is the Crimean Bridge at about 19 km, if you count \
             the crossing built in 2018; inside the European Union the longest is the Vasco \
             da Gama Bridge in Lisbon at about 17 km, opened in 1998. The Oresund link between \
             Denmark and Sweden is about 8 km of bridge and 4 km of tunnel."
                .into(),
        );
    }
    if ["uk", "britain", "england", "scotland", "wales", "british"].iter().any(|w| said.contains(&format!(" {w} "))) {
        return Some(
            "The longest bridge in Britain is the Prince of Wales Bridge over the Severn, \
             about 5 km, which carries the M4; the Queen Elizabeth II Bridge at Dartford is \
             about 2.9 km and the Humber Bridge 2.2 km, though the Humber has the longest \
             single span in Britain."
                .into(),
        );
    }
    None
}

/// A kind of bridge or tunnel: the longest ones, a motorway bridge going up, a named
/// structure, or the four ways a bridge carries its load; None for a place, a time or a cost.
pub fn _a_kind_of_bridge(_memory: &Memory, understood: &Understood) -> Option<String> {
    let mut said = _the_line(understood);
    if said.is_empty() {
        said = sh::padded(&sh::clean_or_text(understood).to_lowercase());
    }
    if let Some(named) = _the_longest_bridge(&said).or_else(|| _a_tunnel_named(&said)) {
        return Some(named);
    }
    if said.contains("longest") || said.contains("biggest") || said.contains("largest") {
        if let Some(place) = _the_place_asked(&said, &PLACES_WITH_A_LONGEST) {
            if !PLACES_WITH_A_LONGEST.contains(&place) {
                return None;
            }
        }
    }
    let spaced = |words: &[&str]| words.iter().any(|w| said.contains(&format!(" {w} ")));
    if [" build ", " built ", " builds ", " building ", " construct ", " constructed "].iter().any(|w| said.contains(w))
        && spaced(&["bridge", "bridges", "viaduct", "flyover"])
        && spaced(&["motorway", "motorways", "road", "roads", "railway", "railways", "rail"])
    {
        return Some(
            "A bridge over a motorway is built a piece at a time so the road can stay open: \
             the beams or the whole deck are put together beside it and lifted or slid into \
             place in overnight closures, or the deck is pushed out from one side with a \
             launching girder. The piers go in first, the beams land on them, and the road \
             below is only shut for the hours it takes to place each piece."
                .into(),
        );
    }
    if [" how long does it take ", " how long to ", " cost of ", " how much does "].iter().any(|w| said.contains(w)) {
        return None;
    }
    if [" what does ", " what do ", " meaning ", " in english "].iter().any(|w| said.contains(w))
        && !spaced(&["bridge", "bridges", "tunnel", "tunnels", "viaduct", "aqueduct", "span", "spans"])
    {
        return None;
    }
    for (names, answer) in STRUCTURE_ANSWERS.iter() {
        if sh::any_word_in(&said, names) {
            return Some(answer.clone());
        }
    }
    Some(
        "A bridge carries a load across a gap, and it does it in one of four ways: a beam \
         resting on a support at each end, an arch pushing the weight outwards into solid \
         ground, a truss or box girder making a beam stiffer, or cables hanging the deck from \
         towers - suspension or cable-stayed. Stone and brick work in an arch, steel works in \
         all four."
            .into(),
    )
}

/// The tallest building named, in London, in Britain or in the world, or what a skyscraper
/// is; None for a place this table does not know.
pub fn _the_tallest_building(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = sh::padded(&sh::clean_or_text(understood).to_lowercase());
    for (names, answer) in TALL_BUILDINGS.iter() {
        if sh::any_word_in(&said, names) {
            return Some(answer.clone());
        }
    }
    if let Some(place) = _the_place_asked(&said, &PLACES_WITH_A_TALLEST) {
        if !PLACES_WITH_A_TALLEST.contains(&place) {
            return None;
        }
    }
    let spaced = |words: &[&str]| words.iter().any(|w| said.contains(&format!(" {w} ")));
    if (said.contains(" skyscraper ") || said.contains(" skyscrapers "))
        && !spaced(&["tallest", "highest", "biggest", "world", "london", "uk", "britain"])
    {
        return Some(
            "A skyscraper is a very tall building, and the usual line is 100 m for one and \
             150 m for a supertall. Two things made them possible: a steel frame that carries \
             the weight instead of the walls, and the safety lift, so the building could go up \
             rather than out. The first was the Home Insurance Building in Chicago in 1885, \
             ten storeys; the Burj Khalifa is 163."
                .into(),
        );
    }
    if said.contains(" london ") && !said.contains(" world ") {
        return Some(
            "The tallest building in London is The Shard at 310 m, finished in 2012, with 95 \
             storeys; One Canada Square at Canary Wharf, 235 m, was the tallest before it. \
             The tallest in the United Kingdom is still The Shard."
                .into(),
        );
    }
    if spaced(&["uk", "britain", "england", "scotland", "wales", "british"]) {
        return Some(
            "The tallest building in the United Kingdom is The Shard in London at 310 m, \
             finished in 2012. The tallest in Scotland is the Aughton Street tower in \
             Glasgow, and in Wales it is the Sky Plaza in Cardiff."
                .into(),
        );
    }
    Some(
        "The tallest building in the world is the Burj Khalifa in Dubai at 828 m, 163 floors, \
         finished in 2010; Merdeka 118 in Kuala Lumpur at 679 m is second, and the Shanghai \
         Tower at 632 m third. The Burj was built that tall because the client wanted it to \
         be, and the limits above 500 m are the lifts - a single lift run gets too long - and \
         the wind, which is why the tower steps in as it rises."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Physics pairs.

/// The line (clean, else text) lower-cased with a space at each end.
pub fn _the_line(understood: &Understood) -> String {
    sh::padded(&sh::clean_or_text(understood).to_lowercase())
}

/// The word of the list said first in the line (as a whole word, else anywhere); "" for none.
pub fn _names<S: AsRef<str>>(said: &str, words: &[S]) -> String {
    let mut best: (&str, Option<usize>) = ("", None);
    for word in words {
        let word = word.as_ref();
        let found = match said.find(&format!(" {word} ")) {
            Some(at) => at,
            None => match said.find(word) {
                Some(at) => at,
                None => continue,
            },
        };
        if best.1.is_none_or(|at| found < at) {
            best = (word, Some(found));
        }
    }
    best.0.to_string()
}

/// True when the line asks about mass or weight (or defines a unit of weight).
pub fn asks_about_mass_or_weight(understood: &Understood) -> bool {
    let said = sh::padded(&sh::text_or_clean(understood).to_lowercase());
    // LEANS_ON: Python drops the earlier line's words tacked on the end of a follow-up; none here.
    if !_names(&said, &MASS_OR_WEIGHT_WORDS).is_empty() {
        return true;
    }
    !_names(&said, &MASS_OR_WEIGHT_UNITS).is_empty() && !_names(&said, &_DEFINE).is_empty()
}

/// Mass and weight, weight first when the line asks about weight alone.
pub fn _mass_and_weight(_memory: &Memory, understood: &Understood) -> String {
    let said = _the_line(understood);
    if _names(&said, &["weight", "weigh"]) == "weight" && _names(&said, &["mass"]).is_empty() {
        return "Weight is the pull of gravity on a mass, measured in newtons: a 60 kg person \
                weighs about 600 N on Earth, about 100 N on the Moon and nothing at all in \
                orbit. Mass is how much stuff is in something, measured in kilograms, and it is \
                60 kg in all three places. The scales in your bathroom measure weight and are \
                marked in kilograms for the mass that goes with it here."
            .into();
    }
    "Mass is how much stuff is in something, measured in kilograms, and it does not change \
     wherever you go. Weight is the pull of gravity on that mass, measured in newtons: a \
     60 kg person weighs about 600 N on Earth, about 100 N on the Moon and nothing at all \
     in orbit, while their mass is 60 kg in all three places. So the scales in your \
     bathroom measure weight and are marked in kilograms for the mass that goes with it \
     here."
        .into()
}

/// Kinetic and potential energy, the one named first leading.
pub fn _kinetic_and_potential(_memory: &Memory, understood: &Understood) -> String {
    let said = _the_line(understood);
    if _names(&said, &["potential", "kinetic"]) == "potential" {
        return "Potential energy is energy stored ready to be used: a ball held up high has \
                potential energy, and it turns into kinetic energy as it falls, faster and \
                faster. Kinetic energy is the energy something has because it is moving. Both \
                are measured in joules. Potential is mass times gravity times height, so twice \
                the height is twice the energy; kinetic is half the mass times the speed \
                squared, so doubling the speed gives four times the energy."
            .into();
    }
    "Kinetic energy is the energy something has because it is moving, and potential \
     energy is energy stored ready to move: a ball held up high has potential energy, \
     and it turns into kinetic energy as it falls, faster and faster. Both are measured \
     in joules. Kinetic is half the mass times the speed squared, so doubling the speed \
     gives four times the energy; potential is mass times gravity times height, so twice \
     the height is twice the energy."
        .into()
}

/// Mitosis and meiosis (or the cell cycle), the one named first leading.
pub fn _mitosis_meiosis(_memory: &Memory, understood: &Understood) -> String {
    let said = _the_line(understood);
    let named = _names(&said, &["meiosis", "cell cycle", "mitosis"]);
    if named == "meiosis" {
        return "Meiosis makes four cells with half the chromosomes each, and that is how eggs \
                and sperm are made, so that when two of them join the new cell has a full set \
                again. Mitosis makes two identical cells, each with the same chromosomes as the \
                parent, and that is how you grow and how a cut heals. Meiosis has two divisions \
                in a row, mitosis one; the mnemonic that works is mitosis for maintenance, \
                meiosis for making babies."
            .into();
    }
    if named == "cell cycle" {
        return "The cell cycle is the whole life of a cell from one division to the next: a \
                growth stage where the cell gets bigger and copies its DNA, then the division \
                itself, which is mitosis if it makes two identical cells. Cells that stop \
                dividing leave the cycle and sit in a resting state instead. Meiosis is the \
                different division that makes eggs and sperm, with half the chromosomes each."
            .into();
    }
    "Mitosis makes two identical cells, each with the same chromosomes as the parent, and \
     that is how you grow and how a cut heals. Meiosis makes four cells with half the \
     chromosomes, and that is how eggs and sperm are made, so that when two of them join \
     the new cell has a full set again. The mnemonic that works: mitosis for maintenance, \
     meiosis for making babies. Mitosis has one division, meiosis has two in a row."
        .into()
}

/// Frequency, wavelength and amplitude, the one named first leading.
pub fn _frequency_wavelength_amplitude(_memory: &Memory, understood: &Understood) -> String {
    let said = _the_line(understood);
    let named = _names(&said, &["wavelength", "amplitude", "frequency"]);
    if named == "wavelength" {
        return "Wavelength is the length of one whole wave, from one peak to the next, measured \
                in metres. Frequency is how many of those waves pass a point each second, in \
                hertz, and the two are locked together by the speed of the wave: wavelength is \
                the speed divided by the frequency, so a 340 m/s sound at 50 Hz has a wavelength \
                of 6.8 m. That is why a low note travels further round a corner than a high one."
            .into();
    }
    if named == "amplitude" {
        return "Amplitude is how big the wave is - how far it swings from the middle - and it is \
                what you hear as volume or see as brightness: a bigger amplitude is a louder \
                sound or a brighter light. Frequency is the different thing, how many waves pass \
                each second, measured in hertz, and that is the pitch. So a loud low note is big \
                amplitude and small frequency, and turning the volume up changes the amplitude \
                only."
            .into();
    }
    "Frequency is how many times something happens in a second, measured in hertz: 50 Hz \
     is fifty times a second. A sound's frequency is its pitch - 440 Hz is the A above \
     middle C, and the higher the number the higher the note - while a radio station's \
     100 MHz is a hundred million waves a second. It is the reciprocal of the period, so \
     a wave that repeats every 0.02 seconds has a frequency of 50 Hz, and the wavelength \
     is the speed divided by the frequency."
        .into()
}

/// Newton's three laws, the one the line asks about first (the second by default).
pub fn _newtons_laws(_memory: &Memory, understood: &Understood) -> String {
    let said = understood.text.as_str();
    let mut order: [&String; 3] = [&_NEWTON_SECOND, &_NEWTON_FIRST, &_NEWTON_THIRD];
    if let Some(m) = _NEWTON_EACH.search(&sh::padded(&said.to_lowercase())) {
        let word = [m.str(1), m.str(2), m.str(3)].into_iter().find(|w| !w.is_empty()).unwrap_or("");
        match _NEWTON_WORD.get(word) {
            Some(0) => order = [&_NEWTON_FIRST, &_NEWTON_SECOND, &_NEWTON_THIRD],
            Some(2) => order = [&_NEWTON_THIRD, &_NEWTON_SECOND, &_NEWTON_FIRST],
            _ => {}
        }
    }
    order.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
}

/// Speed and velocity, velocity first when the line names it and not speed.
pub fn _speed_and_velocity(_memory: &Memory, understood: &Understood) -> String {
    let said = _the_line(understood);
    if _names(&said, &["velocity"]) == "velocity" && _names(&said, &["speed"]) != "speed" {
        return "Velocity is how fast you are going and which way: 50 mph north is a velocity, \
                and 50 mph on its own is a speed. So a car going round a roundabout at a steady \
                30 mph has a constant speed and a changing velocity, because the direction keeps \
                changing. Speed is distance over time, and average speed is the whole trip's \
                distance over the whole time, which is why a two-hour journey with a stop does \
                not average the speed you had on the motorway."
            .into();
    }
    "Speed is how fast you are going, distance over time: 100 miles in two hours is \
     50 mph. Velocity is that speed with a direction, so a car going round a roundabout \
     at a steady 30 mph has a constant speed and a changing velocity. Average speed is \
     the whole trip's distance over the whole time, which is why a two-hour journey with \
     a stop does not average the speed you had on the motorway."
        .into()
}

/// True when the words are about rock (erosion, homework), or say nothing of the weather.
pub fn _the_rock_and_not_the_sky(understood: &Understood) -> bool {
    if understood.words.iter().any(|w| THE_ROCK.contains(w)) {
        return true;
    }
    !understood.words.iter().any(|w| THE_SKY.contains(w))
}

/// True when the words name the Gulf Stream or the Atlantic.
pub fn _the_ocean_current(understood: &Understood) -> bool {
    let has = |w: &str| understood.words.iter().any(|x| x == w);
    has("stream") && has("gulf") || has("atlantic")
}

/// Sleep supplements: a child's (pharmacist first), herbal, gummies and melatonin, or
/// magnesium first.
pub fn _sleep_supplements_reply(_memory: &Memory, understood: &Understood) -> String {
    let clean = sh::padded(&sh::clean_or_text(understood).to_lowercase());
    if sh::any_in(&clean, &_SLEEP_CHILD_WORDS) {
        return [_SLEEP_CHILD.as_str(), _SLEEP_MAGNESIUM.as_str(), _SLEEP_HABIT.as_str()].join(" ");
    }
    let order: [&str; 3] = if clean.contains("valerian") || clean.contains("herbal") || clean.contains("tea") {
        [&_SLEEP_VALERIAN, &_SLEEP_MAGNESIUM, &_SLEEP_GUMMIES]
    } else if clean.contains("gumm") || clean.contains("melatonin") || clean.contains("antihistamine") {
        [&_SLEEP_GUMMIES, &_SLEEP_MAGNESIUM, &_SLEEP_VALERIAN]
    } else {
        [&_SLEEP_MAGNESIUM, &_SLEEP_VALERIAN, &_SLEEP_GUMMIES]
    };
    let mut parts: Vec<&str> = order.to_vec();
    parts.push(&_SLEEP_HABIT);
    parts.join(" ")
}

// ---------------------------------------------------------------------------------------------
// Registry tables.

fn g_lambda_6619(_f: &FnRef, u: &Understood) -> bool { lambda_6619(u) }
fn g_lambda_6694(_f: &FnRef, u: &Understood) -> bool { lambda_6694(u) }
fn g_lambda_7693(_f: &FnRef, u: &Understood) -> bool { lambda_7693(u) }
fn g_a_fall_a_person_had(_f: &FnRef, u: &Understood) -> bool { _a_fall_a_person_had(u) }
fn g_a_place_for_the_kit(_f: &FnRef, u: &Understood) -> bool { _a_place_for_the_kit(u) }
fn g_an_amount_of_losing(_f: &FnRef, u: &Understood) -> bool { _an_amount_of_losing(u) }
fn g_names_the_feeling(_f: &FnRef, u: &Understood) -> bool { _names_the_feeling(u) }
fn g_not_a_food_cold(_f: &FnRef, u: &Understood) -> bool { _not_a_food_cold(u) }
fn g_not_a_volcano_build(_f: &FnRef, u: &Understood) -> bool { _not_a_volcano_build(u) }
fn g_not_about_itching(_f: &FnRef, u: &Understood) -> bool { _not_about_itching(u) }
fn g_the_backs_are_a_body(_f: &FnRef, u: &Understood) -> bool { _the_backs_are_a_body(u) }
fn g_the_ocean_current(_f: &FnRef, u: &Understood) -> bool { _the_ocean_current(u) }
fn g_the_pounds_are_weight(_f: &FnRef, u: &Understood) -> bool { _the_pounds_are_weight(u) }
fn g_the_rock_and_not_the_sky(_f: &FnRef, u: &Understood) -> bool { _the_rock_and_not_the_sky(u) }
fn g_water_is_the_subject(_f: &FnRef, u: &Understood) -> bool { _water_is_the_subject(u) }
fn g_asks_about_mass_or_weight(_f: &FnRef, u: &Understood) -> bool { asks_about_mass_or_weight(u) }

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.science:<lambda>:6619", g_lambda_6619),
    ("bot.knowledge.science:<lambda>:6694", g_lambda_6694),
    ("bot.knowledge.science:<lambda>:7693", g_lambda_7693),
    ("bot.knowledge.science:_a_fall_a_person_had:391", g_a_fall_a_person_had),
    ("bot.knowledge.science:_a_place_for_the_kit:2602", g_a_place_for_the_kit),
    ("bot.knowledge.science:_an_amount_of_losing:2221", g_an_amount_of_losing),
    ("bot.knowledge.science:_names_the_feeling:244", g_names_the_feeling),
    ("bot.knowledge.science:_not_a_food_cold:2280", g_not_a_food_cold),
    ("bot.knowledge.science:_not_a_volcano_build:2573", g_not_a_volcano_build),
    ("bot.knowledge.science:_not_about_itching:1952", g_not_about_itching),
    ("bot.knowledge.science:_the_backs_are_a_body:2709", g_the_backs_are_a_body),
    ("bot.knowledge.science:_the_ocean_current:12495", g_the_ocean_current),
    ("bot.knowledge.science:_the_pounds_are_weight:2184", g_the_pounds_are_weight),
    ("bot.knowledge.science:_the_rock_and_not_the_sky:12464", g_the_rock_and_not_the_sky),
    ("bot.knowledge.science:_water_is_the_subject:2263", g_water_is_the_subject),
    ("bot.knowledge.science:asks_about_mass_or_weight:11559", g_asks_about_mass_or_weight),
];

fn r_lambda_5221(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(lambda_5221(m, u)) }
fn r_a_baby_sleep_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _a_baby_sleep_reply(m, u) }
fn r_a_dizzy_spell(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_a_dizzy_spell(m, u)) }
fn r_a_kind_of_bridge(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _a_kind_of_bridge(m, u) }
fn r_cold_help(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_cold_help(m, u)) }
fn r_dinosaurs(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_dinosaurs(m, u)) }
fn r_eyes_test(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_eyes_test(m, u)) }
fn r_forgetting(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_forgetting(m, u)) }
fn r_freezing_point(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_freezing_point(m, u)) }
fn r_frequency_wavelength_amplitude(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_frequency_wavelength_amplitude(m, u))
}
fn r_grinding(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_grinding(m, u)) }
fn r_hangover(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_hangover(m, u)) }
fn r_hayfever_or_a_cold(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_hayfever_or_a_cold(m, u)) }
fn r_headache_days_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _headache_days_reply(m, u) }
fn r_headache_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_headache_reply(m, u)) }
fn r_heartburn(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_heartburn(m, u)) }
fn r_hiccups_remedy(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_hiccups_remedy(m, u)) }
fn r_keep_taking(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_keep_taking(m, u)) }
fn r_kinetic_and_potential(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_kinetic_and_potential(m, u)) }
fn r_mass_and_weight(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_mass_and_weight(m, u)) }
fn r_missed_medication(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_missed_medication(m, u)) }
fn r_missed_pill(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_missed_pill(m, u)) }
fn r_mitosis_meiosis(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_mitosis_meiosis(m, u)) }
fn r_moons(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_moons(m, u)) }
fn r_newtons_laws(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_newtons_laws(m, u)) }
fn r_night_waking(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _night_waking(m, u) }
fn r_period_answer(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _period_answer(m, u) }
fn r_sleep_clock(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_sleep_clock(m, u)) }
fn r_sleep_hours(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_sleep_hours(m, u)) }
fn r_sleep_supplements_reply(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_sleep_supplements_reply(m, u))
}
fn r_speed_and_velocity(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_speed_and_velocity(m, u)) }
fn r_steps_calories(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_steps_calories(m, u)) }
fn r_steps_in_a_distance(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_steps_in_a_distance(m, u)) }
fn r_steps_in_a_time(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_steps_in_a_time(m, u)) }
fn r_steps_up_a_building(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_steps_up_a_building(m, u)) }
fn r_tall_thing(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_tall_thing(m, u)) }
fn r_the_bruise_answer(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { Some(_the_bruise_answer(m, u)) }
fn r_the_limit_for_the_drink_named(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _the_limit_for_the_drink_named(m, u)
}
fn r_the_planet_size(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _the_planet_size(m, u) }
fn r_the_tallest_building(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _the_tallest_building(m, u) }
fn r_what_the_body_is_like(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { _what_the_body_is_like(m, u) }
fn r_who_discovered(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> { who_discovered(m, u) }

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.science:<lambda>:5221", r_lambda_5221),
    ("bot.knowledge.science:_a_baby_sleep_reply:2364", r_a_baby_sleep_reply),
    ("bot.knowledge.science:_a_dizzy_spell:2162", r_a_dizzy_spell),
    ("bot.knowledge.science:_a_kind_of_bridge:11357", r_a_kind_of_bridge),
    ("bot.knowledge.science:_cold_help:1146", r_cold_help),
    ("bot.knowledge.science:_dinosaurs:195", r_dinosaurs),
    ("bot.knowledge.science:_eyes_test:704", r_eyes_test),
    ("bot.knowledge.science:_forgetting:1851", r_forgetting),
    ("bot.knowledge.science:_freezing_point:1697", r_freezing_point),
    ("bot.knowledge.science:_frequency_wavelength_amplitude:11717", r_frequency_wavelength_amplitude),
    ("bot.knowledge.science:_grinding:471", r_grinding),
    ("bot.knowledge.science:_hangover:462", r_hangover),
    ("bot.knowledge.science:_hayfever_or_a_cold:2549", r_hayfever_or_a_cold),
    ("bot.knowledge.science:_headache_days_reply:1242", r_headache_days_reply),
    ("bot.knowledge.science:_headache_reply:1259", r_headache_reply),
    ("bot.knowledge.science:_heartburn:1654", r_heartburn),
    ("bot.knowledge.science:_hiccups_remedy:1355", r_hiccups_remedy),
    ("bot.knowledge.science:_keep_taking:844", r_keep_taking),
    ("bot.knowledge.science:_kinetic_and_potential:11621", r_kinetic_and_potential),
    ("bot.knowledge.science:_mass_and_weight:11585", r_mass_and_weight),
    ("bot.knowledge.science:_missed_medication:874", r_missed_medication),
    ("bot.knowledge.science:_missed_pill:776", r_missed_pill),
    ("bot.knowledge.science:_mitosis_meiosis:11684", r_mitosis_meiosis),
    ("bot.knowledge.science:_moons:566", r_moons),
    ("bot.knowledge.science:_newtons_laws:11807", r_newtons_laws),
    ("bot.knowledge.science:_night_waking:521", r_night_waking),
    ("bot.knowledge.science:_period_answer:1297", r_period_answer),
    ("bot.knowledge.science:_sleep_clock:9407", r_sleep_clock),
    ("bot.knowledge.science:_sleep_hours:659", r_sleep_hours),
    ("bot.knowledge.science:_sleep_supplements_reply:12633", r_sleep_supplements_reply),
    ("bot.knowledge.science:_speed_and_velocity:11875", r_speed_and_velocity),
    ("bot.knowledge.science:_steps_calories:1773", r_steps_calories),
    ("bot.knowledge.science:_steps_in_a_distance:1420", r_steps_in_a_distance),
    ("bot.knowledge.science:_steps_in_a_time:1813", r_steps_in_a_time),
    ("bot.knowledge.science:_steps_up_a_building:1378", r_steps_up_a_building),
    ("bot.knowledge.science:_tall_thing:11169", r_tall_thing),
    ("bot.knowledge.science:_the_bruise_answer:2699", r_the_bruise_answer),
    ("bot.knowledge.science:_the_limit_for_the_drink_named:1082", r_the_limit_for_the_drink_named),
    ("bot.knowledge.science:_the_planet_size:11045", r_the_planet_size),
    ("bot.knowledge.science:_the_tallest_building:11415", r_the_tallest_building),
    ("bot.knowledge.science:_what_the_body_is_like:2507", r_what_the_body_is_like),
    ("bot.knowledge.science:who_discovered:624", r_who_discovered),
];

fn l_cold_or_flu_verdict(_f: &FnRef, u: &Understood) -> String { _cold_or_flu_verdict(u) }

/// The entries' `lead` callables (the sentence the row owes the line).
pub const LEADS: &[(&str, LeadFn)] = &[("bot.knowledge.science:_cold_or_flu_verdict:429", l_cold_or_flu_verdict)];
