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

// DUP(bot.knowledge.units::people_in_groups)
/// How many people the groups named in the line add up to, or None when it names none.
pub fn people_in_groups(text_: &str) -> Option<i64> {
    let values = data::get("bot.knowledge.units._GROUP_VALUE");
    let mut total = 0i64;
    for raw in TABLE_GROUPS.findall_str(&text_.to_lowercase()) {
        let count = if !raw.is_empty() && text::is_digit(&raw) {
            raw.parse::<i64>().ok()
        } else {
            values.get(&raw).and_then(Value::as_int)
        };
        if let Some(c) = count {
            if c != 0 {
                total += c;
            }
        }
    }
    if total != 0 { Some(total) } else { None }
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.garden

// DUP(bot.knowledge.garden::harvest_crops)
/// Every crop that has a harvest row, so a cooking method can stand down for one.
pub fn garden_harvest_crops() -> HashSet<String> {
    let mut names = HashSet::new();
    for row in data::get("bot.knowledge.garden.HARVEST_MORE").items() {
        names.extend(row.items()[0].strs());
    }
    let harvest = data::get("bot.knowledge.garden.HARVEST");
    for row in data::get("bot.knowledge.garden.SOWING").items() {
        let words = row.items()[0].strs();
        if harvest.get(&words[0]).is_some_and(Value::truthy) {
            names.extend(words);
        }
    }
    names
}

// DUP(bot.knowledge.garden::DONE_NOT)
pub fn garden_done_not() -> Vec<String> {
    data::strs("bot.knowledge.garden.DONE_NOT")
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.dishes (D1)

// DUP(bot.knowledge.dishes::_same_word)
/// eggs/egg, potatoes/potato, roasting/roast: the loose forms cooking.py matches with.
pub fn dishes_same_word(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    for (one, two) in [(a, b), (b, a)] {
        if one.chars().count() > 3 {
            if one.ends_with('s') && &one[..one.len() - 1] == two {
                return true;
            }
            if one.ends_with("es") && &one[..one.len() - 2] == two {
                return true;
            }
            if one.ends_with("ing") && (&one[..one.len() - 3] == two || format!("{}e", &one[..one.len() - 3]) == two) {
                return true;
            }
            if one.ends_with("ed") && &one[..one.len() - 2] == two {
                return true;
            }
        }
    }
    false
}

// DUP(bot.knowledge.dishes::_phrase_in)
/// True when the words of `phrase` appear in `text` in order (a short run).
pub fn dishes_phrase_in(text_: &str, phrase: &str) -> bool {
    let wanted = tokens(phrase);
    let have = tokens(text_);
    if wanted.is_empty() || have.is_empty() || have.len() < wanted.len() {
        return false;
    }
    let n = wanted.len();
    (0..=(have.len() - n)).any(|i| (0..n).all(|j| have[i + j] == wanted[j] || dishes_same_word(&have[i + j], &wanted[j])))
}

// DUP(bot.knowledge.dishes::recent_texts)
/// What the conversation has been about, newest first, in the user's own words.
pub fn dishes_recent_texts(memory: &Memory) -> Vec<String> {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(4);
    let mut users: Vec<String> = turns[start..].iter().rev().map(|t| t.user.clone()).collect();
    if let Some(record) = memory.extra.get("topic") {
        if let Some(t) = record.get("text").and_then(Value::as_str) {
            users.push(t.to_string());
        }
    }
    users
}

static GOES_WITH_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.dishes.GOES_WITH_SHAPE"));

fn dishes_names_any(text_: &str, words_name: &str) -> bool {
    data::strs(words_name).iter().any(|w| dishes_phrase_in(text_, w))
}

// DUP(bot.knowledge.dishes::asks_for_a_side_of_a_food)
/// True when the line asks the pairing way and names a food it can be paired with.
pub fn dishes_asks_for_a_side_of_a_food(text_: &str) -> bool {
    GOES_WITH_SHAPE.is_match(text_) && dishes_names_any(text_, "bot.knowledge.dishes.PAIRING_FOODS")
}

// DUP(bot.knowledge.dishes::general_sides)
/// The pairing rule for a food there is no dish for.
pub fn dishes_general_sides(text_: &str) -> String {
    if dishes_names_any(text_, "bot.knowledge.dishes.RICH_FOODS") {
        return data::string("bot.knowledge.dishes.PAIRING_RICH");
    }
    if dishes_names_any(text_, "bot.knowledge.dishes.PLAIN_FOODS") {
        return data::string("bot.knowledge.dishes.PAIRING_PLAIN");
    }
    data::string("bot.knowledge.dishes.PAIRING_LEAN")
}

static OVEN_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.dishes.OVEN_WORD"));

// DUP(bot.knowledge.dishes::_names_an_appliance_and_a_cut)
pub fn dishes_names_an_appliance_and_a_cut(text_: &str) -> bool {
    let ideas = data::get("bot.knowledge.dishes.APPLIANCE_IDEAS");
    let named = ideas.items().iter().any(|row| row.items()[0].strs().iter().any(|p| dishes_phrase_in(text_, p)));
    if !named && !OVEN_WORD.is_match(text_) {
        return false;
    }
    let cuts = data::set("bot.knowledge.dishes.CUT_WORDS");
    tokens(text_).iter().any(|w| cuts.contains(&w.to_lowercase()))
}

/// `dishes.PLANS.get(key)`.
pub fn dishes_plan(key: &str) -> Option<&'static Value> {
    data::get("bot.knowledge.dishes.PLANS").get(key)
}

/// `dishes.dish_in`: the plan key a text names (D1's port).
pub fn dishes_dish_in(text: &str) -> Option<String> {
    crate::knowledge::dishes::dish_in(text).map(str::to_string)
}

// NEEDS(bot.knowledge.dishes::ask_kind) — which kind of follow-up the line is.
pub fn dishes_ask_kind(u: &Understood) -> Option<String> {
    crate::knowledge::dishes::ask_kind(u).map(str::to_string)
}

// NEEDS(bot.knowledge.dishes::reply) — the plan's answer to a follow-up of that kind.
pub fn dishes_reply(_kind: &str, _memory: &mut Memory, _u: &Understood) -> Option<String> {
    None
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup (U)

// NEEDS(bot.knowledge.lookup::find) — the id of the entry the matcher finds for the line, with
// the rows of these ids rejected. Until U's matcher is merged: None.
pub fn lookup_find_id(_u: &Understood, _reject_ids: &[&str]) -> Option<String> {
    None
}

// NEEDS(bot.knowledge.lookup::find) with the garden's entries: whether the garden table answers
// the line. Until U's matcher is merged: False.
pub fn lookup_find_in_garden(_line: &str) -> bool {
    false
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.meals (D1)

fn meals(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.meals.{name}"))
}

// DUP(bot.knowledge.meals::says_a_diet)
/// True when the text names a diet.
pub fn meals_says_a_diet(text_: &str) -> bool {
    let low = text_.to_lowercase();
    meals("DIET_WORDS").strs().iter().any(|cue| low.contains(cue.as_str()))
}

// DUP(bot.knowledge.meals::_diet_said)
/// The diet the conversation (or the stored facts) has named, or "".
pub fn meals_diet_said(memory: &Memory, understood: Option<&Understood>) -> String {
    let start = memory.turns.len().saturating_sub(6);
    let mut said = memory.turns[start..].iter().map(|t| t.user.to_lowercase()).collect::<Vec<_>>().join(" ");
    if let Some(u) = understood {
        if !u.clean.is_empty() {
            said.push(' ');
            said.push_str(&u.clean.to_lowercase());
        }
    }
    let diet = match memory.extra.get("user") {
        Some(user @ Value::Dict(_)) => user.get("diet").filter(|v| v.truthy()).map(|v| v.str().to_lowercase()).unwrap_or_default(),
        _ => String::new(),
    };
    let said = text::strip(&format!("{diet} {said}")).to_string();
    for row in meals("DIET_SAID").items() {
        let row = row.items();
        if said.contains(row[0].str()) {
            return row[1].str().to_string();
        }
    }
    String::new()
}

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

// DUP(bot.knowledge.meals::_ruled_out)
/// The ingredients a line rules out ("no onion", "don't have any eggs").
pub fn meals_ruled_out(text_: &str) -> HashSet<String> {
    let mut plain = format!(" {} ", MEALS_WORD.findall_str(&text_.replace('\'', "")).join(" "));
    for phrase in meals("DOUBLE_NO").strs() {
        let padded = format!(" {phrase} ");
        if plain.contains(&padded) {
            plain = plain.replace(&padded, " ");
        }
    }
    let no_cue = meals("NO_CUE").strs();
    let no_gap = meals("NO_GAP").strs();
    let no_have = meals("NO_HAVE").strs();
    let mut out = HashSet::new();
    for (ingredient, forms) in ingredient_forms() {
        for form in &forms {
            if !text::is_alpha(form) && !form.contains(' ') {
                continue;
            }
            let shape = if !text::is_alpha(form) { form.replace('\'', "") } else { form.clone() };
            if !plain.contains(&format!(" {shape} ")) {
                continue;
            }
            let mut hit = false;
            for cue in &no_cue {
                if no_gap.iter().any(|gap| plain.contains(&format!(" {cue}{gap} {shape} "))) {
                    out.insert(ingredient.clone());
                    hit = true;
                    break;
                }
            }
            if !hit {
                for cue in &no_have {
                    if no_have_match(cue, &shape, &plain) {
                        out.insert(ingredient.clone());
                        break;
                    }
                }
            }
        }
    }
    out
}

// DUP(bot.knowledge.meals::_is_the_ruled_out_food)
/// True when the word is one of the ingredients the text rules out.
pub fn meals_is_the_ruled_out_food(word: &str, text_: &str) -> bool {
    let word = text::strip(word).to_lowercase();
    if word.is_empty() {
        return false;
    }
    let ingredients = meals("INGREDIENTS");
    for ingredient in meals_ruled_out(text_) {
        let mut forms = ingredients.get(&ingredient).map(Value::strs).unwrap_or_default();
        forms.push(ingredient.clone());
        for form in forms {
            let form = text::strip(&form).to_lowercase();
            if form == word || form.starts_with(&word) || word.starts_with(&form) {
                return true;
            }
        }
    }
    false
}

fn strip_set<'a>(word: &'a str, chars: &str) -> &'a str {
    word.trim_end_matches(|c| chars.contains(c))
}

fn known_word(word: &str, known: &HashSet<String>) -> bool {
    known.contains(word) || known.contains(strip_set(word, "s")) || known.contains(strip_set(word, "es"))
}

// DUP(bot.knowledge.meals::unknown_food)
/// True when the line asks what to make with (or a recipe for) a food the tables do not know.
pub fn meals_unknown_food(u: &Understood) -> bool {
    let clean = super::cooking::line_as_read(u).to_lowercase();
    let mut known: HashSet<String> = meals("FOOD_WORDS").strs().into_iter().collect();
    for (ingredient, forms) in ingredient_forms() {
        known.insert(ingredient);
        known.extend(forms);
    }
    let kitchen = meals("KITCHEN_WORDS").str_set();
    // _unknown_ingredient
    let not_an_ingredient = meals("NOT_AN_INGREDIENT").str_set();
    let company = meals("COMPANY").str_set();
    let with_ask = data::re("bot.knowledge.meals.WITH_ASK");
    let mut unknown_ingredient = false;
    for m in with_ask.finditer(&clean) {
        let rest: Vec<String> = MEALS_WORD
            .findall_str(m.str(1))
            .into_iter()
            .filter(|w| !kitchen.contains(w) && !not_an_ingredient.contains(w) && !text::is_digit(w))
            .collect();
        if rest.is_empty() {
            break;
        }
        if rest.iter().any(|w| company.contains(w) || company.contains(strip_set(w, "s"))) {
            continue;
        }
        if !rest.iter().any(|w| known_word(w, &known)) {
            unknown_ingredient = true;
            break;
        }
    }
    if unknown_ingredient {
        return true;
    }
    // _unknown_dish_recipe
    let Some(m) = data::re("bot.knowledge.meals.RECIPE_FOR").search(&clean) else { return false };
    let not_a_dish = meals("NOT_A_DISH_NAME").strs();
    let tail: Vec<String> = MEALS_WORD
        .findall_str(m.str(1))
        .into_iter()
        .filter(|w| !kitchen.contains(w) && !not_a_dish.contains(w) && !text::is_digit(w))
        .collect();
    if tail.is_empty() {
        return false;
    }
    !tail.iter().any(|w| known_word(w, &known))
}
