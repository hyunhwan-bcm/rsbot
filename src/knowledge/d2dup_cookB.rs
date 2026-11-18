//! Duplicates the second half of `cooking.py` (`cooking_b`) needs from code it does not own:
//! the first half of `cooking.py` (fork cookA, `DUP-D2`), and helpers in `understanding`,
//! `units`, `memory`, `meals`, `cooking_more`, `picking`, `dishes` and `lookup` (`DUP` /
//! `NEEDS`). Each is a complete port of the original, named and shaped as its owner will be.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

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
/// A `{str: set/tuple of str}` table, in order.
fn cset_map(name: &str) -> Vec<(String, HashSet<String>)> {
    data::get(&c(name))
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.str_set())))
        .collect()
}

/// `a & b` is not empty.
pub fn meets(a: &HashSet<String>, b: &HashSet<String>) -> bool {
    if a.len() <= b.len() { a.iter().any(|x| b.contains(x)) } else { b.iter().any(|x| a.contains(x)) }
}

/// The text from a byte offset on, at most `n` characters (Python `text[at:at + n]`).
pub fn chars_after(text: &str, at: usize, n: usize) -> &str {
    let rest = &text[at..];
    let end = rest.char_indices().nth(n).map(|(b, _)| b).unwrap_or(rest.len());
    &rest[..end]
}

/// The text before a byte offset, at most `n` characters (Python `text[max(0, at - n):at]`).
pub fn chars_before(text: &str, at: usize, n: usize) -> &str {
    let head = &text[..at];
    let count = head.chars().count();
    if count <= n {
        return head;
    }
    let start = head.char_indices().nth(count - n).map(|(b, _)| b).unwrap_or(0);
    &head[start..]
}

// ---------------------------------------------------------------------------------------------
// bot/understanding, bot/memory, bot/knowledge/units

// DUP(bot.understanding::line_as_read)
/// The line with the engine's repairs (`plain`), else as typed.
pub fn line_as_read(understood: &Understood) -> String {
    if !text::strip(&understood.plain).is_empty() {
        return understood.plain.clone();
    }
    crate::knowledge::d2_shared::or(&understood.clean, &understood.text).to_string()
}

// DUP(bot.memory::recent)
/// The last n turns, oldest first.
pub fn recent(memory: &Memory, n: usize) -> &[Turn] {
    let turns = &memory.turns;
    &turns[turns.len().saturating_sub(n)..]
}

// DUP(bot.memory::user)
/// The user's facts (a dict), created on first use.
pub fn user(memory: &mut Memory) -> Value {
    match memory.extra.get("user") {
        Some(v @ Value::Dict(_)) => v.clone(),
        _ => {
            let empty = Value::Dict(Arc::new(Vec::new()));
            memory.extra.insert("user".to_string(), empty.clone());
            empty
        }
    }
}

static TABLE_GROUPS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.units._TABLE_GROUPS"));
static GROUP_VALUE: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get("bot.knowledge.units._GROUP_VALUE")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?)))
        .collect()
});

// DUP(bot.knowledge.units::people_in_groups)
/// The people a line counts in groups ("4 adults and 3 kids" is 7), or None.
pub fn people_in_groups(text: &str) -> Option<i64> {
    let mut total = 0;
    for raw in TABLE_GROUPS.findall_str(&text.to_lowercase()) {
        let count = if text::is_digit(&raw) { raw.parse::<i64>().ok() } else { GROUP_VALUE.get(&raw).copied() };
        if let Some(n) = count {
            if n != 0 {
                total += n;
            }
        }
    }
    if total != 0 { Some(total) } else { None }
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking_more.py

// DUP(bot.knowledge.cooking_more::_clean)
pub fn _clean(understood: &Understood) -> String {
    understood.clean.to_lowercase()
}

// DUP(bot.knowledge.cooking_more::_said_words)
pub fn _said_words(understood: &Understood) -> Vec<String> {
    understood.words.iter().map(|w| super::cooking_b::_fold(w)).collect()
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/meals.py

static MEALS_INGREDIENTS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.knowledge.meals.INGREDIENTS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.strs())))
        .collect()
});
static UP_FOOD_SET: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.meals._UP_FOOD_SET"));
static MEALS_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.meals.WORD"));
static USE_DISH_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.meals.USE_DISH_WORDS"));
static UP_CLASSES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.meals.UP_CLASSES")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});
static ASK_IN_NEWS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.meals.ASK_IN_NEWS"));

fn is_alpha_word(s: &str) -> bool {
    text::is_alpha(s)
}

// DUP(bot.knowledge.meals::_up_food)
pub fn _up_food(words: &[String]) -> Option<String> {
    for (index, word) in words.iter().enumerate() {
        if UP_FOOD_SET.contains(word) {
            return Some(word.clone());
        }
        if index + 1 < words.len() {
            let two = words[index..index + 2].join(" ");
            if UP_FOOD_SET.contains(&two) {
                return Some(two);
            }
        }
    }
    None
}

// DUP(bot.knowledge.meals::_named)
/// The ingredients a reading names, phrases first.
pub fn meals_named(understood: &Understood) -> Vec<String> {
    let mut words: Vec<String> = understood.words.clone();
    for word in MEALS_WORD.findall_str(&understood.norm_text) {
        if !words.contains(&word) {
            words.push(word);
        }
    }
    let mut text = words.join(" ");
    let mut found: Vec<String> = Vec::new();
    for (ingredient, forms) in MEALS_INGREDIENTS.iter() {
        let hit: Vec<&String> = forms.iter().filter(|f| !is_alpha_word(f)).filter(|p| text.contains(p.as_str())).collect();
        if !hit.is_empty() {
            found.push(ingredient.clone());
            for phrase in hit {
                text = text.replace(phrase.as_str(), " ");
                words.retain(|w| !phrase.contains(w.as_str()));
            }
        }
    }
    for (ingredient, forms) in MEALS_INGREDIENTS.iter() {
        if found.contains(ingredient) {
            continue;
        }
        if forms.iter().any(|f| is_alpha_word(f) && words.contains(f)) {
            found.push(ingredient.clone());
        }
    }
    found
}

// DUP(bot.knowledge.meals::named_food)
pub fn named_food(words: &[String], understood: Option<&Understood>) -> Option<String> {
    if let Some(up) = _up_food(words) {
        if !up.is_empty() {
            return Some(up);
        }
    }
    let empty = Understood::default();
    let named = meals_named(understood.unwrap_or(&empty));
    named.into_iter().next().filter(|n| !n.is_empty())
}

// DUP(bot.knowledge.meals::idea_is_a_dish)
pub fn idea_is_a_dish(idea: &str) -> bool {
    let low = idea.to_lowercase();
    USE_DISH_WORDS.iter().any(|w| low.contains(w.as_str()))
}

// DUP(bot.knowledge.meals::use_it_up)
pub fn use_it_up(name: &str) -> String {
    let said = text::strip(name);
    let folded = said.to_lowercase();
    for (words, mv) in UP_CLASSES.iter() {
        if words.contains(&folded) {
            return format!("Leftover {said}: {mv}");
        }
    }
    for (words, mv) in UP_CLASSES.iter() {
        if words.iter().any(|w| folded.ends_with(&format!(" {w}")) || folded.starts_with(&format!("{w} "))) {
            return format!("Leftover {said}: {mv}");
        }
    }
    format!(
        "Leftover {said}: already cooked, so treat it as an ingredient - stir through a soup or a curry, fry with an onion and an egg, or eat cold in a salad with oil and vinegar."
    )
}

// DUP(bot.knowledge.meals::reads_as_news)
pub fn reads_as_news(understood: &Understood, clean: &str) -> bool {
    if understood.intent != "news" {
        return false;
    }
    let low = crate::knowledge::d2_shared::or(clean, &understood.clean).to_lowercase();
    !ASK_IN_NEWS.iter().any(|cue| low.contains(cue.as_str()))
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/picking.py, bot/knowledge/dishes.py, bot/knowledge/lookup.py

static PICKING_GUARDS: LazyLock<Vec<Option<data::FnRef>>> = LazyLock::new(|| {
    data::get("bot.knowledge.picking.ENTRIES")
        .items()
        .iter()
        .map(|e| e.attr("guard").and_then(|g| g.as_fn()).cloned())
        .collect()
});

// DUP-D2(bot.knowledge.picking::choosing)
/// Whether any picking entry's guard takes the line: it asks which one to choose. Runs the
/// entries' own guards through the registry.
pub fn choosing(understood: &Understood) -> bool {
    PICKING_GUARDS.iter().any(|g| match g {
        Some(f) => crate::knowledge::registry::guard(f, understood),
        None => panic!("TypeError: 'NoneType' object is not callable"),
    })
}

// NEEDS(bot.knowledge.dishes::dish_in, bot.knowledge.dishes::PLANS)
/// Whether the dish a text names has an oven temperature in its plan. dishes.dish_in is a
/// matching engine of its own (D1); until it is merged no dish is found.
pub fn a_dish_plan_has_a_heat(_text: &str) -> bool {
    false
}

// NEEDS(bot.knowledge.lookup::find with entries=garden.ENTRIES)
/// Whether the garden table answers the line as a reading. Needs lookup.find (U).
pub fn the_garden_finds(_line: &str) -> bool {
    false
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking.py, first half (fork cookA)

static OCCASIONS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_OCCASIONS"));
static ORDINAL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b\d{1,3}(?:st|nd|rd|th)\b", 0));

// DUP-D2(bot.knowledge.cooking::_occasion)
pub fn _occasion(text: &str) -> bool {
    if OCCASIONS.iter().any(|w| text.contains(w.as_str())) {
        return true;
    }
    ORDINAL.is_match(text)
}

static NOT_A_TABLE: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_NOT_A_TABLE"));
static CROWD_NUMBERS: LazyLock<HashMap<String, i64>> = LazyLock::new(|| {
    data::get(&c("_CROWD_NUMBERS"))
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?)))
        .collect()
});
static CROWD_FOR: LazyLock<&'static Re> = LazyLock::new(|| cre("_CROWD_FOR"));
static CROWD_WITHIN: LazyLock<&'static Re> = LazyLock::new(|| cre("_CROWD_WITHIN"));
static CROWD_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CROWD_PHRASES"));
static NAMED_TABLE: LazyLock<&'static Re> = LazyLock::new(|| cre("_NAMED_TABLE"));
static TABLE_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_TABLE_PHRASES"));
static CROWD_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CROWD_VERBS"));
static CROWD_NOT_A_MEAL: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CROWD_NOT_A_MEAL"));
static CROWD_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CROWD_FOOD"));
static CROWD_QUANTITY: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_CROWD_QUANTITY"));
static CROWD_WORDS: LazyLock<HashMap<i64, String>> = LazyLock::new(|| {
    data::get(&c("_CROWD_WORDS"))
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_int()?, v.as_str()?.to_string())))
        .collect()
});
static FOR_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfor\s+(\d{1,3})\b", 0));
static NUMBER_OF_PEOPLE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,3})\s+(?:people|of us|adults|guests|coming)\b", 0));

// DUP-D2(bot.knowledge.cooking::_people_after)
/// Whether the word after a number (from byte offset `end`) can be people, not minutes.
pub fn _people_after(text: &str, end: usize) -> bool {
    let rest = text::split(text::strip(&text[end..]));
    let after = rest.first().map(|w| text::strip_chars(w, ".,!?;:").to_lowercase()).unwrap_or_default();
    !NOT_A_TABLE.contains(&after)
}

// DUP-D2(bot.knowledge.cooking::_crowd)
/// How many people a line says it is cooking for (0 when it does not say).
pub fn _crowd(text: &str) -> i64 {
    let text = text.to_lowercase().replace('-', " ");
    let mut numbers: Vec<i64> = FOR_NUMBER
        .finditer(&text)
        .into_iter()
        .filter(|m| _people_after(&text, m.end()))
        .filter_map(|m| m.str(1).parse().ok())
        .collect();
    for m in CROWD_FOR.finditer(&text) {
        if _people_after(&text, m.end()) {
            numbers.push(*CROWD_NUMBERS.get(m.str(1)).expect("KeyError"));
        }
    }
    for word in CROWD_WITHIN.findall_str(&text) {
        numbers.push(*CROWD_NUMBERS.get(&word).expect("KeyError"));
    }
    for count in NUMBER_OF_PEOPLE.findall_str(&text) {
        numbers.push(count.parse().unwrap_or(0));
    }
    if let Some(grouped) = people_in_groups(&text) {
        numbers.push(grouped);
    }
    if let Some(m) = numbers.iter().max() {
        return *m;
    }
    if CROWD_PHRASES.iter().any(|p| text.contains(p.as_str())) {
        return 6;
    }
    0
}

// DUP-D2(bot.knowledge.cooking::_named_table)
pub fn _named_table(text: &str) -> bool {
    NAMED_TABLE.is_match(text)
}

// DUP-D2(bot.knowledge.cooking::_table_text)
pub fn _table_text(text: &str) -> bool {
    let text = text.to_lowercase();
    let count = _crowd(&text);
    if count >= 6 {
        return true;
    }
    if count == 0 {
        return _named_table(&text);
    }
    if count >= 3 && (CROWD_PHRASES.iter().any(|p| text.contains(p.as_str())) || text.contains("of us")) {
        return true;
    }
    let tabled = TABLE_PHRASES.iter().any(|p| text.contains(p.as_str()));
    if tabled && ["cook", "cooking", "make", "feed", "serve"].iter().any(|w| text.contains(w)) {
        return true;
    }
    count >= 3 && tabled && ["lunch", "lunches", "meal", "meals", "eat"].iter().any(|w| text.contains(w))
}

// DUP-D2(bot.knowledge.cooking::_crowd_ask)
pub fn _crowd_ask(clean: &str) -> bool {
    let text = clean.to_lowercase();
    if !_table_text(&text) {
        return false;
    }
    if CROWD_VERBS.iter().any(|w| text.contains(w.as_str())) {
        return true;
    }
    if CROWD_NOT_A_MEAL.iter().any(|p| text.contains(p.as_str())) {
        return false;
    }
    if CROWD_FOOD.iter().any(|w| text.contains(w.as_str())) {
        return !CROWD_QUANTITY.iter().any(|p| text.contains(p.as_str()));
    }
    false
}

fn crowd_word(count: i64) -> String {
    CROWD_WORDS.get(&count).cloned().unwrap_or_else(|| count.to_string())
}

// DUP-D2(bot.knowledge.cooking::_trays)
pub fn _trays(count: i64) -> String {
    if count >= 13 {
        return "two or three trays of it, on the top and middle shelves, or one dish made in the biggest pot you own"
            .to_string();
    }
    if count >= 9 {
        return format!("two trays of it, on the top and middle shelves and swapped halfway, to feed {}", crowd_word(count));
    }
    if count != 0 && count <= 5 {
        return format!("one tray of it is plenty for {}", crowd_word(count));
    }
    if count == 0 {
        return "which does four to six, so count the potatoes yourself".to_string();
    }
    "which feeds eight from one oven".to_string()
}

const VEGETARIAN_BANS: [&str; 17] = [
    "meat", "chicken", "beef", "pork", "lamb", "mince", "bacon", "sausage", "sausages", "ham", "turkey", "fish",
    "salmon", "cod", "tuna", "prawn", "prawns",
];

// DUP-D2(bot.knowledge.cooking::_crowd_meal)
pub fn _crowd_meal(banned: &HashSet<String>, occasion: bool, count: i64) -> String {
    let vegetarian = VEGETARIAN_BANS.iter().any(|w| banned.contains(*w));
    let (dish, amount) = if vegetarian {
        (
            "a big mushroom and lentil lasagne or a chickpea and vegetable traybake, made in the largest dish you own",
            "about 300g of the dish per person",
        )
    } else {
        (
            "a tray of chicken thighs or sausages with halved potatoes, peppers and onions at 200C (180C fan) for 45 minutes",
            "about 200g of meat and a big potato each",
        )
    };
    let mut reply = format!(
        "Cook one big oven dish rather than several pans: {dish}, {}. Allow {amount}, add a loaf of bread and a bowl of salad, and write the times down so nothing needs the oven at once.",
        _trays(count)
    );
    if occasion {
        reply.push_str(" Anything that can be made that morning - a chilli, a stew, a lasagne - just gets reheated, which leaves the day free.");
    }
    reply
}

// DUP-D2(bot.knowledge.cooking::_same)
/// The word is the wanted one or starts with it ("pancakes" for "pancake").
pub fn _same(word: &str, wanted: &str) -> bool {
    !word.is_empty() && !wanted.is_empty() && (word == wanted || word.starts_with(wanted))
}

// DUP-D2(bot.knowledge.cooking::_said_so_far)
/// The line as read, after the person's last four lines.
pub fn _said_so_far(understood: &Understood, memory: &Memory) -> String {
    let line = line_as_read(understood);
    let turns = &memory.turns;
    let mut parts: Vec<String> = turns[turns.len().saturating_sub(4)..].iter().map(|t| t.user.clone()).collect();
    parts.push(line);
    parts.join(" ")
}

static DIET_ASKED: LazyLock<&'static Re> = LazyLock::new(|| cre("_DIET_ASKED"));
static DIET_ASK_NOT_A_FOOD: LazyLock<HashSet<String>> = LazyLock::new(|| cset("_DIET_ASK_NOT_A_FOOD"));

// DUP-D2(bot.knowledge.cooking::_ask_about_the_food)
/// Whether a diet word at byte offset `at` sits in a question about a food ("is this vegan").
pub fn _ask_about_the_food(text: &str, at: usize) -> bool {
    for m in DIET_ASKED.finditer(&text[..at]) {
        if m.end() <= at && !text::split(m.str(1)).iter().any(|w| DIET_ASK_NOT_A_FOOD.contains(w)) {
            return true;
        }
    }
    false
}

static NO_MEAT_DIET: LazyLock<HashSet<String>> = LazyLock::new(|| cset("NO_MEAT_DIET"));
static DAIRY: LazyLock<HashSet<String>> = LazyLock::new(|| cset("DAIRY"));
static EGGS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("EGGS"));
static GLUTEN: LazyLock<HashSet<String>> = LazyLock::new(|| cset("GLUTEN"));
static MEAT: LazyLock<HashSet<String>> = LazyLock::new(|| cset("MEAT"));
static FISH: LazyLock<HashSet<String>> = LazyLock::new(|| cset("FISH"));
static CHEESES: LazyLock<HashSet<String>> = LazyLock::new(|| cset("CHEESES"));
static AVOIDED: LazyLock<&'static Re> = LazyLock::new(|| cre("AVOIDED"));
static FREEFROM: LazyLock<&'static Re> = LazyLock::new(|| cre("FREEFROM"));
static TROUBLE: LazyLock<&'static Re> = LazyLock::new(|| cre("TROUBLE"));
static CONDITION_BANS: LazyLock<Vec<(String, HashSet<String>)>> = LazyLock::new(|| cset_map("CONDITION_BANS"));
static CONDITION_RES: LazyLock<Vec<Re>> = LazyLock::new(|| {
    CONDITION_BANS.iter().map(|(cond, _)| Re::new(&format!(r"\b{}\b", cond.replace(' ', r"\s+")), 0)).collect()
});
static FAMILIES: LazyLock<HashMap<String, HashSet<String>>> =
    LazyLock::new(|| cset_map("FAMILIES").into_iter().collect());
static LIST_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("LIST_JOINERS"));
static LIST_FOODS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("LIST_FOODS"));
static NOT_THIS: LazyLock<&'static Re> = LazyLock::new(|| cre("NOT_THIS"));
static VEGETARIAN_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:vegetarian|veggie|vego)\b", 0));
static VEGAN_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:vegan|plant based|plantbased)\b", 0));
static LIST_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+|[&+,]", 0));

// DUP-D2(bot.knowledge.cooking::_excluded)
/// The food words a line rules out for the person ("im vegan", "no nuts", "gluten free"),
/// with a later "not" or "actually i do" turning one back on.
pub fn _excluded(text: &str) -> HashSet<String> {
    let text = text.to_lowercase().replace('\u{2019}', "'").replace('-', " ");
    // (char position, families, positive)
    let mut events: Vec<(usize, HashSet<String>, bool)> = Vec::new();
    let pos = |b: usize| text::char_index(&text, b);
    for m in VEGETARIAN_WORD.finditer(&text) {
        if _ask_about_the_food(&text, m.start()) {
            continue;
        }
        events.push((pos(m.start()), NO_MEAT_DIET.clone(), !_denied_at(&text, m.start())));
    }
    for m in VEGAN_WORD.finditer(&text) {
        if _ask_about_the_food(&text, m.start()) {
            continue;
        }
        let mut all = NO_MEAT_DIET.clone();
        all.extend(DAIRY.iter().cloned());
        all.extend(EGGS.iter().cloned());
        events.push((pos(m.start()), all, !_denied_at(&text, m.start())));
    }
    for (which, pattern) in [(0, *AVOIDED), (1, *FREEFROM), (2, *TROUBLE)] {
        for m in pattern.finditer(&text) {
            let word = (1..=m.group_count()).filter_map(|i| m.get(i)).find(|p| !p.is_empty()).unwrap_or("");
            if word.is_empty() {
                continue;
            }
            let families = _families_of(word);
            let denied = _denied_at(&text, m.start());
            events.push((pos(m.start()), families, !denied));
            let after = chars_after(&text, m.end(), 80);
            events.push((pos(m.start()), _foods_in_list(&_list_words(after, false)), !denied));
            if which == 1 {
                let before = chars_before(&text, m.start(), 80);
                events.push((pos(m.start()), _foods_in_list(&_list_words(before, true)), !denied));
            }
        }
    }
    for ((_, foods), rx) in CONDITION_BANS.iter().zip(CONDITION_RES.iter()) {
        for m in rx.finditer(&text) {
            events.push((pos(m.start()), foods.clone(), !_denied_at(&text, m.start())));
        }
    }
    events.sort_by_key(|e| e.0);
    let mut state: HashMap<String, bool> = HashMap::new();
    for (_, families, positive) in events {
        for word in families {
            state.insert(word, positive);
        }
    }
    let mut banned: HashSet<String> = state.into_iter().filter(|(_, on)| *on).map(|(w, _)| w).collect();
    if banned.contains("cheese") || meets(&DAIRY, &banned) {
        banned.extend(CHEESES.iter().cloned());
    }
    banned
}

static LACTOSE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_LACTOSE_WORDS"));
static NOT_LACTOSE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_NOT_LACTOSE_WORDS"));

// DUP-D2(bot.knowledge.cooking::_a_lactose_intolerance)
pub fn _a_lactose_intolerance(said: &str) -> bool {
    if !LACTOSE_WORDS.iter().any(|w| said.contains(w.as_str())) {
        return false;
    }
    let plain = said.replace('\u{2019}', "'");
    !NOT_LACTOSE_WORDS.iter().any(|w| plain.contains(w.as_str()))
}

/// Python `str(x)` of a stored fact value (strings as they are, numbers printed).
fn value_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        _ => String::new(),
    }
}

/// Iterate a stored value as Python would (`for x in value`): a string by characters.
fn value_iter(v: &Value) -> Vec<String> {
    match v {
        Value::Str(s) => s.chars().map(|c| c.to_string()).collect(),
        Value::List(_) | Value::Tuple(_) | Value::Set(_) => v.items().iter().map(value_str).collect(),
        Value::Dict(d) => d.iter().map(|(k, _)| value_str(k)).collect(),
        _ => Vec::new(),
    }
}

// DUP-D2(bot.knowledge.cooking::_record_says_lactose)
pub fn _record_says_lactose(known: &Value) -> bool {
    if !matches!(known, Value::Dict(_)) {
        return false;
    }
    let mut said: Vec<String> = Vec::new();
    for key in ["allergies", "diet", "diets", "condition", "conditions"] {
        match known.get(key) {
            Some(Value::Str(s)) => said.push(s.to_string()),
            Some(v @ (Value::List(_) | Value::Tuple(_) | Value::Set(_))) => said.extend(v.items().iter().map(value_str)),
            _ => {}
        }
    }
    _a_lactose_intolerance(&said.join(" ").to_lowercase().replace('-', " "))
}

// DUP-D2(bot.knowledge.cooking::_their_diet_is_a_lactose_intolerance)
pub fn _their_diet_is_a_lactose_intolerance(memory: &mut Memory) -> bool {
    let known = user(memory);
    _record_says_lactose(&known)
}

// DUP-D2(bot.knowledge.cooking::_families_of)
pub fn _families_of(word: &str) -> HashSet<String> {
    if word == "lactose" {
        return DAIRY.clone();
    }
    if word == "wheat" {
        return GLUTEN.clone();
    }
    FAMILIES.get(word).cloned().unwrap_or_else(|| HashSet::from([word.to_string()]))
}

// DUP-D2(bot.knowledge.cooking::_foods_in_list)
pub fn _foods_in_list(words: &[String]) -> HashSet<String> {
    let mut found = HashSet::new();
    for word in words {
        let word = text::strip_chars(word, ".,!?;:'");
        if word.is_empty() || LIST_JOINERS.contains(word) {
            continue;
        }
        if !LIST_FOODS.contains(word) {
            break;
        }
        found.extend(_families_of(word));
    }
    found
}

// DUP-D2(bot.knowledge.cooking::_list_words)
pub fn _list_words(text: &str, backwards: bool) -> Vec<String> {
    let mut words = LIST_WORD.findall_str(&text.to_lowercase());
    if backwards {
        words.reverse();
    }
    words
}

// DUP-D2(bot.knowledge.cooking::_denied_at)
/// Whether a "not"/"no longer" sits in the 30 characters before byte offset `start`.
pub fn _denied_at(text: &str, start: usize) -> bool {
    NOT_THIS.is_match(chars_before(text, start, 30))
}

static MADE_OF_KEYS: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::get(&c("MADE_OF")).pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect());
static LETTER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

// DUP-D2(bot.knowledge.cooking::_food_named)
/// The food a line names that the MADE_OF table knows, two-word names first.
pub fn _food_named(text: &str) -> Option<String> {
    let words = LETTER_WORDS.findall_str(&text.to_lowercase());
    for size in [2usize, 1] {
        if words.len() < size {
            continue;
        }
        for start in 0..=(words.len() - size) {
            let phrase = words[start..start + size].join(" ");
            if MADE_OF_KEYS.contains(&phrase) {
                return Some(phrase);
            }
        }
    }
    None
}

static SINGULAR_IN_S: LazyLock<HashSet<String>> = LazyLock::new(|| cset("SINGULAR_IN_S"));

// DUP-D2(bot.knowledge.cooking::_is_plural)
pub fn _is_plural(food: &str) -> bool {
    let word = text::split(food).last().cloned().expect("IndexError: list index out of range");
    if SINGULAR_IN_S.contains(&word) || word.ends_with("ss") {
        return false;
    }
    word.ends_with('s')
}

static FREE_OF: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::get(&c("FREE_OF")).pairs().iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect());
static AFTER_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s-]+([a-z']+)", 0));
static FREE_BEFORE: LazyLock<Re> = LazyLock::new(|| Re::new(r"([a-z]+)\s+free$", 0));
static LAST_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"([a-z']+)$", 0));

// DUP-D2(bot.knowledge.cooking::_named_as)
/// The food as the line names it, with its article and any "x free" qualifier.
pub fn _named_as(text: &str, food: &str) -> String {
    let text = text.to_lowercase();
    let parts: Vec<String> = text::split(food).iter().map(|p| crate::py::re::escape(p)).collect();
    let rx = Re::new(&format!(r"\b{}\b", parts.join(r"\s+")), 0);
    for m in rx.finditer(&text) {
        if let Some(after) = AFTER_WORD.match_(&text[m.end()..]) {
            if MADE_OF_KEYS.contains(after.str(1)) {
                continue;
            }
        }
        let mut before = text::rstrip(&text[..m.start()]).to_string();
        let mut qualifier = String::new();
        if let Some(free) = FREE_BEFORE.search(&before) {
            if FREE_OF.contains(free.str(1)) {
                qualifier = format!("{} free ", free.str(1));
                let cut = free.start();
                before = text::rstrip(&before[..cut]).to_string();
            }
        }
        if let Some(article) = LAST_WORD.search(&before) {
            if ["a", "an", "the"].contains(&article.str(1)) {
                return format!("{} {}{}", article.str(1), qualifier, food);
            }
        }
        return format!("{qualifier}{food}");
    }
    food.to_string()
}

// DUP-D2(bot.knowledge.cooking::_bans_for)
/// Everything the person cannot eat: their stored facts, what they have said of themselves, and
/// a lactose intolerance.
pub fn _bans_for(understood: &Understood, memory: &mut Memory) -> HashSet<String> {
    let mut banned = _bans_from_store(memory);
    banned.extend(_excluded(&_mine_only(&_said_so_far(understood, memory))));
    let line = crate::knowledge::d2_shared::or(&understood.clean, &understood.text).to_lowercase();
    if NOT_LACTOSE_WORDS.iter().any(|w| line.contains(w.as_str())) {
        // an intolerance said not to be one
    } else if _a_lactose_intolerance(&line) || _their_diet_is_a_lactose_intolerance(memory) {
        banned.insert("lactose".to_string());
    }
    banned
}

static THIRD_PERSON: LazyLock<&'static Re> = LazyLock::new(|| cre("THIRD_PERSON"));
static MINE_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?;])\s+|\s*,\s*|\s+and\s+|\s+but\s+", 0));

// DUP-D2(bot.knowledge.cooking::_mine_only)
/// The parts of a text about the speaker, not about someone else.
pub fn _mine_only(text: &str) -> String {
    let parts: Vec<String> = MINE_SPLIT
        .split(text)
        .iter()
        .map(|p| text::strip(p).to_string())
        .filter(|p| !p.is_empty())
        .collect();
    parts.into_iter().filter(|p| THIRD_PERSON.match_(p).is_none()).collect::<Vec<_>>().join(" ")
}

// DUP-D2(bot.knowledge.cooking::_bans_from_store)
pub fn _bans_from_store(memory: &Memory) -> HashSet<String> {
    match memory.extra.get("user") {
        Some(user @ Value::Dict(_)) => _bans_of(user),
        _ => HashSet::new(),
    }
}

// DUP-D2(bot.knowledge.cooking::_bans_of)
pub fn _bans_of(facts: &Value) -> HashSet<String> {
    let mut banned = HashSet::new();
    if !matches!(facts, Value::Dict(_)) {
        return banned;
    }
    if let Some(v) = facts.get("allergies").filter(|v| v.truthy()) {
        for item in value_iter(v) {
            let word = item.to_lowercase();
            match FAMILIES.get(&word).filter(|f| !f.is_empty()) {
                Some(f) => banned.extend(f.iter().cloned()),
                None => {
                    banned.insert(word);
                }
            }
        }
    }
    if let Some(v) = facts.get("condition").filter(|v| v.truthy()) {
        for item in value_iter(v) {
            let said = item.to_lowercase();
            for (condition, foods) in CONDITION_BANS.iter() {
                if said.contains(condition.as_str()) {
                    banned.extend(foods.iter().cloned());
                }
            }
        }
    }
    let diet = facts.get("diet").filter(|v| v.truthy()).map(value_str).unwrap_or_default().to_lowercase();
    if ["vegetarian", "veggie", "vego"].contains(&diet.as_str()) {
        banned.extend(NO_MEAT_DIET.iter().cloned());
    } else if ["vegan", "plant based", "plantbased"].contains(&diet.as_str()) {
        banned.extend(NO_MEAT_DIET.iter().cloned());
        banned.extend(DAIRY.iter().cloned());
        banned.extend(EGGS.iter().cloned());
    } else if let Some(rest) = diet.strip_prefix("no ") {
        let word = text::strip(rest).to_string();
        match FAMILIES.get(&word).filter(|f| !f.is_empty()) {
            Some(f) => banned.extend(f.iter().cloned()),
            None => {
                banned.insert(word);
            }
        }
    }
    banned
}

// DUP-D2(bot.knowledge.cooking::_no_meat_also_means_fish)
pub fn _no_meat_also_means_fish(banned: HashSet<String>) -> HashSet<String> {
    if meets(&MEAT, &banned) {
        let mut b = banned;
        b.extend(FISH.iter().cloned());
        b
    } else {
        banned
    }
}

// DUP-D2(bot.knowledge.cooking::avoids)
/// Everything the person has said they cannot eat, from the store and this chat.
pub fn avoids(memory: &mut Memory, understood: Option<&Understood>) -> HashSet<String> {
    let read = match understood {
        Some(u) => u.clone(),
        None => memory.understood.clone().unwrap_or_default(),
    };
    crate::runtime::guarded(std::panic::AssertUnwindSafe(|| _bans_for(&read, memory))).unwrap_or_default()
}

static ANY_MEAL: LazyLock<String> = LazyLock::new(|| cstring("ANY_MEAL"));

// DUP-D2(bot.knowledge.cooking::_any_meal)
pub fn _any_meal(banned: &HashSet<String>) -> String {
    let eggs = banned.contains("egg") || banned.contains("eggs");
    if meets(&GLUTEN, banned) {
        let egg = if eggs { "" } else { " and an egg" };
        let cheese = if meets(&DAIRY, banned) { "hummus" } else { "cheese" };
        return format!(
            "Ten-minute dinners, all gluten free: rice fried with frozen veg{egg}; a jacket potato with beans and {cheese}; or a chickpea and tomato curry with rice. Tell me what is in the fridge and I will pick one."
        );
    }
    if eggs {
        if meets(&DAIRY, banned) {
            return "Ten-minute dinners: a chickpea and tomato curry with rice; pasta with garlic, chilli and olive oil; or beans on toast with hummus. Tell me what is in the fridge and I will pick one.".to_string();
        }
        return "Ten-minute dinners: a chickpea and tomato curry with rice; pasta with garlic, chilli and olive oil; or beans on toast with a little cheese or hummus. Tell me what is in the fridge and I will pick one.".to_string();
    }
    if MEAT.iter().all(|m| banned.contains(m)) {
        return "Ten-minute dinners: pasta with garlic, chilli and olive oil; a chickpea and tomato curry with rice; or eggs on toast. Tell me what is in the fridge and I will pick one.".to_string();
    }
    ANY_MEAL.clone()
}

static LEFTOVER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("LEFTOVER_WORDS"));
static LEFTOVER_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("LEFTOVER_PHRASES"));

// DUP-D2(bot.knowledge.cooking::_leftover)
pub fn _leftover(folded: &[String], clean: &str) -> bool {
    folded.iter().any(|w| LEFTOVER_WORDS.contains(w)) || LEFTOVER_PHRASES.iter().any(|p| clean.contains(p.as_str()))
}

static ORDER_NOT_A_MEAL: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_ORDER_NOT_A_MEAL"));
static MILK_IN_TEA_NOT_A_MEAL: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_MILK_IN_TEA_NOT_A_MEAL"));

// DUP-D2(bot.knowledge.cooking::_a_brew_not_a_dinner)
pub fn _a_brew_not_a_dinner(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    if ORDER_NOT_A_MEAL.iter().any(|p| low.contains(&format!(" {p} "))) {
        return true;
    }
    if !low.contains(" milk ") && !low.starts_with(" milk") {
        return false;
    }
    MILK_IN_TEA_NOT_A_MEAL.iter().any(|p| low.contains(&format!(" {p} ")) || low.contains(&format!(" {p}.")))
}

// DUP-D2(bot.knowledge.cooking::_the_garden_has_this_plant)
pub fn _the_garden_has_this_plant(line: &str) -> bool {
    the_garden_finds(line)
}

static PLANT_WOODY: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PLANT_WOODY"));
static PLANT_PURCHASE: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PLANT_PURCHASE"));
static PLANT_NOT_A_DINNER: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PLANT_NOT_A_DINNER"));
static PLANT_KITCHEN_PLANT: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PLANT_KITCHEN_PLANT"));
static PLANT_GARDEN_VERB: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PLANT_GARDEN_VERB"));
static PLANT_BEARS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PLANT_BEARS"));
static PLANT_COOK_ASK: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PLANT_COOK_ASK"));
static PLANT_DO_WITH: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PLANT_DO_WITH"));
static PLANT_SOURCE: LazyLock<&'static Re> = LazyLock::new(|| cre("_PLANT_SOURCE"));

fn spaced_in(low: &str, words: &[String]) -> bool {
    words.iter().any(|w| low.contains(&format!(" {w} ")))
}

// DUP-D2(bot.knowledge.cooking::_a_plant_from_the_shop)
pub fn _a_plant_from_the_shop(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase().replace('\'', " "));
    spaced_in(&low, &PLANT_WOODY) && spaced_in(&low, &PLANT_PURCHASE)
}

// DUP-D2(bot.knowledge.cooking::_a_plant_not_a_dinner)
pub fn _a_plant_not_a_dinner(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase().replace('\'', " "));
    if !spaced_in(&low, &PLANT_NOT_A_DINNER) {
        return false;
    }
    if spaced_in(&low, &PLANT_KITCHEN_PLANT)
        && ["bought", "buy", "buying", "got", "picked up"].iter().any(|w| low.contains(&format!(" {w} ")))
    {
        return false;
    }
    if spaced_in(&low, &PLANT_GARDEN_VERB) {
        return _the_garden_has_this_plant(line) || _a_plant_from_the_shop(line);
    }
    if spaced_in(&low, &PLANT_BEARS) {
        return false;
    }
    let named: usize = PLANT_NOT_A_DINNER.iter().map(|w| text::count(&low, &format!(" {w} "))).sum();
    let asked = spaced_in(&low, &PLANT_COOK_ASK) || spaced_in(&low, &PLANT_DO_WITH);
    if named > 0 && asked && PLANT_SOURCE.finditer(&low.replace("  ", " ")).len() >= named {
        return false;
    }
    if spaced_in(&low, &PLANT_COOK_ASK) {
        return false;
    }
    _the_garden_has_this_plant(line) || _a_plant_from_the_shop(line)
}

static FEEDING_NOT_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("FEEDING_NOT_FOOD"));
static FEEDING_EATS: LazyLock<&'static Re> = LazyLock::new(|| cre("FEEDING_EATS"));
static FEEDING_GIVEN: LazyLock<&'static Re> = LazyLock::new(|| cre("FEEDING_GIVEN"));
static FEEDING_SAFE_FOR: LazyLock<&'static Re> = LazyLock::new(|| cre("FEEDING_SAFE_FOR"));

// DUP-D2(bot.knowledge.cooking::_feeding_ask)
pub fn _feeding_ask(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    if FEEDING_NOT_FOOD.iter().any(|w| low.contains(w.as_str())) {
        return false;
    }
    FEEDING_EATS.is_match(&low) || FEEDING_GIVEN.is_match(&low) || FEEDING_SAFE_FOR.is_match(&low)
}

static PET_ANIMALS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("PET_ANIMALS"));
static PET_OWNED: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PET_OWNED"));
static PET_EATING: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("_PET_EATING"));
static PET_NOT_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("PET_NOT_FOOD"));
static PET_MEAT_ANIMALS: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("PET_MEAT_ANIMALS"));
static PET_HOME: LazyLock<&'static Re> = LazyLock::new(|| cre("PET_HOME"));
static PET_COOKING_ASK: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("PET_COOKING_ASK"));
static PET_CARE: LazyLock<Vec<String>> = LazyLock::new(|| cstrs("PET_CARE"));

// DUP-D2(bot.knowledge.cooking::_pet_not_dinner)
pub fn _pet_not_dinner(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    let animals: Vec<&String> = PET_ANIMALS.iter().filter(|a| low.contains(&format!(" {a} "))).collect();
    if animals.is_empty() {
        return false;
    }
    if animals.iter().any(|a| PET_OWNED.iter().any(|o| low.contains(&format!(" {o}{a} ")))) {
        return true;
    }
    PET_EATING.iter().any(|v| low.contains(&format!(" {v} ")))
}

// DUP-D2(bot.knowledge.cooking::_reads_as_a_pet)
pub fn _reads_as_a_pet(line: &str) -> bool {
    let low = format!(" {} ", line.to_lowercase());
    if PET_NOT_FOOD.iter().any(|m| low.contains(m.as_str())) {
        return false;
    }
    if _pet_not_dinner(line) {
        return true;
    }
    let owned = PET_MEAT_ANIMALS.iter().any(|a| low.contains(&format!("my {a} ")) || low.contains(&format!("our {a} ")));
    owned && PET_HOME.is_match(&low)
}

// DUP-D2(bot.knowledge.cooking::_pet_or_meat)
/// A pet named as if it were a dinner: keep it off the plate, and give the meat's idea.
pub fn _pet_or_meat(line: &str) -> Option<String> {
    let low = line.to_lowercase();
    if !PET_COOKING_ASK.iter().any(|w| low.contains(w.as_str())) {
        return None;
    }
    if PET_CARE.iter().any(|w| low.contains(w.as_str())) {
        return None;
    }
    if !_reads_as_a_pet(&low) {
        return None;
    }
    for (name, (_aliases, idea)) in super::cooking_b::PANTRY.iter() {
        if !PET_ANIMALS.contains(name) && !PET_MEAT_ANIMALS.contains(name) {
            continue;
        }
        if !low.contains(&format!(" {name} ")) && !low.contains(&format!(" {name}s ")) {
            continue;
        }
        let meat = idea.as_str();
        let (head, _, rest) = text::partition(meat, ": ");
        let mut rest = rest;
        if rest.is_empty()
            || head.is_empty()
            || !name.contains(text::split(&head.to_lowercase()).first().expect("IndexError: list index out of range").as_str())
        {
            rest = meat;
        }
        let rest = text::strip(rest);
        if rest.is_empty() {
            return None;
        }
        let mut chars = rest.chars();
        let first = chars.next().unwrap();
        return Some(format!(
            "That reads as a pet rather than a dinner, and I would keep it off the plate. If you meant the meat, {}{}",
            first.to_lowercase(),
            chars.as_str()
        ));
    }
    None
}

// DUP-D2(bot.knowledge.cooking::_porridge)
/// The porridge a diet allows (BREAKFAST_CHOICES' first idea).
pub fn _porridge(banned: &HashSet<String>) -> String {
    if meets(&GLUTEN, banned) {
        let milk = if meets(&DAIRY, banned) { "oat milk" } else { "milk" };
        return format!("Porridge made with gluten free oats and {milk}");
    }
    if meets(&DAIRY, banned) {
        return "Porridge made with oat milk and a spoon of jam".to_string();
    }
    "Porridge with a banana and a spoon of jam".to_string()
}
