//! Port of PyBot's `bot/knowledge/dishes.py`: dish plans (what a cooking conversation needs after
//! the first answer: heat, time, doneness, keeping, sides, the fault that keeps happening), the
//! idea, dislike, diet and pudding rows, and what a named dish is.
#![allow(non_snake_case)]

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g3 as dup;
use crate::knowledge::d1_dup_g3::tokens;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{BTreeSet, HashSet};
use std::sync::LazyLock;

fn d(name: &str) -> String {
    format!("bot.knowledge.dishes.{name}")
}
fn strs(name: &str) -> Vec<String> {
    data::strs(&d(name))
}
fn set(name: &str) -> HashSet<String> {
    data::set(&d(name))
}
fn rx(name: &str) -> &'static Re {
    data::re(&d(name))
}
fn string(name: &str) -> String {
    data::string(&d(name))
}

/// `any(word in text for word in words)` over owned strings.
fn any_in(text: &str, words: &[String]) -> bool {
    words.iter().any(|w| text.contains(w.as_str()))
}

// ---------------------------------------------------------------------------------------------
// The plans

/// One dish's plan: its names (groups of names), the words that make the name another thing,
/// and its facts by kind ("temp", "time", "done", "keep", ...).
#[derive(Debug)]
pub struct Plan {
    pub key: String,
    pub names: Vec<Vec<String>>,
    /// `tokens(name)` for each name, in the same shape as `names`.
    pub name_tokens: Vec<Vec<Vec<String>>>,
    pub avoid: Vec<String>,
    pub fields: Vec<(String, String)>,
}

impl Plan {
    /// `plan.get(kind)` for a string field ("" is kept as it is; None when absent).
    pub fn get(&self, kind: &str) -> Option<&str> {
        self.fields.iter().find(|(k, _)| k == kind).map(|(_, v)| v.as_str())
    }
    /// `plan.get(kind) or None`.
    pub fn get_truthy(&self, kind: &str) -> Option<&str> {
        self.get(kind).filter(|v| !v.is_empty())
    }
    /// Every (name, its tokens), in table order.
    fn each_name(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.names
            .iter()
            .zip(self.name_tokens.iter())
            .flat_map(|(g, t)| g.iter().zip(t.iter()))
    }
}

/// PLANS: every dish the follow-up rows know, in table order.
pub static PLANS: LazyLock<Vec<Plan>> = LazyLock::new(|| {
    data::get(&d("PLANS"))
        .pairs()
        .iter()
        .map(|(k, v)| {
            let mut plan = Plan {
                key: k.str().to_string(),
                names: Vec::new(),
                name_tokens: Vec::new(),
                avoid: Vec::new(),
                fields: Vec::new(),
            };
            for (fk, fv) in v.pairs() {
                match fk.str() {
                    "names" => {
                        for group in fv.items() {
                            let names: Vec<String> = match group {
                                Value::Str(s) => vec![s.to_string()],
                                _ => group.strs(),
                            };
                            plan.name_tokens.push(names.iter().map(|n| tokens(n)).collect());
                            plan.names.push(names);
                        }
                    }
                    "avoid" => plan.avoid = fv.strs(),
                    other => plan.fields.push((other.to_string(), fv.str().to_string())),
                }
            }
            plan
        })
        .collect()
});

/// `PLANS.get(key)`.
pub fn plan(key: &str) -> Option<&'static Plan> {
    PLANS.iter().find(|p| p.key == key)
}

static DISH_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISH_WORDS"));
static READY_NOTES: LazyLock<Vec<(String, Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&d("READY_NOTES"))
        .pairs()
        .iter()
        .map(|(k, v)| {
            let parts = v.items();
            (k.str().to_string(), parts[0].strs(), parts[1].str().to_string())
        })
        .collect()
});

/// The food among the doneness notes a line names, longest name first, or None.
pub fn _ready_food(text: &str) -> Option<&'static str> {
    let mut best: Option<&'static str> = None;
    let mut best_len = 0;
    for (food, names, _) in READY_NOTES.iter() {
        for name in names {
            let n = name.chars().count();
            if n > best_len && _phrase_in(text, name) {
                best = Some(food.as_str());
                best_len = n;
            }
        }
    }
    best
}

/// The doneness cue for the food the conversation is cooking, or None.
pub fn _ready_note(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    if dish_in(text).is_some() {
        return None;
    }
    let mut food = _ready_food(text);
    if food.is_none() {
        for earlier in recent_texts(memory).iter().take(4) {
            food = _ready_food(earlier);
            if food.is_some() {
                break;
            }
        }
    }
    let food = food?;
    READY_NOTES.iter().find(|(k, _, _)| k == food).map(|(_, _, note)| note.clone())
}

static SIDES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&d("SIDES")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});
static SIDE_WORDS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&d("SIDE_WORDS")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});

/// ASK_KINDS: (kind, the ask phrases, the helper words that come with them).
static ASK_KINDS: LazyLock<Vec<(String, Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get(&d("ASK_KINDS"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].str().to_string(), p[1].strs(), p[2].strs())
        })
        .collect()
});

/// The tokens of every ask phrase, parallel to ASK_KINDS.
static ASK_KIND_TOKENS: LazyLock<Vec<Vec<Vec<String>>>> =
    LazyLock::new(|| ASK_KINDS.iter().map(|(_, asks, _)| asks.iter().map(|a| tokens(a)).collect()).collect());

// ---------------------------------------------------------------------------------------------
// Matching words loosely

/// True when the words of `phrase` appear in `text` in order (a short run).
pub fn _phrase_in(text: &str, phrase: &str) -> bool {
    phrase_in_tokens(&tokens(text), &tokens(phrase))
}

/// `_phrase_in` on words already split by `tokens`.
pub fn phrase_in_tokens(have: &[String], wanted: &[String]) -> bool {
    if wanted.is_empty() || have.is_empty() {
        return false;
    }
    let n = wanted.len();
    if have.len() < n {
        return false;
    }
    (0..=have.len() - n).any(|i| (0..n).all(|j| have[i + j] == wanted[j] || _same_word(&have[i + j], &wanted[j])))
}

static CUT_OFF: LazyLock<Vec<String>> = LazyLock::new(|| strs("CUT_OFF"));

fn clen(s: &str) -> usize {
    if s.is_ascii() {
        s.len()
    } else {
        s.chars().count()
    }
}

/// True when `word` is `target` with one letter wrong, missing, extra or swapped.
pub fn _one_slip(word: &str, target: &str) -> bool {
    let w: Vec<char> = word.chars().collect();
    let t: Vec<char> = target.chars().collect();
    if word == target || w.len() < 4 || w.len().abs_diff(t.len()) > 1 {
        return false;
    }
    if w.len() == t.len() {
        return w.iter().zip(t.iter()).filter(|(a, b)| a != b).count() == 1;
    }
    let (shorter, longer) = if w.len() < t.len() { (&w, &t) } else { (&t, &w) };
    (0..longer.len()).any(|i| {
        longer[..i].iter().chain(longer[i + 1..].iter()).copied().eq(shorter.iter().copied())
    })
}

/// True when a word names a cut of the meat, allowing a plural and a single slip.
pub fn _a_cut(word: &str) -> bool {
    CUT_OFF.iter().any(|cut| _same_word(word, cut) || _one_slip(word, cut))
}

/// eggs/egg, potatoes/potato, roasting/roast: the loose forms cooking.py matches with.
pub fn _same_word(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    for (one, two) in [(a, b), (b, a)] {
        if clen(one) > 3 {
            if one.strip_suffix('s') == Some(two) || one.strip_suffix("es") == Some(two) {
                return true;
            }
            if let Some(stem) = one.strip_suffix("ing") {
                if stem == two || (two.len() == stem.len() + 1 && two.starts_with(stem) && two.ends_with('e')) {
                    return true;
                }
            }
            if one.strip_suffix("ed") == Some(two) {
                return true;
            }
        }
    }
    false
}

/// True when the line's word is one slip from an avoided one ("brest", "thight").
pub fn _slip_of(typed: &str, word: &str) -> bool {
    let key = word.trim().to_lowercase();
    let typed = typed.trim().to_lowercase();
    let min = *dup::MIN_TYPO_LENGTH;
    if clen(&key) < min || clen(&typed) < min || key.contains(' ') {
        return false;
    }
    if typed.chars().next() != key.chars().next() || dup::_engine_reads(&typed) {
        return false;
    }
    dup::_one_edit(&typed, &key)
}

static A_MADE_DISH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_MADE_DISH"));

/// True when the word after a dish's name makes it another dish ("chicken soup").
pub fn _a_made_dish(word: &str) -> bool {
    A_MADE_DISH.iter().any(|dish| _same_word(word, dish))
}

static RULE_OUT: LazyLock<HashSet<String>> = LazyLock::new(|| set("_RULE_OUT"));
static RULE_OUT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| set("_RULE_OUT_FILLER"));
static INGREDIENT_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| set("_INGREDIENT_AFTER"));

fn no_quotes(w: &str) -> String {
    w.replace('\u{2019}', "").replace('\'', "")
}

/// True when the words just before `at` rule the dish that starts there out ("isnt a roast").
pub fn _ruled_out_near(have: &[String], at: usize, width: usize) -> bool {
    let after = at + width;
    if after < have.len() && INGREDIENT_AFTER.contains(&no_quotes(&have[after])) {
        return false;
    }
    for step in 1..=at.min(5) {
        let word = no_quotes(&have[at - step]);
        if RULE_OUT_FILLER.contains(&word) {
            continue;
        }
        return RULE_OUT.contains(&word);
    }
    false
}

/// A row guard made by `not_ruled_out(*keys)`: the entry answers unless the line names one of
/// these dishes only to rule it out.
pub fn not_ruled_out__guard(keys: &[String], understood: &Understood) -> bool {
    let text = sh::clean_or_text(understood);
    let out = dishes_ruled_out(text);
    !keys.iter().any(|k| out.contains(k))
}

fn g_not_ruled_out(f: &FnRef, u: &Understood) -> bool {
    let keys = f.cell("keys").map(|v| v.strs()).unwrap_or_default();
    not_ruled_out__guard(&keys, u)
}

/// The word positions of dish names the line rules out, so no shorter name inside one reads.
pub fn _ruled_out_spans(have: &[String]) -> HashSet<usize> {
    let mut covered = HashSet::new();
    for plan in PLANS.iter() {
        for (_, wanted) in plan.each_name() {
            if wanted.is_empty() || wanted.len() > have.len() {
                continue;
            }
            for i in 0..=have.len() - wanted.len() {
                if !_ruled_out_near(have, i, wanted.len()) {
                    continue;
                }
                if (0..wanted.len()).all(|j| _same_word(&have[i + j], &wanted[j])) {
                    covered.extend(i..i + wanted.len());
                }
            }
        }
    }
    covered
}

/// The plan keys a text names only to rule them out ("isnt a roast").
pub fn dishes_ruled_out(text: &str) -> HashSet<String> {
    let have = tokens(text);
    let mut out = HashSet::new();
    for plan in PLANS.iter() {
        for (_, wanted) in plan.each_name() {
            if wanted.is_empty() || wanted.len() > have.len() {
                continue;
            }
            for i in 0..=have.len() - wanted.len() {
                if (0..wanted.len()).all(|j| _same_word(&have[i + j], &wanted[j]))
                    && _ruled_out_near(&have, i, wanted.len())
                {
                    out.insert(plan.key.clone());
                    break;
                }
            }
        }
    }
    out
}

/// (words matched, position, plan key) of the most specific dish a text names, or None: the
/// longest name wins, and on a tie the one mentioned last.
pub fn _named(text: &str) -> Option<(usize, usize, &'static str)> {
    let have = tokens(text);
    let elsewhere = dup::says_another_thing(text);
    let ruled = _ruled_out_spans(&have);
    let mut best: Option<(usize, usize, &'static str)> = None;
    for plan in PLANS.iter() {
        for (_, wanted) in plan.each_name() {
            if wanted.is_empty() || wanted.len() > have.len() {
                continue;
            }
            let n = wanted.len();
            for i in 0..=have.len() - n {
                if !(0..n).all(|j| _same_word(&have[i + j], &wanted[j]) || (n > 1 && _slip_of(&have[i + j], &wanted[j]))) {
                    continue;
                }
                let after = if i + n < have.len() { have[i + n].as_str() } else { "" };
                if !after.is_empty() && plan.avoid.iter().any(|w| _same_word(after, w) || _slip_of(after, w)) {
                    continue;
                }
                if !after.is_empty() && _a_cut(after) {
                    continue;
                }
                if !after.is_empty() && _a_made_dish(after) {
                    continue;
                }
                if !elsewhere.is_empty() && have[i..i + n].iter().any(|w| elsewhere.contains(w)) {
                    continue;
                }
                if (i..i + n).any(|at| ruled.contains(&at)) {
                    continue;
                }
                let found = (n, i, plan.key.as_str());
                if best.is_none_or(|b| (found.0, found.1) > (b.0, b.1)) {
                    best = Some(found);
                }
            }
        }
    }
    best
}

static PLAN_MARKER: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLAN_MARKER"));
static IDEA_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_IDEA_ASK"));

/// True when the line names a dish and says that is what is being cooked.
pub fn a_dish_is_already_the_plan(text: &str) -> bool {
    let text = text.to_lowercase();
    if text.is_empty() || dish_in(&text).is_none() {
        return false;
    }
    if !any_in(&text, &PLAN_MARKER) {
        return false;
    }
    !any_in(&text, &IDEA_ASK)
}

/// The plan key a piece of text names, or None.
pub fn dish_in(text: &str) -> Option<&'static str> {
    if text.trim().is_empty() {
        return None;
    }
    _named(text).map(|f| f.2)
}

static PART_OF: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&d("PART_OF")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});
static IMPLIED_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("IMPLIED_ASKS"));
static PART_NOT: LazyLock<Vec<String>> = LazyLock::new(|| strs("PART_NOT"));

/// The dish a part word belongs to ('frosting' -> the cake plan), or None.
pub fn implied_dish(text: &str) -> Option<&'static str> {
    if text.trim().is_empty() {
        return None;
    }
    if PART_NOT.iter().any(|no| _phrase_in(text, no)) {
        return None;
    }
    for (part, key) in PART_OF.iter() {
        if _phrase_in(text, part) {
            if let Some(p) = plan(key) {
                return Some(p.key.as_str());
            }
        }
    }
    None
}

/// The user's line as the matcher keeps it (norm_text, else clean), or ''.
pub fn _line_text(understood: &Understood) -> &str {
    sh::or(&understood.norm_text, &understood.clean)
}

/// The side a line names ('rice', 'potatoes', ...), or None.
pub fn side_in(text: &str) -> Option<&'static str> {
    if a_colour_choice(text) {
        return None;
    }
    for (key, phrases) in SIDE_WORDS.iter() {
        for phrase in phrases {
            if _phrase_in(text, phrase) {
                return Some(key.as_str());
            }
        }
    }
    None
}

/// What the conversation has been about, newest first, in the user's own words.
pub fn recent_texts(memory: &Memory) -> Vec<String> {
    let mut users: Vec<String> = Vec::new();
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(4);
    for turn in turns[start..].iter().rev() {
        users.push(turn.user.clone());
    }
    if let Some(record) = memory.extra.get("topic") {
        if let Some(Value::Str(t)) = record.get("text") {
            users.push(t.to_string());
        }
    }
    users
}

/// The dish this conversation is about: named in the line itself, or just before it.
pub fn dish_for(memory: &Memory, understood: &Understood) -> Option<&'static str> {
    let text = _line_text(understood);
    if let Some(key) = dish_in(text) {
        return Some(key);
    }
    if _names_food(text) {
        return None;
    }
    for earlier in recent_texts(memory).iter().take(6) {
        if let Some(key) = dish_in(earlier) {
            return Some(key);
        }
    }
    None
}

// ---------------------------------------------------------------------------------------------
// Pairing: what goes with a food

static GOES_WITH_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| rx("GOES_WITH_SHAPE"));
static PAIRING_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PAIRING_FOODS"));
static PAIRING_RICH: LazyLock<String> = LazyLock::new(|| string("PAIRING_RICH"));
static PAIRING_PLAIN: LazyLock<String> = LazyLock::new(|| string("PAIRING_PLAIN"));
static PAIRING_LEAN: LazyLock<String> = LazyLock::new(|| string("PAIRING_LEAN"));
static RICH_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| strs("RICH_FOODS"));
static PLAIN_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PLAIN_FOODS"));
static COLOUR_CHOICE: LazyLock<&'static Re> = LazyLock::new(|| rx("COLOUR_CHOICE"));
static CHOICE_FRAME: LazyLock<&'static Re> = LazyLock::new(|| rx("_CHOICE_FRAME"));

/// True when the line asks what to put beside something, however the verb is dressed.
pub fn _asking_to_pair(text: &str) -> bool {
    GOES_WITH_SHAPE.is_match(text)
}

/// True when the line says one of these words, as a word rather than a run of letters.
pub fn _names_any<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    let have = tokens(text);
    words.iter().any(|w| phrase_in_tokens(&have, &tokens(w.as_ref())))
}

/// True when the line asks the pairing way and names a food it can be paired with.
pub fn asks_for_a_side_of_a_food(text: &str) -> bool {
    _asking_to_pair(text) && _names_any(text, &PAIRING_FOODS)
}

/// The pairing rule for a food with no dish: fat wants acid, starch wants something wet, and
/// anything else wants one of each.
pub fn general_sides(text: &str) -> String {
    if _names_any(text, &RICH_FOODS) {
        return PAIRING_RICH.clone();
    }
    if _names_any(text, &PLAIN_FOODS) {
        return PAIRING_PLAIN.clone();
    }
    PAIRING_LEAN.clone()
}

/// True when the line is choosing between two things by their colour, and names neither.
pub fn a_colour_choice(text: &str) -> bool {
    COLOUR_CHOICE.is_match(text) && CHOICE_FRAME.is_match(text)
}

static FOOD_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("FOOD_WORDS"));

/// True when the text names one of the foods a fix can be about (FOOD_WORDS), loosely.
pub fn _names_food(text: &str) -> bool {
    tokens(text).iter().any(|word| FOOD_WORDS.iter().any(|food| _same_word(word, food)))
}


// ---------------------------------------------------------------------------------------------
// What goes wrong, and the one thing that fixes it

static STARCH: LazyLock<Vec<String>> = LazyLock::new(|| strs("_STARCH"));
static POT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_POT"));
static GLUE: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get(&d("_GLUE")).items().iter().map(|g| g.strs()).collect());
static MUSHY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MUSHY_WORDS"));
static MUSHY_FIX: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_MUSHY_FIX"));
static GRAINY: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| groups("_GRAINY"));
static GRAINY_FIX: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_GRAINY_FIX"));
static LUMPY: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| groups("_LUMPY"));
static LUMPY_FIX: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_LUMPY_FIX"));

/// A `{str: str}` global, in order.
fn str_pairs(name: &str) -> Vec<(String, String)> {
    data::get(&d(name)).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
}

/// A tuple of `(names, key)` pairs.
fn groups(name: &str) -> Vec<(Vec<String>, String)> {
    data::get(&d(name))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].strs(), p[1].str().to_string())
        })
        .collect()
}

fn lookup(table: &[(String, String)], key: &str) -> Option<String> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

/// The line's words, lower case (`[str(w).lower() for w in understood["words"]]`).
fn lower_words(understood: &Understood) -> Vec<String> {
    understood.words.iter().map(|w| w.to_lowercase()).collect()
}

/// `any(_same_word(word, wanted) for word in words for wanted in wanted)`.
fn any_same<S: AsRef<str>, T: AsRef<str>>(words: &[S], wanted: &[T]) -> bool {
    words.iter().any(|w| wanted.iter().any(|x| _same_word(w.as_ref(), x.as_ref())))
}

/// Bread and cakes go heavy for one reason, and mash, rice or porridge for the other.
pub fn _heavy(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    if any_same(&words, &POT) {
        return None;
    }
    if any_same(&words, &["mash", "mashed potato", "mashed potatoes", "potato", "potatoes"]) {
        return Some("Heavy mash is wet potatoes and too much mashing: drain them well, let the steam off for a minute, mash them hot with warm milk and butter, and stop as soon as they are smooth. If it is already heavy, beat in a spoon of warm milk or a knob of butter and serve it at once.".into());
    }
    if any_same(&words, &STARCH) {
        return Some("Stodgy rice, porridge or couscous is too much water and too much stirring: use less liquid than the packet says, stop stirring once it is in the pan, and let it stand off the heat for 5 minutes so the starch sets before you serve it.".into());
    }
    Some("Bread and cakes go heavy when the raising agent is stale, the mix is overworked or the oven is too cool: check the date on the powder, fold rather than beat, and give the oven 20 minutes to heat properly.".into())
}

/// A gluey dish gets the fix for the dish it is, not the rice instruction.
pub fn _gluey(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let mushy = any_same(&words, &MUSHY_WORDS);
    if mushy {
        if any_same(&words, &["mash", "mashed potato", "mashed potatoes", "potato", "potatoes"]) {
            return lookup(&MUSHY_FIX, "mash");
        }
        if any_same(
            &words,
            &["bread", "cake", "sponge", "loaf", "loaves", "muffin", "muffins", "brownie", "brownies", "pastry", "bun", "buns", "scone", "scones"],
        ) {
            return lookup(&MUSHY_FIX, "bake");
        }
        if any_same(&words, &["risotto", "risotto rice", "paella"]) {
            return lookup(&MUSHY_FIX, "risotto");
        }
        if any_same(&words, &["sauce", "gravy", "curry", "soup", "stew", "custard", "bolognese"]) {
            return lookup(&MUSHY_FIX, "sauce");
        }
        if any_same(&words, &["rice", "risotto"]) {
            return lookup(&MUSHY_FIX, "rice");
        }
        if any_same(&words, &["couscous", "bulgur", "bulghur", "quinoa", "polenta"]) {
            return lookup(&MUSHY_FIX, "grain");
        }
        if any_same(&words, &["porridge", "oats", "oatmeal", "porage"]) {
            return lookup(&MUSHY_FIX, "porridge");
        }
        return lookup(&MUSHY_FIX, "");
    }
    let have = GLUE.iter().any(|group| any_same(&words, group));
    if !have {
        return Some("Gluey starch is nearly always too much water and too much stirring: stop stirring it, drain off any water it is still sitting in as soon as a piece is tender, and let it stand off the heat for a minute before you serve it.".into());
    }
    if any_same(&words, &["mash", "mashed potato", "mashed potatoes", "potato", "potatoes"]) {
        return Some("Mash goes gluey when it is mashed too long, or the potatoes went in wet: drain them well and let the steam off for a minute, mash while they are hot with warm milk and butter rather than cold, and stop as soon as it is smooth. A gluey mash that is already made comes back if you beat in a spoon of warm milk and serve it at once, but never put it in a food processor.".into());
    }
    if any_same(&words, &["noodle", "noodles", "udon", "ramen", "rice noodle", "rice noodles"]) {
        return Some("Noodles clump when they are boiled in a small pan and then left to sit: give them a big pan of boiling water, stir them apart in the first minute, and rinse them under warm water once they are drained if you are not serving them at once. A drop of oil in the water does nothing.".into());
    }
    if any_same(&words, &["couscous", "bulgur", "bulghur", "quinoa", "polenta"]) {
        return Some("This one wants less water and a fork, not a spoon: pour boiling water or stock over it in a wide bowl, 1 part to 1.5 parts for couscous and 1 to 2 for quinoa, cover it and leave it 5 minutes, then fork it through to break the lumps. Stirring it while it soaks is what makes it claggy.".into());
    }
    Some("Sticky pasta is a starch coating, not a cooking fault: use a big pan with plenty of water, stir it for the first minute so the pieces separate, and drain it but do not rinse it unless it is for a cold salad. If it has already clumped, a spoon of the cooking water tossed through it loosens it again.".into())
}

/// Which food family the line is about, or None when the line names no food at all.
pub fn _fault_groups(_memory: &Memory, understood: &Understood, groups: &[(Vec<String>, String)]) -> Option<String> {
    let said = lower_words(understood);
    for (names, key) in groups {
        if any_same(&said, names) {
            return Some(key.clone());
        }
    }
    if !_names_food(_line_text(understood)) {
        return None;
    }
    Some(String::new())
}

/// A grainy dish gets the cause of the break, which is different for meat, mash and eggs.
pub fn _grainy(memory: &Memory, understood: &Understood) -> Option<String> {
    let key = _fault_groups(memory, understood, &GRAINY)?;
    lookup(&GRAINY_FIX, &key)
}

/// Lumpy is a mixing fault, so the food in the line decides the tool and the next step.
pub fn _lumpy(memory: &Memory, understood: &Understood) -> Option<String> {
    let key = _fault_groups(memory, understood, &LUMPY)?;
    lookup(&LUMPY_FIX, &key)
}

/// A fault's fallback answer: a sentence, or a function that reads the food in the line.
enum General {
    Text(String),
    Fn(String),
}

/// One everyday problem: the words it is said with, the general answer, and per-dish answers.
struct Problem {
    key: String,
    words: Vec<String>,
    general: General,
    by_dish: Vec<(String, String)>,
}

static PROBLEMS: LazyLock<Vec<Problem>> = LazyLock::new(|| {
    data::get(&d("PROBLEMS"))
        .pairs()
        .iter()
        .map(|(k, v)| {
            let p = v.items();
            let general = match &p[1] {
                Value::Fn(f) => General::Fn(f.id.clone()),
                other => General::Text(other.str().to_string()),
            };
            Problem {
                key: k.str().to_string(),
                words: p[0].strs(),
                general,
                by_dish: p[2].pairs().iter().map(|(a, b)| (a.str().to_string(), b.str().to_string())).collect(),
            }
        })
        .collect()
});

fn problem(key: &str) -> &'static Problem {
    PROBLEMS.iter().find(|p| p.key == key).expect("a PROBLEMS key")
}

static VEG_FAMILIES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| groups("VEG_FAMILIES"));
static VEG_FAULTS: LazyLock<Vec<String>> = LazyLock::new(|| strs("VEG_FAULTS"));

/// The fix for a vegetable that stayed hard or went tough, or None.
pub fn _veg_fix(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let have = tokens(text);
    let says = |w: &str| phrase_in_tokens(&have, &tokens(w));
    if !VEG_FAULTS.iter().any(|f| says(f)) {
        return None;
    }
    if ["jacket potato", "baked potato", "jacket potatoes", "baked potatoes"].iter().any(|w| says(w)) {
        return Some("A jacket potato is hard in the middle because it was rushed: prick it, microwave it 8 to 10 minutes until a knife goes in, then 15 to 20 minutes at 200C to crisp the skin. In the oven alone it needs an hour and a quarter at 200C, so give it that long or start it early.".into());
    }
    if ["potato", "potatoes", "new potatoes"].iter().any(|w| says(w)) {
        if ["roast", "roasted", "roasting", "oven", "crispy", "tray"].iter().any(|w| says(w)) {
            return Some("Hard roast potatoes went in too big or came out too early: give them another 10 to 15 minutes at 200C, and next time cut them no bigger than a golf ball and parboil them 8 minutes first, which is what cooks the middle before the outside crisps.".into());
        }
        return Some("Boiled potatoes go hard in the middle when the pieces are uneven: cut them the same size, start them in cold salted water and simmer 15 to 20 minutes, until a knife slides in. An old potato can be woody at the core, and that will not soften however long it cooks.".into());
    }
    if ["mushroom", "mushrooms"].iter().any(|w| says(w)) {
        return Some("Rubbery mushrooms have been steamed rather than fried: they hold a lot of water, so give them a wide hot pan with a little oil, one layer and no lid, and let that water cook off before you turn them. A pinch of salt only at the end, or they give up their water early and stew.".into());
    }
    if [
        "dried", "haricot", "haricots", "cannellini", "borlotti", "butter bean", "butter beans", "chickpea", "chickpeas", "split peas",
        "lentil", "lentils", "puy",
    ]
    .iter()
    .any(|w| says(w))
    {
        return None;
    }
    for (names, fix) in VEG_FAMILIES.iter() {
        for name in names {
            if says(name) {
                return Some(fix.clone());
            }
        }
    }
    None
}

static SINK_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SINK_VERBS"));
static SINK_AS_A_BAKE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SINK_AS_A_BAKE"));

/// True when a "sink" in this text is the verb about a bake, not the kitchen sink.
pub fn _a_sink_is_a_bake(text: &str) -> bool {
    let low = text.to_lowercase();
    _names_any(&low, &SINK_AS_A_BAKE)
}

static PLAN_THAT_MIGHT_BE_RUINED: LazyLock<&'static Re> = LazyLock::new(|| rx("_PLAN_THAT_MIGHT_BE_RUINED"));
static ASKED_ABOUT_THE_FAULT: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASKED_ABOUT_THE_FAULT"));

/// True unless the line is weighing what a choice would do to a dish, not reporting a fault.
pub fn a_plan_that_might_be_ruined(understood: &Understood) -> bool {
    let text = sh::or(sh::or(&understood.plain, &understood.clean), &understood.text);
    if ASKED_ABOUT_THE_FAULT.is_match(text) {
        return true;
    }
    !PLAN_THAT_MIGHT_BE_RUINED.is_match(text)
}

fn g_a_plan_that_might_be_ruined(_f: &FnRef, u: &Understood) -> bool {
    a_plan_that_might_be_ruined(u)
}

/// Which of the everyday problems this text names, or None.
pub fn problem_in(text: &str) -> Option<&'static str> {
    let have = tokens(text);
    for p in PROBLEMS.iter() {
        for word in &p.words {
            if SINK_VERBS.contains(word) && !_a_sink_is_a_bake(text) {
                continue;
            }
            if phrase_in_tokens(&have, &tokens(word)) {
                return Some(p.key.as_str());
            }
        }
    }
    None
}

static A_STATE_VERB: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_STATE_VERB"));
static NOT_THE_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| strs("NOT_THE_FOOD"));
static STANDS_FOR_IT: LazyLock<HashSet<String>> = LazyLock::new(|| set("_STANDS_FOR_IT"));

/// The words the state `word` is said of, or [] when the line states no subject.
pub fn _the_subject_of_the_state(text: &str, word: &str) -> Vec<String> {
    static CLAUSE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,.;:!?]|\band\b", 0));
    let low = text.to_lowercase();
    let pattern = format!(r"\b{}\b", sh::escape(&word.to_lowercase()));
    let Some(found) = sh::re_cached(&pattern, 0).search(&low) else { return Vec::new() };
    let before = &low[..found.start()];
    let parts = CLAUSE.split(before);
    let clause = parts.last().cloned().unwrap_or_default();
    let words: Vec<&str> = clause.split_whitespace().collect();
    let verbs: Vec<usize> = (0..words.len()).filter(|&i| A_STATE_VERB.fullmatch(words[i]).is_some()).collect();
    let Some(&last) = verbs.last() else { return Vec::new() };
    let subject: Vec<String> = words[..last]
        .iter()
        .filter(|w| !w.is_empty() && w.chars().all(char::is_alphabetic))
        .map(|w| w.to_string())
        .collect();
    subject[subject.len().saturating_sub(2)..].to_vec()
}

/// Whether the fault `key` is said of the dish, and not of something else in the room.
pub fn _the_state_is_about_the_food(text: &str, key: &str) -> bool {
    for word in &problem(key).words {
        let subject = _the_subject_of_the_state(text, word);
        if subject.is_empty() {
            continue;
        }
        if subject.iter().any(|one| STANDS_FOR_IT.contains(one)) {
            return true;
        }
        if any_same(&subject, &NOT_THE_FOOD) {
            return false;
        }
        return true;
    }
    true
}

static COOK_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_COOK_PHRASES"));

/// True when the line asks what to make or eat with a food it names.
pub fn cook_ask(understood: &Understood) -> bool {
    let text = _line_text(understood).to_lowercase();
    if !any_in(&text, &COOK_PHRASES) {
        return false;
    }
    any_in(&text, &FOOD_WORDS)
}

/// Run a fault's general answer when it is one of this module's functions.
fn call_general(id: &str, memory: &Memory, understood: &Understood) -> Option<String> {
    match id {
        "bot.knowledge.dishes:_grainy:1338" => _grainy(memory, understood),
        "bot.knowledge.dishes:_lumpy:1380" => _lumpy(memory, understood),
        "bot.knowledge.dishes:_heavy:1116" => _heavy(memory, understood),
        "bot.knowledge.dishes:_gluey:1194" => _gluey(memory, understood),
        _ => None,
    }
}

/// The fix for the thing that keeps going wrong, about the dish being cooked.
pub fn _problem(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    let mut key = problem_in(text);
    if let Some(k) = key {
        if !_the_state_is_about_the_food(text, k) {
            return None;
        }
    }
    if key.is_none() {
        for earlier in recent_texts(memory).iter().take(2) {
            key = problem_in(earlier);
            if let Some(k) = key {
                if !_the_state_is_about_the_food(earlier, k) {
                    key = None;
                    continue;
                }
                break;
            }
        }
    }
    let Some(key) = key else { return _veg_fix(text) };
    let p = problem(key);
    let dish = dish_for(memory, understood);
    if let Some(dish) = dish {
        if let Some((_, said)) = p.by_dish.iter().find(|(k, _)| k == dish) {
            return Some(said.clone());
        }
    }
    if dish.is_none() {
        let mut lines = vec![text.to_string()];
        lines.extend(recent_texts(memory).into_iter().take(2));
        if !lines.iter().any(|one| _names_food(one)) {
            return None;
        }
    }
    if let Some(veg) = _veg_fix(text) {
        return Some(veg);
    }
    match &p.general {
        General::Fn(id) => call_general(id, memory, understood),
        General::Text(t) => Some(t.clone()),
    }
}

// ---------------------------------------------------------------------------------------------
// Which follow-up a line is

static PLAIN: LazyLock<HashSet<String>> = LazyLock::new(|| set("PLAIN"));
static PART: LazyLock<HashSet<String>> = LazyLock::new(|| set("PART"));
static DONE_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("DONE_ASK"));
static DONE_VERB_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("DONE_VERB_ASK"));
static DONE_WORD: LazyLock<Vec<String>> = LazyLock::new(|| strs("DONE_WORD"));
static ASK_NOISE: LazyLock<HashSet<String>> = LazyLock::new(|| set("_ASK_NOISE"));
static GENERIC_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| set("_GENERIC_SUBJECT"));
static PLAN_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("PLAN_NAMES"));

/// Every name the plans, the food words and the dish words know (sorted), for the food test.
pub fn _any_food() -> &'static [String] {
    static ANY_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| {
        let all: BTreeSet<String> =
            PLAN_NAMES.iter().chain(FOOD_WORDS.iter()).chain(DISH_WORDS.iter()).cloned().collect();
        all.into_iter().collect()
    });
    &ANY_FOOD
}

/// True when the line asks whether a *food* is done, by name or by pronoun.
pub fn _asks_doneness(text: &str) -> bool {
    static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
    static ANY_FOOD_TOKENS: LazyLock<Vec<Vec<String>>> =
        LazyLock::new(|| _any_food().iter().map(|n| tokens(n)).collect());
    let found = DONE_ASK.search(text).or_else(|| DONE_VERB_ASK.search(text));
    let Some(found) = found else { return false };
    let span = found.as_str().to_lowercase();
    if _ready_food(&span).is_some() {
        return true;
    }
    let have = tokens(&span);
    if ANY_FOOD_TOKENS.iter().any(|name| phrase_in_tokens(&have, name)) {
        return true;
    }
    WORDS
        .findall_str(&span)
        .iter()
        .filter(|w| !ASK_NOISE.contains(*w) && !DONE_WORD.contains(*w))
        .all(|w| GENERIC_SUBJECT.contains(w))
}

static MAKING_A_FAULT: LazyLock<&'static Re> = LazyLock::new(|| rx("MAKING_A_FAULT"));
static KEEP_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("KEEP_ASK"));

/// Which kind of follow-up this line is: 'temp', 'time', 'done', ... or None.
pub fn ask_kind(understood: &Understood) -> Option<&'static str> {
    let text = _line_text(understood);
    let have = tokens(text);
    if have.is_empty() {
        return None;
    }
    let ready = if _asks_doneness(text) { 6 } else { 0 };
    let keeps = if KEEP_ASK.is_match(text) { 6 } else { 0 };
    let making_a_fault = MAKING_A_FAULT.is_match(text);
    let mut best: Option<&'static str> = None;
    let mut best_rank = (0, 0);
    for ((kind, _asks, helpers), ask_tokens) in ASK_KINDS.iter().zip(ASK_KIND_TOKENS.iter()) {
        let mut score = 0;
        for wanted in ask_tokens {
            if phrase_in_tokens(&have, wanted) {
                score += 2 + wanted.len();
                break;
            }
        }
        if kind == "done" && ready > 0 {
            score = score.max(ready);
        }
        if kind == "keep" && keeps > 0 {
            score = score.max(keeps);
        }
        if score == 0 {
            continue;
        }
        if any_same(&have, helpers) {
            score += 1;
        }
        if kind == "keep" && making_a_fault {
            continue;
        }
        let rank = (if PART.contains(kind) { 2 } else if PLAIN.contains(kind) { 0 } else { 1 }, score);
        if rank > best_rank {
            best = Some(kind.as_str());
            best_rank = rank;
        }
    }
    best
}

static TIME_IN_ANSWER: LazyLock<&'static Re> = LazyLock::new(|| rx("TIME_IN_ANSWER"));
static KEEPING_LEAD: LazyLock<Vec<String>> = LazyLock::new(|| strs("KEEPING_LEAD"));

/// The first cooking time in a sentence, in minutes, or None.
pub fn _first_minutes(text: &str) -> Option<i64> {
    for found in TIME_IN_ANSWER.finditer(text) {
        let head_all = text[..found.start()].to_lowercase();
        let n = head_all.chars().count();
        let head: String = head_all.chars().skip(n.saturating_sub(30)).collect();
        if any_in(&head, &KEEPING_LEAD) {
            continue;
        }
        let count: i64 = found.str(1).parse().unwrap_or(0);
        return Some(if found.str(2).to_lowercase().starts_with("hour") { count * 60 } else { count });
    }
    None
}

/// What the bot said last, or "".
pub fn _last_answer_said(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

/// True when the last answer gave a time for a different version of this dish.
pub fn _plan_contradicts_the_last_answer(memory: &Memory, plan: &Plan) -> bool {
    let told = match _first_minutes(&_last_answer_said(memory)) {
        Some(t) if t != 0 => t,
        _ => return false,
    };
    let ours = match _first_minutes(plan.get("time").unwrap_or("")) {
        Some(t) if t != 0 => t,
        _ => return false,
    };
    told.max(ours) > 2 * told.min(ours)
}

static PART_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PART_WORDS"));
static PART_BACKREF: LazyLock<Vec<String>> = LazyLock::new(|| strs("PART_BACKREF"));

/// The parts of a dish a line names, so "the filling" is not "the icing" said twice.
pub fn _parts_in(text: &str) -> BTreeSet<String> {
    let have = tokens(text);
    PART_WORDS.iter().filter(|w| phrase_in_tokens(&have, &tokens(w))).cloned().collect()
}

/// The person's line that the last answer was answering, as they wrote it.
pub fn _answering_line(memory: &Memory) -> String {
    for turn in memory.turns.iter().rev() {
        if !turn.user.trim().is_empty() {
            return turn.user.clone();
        }
    }
    String::new()
}

/// The answer said for the part this line names, when the last answer already said it.
pub fn _same_answer_but_a_new_part(memory: &Memory, understood: &Understood, answer: &str) -> Option<String> {
    if answer.is_empty() {
        return None;
    }
    let text = _line_text(understood);
    let asked = _parts_in(text);
    if asked.is_empty() || !_names_any(text, &PART_BACKREF) {
        return None;
    }
    let already = _parts_in(&_answering_line(memory));
    let new: Vec<&String> = asked.difference(&already).collect();
    if let Some(part) = new.first() {
        let mut chars = answer.chars();
        let first = chars.next().map(|c| c.to_lowercase().collect::<String>()).unwrap_or_default();
        return Some(format!("Same for the {part}: {first}{}", chars.as_str()));
    }
    None
}

/// The dish this line names itself, or None when it only leans on the one before.
pub fn _dish_named_in_the_line(understood: &Understood) -> Option<&'static str> {
    dish_in(_line_text(understood))
}

static NAMES_A_SAUCE: LazyLock<&'static Re> = LazyLock::new(|| rx("_NAMES_A_SAUCE"));

/// The answer to a follow-up of this kind about the dish in the conversation, or None.
pub fn reply(kind: &str, memory: &Memory, understood: &Understood) -> Option<String> {
    let plan = plan(dish_for(memory, understood)?)?;
    if NAMES_A_SAUCE.is_match(_line_text(understood)) {
        return None;
    }
    if kind == "time" && _dish_named_in_the_line(understood).is_none() && _plan_contradicts_the_last_answer(memory, plan) {
        return None;
    }
    if PART_OF.iter().any(|(k, _)| k == kind) && PART_NOT.iter().any(|no| _phrase_in(_line_text(understood), no)) {
        return None;
    }
    let mut answer: Option<&str> = plan.get_truthy(kind);
    if kind == "ahead" && answer.is_none() {
        answer = plan.get("keep");
    }
    if let Some(a) = answer {
        if !a.is_empty() && a == _last_answer_said(memory) {
            if let Some(part) = _same_answer_but_a_new_part(memory, understood, a) {
                if !part.trim().is_empty() {
                    return Some(part);
                }
            }
            return Some(a.to_string());
        }
    }
    answer.map(str::to_string)
}

/// The answer about a part the line names when it names no dish of its own.
pub fn _part_ahead(memory: &Memory, understood: &Understood) -> Option<String> {
    if dish_for(memory, understood).is_some() {
        return None;
    }
    let text = _line_text(understood);
    let key = implied_dish(text)?;
    if !_names_any(text, &IMPLIED_ASKS) {
        return None;
    }
    let plan = plan(key)?;
    let answer = plan.get_truthy("frosting").or_else(|| plan.get("keep"))?;
    if !answer.is_empty() && answer == _last_answer_said(memory) {
        return None;
    }
    Some(answer.to_string())
}

static KEEP_ASK_LOOSE: LazyLock<&'static Re> = LazyLock::new(|| rx("_KEEP_ASK"));
static KEEP_BUT_COOKING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_KEEP_BUT_COOKING"));

/// True for "how long does lasagne keep", false for "how long do i keep it in the oven".
pub fn _is_a_keeping_line(text: &str) -> bool {
    let line = text.to_lowercase();
    KEEP_ASK_LOOSE.is_match(&line) && !any_in(&line, &KEEP_BUT_COOKING)
}

static PERSON_STATE: LazyLock<&'static Re> = LazyLock::new(|| rx("PERSON_STATE"));
static PERSON_REST: LazyLock<&'static Re> = LazyLock::new(|| rx("PERSON_REST"));
static BODY_TEMPERATURE: LazyLock<&'static Re> = LazyLock::new(|| rx("BODY_TEMPERATURE"));

/// True when the line is about a person rather than about the dish being cooked.
pub fn _about_a_person(text: &str) -> bool {
    PERSON_STATE.match_(text).is_some() || BODY_TEMPERATURE.is_match(text) || PERSON_REST.is_match(text)
}

static RAW_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("RAW_WORDS"));

/// True when the conversation is about eating something raw or undercooked.
pub fn _raw_in_flight(memory: &Memory) -> bool {
    recent_texts(memory).iter().take(3).any(|line| any_same(&tokens(line), &RAW_WORDS))
}

/// The answer to whatever kind of follow-up the line is, about the dish so far.
pub fn _follow_up(memory: &Memory, understood: &Understood) -> Option<String> {
    let line = _line_text(understood);
    if _about_a_person(line) && !(_dish_named_in_the_line(understood).is_some() || dish_in(line).is_some()) {
        return None;
    }
    let kind = ask_kind(understood);
    if kind == Some("keep") && _raw_in_flight(memory) {
        return None;
    }
    if kind != Some("keep") && _parts_in(line).is_empty() && _is_a_keeping_line(line) {
        if let Some(answer) = reply("keep", memory, understood).filter(|a| !a.is_empty()) {
            return Some(answer);
        }
    }
    if kind == Some("fix") {
        return _problem(memory, understood)
            .filter(|a| !a.is_empty())
            .or_else(|| _part_ahead(memory, understood));
    }
    if let Some(answer) = kind.and_then(|k| reply(k, memory, understood)).filter(|a| !a.is_empty()) {
        return Some(answer);
    }
    if kind == Some("done") {
        if let Some(ready) = _ready_note(memory, understood).filter(|a| !a.is_empty()) {
            return Some(ready);
        }
    }
    _part_ahead(memory, understood)
}

fn r_follow_up(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _follow_up(m, u)
}

/// The plan's answer for a line that only asks, when no entry claimed the line itself.
pub fn follow_up(memory: &Memory, understood: &Understood) -> Option<String> {
    _follow_up(memory, understood)
}


/// The ways of asking for the oven's heat, as one key (asks, helpers).
pub fn _temp_asks() -> (Vec<String>, Vec<String>) {
    let (mut asks, mut helpers): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for (kind, parts, words) in ASK_KINDS.iter() {
        if kind != "temp" {
            continue;
        }
        for part in parts {
            if !asks.contains(part) {
                asks.push(part.clone());
            }
        }
        for word in words {
            if !helpers.contains(word) {
                helpers.push(word.clone());
            }
        }
    }
    (asks, helpers)
}

/// The plan's figure for the heat, when the card for this dish answers with a recipe.
pub fn _the_heat(memory: &Memory, understood: &Understood) -> Option<String> {
    if ask_kind(understood) != Some("temp") {
        return None;
    }
    _dish_named_in_the_line(understood)?;
    reply("temp", memory, understood)
}

fn r_the_heat(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _the_heat(m, u)
}

/// Every way of asking a follow-up, and the words that come with it, as one key.
pub fn _asks() -> (Vec<String>, Vec<String>) {
    let (mut asks, mut helpers): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for (_, parts, words) in ASK_KINDS.iter() {
        for part in parts {
            if !asks.contains(part) {
                asks.push(part.clone());
            }
        }
        for word in words {
            if !helpers.contains(word) {
                helpers.push(word.clone());
            }
        }
    }
    (asks, helpers)
}

static ASK_OPENERS: LazyLock<Vec<String>> = LazyLock::new(|| strs("ASK_OPENERS"));
static ASK_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("ASK_PHRASES"));
static USE_UP_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("USE_UP_PHRASES"));

/// True when the line is asking about a side, not telling what is in the kitchen.
pub fn _asking_for_a_side(text: &str) -> bool {
    let have = tokens(text);
    if have.is_empty() {
        return false;
    }
    if USE_UP_PHRASES.iter().any(|p| phrase_in_tokens(&have, &tokens(p))) {
        return false;
    }
    if ASK_PHRASES.iter().any(|p| phrase_in_tokens(&have, &tokens(p))) {
        return true;
    }
    ASK_OPENERS.contains(&have[0])
}

/// The sort key of PLAN_NAMES_LONGEST: more words first, then by name.
pub fn lambda_2469(name: &str) -> (i64, String) {
    (-(name.split_whitespace().count() as i64), name.to_string())
}

static WHEN_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("WHEN_WORDS"));
static PERSON_START: LazyLock<&'static Re> = LazyLock::new(|| rx("PERSON_START"));
static DELIBERATING_START: LazyLock<&'static Re> = LazyLock::new(|| rx("DELIBERATING_START"));

/// The practical plan for the dish the line says is being cooked, or None.
pub fn _start(memory: &Memory, understood: &Understood) -> Option<String> {
    if ask_kind(understood).is_some() {
        return None;
    }
    let text = _line_text(understood);
    let announced = PERSON_START.is_match(text);
    if !(announced || DELIBERATING_START.is_match(text)) {
        return None;
    }
    if !announced && !any_in(text, &WHEN_WORDS) {
        return None;
    }
    if _their_diet_refuses(memory, understood, text) {
        return None;
    }
    _start_sentence(memory, understood)
}

fn r_start(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _start(m, u)
}

/// True when a diet the conversation holds bans what this line names.
pub fn _their_diet_refuses(memory: &Memory, understood: &Understood, text: &str) -> bool {
    let banned = dup::cooking::_bans_for(understood, memory);
    if dup::cooking::_bans_said(&banned).is_empty() {
        return false;
    }
    dup::cooking::_foods_named(text).iter().any(|food| dup::cooking::_bans_it(food, &banned))
}

/// The plan's own first sentence for the dish in the line: name, time, heat.
pub fn _start_sentence(memory: &Memory, understood: &Understood) -> Option<String> {
    let plan = plan(dish_for(memory, understood)?)?;
    let mut label: String = plan
        .get_truthy("label")
        .map(str::to_string)
        .unwrap_or_else(|| plan.names[0][0].clone());
    let low = label.to_lowercase();
    let text = _line_text(understood).to_lowercase();
    let said = plan.names.iter().flatten().find(|name| {
        let first = name.split_whitespace().next().unwrap_or("");
        !["a", "an", "the"].contains(&first)
            && sh::re_search(&format!(r"(?<![a-z0-9]){}(?![a-z0-9])", sh::escape(name)), &text, 0)
    });
    if let Some(said) = said {
        let said6: String = said.chars().take(6).collect();
        let low6: String = low.chars().take(6).collect();
        if !said.is_empty() && !low.contains(said.as_str()) && !said.contains(low.as_str()) && said6 != low6 {
            label = said.clone();
        }
    }
    let mut chars = label.chars();
    let label = match chars.next() {
        Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
        None => return None, // label[0] of "" raises
    };
    let time = plan.get("time");
    let mut temp = plan.get("temp");
    if let (Some(ti), Some(te)) = (time, temp) {
        if !ti.is_empty() && !te.is_empty() {
            let figure = te.split('C').next().unwrap_or("").trim();
            if ti.contains(figure) {
                temp = None;
            }
        }
    }
    let parts: Vec<&str> = [time, temp].into_iter().flatten().filter(|p| !p.is_empty()).collect();
    Some(format!("{label}: {}", parts.join(" ")))
}

static STARTED_PARTS: LazyLock<Vec<String>> = LazyLock::new(|| strs("STARTED_PARTS"));
static STAGE_PARTS: LazyLock<Vec<String>> = LazyLock::new(|| strs("STAGE_PARTS"));
static ASK_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| set("ASK_FILLER"));

/// True when the ask names a subject of its own that is not the dish being announced.
pub fn _asks_about_another_thing(text: &str, key: &str) -> bool {
    static SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow\b|\bwhat temperature\b|\bwhat temp\b|\bis it done\b", 0));
    let mut own: HashSet<String> = HashSet::new();
    if let Some(p) = plan(key) {
        for name in p.names.iter().flatten() {
            own.extend(tokens(name));
        }
    }
    let parts = SPLIT.split(&text.to_lowercase());
    if parts.len() < 2 {
        return false;
    }
    for word in tokens(parts.last().map(String::as_str).unwrap_or("")) {
        if ASK_FILLER.contains(&word) || own.contains(&word) || word.chars().any(|c| c.is_numeric()) {
            continue;
        }
        return true;
    }
    false
}

/// The plan's answer for a line that announces a dish and asks about cooking it at once.
pub fn announced_part(memory: &Memory, understood: &Understood) -> Option<String> {
    let kind = ask_kind(understood)?;
    if !STARTED_PARTS.iter().any(|p| p == kind) {
        return None;
    }
    if lower_words(understood).iter().any(|w| STAGE_PARTS.contains(w)) {
        return None;
    }
    let text = _line_text(understood);
    if !PERSON_START.is_match(text) {
        return None;
    }
    let key = dish_in(text)?;
    if _asks_about_another_thing(text, key) {
        return None;
    }
    let answer = if kind == "time" || kind == "temp" {
        _start_sentence(memory, understood)
    } else {
        reply(kind, memory, understood)
    };
    answer.filter(|a| !a.trim().is_empty())
}

static SIDE_METHOD: LazyLock<Vec<String>> = LazyLock::new(|| strs("SIDE_METHOD"));

/// The side for the dish in the conversation, the side named, or the pairing rule for a food.
pub fn _sides(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    if a_colour_choice(text) {
        return None;
    }
    if !_asking_for_a_side(text) {
        return None;
    }
    if any_in(text, &SIDE_METHOD) {
        return None;
    }
    let key = side_in(text);
    let plan = plan(dish_for(memory, understood).unwrap_or(""));
    if key.is_some() && plan.is_none() && _asking_to_pair(text) {
        return None;
    }
    if let Some(key) = key {
        return lookup(&SIDES, key);
    }
    if let Some(plan) = plan {
        return plan.get("sides").map(str::to_string);
    }
    if asks_for_a_side_of_a_food(text) {
        return Some(general_sides(text));
    }
    None
}

fn r_sides(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _sides(m, u)
}

// ---------------------------------------------------------------------------------------------
// A food somebody will not eat, and what goes in its place

static BIRDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BIRDS"));
static RED_MEAT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_RED_MEAT"));

/// A meat to put in place of the one somebody will not eat.
pub fn _another_meat(food: &str) -> &'static str {
    let name = food.to_lowercase();
    if BIRDS.iter().any(|b| _same_word(&name, b)) {
        return "beef or pork";
    }
    if RED_MEAT.iter().any(|r| _same_word(&name, r)) {
        return "chicken thighs";
    }
    "chicken thighs or beef"
}

static SUBSTITUTES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| groups("SUBSTITUTES"));
static DISLIKED: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISLIKED"));
static DISLIKE_SHAPES: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISLIKE_SHAPES"));
static OUT_OF_CUE: LazyLock<&'static Re> = LazyLock::new(|| rx("OUT_OF_CUE"));
static KITCHEN_SWAP: LazyLock<Vec<String>> = LazyLock::new(|| strs("KITCHEN_SWAP"));
static WASTE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("WASTE_WORDS"));
static FOOD_TAIL_OK: LazyLock<Vec<String>> = LazyLock::new(|| strs("FOOD_TAIL_OK"));
static HOW_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("HOW_WORDS"));
static GENERIC_SWAP: LazyLock<String> = LazyLock::new(|| string("GENERIC_SWAP"));
static CUPBOARD: LazyLock<&'static Re> = LazyLock::new(|| rx("CUPBOARD"));
static HAVE_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("HAVE_WORDS"));

/// `template.format(food=food, other=other)` for the swap sentences (they hold only these two).
fn fmt_swap(template: &str, food: &str, other: &str) -> String {
    template.replace("{food}", food).replace("{other}", other)
}

/// True when the line names a food of its own, so the question is about that food.
pub fn _foods_named(understood: &Understood) -> bool {
    !dup::cooking::ingredients(understood).is_empty()
}

/// True when the line says what the kitchen has, not only what it is missing.
pub fn _states_what_they_have(text: &str) -> bool {
    HAVE_WORDS.is_match(text)
}

static COOKING_FOOD: LazyLock<Vec<String>> = LazyLock::new(|| {
    data::try_get("bot.knowledge.cooking._FOOD").map(|v| v.strs()).unwrap_or_default()
});
static BAKE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("BAKE_WORDS"));
static MEAL_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("MEAL_WORDS"));

/// True when a dish is named in this line or has been named a turn or two ago.
pub fn _dish_in_flight(memory: &Memory, understood: &Understood, _food: &str) -> bool {
    let said = _recent_answers(memory);
    if tokens(&said).iter().any(|w| DISH_WORDS.contains(w) || BAKE_WORDS.contains(w)) {
        return true;
    }
    let text = _line_text(understood);
    for word in tokens(text) {
        if MEAL_WORDS.contains(&word) {
            continue;
        }
        let is_food = DISH_WORDS.contains(&word) || BAKE_WORDS.contains(&word) || COOKING_FOOD.contains(&word);
        if is_food && !DISLIKED.iter().any(|name| _same_word(&word, name)) {
            return true;
        }
    }
    for line in recent_texts(memory).iter().skip(1).take(3) {
        if tokens(line).iter().any(|w| DISH_WORDS.contains(w)) {
            return true;
        }
    }
    false
}

static SAID_DISLIKE: LazyLock<Vec<String>> = LazyLock::new(|| strs("SAID_DISLIKE"));
static DINNER_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("DINNER_ASK"));
static OTHER_PEOPLE: LazyLock<Vec<String>> = LazyLock::new(|| strs("OTHER_PEOPLE"));
static SAID_DISLIKE_OTHER: LazyLock<Vec<String>> = LazyLock::new(|| strs("SAID_DISLIKE_OTHER"));
static LIKING: LazyLock<Vec<String>> = LazyLock::new(|| strs("LIKING"));
static DIET_FAMILIES: LazyLock<Vec<String>> = LazyLock::new(|| strs("DIET_FAMILIES"));
static EATING_REFUSALS: LazyLock<Vec<String>> = LazyLock::new(|| strs("EATING_REFUSALS"));

fn curly_lower(text: &str) -> String {
    text.to_lowercase().replace('\u{2019}', "'")
}

/// True when the line is a person refusing a food, not a bare "no" on a dinner.
pub fn _said_dislike(text: &str) -> bool {
    _names_any(&curly_lower(text), &SAID_DISLIKE)
}

/// True when the line asks for something to cook.
pub fn _asks_for_a_dinner(text: &str) -> bool {
    _names_any(&curly_lower(text), &DINNER_ASK)
}

/// True when the refusal in the line is somebody else's.
pub fn _somebody_else_refuses(text: &str) -> bool {
    let lowered = curly_lower(text);
    if !_names_any(&lowered, &SAID_DISLIKE_OTHER) {
        return false;
    }
    let people: Vec<&str> = OTHER_PEOPLE.iter().map(|p| p.trim()).collect();
    _names_any(&lowered, &people)
}

/// True when the line says one taste it refuses and another it likes.
pub fn _two_tastes(text: &str) -> bool {
    for part in curly_lower(text).split('|') {
        if _names_any(part, &DISLIKE_SHAPES) {
            continue;
        }
        if _names_any(part, &LIKING) {
            return true;
        }
    }
    false
}

/// True when the word the line rules out is a family of foods, not one food.
pub fn _a_diet_family(food: &str) -> bool {
    DIET_FAMILIES.iter().any(|f| _same_word(food, f))
}

/// The ingredient the line is ruling out, as it is written, or None.
pub fn _disliked_food(have: &[String]) -> Option<String> {
    for (index, word) in have.iter().enumerate() {
        if !DISLIKED.iter().any(|name| _same_word(word, name)) {
            continue;
        }
        let before = have[index.saturating_sub(4)..index].join(" ");
        if !_names_any(&before, &DISLIKE_SHAPES) {
            continue;
        }
        if !_names_any(&before, &EATING_REFUSALS) && any_same(&have[index.saturating_sub(3)..index], &WASTE_WORDS) {
            continue;
        }
        if _same_word(word, "peppers")
            && any_same(&have[index.saturating_sub(2)..index], &["black", "white", "ground", "cracked", "peppercorn", "peppercorns"])
        {
            continue;
        }
        if let Some(after) = have.get(index + 1) {
            if !FOOD_TAIL_OK.iter().any(|w| _same_word(after, w)) {
                continue;
            }
        }
        return Some(word.clone());
    }
    None
}

static SHORTAGE_MARKERS: LazyLock<Vec<String>> = LazyLock::new(|| strs("SHORTAGE_MARKERS"));
static DISLIKE_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISLIKE_VERBS"));
static OWN_DISLIKE: LazyLock<Vec<String>> = LazyLock::new(|| strs("OWN_DISLIKE"));

/// True when the person in the line is not the one speaking.
pub fn _someone_elses_dislike(have: &[String], text: &str) -> bool {
    if _names_any(text, &OWN_DISLIKE) {
        return false;
    }
    have.iter().any(|w| OTHER_PEOPLE.contains(w))
}

/// The cooking table's swap rows ("swap-..."), with their first key part and their reply.
static SWAP_ROWS: LazyLock<Vec<(Vec<String>, Value)>> = LazyLock::new(|| {
    data::get("bot.knowledge.cooking.ENTRIES")
        .items()
        .iter()
        .filter(|row| row.attr("id").is_some_and(|id| id.str().starts_with("swap-")))
        .map(|row| {
            let names = row.attr("key").and_then(|k| k.items().first().map(|p| p.strs())).unwrap_or_default();
            (names, row.attr("reply").cloned().unwrap_or(Value::None))
        })
        .collect()
});

/// The kitchen's swap answer for a shortage, or None when the line is a dislike.
pub fn _a_shortage_the_swap_table_answers(text: &str, have: &[String], memory: &Memory, understood: &Understood) -> Option<String> {
    if _names_any(text, &DISLIKE_VERBS) {
        return None;
    }
    if CUPBOARD.is_match(text) {
        return None;
    }
    if !_names_any(text, &SHORTAGE_MARKERS) {
        return None;
    }
    for (names, reply) in SWAP_ROWS.iter() {
        if !any_same(have, names) {
            continue;
        }
        let said = match reply {
            Value::Fn(f) => {
                let mut m = memory.clone();
                crate::knowledge::registry::reply(f, &mut m, understood)
            }
            other => other.str().to_string(),
        };
        if !said.trim().is_empty() {
            return Some(said);
        }
    }
    None
}

static A_CHILD: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_CHILD"));
static COOK_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("COOK_ASK_WORDS"));
static EATING_REFUSAL: LazyLock<Vec<String>> = LazyLock::new(|| strs("EATING_REFUSAL"));
static LUNCH_MEALS: LazyLock<HashSet<String>> = LazyLock::new(|| set("LUNCH_MEALS"));

/// True when the line is a child's refusal to eat and nothing is asked to be cooked for it.
pub fn _a_child_who_will_not_eat(text: &str, have: &[String]) -> bool {
    if !_names_any(text, &A_CHILD) {
        return false;
    }
    if !_names_any(text, &EATING_REFUSAL) {
        return false;
    }
    !have.iter().any(|w| COOK_ASK_WORDS.contains(w))
}

/// The substitute in place of `food`, or None when it offers another refused food.
pub fn _swap_said(food: &str, said: &str, _understood: &Understood, text: &str) -> Option<String> {
    let refused: Vec<String> = dup::cooking::_refused_foods(text, true)
        .into_iter()
        .filter(|name| !_same_word(name, food))
        .collect();
    if !refused.is_empty() && any_same(&tokens(said), &refused) {
        if any_same(&tokens(&GENERIC_SWAP), &refused) {
            return None;
        }
        return Some(fmt_swap(&GENERIC_SWAP, food, ""));
    }
    Some(said.to_string())
}

/// The diet family this line itself refuses ("and no dairy"), or None.
pub fn _a_family_ruled_out(have: &[String]) -> Option<String> {
    for (index, word) in have.iter().enumerate() {
        if !_a_diet_family(word) {
            continue;
        }
        let before = have[index.saturating_sub(4)..index].join(" ");
        if _names_any(&before, &DISLIKE_SHAPES) {
            return Some(word.clone());
        }
    }
    None
}

static EGG_FREE_BAKE: LazyLock<String> = LazyLock::new(|| string("EGG_FREE_BAKE"));
static NUT_FREE_BAKE: LazyLock<String> = LazyLock::new(|| string("NUT_FREE_BAKE"));
static DEFER_TO_ALLERGY: LazyLock<Vec<String>> = LazyLock::new(|| strs("DEFER_TO_ALLERGY"));
static ALLERGENS: LazyLock<Vec<String>> = LazyLock::new(|| strs("ALLERGENS"));
static ALLERGEN_NOTE: LazyLock<String> = LazyLock::new(|| string("_ALLERGEN_NOTE"));
static EGG_DISH_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("EGG_DISH_WORDS"));
static EGG_IS_THE_DISH: LazyLock<String> = LazyLock::new(|| string("EGG_IS_THE_DISH"));
static NO_APPLIANCE: LazyLock<&'static Re> = LazyLock::new(|| rx("NO_APPLIANCE"));

fn allergen_note(food: &str) -> &'static str {
    if _an_allergen(food) {
        ALLERGEN_NOTE.as_str()
    } else {
        ""
    }
}

/// The kitchen tables' own answer for the line (no appliance, a cupboard, a meal), or None.
fn kitchen_tables_dislike(understood: &Understood, text: &str) -> Option<String> {
    if let Some(said) = dup::cooking::without(understood).filter(|s| !s.trim().is_empty()) {
        return Some(said);
    }
    if let Some(said) = dup::cooking::dish(understood, &[], None).filter(|s| !s.trim().is_empty()) {
        return Some(said);
    }
    let args = dup::cooking::MealArgs { text: Some(text), ..Default::default() };
    dup::cooking::meal(understood, args).filter(|s| !s.trim().is_empty())
}

/// What to put in place of the ingredient someone does not eat.
pub fn _dislike(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    let have = tokens(text);
    if have.iter().any(|w| LUNCH_MEALS.contains(w)) {
        return None;
    }
    if _a_child_who_will_not_eat(text, &have) && !_names_any(text, &SHORTAGE_MARKERS) {
        return None;
    }
    if let Some(shortage) = _a_shortage_the_swap_table_answers(text, &have, memory, understood) {
        return Some(shortage);
    }
    let mut food = _disliked_food(&have);
    let family = _a_family_ruled_out(&have);
    if let Some(family) = &family {
        if food.as_ref().is_none_or(|f| !_same_word(f, family)) {
            return None;
        }
    }
    if food.as_ref().is_some_and(|f| _a_diet_family(f)) {
        return None;
    }
    if food.is_some() && CUPBOARD.is_match(text) {
        food = None;
    }
    if let Some(f) = food.clone() {
        if !_dish_in_flight(memory, understood, &f) && _someone_elses_dislike(&have, text) {
            for (names, said) in SUBSTITUTES.iter() {
                if names.iter().any(|n| _same_word(&f, n)) {
                    let said = _swap_said(&f, &fmt_swap(said, &f, _another_meat(&f)), understood, text)?;
                    return Some(said + allergen_note(&f));
                }
            }
            return _swap_said(&f, &fmt_swap(&GENERIC_SWAP, &f, ""), understood, text);
        }
    }
    if let Some(f) = food.clone() {
        if !_dish_in_flight(memory, understood, &f) {
            if _two_tastes(text) {
                return None;
            }
            if _states_what_they_have(text) && !_somebody_else_refuses(text) {
                food = None;
            } else if !(_foods_named(understood)
                || _somebody_else_refuses(text)
                || (_said_dislike(text) && _asks_for_a_dinner(text)))
            {
                food = None;
            }
        }
    }
    if food.is_none() && !(HOW_WORDS.is_match(text) || NO_APPLIANCE.is_match(text)) {
        return None;
    }
    if food.is_none() && !_food_context(memory, understood) {
        return None;
    }
    let food = match food {
        Some(f) if !HOW_WORDS.is_match(text) => f,
        _ => return kitchen_tables_dislike(understood, text),
    };
    if !_food_context(memory, understood) {
        return None;
    }
    let is_egg = _same_word(&food, "egg") || _same_word(&food, "eggs");
    if _bake_in_flight(memory) && is_egg {
        return Some(EGG_FREE_BAKE.clone());
    }
    if is_egg && _an_egg_dish(memory) {
        return Some(EGG_IS_THE_DISH.clone());
    }
    if DEFER_TO_ALLERGY.iter().any(|n| _same_word(&food, n)) && _bake_in_flight(memory) {
        return Some(NUT_FREE_BAKE.clone());
    }
    if OUT_OF_CUE.is_match(text) && KITCHEN_SWAP.iter().any(|n| _same_word(&food, n)) {
        return None;
    }
    for (names, said) in SUBSTITUTES.iter() {
        if names.iter().any(|n| _same_word(&food, n)) {
            let said = fmt_swap(said, &food, _another_meat(&food));
            return Some(format!("{said}{}{}", allergen_note(&food), _the_dish_still(memory, &food, &said)));
        }
    }
    let generic = fmt_swap(&GENERIC_SWAP, &food, "");
    Some(format!("{generic}{}{}", allergen_note(&food), _the_dish_still(memory, &food, &generic)))
}

fn r_dislike(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _dislike(m, u)
}

static TIME_IN_A_STEP: LazyLock<&'static Re> = LazyLock::new(|| rx("TIME_IN_A_STEP"));
static DISH_LEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("DISH_LEAD"));
static STEP_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| strs("STEP_VERBS"));

/// The dish in hand in one of its own sentences, to put after a substitution, or "".
pub fn _the_dish_still(memory: &Memory, food: &str, said: &str) -> String {
    static SENTENCES: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(4);
    for turn in turns[start..].iter().rev() {
        let answer = turn.bot.trim();
        if answer.is_empty() || said.contains(answer) {
            continue;
        }
        for sentence in SENTENCES.split(answer) {
            let sentence = sentence.trim();
            if sentence.is_empty() || !TIME_IN_A_STEP.is_match(sentence) {
                continue;
            }
            if tokens(sentence).iter().any(|item| _same_word(item, food)) {
                continue;
            }
            let last = sentence.rsplit(':').next().unwrap_or("");
            let head: String = last.chars().take(24).collect();
            if _phrase_in(said, &head.trim().to_lowercase()) {
                continue;
            }
            let mut step = DISH_LEAD.sub("", sentence).trim().to_string();
            if step.chars().next().is_some_and(|c| c.is_lowercase()) {
                let mut cs = step.chars();
                let first = cs.next().unwrap();
                step = format!("{}{}", first.to_uppercase(), cs.as_str());
            }
            let words = tokens(&step);
            if !words.is_empty() && words.iter().any(|w| STEP_VERBS.contains(w)) {
                return format!(" {step}");
            }
            return String::new();
        }
    }
    String::new()
}

/// True when a conversation line names one of `words`.
pub fn _a_dish_in_flight(memory: &Memory, words: &[String]) -> bool {
    recent_texts(memory).iter().take(4).any(|line| any_same(&tokens(line), words))
}

/// True when the conversation is on a cake, a biscuit or a loaf, where nuts are the allergy.
pub fn _bake_in_flight(memory: &Memory) -> bool {
    _a_dish_in_flight(memory, &BAKE_WORDS)
}

/// True when the conversation is on a dish made of eggs: an omelette or a quiche.
pub fn _an_egg_dish(memory: &Memory) -> bool {
    _a_dish_in_flight(memory, &EGG_DISH_WORDS)
}

/// True when what is being left out is one of the things that cause reactions.
pub fn _an_allergen(food: &str) -> bool {
    ALLERGENS.iter().any(|n| _same_word(food, n))
}

// ---------------------------------------------------------------------------------------------
// Pudding

static PUDDINGS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| groups("PUDDINGS"));
static CROWD_PUDDING: LazyLock<String> = LazyLock::new(|| string("CROWD_PUDDING"));
static CROWD_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("CROWD_WORDS"));
static FRUIT_PUDDING: LazyLock<String> = LazyLock::new(|| string("FRUIT_PUDDING"));
static DESSERT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DESSERT_WORDS"));
static ATE_ALREADY: LazyLock<&'static Re> = LazyLock::new(|| rx("ATE_ALREADY"));
static REWARDED: LazyLock<&'static Re> = LazyLock::new(|| rx("REWARDED"));
static PUDDING_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("PUDDING_ASK"));

/// Six or more people, however the conversation says so.
pub fn _a_crowd(memory: &Memory, text: &str, _have: &[String]) -> bool {
    let mut lines = vec![text.to_string()];
    lines.extend(recent_texts(memory));
    for line in &lines {
        let words = tokens(line);
        if CROWD_WORDS.iter().any(|w| _says(line, w, &words)) {
            return true;
        }
        for chunk in &words {
            if !chunk.is_empty() && chunk.bytes().all(|b| b.is_ascii_digit()) {
                let big = chunk.trim_start_matches('0').len() > 3;
                if !big {
                    let n: u32 = chunk.parse().unwrap_or(0);
                    if (6..=200).contains(&n) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// The word itself, not its loose form: "after" is not "afters", "sweetcorn" is not "sweet".
pub fn _says(text: &str, word: &str, have: &[String]) -> bool {
    if word.contains(' ') {
        return text.contains(word);
    }
    have.iter().any(|w| w == word)
}

/// The pudding that fits the constraint in the line.
pub fn _pudding_for(have: &[String], text: &str) -> Option<String> {
    let line = tokens(text);
    for (phrases, said) in PUDDINGS.iter() {
        for phrase in phrases {
            if phrase_in_tokens(&line, &tokens(phrase)) || have.iter().any(|w| _same_word(w, phrase)) {
                return Some(said.clone());
            }
        }
    }
    None
}

/// What to have for pudding, asked as the second half of a dinner conversation.
pub fn _pudding(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    let have = tokens(text);
    if !DESSERT_WORDS.iter().any(|w| _says(text, w, &have)) {
        return None;
    }
    if REWARDED.is_match(text) {
        return None;
    }
    if ATE_ALREADY.is_match(text) && !PUDDING_ASK.is_match(text) {
        return None;
    }
    if _diet(memory, understood).is_some_and(|s| !s.is_empty()) {
        return None;
    }
    if let Some(said) = _pudding_for(&have, text).filter(|s| !s.is_empty()) {
        return Some(said);
    }
    if _a_crowd(memory, text, &have) {
        return Some(CROWD_PUDDING.clone());
    }
    Some(FRUIT_PUDDING.clone())
}

fn r_pudding(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _pudding(m, u)
}

// ---------------------------------------------------------------------------------------------
// Is the conversation about food at all

const FOOD_VERBS: [&str; 14] =
    ["make", "made", "cook", "cooking", "fix", "fixing", "taste", "tastes", "tasted", "came", "out", "turned", "went", "wrong"];

/// The foods both tables know (DISH_WORDS and `cooking._FOOD`), without the verbs.
pub fn _food_words() -> &'static HashSet<String> {
    static FOODS: LazyLock<HashSet<String>> = LazyLock::new(|| {
        DISH_WORDS
            .iter()
            .chain(COOKING_FOOD.iter())
            .filter(|w| !FOOD_VERBS.contains(&w.as_str()))
            .cloned()
            .collect()
    });
    &FOODS
}

/// True when this line is part of a conversation about cooking and eating.
pub fn _food_context(memory: &Memory, understood: &Understood) -> bool {
    let text = _line_text(understood).to_string();
    let mut lines = vec![text];
    lines.extend(recent_texts(memory).into_iter().take(3));
    let foods = _food_words();
    for line in &lines {
        if tokens(line).iter().any(|w| foods.contains(w) || MEAL_WORDS.contains(w)) {
            return true;
        }
    }
    false
}

static IDEAS: LazyLock<Vec<(String, Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&d("IDEAS"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].str().to_string(), p[1].strs(), p[2].str().to_string())
        })
        .collect()
});

/// The idea for the first food in the line, or None: (key, the idea).
pub fn _idea_for(have: &[String]) -> Option<(&'static str, &'static str)> {
    for (key, names, words) in IDEAS.iter() {
        for name in names {
            if have.iter().any(|w| _same_word(w, name)) {
                return Some((key.as_str(), words.as_str()));
            }
        }
    }
    None
}

/// True when the bot's last reply was this idea already.
pub fn _already(reply: &str, memory: &Memory) -> bool {
    let opening = reply.to_lowercase().split_whitespace().take(4).collect::<Vec<_>>().join(" ");
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(2);
    if let Some(turn) = turns[start..].iter().next_back() {
        return turn.bot.to_lowercase().split_whitespace().take(4).collect::<Vec<_>>().join(" ") == opening;
    }
    false
}

/// The plan dish the line names as a dish rather than as an ingredient.
pub fn _named_dish(text: &str) -> Option<&'static str> {
    let key = dish_in(text)?;
    let p = plan(key)?;
    let have = tokens(text);
    for (name, toks) in p.each_name() {
        if name.split_whitespace().count() > 1 && phrase_in_tokens(&have, toks) {
            return Some(key);
        }
    }
    None
}

/// The ways a line says somebody does not eat one food, built from the food's own name.
pub fn _negative_forms(food: &str) -> Vec<String> {
    let mut forms: Vec<String> = Vec::new();
    for subject_verb in
        ["doesnt", "doesn t", "does not", "didnt", "did not", "wont", "won t", "will not", "cant", "can t", "can not", "cannot", "never"]
    {
        for eat in ["eat", "eats", "have", "has"] {
            forms.push(format!("{subject_verb} {eat} {food}"));
        }
    }
    for frame in [
        "avoids {}", "avoid {}", "avoiding {}", "off {}", "is off {}", "no {} at all", "nothing with {}", "{} is out", "cut out {}",
        "cutting out {}", "free of {}", "free from {}",
    ] {
        forms.push(frame.replace("{}", food));
    }
    let mut seen = HashSet::new();
    forms.retain(|f| seen.insert(f.clone()));
    forms
}

static DIETS: LazyLock<Vec<(String, Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&d("DIETS"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].str().to_string(), p[1].strs(), p[2].str().to_string())
        })
        .collect()
});
static DAIRY_FREE_VEGGIE: LazyLock<String> = LazyLock::new(|| string("DAIRY_FREE_VEGGIE"));
static DIET_MEAL_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("DIET_MEAL_ASK"));
static DIET_NOT_THIS: LazyLock<&'static Re> = LazyLock::new(|| rx("DIET_NOT_THIS"));
static DIET_TOO_MEDICAL: LazyLock<&'static Re> = LazyLock::new(|| rx("DIET_TOO_MEDICAL"));
static DIET_RECALL: LazyLock<&'static Re> = LazyLock::new(|| rx("DIET_RECALL"));
static VEGAN_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("VEGAN_WORD"));
static GUEST_DIET: LazyLock<&'static Re> = LazyLock::new(|| rx("GUEST_DIET"));
static COOKING_VERB: LazyLock<&'static Re> = LazyLock::new(|| rx("COOKING_VERB"));
static MAKE_VERB: LazyLock<&'static Re> = LazyLock::new(|| rx("MAKE_VERB"));
static VEGGIE_KEY: LazyLock<Vec<String>> = LazyLock::new(|| strs("VEGGIE_KEY"));
static VEGAN_IDEA: LazyLock<String> = LazyLock::new(|| string("VEGAN_IDEA"));
static VEGGIE_IDEA: LazyLock<String> = LazyLock::new(|| string("VEGGIE_IDEA"));
static VEGGIE_SWAP_IDEA: LazyLock<String> = LazyLock::new(|| string("VEGGIE_SWAP_IDEA"));
static VEGAN_SWAP_IDEA: LazyLock<String> = LazyLock::new(|| string("VEGAN_SWAP_IDEA"));

/// True when the line names something you cook, as `_food_context` counts foods.
pub fn _names_a_food(line: &str) -> bool {
    let foods = _food_words();
    tokens(line).iter().any(|w| foods.contains(w))
}

/// The person's own lines before this one (non-empty user lines).
pub fn _earlier_turns(memory: &Memory) -> Vec<&crate::state::Turn> {
    memory.turns.iter().filter(|t| !t.user.is_empty()).collect()
}

/// True when there is a food on the record but nothing is being cooked.
pub fn _a_food_word_with_no_cooking(memory: &Memory, understood: &Understood) -> bool {
    !_earlier_turns(memory).is_empty() && !cooking_under_way(memory, understood)
}

/// Whether the conversation, in the person's own words, is about cooking something.
pub fn cooking_under_way(memory: &Memory, understood: &Understood) -> bool {
    let mut lines = vec![_line_text(understood).to_string()];
    lines.extend(recent_texts(memory).into_iter().take(4));
    for line in &lines {
        if line.is_empty() {
            continue;
        }
        if COOKING_VERB.is_match(line) {
            return true;
        }
        if MAKE_VERB.is_match(line) && _names_a_food(line) {
            return true;
        }
    }
    false
}

/// A vegetarian dinner, when the line is about someone in the party not eating meat.
pub fn _veggie(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    if DIET_RECALL.is_match(text) {
        return None;
    }
    if !_food_context(memory, understood) && !GUEST_DIET.is_match(text) {
        return None;
    }
    if _asks_about_the_diet(text) {
        return None;
    }
    if _a_food_word_with_no_cooking(memory, understood) {
        return Some(if VEGAN_WORD.is_match(text) { VEGAN_SWAP_IDEA.clone() } else { VEGGIE_SWAP_IDEA.clone() });
    }
    if VEGAN_WORD.is_match(text) {
        return Some(VEGAN_IDEA.clone());
    }
    Some(VEGGIE_IDEA.clone())
}

fn r_veggie(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _veggie(m, u)
}

static YES_NO_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("YES_NO_ASK"));
static COOK_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("COOK_ASK"));

/// True when the line is a yes/no question about the diet itself, not a request for a meal.
pub fn _asks_about_the_diet(text: &str) -> bool {
    YES_NO_ASK.is_match(text) && !COOK_ASK.is_match(text)
}

/// What to swap for a diet the line says they are on, when the line is about cooking.
pub fn _diet(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    if text.trim().is_empty() {
        return None;
    }
    if DIET_RECALL.is_match(text) || DIET_NOT_THIS.is_match(text) || DIET_TOO_MEDICAL.is_match(text) || DIET_MEAL_ASK.is_match(text) {
        return None;
    }
    let have = tokens(text);
    for (_name, words, said) in DIETS.iter() {
        if !words.iter().any(|w| phrase_in_tokens(&have, &tokens(w))) {
            continue;
        }
        if VEGGIE_KEY.iter().any(|w| phrase_in_tokens(&have, &tokens(w))) {
            return Some(DAIRY_FREE_VEGGIE.clone());
        }
        if _already(said, memory) {
            return None;
        }
        return Some(said.clone());
    }
    None
}

fn r_diet(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _diet(m, u)
}

/// Whether the meat-free dinner is the answer to this line.
pub fn has_a_veggie_reply(memory: &Memory, understood: &Understood) -> bool {
    _veggie(memory, understood).is_some_and(|s| !s.is_empty())
}

/// Whether the diet table has a reply for this line.
pub fn has_a_diet_reply(memory: &Memory, understood: &Understood) -> bool {
    _diet(memory, understood).is_some_and(|s| !s.is_empty())
}

/// Whether the substitution table has a reply for the food this line rules out.
pub fn has_a_dislike_reply(memory: &Memory, understood: &Understood) -> bool {
    _dislike(memory, understood).is_some_and(|s| !s.is_empty())
}

/// The last two things the bot said, as one string.
pub fn _recent_answers(memory: &Memory) -> String {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(2);
    turns[start..].iter().map(|t| t.bot.as_str()).collect::<Vec<_>>().join(" ")
}

/// The food word in this line the idea list knows, as it is written, or None.
pub fn _idea_food(have: &[String]) -> Option<String> {
    for (_key, names, _words) in IDEAS.iter() {
        for name in names {
            for word in have {
                if _same_word(word, name) {
                    return Some(word.clone());
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------------------------
// One dish that fits the constraint

static PICK_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PICK_WORDS"));
static NICE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("NICE_WORDS"));
static GUEST_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("GUEST_WORDS"));
static CLEANING_IT: LazyLock<&'static Re> = LazyLock::new(|| rx("_CLEANING_IT"));
static PICK_ONE: LazyLock<&'static Re> = LazyLock::new(|| rx("_PICK_ONE"));
static APPLIANCE_IDEAS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| groups("APPLIANCE_IDEAS"));
static COMPARE_IDEAS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| groups("COMPARE_IDEAS"));
static OVEN_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("OVEN_WORD"));
static TRAY_IDEA: LazyLock<String> = LazyLock::new(|| string("TRAY_IDEA"));
static NICE_IDEA: LazyLock<String> = LazyLock::new(|| string("NICE_IDEA"));
static KIDS_IDEA: LazyLock<String> = LazyLock::new(|| string("KIDS_IDEA"));
static CHEAP_IDEA: LazyLock<String> = LazyLock::new(|| string("CHEAP_IDEA"));
static AHEAD_IDEA: LazyLock<String> = LazyLock::new(|| string("AHEAD_IDEA"));
static WHOLE_FOOD: LazyLock<&'static Re> = LazyLock::new(|| rx("WHOLE_FOOD"));
static CUT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("CUT_WORDS"));
static OR_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("OR_WORD"));
static PRONOUN: LazyLock<&'static Re> = LazyLock::new(|| rx("PRONOUN"));
static SAFETY_WORDS: LazyLock<&'static Re> = LazyLock::new(|| rx("SAFETY_WORDS"));

/// True when the line names two appliances, so it is choosing between them.
pub fn _two_appliances(text: &str) -> bool {
    let mut named: HashSet<&str> = APPLIANCE_IDEAS
        .iter()
        .filter(|(phrases, _)| _names_any(text, phrases))
        .map(|(phrases, _)| phrases[0].as_str())
        .collect();
    if OVEN_WORD.is_match(text) {
        named.insert("oven");
    }
    named.len() > 1
}

/// The appliance this line is about, as a dinner idea, or None.
pub fn _appliance(text: &str) -> Option<String> {
    if CLEANING_IT.is_match(text) {
        return None;
    }
    if _two_appliances(text) && PICK_ONE.is_match(text) {
        return None;
    }
    for (phrases, reply_text) in APPLIANCE_IDEAS.iter() {
        if _names_any(text, phrases) {
            return Some(reply_text.clone());
        }
    }
    if OVEN_WORD.is_match(text) {
        return Some(TRAY_IDEA.clone());
    }
    None
}

/// True when a line or two ago the person said what they are cooking tonight.
pub fn _already_cooking(memory: &Memory) -> bool {
    recent_texts(memory).iter().take(2).any(|line| WHOLE_FOOD.is_match(line))
}

/// True when the line names both an appliance and a cut of meat.
pub fn _names_an_appliance_and_a_cut(text: &str) -> bool {
    let have = tokens(text);
    let appliance = APPLIANCE_IDEAS.iter().any(|(phrases, _)| phrases.iter().any(|p| phrase_in_tokens(&have, &tokens(p))));
    if !appliance && !OVEN_WORD.is_match(text) {
        return false;
    }
    have.iter().any(|w| CUT_WORDS.contains(&w.to_lowercase()))
}

/// One dish that fits the constraint this line gives, in a conversation about dinner.
pub fn _idea(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    if _names_an_appliance_and_a_cut(text) {
        return None;
    }
    let have = tokens(text);
    if have.is_empty() {
        return None;
    }
    if !_food_context(memory, understood) {
        return None;
    }
    if _named_dish(text).is_some() {
        return None;
    }
    if _already_cooking(memory) {
        return None;
    }
    if SAFETY_WORDS.is_match(text) {
        return None;
    }
    if any_same(&have, &["kids", "kid", "children", "child", "toddler", "toddlers", "family"]) {
        return Some(KIDS_IDEA.clone());
    }
    if let Some(said) = dup::cooking::dish(understood, &[], None).filter(|s| !s.trim().is_empty()) {
        return Some(said);
    }
    let args = dup::cooking::MealArgs { text: Some(text), ..Default::default() };
    if let Some(said) = dup::cooking::meal(understood, args).filter(|s| !s.trim().is_empty()) {
        return Some(said);
    }
    if let Some(said) = dup::cooking::without(understood).filter(|s| !s.trim().is_empty()) {
        return Some(said);
    }
    if let Some(named) = _idea_food(&have) {
        if tokens(&_recent_answers(memory)).contains(&named) {
            return None;
        }
    }
    if OR_WORD.is_match(text) {
        for (phrases, reply_text) in COMPARE_IDEAS.iter() {
            if phrases.iter().all(|p| phrase_in_tokens(&have, &tokens(p))) {
                return Some(reply_text.clone());
            }
        }
    }
    let mut named = _appliance(text);
    if named.is_none() && PRONOUN.is_match(text) {
        for earlier in recent_texts(memory).iter().take(3) {
            if NO_APPLIANCE.is_match(earlier) {
                continue;
            }
            named = _appliance(earlier);
            if named.is_some() {
                break;
            }
        }
    }
    if named.is_some() {
        return named;
    }
    if any_same(&have, &["cheap", "cheaper", "cheapest", "budget", "broke"]) {
        return Some(CHEAP_IDEA.clone());
    }
    if any_same(&have, &["tomorrow", "tomorrows"]) {
        return Some(AHEAD_IDEA.clone());
    }
    let found = _idea_for(&have);
    if found.is_none() && (any_same(&have, &GUEST_WORDS) || any_same(&have, &NICE_WORDS)) {
        return Some(NICE_IDEA.clone());
    }
    let Some((key, reply_text)) = found else {
        if !any_same(&have, &PICK_WORDS) {
            return None;
        }
        for (_, _, fallback) in IDEAS.iter() {
            if !_already(fallback, memory) {
                return Some(fallback.clone());
            }
        }
        return Some(IDEAS[0].2.clone());
    };
    if _already(reply_text, memory) {
        for (other_key, _, other) in IDEAS.iter() {
            if other_key != key && !_already(other, memory) {
                return Some(other.clone());
            }
        }
    }
    Some(reply_text.to_string())
}

fn r_idea(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _idea(m, u)
}

// ---------------------------------------------------------------------------------------------
// Baking faults the cooking table had no room for

static DOUGH_STICK_LEAD: LazyLock<String> = LazyLock::new(|| string("DOUGH_STICK_LEAD"));

/// Why the dough sticks, led by the surface the line names.
pub fn _dough_sticks(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    let lead = DOUGH_STICK_LEAD.as_str();
    if text.contains("banneton") || text.contains("basket") || text.contains("brotform") {
        return Some(format!("{lead}Dust the banneton generously with rice flour, which does not absorb the way wheat flour does, or line it with a well-floured cloth, and chill the shaped loaf 20 minutes before it goes in."));
    }
    if text.contains("pastry") || text.contains("shortcrust") || text.contains("pie") || text.contains("tart") {
        let pin = if text.contains("pin") { " Dust the pin as well." } else { "" };
        return Some(format!("{lead}Chill the pastry 20 to 30 minutes before rolling, keep the flour light and roll from the middle out with short strokes: warm pastry sticks however much flour you use.{pin}"));
    }
    if text.contains("hands") || text.contains("fingers") {
        return Some(format!("{lead}Wet your hands or rub them with a little oil, and lift the dough with a scraper instead of pulling at it."));
    }
    if text.contains("pin") || text.contains("rolling") {
        return Some(format!("{lead}Dust the pin lightly, lift the dough with a scraper rather than a hand, and give it a quarter turn after each roll."));
    }
    Some(format!("{lead}Keep a light dusting of flour on the bench, lift the dough with a scraper instead of pulling at it, and add strong white flour a spoonful at a time if the dough itself is too wet to handle."))
}

fn r_dough_sticks(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _dough_sticks(m, u)
}

/// `has(*wanted)` inside `_bake_fault_more`: the line says one of these words, loosely.
pub fn _bake_fault_more__has(words: &[String], wanted: &[&str]) -> bool {
    any_same(words, wanted)
}

/// A waffle iron, a tray of flapjacks, a brownie and a batch of scones go wrong in their own ways.
pub fn _bake_fault_more(_memory: &Memory, understood: &Understood) -> Option<String> {
    let words = lower_words(understood);
    let has = |wanted: &[&str]| _bake_fault_more__has(&words, wanted);
    if has(&["waffle", "waffles"]) {
        return Some("A soggy waffle is a batter that is too thin or an iron that is not hot enough: make it thicker than pancake batter - it should plop off the ladle rather than pour - let it rest 10 minutes so the flour takes up the liquid, and heat the iron until a drop of water skitters on it. Fill it three-quarters so the steam can escape, and stand the cooked ones on a rack in a low oven rather than stacking them, because a stack steams itself soft.".into());
    }
    if has(&["flapjack", "flapjacks", "oat bar", "oat bars"]) {
        if has(&["set", "firm", "firmed", "runny", "syrup", "sloppy", "sticky", "soggy", "soft"]) {
            return Some("A flapjack that will not set wants less liquid or more oats: about 25g more oats and 2 tablespoons less syrup, pressed down hard into the tin with the back of a spoon before it bakes. Bake it at 180C (160C fan) for 25 minutes until the edges are golden, then leave it in the tin until it is completely cold - warm flapjacks are always soft and they firm up as they cool.".into());
        }
        return Some("Flapjacks that crumble were cut warm or short of syrup: cool them completely in the tin, then cut with a sharp knife straight down, and add 2 tablespoons more golden syrup or honey to the next batch. Pack the mix down firmly before it bakes, because a loose tray bakes into crumbs.".into());
    }
    if has(&["brownie", "brownies"]) {
        if has(&["dry", "cakey", "cake-like", "crumbly", "overbaked", "overbaked", "too long", "hard", "stale"]) {
            return Some("A dry brownie stayed in the oven too long or went into too big a tin: 20 to 25 minutes at 180C (160C fan) in a 20cm square tin, and take it out while the middle still looks slightly under, because it sets as it cools. Melt the butter and chocolate together gently rather than boiling them, and keep the flour light - it is there to hold the brownie together, not to make a cake.".into());
        }
        return None;
    }
    if has(&["scone", "scones"]) {
        return Some("Dense scones come from working the dough or a wet mix: use cold butter, rub it in until it looks like breadcrumbs, and bring it together with a knife and one fold rather than kneading. Pat it 2cm thick, cut straight down with a floured cutter and bake at 220C (200C fan) for 12 to 15 minutes, because a slow oven sets them heavy.".into());
    }
    None
}

fn r_bake_fault_more(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _bake_fault_more(m, u)
}

/// The veg-tough row's reply: the fix for the vegetable in the line.
pub fn lambda_4874(_memory: &Memory, understood: &Understood) -> Option<String> {
    _veg_fix(_line_text(understood))
}

fn r_lambda_4874(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    lambda_4874(m, u)
}

static TWO_OVEN_HEAD: LazyLock<String> = LazyLock::new(|| string("_TWO_OVEN_HEAD"));
static TWO_OVEN_GAP: LazyLock<String> = LazyLock::new(|| string("_TWO_OVEN_GAP"));
static TWO_OVEN_CAKE: LazyLock<String> = LazyLock::new(|| string("_TWO_OVEN_CAKE"));
static TWO_OVEN_SPONGE: LazyLock<String> = LazyLock::new(|| string("_TWO_OVEN_SPONGE"));
static TWO_OVEN_BEEF: LazyLock<Vec<String>> = LazyLock::new(|| strs("_TWO_OVEN_BEEF"));

/// The oven answer for the pair the line names: a cake and a roast first, the rest after.
pub fn _two_dishes_plan(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = format!(" {} ", understood.clean.to_lowercase());
    if text.contains("cake") && (text.contains("roast") || TWO_OVEN_BEEF.iter().any(|w| text.contains(&format!(" {w} ")))) {
        return Some([TWO_OVEN_CAKE.as_str(), TWO_OVEN_SPONGE.as_str()].join(" "));
    }
    Some([TWO_OVEN_HEAD.as_str(), TWO_OVEN_GAP.as_str(), TWO_OVEN_CAKE.as_str(), TWO_OVEN_SPONGE.as_str()].join(" "))
}

fn r_two_dishes_plan(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _two_dishes_plan(m, u)
}

// ---------------------------------------------------------------------------------------------
// What a named dish is

static DISH_WHAT: LazyLock<Vec<(String, Vec<String>, String)>> = LazyLock::new(|| {
    data::get(&d("DISH_WHAT"))
        .items()
        .iter()
        .map(|t| {
            let p = t.items();
            (p[0].str().to_string(), p[1].strs(), p[2].str().to_string())
        })
        .collect()
});
static DISH_WHAT_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISH_WHAT_ASKS"));
static DISH_WHAT_LEAD: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISH_WHAT_LEAD"));
static DISH_BARE_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("DISH_BARE_NAMES"));

/// True when the line opens with a definition ask, after at most a short lead ("so", "ok").
pub fn _opens_with_the_ask(said: &str) -> bool {
    static LEADS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
        let mut leads: Vec<&String> = DISH_WHAT_LEAD.iter().collect();
        leads.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
        leads.into_iter().map(|l| l.split_whitespace().map(str::to_string).collect()).collect()
    });
    let mut words: Vec<&str> = said.trim_matches(|c| " .!?,".contains(c)).split_whitespace().collect();
    for lead in LEADS.iter() {
        if lead.is_empty() {
            continue;
        }
        if words.len() >= lead.len() && words[..lead.len()].iter().zip(lead.iter()).all(|(a, b)| a == b) {
            words = words[lead.len()..].to_vec();
            break;
        }
    }
    let head = words.iter().take(3).copied().collect::<Vec<_>>().join(" ");
    DISH_WHAT_ASKS.iter().any(|ask| head.starts_with(ask.as_str()))
}

/// True when a bare family word ("pie", "pudding") is qualified, so this row would only guess.
pub fn _qualified_bare_word(names: &[String], said: &str) -> bool {
    let found: Vec<&String> = names.iter().filter(|n| said.contains(n.as_str())).collect();
    if found.is_empty() || found.iter().any(|n| !DISH_BARE_NAMES.contains(n)) {
        return false;
    }
    for name in found {
        let noun = name.split_whitespace().last().unwrap_or("");
        let (head, _, tail) = crate::py::text::partition(said, &format!(" {noun} "));
        if let Some(last) = head.split_whitespace().last() {
            if !["a", "an", "the", "and", "or", "this", "that", "my", "is", "are", "was", "were"].contains(&last) {
                return true;
            }
        }
        if let Some(first) = tail.split_whitespace().next() {
            if !["for", "is", "are", "and", "or", "with", "s"].contains(&first) {
                return true;
            }
        }
    }
    false
}

/// The sort key of the rows found: the longest name a row has.
pub fn _what_is_a_dish__lambda_6273(names: &[String]) -> usize {
    names.iter().map(|n| n.chars().count()).max().unwrap_or(0)
}

/// What a named dish is, when the line asks that and nothing else.
pub fn _what_is_a_dish(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = sh::or(&understood.clean, "").to_string();
    let said = if said.is_empty() { understood.words.join(" ") } else { said }.to_lowercase();
    if !_opens_with_the_ask(&said) {
        return None;
    }
    let said = format!(" {said} ");
    let found: Vec<&(String, Vec<String>, String)> =
        DISH_WHAT.iter().filter(|row| row.1.iter().any(|n| said.contains(&format!(" {n} ")))).collect();
    if found.is_empty() {
        return None;
    }
    let mut found: Vec<&(String, Vec<String>, String)> =
        found.into_iter().filter(|row| !_qualified_bare_word(&row.1, &said)).collect();
    if found.is_empty() {
        return None;
    }
    found.sort_by(|a, b| _what_is_a_dish__lambda_6273(&b.1).cmp(&_what_is_a_dish__lambda_6273(&a.1)));
    let mut longest = &found[0].1[0];
    for n in &found[0].1 {
        if n.chars().count() > longest.chars().count() {
            longest = n;
        }
    }
    let top = format!(" {longest} ");
    let first = found[0];
    let mut kept = vec![first];
    kept.extend(found[1..].iter().copied().filter(|row| !row.1.iter().any(|n| top.contains(&format!(" {n} ")))));
    if kept.len() > 1 {
        return Some(format!("{} Or the other way round: {}", kept[0].2, kept[1].2));
    }
    Some(kept[0].2.clone())
}

fn r_what_is_a_dish(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _what_is_a_dish(m, u)
}

fn r_problem(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _problem(m, u)
}
fn r_grainy(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _grainy(m, u)
}
fn r_gluey(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _gluey(m, u)
}
fn r_heavy(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _heavy(m, u)
}
fn r_lumpy(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _lumpy(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.dishes:a_plan_that_might_be_ruined:1681", g_a_plan_that_might_be_ruined),
    ("bot.knowledge.dishes:not_ruled_out.<locals>.guard:856", g_not_ruled_out),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.dishes:<lambda>:4874", r_lambda_4874),
    ("bot.knowledge.dishes:_bake_fault_more:4654", r_bake_fault_more),
    ("bot.knowledge.dishes:_diet:4274", r_diet),
    ("bot.knowledge.dishes:_dislike:3445", r_dislike),
    ("bot.knowledge.dishes:_dough_sticks:4627", r_dough_sticks),
    ("bot.knowledge.dishes:_follow_up:2266", r_follow_up),
    ("bot.knowledge.dishes:_gluey:1194", r_gluey),
    ("bot.knowledge.dishes:_grainy:1338", r_grainy),
    ("bot.knowledge.dishes:_heavy:1116", r_heavy),
    ("bot.knowledge.dishes:_idea:4431", r_idea),
    ("bot.knowledge.dishes:_lumpy:1380", r_lumpy),
    ("bot.knowledge.dishes:_problem:1813", r_problem),
    ("bot.knowledge.dishes:_pudding:3867", r_pudding),
    ("bot.knowledge.dishes:_sides:2798", r_sides),
    ("bot.knowledge.dishes:_start:2518", r_start),
    ("bot.knowledge.dishes:_the_heat:2397", r_the_heat),
    ("bot.knowledge.dishes:_two_dishes_plan:5439", r_two_dishes_plan),
    ("bot.knowledge.dishes:_veggie:4234", r_veggie),
    ("bot.knowledge.dishes:_what_is_a_dish:6258", r_what_is_a_dish),
];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/dishes.py:848 `not_ruled_out()` (never called on the trace corpus)
    pub fn not_ruled_out(keys: &[Value]) -> Value {
        crate::unported!("bot.knowledge.dishes:not_ruled_out")
    }

}

