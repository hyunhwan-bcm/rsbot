//! Port of `bot/knowledge/babyfood.py`: weaning, first foods, and what a baby can have. Tables,
//! replies and entries are data from the bundle; every row's reply runs behind one gate
//! (`_only_for_a_baby`): nobody who is not feeding a baby is answered from this table.
//!
//! Build-time only: the outer bodies of `_verdict` and `_only_for_a_baby` (they build and wrap
//! the entries); their inner `answer` and `gated` are ported.

use crate::data;
use crate::data::{FnRef, Value};
use crate::knowledge::d2_shared::not_the_heat_of_an_argument;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::state::Memory;
use crate::understood::{Num, Number, Understood};
use std::collections::HashSet;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.babyfood.{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("bot.knowledge.babyfood.{name}"))
}
fn re(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.babyfood.{name}"))
}

static _BABY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BABY"));
static BABY: LazyLock<Vec<String>> = LazyLock::new(|| strs("BABY"));
static EATING: LazyLock<Vec<String>> = LazyLock::new(|| strs("EATING"));
static STAGES: LazyLock<Vec<(i64, Option<i64>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.babyfood.STAGES")
        .items()
        .iter()
        .map(|s| {
            (
                s.get("from").and_then(Value::as_int).unwrap_or(0),
                s.get("until").and_then(Value::as_int),
                s.get("reply").map(|r| r.str().to_string()).unwrap_or_default(),
            )
        })
        .collect()
});
static _NUMBER_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NUMBER_WORDS"));
/// Per number word: `\b<w>\s+months?\b`, `\b<w>\s+years?[\s-]*old\b`, and its value.
static NUMBER_WORD_AGES: LazyLock<Vec<(Re, Re, i64)>> = LazyLock::new(|| {
    _NUMBER_WORDS
        .iter()
        .enumerate()
        .map(|(i, w)| {
            (
                Re::new(&format!(r"\b{w}\s+months?\b"), 0),
                Re::new(&format!(r"\b{w}\s+years?[\s-]*old\b"), 0),
                i as i64 + 1,
            )
        })
        .collect()
});
static _FIRST_FOODS: LazyLock<String> = LazyLock::new(|| string("_FIRST_FOODS"));
static _AGE_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| re("_AGE_FIGURE"));
static _THE_CHILD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_THE_CHILD"));
static _MILK_UNDER_SIX: LazyLock<String> = LazyLock::new(|| string("_MILK_UNDER_SIX"));
static _MILK_SIX_TO_TWELVE: LazyLock<String> = LazyLock::new(|| string("_MILK_SIX_TO_TWELVE"));
static _MILK_OVER_ONE: LazyLock<String> = LazyLock::new(|| string("_MILK_OVER_ONE"));
static _WATER_UNDER_SIX: LazyLock<String> = LazyLock::new(|| string("_WATER_UNDER_SIX"));
static _WATER_OVER_ONE: LazyLock<String> = LazyLock::new(|| string("_WATER_OVER_ONE"));
static _WATER_NO_AGE: LazyLock<String> = LazyLock::new(|| string("_WATER_NO_AGE"));
static _WATER_JUICE: LazyLock<String> = LazyLock::new(|| string("_WATER_JUICE"));
static _DRINK_UNDER_SIX: LazyLock<String> = LazyLock::new(|| string("_DRINK_UNDER_SIX"));
static _DRINK_SIX_TO_TWELVE: LazyLock<String> = LazyLock::new(|| string("_DRINK_SIX_TO_TWELVE"));
static _DRINK_OVER_ONE: LazyLock<String> = LazyLock::new(|| string("_DRINK_OVER_ONE"));
static _SNACK_UNDER_SIX: LazyLock<String> = LazyLock::new(|| string("_SNACK_UNDER_SIX"));
static _SNACK_SIX_TO_TWELVE: LazyLock<String> = LazyLock::new(|| string("_SNACK_SIX_TO_TWELVE"));
static _SNACK_OVER_ONE: LazyLock<String> = LazyLock::new(|| string("_SNACK_OVER_ONE"));
static _PLANT_MILK_BABY: LazyLock<String> = LazyLock::new(|| string("_PLANT_MILK_BABY"));
static _PLANT_MILK_RICE: LazyLock<String> = LazyLock::new(|| string("_PLANT_MILK_RICE"));
static _PLANT_MILK_CHILD: LazyLock<String> = LazyLock::new(|| string("_PLANT_MILK_CHILD"));
static _A_CHILD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_CHILD"));
static _NAMES_A_BABY: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NAMES_A_BABY"));
static _FOR_A_GROWN_UP: LazyLock<Vec<String>> = LazyLock::new(|| strs("_FOR_A_GROWN_UP"));
static _NOT_A_BABY_NAME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOT_A_BABY_NAME"));
static _A_RATE_IN_MONTHS: LazyLock<&'static Re> = LazyLock::new(|| re("_A_RATE_IN_MONTHS"));
static _A_BABY_TILL: LazyLock<i64> =
    LazyLock::new(|| data::get("bot.knowledge.babyfood._A_BABY_TILL").as_int().unwrap_or(36));
static _A_CHILD_PRONOUN: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_CHILD_PRONOUN"));
static _A_CONDITION: LazyLock<&'static Re> = LazyLock::new(|| re("_A_CONDITION"));
static _AN_ADULT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_AN_ADULT"));
static _A_TABLE_WORD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_TABLE_WORD"));
static _A_TABLE_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_TABLE_FOOD"));
static _OLDER_CHILD_WORD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_OLDER_CHILD_WORD"));
static OLDER_CHILD_WORD_RX: LazyLock<Vec<Re>> =
    LazyLock::new(|| _OLDER_CHILD_WORD.iter().map(|w| Re::new(&format!(r"\b{w}\b"), 0)).collect());
static OLDER_CHILD_BARE_FIGURE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        &format!(
            r"\b(?:{})\s+(?:is\s+|are\s+|aged\s+|at\s+|of\s+)?(\d{{1,2}})\b(?!\s*(?:month|months|mth|mths|week|weeks|wk|wks|day|days|stone|kilos|kilo|kg|lbs|pounds)\b)",
            _OLDER_CHILD_WORD.join("|")
        ),
        0,
    )
});
static _SELF_PRONOUN: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SELF_PRONOUN"));
static _OWN_EATING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_OWN_EATING"));
static _FOR_ME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_FOR_ME"));
static _BABY_ACT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BABY_ACT"));
static _INTRODUCES_A_FOOD: LazyLock<&'static Re> = LazyLock::new(|| re("_INTRODUCES_A_FOOD"));
static _NOT_MY_DIET: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NOT_MY_DIET"));
static _A_FEEDING_PHRASE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_FEEDING_PHRASE"));
static _GIVING_WATER: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GIVING_WATER"));
static _GIVING_WATER_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.babyfood._GIVING_WATER_ASKS"));
static _GIVING_WATER_OUT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GIVING_WATER_OUT"));
static _WHILE_SHE_FEEDS: LazyLock<&'static Re> = LazyLock::new(|| re("_WHILE_SHE_FEEDS"));
static _HER_OWN_EATING: LazyLock<&'static Re> = LazyLock::new(|| re("_HER_OWN_EATING"));
static _ALLERGEN_ROW: LazyLock<String> = LazyLock::new(|| string("_ALLERGEN_ROW"));
static _ALLERGEN_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ALLERGEN_FOOD"));
static _INTRODUCING_PEOPLE: LazyLock<&'static Re> = LazyLock::new(|| re("_INTRODUCING_PEOPLE"));
static _GROWN_UP: LazyLock<Vec<String>> = LazyLock::new(|| strs("_GROWN_UP"));
static _ANIMAL: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ANIMAL"));
static _SEASON_OF_A_SHOW: LazyLock<&'static Re> = LazyLock::new(|| re("_SEASON_OF_A_SHOW"));

static BABY_IS_ONE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bbaby is (one|1)\b", 0));
static ONE_YEAR_OLD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(one|1) year old\b", 0));
static TODDLER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(toddler|toddlers)\b", 0));
static NEWBORN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(newborn|newborns)\b", 0));
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z']+", 0));

fn any_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(w.as_ref()))
}

/// Any of the words standing on its own in the padded line.
fn padded_has<S: AsRef<str>>(padded: &str, words: &[S]) -> bool {
    words.iter().any(|w| padded.contains(&format!(" {} ", w.as_ref())))
}

/// "when can i give water": water, being given, and no drink of the person's own.
pub fn _the_water_ask(understood: &Understood) -> bool {
    let text = format!(" {} ", _text(understood));
    if !text.contains(" water ") && !text.ends_with(" water ") {
        return false;
    }
    if any_in(&text, &["i drink", "should i drink", " a day ", "litre", "liter", "pint", "eight glasses", "2 litres", "2 liters"]) {
        return false;
    }
    any_in(&text, &["give", "giving", "offer", "when can", "how much", "how often", "sips"])
}

/// The line as lower-case words, digits and all.
pub fn _text(understood: &Understood) -> String {
    if !understood.words.is_empty() {
        return understood.words.join(" ").to_lowercase();
    }
    understood.clean.to_lowercase()
}

/// The age the conversation gave a turn or two ago, for a line that only says "he".
pub fn _age_in_the_chat(memory: &Memory, understood: &Understood) -> Option<i64> {
    let words: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if !_THE_CHILD.iter().any(|w| words.contains(w)) {
        return None;
    }
    let said_now = crate::py::text::strip(&understood.clean).to_lowercase();
    for turn in memory.turns.iter().rev() {
        let text = crate::py::text::strip(&turn.user).to_lowercase();
        if text.is_empty() || text == said_now {
            continue;
        }
        let numbers: Vec<Number> = _AGE_FIGURE
            .finditer(&text)
            .into_iter()
            .map(|m| Number {
                value: m.str(1).parse::<i64>().ok().map(Num::Int),
                unit: Some(m.str(2).to_string()),
                text: String::new(),
            })
            .collect();
        let reading = Understood {
            clean: text.clone(),
            words: crate::py::text::split(&text),
            numbers,
            ..Default::default()
        };
        if let Some(age) = _age_in_months(&reading) {
            return Some(age);
        }
    }
    None
}

/// The age the line gives, in months, or None when it names none.
pub fn _age_in_months(understood: &Understood) -> Option<i64> {
    for number in &understood.numbers {
        let Some(Num::Int(value)) = number.value else { continue };
        if value < 0 {
            continue;
        }
        let unit = number.unit.as_deref().unwrap_or("").to_lowercase();
        if unit.starts_with("month") {
            return Some(value);
        }
        if unit.starts_with("year") {
            return Some(12 * value);
        }
        if unit.starts_with("week") {
            return Some(0);
        }
    }
    let said = _text(understood);
    if BABY_IS_ONE.is_match(&said) || ONE_YEAR_OLD.is_match(&said) {
        return Some(12);
    }
    for (months, years, at) in NUMBER_WORD_AGES.iter() {
        if months.is_match(&said) {
            return Some(*at);
        }
        if years.is_match(&said) {
            return Some(12 * at);
        }
    }
    if OLDER_CHILD_WORD_RX.iter().any(|rx| rx.is_match(&said)) {
        if let Some(bare) = OLDER_CHILD_BARE_FIGURE.search(&said) {
            let n: i64 = bare.str(1).parse().unwrap_or(0);
            if (2..=19).contains(&n) {
                return Some(12 * n);
            }
        }
    }
    if TODDLER.is_match(&said) {
        return Some(12);
    }
    if NEWBORN.is_match(&said) {
        return Some(0);
    }
    None
}

/// The reply for the stage the line's age lands on, or None when it names no age.
pub fn _stage(understood: &Understood) -> Option<String> {
    let age = _age_in_months(understood)?;
    for (from, until, reply) in STAGES.iter() {
        if age >= *from && until.is_none_or(|u| age <= u) {
            return Some(reply.clone());
        }
    }
    None
}

fn age_of(memory: &Memory, understood: &Understood) -> Option<i64> {
    _age_in_months(understood).or_else(|| _age_in_the_chat(memory, understood))
}

/// How much water, for the age the line gives or the chat already knows.
pub fn _how_much_water(memory: &mut Memory, understood: &Understood) -> String {
    let age = age_of(memory, understood);
    if age.is_some_and(|a| a < 6) {
        return format!("{}{}", *_WATER_UNDER_SIX, *_WATER_JUICE);
    }
    if age.is_some_and(|a| a >= 12) {
        return format!("{}{}", *_WATER_OVER_ONE, *_WATER_JUICE);
    }
    format!("{}{}", *_WATER_NO_AGE, *_WATER_JUICE)
}

/// How much milk, for the age the line gives - and for all three bands when it gives none.
pub fn _milk_volume(memory: &mut Memory, understood: &Understood) -> String {
    match age_of(memory, understood) {
        Some(a) if a < 6 => _MILK_UNDER_SIX.clone(),
        Some(a) if a < 12 => _MILK_SIX_TO_TWELVE.clone(),
        Some(_) => _MILK_OVER_ONE.clone(),
        None => format!("{} {} {}", *_MILK_UNDER_SIX, *_MILK_SIX_TO_TWELVE, *_MILK_OVER_ONE),
    }
}

/// What goes in the cup, for the age the line gives or the chat already knows.
pub fn _what_a_baby_drinks(memory: &mut Memory, understood: &Understood) -> String {
    match age_of(memory, understood) {
        Some(a) if a < 6 => _DRINK_UNDER_SIX.clone(),
        Some(a) if a < 12 => _DRINK_SIX_TO_TWELVE.clone(),
        Some(_) => _DRINK_OVER_ONE.clone(),
        None => format!("{} {} {}", *_DRINK_UNDER_SIX, *_DRINK_SIX_TO_TWELVE, *_DRINK_OVER_ONE),
    }
}

/// What to give between meals, banded by the age the line names.
pub fn _snacks(memory: &mut Memory, understood: &Understood) -> String {
    match age_of(memory, understood) {
        Some(a) if a < 6 => _SNACK_UNDER_SIX.clone(),
        Some(a) if a < 12 => _SNACK_SIX_TO_TWELVE.clone(),
        Some(_) => _SNACK_OVER_ONE.clone(),
        None => format!("{} {}", *_SNACK_SIX_TO_TWELVE, *_SNACK_OVER_ONE),
    }
}

/// Whether the line is about eating at all, or only has a baby and a question in it.
pub fn _says_what_they_eat(understood: &Understood) -> bool {
    padded_has(&format!(" {} ", _text(understood)), &EATING)
}

/// A plant milk by the age of the child, because the answer changes twice.
pub fn _plant_milk(memory: &mut Memory, understood: &Understood) -> String {
    let age = age_of(memory, understood);
    let rice = _text(understood).contains("rice");
    let Some(age) = age else {
        return format!("{} {}", *_PLANT_MILK_BABY, *_PLANT_MILK_RICE);
    };
    if age < 12 {
        return if rice { format!("{} {}", *_PLANT_MILK_BABY, *_PLANT_MILK_RICE) } else { _PLANT_MILK_BABY.clone() };
    }
    if rice {
        return _PLANT_MILK_RICE.clone();
    }
    _PLANT_MILK_CHILD.clone()
}

/// The stage the line names, and the first foods for a line that names no age.
pub fn _by_age(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_says_what_they_eat(understood) {
        return None;
    }
    Some(_stage(understood).filter(|s| !s.is_empty()).unwrap_or_else(|| _FIRST_FOODS.clone()))
}

/// The line with the foods that call themselves baby taken out.
pub fn _names_no_baby(text: &str) -> String {
    let mut low = format!(" {text} ");
    for name in _NOT_A_BABY_NAME.iter() {
        low = low.replace(&format!(" {name} "), " thing ");
    }
    low
}

/// True when the line itself says whose food this is: a baby, an age, a little one, a child.
pub fn _a_baby_is_named(understood: &Understood) -> bool {
    let low = format!(" {} ", _text(understood));
    if _AGE_FIGURE.is_match(&low) {
        return true;
    }
    let hit: Vec<&String> = _NAMES_A_BABY.iter().filter(|w| low.contains(&format!(" {w} "))).collect();
    if !hit.is_empty() && hit.iter().all(|w| *w == "month" || *w == "months") && _A_RATE_IN_MONTHS.is_match(&low) {
        return false;
    }
    !hit.is_empty()
}

/// Is the line's condition about this row's food, and is that the whole question?
pub fn _a_condition_of_this_food<S: AsRef<str>>(names: &[S], understood: &Understood) -> bool {
    let text = _text(understood);
    let Some(m) = _A_CONDITION.search(&text) else { return false };
    let mut organ = m.str(1).to_lowercase();
    if organ.ends_with('s') {
        organ.pop();
    }
    if !names.iter().any(|n| n.as_ref().to_lowercase().contains(&organ)) {
        return false;
    }
    let rest_line = format!("{} {}", &text[..m.start()], &text[m.end()..]);
    let rest = rest_line.to_lowercase();
    if _A_CONDITION.is_match(&rest) {
        return false;
    }
    for name in names {
        let word = name.as_ref().to_lowercase();
        if Re::new(&format!(r"(?<!\w){}(?!\w)", escape(&word)), 0).is_match(&rest.to_lowercase()) {
            return false;
        }
    }
    true
}

/// Whether the line names something of this table's own: a weaning word or one of its foods.
pub fn _a_table_word_is_named(understood: &Understood) -> bool {
    let low = format!(" {} ", _text(understood));
    if padded_has(&low, &_A_TABLE_WORD) {
        return true;
    }
    padded_has(&low, &_A_TABLE_FOOD)
}

/// Whether a child pronoun is in the line and the line names no grown-up (unless it names a
/// food of this table's own).
pub fn _a_pronoun_is_in_the_line(understood: &Understood) -> bool {
    let low = format!(" {} ", _text(understood));
    if !padded_has(&low, &_A_CHILD_PRONOUN) {
        return false;
    }
    if _a_table_word_is_named(understood) {
        return true;
    }
    !padded_has(&low, &_AN_ADULT)
}

/// The feeding rows' guard: the line has to be about a young child, not an adult.
pub fn _a_baby_or_a_pronoun(understood: &Understood) -> bool {
    if _a_baby_is_named(understood) {
        return true;
    }
    let low = format!(" {} ", _text(understood));
    if !padded_has(&low, &_A_CHILD_PRONOUN) {
        return false;
    }
    !padded_has(&low, &_AN_ADULT)
}

/// True when the line gives the child an age of a year or more.
pub fn _an_age_of_a_year_or_more(understood: &Understood) -> bool {
    _age_in_months(understood).is_some_and(|a| a >= 12)
}

/// The feeding rows' other guard: a first food is for a child young enough to need one.
pub fn _of_weaning_age(understood: &Understood) -> bool {
    !_an_age_of_a_year_or_more(understood)
}

/// Whether the line is somebody asking about their own food rather than a baby's.
pub fn _says_the_speaker_is_eating(words: &[String], said: &str) -> bool {
    if any_in(said, &_FOR_ME) {
        return true;
    }
    for (index, word) in words.iter().enumerate() {
        if _SELF_PRONOUN.contains(word) {
            if index + 1 < words.len() && (_A_CHILD.contains(&words[index + 1]) || BABY.contains(&words[index + 1])) {
                continue;
            }
            let later = &words[(index + 1).min(words.len())..(index + 4).min(words.len())];
            if later.iter().any(|w| _OWN_EATING.contains(w)) {
                return true;
            }
        }
    }
    false
}

/// Whether this line is about feeding a baby, the ages they are, or the child just named.
pub fn _the_line_feeds_somebody(understood: &Understood, memory: &Memory) -> bool {
    let word_list: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let words: HashSet<&str> = word_list.iter().map(String::as_str).collect();
    let said = _text(understood);
    let padded = format!(" {said} ");
    if understood.topic.as_deref() == Some("pets") || understood.topic_own.as_deref() == Some("pets") {
        if !_a_baby_is_named(understood) && _age_in_months(understood).is_none() {
            return false;
        }
    }
    if any_in(&padded, &_FOR_A_GROWN_UP) {
        return false;
    }
    let stripped = _names_no_baby(&said);
    let names_a_baby = padded_has(&stripped, &_NAMES_A_BABY) || _AGE_FIGURE.is_match(&stripped);
    let mut feeds = any_in(&padded, &_A_FEEDING_PHRASE);
    if !feeds && word_list.len() >= 3 && words.iter().any(|w| _GIVING_WATER_ASKS.contains(*w)) && any_in(&padded, &_GIVING_WATER) {
        feeds = !padded_has(&padded, &_GIVING_WATER_OUT);
    }
    if !feeds
        && _INTRODUCES_A_FOOD.is_match(&padded)
        && !any_in(&padded, &_NOT_MY_DIET)
        && _ALLERGEN_FOOD.iter().filter(|w| *w != "bottle" && *w != "formula").any(|w| padded.contains(&format!(" {w} ")))
    {
        feeds = true;
    }
    if !names_a_baby && !feeds && _says_the_speaker_is_eating(&word_list, &said) {
        return false;
    }
    if !names_a_baby && _WHILE_SHE_FEEDS.is_match(&said) && _HER_OWN_EATING.is_match(&said) {
        return false;
    }
    if _a_baby_is_named(understood) {
        return true;
    }
    if _age_in_months(understood).is_some() {
        return true;
    }
    if feeds {
        return true;
    }
    if ["he", "she", "him", "her", "his", "hers", "they"].iter().any(|w| words.contains(w)) {
        return !_the_pronoun_is_an_animal(understood);
    }
    if _says_the_speaker_is_eating(&word_list, &said) {
        return false;
    }
    if padded_has(&padded, &_BABY_ACT) {
        return true;
    }
    let said_now = crate::py::text::strip(&said).to_lowercase();
    for turn in memory.turns.iter().rev() {
        let text = crate::py::text::strip(&turn.user).to_lowercase();
        if text.is_empty() || text == said_now {
            continue;
        }
        let earlier = format!(" {text} ");
        if _AGE_FIGURE.is_match(&earlier) {
            return true;
        }
        if padded_has(&earlier, &_NAMES_A_BABY) {
            return true;
        }
    }
    false
}

/// The allergen answer, unless the line is introducing two people to each other.
pub fn _the_allergen_row(_memory: &mut Memory, understood: &Understood) -> String {
    let text = format!(" {} ", _text(understood));
    if _INTRODUCING_PEOPLE.is_match(&text) && !padded_has(&text, &_ALLERGEN_FOOD) {
        return String::new();
    }
    _ALLERGEN_ROW.clone()
}

/// The words of a referent, split as `re.split(r"[^a-z']+", place.lower())`.
fn place_words(place: &str) -> Vec<String> {
    NOT_A_WORD.split(&place.to_lowercase()).into_iter().filter(|w| !w.is_empty()).collect()
}

/// The referents a he/she/they can point at (the typed `ref` holds he, she and they).
fn person_refs(understood: &Understood) -> Vec<&str> {
    let r = &understood.reference;
    [&r.he, &r.she, &r.they].iter().filter_map(|v| v.as_deref()).collect()
}

/// True when the "he" or "she" of the line points at an animal the chat named.
pub fn _the_pronoun_is_an_animal(understood: &Understood) -> bool {
    let mut places: Vec<String> = person_refs(understood).into_iter().map(str::to_string).collect();
    // TYPE(understood.leans_on): Python reads leans_on["subject"] and ["text"]; the typed reading
    // holds the leaned-on line as a string.
    if let Some(leans) = &understood.leans_on {
        places.push(leans.clone());
    }
    places.iter().any(|place| place_words(place).iter().any(|w| _ANIMAL.contains(w)))
}

/// True when the "she" or "he" of the line points at a grown-up the chat named.
pub fn _the_pronoun_is_a_grown_up(understood: &Understood) -> bool {
    for place in person_refs(understood) {
        if crate::py::text::strip(place).is_empty() {
            continue;
        }
        let words = place_words(place);
        if words.iter().any(|w| _BABY.contains(w) || _A_CHILD_PRONOUN.contains(w) || w == "one" || w == "baby" || w == "child") {
            continue;
        }
        if words.iter().any(|w| _GROWN_UP.contains(w)) {
            return true;
        }
    }
    false
}

/// `_verdict(names, reply, own_ask=...).<locals>.answer`: the row's verdict when the line (or
/// the chat behind it) is about a young child; "" when it is not this table's.
pub fn _verdict_answer(
    names: &[String],
    reply: &str,
    own_ask: Option<fn(&Understood) -> bool>,
    memory: &Memory,
    understood: &Understood,
) -> String {
    if _a_condition_of_this_food(names, understood) {
        return String::new();
    }
    if own_ask.is_some_and(|ask| ask(understood)) {
        return reply.to_string();
    }
    if _a_baby_is_named(understood) {
        return reply.to_string();
    }
    if _a_pronoun_is_in_the_line(understood) {
        if let Some(age) = _age_in_the_chat(memory, understood) {
            return if age <= *_A_BABY_TILL { reply.to_string() } else { String::new() };
        }
        if _the_pronoun_is_a_grown_up(understood) && !_a_table_word_is_named(understood) {
            return String::new();
        }
        return reply.to_string();
    }
    if _age_in_the_chat(memory, understood).is_some_and(|age| age <= *_A_BABY_TILL) {
        return reply.to_string();
    }
    // TYPE(understood.leans_on): Python reads leans_on["words"] (a list) for a baby word; the
    // typed reading holds no word list, so there is nothing to read here.
    String::new()
}

/// A guard function a row carries as `own_ask`, by its Python id.
fn own_ask_of(v: &Value) -> Option<fn(&Understood) -> bool> {
    match v.as_fn()?.name() {
        "bot.knowledge.babyfood:_the_water_ask" => Some(_the_water_ask),
        other => panic!("no port of own_ask {other}"),
    }
}

/// A row's reply value: a string, or one of this module's reply functions.
fn call_reply_value(reply: &Value, memory: &mut Memory, understood: &Understood) -> Option<String> {
    match reply {
        Value::Str(s) => Some(s.to_string()),
        Value::Fn(f) => match f.name() {
            "bot.knowledge.babyfood:_verdict.<locals>.answer" => {
                let names = f.cell("names").map(|v| v.strs()).unwrap_or_default();
                let text = f.cell("reply").map(|v| v.str().to_string()).unwrap_or_default();
                let own_ask = f.cell("own_ask").and_then(own_ask_of);
                Some(_verdict_answer(&names, &text, own_ask, memory, understood))
            }
            "bot.knowledge.babyfood:_by_age" => _by_age(memory, understood),
            "bot.knowledge.babyfood:_how_much_water" => Some(_how_much_water(memory, understood)),
            "bot.knowledge.babyfood:_milk_volume" => Some(_milk_volume(memory, understood)),
            "bot.knowledge.babyfood:_plant_milk" => Some(_plant_milk(memory, understood)),
            "bot.knowledge.babyfood:_snacks" => Some(_snacks(memory, understood)),
            "bot.knowledge.babyfood:_the_allergen_row" => Some(_the_allergen_row(memory, understood)),
            "bot.knowledge.babyfood:_what_a_baby_drinks" => Some(_what_a_baby_drinks(memory, understood)),
            _ => {
                let got = crate::knowledge::registry::reply(f, memory, understood);
                if got.is_empty() { None } else { Some(got) }
            }
        },
        _ => None,
    }
}

/// `_only_for_a_baby(row).<locals>.gated`: the row's own reply, only for a line that feeds a baby.
pub fn _only_for_a_baby_gated(reply: &Value, memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_the_line_feeds_somebody(understood, memory) {
        return None;
    }
    call_reply_value(reply, memory, understood)
}

fn gated(f: &FnRef, memory: &mut Memory, understood: &Understood) -> Option<String> {
    _only_for_a_baby_gated(f.cell("reply").unwrap_or(&Value::None), memory, understood)
}

/// True unless "season" here is episodes or a league: then the spice row is not the answer.
pub fn _not_a_season_of_a_show(understood: &Understood) -> bool {
    let low = format!(" {} ", _text(understood));
    if padded_has(&low, &_BABY) || padded_has(&low, &_A_CHILD) {
        return true;
    }
    !_SEASON_OF_A_SHOW.is_match(&low)
}

/// True when the line names the child whose food this is, so the spice row may answer.
pub fn _a_child_is_in_the_line(understood: &Understood) -> bool {
    let low = format!(" {} ", _text(understood));
    if padded_has(&low, &BABY) || padded_has(&low, &_A_CHILD) || _AGE_FIGURE.is_match(&low) {
        return true;
    }
    if _SEASON_OF_A_SHOW.is_match(&low) {
        return false;
    }
    padded_has(&low, &_A_CHILD_PRONOUN)
}

/// Whether this line, or the line it leans on, names somebody being fed.
pub fn _a_line_that_feeds_a_baby(understood: &Understood) -> bool {
    // Python passes an empty memory: no turns to read back.
    _the_line_feeds_somebody(understood, &Memory::new())
    // TYPE(understood.leans_on): Python also reads leans_on["words"] (a list) for an age or a
    // baby word; the typed reading holds no word list.
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.babyfood:<lambda>:1946", |_: &FnRef, u: &Understood| _a_baby_or_a_pronoun(u) && _of_weaning_age(u)),
    ("bot.knowledge.babyfood:<lambda>:2064", |_: &FnRef, u: &Understood| _a_baby_is_named(u) || _a_pronoun_is_in_the_line(u)),
    ("bot.knowledge.babyfood:<lambda>:2112", |_: &FnRef, u: &Understood| {
        (_a_baby_is_named(u) || _a_pronoun_is_in_the_line(u)) && not_the_heat_of_an_argument(u)
    }),
    ("bot.knowledge.babyfood:<lambda>:2257", |_: &FnRef, u: &Understood| _a_baby_is_named(u) || _a_pronoun_is_in_the_line(u)),
    ("bot.knowledge.babyfood:_a_baby_or_a_pronoun:1069", |_: &FnRef, u: &Understood| _a_baby_or_a_pronoun(u)),
    ("bot.knowledge.babyfood:_a_child_is_in_the_line:1502", |_: &FnRef, u: &Understood| _a_child_is_in_the_line(u)),
    ("bot.knowledge.babyfood:_a_line_that_feeds_a_baby:2291", |_: &FnRef, u: &Understood| _a_line_that_feeds_a_baby(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.babyfood:_only_for_a_baby.<locals>.gated:2320", gated)];
