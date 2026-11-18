//! Port of `bot/understanding/lexicon.py`: the word tables the engine reads (all data, from the
//! bundle by Python name) and the two functions written against them.

use crate::data;
use crate::py::re::Re;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.understanding.lexicon.";

fn set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static NOT_A_LAUGH: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_A_LAUGH"));
static LAUGHTER: LazyLock<HashSet<String>> = LazyLock::new(|| set("LAUGHTER"));
static _LAUGH_RUN: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_LAUGH_RUN")));
static _H_RUN: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}_H_RUN")));

/// `_words(text)`: the words of a word-list string, lower-cased (comments after `#` dropped).
pub fn _words(text: &str) -> HashSet<String> {
    let mut words = HashSet::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("");
        for word in crate::py::text::split_ref(line) {
            let bare = word.replace(['-', '\''], "");
            if crate::py::text::is_alpha(&bare) {
                words.insert(word.to_lowercase());
            }
        }
    }
    words
}

/// `is_a_laugh(word)`: "haha", "lol", "hahahah" and friends.
pub fn is_a_laugh(word: &str) -> bool {
    if NOT_A_LAUGH.contains(word) {
        return false;
    }
    if LAUGHTER.contains(word) {
        return true;
    }
    (crate::py::text::len(word) > 2 && _LAUGH_RUN.fullmatch(word).is_some()) || _H_RUN.fullmatch(word).is_some()
}

/// `every_word()`: every word in the engine's public word tables (lexicon, topics, kinds, body),
/// the words the engine can read without a typo fix.
pub fn every_word() -> &'static HashSet<String> {
    static ENGINE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| {
        let mut words = HashSet::new();
        for module in ["lexicon", "topics", "kinds", "body"] {
            let prefix = format!("bot.understanding.{module}.");
            for full in data::names_with_prefix(&prefix) {
                let name = &full[prefix.len()..];
                if name.starts_with('_') || name.contains("MISSPELLING") {
                    continue;
                }
                let value = data::get(full);
                let items: Vec<&data::Value> = match value {
                    data::Value::List(_) | data::Value::Tuple(_) | data::Value::Set(_) => value.items().iter().collect(),
                    data::Value::Dict(d) => d.iter().map(|(k, _)| k).collect(),
                    _ => continue,
                };
                for item in items {
                    let Some(s) = item.as_str() else { continue };
                    for word in crate::py::text::split_ref(&s.to_lowercase().replace(',', " ")) {
                        if crate::py::text::is_alpha(word) {
                            words.insert(word.to_string());
                        }
                    }
                }
            }
        }
        words
    });
    &ENGINE_WORDS
}
