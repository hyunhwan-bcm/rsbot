//! Port of `bot/memory/rename.py`: a pet whose name has moved in the sentence: "we renamed him
//! max", "hes max now".
//!
//! DATA NOTE: this module's globals are not in the exported bundle (the export ran before the
//! module was imported), so its patterns and word lists are written here, copied character for
//! character from the Python source.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::facts;
use crate::py::re::Re;
use crate::py::text as pt;
use std::collections::HashSet;
use std::sync::LazyLock;

const I: u32 = crate::py::re::I;

pub static RENAMING: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\brenam(?:e|ed|es|ing)\b",
            r"|\bchang(?:e|ed|es|ing)\b[^.?!]{0,24}?\bnames?\b",
            r"|\bchang(?:e|ed|es|ing)\s+(?:it|him|her|them)\s+(?:to|into)\b",
            r"|\bcall\s+(?:him|her|it|them)\b[^.?!]*\bnow\b",
            r"|\banswers?\s+to\b|\bgoes?\s+by\b",
            r"|\bnow\s+(?:called|named)\b",
            r"|\b(?:he|she|it|they)(?:'s|s| is| are)\s+[a-z'-]{2,}\s+now\b",
            r"|\bnow\s+(?:he|she|it|they)(?:'s|s| is)\b",
        ),
        I,
    )
});

fn words(text: &str) -> HashSet<String> {
    pt::split(text).into_iter().collect()
}

/// The words a rename can trail after the new name ("max now", "max instead").
pub static AFTER_A_NEW_NAME: LazyLock<HashSet<String>> = LazyLock::new(|| {
    words("now instead please ok okay tbh tho though then actually too also\nfrom onwards today anyway honestly like")
});
pub static NOT_A_NEW_NAME: LazyLock<HashSet<String>> = LazyLock::new(|| {
    words("it him her them the to as called named name rename renamed\nthey we i you us me dog cat puppy kitten pet animal")
});
/// A name stands bare: a determiner in front of the word makes it a thing.
pub static A_THING_IN_FRONT: LazyLock<HashSet<String>> = LazyLock::new(|| {
    words("the a an this that these those my our his their your its\nsome any another one")
});
pub static A_PERSON_IN_THE_LINE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\b(?:son|daughter|mum|mom|mother|father|dad|wife|husband|partner|boyfriend|girlfriend|",
            r"brother|sister|friend|boss|colleague|neighbour|neighbor|child|kid|baby|gran|grandma|",
            r"grandad|grandpa|nan|nana|aunt|uncle|cousin|mate|flatmate|housemate|carer|teacher|",
            r"people|someone|somebody)\b",
        ),
        I,
    )
});
pub static THE_PERSONS_OWN_NAME: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:my|our)\s+names?\b", I));

/// Python: bot/memory/rename.py:64 `moves(low)`: true when the line says in as many words that a
/// name has changed.
pub fn moves(low: &str) -> bool {
    RENAMING.is_match(&pt::squash(low))
}

/// Python: bot/memory/rename.py:69 `the_name_it_gave(low)`: the name at the end of a line that says
/// the name has moved, or "" (a str).
pub fn the_name_it_gave(low: &str) -> Value {
    Value::from(the_name_it_gave_str(low))
}

/// `the_name_it_gave(low)` as a string.
pub fn the_name_it_gave_str(low: &str) -> String {
    let mut words: Vec<String> = pt::split(low).iter().map(|w| pt::strip_chars(w, ".,!?;:'\"").to_string()).collect();
    while words.last().is_some_and(|w| AFTER_A_NEW_NAME.contains(&pt::lower(w))) {
        words.pop();
    }
    let Some(word) = words.last().cloned() else {
        return String::new();
    };
    if pt::len(&word) < 2 || NOT_A_NEW_NAME.contains(&pt::lower(&word)) {
        return String::new();
    }
    if words.len() > 1 && A_THING_IN_FRONT.contains(&pt::lower(&words[words.len() - 2])) {
        return String::new();
    }
    if facts::a_name_could_be(&word) {
        word
    } else {
        String::new()
    }
}

/// Python: bot/memory/rename.py:91 `the_name_a_line_moved_on(low, pets, animal_named)`: the new name
/// for the animal a rename line is about, or "" (a str).
pub fn the_name_a_line_moved_on(low: &str, pets: &Value, animal_named: &Value) -> Value {
    let low = pt::squash(low);
    if low.is_empty() || A_PERSON_IN_THE_LINE.is_match(&low) || THE_PERSONS_OWN_NAME.is_match(&low) {
        return Value::from("");
    }
    let pets: Vec<&Value> = pets.items().iter().filter(|p| p.is_dict()).collect();
    if pets.is_empty() {
        return Value::from("");
    }
    if !animal_named.truthy() && pets.iter().filter(|p| p.get("name").is_some_and(Value::truthy)).count() != 1 {
        return Value::from("");
    }
    the_name_it_gave(&low)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rename_is_read() {
        assert!(moves("we renamed him max"));
        assert!(moves("hes max now"));
        assert!(!moves("my dog is called rex"));
        assert_eq!(the_name_it_gave_str("we renamed him to the vet"), "");
        assert_eq!(the_name_it_gave_str("we renamed the pub"), "");
        // Expected values from PyBot (`bot.memory.rename`).
        assert_eq!(the_name_it_gave_str("we renamed him max now"), "max");
        assert_eq!(the_name_it_gave_str("i changed it to friday"), "");
        let pets = |v: serde_json::Value| Value::from_plain(&v);
        let rex = pets(serde_json::json!([{"animal": "dog", "name": "rex"}]));
        let two = pets(serde_json::json!([{"animal": "dog", "name": "rex"}, {"animal": "cat", "name": "tom"}]));
        let unnamed = pets(serde_json::json!([{"animal": "dog"}]));
        let f = Value::Bool(false);
        let t = Value::Bool(true);
        assert_eq!(the_name_a_line_moved_on("we renamed him max", &rex, &f).str(), "max");
        assert_eq!(the_name_a_line_moved_on("we renamed him max", &two, &f).str(), "");
        assert_eq!(the_name_a_line_moved_on("my son changed his name to sam", &rex, &t).str(), "");
        assert_eq!(the_name_a_line_moved_on("we renamed the dog milo", &unnamed, &t).str(), "milo");
    }
}
