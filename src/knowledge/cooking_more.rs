//! Port of PyBot's `bot/knowledge/cooking_more.py`: the cooking rows that answer "is it done, why
//! did it do that, what is the plan" (sides, flatbreads, browned produce, a wake and a buffet,
//! deseeding, a meal for the slot), and the meal plan a short follow-up narrows.
#![allow(non_snake_case)]

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g3 as dup;
use crate::knowledge::d1_dup_g3::cooking;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, LazyLock};

fn c(name: &str) -> String {
    format!("bot.knowledge.cooking_more.{name}")
}
fn strs(name: &str) -> Vec<String> {
    data::strs(&c(name))
}
fn set(name: &str) -> HashSet<String> {
    data::set(&c(name))
}
fn rx(name: &str) -> &'static Re {
    data::re(&c(name))
}
fn string(name: &str) -> String {
    data::string(&c(name))
}
fn any_in(text: &str, words: &[String]) -> bool {
    words.iter().any(|w| text.contains(w.as_str()))
}
fn intersects(a: &HashSet<String>, b: &HashSet<String>) -> bool {
    a.iter().any(|x| b.contains(x))
}

// ---------------------------------------------------------------------------------------------
// The rows' own replies and guards

static A_SIDE_WITH_NO_DISH: LazyLock<String> = LazyLock::new(|| string("A_SIDE_WITH_NO_DISH"));

/// The side for the dish the line names, or for the dish the conversation is cooking, or the
/// jobs a side can do when there is no dish anywhere.
pub fn _a_side_or_the_three_jobs(memory: &Memory, understood: &Understood) -> Option<String> {
    if let Some(s) = cooking::_sides(memory, understood).filter(|s| !s.is_empty()) {
        return Some(s);
    }
    if let Some(s) = cooking::side_for(memory, &understood.clean).filter(|s| !s.is_empty()) {
        return Some(s);
    }
    Some(A_SIDE_WITH_NO_DISH.clone())
}

fn r_a_side_or_the_three_jobs(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _a_side_or_the_three_jobs(m, u)
}

static FLATBREAD_KINDS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&c("_FLATBREAD_KINDS"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].str().to_string(), p[1].strs())
        })
        .collect()
});
static FLATBREAD_RECIPES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&c("_FLATBREAD_RECIPES")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

fn flatbread(name: &str) -> Option<String> {
    FLATBREAD_RECIPES.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
}

/// `_flatbread_answer()`: the factory of the flatbread row's reply (run once at import).
pub fn _flatbread_answer() -> fn(&Memory, &Understood) -> Option<String> {
    _flatbread_answer__answer
}

/// The reply `_flatbread_answer()` makes: the flatbread the line names, or the quick one.
pub fn _flatbread_answer__answer(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", understood.clean.to_lowercase());
    if said.contains("spanish") || said.contains("espanola") || said.contains("espanol") || said.contains("omelette") {
        return None;
    }
    for (name, words) in FLATBREAD_KINDS.iter() {
        if words.iter().any(|w| said.contains(&format!(" {w} ")) || said.trim().ends_with(&format!(" {w}"))) {
            return flatbread(name);
        }
    }
    flatbread("quick")
}

fn r_flatbread_answer(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _flatbread_answer__answer(m, u)
}

static BROWN_SAFE_ASKS: LazyLock<&'static Re> = LazyLock::new(|| rx("_BROWN_SAFE_ASKS"));
static BROWN_SAFE: LazyLock<String> = LazyLock::new(|| string("_BROWN_SAFE"));
static MUSHROOM_SAFE: LazyLock<String> = LazyLock::new(|| string("_MUSHROOM_SAFE"));
static MUSHROOM_BROWN: LazyLock<String> = LazyLock::new(|| string("_MUSHROOM_BROWN"));
static PRODUCE_BROWN: LazyLock<String> = LazyLock::new(|| string("_PRODUCE_BROWN"));
static MUSHROOM_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("_MUSHROOM_WORDS"));
static BROWN_KINDS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&c("_BROWN_KINDS"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].strs(), p[1].str().to_string())
        })
        .collect()
});

/// The cut-and-brown answer for the fruit or vegetable the line names.
pub fn _produce_brown(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", understood.clean.to_lowercase());
    let mushrooms = MUSHROOM_WORDS.is_match(&said);
    if BROWN_SAFE_ASKS.is_match(&said) {
        return Some(if mushrooms { MUSHROOM_SAFE.clone() } else { BROWN_SAFE.clone() });
    }
    if mushrooms {
        return Some(MUSHROOM_BROWN.clone());
    }
    for (names, reply) in BROWN_KINDS.iter() {
        if names.iter().any(|n| said.contains(&format!(" {n} "))) {
            return Some(reply.clone());
        }
    }
    Some(PRODUCE_BROWN.clone())
}

fn r_produce_brown(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _produce_brown(m, u)
}

static HOT_HELD: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOT_HELD"));
static COLD_HELD: LazyLock<&'static Re> = LazyLock::new(|| rx("_COLD_HELD"));
static AHEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("_AHEAD"));
static FOOD_IN_LINE: LazyLock<HashSet<String>> = LazyLock::new(|| set("_FOOD_IN_LINE"));
static FOOD_JOB: LazyLock<&'static Re> = LazyLock::new(|| rx("_FOOD_JOB"));
static OUR_OWN: LazyLock<&'static Re> = LazyLock::new(|| rx("_OUR_OWN"));
static A_PRONOUN_HELD: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PRONOUN_HELD"));

/// The words of the line, lower case and padded (`" " + " ".join(words).lower() + " "`).
fn padded_words(understood: &Understood) -> String {
    format!(" {} ", understood.words.join(" ").to_lowercase())
}

/// True when a crowd line is about the food: what it is, or what the food has to do.
pub fn _about_the_food(understood: &Understood) -> bool {
    static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
    static PEOPLE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:people|guests|heads|crowd|everyone)\b", 0));
    let line = padded_words(understood);
    if WORDS.findall_str(&line).iter().any(|w| FOOD_IN_LINE.contains(w)) {
        return true;
    }
    if OUR_OWN.is_match(&line) {
        return true;
    }
    if A_PRONOUN_HELD.is_match(&line) {
        return true;
    }
    FOOD_JOB.is_match(&line) && PEOPLE.is_match(&line)
}

fn g_about_the_food(_f: &FnRef, u: &Understood) -> bool {
    _about_the_food(u)
}

/// What to put out at a wake: food people can take with one hand.
pub fn _wake_food(_memory: &Memory, _understood: &Understood) -> Option<String> {
    Some("Food that needs no carving and no serving: sandwiches cut small, sausage rolls, a quiche, a pot of soup kept hot, cut cake, and plenty of tea. Make it the day before, allow three or four sandwich pieces a head, and ask two people to keep the teapot going and the plates moving, because that is the job nobody thinks to do.".into())
}

fn r_wake_food(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _wake_food(m, u)
}

/// Keeping a buffet hot or cold, and how far ahead it can be made.
pub fn _buffet_holding(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = padded_words(understood);
    if AHEAD.is_match(&line) {
        return Some("Make sandwiches the night before, wrapped in the fridge, and leave the salad or tomato out until the morning because it turns the bread wet. Egg, cheese and ham fillings keep the best; nothing already filled should sit out for more than two hours.".into());
    }
    if COLD_HELD.is_match(&line) && !HOT_HELD.is_match(&line) {
        return Some("Cold food wants to stay under 5C: keep the bowls in the fridge until people arrive, stand them in a bigger bowl of ice, and put out a little at a time. Rice, cooked meat and dairy are the things to be strict about, and anything out for more than two hours goes in the bin.".into());
    }
    Some("Hot food wants to stay above 63C: a slow cooker on low, an oven on its lowest with the door ajar, or a lidded pan on a trivet, served in small batches so what is out gets eaten. Cold food wants to stay under 5C. Anything that has sat between the two for more than two hours is the food that makes people ill, so bin it.".into())
}

fn r_buffet_holding(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _buffet_holding(m, u)
}

// ---------------------------------------------------------------------------------------------
// A plan kept over a few turns

/// The meal plan (`memory["meal"]` and what `wants` builds): Python's dict, by key.
pub type Context = IndexMap<String, Value>;

fn vstr(s: &str) -> Value {
    Value::Str(Arc::from(s))
}
fn vlist(items: &[String]) -> Value {
    Value::List(Arc::new(items.iter().map(|s| vstr(s)).collect()))
}

/// A dict value as a Context (a plan read back from memory).
pub fn context_from(value: &Value) -> Context {
    value.pairs().iter().map(|(k, v)| (k.str().to_string(), v.clone())).collect()
}

/// A Context as the dict value memory keeps.
pub fn context_value(ctx: &Context) -> Value {
    Value::Dict(Arc::new(ctx.iter().map(|(k, v)| (vstr(k), v.clone())).collect()))
}

/// `context.get(key)` is truthy.
fn truthy(ctx: &Context, key: &str) -> bool {
    ctx.get(key).is_some_and(|v| v.truthy())
}

/// `context.get(key) or ()` as strings.
fn list(ctx: &Context, key: &str) -> Vec<String> {
    ctx.get(key).map(|v| v.items().iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()
}

/// `context.setdefault(key, []).append(name)` unless it is there already.
fn push_unique(ctx: &mut Context, key: &str, name: &str) {
    let mut items: Vec<Value> = ctx.get(key).map(|v| v.items().to_vec()).unwrap_or_default();
    if !items.iter().any(|x| x.as_str() == Some(name)) {
        items.push(vstr(name));
    }
    ctx.insert(key.to_string(), Value::List(Arc::new(items)));
}

/// `context.setdefault(key, [])` (an empty list when the key is missing).
fn setdefault_list(ctx: &mut Context, key: &str) {
    if !ctx.contains_key(key) {
        ctx.insert(key.to_string(), Value::List(Arc::new(Vec::new())));
    }
}

/// `int(value or 0)` for a plan's turn counters.
fn int_or_zero(v: Option<&Value>) -> i64 {
    match v {
        Some(Value::Int(i)) => *i,
        Some(Value::Bool(b)) => *b as i64,
        Some(Value::Float(f)) => f.trunc() as i64,
        _ => 0,
    }
}

static STAPLES: LazyLock<HashSet<String>> = LazyLock::new(|| set("STAPLES"));
static MEAT_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| set("MEAT_NAMES"));
static GREEN_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| set("GREEN_NAMES"));
static GREEN_IN_TEXT: LazyLock<Vec<String>> = LazyLock::new(|| strs("GREEN_IN_TEXT"));
static LIMIT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("LIMIT_WORDS"));
static SIMPLE_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("SIMPLE_PHRASES"));
static HEAT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("HEAT_WORDS"));
static HEAT_IN_TEXT: LazyLock<Vec<String>> = LazyLock::new(|| strs("HEAT_IN_TEXT"));
static HEAT_INGREDIENTS: LazyLock<Vec<String>> = LazyLock::new(|| strs("HEAT_INGREDIENTS"));
static TASTE_RULED_OUT: LazyLock<&'static Re> = LazyLock::new(|| rx("_TASTE_RULED_OUT"));
static HOT_FOOD_RULED_OUT: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOT_FOOD_RULED_OUT"));
static VEG_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("VEG_WORDS"));
static MEAT_FREE_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("MEAT_FREE_PHRASES"));
static DAIRY_FREE_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("DAIRY_FREE_PHRASES"));
static DAIRY_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DAIRY_FOODS"));
static COMING_FOR: LazyLock<Vec<String>> = LazyLock::new(|| strs("COMING_FOR"));
static EVENT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("EVENT_WORDS"));
static MENU_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("MENU_PHRASES"));
static BIG_DAYS: LazyLock<HashSet<String>> = LazyLock::new(|| set("BIG_DAYS"));
static COOKING_FOR: LazyLock<Vec<String>> = LazyLock::new(|| strs("COOKING_FOR"));
static PEOPLE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("PEOPLE_WORDS"));
static CROWD_PHRASE: LazyLock<&'static Re> = LazyLock::new(|| rx("CROWD_PHRASE"));
static DISLIKE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("DISLIKE_WORDS"));
static SOMEONE_ELSE: LazyLock<&'static Re> = LazyLock::new(|| rx("SOMEONE_ELSE"));
static GREEN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("GREEN_WORDS"));
static PLAN_LAPSE: LazyLock<i64> = LazyLock::new(|| data::get(&c("PLAN_LAPSE")).as_int().unwrap_or(2));
static MINUTES: LazyLock<HashMap<String, i64>> = LazyLock::new(|| int_map("MINUTES"));
static PANTRY_MINUTES: LazyLock<HashMap<String, i64>> = LazyLock::new(|| int_map("PANTRY_MINUTES"));
static CROWD_DISHES: LazyLock<HashSet<String>> = LazyLock::new(|| set("CROWD_DISHES"));
static FOR_ONE: LazyLock<HashSet<String>> = LazyLock::new(|| set("FOR_ONE"));
static THIN: LazyLock<HashSet<String>> = LazyLock::new(|| set("THIN"));
static SWEET_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("SWEET_WORDS"));
static SWEET_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("SWEET_PHRASES"));
static SWEET_DISHES: LazyLock<HashSet<String>> = LazyLock::new(|| set("SWEET_DISHES"));
static DINNER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("DINNER_WORDS"));
static CHATTER: LazyLock<HashSet<String>> = LazyLock::new(|| set("CHATTER"));
static TECHNIQUE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("TECHNIQUE_WORDS"));
static BAKING_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("BAKING_WORDS"));
static DISH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("DISH_WORDS"));
static MEAL_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| set("MEAL_FRAME"));
static SAID_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("SAID_PHRASES"));
static PLAN_KEYS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PLAN_KEYS"));
static AVOID_TEXT: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&c("AVOID_TEXT")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static VEGAN_BAD: LazyLock<Vec<String>> = LazyLock::new(|| strs("VEGAN_BAD"));
static PERSON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("PERSON_WORDS"));
static TASTE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("TASTE_WORDS"));
static SWAP_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("SWAP_PHRASES"));
static METHOD_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("METHOD_PHRASES"));
static NEW_MEAL_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("NEW_MEAL_PHRASES"));
static GROWING_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| set("_GROWING_WORD"));
static STOCK_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("STOCK_PHRASES"));
static ALLERGY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("ALLERGY_WORDS"));
static ALLERGY_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("ALLERGY_PHRASES"));
static A_WISH_FOR_A_FOOD: LazyLock<&'static Re> = LazyLock::new(|| rx("A_WISH_FOR_A_FOOD"));
static QUICK_MINUTES: LazyLock<i64> = LazyLock::new(|| data::get(&c("QUICK_MINUTES")).as_int().unwrap_or(20));
static PET_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("PET_WORDS"));
static MEAL_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("MEAL_PHRASES"));
static INGREDIENT_WORDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map(&c("_INGREDIENT_WORDS")));

/// PANTRY: food name -> (its aliases, its one idea), in order.
static PANTRY: LazyLock<Vec<(String, Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&c("PANTRY"))
        .pairs()
        .iter()
        .map(|(k, v)| {
            let p = v.items();
            (k.str().to_string(), p[0].strs(), p[1].str().to_string())
        })
        .collect()
});
/// MEAL_IDEAS: (names, idea).
static MEAL_IDEAS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&c("MEAL_IDEAS"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].strs(), p[1].str().to_string())
        })
        .collect()
});

/// One of cooking's DISHES: its id, the foods it uses and its text.
#[derive(Debug, Clone)]
pub struct CookDish {
    pub id: String,
    pub uses: Vec<String>,
    pub text: String,
}

static DISHES: LazyLock<Vec<CookDish>> = LazyLock::new(|| {
    data::get(&c("DISHES"))
        .items()
        .iter()
        .map(|d| CookDish {
            id: d.get("id").map(|v| v.str().to_string()).unwrap_or_default(),
            uses: d.get("uses").map(|v| v.strs()).unwrap_or_default(),
            text: d.get("text").map(|v| v.str().to_string()).unwrap_or_default(),
        })
        .collect()
});

fn int_map(name: &str) -> HashMap<String, i64> {
    data::get(&c(name)).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
}

fn pantry(name: &str) -> Option<&'static (String, Vec<String>, String)> {
    PANTRY.iter().find(|(k, _, _)| k == name)
}

/// Whether the line rules the food out for somebody who is not the person typing.
pub fn _about_someone_else(understood: &Understood) -> bool {
    SOMEONE_ELSE.is_match(&_clean(understood))
}

/// True when the food the line names is one it is growing rather than one it has in.
pub fn _a_crop_not_a_larder(understood: &Understood) -> bool {
    let words: HashSet<String> = _said_words(understood).into_iter().collect();
    if !intersects(&words, &GROWING_WORD) {
        return false;
    }
    if _meal_asked(understood) {
        return false;
    }
    true
}

/// The line as typed, lower case.
pub fn _clean(understood: &Understood) -> String {
    understood.clean.to_lowercase()
}

/// The line's words with their plurals undone (`cooking._fold`).
pub fn _said_words(understood: &Understood) -> Vec<String> {
    understood.words.iter().map(|w| cooking::_fold(w)).collect()
}

fn word_set(understood: &Understood) -> HashSet<String> {
    _said_words(understood).into_iter().collect()
}

/// The time a line gives ("20 mins", "half an hour", "an hour and a half"), in minutes.
pub fn minutes_said(understood: &Understood) -> Option<i64> {
    static PATTERNS: LazyLock<Vec<(Re, i64)>> = LazyLock::new(|| {
        vec![
            (Re::new(r"(\d+)\s*(?:mins|min|minutes|minute)\b", 0), 1),
            (Re::new(r"(\d+)\s*(?:hrs|hr|hours|hour)\b", 0), 60),
            (Re::new(r"(\d+)\s*and\s*a\s*half\s*(?:hours|hour|hrs|hr)\b", 0), 90),
        ]
    });
    static HALF: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhalf an hour\b", 0));
    let text = format!(" {} ", _clean(understood));
    for (pattern, unit) in PATTERNS.iter() {
        if let Some(found) = pattern.search(&text) {
            // int() of a long run of digits does not overflow in Python; saturate here.
            let n: i64 = found.str(1).parse().unwrap_or(i64::MAX / 100);
            return Some(n.saturating_mul(*unit));
        }
    }
    if HALF.is_match(&text) {
        return Some(30);
    }
    None
}

/// The pantry name a wished-for word stands for (`_INGREDIENT_WORDS`, allowing a plural).
fn ingredient_word(word: &str) -> Option<String> {
    if let Some(n) = INGREDIENT_WORDS.get(word) {
        return Some(n.clone());
    }
    let folded = cooking::_fold(word);
    for ending in ["", "s", "es"] {
        if let Some(n) = INGREDIENT_WORDS.get(&format!("{folded}{ending}")) {
            return Some(n.clone());
        }
    }
    None
}

/// The named foods this line only wants more or less of, and so is not stock of.
pub fn _the_foods_wanted_more_or_less(said_clean: &str, named: &[String]) -> HashSet<String> {
    let mut wished = HashSet::new();
    if named.is_empty() {
        return wished;
    }
    for found in A_WISH_FOR_A_FOOD.finditer(said_clean) {
        let word = found.str(2).trim_matches(|c| c == ',' || c == '.');
        if let Some(name) = ingredient_word(word) {
            if named.contains(&name) {
                wished.insert(name);
            }
        }
    }
    wished
}

/// The named foods this line says it wants more of, which a later dish is picked from.
pub fn _the_foods_wanted_more(said_clean: &str, named: &[String]) -> HashSet<String> {
    let mut more = HashSet::new();
    if named.is_empty() {
        return more;
    }
    for found in A_WISH_FOR_A_FOOD.finditer(said_clean) {
        if found.str(1) != "more" {
            continue;
        }
        let word = found.str(2).trim_matches(|c| c == ',' || c == '.');
        if let Some(name) = ingredient_word(word) {
            if named.contains(&name) {
                more.insert(name);
            }
        }
    }
    more
}

/// What this line says about the meal, read on top of the plan already in `old`.
pub fn wants(understood: &Understood, old: Option<&Context>) -> Context {
    let mut context: Context = old.cloned().unwrap_or_default();
    let words = word_set(understood);
    let said_clean = _clean(understood);
    let named = cooking::ingredients(understood);
    let mut refused = cooking::negated_ingredients(understood);
    if intersects(&words, &ALLERGY_WORDS) || any_in(&said_clean, &ALLERGY_PHRASES) {
        for name in &named {
            if !refused.contains(name) {
                refused.push(name.clone());
            }
        }
    }
    if _refuses(understood) {
        for name in &named {
            if !refused.contains(name) {
                refused.push(name.clone());
            }
        }
    }
    let wished = _the_foods_wanted_more_or_less(&said_clean, &named);
    if !wished.is_empty() {
        let mut more: Vec<String> = _the_foods_wanted_more(&said_clean, &named).into_iter().collect();
        more.sort();
        for name in more {
            push_unique(&mut context, "fancy", &name);
        }
    }
    for name in &named {
        if refused.contains(name) || wished.contains(name) {
            continue;
        }
        push_unique(&mut context, "have", name);
    }
    for name in &refused {
        push_unique(&mut context, "avoid", name);
    }
    if truthy(&context, "have") && any_in(&said_clean, &ONLY_PHRASES) {
        context.insert("only".into(), Value::Bool(true));
    }
    if let Some(limit) = minutes_said(understood) {
        context.insert("limit".into(), Value::Int(limit));
    }
    if intersects(&words, &VEG_WORDS) || any_in(&_clean(understood), &MEAT_FREE_PHRASES) {
        context.insert("veg".into(), Value::Bool(true));
    }
    if any_in(&said_clean, &DAIRY_FREE_PHRASES) {
        setdefault_list(&mut context, "avoid");
        for name in DAIRY_FOODS.iter() {
            push_unique(&mut context, "avoid", name);
        }
    }
    if words.contains("vegan") {
        context.insert("vegan".into(), Value::Bool(true));
    }
    if CROWD_PHRASE.is_match(&said_clean) {
        context.insert("crowd".into(), Value::Bool(true));
    }
    if (truthy(&context, "veg") || truthy(&context, "vegan"))
        && !truthy(&context, "have")
        && (!truthy(&context, "crowd") || _meal_asked(understood))
    {
        context.insert("diet".into(), Value::Bool(true));
    }
    if truthy(&context, "veg") && truthy(&context, "have") {
        let have = list(&context, "have");
        if have.iter().any(|n| MEAT_NAMES.contains(n)) {
            let left: Vec<String> = have.into_iter().filter(|n| !MEAT_NAMES.contains(n)).collect();
            context.insert("have".into(), vlist(&left));
            if left.is_empty() {
                context.shift_remove("have");
                context.insert("diet".into(), Value::Bool(true));
            }
        }
    }
    let foodish = intersects(&words, &DINNER_WORDS)
        || !cooking::ingredients(understood).is_empty()
        || any_in(&_clean(understood), &COMING_FOR);
    let said_clean = _clean(understood);
    if foodish
        && ((cooking::_occasion(&said_clean) && any_in(&said_clean, &MENU_PHRASES))
            || CROWD_PHRASE.is_match(&said_clean)
            || intersects(&words, &BIG_DAYS)
            || (intersects(&words, &PEOPLE_WORDS) && any_in(&said_clean, &COMING_FOR))
            || (intersects(&words, &EVENT_WORDS) && any_in(&said_clean, &COOKING_FOR)))
    {
        context.insert("occasion".into(), Value::Bool(true));
    }
    if intersects(&words, &LIMIT_WORDS) || any_in(&said_clean, &SIMPLE_PHRASES) {
        context.insert("simple".into(), Value::Bool(true));
    }
    if intersects(&words, &HEAT_WORDS) {
        if _a_taste_ruled_out(&said_clean) {
            context.insert("no_heat".into(), Value::Bool(true));
            setdefault_list(&mut context, "avoid");
            for name in HEAT_INGREDIENTS.iter() {
                push_unique(&mut context, "avoid", name);
            }
        } else {
            context.insert("heat".into(), Value::Bool(true));
        }
    }
    let sweet = intersects(&words, &SWEET_WORDS) || any_in(&_clean(understood), &SWEET_PHRASES);
    if sweet && cooking::ingredients(understood).is_empty() {
        context.insert("sweet".into(), Value::Bool(true));
        context.shift_remove("have");
    }
    let greens: HashSet<String> = GREEN_WORDS.iter().cloned().chain(["veg", "vegetables", "veggies"].map(String::from)).collect();
    if intersects(&words, &DISLIKE_WORDS) && intersects(&words, &greens) {
        context.insert("no_green".into(), Value::Bool(true));
    }
    context
}

/// Whether a dish or an idea is allowed by the plan's refusals.
pub fn _fits(names: &[String], context: &Context) -> bool {
    let names: HashSet<&String> = names.iter().collect();
    let avoid = list(context, "avoid");
    if avoid.iter().any(|a| names.contains(a)) {
        return false;
    }
    if truthy(context, "veg") && names.iter().any(|n| MEAT_NAMES.contains(*n)) {
        return false;
    }
    if truthy(context, "no_green") && names.iter().any(|n| GREEN_NAMES.contains(*n)) {
        return false;
    }
    if truthy(context, "no_heat") && names.iter().any(|n| HEAT_INGREDIENTS.contains(*n)) {
        return false;
    }
    true
}

/// Whether the dish's own words break a refusal: pasta with parmesan is not "no cheese".
pub fn _text_conflict(text: &str, context: &Context) -> bool {
    let text = text.to_lowercase();
    for name in list(context, "avoid") {
        let words = AVOID_TEXT.iter().find(|(k, _)| *k == name).map(|(_, v)| v.clone()).unwrap_or_else(|| vec![name.clone()]);
        if words.iter().any(|w| text.contains(w.as_str())) {
            return true;
        }
    }
    truthy(context, "vegan") && any_in(&text, &VEGAN_BAD)
}

/// The longest a dish may take for this plan: the number they said, or "quick" as twenty.
pub fn _quick_limit(context: &Context) -> Option<i64> {
    if truthy(context, "limit") {
        return context.get("limit").and_then(|v| v.as_int());
    }
    if truthy(context, "simple") {
        Some(*QUICK_MINUTES)
    } else {
        None
    }
}

/// Whether a dish's minutes are inside the plan's limit.
pub fn _quick_enough(minutes: Option<i64>, context: &Context) -> bool {
    let limit = _quick_limit(context).unwrap_or(0);
    let minutes = minutes.unwrap_or(0);
    !(limit != 0 && minutes != 0 && minutes > limit)
}

/// How well a dish answers the plan: what it uses, what it leaves over, and the time.
pub fn _score(dish: &CookDish, context: &Context) -> i64 {
    let have: HashSet<String> = list(context, "have").into_iter().collect();
    let uses: HashSet<String> = dish.uses.iter().cloned().collect();
    let minutes = MINUTES.get(&dish.id).copied();
    let mut score = uses.intersection(&have).count() as i64 * 4 - have.difference(&uses).count() as i64 * 3;
    if let Some(m) = minutes {
        score -= m.div_euclid(25);
        if let Some(limit) = _quick_limit(context) {
            if limit != 0 && m > limit {
                score -= 6;
            }
        }
    }
    if truthy(context, "simple") {
        if let Some(m) = minutes {
            score -= (m - 20).max(0).div_euclid(5);
        }
    }
    if !have.is_empty() && have.is_subset(&uses) {
        score += 4;
    }
    if !have.is_empty() {
        score -= uses.iter().filter(|u| !have.contains(*u) && !STAPLES.contains(*u)).count() as i64;
    } else {
        score -= uses.len() as i64 * 2;
    }
    if truthy(context, "occasion") && THIN.contains(&dish.id) {
        score -= 6;
    }
    if truthy(context, "occasion") && FOR_ONE.contains(&dish.id) {
        score -= 10;
    }
    if truthy(context, "heat") {
        let text = dish.text.to_lowercase();
        score += if any_in(&text, &HEAT_IN_TEXT) { 6 } else { -2 };
    }
    if truthy(context, "occasion") && CROWD_DISHES.contains(&dish.id) {
        score += 8;
        if minutes.unwrap_or(0) >= 30 {
            score += 4;
        } else if context.get("limit").is_none_or(|v| v.is_none()) {
            score -= 2;
        }
    }
    score
}

/// The meal idea for the same ingredient, for a plan that has to be quick.
pub fn _meal_idea_for(name: &str) -> Option<String> {
    let (_, aliases, _) = pantry(name)?;
    let mut words: HashSet<&str> = aliases.iter().map(String::as_str).collect();
    words.insert(name);
    for (names, idea) in MEAL_IDEAS.iter() {
        if names.iter().any(|n| words.contains(n.as_str())) {
            return Some(idea.clone());
        }
    }
    None
}

/// One-ingredient ideas for what is in the plan, in the order the plan mentions it.
pub fn _ideas(context: &Context, limit: bool) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let have = list(context, "have");
    let quick = if limit { _quick_limit(context) } else { None };
    for name in have.iter().rev() {
        let Some((_, _, pantry_idea)) = pantry(name) else { continue };
        if !_fits(std::slice::from_ref(name), context) {
            continue;
        }
        let mut idea = pantry_idea.clone();
        if let Some(q) = quick.filter(|&q| q != 0) {
            if PANTRY_MINUTES.get(name).copied().unwrap_or(0) > q {
                if let Some(faster) = _meal_idea_for(name) {
                    idea = faster;
                } else if limit {
                    continue;
                }
            }
        }
        if cooking::_mentions(&idea, &list(context, "avoid")) || _text_conflict(&idea, context) {
            continue;
        }
        out.push((name.clone(), idea));
    }
    out
}

/// An answer for the plan, or None when the plan has nothing to work with. A dish it offers is
/// added to `memory["meal"]["offered"]` when memory is given.
pub fn plan_reply(context: &Context, memory: Option<&mut Memory>, skip: &[String]) -> Option<String> {
    if context.is_empty() {
        return None;
    }
    let promoted;
    let context = if !truthy(context, "have") && truthy(context, "fancy") {
        let mut c2 = context.clone();
        c2.insert("have".into(), vlist(&list(context, "fancy")));
        promoted = c2;
        &promoted
    } else {
        context
    };
    if truthy(context, "only") {
        let ideas = _ideas(context, truthy(context, "limit") || truthy(context, "simple"));
        if !ideas.is_empty() {
            return Some(ideas.iter().take(2).map(|(_, i)| i.as_str()).collect::<Vec<_>>().join(" "));
        }
    }
    let mut skip_ids: HashSet<String> = skip.iter().cloned().collect();
    if let Some(offered) = context.get("offered") {
        if matches!(offered, Value::List(_) | Value::Tuple(_) | Value::Set(_)) {
            skip_ids.extend(offered.strs());
        }
    }
    let have: HashSet<String> = list(context, "have").into_iter().collect();
    let sweet = truthy(context, "sweet");
    let mut scored: Vec<(i64, &CookDish)> = Vec::new();
    for dish in DISHES.iter() {
        if skip_ids.contains(&dish.id) || !_fits(&dish.uses, context) {
            continue;
        }
        if SWEET_DISHES.contains(&dish.id) != sweet {
            continue;
        }
        let uses: HashSet<String> = dish.uses.iter().cloned().collect();
        if have.is_empty() && !sweet && !truthy(context, "diet") {
            continue;
        }
        if !have.is_empty() && uses.is_disjoint(&have) {
            continue;
        }
        if !have.is_empty() && uses.iter().any(|u| !have.contains(u) && !STAPLES.contains(u)) {
            continue;
        }
        if (truthy(context, "limit") || truthy(context, "simple")) && !_quick_enough(MINUTES.get(&dish.id).copied(), context) {
            continue;
        }
        let lowered = dish.text.to_lowercase();
        if truthy(context, "no_green") && any_in(&lowered, &GREEN_IN_TEXT) {
            continue;
        }
        if truthy(context, "no_heat") && any_in(&lowered, &HEAT_IN_TEXT) {
            continue;
        }
        if _text_conflict(&dish.text, context) {
            continue;
        }
        scored.push((_score(dish, context), dish));
    }
    if !scored.is_empty() {
        scored.sort_by_key(|(s, _)| plan_reply__lambda_1369(*s));
        let best = scored[0].1;
        if let Some(memory) = memory {
            if let Some(Value::Dict(_)) = memory.extra.get("meal") {
                let mut meal = context_from(&memory.extra["meal"]);
                let mut offered: Vec<Value> = meal.get("offered").map(|v| v.items().to_vec()).unwrap_or_default();
                offered.push(vstr(&best.id));
                meal.insert("offered".into(), Value::List(Arc::new(offered)));
                memory.extra.insert("meal".into(), context_value(&meal));
            }
        }
        return Some(_with_heat(&best.text, context));
    }
    let mut ideas = _ideas(context, true);
    if ideas.is_empty() {
        ideas = _ideas(context, false);
    }
    if !ideas.is_empty() {
        return Some(ideas.iter().take(2).map(|(_, i)| i.as_str()).collect::<Vec<_>>().join(" "));
    }
    None
}

/// `plan_reply`'s sort key: the best score first.
pub fn plan_reply__lambda_1369(score: i64) -> i64 {
    -score
}

/// True when the line refuses the heat rather than asking for it.
pub fn _a_taste_ruled_out(said: &str) -> bool {
    for pattern in [*TASTE_RULED_OUT, *HOT_FOOD_RULED_OUT] {
        for found in pattern.finditer(said) {
            let after: String = said[found.end()..].chars().take(12).collect::<String>().to_lowercase();
            if after.trim().starts_with("enough") {
                continue;
            }
            return true;
        }
    }
    false
}

/// The dish, with the heat asked for put into it when its own words have none.
pub fn _with_heat(text: &str, context: &Context) -> String {
    if !truthy(context, "heat") || truthy(context, "no_heat") {
        return text.to_string();
    }
    if any_in(&text.to_lowercase(), &HEAT_IN_TEXT) {
        return text.to_string();
    }
    format!(
        "{} Add chopped chilli or a pinch of chilli flakes with the onion for the heat - taste at the end and add more if it needs it.",
        text.trim_end()
    )
}

/// The plan still in force, or None once the conversation has moved on.
pub fn plan_recent(memory: &Memory) -> Option<Context> {
    let plan = memory.extra.get("meal").filter(|v| matches!(v, Value::Dict(_)))?;
    let turn = int_or_zero(plan.get("turn"));
    if memory.turns.len() as i64 - turn > *PLAN_LAPSE {
        return None;
    }
    Some(context_from(plan))
}

/// `plan_recent(memory)` when it is truthy (a non-empty dict), as the callers test it.
fn recent(memory: &Memory) -> Option<Context> {
    plan_recent(memory).filter(|p| !p.is_empty())
}

/// What the bot said last, so a dish already on the table is not put there twice.
pub fn _last_reply(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

/// What of this line to keep for the next short reply to narrow, or None.
pub fn remembered(understood: &Understood, memory: &Memory) -> Option<Context> {
    if understood.kind == "empty" || understood.kind == "greeting" {
        return None;
    }
    if dup::says_it_has_gone_off(&understood.clean) {
        return None;
    }
    let plan = recent(memory);
    let words = word_set(understood);
    let said = _clean(understood);
    let topic = understood.topic_own.as_deref();
    if intersects(&words, &PET_WORDS) && plan.is_none() {
        return None;
    }
    if plan.is_none() && _a_crop_not_a_larder(understood) {
        return None;
    }
    let ingredients = cooking::ingredients(understood);
    let negated = cooking::negated_ingredients(understood);
    let food = matches!(topic, Some("cooking" | "food-and-drink"))
        || intersects(&words, &DINNER_WORDS)
        || !ingredients.is_empty()
        || !negated.is_empty();
    if !food && plan.is_none() {
        return None;
    }
    let stocked = (!ingredients.is_empty() || !negated.is_empty()) && understood.kind == "statement" && any_in(&said, &STOCK_PHRASES);
    let wish = !_the_foods_wanted_more(&said, &ingredients).is_empty();
    let asking_with_food = !ingredients.is_empty() && any_in(&_clean(understood), &NEW_MEAL_PHRASES);
    let asking_how = any_in(&said, &METHOD_PHRASES)
        && ingredients.len() == 1
        && !intersects(&words, &EVENT_WORDS)
        && !CROWD_PHRASE.is_match(&said);
    if plan.is_none()
        && (intersects(&words, &TECHNIQUE_WORDS)
            || asking_how
            || (!_meal_asked(understood) && !stocked && !asking_with_food && !wish))
    {
        return None;
    }
    let new = wants(understood, None);
    let mut kept: Context = IndexMap::new();
    for key in PLAN_KEYS.iter() {
        if let Some(v) = new.get(key) {
            kept.insert(key.clone(), v.clone());
        }
    }
    if truthy(&new, "fancy") {
        kept.insert("fancy".into(), new["fancy"].clone());
    }
    if kept.is_empty() && !food {
        return None;
    }
    if plan.is_none() && !kept.is_empty() && kept.keys().all(|k| k == "veg" || k == "vegan") {
        let last = memory.turns.last().map(|t| t.user.to_lowercase()).unwrap_or_default();
        let last_ask: HashSet<String> = last.split_whitespace().map(str::to_string).collect();
        if !intersects(&last_ask, &DINNER_WORDS) {
            return None;
        }
    }
    if plan.is_none() && kept.is_empty() && understood.kind != "question" {
        return None;
    }
    if let Some(p) = &plan {
        if truthy(p, "have") && _asks_about_a_food_it_has_not_got(understood, p) {
            return plan;
        }
    }
    let mut merged = wants(understood, plan.as_ref());
    let turns = memory.turns.len() as i64;
    if !merged.contains_key("started") {
        merged.insert("started".into(), Value::Int(turns));
    }
    merged.insert("turn".into(), Value::Int(turns));
    let mut offered: Vec<String> = merged
        .get("offered")
        .map(|v| v.items().iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    let said_last = _last_reply(memory);
    if !said_last.is_empty() {
        for dish in DISHES.iter() {
            if !offered.contains(&dish.id) && said_last.contains(dish.text.as_str()) {
                offered.push(dish.id.clone());
            }
        }
    }
    if !offered.is_empty() {
        let start = offered.len().saturating_sub(4);
        merged.insert("offered".into(), vlist(&offered[start..]));
    }
    let allergies = _allergy_names(memory);
    if !allergies.is_empty() {
        let have = list(&merged, "have");
        let gone: Vec<String> = have.iter().filter(|n| _is_allergy_food(n, &allergies)).cloned().collect();
        if !gone.is_empty() {
            let left: Vec<String> = have.iter().filter(|n| !gone.contains(n)).cloned().collect();
            if !left.is_empty() {
                merged.insert("have".into(), vlist(&left));
            } else {
                merged.shift_remove("have");
            }
            let mut avoid = list(&merged, "avoid");
            for name in gone {
                if !avoid.contains(&name) {
                    avoid.push(name);
                }
            }
            merged.insert("avoid".into(), vlist(&avoid));
        }
    }
    Some(merged)
}

/// The foods the person has said they cannot eat, as the memory lane keeps them.
pub fn _allergy_names(memory: &Memory) -> Vec<String> {
    let Some(user) = memory.extra.get("user").filter(|v| v.truthy()) else { return Vec::new() };
    let Some(allergies) = user.get("allergies") else { return Vec::new() };
    allergies
        .items()
        .iter()
        .map(py_str)
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// `str(value)` for the scalar values a memory record holds.
fn py_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => crate::py::text::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        other => other.str().to_string(),
    }
}

/// Whether a food is one of the allergies, allowing for "peanut" and "peanuts".
pub fn _is_allergy_food(name: &str, allergies: &[String]) -> bool {
    let folded = cooking::_fold(name);
    let stem: String = folded.chars().take(4).collect();
    !stem.is_empty()
        && allergies.iter().filter(|a| a.chars().count() >= 4).any(|a| {
            let head: String = a.chars().take(4).collect();
            head.contains(&stem) || folded.contains(&head)
        })
}

static ABOUT_A_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ABOUT_A_FOOD"));
static ABSENCE: LazyLock<&'static Re> = LazyLock::new(|| rx("ABSENCE"));

/// Whether this line asks about a food of its own rather than narrowing the dish.
pub fn _asks_about_a_food_it_has_not_got(understood: &Understood, plan: &Context) -> bool {
    let have = list(plan, "have");
    let named: Vec<String> = cooking::ingredients(understood).into_iter().filter(|n| !have.contains(n)).collect();
    let avoid = list(plan, "avoid");
    if named.len() != 1 || named.iter().any(|n| avoid.contains(n)) {
        return false;
    }
    if intersects(&word_set(understood), &DINNER_WORDS) {
        return false;
    }
    let said = _clean(understood);
    if any_in(&said, &ABOUT_A_FOOD) {
        return true;
    }
    ABSENCE.is_match(&said)
}

const HOW_ASKS: [&str; 6] = ["how", "how_long", "how_much", "where", "how_often", "why"];

/// The plan a short line adds to, or None when the line is not narrowing one.
pub fn _narrowing(understood: &Understood, plan: Option<&Context>, longest: usize) -> Option<Context> {
    let words = word_set(understood);
    if words.is_empty() || words.len() > longest {
        return None;
    }
    if ["empty", "greeting", "thanks"].contains(&understood.kind.as_str()) {
        return None;
    }
    let said = _clean(understood);
    let own = understood.topic_own.as_deref();
    if !matches!(own, None | Some("cooking") | Some("food-and-drink")) && cooking::ingredients(understood).is_empty() {
        let w = wants(understood, None);
        let narrowed = PLAN_KEYS.iter().any(|k| k != "simple" && w.contains_key(k));
        if !(plan.is_some_and(|p| !p.is_empty()) && (narrowed || w.contains_key("limit"))) {
            return None;
        }
    }
    if words.iter().all(|w| CHATTER.contains(w)) && !intersects(&words, &DINNER_WORDS) {
        return None;
    }
    if any_in(&said, &SWAP_PHRASES) {
        return None;
    }
    if any_in(&said, &WITH_PHRASES) {
        return None;
    }
    if understood.asks.iter().any(|a| HOW_ASKS.contains(&a.as_str())) {
        return None;
    }
    if intersects(&words, &TECHNIQUE_WORDS) && minutes_said(understood).is_none() {
        return None;
    }
    if understood.asks.iter().any(|a| a == "yesno")
        && (!cooking::ingredients(understood).is_empty()
            || ["it", "its", "that", "this", "them", "those"].iter().any(|w| words.contains(*w)))
    {
        return None;
    }
    if !cooking::ingredients(understood).is_empty() && any_in(&said, &NEW_MEAL_PHRASES) {
        return None;
    }
    if intersects(&words, &BAKING_WORDS) && !["dinner", "tea", "supper", "lunch", "meal"].iter().any(|w| words.contains(*w)) {
        return None;
    }
    if any_in(&said, &SAID_PHRASES) {
        return None;
    }
    if intersects(&words, &PERSON_WORDS)
        && intersects(&words, &TASTE_WORDS)
        && !intersects(&words, &DINNER_WORDS)
        && minutes_said(understood).is_none()
    {
        return None;
    }
    let new = wants(understood, None);
    if truthy(&new, "heat") && plan.is_some_and(|p| truthy(p, "sweet")) {
        return None;
    }
    let mut kept = PLAN_KEYS.iter().any(|k| k != "simple" && new.contains_key(k));
    kept = kept
        || new.contains_key("limit")
        || intersects(&words, &LIMIT_WORDS)
        || new.contains_key("heat")
        || any_in(&said, &SIMPLE_PHRASES);
    let asked = intersects(&words, &DINNER_WORDS)
        || MEAL_PHRASES.iter().any(|p| !BARE_MEAL_NOUNS.contains(p) && said.contains(p.as_str()))
        || _the_bare_ideas_are_about_food(understood, &said);
    let pointed = plan.is_some_and(|p| PLAN_KEYS.iter().any(|k| truthy(p, k)));
    let asking = understood.kind == "question" || !understood.asks.is_empty();
    if !kept && !(asked && pointed && asking) {
        return None;
    }
    Some(wants(understood, plan))
}

static BARE_MEAL_NOUNS: LazyLock<Vec<String>> = LazyLock::new(|| strs("BARE_MEAL_NOUNS"));
static BARE_MEAL_OBJECT: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_MEAL_OBJECT"));
static THE_FOOD_IN_HAND: LazyLock<HashSet<String>> = LazyLock::new(|| set("THE_FOOD_IN_HAND"));
static ONLY_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("ONLY_PHRASES"));
static WITH_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("WITH_PHRASES"));
static PICK_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("PICK_PHRASES"));

/// Whether a line asking for "ideas" without a meal word is about the food on the table.
pub fn _the_bare_ideas_are_about_food(understood: &Understood, said: &str) -> bool {
    static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
    if !any_in(said, &BARE_MEAL_NOUNS) {
        return false;
    }
    if !cooking::ingredients(understood).is_empty() {
        return true;
    }
    let Some(named) = BARE_MEAL_OBJECT.search(said) else { return true };
    WORDS.findall_str(&named.str(1).to_lowercase()).iter().any(|w| THE_FOOD_IN_HAND.contains(w))
}

/// The meal idea the last answer gave, when it was built from an ingredient and not a dish.
pub fn _idea_on_the_plate(plan: &Context, memory: &Memory) -> Option<String> {
    let said = _last_reply(memory);
    if said.is_empty() {
        return None;
    }
    let lowered = said.to_lowercase();
    for name in list(plan, "have") {
        if let Some((_, aliases, _)) = pantry(&name) {
            if aliases.iter().any(|a| lowered.contains(a.as_str())) {
                return Some(said);
            }
        }
    }
    None
}

/// The dish to put forward when they leave the choice to the bot: the last one offered.
pub fn _chosen(plan: &Context) -> Option<String> {
    let offered: Vec<String> = plan.get("offered").map(|v| v.strs()).unwrap_or_default();
    for name in offered.iter().rev() {
        if let Some(dish) = DISHES.iter().rev().find(|d| &d.id == name) {
            if _fits(&dish.uses, plan) && !_text_conflict(&dish.text, plan) {
                return Some(dish.text.clone());
            }
        }
    }
    None
}

static A_REACTION: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_REACTION"));

/// Whether the line names an allergy, in any shape the kitchen already reads as one.
pub fn _an_allergy(understood: &Understood) -> bool {
    let said = _clean(understood);
    if intersects(&word_set(understood), &ALLERGY_WORDS) {
        return true;
    }
    any_in(&said, &A_REACTION)
}

/// An answer for a line that only rules something out of the dish already on the table.
pub fn _nothing_to_change(understood: &Understood, memory: &Memory) -> Option<String> {
    let new = wants(understood, None);
    if PLAN_KEYS.iter().any(|k| k != "avoid" && new.contains_key(k)) {
        return None;
    }
    let plan = plan_recent(memory).unwrap_or_default();
    if !truthy(&plan, "offered") {
        return None;
    }
    let kept = list(&new, "avoid");
    let said = _last_reply(memory);
    if kept.is_empty() || said.is_empty() {
        return None;
    }
    let names = kept.iter().take(2).cloned().collect::<Vec<_>>().join(" or ");
    if _an_allergy(understood) {
        let banned: BTreeSet<String> = cooking::_excluded(&_clean(understood)).into_iter().collect();
        let mine: Vec<String> = _said_words(understood).into_iter().filter(|w| banned.contains(w)).collect();
        let theirs = _about_someone_else(understood);
        let in_dish: Vec<&String> = banned.iter().filter(|w| cooking::_mentions(&said, std::slice::from_ref(*w))).collect();
        let who = if theirs { "theirs" } else { "that one" };
        if let Some(left_out) = in_dish.first() {
            let mine2 = mine.iter().take(2).cloned().collect::<Vec<_>>().join(" or ");
            let where_ = if mine2.is_empty() { left_out.to_string() } else { mine2 };
            return Some(format!(
                "The {left_out} is where the {where_} is, so leave it out of {who} and keep the rest the same - and read the label on anything that came in a packet before you use it."
            ));
        }
        if !mine.is_empty() {
            let foods = mine.iter().take(2).cloned().collect::<Vec<_>>().join(" or ");
            let them = if theirs { " for them" } else { "" };
            return Some(format!(
                "There is no {foods} in that dish as it stands, so it is safe{them} - just keep the boards and knives clean, and read the label on anything that came in a packet before you use it."
            ));
        }
        return None;
    }
    if _about_someone_else(understood) {
        if cooking::_mentions(&said, &kept) {
            return Some(format!("Then make theirs without the {names} and keep the rest the same - it works."));
        }
        return Some(format!("That one has no {names} in it anyway, so theirs is fine."));
    }
    if cooking::_mentions(&said, &kept) {
        return Some(format!("Leave the {names} out of that one and it still works."));
    }
    Some(format!("That one has no {names} in it anyway, so you're fine."))
}

static ACK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("ACK_WORDS"));

/// True when the line is a yes, a thanks or an echo of advice, and asks for nothing.
pub fn _only_an_acknowledgement(text: &str) -> bool {
    static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
    let words = WORDS.findall_str(&text.to_lowercase());
    if words.is_empty() || words.len() > 7 {
        return false;
    }
    words.iter().all(|w| ACK_WORDS.contains(w))
}

static A_SUM_OF_MONEY: LazyLock<&'static Re> = LazyLock::new(|| rx("A_SUM_OF_MONEY"));

/// The next line of a cooking conversation that only narrows the dish already on the table.
pub fn follow_up(understood: &Understood, memory: &mut Memory) -> Option<String> {
    if dup::says_it_has_gone_off(&understood.clean) {
        return None;
    }
    if let Some(pointed_food) = cooking::pointed_at_a_food(understood, memory).filter(|s| !s.is_empty()) {
        return Some(pointed_food);
    }
    let plan = recent(memory)?;
    let said = _clean(understood);
    if dup::asks_the_kinds(understood) {
        return None;
    }
    if _only_an_acknowledgement(&said) {
        return None;
    }
    if A_SUM_OF_MONEY.is_match(&said) {
        return None;
    }
    if cooking::_a_brew_not_a_dinner(&said) {
        return None;
    }
    if said.split_whitespace().count() <= 6 && any_in(&said, &PICK_PHRASES) {
        return _chosen(&plan).or_else(|| _idea_on_the_plate(&plan, memory));
    }
    if int_or_zero(plan.get("started")) >= memory.turns.len() as i64 {
        return None;
    }
    let Some(mut context) = _narrowing(understood, Some(&plan), 10) else {
        if truthy(&plan, "fancy") && !truthy(&plan, "have") && _a_bare_meal_ask(understood) {
            let mut promoted = plan.clone();
            promoted.insert("have".into(), vlist(&list(&plan, "fancy")));
            return plan_reply(&promoted, Some(memory), &[]);
        }
        return None;
    };
    if truthy(&context, "sweet") && !(truthy(&plan, "have") || truthy(&plan, "offered")) {
        return None;
    }
    let plan_have = list(&plan, "have");
    let added: Vec<String> = list(&context, "have").into_iter().filter(|n| !plan_have.contains(n)).collect();
    if !added.is_empty() && !(truthy(&context, "avoid") || truthy(&context, "veg") || truthy(&context, "vegan")) {
        return None;
    }
    let mut answering = context.clone();
    if !truthy(&context, "have") && truthy(&context, "fancy") {
        answering.insert("have".into(), vlist(&list(&context, "fancy")));
    }
    let reply = match _nothing_to_change(understood, memory).filter(|s| !s.is_empty()) {
        Some(r) => Some(r),
        None => plan_reply(&answering, Some(memory), &[]),
    };
    let mut reply = reply?;
    if reply.trim() == _last_reply(memory).trim() {
        let others: Vec<String> = _ideas(&answering, false).into_iter().map(|(_, i)| i).filter(|i| *i != reply).collect();
        if !others.is_empty() {
            reply = others.iter().take(2).cloned().collect::<Vec<_>>().join(" ");
        }
    }
    if let Some(meal @ Value::Dict(_)) = memory.extra.get("meal") {
        let offered: Vec<Value> = match context.get("offered").filter(|v| v.truthy()) {
            Some(v) => v.items().to_vec(),
            None => meal.get("offered").filter(|v| v.truthy()).map(|v| v.items().to_vec()).unwrap_or_default(),
        };
        let start = offered.len().saturating_sub(4);
        context.insert("offered".into(), Value::List(Arc::new(offered[start..].to_vec())));
        memory.extra.insert("meal".into(), context_value(&context));
    }
    Some(reply)
}

static BARE_MEAL_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("BARE_MEAL_ASK_WORDS"));

/// Whether the line asks for a dinner and names nothing else: "what should i cook tonight".
pub fn _a_bare_meal_ask(understood: &Understood) -> bool {
    if !cooking::ingredients(understood).is_empty()
        || understood.asks.iter().any(|a| ["how", "how_long", "how_much", "why"].contains(&a.as_str()))
    {
        return false;
    }
    let words = word_set(understood);
    if words.is_empty() || words.iter().any(|w| !BARE_MEAL_ASK_WORDS.contains(w)) {
        return false;
    }
    ["cook", "make", "cooking", "dinner", "tea", "supper", "lunch", "meal", "eat"].iter().any(|w| words.contains(*w))
}

static REFUSAL: LazyLock<&'static Re> = LazyLock::new(|| rx("REFUSAL"));

/// Whether the line rules a food out: "they won't eat mushrooms", "she doesnt like onions".
pub fn _refuses(understood: &Understood) -> bool {
    if intersects(&word_set(understood), &DISLIKE_WORDS) {
        return true;
    }
    REFUSAL.is_match(&_clean(understood))
}

static THE_WAY_TO_COOK: LazyLock<&'static Re> = LazyLock::new(|| rx("THE_WAY_TO_COOK"));

/// Is this line a meal being asked for, rather than a dish, a bake or two foods?
pub fn _meal_asked(understood: &Understood) -> bool {
    let said = _clean(understood);
    if THE_WAY_TO_COOK.is_match(&said) {
        return false;
    }
    let words = word_set(understood);
    if !intersects(&words, &MEAL_FRAME) {
        return false;
    }
    if intersects(&words, &DISH_WORDS) || intersects(&words, &BAKING_WORDS) {
        return false;
    }
    if ["breakfast", "lunch", "brunch"].iter().any(|w| words.contains(*w)) {
        return false;
    }
    if cooking::ingredients(understood).len() > 1 {
        return false;
    }
    !any_in(&said, &SAID_PHRASES)
}

static FOOD_PROPERTY: LazyLock<&'static Re> = LazyLock::new(|| rx("FOOD_PROPERTY"));
static COOK_REQUEST: LazyLock<&'static Re> = LazyLock::new(|| rx("COOK_REQUEST"));

/// Whether the line asks a fact about the food it names instead of asking what to cook.
pub fn asks_a_food_fact(understood: &Understood) -> bool {
    let said = _clean(understood);
    if said.is_empty() || COOK_REQUEST.is_match(&said) {
        return false;
    }
    if understood.asks.is_empty() && understood.kind != "question" {
        return false;
    }
    FOOD_PROPERTY.is_match(&said)
}

/// A meal for the first line of a conversation when it says what the meal has to be.
pub fn first_plan(understood: &Understood, memory: Option<&Memory>) -> Option<String> {
    let turns = memory.map(|m| m.turns.len() as i64).unwrap_or(0);
    if let Some(plan) = memory.and_then(recent) {
        if int_or_zero(plan.get("started")) < turns {
            return None;
        }
    }
    let context = _narrowing(understood, None, 14)?;
    if !_meal_asked(understood) {
        return None;
    }
    if !["limit", "veg", "vegan", "no_green", "occasion", "simple", "sweet"].iter().any(|k| context.contains_key(*k)) {
        return None;
    }
    let only = |allowed: &[&str]| {
        context.keys().filter(|k| *k != "started" && *k != "turn").all(|k| allowed.contains(&k.as_str()))
    };
    if only(&["veg", "vegan", "diet", "simple", "limit"]) {
        return None;
    }
    if _left_to_the_meal_entry(understood, &context) {
        return None;
    }
    plan_reply(&context, None, &[])
}

static THIN_CUPBOARD_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("THIN_CUPBOARD_PHRASES"));

/// True when this module's meal entries already answer the line well.
pub fn _left_to_the_meal_entry(understood: &Understood, context: &Context) -> bool {
    let text = if understood.clean.is_empty() {
        understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ")
    } else {
        understood.clean.clone()
    };
    if cooking::_crowd(&text) >= 6 {
        return true;
    }
    if cooking::_occasion(&text) && ["cook", "dinner", "meal", "make", "eat"].iter().any(|p| text.contains(p)) {
        return true;
    }
    if context.keys().filter(|k| *k != "started" && *k != "turn").all(|k| k == "sweet") {
        // UNRESOLVED(lookup.find): the matching engine is not ported. Python wraps the call in
        // try/except and carries on, which is what an engine that cannot answer does here.
        if let Some(true) = dup::lookup_find_is_some(understood) {
            return true;
        }
    }
    if context.keys().all(|k| k == "sweet") && any_in(&text, &THIN_CUPBOARD_PHRASES) {
        return true;
    }
    if context.keys().all(|k| k == "simple" || k == "started" || k == "turn") {
        return true;
    }
    false
}

// ---------------------------------------------------------------------------------------------
// Seeds, and a meal for the slot

static POMEGRANATE_SEEDS: LazyLock<String> = LazyLock::new(|| string("_POMEGRANATE_SEEDS"));
static TOMATO_SEEDS: LazyLock<String> = LazyLock::new(|| string("_TOMATO_SEEDS"));
static CUCUMBER_SEEDS: LazyLock<String> = LazyLock::new(|| string("_CUCUMBER_SEEDS"));
static MELON_SEEDS: LazyLock<String> = LazyLock::new(|| string("_MELON_SEEDS"));
static SQUASH_SEEDS: LazyLock<String> = LazyLock::new(|| string("_SQUASH_SEEDS"));
static CHILLI_SEEDS: LazyLock<String> = LazyLock::new(|| string("_CHILLI_SEEDS"));

/// The seeds out, by the fruit's own method.
pub fn _deseed(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words = understood.words.join(" ");
    let families: [(&[&str], &String); 5] = [
        (&["pomegranate", "pomegranates"], &POMEGRANATE_SEEDS),
        (&["tomato", "tomatoes"], &TOMATO_SEEDS),
        (&["cucumber", "cucumbers", "courgette", "courgettes", "marrow"], &CUCUMBER_SEEDS),
        (&["melon", "melons", "watermelon"], &MELON_SEEDS),
        (&["pumpkin", "squash", "butternut"], &SQUASH_SEEDS),
    ];
    for (family, reply) in families {
        if family.iter().any(|w| words.contains(w)) {
            return Some(reply.clone());
        }
    }
    Some(CHILLI_SEEDS.clone())
}

fn r_deseed(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _deseed(m, u)
}

static SLOT_IDEAS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&c("SLOT_IDEAS"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].str().to_string(), p[1].str().to_string())
        })
        .collect()
});

/// Whether the line is one person's meal rather than a table to cook for.
pub fn _one_person_at_this_meal(understood: &Understood) -> bool {
    for line in [&understood.text, &understood.clean] {
        let line = line.to_lowercase();
        if !line.is_empty() && cooking::_table_text(&line) {
            return false;
        }
    }
    true
}

fn g_one_person_at_this_meal(_f: &FnRef, u: &Understood) -> bool {
    _one_person_at_this_meal(u)
}

/// A shape for the meal the line names, when it names no food to cook.
pub fn _a_meal_for_the_slot(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = format!(" {} ", understood.clean.to_lowercase());
    for (slot, reply) in SLOT_IDEAS.iter() {
        if text.contains(&format!(" {slot}")) || text.contains(&format!("{slot}s ")) {
            return Some(reply.clone());
        }
    }
    None
}

fn r_a_meal_for_the_slot(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _a_meal_for_the_slot(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.cooking_more:_about_the_food:638", g_about_the_food),
    ("bot.knowledge.cooking_more:_one_person_at_this_meal:2236", g_one_person_at_this_meal),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.cooking_more:_a_meal_for_the_slot:2254", r_a_meal_for_the_slot),
    ("bot.knowledge.cooking_more:_a_side_or_the_three_jobs:73", r_a_side_or_the_three_jobs),
    ("bot.knowledge.cooking_more:_buffet_holding:662", r_buffet_holding),
    ("bot.knowledge.cooking_more:_deseed:2155", r_deseed),
    ("bot.knowledge.cooking_more:_flatbread_answer.<locals>.answer:383", r_flatbread_answer),
    ("bot.knowledge.cooking_more:_produce_brown:552", r_produce_brown),
    ("bot.knowledge.cooking_more:_wake_food:654", r_wake_food),
];
