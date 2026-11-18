//! Port of `bot/memory/lineback.py`: a detail the person gave earlier, found again from the
//! words of the ask that wants it back.
//!
//! "and which knee was it" late in a chat is answered by the person's own line that carries the
//! ask's content words ("You said: \"it's on the outside of the knee, just under the bone\""),
//! or by the value of the attribute the ask names. Nothing is filed: the lines are in
//! `memory["turns"]`. Narrow: the line has to be a recall ask as people phrase it, a recall ask
//! is never the answer, and a content word has to be shared.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, inline_re, res, sets, values};
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

res!("lineback"; ASKS_BACK, _AN_ATTRIBUTE, _NOT_A_DETAIL, _QUESTION, _INTENTION, ASKS_BACK_A_KIND, _A_TELLING, _A_PAST_ACT, _A_MY_VALUE, _A_VALUE_START, _A_VALUE_VERB, _A_THE_CALENDAR, _A_SAID_AGAIN, _A_FIGURE, _AN_AMOUNT_ASKED);
sets!("lineback"; _THE_ASK, _VAGUE);
values!("lineback"; _A_KIND_FOR_WORD, _SHORT_WORD);
inline_re! {
    WORDS = r"[a-z']+", 0;
    HOW_MUCH = r"\bhow\s+much\b", 0;
    A_LETTER = r"[a-z0-9]", 0;
    LONG_LINE_PARTS = r"(?<=[.!?;])\s+|,\s+(?=and\b|but\b)", 0;
}
static A_KIND: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.memory.lineback._A_KIND").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().expect("a kind pattern"))).collect()
});

fn short_word() -> usize {
    _SHORT_WORD.as_int().unwrap_or(3) as usize
}

/// `_A_KIND_FOR_WORD.get(word)`.
fn kind_for_word(word: &str) -> Option<&'static str> {
    _A_KIND_FOR_WORD.get(word).and_then(Value::as_str)
}

/// Python: bot/memory/lineback.py:160 `flat(text)`
///
/// The line as typed, with the spacing evened out.
pub fn flat(text: &str) -> String {
    pt::squash(text)
}

/// Python: bot/memory/lineback.py:165 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    WORDS.findall_str(&pt::lower(&flat(text))).iter().map(|w| pt::strip_chars(w, "'").to_string()).collect()
}

/// Python: bot/memory/lineback.py:169 `_stem(word)`
///
/// A crude shared stem, so "revising" answers "revise" and "shifts" answers "shift".
pub fn _stem(word: &str) -> String {
    for suffix in ["ing", "ies", "es", "ed", "s"] {
        if pt::len(word) > suffix.len() + short_word() && word.ends_with(suffix) {
            return word[..word.len() - suffix.len()].to_string();
        }
    }
    word.to_string()
}

/// Python: bot/memory/lineback.py:177 `about(text)`
///
/// The content words of a recall ask, as stems: what the line wants said back.
pub fn about(text: &str) -> HashSet<String> {
    _words(text).iter().filter(|w| !_THE_ASK.contains(*w) && !_VAGUE.contains(*w)).map(|w| _stem(w)).collect()
}

/// Python: bot/memory/lineback.py:207 `_a_kind_named(text)`
///
/// The first attribute word the line names, or None.
pub fn _a_kind_named(text: &str) -> Option<String> {
    _words(text).into_iter().find(|w| kind_for_word(w).is_some())
}

/// Python: bot/memory/lineback.py:215 `is_a_recall_ask(text)`
///
/// Whether the line asks for something said earlier in this chat.
pub fn is_a_recall_ask(text: &str) -> bool {
    let flat_text = flat(text);
    if ASKS_BACK.is_match(&flat_text) {
        return true;
    }
    if let Some(named) = ASKS_BACK_A_KIND.search(&flat_text) {
        if _A_TELLING.match_(&flat_text[named.end()..]).is_none() {
            return true;
        }
    }
    if !_A_PAST_ACT.is_match(&flat_text) {
        return false;
    }
    _a_kind_named(&flat_text).is_some() || HOW_MUCH.is_match(&flat_text)
}

/// Python: bot/memory/lineback.py:234 `_the_last_reply(memory)`
///
/// The bot's own last line, in the same shape as the person's lines.
pub fn _the_last_reply(memory: &Memory) -> String {
    memory.turns.iter().rev().map(|turn| flat(&turn.bot)).find(|bot| !bot.is_empty()).unwrap_or_default()
}

/// Python: bot/memory/lineback.py:243 `the_lines(memory)`
///
/// The person's own lines this chat, oldest first.
pub fn the_lines(memory: &Memory) -> Vec<String> {
    memory.turns.iter().map(|turn| flat(&turn.user)).filter(|line| !line.is_empty()).collect()
}

/// Python: bot/memory/lineback.py:257 `_the_best_line(wanted, lines, questions=False)`
///
/// The earlier line that carries most of what the ask is about, or None.
pub fn _the_best_line(wanted: &HashSet<String>, lines: &[String], questions: bool) -> Option<String> {
    let mut best: Option<((usize, usize, usize), &String)> = None;
    for (place, line) in lines.iter().enumerate() {
        if is_a_recall_ask(line) {
            continue; // an ask for the thing is not the thing
        }
        if _QUESTION.is_match(line) && !questions {
            continue; // a question they asked is not a detail they gave
        }
        let shared: Vec<String> = about_any(line).into_iter().filter(|w| wanted.contains(w)).collect();
        if shared.is_empty() {
            continue;
        }
        let longest = shared.iter().map(|w| pt::len(w)).max().unwrap_or(0);
        if longest < short_word() {
            continue;
        }
        let score = (shared.len(), longest, place);
        if best.as_ref().map_or(true, |(b, _)| score > *b) {
            best = Some((score, line));
        }
    }
    best.map(|(_, line)| line.clone())
}

/// Python: bot/memory/lineback.py:277 `about_any(text)`
///
/// Every stem the line carries, content word or not: what it can be found by.
pub fn about_any(text: &str) -> HashSet<String> {
    _words(text).iter().filter(|w| !_THE_ASK.contains(*w) && pt::len(w) >= short_word()).map(|w| _stem(w)).collect()
}

/// Python: bot/memory/lineback.py:319 `_the_kind_asked(said)`
///
/// The kind of value the ask is about, when the ask is about the attribute and nothing else.
pub fn _the_kind_asked(said: &str) -> Option<String> {
    if _A_THE_CALENDAR.is_match(said) && !_A_SAID_AGAIN.is_match(said) {
        return None;
    }
    let words: std::collections::BTreeSet<String> = _words(said).into_iter().filter(|w| kind_for_word(w).is_some()).collect();
    if words.is_empty() {
        // "how much was it again" names no attribute: a quantity asked that way is a price.
        if !HOW_MUCH.is_match(said) || !about(said).is_empty() {
            return None;
        }
        return Some("price".into());
    }
    // "what size was the coat" has a subject of its own.
    let stems: HashSet<String> = words.iter().map(|w| _stem(w)).collect();
    if about(said).iter().any(|w| !stems.contains(w)) {
        return None;
    }
    kind_for_word(words.iter().next().expect("not empty")).map(str::to_string)
}

/// Python: bot/memory/lineback.py:338 `_a_value_of_the_kind(kind, lines)`
///
/// The last value of that kind the person gave, said back, or None when they gave none.
pub fn _a_value_of_the_kind(kind: &str, lines: &[String]) -> Option<String> {
    let pattern = A_KIND.iter().find(|(k, _)| k == kind)?.1;
    for line in lines.iter().rev() {
        let flattened = flat(line);
        let Some(m) = pattern.search(&flattened) else {
            continue;
        };
        let first = m.groups().into_iter().flatten().find(|g| !g.is_empty());
        let value = pt::strip_chars(first.unwrap_or(m.as_str()), " .,;").to_string();
        let value = pt::strip_chars(&_A_VALUE_VERB.sub("", &value), " .,;").to_string();
        if A_LETTER.is_match(&value) && mem7::word_count(&value) <= 4 {
            return Some(value);
        }
    }
    None
}

/// Python: bot/memory/lineback.py:354 `_the_value(line)`
///
/// The value of a "my <thing> is <value>" line, or None when the line is not that shape.
pub fn _the_value(line: &str) -> Option<String> {
    let flattened = flat(line);
    let m = _A_MY_VALUE.search(&flattened)?;
    let value = pt::strip(m.name_str("value"));
    if !A_LETTER.is_match(value) || mem7::word_count(value) > 6 {
        return None;
    }
    _A_VALUE_START.match_(value).map(|_| value.to_string())
}

/// Python: bot/memory/lineback.py:366 `_carried(line, wanted)`
///
/// The sentence of the line that carries what the ask is about, for a long line.
pub fn _carried(line: &str, wanted: &HashSet<String>) -> String {
    if pt::len(line) <= 120 {
        return line.to_string();
    }
    for part in LONG_LINE_PARTS.split(line) {
        if about_any(&part).iter().any(|w| wanted.contains(w)) {
            return pt::strip_chars(&part, " ,.;").to_string();
        }
    }
    line.to_string()
}

/// Python: bot/memory/lineback.py:376 `_the_value_of_the_kind(said, lines, memory)`
///
/// The value of the attribute the ask names, when the ask names the attribute and nothing else.
pub fn _the_value_of_the_kind(said: &str, lines: &[String], _memory: &Memory) -> Option<String> {
    let kind = _the_kind_asked(said)?;
    _a_value_of_the_kind(&kind, lines).map(|value| format!("You said {value}."))
}

/// Python: bot/memory/lineback.py:404 `_the_figure_for_a_thing(said, lines, memory)`
///
/// The figure they gave for the thing the ask names, when the ask is "how much <thing>".
pub fn _the_figure_for_a_thing(said: &str, lines: &[String], _memory: &Memory) -> Option<String> {
    if !_AN_AMOUNT_ASKED.is_match(said) {
        return None;
    }
    let wanted: HashSet<String> = about(said).into_iter().filter(|w| !_VAGUE.contains(w)).collect();
    if wanted.is_empty() {
        return None;
    }
    for line in lines.iter().rev() {
        if about_any(line).is_disjoint(&wanted) {
            continue;
        }
        if let Some(found) = _A_FIGURE.search(line) {
            return Some(format!("You said {}.", pt::rstrip_chars(pt::strip(found.as_str()), " .,")));
        }
    }
    None
}

/// Python: bot/memory/lineback.py:425 `_the_line_itself(said, lines, memory)`
///
/// The earlier line that carries what the ask is about, quoted back or said as its value.
pub fn _the_line_itself(said: &str, lines: &[String], memory: &Memory) -> Option<String> {
    let figure = _NOT_A_DETAIL.is_match(said);
    if figure && _the_kind_asked(said).is_none() {
        return None;
    }
    let wanted = about(said);
    if wanted.is_empty() || figure {
        return None;
    }
    let line = _the_best_line(&wanted, lines, _INTENTION.is_match(said))?;
    // The ask names an attribute the line does not carry: another reader's, or nobody's.
    let named: HashSet<String> = _words(said).iter().filter(|w| _AN_ATTRIBUTE.fullmatch(w).is_some()).map(|w| _stem(w)).collect();
    let in_line = about_any(&line);
    if !named.is_empty() && named.is_disjoint(&in_line) {
        return None;
    }
    // The bot's own last answer already names what the ask is about, so it has said it.
    let answered = about_any(&_the_last_reply(memory));
    if wanted.iter().any(|w| in_line.contains(w) && answered.contains(w)) {
        return None;
    }
    if let Some(value) = _the_value(&line) {
        return Some(format!("You said {}.", pt::rstrip_chars(&value, " .")));
    }
    Some(format!("You said: \"{}\"", _carried(&line, &wanted)))
}

/// Python: bot/memory/lineback.py:477 `_the_kind_they_gave(said, lines, memory)`
///
/// The earlier line that holds the kind of detail this line asks for, said back.
pub fn _the_kind_they_gave(said: &str, lines: &[String], _memory: &Memory) -> Option<String> {
    let found = mem7::PyVal::val(crate::memory::kinds::the_line_that_gives(said, lines));
    if !found.truthy() {
        return None;
    }
    // `found["line"]`
    let line = found.get("line").unwrap_or_else(|| panic!("KeyError: 'line'"));
    Some(format!("You said: \"{}\"", mem7::py_str(line)))
}

/// Python: bot/memory/lineback.py:494 `the_kind_they_gave(text, memory)`
///
/// The earlier line of theirs that gives the kind of detail this line asks for, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns what `kinds.the_line_that_gives`
/// returns (a dict) or None.
pub fn the_kind_they_gave(text: &str, memory: &Memory) -> Option<Value> {
    let said = flat(text);
    if said.is_empty() {
        return None;
    }
    if _A_THE_CALENDAR.is_match(&said) && !_A_SAID_AGAIN.is_match(&said) {
        return None; // "what day is it" is the calendar
    }
    Some(mem7::PyVal::val(crate::memory::kinds::the_line_that_gives(&said, &the_lines(memory)))).filter(|found| !found.is_none())
}

/// Python: bot/memory/lineback.py:509 `an_earlier_turn(text, memory)`
///
/// Is this line asking for something said earlier, and what was said? The reply that says it
/// back, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn an_earlier_turn(text: &str, memory: &Memory) -> Option<String> {
    let said = flat(text);
    let mut lines = the_lines(memory);
    if lines.len() > 1 && lines.last() == Some(&said) {
        lines.pop(); // the line being answered is the last of them
    }
    if !is_a_recall_ask(&said) {
        return _the_kind_they_gave(&said, &lines, memory);
    }
    let readers: [fn(&str, &[String], &Memory) -> Option<String>; 3] = [_the_value_of_the_kind, _the_figure_for_a_thing, _the_line_itself];
    for reader in readers {
        if let Some(answer) = reader(&said, &lines, memory).filter(|a| !a.is_empty()) {
            return Some(answer);
        }
    }
    _the_kind_they_gave(&said, &lines, memory)
}

/// Python: bot/memory/lineback.py:532 `reply_for(text, memory)`
///
/// What to say when the line asks again for something the person gave earlier, or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    an_earlier_turn(text, memory)
}
