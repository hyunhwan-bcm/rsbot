//! Port of PyBot's `bot/knowledge/dishes.py`: dish plans (what a cooking conversation needs after
//! the first answer: heat, time, doneness, keeping, sides, the fault that keeps happening), the
//! idea, dislike, diet and pudding rows, and what a named dish is.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g3 as dup;
use crate::knowledge::d1_dup_g3::tokens;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as re, Re};
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{BTreeSet, HashMap, HashSet};
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

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.dishes:not_ruled_out.<locals>.guard:856", g_not_ruled_out),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
