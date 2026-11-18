//! Helpers the g6 modules (health, cooking_entries, ...) need from code other groups or
//! workstreams own. Each is marked `DUP(...)` (another workstream), `DUP-D2(...)` (another D2
//! group: swap for a direct call at merge) or `NEEDS(...)` (a stub until the owner's port lands).

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// small Python shims for memory values

/// `str(value)` for the plain values memory holds.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::None => "None".into(),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Str(s) => s.to_string(),
        _ => String::new(),
    }
}

/// `memory.get(key)` for a key kept in `memory.extra`.
pub fn mem<'a>(memory: &'a Memory, key: &str) -> Option<&'a Value> {
    memory.extra.get(key)
}

/// `memory.get("user")` when it is a dict.
pub fn user_facts(memory: &Memory) -> Option<&Value> {
    mem(memory, "user").filter(|v| matches!(v, Value::Dict(_)))
}

fn set_of(name: &str) -> HashSet<String> {
    data::set(name)
}


/// `str(understood.get(key) or "")` for "norm": its Python list repr when it has words.
pub fn norm_repr(u: &Understood) -> String {
    if u.norm.is_empty() { String::new() } else { crate::knowledge::d2_shared::py_repr_list(&u.norm) }
}

/// `" ".join(str(understood.get(key) or "") for key in ("clean", "norm_text", "norm"))`.
pub fn clean_norm_text_norm(u: &Understood) -> String {
    format!("{} {} {}", u.clean, u.norm_text, norm_repr(u))
}

// ---------------------------------------------------------------------------------------------
// bot/understanding

// was DUP(bot.understanding::line_as_read): now the real port
pub use crate::understanding::line_as_read;

// ---------------------------------------------------------------------------------------------
// bot/memory

// DUP(bot.memory::last_bot)
/// The bot's last reply ("" when there is no earlier turn).
pub fn last_bot(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/units.py

static UNITS_TABLE_GROUPS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.units._TABLE_GROUPS"));
static UNITS_GROUP_VALUE: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.knowledge.units._GROUP_VALUE")
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0)))
        .collect()
});

pub use crate::knowledge::units::people_in_groups;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/freshness.py

static FRESHNESS_A_PERIOD_OF_TIME: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.freshness._A_PERIOD_OF_TIME"));

// was DUP(bot.knowledge.freshness::a_cooling_question_not_a_keeping_one): now the real port
pub use crate::knowledge::freshness::a_cooling_question_not_a_keeping_one;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/body.py

static BODY_TEMPERATURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.knowledge.body.TEMPERATURE_WORDS"));
static BODY_THING_AFTER_THE_WORD: LazyLock<HashSet<String>> =
    LazyLock::new(|| set_of("bot.knowledge.body.THING_AFTER_THE_WORD"));
static BODY_OTHER_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.knowledge.body.OTHER_PEOPLE"));
static BODY_WE_OUR_OURS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.knowledge.body.WE_OUR_OURS"));
static BODY_POSSESSIVE_OF_A_PERSON: LazyLock<HashSet<String>> =
    LazyLock::new(|| set_of("bot.knowledge.body.POSSESSIVE_OF_A_PERSON"));

/// Python's `string.punctuation`.
pub const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

// was DUP(bot.knowledge.body::a_persons_temperature): now the real port
pub use crate::knowledge::body::a_persons_temperature;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/dishes.py

static DISHES_GOES_WITH_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.dishes.GOES_WITH_SHAPE"));
static DISHES_PAIRING_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes.PAIRING_FOODS"));

// was DUP(bot.knowledge.dishes::_same_word): now the real port
pub use crate::knowledge::dishes::_same_word;

// was DUP(bot.knowledge.dishes::_phrase_in): now the real port
pub use crate::knowledge::dishes::_phrase_in;

// was DUP(bot.knowledge.dishes::asks_for_a_side_of_a_food): now the real port
pub use crate::knowledge::dishes::asks_for_a_side_of_a_food;

// NEEDS(bot.knowledge.dishes::dish_in)
/// The dish the text names (D1 ports the dish matcher, `dishes._named`); until it is merged this
/// stand-in names none.
pub fn dish_in(text: &str) -> Option<String> {
    crate::knowledge::dishes::dish_in(text).map(str::to_string)
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking.py (owned by the cookA/cookB groups)




// was DUP-D2(bot.knowledge.cooking::_line): now the real port
pub use crate::knowledge::cooking::_line;

// was DUP-D2(bot.knowledge.cooking::_people_after): now the real port
pub use crate::knowledge::cooking::_people_after;

// was DUP-D2(bot.knowledge.cooking::_crowd): now the real port
pub use crate::knowledge::cooking::_crowd;

// was DUP-D2(bot.knowledge.cooking::_bans_of): now the real port
pub use crate::knowledge::cooking::_bans_of;

// was DUP-D2(bot.knowledge.cooking::_family_of): now the real port
pub use crate::knowledge::cooking::_family_of;

// was DUP-D2(bot.knowledge.cooking::avoids_for_the_one_talked_about): now the real port
pub use crate::knowledge::cooking::avoids_for_the_one_talked_about;

// was DUP-D2(bot.knowledge.cooking::_said_so_far): now the real port
pub use crate::knowledge::cooking::_said_so_far;

// was DUP-D2(bot.knowledge.cooking::_no_meat_also_means_fish): now the real port
pub fn _no_meat_also_means_fish(banned: HashSet<String>) -> HashSet<String> {
    crate::knowledge::cooking::_no_meat_also_means_fish(&banned)
}



// was DUP-D2(bot.knowledge.cooking::_excluded): now the real port
pub use crate::knowledge::cooking::_excluded;

// was DUP-D2(bot.knowledge.cooking::_lunch_ideas): now the real port
pub use crate::knowledge::cooking::_lunch_ideas;

// was DUP-D2(bot.knowledge.cooking::missing): now the real port
pub use crate::knowledge::cooking::missing;


// was DUP-D2(bot.knowledge.cooking::_griddle_answer): now the real port
pub use crate::knowledge::cooking::_griddle_answer;

// was DUP-D2(bot.knowledge.cooking::_no_baking): now the real port
pub use crate::knowledge::cooking::_no_baking;

// was DUP-D2(bot.knowledge.cooking::available): now the real port
pub use crate::knowledge::cooking::available;

// was DUP-D2(bot.knowledge.cooking::without): now the real port
pub use crate::knowledge::cooking::without;


// was DUP-D2(bot.knowledge.cooking::_jacket): now the real port
pub use crate::knowledge::cooking::_jacket;

// was DUP-D2(bot.knowledge.cooking::_jam_make): now the real port
pub use crate::knowledge::cooking::_jam_make;

// was DUP-D2(bot.knowledge.cooking::_poach_fish): now the real port
pub use crate::knowledge::cooking::_poach_fish;

// was DUP-D2(bot.knowledge.cooking::_mushroom_wash): now the real port
pub use crate::knowledge::cooking::_mushroom_wash;

// was DUP-D2(bot.knowledge.cooking::_dip_fault): now the real port
pub use crate::knowledge::cooking::_dip_fault;
