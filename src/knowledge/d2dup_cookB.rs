//! Duplicates the second half of `cooking.py` (`cooking_b`) needs from code it does not own:
//! helpers in `understanding`, `units`, `memory`, `meals`, `cooking_more`, `picking` and
//! `dishes` (`DUP` / `NEEDS`). Each is a complete port of the original, named and shaped as its
//! owner will be. (cooking.py's first half is `super::cooking`.)

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// `a & b` is not empty.
pub fn meets(a: &HashSet<String>, b: &HashSet<String>) -> bool {
    if a.len() <= b.len() { a.iter().any(|x| b.contains(x)) } else { b.iter().any(|x| a.contains(x)) }
}

/// The text from a byte offset on, at most `n` characters (Python `text[at:at + n]`).
pub fn chars_after(text: &str, at: usize, n: usize) -> &str {
    let rest = &text[at..];
    let end = rest.char_indices().nth(n).map(|(b, _)| b).unwrap_or(rest.len());
    &rest[..end]
}

/// The text before a byte offset, at most `n` characters (Python `text[max(0, at - n):at]`).
pub fn chars_before(text: &str, at: usize, n: usize) -> &str {
    let head = &text[..at];
    let count = head.chars().count();
    if count <= n {
        return head;
    }
    let start = head.char_indices().nth(count - n).map(|(b, _)| b).unwrap_or(0);
    &head[start..]
}

// ---------------------------------------------------------------------------------------------
// bot/understanding, bot/memory, bot/knowledge/units

// DUP(bot.understanding::line_as_read)
/// The line with the engine's repairs (`plain`), else as typed.
pub fn line_as_read(understood: &Understood) -> String {
    if !text::strip(&understood.plain).is_empty() {
        return understood.plain.clone();
    }
    crate::knowledge::d2_shared::or(&understood.clean, &understood.text).to_string()
}

// DUP(bot.memory::recent)
/// The last n turns, oldest first.
pub fn recent(memory: &Memory, n: usize) -> &[Turn] {
    let turns = &memory.turns;
    &turns[turns.len().saturating_sub(n)..]
}

// DUP(bot.memory::user)
/// The user's facts (a dict), created on first use.
pub fn user(memory: &mut Memory) -> Value {
    match memory.extra.get("user") {
        Some(v @ Value::Dict(_)) => v.clone(),
        _ => {
            let empty = Value::Dict(Arc::new(Vec::new()));
            memory.extra.insert("user".to_string(), empty.clone());
            empty
        }
    }
}

static TABLE_GROUPS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.units._TABLE_GROUPS"));
static GROUP_VALUE: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.knowledge.units._GROUP_VALUE")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?)))
        .collect()
});

pub use crate::knowledge::units::people_in_groups;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking_more.py

// DUP(bot.knowledge.cooking_more::_clean)
pub fn _clean(understood: &Understood) -> String {
    understood.clean.to_lowercase()
}

// DUP(bot.knowledge.cooking_more::_said_words)
pub fn _said_words(understood: &Understood) -> Vec<String> {
    understood.words.iter().map(|w| super::cooking_b::_fold(w)).collect()
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/meals.py

static MEALS_INGREDIENTS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.meals.INGREDIENTS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.strs())))
        .collect()
});
static UP_FOOD_SET: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.meals._UP_FOOD_SET"));
static MEALS_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.meals.WORD"));
static USE_DISH_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.meals.USE_DISH_WORDS"));
static UP_CLASSES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.meals.UP_CLASSES")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});
static ASK_IN_NEWS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.meals.ASK_IN_NEWS"));

fn is_alpha_word(s: &str) -> bool {
    text::is_alpha(s)
}

// DUP(bot.knowledge.meals::_up_food)
pub fn _up_food(words: &[String]) -> Option<String> {
    for (index, word) in words.iter().enumerate() {
        if UP_FOOD_SET.contains(word) {
            return Some(word.clone());
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

// DUP(bot.knowledge.meals::_named)
/// The ingredients a reading names, phrases first.
pub fn meals_named(understood: &Understood) -> Vec<String> {
    let mut words: Vec<String> = understood.words.clone();
    for word in MEALS_WORD.findall_str(&understood.norm_text) {
        if !words.contains(&word) {
            words.push(word);
        }
    }
    let mut text = words.join(" ");
    let mut found: Vec<String> = Vec::new();
    for (ingredient, forms) in MEALS_INGREDIENTS.iter() {
        let hit: Vec<&String> = forms.iter().filter(|f| !is_alpha_word(f)).filter(|p| text.contains(p.as_str())).collect();
        if !hit.is_empty() {
            found.push(ingredient.clone());
            for phrase in hit {
                text = text.replace(phrase.as_str(), " ");
                words.retain(|w| !phrase.contains(w.as_str()));
            }
        }
    }
    for (ingredient, forms) in MEALS_INGREDIENTS.iter() {
        if found.contains(ingredient) {
            continue;
        }
        if forms.iter().any(|f| is_alpha_word(f) && words.contains(f)) {
            found.push(ingredient.clone());
        }
    }
    found
}

// DUP(bot.knowledge.meals::named_food)
pub fn named_food(words: &[String], understood: Option<&Understood>) -> Option<String> {
    if let Some(up) = _up_food(words) {
        if !up.is_empty() {
            return Some(up);
        }
    }
    let empty = Understood::default();
    let named = meals_named(understood.unwrap_or(&empty));
    named.into_iter().next().filter(|n| !n.is_empty())
}

// DUP(bot.knowledge.meals::idea_is_a_dish)
pub fn idea_is_a_dish(idea: &str) -> bool {
    let low = idea.to_lowercase();
    USE_DISH_WORDS.iter().any(|w| low.contains(w.as_str()))
}

// DUP(bot.knowledge.meals::use_it_up)
pub fn use_it_up(name: &str) -> String {
    let said = text::strip(name);
    let folded = said.to_lowercase();
    for (words, mv) in UP_CLASSES.iter() {
        if words.contains(&folded) {
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

// DUP(bot.knowledge.meals::reads_as_news)
pub fn reads_as_news(understood: &Understood, clean: &str) -> bool {
    if understood.intent != "news" {
        return false;
    }
    let low = crate::knowledge::d2_shared::or(clean, &understood.clean).to_lowercase();
    !ASK_IN_NEWS.iter().any(|cue| low.contains(cue.as_str()))
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/picking.py, bot/knowledge/dishes.py, bot/knowledge/lookup.py


// was DUP-D2(bot.knowledge.picking::choosing): now the real port
pub use crate::knowledge::picking::choosing;

/// `cooking._a_dish_with_a_heat`: the dish the text names (`dishes.dish_in`) has a plan with a
/// truthy "temp" (`dishes.PLANS.get(name or "")`).
pub fn a_dish_plan_has_a_heat(text: &str) -> bool {
    let name = crate::knowledge::dishes::dish_in(text).unwrap_or("");
    crate::data::get("bot.knowledge.dishes.PLANS").get(name).and_then(|plan| plan.get("temp")).map(|t| t.truthy()).unwrap_or(false)
}

