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

// was DUP(bot.memory::last_bot): now the real port
pub use crate::memory::last_bot;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/units.py


pub use crate::knowledge::units::people_in_groups;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/freshness.py


// was DUP(bot.knowledge.freshness::a_cooling_question_not_a_keeping_one): now the real port
pub use crate::knowledge::freshness::a_cooling_question_not_a_keeping_one;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/body.py


/// Python's `string.punctuation`.
pub const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

// was DUP(bot.knowledge.body::a_persons_temperature): now the real port
pub use crate::knowledge::body::a_persons_temperature;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/dishes.py


// was DUP(bot.knowledge.dishes::_same_word): now the real port
pub use crate::knowledge::dishes::_same_word;

// was DUP(bot.knowledge.dishes::_phrase_in): now the real port
pub use crate::knowledge::dishes::_phrase_in;

// was DUP(bot.knowledge.dishes::asks_for_a_side_of_a_food): now the real port
pub use crate::knowledge::dishes::asks_for_a_side_of_a_food;

// `dishes.dish_in` (the real port), owned as a String.
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
