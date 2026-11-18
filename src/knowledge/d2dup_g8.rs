//! Helpers the g8 modules (consumer, plans, cleaning, lifeadmin) need from code they do not own,
//! and a cache for regexes PyBot builds on the fly. Every port of someone else's function is
//! marked `DUP(...)`, `DUP-D2(...)` or `NEEDS(...)` for the integrator.

use crate::data;
use crate::py::re::{escape, Re};
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex};

// ---------------------------------------------------------------------------------------------
// Regexes built at run time

static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// `re.compile(pattern, flags)` for a pattern PyBot builds from data at run time, built once.
pub fn cached_re(pattern: &str, flags: u32) -> &'static Re {
    let key = (pattern.to_string(), flags);
    if let Some(r) = CACHE.lock().unwrap().get(&key) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    CACHE.lock().unwrap().insert(key, r);
    r
}

/// `re.search(r"\b" + re.escape(word) + r"\b", text)`.
pub fn has_bounded_word(text: &str, word: &str) -> bool {
    cached_re(&format!("\\b{}\\b", escape(word)), 0).is_match(text)
}

/// `re.search(r"(?<![a-z0-9])" + re.escape(word) + r"(?![a-z0-9])", text)`.
pub fn has_fenced_word(text: &str, word: &str) -> bool {
    cached_re(&format!("(?<![a-z0-9]){}(?![a-z0-9])", escape(word)), 0).is_match(text)
}

// ---------------------------------------------------------------------------------------------
// DUP-D2(bot.knowledge.cooking::without) and the helpers it runs

macro_rules! cooking {
    ($t:ty, $name:ident, $f:path) => {
        static $name: LazyLock<$t> = LazyLock::new(|| $f(concat!("bot.knowledge.cooking.", stringify!($name))));
    };
}
cooking!(&'static Re, GRIDDLE_BAKES, data::re);
cooking!(&'static Re, BREAD, data::re);
cooking!(&'static Re, CAKEISH, data::re);
cooking!(&'static Re, CROWD, data::re);
cooking!(&'static Re, GRIDDLE_BATTER, data::re);
cooking!(&'static Re, GRIDDLE_DOUGH, data::re);
cooking!(&'static Re, SWEET_ASK, data::re);
cooking!(String, NO_OVEN_ONLY, data::string);
cooking!(String, NO_HOB, data::string);
cooking!(String, NO_KETTLE, data::string);
cooking!(String, NO_OVEN, data::string);
cooking!(String, NO_MICROWAVE, data::string);
cooking!(String, NO_OVEN_BREAD, data::string);
cooking!(String, NO_BAKING, data::string);
cooking!(String, _GRIDDLE_TAIL, data::string);
cooking!(Vec<String>, COOKING_ASKS, data::strs);
cooking!(Vec<String>, LISTING, data::strs);
cooking!(Vec<String>, LACK_ATTACHED, data::strs);

/// `LACKS`: (appliance, phrases that say it is gone).
static LACKS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking.LACKS")
        .items()
        .iter()
        .map(|row| (row.items()[0].str().to_string(), row.items()[1].strs()))
        .collect()
});
/// `LACK_SAID`: (appliance, one phrase).
static LACK_SAID: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking.LACK_SAID")
        .items()
        .iter()
        .map(|row| (row.items()[0].str().to_string(), row.items()[1].str().to_string()))
        .collect()
});
/// `HAVE_ONE` / `ONLY_ONE`: appliance -> phrases, in dict order.
fn appliance_phrases(name: &str) -> Vec<(String, Vec<String>)> {
    data::get(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
}
static HAVE_ONE: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| appliance_phrases("bot.knowledge.cooking.HAVE_ONE"));
static ONLY_ONE: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| appliance_phrases("bot.knowledge.cooking.ONLY_ONE"));

fn phrases_of<'a>(table: &'a [(String, Vec<String>)], name: &str) -> &'a [String] {
    table.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_slice()).unwrap_or(&[])
}

// DUP-D2(bot.knowledge.cooking::_line)
/// The line as the cooking module reads it: what they typed, and the words it was read into.
pub fn cooking_line(understood: &Understood) -> String {
    format!("{} {}", understood.clean, understood.norm_text).to_lowercase()
}

// DUP-D2(bot.knowledge.cooking::_says_gone)
/// True when a segment says one of these phrases, unless a word hangs off the appliance.
fn says_gone(phrases: &[&str], segments: &[&str]) -> bool {
    for segment in segments {
        for phrase in phrases {
            if phrase.is_empty() {
                continue;
            }
            for (at, _) in segment.match_indices(phrase) {
                let rest = crate::py::text::strip(&segment[at + phrase.len()..]);
                let first = rest.split(' ').next().unwrap_or("");
                let after = first.trim_matches(|c| ".,!?;:".contains(c));
                if !LACK_ATTACHED.iter().any(|w| w == after) {
                    return true;
                }
            }
        }
    }
    false
}

// DUP-D2(bot.knowledge.cooking::missing)
/// The appliances this line says it does not have.
pub fn cooking_missing(understood: &Understood) -> HashSet<String> {
    let text = cooking_line(understood);
    let segments: Vec<&str> = text.split('|').collect();
    let mut gone = HashSet::new();
    for (name, phrases) in LACKS.iter() {
        let ps: Vec<&str> = phrases.iter().map(String::as_str).collect();
        if says_gone(&ps, &segments) {
            gone.insert(name.clone());
        }
    }
    for (name, phrase) in LACK_SAID.iter() {
        if !gone.contains(name) && says_gone(&[phrase.as_str()], &segments) {
            gone.insert(name.clone());
        }
    }
    gone
}

// DUP-D2(bot.knowledge.cooking::available)
/// What the kitchen has, as far as the line says: what it names, else whatever is not gone.
pub fn cooking_available(text: &str, gone: &HashSet<String>) -> HashSet<String> {
    let text = text.to_lowercase();
    for (name, phrases) in ONLY_ONE.iter() {
        if phrases.iter().any(|p| text.contains(p.as_str())) {
            return HashSet::from([name.clone()]);
        }
    }
    let named: HashSet<String> = HAVE_ONE
        .iter()
        .filter(|(_, phrases)| phrases.iter().any(|p| text.contains(p.as_str())))
        .map(|(n, _)| n.clone())
        .filter(|n| !gone.contains(n))
        .collect();
    if !named.is_empty() {
        return named;
    }
    ["oven", "hob", "microwave", "kettle"].iter().map(|s| s.to_string()).filter(|n| !gone.contains(n)).collect()
}

// DUP-D2(bot.knowledge.cooking::_asks_for_sweet)
fn asks_for_sweet(text: &str) -> bool {
    let text = text.to_lowercase();
    if text.contains("sweet potato") {
        return false;
    }
    SWEET_ASK.is_match(&text)
}

static WARM_BREAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:wrap|wraps|tortilla|tortillas|pitta|pittas|pita|pitas)\b", 0));

fn rstrip_s(s: &str) -> &str {
    s.trim_end_matches('s')
}

// DUP-D2(bot.knowledge.cooking::_griddle_answer)
fn griddle_answer(said: &str) -> String {
    let text = said.to_lowercase();
    if let Some(batter) = GRIDDLE_BATTER.search(&text) {
        return format!(
            "A {} is a pan job as it is: the batter goes into a hot non-stick pan with a little butter, about a minute a side until bubbles pop and the edges set. Crumpets want rings in a dry pan on a low heat until the top sets and the holes stay open. {}",
            rstrip_s(batter.as_str()),
            *_GRIDDLE_TAIL
        );
    }
    if let Some(dough) = GRIDDLE_DOUGH.search(&text) {
        let name = dough.as_str();
        if !["scone", "drop scone", "pikelet"].contains(&rstrip_s(name)) {
            return format!(
                "A {} cooks in a heavy dry pan, not a tin: the same dough made a little thicker and patted out, about 5 minutes a side on a low heat with a lid on, until the middle is set and the edges have caught. Scones, pikelets and roti are the same trick. {}",
                rstrip_s(name),
                *_GRIDDLE_TAIL
            );
        }
        return format!(
            "A scone cooks in a heavy pan: the same dough made a little thicker and patted out, in a dry pan on a low heat with a lid on for about 5 minutes a side, until the middle is set and the edges have caught. Drop scones, pikelets, chapatis and roti are the same trick on a hot dry pan. {}",
            *_GRIDDLE_TAIL
        );
    }
    let named = WARM_BREAD.search(&text).map(|m| rstrip_s(m.as_str()).to_string()).unwrap_or_else(|| "wrap".to_string());
    let other = if named.starts_with("pitt") { "and a wrap warms the same way" } else { "and a pitta puffs the same way" };
    format!(
        "A {named} is warmed rather than baked: 30 seconds a side in a dry hot pan and it is soft enough to fold, {other}. {}",
        *_GRIDDLE_TAIL
    )
}

// DUP-D2(bot.knowledge.cooking::_no_baking)
fn no_baking(text: &str) -> String {
    if GRIDDLE_BAKES.is_match(text) {
        return griddle_answer(text);
    }
    let base = if BREAD.is_match(text) && !CAKEISH.is_match(text) {
        NO_OVEN_BREAD.clone()
    } else {
        format!("{}{}", *NO_BAKING, if BREAD.is_match(text) { format!(" {}", *NO_OVEN_BREAD) } else { String::new() })
    };
    match CROWD.search(text) {
        None => base,
        Some(many) => format!(
            "{base} For {} people make two tins of it and cut it small, or set one big cheesecake in a deep tin and chill it overnight.",
            many.as_str()
        ),
    }
}

const BAKING: &[&str] = &[
    "bake", "baking", "cake", "cakes", "bread", "pie", "pastry", "pastries", "biscuit", "biscuits", "cookie", "cookies",
    "meringue", "meringues", "yorkshire", "scone", "scones", "loaf", "loaves", "muffin", "muffins", "brownies",
    "flapjack", "flapjacks",
];

// DUP-D2(bot.knowledge.cooking::without)
/// An answer for a line that says the kitchen has no oven or no microwave, or None.
pub fn cooking_without(understood: &Understood) -> Option<String> {
    let gone = cooking_missing(understood);
    if gone.is_empty() {
        return None;
    }
    // `words` is always a list on a reading
    let text = understood.words.join(" ").to_lowercase();
    let asking = COOKING_ASKS.iter().any(|one| has_bounded_word(&text, one));
    let listing = LISTING.iter().any(|one| text.contains(one.as_str()));
    if !(asking || listing) {
        let named = ["hob", "microwave", "kettle", "oven"]
            .iter()
            .flat_map(|n| phrases_of(&HAVE_ONE, n).iter())
            .chain(["hob", "microwave", "kettle"].iter().flat_map(|n| phrases_of(&ONLY_ONE, n).iter()))
            .any(|one| text.contains(one.as_str()));
        if !(named && crate::py::text::split_ref(&text).len() <= 8) {
            return None;
        }
    }
    let how = if BAKING.iter().any(|one| text.contains(one)) || asks_for_sweet(&text) { "bake" } else { "" };
    if gone.contains("oven") && how == "bake" && !gone.contains("hob") {
        return Some(no_baking(&text));
    }
    if gone.contains("oven") && !gone.contains("hob") && GRIDDLE_BAKES.is_match(&text) {
        return Some(griddle_answer(&text));
    }
    let have = cooking_available(&text, &gone);
    if gone.contains("hob") || !have.contains("hob") {
        if have.contains("oven") {
            return Some(NO_OVEN_ONLY.clone());
        }
        if have.contains("microwave") {
            return Some(NO_HOB.clone());
        }
        return Some(NO_KETTLE.clone());
    }
    if gone.contains("oven") || !have.contains("oven") {
        return Some(NO_OVEN.clone());
    }
    if gone.contains("microwave") && how != "bake" {
        return Some(NO_MICROWAVE.clone());
    }
    None
}

// ---------------------------------------------------------------------------------------------
// DUP(bot.knowledge.dishes::a_dish_is_already_the_plan)

static PLAN_MARKER: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._PLAN_MARKER"));
static IDEA_ASK: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.dishes._IDEA_ASK"));

// NEEDS(bot.knowledge.dishes::dish_in)
/// The plan key a piece of text names, or None. Needs D1's `dishes._named` (the dish-name
/// matcher, with `understanding.topics.says_another_thing` and lookup's typo rules); until that
/// is merged it names nothing.
pub fn dish_in(text: &str) -> Option<String> {
    if crate::py::text::strip(text).is_empty() {
        return None;
    }
    None
}

// DUP(bot.knowledge.dishes::a_dish_is_already_the_plan)
/// True when the line names a dish and says that is what is being cooked.
pub fn a_dish_is_already_the_plan(text: &str) -> bool {
    let text = text.to_lowercase();
    // `dish_in` is free of side effects, so the cheap tests go first
    if text.is_empty() {
        return false;
    }
    if !PLAN_MARKER.iter().any(|m| text.contains(m.as_str())) {
        return false;
    }
    if IDEA_ASK.iter().any(|p| text.contains(p.as_str())) {
        return false;
    }
    dish_in(&text).is_some()
}
