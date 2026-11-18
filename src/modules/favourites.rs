//! Port of `bot/modules/favourites.py`: a favourite the person names, corrects, changes or asks for, kept and said back (`bot/memory/favourite.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::favourite;

/// Python: bot/modules/favourites.py `observe(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Keep the favourite a line names, corrects or changes (bot/memory/favourite.py).
pub fn observe(text: &str, memory: &mut Memory) {
    // an observer must never lose a turn
    let _ = py_try(|| {
        favourite::watch(text, memory);
    });
}

/// Python: bot/modules/favourites.py `TAKES_AS_IT_COMES(text, memory=None)`
///
/// Whether this line is one this module answers as it comes (bot/modules/arbiter.py). Asked on a
/// copy of memory, because answering pops the flags the reply was marked with - the live memory
/// is the store's to write, not this hook's.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    // a hook that raises claims nothing
    or_on_raise(|| _said_back(text, &mut memory.clone()).is_some(), false)
}

/// Python: bot/modules/favourites.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Say the favourite back, or leave the turn to whoever answers the line better.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    // a module must never lose a turn
    or_on_raise(|| _said_back(text, memory), None)
}

/// `favourite.their_words(value)` as the text it is.
fn their_words(value: &Value) -> String {
    ret_str(favourite::their_words(&s_or(Some(value))))
}

/// `read_tastes(memory)`: PyBot hands the taste reader the memory itself, and the reader reads
/// `str()` of whatever it is given - so this is "a taste is said somewhere in the memory, as
/// Python prints it". (The print is rebuilt from the memory's plain form; its key order is the
/// only thing that can differ from Python's, and only the truth of the result is read.)
fn tastes_in_the_memory(memory: &Memory) -> bool {
    ret_true(crate::memory::taste::read_all(&py_repr(&memory_value(memory))))
}

/// Python: bot/modules/favourites.py `_said_back(text, memory)`
///
/// The favourite a line names or asks for, said back (bot/memory/favourite.py). A thing they
/// have never said a favourite of is left to the other modules rather than denied.
pub fn _said_back(text: &str, memory: &mut Memory) -> Option<String> {
    let told = memory.remove("favourite_told").unwrap_or(Value::None);
    let changed = memory.remove("favourite_changed").unwrap_or(Value::None);
    if ret_true(favourite::asks_something_too(text)) {
        // "my favourite genre is comedy, what should i watch" is a fact and a request in one
        // line. The fact is kept above, but the line is answered by whatever answers the
        // question - the store saying the fact back ("Comedy for your favourite genre, then -
        // noted.") leaves the person with no suggestion and reads as not listening.
        return None;
    }
    if changed.is_dict() && has(&changed, "thing") {
        let value = their_words(vget(&changed, "value"));
        let shown = cap1(&value);
        let thing = py_str(vget(&changed, "thing"));
        if has(&changed, "was") {
            return Some(pick(
                &[
                    format!("{shown} now, then \u{2014} I have your favourite {thing} as {value}."),
                    format!("Changed: your favourite {thing} is {value} now, not {}.", their_words(vget(&changed, "was"))),
                ],
                memory,
                text,
            ));
        }
        return Some(format!("{shown} for your favourite {thing}, then \u{2014} noted."));
    }
    if told.is_dict() && has(&told, "thing") {
        // The answer to the bot's own ask a turn ago ("what is it?" -> "green"): said back as the
        // fact it now is, rather than the listening line a short answer otherwise gets.
        return Some(format!("Noted: your favourite {} is {}.", py_str(vget(&told, "thing")), their_words(vget(&told, "value"))));
    }
    if ret_true(favourite::asks_all(text)) {
        let held = favourite::the_favourites(memory).val();
        if held.truthy() {
            let held = held.items();
            let last = &held[held.len().saturating_sub(4)..];
            let parts: Vec<String> = last
                .iter()
                .map(|pair| match pair.items() {
                    [thing, value] => format!("your favourite {} is {}", py_str(thing), py_str(value)),
                    _ => panic!("ValueError: not enough values to unpack (expected 2)"),
                })
                .collect();
            let said_all = if parts.len() == 1 {
                format!("You have told me one: {}.", parts[0])
            } else {
                format!("You have told me {} and {}.", parts[..parts.len() - 1].join(", "), parts[parts.len() - 1])
            };
            let other = said_all.replace("You have told me ", "Here is what I have: ");
            return Some(pick(&[said_all, other], memory, text));
        }
        return None;
    }
    let asked = or_none(favourite::asked(text));
    if let Some(asked) = asked.as_deref() {
        if !ret_true(favourite::kept(memory, asked)) && tastes_in_the_memory(memory) {
            // A taste the store already holds is the nearest thing it has to a favourite, and the
            // taste list is answered by the line about it ("i love cats" then "what are my favourite
            // foods?" -> "You like cats."). The favourite slot has nothing, and "Your favourite foods
            // hasn't come up yet. What is it?" throws away the one thing the store did keep.
            return None;
        }
        let value = favourite::kept(memory, asked).val();
        if !value.truthy() {
            // Nothing on record: the honest answer is that they have not said, and the thing they
            // named is what the bot then asks for. Kept as the slot it is, so the one-word answer
            // ("green") lands on the same favourite rather than on nothing at all.
            memory.set("favourite_asked", asked);
            return Some(pick(
                &[
                    format!("You haven't told me your favourite {asked} \u{2014} what is it?"),
                    format!("I don't have a favourite {asked} for you yet. What is it?"),
                    format!("Your favourite {asked} hasn't come up yet. What is it?"),
                ],
                memory,
                text,
            ));
        }
        let value = their_words(&value);
        let shown = cap1(&value);
        return Some(pick(
            &[
                format!("Your favourite {asked} is {value}."),
                format!("You said {value} \u{2014} that is your favourite {asked}."),
                format!("{shown}, you said. That is your favourite {asked}."),
            ],
            memory,
            text,
        ));
    }
    let said = favourite::read(text).val();
    if !said.truthy() {
        return None;
    }
    // A favourite of a thing the knowledge base has an entry for is a line the answers module can
    // answer for itself: "my favourite colour is green" has the colours that go with green in it,
    // and the store's receipt ("Green for your favourite colour, then - noted.") stood in front of
    // that answer. The fact is kept in `_observe` either way; only the claim on the turn is given
    // up, so the turn the person gets is the one about the thing they named.
    if let Some(understood) = memory.understood.as_ref() {
        if let Ok(Some(_)) = py_try(|| crate::knowledge::lookup::find(understood, None, None, false)) {
            return None;
        }
    }
    let item = |key: &str| said.get(key).cloned().unwrap_or_else(|| panic!("KeyError: {key:?}"));
    let value = their_words(&item("value"));
    // `"..." + said["thing"] + "..."`: the thing is a str, or the concatenation raises
    let thing = item("thing").as_str().map(str::to_string).unwrap_or_else(|| panic!("TypeError: can only concatenate str"));
    let mut options = vec![format!("Noted: your favourite {thing} is {value}."), format!("Favourite {thing}: {value}. Got it.")];
    let lowered = value.to_lowercase();
    if text::split(&value).len() <= 2 && !(lowered.starts_with("my ") || lowered.starts_with("the ")) {
        // "Green for your favourite colour, then - noted." reads as a sentence, and "my mums
        // curry for your favourite food, then - noted." does not: a thing they named with more
        // than a word or two in it keeps the plain form.
        options.push(format!("{} for your favourite {thing}, then \u{2014} noted.", cap1(&value)));
    }
    Some(pick(&options, memory, text))
}
