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

// was DUP(bot.understanding::line_as_read): now the real port
pub use crate::understanding::line_as_read;

/// `memory.recent(memory, n)` (the real port): the last n turns.
pub fn recent(memory: &Memory, n: usize) -> &[Turn] {
    crate::memory::recent(Some(memory), n as i64)
}

/// `memory.user(memory)` (the real port), as a copy of the dict.
pub fn user(memory: &mut Memory) -> Value {
    crate::memory::user(memory).clone()
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

// was DUP(bot.knowledge.cooking_more::_clean): now the real port
pub use crate::knowledge::cooking_more::_clean;

// was DUP(bot.knowledge.cooking_more::_said_words): now the real port
pub use crate::knowledge::cooking_more::_said_words;

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

// was DUP(bot.knowledge.meals::_up_food): now the real port
pub use crate::knowledge::meals::_up_food;

// was DUP(bot.knowledge.meals::_named): now the real port
pub use crate::knowledge::meals::_named as meals_named;

// was DUP(bot.knowledge.meals::named_food): now the real port
pub use crate::knowledge::meals::named_food;

// was DUP(bot.knowledge.meals::idea_is_a_dish): now the real port
pub use crate::knowledge::meals::idea_is_a_dish;

// was DUP(bot.knowledge.meals::use_it_up): now the real port
pub use crate::knowledge::meals::use_it_up;

// was DUP(bot.knowledge.meals::reads_as_news): now the real port
pub use crate::knowledge::meals::reads_as_news;

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

