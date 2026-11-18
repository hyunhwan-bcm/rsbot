//! DUP ports (group 3 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.

use crate::data::{self, Value};
use crate::knowledge::d1_shared as sh;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup

// was DUP(bot.knowledge.lookup::tokens): now the real port
pub use crate::knowledge::lookup::tokens;

/// DUP(bot.knowledge.lookup::MIN_TYPO_LENGTH).
pub static MIN_TYPO_LENGTH: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.knowledge.lookup.MIN_TYPO_LENGTH").as_int().unwrap_or(4) as usize);

// was DUP(bot.knowledge.lookup::_one_edit): now the real port
pub use crate::knowledge::lookup::_one_edit;

static ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.ENDINGS"));

/// DUP(bot.knowledge.lookup::_stem): the word a form belongs to (radiator/radiators are one).
pub fn _stem(word: &str) -> &str {
    let n = word.chars().count();
    for ending in ENDINGS.iter() {
        if word.ends_with(ending.as_str()) && n >= ending.chars().count() + 3 {
            return &word[..word.len() - ending.len()];
        }
    }
    word
}

static COMMON: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup._COMMON"));
static COMMON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.COMMON_WORDS"));
static STEMS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup._STEMS"));
static COLOUR_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.COLOUR_WORDS"));
static LEX_CHAT: LazyLock<HashSet<String>> = LazyLock::new(|| dict_keys("bot.understanding.lexicon.CHAT"));
static LEX_EXPAND: LazyLock<HashSet<String>> = LazyLock::new(|| dict_keys("bot.understanding.lexicon.EXPAND"));

fn dict_keys(name: &str) -> HashSet<String> {
    data::get(name).pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect()
}

// was DUP(bot.knowledge.lookup::_engine_reads): now the real port
pub use crate::knowledge::lookup::_engine_reads;

static GONE_OFF_SIGNS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.GONE_OFF_SIGNS"));

// was DUP(bot.knowledge.lookup::says_it_has_gone_off): now the real port
pub use crate::knowledge::lookup::says_it_has_gone_off;

// ---------------------------------------------------------------------------------------------
// bot.understanding

// was DUP(bot.understanding.lexicon::every_word): now the real port
pub use crate::understanding::lexicon::every_word;

// was DUP(bot.understanding::line_as_read): now the real port
pub use crate::understanding::line_as_read;

static SENSE_ELSEWHERE_FORMS: LazyLock<HashMap<String, Vec<(i64, HashSet<String>)>>> = LazyLock::new(|| {
    data::get("bot.understanding.topics.SENSE_ELSEWHERE_FORMS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let tiers = v
                .items()
                .iter()
                .map(|t| {
                    let parts = t.items();
                    (parts[0].as_int().unwrap_or(0), parts[1].str_set())
                })
                .collect();
            Some((k.as_str()?.to_string(), tiers))
        })
        .collect()
});
static BODY_TWO_LIFE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.topics.BODY_TWO_LIFE"));
static OWNING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.topics.OWNING_WORDS"));
static TEETHING_IS_A_FAULT: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.understanding.topics.TEETHING_IS_A_FAULT"));
static TEETHING_AND_A_CHILD: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.understanding.topics.TEETHING_AND_A_CHILD"));

/// DUP(bot.understanding.topics::_teething_is_a_fault): "teething" is a new thing finding its
/// feet, not a child cutting teeth.
pub fn _teething_is_a_fault(norm: &[String]) -> bool {
    norm.iter().any(|w| TEETHING_IS_A_FAULT.contains(w)) && !norm.iter().any(|w| TEETHING_AND_A_CHILD.contains(w))
}

/// DUP(bot.understanding.topics::_sense_cue): a cue word sits near one of the two-life words.
pub fn _sense_cue(norm: &[String], positions: &[usize], tiers: &[(i64, HashSet<String>)]) -> bool {
    for (cue_at, word) in norm.iter().enumerate() {
        for (reach, cues) in tiers {
            if !cues.contains(word) {
                continue;
            }
            for &spot in positions {
                let (spot, cue) = (spot as i64, cue_at as i64);
                if spot - reach <= cue && cue <= spot + reach {
                    return true;
                }
            }
        }
    }
    false
}

/// DUP(bot.understanding.topics::_means_something_else): the word at `position` names another
/// thing in this line.
pub fn _means_something_else(norm: &[String], position: usize) -> bool {
    if norm[position] == "teething" && _teething_is_a_fault(norm) {
        return true;
    }
    let Some(tiers) = SENSE_ELSEWHERE_FORMS.get(&norm[position]) else { return false };
    if tiers.is_empty() {
        return false;
    }
    if BODY_TWO_LIFE.contains(&norm[position]) && position > 0 && OWNING_WORDS.contains(&norm[position - 1]) {
        return true;
    }
    _sense_cue(norm, &[position], tiers)
}

/// DUP(bot.understanding.topics::names_another_thing): positions of two-life words a cue has
/// taken to their other life.
pub fn names_another_thing(norm: &[String]) -> Vec<usize> {
    (0..norm.len())
        .filter(|&at| SENSE_ELSEWHERE_FORMS.contains_key(&norm[at]) && _means_something_else(norm, at))
        .collect()
}

// was DUP(bot.understanding.topics::says_another_thing): now the real port
pub use crate::understanding::topics::says_another_thing;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.cleaning and bot.compose.fit

static LANDED_ON: LazyLock<&'static crate::py::re::Re> = LazyLock::new(|| data::re("bot.knowledge.cleaning.LANDED_ON"));
static LANDED_ON_THINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cleaning.LANDED_ON_THINGS"));

// was DUP(bot.knowledge.cleaning::landed_on_a_thing): now the real port
pub use crate::knowledge::cleaning::landed_on_a_thing;

// was DUP(bot.knowledge.cleaning::not_a_mark_on_a_thing): now the real port
pub use crate::knowledge::cleaning::not_a_mark_on_a_thing;

static FIT_WANTS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fit.WANTS"));

// was DUP(bot.compose.fit::wanted): now the real port
pub use crate::compose::fit::wanted as fit_wanted;

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup: the kinds ask, and the engine's `find`

static KINDS_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KINDS_WORDS"));
static KINDS_SHAPE: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.lookup.KINDS_SHAPE"));
static KINDS_ASKS_ME: LazyLock<&'static crate::py::re::Re> = LazyLock::new(|| data::re("bot.knowledge.lookup.KINDS_ASKS_ME"));
static KINDS_ASKS_WITH_A_VERB: LazyLock<&'static crate::py::re::Re> =
    LazyLock::new(|| data::re("bot.knowledge.lookup.KINDS_ASKS_WITH_A_VERB"));
static KINDS_SAYS_HOW_IT_IS: LazyLock<&'static crate::py::re::Re> =
    LazyLock::new(|| data::re("bot.knowledge.lookup.KINDS_SAYS_HOW_IT_IS"));
static KINDS_NAMED_A_USE: LazyLock<&'static crate::py::re::Re> =
    LazyLock::new(|| data::re("bot.knowledge.lookup.KINDS_NAMED_A_USE"));

// was DUP(bot.knowledge.lookup::asks_the_kinds): now the real port
pub use crate::knowledge::lookup::asks_the_kinds;

/// DUP(bot.knowledge.lookup::find) as `find(understood) is not None`. UNRESOLVED: the matching
/// engine (lookup.find, score, _the_best_entry ...) belongs to the engine workstream and is not
/// ported; None stands for "the engine could not answer" (the callers treat it as Python treats
/// an exception there).
pub fn lookup_find_is_some(_u: &Understood) -> Option<bool> {
    None
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.cooking (the functions the dish, meal and cooking_more tables call, with their
// closure). Every `pub fn` here is DUP(bot.knowledge.cooking::<same name>).

pub mod cooking {
    #![allow(non_snake_case, unused_variables, dead_code)]
    //! VERIFY ONLY: delegates to D2's port of bot.knowledge.cooking.
    use crate::knowledge::cooking as c;
    use crate::state::Memory;
    use crate::understood::Understood;
    use std::collections::HashSet;

    #[derive(Default, Clone, Copy)]
    pub struct MealArgs<'a> {
        pub dishes_only: bool,
        pub text: Option<&'a str>,
        pub banned: Option<&'a HashSet<String>>,
        pub diet: Option<&'a str>,
        pub memory: Option<&'a Memory>,
        pub avoid: &'a [String],
        pub pointed_back: bool,
    }
    pub fn _a_brew_not_a_dinner(line: &str) -> bool { c::_a_brew_not_a_dinner(line) }
    pub fn _bans_for(understood: &Understood, memory: &Memory) -> HashSet<String> { c::_bans_for(understood, &mut memory.clone()) }
    pub fn _bans_it(food: &str, banned: &HashSet<String>) -> bool { c::_bans_it(food, banned) }
    pub fn _bans_said(banned: &HashSet<String>) -> Vec<String> { c::_bans_said(banned) }
    pub fn _crowd(text: &str) -> i64 { c::_crowd(text) }
    pub fn _crowd_or_any(text: &str, banned: &HashSet<String>) -> Option<String> { Some(c::_crowd_or_any(text, banned)) }
    pub fn _excluded(text: &str) -> HashSet<String> { c::_excluded(text) }
    pub fn _fold(word: &str) -> String { c::_fold(word) }
    pub fn _foods_named(text: &str) -> Vec<String> { c::_foods_named(text) }
    pub fn _mentions(text: &str, names: &[String]) -> bool { c::_mentions(text, names) }
    pub fn _occasion(text: &str) -> bool { c::_occasion(text) }
    pub fn _refused_foods(text: &str, families: bool) -> HashSet<String> { c::_refused_foods(text, families) }
    pub fn _sides(memory: &Memory, understood: &Understood) -> Option<String> { c::_sides(&mut memory.clone(), understood) }
    pub fn _table_text(text: &str) -> bool { c::_table_text(text) }
    pub fn dish(understood: &Understood, avoid: &[String], memory: Option<&Memory>) -> Option<String> { c::dish(understood, avoid, memory) }
    pub fn ingredients(understood: &Understood) -> Vec<String> { c::ingredients(understood) }
    pub fn meal(understood: &Understood, a: MealArgs) -> Option<String> {
        c::meal(understood, c::MealArgs { dishes_only: a.dishes_only, text: a.text, banned: a.banned.cloned(), diet: a.diet.map(String::from), memory: a.memory, avoid: a.avoid.to_vec(), pointed_back: a.pointed_back })
    }
    pub fn negated_ingredients(understood: &Understood) -> Vec<String> { c::negated_ingredients(understood) }
    pub fn pantry_idea(name: &str) -> Option<String> { c::pantry_idea(name) }
    pub fn pointed(memory: &Memory, understood: &Understood) -> Option<(Understood, bool)> { c::pointed(memory, understood) }
    pub fn pointed_at_a_food(understood: &Understood, memory: &Memory) -> Option<String> { c::pointed_at_a_food(understood, &mut memory.clone()) }
    pub fn side_for(memory: &Memory, text: &str) -> Option<String> { c::side_for(memory, text) }
    pub fn without(understood: &Understood) -> Option<String> { c::without(understood) }
    pub fn missing(understood: &Understood) -> HashSet<String> { c::missing(understood) }
}
