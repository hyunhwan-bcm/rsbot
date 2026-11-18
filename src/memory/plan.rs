//! Port of `bot/memory/plan.py`: the order in an answer, kept in the chat, so "what next" moves on
//! instead of starting again.
//!
//! The steps of the answer in hand and the step the chat has reached are kept under
//! `memory["steps"]`, and a bare "what next" / "and then" gets the following step, in the answer's
//! own words and order. Nothing here writes a step of its own.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, inline_re, lists, res, sets, values};
use crate::memory::repeat;
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

res!("plan"; CARRY_ON, _OPENS_ON_THE_INSTRUCTION, _HAD_A_GO, AMOUNTS);
lists!("plan"; FIRST_LEADS, AMOUNTS_LEADS, LEADS, DONE, FINISHED, NOT_A_JOB);
sets!("plan"; _NOT_THE_THING);
values!("plan"; _COUNTS, WHILE_IT_LASTS);
inline_re! {
    WORDS = r"[a-z']+", 0;
    LONG_WORDS = r"[a-z']{4,}", 0;
    DONE_WORD = r"\b[a-z]{3,}ed\b", 0;
}
static SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.SENTENCE"));

/// Python: bot/memory/plan.py:145 `_the_thing_named(text)`
///
/// The words in a line that name a thing to be done, without the grammar around them.
pub fn _the_thing_named(text: &str) -> HashSet<String> {
    WORDS.findall_str(&pt::lower(text)).into_iter().filter(|w| pt::len(w) > 3 && !_NOT_THE_THING.contains(w)).collect()
}

/// Python: bot/memory/plan.py:151 `_tried(step, memory)`
///
/// True when a line just before this one reports having done this very thing already.
pub fn _tried(step: &str, memory: &Memory) -> bool {
    let mine = _the_thing_named(step);
    if mine.is_empty() {
        return false;
    }
    mem7::tail(&memory.turns, 3).iter().any(|turn| _HAD_A_GO.is_match(&turn.user) && !mine.is_disjoint(&_the_thing_named(&turn.user)))
}

/// Python: bot/memory/plan.py:173 `the_last_answer(memory)`
///
/// The bot's last line in this chat, or "" when there is none.
pub fn the_last_answer(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|turn| !turn.bot.is_empty()).map_or(String::new(), |turn| turn.bot.clone())
}

/// `memory["steps"]` when it is a dict.
fn the_plan(memory: &Memory) -> Option<&Value> {
    memory.get("steps").filter(|plan| plan.is_dict())
}

/// Python: bot/memory/plan.py:181 `the_answer_in_hand(memory)`
///
/// The whole answer the last line was a step of, or "" when the last line is not one of its steps.
pub fn the_answer_in_hand(memory: &Memory) -> String {
    let Some(plan) = the_plan(memory) else {
        return String::new();
    };
    let answer = pt::strip(&mem7::text_of(plan.get("answer"))).to_string();
    if answer.is_empty() {
        return String::new();
    }
    let last = pt::squash(&the_last_answer(memory));
    if last.is_empty() {
        return String::new();
    }
    if last == pt::squash(&mem7::text_of(plan.get("said"))) {
        return answer;
    }
    if _our_own_line(&last, plan) {
        answer
    } else {
        String::new()
    }
}

/// Python: bot/memory/plan.py:206 `_module_of_last(memory)`
pub fn _module_of_last(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|turn| !turn.bot.is_empty()).map_or(String::new(), |turn| turn.module.clone().unwrap_or_default())
}

/// Python: bot/memory/plan.py:214 `_last_user(memory)`
pub fn _last_user(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|turn| !turn.user.is_empty()).map_or(String::new(), |turn| turn.user.clone())
}

/// Python: bot/memory/plan.py:222 `_a_cut_at_a_colon(step)`
///
/// A step cut off at a colon with a fragment after it is the step above the colon.
pub fn _a_cut_at_a_colon(step: &str) -> String {
    let (head, sep, tail) = pt::partition(step, ": ");
    if !sep.is_empty() && mem7::word_count(tail) < 5 {
        return pt::strip(head).to_string();
    }
    step.to_string()
}

/// Python: bot/memory/plan.py:237 `steps_of(answer)`
///
/// The instructions in an answer, in the order it gives them, and only when there are two.
pub fn steps_of(answer: &str) -> Vec<String> {
    let steps: Vec<String> = repeat::the_steps(answer).iter().map(|step| _a_cut_at_a_colon(step)).filter(|step| !step.is_empty()).collect();
    let steps = _the_amounts_first(answer, steps);
    if steps.len() < 2 {
        return Vec::new();
    }
    steps
}

/// Python: bot/memory/plan.py:257 `_the_amounts_first(answer, steps)`
///
/// An answer that opens with what goes in has its first piece of work in that sentence.
pub fn _the_amounts_first(answer: &str, steps: Vec<String>) -> Vec<String> {
    let first = SENTENCE.split(pt::strip(answer)).into_iter().next().unwrap_or_default();
    let first = pt::strip(&first);
    if first.is_empty() || AMOUNTS.findall_str(first).len() < 2 {
        return steps;
    }
    let low = pt::lower(first);
    if !DONE_WORD.is_match(&low) {
        return steps; // nothing yet done with what the answer talks about
    }
    if steps.first().is_some_and(|step| low.contains(mem7::first_chars(&pt::lower(step), 40))) {
        return steps; // the reader's first step is this sentence already
    }
    let mut out = vec![pt::rstrip_chars(first, ".").to_string()];
    out.extend(steps);
    out
}

/// Python: bot/memory/plan.py:277 `_words(text)`
pub fn _words(text: &str) -> HashSet<String> {
    WORDS.findall_str(&pt::lower(text)).into_iter().filter(|w| pt::len(w) > 3).collect()
}

/// Python: bot/memory/plan.py:282 `_stems(text)`
pub fn _stems(text: &str) -> HashSet<String> {
    LONG_WORDS.findall_str(&pt::lower(text)).iter().map(|w| mem7::first_chars(w, 5).to_string()).collect()
}

/// The plan's steps as strings.
fn steps_in(plan: &Value) -> Vec<String> {
    mem7::iterate_or_empty(plan.get("steps")).iter().map(mem7::py_str).collect()
}

/// Python: bot/memory/plan.py:293 `_the_same_job(answer, plan)`
///
/// True when the last thing the bot said is still about the steps in hand.
pub fn _the_same_job(answer: &str, plan: &Value) -> bool {
    let said = _words(answer);
    !said.is_empty() && steps_in(plan).iter().any(|step| !_words(step).is_disjoint(&said))
}

/// Python: bot/memory/plan.py:311 `_the_same_topic(memory, plan)`
///
/// True while the chat is still on the subject the steps belong to.
pub fn _the_same_topic(memory: &Memory, plan: &Value) -> bool {
    let mut here = _stems(&_last_user(memory));
    here.extend(_stems(&the_last_answer(memory)));
    if here.is_empty() {
        return false;
    }
    let mut about: HashSet<String> = HashSet::new();
    for step in steps_in(plan) {
        about.extend(_stems(&step));
    }
    let topic = mem7::get(plan, "topic");
    if topic.is_dict() {
        about.extend(_stems(&mem7::py_str(mem7::get(topic, "text"))));
    }
    !here.is_disjoint(&about)
}

/// Python: bot/memory/plan.py:331 `_said_in(answer, plan)`
///
/// How far into the steps the last bot line got, so a step it already said is not said again.
pub fn _said_in(answer: &str, plan: &Value) -> i64 {
    let said = _words(answer);
    if said.is_empty() {
        return 0;
    }
    let mut reached = 0;
    for (at, step) in steps_in(plan).iter().enumerate() {
        if !_words(step).is_disjoint(&said) {
            reached = at as i64 + 1;
        }
    }
    reached
}

/// Python: bot/memory/plan.py:343 `_our_own_line(answer, plan)`
///
/// True when the last thing the bot said is a step this module handed over.
pub fn _our_own_line(answer: &str, plan: &Value) -> bool {
    let said = pt::lower(&pt::squash(answer));
    if said.is_empty() {
        return false;
    }
    steps_in(plan).iter().any(|step| {
        let flat = pt::squash(step);
        !flat.is_empty() && said.contains(&pt::lower(&flat))
    })
}

/// `plan.get(key) or 0` as an int.
fn int_field(plan: &Value, key: &str) -> i64 {
    mem7::int_or_0(plan.get(key))
}

/// `memory["topic"]` copied when it is a dict, else None.
fn the_topic(memory: &Memory) -> Value {
    match memory.get("topic") {
        Some(topic) if topic.is_dict() => topic.clone(),
        _ => Value::None,
    }
}

/// Python: bot/memory/plan.py:361 `_plan_for(memory)`
///
/// The steps in hand and the step the chat has reached, made fresh when the answer changes. When
/// the plan memory holds is carried on, it is moved on where it is kept, as PyBot's dict is.
pub fn _plan_for(memory: &mut Memory) -> Option<Value> {
    let answer = the_last_answer(memory);
    if let Some(plan) = the_plan(memory).filter(|plan| mem7::get(plan, "steps").truthy()).cloned() {
        if mem7::get(&plan, "answer").as_str() == Some(answer.as_str()) || mem7::get(&plan, "said").as_str() == Some(answer.as_str()) {
            return Some(plan);
        }
        let carried_on = if _our_own_line(&answer, &plan) {
            true // the same answer, one step further on
        } else {
            // An answer with steps of its own is a new set of steps.
            let turns = mem7::turn_count(memory);
            let fresh = steps_of(&answer);
            let same_steps = fresh.is_empty() || Value::from(fresh) == *mem7::get(&plan, "steps");
            same_steps
                && turns - int_field(&plan, "turn") < WHILE_IT_LASTS.as_int().unwrap_or(3)
                && (_the_same_topic(memory, &plan) || _the_same_job(&answer, &plan))
        };
        if carried_on {
            let at = int_field(&plan, "at").max(_said_in(&answer, &plan));
            let kept = memory.get_mut("steps").expect("the plan is held");
            kept.set_key("answer", answer.as_str());
            kept.set_key("at", at);
            return Some(kept.clone());
        }
    }
    if NOT_A_JOB.contains(&_module_of_last(memory)) {
        return None;
    }
    let steps = steps_of(&answer);
    if steps.is_empty() {
        return None;
    }
    Some(mem7::dict(vec![
        ("answer", Value::from(answer)),
        ("steps", Value::from(steps)),
        ("at", Value::Int(0)),
        ("turn", Value::from(mem7::turn_count(memory))),
        ("topic", the_topic(memory)),
    ]))
}

/// `step[0].upper() + step[1:]`.
fn up(step: &str) -> String {
    pt::upper_first(step)
}

/// Python: bot/memory/plan.py:400 `carry_on(text, memory)`
///
/// The next step of the answer in hand, for a bare "what next" or "and then". Else None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the step or None.
pub fn carry_on(text: &str, memory: &mut Memory) -> Option<String> {
    let line = pt::strip(text);
    if line.is_empty() || CARRY_ON.match_(line).is_none() {
        return None;
    }
    let plan = _plan_for(memory)?;
    // One turn, one step: a reader later in the same turn gets the answer the first one gave.
    let here = mem7::turn_count(memory);
    if *mem7::get(&plan, "at_turn") == Value::from(here) && mem7::get(&plan, "said").truthy() {
        return Some(mem7::py_str(mem7::get(&plan, "said")));
    }
    let steps = steps_in(&plan);
    let mut at = int_field(&plan, "at") as usize;
    // A step is still to be done if they have not told this chat they did it.
    while at < steps.len() && _tried(&steps[at], memory) {
        at += 1;
    }
    let answer = mem7::text_of(plan.get("answer"));
    if at >= steps.len() {
        let last = steps.last().expect("a plan has steps");
        let said = if mem7::get(&plan, "done").truthy() {
            mem7::pick(&FINISHED, memory, line)
        } else {
            mem7::fmt(&mem7::pick(&DONE, memory, line), &[("step", &mem7::low1(last))])
        };
        _record(memory, &answer, &steps, at as i64, &said);
        memory.get_mut("steps").expect("just recorded").set_key("done", true);
        return Some(said);
    }
    let step = &steps[at];
    let mut said = if _OPENS_ON_THE_INSTRUCTION.match_(step).is_some() {
        let mut said = up(step);
        if !said.ends_with(['.', '!', '?']) {
            said.push('.');
        }
        said
    } else if at == 0 {
        let leads = if AMOUNTS.match_(step).is_some() { &*AMOUNTS_LEADS } else { &*FIRST_LEADS };
        mem7::fmt(&mem7::pick(leads, memory, line), &[("step", &up(step))])
    } else {
        // Turned by the step, not picked.
        mem7::fmt(&LEADS[(at - 1) % LEADS.len()], &[("step", &up(step))])
    };
    if at == 0 {
        if let Some(count) = _COUNTS.pairs().iter().find(|(k, _)| k.as_int() == Some(steps.len() as i64)) {
            said = format!("{said} {}", count.1.str());
        }
    }
    _record(memory, &answer, &steps, at as i64 + 1, &said);
    Some(said)
}

/// Python: bot/memory/plan.py:458 `_record(memory, answer, steps, at, said="")`
///
/// Remember where the chat has got to, for the next "what next".
pub fn _record(memory: &mut Memory, answer: &str, steps: &[String], at: i64, said: &str) {
    let turn = mem7::turn_count(memory);
    let before = the_plan(memory).cloned().unwrap_or_else(Value::dict);
    let steps_value = Value::from(steps.to_vec());
    let same = *mem7::get(&before, "steps") == steps_value;
    let topic = if same && mem7::get(&before, "topic").truthy() { mem7::get(&before, "topic").clone() } else { the_topic(memory) };
    memory.set(
        "steps",
        mem7::dict(vec![
            ("answer", Value::from(answer)),
            ("steps", steps_value),
            ("at", Value::Int(at)),
            ("turn", Value::from(turn)),
            ("at_turn", Value::from(turn)),
            ("said", Value::from(said)),
            ("done", Value::Bool(mem7::get(&before, "done").truthy() && same)),
            ("topic", topic),
        ]),
    );
}
