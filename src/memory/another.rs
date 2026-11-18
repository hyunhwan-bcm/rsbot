//! Port of `bot/memory/another.py`: they ask for another option, and the answer they were given
//! had more than one thing in it.
//!
//! "my tomato plants have blight" / "what else can i do about it" is about the answer just given,
//! and means "something other than that". The part of that answer the person has not been
//! handed back yet is the reply, said as what it is. Every word of it comes from the answer.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, lists, res, sets, texts, values};
use crate::memory::{named, repeat, topics};
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

res!("another"; A_MISSING_PART, OTHER_OPTION, AN_OPTION, CARRY_ON, LEAD_IN, HEDGE, STOCK, FRAME, DO_THIS, A_COMMENT, OFFERS_ANOTHER, LENGTH_IDIOM, ABOUT_ANOTHER, A_PERSON, FILING, RULED_OUT, A_CONDITION, A_PROFESSIONAL, FRAME_WORDS, URGENT_LINE);
sets!("another"; ASK, THEM, NOT_A_THING);
lists!("another"; ANSWER_MODULES, LEADS);
texts!("another"; QUOTE_MARKS);
values!("another"; LOOK_BACK, MOST, LEAST);
sets!("topics"; STOPWORDS);
static ABOUT_THE_BOT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.ABOUT_THE_BOT"));
static RED_FLAG: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.RED_FLAG"));

fn int_of(value: &Value) -> i64 {
    value.as_int().expect("an int global")
}

/// Python: bot/memory/another.py:260 `names_nothing(text)`
///
/// True when the line asks for another option or the rest of the answer, naming nothing itself.
pub fn names_nothing(text: &str) -> bool {
    let stripped = pt::strip(text);
    let asks_for_more = CARRY_ON.match_(stripped).is_some()
        || CARRY_ON.match_(&LEAD_IN.sub("", stripped)).is_some()
        || (OTHER_OPTION.is_match(text) && AN_OPTION.is_match(text))
        || A_MISSING_PART.is_match(text);
    if !asks_for_more || ABOUT_THE_BOT.is_match(text) || A_CONDITION.is_match(text) {
        return false;
    }
    let words = topics::words(text);
    !words.is_empty() && words.iter().all(|w| ASK.contains(w) || STOPWORDS.contains(w) || pt::is_digit(w))
}

/// `word.rstrip("s")`.
fn unplural(word: &str) -> String {
    pt::rstrip_chars(word, "s").to_string()
}

/// Python: bot/memory/another.py:277 `ruled_out_words(text)`
///
/// The things a line says the person does not want, as stems, so "gyms" meets "gym".
pub fn ruled_out_words(text: &str) -> HashSet<String> {
    if !RULED_OUT.is_match(text) {
        return HashSet::new();
    }
    topics::words(text).iter().filter(|w| pt::len(w) > 2 && !NOT_A_THING.contains(*w) && !pt::is_digit(w)).map(|w| unplural(w)).collect()
}

/// Python: bot/memory/another.py:285 `_names_one(part, ruled_out)`
///
/// True when a sentence of the old answer offers something the person has just ruled out.
pub fn _names_one(part: &str, ruled_out: &HashSet<String>) -> bool {
    !ruled_out.is_empty() && topics::words(part).iter().any(|w| ruled_out.contains(&unplural(w)))
}

/// Python: bot/memory/another.py:292 `_the_turn_to_lean_on(turns, memory=None)`
///
/// The last answer to lean on (its index in `turns`), and what the person ruled out between it
/// and the ask.
pub fn _the_turn_to_lean_on(turns: &[Turn], memory: &Memory) -> (Option<usize>, HashSet<String>) {
    let mut ruled_out = HashSet::new();
    for back in 1..=int_of(&LOOK_BACK) as usize {
        if turns.len() < back {
            break;
        }
        let index = turns.len() - back;
        let turn = &turns[index];
        let said = pt::strip(&turn.bot);
        // A step handed back out of an answer is not the answer.
        if !said.is_empty() && FILING.match_(said).is_none() && !repeat::a_hand_back(said, memory) {
            ruled_out.extend(ruled_out_words(&turn.user));
            return (Some(index), ruled_out);
        }
        // This turn only filed what they said.
        ruled_out.extend(ruled_out_words(&turn.user));
    }
    (None, ruled_out)
}

/// Python: bot/memory/another.py:313 `split_sentences(said)`
///
/// The sentences of a line, with a full stop inside a quotation not counted as an end.
pub fn split_sentences(said: &str) -> Vec<String> {
    let said: Vec<char> = pt::strip(said).chars().collect();
    let quote = |c: char| QUOTE_MARKS.contains(c);
    let mut parts: Vec<String> = Vec::new();
    let (mut start, mut opened, mut ended) = (0usize, false, false);
    for at in 0..said.len() {
        let c = said[at];
        if quote(c) {
            if opened {
                opened = false;
                if ended && at + 1 < said.len() && pt::is_space(said[at + 1]) {
                    parts.push(said[start..at + 1].iter().collect());
                    start = at + 1;
                }
                ended = false;
            } else {
                opened = true;
            }
        } else if ".!?".contains(c) {
            if opened {
                ended = true;
                continue;
            }
            let mut after = at + 1;
            while after < said.len() && quote(said[after]) {
                after += 1;
            }
            if after >= said.len() || pt::is_space(said[after]) {
                // `said[start:after]` (an empty slice when a quote run already moved start on)
                parts.push(if start < after { said[start..after].iter().collect() } else { String::new() });
                start = after;
            }
        }
    }
    parts.push(if start < said.len() { said[start..].iter().collect() } else { String::new() });
    parts.iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/memory/another.py:347 `_sentences(said)`
///
/// The answer as sentences, with the lead-in frame the bot puts on a repeat taken off.
pub fn _sentences(said: &str) -> Vec<String> {
    split_sentences(&FRAME.sub("", pt::strip(said)))
}

/// Python: bot/memory/another.py:354 `_said_before(memory, said, before=None)`
///
/// True when the bot has already said this sentence in a turn before the one at `before`
/// (default: the last turn). PyBot hands `_the_rest`'s sentence index here, and that is kept.
pub fn _said_before(memory: &Memory, said: &str, before: Option<i64>) -> bool {
    let wanted = pt::lower(mem7::first_chars(pt::strip(said), 50));
    if wanted.is_empty() {
        return false;
    }
    let turns = &memory.turns;
    let before = before.unwrap_or(turns.len() as i64 - 1);
    // `turns[max(0, before - LOOK_BACK + 1):before]` with Python's slice rules.
    let n = turns.len() as i64;
    let clamp = |i: i64| -> usize { (if i < 0 { (n + i).max(0) } else { i.min(n) }) as usize };
    let (from, to) = (clamp((before - int_of(&LOOK_BACK) + 1).max(0)), clamp(before));
    if from >= to {
        return false;
    }
    turns[from..to].iter().any(|turn| pt::lower(&turn.bot).contains(&wanted))
}

/// Python: bot/memory/another.py:374 `_cut(said)`
///
/// A sentence short enough to say, cut at a comma or semicolon when it is not.
pub fn _cut(said: &str) -> String {
    let said = pt::strip(said);
    if pt::len(said) as i64 <= int_of(&MOST) {
        return said.to_string();
    }
    format!("{}.", pt::rstrip_chars(&crate::compose::whole::a_whole_cut(said, int_of(&MOST)), " ,;:."))
}

/// Python: bot/memory/another.py:403 `_a_caveat_about_getting_help(part)`
///
/// True when the sentence is when to get help rather than something to try.
pub fn _a_caveat_about_getting_help(part: &str) -> bool {
    if RED_FLAG.match_(part).is_some() {
        return true;
    }
    if !(A_PROFESSIONAL.is_match(part) || URGENT_LINE.is_match(part)) {
        return false;
    }
    let left = A_PROFESSIONAL.sub(" ", &URGENT_LINE.sub(" ", part));
    let left = FRAME_WORDS.sub(" ", &left);
    !is_a_step(&left)
}

/// Python: bot/memory/another.py:424 `is_a_step(part)`
///
/// True when the sentence tells the person to do something, not just how long it takes.
pub fn is_a_step(part: &str) -> bool {
    DO_THIS.is_match(&LENGTH_IDIOM.sub(" ", part))
}

/// Python: bot/memory/another.py:429 `_the_rest(said, memory, ruled_out=(), at=None)`
///
/// The part of the answer the person has not been handed back yet, or "" when there is none.
/// (PyBot's loop rebinds `at` to the index of the sentence given, so the `at` passed in is never
/// read; the parameter is kept for the contract.)
pub fn _the_rest(said: &str, memory: &Memory, ruled_out: &HashSet<String>, _at: Option<i64>) -> String {
    let parts = _sentences(said);
    if parts.len() < 2 {
        return String::new();
    }
    // The first sentence that tells them to do something is the one they were given.
    let Some(given) = parts.iter().position(|part| is_a_step(part) && !_a_caveat_about_getting_help(part)) else {
        return String::new();
    };
    let at = given as i64;
    let least = int_of(&LEAST) as usize;
    let is_the_rest = |part: &str| -> bool {
        let offers = OFFERS_ANOTHER.match_(part).is_some();
        !(pt::len(part) < least
            || _a_caveat_about_getting_help(part)
            || (!is_a_step(part) && !offers)
            || A_COMMENT.match_(part).is_some()
            || (ABOUT_ANOTHER.match_(part).is_some() && !offers)
            || _names_one(part, ruled_out)
            || _said_before(memory, part, Some(at)))
    };
    for here in given + 1..parts.len() {
        let mut part = parts[here].clone();
        if !is_the_rest(&part) {
            continue;
        }
        // A quotation ends the sentence without ending the step: the sentence after it comes too.
        if part.chars().last().is_some_and(|c| QUOTE_MARKS.contains(c)) {
            if let Some(next) = parts[here + 1..].iter().find(|p| is_the_rest(p)) {
                part = format!("{part} {next}");
            }
        }
        if OFFERS_ANOTHER.match_(&part).is_some() && pt::len(&part) as i64 <= int_of(&MOST) {
            // The sentence names the other option itself, so it is handed over whole.
            return pt::upper_first(&part);
        }
        return _cut(&part);
    }
    String::new()
}

/// Python: bot/memory/another.py:482 `other_option(text, memory)`
///
/// The second half of the answer they just asked for more of, or None when it is not that.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn other_option(text: &str, memory: &Memory) -> Option<String> {
    if !names_nothing(text) {
        return None;
    }
    if named::asks_a_part_of_the_trip(text, memory) {
        // A line that steers to one part of a trip asks its own question.
        return None;
    }
    let turns = &memory.turns;
    if turns.is_empty() {
        return None;
    }
    let (index, ruled_out) = _the_turn_to_lean_on(turns, memory);
    let turn = &turns[index?];
    let said = pt::strip(&turn.bot);
    let module = turn.module.as_deref().unwrap_or("");
    if !ANSWER_MODULES.iter().any(|m| m == module) && said.is_empty() {
        return None;
    }
    if said.is_empty() || pt::rstrip(said).ends_with('?') || HEDGE.is_match(said) || STOCK.is_match(said) {
        return None;
    }
    let rest = _the_rest(said, memory, &ruled_out, index.map(|i| i as i64));
    if rest.is_empty() {
        return None;
    }
    if A_PERSON.is_match(&turn.user) && !topics::words(&rest).iter().any(|w| THEM.contains(w)) {
        return None; // a sentence about the person, when the question was about someone else
    }
    if OFFERS_ANOTHER.match_(&rest).is_some() {
        // The sentence is the other option in its own words.
        return Some(rest);
    }
    Some(mem7::fmt(&mem7::pick(&LEADS, memory, text), &[("rest", &rest)]))
}
