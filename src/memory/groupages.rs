//! Port of `bot/memory/groupages.py`: the ages of a group of people, kept as the list the
//! person said (`memory["group_ages"] = {"word": "kids", "ages": [4, 7], "line": ...}`), and
//! the plural ask answered from it.

use super::mem8_py::{py_int_str, rx, strs, val};
use crate::data::Value;
use crate::py::re::{self, Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.groupages.";

strs!(GROUP_WORDS);
val!(_THE_SAME_GROUP, _THE_SINGULARS);
rx!(GIVES_A_LIST, GIVES_A_LIST_BARE, THEYRE_A_LIST, ASKS_THE_LIST, ASKS_THE_END_ONE);

static A_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d{1,3}", 0));
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z']+", 0));
/// `re.compile(r"\b%s\b" % re.escape(found))` for each group word, in GROUP_WORDS order.
static EACH_GROUP: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    GROUP_WORDS.iter().map(|w| (w.clone(), Re::new(&format!(r"\b{}\b", re::escape(w)), 0))).collect()
});

/// Python: bot/memory/groupages.py:92 `_the_word(word)`
///
/// The group word as the answer says it, or "" for a word that is not a group.
pub fn _the_word(word: &str) -> String {
    let found = text::strip(&word.to_lowercase()).to_string();
    let found = _THE_SAME_GROUP.get(&found).map(|v| v.str().to_string()).unwrap_or(found);
    if GROUP_WORDS.contains(&found) { found } else { String::new() }
}

/// Python: bot/memory/groupages.py:99 `_ages_in(value)`
///
/// The ages a match holds, as integers, in the order they were said.
pub fn _ages_in(value: &str) -> Vec<i64> {
    A_NUMBER.findall_str(value).iter().filter_map(|n| py_int_str(n)).collect()
}

/// Python: bot/memory/groupages.py:104 `the_list_given(text)`
///
/// The group word and the ages the line gives, or None. Two ages at least.
pub fn the_list_given(text: &str) -> Option<(String, Vec<i64>)> {
    for pattern in [*GIVES_A_LIST, *GIVES_A_LIST_BARE, *THEYRE_A_LIST] {
        let Some(found) = pattern.search(text) else { continue };
        let word = _the_word(found.name_str("word"));
        let ages = _ages_in(found.name_str("ages"));
        if !word.is_empty() && ages.len() >= 2 {
            return Some((word, ages));
        }
    }
    None
}

/// Python: bot/memory/groupages.py:122 `watch(text, memory)`
///
/// Keep the list a line gives, so the ask a turn later is answered from it.
pub fn watch(text: &str, memory: &mut Memory) {
    let Some((word, ages)) = the_list_given(text) else { return };
    let mut kept = Value::dict();
    kept.set_key("word", word);
    kept.set_key("ages", Value::from(ages.into_iter().map(Value::Int).collect::<Vec<_>>()));
    kept.set_key("line", text::squash(text));
    memory.set("group_ages", kept);
}

/// Python: bot/memory/groupages.py:133 `_the_kept(memory)`
///
/// The group and the ages the chat holds, or ("", []) when it holds none.
pub fn _the_kept(memory: &Memory) -> (String, Vec<i64>) {
    let Some(kept) = memory.get("group_ages").filter(|k| k.is_dict()) else { return (String::new(), Vec::new()) };
    let word = match kept.get("word") {
        Some(Value::Str(s)) => _the_word(s),
        Some(v) if v.truthy() => _the_word(&super::mem8_py::py_str(v)),
        _ => String::new(),
    };
    // isinstance(age, int): a bool is an int in Python.
    let ages: Vec<i64> = match kept.get("ages") {
        Some(v) if v.truthy() => v.items().iter().filter_map(Value::as_int).collect(),
        _ => Vec::new(),
    };
    if word.is_empty() || ages.len() < 2 {
        return (String::new(), Vec::new());
    }
    (word, ages)
}

/// Python: bot/memory/groupages.py:147 `_a_list(ages)`
///
/// The ages as a person says them: "4 and 7", "4, 7 and 9".
pub fn _a_list(ages: &[i64]) -> String {
    let said: Vec<String> = ages.iter().map(i64::to_string).collect();
    if said.len() == 1 {
        return said[0].clone();
    }
    format!("{} and {}", said[..said.len() - 1].join(", "), said[said.len() - 1])
}

/// Python: bot/memory/groupages.py:170 `_the_singulars(word)`
///
/// The singular of the group word the chat holds, in the words an ask may put after it.
pub fn _the_singulars(word: &str) -> HashSet<String> {
    let mut out: HashSet<String> = _THE_SINGULARS.get(word).map(|v| v.str_set()).unwrap_or_default();
    if let Some(one) = word.strip_suffix('s') {
        out.insert(one.to_string());
    }
    out.into_iter().filter(|one| !one.is_empty()).collect()
}

/// Python: bot/memory/groupages.py:179 `_the_end_one_is_the_group(text, end_one, word)`
///
/// Whether the end-of-the-list ask is about the group the chat holds. (`end_one` is the match
/// of ASKS_THE_END_ONE in `text`; the Python takes the match object.)
pub fn _the_end_one_is_the_group(text: &str, end_one: &Match, word: &str) -> bool {
    let after = &text[end_one.end()..];
    let low = after.to_lowercase();
    let words: Vec<String> = NOT_A_WORD.split(&low).into_iter().filter(|w| !w.is_empty()).collect();
    let Some(first) = words.first() else { return true };
    if ["one", "ones", "of"].contains(&first.as_str()) {
        return true;
    }
    _the_singulars(word).contains(first)
}

/// Python: bot/memory/groupages.py:197 `_the_group_asked(text, word)`
///
/// The group word the ask names, "" when it names no group, or None when it names another.
pub fn _the_group_asked(text: &str, word: &str) -> Option<String> {
    let line = text.to_lowercase();
    let named: Vec<String> =
        EACH_GROUP.iter().filter(|(_, rx)| rx.is_match(&line)).map(|(found, _)| _the_word(found)).collect();
    if named.is_empty() {
        return Some(String::new());
    }
    if named.iter().any(|n| n == word) { Some(word.to_string()) } else { None }
}

/// Python: bot/memory/groupages.py:211 `reply_for(text, memory)`
///
/// The reply to an ask about the ages the chat holds, or "" for every other line.
pub fn reply_for(text: &str, memory: &Memory) -> String {
    let (word, ages) = _the_kept(memory);
    if word.is_empty() || ages.is_empty() {
        return String::new();
    }
    let line = text;
    let Some(asked) = _the_group_asked(line, &word) else { return String::new() };
    let end_one = ASKS_THE_END_ONE.search(line);
    if let Some(end) = &end_one {
        if !_the_end_one_is_the_group(line, end, &word) {
            return String::new();
        }
        let which = end.name_str("which").to_lowercase();
        let young = ["youngest", "younger", "little", "little one"].contains(&which.as_str());
        let youngest = *ages.iter().min().unwrap();
        let oldest = *ages.iter().max().unwrap();
        if asked.is_empty() {
            return if young { format!("Your youngest is {youngest}.") } else { format!("Your oldest is {oldest}.") };
        }
        // The group is named as well as the end of it: "how old is my youngest kid".
        let one = word.strip_suffix('s').unwrap_or(&word);
        return if young { format!("Your youngest {one} is {youngest}.") } else { format!("Your oldest {one} is {oldest}.") };
    }
    if !ASKS_THE_LIST.is_match(line) {
        return String::new();
    }
    format!("Your {} are {}.", word, _a_list(&ages))
}

/// Python: bot/memory/groupages.py:240 `asks_for_the_list(text, memory)`
///
/// True when the line asks for the ages of the group the chat holds.
pub fn asks_for_the_list(text: &str, memory: &Memory) -> bool {
    !reply_for(text, memory).is_empty()
}
