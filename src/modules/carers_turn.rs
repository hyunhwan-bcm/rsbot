//! Port of `bot/modules/carers_turn.py`: the turn after a safety answer, when the person is
//! caring for somebody in trouble (they are coming round, they say it is fine) or is the one an
//! abuse thread is about (he says sorry, it was my fault, I have nowhere to go).
#![allow(non_snake_case, clippy::all)]

use super::chat::{py, py_format};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::sync::LazyLock;

macro_rules! res {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.carers_turn.", stringify!($n))));)* };
}
macro_rules! vals {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Value> = LazyLock::new(|| data::get(concat!("bot.modules.carers_turn.", stringify!($n))));)* };
}

res!(
    ROUTE_NAMED, THEY_ARE_EASING, EASING_UNDONE, THEM_ASKING, EASING_NOT_ONE, MOVE_ON,
    THEY_SAY_ITS_FINE, THEY_ARE_RECOVERING, ABUSE_ANSWER, ABUSER_ANSWER, THE_APOLOGY, THE_KINDNESS,
    THE_DRINK, THEIR_OWN_FAULT, THREAT_AGAIN, NODDING_ALONG, THE_TROUBLE_IS_OVER, NO_WAY_OUT,
    SWALLOWED_NOT_RELIEF, TOO_MUCH_TO_CARRY, THE_ANSWER_ASKED_A_QUESTION, ANSWERED,
    ABOUT_THE_THREAD, CHANGE_THE_SUBJECT, ABOUT_THE_INCIDENT, THE_PLAN_ITSELF, PACKING,
    THE_FEAR_IT_REPEATS,
);
vals!(
    CRISIS_BETTER_THEM, CRISIS_WORDS, NOT_RIGHT_AFTER_THE_WORST, NOT_OUT_OF_THE_WOODS,
    SWALLOWED_BETTER, BEFORE_THE_ANSWER, ITS_FINE, RECOVERING, AFTER_THE_KINDNESS,
    AFTER_THE_APOLOGY, IT_IS_NOT_YOUR_FAULT, THE_NEXT_ONE, NOWHERE_TO_TURN, CHANGED_SUBJECT,
    AFTER_THE_DRINK, THE_BAG, THE_LOAD_IS_TOO_MUCH, STILL_THE_SAME_THREAD,
);

/// Python: bot/modules/carers_turn.py:388 `_clean(text)`
pub fn _clean(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

/// Python: bot/modules/carers_turn.py:392 `_turns(memory)`
pub fn _turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// Python: bot/modules/carers_turn.py:399 `_the_answer_above(memory)`
pub fn _the_answer_above(memory: &Memory) -> Option<&Turn> {
    let turns = _turns(memory);
    let last = turns.last()?;
    let module = last.module.as_deref().unwrap_or("");
    if module == "safety" || module == "safety_cards" {
        return Some(last);
    }
    if module == "carers_turn" && turns[..turns.len() - 1].iter().any(|t| _an_abuse_thread(&t.bot)) {
        return Some(last);
    }
    None
}

/// Python: bot/modules/carers_turn.py:424 `_reads_as_easing(clean)`
pub fn _reads_as_easing(clean: &str) -> bool {
    if THEY_ARE_EASING.search(clean).is_none() || EASING_NOT_ONE.search(clean).is_some() {
        return false;
    }
    !(clean.contains('?') || MOVE_ON.search(clean).is_some() || THEM_ASKING.match_(clean).is_some())
}

/// Python: bot/modules/carers_turn.py:431 `_reads_as_easing_but_worse(clean)`
pub fn _reads_as_easing_but_worse(clean: &str) -> bool {
    if THEY_ARE_EASING.search(clean).is_none() || clean.contains('?') {
        return false;
    }
    EASING_UNDONE.search(clean).is_some()
}

/// Python: bot/modules/carers_turn.py:438 `_an_abuse_thread(last)`
pub fn _an_abuse_thread(last: &str) -> bool {
    let t = last.to_lowercase();
    ABUSE_ANSWER.search(&t).is_some() && ABUSER_ANSWER.search(&t).is_none()
}

/// The words for the person a pool is about (`subject`, `object`, `possessive`, `be`, `who`).
pub type Forms = Value;

/// `fallback._wrote_for_them(pool, forms)`: each line of the pool filled with the words for the
/// person (`line.format(who=, subj=, obj=, poss=, be=)`). The contract types `forms` as a list
/// of strings and cannot carry the dict, so the fill is said here, word for word as Python's.
fn wrote_for_them(pool: &Value, forms: &Forms) -> Vec<String> {
    let field = |k: &str| forms.get(k).and_then(|v| v.as_str()).unwrap_or_else(|| panic!("KeyError: {k:?}")).to_string();
    let (who, subj, obj, poss, be) = (field("who"), field("subject"), field("object"), field("possessive"), field("be"));
    pool.strs()
        .iter()
        .map(|line| py_format(line, &[("who", &who), ("subj", &subj), ("obj", &obj), ("poss", &poss), ("be", &be)]))
        .collect()
}

/// Python: bot/modules/carers_turn.py:443 `_pick(pool, forms, clean, memory=None)`
pub fn _pick(pool: &Value, forms: Option<&Forms>, clean: &str, memory: &Memory) -> Option<String> {
    let mut lines: Vec<String> = match forms.filter(|f| f.truthy()) {
        Some(forms) => wrote_for_them(pool, forms),
        None => pool.strs(),
    };
    if lines.is_empty() {
        return None;
    }
    let said = _turns(memory).last().map(|t| text::strip(&t.bot).to_string()).unwrap_or_default();
    if !said.is_empty() && lines.len() > 1 {
        let fresh: Vec<String> = lines.iter().filter(|l| text::strip(l) != said).cloned().collect();
        if !fresh.is_empty() {
            lines = fresh;
        }
    }
    let start = (crate::py::random::ord_sum(clean) as usize) % lines.len();
    Some(lines[start].clone())
}

/// Python: bot/modules/carers_turn.py:466 `_the_thread(memory, last)`
pub fn _the_thread(memory: &Memory, last: &str) -> String {
    let mut lines = vec![last.to_string()];
    for past in _turns(memory) {
        lines.push(past.user.clone());
        lines.push(past.bot.clone());
    }
    lines.iter().filter(|l| !l.is_empty()).map(|l| l.to_lowercase()).collect::<Vec<_>>().join(" ")
}

/// Python: bot/modules/carers_turn.py:475 `_the_answer_sent_them_somewhere(last)`
pub fn _the_answer_sent_them_somewhere(last: &str) -> bool {
    ROUTE_NAMED.search(&last.to_lowercase()).is_some()
}

/// Python: bot/modules/carers_turn.py:480 `_the_words_for_the_one_who_did_it(memory)`
pub fn _the_words_for_the_one_who_did_it(memory: &Memory) -> Forms {
    let said = memory.get("safety_person").map(text_of).map(|s| text::strip(&s).to_lowercase()).unwrap_or_default();
    let relation = memory
        .get("person")
        .and_then(|p| p.get("who"))
        .map(text_of)
        .map(|s| text::strip(&s).to_lowercase())
        .unwrap_or_default();
    let (subject, object, possessive, be) = if said.starts_with("she") {
        ("she", "her", "her", "is")
    } else if said.starts_with("they") {
        ("they", "them", "their", "are")
    } else {
        ("he", "him", "his", "is")
    };
    let mut forms = Value::dict();
    forms.set_key("subject", subject);
    forms.set_key("object", object);
    forms.set_key("possessive", possessive);
    forms.set_key("be", be);
    forms.set_key("who", if relation.is_empty() { object.to_string() } else { format!("your {relation}") });
    forms
}

/// `str(value or "")`.
fn text_of(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        v if !v.truthy() => String::new(),
        Value::Int(i) => i.to_string(),
        other => panic!("str() of {other:?} is not ported"),
    }
}

/// Python: bot/modules/carers_turn.py:563 `respond(text, memory)` - a runtime hook.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    let memory: &Memory = memory;
    let clean = _clean(text_);
    if clean.is_empty() {
        return None;
    }
    let above = _the_answer_above(memory)?;
    let last = above.bot.to_lowercase();
    if NODDING_ALONG.match_(&clean).is_some() || THE_TROUBLE_IS_OVER.search(&clean).is_some() {
        return None;
    }
    if CHANGE_THE_SUBJECT.search(&clean).is_some() {
        return _pick(&CHANGED_SUBJECT, None, &clean, memory);
    }
    let in_abuse = _an_abuse_thread(&last) || _turns(memory).iter().any(|t| _an_abuse_thread(&t.bot));
    let forms = if in_abuse {
        _the_words_for_the_one_who_did_it(memory)
    } else {
        let mut safety = Value::dict();
        safety.set_key("user", above.user.as_str());
        let forms = py(crate::compose::fallback::_about_somebody_else(&safety, memory));
        if !forms.truthy() {
            return None;
        }
        forms
    };
    let pick = |pool: &Value| _pick(pool, Some(&forms), &clean, memory);
    if in_abuse {
        if PACKING.search(&clean).is_some() {
            return _pick(&THE_BAG, None, &clean, memory);
        }
        if THE_PLAN_ITSELF.search(&clean).is_some() {
            return pick(data::get("bot.compose.fallback.ABUSE_AFTER"));
        }
        if THEIR_OWN_FAULT.search(&clean).is_some() {
            return pick(&IT_IS_NOT_YOUR_FAULT);
        }
        if NO_WAY_OUT.search(&clean).is_some() {
            return pick(&NOWHERE_TO_TURN);
        }
        if THREAT_AGAIN.search(&clean).is_some() {
            return pick(&THE_NEXT_ONE);
        }
        if THE_FEAR_IT_REPEATS.search(&clean).is_some() {
            return pick(&THE_NEXT_ONE);
        }
        if THE_DRINK.search(&clean).is_some() {
            return pick(&AFTER_THE_DRINK);
        }
        if THE_KINDNESS.search(&clean).is_some() {
            return pick(&AFTER_THE_KINDNESS);
        }
        if THE_APOLOGY.search(&clean).is_some() {
            return pick(&AFTER_THE_APOLOGY);
        }
        if TOO_MUCH_TO_CARRY.search(&clean).is_some() {
            return _pick(&THE_LOAD_IS_TOO_MUCH, None, &clean, memory);
        }
        if ABOUT_THE_THREAD.search(&clean).is_none() && ABOUT_THE_INCIDENT.search(&clean).is_none() {
            return None;
        }
        return pick(&STILL_THE_SAME_THREAD);
    }
    if _reads_as_easing_but_worse(&clean) {
        return pick(&NOT_RIGHT_AFTER_THE_WORST);
    }
    let asked = THE_ANSWER_ASKED_A_QUESTION.search(text::strip(&above.bot)).is_some();
    if asked && ANSWERED.match_(&clean).is_none() && _reads_as_easing(&clean) {
        return pick(&BEFORE_THE_ANSWER);
    }
    let in_crisis = CRISIS_WORDS.items().iter().any(|w| last.contains(w.str()));
    if in_crisis {
        if THEY_SAY_ITS_FINE.search(&clean).is_some() {
            return pick(&ITS_FINE);
        }
        if THEY_ARE_RECOVERING.search(&clean).is_some() {
            return pick(&RECOVERING);
        }
        if _reads_as_easing(&clean) {
            if SWALLOWED_NOT_RELIEF.search(&_the_thread(memory, &last)).is_some() {
                return pick(&SWALLOWED_BETTER);
            }
            return pick(&CRISIS_BETTER_THEM);
        }
        return None;
    }
    if _reads_as_easing(&clean) && _the_answer_sent_them_somewhere(&last) {
        if SWALLOWED_NOT_RELIEF.search(&_the_thread(memory, &last)).is_some() {
            return pick(&SWALLOWED_BETTER);
        }
        return pick(&NOT_OUT_OF_THE_WOODS);
    }
    None
}
