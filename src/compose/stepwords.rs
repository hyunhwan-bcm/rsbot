//! Port of `bot/compose/stepwords.py`: the steps a plan's "to" can be followed by, and the
//! phrase after it said back ("dye my hair pink" -> "dyeing my hair pink").

use crate::data::{self, Value};
use crate::py::text;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static STEPS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.stepwords.STEPS"));
static DOUBLES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.stepwords.DOUBLES"));
static IRREGULAR: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.stepwords.IRREGULAR"));

/// Python: bot/compose/stepwords.py:38 `_ing(word)`
///
/// The -ing form of a step: "learn" -> "learning", "run" -> "running", "dye" -> "dyeing". Only
/// this file calls it, and it always returns a str.
pub fn _ing(word: &str) -> String {
    if word.is_empty() || word.ends_with("ing") {
        return word.to_string();
    }
    if let Some(irregular) = IRREGULAR.get(word) {
        return irregular.clone();
    }
    let w: Vec<char> = word.chars().collect();
    let n = w.len();
    if word.ends_with("ie") {
        return w[..n - 2].iter().collect::<String>() + "ying";
    }
    if word.ends_with('e') && !(word.ends_with("ee") || word.ends_with("ye") || word.ends_with("oe")) {
        return w[..n - 1].iter().collect::<String>() + "ing";
    }
    // A short word ending consonant-vowel-consonant doubles it ("run", "quit", "swim"); the "qu"
    // in "quit" is a consonant sound.
    let short = n <= 4 || DOUBLES.contains(word);
    let vowel = |c: char| "aeiou".contains(c);
    if short
        && n > 2
        && !"aeiouwxy".contains(w[n - 1])
        && vowel(w[n - 2])
        && (!vowel(w[n - 3]) || (w[n - 3] == 'u' && n >= 4 && w[n - 4] == 'q'))
    {
        return format!("{word}{}ing", w[n - 1]);
    }
    format!("{word}ing")
}

/// Python: bot/compose/stepwords.py:58 `a_step(word)`
///
/// True when the word after a plan's "to" is the step itself rather than a place or a thing.
pub fn a_step(word: &str) -> bool {
    STEPS.contains(text::strip_chars(&text::lower(text::strip(word)), ".,!?"))
}

/// Python: bot/compose/stepwords.py:63 `said_after_a_to(phrase, ing=None)`
///
/// The words after a plan's "to", said back as a phrase a sentence can hold: "the gym" ->
/// "going to the gym". `ing` is None (the default fold, `_ing`) or a function value; no caller
/// passes another fold, so only `_ing` itself is known here.
pub fn said_after_a_to(phrase: &str, ing: &Value) -> String {
    let squashed = text::squash(phrase);
    let phrase = text::strip_chars(&squashed, " .,!?:;");
    let words = text::split_ref(phrase);
    if words.is_empty() {
        return String::new();
    }
    if a_step(words[0]) {
        let folded = match ing {
            Value::None => _ing(words[0]),
            Value::Fn(f) if f.name() == "bot.compose.stepwords:_ing" => _ing(words[0]),
            other => panic!("TypeError: no Rust port of the fold {other:?}"),
        };
        let rest = if words.len() > 1 { format!(" {}", words[1..].join(" ")) } else { String::new() };
        return folded + &rest;
    }
    format!("going to {phrase}")
}
