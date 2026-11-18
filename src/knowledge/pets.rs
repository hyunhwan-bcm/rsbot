//! Port of PyBot's `bot/knowledge/pets.py`: dogs, cats, fish and small animals in everyday
//! plain advice - the guards that keep each card to the animal and the problem it is written
//! for, and the replies that say it about the animal the line actually names.
//!
//! Import-time-only Python (no runtime port): `_bird_lifespan` (factory; its closure `say` is
//! ported), `_lifespan_entries` (builds entries already in the bundle), the dead `entry(...)`
//! expression after `_a_lost_pet`'s return, and the module-level `for _item in ENTRIES` loop.

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

/// A module global of pets.py by its Python name.
fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.pets.{name}"))
}
fn g_re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("pets.{name} is not a regex"))
}
fn g_str(name: &str) -> String {
    g(name).str().to_string()
}
fn g_strs(name: &str) -> Vec<String> {
    g(name).strs()
}
fn g_set(name: &str) -> HashSet<String> {
    g(name).str_set()
}
/// A `{str: str}` dict global, in insertion order.
fn g_pairs(name: &str) -> Vec<(String, String)> {
    g(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}
/// `table[key]` for a `{str: str}` dict (None where Python raises KeyError).
fn lookup(table: &[(String, String)], key: &str) -> Option<String> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

macro_rules! re_global {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<&'static Re> = LazyLock::new(|| g_re(stringify!($name)));)*
    };
}
macro_rules! str_global {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<String> = LazyLock::new(|| g_str(stringify!($name)));)*
    };
}
macro_rules! strs_global {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<Vec<String>> = LazyLock::new(|| g_strs(stringify!($name)));)*
    };
}
macro_rules! set_global {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g_set(stringify!($name)));)*
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

/// `set(re.findall(r"[a-z']+", s))`.
fn az_words(s: &str) -> HashSet<&str> {
    s.split(|c: char| !(c.is_ascii_lowercase() || c == '\'')).filter(|w| !w.is_empty()).collect()
}

/// `bool(a & b)` for a word set against a string set.
fn meets(words: &HashSet<&str>, set: &HashSet<String>) -> bool {
    words.iter().any(|w| set.contains(*w))
}

/// `str(understood.get("clean") or understood.get("text") or "").lower()`.
fn clean_or_text_lower(u: &Understood) -> String {
    sh::clean_or_text(u).to_lowercase()
}

/// `[(turn.get("user") or "").lower() for turn in (memory.get("turns") or [])[-n:]]`.
fn last_users(memory: &Memory, n: usize) -> Vec<String> {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(n)..].iter().map(|t| t.user.to_lowercase()).collect()
}

/// `str.format(**fills)` for a template whose only fields are the names given: None where
/// Python would raise (a field it was not given).
fn py_format(template: &str, fills: &[(&str, &str)]) -> Option<String> {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(i) = rest.find(['{', '}']) {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        if tail.starts_with("{{") {
            out.push('{');
            rest = &tail[2..];
        } else if tail.starts_with("}}") {
            out.push('}');
            rest = &tail[2..];
        } else if tail.starts_with('{') {
            let end = tail.find('}')?;
            let name = &tail[1..end];
            let (_, value) = fills.iter().find(|(k, _)| *k == name)?;
            out.push_str(value);
            rest = &tail[end + 1..];
        } else {
            return None; // a single '}' is a ValueError
        }
    }
    out.push_str(rest);
    Some(out)
}

/// Python's `"%g" % x`.
fn fmt_g(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    let exp = x.abs().log10().floor() as i32;
    if !(-4..6).contains(&exp) {
        let s = format!("{:.5e}", x);
        let (m, e) = s.split_once('e').unwrap();
        let m = if m.contains('.') { m.trim_end_matches('0').trim_end_matches('.') } else { m };
        let e: i32 = e.parse().unwrap();
        return format!("{}e{}{:02}", m, if e < 0 { '-' } else { '+' }, e.abs());
    }
    let prec = (5 - exp).max(0) as usize;
    let s = format!("{:.*}", prec, x);
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
}

/// A word list compiled into one `(?<![a-z0-9])word(?![a-z0-9])` test per word.
fn whole_word_res(words: &[String]) -> Vec<Re> {
    words.iter().map(|w| Re::new(&format!("(?<![a-z0-9]){}(?![a-z0-9])", sh::escape(w)), 0)).collect()
}

/// A word list compiled into one `\bword\b` test per word.
fn bounded_res(words: &[String]) -> Vec<Re> {
    words.iter().map(|w| Re::new(&format!(r"\b{}\b", sh::escape(w)), 0)).collect()
}

/// A word list compiled into one `\bword` test per word.
fn front_bounded_res(words: &[String]) -> Vec<Re> {
    words.iter().map(|w| Re::new(&format!(r"\b{}", sh::escape(w)), 0)).collect()
}

// ---------------------------------------------------------------------------------------------
// Module data.

strs_global!(
    DOG_BREEDS, CAT_BREEDS, ANIMAL_NOUNS, _CAT_WARNING, _PREGNANT_SELF, NEUTER_NOT_WRITTEN_FOR,
    GUMS_COLOUR, GUMS_FINE, GUMS_ASKING, GUMS_CHANGED, GUMS_BAD, GUMS_RED, ILL, SWITCH_WORDS,
    SLIP_WORDS, TIGHT_WORDS, PULL_WORDS, _LEAD_PHRASES, OTHER_PETS, _OTHER_PETS_DINNER,
    _DINNER_TAKEN, _DINNER_STOPPED, _DINNER_ASKED_OR_REFUSED, _A_PERSON_DID_IT, _THE_ANIMAL_IS_ILL,
    _CHILD_WORRY, _GOING_THIN, _THIN_WORDS, _WEIGHT_UP, _ASKING, _WEIGH_WORDS, _WEIGH_DIRECTION,
    BITE_NOW, BITE_TEETH, BITE_SKIN, BITE_SOMEONE_ELSE, BITE_DONE_DEED, BITE_CHILD_WORDS,
    BITE_DOOR_WORDS, BITE_OTHERS_HABIT, LEFT_OVERS_EATEN, LEFT_OVERS_EATEN_PHRASES, _A_RABBIT,
    _PET_SUBJECT, _A_PERSON, _A_PERSON_NAMED, _NOT_A_CAT_OR_DOG, _A_SPEAKER_IS_ILL,
    _SHOPPING_WORDS, _DOG_COOL, _A_LOST_ANIMAL, _OFF_FOOD, _OFF_FOOD_RABBIT, _OFF_FOOD_RODENT,
    _OFF_FOOD_BIRD, _OFF_FOOD_REPTILE, _OFF_FOOD_FISH, _THIN_COAT, _HORSE, _FLAT_FACE,
    _PET_ANIMAL, _NOT_A_PERSON_BREATHING, _BEEN_ACTIVE, _PET_BREATHING_REPLY, _HORSE_BEEN_WORKING,
    _SWEATING_REPLY,
);
set_global!(
    _THE_CLAW_ANIMALS, _DOG_STIFF, _FISHING_SPORT, _FISH_CARE, _A_MEDICINE_WORD, _A_WARNING_WORD,
    _A_HELD_WORD, _HANDLING_WARNING, _HANDLING_WORD, _A_TEETH_SHOW, _A_LITTLE_ONE, _HANDS_OR_PLAY,
    PET, _LEAD_SUBJECTS, _CALLED_NOT_A_NAME, _CALLED_NOT_A_NAME_AT_HOME, _DOG_SLEEP_WORDS,
    _CAT_SLEEP_WORDS, _HURT_PERSON, _HURT_VERBS, _HURT_STRANGER, _DOG_WORDS,
);
str_global!(
    NEUTER_AGAINST, NEUTER_AGAINST_RABBIT, LITTER_REPLY, NO_INFECTION_TEXT, _PET_DAIRY_MILK,
    PET_WOUND_REPLY, CAT_PAW_LICK_REPLY, DOG_PAW_LICK_REPLY, LOSS_REPLY, PREY_DEATH_REPLY,
    _DOG_RECALL_REPLY, CAT_AND_CHILDREN, SMALL_PET_RABBIT_LINE, SMALL_PET_HAMSTER_LINE,
    SMALL_PET_GUINEA_LINE, SMALL_PET_BUDGIE_LINE, SMALL_PET_HORSE_LINE, SMALL_PET_FISH_LINE,
    PET_COST_COMPARE_LINE, PET_COST_KITTEN_LINE, PET_COST_RESCUE_LINE, PET_COST_BREEDER_LINE,
    PET_COST_PUPPY_LINE, _PONY_FAT, _RABBIT_FAT, _RODENT_FAT, _CAT_THIN_REPLY, _DOG_THIN_REPLY,
    BITE_REPORT_CHILD, BITE_REPORT, PLATE_LEFTOVERS_TEXT, ATE_LEFTOVERS_TEXT, _A_VET_IS_FOR_ANIMALS,
    _WHEN_A_VET_IS_NEEDED, _RABBIT_SICK_REPLY, _CAT_SICK_REPLY, _DOG_SICK_REPLY, _DOG_KIT_LIST,
    _DOG_KIT_NEWS, _PUPPY_KIT_LIST, _PUPPY_KIT_NEWS, _DOG_CAR_FIRST_AID, _DOG_CAR_RULE,
    _WHERE_THE_DOG_SLEEPS, _WHERE_THE_CAT_SLEEPS, _WHERE_EITHER_SLEEPS, ANIMAL, _CAT_NOTWASH_REPLY,
    _CAT_SHED_REPLY, _CAT_BALD_REPLY, _KNEAD_REPLY, _HEADBUTT_REPLY, _ON_YOU_REPLY, _HURT_WOUND,
    _HURT_ANOTHER_ANIMAL, _HURT_OWN_HOME,
);
re_global!(
    _AGE_PATTERN, _A_CREATURE, _A_FUNGUS, _A_FUNGUS_HAS_GONE_DOWN, NEUTER_SAYS_NOT, _WEIGHING_UP,
    _FOR_THE_CAT, _TWO_OF_THEM, _BUYING_FOR, _CLEANING_ASK, FEED_ASK, SICK_NOW, DOG_WORD, NOT_DOG,
    _WHERE_IT_LANDED, _BLOCKED, _TRYING, _LITTLE, _SPRAY, _BLOOD, _A_CLEAR_RESULT, _A_CAT_NAMED,
    _IS_PANTING, _BREATHING_ITSELF, _NEUTER_ALREADY_DONE, _NEUTER_NOT_DONE, _NEUTER_STILL_ASKED,
    _NEUTER_THE_DAY_OR_AFTER, _NEUTER_HOW_TO_TELL, _NEUTER_BEHAVIOUR, _NEUTER_NOT_YET,
    _NEUTER_ALSO_ABOUT, PET_NAMED, PERSON_NAMED, _ANIMAL_IN_A_TURN, _FOOD_OR_DRINK,
    _PERSON_IN_A_TURN, SMALL_PETS, _SMALL_PET_NAMED, _A_RABBIT_SAID, _A_GUINEA_SAID,
    _A_RODENT_SAID, _A_FERRET_SAID, _A_RODENT_OR_GUINEA, _A_HOW_TO_HOLD, _A_WONT_BE_HELD,
    _A_HURT_OR_FRIGHT, _LIFE_SAID, _NAME_ASK, _CALLED_NAME, _A_SEASON_WORD, _A_CAT_IN_THE_LINE,
    _A_NEWBORN_WORD, _A_CARE_PHRASE, _LOSS_SAID, _PUT_TO_SLEEP, _DEAD_NOT_THE_PET, _PET_ATE_OR_ILL,
    _THE_PET_IS_DYING, _PREY_NAMED, _BRINGING_IT_IN, _A_BARE_DEATH, _THE_ANIMAL_IS_NAMED,
    _A_REPEATED_DEATH, _AN_ANIMAL_OF_THEIRS, PET_COST_RESCUE, PET_COST_BREEDER, PET_COST_KITTEN,
    PET_COST_DOG, PET_COST_ONE_OR_OTHER, SMALL_PET_RABBIT, SMALL_PET_HAMSTER, SMALL_PET_GUINEA,
    SMALL_PET_BUDGIE, SMALL_PET_HORSE, SMALL_PET_FISH, _COAT_MAT, _COAT_GREASY, _A_BIN,
    _A_DOG_OR_CAT, SMALL_FUR, SMALL_FUR_STATE, BIRD_FUR, BIRD_PLUCKING, _STOPPING_A_PET_GETTING_OUT,
    _SMALL_ANIMAL, _NOT_FOOD, _CARE_OF_A_NEW_PET, _A_SHOPPING_ASK, _EAT_VERB, _CAT_SHED,
    _CAT_NOTWASH, _KNEAD_SAID, _HEAD_SAID, _HURT_PASSIVE, _SMELL_OF_AN_ANIMAL,
    _THE_COAT_THE_MACHINE_IS_FULL_OF, _A_SOAKING, _AN_ANIMAL_IN_THE_WATER, _PET_SHOCKED,
    _PET_CHEWED, _PET_LIVE,
);

static _DOG_SENIOR_YEARS: LazyLock<f64> = LazyLock::new(|| g("_DOG_SENIOR_YEARS").as_f64().unwrap_or(7.0));

// ---------------------------------------------------------------------------------------------
// The claw, the age, the fishing rod, and the warnings a dog gives.

/// True only when the line names the animal whose claw the wound came from.
pub fn _an_animal_did_it(understood: &Understood) -> bool {
    let words = format!(" {} ", understood.words.join(" "));
    _THE_CLAW_ANIMALS.iter().any(|word| words.contains(&format!(" {word} ")))
}

/// The age a line states for an animal, in years, or None when it states none.
pub fn _dog_age_years(understood: &Understood) -> Option<f64> {
    let text_ = format!(" {} ", clean_or_text_lower(understood));
    let found = _AGE_PATTERN.search(&text_)?;
    let (raw, unit) = (found.str(1), found.str(2));
    let number = match g("_AGE_WORDS").get(raw) {
        Some(v) => v.as_f64()?,
        None => raw.parse::<f64>().ok()?,
    };
    if unit.starts_with("month") || unit.starts_with("mth") {
        return Some(number / 12.0);
    }
    if unit.starts_with("week") || unit.starts_with("wk") {
        return Some(number / 52.0);
    }
    Some(number)
}

/// False when the line is the sport of fishing and not a fish being kept.
pub fn _fish_not_the_sport(understood: &Understood) -> bool {
    let clean = format!(" {} ", clean_or_text_lower(understood));
    let words = az_words(&clean);
    if meets(&words, &_FISH_CARE) {
        return true;
    }
    !meets(&words, &_FISHING_SPORT)
}

/// True unless the line says the dog is under seven, which is not an older dog.
pub fn _dog_not_young(understood: &Understood) -> bool {
    match _dog_age_years(understood) {
        None => return true,
        Some(years) if years >= *_DOG_SENIOR_YEARS => return true,
        _ => {}
    }
    let clean = format!(" {} ", clean_or_text_lower(understood));
    meets(&az_words(&clean), &_DOG_STIFF)
}

/// True when the line itself names a medicine, which is what makes the pet's line the pet's.
pub fn _a_medicine_line(understood: &Understood) -> bool {
    let clean = format!(" {} ", clean_or_text_lower(understood));
    meets(&az_words(&clean), &_A_MEDICINE_WORD)
}

/// True when the line says the warning and the handling, which is what this row answers.
pub fn _a_handling_growl_line(understood: &Understood) -> bool {
    let clean = format!(" {} ", clean_or_text_lower(understood));
    let words = az_words(&clean);
    meets(&words, &_HANDLING_WARNING) && meets(&words, &_HANDLING_WORD)
}

/// True when the line is about something held and a warning over it, not a growl on its own.
pub fn _a_guarding_line(understood: &Understood) -> bool {
    let clean = format!(" {} ", clean_or_text_lower(understood));
    let words = az_words(&clean);
    meets(&words, &_A_WARNING_WORD) && meets(&words, &_A_HELD_WORD)
}

/// True when the line itself has a growl, a snarl, a snap or bared teeth in it.
pub fn _a_warning_sound_said(understood: &Understood) -> bool {
    let clean = format!(" {} ", clean_or_text_lower(understood));
    let words = az_words(&clean);
    if meets(&words, &_A_LITTLE_ONE) && meets(&words, &_HANDS_OR_PLAY) {
        return false;
    }
    const GUARDING: [&str; 6] = ["guard", "guards", "guarding", "guarded", "possessive", "protective"];
    if words.iter().any(|w| _A_WARNING_WORD.contains(*w) && !GUARDING.contains(w)) {
        return true;
    }
    words.contains("teeth") && meets(&words, &_A_TEETH_SHOW)
}

static CAT_WARNING_RES: LazyLock<Vec<Re>> = LazyLock::new(|| whole_word_res(&_CAT_WARNING));

/// A cat that bites: the play answer, or the give-her-space answer once a warning is named.
pub fn _cat_biting(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = format!(" {} ", understood.clean.to_lowercase());
    let warning = CAT_WARNING_RES.iter().any(|r| r.is_match(&clean));
    if !warning {
        return Some(
            "Hands are not the toy: play with a rod toy on a string and put it away after ten \
minutes, twice a day, because an under-exercised cat bites. Watch for the warning \
- tail flicking, ears back, skin twitching along the back - and stop stroking \
before it, since most bites are a cat asking for a break. A bite that comes out \
of nowhere in a calm cat is worth a vet visit, because pain or a bad tooth is \
often behind it."
                .into(),
        );
    }
    Some(
        "Take the warning as a no and give her the space: put her down or step back, stop what \
you were doing, and let her come to you rather than picking her up to show her it is \
fine. Never tell a cat off for hissing or growling - a cat punished for the warning \
skips it and goes straight to the bite - and give her somewhere high to get away to, \
because most of this is a cat with no escape route. Watch for the tail flick, the \
ears going back and the skin twitching along the back, and stop stroking before \
those. A cat who suddenly hisses or growls at someone she knows, or when she is \
picked up, needs a vet: pain, a bad tooth or arthritis is the usual reason, and this \
is often the first sign of it."
            .into(),
    )
}

/// True only when the line names the creature, so "my dog ate a mushroom" is not this card.
pub fn _an_eaten_creature(understood: &Understood) -> bool {
    let line = clean_or_text_lower(understood);
    !line.is_empty() && _A_CREATURE.is_match(&line)
}

/// True only when the line names the fungus AND something has already gone down.
pub fn _an_eaten_fungus(understood: &Understood) -> bool {
    let line = clean_or_text_lower(understood);
    !line.is_empty() && _A_FUNGUS.is_match(&line) && _A_FUNGUS_HAS_GONE_DOWN.is_match(&line)
}

static PREGNANT_SELF_RES: LazyLock<Vec<Re>> = LazyLock::new(|| whole_word_res(&_PREGNANT_SELF));

/// A cat and a baby: the litter-tray answer while the person is expecting, the house rules
/// otherwise.
pub fn _cat_and_baby(memory: &Memory, understood: &Understood) -> Option<String> {
    let mut said = format!(" {} ", understood.clean.to_lowercase());
    said += &last_users(memory, 2).join(" ");
    said += " ";
    let expecting = PREGNANT_SELF_RES.iter().any(|r| r.is_match(&said));
    if expecting {
        return Some(
            "Yes - keep the cat, and the one job to sort out before the baby comes is the \
litter tray: someone else should empty it every day while you are pregnant, or \
if you have to do it yourself, wear disposable gloves and wash your hands after, \
because toxoplasmosis - a parasite a cat picks up from raw meat and soil - can \
harm an unborn baby. A cat that stays indoors and eats tinned or cooked food \
carries almost no risk, and the other route is garden soil, so wear gloves for \
the weeding too. Wash fruit and veg and cook meat through, and that is the lot. \
When the baby arrives: the cot, the pram and the bouncy chair are out of bounds \
from day one, keep the door of the room the baby sleeps in shut, never hold the \
baby and the cat at once, give her somewhere high to get away to, keep her \
routine exactly as it is now, and have someone bring home a blanket the baby \
has slept on so the smell is familiar before they meet."
                .into(),
        );
    }
    Some(
        "Yes - a cat and a baby are fine together, and the house rules do the work: the cot, \
the pram and the bouncy chair are out of bounds from day one, so keep the cat out of \
the room the baby sleeps in and shut the door rather than shooing her away from the \
cot later. Never leave them alone in a room, and never hold the baby and the cat at \
the same time, because a startled cat scratches whatever is nearest. Cut the cat's \
claws out of the equation by giving her somewhere high to get away to, keep her \
routine the same as before the birth, and have someone bring home a blanket the baby \
has slept on so the smell is familiar before they meet. If you are pregnant, let \
someone else empty the litter tray and wear gloves for the garden, because of \
toxoplasmosis - a cat that stays indoors and is fed cooked or tinned food carries \
almost no risk."
            .into(),
    )
}

/// `NEUTER_WHY`: (the animal's names as `\bname\b` tests, the first name, the case for it).
static NEUTER_WHY: LazyLock<Vec<(Vec<Re>, String, String)>> = LazyLock::new(|| {
    g("NEUTER_WHY")
        .items()
        .iter()
        .map(|pair| {
            let names = pair.items()[0].strs();
            let first = names.first().cloned().unwrap_or_default();
            (bounded_res(&names), first, pair.items()[1].str().to_string())
        })
        .collect()
});
static NEUTER_NOT_WRITTEN_FOR_RES: LazyLock<Vec<Re>> = LazyLock::new(|| bounded_res(&NEUTER_NOT_WRITTEN_FOR));

/// The health case for neutering the animal named (or the case against, when the line says no).
pub fn _neuter_why(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = clean_or_text_lower(understood);
    let against = NEUTER_SAYS_NOT.is_match(&said);
    for (names, first, reply) in NEUTER_WHY.iter() {
        if names.iter().any(|r| r.is_match(&said)) {
            if against {
                if first == "rabbit" {
                    return Some(NEUTER_AGAINST_RABBIT.clone());
                }
                return Some(NEUTER_AGAINST.clone());
            }
            return Some(reply.clone());
        }
    }
    if NEUTER_NOT_WRITTEN_FOR_RES.iter().any(|r| r.is_match(&said)) {
        return None;
    }
    if against {
        return Some(NEUTER_AGAINST.clone());
    }
    Some(
        "Usually yes. Spaying before the first season cuts the risk of mammary tumours and \
removes pyometra, an emergency operation; neutering a male stops the roaming, \
spraying and fights. The one trade-off is timing in a large dog, where vets often \
wait until it has stopped growing - so ask about the breed and the size you have."
            .into(),
    )
}

/// Supplies for a new cat or kitten; "food for its age" as most of these lines say "cat".
pub fn _kitten_kit(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", text::squash(&clean_or_text_lower(understood)));
    if _WEIGHING_UP.is_match(&said) && !_FOR_THE_CAT.is_match(&said) && !_TWO_OF_THEM.is_match(&said) {
        return None;
    }
    let kitten = understood.words.iter().any(|w| w == "kitten" || w == "kittens");
    let food = if kitten { "kitten food" } else { "food for its age" };
    Some(format!(
        "Litter tray and litter, two bowls, a scratching post, a carrier, a soft bed and \
{food}, plus a vet visit booked for the first week. A second litter tray and a few \
toys save a lot of trouble, and nothing stringy or small enough to swallow."
    ))
}

/// The buying shape ("what do i need for a rabbit"): a line asking what to get for the animal.
pub fn _the_buying_shape(text_: &str) -> bool {
    _BUYING_FOR.is_match(&text_.to_lowercase())
}

/// `_PET_NAMES`: each small animal's name and the pattern that finds it in a line.
static _PET_NAMES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    g("_PET_NAMES")
        .items()
        .iter()
        .map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].as_re().expect("a regex")))
        .collect()
});

/// The animal the line names, else the one the chat was last about, else `default`.
pub fn _the_pet_named(memory: &Memory, understood: &Understood, default: Option<&str>) -> Option<String> {
    let mut said = vec![_said(understood)];
    let mut earlier = last_users(memory, 3);
    earlier.reverse();
    said.extend(earlier);
    for line in &said {
        for (name, pattern) in _PET_NAMES.iter() {
            if pattern.is_match(line) {
                return Some(name.clone());
            }
        }
    }
    default.map(str::to_string)
}

/// The small animal this card is about: the one named, or the one the chat was about.
pub fn _the_small_pet(memory: &Memory, understood: &Understood) -> String {
    _the_pet_named(memory, understood, Some("rabbit")).unwrap_or_default()
}

static _PET_CLEANING: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_PET_CLEANING"));
static _PET_SETUP: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_PET_SETUP"));
static _PET_CHECKUP: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_PET_CHECKUP"));
static _PET_LITTER: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_PET_LITTER"));

/// The set-up (or the cleaning) for the small animal the line actually names.
pub fn _small_pet_setup(memory: &Memory, understood: &Understood) -> Option<String> {
    let animal = _the_small_pet(memory, understood);
    if _CLEANING_ASK.is_match(&_said(understood)) {
        return lookup(&_PET_CLEANING, &animal);
    }
    lookup(&_PET_SETUP, &animal)
}

/// The hamster card holds the same two questions, so it answers the one that was asked.
pub fn _the_hamster_basics(_memory: &Memory, understood: &Understood) -> Option<String> {
    if _CLEANING_ASK.is_match(&_said(understood)) {
        return lookup(&_PET_CLEANING, "hamster");
    }
    lookup(&_PET_SETUP, "hamster")
}

/// The yearly visit, in the animal the line names: a rabbit's jabs are not a puppy's.
pub fn _the_pet_checkup(memory: &Memory, understood: &Understood) -> Option<String> {
    let animal = _the_pet_named(memory, understood, Some("other")).unwrap_or_default();
    lookup(&_PET_CHECKUP, &animal).or_else(|| lookup(&_PET_CHECKUP, "other"))
}

/// The tray question, in the animal the line names: a rabbit's tray is not a cat's.
pub fn _the_litter_routine(memory: &Memory, understood: &Understood) -> Option<String> {
    let animal = _the_pet_named(memory, understood, Some("cat")).unwrap_or_default();
    lookup(&_PET_LITTER, &animal).or_else(|| lookup(&_PET_LITTER, "cat"))
}

/// The colour of the gums first, the toothbrush second.
pub fn _pet_mouth(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<&str> = understood.words.iter().map(String::as_str).collect();
    let said = understood.clean.as_str();
    let has = |list: &[String]| list.iter().any(|w| words.contains(w.as_str()));
    let about_gums = said.contains("gums") || said.contains("gum colour") || said.contains("gum color");
    if about_gums && ["black", "dark", "patch", "patches"].iter().any(|w| words.contains(w)) {
        return Some(
            "Black or dark patches are usually just pigment, and they are fine if they have \
always been there. A black or bruised patch that has just appeared, or gums that \
are dark and cold, is a vet call today."
                .into(),
        );
    }
    if about_gums && has(&GUMS_RED) {
        return Some(
            "Red or swollen gums are gingivitis, and brushing will not shift it on its own: \
book a vet check for a scale and polish, then brush at the gum line every day, \
because that is what keeps it from coming back. Sore gums that bleed when touched \
are worth showing to the vet rather than watching for a week."
                .into(),
        );
    }
    if about_gums && has(&GUMS_COLOUR) {
        if (has(&GUMS_FINE) || has(&GUMS_ASKING)) && !(has(&GUMS_BAD) || has(&GUMS_CHANGED)) {
            return Some(
                "Pink and wet is right, and a pale pink is normal too. Press a fingertip on the \
gum until it goes white: the colour should come back in under two seconds, and \
pale, white, grey or blue gums mean the vet today, because they point to blood \
loss, shock or too little oxygen."
                    .into(),
            );
        }
        return Some(
            "Pale, white, grey or blue gums mean the vet today, not a wait and see: they point \
to blood loss, shock or too little oxygen, and a pet that is also weak or breathing \
fast is an emergency. Press a fingertip on the gum until it goes white - the \
colour should come back in under two seconds, and pink and wet is what you want."
                .into(),
        );
    }
    Some(
        "Brush them, with a pet toothpaste and a finger brush, at the back first and only for \
a few seconds to start - human toothpaste is not for animals. Brown tartar and bad \
breath mean a vet check: a scale and polish is what stops gum disease, and chewing \
is not a substitute for it."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Feeding a dog, and the follow-up that names the dog only by a pronoun.

/// The age a dog comes off puppy food, by size, and the week-long changeover.
pub fn _a_switch_to_adult_food(understood: &Understood) -> bool {
    let here = understood.clean.to_lowercase();
    SWITCH_WORDS.iter().any(|w| here.contains(w.as_str()))
}

inline_re!(A_PRONOUN_FOR_THE_DOG, r"\b(?:him|her|his|he|she|it)\b");

/// The dog's feeding answer for a line whose only subject is a pronoun, or "".
pub fn the_dog_feed_follow_up(text_: &str, memory: &Memory) -> String {
    let said = text_.to_lowercase();
    if !FEED_ASK.is_match(&said) || SICK_NOW.is_match(&said) {
        return String::new();
    }
    if DOG_WORD.is_match(&said) {
        return String::new(); // the dog row's own key reaches an explicit dog line
    }
    if !A_PRONOUN_FOR_THE_DOG.is_match(&said) {
        return String::new();
    }
    let recent = last_users(memory, 3);
    let (mut dog_at, mut other_at) = (-1i64, -1i64);
    for (at, line) in recent.iter().enumerate() {
        if DOG_WORD.is_match(line) {
            dog_at = at as i64;
        }
        if NOT_DOG.is_match(line) {
            other_at = at as i64;
        }
    }
    if dog_at < 0 || other_at > dog_at {
        return String::new();
    }
    let reading = Understood { clean: said.clone(), text: said, ..Default::default() };
    _dog_feed(memory, &reading)
}

/// A portion guide, or bland food when the dog has been ill in the last few turns.
pub fn _dog_feed(memory: &Memory, understood: &Understood) -> String {
    if _a_switch_to_adult_food(understood) {
        return "Most dogs move onto adult food at about a year. Small breeds can go at nine to \
twelve months; the big ones, a Labrador upwards, do better on puppy food until \
eighteen months to two years, because their joints are still growing. Change \
over a week or two rather than overnight: a quarter of the new food for two or \
three days, then half, then three quarters, and watch his poo - loose means slow \
down. Keep roughly the same amount and go by his ribs and waist rather than the \
packet."
            .into();
    }
    let mut said = vec![understood.clean.clone()];
    said.extend(last_users(memory, 3));
    let text_ = said.join(" ");
    // The illness has to be the dog's: a person who felt sick yesterday still wants the
    // portion guide, not a bland diet for the dog.
    let ill = said
        .iter()
        .any(|line| ILL.iter().any(|word| line.contains(word.as_str()) && meets(&az_words(line), &PET)));
    if ill {
        return "Keep tomorrow bland and small: plain chicken and rice, or his normal food soaked \
soft, in four little meals rather than two big ones. Over a day or two mix his \
usual food back in. If the sickness comes back, or he stops drinking, ring the \
vet rather than waiting."
            .into();
    }
    if text_.contains("puppy") || text_.contains("pup") {
        return "Puppies need more than the adult 2% guide: feed a complete puppy food and follow \
the packet's weight and age chart. Four small meals a day up to about four \
months, then three, then two after six months. You should feel his ribs easily \
but not see them; ask the vet to weigh him at each vaccination visit and adjust \
from there."
            .into();
    }
    "As a rough guide, an adult dog eats about 2% of his body weight a day: 400g of food \
for a 20kg dog, split into two meals. Use the bag or tin as a starting point, then go \
by condition: you should feel ribs easily but not see them, and see a waist from \
above. Puppies need more, and a vet can weigh him and give you an exact amount."
        .into()
}

/// A plan, with the cause in its opening sentence when the line asked why.
pub fn _with_cause(memory: &Memory, plan: &str, cause: &str) -> String {
    // `memory["understood"]["wants"]`: the intent observer writes it together with
    // `memory["intent_wants"]`, which is where this port's memory keeps it.
    let wants = memory
        .understood
        .as_ref()
        .and_then(|u| u.wants.as_deref())
        .or(memory.intent_wants.as_deref())
        .unwrap_or("");
    if wants != "reason" {
        return plan.to_string();
    }
    let reason = text::rstrip_chars(text::strip(cause), ".");
    if reason.is_empty() {
        return plan.to_string();
    }
    let opening = format!("{}{}", text::slice(plan, 0, Some(1)).to_lowercase(), text::slice(plan, 1, None));
    format!("{}{}, so {}", text::slice(reason, 0, Some(1)).to_uppercase(), text::slice(reason, 1, None), opening)
}

/// Barking: the plan, and the alarm in it when the line asked why.
pub fn _barking(memory: &Memory, _understood: &Understood) -> Option<String> {
    Some(_with_cause(
        memory,
        "Find what starts it and block that first: a curtain over the window he watches, or the \
post put through a box away from the door. Then reward the quiet - a treat the moment he \
stops - and don't shout, because to a dog shouting sounds like you joining in. If he \
barks for hours when he is left, that is the being-left problem rather than the barking \
one.",
        "Barking is usually the alarm going off",
    ))
}

/// A dog wary of a person: the plan, and the fear in it when the line asked why.
pub fn _scared_of(memory: &Memory, understood: &Understood) -> Option<String> {
    Some(_with_cause(memory, &_scared_of_plan(memory, understood), "Wary dogs are usually more frightened than bold"))
}

static WARY_ONE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("WARY_ONE").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
static WARY_MANY: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("WARY_MANY").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});

/// How to win over a dog wary of the person (or the people) the line names.
pub fn _scared_of_plan(_memory: &Memory, understood: &Understood) -> String {
    let lowered = understood.clean.to_lowercase();
    let words = az_words(&lowered);
    for (word, phrase) in WARY_ONE.iter() {
        if words.contains(word.as_str()) {
            return format!(
                "Let him come to {phrase} on his own terms: no eye contact, no reaching over him, \
and treats tossed on the floor while {phrase} sits nearby and ignores the dog. \
Being boring and calm is what builds trust, and it usually takes weeks \
rather than days."
            );
        }
    }
    for (word, phrase) in WARY_MANY.iter() {
        if words.contains(word.as_str()) {
            return format!(
                "Let him come to {phrase} on his own terms: no eye contact, no reaching over him, \
and treats tossed on the floor while they sit nearby and ignore the dog. \
Being boring and calm is what builds trust, and it usually takes weeks \
rather than days."
            );
        }
    }
    "Let him come to the person he is wary of on his own terms: no eye contact, no \
reaching over him, and treats tossed on the floor while that person sits nearby \
and ignores the dog. Being boring and calm is what builds trust, and it usually \
takes weeks rather than days."
        .into()
}

inline_re!(
    A_DOGS_AGE,
    r"\b(?:(?:is|aged|about|nearly|almost|turned|turning|over|just over)\s+(\d{1,2})\b|(\d{1,2})\s*(?:years?\s+old|years?|yrs?|yo)\b)"
);

/// A dog up at night, answered for the dog the line describes: a puppy, an old dog, or neither.
pub fn _dog_night(memory: &Memory, understood: &Understood) -> Option<String> {
    let mut words: HashSet<&str> = understood.words.iter().map(String::as_str).collect();
    let clean = format!(" {} ", understood.clean);
    let earlier = format!(" {} ", last_users(memory, 2).join(" "));
    const OLD: [&str; 7] = ["elderly", "senior", "old", "older", "oldest", "aging", "ageing"];
    let older = OLD.iter().any(|w| words.contains(w)) || A_DOGS_AGE.is_match(&clean);
    if !older && (earlier.contains("puppy") || earlier.contains(" pup ")) {
        words.insert("puppy");
    }
    if words.contains("puppy") || words.contains("puppies") || clean.contains("weeks old") || clean.contains("months old") {
        return Some(
            "A puppy cannot hold it all night: at eight weeks expect to get up once, around 2 \
or 3am, and take him out just before you go to bed. Keep the last meal early and \
the bed beside yours so he can smell you, and answer the crying quietly and \
briefly with no lights and no play, so night stays boring. Most puppies sleep \
seven hours by about four months."
                .into(),
        );
    }
    if older {
        return Some(
            "An older dog who starts waking at night is usually in discomfort rather than \
being naughty: arthritis makes lying still painful, and a dog who is stiff, \
pacing, panting or confused in the evening needs the vet rather than a training \
plan. Book a check, ask about pain relief, put a soft bed in a warm room with a \
night light, and take him out once last thing; sudden confusion in an old dog is \
worth a same-day call."
                .into(),
        );
    }
    Some(
        "Keep the night dull and the day busy: a last toilet trip on the lead, no lights, no \
chat and no play when he wakes, and a long walk plus some training in the afternoon \
so he is tired at bedtime. If he is sleeping a lot in the day, wake him gently for a \
short walk. A dog who starts waking at night after years of sleeping through is worth \
a vet check, especially if he is drinking more or seems stiff."
            .into(),
    )
}

/// Whether the line is about a dog on a lead, rather than a razor or a fitness plan.
pub fn _on_a_lead(understood: &Understood) -> bool {
    let clean = text::squash(&understood.clean);
    let subjects = text::split_ref(&clean).iter().any(|w| _LEAD_SUBJECTS.contains(*w));
    if !(subjects || _LEAD_PHRASES.iter().any(|p| clean.contains(p.as_str()))) {
        return false;
    }
    let subject = text::squash(&sh::field(understood, "subject").to_lowercase());
    if subject.is_empty() {
        return true;
    }
    meets(&az_words(&subject), &_LEAD_SUBJECTS)
}

/// A dog on a lead: the fit of a collar he slips or that rubs, else the stop-and-go training.
///
/// Python joins the line's words as a *set* (`" ".join(set(clean.split()))`), so a two-word
/// slip phrase ("comes off") matches only when hash order happens to put the two side by side;
/// this port keeps the line's own order of first appearance (set order noted, not reproduced).
pub fn _dog_pulling(_memory: &Memory, understood: &Understood) -> Option<String> {
    let mut words: Vec<&str> = Vec::new();
    for w in text::split_ref(&understood.clean) {
        if !words.contains(&w) {
            words.push(w);
        }
    }
    let clean = words.join(" ");
    if SLIP_WORDS.iter().any(|s| clean.contains(s.as_str())) {
        return Some(
            "Check the fit before the training: a collar should take two fingers and no more, \
and a dog who backs out of a lead does it at the widest part of his head, so a \
sighthound or a narrow-headed dog needs a harness or a collar that tightens a \
little as he pulls. Practise putting it on and off with a treat so the lead is \
not the thing he braces against, and check the clip and stitching for wear."
                .into(),
        );
    }
    if clean.contains("collar")
        && TIGHT_WORDS.iter().any(|t| words.contains(&t.as_str()))
        && !PULL_WORDS.iter().any(|p| words.contains(&p.as_str()))
    {
        return Some(
            "Loosen it a notch: two fingers should slide under the collar and it should sit high \
on the neck, behind the ears, rather than low over the shoulders. Check for a \
mark or rubbed hair after a walk, and if he pulls into it, a harness takes the \
pressure off his throat instead."
                .into(),
        );
    }
    Some(
        "Train the lead as a stop-and-go game: the moment the lead goes tight, stand still \
until it slackens, then walk on, and reward him at your side with a treat. A \
front-clip harness or head halter helps while he learns, and a treat held at your \
knee keeps him where you want him."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// The litter tray and a cat's waterworks.

/// `(understood.get("norm_text") or understood.get("clean") or "").lower()`.
fn norm_or_clean_lower(understood: &Understood) -> String {
    sh::or(&understood.norm_text, &understood.clean).to_lowercase()
}

/// The tray answer, with the spot first when the person says where it happened.
pub fn _litter_reply(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = norm_or_clean_lower(understood);
    if _WHERE_IT_LANDED.is_match(&text_) {
        return Some(
            "Clean the spot properly first, because the smell is what takes her back to it: \
soak it with an enzyme cleaner, leave it to work and let it dry, and skip bleach \
and ammonia, because both smell like wee to a cat. Then the tray - one per cat \
plus one, scooped daily, unscented litter, in a quiet corner away from the food - \
and whatever changed in the house: a move, a new pet or a baby, building work, a \
door that now stays shut. A cat who was always clean and has suddenly started \
needs the vet first, because a urine infection or a bladder problem sits behind \
a lot of it."
                .into(),
        );
    }
    Some(LITTER_REPLY.clone())
}

/// A cat's urine: marking, a blocked bladder (an emergency), or cystitis.
pub fn _cat_urine(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = norm_or_clean_lower(understood);
    if _SPRAY.is_match(&text_) && !_BLOOD.is_match(&text_) {
        if _the_neuter_is_done(understood) {
            return Some(
                "A cat who is already done and still sprays is not doing it for the hormones: \
in a neutered tom it is usually stress - a new animal in the house, a tray \
moved, a cat outside the window - or a habit left from being done late. Put \
a tray on each floor, plug in a pheromone diffuser, clean the spot with an \
enzyme cleaner rather than bleach, and never shout at him for it. Spraying \
with straining, blood or a tray he has stopped using is a vet check instead."
                    .into(),
            );
        }
        return Some(
            "Spraying up a wall or the back of the sofa is marking, and an unneutered tom does \
it most: neutering stops most of it, and the smell with it, though a tom done \
late may keep the habit. In a neutered cat new spraying is usually stress - a \
new animal, a moved tray, a cat outside the window - so put a tray on each \
floor, plug in a pheromone diffuser, and clean the spot with an enzyme cleaner \
rather than bleach. Never shout at him for it. Spraying with straining, blood \
or a litter tray he has stopped using is a vet check instead."
                .into(),
        );
    }
    if _BLOCKED.is_match(&text_) || (_TRYING.is_match(&text_) && _LITTLE.is_match(&text_)) {
        return Some(
            "A cat straining and passing little or nothing needs the vet now, tonight if you \
have to: a blocked bladder is an emergency, and it is most dangerous in male cats, \
who can go downhill in a day. Do not wait for the morning, do not press on his \
belly, and take him in a carrier with a towel over it. Keep a note of the last \
time you saw him pass a proper amount of urine."
                .into(),
        );
    }
    Some(
        "Blood or discomfort when a cat wees is usually cystitis, and it needs the vet in the \
next day or two rather than a wait-and-see, because the same signs cover crystals, a \
stone and the start of a blockage. Ring and ask them to test a urine sample if you can \
catch one in a clean tray with a little non-absorbent litter, keep the water bowl \
fresh and away from the food, and cut dry food back for a few weeks. If he ever \
strains and passes nothing at all, that is an emergency, not a next-day call."
            .into(),
    )
}

/// The vet's clear result in a cat chat, which is the behaviour answer, not a shrug.
pub fn _after_a_clear_urine_result(memory: &Memory, understood: &Understood) -> Option<String> {
    let mut said = vec![clean_or_text_lower(understood)];
    said.extend(last_users(memory, 3));
    if !said.iter().any(|line| _A_CAT_NAMED.is_match(line)) {
        return None;
    }
    if !_A_CLEAR_RESULT.is_match(&said[0]) {
        return None;
    }
    Some(NO_INFECTION_TEXT.clone())
}

/// A spoon of plain yoghurt is the dairy this row never answered; milk gets the milk answer.
pub fn _pet_dairy(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = norm_or_clean_lower(understood);
    if text_.contains("yoghurt") || text_.contains("yogurt") {
        return Some(
            "Plain yoghurt is fine: a spoonful or two of the natural sort with no sugar and \
no sweetener in it is a cool treat in summer, and it is gentle on a poorly \
stomach. Skip the fruit and vanilla ones, which are mostly sugar, and take it \
off the day's dinner rather than adding it on top."
                .into(),
        );
    }
    Some(_PET_DAIRY_MILK.clone())
}

inline_re!(A_RABBIT_WORD, r"\b(rabbit|rabbits|bunny|bunnies)\b");

/// Loose stools in a cat or a rabbit, the two animals that must not skip a meal.
pub fn _pet_tummy(_memory: &Memory, understood: &Understood) -> Option<String> {
    if A_RABBIT_WORD.is_match(&understood.clean) {
        return Some(
            "A rabbit that is passing soft or watery droppings should not be starved: keep the \
hay and water going, take away the greens and pellets for a day, and ring a \
rabbit-savvy vet today. A rabbit that stops eating or passing droppings at all is \
an emergency, because the gut can stop moving within a day."
                .into(),
        );
    }
    Some(
        "Don't fast a cat: a cat that goes a day without food can run into liver trouble, so \
keep the food going as small bland meals - plain cooked chicken or white fish, a \
spoon at a time - with plenty of water, and leave her normal food out. Ring the vet \
the same day if there is blood, if she is flat or off her water, if it lasts more \
than a day, or if it is a kitten, because a small animal dehydrates fast."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// What a small animal may eat, and which animal a line is about.

inline_re!(A_GERBIL, r"\b(?:gerbil|gerbils)\b");
inline_re!(A_BIRD, r"\b(?:budgie|budgies|budgerigar|parakeet|cockatiel|canary|parrot|parrots|bird|birds)\b");
inline_re!(A_CHINCHILLA, r"\b(?:chinchilla|chinchillas)\b");
inline_re!(A_TORTOISE, r"\b(?:tortoise|tortoises|turtle|turtles)\b");
inline_re!(A_HORSE, r"\b(?:horse|horses|pony|ponies|foal|foals|mare|mares|gelding|geldings)\b");

/// The small animal the line names, as the name this table is keyed by.
pub fn _small_pet_kind(text_: &str) -> &'static str {
    if A_GERBIL.is_match(text_) {
        return "gerbil";
    }
    if SMALL_PET_HAMSTER.is_match(text_) {
        return "hamster";
    }
    if SMALL_PET_GUINEA.is_match(text_) {
        return "guinea pig";
    }
    if A_BIRD.is_match(text_) {
        return "bird";
    }
    if A_CHINCHILLA.is_match(text_) {
        return "chinchilla";
    }
    if A_TORTOISE.is_match(text_) {
        return "tortoise";
    }
    if A_HORSE.is_match(text_) {
        return "horse";
    }
    ""
}

static SMALL_PET_FOODS: LazyLock<Vec<(String, Vec<(String, String)>)>> = LazyLock::new(|| {
    g("SMALL_PET_FOODS")
        .pairs()
        .iter()
        .map(|(k, v)| {
            (k.str().to_string(), v.pairs().iter().map(|(f, h)| (f.str().to_string(), h.str().to_string())).collect())
        })
        .collect()
});
static SMALL_PET_DIET: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("SMALL_PET_DIET"));

/// What the animal in the line may eat, in that animal's own terms.
pub fn _small_pet_food(understood: &Understood) -> Option<String> {
    let text_ = clean_or_text_lower(understood);
    let kind = _small_pet_kind(&text_);
    if kind.is_empty() {
        return None;
    }
    let words = az_words(&text_);
    if let Some((_, foods)) = SMALL_PET_FOODS.iter().find(|(k, _)| k == kind) {
        for (food, how) in foods {
            if words.contains(food.as_str()) || (food.chars().count() > 3 && words.contains(format!("{food}s").as_str())) {
                return Some(how.clone());
            }
        }
    }
    if kind == "tortoise" || kind == "horse" {
        return None;
    }
    lookup(&SMALL_PET_DIET, kind)
}

/// The animal a reply is said about: `{one}`, `{kind}` and `{they}` in its templates.
#[derive(Clone, Debug, PartialEq)]
pub struct PetWords {
    pub one: &'static str,
    pub kind: &'static str,
    pub they: &'static str,
}

impl PetWords {
    const CAT: PetWords = PetWords { one: "a cat", kind: "cat", they: "cats" };
    const RABBIT: PetWords = PetWords { one: "a rabbit", kind: "rabbit", they: "rabbits" };
    const DOG: PetWords = PetWords { one: "a dog", kind: "dog", they: "dogs" };

    /// `template.format(**pet)`.
    fn fill(&self, template: &str) -> Option<String> {
        py_format(template, &[("one", self.one), ("kind", self.kind), ("they", self.they)])
    }
}

inline_re!(A_CAT_WORD, r"\b(cat|cats|kitten|kittens)\b");

/// The animal the line names, so a cat question is answered about a cat and not about a dog.
pub fn _pet_words(understood: &Understood) -> PetWords {
    let mut text_ = understood.clean.as_str();
    let pointed_at = understood.animal_in_hand.as_str();
    if !pointed_at.is_empty() {
        if pointed_at.starts_with("cat") || pointed_at.starts_with("kitten") {
            return PetWords::CAT;
        }
        if pointed_at.starts_with("rabbit") || pointed_at.starts_with("bunny") {
            return PetWords::RABBIT;
        }
        if pointed_at.starts_with("dog") || pointed_at.starts_with("pupp") {
            return PetWords::DOG;
        }
        text_ = pointed_at;
    }
    if A_CAT_WORD.is_match(text_) {
        return PetWords::CAT;
    }
    if A_RABBIT_WORD.is_match(text_) {
        return PetWords::RABBIT;
    }
    PetWords::DOG
}

/// Fast or laboured breathing is an emergency, for whichever animal the line named.
pub fn _breathing_fast(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = understood.clean.as_str();
    let pet = _pet_words(understood);
    if pet.kind == "dog" && _IS_PANTING.is_match(text_) && !_BREATHING_ITSELF.is_match(text_) {
        return None;
    }
    pet.fill(
        "Ring the vet now: {one} breathing more than about 40 breaths a minute at rest, or \
breathing with the mouth open, is an emergency, and cats and dogs both hide chest and \
heart trouble until it is serious. Keep them cool and quiet, and carry them in rather \
than letting them walk.",
    )
}

/// True when the line names a pet these answers are not written for.
pub fn _other_pet(understood: &Understood) -> bool {
    let text_ = format!(" {} ", understood.clean.to_lowercase());
    OTHER_PETS.iter().any(|w| text_.contains(&format!(" {w} ")))
}

/// A cut on a pet, about the animal the line named: a cat is not "him".
pub fn _pet_wound(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_words(understood).fill(&PET_WOUND_REPLY)
}

// ---------------------------------------------------------------------------------------------
// Neutering.

/// True when the line states the animal has already been neutered, and asks nothing.
pub fn _the_neuter_is_done(understood: &Understood) -> bool {
    let said = format!("{} {}", understood.norm_text, understood.clean).to_lowercase();
    if !_NEUTER_ALREADY_DONE.is_match(&said) || _NEUTER_NOT_DONE.is_match(&said) {
        return false;
    }
    !_NEUTER_STILL_ASKED.is_match(&said)
}

/// The line's own words, for reading the sex of the animal it is about.
pub fn _the_sex_words(understood: &Understood) -> String {
    format!("{} {} {}", understood.clean, understood.text, understood.animal_in_hand).to_lowercase()
}

inline_re!(A_FEMALE, r"\b(spay|spays|spayed|spaying|queen|bitch|doe|she|shes|she's|her|hers|girl|female)\b");
inline_re!(A_MALE, r"\b(tom|castrat\w*|buck|he|hes|he's|him|his|boy|male|sire)\b");

/// Whether the animal the line is about is male, female, or not said ("").
pub fn _the_neuter_sex(memory: &Memory, understood: &Understood) -> &'static str {
    let mut said = _the_sex_words(understood);
    for user in last_users(memory, 2) {
        said.push(' ');
        said.push_str(&user);
    }
    if A_FEMALE.is_match(&said) {
        return "female";
    }
    if A_MALE.is_match(&said) {
        return "male";
    }
    ""
}

/// `_ANI_KIND`: each animal's kind and the words that name it, in the order they are tried.
static _ANI_KIND: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    g("_ANI_KIND").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].strs())).collect()
});
static ANI_KIND_RES: LazyLock<Vec<(String, Vec<Re>)>> =
    LazyLock::new(|| _ANI_KIND.iter().map(|(k, words)| (k.clone(), bounded_res(words))).collect());

/// What being neutered does and does not change, for the animal and the behaviour in hand.
pub fn _the_neuter_is_done_reply(memory: &Memory, understood: &Understood) -> String {
    let mut said = format!(" {} ", understood.clean.to_lowercase());
    for user in last_users(memory, 3) {
        said += &user;
        said.push(' ');
    }
    let mut animal = _the_animal(understood).kind;
    if animal == "animal" {
        for (kind, words) in ANI_KIND_RES.iter() {
            if words.iter().any(|r| r.is_match(&said)) {
                animal = kind.clone();
                break;
            }
        }
    }
    let sex = _the_neuter_sex(memory, understood);
    let (poss, them, who) = match sex {
        "female" => ("her", "her", "she"),
        "male" => ("his", "him", "he"),
        _ => ("their", "them", "the cat"),
    };
    let behaviour = _NEUTER_BEHAVIOUR.is_match(&said);
    if animal == "cat" && behaviour {
        return format!(
            "Then it is not the hormones: a neutered cat that sprays or wees outside the tray \
is usually stressed or has a urine infection. Put a tray on every floor {who} uses \
and scoop it daily, plug in a pheromone diffuser, clean the spot with an enzyme \
cleaner rather than bleach, and see the vet first if it started suddenly or {who} \
strains and passes little. The other sprayers are new cats outside the window, \
a moved tray and building work."
        );
    }
    if animal == "cat" {
        return format!(
            "Then the spraying, roaming and fighting that the operation stops are already \
handled: {poss} hormones settle about six weeks after it, so anything that starts \
after that is stress or habit rather than hormones. Neutering does not make a \
cat lazy or change its character, only the urge to roam, spray and fight."
        );
    }
    if animal == "dog" && behaviour {
        return format!(
            "A neutered dog can still mount, mark and pull towards a bitch, because the \
hormones take about six weeks to settle after the operation, and a dog done \
later can keep the habit for longer. Interrupt it and reward the alternative - \
send {them} to the same spot outside and praise it - and wash an indoor mark with \
an enzyme cleaner so the smell does not call {them} back."
        );
    }
    if animal == "dog" {
        return "Then the roaming, mounting and marking that the operation changes settle about \
six weeks after it as the hormones drop, and they are the main thing it changes. \
Anything left after that is training rather than hormones, and a neutered dog \
does not need to put on weight - feed the same portions unless the vet says \
otherwise."
            .into();
    }
    if animal == "rabbit" {
        return "For a doe the operation is the protection that matters: unspayed does get cancer \
of the womb young, and most have it by four, so being done is the right state to \
be in. It does not change a rabbit's character or make it put on weight on its \
own, and a buck is done mostly for the spraying and the mounting - keep the hay \
and the portions as they were and watch the weight over the winter."
            .into();
    }
    "The urge to roam, spray and fight fades over about six weeks after the operation as \
the hormones drop, so behaviour that starts later is habit or stress rather than \
hormones. Being neutered does not change an animal's character, and it does not make \
one put on weight on its own - keep the food and exercise as they were."
        .into()
}

inline_re!(NEUTER_ASKS_WHEN, r"\b(should|when|what age|how old)\b");
inline_re!(NEUTER_THE_OPERATION, r"\b(neutered|spayed|castrated|the snip|done)\b");
inline_re!(NEUTER_PRICE, r"\b(cost|costs|price|prices|much does it|how much (is|will|would)|cheapest|afford|pay for)\b");
inline_re!(NEUTER_CAT_OR_RABBIT, r"\b(cat|cats|kitten|kittens|moggy|rabbit|rabbits|bunny|bunnies)\b");
inline_re!(
    NEUTER_RECOVERY,
    r"\b(recover|recovery|recovering|after the (operation|op)|stitches|cones?|how long (does|till|until).*(take|recover)|come home)\b"
);
inline_re!(NEUTER_A_CAT, r"\b(cat|cats|kitten|kittens|moggy)\b");

/// When to have an animal neutered, about the animal the line named.
pub fn _pet_neuter(memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!("{} {}", understood.clean, understood.norm_text).to_lowercase();
    if _NEUTER_THE_DAY_OR_AFTER.is_match(&said) && !NEUTER_ASKS_WHEN.is_match(&said) {
        return None;
    }
    if _NEUTER_HOW_TO_TELL.is_match(&said) && NEUTER_THE_OPERATION.is_match(&said) && !_NEUTER_BEHAVIOUR.is_match(&said) {
        return Some(
            "Ask the vet to look, because most of it is a hands-on check: a male done a while \
ago has an empty scrotum, and a spayed female usually has no scar you can see. \
Rescue cats are often ear-tipped or tattooed, and the practice that did the \
operation has it on the record - so ring them, or the vet who has the animal's \
history, rather than guessing from behaviour alone."
                .into(),
        );
    }
    if _the_neuter_is_done(understood) && !_NEUTER_NOT_YET.is_match(&said) {
        if _NEUTER_ALSO_ABOUT.is_match(&said) && !_NEUTER_BEHAVIOUR.is_match(&said) {
            return None;
        }
        return Some(_the_neuter_is_done_reply(memory, understood));
    }
    let text_ = understood.clean.as_str();
    if NEUTER_PRICE.is_match(text_) {
        if NEUTER_CAT_OR_RABBIT.is_match(text_) {
            return Some(
                "Around £60 to £120 for a cat, more for a female than a male and more in a city \
practice, and a rabbit is usually £80 to £150. Ask for a written quote, since \
some practices add the pain relief or the cone on top, and Cats Protection or \
the PDSA can help with a voucher if money is tight."
                    .into(),
            );
        }
        return Some(
            "Around £150 to £300 to castrate a dog and £200 to £400 to spay a bitch, depending \
on the size of the dog and the practice. Ask for a written quote that includes the \
pain relief and the follow-up check, and the Dogs Trust or the PDSA can help if \
money is tight."
                .into(),
        );
    }
    if A_RABBIT_WORD.is_match(text_) {
        return Some(
            "Rabbits are done from four months for a buck and five or six for a doe, and a doe \
that is never bred should be spayed: unspayed does get cancer of the womb young, \
and most have it by four. Ask a rabbit-savvy vet, since the anaesthetic is the \
risky part."
                .into(),
        );
    }
    if NEUTER_RECOVERY.is_match(text_) {
        return _pet_words(understood).fill(
            "Most are home the same day and back to normal in two or three days: keep {one} \
quiet and indoors, no jumping or stairs, and let {one} sleep it off. A cone or a \
baby vest goes on if {one} licks at the wound, and the wound wants a look twice a \
day for redness, swelling or oozing. Ring the vet if {one} is not eating by the \
next morning, is sick, or the skin looks open. A male is usually himself again in \
a day or two; a female takes the longer end, about a week, with the stitches out \
at ten days.",
        );
    }
    if NEUTER_A_CAT.is_match(text_) {
        if _the_neuter_sex(memory, understood) == "male" {
            return Some(
                "Cats are usually done from four to six months, and a male is done at the the \
same age as a female: the reason for a tom is the spraying, the roaming and \
the fights, and those calm down about six weeks after the operation as his \
hormones settle. Book it once he is his normal self after his vaccinations, \
keep him in for a day or two after, and ring the vet if he is not eating by \
the next morning."
                    .into(),
            );
        }
        if _the_neuter_sex(memory, understood) == "female" {
            return Some(
                "Cats are usually done from four to six months, before the first season: a \
queen can get pregnant at four months. Book it once she is her normal self \
after her vaccinations, keep her in for a day or two after and give the vet \
a call if she is not eating by the next morning. A male cat is done the same \
age and calms down from the spraying once his hormones settle, about six \
weeks later."
                    .into(),
            );
        }
        return Some(
            "Cats are usually done from four to six months, before the first season: a queen \
can get pregnant at four months, and a tom is done at the same age, which stops \
most of the spraying and roaming about six weeks later as the hormones settle. \
Book it once the cat is its normal self after the vaccinations, keep it in for \
a day or two after, and ring the vet if it is not eating by the next morning."
                .into(),
        );
    }
    Some(
        "Dogs are usually done from six months, and puppies of the big breeds wait until nine \
to twelve months so their joints finish growing - your vet will say which for your \
breed. Book it in once he is well and his vaccinations are done, and expect him to be \
quiet for a day or two with a cone on for a week. Neutering a bitch before her first \
season lowers her chance of mammary tumours."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Paws, bones and food off the plate.

/// The same paw-licking question for a dog and for a cat, in the words each owner needs.
pub fn _paw_licking(_memory: &Memory, understood: &Understood) -> Option<String> {
    if _pet_words(understood).kind == "cat" {
        return Some(CAT_PAW_LICK_REPLY.clone());
    }
    Some(DOG_PAW_LICK_REPLY.clone())
}

/// Cooked bone advice, about the animal the line named.
pub fn _bone_hazard(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_words(understood).fill(
        "Cooked bones are the worry, because they splinter: chicken, turkey and lamb chop \
bones in particular. Don't make {one} sick and don't pull anything out of the mouth \
with your fingers. Ring the vet today and say what kind of bone, roughly how big and \
how long ago, and watch for retching, drooling, a hard or swollen belly, or pain when \
you touch it - any of those means the vet now, not tomorrow. A raw bone is softer, \
but a piece that gets stuck is the same problem.",
    )
}

/// Raw meat is a bacteria question, and the answer has to be about the pet in the line.
pub fn _raw_meat(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_words(understood).fill(
        "Raw chicken often carries campylobacter and salmonella, and a {kind} that has eaten \
some may bring it up or have a day or two of loose stools; most ride it out. Give \
{one} water, keep {one} off the beds and out of the kitchen while loose, wash your \
hands and the bowl, and ring the vet if the sickness lasts more than a day, if there \
is blood, or if {one} is flat and not drinking.",
    )
}

/// Grapes and raisins: ring the vet today, about the animal the line named.
pub fn _grapes(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_words(understood).fill(
        "Ring your vet today and say how many and how long ago, even if {one} seems fine: \
grapes and raisins can shut down a {kind}'s kidneys, and there is no amount known to \
be safe, so a small one and a handful is enough to treat. Treatment works much better \
started early, and the vet may make {one} sick or give fluids.",
    )
}

inline_re!(NOT_A_FOOD_CHARACTER, r"[^a-z' ]+");
static CAUTION_FOODS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("CAUTION_FOODS"));
static SAFE_FOODS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("SAFE_FOODS"));

/// `_can_pet_eat.<locals>.named`: whether the line names the food (a word that starts with it,
/// or a two-word name found in the line itself).
#[allow(non_snake_case)]
pub fn _can_pet_eat__named(food: &str, words: &HashSet<&str>, text_: &str) -> bool {
    let long = food.chars().count() > 3;
    if words.iter().any(|w| *w == food || (long && w.starts_with(food))) {
        return true;
    }
    food.contains(' ') && text_.contains(food)
}

/// The answer for a named food: a word of caution, a yes, or a pointer to the bad list.
pub fn _can_pet_eat(understood: &Understood) -> Option<String> {
    let text_ = format!(" {} ", NOT_A_FOOD_CHARACTER.sub(" ", &understood.clean.to_lowercase()));
    let words = az_words(&text_);
    let pet = _pet_words(understood);
    for (food, how) in CAUTION_FOODS.iter() {
        if _can_pet_eat__named(food, &words, &text_) {
            return pet.fill(&format!("Careful: {how}"));
        }
    }
    for (food, how) in SAFE_FOODS.iter() {
        if _can_pet_eat__named(food, &words, &text_) {
            return pet.fill(&format!(
                "Yes: {how}. Keep it to a small amount and off the dinner plate, \
because a pet that begs gets fat and a fat {{kind}} gets joint trouble. \
Anything new is worth a small piece of first, and a look an hour later."
            ));
        }
    }
    Some(
        "The ones to refuse are grapes and raisins, onions, garlic, leeks, chocolate, \
macadamia nuts, xylitol in sugar-free gum and raw dough. Plain cooked meat, rice, \
carrot and apple are fine in small amounts. If it was one of the bad list, ring the \
vet now rather than waiting for a symptom."
            .into(),
    )
}

/// True when the line is one animal getting at the other animal's dinner.
pub fn _one_pets_dinner(understood: &Understood) -> bool {
    let words = format!(" {} ", understood.words.join(" "));
    let said = |list: &[String]| list.iter().any(|p| words.contains(&format!(" {p} ")));
    if !said(&_OTHER_PETS_DINNER) {
        return false;
    }
    if said(&_A_PERSON_DID_IT) {
        return false;
    }
    if said(&_THE_ANIMAL_IS_ILL) {
        return false;
    }
    if said(&_DINNER_STOPPED) {
        return true;
    }
    if said(&_DINNER_ASKED_OR_REFUSED) {
        return false;
    }
    said(&_DINNER_TAKEN)
}

/// Hot or cold: a rabbit needs different things in each, and both are asked as "keep it ...".
pub fn _rabbit_weather(_memory: &Memory, understood: &Understood) -> Option<String> {
    const COLD: [&str; 7] = ["cold", "winter", "freezing", "frost", "warm", "warmth", "snow"];
    if understood.words.iter().any(|w| COLD.contains(&w.as_str())) {
        return Some(
            "Keep the hutch out of the wind and rain, with a dry, deep bed of hay or straw to \
burrow into, and a cover or an old duvet over the mesh at night, leaving the \
front open for air. Two rabbits huddle and keep each other warm, so a single one \
needs the extra bedding. Give warm water rather than cold, and check the bottle \
has not frozen."
                .into(),
        );
    }
    Some(
        "Shade, a second bowl of water and a frozen bottle of water wrapped in a towel for her \
to lean against, with the hutch moved out of the sun for the day. Rabbits cannot \
sweat and over 25C is already risky, and you must never bath one to cool her down: a \
damp bottom in summer is how flystrike starts. A rabbit stretched out and drooling, or \
not eating in the heat, needs the vet that day."
            .into(),
    )
}

/// The noise answer, with the animal the line named in it.
pub fn _noise_fear(_memory: &Memory, understood: &Understood) -> Option<String> {
    let pet = _pet_words(understood);
    py_format(
        "Let {one} leave the room and shut the door - walking away is what settles them - \
then switch it on for a few seconds at a time with the food bowl down, stopping \
before they bolt. Don't hold {one} still in front of it: a frightened animal that \
can't get away gets worse, not braver. If they shake for hours or won't eat \
afterwards, ask the vet.",
        &[("one", pet.one)],
    )
}

// ---------------------------------------------------------------------------------------------
// Drinking: the pet in the talk, salt water, too little and too much.

/// True when the nearest turn that names anyone names a pet.
pub fn _pet_in_the_talk(memory: &Memory, understood: &Understood) -> bool {
    let mut said = vec![understood.clean.clone()];
    said.extend(last_users(memory, 3));
    for line in said.iter().rev() {
        if PET_NAMED.is_match(line) {
            return true;
        }
        if PERSON_NAMED.is_match(line) {
            return false;
        }
    }
    false
}

inline_re!(A_PRONOUN_FOR_THE_PET, r"\b(?:he|she|him|her)\b");
static _ONE_ANIMAL: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_ONE_ANIMAL"));

/// The line with the chat's animal in place of a bare pronoun, or "".
pub fn the_animal_follow_up_line(text_: &str, memory: &Memory) -> String {
    let said = text_.to_lowercase();
    if said.is_empty() || PET_NAMED.is_match(&said) || !_FOOD_OR_DRINK.is_match(&said) {
        return String::new();
    }
    let Some(pronoun) = A_PRONOUN_FOR_THE_PET.search(&said) else {
        return String::new();
    };
    let mut lines = vec![said.clone()];
    let mut recent = last_users(memory, 3);
    recent.reverse();
    lines.extend(recent);
    for line in &lines {
        if let Some(named) = _ANIMAL_IN_A_TURN.search(line) {
            let word = named.str(1).to_lowercase();
            let animal = lookup(&_ONE_ANIMAL, &word).unwrap_or_else(|| word.trim_end_matches('s').to_string());
            return format!("{}my {} {}", &said[..pronoun.start()], animal, &said[pronoun.end()..]);
        }
        if _PERSON_IN_A_TURN.is_match(line) {
            return String::new();
        }
    }
    String::new()
}

inline_re!(A_DEHYDRATION_ASK, r"dehydrat|how do i know|drinking enough|signs?\b");

/// "How do I get my cat to drink more": the encourage-it question, or how to tell it is dry.
pub fn _pet_drink_more(memory: &Memory, understood: &Understood) -> Option<String> {
    if !_pet_in_the_talk(memory, understood) {
        return None;
    }
    let pet = _pet_words(understood);
    if A_DEHYDRATION_ASK.is_match(&understood.clean) {
        if pet.kind == "cat" {
            return Some(
                "Lift her scruff: on a hydrated cat it springs back at once, and on a dry one it \
stays up in a tent. Check the gums too - wet and pink is right, tacky or pale \
needs the vet - and the litter tray: a cat that stops weeing for a day is an \
emergency. A cat that is also off her food or hiding is not a wait-and-see."
                    .into(),
            );
        }
        return Some(
            "Lift the skin on his back: it should drop straight back, and slow is a sign he is \
short of water. Look at the gums - wet and pink is right, tacky or pale is not - \
and count the wees: dark, strong-smelling or fewer than usual means water first \
and the vet today. A dog that is also sick or loose dehydrates fast."
                .into(),
        );
    }
    if pet.kind == "cat" {
        return Some(
            "Switch part of the dry food to wet, which is where most of a cat's water comes \
from, and add a spoon of warm water to the wet food. Then make water worth \
walking to: a wide bowl filled to the brim, several bowls out of the litter tray \
and away from the food, and a running tap or a pet fountain. Tuna water in oil \
drained off, or a splash of unsalted broth, gets a fussy one going."
                .into(),
        );
    }
    Some(
        "Add water to his food - a splash in the bowl, or soak the kibble 10 minutes - and put \
a second bowl down somewhere he passes, filled to the brim, washed daily. A splash \
of unsalted broth or the water from boiled chicken makes it worth drinking, and a \
dog that has been running or is on dry food in warm weather needs more than the \
same dog on wet food."
            .into(),
    )
}

/// A pet that has drunk salt water needs the vet, and only a pet's line says so.
pub fn _pet_salt_water(memory: &Memory, understood: &Understood) -> Option<String> {
    if !_pet_in_the_talk(memory, understood) {
        return None;
    }
    _pet_words(understood).fill(
        "Ring the vet now and say how much salt water and when: salty water pulls water out of \
the body, and {one} that has had a lot will be sick, wobbly or drowsy within a few \
hours. Take the fresh water away so there is none to drink, offer small sips of plain \
water once the vet has said so, and do not wait for symptoms before you ring - this \
one is much easier treated early.",
    )
}

/// A pet that will not drink, kept apart from one that drinks too much.
pub fn _pet_not_drinking(memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = understood.clean.as_str();
    if !(_pet_in_the_talk(memory, understood) || SMALL_PETS.is_match(text_) || A_RABBIT_WORD.is_match(text_)) {
        return None;
    }
    let pet = _pet_words(understood);
    if A_RABBIT_WORD.is_match(text_) {
        return Some(
            "Ring the vet today: a rabbit that stops drinking is urgent, because the gut slows \
within hours and a rabbit not eating or drinking is a same-day emergency rather \
than a wait-and-see. While you ring, check the bottle - a stuck ball or a bottle \
that has stopped flowing is the commonest reason of all - and put a heavy bowl \
down as well, since a rabbit drinking from a bowl takes more. Wet the greens, \
offer a little unsweetened apple juice in the water, and keep him warm and quiet \
with hay within reach. Hunched, grinding his teeth, or no droppings for a few \
hours means the vet now, not tomorrow."
                .into(),
        );
    }
    if SMALL_PETS.is_match(text_) {
        return Some(
            "Ring the vet today: a guinea pig, hamster or rat that stops drinking goes \
downhill fast, and an animal that is also off its food, hunched or cold needs to \
be seen rather than watched. Check the bottle first - the ball stuck, the spout \
blocked, the water leaking away or warm - and put a shallow bowl down as well, \
because a guinea pig often drinks more from a bowl. Wash the veg and leave it wet \
for the water, keep fresh water in two places, and weigh the animal: a small \
animal that has lost weight is a vet call even if it is still nibbling."
                .into(),
        );
    }
    if pet.kind == "cat" {
        return Some(
            "Cats drink little anyway, so check the food first: a cat on wet food gets most \
of its water there, and a bowl next to the food or a running tap often starts one \
drinking. Offer tuna water or a splash of milk-free broth, keep the water away \
from the litter tray, and ring the vet the same day if she has not drunk for 24 \
hours, is straining in the tray, or is hiding and off her food."
                .into(),
        );
    }
    Some(
        "Check the boring things first: a dirty bowl, water left in the sun, or a bowl beside \
the food all put a dog off. Wash the bowl, put a second one down somewhere cool, and \
add a little water to his food or a splash of unsalted broth. A dog that goes a full \
day without drinking, or that is also sick, loose, or dribbling, needs the vet - \
especially in warm weather, when dehydration comes on quickly."
            .into(),
    )
}

/// How much a pet drinks, in the numbers for the animal the line named.
pub fn _pet_thirst(_memory: &Memory, understood: &Understood) -> Option<String> {
    if _pet_words(understood).kind == "cat" {
        return Some(
            "Measure it for a few days: a cat takes roughly 40 to 60ml per kilo a day, more \
on dry food and very little on wet. A cat that suddenly drinks a lot, or sits at \
the tap, needs the vet - thirst is how kidney disease and diabetes show first."
                .into(),
        );
    }
    Some(
        "Measure it: a dog takes about 50 to 100ml per kilo a day, so a 10kg dog drinking \
more than a litre is drinking too much. Sudden thirst is a symptom - diabetes, a \
kidney problem, or an infection of the womb in a bitch - so ring the vet, and sooner \
if he is also off his food or weeing in the house."
            .into(),
    )
}

/// A limp, for the animal that has it: the keeping-still half follows the animal.
pub fn _pet_limp(_memory: &Memory, understood: &Understood) -> Option<String> {
    let pet = _pet_words(understood);
    let rest = if _SMALL_PET_NAMED.is_match(&understood.clean) {
        "Keep him in his own cage on soft, dry bedding with the food and water where he \
can reach them without climbing, and handle him as little as you can"
    } else if pet.kind == "cat" {
        "Keep him in for a couple of days with no jumping up or down - carry him up the \
stairs if he will let you, and put the food and litter on the floor"
    } else if pet.kind == "rabbit" {
        "Keep him in a small run on soft bedding, not the garden, and off anything he can \
jump on or off"
    } else {
        "Rest him - lead walks to the toilet and nothing more for two days"
    };
    py_format(
        "{rest} - and look at the pad for a cut, a thorn or a torn nail. A leg he won't put \
weight on, a swelling, or a limp that is no better after two days needs the vet, and \
never give ibuprofen or paracetamol: both are poisonous to pets.",
        &[("rest", rest)],
    )
}

/// How this animal is picked up, said for the animal the line names.
pub fn _small_pet_handling(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = understood.clean.as_str();
    let (lead, more) = if _A_GUINEA_SAID.is_match(text_) {
        (
            "A guinea pig is scooped up in both hands from underneath, one hand under the \
hindquarters and the other across the shoulders, and held low over your lap - \
never by the scruff, the ears or a leg, and never up at chest height, because a \
guinea pig that is dropped or that jumps from a height breaks its back",
            "",
        )
    } else if _A_RODENT_SAID.is_match(text_) {
        (
            "A hamster, gerbil, rat or mouse is cupped in both hands, one under the chest and \
one over the back, and never picked up by the tail or the scruff",
            "",
        )
    } else if _A_FERRET_SAID.is_match(text_) {
        (
            "A ferret is picked up round the shoulders with the hindquarters supported in your \
other hand, and never by the tail or the ears",
            "",
        )
    } else {
        (
            "A rabbit is picked up with one hand under the chest and the other under the \
hindquarters, held against your body so the back legs cannot kick out - never by \
the ears, the scruff or the back of the neck, because a rabbit that kicks while \
nothing is holding its back end can break its own spine",
            " and a rabbit that stops eating after it is worth ringing the vet the same day",
        )
    };
    Some(format!(
        "{lead}. Sit on the floor with him on your lap for a few minutes a day and a bit of \
food in your hand, and put him down before he starts to struggle, so being picked up \
stops being a surprise. A pet that squeals, flinches or struggles at being handled \
when he did not before needs a vet - a sore tooth, an injured back or mites is the \
usual reason{more}."
    ))
}

/// True unless this is a plain how-to-hold line about a small animal that is not a rabbit.
pub fn _a_plain_hold_is_the_rodents_card(understood: &Understood) -> bool {
    let text_ = understood.clean.as_str();
    if _A_RABBIT_SAID.is_match(text_) || _A_FERRET_SAID.is_match(text_) {
        return true;
    }
    if !_A_RODENT_OR_GUINEA.is_match(text_) {
        return true;
    }
    if _A_HURT_OR_FRIGHT.is_match(text_) {
        return true;
    }
    !(_A_HOW_TO_HOLD.is_match(text_) || _A_WONT_BE_HELD.is_match(text_))
}

// ---------------------------------------------------------------------------------------------
// Lifespans, names, seasons and litters.

/// `_bird_lifespan(reply).<locals>.say`: a bird's life for a line that says life; None (pass)
/// for the same bird on a plate.
#[allow(non_snake_case)]
pub fn _bird_lifespan__say(reply: &str, understood: &Understood) -> Option<String> {
    if _LIFE_SAID.is_match(&understood.clean.to_lowercase()) {
        return Some(reply.to_string());
    }
    None
}

/// False for a line that gives a weight: "how long does a 5kg chicken take" is a dinner.
pub fn _not_a_weight_of_food(understood: &Understood) -> bool {
    !dup::a_weight(understood)
}

/// False when the line is naming a cat, so the season-and-pregnancy row stands down.
pub fn _not_a_named_cat(understood: &Understood) -> bool {
    let text_ = understood.clean.to_lowercase();
    if _A_SEASON_WORD.is_match(&text_) {
        return true;
    }
    for found in _CALLED_NAME.finditer(&text_) {
        // group 1 is None when the second form matched, and None is not in the set
        match found.get(1) {
            Some(name) if _CALLED_NOT_A_NAME_AT_HOME.contains(name) => {}
            _ => return false,
        }
    }
    true
}

/// Whether the line names a cat, kittens or a litter: what the newborn row is about.
pub fn _a_cat_in_the_line(understood: &Understood) -> bool {
    _A_CAT_IN_THE_LINE.is_match(&understood.clean.to_lowercase())
}

/// False for a care question about one cat, which the kitten-care row answers.
pub fn _a_litter_is_what_is_asked_about(understood: &Understood) -> bool {
    let text_ = understood.clean.to_lowercase().replace('\u{2019}', "'");
    if _A_NEWBORN_WORD.is_match(&text_) {
        return true;
    }
    !_A_CARE_PHRASE.is_match(&text_)
}

/// The condolence, or nothing ("") when this line does not say a pet has died.
pub fn _pet_loss(memory: &Memory, understood: &Understood) -> Option<String> {
    let said = clean_or_text_lower(understood);
    if said.is_empty() {
        return Some(LOSS_REPLY.clone());
    }
    if _DEAD_NOT_THE_PET.is_match(&said) {
        return Some(String::new());
    }
    if _PET_ATE_OR_ILL.is_match(&said) && !_THE_PET_IS_DYING.is_match(&said) {
        return Some(String::new());
    }
    if _a_death_the_line_does_not_name(&said, memory) {
        return Some(PREY_DEATH_REPLY.clone());
    }
    if _LOSS_SAID.is_match(&said) || _PUT_TO_SLEEP.is_match(&said) {
        return Some(LOSS_REPLY.clone());
    }
    Some(String::new())
}

/// The person's own last line and the bot's answer to it, as one piece of text.
pub fn _the_line_above(memory: &Memory) -> String {
    match memory.turns.last() {
        Some(turn) => format!("{} {}", turn.user, turn.bot),
        None => String::new(),
    }
}

/// True when the death is the prey's: a bare pronoun, a named prey, or one said again.
pub fn _a_death_the_line_does_not_name(said: &str, memory: &Memory) -> bool {
    let above = _the_line_above(memory).to_lowercase();
    if above.is_empty() {
        return false;
    }
    if !(_PREY_NAMED.is_match(&above) && _BRINGING_IT_IN.is_match(&above)) {
        return false;
    }
    if _PREY_NAMED.is_match(said) && !_AN_ANIMAL_OF_THEIRS.is_match(said) {
        return true;
    }
    if _A_REPEATED_DEATH.is_match(said) {
        return true;
    }
    if !_A_BARE_DEATH.is_match(said) {
        return false;
    }
    if _THE_ANIMAL_IS_NAMED.is_match(said) {
        return false;
    }
    true
}

/// The recall routine, or nothing ("") when the line is asking what the dog is called.
pub fn _dog_recall(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = understood.clean.as_str();
    if _NAME_ASK.is_match(said) {
        return Some(String::new());
    }
    if let Some(named) = _CALLED_NAME.search(said) {
        match named.get(1) {
            Some(name) if _CALLED_NOT_A_NAME.contains(name) => {}
            _ => return Some(String::new()),
        }
    }
    Some(_DOG_RECALL_REPLY.clone())
}

/// True when the line is telling the bot what the animal is called.
pub fn _a_name_is_being_given(understood: &Understood) -> bool {
    let said = clean_or_text_lower(understood);
    if _A_SEASON_WORD.is_match(&said) {
        return false;
    }
    if _NAME_ASK.is_match(&said) {
        return true;
    }
    match _CALLED_NAME.search(&said) {
        Some(named) => !matches!(named.get(1), Some(name) if _CALLED_NOT_A_NAME.contains(name)),
        None => false,
    }
}

inline_re!(NOT_A_WORD_CHARACTER, r"[^a-z0-9' ]+");

/// The safety answer - but a plain description of a household gets a note back instead.
pub fn _cat_and_children(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!("{} {}", understood.clean, understood.text);
    let said = format!(" {} ", NOT_A_WORD_CHARACTER.sub(" ", &said.to_lowercase()));
    if understood.kind == "statement" && !_CHILD_WORRY.iter().any(|w| said.contains(&format!(" {w} "))) {
        return None;
    }
    Some(CAT_AND_CHILDREN.clone())
}

// ---------------------------------------------------------------------------------------------
// What a pet costs.

/// The price of the small animal the line named, since each one costs a different amount.
pub fn _small_pet_cost(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = clean_or_text_lower(understood);
    let table: [(&Re, &str); 6] = [
        (&SMALL_PET_RABBIT, &SMALL_PET_RABBIT_LINE),
        (&SMALL_PET_HAMSTER, &SMALL_PET_HAMSTER_LINE),
        (&SMALL_PET_GUINEA, &SMALL_PET_GUINEA_LINE),
        (&SMALL_PET_BUDGIE, &SMALL_PET_BUDGIE_LINE),
        (&SMALL_PET_HORSE, &SMALL_PET_HORSE_LINE),
        (&SMALL_PET_FISH, &SMALL_PET_FISH_LINE),
    ];
    table.iter().find(|(pattern, _)| pattern.is_match(&said)).map(|(_, line)| line.to_string())
}

/// What a dog or a cat costs, priced for the route the person named.
pub fn _pet_cost(memory: &Memory, understood: &Understood) -> Option<String> {
    let mut said = clean_or_text_lower(understood);
    if !PET_COST_KITTEN.is_match(&said) && !PET_COST_DOG.is_match(&said) {
        said = format!("{} {}", said, dup::subject_word(memory).to_lowercase());
    }
    if PET_COST_ONE_OR_OTHER.is_match(&said) && PET_COST_KITTEN.is_match(&said) && PET_COST_DOG.is_match(&said) {
        return Some(PET_COST_COMPARE_LINE.clone());
    }
    if PET_COST_KITTEN.is_match(&said) {
        return Some(PET_COST_KITTEN_LINE.clone());
    }
    if PET_COST_RESCUE.is_match(&said) {
        return Some(PET_COST_RESCUE_LINE.clone());
    }
    if PET_COST_BREEDER.is_match(&said) {
        return Some(PET_COST_BREEDER_LINE.clone());
    }
    Some(PET_COST_PUPPY_LINE.clone())
}

/// A cat's coat: greasy, matted, or the plain brushing routine.
pub fn _cat_coat(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = norm_or_clean_lower(understood);
    if _COAT_GREASY.is_match(&text_) && !_COAT_MAT.is_match(&text_) {
        return Some(
            "A greasy or scurfy coat is rarely a brushing problem: a cat who has stopped washing \
herself, or an unneutered tom with a greasy patch at the base of the tail, means \
have her seen. Otherwise it is diet or over-grooming - a cheaper food, or fleas \
making her lick - so check for flea dirt with a damp tissue and move her onto a \
food with meat as the first ingredient."
                .into(),
        );
    }
    if _COAT_MAT.is_match(&text_) {
        return Some(
            "Work a mat out from the tip inwards with a metal comb or a slicker brush, a \
little at a time with your fingers holding the skin so you are not pulling it. \
Never cut one with scissors: cat skin tents up into them. A big or sore mat, or \
a cat that will not be handled, comes off in minutes with a groomer's clippers, \
and an old cat that stops grooming needs a vet look too."
                .into(),
        );
    }
    Some(
        "About 5 minutes: once a day for a long-haired cat, twice a week for a short-haired \
one. The two brushes worth having are a metal-toothed comb for long coats and a \
rubber curry brush for short ones. Brush along the way the fur lies, in sections on \
the back legs and under the belly where the tangles start, and keep it short and calm. \
A cat that hates it does better with a few minutes after a meal than one long \
session."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Weight.

static GOING_THIN_RES: LazyLock<Vec<Re>> = LazyLock::new(|| front_bounded_res(&_GOING_THIN));
static THIN_WORDS_RES: LazyLock<Vec<(Re, Re)>> = LazyLock::new(|| {
    _THIN_WORDS
        .iter()
        .map(|w| {
            let e = sh::escape(w);
            (Re::new(&format!(r"\b{e}\b"), 0), Re::new(&format!(r"\b{e} (coat|fur|hair|skin)"), 0))
        })
        .collect()
});
static WEIGHT_UP_RES: LazyLock<Vec<Re>> = LazyLock::new(|| front_bounded_res(&_WEIGHT_UP));

/// Whether the line says the animal is going thin, and is not asking to fatten one up.
pub fn _going_thin(text_: &str) -> bool {
    let said = GOING_THIN_RES.iter().any(|r| r.is_match(text_));
    let bare = THIN_WORDS_RES.iter().any(|(word, coat)| word.is_match(text_) && !coat.is_match(text_));
    if !said && !bare {
        return false;
    }
    let up = WEIGHT_UP_RES.iter().any(|r| r.is_match(text_));
    let asking = _ASKING.iter().any(|w| text_.contains(w.as_str()));
    !(up && asking)
}

/// `(understood.get("norm_text") or understood.get("clean") or "").lower()`.
pub fn _said(understood: &Understood) -> String {
    norm_or_clean_lower(understood)
}

inline_re!(A_HORSE_OR_PONY, r"\b(horse|horses|pony|ponies|donkey|donkeys|shettie|shetties|equine)\b");

/// The fat question about an animal whose weight card is not the dog's.
pub fn _small_pet_fat(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = _said(understood);
    if A_HORSE_OR_PONY.is_match(&text_) {
        return Some(_PONY_FAT.clone());
    }
    if A_RABBIT_WORD.is_match(&text_) {
        return Some(_RABBIT_FAT.clone());
    }
    Some(_RODENT_FAT.clone())
}

static WEIGH_WORDS_RES: LazyLock<Vec<Re>> = LazyLock::new(|| front_bounded_res(&_WEIGH_WORDS));
static WEIGH_DIRECTION_RES: LazyLock<Vec<Re>> = LazyLock::new(|| front_bounded_res(&_WEIGH_DIRECTION));

/// What should it weigh, rather than what to do about its weight.
pub fn _a_weigh_question(text_: &str) -> bool {
    if !WEIGH_WORDS_RES.iter().any(|r| r.is_match(text_)) {
        return false;
    }
    !WEIGH_DIRECTION_RES.iter().any(|r| r.is_match(text_))
}

/// A breed weight table (`CAT_KG`, `DOG_KG`): name, low kg, high kg, longest name first, with
/// each name's `\bname\b` test.
type KgTable = Vec<(String, f64, f64, Re)>;

fn kg_table(name: &str) -> KgTable {
    let mut rows: Vec<(String, f64, f64, Re)> = g(name)
        .pairs()
        .iter()
        .map(|(k, v)| {
            let name = k.str().to_string();
            let re = Re::new(&format!(r"\b{}\b", sh::escape(&name)), 0);
            (name, v.items()[0].as_f64().unwrap_or(0.0), v.items()[1].as_f64().unwrap_or(0.0), re)
        })
        .collect();
    // sorted(table, key=len, reverse=True): stable, so equal lengths keep the table's order
    rows.sort_by(|a, b| b.0.chars().count().cmp(&a.0.chars().count()));
    rows
}

static CAT_KG: LazyLock<KgTable> = LazyLock::new(|| kg_table("CAT_KG"));
static DOG_KG: LazyLock<KgTable> = LazyLock::new(|| kg_table("DOG_KG"));

/// The longest breed name the line says a word for, so "jack russell" beats "terrier".
pub fn _the_range<'a>(text_: &str, table: &'a KgTable) -> Option<(&'a str, f64, f64)> {
    table.iter().find(|(_, _, _, re)| re.is_match(text_)).map(|(name, low, high, _)| (name.as_str(), *low, *high))
}

inline_re!(A_YOUNG_ONE, r"\b(kitten|kittens|puppy|pup|puppies)\b");

/// What a cat or dog should weigh: the young one's curve, the breed's range, or the bands.
pub fn _weigh_reply(text_: &str, table: &KgTable, bands: &str, adult: &str) -> String {
    if A_YOUNG_ONE.is_match(text_) {
        return format!(
            "Weigh it weekly on the kitchen scales and write the figures down, because the \
number that matters for a young animal is that it is going up, not where it is \
on a chart: a puppy roughly doubles its birth weight in the first ten days and \
gains most before it is six months old, and a kitten gains about 100g a week. \
A grown one is {bands}. A vet can say whether yours is on its own curve at the next \
jab, and the line going flat or down is worth a call before then."
        );
    }
    if let Some((name, low, high)) = _the_range(text_, table) {
        return format!(
            "An adult {name} is usually {} to {}kg, males at the top of that range and females at \
the bottom, and a line bred for work or for show can sit outside it. Go by hand \
as well as by figure: you should feel the ribs easily under a thin cover and see \
a waist from above, which is what the vet scores. A young one is not that weight \
until it is full grown, at {adult}, so weigh it monthly and hold the food steady \
while it is still filling out.",
            fmt_g(low),
            fmt_g(high)
        );
    }
    format!(
        "It depends on the breed and the bands are wide: {bands}. What you can check at home is the \
shape - ribs felt easily under a thin cover, a waist seen from above - and the vet \
scores the same thing by hand. Weigh it monthly on the same scales, so a slow change \
shows up before it is a problem."
    )
}

/// A cat's weight: what it should weigh, a cat going thin, or the slimming plan.
pub fn _cat_weight(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = _said(understood);
    if _a_weigh_question(&text_) {
        return Some(_weigh_reply(
            &text_,
            &CAT_KG,
            "3.5 to 5.5kg for most cats, and 5 to 9kg for a big breed like a Maine Coon",
            "about a year",
        ));
    }
    if _going_thin(&text_) {
        return Some(_CAT_THIN_REPLY.clone());
    }
    Some(
        "Cats get overweight on very little food, so weigh the dry food instead of filling \
the bowl and cut it by a tenth: a 4kg cat needs about 3 or 4 pouches of 85g, or 40 \
to 50g of dry, a day, and should lose about 1 per cent of body weight a week, never \
fast. Split the day into small meals, use a puzzle feeder and a wand toy for five \
minutes twice a day, and ask the vet for a target weight - a cat gaining on the same \
food needs a thyroid check."
            .into(),
    )
}

/// A dog's weight: what it should weigh, a dog going thin, or the slimming plan.
pub fn _dog_weight(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = _said(understood);
    if _a_weigh_question(&text_) {
        return Some(_weigh_reply(
            &text_,
            &DOG_KG,
            "2 to 10kg for a small dog, 10 to 25kg for a medium one, 25 to 40kg for a large one and 40kg and up for a giant",
            "a year for a small dog and eighteen months for a large one",
        ));
    }
    if _going_thin(&text_) {
        return Some(_DOG_THIN_REPLY.clone());
    }
    Some(
        "You should be able to feel the ribs without pressing and see a waist from above; if \
you can't, cut treats first, weigh the food instead of scooping it, and swap some of \
the kibble for green beans or carrot. Ask the vet for a target weight, since a dog \
that loses 10% of its body weight is often livelier and longer-lived for it."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Chewing and biting.

/// `has(*wanted)` in `_dog_chewing` / `_dog_bite_person`: whether one of the phrases is in the
/// line's padded words.
#[allow(non_snake_case)]
pub fn _dog_chewing__has<S: AsRef<str>>(said: &str, wanted: &[S]) -> bool {
    wanted.iter().any(|w| said.contains(&format!(" {} ", w.as_ref())))
}

/// `_dog_bite_person.<locals>.has`: the same test as `_dog_chewing`'s.
#[allow(non_snake_case)]
pub fn _dog_bite_person__has<S: AsRef<str>>(said: &str, wanted: &[S]) -> bool {
    _dog_chewing__has(said, wanted)
}

/// The padded, lower-cased words of the line, as `has` reads them.
fn padded_lower_words(understood: &Understood) -> String {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    format!(" {} ", words.join(" "))
}

/// Why a dog chews, which is a puppy question only while the dog is a puppy.
pub fn _dog_chewing(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = padded_lower_words(understood);
    let has = |wanted: &[&str]| _dog_chewing__has(&said, wanted);
    let puppy = "Chewing is normal while teeth change, so redirect it: plenty of chew toys, a \
frozen carrot or stuffed Kong, and pick everything else up out of reach. Puppies \
usually grow out of it by seven or eight months.";
    let adult = "An adult dog chews for a reason and teeth are not it: boredom, a walk that is too \
short, or being left alone longer than it can take. Give it a proper sniffy walk \
before you go out, leave a stuffed Kong or a long-lasting chew, and block off the \
sofa for a fortnight while the habit breaks. If it has just started in a dog that \
never did it, get its teeth and gums checked, because a sore mouth makes a dog chew \
to soothe it.";
    if let Some(years) = _dog_age_years(understood) {
        return Some(if years < 1.0 { puppy } else { adult }.to_string());
    }
    if has(&[
        "puppy", "puppies", "pup", "pups", "teething", "teeth", "months old", "month old", "weeks old", "week old",
        "just got him", "just got her", "just got it", "new puppy", "young dog",
    ]) {
        return Some(puppy.into());
    }
    Some(adult.into())
}

/// How to stop a dog biting people, at the weight the line carries.
pub fn _dog_bite_person(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = padded_lower_words(understood);
    let has = |wanted: &[&str]| _dog_bite_person__has(&said, wanted);
    let has_list = |wanted: &[String]| _dog_bite_person__has(&said, wanted);
    if has(&["his", "her", "its", "their"])
        && has(&["paw", "paws", "tail", "leg", "legs", "foot", "feet", "ear", "ears", "fur", "skin"])
    {
        return None;
    }
    if has(&["himself", "herself", "itself"]) && has(&["scratching", "scratches", "scratch", "itching", "itches", "itchy"]) {
        return None;
    }
    if has(&[
        "fence", "fences", "lead", "leads", "sofa", "toy", "toys", "bed", "cushion", "blanket", "cage", "nappy", "slipper",
        "slippers", "shoe", "furniture", "wall", "stick", "sticks", "skirting",
    ]) {
        return None;
    }
    if has_list(&BITE_NOW) {
        return Some(
            "Call 999 or 911 now, and don't put yourself in the way: shout and use a barrier \
- a door, a chair, a bin - not your arms, and get children and the dog into \
separate rooms. A dog that has hold of someone and will not let go is a police \
job too, and 101 takes the report once everyone is safe."
                .into(),
        );
    }
    if has_list(&BITE_TEETH) && !has_list(&BITE_SKIN) && !has_list(&BITE_CHILD_WORDS) {
        return Some(
            "Yelp or say \"ow\" and stop the game the moment teeth touch skin, then turn away \
for ten seconds before playing again; the puppy learns that teeth end the fun. \
Give a chew toy instead every time he goes for hands, and if he is over-tired, a \
nap settles more of it than any training."
                .into(),
        );
    }
    if has_list(&BITE_CHILD_WORDS) && has_list(&BITE_DONE_DEED) {
        return Some(BITE_REPORT_CHILD.clone());
    }
    if has_list(&BITE_SKIN)
        || (has_list(&BITE_SOMEONE_ELSE) && !has_list(&BITE_CHILD_WORDS) && has(&["bit", "bitten", "nipped"]))
    {
        return Some(BITE_REPORT.clone());
    }
    let mut advice = String::from(
        "Stop the biting by changing what happens before it: keep him away from the \
situations that set it off - a baby gate or a lead indoors is not a failure - \
muzzle-train him for the vet and the walk, and book the vet to rule out pain, a bad \
ear or bad eyesight. Punishing him after the event teaches nothing, and a \
qualified behaviourist is worth the money for a dog that has bitten once.",
    );
    if has_list(&BITE_OTHERS_HABIT) {
        let (who, alone) = if has_list(&BITE_CHILD_WORDS) {
            ("the children", ", and never leave them alone together")
        } else if has_list(&BITE_DOOR_WORDS) {
            ("anyone coming to the door", "")
        } else {
            ("visitors", "")
        };
        advice += &format!(" Until someone has seen him, keep a shut door or a gate between him and {who}{alone}.");
    }
    Some(advice)
}

// ---------------------------------------------------------------------------------------------
// Leftovers, chocolate, and an animal that keeps being sick.

/// Whether the line is a dog or a cat in the bin, which has a card of its own.
pub fn _the_bin_is_the_dogs_question(understood: &Understood) -> bool {
    let text_ = understood.clean.to_lowercase();
    _A_BIN.is_match(&text_) && _A_DOG_OR_CAT.is_match(&text_)
}

static _SMALL_PET_ATE_LEFTOVERS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_SMALL_PET_ATE_LEFTOVERS"));
static _SMALL_PET_DIET_OR_PLATE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_SMALL_PET_DIET_OR_PLATE"));

/// The plate answer, or the what-to-do-now answer when the animal has already eaten.
pub fn _leftovers_answer(_memory: &Memory, understood: &Understood) -> Option<String> {
    let pet = _pet_words(understood);
    let text_ = understood.clean.to_lowercase();
    let words = az_words(&text_);
    let eaten = LEFT_OVERS_EATEN.iter().any(|w| words.contains(w.as_str()))
        || LEFT_OVERS_EATEN_PHRASES.iter().any(|p| text_.contains(p.as_str()));
    let kind = if A_RABBIT_WORD_NC.is_match(&text_) { "rabbit" } else { _small_pet_kind(&text_) };
    if let Some(ate) = lookup(&_SMALL_PET_ATE_LEFTOVERS, kind) {
        if !eaten {
            return lookup(&_SMALL_PET_DIET_OR_PLATE, kind);
        }
        return Some(ate);
    }
    if !eaten {
        return Some(PLATE_LEFTOVERS_TEXT.clone());
    }
    py_format(&ATE_LEFTOVERS_TEXT, &[("kind", pet.kind), ("they", pet.they)])
}

inline_re!(A_RABBIT_WORD_NC, r"\b(?:rabbit|rabbits|bunny|bunnies)\b");

/// `STANDS_ASIDE`: every card in this file stands aside for a line that reports the person's
/// own routine with the animal ("i walk the dog every day").
#[allow(non_snake_case)]
pub fn STANDS_ASIDE(understood: &Understood) -> bool {
    !understood.routine
}

/// The poison card for the animal that ate the chocolate.
pub fn _chocolate_for(animal: Option<&str>) -> String {
    match animal {
        Some("cat") => "Yes, chocolate is poisonous to cats as well as dogs: the darker the chocolate, \
the worse it is, and a cat is small enough that a couple of squares matters. Ring \
your vet with the cat's weight and roughly how much and what kind of chocolate it \
ate, and don't wait for symptoms - the first signs are being sick, a racing heart \
and restlessness."
            .into(),
        Some(a) if !a.is_empty() && a != "dog" => format!(
            "Yes, chocolate is poisonous to {a}s: the darker the chocolate, the \
worse it is. Ring your vet with its weight and roughly how much and what kind of \
chocolate it ate, and don't wait for symptoms."
        ),
        _ => "Yes, chocolate is poisonous to dogs: the darker the chocolate, the worse it is. Ring \
your vet with the dog's weight and roughly how much and what kind of chocolate it \
ate; don't wait for symptoms."
            .into(),
    }
}

/// The animal named in the line, as "cat", "dog", "rabbit" and so on, or None.
pub fn _the_animal_that_ate(understood: &Understood) -> Option<&'static str> {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let joined = words.join(" ");
    let text_ = if joined.is_empty() { understood.clean.to_lowercase() } else { joined };
    let split = text::split_ref(&text_);
    let padded = format!(" {text_} ");
    const ANIMALS: [(&str, &str); 14] = [
        ("kitten", "cat"),
        ("kittens", "cat"),
        ("cat", "cat"),
        ("cats", "cat"),
        ("puppy", "dog"),
        ("pup", "dog"),
        ("dog", "dog"),
        ("dogs", "dog"),
        ("rabbit", "rabbit"),
        ("bunny", "rabbit"),
        ("hamster", "hamster"),
        ("guinea pig", "guinea pig"),
        ("parrot", "parrot"),
        ("budgie", "budgie"),
    ];
    for (word, animal) in ANIMALS {
        if split.contains(&word) || padded.contains(&format!(" {word} ")) {
            return Some(animal);
        }
    }
    None
}

static _A_SMALL_ANIMAL: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("_A_SMALL_ANIMAL").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});

/// The small animal named in `text`, as the person would say it, or None.
pub fn _the_small_animal(text_: &str) -> Option<String> {
    let said = format!(" {} ", text_.to_lowercase());
    let mut best: Option<(&str, usize)> = None;
    for (word, noun) in _A_SMALL_ANIMAL.iter() {
        let Some(at) = said.find(&format!(" {word} ")) else { continue };
        if best.map(|(_, b)| at < b).unwrap_or(true) {
            best = Some((noun.as_str(), at));
        }
    }
    best.map(|(noun, _)| noun.to_string())
}

/// The card for a small animal that keeps being sick.
pub fn _small_animal_sick(animal: &str) -> String {
    format!(
        "A {animal} that keeps being sick needs the vet today rather than a \
wait-and-see: something that small can be flat and dehydrated within a day, and the \
sickness is usually the teeth, an ulcer or something it should not have eaten. Put \
water in a shallow bowl as well as the bottle, keep it warm and quiet, offer its usual \
food or wet food, and weigh it - a small animal that has lost weight is a vet call even \
if it is still nibbling."
    )
}

/// Whether the line is being read as the pets talk, or names an animal itself.
pub fn _this_is_the_pets_talk(understood: &Understood) -> bool {
    if understood.topics.iter().any(|t| t == "pets") || understood.topic.as_deref() == Some("pets") {
        return true;
    }
    let lowered = format!(" {} ", understood.clean.to_lowercase());
    _a_word_in_list(&az_words(&lowered), &_PET_SUBJECT)
}

/// `bool(found & set(words))` for a list of words.
fn _a_word_in_list<S: AsRef<str>>(found: &HashSet<&str>, words: &[S]) -> bool {
    words.iter().any(|w| found.contains(w.as_ref()))
}

/// Whether one of `words` is in `text` as a whole word.
pub fn _a_word_in<S: AsRef<str>>(text_: &str, words: &[S]) -> bool {
    let padded = format!(" {text_} ");
    _a_word_in_list(&az_words(&padded), words)
}

/// Whether the line names a person and no animal, so the vet is not who is being asked.
pub fn _a_person_who_is_not_the_patient(understood: &Understood) -> bool {
    let said = text::squash(&clean_or_text_lower(understood));
    if _a_word_in(&said, &_PET_SUBJECT) || _the_small_animal(&said).is_some() {
        return false;
    }
    if _a_word_in(&said, &_A_PERSON_NAMED) {
        return true;
    }
    _A_SPEAKER_IS_ILL.iter().any(|p| said.contains(p.as_str()))
}

/// The vet decision for an animal, and the doctor for a person with no animal in the line.
pub fn _the_vet_question(_memory: &Memory, understood: &Understood) -> Option<String> {
    if _a_person_who_is_not_the_patient(understood) {
        return Some(_A_VET_IS_FOR_ANIMALS.clone());
    }
    Some(_WHEN_A_VET_IS_NEEDED.clone())
}

/// The cat or dog card for an animal that keeps being sick, or None for anybody else.
pub fn _pet_sick_again(memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = text::squash(&clean_or_text_lower(understood));
    let padded = format!(" {clean} ");
    let here = az_words(&padded);
    let cat_here = ["cat", "cats", "kitten", "kittens"].iter().any(|w| here.contains(w));
    let dog_here = ["dog", "dogs", "puppy", "puppies", "pup", "pups"].iter().any(|w| here.contains(w));
    let rabbit_here = _a_word_in_list(&here, &_A_RABBIT);
    let small_here = _the_small_animal(&clean);
    if let Some(small) = &small_here {
        if !(cat_here || dog_here || rabbit_here) {
            return Some(_small_animal_sick(small));
        }
    }
    if rabbit_here && !(cat_here || dog_here) {
        return Some(_RABBIT_SICK_REPLY.clone());
    }
    if cat_here || dog_here {
        return Some(if cat_here && !dog_here { _CAT_SICK_REPLY.clone() } else { _DOG_SICK_REPLY.clone() });
    }
    let mut earlier = String::new();
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(4)..] {
        let said = text::squash(&turn.user.to_lowercase());
        if !said.is_empty() && said != clean {
            earlier.push(' ');
            earlier.push_str(&said);
        }
    }
    if text::strip(&earlier).is_empty() {
        return None;
    }
    if _a_word_in(&earlier, &_A_PERSON) && !_a_word_in(&earlier, &_PET_SUBJECT) {
        return None;
    }
    if !(_a_word_in(&earlier, &_PET_SUBJECT) || _the_small_animal(&earlier).is_some()) {
        return None;
    }
    if _a_word_in(&earlier, &_NOT_A_CAT_OR_DOG)
        && !_a_word_in(&earlier, &["cat", "cats", "kitten", "kittens", "dog", "dogs", "puppy", "puppies"])
    {
        return None;
    }
    let cat_named = _a_word_in(&earlier, &["cat", "cats", "kitten", "kittens"]);
    let dog_named = _a_word_in(&earlier, &["dog", "dogs", "puppy", "puppies", "pup", "pups"]);
    if _a_word_in(&earlier, &_A_RABBIT) && !(cat_named || dog_named) {
        return Some(_RABBIT_SICK_REPLY.clone());
    }
    if !(cat_named || dog_named) {
        if let Some(small) = _the_small_animal(&earlier) {
            return Some(_small_animal_sick(&small));
        }
    }
    if cat_named && !dog_named {
        return Some(_CAT_SICK_REPLY.clone());
    }
    if dog_named && !cat_named {
        return Some(_DOG_SICK_REPLY.clone());
    }
    Some(format!("{} The same goes for a dog that keeps being sick.", *_CAT_SICK_REPLY))
}

// ---------------------------------------------------------------------------------------------
// Kit, the fish tank, rescue or baby, fur loss, the hot car and the lost pet.

/// True when the line is the question to buy, False when it is the news that one is coming.
pub fn _shopping_or_news(understood: &Understood) -> bool {
    _SHOPPING_WORDS.iter().any(|w| understood.words.iter().any(|x| x == w))
}

/// The dog's kit: the shopping list, or a note back on the news.
pub fn _dog_kit(_memory: &Memory, understood: &Understood) -> Option<String> {
    Some(if _shopping_or_news(understood) { _DOG_KIT_LIST.clone() } else { _DOG_KIT_NEWS.clone() })
}

/// The puppy's kit: the shopping list, or a note back on the news.
pub fn _puppy_kit(_memory: &Memory, understood: &Understood) -> Option<String> {
    Some(if _shopping_or_news(understood) { _PUPPY_KIT_LIST.clone() } else { _PUPPY_KIT_NEWS.clone() })
}

/// The weekly water change, or the impeller when the filter is the thing making the noise.
pub fn _fish_tank(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = format!(" {} ", understood.clean.to_lowercase());
    if [" noisy", " noise", " rattl", " humming", " buzzing", " loud"].iter().any(|w| line.contains(w)) {
        return Some(
            "A filter that rattles or hums is usually air in the housing or grit in the \
impeller: unplug it, lift the impeller out and rinse it, and top the tank up so \
the water is above the intake. If it still grinds the impeller is worn, and that \
is a cheap part rather than a new filter."
                .into(),
        );
    }
    Some(
        "Change about a quarter of the water once a week, and rinse the filter sponge in the \
water you take out, never under the tap. A full clean strips the good bacteria and \
does far more harm than the dirt."
            .into(),
    )
}

/// The rescue-versus-baby pick, in the animal the line names.
pub fn _rescue_or_baby(_memory: &Memory, understood: &Understood) -> Option<String> {
    let lowered = understood.clean.to_lowercase();
    let words = az_words(&lowered);
    if ["cat", "cats", "kitten", "kittens", "kitty"].iter().any(|w| words.contains(w)) {
        return Some(
            "A rescue cat comes with a known character, is past the kitten climbing and \
biting, and settles in a week or two; a kitten is months of work but you shape \
it from the start. Out at work all day, or a first cat: the adult rescue is the \
kinder start."
                .into(),
        );
    }
    Some(
        "A rescue adult usually comes with a known character, is past the chewing, and settles \
in a week or two; a puppy is a year of work but you shape everything from the start. \
First dog, or a busy household: the rescue is the kinder start."
            .into(),
    )
}

inline_re!(FUR_GUINEA_PIG, r"\b(?:guinea ?pigs?)\b");
inline_re!(FUR_HAMSTER, r"\b(?:hamsters?)\b");
inline_re!(FUR_RODENT, r"\b(?:gerbils?|chinchillas?|degus?|rats?|mice|mouse)\b");
inline_re!(FUR_BIRD, r"\b(?:budgies?|budgerigars?|cockatiels?|canar(?:y|ies))\b");
inline_re!(FUR_HORSE, r"\b(?:horses?|pony|ponies)\b");

/// A bald or itchy patch on a small animal, answered for the animal the line names.
pub fn _small_fur_loss(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if !SMALL_FUR.is_match(&clean) {
        return None;
    }
    if BIRD_FUR.is_match(&clean) && BIRD_PLUCKING.is_match(&clean) {
        return None;
    }
    if A_RABBIT_WORD_NC.is_match(&clean) {
        return Some(
            "Bald patches on a rabbit are usually fur mites - the flaky white dandruff that \
crawls is the giveaway - ringworm, which is round and scaly, or a normal moult, \
which comes out in tufts and leaves patchy fur for a week or two. A vet treats \
mites with a spot-on made for rabbits, and every rabbit in contact needs it, \
since dog and cat flea products can kill a rabbit. A doe pulling fur from her \
dewlap is building a nest."
                .into(),
        );
    }
    if FUR_GUINEA_PIG.is_match(&clean) {
        return Some(
            "Hair loss in a guinea pig is usually mites, which they carry and then suffer from \
under stress, or ringworm: both need a vet, and every pig in the hutch treated. \
In an older sow, bald flanks point at ovarian cysts. Guinea pigs cannot make \
their own vitamin C, so a diet without it shows in a poor coat too."
                .into(),
        );
    }
    if FUR_HAMSTER.is_match(&clean) {
        return Some(
            "A bald patch on a hamster is often normal - the scent gland on the belly wears \
bare - but a patch that spreads, itches or has flakes on the skin is mites, \
ringworm or an over-grooming cagemate, and it needs a vet who treats small \
furries. Check the bedding and the wheel for rubbing, and never use a dog or \
cat mite product on it."
                .into(),
        );
    }
    if FUR_RODENT.is_match(&clean) {
        return Some(
            "Mites, ringworm and a cagemate over-grooming it are the usual causes of a bald \
patch, scabs or flaky skin on a rat, gerbil, chinchilla or mouse: hair thin \
along the back and shoulders point at mites or ringworm, and bald patches on \
the head and shoulders of the others in the cage point at barbering. Both need \
a vet who treats small furries, and never use a dog or cat mite product on it."
                .into(),
        );
    }
    if FUR_BIRD.is_match(&clean) {
        return Some(
            "Feather loss in a bird is usually a moult, a companion pulling the feathers out, or \
mites, and a moult leaves the wing and tail feathers even. A bare patch with \
sore or flaky skin, or a bird plucking itself, needs an avian vet - the \
condition is often boredom, a bad diet or a liver problem rather than mites."
                .into(),
        );
    }
    if FUR_HORSE.is_match(&clean) {
        return Some(
            "A bald patch on a horse is usually a rub: the saddle, a rug or a tight browband \
wearing the hair off where the edges sit, and it grows back once the tack fits. \
The other causes are ringworm, which is round and scaly with hair standing up \
around the edge, rain scald and lice along the back, and sweet itch where the \
mane and tail meet the skin at the tail base. Ringworm and lice need the vet or \
an SQP at the feed merchant, and ringworm spreads to people, so wash your hands."
                .into(),
        );
    }
    Some(
        "A bald or itchy patch on a small animal is usually mites, ringworm or over-grooming, \
and all three need a vet who treats that species. Check the skin for white flakes or \
ringworm's round scaly edge, and never use a dog or cat flea or mite product on a \
rabbit, guinea pig, hamster or bird."
            .into(),
    )
}

/// First aid for a dog already in a hot car, the rule for one that might be left there.
pub fn _dog_in_a_car(_memory: &Memory, reading: &Understood) -> Option<String> {
    let said = format!(" {} ", clean_or_text_lower(reading));
    if _DOG_COOL.iter().any(|p| said.contains(&format!(" {p} "))) {
        return Some(_DOG_CAR_FIRST_AID.clone());
    }
    Some(_DOG_CAR_RULE.clone())
}

/// The lost-pet routine, said about the animal the line named ("" for keeping one in).
pub fn _a_lost_pet(_memory: &Memory, understood: &Understood) -> Option<String> {
    if _STOPPING_A_PET_GETTING_OUT.is_match(&understood.clean.to_lowercase()) {
        return Some(String::new());
    }
    let words = format!(" {} ", understood.words.join(" ")).to_lowercase();
    let pick = |word: &str| if word == "kitten" { "cat".to_string() } else { word.to_string() };
    let mut noun = _A_LOST_ANIMAL.iter().find(|w| words.contains(&format!(" {w} "))).map(|w| pick(w));
    if noun.is_none() {
        let text_ = format!(" {} ", understood.clean.to_lowercase());
        noun = _A_LOST_ANIMAL.iter().find(|w| text_.contains(&format!(" {w} "))).map(|w| pick(w));
    }
    let noun = noun.unwrap_or_else(|| "pet".to_string());
    Some(format!(
        "Ring the microchip register and the council's animal warden to report the {noun} \
missing, then a photo with the last place seen on the local Facebook and Nextdoor \
pages and a call to the vets within five miles. Walk the streets calling calmly, \
since a frightened {noun} hides close by, and leave something that smells of \
home and water at the door. Never chase it: kneel down and let it come to you."
    ))
}

/// "dog", "cat" or None for the animal the line - or the chat it carries on - is about.
pub fn _the_animal_in_the_chat(memory: &Memory, understood: &Understood) -> Option<&'static str> {
    let said: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let dogs = said.iter().any(|w| _DOG_SLEEP_WORDS.contains(w));
    let cats = said.iter().any(|w| _CAT_SLEEP_WORDS.contains(w));
    if dogs != cats {
        return Some(if dogs { "dog" } else { "cat" });
    }
    if dogs && cats {
        return None;
    }
    let earlier = dup::topics_context(memory, 2)?.iter().map(|t| t.to_lowercase()).collect::<Vec<_>>().join(" ");
    let heard = az_words(&earlier);
    let dogs = meets(&heard, &_DOG_SLEEP_WORDS);
    let cats = meets(&heard, &_CAT_SLEEP_WORDS);
    if dogs != cats {
        return Some(if dogs { "dog" } else { "cat" });
    }
    None
}

/// The sleeping place of the animal the person has.
pub fn _where_the_pet_sleeps(memory: &Memory, understood: &Understood) -> Option<String> {
    Some(match _the_animal_in_the_chat(memory, understood) {
        Some("dog") => _WHERE_THE_DOG_SLEEPS.clone(),
        Some("cat") => _WHERE_THE_CAT_SLEEPS.clone(),
        _ => _WHERE_EITHER_SLEEPS.clone(),
    })
}

/// False only for a small animal whose mouthful was something other than food.
pub fn _eaten_thing_is_not_food(understood: &Understood) -> bool {
    let text_ = sh::clean_or_text(understood);
    if !_SMALL_ANIMAL.is_match(text_) {
        return true;
    }
    _NOT_FOOD.is_match(text_)
}

/// False for a line that only says what happened, or that leans on the turn before.
pub fn _it_asks_what_to_do(understood: &Understood) -> bool {
    // LEANS_ON: Python reads `understood["leans_on"]` (an earlier reading) here; it is always
    // None on a single-turn reading.
    if understood.leans_on.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
        return false;
    }
    let said = sh::text_or_clean(understood).to_lowercase().replace('\u{2019}', "'");
    if !_CARE_OF_A_NEW_PET.is_match(&said) {
        return false;
    }
    if _A_SHOPPING_ASK.is_match(&said) {
        return false;
    }
    !understood.asks.is_empty()
}

// ---------------------------------------------------------------------------------------------
// Boredom, names, a pet off its food, coats and habits.

static ANIMAL_RE: LazyLock<Re> = LazyLock::new(|| Re::new(&ANIMAL, 0));

/// Which animal this line is about: the one it names, otherwise the one the chat is on.
pub fn _pet_kind(memory: &Memory, understood: &Understood) -> String {
    if ANIMAL_RE.is_match(&understood.clean) {
        return _pet_words(understood).kind.to_string();
    }
    if let Some(user) = memory.extra.get("user") {
        if let Some(pets) = user.get("pets") {
            for one in pets.items() {
                let animal = text::strip(one.get("animal").map(Value::str).unwrap_or("")).to_lowercase();
                if animal == "cat" || animal == "dog" || animal == "rabbit" {
                    return animal;
                }
            }
        }
    }
    _pet_words(understood).kind.to_string()
}

inline_re!(LEFT_ALONE, r"\b(work|working|left|alone|out)\b");

/// What to do about a bored pet, in the animal's own terms.
pub fn _bored_pet(memory: &Memory, understood: &Understood) -> Option<String> {
    if _other_pet(understood) {
        return None;
    }
    let kind = _pet_kind(memory, understood);
    if kind == "cat" {
        return Some(
            "Play with a wand toy for five or ten minutes, two or three times a day, and let \
her stalk and pounce before you let her catch it, then give her a meal so the \
hunt has an end. The rest of the day wants things rather than toys: a food \
puzzle, a cardboard box, a paper bag, somewhere high to sit and a window to \
watch. Rotate the toys and put half of them away, because a basket she has seen \
for months is furniture."
                .into(),
        );
    }
    if kind == "rabbit" {
        return Some(
            "A rabbit is bored when it has nothing to chew and nowhere to dig: give it a run \
of a few hours a day outside a hutch, a box of hay to root through, willow or \
apple sticks to chew and something to hide under. A lone rabbit wants a rabbit \
friend more than it wants toys, and the two are happiest neutered and introduced \
slowly."
                .into(),
        );
    }
    if LEFT_ALONE.is_match(&understood.clean) {
        return Some(
            "Four hours at a time is about the usual comfortable limit for an adult dog, so \
build the time up in short chunks and have someone look in at lunchtime for a \
puppy. Then fill the day: half his dinner scattered on the \
lawn or in a snuffle mat, something legal to chew, the radio on, and a walk where \
he is allowed to sniff when you get back. A dog left with nothing to do chews \
the furniture and paces, and that is boredom rather than badness."
                .into(),
        );
    }
    Some(
        "A dog is bored because the day is short on sniffing and chewing, not because he \
needs more toys. One walk where he is allowed to stop and sniff does more than a \
long march, and a scatter of his dinner on the lawn or in a snuffle mat takes ten \
minutes off the clock. Add five minutes of training a day, something legal to chew \
and a game of tug, and break the day up rather than saving it all for the evening."
            .into(),
    )
}

/// Whether a pet knows its name: a dog learns the word, and a cat does too, but answers less.
pub fn _knows_name(memory: &Memory, understood: &Understood) -> Option<String> {
    if _other_pet(understood) {
        return None;
    }
    let kind = _pet_kind(memory, understood);
    if kind == "cat" {
        return Some(
            "Yes - a cat can pick her own name out of a list of similar words, so she knows it \
about as well as a dog does; what she does not do is come running when she hears \
it. Say it before the food or the wand toy rather than after, and she will start \
looking up when you use it. A cat that stops answering her name may have gone \
deaf, which a vet can check, or may just have decided the sofa is better."
                .into(),
        );
    }
    if kind == "rabbit" {
        return Some(
            "A rabbit learns its name as a sound that means food, not the way a dog does: say \
it just before you put the bowl down or open the run, and it will start turning \
towards you. They are prey animals and will not usually come when called, and a \
rabbit that ignores its name has not forgotten it."
                .into(),
        );
    }
    Some(
        "Yes - most family dogs understand a few dozen words, and the best of them well over a \
hundred, and their own name is the first. Say the name once and then the cue - \
'Biscuit, sit' - and reward, so the name becomes the thing to look at before \
something good happens. Never use it to tell him off: a name that means trouble is a \
name he learns to ignore."
            .into(),
    )
}

/// `any(word in line for word in words)`.
pub fn _off_food_said<S: AsRef<str>>(line: &str, words: &[S]) -> bool {
    words.iter().any(|w| line.contains(w.as_ref()))
}

/// Whether the animal in the line is what someone eats, rather than the one off its food.
pub fn _the_animal_is_eaten(line: &str) -> bool {
    let Some(verb) = _EAT_VERB.search(line) else {
        return false;
    };
    // positions are bytes on both sides of the comparison, which orders them as Python does
    let first = _OFF_FOOD.iter().filter_map(|w| line.find(w.as_str())).min();
    match first {
        Some(at) => at > verb.start(),
        None => false, // min(..., default=-1) > start is never true
    }
}

/// A pet off its food: what to do for a fish, a reptile, a bird, a rodent or a rabbit.
pub fn _off_food_reply(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = clean_or_text_lower(understood);
    if _the_animal_is_eaten(&line) {
        return None;
    }
    if _off_food_said(&line, &_OFF_FOOD_FISH) {
        return Some(
            "Don't feed it and test the water: a fish that stops feeding is nearly always \
ammonia, nitrite or a tank that has gone cold, so use a liquid test kit, change a \
quarter of the water with dechlorinated water at the same temperature and leave \
the feeding for a day or two. A fish that is also gulping at the top or swimming \
on its side is the urgent one."
                .into(),
        );
    }
    if _off_food_said(&line, &_OFF_FOOD_REPTILE) {
        return Some(
            "A reptile that skips a feed is usually nothing: a snake can go weeks, and a \
tortoise off its food in winter or before shedding is normal. What makes it a vet \
visit is weight loss, a soft shell, a runny nose, sunken eyes, or no food for more \
than a week - and check the basking temperature and the UVB bulb first, because a \
cold reptile cannot digest."
                .into(),
        );
    }
    if _off_food_said(&line, &_OFF_FOOD_BIRD) {
        return Some(
            "Ring an avian vet today: a budgie or parrot hides illness until it is very thin, \
so a bird that has taken nothing for 24 hours is an emergency. Keep the cage warm \
and covered, put food and water where the bird is sitting rather than on the \
floor, and offer its usual food warm - not a new one it has never seen."
                .into(),
        );
    }
    if _off_food_said(&line, &_OFF_FOOD_RODENT) {
        return Some(
            "Ring the vet today: a hamster, rat or ferret that stops eating usually has a bad \
front tooth or an illness it has hidden, and 12 hours is already long for \
something that small. Look at the teeth, offer the food it likes best, warm, and \
keep it quiet and warm while you ring rather than waiting to see if it picks up."
                .into(),
        );
    }
    if _off_food_said(&line, &_OFF_FOOD_RABBIT) {
        return Some(
            "Ring the vet today, not tomorrow: 12 hours without food is an emergency for a \
rabbit or guinea pig, because the gut stops moving and will not restart on its \
own. Keep hay and water in front of it, offer warm strong-smelling food and gentle \
exercise, and do not wait to see if it picks up."
                .into(),
        );
    }
    Some(
        "Ring the vet today rather than waiting: anything small that has taken nothing for 24 \
hours is a vet visit, and a rabbit or guinea pig is an emergency at 12. Offer the food \
it likes best, warm and strong-smelling, somewhere quiet, and look for a cause you can \
see - a blocked nose, an overgrown tooth, a new food, a move, a new pet."
            .into(),
    )
}

/// A cat's hair: one that has stopped washing, one shedding, or a bald patch.
pub fn _cat_hair(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if _CAT_NOTWASH.is_match(&text_) {
        return Some(_CAT_NOTWASH_REPLY.clone());
    }
    if _CAT_SHED.is_match(&text_) {
        return Some(_CAT_SHED_REPLY.clone());
    }
    Some(_CAT_BALD_REPLY.clone())
}

/// A cat's habit with its person: kneading, headbutting, or sitting on them.
pub fn _cat_habit(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = format!(" {} ", understood.clean.to_lowercase());
    if _KNEAD_SAID.is_match(&clean) {
        return Some(_KNEAD_REPLY.clone());
    }
    if _HEAD_SAID.is_match(&clean) {
        return Some(_HEADBUTT_REPLY.clone());
    }
    Some(_ON_YOU_REPLY.clone())
}

static _SMALL_PET_ALONE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_SMALL_PET_ALONE"));

/// The companion answer, in the animal the line names or the chat was last about.
pub fn _small_pet_alone(memory: &Memory, understood: &Understood) -> Option<String> {
    lookup(&_SMALL_PET_ALONE, &_the_small_pet(memory, understood))
}

/// The thing being scratched: "door", "floor", "furniture" or "skin", else "".
pub fn _scratching_what(understood: &Understood) -> &'static str {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let text_ = words.join(" ");
    const WHAT: [(&[&str], &str); 4] = [
        (&["door", "doors", "back door", "front door", "bedroom door", "gate"], "door"),
        (
            &["floor", "floors", "carpet", "carpets", "rug", "rugs", "bedding", "bed", "blanket", "cushion", "cushions", "mat", "mattress"],
            "floor",
        ),
        (
            &[
                "sofa", "sofas", "settee", "couch", "armchair", "chair", "chairs", "curtain", "curtains", "furniture", "table",
                "tables", "woodwork", "wallpaper", "wall",
            ],
            "furniture",
        ),
        (
            &[
                "himself", "herself", "themselves", "skin", "raw", "sore", "itchy", "itching", "scratching", "so much",
                "all the time", "constantly", "bleeding", "bald", "patch", "patches",
            ],
            "skin",
        ),
    ];
    for (keys, what) in WHAT {
        for key in keys {
            if words.iter().any(|w| w == key) || text_.contains(key) {
                return what;
            }
        }
    }
    ""
}

static _PET_SCRATCHING: LazyLock<Vec<(String, String)>> = LazyLock::new(|| g_pairs("_PET_SCRATCHING"));

/// The scratching card for the thing being scratched (and a dog's furniture its own).
pub fn _pet_scratching(understood: &Understood) -> Option<String> {
    let what = _scratching_what(understood);
    if what == "furniture" && understood.words.iter().any(|w| _DOG_WORDS.contains(w.to_lowercase().as_str())) {
        return lookup(&_PET_SCRATCHING, "furniture-dog");
    }
    lookup(&_PET_SCRATCHING, what)
}

// ---------------------------------------------------------------------------------------------
// A person hurt by an animal; an animal soaked; a chewed cable.

/// True when a person other than the speaker is the one bitten, scratched or stung.
pub fn _a_person_was_hurt(understood: &Understood) -> bool {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    for (at, word) in words.iter().enumerate() {
        if _HURT_VERBS.contains(word.as_str()) {
            let near = &words[(at + 1).min(words.len())..(at + 5).min(words.len())];
            if near.iter().any(|w| _HURT_PERSON.contains(w.as_str())) {
                return true;
            }
        }
    }
    if _HURT_PASSIVE.is_match(&format!(" {} ", words.join(" "))) {
        return words.iter().any(|w| _HURT_PERSON.contains(w.as_str()));
    }
    false
}

/// The wound care, and then the half that belongs to the person who was hurt.
pub fn _the_person_hurt(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let any_of = |set: &dyn Fn(&str) -> bool| words.iter().any(|w| set(w));
    let child = any_of(&|w| BITE_CHILD_WORDS.iter().any(|c| c == w));
    let dog = any_of(&|w| _DOG_WORDS.contains(w));
    if child {
        if dog {
            return Some(format!("{}{}", *_HURT_WOUND, *BITE_REPORT_CHILD));
        }
        return Some(format!("{}{}", *_HURT_WOUND, *_HURT_ANOTHER_ANIMAL));
    }
    if dog {
        if any_of(&|w| _HURT_STRANGER.contains(w)) {
            return Some(format!("{}{}", *_HURT_WOUND, *BITE_REPORT));
        }
        return Some(format!("{}{}", *_HURT_WOUND, *_HURT_OWN_HOME));
    }
    Some(format!("{}{}", *_HURT_WOUND, *_HURT_ANOTHER_ANIMAL))
}

/// "it smells like a wet dog" describes a smell, so no animal has been in the water.
pub fn _a_smell_is_not_a_soaking(understood: &Understood) -> bool {
    !_SMELL_OF_AN_ANIMAL.is_match(&_said(understood))
}

/// Whether the line says an animal went in the water or is soaked.
pub fn _the_animal_got_wet(understood: &Understood) -> bool {
    let said = _said(understood);
    _AN_ANIMAL_IN_THE_WATER.is_match(&said) || _A_SOAKING.is_match(&said)
}

/// True unless the line is a smell of an animal, or a coat the machine is full of.
pub fn _a_soaking_is_not_a_smell_or_a_hair(understood: &Understood) -> bool {
    let said = _said(understood);
    if _SMELL_OF_AN_ANIMAL.is_match(&said) {
        return false;
    }
    !(_THE_COAT_THE_MACHINE_IS_FULL_OF.is_match(&said) && !_A_SOAKING.is_match(&said))
}

/// The cable card, with the after-a-bite half in front when the bite has already happened.
pub fn _pet_cable(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text_ = format!(" {} ", NOT_A_WORD_CHARACTER.sub(" ", &understood.clean.to_lowercase()));
    let shocked = _PET_SHOCKED.is_match(&text_);
    let bitten_live = _PET_CHEWED.is_match(&text_) && _PET_LIVE.is_match(&text_);
    if shocked || bitten_live {
        return Some(
            "Switch the power off at the wall before you touch it, then ring the vet now: a bite \
through a live cable can stop the heart, and a burnt mouth swells over the next \
hour. Do not pull the cable out of its mouth and do not give it food or water. \
Afterwards, put the cable out of reach - plastic trunking along the skirting, a \
socket cover on the plug, cables behind furniture - and give it a chew toy where \
the chewing happened."
                .into(),
        );
    }
    Some(
        "Put the cable out of reach instead of trusting training: plastic trunking or a cable \
cover along the skirting, clipped to the wall, a socket cover on the plug, and cables \
behind furniture rather than across the floor. Then give the chewing somewhere to go \
- a chew toy or a stuffed Kong for a dog, a catnip toy for a cat, hay and a wooden \
block for a rabbit - and dab a bitter pet spray on the cable for a week. A pet that \
has bitten through a live cable needs the vet, and a chewed plug needs replacing, \
not taping up."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// Shivers, horses, moults, snoring, breathing and sweating.

/// A pet that shivers: a horse, a thin-coated dog, or pain or a fever in a warm room.
pub fn _pet_shivers(understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if _a_word_in(&text_, &_HORSE) {
        return Some(
            "A horse that shivers is cold, in pain or tying up, so rug it, give it hay and \
water and take its temperature: ring the vet today if it is sweating, stiff behind \
or unwilling to move, because tying-up needs a vet and a blood test rather than a \
walk round the field to loosen it off."
                .into(),
        );
    }
    if _a_word_in(&text_, &_THIN_COAT) {
        return Some(
            "A thin-coated dog like a whippet or a greyhound shivers from cold before anything \
else, so a coat indoors, a bed off the floor and a coat for winter walks are what \
settle it. If it is shivering in a warm room, tucked up and off its food, that is \
pain or a fever and it wants the vet today."
                .into(),
        );
    }
    Some(
        "Shivering in a warm room is pain or a fever rather than cold: feel the ears and the \
belly, and a pet that is tucked up, hot or off its food wants the vet today. A small \
or thin-coated one can simply be cold, which a coat and a bed off the floor fix, and a \
pet shivering after a fright settles once it has somewhere to hide."
            .into(),
    )
}

/// A lame horse is a vet visit today, with the foot asked about when the line names it.
pub fn _horse_lame(understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if _a_word_in(&text_, &["hoof", "hooves", "foot", "feet", "foot sole", "sole"]) {
        return Some(
            "A lame horse is a vet out today: keep it in on a deep clean bed with hay and \
water, and do not ride it or trot it up for anyone, because an abscess in the hoof \
and laminitis both get worse with movement. Tell the vet which foot it is, whether \
the hoof feels hot and whether it will put weight on it at all."
                .into(),
        );
    }
    Some(
        "A horse that goes lame wants the vet today rather than a wait and see: an abscess in \
the hoof or laminitis is what it usually is, and both get worse if it is walked on. \
Keep it in on a deep bed with hay and water, don't ride it, and tell the vet which leg \
it is and whether it will put weight on it."
            .into(),
    )
}

/// A horse off its food is colic until the vet says otherwise; off its water, check the water.
pub fn _horse_off_food(understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if _a_word_in(&text_, &["eating", "eat", "eats", "eaten", "food", "feed", "hay", "feed"]) {
        return Some(
            "A horse that stops eating is a colic until the vet says otherwise, so ring them now \
rather than waiting for the morning. Keep it walking gently if it wants to roll, \
take the hard feed away, leave water and hay in reach, and tell the vet how long it \
has been, whether there are droppings and whether it has a temperature."
                .into(),
        );
    }
    Some(
        "Check the water first, because a frozen, dirty or new trough is the commonest reason a \
horse stops drinking: fresh warm water in a clean bucket, and a pinch of salt in the \
feed, often settles it. A horse that still will not drink by the next morning, or that \
is off its hay as well, is a vet call - colic starts that way."
            .into(),
    )
}

/// A moult: a rabbit's needs daily brushing, a dog blowing its coat is normal.
pub fn _pet_moult(understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if _a_word_in(&text_, &["rabbit", "rabbits", "bunny", "bunnies"]) {
        return Some(
            "A rabbit in moult needs brushing every day, and twice a day at the peak, because \
the fur it swallows is what can block the gut: keep the hay coming and watch the \
droppings. Bare patches, or skin that is sore or scurfy, are the vet rather than \
the moult."
                .into(),
        );
    }
    Some(
        "A dog blowing its coat twice a year is normal: brush it daily while it lasts, more if \
the coat is long, and a rubber glove or a damp hand takes the loose hair off the \
furniture. Bare skin, bald patches or constant scratching is a skin question for the \
vet rather than the moult."
            .into(),
    )
}

/// Snoring: the shape of a flat face, or weight and position.
pub fn _pet_snoring(understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if _a_word_in(&text_, &_FLAT_FACE) {
        return Some(
            "Snoring in a flat-faced dog or cat is the shape of the face: the soft palate and the \
narrowed nostrils vibrate, and every extra kilo makes it worse, so keep the weight \
down and ask the vet about the airway at the next check. Ring them sooner if it \
stops breathing for a few seconds and wakes with a snort, if the gums look blue, or \
if it cannot settle at all at night - that is when an operation is talked about."
                .into(),
        );
    }
    Some(
        "Snoring in a dog or cat is usually weight and position: keep the weight down and let it \
sleep with the head a little raised, and a squashed-up sleeper often snores less on a \
flat bed than on a piled one. It is worth the vet if it has started suddenly, if it \
stops breathing for a few seconds and wakes with a snort, or if the animal is sleepy \
all day."
            .into(),
    )
}

/// The breathing row, or None for a person's own breath.
pub fn _pet_breathing(understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if _NOT_A_PERSON_BREATHING.iter().any(|w| text_.contains(w.as_str())) {
        return None;
    }
    if !_a_word_in(&text_, &_PET_ANIMAL) {
        return None;
    }
    if text_.contains("asthma") || text_.contains("inhaler") {
        return None;
    }
    if _BEEN_ACTIVE.iter().any(|w| text_.contains(w.as_str())) {
        return _PET_BREATHING_REPLY.get(1).cloned();
    }
    _PET_BREATHING_REPLY.first().cloned()
}

/// A horse sweating: at rest it is pain until the vet says otherwise; after work, cool it off.
pub fn _horse_sweating(understood: &Understood) -> Option<String> {
    let text_ = understood.clean.to_lowercase();
    if !_HORSE.iter().any(|w| text_.contains(w.as_str())) {
        return None;
    }
    if _HORSE_BEEN_WORKING.iter().any(|p| text_.contains(p.as_str())) {
        return _SWEATING_REPLY.get(1).cloned();
    }
    _SWEATING_REPLY.first().cloned()
}

/// The animal a back-legs or wobbly answer is said about: `{one}`, `{One}`, `{they}`, `{them}`.
#[derive(Clone, Debug, PartialEq)]
pub struct Animal {
    pub one: String,
    pub one_cap: String,
    pub they: String,
    pub them: String,
    pub kind: String,
}

impl Animal {
    /// `template.format(**pet)`.
    fn fill(&self, template: &str) -> Option<String> {
        py_format(
            template,
            &[("one", &self.one), ("One", &self.one_cap), ("they", &self.they), ("them", &self.them), ("kind", &self.kind)],
        )
    }
}

/// The animal the line names, as {one, they, kind}, or the dog these answers default to.
pub fn _the_animal(understood: &Understood) -> Animal {
    let text_ = format!(" {} ", understood.clean.to_lowercase());
    let pointed_at = understood.animal_in_hand.to_lowercase();
    for (kind, words) in _ANI_KIND.iter() {
        for word in words {
            if text_.contains(&format!(" {word} ")) || (!pointed_at.is_empty() && pointed_at.starts_with(word.as_str())) {
                let (one, they) = match kind.as_str() {
                    "guinea pig" => ("a guinea pig".to_string(), "guinea pigs".to_string()),
                    "horse" => ("a horse".to_string(), "horses".to_string()),
                    _ => (format!("a {kind}"), format!("{kind}s")),
                };
                return Animal { one_cap: text::upper_first(&one), one, they, them: "it".into(), kind: kind.clone() };
            }
        }
    }
    Animal {
        one: "an animal".into(),
        one_cap: "An animal".into(),
        they: "animals".into(),
        them: "it".into(),
        kind: "animal".into(),
    }
}

/// A pet that cannot use its back legs: the vet now, for a horse or any other animal.
pub fn _pet_back_legs(_memory: &Memory, understood: &Understood) -> Option<String> {
    let pet = _the_animal(understood);
    if pet.kind == "horse" {
        return Some(
            "That is an emergency - ring the vet now. A horse that cannot use its back legs \
may have a fracture, a spinal injury or a poisoning, and it must not be walked \
or moved far: keep it still on level ground, do not try to get it up by pulling \
the head or the tail, and keep other horses away while you wait for the vet."
                .into(),
        );
    }
    pet.fill(
        "Ring the vet now if this came on today: {one} that suddenly cannot use its back legs \
usually has a blood clot at the back of the aorta or a spinal injury, and the first \
hours decide how much comes back. Keep {them} warm and quiet on a towel, carry \
{them} to the car rather than letting {them} walk, and note the time it started. A \
slow drag that has built up over months is a different, less urgent question - but \
a change today is not one to watch.",
    )
}

/// A wobbly animal is a vet visit today.
pub fn _pet_wobbly(_memory: &Memory, understood: &Understood) -> Option<String> {
    let pet = _the_animal(understood);
    if pet.kind == "horse" {
        return pet.fill(
            "A horse that is wobbly or unsteady wants the vet out today, not a day's rest: \
it can be the neck or spine, a poisoning, or the balance centre, and a horse \
that goes down is hard to get up again. Keep {them} on level ground with food \
and water within reach, do not ride or lunge {them}, and note when it started \
and whether it is worse on one side.",
        );
    }
    pet.fill(
        "A wobbly animal is a vet visit today, not a wait and see: in a dog or cat it is often \
the inner ear, a poisoning or something pressing on the spine. Keep {them} off \
stairs and off the furniture, with food and water within reach, and note whether \
{they} is falling to one side, tilting its head or crossing its legs - a vet reads \
that off the note better than off a memory.",
    )
}

/// An animal that has stopped moving about is ill: a vet call today.
pub fn _pet_not_moving(_memory: &Memory, understood: &Understood) -> Option<String> {
    _the_animal(understood).fill(
        "{One} that has stopped moving about is ill, and small animals hide it until they are \
very sick - a rabbit, guinea pig or hamster sitting still and fluffed up is a vet \
call today, tonight if it is also off its food. Keep {them} warm and quiet, put \
water and hay or its usual food within reach so it need not move for them, and do \
not bath or handle it more than you have to.",
    )
}

/// A rabbit or guinea pig with a dirty bottom: flystrike risk, wash it and ring the vet.
pub fn _pet_dirty_bottom(_memory: &Memory, _understood: &Understood) -> Option<String> {
    Some(
        "Wash and dry it today and ring the vet: a rabbit or guinea pig with a dirty bottom is \
at risk of flystrike, where flies lay eggs in the soiled fur and maggots hatch within \
hours in warm weather. Check the bottom twice a day in summer and keep the hutch \
clean; an animal that cannot keep itself clean needs a vet for the cause, which is \
usually teeth or a sore back."
            .into(),
    )
}

// ---------------------------------------------------------------------------------------------
// The lambdas written inline in the entries.

/// `lambda memory, understood: _chocolate_for(_the_animal_that_ate(understood))` (line 4593).
pub fn lambda_4593(_memory: &Memory, understood: &Understood) -> Option<String> {
    Some(_chocolate_for(_the_animal_that_ate(understood)))
}
/// `lambda understood: not _the_bin_is_the_dogs_question(understood)` (line 4649).
pub fn lambda_4649(understood: &Understood) -> bool {
    !_the_bin_is_the_dogs_question(understood)
}
/// `lambda understood: not _one_pets_dinner(understood)` (lines 5979, 6310, 6351, 8915).
pub fn lambda_5979(understood: &Understood) -> bool {
    !_one_pets_dinner(understood)
}
/// `lambda understood: _not_a_named_cat(understood) and not _a_name_is_being_given(understood)`.
pub fn lambda_6873(understood: &Understood) -> bool {
    _not_a_named_cat(understood) && !_a_name_is_being_given(understood)
}
/// `lambda understood: _a_cat_in_the_line(understood) and _a_litter_is_what_is_asked_about(...)`.
pub fn lambda_6900(understood: &Understood) -> bool {
    _a_cat_in_the_line(understood) && _a_litter_is_what_is_asked_about(understood)
}
/// `lambda memory, understood: _can_pet_eat(understood)` (line 8896).
pub fn lambda_8896(_memory: &Memory, understood: &Understood) -> Option<String> {
    _can_pet_eat(understood)
}
/// `lambda memory, understood: _small_pet_food(understood)` (line 8956).
pub fn lambda_8956(_memory: &Memory, understood: &Understood) -> Option<String> {
    _small_pet_food(understood)
}
/// `lambda understood: not _one_pets_dinner(understood)` (line 6310; same body as 5979).
pub fn lambda_6310(understood: &Understood) -> bool {
    lambda_5979(understood)
}
/// `lambda understood: not _one_pets_dinner(understood)` (line 6351; same body as 5979).
pub fn lambda_6351(understood: &Understood) -> bool {
    lambda_5979(understood)
}
/// `lambda understood: not _one_pets_dinner(understood)` (line 8915; same body as 5979).
pub fn lambda_8915(understood: &Understood) -> bool {
    lambda_5979(understood)
}
/// `lambda memory, understood: _pet_scratching(understood)` (line 11295).
pub fn lambda_11295(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_scratching(understood)
}
/// `lambda memory, understood: _pet_shivers(understood)` (line 11992).
pub fn lambda_11992(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_shivers(understood)
}
/// `lambda memory, understood: _horse_lame(understood)` (line 12003).
pub fn lambda_12003(_memory: &Memory, understood: &Understood) -> Option<String> {
    _horse_lame(understood)
}
/// `lambda memory, understood: _horse_off_food(understood)` (line 12012).
pub fn lambda_12012(_memory: &Memory, understood: &Understood) -> Option<String> {
    _horse_off_food(understood)
}
/// `lambda memory, understood: _pet_snoring(understood)` (line 12047).
pub fn lambda_12047(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_snoring(understood)
}
/// `lambda memory, understood: _pet_moult(understood)` (line 12066).
pub fn lambda_12066(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_moult(understood)
}
/// `lambda memory, understood: _pet_breathing(understood)` (line 12127).
pub fn lambda_12127(_memory: &Memory, understood: &Understood) -> Option<String> {
    _pet_breathing(understood)
}
/// `lambda memory, understood: _horse_sweating(understood)` (line 12166).
pub fn lambda_12166(_memory: &Memory, understood: &Understood) -> Option<String> {
    _horse_sweating(understood)
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

guard_fn!(g_lambda_4649, lambda_4649);
guard_fn!(g_lambda_5979, lambda_5979);
guard_fn!(g_lambda_6310, lambda_6310);
guard_fn!(g_lambda_6351, lambda_6351);
guard_fn!(g_lambda_8915, lambda_8915);
guard_fn!(g_lambda_6873, lambda_6873);
guard_fn!(g_lambda_6900, lambda_6900);
guard_fn!(g_a_guarding_line, _a_guarding_line);
guard_fn!(g_a_handling_growl_line, _a_handling_growl_line);
guard_fn!(g_a_medicine_line, _a_medicine_line);
guard_fn!(g_a_person_was_hurt, _a_person_was_hurt);
guard_fn!(g_a_plain_hold_is_the_rodents_card, _a_plain_hold_is_the_rodents_card);
guard_fn!(g_a_warning_sound_said, _a_warning_sound_said);
guard_fn!(g_an_animal_did_it, _an_animal_did_it);
guard_fn!(g_an_eaten_creature, _an_eaten_creature);
guard_fn!(g_an_eaten_fungus, _an_eaten_fungus);
guard_fn!(g_dog_not_young, _dog_not_young);
guard_fn!(g_eaten_thing_is_not_food, _eaten_thing_is_not_food);
guard_fn!(g_fish_not_the_sport, _fish_not_the_sport);
guard_fn!(g_it_asks_what_to_do, _it_asks_what_to_do);
guard_fn!(g_not_a_weight_of_food, _not_a_weight_of_food);
guard_fn!(g_on_a_lead, _on_a_lead);
guard_fn!(g_one_pets_dinner, _one_pets_dinner);
guard_fn!(g_the_animal_got_wet, _the_animal_got_wet);
guard_fn!(g_this_is_the_pets_talk, _this_is_the_pets_talk);

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.pets:<lambda>:4649", g_lambda_4649),
    ("bot.knowledge.pets:<lambda>:5979", g_lambda_5979),
    ("bot.knowledge.pets:<lambda>:6310", g_lambda_6310),
    ("bot.knowledge.pets:<lambda>:6351", g_lambda_6351),
    ("bot.knowledge.pets:<lambda>:6873", g_lambda_6873),
    ("bot.knowledge.pets:<lambda>:6900", g_lambda_6900),
    ("bot.knowledge.pets:<lambda>:8915", g_lambda_8915),
    ("bot.knowledge.pets:_a_guarding_line:246", g_a_guarding_line),
    ("bot.knowledge.pets:_a_handling_growl_line:231", g_a_handling_growl_line),
    ("bot.knowledge.pets:_a_medicine_line:209", g_a_medicine_line),
    ("bot.knowledge.pets:_a_person_was_hurt:11465", g_a_person_was_hurt),
    ("bot.knowledge.pets:_a_plain_hold_is_the_rodents_card:2470", g_a_plain_hold_is_the_rodents_card),
    ("bot.knowledge.pets:_a_warning_sound_said:276", g_a_warning_sound_said),
    ("bot.knowledge.pets:_an_animal_did_it:60", g_an_animal_did_it),
    ("bot.knowledge.pets:_an_eaten_creature:332", g_an_eaten_creature),
    ("bot.knowledge.pets:_an_eaten_fungus:365", g_an_eaten_fungus),
    ("bot.knowledge.pets:_dog_not_young:172", g_dog_not_young),
    ("bot.knowledge.pets:_eaten_thing_is_not_food:4225", g_eaten_thing_is_not_food),
    ("bot.knowledge.pets:_fish_not_the_sport:151", g_fish_not_the_sport),
    ("bot.knowledge.pets:_it_asks_what_to_do:4256", g_it_asks_what_to_do),
    ("bot.knowledge.pets:_not_a_weight_of_food:2659", g_not_a_weight_of_food),
    ("bot.knowledge.pets:_on_a_lead:1121", g_on_a_lead),
    ("bot.knowledge.pets:_one_pets_dinner:2112", g_one_pets_dinner),
    ("bot.knowledge.pets:_the_animal_got_wet:11681", g_the_animal_got_wet),
    ("bot.knowledge.pets:_this_is_the_pets_talk:3748", g_this_is_the_pets_talk),
];

/// `_bird_lifespan(reply).<locals>.say`, with the reply its closure captured.
fn r_bird_lifespan_say(f: &FnRef, _m: &mut Memory, u: &Understood) -> Option<String> {
    let reply = f.cell("reply").map(Value::str).unwrap_or("");
    _bird_lifespan__say(reply, u)
}

reply_fn!(r_lambda_4593, lambda_4593);
reply_fn!(r_lambda_8896, lambda_8896);
reply_fn!(r_lambda_8956, lambda_8956);
reply_fn!(r_lambda_11295, lambda_11295);
reply_fn!(r_lambda_11992, lambda_11992);
reply_fn!(r_lambda_12003, lambda_12003);
reply_fn!(r_lambda_12012, lambda_12012);
reply_fn!(r_lambda_12047, lambda_12047);
reply_fn!(r_lambda_12066, lambda_12066);
reply_fn!(r_lambda_12127, lambda_12127);
reply_fn!(r_lambda_12166, lambda_12166);
reply_fn!(r_a_lost_pet, _a_lost_pet);
reply_fn!(r_after_a_clear_urine_result, _after_a_clear_urine_result);
reply_fn!(r_barking, _barking);
reply_fn!(r_bone_hazard, _bone_hazard);
reply_fn!(r_bored_pet, _bored_pet);
reply_fn!(r_breathing_fast, _breathing_fast);
reply_fn!(r_cat_and_baby, _cat_and_baby);
reply_fn!(r_cat_and_children, _cat_and_children);
reply_fn!(r_cat_biting, _cat_biting);
reply_fn!(r_cat_coat, _cat_coat);
reply_fn!(r_cat_habit, _cat_habit);
reply_fn!(r_cat_hair, _cat_hair);
reply_fn!(r_cat_urine, _cat_urine);
reply_fn!(r_cat_weight, _cat_weight);
reply_fn!(r_dog_bite_person, _dog_bite_person);
reply_fn!(r_dog_chewing, _dog_chewing);
reply_fn!(r_dog_in_a_car, _dog_in_a_car);
reply_fn!(r_dog_kit, _dog_kit);
reply_fn!(r_dog_night, _dog_night);
reply_fn!(r_dog_pulling, _dog_pulling);
reply_fn!(r_dog_recall, _dog_recall);
reply_fn!(r_dog_weight, _dog_weight);
reply_fn!(r_fish_tank, _fish_tank);
reply_fn!(r_grapes, _grapes);
reply_fn!(r_kitten_kit, _kitten_kit);
reply_fn!(r_knows_name, _knows_name);
reply_fn!(r_leftovers_answer, _leftovers_answer);
reply_fn!(r_litter_reply, _litter_reply);
reply_fn!(r_neuter_why, _neuter_why);
reply_fn!(r_noise_fear, _noise_fear);
reply_fn!(r_off_food_reply, _off_food_reply);
reply_fn!(r_paw_licking, _paw_licking);
reply_fn!(r_pet_back_legs, _pet_back_legs);
reply_fn!(r_pet_cable, _pet_cable);
reply_fn!(r_pet_cost, _pet_cost);
reply_fn!(r_pet_dairy, _pet_dairy);
reply_fn!(r_pet_dirty_bottom, _pet_dirty_bottom);
reply_fn!(r_pet_drink_more, _pet_drink_more);
reply_fn!(r_pet_limp, _pet_limp);
reply_fn!(r_pet_loss, _pet_loss);
reply_fn!(r_pet_mouth, _pet_mouth);
reply_fn!(r_pet_neuter, _pet_neuter);
reply_fn!(r_pet_not_drinking, _pet_not_drinking);
reply_fn!(r_pet_not_moving, _pet_not_moving);
reply_fn!(r_pet_salt_water, _pet_salt_water);
reply_fn!(r_pet_sick_again, _pet_sick_again);
reply_fn!(r_pet_thirst, _pet_thirst);
reply_fn!(r_pet_tummy, _pet_tummy);
reply_fn!(r_pet_wobbly, _pet_wobbly);
reply_fn!(r_pet_wound, _pet_wound);
reply_fn!(r_puppy_kit, _puppy_kit);
reply_fn!(r_rabbit_weather, _rabbit_weather);
reply_fn!(r_raw_meat, _raw_meat);
reply_fn!(r_rescue_or_baby, _rescue_or_baby);
reply_fn!(r_scared_of, _scared_of);
reply_fn!(r_small_fur_loss, _small_fur_loss);
reply_fn!(r_small_pet_alone, _small_pet_alone);
reply_fn!(r_small_pet_cost, _small_pet_cost);
reply_fn!(r_small_pet_fat, _small_pet_fat);
reply_fn!(r_small_pet_handling, _small_pet_handling);
reply_fn!(r_small_pet_setup, _small_pet_setup);
reply_fn!(r_the_hamster_basics, _the_hamster_basics);
reply_fn!(r_the_litter_routine, _the_litter_routine);
reply_fn!(r_the_person_hurt, _the_person_hurt);
reply_fn!(r_the_pet_checkup, _the_pet_checkup);
reply_fn!(r_the_vet_question, _the_vet_question);
reply_fn!(r_where_the_pet_sleeps, _where_the_pet_sleeps);

fn r_dog_feed(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_dog_feed(m, u))
}

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.pets:<lambda>:11295", r_lambda_11295),
    ("bot.knowledge.pets:<lambda>:11992", r_lambda_11992),
    ("bot.knowledge.pets:<lambda>:12003", r_lambda_12003),
    ("bot.knowledge.pets:<lambda>:12012", r_lambda_12012),
    ("bot.knowledge.pets:<lambda>:12047", r_lambda_12047),
    ("bot.knowledge.pets:<lambda>:12066", r_lambda_12066),
    ("bot.knowledge.pets:<lambda>:12127", r_lambda_12127),
    ("bot.knowledge.pets:<lambda>:12166", r_lambda_12166),
    ("bot.knowledge.pets:<lambda>:4593", r_lambda_4593),
    ("bot.knowledge.pets:<lambda>:8896", r_lambda_8896),
    ("bot.knowledge.pets:<lambda>:8956", r_lambda_8956),
    ("bot.knowledge.pets:_a_lost_pet:4100", r_a_lost_pet),
    ("bot.knowledge.pets:_after_a_clear_urine_result:1271", r_after_a_clear_urine_result),
    ("bot.knowledge.pets:_barking:1024", r_barking),
    ("bot.knowledge.pets:_bird_lifespan.<locals>.say:2590", r_bird_lifespan_say),
    ("bot.knowledge.pets:_bone_hazard:1997", r_bone_hazard),
    ("bot.knowledge.pets:_bored_pet:10332", r_bored_pet),
    ("bot.knowledge.pets:_breathing_fast:1682", r_breathing_fast),
    ("bot.knowledge.pets:_cat_and_baby:382", r_cat_and_baby),
    ("bot.knowledge.pets:_cat_and_children:2959", r_cat_and_children),
    ("bot.knowledge.pets:_cat_biting:299", r_cat_biting),
    ("bot.knowledge.pets:_cat_coat:3099", r_cat_coat),
    ("bot.knowledge.pets:_cat_habit:10732", r_cat_habit),
    ("bot.knowledge.pets:_cat_hair:10560", r_cat_hair),
    ("bot.knowledge.pets:_cat_urine:1213", r_cat_urine),
    ("bot.knowledge.pets:_cat_weight:3270", r_cat_weight),
    ("bot.knowledge.pets:_dog_bite_person:3410", r_dog_bite_person),
    ("bot.knowledge.pets:_dog_chewing:3368", r_dog_chewing),
    ("bot.knowledge.pets:_dog_feed:942", r_dog_feed),
    ("bot.knowledge.pets:_dog_in_a_car:4081", r_dog_in_a_car),
    ("bot.knowledge.pets:_dog_kit:3931", r_dog_kit),
    ("bot.knowledge.pets:_dog_night:1064", r_dog_night),
    ("bot.knowledge.pets:_dog_pulling:1138", r_dog_pulling),
    ("bot.knowledge.pets:_dog_recall:2910", r_dog_recall),
    ("bot.knowledge.pets:_dog_weight:3294", r_dog_weight),
    ("bot.knowledge.pets:_fish_tank:3939", r_fish_tank),
    ("bot.knowledge.pets:_grapes:2018", r_grapes),
    ("bot.knowledge.pets:_kitten_kit:541", r_kitten_kit),
    ("bot.knowledge.pets:_knows_name:10376", r_knows_name),
    ("bot.knowledge.pets:_leftovers_answer:3567", r_leftovers_answer),
    ("bot.knowledge.pets:_litter_reply:1173", r_litter_reply),
    ("bot.knowledge.pets:_neuter_why:482", r_neuter_why),
    ("bot.knowledge.pets:_noise_fear:2163", r_noise_fear),
    ("bot.knowledge.pets:_off_food_reply:10498", r_off_food_reply),
    ("bot.knowledge.pets:_paw_licking:1990", r_paw_licking),
    ("bot.knowledge.pets:_pet_back_legs:12231", r_pet_back_legs),
    ("bot.knowledge.pets:_pet_cable:11830", r_pet_cable),
    ("bot.knowledge.pets:_pet_cost:3070", r_pet_cost),
    ("bot.knowledge.pets:_pet_dairy:1301", r_pet_dairy),
    ("bot.knowledge.pets:_pet_dirty_bottom:12291", r_pet_dirty_bottom),
    ("bot.knowledge.pets:_pet_drink_more:2253", r_pet_drink_more),
    ("bot.knowledge.pets:_pet_limp:2377", r_pet_limp),
    ("bot.knowledge.pets:_pet_loss:2825", r_pet_loss),
    ("bot.knowledge.pets:_pet_mouth:781", r_pet_mouth),
    ("bot.knowledge.pets:_pet_neuter:1883", r_pet_neuter),
    ("bot.knowledge.pets:_pet_not_drinking:2313", r_pet_not_drinking),
    ("bot.knowledge.pets:_pet_not_moving:12276", r_pet_not_moving),
    ("bot.knowledge.pets:_pet_salt_water:2284", r_pet_salt_water),
    ("bot.knowledge.pets:_pet_sick_again:3820", r_pet_sick_again),
    ("bot.knowledge.pets:_pet_thirst:2360", r_pet_thirst),
    ("bot.knowledge.pets:_pet_tummy:1422", r_pet_tummy),
    ("bot.knowledge.pets:_pet_wobbly:12255", r_pet_wobbly),
    ("bot.knowledge.pets:_pet_wound:1726", r_pet_wound),
    ("bot.knowledge.pets:_puppy_kit:3935", r_puppy_kit),
    ("bot.knowledge.pets:_rabbit_weather:2134", r_rabbit_weather),
    ("bot.knowledge.pets:_raw_meat:2008", r_raw_meat),
    ("bot.knowledge.pets:_rescue_or_baby:3953", r_rescue_or_baby),
    ("bot.knowledge.pets:_scared_of:1036", r_scared_of),
    ("bot.knowledge.pets:_small_fur_loss:4003", r_small_fur_loss),
    ("bot.knowledge.pets:_small_pet_alone:10787", r_small_pet_alone),
    ("bot.knowledge.pets:_small_pet_cost:3056", r_small_pet_cost),
    ("bot.knowledge.pets:_small_pet_fat:3174", r_small_pet_fat),
    ("bot.knowledge.pets:_small_pet_handling:2420", r_small_pet_handling),
    ("bot.knowledge.pets:_small_pet_setup:671", r_small_pet_setup),
    ("bot.knowledge.pets:_the_hamster_basics:682", r_the_hamster_basics),
    ("bot.knowledge.pets:_the_litter_routine:754", r_the_litter_routine),
    ("bot.knowledge.pets:_the_person_hurt:11508", r_the_person_hurt),
    ("bot.knowledge.pets:_the_pet_checkup:719", r_the_pet_checkup),
    ("bot.knowledge.pets:_the_vet_question:3813", r_the_vet_question),
    ("bot.knowledge.pets:_where_the_pet_sleeps:4202", r_where_the_pet_sleeps),
];

fn s_the_buying_shape(_f: &FnRef, text_: &str) -> bool {
    _the_buying_shape(text_)
}

pub const SHAPES: &[(&str, ShapeFn)] = &[("bot.knowledge.pets:_the_buying_shape:583", s_the_buying_shape)];
