//! Port of `bot/memory/remedy.py`: "i tried a hot bath" after advice about a problem, answered
//! with the question a friend asks - did it do anything?

use super::mem8_py::{rx, strs, strset};
use crate::py::random::Random;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.remedy.";

rx!(ATTEMPT, VERDICT);
strset!(POINTING, A_PROBLEM, ADVICE_MODULES, NOT_THE_THING);
strs!(ASK_IF_IT_HELPED);

static TURNS_AT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:but|so|then|though|because|again|twice|afterwards)\b", 0));
static AND_A_VERDICT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"\band\b(?=\s+(?:it|its|it's|they|i|we|now|still|nothing|didnt|doesnt|",
            r"no\b|the\s+\w+\s+(?:did|helped|worked)))"
        ),
        0,
    )
});
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static ADVICE_SAID: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:try|trying|keep|rest|ice|see (?:a|your) (?:gp|doctor))\b", 0));

/// Python: bot/memory/remedy.py:79 `reply_for(text, memory)`
///
/// The question about the thing they tried, or "" when this is not that line.
pub fn reply_for(text: &str, memory: &Memory) -> String {
    let said = text::squash(&text.to_lowercase());
    if said.is_empty() {
        return String::new();
    }
    let Some(found) = ATTEMPT.match_(&said) else { return String::new() };
    let rest = found.name_str("thing");
    let thing = _the_thing(rest, found.name_str("verb"));
    if thing.is_empty() {
        return String::new();
    }
    // The verdict words are read after the thing.
    if VERDICT.is_match(&text::slice(rest, text::len(&thing) as isize, None)) {
        return String::new();
    }
    if !_a_problem_was_spoken_of(memory) || !_advice_was_given(memory) {
        return String::new();
    }
    let options: Vec<String> = ASK_IF_IT_HELPED.iter().map(|template| _fill(template, &thing)).collect();
    _fresh(&options, memory)
}

/// Python: bot/memory/remedy.py:100 `_the_thing(rest, verb)`
///
/// The named thing they tried, without the article, as the words a sentence can carry.
pub fn _the_thing(rest: &str, verb: &str) -> String {
    let said = rest.to_lowercase();
    // The thing ends where the sentence turns.
    let said = TURNS_AT.split(&said).swap_remove(0);
    let said = AND_A_VERDICT.split(&said).swap_remove(0);
    let mut words: Vec<String> = WORDS.findall_str(&said);
    if words.is_empty() || POINTING.contains(&words[0]) || NOT_THE_THING.contains(&words[0]) || words[0] == "to" {
        return String::new();
    }
    // "we gave it a go" names nothing new; "we gave the ice pack a go" does.
    if verb == "gave" {
        let tail: Vec<&str> = words[words.len().saturating_sub(2)..].iter().map(String::as_str).collect();
        if tail != ["a", "go"] && tail != ["a", "try"] {
            return String::new();
        }
        words.truncate(words.len() - 2);
    }
    words.truncate(5);
    // Only the article and the possessive come off the end.
    while words.last().is_some_and(|w| ["a", "an", "the", "some", "my", "our", "their"].contains(&w.as_str())) {
        words.pop();
    }
    while words.first().is_some_and(|w| {
        ["a", "an", "the", "some", "my", "our", "their", "his", "her", "of", "that", "this"].contains(&w.as_str())
    }) {
        if words.len() < 2 {
            return String::new();
        }
        words.remove(0);
    }
    if words.first().is_some_and(|w| w == "of") {
        words.remove(0);
    }
    words.join(" ")
}

/// Python: bot/memory/remedy.py:131 `_advice_was_given(memory)`
///
/// Whether the chat holds an answer for them to be partway through (the last four turns).
pub fn _advice_was_given(memory: &Memory) -> bool {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(4)..].iter().rev() {
        let module = turn.module.clone().unwrap_or_default();
        if ADVICE_MODULES.contains(&module) {
            return true;
        }
        let reply = text::strip(&turn.bot);
        if ADVICE_SAID.is_match(&reply.to_lowercase()) {
            return true;
        }
        // A question about the thing in their hands is the chat being with them on it.
        if reply.ends_with('?') {
            return true;
        }
    }
    false
}

/// Python: bot/memory/remedy.py:158 `_a_problem_was_spoken_of(memory)`
///
/// Whether the last three turns have a problem in them, which is what an attempt is at.
pub fn _a_problem_was_spoken_of(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let mut words: HashSet<String> = HashSet::new();
    for turn in &turns[turns.len().saturating_sub(3)..] {
        words.extend(LETTERS.findall_str(&turn.user.to_lowercase()));
    }
    words.iter().any(|w| A_PROBLEM.contains(w))
}

/// Python: bot/memory/remedy.py:173 `_fill(template, thing)`
///
/// The template with the thing in it, capitalised where it opens the sentence.
pub fn _fill(template: &str, thing: &str) -> String {
    template.replace("{thing}", thing).replace("{Thing}", &text::upper_first(thing))
}

/// Python: bot/memory/remedy.py:178 `_fresh(options, memory)`
///
/// An option that has not just been said, so the same question does not come twice.
pub fn _fresh(options: &[String], memory: &Memory) -> String {
    let said: String = memory.turns.iter().map(|t| t.bot.as_str()).collect::<Vec<_>>().join(" ").to_lowercase();
    let unseen: Vec<&String> = options.iter().filter(|o| !said.contains(&o.to_lowercase())).collect();
    let pool: Vec<&String> = if unseen.is_empty() { options.iter().collect() } else { unseen };
    Random::new(memory.turns.len() as i128).choice(&pool).to_string()
}
