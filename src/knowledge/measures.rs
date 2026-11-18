//! Port of `bot/knowledge/measures.py`: amounts (caffeine in a drink, coffee for a crowd, how
//! many come in a pack, how much of an ingredient goes in a dish).

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.measures",
    value { CAFFEINE_MG, COUNT_IN_PACK, AMOUNT_IN_A_DISH },
    strs { _COFFEE_TIME, _COFFEE_CROWD, _COUNT_NOT_COUNTING, _COUNT_ASK, _RECIPE_ASK, _PORTION, _NOT_A_RECIPE,
           _COOKING_WORDS },
    set { _ONLY_MADE },
    string { _COFFEE_RATIO, _COFFEE_METHOD },
    re { _DRESS_IS_A_GARMENT });

static NOT_A_WORD_OR_PERCENT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9% ]+", 0));
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9 ]+", 0));

/// The clean line, lower-cased, with `rx`'s characters made spaces, padded with a space each end.
fn said_with(rx: &Re, understood: &Understood) -> String {
    format!(" {} ", rx.sub(" ", &understood.clean.to_lowercase()))
}

/// How much caffeine is in the drink the line names, and the daily limit.
pub fn _caffeine_per_drink(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said_with(&NOT_A_WORD_OR_PERCENT, understood);
    let mut lead = "";
    for row in CAFFEINE_MG.items() {
        let (words, phrase) = (row.items()[0].strs(), row.items()[1].str());
        if words.iter().any(|w| said.contains(&format!(" {w} "))) {
            lead = phrase;
        }
        if !lead.is_empty() {
            break;
        }
    }
    let rest = "The usual daily limit for an adult is 400mg, which is about four instant coffees, and 200mg in pregnancy.";
    if lead.is_empty() {
        return Some(format!(
            "It varies with the drink: a mug of instant coffee is 60 to 80mg, filter coffee 90 to 140mg, a single espresso about 65mg, black tea about 50mg, green tea 25 to 40mg and a 330ml cola about 35mg. {rest}"
        ));
    }
    Some(format!("{lead}. {rest}"))
}

/// How much coffee to make, in the order the line asked for it.
pub fn _coffee_strength(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said_with(&NOT_A_WORD_OR_PERCENT, understood);
    let says = |words: &[String]| words.iter().any(|w| said.contains(&format!(" {w} ")));
    if says(&_COFFEE_TIME) {
        return Some(format!(
            "In a cafetiere: pour water a minute off the boil, leave it three to four minutes, then press and pour it all off the grounds or it goes on brewing and turns bitter. That is {}.",
            *_COFFEE_RATIO
        ));
    }
    if says(&_COFFEE_CROWD) {
        return Some(format!(
            "240g of ground coffee makes 20 mugs, each mug holding about two level tablespoons, 12g, and 60g filling a litre jug. {}. Go stronger for espresso, weaker with milk.",
            text::capitalize(&_COFFEE_METHOD)
        ));
    }
    Some(format!(
        "Measure it by the mug: {}. {}. Stronger for espresso, weaker with milk.",
        *_COFFEE_RATIO,
        text::capitalize(&_COFFEE_METHOD)
    ))
}

/// How many come in the bunch or the pack, from COUNT_IN_PACK, or nothing ("").
pub fn _count_in_a_pack(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said_with(&NOT_A_WORD, understood);
    if _COUNT_NOT_COUNTING.iter().any(|w| said.contains(&format!(" {w} "))) {
        return Some(String::new());
    }
    if !_COUNT_ASK.iter().any(|p| said.contains(p.as_str())) {
        return Some(String::new());
    }
    for row in COUNT_IN_PACK.items() {
        let r = row.items();
        let (things, packs, count, note) = (r[0].strs(), r[1].strs(), r[2].str(), r[3].str());
        if !packs.iter().any(|w| said.contains(&format!(" {w} ")) || said.contains(&format!(" {w}"))) {
            continue;
        }
        if !things.iter().any(|w| said.contains(w.as_str())) {
            continue;
        }
        return Some(format!("{count}: {note}, so buy by weight and use the count as a guide."));
    }
    Some(String::new())
}

/// How much of an ingredient goes into the dish the line names, or nothing ("").
pub fn _amount_in_a_dish(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said_with(&NOT_A_WORD, understood);
    if !_RECIPE_ASK.iter().any(|p| said.contains(p.as_str())) {
        return Some(String::new());
    }
    for dish_row in AMOUNT_IN_A_DISH.items() {
        let (dish_words, amounts) = (dish_row.items()[0].strs(), &dish_row.items()[1]);
        let named: Vec<&String> = dish_words.iter().filter(|d| said.contains(&format!(" {d} "))).collect();
        if named.is_empty() {
            continue;
        }
        for (ingredients, row) in amounts.pairs() {
            if !ingredients.strs().iter().any(|w| said.contains(&format!(" {w} ")) || said.contains(&format!(" {w}s "))) {
                continue;
            }
            let r = row.items();
            let (amount, note) = (r[0].str(), r[1].str());
            let as_eaten = if r.len() > 2 { r[2].str() } else { "" };
            if _PORTION.iter().any(|p| said.contains(p.as_str())) {
                return Some(String::new());
            }
            if _NOT_A_RECIPE.iter().any(|p| said.contains(p.as_str()))
                && !named.iter().any(|d| _ONLY_MADE.contains(*d))
                && !_cooking_words(&said)
            {
                // the sugar already in the slice: answered from the as-eaten line where there is one
                if as_eaten.is_empty() {
                    return Some(String::new());
                }
                let stop = if as_eaten.ends_with(['.', '!', '?']) { "" } else { "." };
                return Some(format!("{as_eaten}{stop}"));
            }
            let mut chars = note.chars();
            let head: String = chars.next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
            return Some(format!("{amount}. {head}{}.", chars.as_str()));
        }
    }
    Some(String::new())
}

/// The line asks for something to cook with, not for what is already in the food.
pub fn _cooking_words(said: &str) -> bool {
    _COOKING_WORDS.iter().any(|w| said.contains(&format!(" {w} ")))
}

/// True unless the line's "dress" is the garment, for the rows that key "dressing".
pub fn _a_dress_is_not_dressing(understood: &Understood) -> bool {
    let mut said = understood.clean.to_lowercase();
    if said.is_empty() {
        said = understood.text.to_lowercase();
    }
    !_DRESS_IS_A_GARMENT.is_match(&said)
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.measures:_a_dress_is_not_dressing:387", |_: &FnRef, u: &Understood| _a_dress_is_not_dressing(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.measures:_caffeine_per_drink:50", |_: &FnRef, m: &mut Memory, u: &Understood| _caffeine_per_drink(m, u)),
    ("bot.knowledge.measures:_coffee_strength:90", |_: &FnRef, m: &mut Memory, u: &Understood| _coffee_strength(m, u)),
    ("bot.knowledge.measures:_count_in_a_pack:154", |_: &FnRef, m: &mut Memory, u: &Understood| _count_in_a_pack(m, u)),
    ("bot.knowledge.measures:_amount_in_a_dish:317", |_: &FnRef, m: &mut Memory, u: &Understood| _amount_in_a_dish(m, u)),
];
