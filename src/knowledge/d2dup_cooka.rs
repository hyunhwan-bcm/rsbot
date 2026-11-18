//! Helpers `cooking.rs` (lines 1-5999 of cooking.py) needs from code it does not own: the second
//! half of cooking.py (cooking_b.rs, `DUP-D2`), `prices` (`DUP-D2`), and D1's `dishes`, `garden`,
//! `units` and U's `lookup` (`DUP` / `NEEDS`). Each is a full port of the original unless it is
//! marked NEEDS (a stub until the owner's port is merged).

use crate::data::{self, Value};
use crate::knowledge::d2_shared::tokens;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn cook(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.cooking.{name}"))
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.cooking, second half (cooking_b.rs)

// DUP-D2(bot.knowledge.cooking::_fold)
/// A word with its plural undone, so eggs/egg and tomatoes/tomato are one word.
pub fn _fold(word: &str) -> String {
    let lowered = word.to_lowercase().replace('\u{2019}', "'");
    let w = text::strip_chars(&lowered, ".,!?;:'\"");
    let n = w.chars().count();
    if n > 4 && w.ends_with("ies") {
        return format!("{}y", &w[..w.len() - 3]);
    }
    if n > 4 && w.ends_with("oes") {
        return w[..w.len() - 2].to_string();
    }
    if n > 3 && w.ends_with("es") {
        let third = w[..w.len() - 2].chars().last();
        if matches!(third, Some('s' | 'x' | 'z' | 'h')) {
            return w[..w.len() - 2].to_string();
        }
    }
    if n > 2 && w.ends_with('s') && !w.ends_with("ss") {
        return w[..w.len() - 1].to_string();
    }
    w.to_string()
}

static FOODS_SAID: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut out: HashSet<String> = cook("_NAMED_FOODS").strs().iter().map(|w| _fold(&w.to_lowercase())).collect();
    out.extend(cook("_STAPLE_NAMES").strs().iter().map(|w| _fold(&w.to_lowercase())));
    out
});

// DUP-D2(bot.knowledge.cooking::_foods_said)
/// Every food name the pantry and the staples table know, with its plural undone.
pub fn _foods_said() -> &'static HashSet<String> {
    &FOODS_SAID
}

static INGREDIENT_WORDS: LazyLock<std::collections::HashMap<String, String>> =
    LazyLock::new(|| cook("_INGREDIENT_WORDS").str_map());

// DUP-D2(bot.knowledge.cooking::_mentions)
/// Whether a reply names one of these pantry ingredients ("150g flour", "grated cheese").
pub fn _mentions<S: AsRef<str>>(text_: &str, names: &[S]) -> bool {
    if names.is_empty() {
        return false;
    }
    let has = |name: &str| names.iter().any(|n| n.as_ref() == name);
    for word in text::split(&text_.to_lowercase()).iter().map(|w| _fold(w)) {
        let mut name = INGREDIENT_WORDS.get(&word).cloned();
        if name.is_none() {
            for ending in ["", "s", "es"] {
                let key = format!("{}{}", _fold(text::rstrip_chars(&word, ",.")), ending);
                name = INGREDIENT_WORDS.get(&key).cloned();
                if name.is_some() {
                    break;
                }
            }
        }
        if let Some(n) = name {
            if has(&n) {
                return true;
            }
        }
    }
    false
}

// DUP-D2(bot.knowledge.cooking::_recent_user_turns)
/// What the person typed in the last few turns, oldest first.
pub fn _recent_user_turns(memory: &Memory, count: usize) -> Vec<String> {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(count);
    turns[start..].iter().map(|t| t.user.clone()).collect()
}

static SHORT_OF_ASK: LazyLock<&'static Re> = LazyLock::new(|| cook("SHORT_OF_ASK").as_re().unwrap());

// DUP-D2(bot.knowledge.cooking::_short_of_food)
/// The foods a line says they have none of, as opposed to cannot eat.
pub fn _short_of_food(text_: &str) -> HashSet<String> {
    let low = text_.to_lowercase();
    let mut found = HashSet::new();
    for m in SHORT_OF_ASK.finditer(&low) {
        let at = text::char_index(&low, m.end()) as isize;
        let piece = text::slice(&low, at, Some(at + 60));
        found.extend(super::cooking::_foods_in_list(&super::cooking::_list_words(&piece, false)));
    }
    found
}

// DUP-D2(bot.knowledge.cooking::_words_and_text)
/// The line as one lower-case string: clean, norm_text and the normalised words.
pub fn _words_and_text(u: &Understood) -> String {
    format!("{} {} {}", u.clean, u.norm_text, u.norm.join(" ")).to_lowercase()
}

// DUP-D2(bot.knowledge.cooking::_says)
/// True when one of these phrases is in the line as whole words.
pub fn _says<S: AsRef<str>>(said: &str, phrases: &[S]) -> bool {
    let alts: Vec<String> = phrases.iter().map(|p| crate::py::re::escape(p.as_ref())).collect();
    Re::new(&format!("\\b(?:{})\\b", alts.join("|")), 0).is_match(said)
}

static BURNT_ON_A_PAN: LazyLock<Vec<String>> = LazyLock::new(|| cook("_BURNT_ON_A_PAN").strs());
static BURNT_WORDS_RE: LazyLock<Re> = LazyLock::new(|| {
    let words = [
        "burnt", "burned", "black", "blackened", "scorched", "scorch", "burn mark", "scorch mark", "scorched on",
        "stain", "stained", "discoloured", "discolored", "scratch", "scratched",
    ];
    let alts: Vec<String> = words.iter().map(|p| crate::py::re::escape(p)).collect();
    Re::new(&format!("\\b(?:{})\\b", alts.join("|")), 0)
});
static BURNT_ON_A_PAN_RE: LazyLock<Re> = LazyLock::new(|| {
    let alts: Vec<String> = BURNT_ON_A_PAN.iter().map(|p| crate::py::re::escape(p)).collect();
    Re::new(&format!("\\b(?:{})\\b", alts.join("|")), 0)
});

// DUP-D2(bot.knowledge.cooking::_burnt_on_the_pan)
/// True for a pan with something burnt on it that the person wants off.
pub fn _burnt_on_the_pan(text_: &str) -> bool {
    BURNT_ON_A_PAN_RE.is_match(text_) && BURNT_WORDS_RE.is_match(text_)
}

static ALIASES_BY_LENGTH: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    cook("_ALIASES_BY_LENGTH").items().iter().map(|row| (row.items()[0].strs(), row.items()[1].str().to_string())).collect()
});
static NEGATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| cook("NEGATIONS").str_set());
static SOFTENED: LazyLock<HashSet<String>> = LazyLock::new(|| cook("SOFTENED").str_set());
static CLAUSE_BREAK: LazyLock<HashSet<String>> = LazyLock::new(|| cook("CLAUSE_BREAK").str_set());
static RULING_OUT_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cook("RULING_OUT_PHRASES").strs());
static EVENT_HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| cook("EVENT_HEADS").str_set());
static REFUSAL_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| cook("REFUSAL_JOINERS").str_set());

// DUP-D2(bot.knowledge.cooking::_refused_before)
pub fn _refused_before(folded: &[String], place: usize) -> bool {
    if place == 0 {
        return false;
    }
    let reach = cook("REFUSAL_REACH").as_int().unwrap_or(6) as usize;
    let mut window: Vec<&str> = Vec::new();
    for word in folded[place.saturating_sub(reach)..place].iter().rev() {
        if CLAUSE_BREAK.contains(word) {
            break;
        }
        window.insert(0, word);
    }
    let plain = format!(" {} ", window.join(" "));
    RULING_OUT_PHRASES.iter().any(|cue| plain.contains(&format!(" {cue} ")))
}

// DUP-D2(bot.knowledge.cooking::_renamed_by_an_event)
pub fn _renamed_by_an_event(folded: &[String], place: usize) -> bool {
    if place == 0 {
        return false;
    }
    EVENT_HEADS.contains(&folded[place - 1])
}

// DUP-D2(bot.knowledge.cooking::_on_a_list_already_ruled_out)
pub fn _on_a_list_already_ruled_out(folded: &[String], place: usize, refused_ends: &[usize]) -> bool {
    for &end in refused_ends {
        let between: &[String] = if end < place { &folded[end..place] } else { &[] };
        if !between.is_empty() && between.iter().all(|w| REFUSAL_JOINERS.contains(w)) {
            return true;
        }
    }
    false
}

// DUP-D2(bot.knowledge.cooking::_work_out_ingredients)
/// (the ingredients these folded words name, the ones they rule out).
pub fn _work_out_ingredients(folded: &[String]) -> (Vec<String>, Vec<String>) {
    let mut used = vec![false; folded.len()];
    let present: HashSet<&str> = folded.iter().map(String::as_str).collect();
    let mut hits: Vec<(usize, String, usize)> = Vec::new();
    for (parts, name) in ALIASES_BY_LENGTH.iter() {
        if !present.contains(parts[0].as_str()) {
            continue;
        }
        let span = parts.len();
        if folded.len() < span {
            continue;
        }
        for start in 0..=(folded.len() - span) {
            if used[start..start + span].iter().any(|&u| u) {
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
    let reach = cook("NEGATION_REACH").as_int().unwrap_or(3) as usize;
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
        let before = &folded[place.saturating_sub(reach)..place];
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

// DUP-D2(bot.knowledge.cooking::_named_ingredients)
/// Every pantry ingredient the line names, and the ones it rules out.
pub fn _named_ingredients(u: &Understood) -> (Vec<String>, Vec<String>) {
    // TYPE(words is always a list in a reading; the clean-split fallback is for a non-list)
    let mut folded: Vec<String> = u.words.iter().map(|w| _fold(w)).collect();
    let repaired: Vec<String> = text::split(&u.plain).iter().map(|w| _fold(w)).collect();
    if repaired.len() == folded.len() && repaired != folded {
        folded = repaired;
    }
    _work_out_ingredients(&folded)
}

// DUP-D2(bot.knowledge.cooking::negated_ingredients)
/// The pantry ingredients the line says it has not got ("but no flour", "without cheese").
pub fn negated_ingredients(u: &Understood) -> Vec<String> {
    let (named, negated) = _named_ingredients(u);
    negated.into_iter().filter(|n| named.contains(n)).collect()
}

// DUP-D2(bot.knowledge.cooking::meal) NEEDS: cookB's full port (a 170-line dispatcher over the
// pantry tables); until it is merged this stands in with None ("no meal idea").
#[allow(clippy::too_many_arguments)]
pub fn meal(
    _understood: &Understood,
    _dishes_only: bool,
    _text: Option<&str>,
    _banned: Option<&HashSet<String>>,
    _diet: Option<&str>,
    _memory: Option<&Memory>,
    _avoid: &[String],
    _pointed_back: bool,
) -> Option<String> {
    None
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.prices (g6)

static JUDGING: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.prices._JUDGING"));

// DUP-D2(bot.knowledge.prices::a_price_is_judged)
/// True when the line brings a figure in money and asks whether it is a lot.
pub fn a_price_is_judged(text_: &str) -> bool {
    JUDGING.is_match(&text::squash(&text_.to_lowercase()))
}

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

// NEEDS(bot.knowledge.dishes::dish_in) — the plan key a text names (D1's dish matcher).
pub fn dishes_dish_in(_text: &str) -> Option<String> {
    None
}

// NEEDS(bot.knowledge.dishes::ask_kind) — which kind of follow-up the line is.
pub fn dishes_ask_kind(_u: &Understood) -> Option<String> {
    None
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
