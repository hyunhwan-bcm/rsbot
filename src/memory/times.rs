//! Port of `bot/memory/times.py`: the hours a person's own day runs on - the times they give,
//! and the ask that wants them back.
//!
//! "i get up at six thirty" is filed against the doing, newest winning, and "what time do i get
//! up" three turns later is answered from it.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, inline_re, res, sets, values};
use crate::py::text as pt;
use crate::state::Memory;

res!("times"; ROUTINE_CLAIM, ROUTINE_SHIFT, ROUTINE_WORK_START, ROUTINE_WORK_END, ROUTINE_MORE, TIME_FIX, ROUTINE_ASK);
sets!("times"; ROUTINE_ASK_FILLER);
values!("times"; ROUTINE_SAY_BACK);
inline_re! {
    NOT_A_LETTER = r"[^a-z' ]", 0;
    BED = r"\bbed\b|\bsleep\b|\bturn in\b", 0;
    HOME = r"\bhome\b", 0;
    UP = r"\bup\b", 0;
    LEAVING = r"\b(?:leave|leaves|leaving|set off|head off|go out)\b", 0;
    STARTS = r"\b(?:start|starts|begin|begins)\b", 0;
    ENDS = r"\b(?:finish|finishes|end|ends|clock out)\b", 0;
    THE_SHIFT = r"\b(?:shift|work|job)\b", 0;
}

/// Python: bot/memory/times.py:29 `_a_routine(phrase, ask=False)`
///
/// Which part of the day a phrase is about, or "" when it is about none of them. The reading
/// is by the thing named - bed, home, work - and not by the verb. `ask` also reads the bare
/// verb a question can carry, where a statement would be too vague to keep.
pub fn _a_routine(phrase: &str, ask: bool) -> String {
    let low = pt::squash(&NOT_A_LETTER.sub(" ", &pt::lower(phrase)));
    if low.is_empty() {
        return String::new();
    }
    if BED.is_match(&low) {
        return "go to bed".into();
    }
    let work = low.contains("work") || low.contains("shift") || low.contains("job");
    if work && ROUTINE_WORK_END.is_match(&low) {
        return "finish work".into();
    }
    if work && ROUTINE_WORK_START.is_match(&low) {
        return "start work".into();
    }
    if HOME.is_match(&low) {
        return "get home".into();
    }
    if UP.is_match(&low) {
        return "get up".into();
    }
    if LEAVING.is_match(&low) {
        return "leave".into();
    }
    if ask {
        if STARTS.is_match(&low) {
            return "start work".into();
        }
        if ENDS.is_match(&low) {
            return "finish work".into();
        }
    }
    String::new()
}

/// Python: bot/memory/times.py:62 `_the_times(memory)`
///
/// The times held (`memory["times"]` when it is a list, else an empty one). The items are the
/// ones memory holds; a caller that changes one writes the list back.
pub fn _the_times(memory: &Memory) -> Vec<Value> {
    match memory.get("times") {
        Some(Value::List(held)) => held.to_vec(),
        _ => Vec::new(),
    }
}

/// `{"what": what, "at": at, "said": said, "turn": turn}` written onto an item.
fn set_the_time(item: &mut Value, at: &str, said: &str, turn: i64) {
    item.set_key("at", at);
    item.set_key("said", said);
    item.set_key("turn", turn);
}

/// Python: bot/memory/times.py:69 `_keep_a_time(memory, what, at, said)`
///
/// File one routine time, newest winning, under the one key the ask is answered from.
pub fn _keep_a_time(memory: &mut Memory, what: &str, at: &str, said: &str) {
    let at = pt::strip_chars(at, " .,!;:-");
    if what.is_empty() || at.is_empty() {
        return;
    }
    let mut held = _the_times(memory);
    let turn = mem7::turn_count(memory);
    match held.iter_mut().find(|item| item.is_dict() && item.get("what").and_then(Value::as_str) == Some(what)) {
        Some(item) => set_the_time(item, at, said, turn),
        None => {
            let mut item = Value::dict();
            item.set_key("what", what);
            set_the_time(&mut item, at, said, turn);
            held.push(item);
        }
    }
    memory.set("times", Value::from(mem7::tail(&held, 8).to_vec()));
}

/// `len(memory.get("turns") or []) - int(last.get("turn") or 0)`.
fn turns_since(memory: &Memory, last: &Value) -> i64 {
    mem7::turn_count(memory) - mem7::int_or_0(last.get("turn"))
}

/// Python: bot/memory/times.py:106 `keep(text, memory)`
///
/// Keep the times the person's day runs on, and the one they put right on the next line.
pub fn keep(text: &str, memory: &mut Memory) {
    let said = pt::squash(text);
    let low = pt::lower(&said);
    let mut filed = false;
    for m in ROUTINE_SHIFT.finditer(&low) {
        let verb = pt::lower(m.name_str("verb"));
        let start = verb.starts_with("start") || verb.starts_with("begin");
        _keep_a_time(memory, if start { "start work" } else { "finish work" }, m.name_str("at"), &said);
        filed = true;
    }
    for m in ROUTINE_CLAIM.finditer(&low) {
        let what = _a_routine(m.name_str("mid"), false);
        if !what.is_empty() {
            _keep_a_time(memory, &what, m.name_str("at"), &said);
            filed = true;
        }
    }
    if filed {
        return;
    }
    let bare = pt::strip_chars(&low, " .,!;");
    let held = _the_times(memory);
    if let (Some(more), Some(last)) = (ROUTINE_MORE.match_(bare), held.last().filter(|last| last.is_dict())) {
        if matches!(last.get("what").and_then(Value::as_str), Some("start work" | "finish work")) && turns_since(memory, last) <= 1 {
            let verb = pt::lower(more.name_str("verb"));
            let which = if verb.starts_with("end") || verb.starts_with("finish") { "finish work" } else { "start work" };
            _keep_a_time(memory, which, more.name_str("at"), &said);
            return;
        }
    }
    let mut held = _the_times(memory);
    let Some(fixed) = TIME_FIX.match_(bare) else {
        return;
    };
    let Some(last) = held.last().cloned() else {
        return;
    };
    if !last.is_dict() {
        return;
    }
    // Only a time said in the last turn but one can be the one this line puts right. A turn
    // between them that asked about that very time keeps it in hand, though: "i get up at 6" /
    // "what time do i get up" / "actually 7" is still the one time, put right after it was said.
    if turns_since(memory, &last) > 1 && answer(&mem7::last_user(memory), memory).map_or(true, |reply| reply.is_empty()) {
        return;
    }
    let was = pt::strip(&mem7::text_of(last.get("at"))).to_string();
    let at = pt::strip_chars(fixed.name_str("at"), " .,!;:").to_string();
    let turn = mem7::turn_count(memory);
    let what = last.get("what").cloned().unwrap_or(Value::None);
    // `last` is the item memory holds, so the change is made where it is kept.
    let kept = held.last_mut().expect("the list is not empty");
    set_the_time(kept, &at, &said, turn);
    memory.set("times", Value::from(held));
    memory.set(
        "fixed_time",
        mem7::dict(vec![
            ("what", what),
            ("at", Value::from(at)),
            ("from", Value::from(was)),
            ("turn", Value::from(turn)),
            ("said", Value::from(said)),
        ]),
    );
}

/// The words of the rest of an ask that are not filler.
fn the_words_asked(rest: &str) -> Vec<String> {
    pt::split(&NOT_A_LETTER.sub(" ", &pt::lower(rest))).into_iter().filter(|word| !ROUTINE_ASK_FILLER.contains(word)).collect()
}

/// Python: bot/memory/times.py:166 `answer(text, memory)`
///
/// The time of their own day the ask names, said back in the words they gave it in.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn answer(text: &str, memory: &Memory) -> Option<String> {
    let squashed = pt::squash(text);
    let asked = ROUTINE_ASK.match_(&squashed)?;
    let rest = asked.name_str("rest");
    // `{item.get("what"): item.get("at") ...}`: a later item with the same doing wins.
    let mut held: Vec<(Value, Value)> = Vec::new();
    for item in _the_times(memory) {
        let at = item.get("at").filter(|at| at.truthy());
        if let (true, Some(at)) = (item.is_dict(), at) {
            let what = item.get("what").cloned().unwrap_or(Value::None);
            match held.iter_mut().find(|(key, _)| *key == what) {
                Some(slot) => slot.1 = at.clone(),
                None => held.push((what, at.clone())),
            }
        }
    }
    if held.is_empty() {
        return None;
    }
    let time_of = |what: &str| held.iter().find(|(key, _)| key.as_str() == Some(what)).map(|(_, at)| at).filter(|at| at.truthy());
    let mut what = _a_routine(rest, true);
    if what.is_empty() {
        if THE_SHIFT.is_match(&pt::lower(rest)) {
            // "what time is my shift" names the thing and not which end of it, so both ends are
            // the answer when both are held.
            let (start_at, end_at) = (time_of("start work"), time_of("finish work"));
            if let (Some(start_at), Some(end_at)) = (start_at, end_at) {
                // str + value: anything but a str is Python's TypeError
                let (start_at, end_at) = (start_at.as_str().expect("TypeError: a time is a str"), end_at.as_str().expect("TypeError: a time is a str"));
                return Some(format!("Your shift runs from {start_at} to {end_at}."));
            }
            what = if start_at.is_some() { "start work".into() } else if end_at.is_some() { "finish work".into() } else { String::new() };
        }
        if what.is_empty() {
            return None;
        }
    }
    // A long tail names something of its own ("do i need to leave for the airport"), and that is
    // a plan, not the hour their own day runs on.
    if the_words_asked(rest).len() > 3 {
        return None;
    }
    let at = time_of(&what)?;
    let said_back = match ROUTINE_SAY_BACK.get(&what) {
        Some(frame) if frame.truthy() => frame.str(),
        _ => "You {what} at {at}.",
    };
    Some(mem7::fmt(said_back, &[("at", &mem7::py_str(at)), ("what", &what)]))
}

/// Python: bot/memory/times.py:202 `is_a_time_ask(text, memory)`
///
/// Is this line only the ask for one of the times the store keeps?
pub fn is_a_time_ask(text: &str, memory: &Memory) -> bool {
    if answer(text, memory).map_or(true, |reply| reply.is_empty()) {
        return false;
    }
    let squashed = pt::squash(text);
    let Some(asked) = ROUTINE_ASK.match_(&squashed) else {
        return false;
    };
    let rest = asked.name_str("rest");
    let mut what = _a_routine(rest, true);
    if what.is_empty() && THE_SHIFT.is_match(&pt::lower(rest)) {
        what = "start work".into();
    }
    if what.is_empty() {
        return false;
    }
    // A word the ask adds that the time itself does not name makes it a plan, and a plan is the
    // answers lane's, however well the hour fits.
    the_words_asked(rest).len() <= 2
}
