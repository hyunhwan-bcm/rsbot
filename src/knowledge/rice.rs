//! Port of `bot/knowledge/rice.py`: rice on its own - which sort for a dish, and what stands in
//! for what. One table of sorts (`SORTS`), each with its water and its time.

use crate::data::{self, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

const P: &str = "bot.knowledge.rice.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}

/// `NAMES`: {sort: the words for it}, in the dict's order.
static NAMES: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| g("NAMES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());
/// `DISHES`: (the words for a dish, the sort it wants).
static DISHES: LazyLock<Vec<(Vec<String>, String)>> =
    LazyLock::new(|| g("DISHES").items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect());

/// `SORTS[sort][field]`; a missing sort or field is Python's KeyError.
fn sort_field(sort: &str, field: &str) -> String {
    let row = g("SORTS").get(sort).unwrap_or_else(|| panic!("KeyError: {sort:?}"));
    row.get(field).unwrap_or_else(|| panic!("KeyError: {field:?}")).str().to_string()
}

/// The line as one lowercase string, padded with spaces, for the dish names, which are phrases.
pub fn _said(understood: &Understood) -> String {
    let mut said = understood.clean.clone();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    format!(" {} ", text::lower(&said))
}

/// Every sort the line names, in the order the line names them (the first named is the one in
/// hand). `names` None is Python's default (`NAMES`).
pub fn _sort_named(said: &str, names: Option<&[(String, Vec<String>)]>) -> Vec<String> {
    let names = names.unwrap_or(&NAMES);
    let mut found: Vec<(usize, &String)> = Vec::new();
    for (key, words) in names {
        let mut at = said.len();
        for word in words {
            for form in [format!(" {word} "), format!(" {word}s ")] {
                if let Some(where_) = said.find(&form) {
                    if where_ < at {
                        at = where_;
                        break;
                    }
                }
            }
        }
        if at < said.len() {
            found.push((at, key));
        }
    }
    found.sort();
    found.into_iter().map(|(_, key)| key.clone()).collect()
}

/// The dish the last few turns were about, for a follow-up that names no dish of its own.
pub fn _dish_recent(memory: &Memory) -> (Option<String>, Option<String>) {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(4)..].iter().rev() {
        let said = format!(" {} ", text::lower(&turn.user));
        let (dish, word) = _dish_named(&said);
        if dish.is_some() {
            return (dish, word);
        }
    }
    (None, None)
}

/// The sort a dish named in the line wants, and the word that named it.
pub fn _dish_named(said: &str) -> (Option<String>, Option<String>) {
    for (words, sort) in DISHES.iter() {
        for word in words {
            if said.contains(&format!(" {word} ")) || said.contains(&format!(" {word}s ")) {
                return (Some(sort.clone()), Some(word.clone()));
            }
        }
    }
    (None, None)
}

/// "For paella, bomba rice is the one: ..." and the sort's stand-in, if it has one.
fn for_the_dish(dish_word: &str, wanted: &str) -> String {
    let mut said = format!(
        "For {dish_word}, {} is the one: {}, {}. It is {}.",
        sort_field(wanted, "label"),
        sort_field(wanted, "water"),
        sort_field(wanted, "time"),
        sort_field(wanted, "what")
    );
    let stands_in = sort_field(wanted, "stands_in");
    if !stands_in.is_empty() {
        said += &format!(" {stands_in}");
    }
    said
}

/// One sort standing in for another, or one sort named for a dish: the numbers that change.
pub fn _swap(memory: &Memory, understood: &Understood) -> String {
    let said = _said(understood);
    let named = _sort_named(&said, None);
    let (mut dish, mut dish_word) = _dish_named(&said);
    if dish.is_none() && (!named.is_empty() || _dish_recent(memory).0.is_some()) {
        (dish, dish_word) = _dish_recent(memory);
    }
    let field = |sort: &str, name: &str| sort_field(sort, name);
    if named.len() > 1 && named[0] != named[1] {
        let (held, wanted) = (&named[0], &named[1]);
        if (field(held, "water"), field(held, "time")) == (field(wanted, "water"), field(wanted, "time")) {
            return format!(
                "{} will stand in for {} on the same water and time - {}, {} - and the difference is the grain: it is {}.",
                text::capitalize(&field(held, "label")),
                field(wanted, "label"),
                field(held, "water"),
                field(held, "time"),
                field(held, "what")
            );
        }
        return format!(
            "{} will stand in for {}: it wants {} and {}, where {} wants {} and {}. It is {}, so the dish comes out with that grain instead.",
            text::capitalize(&field(held, "label")),
            field(wanted, "label"),
            field(held, "water"),
            field(held, "time"),
            field(wanted, "label"),
            field(wanted, "water"),
            field(wanted, "time"),
            field(held, "what")
        );
    }
    if let Some(dish) = dish.as_deref() {
        let dish_word = dish_word.as_deref().unwrap_or("None");
        if named.is_empty() || named[0] == dish {
            return for_the_dish(dish_word, dish);
        }
        let held = &named[0];
        return format!(
            "You can use {} in {dish_word}, but {} is the one for it: {}, {}. {} is {}, where {} is {}, so the dish comes out with that grain in it.",
            field(held, "label"),
            field(dish, "label"),
            field(dish, "water"),
            field(dish, "time"),
            text::capitalize(&field(dish, "label")),
            field(dish, "what"),
            field(held, "label"),
            field(held, "what")
        );
    }
    if named.is_empty() {
        return format!(
            "Plain long grain rice is the safe one for most things: {}, {}. Basmati and jasmine are the same water and time with a different grain, and brown rice wants {} for {}.",
            field("long grain", "water"),
            field("long grain", "time"),
            field("brown", "water"),
            field("brown", "time")
        );
    }
    let held = &named[0];
    format!(
        "{} works in place of white rice: {}, {}. It is {}.",
        text::capitalize(&field(held, "label")),
        field(held, "water"),
        field(held, "time"),
        field(held, "what")
    )
}

/// Which sort a named dish wants, and the numbers for it.
pub fn _dish(memory: &Memory, understood: &Understood) -> String {
    _swap(memory, understood)
}

/// The `rice-swap` row's reply.
pub fn _swap_reply(memory: &Memory, understood: &Understood) -> String {
    _swap(memory, understood)
}

/// The `rice-for-dish` row's reply.
pub fn _dish_reply(memory: &Memory, understood: &Understood) -> String {
    _swap(memory, understood)
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.rice:_swap_reply:315", |_, m, u| Some(_swap_reply(m, u))),
    ("bot.knowledge.rice:_dish_reply:319", |_, m, u| Some(_dish_reply(m, u))),
];
