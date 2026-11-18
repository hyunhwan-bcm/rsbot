//! Port of `bot/knowledge/swaps.py`: what to use when an ingredient has run out, and what to have
//! in place of a food somebody will not eat. Every row's reply is fenced at import time
//! (`_fence_every_row`): the fence's closure, `_fenced.<locals>.answer`, is ported here and reads
//! the row it captured.

use crate::data::{FnRef, Value};
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashMap;
use std::sync::LazyLock;

globals!("bot.knowledge.swaps",
    strs { ASKS_FOR_A_SWAP, TASTE_SAID, _SWAP_STOP, _MAKE_ASK, _SWAP_SAID },
    value { REFUSED_FOR, ENTRIES });

static SHORT_OF_POWDER: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:no|without|out of|none left of)\s+(?:\w+\s+)?chilli (?:powder|flakes)\b", 0));

/// The phrases a line uses when the *other* ingredient is the one being replaced.
/// (Used at import time to build the rows' avoid lists.)
pub fn _other_way_round(others: &[&str]) -> Vec<String> {
    others.iter().flat_map(|w| [format!("instead of {w}"), format!("in place of {w}")]).collect()
}

/// Which form of chilli the line is short of, and what stands in for that one.
pub fn _the_chilli_swap(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let text = format!(" {} ", reading.clean.to_lowercase());
    let after = _swapped_out(&text).join(" ");
    if after.contains("powder") || after.contains("flakes") || SHORT_OF_POWDER.is_match(&text) {
        return Some("Paprika with a pinch of cayenne is the closest thing to chilli powder: about four parts paprika to one of cayenne, and a little more cayenne if you want it hot. For flakes, crushed dried chillies or a pinch of cayenne do the job.".into());
    }
    Some("A quarter teaspoon of chilli powder or a pinch of flakes stands in for one fresh chilli, and hot sauce works in a sauce rather than where the chilli is sliced over the top.".into())
}

/// The line names, after "instead of", a food REFUSED_FOR holds: the shortage question.
fn names_a_refused_food_after_instead(text: &str) -> bool {
    for phrase in ["instead of", "in place of", "substitute for"] {
        if let Some(at) = text.find(phrase) {
            let tail = &text[at..];
            if REFUSED_FOR.pairs().iter().any(|(name, _)| tail.contains(name.str())) {
                return true;
            }
        }
    }
    false
}

/// Whether the line asks what to have in the place of the food it names.
pub fn _asks_for_a_swap(understood: &Understood) -> bool {
    let text = format!(" {} ", understood.clean.to_lowercase());
    if !ASKS_FOR_A_SWAP.iter().any(|p| text.contains(p.as_str())) {
        return false;
    }
    !names_a_refused_food_after_instead(&text)
}

/// REFUSED_FOR's keys, longest first (Python's stable `sorted(key=len, reverse=True)`).
static REFUSED_LONGEST_FIRST: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    let mut rows: Vec<(String, String)> =
        REFUSED_FOR.pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect();
    rows.sort_by(|a, b| b.0.chars().count().cmp(&a.0.chars().count()));
    rows
});

/// What to put in the place of the food the line says it will not eat.
pub fn _refused_food(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    let text = format!(" {} ", understood.clean.to_lowercase());
    if !ASKS_FOR_A_SWAP.iter().any(|p| text.contains(p.as_str())) {
        return None;
    }
    if names_a_refused_food_after_instead(&text) {
        return None;
    }
    for (name, reply) in REFUSED_LONGEST_FIRST.iter() {
        if said.contains(&format!(" {name} ")) || text::strip(&said) == name {
            return Some(reply.clone());
        }
    }
    None
}

/// The ingredient the line names after "instead of", as the words it used.
pub fn _swapped_out(text: &str) -> Vec<String> {
    let lowered = text.to_lowercase().replace([',', '.'], " ");
    let words = text::split_ref(&lowered);
    let mut said = Vec::new();
    for (position, word) in words.iter().enumerate() {
        if *word != "instead" {
            continue;
        }
        if position + 1 >= words.len() || words[position + 1] != "of" {
            continue;
        }
        for following in &words[position + 2..] {
            if _SWAP_STOP.iter().any(|s| s == following) {
                break;
            }
            said.push(following.to_string());
        }
    }
    said
}

/// The alternative names of one key part, as written ("olive oil" gives one name).
pub fn _names(part: &Value) -> Vec<String> {
    match part {
        Value::Tuple(_) => part.items().iter().map(|v| v.str().to_string()).collect(),
        other => vec![other.str().to_string()],
    }
}

/// True for "olive" and "olives", so a plural never reads as a different ingredient.
pub fn _same_word(one: &str, other: &str) -> bool {
    one.to_lowercase().trim_end_matches('s') == other.to_lowercase().trim_end_matches('s')
}

/// True when every word of `name` is a word of the row's own name. (Import-time helper.)
pub fn _inside(name: &str, mine: &[String]) -> bool {
    text::split_ref(name).iter().all(|w| mine.iter().any(|own| _same_word(w, own)))
}

/// A fenced row's identity in the bundle: its captured `own` words and `said` reply.
fn fence_key(f_own: &Value, said: &Value) -> String {
    let said = match said {
        Value::Fn(g) => format!("fn:{}", g.id),
        other => format!("str:{}", other.str()),
    };
    format!("{}\u{0}{said}", f_own.strs().join("\u{1}"))
}

/// The first key part's names of every fenced row, found by the values its fence captured
/// (the captured `row` itself is not in the bundle: it is a cycle).
static FENCED_ROWS: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let mut rows = HashMap::new();
    for e in ENTRIES.items() {
        let (Some(reply), Some(key)) = (e.attr("reply").and_then(|r| r.as_fn()), e.attr("key")) else { continue };
        if let (Some(own), Some(said)) = (reply.cell("own"), reply.cell("said")) {
            rows.insert(fence_key(own, said), _names(&key.items()[0]));
        }
    }
    rows
});

/// `_fenced.<locals>.answer`: the row's reply, with two general stand-downs in front of it.
fn fenced_answer(f: &FnRef, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let own_value = f.cell("own").expect("fence cell own");
    let own = own_value.strs();
    let said = f.cell("said").expect("fence cell said");
    let taste_row = f.cell("taste_row").is_some_and(|v| v.truthy());
    let names = FENCED_ROWS.get(&fence_key(own_value, said)).cloned().unwrap_or_default();
    let text = format!(" {} ", understood.clean.to_lowercase());
    let swapped = _swapped_out(&text);
    if !swapped.is_empty() && !swapped.iter().any(|w| own.iter().any(|mine| _same_word(w, mine))) {
        return Some(String::new());
    }
    if !taste_row
        && TASTE_SAID.iter().any(|p| text.contains(p.as_str()))
        && ASKS_FOR_A_SWAP.iter().any(|p| text.contains(p.as_str()))
        && names.iter().filter(|n| !n.contains(' ')).any(|n| text.contains(&format!(" {} ", n.to_lowercase())))
    {
        return Some(String::new());
    }
    let asking_for_a_dish = _MAKE_ASK.iter().any(|p| text.contains(p.as_str()));
    if asking_for_a_dish && !_SWAP_SAID.iter().any(|p| text.contains(p.as_str())) {
        return Some(String::new());
    }
    match said {
        Value::Fn(g) => match g.name() {
            "bot.knowledge.swaps:_refused_food" => _refused_food(memory, understood),
            "bot.knowledge.swaps:_the_chilli_swap" => _the_chilli_swap(memory, understood),
            _ => Some(crate::knowledge::registry::reply(g, memory, understood)),
        },
        other => other.as_str().map(str::to_string),
    }
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.swaps:_asks_for_a_swap:204", |_: &FnRef, u: &Understood| _asks_for_a_swap(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.swaps:_fenced.<locals>.answer:1133", fenced_answer),
    ("bot.knowledge.swaps:_the_chilli_swap:92", |_: &FnRef, m: &mut Memory, u: &Understood| _the_chilli_swap(m, u)),
    ("bot.knowledge.swaps:_refused_food:225", |_: &FnRef, m: &mut Memory, u: &Understood| _refused_food(m, u)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fenced_row_is_found() {
        assert_eq!(FENCED_ROWS.len(), ENTRIES.items().len());
        let butter = ENTRIES.items().iter().find(|e| e.attr("id").map(|v| v.str()) == Some("swap-butter-plain")).unwrap();
        let f = butter.attr("reply").unwrap().as_fn().unwrap();
        let u = Understood { clean: "what can i use instead of butter".into(), ..Default::default() };
        let got = fenced_answer(f, &mut Memory::new(), &u).unwrap();
        assert!(got.starts_with("Margarine"), "{got}");
        let u = Understood { clean: "what can i use instead of plain flour".into(), ..Default::default() };
        assert_eq!(fenced_answer(f, &mut Memory::new(), &u).unwrap(), "");
        let taste = ENTRIES.items().iter().find(|e| e.attr("id").map(|v| v.str()) == Some("swap-what-i-will-not-eat")).unwrap();
        let f = taste.attr("reply").unwrap().as_fn().unwrap();
        let u = Understood {
            clean: "i hate mushrooms what else can i have".into(),
            words: "i hate mushrooms what else can i have".split(' ').map(String::from).collect(),
            ..Default::default()
        };
        let got = fenced_answer(f, &mut Memory::new(), &u);
        println!("{got:?}");
    }
}
