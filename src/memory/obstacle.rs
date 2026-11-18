//! Port of `bot/memory/obstacle.py`: a stated dislike of the thing the last answer leaned on.
//!
//! "i want to lose weight" / "i hate exercise": the part of the plan that still holds is handed
//! back ("Fair enough - this part doesn't need exercise: ..."). A thing they cannot use is a limit
//! and gets the one question that decides the plan; a refused move ("i dont want to kill it")
//! gets the part of the answer that never needed it. Every step handed over is a sentence of the
//! answer already given.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, inline_re, lists, res, sets, values};
use crate::memory::{ack, detail, repeat};
use crate::py::re::{self, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

res!("obstacle"; LEAD_ORDERS, STEP_ORDERS, THE_NOUN, FRAME, FRAME_BREAK);
sets!("obstacle"; NOT_A_THING, FRAME_WORDS, STOP, NOT_A_LIMIT, A_DOING, REFUSED_VERBS, PREPOSITIONS);
lists!("obstacle"; LEADS, CANNOT_USE_ASKS, REFUSAL_LEADS, NOTHING_TO_DO_LEADS, REFUSAL_THING_LEADS, NOTHING_THING_LEADS, _ANY_MOVE);
values!("obstacle"; MOST_WORDS, STEP_WORDS, REFUSAL_WORDS, DISLIKE, REFUSAL, CANNOT_USE, THE_SAME_MOVE);
inline_re! {
    WORDS = r"[a-z']+", 0;
    MOVE_LEAD = r"^(?:be\s+|have\s+to\s+|keep\s+)", 0;
    MOVE_SOFTENER = r"[\s,]+(?:though|tho|too|either|really|honestly|at\s+all|as\s+well|any\s+more|now)$", re::I;
    NOT_ALNUM = r"[^a-z0-9]+", 0;
    A_REASON = r"\b(?:because|since|as|though|although|but|unless|if|when)\b", 0;
    CLAUSE_PAUSE = r"(?<=[,;:])\s+", 0;
    SENTENCE_END = r"(?<=[.!?])\s+", 0;
}

fn patterns(value: &'static Value) -> Vec<&'static Re> {
    value.items().iter().map(|p| p.as_re().expect("a pattern")).collect()
}
static DISLIKES: LazyLock<Vec<&'static Re>> = LazyLock::new(|| patterns(&DISLIKE));
static REFUSALS: LazyLock<Vec<&'static Re>> = LazyLock::new(|| patterns(&REFUSAL));
static CANNOT_USES: LazyLock<Vec<&'static Re>> = LazyLock::new(|| patterns(&CANNOT_USE));
/// `THE_SAME_MOVE`, in its order: (key, family).
static SAME_MOVE: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| THE_SAME_MOVE.pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());

fn int_of(v: &Value) -> usize {
    v.as_int().expect("an int global") as usize
}

/// Python: bot/memory/obstacle.py:210 `_flat(text)`
pub fn _flat(text: &str) -> String {
    pt::squash(text)
}

/// Python: bot/memory/obstacle.py:214 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    WORDS.findall_str(&pt::lower(&_flat(text)))
}

/// `understand(text, {}).get("topic_own")`.
fn topic_own(text: &str) -> Option<String> {
    let empty = Memory::new();
    crate::understanding::understand(text, Some(&empty)).topic_own.filter(|t| !t.is_empty())
}

/// The first of a list of patterns that matches `said`, as the text of its named group.
fn first_match(list: &[&Re], said: &str, group: &str) -> Option<String> {
    list.iter().find_map(|pattern| pattern.match_(said).map(|found| found.name_str(group).to_string()))
}

/// Python: bot/memory/obstacle.py:218 `the_thing(text)`
///
/// The thing the person has just said they cannot stand, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the thing or None.
pub fn the_thing(text: &str) -> Option<String> {
    let said = _flat(text);
    if said.is_empty() || said.contains('?') || mem7::word_count(&said) > int_of(&MOST_WORDS) {
        return None;
    }
    if !ack::only_a_dislike(&said, &Memory::new()) {
        return None;
    }
    let thing = first_match(&DISLIKES, &said, "thing")?;
    let thing = pt::strip_chars(&thing, " .!,;:");
    if thing.is_empty() || NOT_A_THING.contains(&pt::lower(thing)) {
        return None;
    }
    Some(thing.to_string())
}

/// Python: bot/memory/obstacle.py:270 `the_thing_they_cannot_use(text)`
///
/// The thing the person has just said they cannot do or have not got, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the thing or None.
pub fn the_thing_they_cannot_use(text: &str) -> Option<String> {
    let said = _flat(text);
    if said.is_empty() || said.contains('?') || mem7::word_count(&said) > int_of(&MOST_WORDS) {
        return None;
    }
    let thing = first_match(&CANNOT_USES, &said, "thing")?;
    let thing = pt::strip_chars(&thing, " .!,;:");
    let core = _words(thing).into_iter().filter(|w| !STOP.contains(w)).collect::<Vec<_>>().join(" ");
    if thing.is_empty() || core.is_empty() || NOT_A_THING.contains(&core) || NOT_A_LIMIT.contains(&core) {
        return None;
    }
    // A limit with a reason or a second half is not a thing the plan can be built around.
    if A_REASON.is_match(&pt::lower(thing)) {
        return None;
    }
    Some(thing.to_string())
}

/// Python: bot/memory/obstacle.py:298 `the_move(text)`
///
/// The move the person has just refused, in their own words, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the move or None.
pub fn the_move(text: &str) -> Option<String> {
    let said = _flat(text);
    if said.is_empty() || said.contains('?') || mem7::word_count(&said) > int_of(&MOST_WORDS) {
        return None;
    }
    let found = first_match(&REFUSALS, &said, "move")?;
    let found = pt::strip_chars(&found, " .!,;:").to_string();
    let r#move = MOVE_LEAD.subn("", &found, 1);
    let r#move = MOVE_SOFTENER.subn("", &r#move, 1);
    if r#move.is_empty() || _the_family(&r#move).is_empty() {
        return None;
    }
    Some(r#move)
}

/// Python: bot/memory/obstacle.py:329 `_the_family(move)`
///
/// The words the answer uses for the move the person has refused, or ().
pub fn _the_family(r#move: &str) -> Vec<String> {
    for word in _words(r#move) {
        if STOP.contains(&word) {
            continue;
        }
        if let Some((_, family)) = SAME_MOVE.iter().find(|(key, _)| word.starts_with(key.as_str())) {
            return family.clone();
        }
    }
    Vec::new()
}

/// Python: bot/memory/obstacle.py:345 `_has_family(words, family)`
pub fn _has_family<S: AsRef<str>>(words: &[String], family: &[S]) -> bool {
    words.iter().any(|w| family.iter().any(|stem| w.starts_with(stem.as_ref())))
}

/// Python: bot/memory/obstacle.py:354 `_carries_a_move(sentence, family)`
///
/// Whether this sentence names the move, by a word of the family or by a name of two words.
pub fn _carries_a_move<S: AsRef<str>>(sentence: &str, family: &[S]) -> bool {
    if _has_family(&_words(sentence), family) {
        return true;
    }
    let said = format!(" {} ", NOT_ALNUM.sub(" ", &pt::lower(&_flat(sentence))));
    family.iter().any(|stem| stem.as_ref().contains(' ') && said.contains(&format!(" {} ", stem.as_ref())))
}

/// Python: bot/memory/obstacle.py:398 `_opens_on_a_verb(move)`
///
/// True when the refusal names the doing ("kill it", "spray anything"), not the thing.
pub fn _opens_on_a_verb(r#move: &str) -> bool {
    let words = _words(r#move);
    words.len() >= 2 && REFUSED_VERBS.contains(&words[0]) && !PREPOSITIONS.contains(&words[1])
}

/// Python: bot/memory/obstacle.py:413 `_the_noun(move)`
///
/// The move as a bare noun: "the pellets" and "pellets at all" are both "pellets".
pub fn _the_noun(r#move: &str) -> String {
    pt::strip_chars(&THE_NOUN.sub("", &_flat(r#move)), " .!,;:").to_string()
}

/// Python: bot/memory/obstacle.py:418 `_a_list_under_a_frame(one)`
pub fn _a_list_under_a_frame(one: &str) -> bool {
    let said = pt::squash(one);
    if said.contains('?') || FRAME.match_(&said).is_none() {
        return false;
    }
    pt::partition(&said, ":").2.matches(',').count() >= 2
}

/// `detail._a_step(said, questions_are_steps=...)` as the step it gives, or "".
fn a_step(said: &str, questions_are_steps: bool) -> String {
    detail::_a_step(said, questions_are_steps)
}

/// `detail._first_clause(step)`.
fn first_clause(step: &str) -> String {
    detail::_first_clause(step)
}

/// Python: bot/memory/obstacle.py:427 `_a_doing(one)`
///
/// True when the sentence tells the person to do something, rather than saying what a thing is.
pub fn _a_doing(one: &str) -> bool {
    if !a_step(one, false).is_empty() {
        return true;
    }
    let Some(first) = pt::split_ref(one).first().map(|w| pt::lower(pt::strip_chars(w, ",;:.-\u{2014}\u{2013}'\""))) else {
        return false;
    };
    A_DOING.contains(&first) || _a_list_under_a_frame(one)
}

/// Python: bot/memory/obstacle.py:448 `_a_whole_step(sentence)`
///
/// The sentence as one move, capped at `REFUSAL_WORDS` words at a clause boundary.
pub fn _a_whole_step(sentence: &str) -> String {
    let said = pt::squash(sentence);
    if said.is_empty() {
        return String::new();
    }
    let most = int_of(&REFUSAL_WORDS);
    if mem7::word_count(&said) <= most {
        return if said.ends_with('.') { said } else { format!("{said}.") };
    }
    let mut cut = String::new();
    for piece in CLAUSE_PAUSE.split(&said) {
        let trial = pt::strip(&format!("{cut} {piece}")).to_string();
        if !cut.is_empty() && mem7::word_count(&trial) > most {
            break;
        }
        cut = trial;
    }
    if mem7::word_count(&cut) > most {
        cut = pt::split_ref(&cut)[..most].join(" ");
    }
    let cut = pt::strip_chars(&cut, " ,;:-");
    if cut.is_empty() {
        String::new()
    } else {
        format!("{cut}.")
    }
}

/// Python: bot/memory/obstacle.py:472 `a_move_refused(text, memory)`
///
/// The reply for a refusal of the move the last answer gave, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn a_move_refused(text: &str, memory: &mut Memory) -> Option<String> {
    let r#move = the_move(text)?;
    let family = _the_family(&r#move);
    if family.is_empty() {
        return None;
    }
    let about = topic_own(&_flat(text));
    // A refusal that is nothing but the move brings no subject of its own.
    let mine: HashSet<String> = _words(&r#move).into_iter().collect();
    let only_the_move = _words(text).iter().all(|w| mine.contains(w) || FRAME_WORDS.contains(w));
    let before = _before_this_turn(memory, text);
    for (back, asked, said) in repeat::recent_answers(&before) {
        if back > 2 {
            continue;
        }
        if back > 1 && !repeat::_about_the_same_thing(&asked, &said, memory) {
            continue;
        }
        let there = topic_own(&_flat(&asked));
        if let (Some(about), Some(there)) = (&about, &there) {
            if about != there && !only_the_move {
                continue;
            }
        }
        // Whether the answer gives the move at all decides the promise the lead can make.
        let gives_it = _carries_a_move(&said, &family);
        let mut step = String::new();
        let mut sentence = String::new();
        for one in SENTENCE_END.split(&_flat(&said)) {
            if _carries_a_move(&one, &family) {
                continue;
            }
            if !gives_it && _carries_a_move(&one, &_ANY_MOVE) {
                continue;
            }
            let piece = _a_whole_step(&one);
            if piece.is_empty() || !_a_doing(&one) {
                continue;
            }
            step = piece;
            sentence = one;
            break;
        }
        if step.is_empty() {
            continue;
        }
        if repeat::_already_repeated(&step, memory) {
            step = first_clause(&step);
            if step.is_empty() || repeat::_already_repeated(&step, memory) {
                return None;
            }
        }
        let (mut pool, said_move): (Vec<String>, String) = if _opens_on_a_verb(&r#move) {
            ((if gives_it { &*REFUSAL_LEADS } else { &*NOTHING_TO_DO_LEADS }).clone(), r#move.clone())
        } else {
            let noun = _the_noun(&r#move);
            ((if gives_it { &*REFUSAL_THING_LEADS } else { &*NOTHING_THING_LEADS }).clone(), if noun.is_empty() { r#move.clone() } else { noun })
        };
        if STEP_ORDERS.match_(&step).is_some() {
            let kept: Vec<String> = pool.iter().filter(|lead| !LEAD_ORDERS.is_match(lead)).cloned().collect();
            if !kept.is_empty() {
                pool = kept;
            }
        }
        let reply = mem7::fmt(&mem7::pick(&pool, memory, text), &[("move", &said_move), ("thing", &said_move), ("step", &step)]);
        if !sentence.is_empty() {
            repeat::_handed_back(memory, &sentence);
        }
        return Some(repeat::_handed_back(memory, &reply));
    }
    None
}

/// Python: bot/memory/obstacle.py:551 `_under_its_frame(one)`
///
/// The move inside a framed sentence, taken from the clause after its colon or dash, or "".
pub fn _under_its_frame(one: &str) -> String {
    let said = pt::squash(one);
    let Some(m) = FRAME_BREAK.search(&said) else {
        return String::new();
    };
    let head = pt::strip(&said[..m.start()]);
    let tail = pt::strip(&said[m.end()..]);
    if head.is_empty() || tail.is_empty() || mem7::word_count(head) > int_of(&REFUSAL_WORDS) {
        return String::new();
    }
    if !_a_doing(tail) {
        return String::new();
    }
    _a_whole_step(tail)
}

/// Python: bot/memory/obstacle.py:575 `_the_frame(one)`
///
/// The lead-in of a framed sentence, or "".
pub fn _the_frame(one: &str) -> String {
    let said = pt::squash(one);
    match FRAME_BREAK.search(&said) {
        Some(m) => pt::strip(&said[..m.start()]).to_string(),
        None => String::new(),
    }
}

/// Python: bot/memory/obstacle.py:581 `_steps(said)`
///
/// Every instruction in the answer, in order, as (the sentence, the step, under a frame).
pub fn _steps(said: &str) -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    for sentence in SENTENCE_END.split(&_flat(said)) {
        let step = a_step(&sentence, true);
        if !step.is_empty() {
            out.push((sentence, step, false));
            continue;
        }
        let under = _under_its_frame(&sentence);
        if !under.is_empty() {
            out.push((sentence, under, true));
        }
    }
    out
}

/// `set(_words(text)) - STOP`.
fn wanted_words(text: &str) -> HashSet<String> {
    _words(text).into_iter().filter(|w| !STOP.contains(w)).collect()
}

/// Python: bot/memory/obstacle.py:606 `_needs_it(step, thing)`
///
/// Whether this step leans on the thing the person has just ruled out.
pub fn _needs_it(step: &str, thing: &str) -> bool {
    let words: HashSet<String> = _words(step).into_iter().collect();
    wanted_words(thing).iter().any(|w| words.contains(w))
}

/// Python: bot/memory/obstacle.py:612 `_leans_on_it(said, thing, asked=None)`
///
/// Whether the answer being objected to names the thing at all, or is in the same area.
pub fn _leans_on_it(said: &str, thing: &str, asked: &str) -> bool {
    let wanted = wanted_words(thing);
    if wanted.is_empty() {
        return false;
    }
    let words: HashSet<String> = _words(said).into_iter().collect();
    if wanted.iter().any(|w| words.contains(w)) {
        return true;
    }
    match (topic_own(&_flat(thing)), topic_own(&_flat(asked))) {
        (Some(there), Some(mine)) => there == mine,
        _ => false,
    }
}

/// Python: bot/memory/obstacle.py:635 `_before_this_turn(memory, text)`
///
/// The memory as it stood before the line being answered, when this turn is already recorded.
pub fn _before_this_turn(memory: &Memory, text: &str) -> Memory {
    let mine = _flat(text);
    let mut turns = memory.turns.clone();
    while turns.last().is_some_and(|turn| _flat(&turn.user) == mine) {
        turns.pop();
    }
    let mut before = memory.clone();
    before.turns = turns;
    before
}

/// Python: bot/memory/obstacle.py:655 `_the_reply_for(text, memory, thing, limited)`
///
/// The reply for a thing the line rules out of the answer above it, or None.
pub fn _the_reply_for(text: &str, memory: &mut Memory, thing: &str, limited: bool) -> Option<String> {
    // The line's own subject, not the chat's.
    let about = topic_own(&_flat(text));
    let before = _before_this_turn(memory, text);
    for (back, asked, said) in repeat::recent_answers(&before) {
        if back > 2 {
            continue;
        }
        if back > 1 && !repeat::_about_the_same_thing(&asked, &said, memory) {
            continue;
        }
        if !limited {
            // The answer being objected to has to be about the same thing the line is about.
            let there = topic_own(&_flat(&asked));
            if let (Some(about), Some(there)) = (&about, &there) {
                if about != there {
                    continue;
                }
            }
            if !_leans_on_it(&said, thing, &asked) {
                continue;
            }
        }
        let found = _steps(&said).into_iter().find(|(_, piece, _)| !_needs_it(piece, thing));
        let Some((sentence, mut step, framed)) = found else {
            // Nothing can be handed back without the thing: for a limit the alternative is asked.
            if limited {
                let ask = mem7::fmt(&mem7::pick(&CANNOT_USE_ASKS, memory, text), &[("thing", thing), ("bare", &THE_NOUN.sub("", thing))]);
                return Some(repeat::_handed_back(memory, &ask));
            }
            continue;
        };
        if repeat::_already_repeated(&step, memory) {
            step = first_clause(&step);
            if step.is_empty() || repeat::_already_repeated(&step, memory) {
                return None;
            }
        }
        if mem7::word_count(&step) > int_of(&STEP_WORDS) {
            step = first_clause(&step);
        }
        let reply = mem7::fmt(&mem7::pick(&LEADS, memory, text), &[("thing", thing), ("step", &step)]);
        // The step under a frame counts as said; otherwise the sentence it was cut from does.
        if framed {
            repeat::_handed_back(memory, &step);
        } else if !sentence.is_empty() {
            repeat::_handed_back(memory, &sentence);
        }
        return Some(repeat::_handed_back(memory, &reply));
    }
    None
}

/// Python: bot/memory/obstacle.py:724 `answer_for(text, memory)`
///
/// The reply for a stated dislike of something the last answer leaned on, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn answer_for(text: &str, memory: &mut Memory) -> Option<String> {
    let thing = the_thing(text)?;
    _the_reply_for(text, memory, &thing, false)
}

/// Python: bot/memory/obstacle.py:734 `answer_for_a_thing_they_cannot_use(text, memory)`
///
/// The reply for a thing the person cannot use, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn answer_for_a_thing_they_cannot_use(text: &str, memory: &mut Memory) -> Option<String> {
    let thing = the_thing_they_cannot_use(text)?;
    _the_reply_for(text, memory, &thing, true)
}
