//! Port of `bot/memory/measure.py`: a follow-up that asks the measure again - "and how much",
//! "and how often", "and how long".
//!
//! The figure is already in the answer above, so the follow-up is answered from that answer:
//! the clause of it that carries the measure asked for, in the answer's own words. Narrow on
//! purpose: the line is only the measure, the answer above is a card the bot really said on the
//! turn before, and the clause handed back carries the measure.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, inline_re, res, sets, values};
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;

res!("measure"; LEAD, TAIL, ASKED, SENTENCE, THING_AFTER, A_CONTAINER);
sets!("measure"; KEEP_THEIR_TURN, A_NEW_CLAUSE, STOPS_THE_THING, THING_NOT_WORTH_SAYING);
values!("measure"; MEASURES);
inline_re! {
    A_BOUNDARY = r"[,;:]\s+", 0;
    SPACES = r"\s+", 0;
}

/// Python: bot/memory/measure.py:73 `the_measure_asked(line)`
///
/// Which measure the line asks for, or "" when the line is not only a measure.
pub fn the_measure_asked(line: &str) -> String {
    let text = pt::strip(line);
    if text.is_empty() {
        return String::new();
    }
    let text = LEAD.sub("", text);
    let text = TAIL.sub("", &text);
    let text = pt::strip_chars(&text, " ?.!,:;-").to_string();
    let text = LEAD.sub("", &text);
    let text = pt::strip_chars(&text, " ?.!,:;-");
    match ASKED.fullmatch(text) {
        Some(m) => pt::lower(m.str(1)),
        None => String::new(),
    }
}

/// `" ".join(text.lower().split()[:count])`.
fn first_words(text: &str, count: usize) -> String {
    let low = pt::lower(text);
    let words = pt::split_ref(&low);
    words[..count.min(words.len())].join(" ")
}

/// Python: bot/memory/measure.py:98 `_the_card_said_last(memory)`
///
/// The card the bot answered with on the turn before, or "" when the last turn was not one.
pub fn _the_card_said_last(memory: &Memory) -> String {
    let Some(last) = memory.turns.last() else {
        return String::new();
    };
    // After the loss and grief modules or the safety band have answered, "and how long" is about
    // what they said, and those modules keep their turn.
    if KEEP_THEIR_TURN.contains(last.module.as_deref().unwrap_or("")) {
        return String::new();
    }
    let said = pt::squash(&pt::lower(&last.bot));
    if said.is_empty() {
        return String::new();
    }
    let records: Vec<Value> = match memory.get("entry_answers") {
        Some(Value::List(records)) if !records.is_empty() => records.to_vec(),
        _ => vec![mem7::mem(memory, "entry_answer").clone()],
    };
    for record in records.iter().rev() {
        if !record.is_dict() {
            continue;
        }
        let text = pt::strip(&mem7::text_of(record.get("text"))).to_string();
        if text.is_empty() {
            continue;
        }
        if said.starts_with(&first_words(&text, 8)) || said.contains(&first_words(&text, 5)) {
            return text;
        }
    }
    String::new()
}

/// Python: bot/memory/measure.py:140 `_the_clause_that_carries(sentence, pattern=None)`
///
/// The first clause of the sentence that carries the measure, in the sentence's own words.
/// (`pattern` is not read by PyBot either.)
pub fn _the_clause_that_carries(sentence: &str, _pattern: Option<&Re>) -> String {
    for boundary in A_BOUNDARY.finditer(sentence) {
        let rest = &sentence[boundary.end()..];
        let pieces = SPACES.split(pt::strip(rest));
        let first = pt::lower(pt::strip_chars(pieces.first().map_or("", String::as_str), ",.;:"));
        if A_NEW_CLAUSE.contains(&first) {
            let said = pt::strip(&sentence[..boundary.start()]);
            if mem7::word_count(said) >= 4 {
                return said.to_string();
            }
        }
    }
    sentence.to_string()
}

/// Python: bot/memory/measure.py:153 `_the_sentence_with(text, measure)`
///
/// The clause of an answer that carries the measure asked for, or "" when none does.
pub fn _the_sentence_with(text: &str, measure: &str) -> String {
    let Some(pattern) = MEASURES.get(measure).and_then(Value::as_re) else {
        return String::new();
    };
    for part in SENTENCE.split(text) {
        let sentence = pt::strip(&part);
        if !sentence.is_empty() && pattern.is_match(sentence) {
            return _the_clause_that_carries(sentence, Some(pattern));
        }
    }
    String::new()
}

/// Python: bot/memory/measure.py:177 `_the_thing_the_line_named(memory)`
///
/// The short phrase the earlier line put straight after its measure word, or "": one to three
/// words with no verb in it, so "how much water" gives "water".
pub fn _the_thing_the_line_named(memory: &Memory) -> String {
    let said = memory.turns.iter().rev().map(|turn| turn.user.as_str()).find(|user| !pt::strip(user).is_empty()).unwrap_or("");
    let Some(m) = THING_AFTER.search(said) else {
        return String::new();
    };
    let mut kept: Vec<String> = Vec::new();
    for word in SPACES.split(pt::strip(m.name_str("thing"))).into_iter().filter(|word| !word.is_empty()) {
        if STOPS_THE_THING.contains(&pt::lower(pt::strip_chars(&word, ",.;:"))) {
            break;
        }
        kept.push(pt::strip_chars(&word, ",.;:").to_string());
        if kept.len() == 3 {
            break;
        }
    }
    let joined = kept.join(" ");
    let thing = pt::strip_chars(&joined, " ,.;:-");
    if !thing.is_empty() && !THING_NOT_WORTH_SAYING.contains(&pt::lower(thing)) {
        thing.to_string()
    } else {
        String::new()
    }
}

/// Python: bot/memory/measure.py:219 `_with_the_thing(sentence, thing)`
///
/// The sentence with the thing the line named set in front of its first container, or "".
pub fn _with_the_thing(sentence: &str, thing: &str) -> String {
    match A_CONTAINER.search(sentence) {
        Some(found) => format!("{} of {}{}", &sentence[..found.end()], thing, &sentence[found.end()..]),
        None => String::new(),
    }
}

/// Python: bot/memory/measure.py:229 `_as_a_headline(thing, sentence)`
///
/// The thing as a one-word headline, then the answer in lower case where it reads as prose.
pub fn _as_a_headline(thing: &str, sentence: &str) -> String {
    let mut chars = sentence.chars();
    let first = chars.next().expect("IndexError: the sentence is not empty");
    let rest = chars.as_str();
    let sentence = if first.is_uppercase() && mem7::is_lower(mem7::first_chars(rest, 1)) {
        mem7::low1(sentence)
    } else {
        sentence.to_string()
    };
    format!("{}: {}", pt::upper_first(thing), sentence)
}

/// Python: bot/memory/measure.py:238 `_as_a_sentence(text)`
///
/// The clause as the one sentence the reply is: a full stop where it has none.
pub fn _as_a_sentence(text: &str) -> String {
    if text.chars().last().is_some_and(|last| !".!?".contains(last)) {
        return format!("{text}.");
    }
    text.to_string()
}

/// Python: bot/memory/measure.py:247 `reply_for(text, memory)`
///
/// The clause of the answer above that answers the measure asked again, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let measure = the_measure_asked(text);
    if measure.is_empty() {
        return None;
    }
    let card = _the_card_said_last(memory);
    if card.is_empty() {
        return None;
    }
    let sentence = _the_sentence_with(&card, &measure);
    if sentence.is_empty() {
        return None;
    }
    let thing = _the_thing_the_line_named(memory);
    if !thing.is_empty() && !pt::lower(&sentence).contains(&pt::lower(&thing)) {
        // A card about water that never says water: the thing goes in front of its container,
        // or as a one-word headline.
        let said = _with_the_thing(&sentence, &thing);
        if !said.is_empty() {
            return Some(_as_a_sentence(&said));
        }
        return Some(_as_a_sentence(&_as_a_headline(&thing, &sentence)));
    }
    Some(_as_a_sentence(&sentence))
}
