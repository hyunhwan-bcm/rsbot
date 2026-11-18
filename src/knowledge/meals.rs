//! Port of PyBot's `bot/knowledge/meals.py`: meal ideas (what to cook tonight, what to make from
//! what is already in the kitchen), the use-up table, carrying food, takeaways and counters.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g3 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as re, Re};
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
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

pub const GUARDS: &[(&str, GuardFn)] = &[];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
