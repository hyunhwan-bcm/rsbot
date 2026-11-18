//! Port of `bot/knowledge/still_ok.py`: is it still ok to eat (leftovers, mince, leaves, raw
//! vegetables, spoilage with no food named).

use crate::data::FnRef;
use crate::knowledge::d2_shared::the_line;
use crate::knowledge::d2dup_g10::{globals, percent_named};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

globals!("bot.knowledge.still_ok",
    strs { _RAW_VEGETABLE_NAMES, EXTRA_FOODS, MINCE_WORDS, MINCE_VERDICT_WORDS, MINCE_KEEPING_WORDS, _COOKING_ASK,
           _KEEPING_IN_THE_LINE, _PAST_KEEPING, _VERDICT_ASKS },
    set { _RAW_VEG_PLURAL_S },
    value { _RAW_VEG_ONE, _COUNT_WORDS, _DAYS_SAID },
    re { _A_FAULT_IN_THE_DISH, _DAYS_AGO, _ENDS_ON_AN_ASK, _MONEY_IN_THE_LINE, _LEAFY_SHOWS },
    string { _LEFTOVERS_RULE });
globals!("bot.knowledge.meals", strs { IN_A_DISH }, value { INGREDIENTS });

static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9']+", 0));

/// Every food name the pantry tables know: `meals.IN_A_DISH | set(meals.INGREDIENTS) | EXTRA_FOODS`.
static KNOWN_FOODS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut known: HashSet<String> = IN_A_DISH.iter().cloned().collect();
    known.extend(INGREDIENTS.pairs().iter().map(|(k, _)| k.str().to_string()));
    known.extend(EXTRA_FOODS.iter().cloned());
    known
});

/// The verdict for the vegetable the line names, in the group it belongs to.
pub fn _raw_vegetable(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let text = format!(" {} ", reading.clean.replace("'s", " "));
    for name in _RAW_VEGETABLE_NAMES.iter() {
        if !text.contains(&format!(" {name} ")) {
            continue;
        }
        let template = _RAW_VEG_ONE.get(name).expect("KeyError: _RAW_VEG_ONE").items()[1].str();
        let plural = name.ends_with('s') && !_RAW_VEG_PLURAL_S.contains(name);
        let cap = text::upper_first(name);
        return Some(percent_named(
            template,
            &[
                ("one", name),
                ("Cap", &cap),
                ("them", if plural { "them" } else { "it" }),
                ("they", if plural { "they" } else { "it" }),
                ("are", if plural { "are" } else { "is" }),
            ],
        ));
    }
    None
}

/// True when the line names no food, so the unnamed spoilage row is the one to answer.
pub fn _names_no_food(understood: &Understood) -> bool {
    let text = format!(" {} ", understood.clean.to_lowercase());
    if understood.words.iter().any(|w| KNOWN_FOODS.contains(&w.to_lowercase())) {
        return false;
    }
    // A pantry name of two words ("soured cream", "coconut milk") is never one word of the line.
    !KNOWN_FOODS.iter().any(|name| text.contains(&format!(" {name} ")))
}

/// False when the line is a fault in the food, which the fix cards answer.
pub fn _not_a_fault_in_the_dish(understood: &Understood) -> bool {
    !_A_FAULT_IN_THE_DISH.is_match(&understood.clean.to_lowercase())
}

/// False when the line is a keeping question about mince, which has a row of its own.
pub fn _mince_keeping_is_its_own_row(understood: &Understood) -> bool {
    let line = if !understood.text.is_empty() {
        understood.text.clone()
    } else if !understood.clean.is_empty() {
        understood.clean.clone()
    } else {
        understood.words.join(" ")
    };
    let said = format!(" {} ", text::strip(&NOT_A_WORD.sub(" ", &line.to_lowercase())));
    let has = |words: &[String]| words.iter().any(|w| said.contains(&format!(" {w} ")));
    if !has(&MINCE_WORDS) {
        return true;
    }
    if has(&MINCE_VERDICT_WORDS) {
        return true;
    }
    !has(&MINCE_KEEPING_WORDS)
}

/// False for "what can i make with the leftover curry", true for "is the curry still ok".
pub fn _not_a_recipe_now(understood: &Understood) -> bool {
    let text = text::squash(&understood.clean.to_lowercase());
    if _COOKING_ASK.iter().any(|w| text.contains(w.as_str())) {
        return false;
    }
    if _KEEPING_IN_THE_LINE.iter().any(|w| text.contains(w.as_str())) {
        return true;
    }
    _PAST_KEEPING.iter().any(|w| text.contains(w.as_str()))
}

/// The day count the line gives for the dish, or None when the line gives none.
pub fn _how_many_days_ago(line: &str) -> Option<i64> {
    if let Some(m) = _DAYS_AGO.search(line) {
        let said = m.groups().into_iter().flatten().find(|p| !p.is_empty()).expect("StopIteration");
        return Some(match _COUNT_WORDS.get(&said.to_lowercase()).and_then(|v| v.as_int()) {
            Some(n) => n,
            None => said.trim().parse().expect("ValueError: invalid literal for int()"),
        });
    }
    for row in _DAYS_SAID.items() {
        if line.contains(row.items()[0].str()) {
            return row.items()[1].as_int();
        }
    }
    None
}

/// The verdict for a dish kept this many days ("" when the line gives no count).
pub fn _the_verdict_on_the_days(days: Option<i64>) -> String {
    let Some(days) = days else { return String::new() };
    if days <= 0 {
        return "Yes - it is from today or last night, so it is fine.".into();
    }
    if days == 1 {
        return "Yes - a day in the fridge is nothing, so it is fine.".into();
    }
    if days <= 3 {
        return format!("Yes - {days} days is inside the 3 to 4 days a cooked dish keeps in the fridge, so it is fine.");
    }
    if days == 4 {
        return "At the edge, I'm afraid: 3 to 4 days is the limit, so today is the last day I would eat it and I would not leave it for tomorrow.".into();
    }
    format!("No, I would not: a cooked dish keeps 3 to 4 days in the fridge, and rice only 1, so {days} days is past it.")
}

/// The keeping rule, opened by the verdict on the days this line gives (only for a question).
pub fn _the_leftovers_answer(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let said = text::squash(&reading.clean.to_lowercase());
    let asked = _VERDICT_ASKS.iter().any(|w| said.contains(w.as_str()))
        || reading.text.contains('?')
        || _ENDS_ON_AN_ASK.is_match(&said);
    if !asked {
        return Some(_LEFTOVERS_RULE.clone());
    }
    let verdict = _the_verdict_on_the_days(_how_many_days_ago(&the_line(reading)));
    if verdict.is_empty() {
        return Some(_LEFTOVERS_RULE.clone());
    }
    if verdict.starts_with("No") {
        return Some(verdict + " Reheating does not make it safe: the toxins old food builds up do not all break down with heat and they do not all smell.");
    }
    Some(format!("{verdict} {}", *_LEFTOVERS_RULE))
}

/// Whether the sum in the line is something to keep rather than something to eat.
pub fn _not_about_money(understood: &Understood) -> bool {
    let said = [&understood.clean, &understood.norm_text, &understood.text, &understood.plain]
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    !_MONEY_IN_THE_LINE.is_match(&said)
}

/// The line names a colour, a slime or a smell on the leaf, not just that it went limp.
pub fn _the_leaf_shows_a_colour(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    _LEAFY_SHOWS.is_match(line)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.still_ok:_names_no_food:250", |_: &FnRef, u: &Understood| _names_no_food(u)),
    ("bot.knowledge.still_ok:_mince_keeping_is_its_own_row:416", |_: &FnRef, u: &Understood| {
        _mince_keeping_is_its_own_row(u)
    }),
    ("bot.knowledge.still_ok:_not_about_money:587", |_: &FnRef, u: &Understood| _not_about_money(u)),
    ("bot.knowledge.still_ok:<lambda>:1049", |_: &FnRef, u: &Understood| _not_a_recipe_now(u) && _not_a_fault_in_the_dish(u)),
    ("bot.knowledge.still_ok:_the_leaf_shows_a_colour:1180", |_: &FnRef, u: &Understood| _the_leaf_shows_a_colour(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.still_ok:_raw_vegetable:112", |_: &FnRef, m: &mut Memory, u: &Understood| _raw_vegetable(m, u)),
    ("bot.knowledge.still_ok:_the_leftovers_answer:540", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _the_leftovers_answer(m, u)
    }),
];
