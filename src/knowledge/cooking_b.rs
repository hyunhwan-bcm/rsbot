//! Port of `bot/knowledge/cooking.py`, second half (functions from line 6000 of the original):
//! breakfast and lunch for a diet, freezing, nut and gluten facts, vegan bakes, nutrients and
//! calories, kitchen without an oven, cooking words, bakes and their faults, burnt and greasy
//! food, smoking oil, the ingredient reader, the pantry meal picker (`meal`), and the follow-ups
//! that lean on the last answer. The first half is `cooking.rs`; `cooking.rs` re-exports this.
//!
//! The tables (BAKES, PANTRY, DISHES, RECIPES, the word lists and regexes) come from the bundle.
//! `_fold_meal_ideas` only reshapes PANTRY at import time, so the bundle's PANTRY is already the
//! result; `_recipe_only_when_asked_for_this` and `_bread_rise_answer` build closures at import
//! time, and their inner `answer` functions are ported below.

// cooking.py's functions before line 6000 (cooking.rs).
use super::cooking::{
    _a_brew_not_a_dinner, _a_plant_not_a_dinner, _any_meal, _crowd, _crowd_ask, _crowd_meal, _excluded, _feeding_ask,
    _food_named, _foods_in_list, _is_plural, _leftover, _list_words, _named_as, _no_meat_also_means_fish, _occasion,
    _pet_not_dinner, _pet_or_meat, _porridge, _reads_as_a_pet, _said_so_far, _same, avoids,
};
// Helpers of other modules, until their owners' ports are merged.
use super::d2dup_cookB::{
    a_dish_plan_has_a_heat, chars_after, choosing, idea_is_a_dish, meets, named_food, recent, use_it_up,
};
use crate::data::{self, FnRef, Value};
use crate::knowledge::d2_shared::{or, py_repr_list, py_repr_str, says_it_has_gone_off};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// The bundle's tables

fn c(name: &str) -> String {
    format!("bot.knowledge.cooking.{name}")
}
fn cstrs(name: &str) -> Vec<String> {
    data::strs(&c(name))
}
fn cset(name: &str) -> HashSet<String> {
    data::set(&c(name))
}
fn cre(name: &str) -> &'static Re {
    data::re(&c(name))
}
fn cstring(name: &str) -> String {
    data::string(&c(name))
}
fn cget(name: &str) -> &'static Value {
    data::get(&c(name))
}
/// A `{str: str}` table in order.
fn cstr_map(name: &str) -> IndexMap<String, String> {
    cget(name).pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string()))).collect()
}
/// A `{str: tuple of str}` table in order.
fn cstrs_map(name: &str) -> IndexMap<String, Vec<String>> {
    cget(name).pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.strs()))).collect()
}
/// A tuple of `(tuple of str, str)` rows.
fn cnames_rows(name: &str) -> Vec<(Vec<String>, String)> {
    cget(name).items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str().to_string())).collect()
}
/// A tuple of `(str, str)` rows.
fn cpairs(name: &str) -> Vec<(String, String)> {
    cget(name).items().iter().map(|row| (row.items()[0].str().to_string(), row.items()[1].str().to_string())).collect()
}

/// Python `str(x)` for a number or a string in a table; `repr` for anything else.
fn py_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        other => py_repr(other),
    }
}

/// Python `repr(x)` for the plain values the tables hold.
fn py_repr(v: &Value) -> String {
    match v {
        Value::None => "None".into(),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Str(s) => py_repr_str(s),
        Value::List(xs) => format!("[{}]", xs.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Tuple(xs) => {
            if xs.len() == 1 {
                format!("({},)", py_repr(&xs[0]))
            } else {
                format!("({})", xs.iter().map(py_repr).collect::<Vec<_>>().join(", "))
            }
        }
        Value::Set(xs) => format!("{{{}}}", xs.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Dict(d) => format!(
            "{{{}}}",
            d.iter().map(|(k, x)| format!("{}: {}", py_repr(k), py_repr(x))).collect::<Vec<_>>().join(", ")
        ),
        _ => String::new(),
    }
}

/// `str(understood.get(key) or "")` for the "norm" list: Python prints the list itself.
fn norm_str(u: &Understood) -> String {
    if u.norm.is_empty() { String::new() } else { py_repr_list(&u.norm) }
}

/// `" ".join(str(understood.get(k) or "") for k in ("clean", "norm_text", "norm")).lower()`.
fn clean_norm_said(u: &Understood) -> String {
    format!("{} {} {}", u.clean, u.norm_text, norm_str(u)).to_lowercase()
}

/// A reading built from a few keys, as the Python builds `{"words": ..., "clean": ...}`.
fn reading_of(words: Vec<String>, clean: &str) -> Understood {
    Understood { words, clean: clean.to_string(), ..Default::default() }
}

/// The first character upper-cased (`s[:1].upper() + s[1:]`).
fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// `max(items, key=len)`: the first of the longest (by characters).
fn first_longest(items: &[String]) -> String {
    let mut best: Option<&String> = None;
    for item in items {
        if best.is_none_or(|b| text::len(item) > text::len(b)) {
            best = Some(item);
        }
    }
    best.cloned().expect("max() arg is an empty sequence")
}

/// `re.search(r"\b(?:a|b|...)\b", ...)` built from a list of phrases.
fn phrases_re(phrases: &[String]) -> Re {
    let alts: Vec<String> = phrases.iter().map(|p| escape(p)).collect();
    Re::new(&format!(r"\b(?:{})\b", alts.join("|")), 0)
}

fn sv(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

static WORDS_AZ: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static WORDS_AZ09: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));
static LETTERS_ONLY: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));

static DAIRY: LazyLock<HashSet<String>> = LazyLock::new(|| cset("DAIRY"));
static EGGS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("EGGS"));
static GLUTEN: LazyLock<HashSet<String>> = LazyLock::new(|| cset("GLUTEN"));
static MEAT: LazyLock<HashSet<String>> = LazyLock::new(|| cset("MEAT"));
static FAMILIES: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| {
    cget("FAMILIES").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.str_set()))).collect()
});

/// PANTRY: food -> (the names it goes by, the idea for it), in the table's order.
pub static PANTRY: LazyLock<IndexMap<String, (Vec<String>, String)>> = LazyLock::new(|| {
    cget("PANTRY")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), (v.items()[0].strs(), v.items()[1].str().to_string()))))
        .collect()
});

static INGREDIENT_WORDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map(&c("_INGREDIENT_WORDS")));

// ---------------------------------------------------------------------------------------------
// Word folding

/// A word with its plural folded away ("pancakes" -> "pancake", "berries" -> "berry"), lower-case,
/// with the punctuation round it stripped.
pub fn _fold(word: &str) -> String {
    let lowered = word.to_lowercase().replace('\u{2019}', "'");
    let word = text::strip_chars(&lowered, ".,!?;:'\"");
    let n = text::len(word);
    if n > 4 && word.ends_with("ies") {
        return format!("{}y", &word[..word.len() - 3]);
    }
    if n > 4 && word.ends_with("oes") {
        return word[..word.len() - 2].to_string();
    }
    if n > 3 && word.ends_with("es") {
        let third = word.chars().rev().nth(2).unwrap_or(' ');
        if "sxzh".contains(third) {
            return word[..word.len() - 2].to_string();
        }
    }
    if n > 2 && word.ends_with('s') && !word.ends_with("ss") {
        return word[..word.len() - 1].to_string();
    }
    word.to_string()
}

fn folded_set(words: &[&str]) -> HashSet<String> {
    words.iter().map(|w| _fold(w)).collect()
}

/// `understood.get("words")`, falling back to the clean line split when the reading has no word
/// list. A typed reading always has its list (possibly empty), so this is the words.
fn words_or_clean(u: &Understood) -> Vec<String> {
    u.words.clone()
}

// ---------------------------------------------------------------------------------------------
// Breakfast and lunch for a diet

static SHORT_OF_ASK: LazyLock<&'static Re> = LazyLock::new(|| cre("SHORT_OF_ASK"));
static ANY_BREAKFAST: LazyLock<String> = LazyLock::new(|| cstring("ANY_BREAKFAST"));
static BREAKFAST_TAIL: LazyLock<String> = LazyLock::new(|| cstring("BREAKFAST_TAIL"));
static ANY_LUNCH: LazyLock<String> = LazyLock::new(|| cstring("ANY_LUNCH"));

/// A breakfast idea: a fixed one, or the porridge that depends on the diet.
enum Idea {
    Said(String),
    Porridge,
}

static BREAKFAST_CHOICES: LazyLock<Vec<(HashSet<String>, Idea)>> = LazyLock::new(|| {
    cget("BREAKFAST_CHOICES")
        .items()
        .iter()
        .map(|row| {
            let needs = row.items()[0].str_set();
            let idea = match &row.items()[1] {
                Value::Str(s) => Idea::Said(s.to_string()),
                _ => Idea::Porridge, // the one lambda: `lambda banned: _porridge(banned)`
            };
            (needs, idea)
        })
        .collect()
});

/// The foods a line says they have none of, as opposed to cannot eat.
pub fn _short_of_food(text: &str) -> HashSet<String> {
    let low = text.to_lowercase();
    let mut found = HashSet::new();
    for m in SHORT_OF_ASK.finditer(&low) {
        found.extend(_foods_in_list(&_list_words(chars_after(&low, m.end(), 60), false)));
    }
    found
}

/// The second sentence of a diet breakfast, so the answer is two sentences like the plain one.
pub fn _rushed(banned: &HashSet<String>) -> String {
    if !meets(&EGGS, banned) {
        return " If mornings are rushed, boil an egg the night before and take a piece of fruit.".to_string();
    }
    " If mornings are rushed, put it out the night before and eat it on the way, because a breakfast you can pick up beats one you skip.".to_string()
}

/// Breakfast for someone who has just said what they cannot eat.
pub fn _breakfast_ideas(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = _said_so_far(understood, memory);
    let banned = _no_meat_also_means_fish(&_excluded(&text));
    let short = _short_of_food(&text);
    if banned.is_empty() && short.is_empty() {
        return Some(ANY_BREAKFAST.clone());
    }
    let banned: HashSet<String> = banned.union(&short).cloned().collect();
    let mut ideas: Vec<String> = Vec::new();
    for (needs, idea) in BREAKFAST_CHOICES.iter() {
        if meets(needs, &banned) {
            continue;
        }
        ideas.push(match idea {
            Idea::Said(s) => s.clone(),
            Idea::Porridge => _porridge(&banned),
        });
        if ideas.len() == 3 {
            break;
        }
    }
    if ideas.is_empty() {
        return Some(format!(
            "A bowl of fruit and a glass of juice, or a slice of toast with jam if bread is allowed{}{}",
            *BREAKFAST_TAIL,
            _rushed(&banned)
        ));
    }
    if ideas.len() == 1 {
        return Some(format!("{}{}{}", ideas[0], *BREAKFAST_TAIL, _rushed(&banned)));
    }
    let said = format!(
        "{}{}{}",
        ideas[..ideas.len() - 1].join(", "),
        if ideas.len() == 2 { " or " } else { ", or " },
        ideas[ideas.len() - 1]
    );
    Some(format!("{said}{}{}", *BREAKFAST_TAIL, _rushed(&banned)))
}

/// A packed lunch for someone who has just said what they cannot eat.
pub fn _lunch_ideas(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = _said_so_far(understood, memory);
    let banned = _no_meat_also_means_fish(&_excluded(&text));
    if banned.is_empty() {
        return Some(ANY_LUNCH.clone());
    }
    let rows: [(&str, HashSet<String>); 7] = [
        ("cheese", DAIRY.clone()),
        ("tuna", HashSet::from(["tuna".to_string()])),
        ("ham", HashSet::from(["ham".to_string()])),
        ("egg", EGGS.clone()),
        ("hummus", HashSet::new()),
        ("avocado", HashSet::new()),
        ("roasted vegetables", HashSet::new()),
    ];
    let fillings: Vec<&str> = rows.iter().filter(|(_, needs)| !meets(needs, &banned)).map(|(n, _)| *n).collect();
    let three = fillings.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
    if banned.contains("gluten") {
        if !fillings.is_empty() {
            return Some(format!(
                "For a lunch that is not a sandwich: a rice or potato salad with {three}, fruit and a drink. Cook extra dinner and take the leftovers in a tub and it is quicker still. Keep it in a cool bag until lunch."
            ));
        }
        return Some("For a lunch that is not a sandwich: cook extra dinner and take the leftovers in a tub with rice, potatoes or salad, fruit and a drink. Keep it in a cool bag until lunch.".to_string());
    }
    if fillings.is_empty() {
        return Some("Cook extra dinner and take the leftovers in a tub with fruit and something crunchy: that is a packed lunch in five minutes. Keep it in a cool bag until lunch.".to_string());
    }
    let reply = format!(
        "A sandwich or wrap with {three}, fruit, something crunchy and a drink is a packed lunch in five minutes. Cook extra dinner and take the leftovers in a tub and it is quicker still."
    );
    // `banned` is never empty here
    Some(reply + " Keep it in a cool bag until lunch.")
}

// ---------------------------------------------------------------------------------------------
// Honey, freezing, nuts

static HONEY_BABY: LazyLock<String> = LazyLock::new(|| cstring("HONEY_BABY"));
static HONEY_OLDER: LazyLock<String> = LazyLock::new(|| cstring("HONEY_OLDER"));
static TODDLER_AGE: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(toddlers?|2 year old|3 year old|two year old|three year old)\b", 0));

/// The age in the line decides which side of the first birthday the answer opens on.
pub fn _honey_age(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    if TODDLER_AGE.is_match(&understood.clean) {
        return Some(HONEY_OLDER.clone());
    }
    Some(HONEY_BABY.clone())
}

static FREEZE_WORD_FORMS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_FREEZE_WORD_FORMS"));
static FREEZE_POINTS_AT: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_FREEZE_POINTS_AT"));
static FREEZE_FACTS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| cnames_rows("FREEZE_FACTS"));
static NOT_A_FREEZER: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_NOT_A_FREEZER"));
static FREEZE_ANY: LazyLock<String> = LazyLock::new(|| cstring("FREEZE_ANY"));

/// True when the line says "freeze it" rather than talking about freezing in general.
pub fn _points_at_something(words: &[String]) -> bool {
    let clean: Vec<String> =
        words.iter().map(|w| text::strip_chars(w, ".,!?").replace('\'', "").to_lowercase()).collect();
    let n = clean.len() as isize;
    for at in 0..clean.len() {
        for size in [3usize, 2, 1] {
            let end = (at + size).min(clean.len());
            let phrase = clean[at..end].join(" ");
            if !FREEZE_POINTS_AT.contains(&phrase) {
                continue;
            }
            for step in [-1isize, -2, size as isize, size as isize + 1] {
                let i = at as isize + step;
                if 0 <= i && i < n && FREEZE_WORD_FORMS.contains(&clean[i as usize]) {
                    return true;
                }
            }
        }
    }
    false
}

/// Every run of the line's words worth comparing with a food name, up to three words long.
pub fn _freeze_units(words: &[String]) -> Vec<String> {
    let mut units: Vec<String> = words.to_vec();
    for size in [2usize, 3] {
        if words.len() >= size {
            for i in 0..=(words.len() - size) {
                units.push(words[i..i + size].join(" "));
            }
        }
    }
    units
}

/// The most specific food freeze fact the words carry, or None.
pub fn _a_freeze_fact(words: &[String]) -> Option<String> {
    let units = _freeze_units(words);
    let mut best: Option<((bool, usize, usize), &String)> = None;
    for (names, fact) in FREEZE_FACTS.iter() {
        for name in names {
            for (at, word) in units.iter().enumerate() {
                if _same(word, name) {
                    let score = (word == name, text::split(name).len(), at);
                    if best.as_ref().is_none_or(|(b, _)| score > *b) {
                        best = Some((score, fact));
                    }
                }
            }
        }
    }
    best.map(|(_, f)| f.clone())
}

/// The one food name a line carries, or None when it carries two or more.
pub fn _names_one_food(words: &[String]) -> Option<String> {
    let units = _freeze_units(words);
    let mut seen: HashSet<&String> = HashSet::new();
    for (names, fact) in FREEZE_FACTS.iter() {
        for name in names {
            if units.iter().any(|unit| _same(unit, name)) {
                seen.insert(fact);
            }
        }
    }
    if seen.len() == 1 { seen.into_iter().next().cloned() } else { None }
}

/// The food the person's own last few lines are about, or None: the thing "it" points at.
pub fn _the_dish_the_chat_names(memory: &Memory) -> Option<String> {
    for turn in recent(memory, 4).iter().rev() {
        let said = &turn.user;
        if text::strip(said).is_empty() {
            continue;
        }
        if let Some(fact) = _names_one_food(&text::split(&said.to_lowercase())) {
            return Some(fact);
        }
    }
    None
}

/// The food the line names gets its own answer; a thing named earlier gets that one too.
pub fn _freeze_food(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if words.iter().any(|w| NOT_A_FREEZER.contains(text::strip_chars(w, ".,!?\"'"))) {
        return None;
    }
    if let Some(fact) = _a_freeze_fact(&words) {
        return Some(fact);
    }
    if !_points_at_something(&words) {
        return None;
    }
    if let Some(earlier) = _the_dish_the_chat_names(memory) {
        return Some(earlier);
    }
    Some(FREEZE_ANY.clone())
}

static SOYA: LazyLock<HashSet<String>> = LazyLock::new(|| cset("SOYA"));
static ALSO_OUT: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("ALSO_OUT"));

/// A nut allergy: food that is safe, and the cake question when that is what is cooking.
pub fn _nut_free(memory: &mut Memory, _understood: &Understood) -> Option<String> {
    let banned = avoids(memory, None);
    if meets(&GLUTEN, &banned) || meets(&EGGS, &banned) {
        return None;
    }
    let mut also: Vec<String> = Vec::new();
    if meets(&FAMILIES["shellfish"], &banned) {
        also.push(ALSO_OUT["shellfish"].clone());
    }
    if meets(&SOYA, &banned) {
        also.push(ALSO_OUT["soya"].clone());
    }
    let turns = &memory.turns;
    let recent: String =
        turns[turns.len().saturating_sub(2)..].iter().map(|t| t.user.clone()).collect::<Vec<_>>().join(" ").to_lowercase();
    if recent.contains("cake") || recent.contains("bake") || recent.contains("birthday") {
        return Some("Then go for a plain chocolate sponge: cocoa, flour, eggs, butter and sugar, no nuts. Check the chocolate, the icing sugar and any sprinkles for 'may contain nuts', and keep the tin and worktop clean if the allergy is serious.".to_string());
    }
    let tail = if also.is_empty() { String::new() } else { format!(" {}", also.join(" ")) };
    Some(format!(
        "Plenty: meat, fish, eggs, rice, pasta, potatoes and plain fruit and veg are all nut-free, so pasta with tomato sauce or roast chicken and potatoes works. Check the labels on chocolate, muesli, pesto and satay, and anything marked 'may contain nuts'.{tail}"
    ))
}

// ---------------------------------------------------------------------------------------------
// Vegan bakes

static V_PANCAKE: LazyLock<HashSet<String>> =
    LazyLock::new(|| folded_set(&["pancake", "pancakes", "crepe", "crepes", "waffle", "waffles", "drop scone"]));
static V_SCONE: LazyLock<HashSet<String>> = LazyLock::new(|| folded_set(&["scone", "scones", "drop scone", "drop scones"]));
static V_PASTRY: LazyLock<HashSet<String>> = LazyLock::new(|| {
    folded_set(&[
        "pastry", "shortcrust", "shortcrust pastry", "pie", "pie crust", "tart", "tarts", "tart case", "flan", "quiche",
        "quiche case",
    ])
});
static V_CRUMBLE: LazyLock<HashSet<String>> = LazyLock::new(|| folded_set(&["crumble", "crumbles", "fruit crumble", "cobbler"]));
static V_BISCUIT: LazyLock<HashSet<String>> =
    LazyLock::new(|| folded_set(&["biscuit", "biscuits", "cookie", "cookies", "shortbread"]));
static V_FLAPJACK: LazyLock<HashSet<String>> = LazyLock::new(|| {
    folded_set(&["flapjack", "flapjacks", "oat bar", "oat bars", "granola bar", "granola bars"])
});
static V_BREAD: LazyLock<HashSet<String>> = LazyLock::new(|| {
    folded_set(&[
        "bread", "loaf", "loaves", "roll", "rolls", "baguette", "focaccia", "pizza", "pizza dough", "flatbread", "naan",
    ])
});
static V_ENRICHED: LazyLock<HashSet<String>> = LazyLock::new(|| {
    folded_set(&[
        "banana", "carrot", "courgette", "zucchini", "apple", "date", "pumpkin", "lemon", "fruit", "soda", "cake",
        "brioche", "challah",
    ])
});
static V_BROWNIE: LazyLock<HashSet<String>> =
    LazyLock::new(|| folded_set(&["brownie", "brownies", "blondie", "blondies"]));

/// The vegan swap the bake needs: pancakes need no egg, a scone is a rubbing-in job, and so on.
pub fn _vegan_bake(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let folded: HashSet<String> = words_or_clean(understood).iter().map(|w| _fold(w)).collect();
    if meets(&folded, &V_PANCAKE) {
        return Some("Pancakes need no egg at all: 100g of plain flour, a teaspoon of baking powder, a pinch of salt and 300ml of unsweetened soya or oat milk, whisked and rested 20 minutes, then cooked in a little oil. The first one always fails, so make it the cook's. For thicker American ones, add a tablespoon of oil and a teaspoon of sugar.".to_string());
    }
    if meets(&folded, &V_SCONE) {
        return Some("Rub 50g of a hard vegan block margarine, or 40ml of neutral oil, into 225g of self-raising flour, then bring it together with about 120ml of unsweetened soya milk and a squeeze of lemon. Pat it out, cut the scones and bake at 220C for 12 to 15 minutes. The block margarine, not a tub spread, is what makes it short.".to_string());
    }
    if meets(&folded, &V_PASTRY) {
        return Some("Vegan shortcrust is the same weights with the fat swapped: 225g plain flour, a pinch of salt and 100g of a hard vegan block margarine (or 90ml of neutral oil), rubbed in until it looks like breadcrumbs, then cold water a spoon at a time until it just comes together. Rest it 30 minutes in the fridge, roll it thin on a floured board, and blind bake at 190C (170C fan) for 15 minutes for a tart or quiche case. Brush the top with plant milk rather than egg and it browns just as well.".to_string());
    }
    if meets(&folded, &V_CRUMBLE) {
        return Some("Swap the butter and nothing else changes: rub 100g of a hard vegan block margarine into 200g plain flour with 75g sugar until it looks like breadcrumbs, scatter it over the fruit and bake at 190C (170C fan) for 35 to 40 minutes, until the fruit bubbles at the edges and the top is golden.".to_string());
    }
    if meets(&folded, &V_BISCUIT) {
        return Some("Biscuits are the easiest bake to make vegan: a hard block margarine in place of the butter at the same weight, and a tablespoon of plant milk if the dough will not come together. No egg and no raising agent needed, so nothing else changes, and they keep a day or two less.".to_string());
    }
    if meets(&folded, &V_FLAPJACK) {
        return Some("A flapjack needs no egg, so the only swap is the butter: 150g of a hard vegan block margarine or 120ml of neutral oil with 120g of golden syrup is the sticky part. Melt them together, stir in 300g of porridge oats, press it into a lined 20cm tin and bake at 180C (160C fan) for 20 to 25 minutes. Let it go completely cold in the tin or it will crumble when you cut it.".to_string());
    }
    if meets(&folded, &V_BREAD) && !meets(&folded, &V_ENRICHED) {
        return Some("Ordinary bread is vegan already: flour, water, yeast and salt, with oil or a hard block margarine where the recipe wants a fat, and sugar feeds the yeast without leaving any behind. Only an enriched loaf needs a swap - oil for the butter, plant milk for the milk, and golden syrup for the honey. Check a bought loaf, though: milk powder and dried egg turn up in cheap sliced bread and in brioche.".to_string());
    }
    if meets(&folded, &V_BROWNIE) {
        return Some("For brownies, two flax eggs do the job: a tablespoon of ground flaxseed and three of water each, left five minutes to go gluey. Use an oil-based or vegan block margarine for the butter and keep the oven 10C lower, because brownies are meant to come out fudgy and the flax makes them catch sooner.".to_string());
    }
    Some("A cake takes three swaps: 60g of yoghurt or half a mashed banana, or a flax egg (a tablespoon of ground flaxseed and three of water, left five minutes), for each egg; unsweetened soya or oat milk for the milk; and a block margarine or 80g of neutral oil for every 100g of butter. Add an extra half teaspoon of baking powder, because the egg was doing some of the lifting, and give it five minutes longer in the oven. Soya milk browns best, and a sponge made this way keeps a day less.".to_string())
}

static VEGAN_ASK: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_VEGAN_ASK"));

/// True when the line asks for a bake with no dairy or no egg in it.
pub fn _asks_vegan(folded: &[String]) -> bool {
    let said = folded.join(" ");
    VEGAN_ASK.iter().any(|p| said.contains(p.as_str()))
}

// ---------------------------------------------------------------------------------------------
// Nutrients and calories

/// The plural a food word takes ("peach" -> "peaches", "berry" -> "berries").
pub fn _plurals(word: &str) -> String {
    if !text::is_alpha(word) || word.ends_with('s') {
        return word.to_string();
    }
    if ["ch", "sh", "ss", "x", "z"].iter().any(|e| word.ends_with(e)) {
        return format!("{word}es");
    }
    if ["potato", "tomato", "hero"].contains(&word) {
        return format!("{word}es");
    }
    if word.ends_with('y') {
        let chars: Vec<char> = word.chars().collect();
        // `word[-2:-1] not in "aeiou"`: an empty slice is "in" any string
        if chars.len() >= 2 && !"aeiou".contains(chars[chars.len() - 2]) {
            return format!("{}ies", &word[..word.len() - 1]);
        }
    }
    format!("{word}s")
}

static COUNT_WORDS: LazyLock<HashMap<String, f64>> = LazyLock::new(|| {
    cget("_COUNT_WORDS").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_f64()?))).collect()
});

/// How many of the food the words before it say: "a couple of", "two", "half a".
pub fn _portion_count(words: &[String], index: usize) -> f64 {
    let back = &words[index.saturating_sub(4)..index];
    let said = back.join(" ");
    if said.ends_with("a couple of") || said.ends_with("couple of") {
        return 2.0;
    }
    if said.ends_with("a few") {
        return 3.0;
    }
    if said.ends_with("half a") || said.ends_with("half an") {
        return 0.5;
    }
    for word in back.iter().rev() {
        if let Some(n) = COUNT_WORDS.get(word) {
            return *n;
        }
    }
    1.0
}

/// A word with its doubled letters squashed to one ("brocolli" and "broccoli" both "brocoli").
pub fn _squashed(word: &str) -> String {
    let mut out = String::new();
    let mut last: Option<char> = None;
    for ch in word.chars() {
        if Some(ch) != last || ch == '\n' {
            out.push(ch);
        }
        last = Some(ch);
    }
    out
}

/// The longer word is the shorter with one letter added somewhere.
pub fn _one_letter_apart(short: &str, long: &str) -> bool {
    let s: Vec<char> = short.chars().collect();
    let l: Vec<char> = long.chars().collect();
    let mut index = 0;
    while index < s.len() && s[index] == l[index] {
        index += 1;
    }
    s[index..] == l[(index + 1).min(l.len())..]
}

/// A food named with a slip in it, and how many: (the table's name, count), or None.
pub fn _the_food_named_by_a_slip(words: &[String], table: &[String]) -> Option<(String, f64)> {
    for (index, word) in words.iter().enumerate() {
        let Some(name) = _a_food_word_misspelt(word, table) else { continue };
        let before = if index > 0 { Some(&words[index - 1]) } else { None };
        let after = words.get(index + 1);
        if let Some(a) = after.filter(|a| !a.is_empty()) {
            if NUTRIENT_DISH.contains(a) || NUTRIENT_JOIN.contains(a) {
                return None;
            }
        }
        if let Some(b) = before.filter(|b| !b.is_empty()) {
            if NUTRIENT_JOIN.contains(b) {
                return None;
            }
        }
        return Some((name, _portion_count(words, index)));
    }
    None
}

/// The table name a word is a slip of (a doubled letter, one letter wrong, added or dropped).
pub fn _a_food_word_misspelt(word: &str, names: &[String]) -> Option<String> {
    if text::len(word) < 5 || !text::is_alpha(word) {
        return None;
    }
    if names.iter().any(|n| n == word) {
        return None;
    }
    let squashed = _squashed(word);
    let wchars: Vec<char> = word.chars().collect();
    for name in names {
        let nchars: Vec<char> = name.chars().collect();
        if name.contains(' ') || nchars.len() < 5 || nchars[0] != wchars[0] {
            continue;
        }
        if nchars.len() == wchars.len() {
            if squashed == _squashed(name) {
                return Some(name.clone());
            }
            if nchars.last() == wchars.last() && wchars.iter().zip(nchars.iter()).filter(|(a, b)| a != b).count() == 1 {
                return Some(name.clone());
            }
        } else if (nchars.len() as isize - wchars.len() as isize).abs() == 1 && nchars.last() == wchars.last() {
            let (shorter, longer) = if wchars.len() <= nchars.len() { (word, name.as_str()) } else { (name.as_str(), word) };
            if _one_letter_apart(shorter, longer) {
                return Some(name.clone());
            }
        }
    }
    None
}

/// A food table's names in order, each with its `(?<![a-z0-9])name s?(?![a-z0-9])` pattern.
pub struct FoodTable {
    pub names: Vec<String>,
    patterns: Vec<Re>,
}

impl FoodTable {
    fn of(name: &str) -> FoodTable {
        let names: Vec<String> = cget(name).pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect();
        let patterns = names.iter().map(|f| Re::new(&format!(r"(?<![a-z0-9]){}s?(?![a-z0-9])", escape(f)), 0)).collect();
        FoodTable { names, patterns }
    }
    fn matches(&self, clean: &str) -> Vec<String> {
        self.names.iter().zip(self.patterns.iter()).filter(|(_, p)| p.is_match(clean)).map(|(n, _)| n.clone()).collect()
    }
}

/// One FOOD_NUTRIENTS row: what it is, the portion, and its numbers per portion.
pub struct NutrientRow {
    pub subject: String,
    pub portion: String,
    pub numbers: IndexMap<String, f64>,
}

static FOOD_NUTRIENTS: LazyLock<IndexMap<String, NutrientRow>> = LazyLock::new(|| {
    cget("FOOD_NUTRIENTS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let row = v.items();
            Some((
                k.as_str()?.to_string(),
                NutrientRow {
                    subject: row[0].str().to_string(),
                    portion: row[1].str().to_string(),
                    numbers: row[2]
                        .pairs()
                        .iter()
                        .filter_map(|(c, n)| Some((c.as_str()?.to_string(), n.as_f64()?)))
                        .collect(),
                },
            ))
        })
        .collect()
});
static NUTRIENT_TABLE: LazyLock<FoodTable> = LazyLock::new(|| FoodTable::of("FOOD_NUTRIENTS"));
static NUTRIENT_DISH: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_NUTRIENT_DISH"));
static NUTRIENT_JOIN: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_NUTRIENT_JOIN"));
static NUTRIENT_WORDS: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| {
    cpairs("_NUTRIENT_WORDS")
        .into_iter()
        .map(|(w, col)| (Re::new(&format!(r"(?<![a-z0-9]){}(?![a-z0-9])", escape(&w)), 0), col))
        .collect()
});
static NUTRIENT_COLUMNS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    cget("NUTRIENT_COLUMNS").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.items()[1].str().to_string()))).collect()
});
static NUTRIENT_CONTEXT: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("NUTRIENT_CONTEXT"));
static PROTEIN_SHARES: LazyLock<Vec<(f64, String)>> = LazyLock::new(|| {
    cget("_PROTEIN_SHARES").items().iter().map(|r| (r.items()[0].as_f64().unwrap_or(0.0), r.items()[1].str().to_string())).collect()
});
static WEIGHT_WORDS: LazyLock<Vec<(Re, f64, String)>> = LazyLock::new(|| {
    cget("_WEIGHT_WORDS")
        .items()
        .iter()
        .map(|r| {
            let it = r.items();
            (
                Re::new(&format!(r"(\d+(?:\.\d+)?)\s*(?:{})\b", it[0].str()), 0),
                it[1].as_f64().unwrap_or(0.0),
                it[2].str().to_string(),
            )
        })
        .collect()
});
static AMOUNT_ASKED: LazyLock<&'static Re> = LazyLock::new(|| cre("_AMOUNT_ASKED"));
static CALORIE_PLURAL: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CALORIE_PLURAL"));
static PORTION_SIZE: LazyLock<Re> = LazyLock::new(|| Re::new(r"(\d+(?:\.\d+)?)\s*(g|ml)\b", 0));
static PINTS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(half a pint|a pint|one pint)\b", 0));

/// The table food a line names, and how many: the longest name, unless the line names two
/// foods or the food is part of a dish ("chicken soup").
pub fn _nutrient_food(clean: &str, table: &FoodTable) -> Option<(String, f64)> {
    let matches = table.matches(clean);
    let words = WORDS_AZ09.findall_str(clean);
    if matches.is_empty() {
        return _the_food_named_by_a_slip(&words, &table.names);
    }
    let best = first_longest(&matches);
    if matches.iter().any(|f| {
        !best.contains(f.as_str())
            && !best.contains(f.trim_end_matches('s'))
            && !best.contains(_plurals(f).as_str())
    }) {
        return None;
    }
    let parts = text::split(&best);
    if words.len() < parts.len() {
        return None;
    }
    for index in 0..=(words.len() - parts.len()) {
        let chunk = &words[index..index + parts.len()];
        if chunk[..chunk.len() - 1] != parts[..parts.len() - 1] {
            continue;
        }
        let last = &parts[parts.len() - 1];
        if chunk[chunk.len() - 1] != *last && chunk[chunk.len() - 1] != _plurals(last) {
            continue;
        }
        let before = if index > 0 { Some(&words[index - 1]) } else { None };
        let after = words.get(index + parts.len());
        if let Some(a) = after.filter(|a| !a.is_empty()) {
            if NUTRIENT_DISH.contains(a) || NUTRIENT_JOIN.contains(a) {
                return None;
            }
        }
        if let Some(b) = before.filter(|b| !b.is_empty()) {
            if NUTRIENT_JOIN.contains(b) {
                return None;
            }
        }
        return Some((best, _portion_count(&words, index)));
    }
    None
}

/// The nutrient the line asks about (a column of the table), or None.
pub fn _nutrient_asked(clean: &str) -> Option<String> {
    NUTRIENT_WORDS.iter().find(|(rx, _)| rx.is_match(clean)).map(|(_, col)| col.clone())
}

/// A weight in grams as it is said ("under a gram", "12", "2.5").
pub fn _grams(value: f64) -> Option<String> {
    if value <= 0.0 {
        return None;
    }
    if value < 1.0 {
        return Some("under a gram".to_string());
    }
    if value == value.trunc() {
        return Some(format!("{}", value as i64));
    }
    Some(strip_point(&format!("{value:.1}")))
}

fn strip_point(s: &str) -> String {
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The size a portion is given in, as (amount, "g" or "ml"), or None.
pub fn _portion_size(portion: &str) -> Option<(f64, String)> {
    let low = portion.to_lowercase();
    let m = PORTION_SIZE.search(&low)?;
    Some((m.str(1).parse().unwrap_or(0.0), m.str(2).to_string()))
}

/// How many portions a weight said in the line is, and the words that said it.
pub fn _weight_in_the_line(clean: &str, portion: &str) -> Option<(f64, String)> {
    let base = _portion_size(portion)?;
    for (rx, scale, unit) in WEIGHT_WORDS.iter() {
        if let Some(m) = rx.search(clean) {
            if base.1 == *unit {
                let n: f64 = m.str(1).parse().unwrap_or(0.0);
                return Some((n * scale / base.0, m.as_str().to_string()));
            }
        }
    }
    if base.1 == "ml" {
        if let Some(m) = PINTS.search(clean) {
            let many = if m.str(1).starts_with("half") { 0.5 } else { 1.0 };
            return Some((many * 568.0 / base.0, m.str(1).to_string()));
        }
    }
    None
}

/// A number as it is said: whole from ten up, one decimal below.
pub fn _figure(value: f64) -> String {
    if value >= 10.0 {
        return format!("{}", text::round_half_even(value));
    }
    strip_point(&format!("{value:.1}"))
}

/// How much of a nutrient a food has: "An egg has about 6g of protein."
pub fn _nutrients(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if clean.is_empty() {
        return None;
    }
    let mut plain = understood.plain.to_lowercase();
    if plain == clean {
        plain = String::new();
    }
    let mut column = _nutrient_asked(&clean);
    let mut read = clean.clone();
    if column.is_none() && !plain.is_empty() {
        column = _nutrient_asked(&plain);
        read = if column.is_some() { plain.clone() } else { clean.clone() };
    }
    let column = column?;
    let mut found = _nutrient_food(&read, &NUTRIENT_TABLE);
    if found.is_none() && !plain.is_empty() {
        found = _nutrient_food(if read == clean { &plain } else { &clean }, &NUTRIENT_TABLE);
    }
    let (food, count) = found?;
    let row = &FOOD_NUTRIENTS[&food];
    let mut subject = row.subject.clone();
    let mut portion = row.portion.clone();
    let per = *row.numbers.get(&column)?;
    let mut value = per * count;
    let mut scaled: Option<f64> = None;
    let mut said_amount = String::new();
    if count == 1.0 {
        if let Some((s, amount)) = _weight_in_the_line(&clean, &portion) {
            scaled = if (s - 1.0).abs() < 0.01 { None } else { Some(s) };
            said_amount = amount;
        } else if _portion_size(&portion).is_none() {
            for word in AMOUNT_ASKED.findall_str(&clean) {
                if word.is_empty() {
                    return None;
                }
                let lp = portion.to_lowercase();
                if lp.contains(&word) || lp.contains(&_plurals(&word)) {
                    continue;
                }
                let ls = subject.to_lowercase();
                if ls.contains(&word) || ls.contains(&_plurals(&word)) {
                    continue;
                }
                return None;
            }
        }
    }
    let mut said = upper_first(&subject);
    if count > 1.0 {
        let mut pieces = text::split(&said);
        if pieces.first().is_some_and(|p| p == "A" || p == "An") {
            pieces.remove(0);
        }
        let head: isize = match pieces.iter().position(|p| p == "of") {
            Some(i) => i as isize - 1,
            None => pieces.len() as isize - 1,
        };
        let head = if head < 0 { (pieces.len() as isize + head) as usize } else { head as usize };
        pieces[head] = _plurals(&pieces[head]);
        let lead = if count != count.trunc() {
            format!("{} ", text::float_repr(count))
        } else {
            let n = count as i64;
            match n {
                2 => "Two".to_string(),
                3 => "Three".to_string(),
                4 => "Four".to_string(),
                5 => "Five".to_string(),
                6 => "Six".to_string(),
                7 => "Seven".to_string(),
                8 => "Eight".to_string(),
                9 => "Nine".to_string(),
                10 => "Ten".to_string(),
                _ => n.to_string(),
            }
        };
        said = format!("{lead} {}", pieces.join(" "));
        subject = said.clone();
        portion = String::new();
    }
    let name = NUTRIENT_COLUMNS[&column].clone();
    let unit = if ["iron", "calcium", "potassium"].contains(&column.as_str()) { "mg" } else { "g" };
    let head2: String = subject.chars().take(2).collect();
    let plural = count > 1.0 || CALORIE_PLURAL.contains(&food) || !["a ", "an", "A ", "An"].contains(&head2.as_str());
    let scaled_on = scaled.is_some_and(|s| s != 0.0);
    if let Some(s) = scaled.filter(|s| *s != 0.0) {
        said = upper_first(&said_amount);
        if !said.to_lowercase().contains(food.as_str()) {
            said.push_str(&format!(" of {food}"));
        }
        value *= s;
    }
    let verb = if plural { "have" } else { "has" };
    let mut sentence = if value != 0.0 {
        format!("{said} {verb} about {}{unit} of {name}", _figure(value))
    } else {
        format!("{said} {verb} no {name} to speak of")
    };
    if !portion.is_empty() && portion != "one" && !scaled_on && !said.to_lowercase().contains(&portion.to_lowercase()) {
        sentence.push_str(&format!(" ({portion})"));
    }
    sentence.push('.');
    if column == "protein" && value != 0.0 {
        if value >= 50.0 {
            sentence.push_str(" That is more protein than an adult needs in a day.");
        } else {
            for (share, word) in PROTEIN_SHARES.iter() {
                if value / 50.0 >= *share {
                    let hedge = if word == "most" { "" } else { "about " };
                    sentence.push_str(&format!(" That is {hedge}{word} of a day's protein for an adult."));
                    break;
                }
            }
        }
    }
    if let Some(ctx) = NUTRIENT_CONTEXT.get(&column) {
        sentence.push(' ');
        sentence.push_str(ctx);
    }
    Some(sentence)
}

static CALORIE_TABLES: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
    ["FOOD_CALORIES", "FOOD_CALORIE_MEALS", "FOOD_NUTRIENTS"]
        .iter()
        .map(|t| cget(t).pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect())
        .collect()
});

/// Whether the line names any food the calorie or nutrient tables know.
pub fn _names_a_food(line: &str) -> bool {
    let lowered = line.to_lowercase();
    CALORIE_TABLES.iter().any(|table| table.iter().any(|name| !name.is_empty() && lowered.contains(name.as_str())))
}

/// One calorie row: what it is, the portion, and the calories (a number, printed as Python would).
struct CalorieRow {
    subject: String,
    portion: String,
    count: String,
}

fn calorie_rows(name: &str) -> IndexMap<String, CalorieRow> {
    cget(name)
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let it = v.items();
            Some((
                k.as_str()?.to_string(),
                CalorieRow { subject: it[0].str().to_string(), portion: it[1].str().to_string(), count: py_str(&it[2]) },
            ))
        })
        .collect()
}

static CALORIE_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("CALORIE_ASKS"));
static CALORIE_OWN: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CALORIE_OWN"));
static CALORIE_NOT_A_DISH: LazyLock<Vec<Re>> = LazyLock::new(|| {
    cstrs("_CALORIE_NOT_A_DISH").iter().map(|w| Re::new(&format!(r"(?<![a-z0-9]){}(?![a-z0-9])", escape(w)), 0)).collect()
});
static CALORIE_DISH: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CALORIE_DISH"));
static CALORIE_JOIN: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CALORIE_JOIN"));
static FOOD_CALORIES: LazyLock<IndexMap<String, CalorieRow>> = LazyLock::new(|| calorie_rows("FOOD_CALORIES"));
static FOOD_CALORIE_MEALS: LazyLock<IndexMap<String, CalorieRow>> = LazyLock::new(|| calorie_rows("FOOD_CALORIE_MEALS"));
static CALORIE_TABLE: LazyLock<FoodTable> = LazyLock::new(|| FoodTable::of("FOOD_CALORIES"));
/// The meal names longest first (a stable sort, so equal lengths keep the table's order).
static MEALS_BY_LENGTH: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    let mut names: Vec<String> = FOOD_CALORIE_MEALS.keys().cloned().collect();
    names.sort_by_key(|n| std::cmp::Reverse(text::len(n)));
    names.into_iter().map(|n| { let rx = Re::new(&format!(r"(?<![a-z0-9]){}(?![a-z0-9])", escape(&n)), 0); (n, rx) }).collect()
});

/// About how many calories a food or a meal has, from the table.
pub fn _calories(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut clean = understood.clean.clone();
    let plain = &understood.plain;
    if !plain.is_empty() && *plain != clean && !_names_a_food(&clean) {
        clean = plain.clone();
    }
    if !CALORIE_ASKS.iter().any(|w| clean.contains(w.as_str())) {
        return None;
    }
    if CALORIE_OWN.iter().any(|f| clean.contains(f.as_str())) {
        return None;
    }
    if CALORIE_NOT_A_DISH.iter().any(|rx| rx.is_match(&clean)) {
        return None;
    }
    let words = WORDS_AZ09.findall_str(&clean);
    let part_of_a_dish = |name: &str| -> bool {
        let tail = text::split(name).last().cloned().unwrap_or_default();
        for (i, word) in words.iter().enumerate() {
            if *word == tail {
                return words.get(i + 1).is_some_and(|after| CALORIE_DISH.contains(after));
            }
        }
        false
    };
    let mut meal: Option<&String> = None;
    for (dish, rx) in MEALS_BY_LENGTH.iter() {
        if part_of_a_dish(dish) {
            continue;
        }
        if rx.is_match(&clean) {
            meal = Some(dish);
            break;
        }
    }
    if let Some(m) = meal {
        let row = &FOOD_CALORIE_MEALS[m];
        return Some(format!("{} is about {} calories ({}).", upper_first(&row.subject), row.count, row.portion));
    }
    let mut matches = CALORIE_TABLE.matches(&clean);
    if matches.is_empty() {
        let slipped = _the_food_named_by_a_slip(&words, &CALORIE_TABLE.names)?;
        matches = vec![slipped.0];
    }
    let best = first_longest(&matches);
    if matches.iter().any(|f| !best.contains(f.as_str())) {
        return None;
    }
    let parts = text::split(&best);
    let tail = parts.last().cloned().unwrap_or_default();
    let (mut before, mut after): (Option<&String>, Option<&String>) = (None, None);
    for (i, word) in words.iter().enumerate() {
        if *word == tail {
            after = words.get(i + 1);
            let back = i as isize - parts.len() as isize + 1;
            before = if back >= 1 { Some(&words[(back - 1) as usize]) } else { None };
        }
    }
    if let Some(a) = after.filter(|a| !a.is_empty()) {
        if CALORIE_DISH.contains(a) || CALORIE_JOIN.contains(a) {
            return None;
        }
    }
    if let Some(b) = before.filter(|b| !b.is_empty()) {
        if CALORIE_JOIN.contains(b) {
            return None;
        }
    }
    let row = &FOOD_CALORIES[&best];
    let head2: String = row.subject.chars().take(2).collect();
    let verb = if CALORIE_PLURAL.contains(&best) && head2 != "a " && head2 != "an" { "are" } else { "is" };
    Some(format!("{} {verb} about {} calories ({}).", upper_first(&row.subject), row.count, row.portion))
}

// ---------------------------------------------------------------------------------------------
// Gluten

static GLUTEN_MAKING: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_GLUTEN_MAKING"));
static NOT_A_Z09: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9 ]+", 0));
static IS_ARE_OPENING: LazyLock<Re> = LazyLock::new(|| Re::new(r"(is|are|does|do|can|should)\b", 0));

fn spaced(folded: &str, words: &[&str]) -> bool {
    words.iter().any(|w| folded.contains(&format!(" {w} ")))
}

/// How to make (or whether to buy) the gluten-free version of a bake.
pub fn _gluten_free(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = &understood.clean;
    let folded = format!(" {} ", NOT_A_Z09.sub(" ", &clean.to_lowercase()));
    if IS_ARE_OPENING.match_(clean).is_some() && !GLUTEN_MAKING.iter().any(|w| folded.contains(&format!(" {w} "))) {
        return Some("Not the ordinary one: wheat flour is what bread, pasta, cakes and most biscuits are made from, so unless the packet says gluten free it is not. A gluten-free loaf is a different product rather than the same loaf without something in it, and check the label for 'may contain wheat' on sauces, stock and chips.".to_string());
    }
    if spaced(&folded, &["bread", "loaf", "loaves", "roll", "rolls", "baguette", "pizza", "naan", "focaccia"]) {
        return Some("Buy this one rather than bake it: a gluten-free loaf needs a batter, not a dough, plus psyllium or xanthan to stand in for the stretch, and it is the one thing a shop does better. For a bun or a pizza base, a gluten-free bread flour blend with a teaspoon of xanthan per 500g and a wet dough, proved in the tin, gets there.".to_string());
    }
    if spaced(&folded, &["pasta", "spaghetti", "penne", "macaroni", "lasagne", "noodles"]) {
        return Some("Buy it: dried pasta made from corn, rice or buckwheat is the one gluten-free swap that works, and the fresh gluten-free pasta in the chiller is better still. Cook it in plenty of salted water and taste a minute before the packet says, because it goes from firm to mushy quickly, and dress it straight away - it does not wait like wheat pasta does.".to_string());
    }
    if spaced(&folded, &["pastry", "shortcrust", "tart", "pie", "quiche", "flan", "crust"]) {
        return Some("Gluten-free shortcrust: 200g of a gluten-free plain flour blend with half a teaspoon of xanthan gum, 100g of cold butter rubbed in, then a beaten egg or a few spoons of iced water to bring it together. Rest it 30 minutes in the fridge and roll it between two sheets of baking paper, because it tears. It needs no blind-bake change: 15 minutes at 190C (170C fan).".to_string());
    }
    let mut said = "A gluten-free bake needs a blend, not one flour: a shop gluten-free plain flour blend with a quarter teaspoon of xanthan gum per 200g, and a splash more liquid than the recipe says, because there is no stretch in it to hold water. Let the batter stand 20 minutes before it goes in the oven and it stops tasting gritty. Expect it to brown faster, so drop the oven 10C and give it five minutes longer.".to_string();
    if spaced(&folded, &["dairy", "milk", "butter", "cheese", "yoghurt", "yogurt", "vegan", "plant"]) {
        said.push_str(" For the dairy, unsweetened soya or oat milk goes in one for one, and a hard dairy-free block margarine replaces the butter at the same weight.");
    }
    Some(said)
}

static GLUTEN_FACT_OPENING: LazyLock<&'static Re> = LazyLock::new(|| cre("GLUTEN_FACT_OPENING"));
static GLUTEN_FACT_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("GLUTEN_FACT_ASKS"));
static GLUTEN_FACT_MAKING: LazyLock<&'static Re> = LazyLock::new(|| cre("GLUTEN_FACT_MAKING"));
static GLUTEN_FACT_NOT_PLAIN: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("GLUTEN_FACT_NOT_PLAIN"));
static GLUTEN_FACT_NAMED_SWAP: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("GLUTEN_FACT_NAMED_SWAP"));
static GLUTEN_FACT_SWAPS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("GLUTEN_FACT_SWAPS"));
static NOT_SETTLED_BY_THE_RECIPE: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("NOT_SETTLED_BY_THE_RECIPE"));
static MADE_OF: LazyLock<HashMap<String, HashSet<String>>> = LazyLock::new(|| {
    cget("MADE_OF").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.str_set()))).collect()
});
/// `FAMILY_SAY[id(GLUTEN)]`: the table is keyed by the family sets' ids, gluten's second.
static GLUTEN_SAY: LazyLock<(String, String)> = LazyLock::new(|| {
    let row = &cget("FAMILY_SAY").pairs()[1].1;
    (row.items()[0].str().to_string(), row.items()[1].str().to_string())
});

/// Whether the line asks a yes/no gluten fact about a named food ("is rice gluten free").
pub fn asks_a_gluten_fact(understood: &Understood) -> bool {
    let clean = &understood.clean;
    if GLUTEN_FACT_OPENING.match_(clean).is_none() {
        return false;
    }
    let lowered = clean.to_lowercase();
    if !GLUTEN_FACT_ASKS.iter().any(|a| lowered.contains(a.as_str())) {
        return false;
    }
    let Some(food) = _food_named(clean) else { return false };
    !GLUTEN_FACT_MAKING.is_match(&lowered.replace(food.as_str(), " "))
}

/// Whether a named food has gluten in it, and the swap when it does.
pub fn _free_of_gluten_fact(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = &understood.clean;
    if GLUTEN_FACT_OPENING.match_(clean).is_none() {
        return Some(String::new());
    }
    let Some(food) = _food_named(clean) else { return Some(String::new()) };
    if GLUTEN_FACT_MAKING.is_match(&clean.to_lowercase().replace(food.as_str(), " ")) {
        return Some(String::new());
    }
    if let Some(said) = GLUTEN_FACT_NOT_PLAIN.get(&food) {
        return Some(said.clone());
    }
    let phrase: Vec<String> = text::split(&food).iter().map(|p| escape(p)).collect();
    let free_of = Re::new(&format!(r"\b(?:[a-z]+)\s+free\s+{}\b", phrase.join(r"\s+")), 0);
    if free_of.is_match(&clean.to_lowercase()) {
        return Some(String::new());
    }
    let named = _named_as(clean, &food);
    let plural = _is_plural(&food);
    if meets(&GLUTEN, &MADE_OF[&food]) {
        let said = if plural { "are made with wheat flour" } else { "is made with wheat flour" };
        let reply = format!("No: {named} {said}, so it is not gluten free.");
        if let Some(swap) = GLUTEN_FACT_NAMED_SWAP.get(&food) {
            return Some(format!("{reply} {swap}"));
        }
        if GLUTEN_FACT_SWAPS.contains(&food) {
            return Some(format!("{reply} {}.", upper_first(&GLUTEN_SAY.1)));
        }
        return Some(format!("{reply} The gluten-free one is a different product, so check the label."));
    }
    if let Some(said) = NOT_SETTLED_BY_THE_RECIPE.get(&food) {
        return Some(said.clone());
    }
    if plural {
        return Some(format!(
            "Yes: {named} have no gluten in them. The plain ones are gluten free; check the packet on flavoured or own-brand versions."
        ));
    }
    Some(format!(
        "Yes: {named} has no gluten in it. The plain one is gluten free; check the packet on flavoured or own-brand versions."
    ))
}

// ---------------------------------------------------------------------------------------------
// A kitchen without an oven, a hob or a microwave

static LACKS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    cget("LACKS").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].strs())).collect()
});
static LACK_SAID: LazyLock<Vec<(String, String)>> = LazyLock::new(|| cpairs("LACK_SAID"));
static LACK_ATTACHED: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("LACK_ATTACHED"));

/// Whether a phrase saying a thing is missing is in the line, not followed by a part of it
/// ("no oven thermometer" is not a missing oven).
pub fn _says_gone(phrases: &[String], segments: &[String]) -> bool {
    for segment in segments {
        for phrase in phrases {
            if phrase.is_empty() {
                continue;
            }
            for (at, _) in segment.match_indices(phrase.as_str()) {
                let rest = text::strip(&segment[at + phrase.len()..]);
                let first = rest.split(' ').next().unwrap_or("");
                let after = text::strip_chars(first, ".,!?;:");
                if !LACK_ATTACHED.iter().any(|a| a == after) {
                    return true;
                }
            }
        }
    }
    false
}

/// The line and its normalised form, lower-case.
pub fn _line(understood: &Understood) -> String {
    format!("{} {}", understood.clean, understood.norm_text).to_lowercase()
}

/// The kitchen things the line says are missing: oven, hob, microwave, kettle.
pub fn missing(understood: &Understood) -> HashSet<String> {
    let text = _line(understood);
    let segments: Vec<String> = text.split('|').map(str::to_string).collect();
    let mut gone = HashSet::new();
    for (name, phrases) in LACKS.iter() {
        if _says_gone(phrases, &segments) {
            gone.insert(name.clone());
        }
    }
    for (name, phrase) in LACK_SAID.iter() {
        if !gone.contains(name) && _says_gone(std::slice::from_ref(phrase), &segments) {
            gone.insert(name.clone());
        }
    }
    gone
}

static SENTENCE_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?;:])\s+|\n+", 0));

/// The sentences of a text.
pub fn _sentences(text: &str) -> Vec<String> {
    SENTENCE_SPLIT.split(text).into_iter().filter(|s| !text::strip(s).is_empty()).collect()
}

static EMPTY_ASK_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("EMPTY_ASK_PHRASES"));
static REQUIRES: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| cstrs_map("REQUIRES"));
static REQUIRES_RE: LazyLock<HashMap<String, Re>> = LazyLock::new(|| {
    REQUIRES.values().flatten().map(|w| (w.clone(), Re::new(&format!(r"\b{}\b", escape(w)), 0))).collect()
});
static TURNS_DOWN: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("TURNS_DOWN"));
static DECIDING: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("DECIDING"));

/// Whether an answer needs a kitchen thing the line says is missing.
pub fn unusable(text: &str, understood: &Understood) -> bool {
    let gone = missing(understood);
    if gone.is_empty() || text::strip(text).is_empty() {
        return false;
    }
    let clean = understood.clean.to_lowercase();
    if EMPTY_ASK_PHRASES.iter().any(|p| clean.contains(p.as_str())) {
        return false;
    }
    let mut words: HashSet<&String> = HashSet::new();
    for name in &gone {
        if let Some(needs) = REQUIRES.get(name) {
            words.extend(needs.iter());
        }
    }
    for sentence in _sentences(&text.to_lowercase()) {
        for word in &words {
            if !REQUIRES_RE[*word].is_match(&sentence) {
                continue;
            }
            let split = text::split(&sentence);
            if TURNS_DOWN.iter().any(|one| split.contains(one)) {
                continue;
            }
            if DECIDING.iter().any(|one| sentence.contains(one.as_str())) {
                continue;
            }
            return true;
        }
    }
    false
}

static GRIDDLE_BATTER: LazyLock<&'static Re> = LazyLock::new(|| cre("GRIDDLE_BATTER"));
static GRIDDLE_DOUGH: LazyLock<&'static Re> = LazyLock::new(|| cre("GRIDDLE_DOUGH"));
static GRIDDLE_TAIL: LazyLock<String> = LazyLock::new(|| cstring("_GRIDDLE_TAIL"));
static WARM_BREAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:wrap|wraps|tortilla|tortillas|pitta|pittas|pita|pitas)\b", 0));

/// A bake that is really a pan job: pancakes, scones on a griddle, a wrap warmed through.
pub fn _griddle_answer(said: &str) -> String {
    let text = said.to_lowercase();
    if let Some(batter) = GRIDDLE_BATTER.search(&text) {
        return format!(
            "A {} is a pan job as it is: the batter goes into a hot non-stick pan with a little butter, about a minute a side until bubbles pop and the edges set. Crumpets want rings in a dry pan on a low heat until the top sets and the holes stay open. {}",
            batter.as_str().trim_end_matches('s'),
            *GRIDDLE_TAIL
        );
    }
    if let Some(dough) = GRIDDLE_DOUGH.search(&text) {
        let name = dough.as_str().trim_end_matches('s');
        if !["scone", "drop scone", "pikelet"].contains(&name) {
            return format!(
                "A {name} cooks in a heavy dry pan, not a tin: the same dough made a little thicker and patted out, about 5 minutes a side on a low heat with a lid on, until the middle is set and the edges have caught. Scones, pikelets and roti are the same trick. {}",
                *GRIDDLE_TAIL
            );
        }
        return format!(
            "A scone cooks in a heavy pan: the same dough made a little thicker and patted out, in a dry pan on a low heat with a lid on for about 5 minutes a side, until the middle is set and the edges have caught. Drop scones, pikelets, chapatis and roti are the same trick on a hot dry pan. {}",
            *GRIDDLE_TAIL
        );
    }
    let named = WARM_BREAD.search(&text).map(|m| m.as_str().trim_end_matches('s').to_string()).unwrap_or_else(|| "wrap".to_string());
    let other = if named.starts_with("pitt") { "and a wrap warms the same way" } else { "and a pitta puffs the same way" };
    format!(
        "A {named} is warmed rather than baked: 30 seconds a side in a dry hot pan and it is soft enough to fold, {other}. {}",
        *GRIDDLE_TAIL
    )
}

static SWEET_ASK: LazyLock<&'static Re> = LazyLock::new(|| cre("SWEET_ASK"));
static GRIDDLE_BAKES: LazyLock<&'static Re> = LazyLock::new(|| cre("GRIDDLE_BAKES"));
static BREAD: LazyLock<&'static Re> = LazyLock::new(|| cre("BREAD"));
static CAKEISH: LazyLock<&'static Re> = LazyLock::new(|| cre("CAKEISH"));
static NO_OVEN_BREAD: LazyLock<String> = LazyLock::new(|| cstring("NO_OVEN_BREAD"));
static NO_BAKING: LazyLock<String> = LazyLock::new(|| cstring("NO_BAKING"));
static CROWD: LazyLock<&'static Re> = LazyLock::new(|| cre("CROWD"));
static PUDDING_IDEAS: LazyLock<String> = LazyLock::new(|| cstring("PUDDING_IDEAS"));

/// Whether the line asks for something sweet (not a sweet potato).
pub fn _asks_for_sweet(text: &str) -> bool {
    let text = text.to_lowercase();
    if text.contains("sweet potato") {
        return false;
    }
    SWEET_ASK.is_match(&text)
}

/// What to make sweet with no oven: a no-bake pudding, or a pan bread.
pub fn _no_baking(text: &str) -> String {
    let said = text;
    if GRIDDLE_BAKES.is_match(said) {
        return _griddle_answer(said);
    }
    let base = if BREAD.is_match(said) && !CAKEISH.is_match(said) {
        NO_OVEN_BREAD.clone()
    } else {
        format!("{}{}", *NO_BAKING, if BREAD.is_match(said) { format!(" {}", *NO_OVEN_BREAD) } else { String::new() })
    };
    match CROWD.search(said) {
        None => base,
        Some(many) => format!(
            "{base} For {} people make two tins of it and cut it small, or set one big cheesecake in a deep tin and chill it overnight.",
            many.as_str()
        ),
    }
}

/// Pudding ideas, doubled for a crowd.
pub fn _pudding_ideas(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.to_lowercase();
    match CROWD.search(&text) {
        None => Some(PUDDING_IDEAS.clone()),
        Some(many) => Some(format!(
            "{} For {} people make two dishes of it and put the custard out in a jug.",
            *PUDDING_IDEAS,
            many.as_str()
        )),
    }
}

static COOKING_ASKS: LazyLock<Vec<Re>> =
    LazyLock::new(|| cstrs("COOKING_ASKS").iter().map(|w| Re::new(&format!(r"\b{}\b", escape(w)), 0)).collect());
static LISTING: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("LISTING"));
static HAVE_ONE: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| cstrs_map("HAVE_ONE"));
static ONLY_ONE: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| cstrs_map("ONLY_ONE"));
static NO_OVEN_ONLY: LazyLock<String> = LazyLock::new(|| cstring("NO_OVEN_ONLY"));
static NO_HOB: LazyLock<String> = LazyLock::new(|| cstring("NO_HOB"));
static NO_KETTLE: LazyLock<String> = LazyLock::new(|| cstring("NO_KETTLE"));
static NO_OVEN: LazyLock<String> = LazyLock::new(|| cstring("NO_OVEN"));
static NO_MICROWAVE: LazyLock<String> = LazyLock::new(|| cstring("NO_MICROWAVE"));
const BAKING: [&str; 24] = [
    "bake", "baking", "cake", "cakes", "bread", "pie", "pastry", "pastries", "biscuit", "biscuits", "cookie", "cookies",
    "meringue", "meringues", "yorkshire", "scone", "scones", "loaf", "loaves", "muffin", "muffins", "brownies",
    "flapjack", "flapjacks",
];

/// What to cook with the kitchen thing the line says is missing, or None.
pub fn without(understood: &Understood) -> Option<String> {
    let gone = missing(understood);
    if gone.is_empty() {
        return None;
    }
    let text = understood.words.join(" ").to_lowercase();
    let asking = COOKING_ASKS.iter().any(|rx| rx.is_match(&text));
    let listing = LISTING.iter().any(|one| text.contains(one.as_str()));
    if !(asking || listing) {
        let named = ["hob", "microwave", "kettle", "oven"].iter().any(|k| HAVE_ONE[*k].iter().any(|o| text.contains(o.as_str())))
            || ["hob", "microwave", "kettle"].iter().any(|k| ONLY_ONE[*k].iter().any(|o| text.contains(o.as_str())));
        if !(named && text::split(&text).len() <= 8) {
            return None;
        }
    }
    let how = if BAKING.iter().any(|one| text.contains(one)) || _asks_for_sweet(&text) { "bake" } else { "" };
    if gone.contains("oven") && how == "bake" && !gone.contains("hob") {
        return Some(_no_baking(&text));
    }
    if gone.contains("oven") && !gone.contains("hob") && GRIDDLE_BAKES.is_match(&text) {
        return Some(_griddle_answer(&text));
    }
    let have = available(&text, &gone);
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

/// The kitchen things there are to cook with: the only one the line names, the ones it says
/// it has, or all four less the missing ones.
pub fn available(text: &str, gone: &HashSet<String>) -> HashSet<String> {
    let text = text.to_lowercase();
    for (name, phrases) in ONLY_ONE.iter() {
        if phrases.iter().any(|one| text.contains(one.as_str())) {
            return HashSet::from([name.clone()]);
        }
    }
    let named: HashSet<String> = HAVE_ONE
        .iter()
        .filter(|(_, phrases)| phrases.iter().any(|one| text.contains(one.as_str())))
        .map(|(name, _)| name.clone())
        .filter(|name| !gone.contains(name))
        .collect();
    if !named.is_empty() {
        return named;
    }
    ["oven", "hob", "microwave", "kettle"].iter().map(|s| s.to_string()).filter(|s| !gone.contains(s)).collect()
}

/// Keeping soft herbs, or what to do with slimy ones.
pub fn _herbs_keep(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<&str> = understood.words.iter().map(String::as_str).collect();
    let clean = format!(" {} ", understood.clean);
    if ["slimy", "slime", "rotten", "mouldy", "moldy"].iter().any(|w| words.contains(w)) || clean.contains("gone over") {
        return Some("A slimy herb is past saving, so bin the slimy stems and pick out the ones that are still green to use today. It happens when the leaves stay wet: a bag with damp kitchen paper is right, a wet one sat in a warm fridge is not. Next bunch, stand the stems in an inch of water with a bag over the top, or freeze what you cannot use in the next few days.".to_string());
    }
    Some("Stand soft herbs in a jar with an inch of water, loosely covered with a bag, or wrap them in damp kitchen paper in the fridge. Basil hates the cold, so keep that one on the windowsill instead.".to_string())
}

// ---------------------------------------------------------------------------------------------
// Cooking words and bakes

static WORD_SUBJECT: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| cstrs_map("WORD_SUBJECT"));
static COOK_WORDS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("COOK_WORDS"));
static DEFINITION_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("DEFINITION_WORDS"));
static NEEDS_DEFINITION: LazyLock<HashSet<String>> = LazyLock::new(|| cset("NEEDS_DEFINITION"));

/// Whether a cooking word is said of something ("reduce the sauce", "fold it in").
pub fn _names_a_subject(word: &str, said: &str) -> bool {
    if WORD_SUBJECT.get(word).is_some_and(|subjects| subjects.iter().any(|s| said.contains(s.as_str()))) {
        return true;
    }
    [" it", " them", " this", " these"].iter().any(|p| said.contains(&format!("{word}{p}")))
}

/// Where a cooking word (or its stem) is in the line, as (character position, word).
pub fn _cook_word_hit(word: &str, said: &str) -> Option<(isize, String)> {
    if said.contains(word) {
        return Some((text::find(said, word), word.to_string()));
    }
    let stem = word.trim_end_matches('e');
    if text::len(stem) > 3 && said.contains(stem) {
        return Some((text::find(said, stem), word.to_string()));
    }
    None
}

/// What a cooking word means ("what does reduce mean"), one or two of them.
pub fn _cook_word(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.clean, &understood.words.join(" ")).to_lowercase();
    let mut found: Vec<(isize, String)> = COOK_WORDS.keys().filter_map(|w| _cook_word_hit(w, &said)).collect();
    found.sort();
    let mut named: Vec<String> = Vec::new();
    for (pos, word) in &found {
        let len = text::len(word) as isize;
        if found
            .iter()
            .any(|(op, other)| *pos >= *op && pos + len <= op + text::len(other) as isize && word != other)
        {
            continue;
        }
        named.push(word.clone());
    }
    if named.is_empty() {
        return None;
    }
    if !DEFINITION_WORDS.iter().any(|w| said.contains(w.as_str())) {
        named.retain(|w| !NEEDS_DEFINITION.contains(w) || _names_a_subject(w, &said));
        if named.is_empty() {
            return None;
        }
    }
    if named.len() == 1 {
        return Some(COOK_WORDS[&named[0]].clone());
    }
    Some(format!("{} {}", COOK_WORDS[&named[0]], COOK_WORDS[&named[1]]))
}

/// BAKES: bake -> (the names it goes by, the recipe), in the table's order.
static BAKES: LazyLock<IndexMap<String, (Vec<String>, String)>> = LazyLock::new(|| {
    cget("BAKES")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), (v.items()[0].strs(), v.items()[1].str().to_string()))))
        .collect()
});
static BAKE_ALIASES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    let mut out: Vec<(Vec<String>, String)> = Vec::new();
    for (name, (aliases, _)) in BAKES.iter() {
        for alias in aliases {
            out.push((text::split(alias).iter().map(|p| _fold(p)).collect(), name.clone()));
        }
    }
    out.sort_by_key(|(parts, _)| std::cmp::Reverse(parts.len()));
    out
});
static PANTRY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| PANTRY.keys().map(|n| _fold(n)).collect());

/// Every bake name as its folded words, longest first.
pub fn _bake_aliases() -> &'static [(Vec<String>, String)] {
    &BAKE_ALIASES
}

/// The pantry foods as folded words.
pub fn _pantry_words() -> &'static HashSet<String> {
    &PANTRY_WORDS
}

/// The bakes the folded words name, as (their words, the bake), longest names first.
pub fn _bake_hits(folded: &[String], skip_pantry: bool) -> Vec<(Vec<String>, String)> {
    let mut hits = Vec::new();
    for (parts, name) in _bake_aliases() {
        if skip_pantry && parts.len() == 1 && _pantry_words().contains(&parts[0]) {
            continue;
        }
        if folded.len() >= parts.len() && (0..=(folded.len() - parts.len())).any(|s| folded[s..s + parts.len()] == parts[..]) {
            hits.push((parts.clone(), name.clone()));
        }
    }
    hits
}

/// Whether the line names a bake it has not already baked.
pub fn _names_a_bake(understood: &Understood) -> bool {
    let folded: Vec<String> = words_or_clean(understood).iter().map(|w| _fold(w)).collect();
    _bake_hits(&folded, true).iter().any(|(parts, _)| !_already_baked(&folded, parts))
}

static ALREADY_BAKED_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_ALREADY_BAKED_WORDS"));
static ALREADY_BAKED_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_ALREADY_BAKED_PHRASES"));

/// Whether the words before the bake say it is already made ("my cake", "the brownies i made").
pub fn _already_baked(folded: &[String], parts: &[String]) -> bool {
    if folded.len() < parts.len() {
        return false;
    }
    for start in 0..=(folded.len() - parts.len()) {
        if folded[start..start + parts.len()] != *parts {
            continue;
        }
        let window = &folded[start.saturating_sub(3)..start];
        if window.iter().any(|w| ALREADY_BAKED_WORDS.contains(w)) {
            return true;
        }
        let joined = window.join(" ");
        return ALREADY_BAKED_PHRASES.iter().any(|p| joined.contains(p.as_str()));
    }
    false
}

static TIMING_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_TIMING_ASKS"));
static NO_YEAST: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_NO_YEAST"));

/// The recipe for a bake the line asks how to make.
pub fn _bake(_memory: &mut Memory, understood: &Understood, text: Option<&str>) -> Option<String> {
    let mut folded: Vec<String> = words_or_clean(understood).iter().map(|w| _fold(w)).collect();
    if let Some(t) = text.filter(|t| !t.is_empty()) {
        folded.extend(text::split(&t.to_lowercase()).iter().map(|w| _fold(w)));
    }
    let said = folded.join(" ");
    if TIMING_ASKS.iter().any(|p| said.contains(p.as_str())) {
        return None;
    }
    if NO_YEAST.iter().any(|p| said.contains(p.as_str())) {
        return None;
    }
    if understood.asks.is_empty() && !folded.iter().any(|w| ["how", "what", "whats", "recipe"].contains(&w.as_str())) {
        return None;
    }
    if _asks_vegan(&folded) {
        return None;
    }
    let hits = _bake_hits(&folded, ingredients(understood).len() > 1);
    if hits.is_empty() || _already_baked(&folded, &hits[0].0) {
        return None;
    }
    Some(BAKES[&hits[0].1].1.clone())
}

static NONSTICK_CLEAN: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_NONSTICK_CLEAN"));
static NONSTICK_KEEP: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_NONSTICK_KEEP"));

/// Cleaning a non-stick pan, or whether a scratched one is still fine.
pub fn _non_stick_pan(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_norm_said(understood);
    let wants_clean = NONSTICK_CLEAN.iter().any(|w| said.contains(w.as_str()));
    if !wants_clean && NONSTICK_KEEP.iter().any(|w| said.contains(w.as_str())) {
        return Some("Small scratches are fine to keep cooking on. Once it is scratched through to the metal, or the coating is flaking or peeling into the food, replace the pan: you do not want flakes of coating in a meal, and a worn pan stops being non-stick anyway. Until then, keep metal tools, scouring pads and an empty pan on high heat away from it, because those are what scratch it.".to_string());
    }
    Some("Warm water, washing-up liquid and a soft sponge, then dry it with a cloth: no scouring pad, no wire wool and no metal tools, because a scratch in the coating is what ruins it. For burnt-on food, simmer a little water in the pan for five minutes and lift it with a wooden spatula rather than scraping. If the coating is flaking or peeling, replace the pan - that is not something to cook through.".to_string())
}

static FOR_A_MEAL: LazyLock<&'static Re> = LazyLock::new(|| cre("_FOR_A_MEAL"));
static ASKS_WHAT_TO_HAVE: LazyLock<&'static Re> = LazyLock::new(|| cre("_ASKS_WHAT_TO_HAVE"));

fn has_any(words: &HashSet<String>, wanted: &[&str]) -> bool {
    wanted.iter().any(|w| words.contains(*w))
}

/// How to make tea, and what has gone wrong with it.
pub fn _tea_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_norm_said(understood);
    if FOR_A_MEAL.is_match(&said) && ASKS_WHAT_TO_HAVE.is_match(&said) {
        return None;
    }
    let words: HashSet<String> = WORDS_AZ.findall_str(&said).into_iter().collect();
    let name = if has_any(&words, &["iced", "ice"]) {
        "TEA_ICED_TEXT"
    } else if has_any(&words, &["pot", "teapot", "teapots"]) {
        "TEA_POT_TEXT"
    } else if has_any(&words, &["cloudy", "cloudiness", "scum", "murky", "film"]) {
        "TEA_CLOUDY_TEXT"
    } else if has_any(&words, &["green", "greens", "sencha", "matcha", "jasmine"]) {
        "TEA_GREEN_TEXT"
    } else if has_any(&words, &["twice", "again", "reuse", "re-use", "second"]) {
        "TEA_TWICE_TEXT"
    } else if has_any(
        &words,
        &["bitter", "sour", "soured", "stewed", "weak", "strong", "stronger", "watery", "tastes", "taste", "tasting", "stewing"],
    ) {
        "TEA_FAULT_TEXT"
    } else {
        "TEA_METHOD_TEXT"
    };
    Some(cstring(name))
}

/// How long rice takes and how much water: brown, or white by the clock.
pub fn _rice_time(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<String> = understood.words.iter().cloned().collect();
    let asks: HashSet<String> = understood.asks.iter().cloned().collect();
    if has_any(&words, &["brown", "wholegrain", "whole"]) {
        return Some("Brown rice takes 25 to 30 minutes, with a little more water: 1 part rice to 2 parts water, lid on, low simmer. Let it sit 5 minutes off the heat, still covered, before you fork it through. White long grain is 1 part rice to 1.5 parts water for 12 minutes, then 10 minutes off the heat. Too much water is what turns rice to porridge.".to_string());
    }
    if has_any(&asks, &["how_long", "when"]) {
        return Some("White long grain takes 10 to 12 minutes on a low simmer with the lid on, then 10 more off the heat. Use 1 part rice to about 1.5 parts water by volume, or the fingertip test: water one knuckle above the rice. Brown and wholegrain are the ones that take longer. Too much water is what turns rice to porridge.".to_string());
    }
    Some("Use 1 part rice to about 1.5 parts water by volume for long grain, or the fingertip test: water one knuckle above the rice, and give it 12 minutes on a low simmer with the lid on, then 10 more off the heat. Brown and wholegrain rice are the ones that take longer. Too much water is what turns rice to porridge.".to_string())
}

/// `_bread_rise_answer()`'s answer: how a loaf rises, and what to do when it will not.
pub fn _bread_rise_answer_answer(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = or(&understood.clean, &understood.words.join(" ")).to_lowercase();
    if NO_YEAST.iter().any(|one| said.contains(one.as_str())) {
        return None;
    }
    let has = |p: &str| said.contains(p);
    if has("sourdough") || has("sour dough") || has("starter") {
        return Some("A sourdough loaf rises on the starter, not on warmth alone: a starter that doubles in 4 to 6 hours after a feed is strong enough to bake with, and one that only bubbles is not, so feed it twice a day for three days first. Then give the bulk 4 to 6 hours at room temperature with three or four sets of stretch and fold in the first two hours, and shape it with real tension. A flat loaf usually wants a stiffer dough and a longer, colder second prove.".to_string());
    }
    if has("fridge") || has("overnight") || has("in the cold") {
        return Some("A night in the fridge is a slow prove on purpose, not a fault: shape it cold, then leave it an hour at room temperature before it goes in the oven. If it hasn't risen at all by the morning, the yeast is dead, so check the date.".to_string());
    }
    if has("twice") || has("second prove") || has("second rise") || has("two proves") {
        return Some("Yes, twice: the first is done when the dough has doubled, and the second is the shaped loaf, about an hour, until it springs back slowly when you poke it. Skipping the second gives you a tight, close crumb.".to_string());
    }
    if ["wont", "won't", "isnt", "isn't", "not rising", "didnt", "didn't", "hasnt risen", "stalled"].iter().any(|p| has(p)) {
        return Some("Give it another hour somewhere around 25C, until it has doubled, and check the water was warm rather than hot and that the yeast is in date. If it still hasn't moved at all, the yeast is dead, so start again with fresh yeast.".to_string());
    }
    Some("Leave it somewhere around 25C for 1 to 2 hours, until the dough has doubled. Yeast eats the sugars in the flour and gives off gas, and the stretchy gluten holds it in bubbles, so warmth and time are the whole job. If it hasn't moved at all, the yeast is dead, so check the date and use water that's warm rather than hot.".to_string())
}

/// Sour, burnt or bitter coffee, by what the line says it tastes of.
pub fn _coffee_fault(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words: HashSet<String> = WORDS_AZ.findall_str(&understood.clean).into_iter().collect();
    if has_any(&words, &["sour", "soured", "acidic", "acid", "vinegary"]) {
        return Some(cstring("_COFFEE_SOUR"));
    }
    if has_any(&words, &["burnt", "burned", "burning", "scorched", "scorching"]) {
        return Some(cstring("_COFFEE_BURNT"));
    }
    Some(cstring("_COFFEE_BITTER"))
}

// ---------------------------------------------------------------------------------------------
// Burnt, greasy, split and undercooked food

/// `re.search(r"\b(?:a|b|c)\b", said)` for one list of phrases, built once.
pub struct Says(Re);

impl Says {
    fn of(phrases: Vec<String>) -> Says {
        Says(phrases_re(&phrases))
    }
    fn data(name: &str) -> Says {
        Says::of(cstrs(name))
    }
    fn words(phrases: &[&str]) -> Says {
        Says::of(sv(phrases))
    }
    /// Whether one of the phrases is in the text as whole words.
    pub fn said(&self, text: &str) -> bool {
        self.0.is_match(text)
    }
}

/// Whether one of the phrases is in the text as whole words.
pub fn _says(said: &str, phrases: &[String]) -> bool {
    phrases_re(phrases).is_match(said)
}

static S_BURNT_ON_A_PAN: LazyLock<Says> = LazyLock::new(|| Says::data("_BURNT_ON_A_PAN"));
static S_BURNT_MARK: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&[
        "burnt", "burned", "black", "blackened", "scorched", "scorch", "burn mark", "scorch mark", "scorched on", "stain",
        "stained", "discoloured", "discolored", "scratch", "scratched",
    ])
});

/// Whether the line is about a mark burnt onto a pan rather than the food.
pub fn _burnt_on_the_pan(text: &str) -> bool {
    S_BURNT_ON_A_PAN.said(text) && S_BURNT_MARK.said(text)
}

static S_BURNT_EATING_ASK: LazyLock<Says> = LazyLock::new(|| Says::data("_BURNT_EATING_ASK"));
static S_BURNT_BATTER: LazyLock<Says> = LazyLock::new(|| Says::data("_BURNT_BATTER"));
static S_BURNT_VEG: LazyLock<Says> = LazyLock::new(|| Says::data("_BURNT_VEG"));
static S_MEATS: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&[
        "chicken", "beef", "lamb", "pork", "turkey", "gammon", "duck", "steak", "sausage", "sausages", "bacon", "ham",
        "joint", "brisket", "meatballs",
    ])
});
static VESSEL_PHRASES: LazyLock<&'static Re> = LazyLock::new(|| cre("_VESSEL_PHRASES"));
static S_ROAST_PARTS: LazyLock<Says> = LazyLock::new(|| Says::data("_ROAST_PARTS"));
static S_BURNT_MEAT: LazyLock<Says> = LazyLock::new(|| Says::data("_BURNT_MEAT"));
static S_PAN_OR_WET: LazyLock<Says> = LazyLock::new(|| {
    let mut words = cstrs("_PAN_WORDS");
    words.extend(sv(&["sauce", "gravy", "soup", "stew", "curry", "rice"]));
    Says::of(words)
});
static S_PAN_DISH: LazyLock<Says> = LazyLock::new(|| Says::data("_PAN_DISH"));
static S_PAN_WORDS: LazyLock<Says> = LazyLock::new(|| Says::data("_PAN_WORDS"));
static S_TIN_OR_OVEN: LazyLock<Says> = LazyLock::new(|| Says::data("_TIN_OR_OVEN"));

/// What to do with a burnt dish, by what burnt: eating it, batter, veg, a roast, meat.
pub fn _burnt_dish(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_norm_said(understood);
    if S_BURNT_EATING_ASK.said(&said) {
        return Some(cstring("_BURNT_EATING_REPLY"));
    }
    if S_BURNT_BATTER.said(&said) {
        return Some(cstring("_BURNT_BATTER_REPLY"));
    }
    if S_BURNT_VEG.said(&said) && !S_MEATS.said(&said) {
        return Some(cstring("_BURNT_VEG_REPLY"));
    }
    let without_vessel = VESSEL_PHRASES.sub(" ", &said);
    if S_ROAST_PARTS.said(&without_vessel) {
        return Some(cstring("_ROAST_PART_REPLY"));
    }
    if S_BURNT_MEAT.said(&without_vessel) && !S_PAN_OR_WET.said(&without_vessel) {
        return Some(cstring("_BURNT_MEAT_REPLY"));
    }
    let baked = VESSEL_PHRASES.sub(" ", &said);
    if S_PAN_DISH.said(&baked) {
        return Some(cstring("_BURNT_DISH_REPLY"));
    }
    if S_PAN_WORDS.said(&said) || S_TIN_OR_OVEN.said(&baked) {
        return Some(String::new());
    }
    Some(cstring("_BURNT_DISH_REPLY"))
}

/// The line, its normalised form and its normalised words, lower-case, as one string.
pub fn _words_and_text(understood: &Understood) -> String {
    format!("{} {} {}", understood.clean, understood.norm_text, understood.norm.join(" ")).to_lowercase()
}

static S_PAN_CLEAN: LazyLock<Says> = LazyLock::new(|| Says::data("_PAN_CLEAN"));

/// Cleaning a burnt pan (unless it is a dish that burnt).
pub fn _burnt_pan(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_norm_said(understood);
    if S_PAN_CLEAN.said(&said) {
        return Some(cstring("_BURNT_PAN_REPLY"));
    }
    if S_PAN_DISH.said(&said) {
        return Some(String::new());
    }
    Some(cstring("_BURNT_PAN_REPLY"))
}

static TIN_FRUIT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_TIN_FRUIT_WORDS"));
static TIN_LINE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_TIN_LINE_WORDS"));
static TIN_GREASE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_TIN_GREASE_WORDS"));

/// The cake tin: its size, filling, lining and greasing, led by what the line asks.
pub fn _tin_answer(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_norm_said(understood);
    let (size, fill, line, fruit, grease) =
        (cstring("_TIN_SIZE"), cstring("_TIN_FILL"), cstring("_TIN_LINE"), cstring("_TIN_FRUIT"), cstring("_TIN_GREASE"));
    if TIN_FRUIT_WORDS.iter().any(|w| said.contains(w.as_str())) {
        return Some([fruit, size].join(" "));
    }
    if TIN_LINE_WORDS.iter().any(|w| said.contains(w.as_str())) {
        return Some([line, size, fill].join(" "));
    }
    if TIN_GREASE_WORDS.iter().any(|w| said.contains(w.as_str())) {
        return Some([grease, size].join(" "));
    }
    Some([size, fill, line].join(" "))
}

static CAKE_FAULT_WORDS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| cget("_CAKE_FAULT_WORDS").items().iter().map(|r| r.strs()).collect());

/// A cake that domed, cracked, tastes of egg or has a soggy bottom.
pub fn _cake_fault(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = clean_norm_said(understood);
    let has = |i: usize| CAKE_FAULT_WORDS[i].iter().any(|w| words.contains(w.as_str()));
    if has(0) {
        return Some("A dome is the outside setting before the middle has caught up: bake 10C lower, 170C (150C fan) on the middle shelf, use the tin the recipe names rather than a smaller one, and stop beating the moment the flour is in. Trim the dome flat with a bread knife before you fill it, and nobody sees it.".to_string());
    }
    if has(1) {
        return Some("A split top is the crust setting while the middle is still rising: lower the oven 10C, put the tin on the middle shelf, and keep the door shut for the first 20 minutes. Cool it in the tin for 10 minutes, then on a rack, because a cake cooled in a hot tin goes clammy and splits.".to_string());
    }
    if has(2) {
        return Some("An eggy taste is too many eggs for the flour or a bake that stopped a few minutes short: add a quarter teaspoon of salt and a teaspoon of vanilla or the zest of a lemon, and give it 5 more minutes, checking with a skewer. The same mix tastes eggier cold, so try it once it has cooled rather than from the tin.".to_string());
    }
    if has(3) {
        return Some("A soggy bottom is underbaked underneath: use a metal tin rather than a ceramic dish, bake on the middle shelf and not above a tray of water, and give it 5 to 10 minutes longer with foil over the top once it is brown. A cold filling or a wet fruit layer under a thick batter does the same thing.".to_string());
    }
    Some("An even rise is a level batter and a steady oven: sift the raising agent through the flour rather than adding it last, spread the batter level with the back of a spoon, and use a tin that the mix fills by two thirds. Turn the tin once, after 15 minutes, if your oven browns the back corner first.".to_string())
}

static PASTRY_SHRINK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PASTRY_SHRINK_WORDS"));

/// Pastry that shrank, or a soggy pastry base.
pub fn _pastry_fault(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_norm_said(understood);
    if PASTRY_SHRINK_WORDS.iter().any(|w| said.contains(w.as_str())) {
        return Some("Pastry shrinks when it is stretched into the tin: press it gently into the corners rather than pulling the sides up, chill the lined case for 20 minutes before it goes in, and leave the overhang on until after baking, because the spare pastry holds the sides up while they set.".to_string());
    }
    Some("A soggy pastry base wants more heat underneath it and less water in the filling: blind bake it 5 minutes longer than the recipe says, brush the warm base with beaten egg white before the filling goes in, and stand the tin on a hot baking sheet so the heat reaches the pastry rather than stopping at the filling.".to_string())
}

static S_BAKE_NOUNS: LazyLock<Says> = LazyLock::new(|| Says::data("_BAKE_NOUNS"));
static S_BAKED_BURNT: LazyLock<Says> = LazyLock::new(|| Says::data("_BAKED_BURNT"));
static BAKE_BOTTOM: LazyLock<&'static Re> = LazyLock::new(|| cre("_BAKE_BOTTOM"));
static BAKE_STOP: LazyLock<&'static Re> = LazyLock::new(|| cre("_BAKE_STOP"));
static BAKE_DONE: LazyLock<&'static Re> = LazyLock::new(|| cre("_BAKE_DONE"));

/// A bake burning: on the bottom, how to stop it, or what to do with a burnt one.
pub fn _bake_burning(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = clean_norm_said(understood);
    let text = _words_and_text(understood);
    let baked = VESSEL_PHRASES.sub(" ", &text);
    if !S_BAKE_NOUNS.said(&baked) && !S_BAKED_BURNT.said(&baked) && S_TIN_OR_OVEN.said(&text) {
        return Some(String::new());
    }
    if BAKE_BOTTOM.is_match(&said) {
        return Some(cstring("_BAKE_BOTTOM_REPLY"));
    }
    if BAKE_STOP.is_match(&said) || said.contains("burning") {
        return Some(cstring("_BAKE_PREVENTION"));
    }
    if BAKE_DONE.is_match(&said) {
        return Some(cstring("_BAKE_BURNT"));
    }
    Some(cstring("_BAKE_PREVENTION"))
}

static BAKED_BURNT: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_BAKED_BURNT"));

/// Whether the line's words name one of the things (a word starting with it, or a phrase in it).
fn have(words: &[String], said: &str, names: &[String]) -> bool {
    words.iter().any(|w| names.iter().any(|n| _same(w, n))) || names.iter().any(|n| n.contains(' ') && said.contains(n.as_str()))
}

fn lower_words(understood: &Understood) -> Vec<String> {
    understood.words.iter().map(|w| w.to_lowercase()).collect()
}

/// A burnt bake, burnt toast, or a burnt pan of food.
pub fn _burnt(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let said = words.join(" ");
    if have(&words, &said, &BAKED_BURNT) {
        return Some("A burnt top on a bake is usually saveable: take it out at once, cover the tin loosely with foil so the middle carries on cooking, and when it is cool cut the black off with a serrated knife or grate it away - a dark top on a fruit cake is often just the sugar. Next time move the shelf down one, drop the oven 10C and tent the tin with foil after 20 minutes, because a bake burns on top while the middle is still wet.".to_string());
    }
    if words.iter().any(|w| w == "toast" || w == "toaster") {
        return Some("Scrape the black off with a knife and eat the rest: burnt toast is not dangerous, and a scrape takes the bitter taste with it. Next time set the toaster one notch lower and stay next to it, because toast goes from golden to black in under a minute.".to_string());
    }
    Some("Take it off the heat and lift out the burnt bits rather than stirring them in, because the smoky taste spreads through everything. If it has gone all the way through, a splash of stock and something creamy hides a little of it, but a properly burnt sauce is better started again.".to_string())
}

static FRIED_GREASY: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_FRIED_GREASY"));
static PASTRY_GREASY: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PASTRY_GREASY"));
static MEAT_GREASY: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_MEAT_GREASY"));
static POT_GREASY: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_POT_GREASY"));

/// Greasy food, by what it is: fried, roast potatoes, pastry, a roast, a stew.
pub fn _greasy(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let said = words.join(" ");
    if have(&words, &said, &FRIED_GREASY) {
        return Some("Greasy fried food means the oil was not hot enough, so the food steamed instead of sealing and took the fat up like a sponge: 180C for chips and about 190C for a batter, test it with a cube of bread that should sizzle and go golden in 30 seconds, and do not crowd the pan, because the temperature drops as soon as it is full. Drain on a rack or kitchen paper, never on a plate, and salt it the moment it comes out.".to_string());
    }
    let potatoes = ["roast potatoes", "roasties", "potatoes", "potato", "wedges", "hash browns"];
    let roasted = ["roast", "roasted", "oven"];
    if words.iter().any(|w| potatoes.iter().any(|p| _same(w, p))) && words.iter().any(|w| roasted.iter().any(|p| _same(w, p))) {
        return Some("Greasy roast potatoes mean the oil was not hot enough or the tin was crowded: parboil them 8 minutes, drain and shake the pan to rough the edges, then roast them in a thin layer of oil that is already smoking hot at 200C (180C fan), in one layer with room between them. Cold oil, or a full tin, is what leaves them oily, and a rack over the tin drains the rest.".to_string());
    }
    if have(&words, &said, &PASTRY_GREASY) {
        return Some("A greasy pastry case is the filling rather than the pastry: cheese in a quiche gives up its fat, and mince in a pie floats it to the top. Drain the cooked filling in a sieve over a bowl, thicken it with a spoon of cornflour, and blind-bake the case first so the fat has somewhere to go; a layer of breadcrumbs on the base soaks up the rest.".to_string());
    }
    if have(&words, &said, &MEAT_GREASY) {
        return Some("Most of that fat is meant to be there, so spoon it off rather than cook it away: tilt the tin and take the fat out with a spoon or a bulb baster, keeping a spoonful for the potatoes, and finish the meat uncovered at a higher heat for the last 10 minutes so the skin or the outside crisps. Resting it on a rack, not in the tin, keeps it out of the fat.".to_string());
    }
    if have(&words, &said, &POT_GREASY) {
        return Some("Skim the fat off the top with a spoon, or lay a piece of kitchen paper flat on the surface and lift it out - that takes the oil and leaves the sauce. A stew that is still greasy wants the lid off for 10 minutes, and a spoon of something sharp (lemon, vinegar, tomato puree) cuts through what is left. Next time brown the meat and pour the fat out of the pan before the liquid goes in.".to_string());
    }
    Some("Greasy comes from too much oil or oil that was not hot enough to seal the food: get the pan properly hot before anything goes in, shake the excess off in a sieve, and drain on kitchen paper rather than on a plate. In a sauce, skim the fat off the top with a spoon and finish with a squeeze of lemon.".to_string())
}

static CUSTARD_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CUSTARD_NAMES"));

/// A split custard, mayonnaise, hollandaise or sauce.
pub fn _split(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let said = words.join(" ");
    if have(&words, &said, &CUSTARD_NAMES) {
        return Some("A curdled custard is the eggs getting hot too fast, so the protein in them sets into grains: take it off the heat at once, whisk in a spoonful of cold cream or milk to stop it cooking, then push it through a sieve and it comes back smooth. Next time pour the hot milk into the beaten eggs a splash at a time, keep the pan over a low heat, stir all the time, and take it off at 82C, because a custard that comes to the boil will always curdle.".to_string());
    }
    let mayo = ["mayonnaise", "mayo", "aioli", "garlic mayo", "garlic mayonnaise", "remoulade"];
    if words.iter().any(|w| mayo.contains(&w.as_str())) {
        return Some("Mayonnaise splits when the oil goes in faster than the yolk can take it up, so start again with a clean bowl: an egg yolk, a teaspoon of water and a pinch of salt, then the oil a drop at a time until it thickens and only then in a thin stream. To rescue a split one, put a fresh yolk in a clean bowl and whisk the split mayonnaise into it a spoon at a time - one yolk will bring back up to about a mugful.".to_string());
    }
    let hollandaise = ["hollandaise", "béarnaise", "bearnaise", "butter sauce"];
    if words.iter().any(|w| hollandaise.contains(&w.as_str())) {
        return Some("Hollandaise splits when the bowl gets too warm or the butter goes in too hot: take the bowl off the water at once, add a teaspoon of cold water and whisk hard. If it will not come back, start a fresh yolk in a clean bowl with a spoon of water and whisk the split sauce into it a little at a time, off the heat. Next time keep the water barely trembling and let the melted butter cool for a minute first.".to_string());
    }
    Some("A sauce splits when it gets too hot: take it off the heat and whisk in a spoonful of cold cream or butter to bring it back. Next time, lower the heat and add dairy at the end.".to_string())
}

static UNDER_BREAD: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_UNDER_BREAD"));
static UNDER_BAKE: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_UNDER_BAKE"));
static UNDER_STARCH: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_UNDER_STARCH"));
static UNDER_VEG: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_UNDER_VEG"));

/// Undercooked food, by what it is: a loaf, a cake, rice and pasta, vegetables.
pub fn _undercooked(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let said = words.join(" ");
    if have(&words, &said, &UNDER_BREAD) {
        return Some("A doughy loaf has not had long enough in the oven, or it did not prove enough: put it back in at 200C (180C fan) for 10 minutes, then turn it out and tap the base - a hollow sound means it is done, a dull thud means it wants 5 more. Next time let it prove until a poke springs back slowly and bake it to 96C in the middle rather than trusting the time on the packet.".to_string());
    }
    if have(&words, &said, &UNDER_BAKE) {
        return Some("Put it back in the oven at 160C (140C fan) for 10 minutes with a sheet of foil loosely over the top, so the middle catches up without the top burning, and test it again with a skewer: it wants a few damp crumbs, not wet mixture. Next time drop the oven 10C and give it 5 minutes longer, and check with a skewer before you take it out - a cake that looks set and wobbles in the middle is not done.".to_string());
    }
    if have(&words, &said, &UNDER_STARCH) {
        return Some("It wants more time and a little more water: a splash of hot water, the lid back on and 5 minutes on the lowest heat will finish rice, couscous or quinoa. Pasta and noodles go back into the boiling water for 2 to 3 minutes, and lentils want another 10. Taste a piece rather than looking at the clock, because the packet time assumes a pan that came back to the boil quickly.".to_string());
    }
    if have(&words, &said, &UNDER_VEG) {
        return Some("Boiled vegetables go hard in the middle when the pieces are different sizes or the water never got back to a rolling boil: cut them the same size, salt the water, and give them another 5 minutes, testing with the point of a knife - it should slide in with no resistance at all. Roast ones want 10 more minutes in a hotter oven, and parboiling potatoes for 8 minutes before they go in the tin stops the hard centre.".to_string());
    }
    Some("It needs a few more minutes rather than more heat: cover it and give it 5, then test it with a knife or a skewer in the middle, where the heat gets last. A wider, shallower dish cooks through faster than a deep one.".to_string())
}

/// Whether the text names a dish whose plan has an oven temperature.
pub fn _a_dish_with_a_heat(text: &str) -> bool {
    a_dish_plan_has_a_heat(text)
}

static BODY_PART: LazyLock<&'static Re> = LazyLock::new(|| cre("_BODY_PART"));
static ONTO_A_PERSON: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_ONTO_A_PERSON"));

/// Hot fat on a hand or an arm: a burn, not a cooking question.
pub fn _fat_has_landed_on_a_person(text: &str) -> bool {
    BODY_PART.is_match(text) && ONTO_A_PERSON.iter().any(|w| text.contains(w.as_str()))
}

static OIL_SMOKE_POINTS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| cpairs("OIL_SMOKE_POINTS"));
static S_SMOKE_POINT: LazyLock<Says> =
    LazyLock::new(|| Says::words(&["smoke point", "smoking point", "smokes at", "smoke at", "smoke too"]));
static S_IN_THE_WATER: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&[
        "in the water", "pasta water", "rice water", "boiling water", "the water for", "water for the pasta",
        "water for the rice", "in the pot of water", "into the water", "salt water", "salted water", "the pasta water",
    ])
});
static S_OIL_FIRST: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&[
        "put the oil in", "put in the oil", "add the oil", "adding the oil", "put oil in", "add oil to", "adding oil",
        "add oil", "oil go in", "oil goes in", "oil first or the pan", "pan first", "oil in the pan", "oil in a pan",
        "oil in pan", "hot pan first", "pan be hot", "pan hot before", "before the oil", "before adding the oil",
        "before adding oil", "hot pan", "pan hot before", "when do i add the oil",
    ])
});
static S_FRYING: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&["oil", "fat", "pan", "fry", "frying", "deep fry", "chip pan", "dripping", "lard", "ghee", "wok"])
});
static S_HOW_HOT: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&[
        "how hot", "hot enough", "what temperature", "temperature for", "hot should", "heat of the oil",
        "temperature does", "is hot", "is ready", "pan heat", "the heat", "test the heat", "check the heat", "hot oil",
        "oil is hot", "oil is ready", "fat is hot", "fat is ready", "heat the pan", "heating the pan", "heat the oil",
        "heating the oil",
    ])
});
static S_FAT_NAMED: LazyLock<Says> =
    LazyLock::new(|| Says::words(&["oil", "fat", "dripping", "lard", "ghee", "grease", "butter"]));
static S_DOWN_THE_SINK: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&["down the sink", "down the drain", "down the toilet", "down the loo", "down the plug hole", "down the plughole"])
});
static S_GET_RID: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&[
        "throw out", "throw away", "throwing out", "get rid", "getting rid", "dispose", "disposal", "bin it", "bin used",
        "bin the oil", "recycle", "recycling", "down the sink", "where do i put", "where do i pour", "what to do with",
        "where does it go", "where can i put", "put used", "pour it", "pour old", "do with old oil", "do with used oil",
        "do with the oil", "do with the fat", "do with leftover oil", "what do i do with", "where does it go", "pour it",
        "pour the oil", "tip it",
    ])
});
static S_REUSE: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&[
        "reuse", "re-use", "again", "second time", "keep the oil", "straining", "strain", "can i use", "can i still use",
        "still use", "still good", "fine to use", "ok to use", "okay to use", "fry in", "fry with",
    ])
});
static S_FOAM: LazyLock<Says> = LazyLock::new(|| {
    Says::words(&[
        "foam", "foams", "foamy", "foaming", "foamed", "bubbling up", "bubbles up", "spitting", "spits", "spattered",
        "spattering",
    ])
});
static S_WRONG_WITH_THE_PAN: LazyLock<Says> = LazyLock::new(|| Says::data("_SOMETHING_IS_WRONG_WITH_THE_PAN"));

fn smoke_points(rows: &[&(String, String)]) -> String {
    rows.iter().map(|(name, temp)| format!("{name} {temp}")).collect::<Vec<_>>().join("; ")
}

/// Oil and fat in the pan: smoke points, when it is hot enough, getting rid of it, re-using it,
/// foaming, and what to do when it smokes.
pub fn _oil_smoking(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = or(&understood.clean, &understood.text).to_lowercase();
    if _fat_has_landed_on_a_person(&text) {
        return Some(String::new());
    }
    if _burnt_on_the_pan(&text) {
        return Some(String::new());
    }
    if S_SMOKE_POINT.said(&text) {
        let all: Vec<&(String, String)> = OIL_SMOKE_POINTS.iter().collect();
        return Some(format!(
            "Every fat has a smoke point: {}. Above that it is breaking down, so keep the pan under it - a thermometer takes the guesswork out, and a drop of the oil in a cold pan, brought up slowly, tells you when it is ready.",
            smoke_points(&all)
        ));
    }
    if S_IN_THE_WATER.said(&text) {
        return Some(String::new());
    }
    if S_OIL_FIRST.said(&text) {
        return Some("Pan first, then the oil: a dry pan heats faster and you can watch the oil for the heat - it thins and shimmers at about 180C, and a cube of bread turns golden in 30 seconds. Give the oil 20 seconds before the food goes in. Cold oil in a cold pan is for rendering fat, not a sear.".to_string());
    }
    if !S_FRYING.said(&text) && _a_dish_with_a_heat(&text) {
        return Some(String::new());
    }
    if S_HOW_HOT.said(&text) {
        let some: Vec<&(String, String)> =
            OIL_SMOKE_POINTS.iter().take(1).chain(OIL_SMOKE_POINTS.iter().skip(2).take(2)).collect();
        return Some(format!(
            "Around 180C for most frying, 170C for chips so they cook through before they colour; with no thermometer a cube of day-old bread turns golden in about 30 seconds. Heat the oil with nothing in it and fry in small batches so it does not drop. Stay under the fat's smoke point: {}.",
            smoke_points(&some)
        ));
    }
    let fat_named = S_FAT_NAMED.said(&text);
    if fat_named && S_DOWN_THE_SINK.said(&text) {
        return Some("No - never down the sink, the loo or the drain: fat sets hard as it cools and closes the pipe, and hot water only pushes it further down to block somewhere worse, which is a plumber's bill. Let it cool, pour it into the bottle or a jar you were going to bin, and put that in the rubbish; wipe the pan out with kitchen roll before you wash it.".to_string());
    }
    if fat_named && S_GET_RID.said(&text) {
        return Some("Let it cool completely, then funnel it back into the bottle it came in or any screw-top jar you were going to bin, and put that in the rubbish - never down the sink, because it sets in the drain and it is what makes fatbergs. A small amount from one pan can be soaked up with kitchen roll and binned; a big batch goes to the tip, and most councils and some supermarkets take used cooking oil for recycling. Keep a jar under the sink and pour into it while it is still warm, and it never gets as far as the drain.".to_string());
    }
    if S_REUSE.said(&text) {
        return Some("Let it cool, strain it through a fine sieve or a coffee filter, and use it once more for something at a lower heat. Oil that has smoked has started to break down, so it will taste acrid and it will smoke sooner next time; bin it if it smells bitter or looks dark and sticky.".to_string());
    }
    if S_FOAM.said(&text) {
        return Some("Oil that foams has water in it rather than too much heat: dry the food with kitchen paper and lower it in gently, cook in two batches so the pan does not drop below its temperature, and keep the lid or the basket dry. If oil you have already used a few times foams as soon as anything goes in, it is spent - let it cool, bin it in a bottle, and start with fresh.".to_string());
    }
    if !S_WRONG_WITH_THE_PAN.said(&text) {
        return Some(String::new());
    }
    Some("The fat is past its smoke point, so turn the heat down and take the pan off the ring for a minute rather than carrying on. Open a window or put the extractor on, and once it has cooled wipe the pan out and start again with fresh fat - the smoked oil has turned acrid and it will make the food taste of it. Butter and extra virgin olive oil smoke first (about 150C and 180C), so use a plain vegetable oil or clarified butter when you want the pan very hot. If it ever actually catches, put a lid or a baking tray over the pan and turn the heat off - never water.".to_string())
}

static DIP_FIXES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| cnames_rows("_DIP_FIXES"));

/// A dip gone wrong: watery houmous, thin tzatziki, and the rest.
pub fn _dip_fault(understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    if ["houmous", "hummus", "humous"].iter().any(|n| text.contains(n))
        && ["watery", "too thin", "runny", " thin"].iter().any(|w| text.contains(w))
    {
        return "Watery houmous usually has too much liquid or not enough tahini: drain the chickpeas well, then stir in a spoonful of tahini and a little olive oil and chill it for 20 minutes, because it firms up cold. Taste for salt and lemon at the end.".to_string();
    }
    for (names, fix) in DIP_FIXES.iter() {
        if names.iter().any(|n| text.contains(n.as_str())) {
            return fix.clone();
        }
    }
    cstring("_DIP_PLAIN")
}

static SOGGY_BREAD: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_SOGGY_BREAD"));
static SOGGY_LEAF: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_SOGGY_LEAF"));

/// A soggy sandwich, salad or packed lunch.
pub fn _soggy_lunch(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let said = words.join(" ");
    if have(&words, &said, &SOGGY_BREAD) {
        return Some("A sandwich goes soggy because the wet filling soaks into the bread: butter both slices right to the edges first, or spread mayonnaise or soft cheese as a barrier, and keep tomato, cucumber and lettuce in the middle rather than against the bread. Packed in the morning, the wet things go in a separate tub and are added at lunchtime, which is the only thing that really works on a hot day.".to_string());
    }
    if have(&words, &said, &SOGGY_LEAF) {
        return Some("A salad goes soggy because the leaves are still wet or the dressing went on early: dry them properly in a tea towel or a salad spinner, dress it at the table, and keep the wet things - tomato, cucumber, dressing - at the bottom of the bowl until you eat. Salt draws the water out too, so season it in the last minute rather than ahead.".to_string());
    }
    Some("A packed lunch goes soggy because water gets in from somewhere. Keep the wet things - tomato, cucumber, dressing, a drink - in their own tub and stand the leaky ones upright, put a layer of kitchen paper or a lettuce leaf under the bread, and pack the dry things on top. A leaking drink and a damp fork do more damage than a whole morning in the bag.".to_string())
}

/// How long a bake keeps, by what it is.
pub fn _bake_keep_sentence(bakes: &[String]) -> String {
    let said = bakes.join(" ");
    if ["scone", "scones", "biscuit", "biscuits", "cookie", "cookies", "shortbread"].iter().any(|w| said.contains(w)) {
        return "Scones and biscuits are at their best the day they are baked and keep 2 days in a tin; freeze them baked for 2 months and warm them through for 5 minutes at 150C, which brings back most of the crust.".to_string();
    }
    if ["bread", "loaf", "loaves", "sourdough", "bun", "buns", "roll", "rolls", "focaccia", "baguette"].iter().any(|w| said.contains(w)) {
        return "Bread keeps 2 to 3 days in a paper bag or a bread bin, cut side down, and a month in the freezer sliced; the fridge is what makes it stale fastest, so freeze what you will not eat in two days.".to_string();
    }
    if said.contains("cupcake") || said.contains("muffin") {
        return "Cupcakes and muffins keep 2 to 3 days in a tin, best the day they are baked, and they freeze well undecorated for 2 months: defrost them at room temperature in a single layer.".to_string();
    }
    "A cake keeps 3 days in a tin, or freezes for 2 months undecorated; a crumbly or dry one is best eaten in the next day or two, or frozen in slices.".to_string()
}

static BAKE_KEEP_SHAPES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_BAKE_KEEP_SHAPES"));
static BAKED: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_BAKED"));
static BAKE_FOLLOW_SHAPES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_BAKE_FOLLOW_SHAPES"));
static USE_SHAPES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_USE_SHAPES"));
static NEXT_SHAPES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_NEXT_SHAPES"));
/// (fault words, cause, cure, rescue).
static BAKE_FAULT_FAMILIES: LazyLock<Vec<(Vec<String>, String, String, String)>> = LazyLock::new(|| {
    cget("_BAKE_FAULT_FAMILIES")
        .items()
        .iter()
        .map(|r| {
            let it = r.items();
            (it[0].strs(), it[1].str().to_string(), it[2].str().to_string(), it[3].str().to_string())
        })
        .collect()
});

/// "how long will they keep" after a line about cupcakes or muffins.
pub fn _bake_keep(memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = or(&reading.text, &reading.clean).to_lowercase();
    if !BAKE_KEEP_SHAPES.iter().any(|s| line.contains(s.as_str())) {
        return Some(String::new());
    }
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        let said = turn.user.to_lowercase();
        if !BAKED.iter().any(|n| said.contains(n.as_str())) {
            continue;
        }
        if said.contains("cupcake") || said.contains("muffin") {
            return Some(_bake_keep_sentence(&[said]));
        }
        return Some(String::new());
    }
    Some(String::new())
}

/// A follow-up about a bake named in the last lines: why it went wrong, next time, still eat it.
pub fn _bake_follow(memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = or(&reading.text, &reading.clean).to_lowercase();
    if !BAKE_FOLLOW_SHAPES.iter().any(|s| line.contains(s.as_str())) {
        return Some(String::new());
    }
    let turns = &memory.turns;
    let bakes: Vec<String> = turns[turns.len().saturating_sub(3)..]
        .iter()
        .rev()
        .map(|t| t.user.to_lowercase())
        .filter(|said| BAKED.iter().any(|n| said.contains(n.as_str())))
        .collect();
    if bakes.is_empty() {
        return Some(String::new());
    }
    let mut family: Option<&(Vec<String>, String, String, String)> = None;
    for said in std::iter::once(&line).chain(bakes.iter()) {
        family = BAKE_FAULT_FAMILIES.iter().find(|cand| cand.0.iter().any(|w| said.contains(w.as_str())));
        if family.is_some() {
            break;
        }
    }
    let Some((_, cause, cure, rescue)) = family else {
        if USE_SHAPES.iter().chain(BAKE_KEEP_SHAPES.iter()).any(|s| line.contains(s.as_str())) {
            return Some(_bake_keep_sentence(&bakes));
        }
        return Some(String::new());
    };
    if USE_SHAPES.iter().any(|s| line.contains(s.as_str())) {
        return Some(format!("It is still worth eating: {rescue}."));
    }
    if NEXT_SHAPES.iter().any(|s| line.contains(s.as_str())) {
        let first = cause.chars().next().expect("IndexError");
        let cause = if !first.is_uppercase() { upper_first(cause) } else { cause.clone() };
        return Some(format!("Next time, {cure}. {cause}."));
    }
    Some(format!("{cause}. {}.", upper_first(cure)))
}

static GUMMY_BREAD_WORDS: LazyLock<Vec<Re>> = LazyLock::new(|| {
    cstrs("_BREAD_WORDS").iter().map(|w| Re::new(&format!(r"(?<![a-z]){w}(?![a-z])"), 0)).collect()
});

/// A gummy loaf or a gummy cake.
pub fn _gummy(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if GUMMY_BREAD_WORDS.iter().any(|rx| rx.is_match(&clean)) {
        return Some(cstring("_GUMMY_BREAD"));
    }
    Some(cstring("_GUMMY_CAKE"))
}

// ---------------------------------------------------------------------------------------------
// Cooked food kept, leftovers, eating when ill

static COOKED_KEEP_ASK: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("COOKED_KEEP_ASK"));
static COOKING_VERBS: LazyLock<&'static Re> = LazyLock::new(|| cre("COOKING_VERBS"));
static A_BLANK_FILLER: LazyLock<&'static Re> = LazyLock::new(|| cre("_A_BLANK_FILLER"));

/// The line asks how long a cooked thing keeps, not how to cook it.
pub fn _already_cooked_is_the_question(understood: &Understood) -> bool {
    let text = text::squash(&understood.clean.to_lowercase());
    if COOKED_KEEP_ASK.iter().any(|w| text.contains(w.as_str())) {
        return true;
    }
    !COOKING_VERBS.is_match(&text)
}

/// The line names a food, not a blank ("how long does it keep").
pub fn _a_food_and_not_a_blank(understood: &Understood) -> bool {
    !A_BLANK_FILLER.is_match(&understood.clean.to_lowercase())
}

/// The guard `lambda u: _already_cooked_is_the_question(u) and _a_food_and_not_a_blank(u)`.
pub fn _cooked_keep_guard(understood: &Understood) -> bool {
    _already_cooked_is_the_question(understood) && _a_food_and_not_a_blank(understood)
}

static FOODS_SAID: LazyLock<HashSet<String>> = LazyLock::new(|| {
    cstrs("_NAMED_FOODS").iter().chain(cstrs("_STAPLE_NAMES").iter()).map(|w| _fold(&w.to_lowercase())).collect()
});

/// Every food name the tables know, folded.
pub fn _foods_said() -> &'static HashSet<String> {
    &FOODS_SAID
}

static LEFTOVER_IDEAS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("LEFTOVER_IDEAS"));
static LEFTOVER_SOUP: LazyLock<String> = LazyLock::new(|| cstring("LEFTOVER_SOUP"));

/// What to make with a leftover food the line names, or a soup.
pub fn _leftover_meal_reply(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut words: Vec<String> = understood.words.clone();
    if words.is_empty() {
        words = text::split(&understood.clean.to_lowercase());
    }
    let named = named_food(&words, Some(understood));
    if let Some(n) = &named {
        if let Some(idea) = LEFTOVER_IDEAS.get(n) {
            return Some(idea.clone());
        }
        let idea = PANTRY.get(n).map(|(_, idea)| idea.clone()).unwrap_or_default();
        if !idea_is_a_dish(&idea) {
            let said = use_it_up(n);
            if !said.is_empty() {
                return Some(said);
            }
        }
    }
    Some(LEFTOVER_SOUP.clone())
}

static JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("JOINERS"));

/// The words just before a place, back to a joining word ("and", "with").
pub fn _words_just_before(folded: &[String], place: usize, reach: usize) -> Vec<String> {
    let mut window = Vec::new();
    for word in folded[place.saturating_sub(reach)..place].iter().rev() {
        if JOINERS.contains(word) {
            break;
        }
        window.push(word.clone());
    }
    window
}

static PREVENTION_ASK: LazyLock<&'static Re> = LazyLock::new(|| cre("PREVENTION_ASK"));
static SICK_FROM_FOOD: LazyLock<&'static Re> = LazyLock::new(|| cre("_SICK_FROM_FOOD"));
static AWAY_FROM_HOME: LazyLock<HashSet<String>> = LazyLock::new(|| cset("AWAY_FROM_HOME"));

/// What to eat when ill, or the food-safety rules when the line asks how not to get ill.
pub fn _eat_when_ill(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = understood.clean.to_lowercase();
    if PREVENTION_ASK.is_match(&text) || text.contains("contamination") || SICK_FROM_FOOD.is_match(&text) {
        if !ingredients(understood).is_empty() || _plain_words(&text).iter().any(|w| AWAY_FROM_HOME.contains(w)) {
            return None;
        }
        return Some(cstring("FOOD_SAFETY_RULES"));
    }
    Some("Keep it plain and small: toast, crackers, rice, a banana, clear soup or plain yoghurt, and sip more than you feel like - little and often beats one big glass. Go back to proper meals over a day or two, and get advice the same day if you cannot keep any drink down or you are passing much less urine than usual.".to_string())
}

// ---------------------------------------------------------------------------------------------
// Frozen, tinned and dried forms of a food

static FORMS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    cget("FORMS").items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].strs())).collect()
});
static ALIASES_BY_LENGTH: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| cnames_rows("_ALIASES_BY_LENGTH"));
static ALIAS_TEXT: LazyLock<HashMap<Vec<String>, String>> = LazyLock::new(|| {
    cget("_ALIAS_TEXT").pairs().iter().filter_map(|(k, v)| Some((k.strs(), v.as_str()?.to_string()))).collect()
});
static SWAP_ASK: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_SWAP_ASK"));
static FRUIT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("FRUIT_WORDS"));
static FORM_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("FORM_FOODS"));
static OUT_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("OUT_ASKS"));
static OUT_OF_FORM: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("OUT_OF_FORM"));
static FORM_ADVICE: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("FORM_ADVICE"));

/// The form ("frozen", "tinned", ...) said of the food at `start`, in it or just before it.
pub fn _form_word(words: &[String], start: usize, span: usize) -> String {
    let end = (start + span).min(words.len());
    for word in &words[start.min(words.len())..end] {
        for (form, names) in FORMS.iter() {
            if names.contains(word) {
                return form.clone();
            }
        }
    }
    for word in &words[start.saturating_sub(2)..start.min(words.len())] {
        for (form, names) in FORMS.iter() {
            if names.contains(word) {
                return form.clone();
            }
        }
    }
    String::new()
}

/// `str.format(food=...)` on a template that names only `{food}`.
fn with_food(template: &str, food: &str) -> String {
    template.replace("{food}", food)
}

/// Using a food in another form: frozen spinach, tinned tomatoes, out of fresh.
pub fn _form_swap(_memory: &mut Memory, understood: &Understood, text: Option<&str>) -> Option<String> {
    let mut folded: Vec<String> = words_or_clean(understood).iter().map(|w| _fold(w)).collect();
    if let Some(t) = text.filter(|t| !t.is_empty()) {
        folded.extend(text::split(&t.to_lowercase()).iter().map(|w| _fold(w)));
    }
    let said = folded.join(" ");
    if !SWAP_ASK.iter().any(|a| said.contains(a.as_str())) {
        return None;
    }
    let mut used = vec![false; folded.len()];
    let mut first: Option<(usize, usize, String, String, String)> = None;
    let mut seen: Vec<String> = Vec::new();
    let mut head: Vec<String> = Vec::new();
    for (parts, name) in ALIASES_BY_LENGTH.iter() {
        let span = parts.len();
        if folded.len() < span {
            continue;
        }
        for start in 0..=(folded.len() - span) {
            if used[start..start + span].iter().any(|u| *u) {
                continue;
            }
            if (0..span).all(|o| folded[start + o] == parts[o]) {
                for o in 0..span {
                    used[start + o] = true;
                }
                seen.push(name.clone());
                let form = _form_word(&folded, start, span);
                if !form.is_empty() && first.is_none() {
                    let spoken = ALIAS_TEXT.get(parts).cloned().unwrap_or_else(|| parts.join(" "));
                    head = FORMS.iter().find(|(one, _)| *one == form).map(|(_, names)| names.clone()).expect("StopIteration");
                    first = Some((start, span, form, spoken, name.clone()));
                }
                break;
            }
        }
    }
    let Some((start, span, form, mut spoken, food)) = first else {
        for (index, word) in folded.iter().enumerate() {
            if FRUIT_WORDS.contains(word) && _form_word(&folded, index, 1) == "frozen" {
                return Some(cstring("FROZEN_FRUIT"));
            }
        }
        return None;
    };
    if !FORM_FOODS.contains(&food) {
        return None;
    }
    if seen.iter().any(|other| *other != food) {
        return None;
    }
    let spoken_words = text::split(&spoken);
    let first_word = spoken_words.first().map(|w| w.to_lowercase()).unwrap_or_default();
    if !first_word.is_empty() && head.iter().any(|w| *w == first_word) {
        let rest = spoken_words[1..].join(" ");
        if !rest.is_empty() {
            spoken = rest;
        }
    }
    let before = folded[..start + span].join(" ");
    if OUT_ASKS.iter().any(|a| before.contains(a.as_str())) {
        return Some(with_food(&OUT_OF_FORM[&form], &spoken));
    }
    if form == "frozen" {
        if MEAT.contains(&food) {
            return Some(with_food(&cstring("FROZEN_MEAT"), &spoken));
        }
        if FRUIT_WORDS.contains(&food) {
            return Some(cstring("FROZEN_FRUIT"));
        }
    }
    Some(with_food(&FORM_ADVICE[&form], &spoken))
}

// ---------------------------------------------------------------------------------------------
// The ingredient reader

static REFUSAL_REACH: LazyLock<usize> = LazyLock::new(|| cget("REFUSAL_REACH").as_int().unwrap_or(0) as usize);
static NEGATION_REACH: LazyLock<usize> = LazyLock::new(|| cget("NEGATION_REACH").as_int().unwrap_or(0) as usize);
static COOKED_REACH: LazyLock<usize> = LazyLock::new(|| cget("COOKED_REACH").as_int().unwrap_or(0) as usize);
static CLAUSE_BREAK: LazyLock<HashSet<String>> = LazyLock::new(|| cset("CLAUSE_BREAK"));
static RULING_OUT_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("RULING_OUT_PHRASES"));
static EVENT_HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("EVENT_HEADS"));
static NEGATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("NEGATIONS"));
static SOFTENED: LazyLock<HashSet<String>> = LazyLock::new(|| cset("SOFTENED"));
static REFUSAL_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("REFUSAL_JOINERS"));

/// Whether a phrase that rules a food out ("no", "without", "allergic to") comes just before it.
pub fn _refused_before(folded: &[String], place: usize) -> bool {
    if place == 0 {
        return false;
    }
    let mut window: Vec<&String> = Vec::new();
    for word in folded[place.saturating_sub(*REFUSAL_REACH)..place].iter().rev() {
        if CLAUSE_BREAK.contains(word) {
            break;
        }
        window.insert(0, word);
    }
    let plain = format!(" {} ", window.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(" "));
    RULING_OUT_PHRASES.iter().any(|cue| plain.contains(&format!(" {cue} ")))
}

/// Whether the word before a food makes it an event ("pancake day", "burns night").
pub fn _renamed_by_an_event(folded: &[String], place: usize) -> bool {
    if place == 0 {
        return false;
    }
    EVENT_HEADS.contains(&folded[place - 1])
}

/// The ingredients a reading names, and the ones it rules out: (named, negated).
pub fn _named_ingredients(understood: &Understood) -> (Vec<String>, Vec<String>) {
    let mut folded: Vec<String> = words_or_clean(understood).iter().map(|w| _fold(w)).collect();
    let repaired: Vec<String> = text::split(&understood.plain).iter().map(|w| _fold(w)).collect();
    if repaired.len() == folded.len() && repaired != folded {
        folded = repaired;
    }
    _work_out_ingredients(&folded)
}

/// The ingredient reader itself: the names in the words, longest first, and which are ruled out.
pub fn _work_out_ingredients(folded: &[String]) -> (Vec<String>, Vec<String>) {
    let mut used = vec![false; folded.len()];
    let present: HashSet<&String> = folded.iter().collect();
    let mut hits: Vec<(usize, String, usize)> = Vec::new();
    for (parts, name) in ALIASES_BY_LENGTH.iter() {
        if !present.contains(&parts[0]) {
            continue;
        }
        let span = parts.len();
        if folded.len() < span {
            continue;
        }
        for start in 0..=(folded.len() - span) {
            if used[start..start + span].iter().any(|u| *u) {
                continue;
            }
            if (0..span).all(|o| folded[start + o] == parts[o]) {
                for o in 0..span {
                    used[start + o] = true;
                }
                hits.push((start, name.clone(), span));
                break;
            }
        }
    }
    hits.sort();
    let (mut named, mut negated): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    let mut refused_ends: Vec<usize> = Vec::new();
    for (place, name, span) in hits {
        if named.contains(&name) {
            continue;
        }
        if _renamed_by_an_event(folded, place) {
            continue;
        }
        named.push(name.clone());
        let before = &folded[place.saturating_sub(*NEGATION_REACH)..place];
        let mut ruled = _refused_before(folded, place)
            || (before.iter().any(|w| NEGATIONS.contains(w)) && !before.iter().any(|w| SOFTENED.contains(w)));
        if !ruled && _on_a_list_already_ruled_out(folded, place, &refused_ends) {
            ruled = true;
        }
        if ruled {
            negated.push(name);
            refused_ends.push(place + span);
        }
    }
    (named, negated)
}

/// Whether a food comes on a list after one already ruled out ("no nuts or eggs").
pub fn _on_a_list_already_ruled_out(folded: &[String], place: usize, refused_ends: &[usize]) -> bool {
    for &end in refused_ends {
        if end >= place {
            continue;
        }
        let between = &folded[end..place];
        if between.iter().all(|w| REFUSAL_JOINERS.contains(w)) {
            return true;
        }
    }
    false
}

static DIETS: LazyLock<Vec<(Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    cget("DIETS").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].strs())).collect()
});
static FOOD_CLASSES: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| cstrs_map("FOOD_CLASSES").into_iter().collect());

/// The foods a diet the line names rules out ("vegan" rules out the meat, fish, dairy and egg).
pub fn diet_exclusions(understood: &Understood) -> Vec<String> {
    let clean = understood.clean.to_lowercase();
    let source = if understood.words.is_empty() { text::split(&clean) } else { understood.words.clone() };
    let folded: Vec<String> = source.iter().map(|w| _fold(w)).collect();
    let mut classes: HashSet<String> = HashSet::new();
    for (cues, ruled) in DIETS.iter() {
        for cue in cues {
            let hit = if cue.contains(' ') { clean.contains(cue.as_str()) } else { folded.contains(cue) };
            if hit {
                classes.extend(ruled.iter().cloned());
                break;
            }
        }
    }
    if classes.is_empty() {
        return Vec::new();
    }
    FOOD_CLASSES.iter().filter(|(_, kinds)| kinds.iter().any(|k| classes.contains(k))).map(|(n, _)| n.clone()).collect()
}

static FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| cset("FILLER"));
static DIET_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_DIET_WORDS"));
static VAGUE_KEYS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_VAGUE_KEYS"));
static VAGUE: LazyLock<HashSet<String>> = LazyLock::new(|| cset("VAGUE"));

/// Whether the line is nothing but a list of foods ("chicken, rice and peppers").
pub fn _is_food_list(folded: &[String], _clean: &str) -> bool {
    let mut named = false;
    for word in folded {
        if word.is_empty() || FILLER.contains(word) || DIET_WORDS.contains(word) || NEGATIONS.contains(word) {
            continue;
        }
        if let Some(name) = INGREDIENT_WORDS.get(word) {
            if !VAGUE_KEYS.contains(name) {
                named = true;
            }
            continue;
        }
        return false;
    }
    named
}

static NOT_THIS_FOOD: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| cstrs_map("NOT_THIS_FOOD"));

/// The ingredients a reading names and does not rule out.
pub fn ingredients(understood: &Understood) -> Vec<String> {
    let (named, negated) = _named_ingredients(understood);
    let kept: Vec<String> = named.into_iter().filter(|n| !negated.contains(n)).collect();
    let kept: Vec<String> = kept
        .iter()
        .filter(|name| !kept.iter().any(|other| *name != other && text::split(other).contains(name)))
        .cloned()
        .collect();
    let clean = understood.clean.to_lowercase();
    kept.into_iter()
        .filter(|name| !NOT_THIS_FOOD.get(name).is_some_and(|ps| ps.iter().any(|p| clean.contains(p.as_str()))))
        .collect()
}

/// The ingredients a reading names and rules out.
pub fn negated_ingredients(understood: &Understood) -> Vec<String> {
    let (named, negated) = _named_ingredients(understood);
    negated.into_iter().filter(|n| named.contains(n)).collect()
}

/// Whether a text names one of the ingredients.
pub fn _mentions(text: &str, names: &[String]) -> bool {
    if names.is_empty() {
        return false;
    }
    for word in text::split(&text.to_lowercase()) {
        let word = _fold(&word);
        let mut name = INGREDIENT_WORDS.get(&word);
        if name.is_none() {
            let base = _fold(word.trim_end_matches([',', '.']));
            for ending in ["", "s", "es"] {
                name = INGREDIENT_WORDS.get(&format!("{base}{ending}"));
                if name.is_some() {
                    break;
                }
            }
        }
        if let Some(n) = name {
            if names.contains(n) {
                return true;
            }
        }
    }
    false
}

fn set_vec(set: &HashSet<String>) -> Vec<String> {
    set.iter().cloned().collect()
}

static COOKED_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("COOKED_WORDS"));

/// The ingredients the line says are already cooked ("leftover roast chicken").
pub fn cooked_ingredients(understood: &Understood) -> Vec<String> {
    let folded: Vec<String> = words_or_clean(understood).iter().map(|w| _fold(w)).collect();
    let mut ready: Vec<String> = Vec::new();
    for (place, word) in folded.iter().enumerate() {
        let Some(name) = INGREDIENT_WORDS.get(word) else { continue };
        if name.is_empty() || ready.contains(name) {
            continue;
        }
        if !_words_just_before(&folded, place, *COOKED_REACH).iter().any(|w| COOKED_WORDS.contains(w)) {
            continue;
        }
        let before = &folded[place.saturating_sub(*NEGATION_REACH)..place];
        if before.iter().any(|w| NEGATIONS.contains(w)) {
            continue;
        }
        ready.push(name.clone());
    }
    ready
}

/// The words of a text, lower-case.
pub fn _plain_words(text: &str) -> Vec<String> {
    WORDS_AZ.findall_str(&text.to_lowercase())
}

static VEG_DINNER: LazyLock<String> = LazyLock::new(|| cstring("_VEG_DINNER"));
static VEG_DINNER_CHEAP: LazyLock<String> = LazyLock::new(|| cstring("_VEG_DINNER_CHEAP"));

/// The vegetarian dinner, unless it needs something ruled out.
pub fn _veg_dinner(missing: &HashSet<String>, cheap: bool) -> String {
    if _mentions(&VEG_DINNER, &set_vec(missing)) {
        return _any_meal(missing);
    }
    format!("{}{}", *VEG_DINNER, if cheap { VEG_DINNER_CHEAP.as_str() } else { "" })
}

static VEG_ELSEWHERE: LazyLock<&'static Re> = LazyLock::new(|| cre("_VEG_ELSEWHERE"));
static SELF_VEG: LazyLock<&'static Re> = LazyLock::new(|| cre("_SELF_VEG"));
static VEG_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_VEG_WORDS"));
static VEG_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_VEG_PHRASES"));
static VEG_GUESTS: LazyLock<&'static Re> = LazyLock::new(|| cre("_VEG_GUESTS"));

/// The diet the line says the cook is on: "vegan", "vegetarian" or "".
pub fn _diet(folded: &[String], clean: &str) -> String {
    if VEG_ELSEWHERE.is_match(clean) && !SELF_VEG.is_match(clean) {
        return String::new();
    }
    if folded.iter().any(|w| w == "vegan" || w == "vegans") {
        return "vegan".to_string();
    }
    if folded.iter().any(|w| VEG_WORDS.contains(w)) || VEG_PHRASES.iter().any(|p| clean.contains(p.as_str())) || VEG_GUESTS.is_match(clean) {
        return "vegetarian".to_string();
    }
    String::new()
}

/// The person's last few lines.
pub fn _recent_user_turns(memory: Option<&Memory>, count: usize) -> Vec<String> {
    let Some(memory) = memory else { return Vec::new() };
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(count)..].iter().map(|t| t.user.clone()).collect()
}

/// The diet an earlier line said, nearest first.
pub fn _diet_from_turns(earlier: &[String]) -> String {
    for said in earlier.iter().rev() {
        let low = said.to_lowercase();
        let found = _diet(&WORDS_AZ09.findall_str(&low), &low);
        if !found.is_empty() {
            return found;
        }
    }
    String::new()
}

static TIRED_OF: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:tired|sick|bored)\s+of\b|\bfed\s+up\s+(?:of|with)\b", 0));
static REFUSAL_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_REFUSAL_CUES"));
static BARE_NO_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_BARE_NO_CUES"));
static SHORTAGE_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| cset("SHORTAGE_AFTER"));
static REFUSAL_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_REFUSAL_FILLER"));
static REFUSAL_JOINERS_: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_REFUSAL_JOINERS"));

/// The ingredients a line refuses ("no mushrooms", "sick of chicken"); with `families`, a
/// family word ("dairy") counts as all of its foods.
pub fn _refused_foods(text: &str, families: bool) -> HashSet<String> {
    let said = _fold(text);
    let said = TIRED_OF.sub(" no ", &said);
    let mut refused = HashSet::new();
    let words = WORDS_AZ.findall_str(&said);
    for (start, word) in words.iter().enumerate() {
        if !REFUSAL_CUES.contains(word) {
            continue;
        }
        let mut index = start + 1;
        while index < words.len() {
            let here = &words[index];
            let name = INGREDIENT_WORDS.get(here).filter(|n| !n.is_empty()).or_else(|| INGREDIENT_WORDS.get(&_fold(here)));
            if let Some(name) = name.filter(|n| !n.is_empty()) {
                let after = &words[(index + 1).min(words.len())..(index + 3).min(words.len())];
                if BARE_NO_CUES.contains(word) && after.iter().any(|p| SHORTAGE_AFTER.contains(p)) {
                    break;
                }
                refused.insert(name.clone());
                index += 1;
                continue;
            }
            if FAMILIES.contains_key(here) {
                if families {
                    let foods = if here == "lactose" { &*DAIRY } else { &FAMILIES[here] };
                    refused.extend(foods.iter().cloned());
                }
                index += 1;
                continue;
            }
            if REFUSAL_FILLER.contains(here) || REFUSAL_JOINERS_.contains(here) {
                index += 1;
                continue;
            }
            break;
        }
    }
    refused
}

/// The ingredients the reading's line refuses.
pub fn refused_foods(understood: &Understood) -> HashSet<String> {
    _refused_foods(or(&understood.clean, &understood.text), false)
}

static STAND_IN_ASK: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_STAND_IN_ASK"));
static STAND_IN_USE: LazyLock<&'static Re> = LazyLock::new(|| cre("_STAND_IN_USE"));

/// Whether the line asks what stands in for a food ("what can i use instead of eggs").
pub fn asks_for_a_stand_in(understood: &Understood) -> bool {
    let said = super::d2dup_cookB::_clean(understood);
    if STAND_IN_ASK.iter().any(|p| said.contains(p.as_str())) {
        return true;
    }
    STAND_IN_USE.is_match(&said)
}

static SHORTAGE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("SHORTAGE_WORDS"));
static NOT_A_DINNER_LIMIT: LazyLock<HashSet<String>> = LazyLock::new(|| cset("NOT_A_DINNER_LIMIT"));
static AND_NO: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("AND_NO"));
static DINNER_NAMED: LazyLock<HashSet<String>> = LazyLock::new(|| cset("DINNER_NAMED"));
static MEAL_ASK_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("MEAL_ASK_PHRASES"));
static DINNER_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("DINNER_ASK_WORDS"));
static LUNCH_AND_BREAKFAST: LazyLock<HashSet<String>> = LazyLock::new(|| cset("LUNCH_AND_BREAKFAST"));

/// The keys Python's memory dict has, which `a_dinner_without` hands to `dish` as its `avoid`
/// (the call passes memory in the place of `avoid`). An approximation: the keys this port keeps.
fn memory_keys(memory: Option<&Memory>) -> Vec<String> {
    let Some(m) = memory else { return Vec::new() };
    let mut keys = Vec::new();
    if !m.turns.is_empty() {
        keys.push("turns".to_string());
    }
    if m.understood.is_some() {
        keys.push("understood".to_string());
    }
    if m.intent_wants.is_some() {
        keys.push("intent_wants".to_string());
    }
    if m.asked_now.is_some() {
        keys.push("asked_now".to_string());
    }
    keys.extend(m.extra.keys().cloned());
    keys
}

/// A dinner for a line that refuses some foods and asks what to cook.
pub fn a_dinner_without(understood: &Understood, memory: Option<&Memory>) -> Option<String> {
    let said = super::d2dup_cookB::_clean(understood);
    if asks_for_a_stand_in(understood) {
        return None;
    }
    let refused = _refused_foods(&said, false);
    if refused.is_empty() {
        return None;
    }
    let empty = Memory::default();
    let memory = memory.unwrap_or(&empty);
    if SHORTAGE_WORDS.iter().any(|p| said.contains(p.as_str())) {
        return None;
    }
    if refused.is_subset(&NOT_A_DINNER_LIMIT) {
        return None;
    }
    let words: HashSet<String> = super::d2dup_cookB::_said_words(understood).into_iter().collect();
    if AND_NO.iter().any(|p| said.contains(p.as_str())) && !meets(&words, &DINNER_NAMED) {
        return None;
    }
    let mut asking = MEAL_ASK_PHRASES.iter().any(|p| said.contains(p.as_str()));
    if !asking {
        asking = !understood.asks.is_empty() && meets(&words, &DINNER_ASK_WORDS);
    }
    if !asking {
        return None;
    }
    if text::split(&said).iter().any(|w| LUNCH_AND_BREAKFAST.contains(w)) {
        return None;
    }
    let avoid = memory_keys(Some(memory));
    let cooked = dish(understood, &avoid, None)
        .filter(|c| !c.is_empty())
        .or_else(|| meal(understood, MealArgs { memory: Some(memory), ..Default::default() }));
    let cooked = cooked.filter(|c| !text::strip(c).is_empty())?;
    if unusable(&cooked, understood) { None } else { Some(cooked) }
}

/// One DISHES row (or a copy cut down for missing trimmings, with what it is without).
#[derive(Clone, Debug, Default)]
pub struct Dish {
    pub id: String,
    pub uses: Vec<String>,
    pub cooked: Vec<String>,
    pub text: String,
    pub without: Vec<String>,
}

static DISHES: LazyLock<Vec<Dish>> = LazyLock::new(|| {
    cget("DISHES")
        .items()
        .iter()
        .map(|d| Dish {
            id: d.get("id").map(|v| v.str().to_string()).unwrap_or_default(),
            uses: d.get("uses").map(|v| v.strs()).unwrap_or_default(),
            cooked: d.get("cooked").map(|v| v.strs()).unwrap_or_default(),
            text: d.get("text").map(|v| v.str().to_string()).unwrap_or_default(),
            without: Vec::new(),
        })
        .collect()
});

/// The dishes that need none of the banned foods.
pub fn _dishes_without(banned: &HashSet<String>) -> Vec<Dish> {
    let banned_v = set_vec(banned);
    DISHES
        .iter()
        .filter(|d| !d.uses.is_empty() && !d.uses.iter().any(|n| banned.contains(n)))
        .filter(|d| !_mentions(&d.text, &banned_v))
        .cloned()
        .collect()
}

/// The dish that uses the most of the foods named (two at least), or one cut down to leave a
/// missing trimming out.
pub fn _best_dish(found: &[String], ready: &[String], missing: &[String], avoid: &[String], diet: &str) -> Option<Dish> {
    let (mut best, mut best_uses): (Option<&Dish>, usize) = (None, 1);
    let mut trimmed: Option<Dish> = None;
    let avoid_folded = _fold(&avoid.join(" "));
    for candidate in DISHES.iter() {
        let uses = &candidate.uses;
        if !diet.is_empty() && uses.iter().any(|n| MEAT.contains(n)) {
            continue;
        }
        if !uses.iter().all(|n| found.contains(n)) {
            continue;
        }
        if avoid.contains(&candidate.text) {
            continue;
        }
        if !missing.is_empty() && _mentions(&candidate.text, missing) {
            if uses.iter().any(|n| missing.contains(n)) {
                continue;
            }
            let cut = _without_the_trimmings(&candidate.text, missing);
            if !cut.is_empty() && trimmed.is_none() && uses.len() > 1 {
                let mut without: Vec<String> =
                    missing.iter().filter(|n| _mentions(&candidate.text, std::slice::from_ref(*n))).cloned().collect();
                without.sort();
                trimmed = Some(Dish { text: cut, without, ..candidate.clone() });
            }
            continue;
        }
        if !avoid.is_empty() && candidate.id.split('-').all(|w| avoid_folded.contains(w)) {
            continue;
        }
        if uses.iter().any(|n| missing.contains(n)) {
            continue;
        }
        let keeps = &candidate.cooked;
        if uses.iter().any(|n| ready.contains(n) && !keeps.contains(n)) {
            continue;
        }
        if uses.len() > best_uses {
            best = Some(candidate);
            best_uses = uses.len();
        }
    }
    best.cloned().or(trimmed)
}

/// A cut-down dish, saying what it is without.
pub fn _served_without(dish: &Dish) -> String {
    let left_out = dish.without.join(" and ");
    let text = &dish.text;
    let (name, colon, method) = text::partition(text, ": ");
    if !colon.is_empty() && !name.is_empty() && !_mentions(name, &dish.without) {
        return format!("{name}, no {left_out}: {method}");
    }
    let mut chars = text.chars();
    let lowered = match chars.next() {
        Some(f) => f.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    format!("Without the {left_out}: {lowered}")
}

static SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| cre("SENTENCE"));

/// A dish's method with its last sentences cut while they need a missing food ("" if the first
/// one does).
pub fn _without_the_trimmings(text: &str, missing: &[String]) -> String {
    let mut parts: Vec<String> =
        SENTENCE.split(text::strip(text)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    while parts.len() > 1 && _mentions(&parts[parts.len() - 1], missing) {
        parts.pop();
    }
    if parts.is_empty() || _mentions(&parts[0], missing) {
        return String::new();
    }
    parts.join(" ")
}

/// A dish made of the foods the line names, or None (a bake is not a dish).
pub fn dish(understood: &Understood, avoid: &[String], memory: Option<&Memory>) -> Option<String> {
    if _names_a_bake(understood) {
        return None;
    }
    meal(understood, MealArgs { dishes_only: true, avoid: avoid.to_vec(), memory, ..Default::default() })
}

/// `meals.reads_as_news(understood, clean)`.
pub fn _reads_as_news(understood: &Understood, clean: &str) -> bool {
    super::d2dup_cookB::reads_as_news(understood, clean)
}

static ANY_FOOD_TEXT: LazyLock<String> = LazyLock::new(|| {
    let mut names: Vec<String> =
        PANTRY.values().flat_map(|(aliases, _)| aliases.iter().map(|a| _fold(a))).collect::<HashSet<_>>().into_iter().collect();
    names.sort();
    names.sort_by_key(|n| std::cmp::Reverse(text::len(n)));
    names.iter().filter(|n| !n.is_empty()).map(|n| escape(n)).collect::<Vec<_>>().join("|")
});
static CHOICE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CHOICE_WORDS"));
static A_CHOICE: LazyLock<Option<Re>> = LazyLock::new(|| {
    let foods = &*ANY_FOOD_TEXT;
    if foods.is_empty() {
        return None;
    }
    let other = format!("{foods}|{}", cstring("_COOKING_MEDIUM"));
    Some(Re::new(&format!(r"\b(?:{foods})\s+or\s+(?:{other})\b"), 0))
});

/// Every pantry food name as one regex alternation.
pub fn _any_food_text() -> &'static str {
    &ANY_FOOD_TEXT
}

/// Whether the line asks which of two foods to have ("chicken or fish tonight"): a choice, not
/// a meal to cook.
pub fn _a_choice_of_foods(clean: &str) -> bool {
    let low = _fold(clean);
    if !low.contains(" or ") {
        return false;
    }
    if !CHOICE_WORDS.iter().any(|w| low.contains(w.as_str())) {
        return false;
    }
    match &*A_CHOICE {
        None => false,
        Some(rx) => rx.is_match(&low),
    }
}

/// `meal()`'s keyword arguments.
#[derive(Default)]
pub struct MealArgs<'a> {
    pub dishes_only: bool,
    pub text: Option<&'a str>,
    pub banned: Option<HashSet<String>>,
    pub diet: Option<String>,
    pub memory: Option<&'a Memory>,
    pub avoid: Vec<String>,
    pub pointed_back: bool,
}

static USE_UP_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_USE_UP_WORDS"));
static CROWD_FOOD_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = cstrs("_CROWD_FOOD");
    v.extend(cstrs("_CROWD_VERBS"));
    v
});
static EMPTY_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("EMPTY_PHRASES"));
static LEFT_POINTER_USE_UP: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = cstrs("LEFT_POINTER");
    v.extend(cstrs("USE_UP_ASKS"));
    v
});
static LEFT_POINTER: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("LEFT_POINTER"));
static USE_UP_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("USE_UP_ASKS"));
static NOT_MEAL_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("NOT_MEAL_ASKS"));
static PROBLEM_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("PROBLEM_WORDS"));
static SUBSTITUTE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("SUBSTITUTE_WORDS"));
static WHAT_IS_IT: LazyLock<&'static Re> = LazyLock::new(|| cre("_WHAT_IS_IT"));
static SUBSTITUTE_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("SUBSTITUTE_PHRASES"));
static PROBLEM_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("PROBLEM_PHRASES"));
static RESTRICTED_EATING: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("RESTRICTED_EATING"));
static MEAL_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| cset("MEAL_CUES"));
static ILLNESS_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("ILLNESS_PHRASES"));
static ACTIVITY_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("ACTIVITY_PHRASES"));
static MAKING_IT_YOURSELF_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("MAKING_IT_YOURSELF_PHRASES"));
static MEAL_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("MEAL_WORDS"));
static PET_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("PET_WORDS"));
static EMPTY_KITCHEN: LazyLock<String> = LazyLock::new(|| cstring("EMPTY_KITCHEN"));
static MEAL_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("MEAL_PHRASES"));
static BOUGHT_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("BOUGHT_PHRASES"));
static FOOD_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| cset("FOOD_CUES"));
static CHEAP_PLAN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("CHEAP_PLAN_WORDS"));
static MEAT_SWAPS: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("MEAT_SWAPS"));
static LEFTOVER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("LEFTOVER_WORDS"));
static LEFTOVER_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("LEFTOVER_PHRASES"));
static PLANT_MILKS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("PLANT_MILKS"));
static WITHOUT: LazyLock<IndexMap<String, String>> = LazyLock::new(|| cstr_map("WITHOUT"));

fn any_phrase(text: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|p| text.contains(p.as_str()))
}

/// A meal from the foods the line names, or for the crowd it is cooking for, or the empty
/// kitchen; None when the line is not asking what to cook with what it has.
pub fn meal(understood: &Understood, args: MealArgs) -> Option<String> {
    let MealArgs { dishes_only, text, banned, diet, memory, avoid, pointed_back } = args;
    if _names_a_bake(understood) {
        return None;
    }
    if says_it_has_gone_off(&understood.clean) {
        return None;
    }
    if names_a_dish(understood).is_some() {
        let said = understood.clean.to_lowercase();
        if !any_phrase(&said, &USE_UP_WORDS) {
            return None;
        }
    }
    if choosing(understood) {
        return None;
    }
    let shared = std::iter::once(understood.clean.clone()).chain(understood.words.iter().cloned()).collect::<Vec<_>>().join(" ");
    if _feeding_ask(&shared) {
        return None;
    }
    if _pet_not_dinner(&shared) || _reads_as_a_pet(&shared) {
        return _pet_or_meat(&shared);
    }
    if _a_brew_not_a_dinner(&shared) {
        return None;
    }
    if _a_plant_not_a_dinner(&shared) {
        return None;
    }
    if ["empty", "greeting", "thanks"].contains(&understood.kind.as_str()) {
        return None;
    }
    let words = &understood.words;
    let folded: Vec<String> = words.iter().map(|w| _fold(w)).collect();
    let earlier = _recent_user_turns(memory, 4);
    let clean_now = understood.clean.to_lowercase();
    let mut whole =
        std::iter::once(clean_now.clone()).chain(earlier.iter().map(|s| s.to_lowercase())).collect::<Vec<_>>().join(" ");
    let mut crowd = _crowd_ask(&clean_now);
    if !crowd && !earlier.is_empty() && any_phrase(&clean_now, &CROWD_FOOD_VERBS) {
        crowd = _crowd_ask(&whole);
    }
    let mut banned: HashSet<String> = match banned {
        Some(b) => b,
        None => {
            let mut parts = vec![understood.clean.clone(), words.join(" ")];
            parts.extend(earlier.iter().cloned());
            _excluded(&parts.join(" "))
        }
    };
    let mut parts = vec![understood.clean.clone()];
    parts.extend(earlier.iter().cloned());
    let refused = _refused_foods(&parts.join(" "), false);
    if refused.iter().any(|r| !banned.contains(r)) {
        banned.extend(refused);
    }
    if let Some(text) = text {
        let mut lower = text.to_lowercase();
        let typed = understood.text.to_lowercase();
        let repaired = understood.plain.to_lowercase();
        if !repaired.is_empty() && repaired != typed && lower == typed {
            lower = repaired;
        }
        let own: Vec<String> = text::split(&lower).iter().map(|w| _fold(w)).collect();
        if !own.iter().any(|w| INGREDIENT_WORDS.contains_key(w))
            && !any_phrase(&lower, &EMPTY_PHRASES)
            && !any_phrase(&lower, &EMPTY_ASK_PHRASES)
            && !any_phrase(&lower, &LEFT_POINTER_USE_UP)
            && !crowd
        {
            return None;
        }
    }
    let asks: HashSet<String> = understood.asks.iter().cloned().collect();
    let not_meal: HashSet<&String> = asks.iter().filter(|a| NOT_MEAL_ASKS.contains(*a)).collect();
    if !not_meal.is_empty()
        && !_crowd_ask(&understood.clean)
        && !(not_meal.len() == 1
            && not_meal.contains(&"how".to_string())
            && any_phrase(&understood.clean.to_lowercase(), &USE_UP_ASKS))
    {
        return None;
    }
    if folded.iter().any(|w| PROBLEM_WORDS.contains(w)) {
        return None;
    }
    if folded.iter().any(|w| SUBSTITUTE_WORDS.contains(w)) {
        return None;
    }
    let clean = understood.clean.to_lowercase();
    if WHAT_IS_IT.is_match(&clean) {
        return None;
    }
    if VEG_GUESTS.is_match(&clean) {
        banned.extend(MEAT.iter().cloned());
    }
    if any_phrase(&clean, &SUBSTITUTE_PHRASES) {
        return None;
    }
    if any_phrase(&clean, &PROBLEM_PHRASES) {
        return None;
    }
    if any_phrase(&clean, &RESTRICTED_EATING)
        && !(folded.iter().any(|w| MEAL_CUES.contains(w)) && !_refused_foods(&clean, false).is_empty())
    {
        return None;
    }
    if any_phrase(&clean, &ILLNESS_PHRASES) {
        return None;
    }
    if any_phrase(&clean, &ACTIVITY_PHRASES) {
        return None;
    }
    if any_phrase(&clean, &MAKING_IT_YOURSELF_PHRASES) {
        return None;
    }
    let mut empty_kitchen = any_phrase(&clean, &EMPTY_PHRASES);
    if !empty_kitchen && any_phrase(&clean, &EMPTY_ASK_PHRASES) {
        empty_kitchen = folded.iter().any(|w| MEAL_WORDS.contains(w));
    }
    if empty_kitchen && !folded.iter().any(|w| PET_WORDS.contains(w)) {
        return Some(EMPTY_KITCHEN.clone());
    }
    let cue = folded.iter().any(|w| MEAL_CUES.contains(w));
    let mut asked = cue || any_phrase(&clean, &MEAL_PHRASES) || crowd;
    let bought = any_phrase(&clean, &BOUGHT_PHRASES);
    asked = asked || bought;
    let asks_what_to_cook = cue || any_phrase(&clean, &MEAL_ASK_PHRASES);
    let named = ingredients(understood);
    let topic = understood.topic.as_deref();
    if !matches!(topic, Some("cooking") | Some("food-and-drink"))
        && !folded.iter().any(|w| FOOD_CUES.contains(w))
        && !(!named.is_empty() && (asks_what_to_cook || bought))
        && !crowd
        && !_is_food_list(&folded, &clean)
    {
        return None;
    }
    if !(asked || _is_food_list(&folded, &clean)) {
        return None;
    }
    let mut found: Vec<String> =
        named.iter().filter(|n| !VAGUE_KEYS.contains(*n) && !VAGUE.contains(*n)).cloned().collect();
    if found.is_empty() {
        found = named.clone();
    }
    let mut missing: Vec<String> = negated_ingredients(understood);
    let said_no = missing.clone();
    for name in diet_exclusions(understood) {
        if found.contains(&name) {
            found.retain(|o| *o != name);
        }
        if !missing.contains(&name) {
            missing.push(name);
        }
    }
    if !banned.is_empty() {
        let refused_here: Vec<String> = found
            .iter()
            .filter(|n| banned.contains(*n) || text::split(n).iter().any(|p| banned.contains(p)))
            .cloned()
            .collect();
        missing.extend(refused_here.iter().cloned());
        found.retain(|n| !refused_here.contains(n));
    }
    let diet: String = match diet.filter(|d| !d.is_empty()) {
        Some(d) => d,
        None => {
            let d = _diet(&folded, &clean);
            if d.is_empty() { _diet_from_turns(&earlier) } else { d }
        }
    };
    if found.is_empty() {
        if any_phrase(&clean, &EMPTY_ASK_PHRASES) {
            return Some(EMPTY_KITCHEN.clone());
        }
        if crowd {
            return Some(_crowd_meal(&banned, _occasion(&whole), _crowd(&whole)));
        }
        if !earlier.is_empty() && any_phrase(&clean, &LEFT_POINTER) {
            for said in earlier.iter().rev() {
                let low = said.to_lowercase();
                for name in ingredients(&reading_of(text::split(&low), &low)) {
                    if let Some(idea) = LEFTOVER_IDEAS.get(&name) {
                        return Some(idea.clone());
                    }
                }
            }
        }
        if !diet.is_empty() {
            whole = std::iter::once(clean.clone()).chain(earlier.iter().map(|s| s.to_lowercase())).collect::<Vec<_>>().join(" ");
            if _crowd(&whole) >= 6 || _occasion(&whole) {
                return Some(_crowd_meal(&banned, _occasion(&whole), _crowd(&whole)));
            }
            let mut all: HashSet<String> = banned.clone();
            all.extend(missing.iter().cloned());
            return Some(_veg_dinner(&all, _plain_words(&whole).iter().any(|w| CHEAP_PLAN_WORDS.contains(w))));
        }
        if !_refused_foods(&clean, false).is_empty() && !banned.is_empty() {
            for d in _dishes_without(&banned) {
                if !d.text.is_empty() && !avoid.contains(&d.text) {
                    return Some(d.text);
                }
            }
        }
        if !said_no.is_empty() && folded.iter().any(|w| MEAL_WORDS.contains(w)) && !_reads_as_news(understood, &clean) {
            let mut all: HashSet<String> = banned.clone();
            all.extend(said_no.iter().cloned());
            return Some(_any_meal(&all));
        }
        return None;
    }
    let no_meat = !diet.is_empty() || meets(&MEAT, &banned);
    let leftover = _leftover(&folded, &clean);
    if found.len() == 1
        && !pointed_back
        && !bought
        && !_is_food_list(&folded, &clean)
        && !any_phrase(&clean, &MEAL_PHRASES)
        && !leftover
        && !(no_meat && found.iter().any(|n| MEAT.contains(n)))
    {
        return None;
    }
    let mut swap = String::new();
    if no_meat {
        let left: Vec<&String> = found.iter().filter(|n| MEAT.contains(*n)).collect();
        if let Some(meat) = left.first() {
            let instead = MEAT_SWAPS.get(*meat).cloned().unwrap_or_else(|| "chickpeas".to_string());
            swap = format!("{} do the same job in most dishes. ", upper_first(&instead));
            let mut picked: Vec<String> = Vec::new();
            for word in LETTERS_ONLY.findall_str(&instead) {
                if let Some(name) = INGREDIENT_WORDS.get(&_fold(&word)) {
                    if !name.is_empty() && !banned.contains(name) {
                        picked.push(name.clone());
                        break;
                    }
                }
            }
            if picked.is_empty() {
                picked.push("chickpea".to_string());
            }
            picked.extend(found.iter().filter(|n| !MEAT.contains(*n)).cloned());
            found = picked;
        }
    }
    if let Some(best) = _best_dish(&found, &cooked_ingredients(understood), &missing, &avoid, &diet) {
        if !best.without.is_empty() {
            return Some(swap + &_served_without(&best));
        }
        return Some(swap + &best.text);
    }
    if dishes_only {
        return None;
    }
    let leftover_said = folded.iter().any(|w| LEFTOVER_WORDS.contains(w)) || any_phrase(&clean, &LEFTOVER_PHRASES);
    if leftover_said {
        for name in &found {
            let idea = PANTRY.get(name).map(|(_, i)| i.clone()).unwrap_or_default();
            if idea.is_empty() || _mentions(&idea, &missing) || avoid.contains(&idea) {
                continue;
            }
            if idea_is_a_dish(&idea) {
                continue;
            }
            let mut said_name = name.clone();
            for word in LETTERS_ONLY.findall_str(&clean) {
                if _fold(&word) == _fold(name) || _fold(&word) == *name {
                    said_name = word;
                    break;
                }
            }
            let used = use_it_up(&said_name);
            if !used.is_empty() {
                return Some(used);
            }
        }
    }
    if swap.is_empty() && leftover_said {
        for name in &found {
            if let Some(idea) = LEFTOVER_IDEAS.get(name) {
                if !avoid.contains(idea) && !_mentions(idea, &missing) {
                    if name == "milk" && _mentions(&clean, &PLANT_MILKS) {
                        continue;
                    }
                    return Some(idea.clone());
                }
            }
        }
    }
    if _a_choice_of_foods(&clean) {
        return None;
    }
    let mut ideas: Vec<String> = Vec::new();
    for name in found.iter().take(2) {
        let idea = &PANTRY.get(name).expect("KeyError: not a pantry food").1;
        if !_mentions(idea, &missing) && !avoid.contains(idea) {
            ideas.push(idea.clone());
        }
    }
    if ideas.is_empty() {
        for name in found.iter().take(2) {
            if let Some(w) = WITHOUT.get(name) {
                if !_mentions(w, &missing) && !avoid.contains(w) {
                    ideas.push(w.clone());
                }
            }
        }
    }
    if ideas.is_empty() {
        return None;
    }
    let said = ideas.join(" ");
    if avoid.contains(&said) {
        return None;
    }
    Some(said)
}

/// Every text a meal answer can be (the import-time `_MEAL_TEXTS`).
pub fn _meal_texts() -> HashSet<String> {
    let mut texts: Vec<String> = DISHES.iter().filter(|d| !d.text.is_empty()).map(|d| d.text.clone()).collect();
    for (name, (_, idea)) in PANTRY.iter() {
        texts.push(idea.clone());
        texts.push(LEFTOVER_IDEAS.get(name).cloned().unwrap_or_default());
        texts.push(WITHOUT.get(name).cloned().unwrap_or_default());
    }
    texts.push(EMPTY_KITCHEN.clone());
    texts.extend(EMPTY_ASK_PHRASES.iter().cloned());
    texts.into_iter().filter(|t| !t.is_empty()).map(|t| text::strip(&t).to_string()).collect()
}

static MEAL_TEXTS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_MEAL_TEXTS"));
static MEAL_METHODS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_MEAL_METHODS"));

/// Whether a reply was one of the meal answers.
pub fn _is_meal_answer(said: &str) -> bool {
    let said = text::strip(said);
    if said.is_empty() {
        return false;
    }
    if MEAL_TEXTS.contains(said) {
        return true;
    }
    let method = text::partition(said, ": ").2;
    text::len(method) > 40 && MEAL_METHODS.iter().any(|m| m.starts_with(method))
}

static SPEAKS_OF_KEEPING: LazyLock<&'static Re> = LazyLock::new(|| cre("_SPEAKS_OF_KEEPING"));
static WORD: LazyLock<&'static Re> = LazyLock::new(|| cre("_WORD"));
static KEEP_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_KEEP_WORDS"));
static COOKING_SENSE: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_COOKING_SENSE"));
static KEEP_TIME_ASK: LazyLock<&'static Re> = LazyLock::new(|| cre("_KEEP_TIME_ASK"));
static BACK_REFERENCE: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_BACK_REFERENCE"));
static BAKED_GOODS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_BAKED_GOODS"));
static TIMING: LazyLock<&'static Re> = LazyLock::new(|| cre("_TIMING"));
static RECIPE_STEP: LazyLock<&'static Re> = LazyLock::new(|| cre("_RECIPE_STEP"));
static TOTAL_TIME: LazyLock<&'static Re> = LazyLock::new(|| cre("_TOTAL_TIME"));

/// Whether a reply talks about keeping food.
pub fn speaks_of_keeping(reply: &str) -> bool {
    SPEAKS_OF_KEEPING.is_match(reply)
}

/// Whether the line asks how long something keeps.
pub fn keeping_ask(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    let words: HashSet<String> = WORD.findall_str(&clean).into_iter().collect();
    if !meets(&words, &KEEP_WORDS)
        || words.iter().any(|w| COOKING_SENSE.contains(w) && w != "roast" && w != "roasting")
    {
        return false;
    }
    KEEP_TIME_ASK.is_match(&clean)
}

/// Whether an answer names a food the tables know.
pub fn _the_answer_names_a_food(said: &str) -> bool {
    WORD.findall_str(&said.to_lowercase()).iter().any(|w| _foods_said().contains(&_fold(w)))
}

/// "how long will it keep" after a meal answer: the cooked-food keeping times.
pub fn keeping_followup(understood: &Understood, memory: &Memory) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let words: HashSet<String> = WORD.findall_str(&clean).into_iter().collect();
    if !meets(&words, &KEEP_WORDS) || meets(&words, &COOKING_SENSE) {
        return None;
    }
    if !KEEP_TIME_ASK.is_match(&clean) {
        return None;
    }
    if !meets(&words, &BACK_REFERENCE) {
        return None;
    }
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(3)..] {
        let said = turn.user.to_lowercase();
        if BAKED_GOODS.iter().any(|n| said.contains(n.as_str())) {
            return None;
        }
    }
    let last = turns.last()?;
    if last.module.as_deref() != Some("answers") {
        return None;
    }
    let said = &last.bot;
    if WORD.findall_str(&said.to_lowercase()).iter().any(|w| KEEP_WORDS.contains(w)) {
        return None;
    }
    if !TIMING.is_match(said) && !RECIPE_STEP.is_match(said) {
        return None;
    }
    if !_the_answer_names_a_food(said) {
        return None;
    }
    Some(cstring("COOKED_KEEPING"))
}

static SENTENCE_ENDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));

/// "how long does that take" after a meal answer: its one total time.
pub fn timing_followup(understood: &Understood, memory: &Memory) -> Option<String> {
    let asks: HashSet<&str> = understood.asks.iter().map(String::as_str).collect();
    let words: HashSet<String> = understood.words.iter().cloned().collect();
    let clean = understood.clean.to_lowercase();
    if !(asks.contains("how_long") || clean.contains("how long") || clean.contains("how many minutes")) {
        return None;
    }
    if !meets(&words, &BACK_REFERENCE) {
        let folded: HashSet<String> = words.iter().map(|w| _fold(w)).collect();
        if folded.iter().any(|w| INGREDIENT_WORDS.contains_key(w)) {
            return None;
        }
        if meets(&folded, &KEEP_WORDS) {
            return None;
        }
    }
    let last = memory.turns.last()?;
    let mut said = text::strip(&last.bot).to_string();
    if last.module.as_deref() != Some("answers") {
        let kept_text = match memory.extra.get("last_answer") {
            Some(kept @ Value::Dict(_)) => kept.get("reply").filter(|v| v.truthy()).map(py_str).unwrap_or_default(),
            _ => String::new(),
        };
        let kept_text = text::strip(&kept_text).to_string();
        let head: String = kept_text.chars().take(40).collect::<String>().to_lowercase();
        if kept_text.is_empty() || text::len(&kept_text) < 40 || !said.to_lowercase().contains(&head) {
            return None;
        }
        said = kept_text;
    }
    if !_is_meal_answer(&said) {
        return None;
    }
    let sentences: Vec<String> = SENTENCE_ENDS.split(&said).into_iter().filter(|p| !text::strip(p).is_empty()).collect();
    let total: Vec<&String> = sentences.iter().filter(|p| TOTAL_TIME.is_match(p)).collect();
    let mut when: Option<String> = None;
    for part in total.iter().copied().chain(sentences.iter().filter(|p| !total.contains(p))) {
        let times: HashSet<String> =
            TIMING.finditer(part).iter().map(|m| text::squash(&m.as_str().to_lowercase())).collect();
        if times.len() == 1 {
            when = times.into_iter().next();
            break;
        }
    }
    let when = when?;
    let said_words: HashSet<String> = WORD.findall_str(&said.to_lowercase()).iter().map(|w| _fold(w)).collect();
    let named = understood.words.iter().any(|w| {
        let f = _fold(w);
        INGREDIENT_WORDS.contains_key(&f) && !said_words.contains(&f)
    });
    if named {
        return None;
    }
    Some(format!("About {when} for that one, start to finish."))
}

/// The line itself, as its words when it has them.
pub fn _this_line_only(understood: &Understood) -> String {
    if !understood.words.is_empty() {
        return understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    }
    understood.clean.to_lowercase()
}

static DISH_ON_ITS_OWN_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_DISH_ON_ITS_OWN_FILLER"));

/// Whether the line is the dish's name and nothing else that matters ("lasagne please").
pub fn _names_nothing_but_the_dish(text: &str, names: &[String]) -> bool {
    let mut left = format!(" {} ", text::split(&text.to_lowercase()).join(" "));
    let mut named = false;
    let mut sorted: Vec<&String> = names.iter().collect();
    sorted.sort_by_key(|n| std::cmp::Reverse(text::len(n)));
    for name in sorted {
        let needle = format!(" {} ", name.to_lowercase());
        if left.contains(&needle) {
            left = left.replace(&needle, " ");
            named = true;
        }
    }
    named && text::split(&left).iter().all(|w| DISH_ON_ITS_OWN_FILLER.contains(w))
}

static WANT_TO_COOK: LazyLock<&'static Re> = LazyLock::new(|| cre("_WANT_TO_COOK"));
static WANT_TO_LEARN_COOK: LazyLock<&'static Re> = LazyLock::new(|| cre("_WANT_TO_LEARN_COOK"));
static WANT_PHRASES: LazyLock<&'static Re> = LazyLock::new(|| cre("_WANT_PHRASES"));
static WANT_ACTION: LazyLock<&'static Re> = LazyLock::new(|| cre("_WANT_ACTION"));

/// Whether the line wants to make the dish (not to buy it or have it delivered).
pub fn _wants_the_dish(text: &str) -> bool {
    if WANT_TO_COOK.is_match(text) || WANT_TO_LEARN_COOK.is_match(text) {
        return true;
    }
    WANT_PHRASES.is_match(text) && !WANT_ACTION.is_match(text)
}

static RECIPE_ASK_WORDS: LazyLock<&'static Re> = LazyLock::new(|| cre("_RECIPE_ASK_WORDS"));
static RECIPE_ANSWERS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_RECIPE_ANSWERS"));
static RECIPE_NOT_THIS: LazyLock<&'static Re> = LazyLock::new(|| cre("_RECIPE_NOT_THIS"));
static RECIPE_KEEPING: LazyLock<&'static Re> = LazyLock::new(|| cre("_RECIPE_KEEPING"));
static RECIPE_COOKING: LazyLock<&'static Re> = LazyLock::new(|| cre("_RECIPE_COOKING"));

/// The first sentence of a recipe (`_recipe_only_when_asked_for_this`'s `total`).
pub fn _recipe_total(reply: &str) -> String {
    SENTENCE_ENDS.split(text::strip(reply)).into_iter().next().unwrap_or_default()
}

/// `_recipe_only_when_asked_for_this(reply, names, no_yeast_ok)`'s answer: the recipe, only
/// when the line asks for this dish (its first sentence for "how long").
pub fn _recipe_only_when_asked_for_this_answer(
    reply: &str,
    names: &[String],
    no_yeast_ok: bool,
    total: &str,
    memory: &Memory,
    understood: &Understood,
) -> Option<String> {
    let text = _said_so_far(understood, memory);
    let alone = _names_nothing_but_the_dish(&_this_line_only(understood), names);
    if !alone && !RECIPE_ASK_WORDS.is_match(&text) {
        return None;
    }
    if WANT_ACTION.is_match(&text) && !WANT_TO_COOK.is_match(&text) && !WANT_TO_LEARN_COOK.is_match(&text) {
        return None;
    }
    if !RECIPE_ANSWERS.contains(&understood.intent) && !_wants_the_dish(&text) && !alone {
        return None;
    }
    if !no_yeast_ok && NO_YEAST.iter().any(|one| text.contains(one.as_str())) {
        return None;
    }
    if RECIPE_NOT_THIS.is_match(&text) {
        return None;
    }
    let line = understood.clean.to_lowercase();
    if RECIPE_KEEPING.is_match(&line) && !RECIPE_COOKING.is_match(&line) {
        return None;
    }
    if understood.asks.len() == 1 && understood.asks[0] == "how_long" {
        return Some(total.to_string());
    }
    Some(reply.to_string())
}

static A_POT_ALREADY_MADE: LazyLock<&'static Re> = LazyLock::new(|| cre("_A_POT_ALREADY_MADE"));

/// A pot already made ("i made a big chilli") is not asking for the recipe.
pub fn _a_made_pot_is_not_its_recipe(understood: &Understood) -> bool {
    !A_POT_ALREADY_MADE.is_match(or(&understood.norm_text, &understood.clean))
}

static PANTRY_FOODS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_PANTRY_FOODS"));
static RECIPE_ALIASES: LazyLock<Vec<Vec<(String, Vec<String>)>>> = LazyLock::new(|| {
    cget("RECIPES")
        .items()
        .iter()
        .map(|r| r.items()[0].strs().into_iter().map(|a| { let parts = text::split(&a).iter().map(|p| _fold(p)).collect(); (a, parts) }).collect())
        .collect()
});

/// The recipe dish the line names (the alias as written), or None.
pub fn names_a_dish(understood: &Understood) -> Option<String> {
    let folded: Vec<String> = words_or_clean(understood).iter().filter(|w| !text::strip(w).is_empty()).map(|w| _fold(w)).collect();
    for aliases in RECIPE_ALIASES.iter() {
        for (alias, parts) in aliases {
            if parts.is_empty() || PANTRY_FOODS.contains(&parts.join(" ")) {
                continue;
            }
            let hit = if parts.len() == 1 {
                folded.contains(&parts[0])
            } else {
                folded.len() >= parts.len() && (0..=(folded.len() - parts.len())).any(|at| folded[at..at + parts.len()] == parts[..])
            };
            if hit {
                return Some(alias.clone());
            }
        }
    }
    None
}

static POINT_BACK_OBJECT: LazyLock<&'static Re> = LazyLock::new(|| cre("POINT_BACK_OBJECT"));
static THE_FOOD_OR_THE_MEAL: LazyLock<HashSet<String>> = LazyLock::new(|| cset("THE_FOOD_OR_THE_MEAL"));

/// "ideas for my sister's birthday": the ideas pointed at are not for a food.
pub fn _ideas_are_for_something_else(said: &str) -> bool {
    let Some(named) = POINT_BACK_OBJECT.search(said) else { return false };
    !WORDS_AZ.findall_str(&named.str(1).to_lowercase()).iter().any(|w| THE_FOOD_OR_THE_MEAL.contains(w))
}

/// The bot's last replies.
pub fn recent_replies(memory: &Memory, count: usize) -> Vec<String> {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(count)..].iter().map(|t| t.bot.clone()).collect()
}

/// The words of a text, lower-case.
pub fn _words_of(text: &str) -> Vec<String> {
    WORDS_AZ09.findall_str(&text.to_lowercase())
}

static POINT_BACK_STOP: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("POINT_BACK_STOP"));
static POINT_BACK: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("POINT_BACK"));
static POINT_BACK_BARE: LazyLock<HashSet<String>> = LazyLock::new(|| cset("POINT_BACK_BARE"));
static PAIRING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("PAIRING_WORDS"));
static SECOND_IDEA: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("SECOND_IDEA"));

/// The words of the person's last line that named foods, nearest first.
pub fn _food_before(memory: &Memory) -> Vec<String> {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        let said = &turn.user;
        if POINT_BACK_STOP.iter().any(|p| said.to_lowercase().contains(p.as_str())) {
            return Vec::new();
        }
        let words = _words_of(said);
        if !ingredients(&reading_of(words.clone(), "")).is_empty() {
            return words;
        }
    }
    Vec::new()
}

/// "what can i make with them": the reading with the foods from the earlier line put in, and
/// whether it asks for a new dish only. None when the line names its own foods or points at
/// nothing.
pub fn pointed(memory: &Memory, understood: &Understood) -> Option<(Understood, bool)> {
    if !ingredients(understood).is_empty() {
        return None;
    }
    let clean = understood.clean.to_lowercase();
    if !any_phrase(&clean, &POINT_BACK) {
        return None;
    }
    if !POINT_BACK.iter().any(|p| !POINT_BACK_BARE.contains(p) && clean.contains(p.as_str()))
        && _ideas_are_for_something_else(&clean)
    {
        return None;
    }
    if any_phrase(&clean, &PAIRING_WORDS) {
        return None;
    }
    let earlier = _food_before(memory);
    if earlier.is_empty() {
        return None;
    }
    let mut reading = understood.clone();
    let mut words = earlier.clone();
    words.extend(_words_of(&understood.words.join(" ")));
    reading.words = words;
    reading.clean = format!("{} {}", earlier.join(" "), clean);
    let new_dish_only = any_phrase(&clean, &SECOND_IDEA);
    Some((reading, new_dish_only))
}

static SCALE_UP_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("SCALE_UP_WORDS"));

/// Scaling a recipe up or down.
pub fn _scaling_reply(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let line = if reading.text.is_empty() { norm_str(reading) } else { reading.text.clone() }.to_lowercase();
    let words: HashSet<String> = WORDS_AZ.findall_str(&line).into_iter().collect();
    if SCALE_UP_WORDS.iter().any(|w| words.contains(w) || (w.contains(' ') && line.contains(w.as_str()))) {
        return Some(cstring("SCALE_UP"));
    }
    if words.contains("scale") && words.contains("up") {
        return Some(cstring("SCALE_UP"));
    }
    Some(cstring("SCALE_DOWN"))
}

// ---------------------------------------------------------------------------------------------
// Registered guards and replies (ids of the original's lines 6000 and on)

fn recipe_answer(f: &FnRef, memory: &mut Memory, u: &Understood) -> Option<String> {
    let cell = |n: &str| f.cell(n).unwrap_or_else(|| panic!("closure has no cell {n}"));
    let reply = cell("reply").str().to_string();
    let names = cell("names").strs();
    let no_yeast_ok = cell("no_yeast_ok").truthy();
    let total = cell("total").str().to_string();
    _recipe_only_when_asked_for_this_answer(&reply, &names, no_yeast_ok, &total, memory, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.cooking:<lambda>:9731", |_: &FnRef, u: &Understood| _cooked_keep_guard(u)),
    ("bot.knowledge.cooking:_a_food_and_not_a_blank:9660", |_: &FnRef, u: &Understood| _a_food_and_not_a_blank(u)),
    ("bot.knowledge.cooking:_a_made_pot_is_not_its_recipe:12348", |_: &FnRef, u: &Understood| _a_made_pot_is_not_its_recipe(u)),
    ("bot.knowledge.cooking:asks_a_gluten_fact:7145", |_: &FnRef, u: &Understood| asks_a_gluten_fact(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.cooking:_bake:7917", |_: &FnRef, m: &mut Memory, u: &Understood| _bake(m, u, None)),
    ("bot.knowledge.cooking:_bake_burning:8606", |_: &FnRef, m: &mut Memory, u: &Understood| _bake_burning(m, u)),
    ("bot.knowledge.cooking:_bake_follow:9126", |_: &FnRef, m: &mut Memory, u: &Understood| _bake_follow(m, u)),
    ("bot.knowledge.cooking:_bake_keep:9110", |_: &FnRef, m: &mut Memory, u: &Understood| _bake_keep(m, u)),
    ("bot.knowledge.cooking:_bread_rise_answer.<locals>.answer:8098", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _bread_rise_answer_answer(m, u)
    }),
    ("bot.knowledge.cooking:_breakfast_ideas:6032", |_: &FnRef, m: &mut Memory, u: &Understood| _breakfast_ideas(m, u)),
    ("bot.knowledge.cooking:_burnt_dish:8361", |_: &FnRef, m: &mut Memory, u: &Understood| _burnt_dish(m, u)),
    ("bot.knowledge.cooking:_cake_fault:8522", |_: &FnRef, m: &mut Memory, u: &Understood| _cake_fault(m, u)),
    ("bot.knowledge.cooking:_calories:6916", |_: &FnRef, m: &mut Memory, u: &Understood| _calories(m, u)),
    ("bot.knowledge.cooking:_coffee_fault:8158", |_: &FnRef, m: &mut Memory, u: &Understood| _coffee_fault(m, u)),
    ("bot.knowledge.cooking:_cook_word:7776", |_: &FnRef, m: &mut Memory, u: &Understood| _cook_word(m, u)),
    ("bot.knowledge.cooking:_eat_when_ill:10241", |_: &FnRef, m: &mut Memory, u: &Understood| _eat_when_ill(m, u)),
    ("bot.knowledge.cooking:_form_swap:10477", |_: &FnRef, m: &mut Memory, u: &Understood| _form_swap(m, u, None)),
    ("bot.knowledge.cooking:_free_of_gluten_fact:7166", |_: &FnRef, m: &mut Memory, u: &Understood| _free_of_gluten_fact(m, u)),
    ("bot.knowledge.cooking:_freeze_food:6250", |_: &FnRef, m: &mut Memory, u: &Understood| _freeze_food(m, u)),
    ("bot.knowledge.cooking:_gluten_free:7045", |_: &FnRef, m: &mut Memory, u: &Understood| _gluten_free(m, u)),
    ("bot.knowledge.cooking:_greasy:8671", |_: &FnRef, m: &mut Memory, u: &Understood| _greasy(m, u)),
    ("bot.knowledge.cooking:_gummy:9171", |_: &FnRef, m: &mut Memory, u: &Understood| _gummy(m, u)),
    ("bot.knowledge.cooking:_herbs_keep:7655", |_: &FnRef, m: &mut Memory, u: &Understood| _herbs_keep(m, u)),
    ("bot.knowledge.cooking:_honey_age:6133", |_: &FnRef, m: &mut Memory, u: &Understood| _honey_age(m, u)),
    ("bot.knowledge.cooking:_leftover_meal_reply:9915", |_: &FnRef, m: &mut Memory, u: &Understood| _leftover_meal_reply(m, u)),
    ("bot.knowledge.cooking:_non_stick_pan:7953", |_: &FnRef, m: &mut Memory, u: &Understood| _non_stick_pan(m, u)),
    ("bot.knowledge.cooking:_nut_free:6283", |_: &FnRef, m: &mut Memory, u: &Understood| _nut_free(m, u)),
    ("bot.knowledge.cooking:_nutrients:6771", |_: &FnRef, m: &mut Memory, u: &Understood| _nutrients(m, u)),
    ("bot.knowledge.cooking:_oil_smoking:8875", |_: &FnRef, m: &mut Memory, u: &Understood| _oil_smoking(m, u)),
    ("bot.knowledge.cooking:_pastry_fault:8559", |_: &FnRef, m: &mut Memory, u: &Understood| _pastry_fault(m, u)),
    ("bot.knowledge.cooking:_pudding_ideas:7555", |_: &FnRef, m: &mut Memory, u: &Understood| _pudding_ideas(m, u)),
    ("bot.knowledge.cooking:_recipe_only_when_asked_for_this.<locals>.answer:12250", recipe_answer),
    ("bot.knowledge.cooking:_rice_time:8056", |_: &FnRef, m: &mut Memory, u: &Understood| _rice_time(m, u)),
    ("bot.knowledge.cooking:_scaling_reply:12718", |_: &FnRef, m: &mut Memory, u: &Understood| _scaling_reply(m, u)),
    ("bot.knowledge.cooking:_soggy_lunch:9040", |_: &FnRef, m: &mut Memory, u: &Understood| _soggy_lunch(m, u)),
    ("bot.knowledge.cooking:_split:8723", |_: &FnRef, m: &mut Memory, u: &Understood| _split(m, u)),
    ("bot.knowledge.cooking:_tea_reply:8032", |_: &FnRef, m: &mut Memory, u: &Understood| _tea_reply(m, u)),
    ("bot.knowledge.cooking:_tin_answer:8501", |_: &FnRef, m: &mut Memory, u: &Understood| _tin_answer(m, u)),
    ("bot.knowledge.cooking:_undercooked:8766", |_: &FnRef, m: &mut Memory, u: &Understood| _undercooked(m, u)),
    ("bot.knowledge.cooking:_vegan_bake:6336", |_: &FnRef, m: &mut Memory, u: &Understood| _vegan_bake(m, u)),
];
