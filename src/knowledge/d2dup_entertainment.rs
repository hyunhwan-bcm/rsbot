//! Helpers `entertainment` needs from code other workstreams own, ported privately until the
//! owners' ports are merged. Each is a complete port of the original and marked with its owner.

use crate::data;
use crate::py::re::Re;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/cooking.py (D2 group cookA/cookB)

// was DUP-D2(bot.knowledge.cooking::_fold): now the real port
pub use crate::knowledge::cooking::_fold;


// was DUP-D2(bot.knowledge.cooking::names_a_dish): now the real port
pub use crate::knowledge::cooking::names_a_dish;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/kept_animals.py (D1)

static KEPT_AVOID: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.kept_animals._AVOID"));
static KEPT_ASKS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.kept_animals._ASKS"));

// DUP(bot.knowledge.kept_animals::keeping_an_animal)
/// True when the line asks how to keep an animal, rather than how to cook one.
pub fn keeping_an_animal(understood: &Understood) -> bool {
    let joined;
    let said = if !understood.text.is_empty() {
        understood.text.as_str()
    } else if !understood.clean.is_empty() {
        understood.clean.as_str()
    } else {
        joined = understood.words.join(" ");
        joined.as_str()
    };
    let text = said.to_lowercase();
    if text.is_empty() {
        return false;
    }
    if KEPT_AVOID.is_match(&text) {
        return false;
    }
    KEPT_ASKS.is_match(&text)
}

// ---------------------------------------------------------------------------------------------
// bot/understanding/attempts.py (U)

fn attempts_set(name: &str) -> HashSet<String> {
    data::set(&format!("bot.understanding.attempts.{name}"))
}
static ONE_MORE: LazyLock<HashSet<String>> = LazyLock::new(|| attempts_set("ONE_MORE"));
static HANDS_OVER: LazyLock<HashSet<String>> = LazyLock::new(|| attempts_set("HANDS_OVER"));
static STARTS_A_QUESTION: LazyLock<HashSet<String>> = LazyLock::new(|| attempts_set("STARTS_A_QUESTION"));
static ENDS_THE_ATTEMPT: LazyLock<HashSet<String>> = LazyLock::new(|| attempts_set("ENDS_THE_ATTEMPT"));
static STARTS_A_CLAUSE: LazyLock<HashSet<String>> = LazyLock::new(|| attempts_set("STARTS_A_CLAUSE"));
static NOT_A_CLAUSE_AFTER_AND: LazyLock<HashSet<String>> = LazyLock::new(|| attempts_set("NOT_A_CLAUSE_AFTER_AND"));
static REPORT_SUBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| attempts_set("REPORT_SUBJECTS"));
static LOOK_BACK: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.understanding.attempts.LOOK_BACK").as_int().unwrap_or(4) as usize);
static CAN_YOU: LazyLock<Re> = LazyLock::new(|| Re::new("\\b(?:can|could|would|will)\\s+(?:you|u)\\b", 0));
static FOR_ME: LazyLock<Re> = LazyLock::new(|| Re::new("\\bfor (?:me|us)\\b|\\bplease\\b", 0));

// DUP(bot.understanding.attempts::_tokens)
fn _tokens(norm: &[String]) -> Vec<String> {
    norm.iter().map(|w| crate::py::text::strip(w).to_lowercase()).filter(|w| !w.is_empty()).collect()
}

// DUP(bot.understanding.attempts::_a_report)
fn _a_report(words: &[String], at: usize) -> bool {
    if at == 0 {
        return true;
    }
    words[at.saturating_sub(*LOOK_BACK)..at].iter().any(|w| REPORT_SUBJECTS.contains(w))
}

// DUP(bot.understanding.attempts::_where_the_attempt_ends)
fn _where_the_attempt_ends(words: &[String], at: usize) -> usize {
    let one_more_soon = |index: usize| words[(index + 1).min(words.len())..(index + 5).min(words.len())].iter().any(|o| ONE_MORE.contains(o));
    for index in at + 1..words.len() {
        let word = &words[index];
        if ENDS_THE_ATTEMPT.contains(word) || STARTS_A_CLAUSE.contains(word) {
            return index;
        }
        if word == "and" && (index + 1 == words.len() || NOT_A_CLAUSE_AFTER_AND.contains(&words[index + 1])) {
            return index;
        }
        if STARTS_A_QUESTION.contains(word) {
            let before = if index > 0 { words[index - 1].as_str() } else { "" };
            if index == at + 1
                || ["and", "but", "so", "then", "now", "everything", "anything", "something", "that", "this", "it"].contains(&before)
                || one_more_soon(index)
            {
                return index;
            }
        }
        if HANDS_OVER.contains(word) && one_more_soon(index) {
            return index;
        }
    }
    words.len()
}

// DUP(bot.understanding.attempts::the_rest_after_the_try)
/// The words left in the line once the thing it says was tried is behind it.
pub fn the_rest_after_the_try(norm: &[String]) -> Vec<String> {
    let words = _tokens(norm);
    for at in 0..words.len() {
        if words[at] != "tried" || !_a_report(&words, at) {
            continue;
        }
        return words[_where_the_attempt_ends(&words, at)..].to_vec();
    }
    Vec::new()
}

// DUP(bot.understanding.attempts::asks_for_one_more)
/// True when the line, past whatever was tried, asks for one more of it.
pub fn asks_for_one_more(norm: &[String]) -> bool {
    let rest = the_rest_after_the_try(norm);
    if rest.is_empty() {
        return false;
    }
    if !rest.iter().any(|w| ONE_MORE.contains(w)) {
        return false;
    }
    if rest.iter().any(|w| HANDS_OVER.contains(w)) {
        return true;
    }
    let joined = rest.join(" ");
    if CAN_YOU.is_match(&joined) {
        return true;
    }
    if rest.iter().any(|w| STARTS_A_QUESTION.contains(w)) {
        return true;
    }
    if rest.iter().any(|w| HANDS_OVER.contains(w)) && FOR_ME.is_match(&joined) {
        return true;
    }
    false
}
