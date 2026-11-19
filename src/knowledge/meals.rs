//! Port of PyBot's `bot/knowledge/meals.py`: meal ideas (what to cook tonight, what to make from
//! what is already in the kitchen), the use-up table, carrying food, takeaways and counters.
#![allow(non_snake_case)]

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g3 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{BTreeSet, HashSet};
use std::sync::LazyLock;

fn m(name: &str) -> String {
    format!("bot.knowledge.meals.{name}")
}
fn strs(name: &str) -> Vec<String> {
    data::strs(&m(name))
}

static USE_DISH_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("USE_DISH_WORDS"));
static UP_CLASSES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&m("UP_CLASSES"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].strs(), p[1].str().to_string())
        })
        .collect()
});
static UP_FOOD_SET: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&m("_UP_FOOD_SET")));

/// The first food in the line this table answers, as the line spells it.
pub fn _up_food(words: &[String]) -> Option<String> {
    for index in 0..words.len() {
        if UP_FOOD_SET.contains(&words[index]) {
            return Some(words[index].clone());
        }
        if index + 1 < words.len() {
            let two = words[index..index + 2].join(" ");
            if UP_FOOD_SET.contains(&two) {
                return Some(two);
            }
        }
    }
    None
}

/// True when a pantry idea is a way to use the food up rather than how to cook it raw.
pub fn idea_is_a_dish(idea: &str) -> bool {
    let low = idea.to_lowercase();
    USE_DISH_WORDS.iter().any(|w| low.contains(w.as_str()))
}

/// How to use up a spare or cooked food, told by the kind of food it is, naming it.
pub fn use_it_up(name: &str) -> String {
    let said = name.trim();
    let folded = said.to_lowercase();
    for (words, mv) in UP_CLASSES.iter() {
        if words.iter().any(|w| *w == folded) {
            return format!("Leftover {said}: {mv}");
        }
    }
    for (words, mv) in UP_CLASSES.iter() {
        if words.iter().any(|w| folded.ends_with(&format!(" {w}")) || folded.starts_with(&format!("{w} "))) {
            return format!("Leftover {said}: {mv}");
        }
    }
    format!(
        "Leftover {said}: already cooked, so treat it as an ingredient - stir through a soup or a curry, fry with an onion and an egg, or eat cold in a salad with oil and vinegar."
    )
}

static ASK_CUES: LazyLock<Vec<String>> = LazyLock::new(|| strs("ASK_CUES"));
static ASK_IN_NEWS: LazyLock<Vec<String>> = LazyLock::new(|| strs("ASK_IN_NEWS"));
static DIET_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DIET_WORDS"));

/// True when the line is news about a meal rather than a request for one.
pub fn reads_as_news(understood: &Understood, clean: &str) -> bool {
    if understood.intent != "news" {
        return false;
    }
    let low = sh::or(clean, &understood.clean).to_lowercase();
    !ASK_IN_NEWS.iter().any(|cue| low.contains(cue.as_str()))
}

/// True when this line is a request for a meal, not news about one.
pub fn asks_for_a_meal(understood: &Understood, clean: &str) -> bool {
    if reads_as_news(understood, clean) {
        return false;
    }
    if understood.kind == "question" || understood.intent == "question" || understood.intent == "request" {
        return true;
    }
    let low = sh::or(clean, &understood.clean).to_lowercase();
    let words: Vec<&str> = low.split_whitespace().collect();
    ASK_CUES.iter().any(|cue| words.contains(&cue.as_str()))
}

/// True when the line itself names a diet or an allergy.
pub fn says_a_diet(text: &str) -> bool {
    let low = text.to_lowercase();
    DIET_WORDS.iter().any(|cue| low.contains(cue.as_str()))
}

// ---------------------------------------------------------------------------------------------
// The dishes

/// One weeknight dish: its name, the ingredients it uses, its total minutes, its diet, the
/// one-clause method, whether it is a favourite for the open ask, and how it is quoted back.
#[derive(Debug, Clone)]
pub struct Dish {
    pub name: String,
    pub uses: Vec<String>,
    pub minutes: i64,
    pub diet: String,
    pub how: String,
    pub favourite: bool,
    pub said: String,
}

impl Dish {
    /// `Dish.__repr__`.
    pub fn repr(&self) -> String {
        format!("<Dish {}>", self.name)
    }
    fn uses_any(&self, names: &[String]) -> bool {
        self.uses.iter().any(|u| names.contains(u))
    }
}

static SAID_AS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("SAID_AS"));
static FAVOURITES: LazyLock<Vec<String>> = LazyLock::new(|| strs("FAVOURITES"));

/// `Dish.__init__`: a dish from its parts (the bundle holds the built ones, so this is only
/// for a caller that makes a new one).
pub fn dish_new(name: &str, uses: &[String], minutes: i64, diet: &str, how: &str) -> Dish {
    Dish {
        name: name.to_string(),
        said: SAID_AS.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone()).unwrap_or_else(|| name.to_string()),
        uses: uses.to_vec(),
        minutes,
        diet: diet.to_string(),
        how: how.to_string(),
        favourite: FAVOURITES.iter().any(|f| f == name),
    }
}

/// DISHES, as the bundle holds them.
pub static DISHES: LazyLock<Vec<Dish>> = LazyLock::new(|| {
    data::get(&m("DISHES"))
        .items()
        .iter()
        .map(|o| {
            let a = |k: &str| o.attr(k).cloned().unwrap_or(Value::None);
            Dish {
                name: a("name").str().to_string(),
                uses: a("uses").strs(),
                minutes: a("minutes").as_int().unwrap_or(0),
                diet: a("diet").str().to_string(),
                how: a("how").str().to_string(),
                favourite: a("favourite").truthy(),
                said: a("said").str().to_string(),
            }
        })
        .collect()
});

fn dish_named(name: &str) -> Option<&'static Dish> {
    DISHES.iter().find(|d| d.name == name)
}

fn str_pairs(name: &str) -> Vec<(String, String)> {
    data::get(&m(name)).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}
fn set(name: &str) -> HashSet<String> {
    data::set(&m(name))
}
fn rx(name: &str) -> &'static Re {
    data::re(&m(name))
}
fn string(name: &str) -> String {
    data::string(&m(name))
}
fn any_in(text: &str, words: &[String]) -> bool {
    words.iter().any(|w| text.contains(w.as_str()))
}

/// INGREDIENTS: the canonical name and the forms a line says it with, in order.
static INGREDIENTS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&m("INGREDIENTS")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static IN_A_DISH: LazyLock<HashSet<String>> = LazyLock::new(|| set("IN_A_DISH"));
static FISH_INGREDIENTS: LazyLock<Vec<String>> = LazyLock::new(|| strs("FISH_INGREDIENTS"));
static WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("WORD"));
static DISH_STYLE: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISH_STYLE"));
static STYLE_WANT: LazyLock<Vec<String>> = LazyLock::new(|| strs("STYLE_WANT"));
static STYLE_CUES: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&m("STYLE_CUES")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static CARRY_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| strs("CARRY_REPLIES"));
static CHEAP_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("CHEAP_ASK"));
static LIGHT_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("LIGHT_ASK"));
static LIGHT_SET: LazyLock<Vec<String>> = LazyLock::new(|| strs("LIGHT_SET"));
static SMALL_PEOPLE: LazyLock<Vec<String>> = LazyLock::new(|| strs("SMALL_PEOPLE"));
static CHILD_AT_TABLE: LazyLock<Vec<String>> = LazyLock::new(|| strs("CHILD_AT_TABLE"));
static EVENT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("EVENT_WORDS"));
static BEFORE_EVENT: LazyLock<Vec<String>> = LazyLock::new(|| strs("BEFORE_EVENT"));
static MEAL_WORD: LazyLock<Vec<String>> = LazyLock::new(|| strs("MEAL_WORD"));
static CHEAP: LazyLock<HashSet<String>> = LazyLock::new(|| set("CHEAP"));
static FOOD_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("FOOD_WORDS"));
static NO_MEAT: LazyLock<Vec<String>> = LazyLock::new(|| strs("NO_MEAT"));
static NO_CHEESE: LazyLock<Vec<String>> = LazyLock::new(|| strs("NO_CHEESE"));
static NO_CUE: LazyLock<Vec<String>> = LazyLock::new(|| strs("NO_CUE"));
static NO_GAP: LazyLock<Vec<String>> = LazyLock::new(|| strs("NO_GAP"));

/// A dish a tight month can carry: nothing bought fresh and skippable in it.
pub fn cheap(item: &Dish) -> bool {
    item.uses.iter().all(|u| CHEAP.contains(u))
}

/// The food a line names, preferring the kind this table knows, or None.
pub fn named_food(words: &[String], understood: Option<&Understood>) -> Option<String> {
    if let Some(up) = _up_food(words) {
        return Some(up);
    }
    let named = match understood {
        Some(u) => _named(u),
        None => Vec::new(),
    };
    named.into_iter().next().filter(|n| !n.is_empty())
}

static OPENERS: LazyLock<Vec<String>> = LazyLock::new(|| strs("OPENERS"));
static TIMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("TIMES"));
static TOTAL_TIMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("TOTAL_TIMES"));
static COOK_TIME: LazyLock<&'static Re> = LazyLock::new(|| rx("COOK_TIME"));
static FOLLOW_ON: LazyLock<Vec<String>> = LazyLock::new(|| strs("FOLLOW_ON"));
static FOLLOW_ON_NO_CHEESE: LazyLock<String> = LazyLock::new(|| string("FOLLOW_ON_NO_CHEESE"));

/// One of options, varied by the line itself rather than by the turn count alone.
pub fn _variant(options: &[String], understood: &Understood, memory: &Memory) -> String {
    let seed = memory.turns.len() as i128 * 7919 + crate::py::random::ord_sum(&understood.clean);
    crate::py::random::Random::new(seed).choice(options).clone()
}

static NOT_A_MEAL_SUBJECT: LazyLock<Vec<String>> = LazyLock::new(|| strs("NOT_A_MEAL_SUBJECT"));
static DESSERT_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("DESSERT_ASK"));
static MAKING_AT_HOME: LazyLock<Vec<String>> = LazyLock::new(|| strs("MAKING_AT_HOME"));
static TECHNIQUE_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("TECHNIQUE_ASK"));
static USE_UP_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("USE_UP_ASK"));
static NO_HAVE: LazyLock<Vec<String>> = LazyLock::new(|| strs("NO_HAVE"));
static NO_HAVE_GAP: LazyLock<String> = LazyLock::new(|| string("NO_HAVE_GAP"));
static DOUBLE_NO: LazyLock<Vec<String>> = LazyLock::new(|| strs("DOUBLE_NO"));

/// The "no X" pattern for one cue and one ingredient, compiled once.
pub fn _no_have(cue: &str, shape: &str) -> &'static Re {
    let pattern = format!(r"(?<![a-z]){}{} {}(?![a-z])", sh::escape(cue), NO_HAVE_GAP.as_str(), sh::escape(shape));
    sh::re_cached(&pattern, 0)
}

fn is_alpha(s: &str) -> bool {
    !s.is_empty() && s.chars().all(char::is_alphabetic)
}

/// The ingredients the line rules out: "anything but pasta", "ive had enough rice".
pub fn _ruled_out(text: &str) -> BTreeSet<String> {
    let mut plain = format!(" {} ", WORD.findall_str(&text.replace('\'', "")).join(" "));
    for phrase in DOUBLE_NO.iter() {
        let padded = format!(" {phrase} ");
        if plain.contains(&padded) {
            plain = plain.replace(&padded, " ");
        }
    }
    let mut out = BTreeSet::new();
    for (ingredient, forms) in INGREDIENTS.iter() {
        for form in forms {
            if !is_alpha(form) && !form.contains(' ') {
                continue;
            }
            let shape = if is_alpha(form) { form.clone() } else { form.replace('\'', "") };
            if !plain.contains(&format!(" {shape} ")) {
                continue;
            }
            let mut hit = false;
            for cue in NO_CUE.iter() {
                if NO_GAP.iter().any(|gap| plain.contains(&format!(" {cue}{gap} {shape} "))) {
                    out.insert(ingredient.clone());
                    hit = true;
                    break;
                }
            }
            if !hit {
                for cue in NO_HAVE.iter() {
                    if _no_have(cue, &shape).is_match(&plain) {
                        out.insert(ingredient.clone());
                        break;
                    }
                }
            }
        }
    }
    out
}

/// True when it is fish itself that is out, not one fish named by itself.
pub fn _the_fish_are_out(clean: &str, diet: &str) -> bool {
    let plain = format!(" {} ", WORD.findall_str(&clean.replace('\'', "")).join(" "));
    for cue in NO_CUE.iter() {
        for family in ["fish", "seafood"] {
            if plain.contains(&format!(" {cue} {family} ")) {
                return true;
            }
        }
    }
    let diet = diet.to_lowercase();
    diet.contains("fish") || diet.contains("seafood")
}

/// Whether a food word in the line is one of the foods the line rules out.
pub fn _is_the_ruled_out_food(word: &str, text: &str) -> bool {
    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return false;
    }
    for ingredient in _ruled_out(text) {
        let forms = INGREDIENTS.iter().find(|(k, _)| *k == ingredient).map(|(_, f)| f.clone()).unwrap_or_default();
        for form in forms.iter().chain(std::iter::once(&ingredient)) {
            let form = form.trim().to_lowercase();
            if form == word || form.starts_with(&word) || word.starts_with(&form) {
                return true;
            }
        }
    }
    false
}

/// The canonical ingredients the line names, in the order they are mentioned.
pub fn _named(understood: &Understood) -> Vec<String> {
    let mut words: Vec<String> = understood.words.clone();
    for word in WORD.findall_str(&understood.norm_text) {
        if !words.contains(&word) {
            words.push(word);
        }
    }
    let mut text = words.join(" ");
    let mut found: Vec<String> = Vec::new();
    for (ingredient, forms) in INGREDIENTS.iter() {
        let hit: Vec<&String> = forms.iter().filter(|f| !is_alpha(f) && text.contains(f.as_str())).collect();
        if !hit.is_empty() {
            found.push(ingredient.clone());
            for phrase in hit {
                text = text.replace(phrase.as_str(), " ");
                words.retain(|w| !phrase.contains(w.as_str()));
            }
        }
    }
    for (ingredient, forms) in INGREDIENTS.iter() {
        if found.contains(ingredient) {
            continue;
        }
        if forms.iter().any(|f| is_alpha(f) && words.contains(f)) {
            found.push(ingredient.clone());
        }
    }
    found
}

/// The minutes a line allows, from "20 minutes" or "half an hour", else None.
pub fn _clock(numbers: &[crate::understood::Number]) -> Option<i64> {
    for number in numbers {
        if !matches!(number.unit.as_deref(), Some("minutes" | "minute" | "min" | "mins")) {
            continue;
        }
        let value = match &number.value {
            Some(crate::understood::Num::Int(i)) => *i,
            Some(crate::understood::Num::Float(f)) if f.is_finite() => f.trunc() as i64,
            _ => continue,
        };
        if value > 0 {
            return Some(value);
        }
    }
    None
}

static NO_MEAT_BLOCKER: LazyLock<Vec<String>> = LazyLock::new(|| strs("_NO_MEAT_BLOCKER"));

/// A line that rules meat out, in whatever order the words come.
pub fn _no_meat(text: &str) -> bool {
    if any_in(text, &NO_MEAT) {
        return true;
    }
    let words: Vec<&str> = text.split_whitespace().map(|w| w.trim_matches(|c| ",.!?;:".contains(c))).collect();
    for cue in NO_CUE.iter() {
        let cue_words: Vec<&str> = cue.split_whitespace().collect();
        let width = cue_words.len();
        if words.len() + 1 < width + 1 {
            continue;
        }
        for start in 0..(words.len() + 1 - width) {
            let mut same = words[start..start + width] == cue_words[..];
            if !same && width == 1 && ["nothing", "none", "nobody"].contains(&words[start]) {
                same = true;
            }
            if !same {
                continue;
            }
            let after = &words[start + width..(start + width + 5).min(words.len())];
            if after.iter().any(|w| NO_MEAT_BLOCKER.iter().any(|b| b == w)) {
                continue;
            }
            if after.contains(&"meat") || after.contains(&"chicken") {
                return true;
            }
        }
    }
    false
}

/// The meal plan this request has built up so far (`memory["meal_plan"]`), when it is a dict.
fn meal_plan(memory: &Memory) -> Option<&Value> {
    memory.extra.get("meal_plan").filter(|v| matches!(v, Value::Dict(_)))
}

/// `plan.get(key)` on the meal plan (None when there is no plan or no key).
fn plan_get<'a>(memory: &'a Memory, key: &str) -> Option<&'a Value> {
    meal_plan(memory).and_then(|p| p.get(key))
}

/// `plan.get(key) or ()` as strings.
fn plan_strs(memory: &Memory, key: &str) -> Vec<String> {
    plan_get(memory, key).map(|v| v.strs()).unwrap_or_default()
}

/// `memory.get("user")` when it is a dict.
fn user_record(memory: &Memory) -> Option<&Value> {
    memory.extra.get("user").filter(|v| matches!(v, Value::Dict(_)))
}

fn value_str(v: Option<&Value>) -> String {
    match v {
        Some(Value::Str(s)) => s.to_string(),
        Some(Value::None) | None => String::new(),
        Some(other) if !other.truthy() => String::new(),
        Some(Value::Int(i)) => i.to_string(),
        Some(Value::Bool(b)) => if *b { "True".into() } else { "False".into() },
        Some(other) => other.str().to_string(),
    }
}

/// A limit said after an idea means something only when an idea is on the table or the line
/// is about food at all.
pub fn _later(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if plan_get(memory, "offered").is_some_and(|v| v.truthy()) {
        return suggest(memory, understood);
    }
    let text = understood.words.join(" ");
    if any_in(&text, &FOOD_WORDS)
        || ["dinner", "tea", "supper", "lunch", "meal", "recipe", "cook", "eating", "eat", "food"].iter().any(|c| text.contains(c))
    {
        return suggest(memory, understood);
    }
    None
}

static LUNCH_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("LUNCH_ASK"));
static IDEA_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("IDEA_ASK"));
static QUICK: LazyLock<Vec<String>> = LazyLock::new(|| strs("QUICK"));
static BEDTIME: LazyLock<Vec<String>> = LazyLock::new(|| strs("BEDTIME"));
static NO_DAIRY: LazyLock<Vec<String>> = LazyLock::new(|| strs("NO_DAIRY"));

/// The diets a dish may be (None: any), the ingredients to leave out, and the minutes allowed.
pub type Limits = (Option<BTreeSet<String>>, BTreeSet<String>, Option<i64>);

fn diets(names: &[&str]) -> Option<BTreeSet<String>> {
    Some(names.iter().map(|s| s.to_string()).collect())
}

/// What this line rules out: (allowed diets or None, ingredients to avoid, max minutes).
pub fn _limits(understood: &Understood, memory: &Memory) -> Limits {
    let clean = understood.clean.as_str();
    let text = understood.words.join(" ");
    let mut allowed: Option<BTreeSet<String>> = None;
    let mut without: BTreeSet<String> = BTreeSet::new();
    let said_now: HashSet<String> = _named(understood).into_iter().collect();

    if _no_meat(clean) {
        allowed = diets(&["vegan", "vegetarian"]);
    }
    if text.contains("vegan") || any_in(clean, &NO_DAIRY) {
        allowed = diets(&["vegan"]);
    }
    if clean.contains("no fish") || clean.contains("without fish") || clean.contains("no seafood") {
        let mut a = allowed.clone().unwrap_or_else(|| diets(&["vegan", "vegetarian", "fish"]).unwrap());
        a.remove("fish");
        allowed = Some(a);
    }
    if any_in(clean, &NO_CHEESE) {
        without.insert("cheese".into());
    }
    without.extend(_ruled_out(clean));
    let mut minutes = _clock(&understood.numbers);
    if minutes.is_none() && (text.contains("quicker") || text.contains("faster")) {
        let was = value_str(plan_get(memory, "pick"));
        let mut order: Vec<String> = if was.is_empty() { Vec::new() } else { vec![was] };
        order.extend(plan_strs(memory, "offered").into_iter().rev());
        for name in order {
            if let Some(item) = dish_named(&name) {
                minutes = Some(item.minutes - 1);
                break;
            }
        }
    }
    if minutes.is_none()
        && QUICK.iter().any(|w| text.contains(w.as_str()))
        && (meal_plan(memory).is_some_and(|p| p.truthy())
            || any_in(&text, &FOOD_WORDS)
            || ["dinner", "tea", "supper", "lunch", "meal", "recipe", "eating", "cook"].iter().any(|c| text.contains(c)))
    {
        minutes = Some(20);
    }
    if minutes.is_none() && (clean.contains("half an hour") || clean.contains("30 minutes")) {
        minutes = Some(30);
    }
    if any_in(clean, &BEDTIME) {
        minutes = Some(minutes.filter(|&m| m != 0).unwrap_or(15).min(15));
    }

    let about = user_record(memory);
    let diet = value_str(about.and_then(|a| a.get("diet"))).to_lowercase();
    if allowed.is_none() && !diet.is_empty() {
        if diet.contains("vegan") {
            allowed = diets(&["vegan"]);
        } else if diet.contains("vegetarian") {
            allowed = diets(&["vegan", "vegetarian"]);
        }
    }
    if diet.contains("dairy") || diet.contains("lactose") || diet.contains("cheese") {
        without.insert("cheese".into());
    }
    if _the_fish_are_out(clean, &diet) {
        without.extend(FISH_INGREDIENTS.iter().cloned());
    }
    if let Some(allergies) = about.and_then(|a| a.get("allergies")) {
        for allergy in allergies.items() {
            let name = value_str(Some(allergy)).to_lowercase();
            let name = name.trim_end_matches('s');
            if INGREDIENTS.iter().any(|(k, _)| k == name) {
                without.insert(name.to_string());
            }
        }
    }

    if let Some(others) = memory.extra.get("others") {
        for person in others.items() {
            let theirs = if matches!(person, Value::Dict(_)) { value_str(person.get("diet")).to_lowercase() } else { String::new() };
            if theirs.is_empty() {
                continue;
            }
            if theirs.contains("vegan") || theirs.contains("plant") {
                allowed = diets(&["vegan"]);
            } else if (theirs.contains("vegetarian") || _no_meat(&theirs)) && allowed.is_none() {
                allowed = diets(&["vegan", "vegetarian"]);
            }
            if theirs.contains("dairy") || theirs.contains("cheese") {
                without.insert("cheese".into());
            }
            if theirs.contains("fish") || theirs.contains("seafood") {
                for f in ["fish", "tuna", "salmon", "prawn", "cod"] {
                    without.insert(f.into());
                }
            }
        }
    }

    if allowed.is_none() || without.is_empty() {
        let turns = &memory.turns;
        let start = turns.len().saturating_sub(6);
        for turn in turns[start..].iter().rev() {
            let said = turn.user.to_lowercase();
            if said.is_empty() {
                continue;
            }
            if (_no_meat(&said) || said.contains("vegan")) && !any_in(&said, &IDEA_ASK) {
                if allowed.is_none() && _no_meat(&said) {
                    allowed = diets(&["vegan", "vegetarian"]);
                }
                if said.contains("vegan") {
                    allowed = diets(&["vegan"]);
                }
            }
            if any_in(&said, &NO_CHEESE) || said.contains("no dairy") {
                without.insert("cheese".into());
            }
            for gone in _ruled_out(&said) {
                if !said_now.contains(&gone) {
                    without.insert(gone);
                }
            }
        }
    }

    if meal_plan(memory).is_some() {
        let wanted: Vec<String> = _named(understood).into_iter().filter(|n| !without.contains(n)).collect();
        let plan_turn = plan_get(memory, "turn").and_then(|v| v.as_int()).unwrap_or(0);
        if wanted.is_empty() && memory.turns.len() as i64 - plan_turn <= 3 {
            if allowed.is_none() {
                if let Some(a) = plan_get(memory, "allowed").filter(|v| v.truthy()) {
                    allowed = Some(a.strs().into_iter().collect());
                }
            }
            without.extend(plan_strs(memory, "without"));
            if minutes.is_none() {
                minutes = plan_get(memory, "minutes").and_then(|v| v.as_int());
            }
        }
    }
    (allowed, without, minutes)
}

/// True when a dish keeps to the diets, the left-out ingredients and the clock (plus grace).
pub fn _fits(item: &Dish, allowed: &Option<BTreeSet<String>>, without: &BTreeSet<String>, minutes: Option<i64>, grace: i64) -> bool {
    if let Some(a) = allowed {
        if !a.contains(&item.diet) {
            return false;
        }
    }
    if !without.is_empty() && item.uses.iter().any(|u| without.contains(u)) {
        return false;
    }
    if let Some(m) = minutes {
        if item.minutes > m + grace {
            return false;
        }
    }
    true
}

/// The list turned to start at a place that follows the line, so the same ask varies.
pub fn _rotate<T: Clone>(items: &[T], understood: &Understood, memory: &Memory) -> Vec<T> {
    if items.is_empty() {
        return Vec::new();
    }
    let seed = memory.turns.len() as i128 * 7919 + crate::py::random::ord_sum(&understood.clean);
    let start = (seed.rem_euclid(items.len() as i128)) as usize;
    items[start..].iter().chain(items[..start].iter()).cloned().collect()
}

static GRACE: LazyLock<i64> = LazyLock::new(|| data::get(&m("GRACE")).as_int().unwrap_or(10));
static HARD_CLOCK: LazyLock<Vec<String>> = LazyLock::new(|| strs("HARD_CLOCK"));
static STAPLE: LazyLock<HashSet<String>> = LazyLock::new(|| set("STAPLE"));
static DISH_SHAPES: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISH_SHAPES"));

/// Three dishes, quickest first: the ten-minute one, the twenty-minute one, the slower one.
pub fn _spread(ranked: &[&'static Dish]) -> Vec<&'static Dish> {
    let mut picks: Vec<&'static Dish> = Vec::new();
    let mut shapes: HashSet<String> = HashSet::new();
    for (low, high) in [(0, 15), (16, 25), (26, 95)] {
        let fits: Vec<&'static Dish> = ranked.iter().copied().filter(|i| low <= i.minutes && i.minutes <= high).collect();
        let distinct: Vec<&'static Dish> = fits.iter().copied().filter(|i| !shapes.contains(&_shape(i))).collect();
        for pool in [&distinct, &fits] {
            if !pool.is_empty() {
                let favs: Vec<&'static Dish> = pool.iter().copied().filter(|i| i.favourite).collect();
                let chosen = if favs.is_empty() { pool.clone() } else { favs };
                picks.push(chosen[0]);
                shapes.insert(_shape(chosen[0]));
                break;
            }
        }
    }
    picks
}

/// The shape of a dish ("soup", "curry", ...), so three picks are three different plans.
pub fn _shape(item: &Dish) -> String {
    let squashed = item.name.replace('-', " ").replace(' ', "");
    for shape in DISH_SHAPES.iter() {
        if squashed.contains(&shape.replace(' ', "")) {
            return shape.clone();
        }
    }
    String::new()
}

/// The dishes in the order to offer them: what they named first, then fewest extras, quickest.
pub fn _best(
    names: &[String],
    allowed: &Option<BTreeSet<String>>,
    without: &BTreeSet<String>,
    minutes: Option<i64>,
    memory: &Memory,
    understood: &Understood,
) -> Vec<&'static Dish> {
    let mut offered: HashSet<String> = plan_strs(memory, "offered").into_iter().collect();
    if meal_plan(memory).is_some() {
        let turn = plan_get(memory, "turn").and_then(|v| v.as_int()).unwrap_or(-1);
        if turn >= memory.turns.len() as i64 {
            offered.clear();
        }
    }
    let said = understood.words.join(" ");
    let mut style: Vec<String> = DISH_STYLE.iter().filter(|c| said.contains(c.as_str())).cloned().collect();
    for (word, shapes) in STYLE_CUES.iter() {
        if said.contains(word.as_str()) {
            for shape in shapes {
                if !style.contains(shape) {
                    style.push(shape.clone());
                }
            }
            style.retain(|shape| DISH_STYLE.contains(shape) || DISHES.iter().any(|i| i.name.contains(shape.as_str())));
        }
    }
    let named: HashSet<&String> = names.iter().collect();
    let uses_named = |i: &Dish| i.uses.iter().any(|u| named.contains(u));
    let words: HashSet<&String> = understood.words.iter().collect();
    let hard = HARD_CLOCK.iter().any(|h| words.contains(h));
    let grace = if !named.is_empty() && !hard { *GRACE } else { 0 };
    let all: Vec<&'static Dish> = DISHES.iter().collect();
    let mut pool: Vec<&'static Dish> = all.iter().copied().filter(|i| _fits(i, allowed, without, minutes, grace)).collect();
    if pool.is_empty() {
        pool = all.iter().copied().filter(|i| _fits(i, allowed, without, None, 0)).collect();
        if pool.is_empty() {
            pool = all.clone();
        }
    }
    let across = _said_about(memory, &said);
    if any_in(&across, &CHEAP_ASK) {
        let mut thrifty: Vec<&'static Dish> = pool.iter().copied().filter(|i| cheap(i)).collect();
        if thrifty.is_empty() {
            thrifty = all.iter().copied().filter(|i| cheap(i)).collect();
        }
        if !thrifty.is_empty() {
            pool = thrifty;
        }
    }
    if any_in(&across, &LIGHT_ASK) {
        let light: Vec<&'static Dish> = pool.iter().copied().filter(|i| LIGHT_SET.contains(&i.name)).collect();
        if !light.is_empty() {
            pool = light;
        }
    }
    if !style.is_empty() {
        let named_shape = DISH_STYLE.iter().any(|c| said.contains(c.as_str()));
        let styled: Vec<&'static Dish> = all
            .iter()
            .copied()
            .filter(|i| {
                let spaced = i.name.replace('-', " ");
                style.iter().any(|c| spaced.contains(c.as_str()))
                    && _fits(i, allowed, without, if named_shape { None } else { minutes }, grace)
            })
            .collect();
        if !styled.is_empty() {
            pool = styled;
        }
    }
    if !named.is_empty() {
        let mut using: Vec<&'static Dish> = pool.iter().copied().filter(|i| uses_named(i)).collect();
        if using.is_empty() {
            using = all.iter().copied().filter(|i| uses_named(i) && _fits(i, allowed, without, None, 0)).collect();
        }
        if !using.is_empty() {
            pool = using;
        }
    }
    if !offered.is_empty() && pool.iter().all(|i| offered.contains(&i.name)) {
        let fresh: Vec<&'static Dish> = all.iter().copied().filter(|i| !offered.contains(&i.name)).collect();
        let still: Vec<&'static Dish> = fresh.iter().copied().filter(|i| uses_named(i)).collect();
        if !named.is_empty() && !fresh.is_empty() && still.is_empty() {
            let kept: Vec<&'static Dish> = pool.iter().copied().filter(|i| uses_named(i)).collect();
            if !kept.is_empty() {
                pool = kept;
            }
        } else {
            let mut wider: Vec<&'static Dish> = fresh.iter().copied().filter(|i| uses_named(i) && _fits(i, allowed, without, minutes, 0)).collect();
            if wider.is_empty() {
                wider = fresh.iter().copied().filter(|i| uses_named(i)).collect();
            }
            if wider.is_empty() {
                wider = fresh.iter().copied().filter(|i| _fits(i, allowed, without, minutes, 0)).collect();
            }
            if wider.is_empty() {
                wider = fresh.clone();
            }
            if !wider.is_empty() {
                pool = wider;
            }
        }
    }
    if named.is_empty() {
        let favourites: Vec<&'static Dish> = pool.iter().copied().filter(|i| i.favourite).collect();
        let others: Vec<&'static Dish> = pool.iter().copied().filter(|i| !i.favourite).collect();
        let mut order = _rotate(&favourites, understood, memory);
        order.extend(_rotate(&others, understood, memory));
        order.sort_by_key(|i| _best__lambda_1421(i, &offered));
        return order;
    }
    let mut pool = pool;
    pool.sort_by_key(|i| _best__rank(i, names, &offered));
    pool
}

/// `_best`'s sort key with nothing named: a dish already offered goes to the back.
pub fn _best__lambda_1421(item: &Dish, offered: &HashSet<String>) -> bool {
    offered.contains(&item.name)
}

/// `_best.<locals>.rank`: offered last, most named used, the first named, fewest extras
/// (non-staples first), named in its title, quickest.
pub fn _best__rank(item: &Dish, names: &[String], offered: &HashSet<String>) -> (bool, i64, bool, usize, usize, bool, i64) {
    let named: HashSet<&String> = names.iter().collect();
    let uses: HashSet<&String> = item.uses.iter().collect();
    let used = uses.iter().filter(|u| named.contains(*u)).count() as i64;
    let extra: Vec<&&String> = uses.iter().filter(|u| !named.contains(**u)).collect();
    let lead = !names.is_empty() && item.uses.contains(&names[0]);
    let titled = names.iter().any(|w| item.name.contains(w.as_str()));
    (
        offered.contains(&item.name),
        -used,
        !lead,
        extra.iter().filter(|e| !STAPLE.contains(***e)).count(),
        extra.len(),
        !titled,
        item.minutes,
    )
}

static UNCOUNTABLE: LazyLock<HashSet<String>> = LazyLock::new(|| set("UNCOUNTABLE"));

/// The ingredients as a person would say them: "chickpeas and tomatoes", not "chickpea".
pub fn _prose(names: &[String]) -> String {
    let mut sorted: Vec<&String> = names.iter().collect();
    sorted.sort();
    let said: Vec<String> = sorted
        .into_iter()
        .map(|n| {
            if UNCOUNTABLE.contains(n) {
                n.clone()
            } else if n == "potato" || n == "tomato" {
                format!("{n}es")
            } else {
                format!("{n}s")
            }
        })
        .collect();
    if said.len() > 1 {
        return format!("{} and {}", said[..said.len() - 1].join(", "), said[said.len() - 1]);
    }
    said.into_iter().next().unwrap_or_default()
}

/// The dish just offered, if it is what uses the ingredient they still have.
pub fn _stick(memory: &Memory, names: &[String], allowed: &Option<BTreeSet<String>>, without: &BTreeSet<String>) -> Option<&'static Dish> {
    for name in plan_strs(memory, "offered").iter().rev() {
        if let Some(item) = dish_named(name) {
            if item.uses_any(names) && _fits(item, allowed, without, None, 0) {
                return Some(item);
            }
        }
    }
    None
}

static DAIRY_EXTRAS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DAIRY_EXTRAS"));

/// The method without its dairy extra, when dairy is ruled out.
pub fn _without_dairy(text: &str, without: &BTreeSet<String>) -> String {
    if !without.contains("cheese") && !without.contains("milk") {
        return text.to_string();
    }
    let mut text = text.to_string();
    for extra in DAIRY_EXTRAS.iter() {
        text = text.replace(extra.as_str(), "");
    }
    text
}

/// The tail that fits the method: a total when the method already named its own cook time.
pub fn _times(how: &str) -> &'static [String] {
    if COOK_TIME.is_match(how) {
        &TOTAL_TIMES
    } else {
        &TIMES
    }
}

/// `s[0].upper() + s[1:]` (Python raises on ""; the methods are never empty).
fn upper_first(s: &str) -> String {
    crate::py::text::upper_first(s)
}

/// The dish said as an answer: an opener, the dish, its method and the time it takes.
pub fn _describe(item: &Dish, understood: &Understood, memory: &Memory, without: &BTreeSet<String>) -> String {
    let last = memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default();
    let mut openers: Vec<String> = OPENERS.iter().filter(|o| !last.starts_with(o.as_str())).cloned().collect();
    if openers.is_empty() {
        openers = OPENERS.clone();
    }
    let opener = _variant(&openers, understood, memory);
    let when = _variant(_times(&item.how), understood, memory).replace("{n}", &item.minutes.to_string());
    let how = upper_first(&_without_dairy(&item.how, without));
    format!("{opener} {}: {how}. {when}", item.said)
}

static USE_UP: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&m("USE_UP"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].strs(), p[1].str().to_string())
        })
        .collect()
});

fn vstr(s: &str) -> Value {
    Value::Str(std::sync::Arc::from(s))
}
fn vlist(items: &[String]) -> Value {
    Value::List(std::sync::Arc::new(items.iter().map(|s| vstr(s)).collect()))
}

/// Remember that this food was offered for using up (the meal plan a use-up answer leaves).
fn plan_use_up(memory: &mut Memory, said: &str) {
    let plan = vec![
        (vstr("turn"), Value::Int(memory.turns.len() as i64)),
        (vstr("ingredients"), vlist(&[])),
        (vstr("allowed"), Value::None),
        (vstr("without"), vlist(&[])),
        (vstr("minutes"), Value::None),
        (vstr("offered"), vlist(&[said.to_string()])),
    ];
    memory.set("meal_plan", Value::Dict(std::sync::Arc::new(plan)));
}

/// What to do with an ingredient they already have cooked: "leftover roast chicken".
pub fn use_up(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = understood.words.clone();
    let text = words.join(" ");
    let up = _up_food(&words);
    for (forms, how) in USE_UP.iter() {
        if let Some(u) = &up {
            if !forms.contains(u) {
                continue;
            }
        }
        let mut hit: Vec<&String> = forms.iter().filter(|f| words.contains(f)).collect();
        if hit.is_empty() {
            hit = forms.iter().filter(|f| text.contains(f.as_str())).collect();
        }
        if let Some(said) = hit.first() {
            let said = said.to_string();
            plan_use_up(memory, &said);
            let mut cs = how.chars();
            let first = cs.next().map(|c| c.to_lowercase().collect::<String>()).unwrap_or_default();
            return Some(format!("Leftover {said}: {first}{}.", cs.as_str()));
        }
    }
    let said = match up {
        Some(u) => u,
        None => _named(understood).into_iter().next()?,
    };
    let idea = dup::cooking::pantry_idea(&said).unwrap_or_default().to_lowercase();
    if USE_DISH_WORDS.iter().any(|w| idea.contains(w.as_str())) {
        return None;
    }
    plan_use_up(memory, &said);
    Some(use_it_up(&said))
}

fn r_use_up(_f: &FnRef, mem: &mut Memory, u: &Understood) -> Option<String> {
    use_up(mem, u)
}

/// Remember what was offered and the limits set, so the next turn can build on it.
pub fn _plan(
    memory: &mut Memory,
    names: &[String],
    allowed: &Option<BTreeSet<String>>,
    without: &BTreeSet<String>,
    minutes: Option<i64>,
    offered: &[String],
) {
    let earlier_ingredients = plan_strs(memory, "ingredients");
    let earlier_allowed = plan_get(memory, "allowed").cloned().unwrap_or(Value::None);
    let mut all_offered = plan_strs(memory, "offered");
    let earlier_pick = value_str(plan_get(memory, "pick"));
    all_offered.extend(offered.iter().cloned());
    let start = all_offered.len().saturating_sub(4);
    let plan = vec![
        (vstr("turn"), Value::Int(memory.turns.len() as i64)),
        (vstr("ingredients"), vlist(if names.is_empty() { &earlier_ingredients } else { names })),
        (
            vstr("allowed"),
            match allowed {
                Some(a) => vlist(&a.iter().cloned().collect::<Vec<_>>()),
                None => earlier_allowed,
            },
        ),
        (vstr("without"), vlist(&without.iter().cloned().collect::<Vec<_>>())),
        (vstr("minutes"), minutes.map(Value::Int).unwrap_or(Value::None)),
        (vstr("offered"), vlist(&all_offered[start..])),
        (vstr("pick"), vstr(offered.first().map(String::as_str).unwrap_or(&earlier_pick))),
    ];
    memory.set("meal_plan", Value::Dict(std::sync::Arc::new(plan)));
}

static ASK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("ASK_WORDS"));
static DIET_SAID: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&m("DIET_SAID"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].str().to_string(), p[1].str().to_string())
        })
        .collect()
});

/// A picnic or a day out: what to pack, not what to cook.
pub fn _carry(memory: &Memory, understood: &Understood) -> Option<String> {
    let line = understood.clean.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| line.contains(w));
    if !has(&[
        "make", "take", "bring", "pack", "packing", "food", "eat", "eating", "lunch", "idea", "ideas", "need", "snack", "snacks",
        "something", "anything", "for a picnic", "on a picnic", "day out", "veggie", "vegetarian", "vegan", "no meat", "gluten", "kid",
        "kids", "children", "child", "toddler", "people", "family", "friends",
    ]) {
        return None;
    }
    if has(&["went to", "was at", "we had", "yesterday", "last week"]) {
        return None;
    }
    if has(&[
        "flight", "flights", "plane", "airport", "airline", "cancelled", "canceled", "delayed", "delay", "refund", "owed",
        "compensation", "lighter", "luggage", "baggage", "security", "passport", "check in", "check-in",
    ]) && !has(&[
        "food", "eat", "eating", "lunch", "snack", "snacks", "picnic", "sandwich", "meal", "meals", "drink", "drinks", "breakfast",
        "dinner", "packed",
    ]) {
        return None;
    }
    if has(&["packed lunch", "lunchbox", "lunch box", "school lunch", "bento"]) {
        return None;
    }
    if has(&["flight", "flights", "plane", "airport", "airline", "security"])
        && has(&[
            "can i take", "can i bring", "can we take", "can we bring", "could i take", "could i bring", "am i allowed",
            "are we allowed", "allowed to take", "allowed to bring", "allowed on", "100ml", "liquids",
        ])
    {
        return None;
    }
    let mut reply = _variant(&CARRY_REPLIES, understood, memory);
    if has(&["veggie", "vegetarian", "vegan", "no meat"]) {
        reply = reply.replace("cold sausages or sausage rolls, ", "").replace("sausage rolls, ", "");
    }
    if has(&["kids", "children", "child", "toddler", "little ones"]) {
        reply.push_str(" For small people, add something they will actually eat: breadsticks, cheese.");
    }
    Some(reply)
}

fn r_carry(_f: &FnRef, mem: &mut Memory, u: &Understood) -> Option<String> {
    _carry(mem, u)
}

/// cooking.py's answer for a kitchen with no oven, hob or microwave, or None.
pub fn _kitchen_answer(understood: &Understood) -> Option<String> {
    dup::cooking::without(understood).filter(|a| !a.trim().is_empty())
}

static GLUTEN: LazyLock<Vec<String>> = LazyLock::new(|| strs("GLUTEN"));

/// The line and the last few turns, lowercased, for cues that hold across turns.
pub fn _said_about(memory: &Memory, clean: &str) -> String {
    let mut said = vec![clean.to_string()];
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(4);
    for turn in &turns[start..] {
        if !turn.user.is_empty() {
            said.push(turn.user.to_lowercase());
        }
    }
    said.join(" ")
}

/// The diet word the person used about themselves, if they used one.
pub fn _diet_said(memory: &Memory, understood: Option<&Understood>) -> String {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(6);
    let mut said = turns[start..].iter().map(|t| t.user.to_lowercase()).collect::<Vec<_>>().join(" ");
    if let Some(u) = understood {
        if !u.clean.is_empty() {
            said.push(' ');
            said.push_str(&u.clean.to_lowercase());
        }
    }
    let diet = value_str(user_record(memory).and_then(|a| a.get("diet"))).to_lowercase();
    let said = format!("{diet} {said}").trim().to_string();
    for (cue, reads) in DIET_SAID.iter() {
        if said.contains(cue.as_str()) {
            return reads.clone();
        }
    }
    String::new()
}

/// A sentence they were told a turn ago is not an answer: say the clock instead.
pub fn _fresh(reply: &str, item: &Dish, memory: &Memory) -> String {
    static HEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=\.)\s+", 0));
    let last = memory.turns.iter().rev().find(|t| !t.bot.is_empty()).map(|t| t.bot.clone()).unwrap_or_default();
    if last.is_empty() {
        return reply.to_string();
    }
    let head = HEAD.split(reply).into_iter().next().unwrap_or_default();
    if !head.is_empty() && last.contains(&head) {
        return format!("{} takes about {} minutes.", crate::py::text::capitalize(&item.said), item.minutes);
    }
    reply.to_string()
}

static CROWD_SAID: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get(&m("CROWD_SAID")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
});
static CROWD_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("CROWD_ASK"));

/// What cooking.py says for a meal for a table, or None when it has nothing.
pub fn _crowd_answer(memory: &Memory, understood: &Understood) -> Option<String> {
    static CLOCK: LazyLock<Re> =
        LazyLock::new(|| Re::new(r"\bfor\s+\d{1,3}\s*(?:min|mins|minute|minutes|hour|hours|sec|seconds)\b", 0));
    let mut said = vec![understood.clean.clone()];
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(4);
    for turn in &turns[start..] {
        if !turn.user.is_empty() {
            said.push(turn.user.clone());
        }
    }
    let text = said.join(" ").to_lowercase();
    let text = CLOCK.sub(" ", &text);
    dup::cooking::_crowd_or_any(&text, &dup::cooking::_excluded(&text))
}

static COUNT_WORDS: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| {
    data::get(&m("COUNT_WORDS")).pairs().iter().map(|(k, v)| (k.as_int().unwrap_or(0), v.str().to_string())).collect()
});

/// The word for how many are eating, so the reply says "four" the way the line did.
pub fn _count_word(number: i64) -> String {
    COUNT_WORDS.iter().find(|(k, _)| *k == number).map(|(_, v)| v.clone()).unwrap_or_else(|| number.to_string())
}

static AGE_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| rx("_AGE_SHAPE"));

/// How many people are eating, when the line or the last few turns said so (0 when not).
pub fn _for_a_table(memory: &Memory, understood: &Understood) -> i64 {
    static FOR_N: LazyLock<Re> = LazyLock::new(|| {
        Re::new(r"\bfor\s+(\d{1,3})\b(?!\s*(?:min|mins|minute|minutes|hour|hours|sec|seconds)\b)", 0)
    });
    static ADULTS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\badults?\b", 0));
    let mut said = vec![understood.clean.to_lowercase()];
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(4);
    for turn in &turns[start..] {
        if !turn.user.is_empty() {
            said.push(turn.user.to_lowercase());
        }
    }
    let joined = said.join(" ");
    let kids_named = AGE_SHAPE.is_match(&joined);
    let text = AGE_SHAPE.sub(" ", &joined);
    let mut counts: Vec<i64> = FOR_N.findall_str(&text).iter().map(|n| n.parse().unwrap_or(0)).collect();
    for (word, value) in CROWD_SAID.iter() {
        if sh::re_search(&format!(r"\bfor\s+{word}\b(?!\s*(?:min|minute|hour))"), &text, 0)
            || sh::re_search(&format!(r"\b{word}\s+(?:people|of us|adults|guests|coming)\b"), &text, 0)
        {
            counts.push(*value);
        }
    }
    if any_in(&text, &CROWD_ASK) {
        counts.push(6);
    }
    if let Some(&best) = counts.iter().max() {
        if kids_named && ADULTS.is_match(&text) {
            return best + AGE_SHAPE.findall(&joined).len() as i64;
        }
        return best;
    }
    0
}

static KITCHEN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("KITCHEN_WORDS"));
static WITH_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("WITH_ASK"));
static RECIPE_FOR: LazyLock<&'static Re> = LazyLock::new(|| rx("RECIPE_FOR"));
static NOT_A_DISH_NAME: LazyLock<Vec<String>> = LazyLock::new(|| strs("NOT_A_DISH_NAME"));
static COMPANY: LazyLock<HashSet<String>> = LazyLock::new(|| set("COMPANY"));
static NOT_AN_INGREDIENT: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_AN_INGREDIENT"));

fn is_digits(w: &str) -> bool {
    !w.is_empty() && w.chars().all(|c| c.is_ascii_digit())
}

/// True when the words after "with" name the people eating, not ingredients.
pub fn _company(tail: &[String]) -> bool {
    tail.iter().any(|w| COMPANY.contains(w) || COMPANY.contains(w.trim_end_matches('s')))
}

fn known_word(word: &str, known: &HashSet<String>) -> bool {
    known.contains(word)
        || known.contains(word.trim_end_matches('s'))
        || known.contains(word.trim_end_matches(|c| c == 'e' || c == 's'))
}

/// True when the line asks what to make with a food the table has never heard of.
pub fn _unknown_ingredient(clean: &str, known: &HashSet<String>) -> bool {
    for found in WITH_ASK.finditer(clean) {
        let rest: Vec<String> = WORD
            .findall_str(found.str(1))
            .into_iter()
            .filter(|w| !KITCHEN_WORDS.contains(w) && !NOT_AN_INGREDIENT.contains(w) && !is_digits(w))
            .collect();
        if rest.is_empty() {
            return false;
        }
        if _company(&rest) {
            continue;
        }
        if !rest.iter().any(|w| known_word(w, known)) {
            return true;
        }
    }
    false
}

/// True for "a recipe for <dish>" where the dish is not food the tables know.
pub fn _unknown_dish_recipe(clean: &str, known: &HashSet<String>) -> bool {
    let Some(found) = RECIPE_FOR.search(clean) else { return false };
    let tail: Vec<String> = WORD
        .findall_str(found.str(1))
        .into_iter()
        .filter(|w| !KITCHEN_WORDS.contains(w) && !NOT_A_DISH_NAME.contains(w) && !is_digits(w))
        .collect();
    if tail.is_empty() {
        return false;
    }
    !tail.iter().any(|w| known_word(w, known))
}

static KNOWN_FOOD: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut known: HashSet<String> = FOOD_WORDS.iter().cloned().collect();
    for (k, forms) in INGREDIENTS.iter() {
        known.insert(k.clone());
        known.extend(forms.iter().cloned());
    }
    known
});

/// True when the line names food the dish table has no row for.
pub fn unknown_food(understood: &Understood) -> bool {
    let clean = dup::line_as_read(understood).to_lowercase();
    if _unknown_ingredient(&clean, &KNOWN_FOOD) {
        return true;
    }
    _unknown_dish_recipe(&clean, &KNOWN_FOOD)
}

static NOT_FOOD_LIMITS: LazyLock<Vec<String>> = LazyLock::new(|| strs("NOT_FOOD_LIMITS"));
static WITHOUT_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("WITHOUT_WORD"));

/// True when the line's "without X" is a thing this table cannot read as a food or a diet.
pub fn _something_it_has_not_got(understood: &Understood, allowed: &Option<BTreeSet<String>>, without: &BTreeSet<String>) -> bool {
    if allowed.is_some() || !without.is_empty() {
        return false;
    }
    let clean = understood.clean.to_lowercase();
    let Some(found) = WITHOUT_WORD.search(&clean) else { return false };
    let word = found.name_str("food").to_lowercase();
    !NOT_FOOD_LIMITS.contains(&word) && !FOOD_WORDS.contains(&word) && !FOOD_WORDS.iter().any(|f| f == word.trim_end_matches('s'))
}

static SHOW_OFF: LazyLock<Vec<String>> = LazyLock::new(|| strs("SHOW_OFF"));
static KIDS_IDEA: LazyLock<String> = LazyLock::new(|| string("KIDS_IDEA"));
static REPAIR: LazyLock<Vec<String>> = LazyLock::new(|| strs("REPAIR"));
static OTHER_PEOPLE: LazyLock<Vec<String>> = LazyLock::new(|| strs("OTHER_PEOPLE"));
static UNWELL: LazyLock<Vec<String>> = LazyLock::new(|| strs("UNWELL"));
static SICK_OF: LazyLock<Vec<String>> = LazyLock::new(|| strs("SICK_OF"));
static CONDITION: LazyLock<Vec<String>> = LazyLock::new(|| strs("CONDITION"));
static IMPRESS: LazyLock<Vec<String>> = LazyLock::new(|| strs("IMPRESS"));
static IMPRESS_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("IMPRESS_ASK"));
static IMPRESS_FOR: LazyLock<Vec<String>> = LazyLock::new(|| strs("IMPRESS_FOR"));
static IMPRESS_WORTH: LazyLock<Vec<String>> = LazyLock::new(|| strs("IMPRESS_WORTH"));
static HUNGRY: LazyLock<Vec<String>> = LazyLock::new(|| strs("HUNGRY"));

/// A dinner idea for this line: from what they named, within the limits they set.
pub fn suggest(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if dup::says_it_has_gone_off(&understood.clean) {
        return None;
    }
    if crate::knowledge::dishes::a_dish_is_already_the_plan(sh::or(&understood.norm_text, &understood.clean)) {
        return None;
    }
    let plan_offered = plan_get(memory, "offered").is_some_and(|v| v.truthy());
    let plan_ingredients = plan_strs(memory, "ingredients");
    let (allowed, without, minutes) = _limits(understood, memory);
    let mut names: Vec<String> = _named(understood).into_iter().filter(|n| !without.contains(n)).collect();
    if _something_it_has_not_got(understood, &allowed, &without) {
        return None;
    }
    if !names.is_empty() && !names.iter().any(|n| IN_A_DISH.contains(n)) {
        return None;
    }
    let clean = understood.clean.to_lowercase();
    if names.is_empty() && _for_a_table(memory, understood) == 0 && any_in(&clean, &MEAL_WORD) && any_in(&clean, &CHILD_AT_TABLE) {
        return Some(KIDS_IDEA.clone());
    }
    if let Some(kitchen) = _kitchen_answer(understood) {
        return Some(kitchen);
    }
    if any_in(&clean, &BEFORE_EVENT) && any_in(&clean, &EVENT_WORDS) {
        return None;
    }
    if any_in(&clean, &SMALL_PEOPLE) && !any_in(&clean, &MEAL_WORD) {
        return None;
    }
    if any_in(&clean, &NOT_A_MEAL_SUBJECT) {
        return None;
    }
    if any_in(&clean, &DESSERT_ASK) {
        return None;
    }
    if any_in(&clean, &TECHNIQUE_ASK) && !any_in(&clean, &USE_UP_ASK) {
        return None;
    }
    if any_in(&clean, &REPAIR) {
        return None;
    }
    if any_in(&clean, &MAKING_AT_HOME) {
        return None;
    }
    if any_in(&clean, &OTHER_PEOPLE) {
        return None;
    }
    if any_in(&clean, &UNWELL) && !any_in(&clean, &SICK_OF) {
        return None;
    }
    if unknown_food(understood) {
        return None;
    }
    let across = _said_about(memory, &clean);
    if GLUTEN.iter().chain(CONDITION.iter()).any(|c| across.contains(c.as_str())) {
        return None;
    }
    let asked = any_in(&clean, &ASK_WORDS);
    let impress = any_in(&clean, &IMPRESS);
    let impress_ask = any_in(&clean, &IMPRESS_ASK) || (any_in(&clean, &IMPRESS_FOR) && any_in(&clean, &IMPRESS_WORTH));
    let news = reads_as_news(understood, &clean);
    let hungry = any_in(&clean, &HUNGRY);
    let fancying_a_shape = any_in(&clean, &DISH_STYLE) || STYLE_CUES.iter().any(|(k, _)| clean.contains(k.as_str()));
    let wanting = any_in(&clean, &STYLE_WANT) && fancying_a_shape;
    let out_of = !_ruled_out(&clean).is_empty() && plan_offered;
    if news && minutes.is_none() && names.is_empty() && plan_ingredients.is_empty() && !hungry && !wanting && !out_of && !impress_ask {
        return None;
    }
    let limit_follow = allowed.is_some() && plan_offered;
    if !(!names.is_empty() || !plan_ingredients.is_empty() || minutes.is_some() || asked || limit_follow || wanting || out_of || impress_ask) {
        return None;
    }
    if names.is_empty() && understood.kind == "thanks" {
        return None;
    }
    if _for_a_table(memory, understood) >= 6 && asks_for_a_meal(understood, &clean) && !says_a_diet(&clean) {
        if let Some(crowd) = _crowd_answer(memory, understood).filter(|c| !c.is_empty()) {
            return Some(crowd);
        }
    }
    if any_in(&clean, &LUNCH_ASK) {
        return None;
    }
    if names.is_empty() {
        if let Some(found) = WITH_ASK.search(&clean) {
            let asked_for: Vec<String> = WORD
                .findall_str(found.str(1))
                .into_iter()
                .filter(|w| !KITCHEN_WORDS.contains(w) && !NOT_AN_INGREDIENT.contains(w) && !is_digits(w))
                .collect();
            let known_here: HashSet<&String> = INGREDIENTS.iter().flat_map(|(_, f)| f.iter()).collect();
            if !asked_for.is_empty()
                && !_company(&asked_for)
                && !asked_for.iter().any(|w| known_here.contains(w) || known_here.iter().any(|k| k.as_str() == w.trim_end_matches('s')))
            {
                return None;
            }
        }
    }
    let carried = names.is_empty();
    if names.is_empty() {
        names = plan_ingredients.iter().filter(|n| !without.contains(*n)).cloned().collect();
    }
    if carried && !names.is_empty() && (any_in(&clean, &DISH_STYLE) || STYLE_CUES.iter().any(|(k, _)| clean.contains(k.as_str()))) {
        names.clear();
    }
    let ruled_now = _ruled_out(&understood.clean);
    let off_limits: Vec<String> = _named(understood)
        .into_iter()
        .filter(|name| {
            !ruled_now.contains(name)
                && (without.contains(name)
                    || (allowed.is_some()
                        && !DISHES.iter().any(|d| d.uses.contains(name) && allowed.as_ref().unwrap().contains(&d.diet))))
        })
        .collect();
    names.retain(|n| !off_limits.contains(n));
    if !off_limits.is_empty() && names.is_empty() {
        let pick = _best(&[], &allowed, &without, minutes, memory, understood)[0];
        _plan(memory, &[], &allowed, &without, minutes, &[pick.name.clone()]);
        let reason = _diet_said(memory, None);
        let lead = if reason.is_empty() { "I'd leave that out".to_string() } else { format!("Since you are {reason}, I'd leave that out") };
        return Some(format!("{lead} and do {} ({} minutes): {}.", pick.said, pick.minutes, pick.how));
    }
    let ranked = _best(&names, &allowed, &without, minutes, memory, understood);
    let offered = plan_strs(memory, "offered");
    let fresh: Vec<&'static Dish> = ranked.iter().copied().filter(|d| !offered.contains(&d.name)).collect();
    if fresh.is_empty() && !names.is_empty() && minutes.is_some() {
        let best = ranked[0];
        _plan(memory, &names, &allowed, &without, minutes, &[best.name.clone()]);
        return Some(format!(
            "{} is the quickest I know with {}, at {} minutes; that is every dish I have for them.",
            crate::py::text::capitalize(&best.said),
            _prose(&names),
            best.minutes
        ));
    }
    let mut ranked = if fresh.is_empty() { ranked } else { fresh };
    if impress && names.is_empty() {
        let slower: Vec<&'static Dish> = ranked.iter().copied().filter(|d| d.minutes >= 30).collect();
        if !slower.is_empty() {
            ranked = slower;
        }
        let shown: Vec<&'static Dish> = ranked.iter().copied().filter(|d| SHOW_OFF.contains(&d.name)).collect();
        if !shown.is_empty() {
            ranked = shown;
        }
    }
    let first = ranked[0];
    if let Some(mins) = minutes {
        if !names.is_empty() && (!first.uses_any(&names) || first.minutes > mins + *GRACE) {
            if let Some(stick) = _stick(memory, &names, &allowed, &without) {
                let when = _variant(_times(&stick.how), understood, memory).replace("{n}", &stick.minutes.to_string());
                _plan(memory, &names, &allowed, &without, minutes, &[stick.name.clone()]);
                return Some(format!(
                    "{} is still the quickest I know with {}: {}. {when}",
                    crate::py::text::capitalize(&stick.said),
                    _prose(&names),
                    stick.how
                ));
            }
        }
    }
    let second = ranked.get(1).copied();
    if names.is_empty() && without.is_empty() && allowed.is_none() && minutes.is_none() && !impress {
        let picks = _spread(&ranked);
        if picks.len() >= 2 {
            let names_offered: Vec<String> = picks.iter().map(|i| i.name.clone()).collect();
            _plan(memory, &names, &allowed, &without, minutes, &names_offered);
            let said = picks[..picks.len() - 1].iter().map(|i| format!("{} in {}", i.said, i.minutes)).collect::<Vec<_>>().join(", ");
            let last = picks[picks.len() - 1];
            let said_words = understood.words.join(" ");
            let thrifty = any_in(&_said_about(memory, &said_words), &CHEAP_ASK);
            let eaters = _for_a_table(memory, understood);
            let opener = if thrifty && (2..=12).contains(&eaters) {
                format!("Cheap, and enough for {}:", _count_word(eaters))
            } else if thrifty {
                "Cheap:".to_string()
            } else {
                "Quick:".to_string()
            };
            if dup::fit_wanted(memory).as_deref() == Some("pick") && !thrifty {
                let one = picks[0];
                let others: String = picks[1..].iter().map(|i| format!(", or {} in {}", i.said, i.minutes)).collect();
                return Some(format!(
                    "Here's my pick: {} in {} minutes{others}. Tell me what is in the fridge and I will narrow it down.",
                    one.said, one.minutes
                ));
            }
            return Some(format!(
                "{opener} {said}, or {} if you have {} minutes. Tell me what is in the fridge and I will pick properly.",
                last.said, last.minutes
            ));
        }
    }
    if names.is_empty() {
        if let Some(second) = second {
            _plan(memory, &names, &allowed, &without, minutes, &[first.name.clone(), second.name.clone()]);
            let tail = if second.minutes < first.minutes - *GRACE {
                format!("If you want something quicker, {} takes {} minutes.", second.said, second.minutes)
            } else if (second.minutes - first.minutes).abs() <= *GRACE {
                format!("Something else that works is {}.", second.said)
            } else {
                format!("Something else that works is {}, at {} minutes.", second.said, second.minutes)
            };
            return Some(format!("I'd do {} ({} minutes): {}. {tail}", first.said, first.minutes, first.how));
        }
    }
    let mut reply = _describe(first, understood, memory, &without);
    let shared = second.map(|s| s.uses.iter().filter(|u| names.contains(u)).collect::<HashSet<_>>().len()).unwrap_or(0);
    if let Some(second) = second {
        if !names.is_empty() && shared >= names.len().min(2) && second.minutes <= 25.max(first.minutes + 10) {
            let mut options: Vec<String> = FOLLOW_ON.clone();
            if without.contains("cheese") && !second.uses.iter().any(|u| u == "cheese") {
                options.push(FOLLOW_ON_NO_CHEESE.clone());
            }
            let alt = if (second.minutes - first.minutes).abs() <= *GRACE {
                let pair = vec!["Something else that works is {alt}.".to_string(), "There is {alt} as well.".to_string()];
                _variant(&pair, understood, memory).replace("{alt}", &second.said)
            } else {
                _variant(&options, understood, memory).replace("{alt}", &second.said).replace("{n}", &second.minutes.to_string())
            };
            reply = _fresh(&format!("{reply} {alt}"), first, memory);
        }
    }
    let mut offered_now = vec![first.name.clone()];
    if let Some(s) = second {
        offered_now.push(s.name.clone());
    }
    _plan(memory, &names, &allowed, &without, minutes, &offered_now);
    Some(_fresh(&reply, first, memory))
}

fn r_suggest(_f: &FnRef, mem: &mut Memory, u: &Understood) -> Option<String> {
    suggest(mem, u)
}

fn r_later(_f: &FnRef, mem: &mut Memory, u: &Understood) -> Option<String> {
    _later(mem, u)
}

// ---------------------------------------------------------------------------------------------
// Counters and takeaways

static TAKEOUT_PICKS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&m("TAKEOUT_PICKS"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].strs(), p[1].str().to_string())
        })
        .collect()
});

fn word_re(name: &str) -> &'static Re {
    sh::re_cached(&format!(r"\b{}\b", sh::escape(name)), 0)
}

/// What to order at the counter the line names: the most specific name wins.
pub fn _takeout_pick(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let mut best: Option<&String> = None;
    let mut longest: i64 = -1;
    for (names, reply) in TAKEOUT_PICKS.iter() {
        for name in names {
            let n = name.chars().count() as i64;
            if n > longest && word_re(name).is_match(&clean) {
                best = Some(reply);
                longest = n;
            }
        }
    }
    best.cloned()
}

fn r_takeout_pick(_f: &FnRef, mem: &mut Memory, u: &Understood) -> Option<String> {
    _takeout_pick(mem, u)
}

/// Which rows of the takeaway table the line names, by row.
pub fn _counters_named(clean: &str) -> HashSet<usize> {
    let mut rows = HashSet::new();
    for (index, (names, _)) in TAKEOUT_PICKS.iter().enumerate() {
        if names.iter().any(|name| word_re(name).is_match(clean)) {
            rows.insert(index);
        }
    }
    rows
}

static DEFINITION_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| rx("DEFINITION_SHAPE"));
static ASKING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("ASKING_WORDS"));

/// True when the line asks what to order at one counter named in it.
pub fn _one_counter(understood: &Understood) -> bool {
    static VERDICT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:is|are|was|were)\b", 0));
    let clean = understood.clean.to_lowercase();
    if VERDICT.match_(&clean).is_some() {
        return false;
    }
    if let Some(shape) = DEFINITION_SHAPE.search(&clean) {
        if !ASKING_WORDS.contains(shape.str(1)) {
            return false;
        }
    }
    _counters_named(&clean).len() <= 1
}

fn g_one_counter(_f: &FnRef, u: &Understood) -> bool {
    _one_counter(u)
}

/// A mark on something and a row that got heated are both not food in the kitchen.
pub fn _not_a_mark_and_not_a_row(understood: &Understood) -> bool {
    dup::not_a_mark_on_a_thing(understood) && sh::not_the_heat_of_an_argument(understood)
}

fn g_not_a_mark_and_not_a_row(_f: &FnRef, u: &Understood) -> bool {
    _not_a_mark_and_not_a_row(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.meals:_not_a_mark_and_not_a_row:2367", g_not_a_mark_and_not_a_row),
    ("bot.knowledge.meals:_one_counter:2348", g_one_counter),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.meals:_carry:1657", r_carry),
    ("bot.knowledge.meals:_later:1108", r_later),
    ("bot.knowledge.meals:_takeout_pick:2318", r_takeout_pick),
    ("bot.knowledge.meals:suggest:1997", r_suggest),
    ("bot.knowledge.meals:use_up:1508", r_use_up),
];
