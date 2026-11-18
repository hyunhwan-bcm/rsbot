//! Port of `bot/modules/their_life.py`: a fact about the person's own life, said on its own
//! ("we live near the park", "we have a dog", "i'm 34", "my mum is 71"), gets its own turn - the
//! fact taken in and the one thing about it asked.
#![allow(non_snake_case)]

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.modules.their_life.";

fn re(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}
fn str_map(name: &str) -> Vec<(String, String)> {
    data::get(&format!("{M}{name}"))
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.str().to_string()))
        .collect()
}

static _PLACES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLACES"));
static _ONE_PET: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ONE_PET"));
static _PETS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PETS"));
static _NUMBER_WORDS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_map("_NUMBER_WORDS"));
static _NOT_A_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| re("_NOT_A_SHAPE"));
static _STORY: LazyLock<&'static Re> = LazyLock::new(|| re("_STORY"));
static _TONE: LazyLock<&'static Re> = LazyLock::new(|| re("_TONE"));
static _LIVE: LazyLock<&'static Re> = LazyLock::new(|| re("_LIVE"));
static _PLACE_KINDS: LazyLock<&'static Re> = LazyLock::new(|| re("_PLACE_KINDS"));
static _PET: LazyLock<&'static Re> = LazyLock::new(|| re("_PET"));
static _SIBLINGS: LazyLock<&'static Re> = LazyLock::new(|| re("_SIBLINGS"));
static _SIBLING_LINES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SIBLING_LINES"));
static _AGE: LazyLock<&'static Re> = LazyLock::new(|| re("_AGE"));
static _NOT_AN_AGE: LazyLock<&'static Re> = LazyLock::new(|| re("_NOT_AN_AGE"));
static _AGE_LINES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_AGE_LINES"));
static _SOMEONE: LazyLock<&'static Re> = LazyLock::new(|| re("_SOMEONE"));
static _SOMEONE_WORK: LazyLock<&'static Re> = LazyLock::new(|| re("_SOMEONE_WORK"));
static _SOMEONE_HOURS: LazyLock<&'static Re> = LazyLock::new(|| re("_SOMEONE_HOURS"));
static _SOMEONE_AGE: LazyLock<&'static Re> = LazyLock::new(|| re("_SOMEONE_AGE"));
static _SOMEONE_KIDS: LazyLock<&'static Re> = LazyLock::new(|| re("_SOMEONE_KIDS"));
static _AGE_READ_ELSEWHERE: LazyLock<std::collections::HashSet<String>> =
    LazyLock::new(|| data::set(&format!("{M}_AGE_READ_ELSEWHERE")));
static _HER: LazyLock<std::collections::HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}_HER")));
static _HIM: LazyLock<std::collections::HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}_HIM")));
static _SOMEONE_WORK_LINES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SOMEONE_WORK_LINES"));
static _SOMEONE_HOURS_LINES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SOMEONE_HOURS_LINES"));
static _SOMEONE_AGE_LINES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SOMEONE_AGE_LINES"));
static _SOMEONE_KIDS_LINES: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SOMEONE_KIDS_LINES"));
static _NUMBER_WORD: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_map("_NUMBER_WORD"));
static _MORE_THAN_ONE: LazyLock<std::collections::HashSet<String>> =
    LazyLock::new(|| data::set(&format!("{M}_MORE_THAN_ONE")));
static _THE_LINE_BEFORE_IS_HEAVY: LazyLock<&'static Re> = LazyLock::new(|| re("_THE_LINE_BEFORE_IS_HEAVY"));

static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

fn lookup<'a>(table: &'a [(String, String)], key: &str) -> Option<&'a str> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// Python `int(s)` for the digits a `\d` group captured (Unicode decimal digits included, as
/// Python's `int` reads them).
fn py_int(s: &str) -> i64 {
    super::answers::py_int(s).unwrap_or_else(|| panic!("ValueError: invalid literal for int() with base 10: {s:?}"))
}

/// Python: bot/modules/their_life.py:257 `_more_than_one(num)`
pub fn _more_than_one(num: &str) -> bool {
    let said = text::strip(num);
    if _MORE_THAN_ONE.contains(said) {
        return true;
    }
    let first = text::slice(said, 0, Some(1));
    crate::understanding::pyuni::is_digit(&first) && said != "0" && said != "1"
}

/// Python: bot/modules/their_life.py:265 `_line(text)`
pub fn _line(text_: &str) -> String {
    let said = text::strip(&text_.replace('\u{2019}', "'")).to_lowercase();
    let said = text::strip_chars(&said, " .,!;:-\"'").to_string();
    let mut said = SPACES.sub(" ", &said);
    loop {
        let trimmed = _TONE.sub("", &said);
        if trimmed == said {
            return said;
        }
        said = trimmed;
    }
}

/// Python: bot/modules/their_life.py:277 `_a_fact_about_their_life(said)`
/// The kind of fact this is, or None: "place", "pet", "siblings" or "age".
pub fn _a_fact_about_their_life(said: &str) -> Option<String> {
    if said.is_empty() || said.contains('?') || _NOT_A_SHAPE.is_match(said) {
        return None;
    }
    if let Some(lived) = _LIVE.match_(said) {
        let where_ = lived.name_str("where");
        if _PLACE_KINDS.match_(where_).is_some() && !_STORY.is_match(where_) {
            return Some("place".to_string());
        }
    }
    if let Some(siblings) = _SIBLINGS.match_(said) {
        if !_STORY.is_match(&said[siblings.end()..]) {
            let what = _TONE.sub("", siblings.name_str("what"));
            let what = text::strip_chars(&what, " ,");
            if !what.is_empty() {
                return Some(format!("siblings:{what}"));
            }
        }
    }
    if let Some(age) = _AGE.match_(said) {
        if !_NOT_AN_AGE.is_match(said) {
            let n = py_int(age.name_str("age"));
            if (13..=99).contains(&n) {
                return Some(format!("age:{}", age.name_str("age")));
            }
        }
    }
    if let Some(pet) = _PET.match_(said) {
        if !_STORY.is_match(&said[pet.end()..]) {
            let num = text::strip(pet.name_str("num"));
            let kind = if _more_than_one(num) { "two_pets" } else { "pet" };
            return Some(format!("{kind}:{num}:{}", pet.name_str("pet")));
        }
    }
    if let Some(someone) = _SOMEONE.match_(said) {
        let rel = someone.name_str("rel");
        if let Some(detail) = _a_detail_about_someone(rel, someone.name_str("rest")) {
            return Some(format!("someone:{rel}:{detail}"));
        }
    }
    None
}

/// Python: bot/modules/their_life.py:306 `_a_detail_about_someone(rel, rest)`
pub fn _a_detail_about_someone(rel: &str, rest: &str) -> Option<String> {
    let said = text::strip(rest);
    if said.is_empty() {
        return None;
    }
    if let Some(worked) = _SOMEONE_WORK.match_(said) {
        if !_STORY.is_match(worked.name_str("place")) {
            return Some(format!("work:{}", text::strip(worked.name_str("place"))));
        }
    }
    if let Some(hours) = _SOMEONE_HOURS.match_(said) {
        return Some(format!("hours:{}", text::strip(hours.name_str("hours"))));
    }
    if let Some(age) = _SOMEONE_AGE.match_(said) {
        let n = py_int(age.name_str("age"));
        if (1..=99).contains(&n) && !_AGE_READ_ELSEWHERE.contains(text::strip(rel)) {
            return Some(format!("age:{n}"));
        }
    }
    if let Some(kids) = _SOMEONE_KIDS.match_(said) {
        return Some(format!("kids:{}", text::strip(kids.name_str("num"))));
    }
    None
}

/// Python: bot/modules/their_life.py:334 `_their_pronoun(rel)`
pub fn _their_pronoun(rel: &str) -> String {
    let said = SPACES.sub(" ", text::strip(rel));
    if _HER.contains(&said) {
        return "her".into();
    }
    if _HIM.contains(&said) {
        return "him".into();
    }
    "them".into()
}

/// Python: bot/modules/their_life.py:344 `_where_words(said)`
pub fn _where_words(said: &str) -> String {
    let where_ = match _LIVE.match_(said) {
        Some(lived) => lived.name_str("where").to_string(),
        None => said.to_string(),
    };
    text::upper_first(&where_)
}

/// Python: bot/modules/their_life.py:351 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    let said = _line(text_);
    let found = _a_fact_about_their_life(&said)?;
    let (kind, _, rest) = text::partition(&found, ":");
    let (num, animal) = match kind {
        "siblings" | "age" | "someone" => (rest.to_string(), String::new()),
        "pet" | "two_pets" => {
            let (a, b) = rest.split_once(':').expect("ValueError: not enough values to unpack");
            (a.to_string(), b.to_string())
        }
        _ => (String::new(), String::new()),
    };
    use crate::compose::fallback;
    if fallback::_the_thread_is_heavy(memory)
        || fallback::_reads_as_heavy_news(&said)
        || _THE_LINE_BEFORE_IS_HEAVY.is_match(&_the_line_before(memory))
    {
        return None;
    }
    Some(_the_fact(kind, &num, &animal, &said, memory))
}

/// Python: bot/modules/their_life.py:385 `_the_line_before(memory)`
pub fn _the_line_before(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// Python: bot/modules/their_life.py:391 `_the_fact(kind, num, animal, said, memory)`
pub fn _the_fact(kind: &str, num: &str, animal: &str, said: &str, memory: &Memory) -> String {
    if kind == "siblings" {
        let what = text::upper_first(num);
        return _pick(&_SIBLING_LINES, memory).replace("{What}", &what);
    }
    if kind == "age" {
        let line = _pick(&_AGE_LINES, memory);
        return line.replace("{Age}", num).replace("{Next}", &(py_int(num) + 1).to_string());
    }
    if kind == "place" {
        return _pick(&_PLACES, memory).replace("{Where}", &_where_words(said));
    }
    if kind == "someone" {
        let (rel, _, detail) = text::partition(num, ":");
        return _the_someone_line(rel, detail, memory);
    }
    if kind == "two_pets" {
        let line = _pick(&_PETS, memory);
        let count = lookup(&_NUMBER_WORDS, num).unwrap_or("The");
        return line.replace("{Pets}", &format!("{count} {}", _plural(animal)));
    }
    let line = _pick(&_ONE_PET, memory);
    // `animal[:1] in "aeiou"`: the empty string is in every string
    let vowel = animal.chars().next().map_or(true, |c| "aeiou".contains(c));
    line.replace("{Pet}", &format!("{}{animal}", if vowel { "An " } else { "A " }))
}

/// Python: bot/modules/their_life.py:411 `_the_someone_line(rel, detail, memory)`
pub fn _the_someone_line(rel: &str, detail: &str, memory: &Memory) -> String {
    let (kind, _, said) = text::partition(detail, ":");
    let them = _their_pronoun(rel);
    if kind == "work" {
        let line = _pick(&_SOMEONE_WORK_LINES, memory);
        let place = if line.starts_with("{Place}") { text::capitalize(said) } else { said.to_string() };
        return line.replace("{Place}", &place).replace("{them}", &them);
    }
    if kind == "hours" {
        let line = _pick(&_SOMEONE_HOURS_LINES, memory);
        return line.replace("{Hours}", &text::capitalize(said)).replace("{them}", &them);
    }
    if kind == "age" {
        let line = _pick(&_SOMEONE_AGE_LINES, memory);
        return line.replace("{Age}", said).replace("{them}", &them);
    }
    let line = _pick(&_SOMEONE_KIDS_LINES, memory);
    let num = match lookup(&_NUMBER_WORD, said) {
        Some(n) => n.to_string(),
        None => text::capitalize(said),
    };
    line.replace("{Num}", &num)
}

/// Python: bot/modules/their_life.py:431 `_plural(word)`
pub fn _plural(word: &str) -> String {
    if word.ends_with('s') || matches!(word, "fish" | "sheep" | "deer") {
        return word.to_string();
    }
    if let Some(stem) = word.strip_suffix('y') {
        return format!("{stem}ies");
    }
    format!("{word}s")
}

/// Python: bot/modules/their_life.py:441 `_pick(pool, memory)`
/// The first line of the pool this conversation has not said yet.
pub fn _pick(pool: &[String], memory: &Memory) -> String {
    let said: std::collections::HashSet<&str> = memory.turns.iter().map(|t| text::strip(&t.bot)).collect();
    for line in pool {
        if !said.contains(line.as_str()) {
            return line.clone();
        }
    }
    pool[0].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values from `harness/pycall.py bot.modules.their_life ...`.
    #[test]
    fn readings() {
        assert_eq!(_line("We have a dog tbh!"), "we have a dog");
        assert_eq!(_plural("puppy"), "puppies");
        assert_eq!(_plural("fish"), "fish");
        assert!(_more_than_one("two"));
        assert!(_more_than_one("10"));
        assert!(!_more_than_one("1"));
    }
}
