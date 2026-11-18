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

// DUP(bot.understanding::line_as_read)
/// The line with the repairs the engine made: `plain` when it has text, else clean or text.
pub fn line_as_read(u: &Understood) -> String {
    if !text::strip(&u.plain).is_empty() {
        return u.plain.clone();
    }
    if !u.clean.is_empty() { u.clean.clone() } else { u.text.clone() }
}

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

// DUP(bot.knowledge.units::people_in_groups)
/// How many people the groups named in the line add up to, or None when it names none.
pub fn people_in_groups(text: &str) -> Option<i64> {
    let mut total = 0;
    for raw in UNITS_TABLE_GROUPS.findall_str(&text.to_lowercase()) {
        let count = if text::is_digit(&raw) { raw.parse::<i64>().ok() } else { UNITS_GROUP_VALUE.get(&raw).copied() };
        if let Some(c) = count {
            if c != 0 {
                total += c;
            }
        }
    }
    if total != 0 { Some(total) } else { None }
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/freshness.py

static FRESHNESS_A_PERIOD_OF_TIME: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.freshness._A_PERIOD_OF_TIME"));

// DUP(bot.knowledge.freshness::a_cooling_question_not_a_keeping_one)
/// True unless the line names a length of time, which makes it a keeping question.
pub fn a_cooling_question_not_a_keeping_one(u: &Understood) -> bool {
    let mut said = u.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    if said.is_empty() {
        said = u.text.to_lowercase();
    }
    !FRESHNESS_A_PERIOD_OF_TIME.is_match(&said)
}

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

// DUP(bot.knowledge.body::a_persons_temperature)
/// True when the temperature in the line is a person's (a fever), not an oven's or a room's.
pub fn a_persons_temperature(u: &Understood) -> bool {
    let words: Vec<String> =
        u.words.iter().map(|w| text::strip_chars(&w.to_lowercase(), PUNCTUATION).to_string()).collect();
    let pairs = || words.iter().zip(words.iter().skip(1));
    if pairs().any(|(one, other)| BODY_TEMPERATURE_WORDS.contains(one) && BODY_THING_AFTER_THE_WORD.contains(other)) {
        return false;
    }
    if u.body.as_deref() == Some("temperature") {
        return true;
    }
    if !words.iter().any(|w| BODY_TEMPERATURE_WORDS.contains(w)) {
        return false;
    }
    if words.iter().any(|w| BODY_OTHER_PEOPLE.contains(w) || BODY_WE_OUR_OURS.contains(w)) {
        return true;
    }
    pairs().any(|(one, other)| BODY_POSSESSIVE_OF_A_PERSON.contains(one) && BODY_TEMPERATURE_WORDS.contains(other))
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/dishes.py

static DISHES_GOES_WITH_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.dishes.GOES_WITH_SHAPE"));
static DISHES_PAIRING_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes.PAIRING_FOODS"));

// DUP(bot.knowledge.dishes::_same_word)
/// The same word give or take a plural, an -ing or an -ed.
pub fn _same_word(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    for (one, two) in [(a, b), (b, a)] {
        if one.chars().count() > 3 {
            if let Some(s) = one.strip_suffix('s') {
                if s == two {
                    return true;
                }
            }
            if let Some(s) = one.strip_suffix("es") {
                if s == two {
                    return true;
                }
            }
            if let Some(s) = one.strip_suffix("ing") {
                if s == two || format!("{s}e") == two {
                    return true;
                }
            }
            if let Some(s) = one.strip_suffix("ed") {
                if s == two {
                    return true;
                }
            }
        }
    }
    false
}

// DUP(bot.knowledge.dishes::_phrase_in)
/// Whether the phrase's words stand in the text in a row (word forms allowed).
pub fn _phrase_in(text: &str, phrase: &str) -> bool {
    let wanted = crate::knowledge::d2_shared::tokens(phrase);
    let have = crate::knowledge::d2_shared::tokens(text);
    if wanted.is_empty() || have.is_empty() || wanted.len() > have.len() {
        return false;
    }
    let n = wanted.len();
    (0..=have.len() - n).any(|i| (0..n).all(|j| have[i + j] == wanted[j] || _same_word(&have[i + j], &wanted[j])))
}

// DUP(bot.knowledge.dishes::asks_for_a_side_of_a_food)
/// Whether the line asks what goes with a food ("what pairs best with blue cheese").
pub fn asks_for_a_side_of_a_food(text: &str) -> bool {
    DISHES_GOES_WITH_SHAPE.is_match(text) && DISHES_PAIRING_FOODS.iter().any(|w| _phrase_in(text, w))
}

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
