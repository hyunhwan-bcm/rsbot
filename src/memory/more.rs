//! Port of `bot/memory/more.py`: "go on", "tell me more", "carry on" after an answer: the part
//! of a fuller answer not yet said, the ask the answer ended on, or a plain "that is all of it".

use super::mem8_py::{pick, rx, strs};
use super::repeat;
use crate::py::text;
use crate::state::Memory;
use crate::understanding::pyuni;
use std::collections::HashSet;

const M: &str = "bot.memory.more.";

rx!(CONTINUE, AN_ASK, MORE_ABOUT, _WORD);
strs!(GO_ON_WITH, NO_MORE, MORE_OF_IT);
static SENTENCE: std::sync::LazyLock<&'static crate::py::re::Re> =
    std::sync::LazyLock::new(|| crate::data::re("bot.memory.repeat.SENTENCE"));

/// `[part.strip() for part in repeat.SENTENCE.split(said.strip()) if part.strip()]`.
fn sentences(said: &str) -> Vec<String> {
    SENTENCE.split(text::strip(said)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/memory/more.py:78 `_its_ask(said)`
///
/// The thing the answer asked for at its end, or "" when it asked for nothing.
pub fn _its_ask(said: &str) -> String {
    let parts = sentences(said);
    let Some(last) = parts.last() else { return String::new() };
    if !(last.ends_with('?') || AN_ASK.match_(last).is_some()) {
        return String::new();
    }
    // It has to read as an ask rather than a paragraph that happens to end in a question.
    if text::split_ref(last).len() > 22 {
        return String::new();
    }
    last.clone()
}

/// Python: bot/memory/more.py:92 `_as_a_line(ask)`
///
/// The answer's ask as a sentence of its own: a capital only where one belongs.
pub fn _as_a_line(ask: &str) -> String {
    let mut said = text::strip(ask).to_string();
    if pyuni::is_upper(&text::slice(&said, 0, Some(1))) && !pyuni::is_upper(&text::slice(&said, 0, Some(2))) {
        said = super::mem8_py::lower_first(&said);
    }
    if text::ends_with_any(&said, &[".", "?", "!"]) { said } else { format!("{said}.") }
}

/// Python: bot/memory/more.py:110 `_saying_something(text)`
///
/// The line as tidied words.
pub fn _saying_something(text: &str) -> String {
    text::squash(text)
}

/// Python: bot/memory/more.py:115 `_about_what(text)`
///
/// "tell me more about X": X, when the line is that ask and X is not "it" or "that".
pub fn _about_what(text: &str) -> String {
    let said = _saying_something(text);
    let Some(found) = MORE_ABOUT.match_(&said) else { return String::new() };
    let what = text::squash(found.name_str("what"));
    if ["it", "that", "this", "them", "those"].contains(&what.to_lowercase().as_str()) {
        return String::new();
    }
    what
}

/// Python: bot/memory/more.py:126 `_is_what_was_asked(what, asked)`
///
/// True when every word of the thing named is a word of the line the answer was given to.
pub fn _is_what_was_asked(what: &str, asked: &str) -> bool {
    let low = asked.to_lowercase();
    let mut known: HashSet<String> = text::split(&low).into_iter().collect();
    let stripped: Vec<String> = known.iter().map(|w| w.trim_end_matches('s').to_string()).collect();
    known.extend(stripped);
    for word in text::split(&what.to_lowercase()) {
        if ["the", "a", "an", "my", "your", "our", "this", "that", "it"].contains(&word.as_str()) {
            continue;
        }
        if !known.contains(&word) && !known.contains(word.trim_end_matches('s')) {
            return false;
        }
    }
    true
}

/// Python: bot/memory/more.py:143 `_flat(text)`
pub fn _flat(text: &str) -> String {
    _WORD.findall_str(&text.to_lowercase()).join(" ")
}

/// Python: bot/memory/more.py:160 `_the_rest(said, answers)`
///
/// The part of a fuller answer from a turn or two back that the short one left out, or "".
/// (`answers` is `repeat.recent_answers(memory)`: (back, asked, said), newest first.)
pub fn _the_rest(said: &str, answers: &[(usize, String, String)]) -> String {
    let given = _flat(said);
    if text::split_ref(&given).len() < 6 || answers.is_empty() {
        return String::new();
    }
    for (_back, _asked, fuller) in &answers[1..] {
        let parts = sentences(fuller);
        if parts.len() < 2 || !given.contains(&_flat(&parts[0])) {
            continue;
        }
        let left: Vec<usize> = (0..parts.len()).filter(|&i| !given.contains(&_flat(&parts[i]))).collect();
        if left.is_empty() || left[0] == 0 {
            continue;
        }
        return parts[left[0]..].join(" ");
    }
    String::new()
}

/// Python: bot/memory/more.py:182 `asks_for_more(text)`
///
/// True when the whole line is "carry on" and nothing else.
pub fn asks_for_more(text: &str) -> bool {
    let said = _saying_something(text);
    if said.is_empty() || text::split_ref(&said).len() > 8 {
        return false;
    }
    CONTINUE.match_(&said).is_some() || MORE_ABOUT.match_(&said).is_some()
}

/// `repeat.handed_back(memory, reply)` as the str it returns.
fn handed_back(memory: &mut Memory, reply: &str) -> Option<String> {
    Some(repeat::_handed_back(memory, reply))
}

/// Python: bot/memory/more.py:190 `reply_for(text, memory)`
///
/// Keep going on the answer just given, or None when this is not that turn.
pub fn reply_for(text: &str, memory: &mut Memory) -> Option<String> {
    if !asks_for_more(text) {
        return None;
    }
    let named = _about_what(text);
    let answers = repeat::recent_answers(memory);
    let (back, asked, said) = answers.first()?.clone();
    // The turn just before this one is the answer being asked about; anything older is only that
    // answer if the chat has not moved on since.
    if back > 1 && !repeat::_about_the_same_thing(&asked, &said, memory) {
        return None;
    }
    if !named.is_empty() && !_is_what_was_asked(&named, &asked) {
        // A different subject, so this is a new question and the entry for it answers.
        return None;
    }
    let rest = _the_rest(&said, &answers);
    if !rest.is_empty() && !repeat::_already_repeated(&rest, memory) {
        let reply = pick(&MORE_OF_IT, memory, text).replace("{rest}", &rest);
        return handed_back(memory, &reply);
    }
    let ask = _its_ask(&said);
    if !ask.is_empty() {
        let reply = pick(&GO_ON_WITH, memory, text).replace("{ask}", &_as_a_line(&ask));
        return handed_back(memory, &reply);
    }
    let mut reply = pick(&NO_MORE, memory, text);
    if repeat::_already_repeated(&reply, memory) {
        // The person asked twice and got this already: a second ask gets the plainest version.
        reply = "That is all of it. Tell me what you have tried and I will work from that.".to_string();
        if repeat::_already_repeated(&reply, memory) {
            return None;
        }
    }
    handed_back(memory, &reply)
}
