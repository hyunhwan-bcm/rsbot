//! Helpers the g6 modules (health, cooking_entries, ...) need from code other groups or
//! workstreams own. Each is marked `DUP(...)` (another workstream), `DUP-D2(...)` (another D2
//! group: swap for a direct call at merge) or `NEEDS(...)` (a stub until the owner's port lands).

use crate::data::{self, Value};
use crate::py::re::{escape, Re};
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

fn str_sets(name: &str) -> IndexMap<String, HashSet<String>> {
    data::get(name).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
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
pub fn dish_in(_text: &str) -> Option<String> {
    None
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking.py (owned by the cookA/cookB groups)

fn ck(name: &str) -> String {
    format!("bot.knowledge.cooking.{name}")
}

static C_CROWD_NUMBERS: LazyLock<IndexMap<String, i64>> = LazyLock::new(|| {
    data::get(&ck("_CROWD_NUMBERS")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap_or(0))).collect()
});
static C_CROWD_FOR: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("_CROWD_FOR")));
static C_CROWD_WITHIN: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("_CROWD_WITHIN")));
static C_CROWD_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&ck("_CROWD_PHRASES")));
static C_NOT_A_TABLE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&ck("_NOT_A_TABLE")));
static C_FOR_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bfor\s+(\d{1,3})\b", 0));
static C_NUMBER_PEOPLE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,3})\s+(?:people|of us|adults|guests|coming)\b", 0));
static C_FAMILIES: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| str_sets(&ck("FAMILIES")));
static C_CONDITION_BANS: LazyLock<IndexMap<String, HashSet<String>>> = LazyLock::new(|| str_sets(&ck("CONDITION_BANS")));
static C_CONDITION_RES: LazyLock<Vec<(Re, HashSet<String>)>> = LazyLock::new(|| {
    C_CONDITION_BANS
        .iter()
        .map(|(c, foods)| (Re::new(&format!(r"\b{}\b", c.replace(' ', r"\s+")), 0), foods.clone()))
        .collect()
});
static C_NO_MEAT_DIET: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("NO_MEAT_DIET")));
static C_DAIRY: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("DAIRY")));
static C_EGGS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("EGGS")));
static C_FISH: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("FISH")));
static C_MEAT: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("MEAT")));
static C_CHEESES: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("CHEESES")));
static C_GLUTEN: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("GLUTEN")));
static C_AVOIDED: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("AVOIDED")));
static C_FREEFROM: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("FREEFROM")));
static C_TROUBLE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("TROUBLE")));
static C_LIST_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("LIST_JOINERS")));
static C_LIST_FOODS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("LIST_FOODS")));
static C_NOT_THIS: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("NOT_THIS")));
static C_DIET_ASKED: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("_DIET_ASKED")));
static C_DIET_ASK_NOT_A_FOOD: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(&ck("_DIET_ASK_NOT_A_FOOD")));
static C_VEGETARIAN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:vegetarian|veggie|vego)\b", 0));
static C_VEGAN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:vegan|plant based|plantbased)\b", 0));
static C_LIST_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+|[&+,]", 0));
static C_ANY_LUNCH: LazyLock<String> = LazyLock::new(|| data::string(&ck("ANY_LUNCH")));
static C_LACKS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&ck("LACKS")).items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].strs())).collect()
});
static C_LACK_SAID: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&ck("LACK_SAID"))
        .items()
        .iter()
        .map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string()))
        .collect()
});
static C_LACK_ATTACHED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&ck("LACK_ATTACHED")));
static C_COOKING_ASKS: LazyLock<Vec<Re>> = LazyLock::new(|| {
    data::strs(&ck("COOKING_ASKS")).iter().map(|one| Re::new(&format!(r"\b{}\b", escape(one)), 0)).collect()
});
static C_LISTING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&ck("LISTING")));
static C_HAVE_ONE: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| {
    data::get(&ck("HAVE_ONE")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static C_ONLY_ONE: LazyLock<IndexMap<String, Vec<String>>> = LazyLock::new(|| {
    data::get(&ck("ONLY_ONE")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static C_GRIDDLE_BAKES: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("GRIDDLE_BAKES")));
static C_GRIDDLE_BATTER: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("GRIDDLE_BATTER")));
static C_GRIDDLE_DOUGH: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("GRIDDLE_DOUGH")));
static C_GRIDDLE_TAIL: LazyLock<String> = LazyLock::new(|| data::string(&ck("_GRIDDLE_TAIL")));
static C_GRIDDLE_WARM: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:wrap|wraps|tortilla|tortillas|pitta|pittas|pita|pitas)\b", 0));
static C_SWEET_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("SWEET_ASK")));
static C_BREAD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("BREAD")));
static C_CAKEISH: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("CAKEISH")));
static C_CROWD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&ck("CROWD")));

fn cs(name: &str) -> String {
    data::string(&ck(name))
}

// DUP-D2(bot.knowledge.cooking::_line)
/// The line and its normalised form, lower-cased.
pub fn _line(u: &Understood) -> String {
    format!("{} {}", u.clean, u.norm_text).to_lowercase()
}

// DUP-D2(bot.knowledge.cooking::_people_after)
fn _people_after(text: &str, end: usize) -> bool {
    let rest = text::split(&text[end..]);
    let after = rest.first().map(|w| text::strip_chars(w, ".,!?;:").to_lowercase()).unwrap_or_default();
    !C_NOT_A_TABLE.contains(&after)
}

// DUP-D2(bot.knowledge.cooking::_crowd)
/// How many people the line cooks for (6 for a crowd said in words), 0 when it says none.
pub fn _crowd(text: &str) -> i64 {
    let text = text.to_lowercase().replace('-', " ");
    let mut numbers: Vec<i64> = Vec::new();
    for m in C_FOR_NUMBER.finditer(&text) {
        if _people_after(&text, m.end()) {
            numbers.push(m.str(1).parse().unwrap_or(0));
        }
    }
    for m in C_CROWD_FOR.finditer(&text) {
        if _people_after(&text, m.end()) {
            numbers.push(C_CROWD_NUMBERS[m.str(1)]);
        }
    }
    for word in C_CROWD_WITHIN.findall_str(&text) {
        numbers.push(C_CROWD_NUMBERS[word.as_str()]);
    }
    for count in C_NUMBER_PEOPLE.findall_str(&text) {
        numbers.push(count.parse().unwrap_or(0));
    }
    if let Some(grouped) = people_in_groups(&text) {
        numbers.push(grouped);
    }
    if let Some(m) = numbers.iter().max() {
        return *m;
    }
    if C_CROWD_PHRASES.iter().any(|p| text.contains(p.as_str())) {
        return 6;
    }
    0
}

// DUP-D2(bot.knowledge.cooking::_bans_of)
/// The foods a person's facts rule out: allergies, conditions and diet.
pub fn _bans_of(facts: Option<&Value>) -> HashSet<String> {
    let mut banned = HashSet::new();
    let Some(facts @ Value::Dict(_)) = facts else { return banned };
    let family = |word: &str| -> HashSet<String> {
        match C_FAMILIES.get(word) {
            Some(f) if !f.is_empty() => f.clone(),
            _ => [word.to_string()].into_iter().collect(),
        }
    };
    for item in facts.get("allergies").map(|v| v.items()).unwrap_or(&[]) {
        banned.extend(family(&py_str(item).to_lowercase()));
    }
    for item in facts.get("condition").map(|v| v.items()).unwrap_or(&[]) {
        let said = py_str(item).to_lowercase();
        for (condition, foods) in C_CONDITION_BANS.iter() {
            if said.contains(condition.as_str()) {
                banned.extend(foods.iter().cloned());
            }
        }
    }
    let diet = facts.get("diet").filter(|v| v.truthy()).map(py_str).unwrap_or_default().to_lowercase();
    if matches!(diet.as_str(), "vegetarian" | "veggie" | "vego") {
        banned.extend(C_NO_MEAT_DIET.iter().cloned());
    } else if matches!(diet.as_str(), "vegan" | "plant based" | "plantbased") {
        banned.extend(C_NO_MEAT_DIET.iter().cloned());
        banned.extend(C_DAIRY.iter().cloned());
        banned.extend(C_EGGS.iter().cloned());
    } else if let Some(rest) = diet.strip_prefix("no ") {
        banned.extend(family(text::strip(rest)));
    }
    banned
}

// DUP-D2(bot.knowledge.cooking::_family_of)
pub fn _family_of(word: &str) -> HashSet<String> {
    let word = text::strip(word).to_lowercase();
    if word.is_empty() {
        return HashSet::new();
    }
    let trimmed = word.strip_suffix('s').unwrap_or(&word).to_string();
    for key in [word.clone(), trimmed] {
        if let Some(f) = C_FAMILIES.get(&key) {
            return f.clone();
        }
    }
    [word].into_iter().collect()
}

// DUP-D2(bot.knowledge.cooking::avoids_for_the_one_talked_about)
/// The allergies kept for the person the line is about (memory["person"]["who"]).
pub fn avoids_for_the_one_talked_about(memory: &Memory) -> HashSet<String> {
    let mut banned = HashSet::new();
    let Some(person @ Value::Dict(_)) = mem(memory, "person") else { return banned };
    let who = person.get("who").filter(|v| v.truthy()).map(py_str).unwrap_or_default();
    let who = text::strip(&who).to_lowercase();
    if who.is_empty() {
        return banned;
    }
    for record in mem(memory, "others").map(|v| v.items()).unwrap_or(&[]) {
        if !matches!(record, Value::Dict(_)) {
            continue;
        }
        let theirs = record.get("who").filter(|v| v.truthy()).map(py_str).unwrap_or_default();
        if text::strip(&theirs).to_lowercase() != who {
            continue;
        }
        for item in record.get("allergies").map(|v| v.items()).unwrap_or(&[]) {
            let word = if item.truthy() { py_str(item) } else { String::new() };
            banned.extend(_family_of(&word));
        }
        break;
    }
    banned
}

// DUP-D2(bot.knowledge.cooking::_said_so_far)
/// This line with up to four earlier lines of theirs before it.
pub fn _said_so_far(u: &Understood, memory: &Memory) -> String {
    let line = line_as_read(u);
    let start = memory.turns.len().saturating_sub(4);
    let mut parts: Vec<String> = memory.turns[start..].iter().map(|t| t.user.clone()).collect();
    parts.push(line);
    parts.join(" ")
}

// DUP-D2(bot.knowledge.cooking::_no_meat_also_means_fish)
pub fn _no_meat_also_means_fish(banned: HashSet<String>) -> HashSet<String> {
    if banned.iter().any(|w| C_MEAT.contains(w)) {
        let mut b = banned;
        b.extend(C_FISH.iter().cloned());
        b
    } else {
        banned
    }
}

fn _families_of(word: &str) -> HashSet<String> {
    if word == "lactose" {
        return C_DAIRY.clone();
    }
    if word == "wheat" {
        return C_GLUTEN.clone();
    }
    C_FAMILIES.get(word).cloned().unwrap_or_else(|| [word.to_string()].into_iter().collect())
}

fn _foods_in_list(words: &[String]) -> HashSet<String> {
    let mut found = HashSet::new();
    for word in words {
        let word = text::strip_chars(word, ".,!?;:'");
        if word.is_empty() || C_LIST_JOINERS.contains(word) {
            continue;
        }
        if !C_LIST_FOODS.contains(word) {
            break;
        }
        found.extend(_families_of(word));
    }
    found
}

fn _list_words(text: &str, backwards: bool) -> Vec<String> {
    let mut words = C_LIST_WORD.findall_str(&text.to_lowercase());
    if backwards {
        words.reverse();
    }
    words
}

/// `text[max(0, start - n):start]` with Python's character arithmetic, for a byte offset.
fn chars_before(text: &str, start: usize, n: usize) -> &str {
    let ci = text::char_index(text, start);
    let from = text::byte_at(text, ci.saturating_sub(n));
    &text[from..start]
}

/// `text[end:end + n]` with Python's character arithmetic, for a byte offset.
fn chars_after(text: &str, end: usize, n: usize) -> &str {
    let ci = text::char_index(text, end);
    let to = text::byte_at(text, ci + n);
    &text[end..to]
}

fn _denied_at(text: &str, start: usize) -> bool {
    C_NOT_THIS.is_match(chars_before(text, start, 30))
}

fn _ask_about_the_food(text: &str, at: usize) -> bool {
    let head = &text[..at];
    for m in C_DIET_ASKED.finditer(head) {
        if m.end() <= at && !m.str(1).split_whitespace().any(|w| C_DIET_ASK_NOT_A_FOOD.contains(w)) {
            return true;
        }
    }
    false
}

// DUP-D2(bot.knowledge.cooking::_excluded)
/// The foods the text rules out (diets, "no X", "X free", allergies, conditions), last word wins.
pub fn _excluded(text: &str) -> HashSet<String> {
    let text = text.to_lowercase().replace('\u{2019}', "'").replace('-', " ");
    let mut events: Vec<(usize, HashSet<String>, bool)> = Vec::new();
    for m in C_VEGETARIAN.finditer(&text) {
        if _ask_about_the_food(&text, m.start()) {
            continue;
        }
        events.push((m.start(), C_NO_MEAT_DIET.clone(), !_denied_at(&text, m.start())));
    }
    for m in C_VEGAN.finditer(&text) {
        if _ask_about_the_food(&text, m.start()) {
            continue;
        }
        let mut fam = C_NO_MEAT_DIET.clone();
        fam.extend(C_DAIRY.iter().cloned());
        fam.extend(C_EGGS.iter().cloned());
        events.push((m.start(), fam, !_denied_at(&text, m.start())));
    }
    for (which, pattern) in [(0, *C_AVOIDED), (1, *C_FREEFROM), (2, *C_TROUBLE)] {
        for m in pattern.finditer(&text) {
            let word = (1..=m.group_count()).filter_map(|i| m.get(i)).find(|p| !p.is_empty()).unwrap_or("");
            if word.is_empty() {
                continue;
            }
            let families = _families_of(word);
            let denied = _denied_at(&text, m.start());
            events.push((m.start(), families, !denied));
            let after = _list_words(chars_after(&text, m.end(), 80), false);
            events.push((m.start(), _foods_in_list(&after), !denied));
            if which == 1 {
                let before = _list_words(chars_before(&text, m.start(), 80), true);
                events.push((m.start(), _foods_in_list(&before), !denied));
            }
        }
    }
    for (rx, foods) in C_CONDITION_RES.iter() {
        for m in rx.finditer(&text) {
            events.push((m.start(), foods.clone(), !_denied_at(&text, m.start())));
        }
    }
    events.sort_by_key(|e| e.0);
    let mut state: IndexMap<String, bool> = IndexMap::new();
    for (_, families, positive) in &events {
        for word in families {
            state.insert(word.clone(), *positive);
        }
    }
    let mut banned: HashSet<String> = state.into_iter().filter(|(_, on)| *on).map(|(w, _)| w).collect();
    if banned.contains("cheese") || banned.iter().any(|w| C_DAIRY.contains(w)) {
        banned.extend(C_CHEESES.iter().cloned());
    }
    banned
}

// DUP-D2(bot.knowledge.cooking::_lunch_ideas)
/// A packed lunch that fits what the line (and the last few) rule out.
pub fn _lunch_ideas(memory: &Memory, u: &Understood) -> String {
    let text = _said_so_far(u, memory);
    let banned = _no_meat_also_means_fish(_excluded(&text));
    if banned.is_empty() {
        return C_ANY_LUNCH.clone();
    }
    let tuna: HashSet<String> = ["tuna".to_string()].into_iter().collect();
    let ham: HashSet<String> = ["ham".to_string()].into_iter().collect();
    let none: HashSet<String> = HashSet::new();
    let options: [(&str, &HashSet<String>); 7] = [
        ("cheese", &C_DAIRY),
        ("tuna", &tuna),
        ("ham", &ham),
        ("egg", &C_EGGS),
        ("hummus", &none),
        ("avocado", &none),
        ("roasted vegetables", &none),
    ];
    let fillings: Vec<&str> =
        options.iter().filter(|(_, needs)| !needs.iter().any(|n| banned.contains(n))).map(|(n, _)| *n).collect();
    let first3 = |f: &[&str]| f[..f.len().min(3)].join(", ");
    if banned.contains("gluten") {
        if !fillings.is_empty() {
            return format!(
                "For a lunch that is not a sandwich: a rice or potato salad with {}, fruit and a drink. Cook extra dinner and take the leftovers in a tub and it is quicker still. Keep it in a cool bag until lunch.",
                first3(&fillings)
            );
        }
        return "For a lunch that is not a sandwich: cook extra dinner and take the leftovers in a tub with rice, potatoes or salad, fruit and a drink. Keep it in a cool bag until lunch.".to_string();
    }
    if fillings.is_empty() {
        return "Cook extra dinner and take the leftovers in a tub with fruit and something crunchy: that is a packed lunch in five minutes. Keep it in a cool bag until lunch.".to_string();
    }
    let reply = format!(
        "A sandwich or wrap with {}, fruit, something crunchy and a drink is a packed lunch in five minutes. Cook extra dinner and take the leftovers in a tub and it is quicker still.",
        first3(&fillings)
    );
    if !banned.is_empty() {
        return reply + " Keep it in a cool bag until lunch.";
    }
    reply + " Keep anything with meat, fish, egg or dairy cold until you eat it."
}

fn _says_gone(phrases: &[String], segments: &[&str]) -> bool {
    for segment in segments {
        for phrase in phrases {
            if phrase.is_empty() {
                continue;
            }
            for (at, _) in segment.match_indices(phrase.as_str()) {
                let rest = text::strip(&segment[at + phrase.len()..]);
                let first = rest.split(' ').next().unwrap_or("");
                let after = text::strip_chars(first, ".,!?;:");
                if !C_LACK_ATTACHED.iter().any(|w| w == after) {
                    return true;
                }
            }
        }
    }
    false
}

// DUP-D2(bot.knowledge.cooking::missing)
/// The kitchen kit the line says it has not got ("oven", "hob", "microwave", "kettle").
pub fn missing(u: &Understood) -> HashSet<String> {
    let line = _line(u);
    let segments: Vec<&str> = line.split('|').collect();
    let mut gone = HashSet::new();
    for (name, phrases) in C_LACKS.iter() {
        if _says_gone(phrases, &segments) {
            gone.insert(name.clone());
        }
    }
    for (name, phrase) in C_LACK_SAID.iter() {
        if !gone.contains(name) && _says_gone(std::slice::from_ref(phrase), &segments) {
            gone.insert(name.clone());
        }
    }
    gone
}

fn _asks_for_sweet(text: &str) -> bool {
    let text = text.to_lowercase();
    if text.contains("sweet potato") {
        return false;
    }
    C_SWEET_ASK.is_match(&text)
}

fn rstrip_s(s: &str) -> &str {
    s.trim_end_matches('s')
}

// DUP-D2(bot.knowledge.cooking::_griddle_answer)
pub fn _griddle_answer(said: &str) -> String {
    let text = said.to_lowercase();
    let tail = C_GRIDDLE_TAIL.as_str();
    if let Some(batter) = C_GRIDDLE_BATTER.search(&text) {
        return format!(
            "A {} is a pan job as it is: the batter goes into a hot non-stick pan with a little butter, about a minute a side until bubbles pop and the edges set. Crumpets want rings in a dry pan on a low heat until the top sets and the holes stay open. {}",
            rstrip_s(batter.as_str()),
            tail
        );
    }
    if let Some(dough) = C_GRIDDLE_DOUGH.search(&text) {
        let name = dough.as_str();
        if !["scone", "drop scone", "pikelet"].contains(&rstrip_s(name)) {
            return format!(
                "A {} cooks in a heavy dry pan, not a tin: the same dough made a little thicker and patted out, about 5 minutes a side on a low heat with a lid on, until the middle is set and the edges have caught. Scones, pikelets and roti are the same trick. {}",
                rstrip_s(name),
                tail
            );
        }
        return format!(
            "A scone cooks in a heavy pan: the same dough made a little thicker and patted out, in a dry pan on a low heat with a lid on for about 5 minutes a side, until the middle is set and the edges have caught. Drop scones, pikelets, chapatis and roti are the same trick on a hot dry pan. {}",
            tail
        );
    }
    let named = C_GRIDDLE_WARM.search(&text).map(|m| rstrip_s(m.as_str()).to_string()).unwrap_or_else(|| "wrap".into());
    let other =
        if named.starts_with("pitt") { "and a wrap warms the same way" } else { "and a pitta puffs the same way" };
    format!(
        "A {} is warmed rather than baked: 30 seconds a side in a dry hot pan and it is soft enough to fold, {}. {}",
        named, other, tail
    )
}

// DUP-D2(bot.knowledge.cooking::_no_baking)
pub fn _no_baking(said: &str) -> String {
    if C_GRIDDLE_BAKES.is_match(said) {
        return _griddle_answer(said);
    }
    let base = if C_BREAD.is_match(said) && !C_CAKEISH.is_match(said) {
        cs("NO_OVEN_BREAD")
    } else {
        let mut b = cs("NO_BAKING");
        if C_BREAD.is_match(said) {
            b.push(' ');
            b.push_str(&cs("NO_OVEN_BREAD"));
        }
        b
    };
    match C_CROWD.search(said) {
        None => base,
        Some(many) => format!(
            "{base} For {} people make two tins of it and cut it small, or set one big cheesecake in a deep tin and chill it overnight.",
            many.as_str()
        ),
    }
}

// DUP-D2(bot.knowledge.cooking::available)
pub fn available(text: &str, gone: &HashSet<String>) -> HashSet<String> {
    let text = text.to_lowercase();
    for (name, phrases) in C_ONLY_ONE.iter() {
        if phrases.iter().any(|p| text.contains(p.as_str())) {
            return [name.clone()].into_iter().collect();
        }
    }
    let named: HashSet<String> = C_HAVE_ONE
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

// DUP-D2(bot.knowledge.cooking::without)
/// What to cook with the kit the line has left, or None when it does not ask.
pub fn without(u: &Understood) -> Option<String> {
    let gone = missing(u);
    if gone.is_empty() {
        return None;
    }
    // `words` is always a list in a reading
    let text = u.words.join(" ").to_lowercase();
    let asking = C_COOKING_ASKS.iter().any(|rx| rx.is_match(&text));
    let listing = C_LISTING.iter().any(|one| text.contains(one.as_str()));
    if !(asking || listing) {
        let named = ["hob", "microwave", "kettle", "oven"]
            .iter()
            .flat_map(|k| C_HAVE_ONE.get(*k).into_iter().flatten())
            .chain(["hob", "microwave", "kettle"].iter().flat_map(|k| C_ONLY_ONE.get(*k).into_iter().flatten()))
            .any(|one| text.contains(one.as_str()));
        if !(named && text::split_ref(&text).len() <= 8) {
            return None;
        }
    }
    let baking = [
        "bake", "baking", "cake", "cakes", "bread", "pie", "pastry", "pastries", "biscuit", "biscuits", "cookie",
        "cookies", "meringue", "meringues", "yorkshire", "scone", "scones", "loaf", "loaves", "muffin", "muffins",
        "brownies", "flapjack", "flapjacks",
    ];
    let how = if baking.iter().any(|one| text.contains(one)) || _asks_for_sweet(&text) { "bake" } else { "" };
    if gone.contains("oven") && how == "bake" && !gone.contains("hob") {
        return Some(_no_baking(&text));
    }
    if gone.contains("oven") && !gone.contains("hob") && C_GRIDDLE_BAKES.is_match(&text) {
        return Some(_griddle_answer(&text));
    }
    let have = available(&text, &gone);
    if gone.contains("hob") || !have.contains("hob") {
        if have.contains("oven") {
            return Some(cs("NO_OVEN_ONLY"));
        }
        if have.contains("microwave") {
            return Some(cs("NO_HOB"));
        }
        return Some(cs("NO_KETTLE"));
    }
    if gone.contains("oven") || !have.contains("oven") {
        return Some(cs("NO_OVEN"));
    }
    if gone.contains("microwave") && how != "bake" {
        return Some(cs("NO_MICROWAVE"));
    }
    None
}

static C_JACKET_FILL_ASK: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(fill|filling|fillings|topping|toppings|goes with|what with|put in|beans|cheese|tuna|coleslaw|chilli)\b", 0)
});
static C_PEEL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bpeel", 0));
static C_SKIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bskin\b", 0));
static C_POACH_NAMES: LazyLock<Vec<(&'static str, Re)>> = LazyLock::new(|| {
    ["haddock", "cod", "trout", "salmon"].iter().map(|n| (*n, Re::new(&format!(r"\b{n}\b"), 0))).collect()
});
static C_DIP_FIXES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&ck("_DIP_FIXES")).items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
});

// DUP-D2(bot.knowledge.cooking::_jacket)
pub fn _jacket(u: &Understood) -> String {
    let said = u.clean.to_lowercase();
    if C_JACKET_FILL_ASK.is_match(&said) { cs("JACKET_FILL") } else { cs("JACKET_BAKE") }
}

// DUP-D2(bot.knowledge.cooking::_jam_make)
pub fn _jam_make(u: &Understood) -> String {
    let said = u.clean.to_lowercase();
    if said.contains("chutney") || said.contains("preserve") { cs("CHUTNEY_RECIPE") } else { cs("JAM_RECIPE") }
}

// DUP-D2(bot.knowledge.cooking::_poach_fish)
pub fn _poach_fish(u: &Understood) -> String {
    let text = u.clean.to_lowercase();
    let fish = C_POACH_NAMES.iter().find(|(_, rx)| rx.is_match(&text)).map(|(n, _)| *n).unwrap_or("salmon");
    cs("POACH_FISH").replace("{fish}", fish)
}

// DUP-D2(bot.knowledge.cooking::_mushroom_wash)
pub fn _mushroom_wash(u: &Understood) -> String {
    let said = u.clean.to_lowercase();
    if C_PEEL.is_match(&said) || C_SKIN.is_match(&said) { cs("MUSHROOM_PEEL") } else { cs("MUSHROOM_WASH") }
}

// DUP-D2(bot.knowledge.cooking::_dip_fault)
pub fn _dip_fault(u: &Understood) -> String {
    let text = u.clean.to_lowercase();
    if ["houmous", "hummus", "humous"].iter().any(|n| text.contains(n))
        && ["watery", "too thin", "runny", " thin"].iter().any(|w| text.contains(w))
    {
        return "Watery houmous usually has too much liquid or not enough tahini: drain the chickpeas well, then stir in a spoonful of tahini and a little olive oil and chill it for 20 minutes, because it firms up cold. Taste for salt and lemon at the end.".to_string();
    }
    for (names, fix) in C_DIP_FIXES.iter() {
        if names.iter().any(|n| text.contains(n.as_str())) {
            return fix.clone();
        }
    }
    cs("_DIP_PLAIN")
}
