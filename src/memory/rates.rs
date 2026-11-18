//! Port of `bot/memory/rates.py`: how often a thing they do happens, said of themselves.
//!
//! "i go to the gym twice a week" / "how often was that again" -> "You go to the gym twice a
//! week." Filed like a time of day: against the doing they named, newest winning, and read back
//! in their own words. A rate said of somebody else's doing is kept apart and said back as what
//! they told the bot; a rate asked for as advice is the knowledge base's.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::mem7::{self, inline_re, res, sets};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;

res!("rates"; RATES_SAID, RATES_SUBJECT, A_BARE_COUNT, A_BARE_RATE, A_COUNT_AND_PERIOD, RATES_ADVICE, RATES_ASK, COUNTS_A_PERIOD);
sets!("rates"; RATES_FIX_LEAD, RATES_IGNORE, RATES_ASK_LEAD, RATES_FILLER_LEAD, RATES_PRONOUN, RATES_ASK_FILLER);
inline_re! {
    A_WORD = r"[a-z']+", 0;
    CLAUSES = r"[.;!?,]|\band\b|\bbut\b", 0;
    LEANS_BACK = r"\b(?:that|it|this|again|remind|said|say|told|tell)\b", 0;
    MY = r"\bmy\b", 0;
    OUR = r"\bour\b", 0;
}

/// Python: bot/memory/rates.py:115 `_the_rates(memory)`
///
/// The rates held (`memory["rates"]` when it is a list). A caller that changes one writes the
/// list back, as the Python changes the list memory holds.
pub fn _the_rates(memory: &Memory) -> Vec<Value> {
    match memory.get("rates") {
        Some(Value::List(held)) => held.to_vec(),
        _ => Vec::new(),
    }
}

/// `" ".join(str(x or "").split()).strip(" .,!;:-")`.
fn tidy(text: &str) -> String {
    pt::strip_chars(&pt::squash(text), " .,!;:-").to_string()
}

/// `int(item.get("turn") or 0)`.
fn turn_of(item: &Value) -> i64 {
    mem7::int_or_0(item.get("turn"))
}

/// Python: bot/memory/rates.py:125 `_keep_a_rate_of_a_doing(said, rate, memory)`
///
/// File a rate a line says of a doing that is not put as their own. `how` is the rate the line
/// says (`rate.group("how")`).
pub fn _keep_a_rate_of_a_doing(said: &str, how: &str, memory: &mut Memory) {
    let words = pt::split_ref(said);
    let Some(first) = words.first() else {
        return;
    };
    if RATES_ASK_LEAD.contains(*first) || RATES_ASK.is_match(said) || RATES_ADVICE.is_match(said) {
        return;
    }
    if matches!(*first, "you" | "your" | "yours" | "themselves") {
        return;
    }
    let mut what = tidy(&RATES_SAID.sub(" ", said));
    if pt::split_ref(&what).first().is_some_and(|lead| RATES_FILLER_LEAD.contains(*lead)) {
        what = tidy(&pt::split_ref(&what)[1..].join(" "));
    }
    if what.is_empty() || A_WORD.findall_str(&what).iter().all(|w| RATES_IGNORE.contains(w)) {
        return;
    }
    _keep_a_rate(memory, &what, how, said, false);
}

/// Set the rate on an item: how, said, turn (and own, when given).
fn set_the_rate(item: &mut Value, how: &str, said: &str, turn: i64) {
    item.set_key("how", how);
    item.set_key("said", said);
    item.set_key("turn", turn);
}

/// Python: bot/memory/rates.py:148 `_keep_a_rate(memory, what, how, said, own=True)`
///
/// File one rate they said of a doing, newest winning for that doing.
pub fn _keep_a_rate(memory: &mut Memory, what: &str, how: &str, said: &str, own: bool) {
    let what = tidy(what);
    let how = tidy(how);
    if what.is_empty() || how.is_empty() {
        return;
    }
    let mut held = _the_rates(memory);
    let turn = mem7::turn_count(memory);
    match held.iter_mut().find(|item| item.is_dict() && item.get("what").and_then(Value::as_str) == Some(what.as_str())) {
        Some(item) => {
            set_the_rate(item, &how, said, turn);
            item.set_key("own", own);
        }
        None => {
            let mut item = mem7::dict(vec![("what", Value::from(what.as_str()))]);
            set_the_rate(&mut item, &how, said, turn);
            item.set_key("own", own);
            held.push(item);
        }
    }
    memory.set("rates", Value::from(mem7::tail(&held, 8).to_vec()));
}

/// The index in `_the_rates` of the newest rate said a turn or two ago.
fn newest_recent_rate(memory: &Memory, within: i64) -> Option<usize> {
    let turn = mem7::turn_count(memory);
    let held = _the_rates(memory);
    (0..held.len()).rev().find(|&i| {
        let item = &held[i];
        item.is_dict() && mem7::get(item, "how").truthy() && turn - turn_of(item) <= within
    })
}

/// Python: bot/memory/rates.py:172 `_the_newest_recent_rate(memory, within=2)`
///
/// The newest rate on file said a turn or two ago, which a bare correction can be about.
pub fn _the_newest_recent_rate(memory: &Memory, within: i64) -> Option<Value> {
    newest_recent_rate(memory, within).map(|i| _the_rates(memory)[i].clone())
}

/// Python: bot/memory/rates.py:183 `_the_rate_put_right(text, memory)`
///
/// File a correction of a rate that names no doing of its own, when there is one to correct.
pub fn _the_rate_put_right(text: &str, memory: &mut Memory) -> bool {
    let said = pt::squash(text);
    if said.is_empty() {
        return false;
    }
    let count = A_BARE_COUNT.match_(&said);
    let whole = if count.is_some() { None } else { A_BARE_RATE.match_(&said) };
    if count.is_none() && whole.is_none() {
        return false;
    }
    let Some(index) = newest_recent_rate(memory, 2) else {
        // Nothing on file to put right.
        return false;
    };
    let mut held = _the_rates(memory);
    let how = if let Some(count) = &count {
        let period = match count.name("period") {
            Some(period) => period.to_string(),
            None => {
                let on_file = pt::strip(&mem7::text_of(held[index].get("how"))).to_string();
                match A_COUNT_AND_PERIOD.match_(&on_file) {
                    Some(m) => m.name_str("period").to_string(),
                    // The rate on file is not a count ("every morning"): guessing a period
                    // would invent one.
                    None => return false,
                }
            }
        };
        format!("{} times a {}", pt::lower(count.name_str("num")), pt::lower(&period))
    } else {
        pt::squash(&pt::lower(whole.as_ref().expect("one of the two matched").name_str("how")))
    };
    let turn = mem7::turn_count(memory);
    set_the_rate(&mut held[index], &how, &pt::lower(&said), turn);
    memory.set("rates", Value::from(held));
    true
}

/// `set(re.findall(r"[a-z']+", text))`.
fn word_set(text: &str) -> HashSet<String> {
    A_WORD.findall_str(text).into_iter().collect()
}

/// Python: bot/memory/rates.py:225 `keep(text, memory)`
///
/// Keep a rate a line says of a doing: "i go to the gym twice a week".
pub fn keep(text: &str, memory: &mut Memory) {
    if _the_rate_put_right(text, memory) {
        // The line was a rate on file put right and named no doing of its own.
        return;
    }
    for clause in CLAUSES.split(&pt::lower(text)) {
        let said = pt::squash(&clause);
        if said.is_empty() {
            continue;
        }
        let Some(rate) = RATES_SAID.search(&said) else {
            continue;
        };
        let how = rate.name_str("how").to_string();
        let Some(subject) = RATES_SUBJECT.match_(&said) else {
            _keep_a_rate_of_a_doing(&said, &how, memory);
            continue;
        };
        if !matches!(pt::lower(subject.name_str("subj")).as_str(), "i" | "we" | "i'm" | "im" | "i've" | "ive" | "we're" | "were") {
            _keep_a_rate_of_a_doing(&said, &how, memory);
            continue;
        }
        let rest = pt::squash(subject.name_str("rest"));
        // The doing is the rest of the clause with the rate taken out of it: "go to the gym".
        let before = match RATES_SAID.search(&rest) {
            Some(inner) => rest[..inner.start()].to_string(),
            None => String::new(),
        };
        let what = pt::squash(pt::strip_chars(&before, " .,!;:-!"));
        if A_WORD.findall_str(&what).iter().all(|w| RATES_IGNORE.contains(w)) {
            continue;
        }
        let lead = pt::split(pt::strip(subject.name_str("lead")));
        if lead.first().is_some_and(|first| RATES_FIX_LEAD.contains(first)) {
            // "actually we go three times a week now" is the rate already on file, put right.
            let mut held = _the_rates(memory);
            let turn = mem7::turn_count(memory);
            let theirs = word_set(&pt::lower(&what));
            let mut moved = false;
            for item in held.iter_mut().rev() {
                if !item.is_dict() || !mem7::get(item, "how").truthy() {
                    continue;
                }
                let mine = word_set(&pt::lower(&mem7::text_of(item.get("what"))));
                if !mine.is_disjoint(&theirs) || turn - turn_of(item) <= 2 {
                    if !mem7::get(item, "what").truthy() {
                        item.set_key("what", what.as_str());
                    }
                    set_the_rate(item, &how, &said, turn);
                    moved = true;
                    break;
                }
            }
            if moved {
                memory.set("rates", Value::from(held));
            } else {
                _keep_a_rate(memory, &what, &how, &said, true);
            }
            continue;
        }
        _keep_a_rate(memory, &what, &how, &said, true);
    }
}

/// Python: bot/memory/rates.py:323 `is_a_rate_ask(text, memory)`
///
/// Is this line only the ask for one of the rates kept here?
pub fn is_a_rate_ask(text: &str, memory: &Memory) -> bool {
    answer(text, memory).is_some_and(|reply| !reply.is_empty())
}

/// `compose.pick([0, 1], memory)`: the contract's pick takes strings, so the two options are
/// strings no reply can be - a NUL never reaches the chat - which the pick reads the way it reads
/// the two ints (never the reply just given, never one said before).
fn pick_one_of_two(memory: &Memory) -> usize {
    let options = ["\u{0}0", "\u{0}1"];
    let picked = mem7::pick(&options, memory, "");
    if picked == options[0] {
        0
    } else {
        1
    }
}

/// Python: bot/memory/rates.py:328 `answer(text, memory)`
///
/// The rate of their own doing a late ask names, said back in their own words.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn answer(text: &str, memory: &Memory) -> Option<String> {
    let held: Vec<Value> = _the_rates(memory).into_iter().filter(|item| item.is_dict() && mem7::get(item, "how").truthy()).collect();
    if held.is_empty() {
        return None;
    }
    let squashed = pt::squash(text);
    let asked = RATES_ASK.match_(&squashed)?;
    // "how often should i water them" is asking for advice about the thing.
    if RATES_ADVICE.is_match(asked.as_str()) {
        return None;
    }
    if let Some(c) = asked.name("c") {
        if !COUNTS_A_PERIOD.is_match(c) {
            // A "how many" ask with no period in it counts things, not rates.
            return None;
        }
    }
    let rest = [asked.name("rest"), asked.name("rest3"), asked.name("rest2")]
        .into_iter()
        .flatten()
        .find(|r| !r.is_empty())
        .map(pt::lower)
        .unwrap_or_default();
    let named: HashSet<String> = A_WORD
        .findall_str(&rest)
        .into_iter()
        .filter(|word| !RATES_ASK_FILLER.contains(word) && !RATES_PRONOUN.contains(word) && pt::len(word) > 2)
        .collect();
    let item = if !named.is_empty() {
        let found = held.iter().rev().find(|held_item| {
            let words: HashSet<String> = A_WORD.findall_str(&pt::lower(&mem7::text_of(held_item.get("what")))).into_iter().filter(|w| !RATES_PRONOUN.contains(w)).collect();
            !named.is_disjoint(&words)
        });
        // A doing this store holds no rate for: not its to answer.
        found?.clone()
    } else {
        let lean_back = LEANS_BACK.is_match(&pt::lower(asked.as_str()));
        if !lean_back && asked.name("b").is_none() && asked.name("c").is_none() {
            return None;
        }
        held.last().expect("not empty").clone()
    };
    let what = pt::squash(&mem7::text_of(item.get("what")));
    let how = pt::squash(&mem7::text_of(item.get("how")));
    if what.is_empty() || how.is_empty() {
        return None;
    }
    // Said back to the person, so their own possessive is ours.
    let what = OUR.sub("your", &MY.sub("your", &what));
    if item.get("own") == Some(&Value::Bool(false)) {
        // Somebody else's doing is said back as what they told the bot.
        return Some(format!("You said {what} {how}."));
    }
    if pick_one_of_two(memory) == 0 {
        Some(format!("You {what} {how}."))
    } else {
        Some(format!("{} \u{2014} you {what}.", pt::upper_first(&how)))
    }
}
