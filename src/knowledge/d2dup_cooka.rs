//! Helpers `cooking.rs` (lines 1-5999 of cooking.py) needs from code it does not own: D1's
//! `dishes`, `garden`, `meals`, `units` and U's `lookup` (`DUP` / `NEEDS`). (The second half of
//! cooking.py is `super::cooking_b`; `prices` is `super::prices`.) Each is a full port of the original unless it is
//! marked NEEDS (a stub until the owner's port is merged).

use crate::data::{self, Value};
use crate::knowledge::d2_shared::tokens;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.units

static TABLE_GROUPS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.units._TABLE_GROUPS"));

pub use crate::knowledge::units::people_in_groups;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.garden

// was DUP(bot.knowledge.garden::harvest_crops): now the real port
pub use crate::knowledge::garden::harvest_crops as garden_harvest_crops;

// DUP(bot.knowledge.garden::DONE_NOT)
pub fn garden_done_not() -> Vec<String> {
    data::strs("bot.knowledge.garden.DONE_NOT")
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.dishes (D1)

// was DUP(bot.knowledge.dishes::_same_word): now the real port
pub use crate::knowledge::dishes::_same_word as dishes_same_word;

// was DUP(bot.knowledge.dishes::_phrase_in): now the real port
pub use crate::knowledge::dishes::_phrase_in as dishes_phrase_in;

// was DUP(bot.knowledge.dishes::recent_texts): now the real port
pub use crate::knowledge::dishes::recent_texts as dishes_recent_texts;

static GOES_WITH_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.dishes.GOES_WITH_SHAPE"));

fn dishes_names_any(text_: &str, words_name: &str) -> bool {
    data::strs(words_name).iter().any(|w| dishes_phrase_in(text_, w))
}

// was DUP(bot.knowledge.dishes::asks_for_a_side_of_a_food): now the real port
pub use crate::knowledge::dishes::asks_for_a_side_of_a_food as dishes_asks_for_a_side_of_a_food;

// was DUP(bot.knowledge.dishes::general_sides): now the real port
pub use crate::knowledge::dishes::general_sides as dishes_general_sides;

static OVEN_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.dishes.OVEN_WORD"));

// was DUP(bot.knowledge.dishes::_names_an_appliance_and_a_cut): now the real port
pub use crate::knowledge::dishes::_names_an_appliance_and_a_cut as dishes_names_an_appliance_and_a_cut;

/// `dishes.PLANS.get(key)`.
pub fn dishes_plan(key: &str) -> Option<&'static Value> {
    data::get("bot.knowledge.dishes.PLANS").get(key)
}

/// `dishes.dish_in`: the plan key a text names (D1's port).
pub fn dishes_dish_in(text: &str) -> Option<String> {
    crate::knowledge::dishes::dish_in(text).map(str::to_string)
}

/// `dishes.ask_kind`: which kind of follow-up the line is.
pub fn dishes_ask_kind(u: &Understood) -> Option<String> {
    crate::knowledge::dishes::ask_kind(u).map(str::to_string)
}

/// `dishes.reply`: the plan's answer to a follow-up of that kind.
pub fn dishes_reply(kind: &str, memory: &mut Memory, u: &Understood) -> Option<String> {
    crate::knowledge::dishes::reply(kind, memory, u)
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup (U)

/// `find(understood, reject=lambda item: item.id in ids)`: the id of the entry the matcher finds
/// for the line with the rows of these ids turned down.
pub fn lookup_find_id(u: &Understood, reject_ids: &[&str]) -> Option<String> {
    let reject = |item: &crate::knowledge::entry::Entry| reject_ids.contains(&item.id.as_str());
    crate::knowledge::lookup::find(u, None, Some(&reject), false).map(|e| e.id.clone())
}

/// `find(reading, entries=garden.ENTRIES) is not None`, on the reading cooking builds from the
/// clean line alone: whether the garden table answers the line.
pub fn lookup_find_in_garden(line: &str) -> bool {
    static GARDEN: LazyLock<Vec<&'static crate::knowledge::entry::Entry>> = LazyLock::new(|| {
        data::get("bot.knowledge.garden.ENTRIES").items().iter().filter_map(crate::knowledge::entry::canonical).collect()
    });
    static NOT_A_WORD: LazyLock<crate::py::re::Re> = LazyLock::new(|| crate::py::re::Re::new(r"[^a-z0-9']+", 0));
    let words: Vec<String> = NOT_A_WORD.split(&line.to_lowercase()).into_iter().filter(|w| !w.is_empty()).collect();
    let reading = Understood { clean: line.to_string(), text: line.to_string(), words, kind: "ask".to_string(), ..Default::default() };
    crate::knowledge::lookup::find(&reading, Some(&GARDEN), None, false).is_some()
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.meals (D1)

fn meals(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.meals.{name}"))
}

// was DUP(bot.knowledge.meals::says_a_diet): now the real port
pub use crate::knowledge::meals::says_a_diet as meals_says_a_diet;

// was DUP(bot.knowledge.meals::_diet_said): now the real port
pub use crate::knowledge::meals::_diet_said as meals_diet_said;

static MEALS_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.meals.WORD"));

fn ingredient_forms() -> Vec<(String, Vec<String>)> {
    meals("INGREDIENTS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
}

static NO_HAVE_PATTERNS: LazyLock<std::sync::Mutex<std::collections::HashMap<(String, String), Re>>> =
    LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

fn no_have_match(cue: &str, shape: &str, plain: &str) -> bool {
    let mut cache = NO_HAVE_PATTERNS.lock().unwrap();
    let key = (cue.to_string(), shape.to_string());
    let re = cache.entry(key).or_insert_with(|| {
        let gap = meals("NO_HAVE_GAP").str().to_string();
        Re::new(
            &format!("(?<![a-z]){}{} {}(?![a-z])", crate::py::re::escape(cue), gap, crate::py::re::escape(shape)),
            0,
        )
    });
    re.is_match(plain)
}

/// `meals._ruled_out(text)` (the real port).
pub fn meals_ruled_out(text_: &str) -> HashSet<String> {
    crate::knowledge::meals::_ruled_out(text_).into_iter().collect()
}

// was DUP(bot.knowledge.meals::_is_the_ruled_out_food): now the real port
pub use crate::knowledge::meals::_is_the_ruled_out_food as meals_is_the_ruled_out_food;

fn strip_set<'a>(word: &'a str, chars: &str) -> &'a str {
    word.trim_end_matches(|c| chars.contains(c))
}

fn known_word(word: &str, known: &HashSet<String>) -> bool {
    known.contains(word) || known.contains(strip_set(word, "s")) || known.contains(strip_set(word, "es"))
}

// was DUP(bot.knowledge.meals::unknown_food): now the real port
pub use crate::knowledge::meals::unknown_food as meals_unknown_food;

#[cfg(test)]
mod wired_tests {
    use super::*;

    /// Expected values from `pycall.py bot.knowledge.cooking _the_garden_has_this_plant`.
    #[test]
    fn the_garden_table_answers() {
        data::load();
        assert!(lookup_find_in_garden("i bought a blueberry bush"));
        assert!(lookup_find_in_garden("when do i prune roses"));
        assert!(!lookup_find_in_garden("how long to boil an egg"));
        assert!(lookup_find_in_garden("my tomato plants have yellow leaves"));
    }
}
