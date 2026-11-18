//! Port of `bot/modules/a_return_brings_back_the_advice.py`: a bare return to a subject ("back to
//! the noise") brings back what the chat said about it, not a question about where we were. The
//! store's reading (`memory/topics.py`, `resumed`) says which subject the line went back to; this
//! module adds the substance - the first sentence of the answer the chat gave about it.

use crate::data::{self, Value};
use crate::memory::topics;
use crate::py::re::Re;
use crate::py::text;
use crate::runtime::guarded;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.modules.a_return_brings_back_the_advice.{n}")
}

static LEADS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("LEADS")));
static A_RECEIPT: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("A_RECEIPT")));
static SUBSTANCE: LazyLock<usize> = LazyLock::new(|| data::get(&name("SUBSTANCE")).as_int().unwrap_or(6) as usize);
static SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("SENTENCE")));
/// `topics.STOPWORDS`.
static STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.STOPWORDS"));
/// `topics.ANSWER_KEPT`.
static ANSWER_KEPT: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.topics.ANSWER_KEPT").as_int().unwrap_or(4) as usize);

/// `" ".join(s.lower().split())`.
fn squashed_lower(s: &str) -> String {
    text::squash(&text::lower(s))
}

/// The first `n` words of `s`, lower case, joined by single spaces.
fn opening(s: &str, n: usize) -> String {
    text::split(&text::lower(s)).into_iter().take(n).collect::<Vec<_>>().join(" ")
}

/// Python: bot/modules/a_return_brings_back_the_advice.py:57 `_the_advice(answer)`: the sentence
/// of the earlier answer that carries the advice (two when the first is under five words), or "".
pub fn _the_advice(answer: &str) -> String {
    let said = text::squash(answer);
    if said.is_empty() {
        return String::new();
    }
    let parts: Vec<String> =
        SENTENCE.split(&said).iter().map(|part| text::strip(part).to_string()).filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return String::new();
    }
    let kept = if text::split_ref(&parts[0]).len() < 5 && parts.len() > 1 { &parts[..2] } else { &parts[..1] };
    kept.join(" ")
}

/// `{form for word in topics.keywords(topic_text) for form in (word, topics.stem(word))}` less the
/// store's stop words.
fn the_wanted_forms(topic_text: &str) -> HashSet<String> {
    let mut wanted = HashSet::new();
    for word in topics::keywords(topic_text) {
        let stemmed = topics::stem(&word);
        wanted.insert(word);
        wanted.insert(stemmed);
    }
    wanted.retain(|form| !STOPWORDS.contains(form));
    wanted
}

/// Python: bot/modules/a_return_brings_back_the_advice.py:71 `_the_answers_for(memory, topic_text)`
///
/// The kept entry answers that fit the subject best (oldest first on a tie), each one really said
/// in the last few turns.
pub fn _the_answers_for(memory: &Memory, topic_text: &str) -> Vec<String> {
    let kept: Vec<Value> = match memory.get("entry_answers") {
        Some(kept @ Value::List(_)) if !kept.items().is_empty() => kept.items().to_vec(),
        _ => vec![memory.get("entry_answer").cloned().unwrap_or(Value::None)],
    };
    let turns = &memory.turns;
    let wanted = the_wanted_forms(topic_text);
    if wanted.is_empty() {
        return Vec::new();
    }
    let recent = &turns[turns.len().saturating_sub(*ANSWER_KEPT)..];
    let mut best_score = 0;
    let mut found = Vec::new();
    for record in kept.iter().filter(|record| record.is_dict()) {
        let said = record.get("text").filter(|v| v.truthy()).map(py_str).unwrap_or_default();
        let said = text::strip(&said).to_string();
        if said.is_empty() {
            continue;
        }
        let first = opening(&said, 8);
        if !recent.iter().any(|turn| squashed_lower(&turn.bot).starts_with(&first)) {
            continue; // never really said, so it is not ours to quote
        }
        let keys: HashSet<String> = record.get("keys").map(|keys| keys.items().iter().map(py_str).collect()).unwrap_or_default();
        let mut forms = HashSet::new();
        for key in keys {
            forms.insert(topics::stem(&key));
            forms.insert(key);
        }
        let score = forms.iter().filter(|form| !STOPWORDS.contains(*form) && wanted.contains(*form)).count();
        if score > best_score {
            best_score = score;
            found = vec![said];
        } else if score > 0 && score == best_score {
            found.push(said);
        }
    }
    found
}

/// `str(value)` for the plain values memory holds.
fn py_str(value: &Value) -> String {
    match value {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::None => "None".to_string(),
        other => format!("{:?}", other.to_plain()),
    }
}

/// Python: bot/modules/a_return_brings_back_the_advice.py:118 `_the_turn_answers_for(memory, topic_text)`
///
/// The bot's replies on the turns whose line fits the subject best, oldest first.
pub fn _the_turn_answers_for(memory: &Memory, topic_text: &str) -> Vec<String> {
    let wanted = the_wanted_forms(topic_text);
    if wanted.is_empty() {
        return Vec::new();
    }
    let mut best_score = 0;
    let mut found = Vec::new();
    for turn in &memory.turns {
        let reply = text::strip(&turn.bot);
        if turn.user.is_empty() || reply.is_empty() {
            continue;
        }
        let mut forms = HashSet::new();
        for word in topics::keywords(&turn.user) {
            forms.insert(topics::stem(&word));
            forms.insert(word);
        }
        let score = forms.iter().filter(|form| wanted.contains(*form)).count();
        if score > best_score {
            best_score = score;
            found = vec![reply.to_string()];
        } else if score > 0 && score == best_score {
            found.push(reply.to_string());
        }
    }
    found
}

/// Python: bot/modules/a_return_brings_back_the_advice.py:153 `_said_recently(advice, memory)`:
/// one of the last two replies opens with the advice's first six words.
pub fn _said_recently(advice: &str, memory: &Memory) -> bool {
    let first = opening(advice, 6);
    if first.is_empty() {
        return false;
    }
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().any(|turn| squashed_lower(&turn.bot).starts_with(&first))
}

/// Python: bot/modules/a_return_brings_back_the_advice.py:172 `_has_substance(advice)`: advice,
/// not a receipt, a question or a shrug.
pub fn _has_substance(advice: &str) -> bool {
    if text::split_ref(advice).len() < *SUBSTANCE {
        return false;
    }
    if A_RECEIPT.match_(advice).is_some() {
        return false;
    }
    if text::rstrip(advice).ends_with('?') {
        return false; // a question asked back is not an answer to bring back
    }
    true
}

/// `topics.resumed(memory)`: the store's record of the subject this line went back to.
/// CONTRACT: `memory::topics::resumed` returns `()`, so no record can come back yet; the call is
/// still made so a raise is a raise.
fn resumed(memory: &Memory) -> Value {
    use crate::modules::arbiter::Reading;
    topics::resumed(memory).read()
}

/// Python: bot/modules/a_return_brings_back_the_advice.py:183 `_the_return_advice(memory)`: the
/// thing the person went back to and the advice we gave about it, or ("", "").
pub fn _the_return_advice(memory: &Memory) -> (String, String) {
    let read = guarded(|| {
        let record = resumed(memory);
        if !record.is_dict() || record.get("asks").map(Value::truthy).unwrap_or(false) {
            return None;
        }
        let thing = text::squash(&record.get("asked").filter(|v| v.truthy()).map(py_str).unwrap_or_default());
        if thing.is_empty() {
            return None;
        }
        let topic_text = record.get("text").filter(|v| v.truthy()).map(py_str).unwrap_or_default();
        let mut answers = _the_answers_for(memory, &topic_text);
        answers.extend(_the_turn_answers_for(memory, &topic_text));
        answers.push(_the_advice(&topics::entry_answer(memory, &topic_text).unwrap_or_default()));
        let mut advice = String::new();
        for answer in answers {
            let short = _the_advice(&answer);
            if _has_substance(&short) && !_said_recently(&short, memory) {
                advice = short;
                break;
            }
        }
        Some((thing, advice))
    });
    // A reader that cannot answer leaves the turn as it was.
    let Some(Some((thing, advice))) = read else { return (String::new(), String::new()) };
    if !_has_substance(&advice) {
        return (String::new(), String::new());
    }
    (thing, advice)
}

/// Python: bot/modules/a_return_brings_back_the_advice.py:209 `respond(text, memory)`: the advice
/// this chat gave about the subject the person went back to.
pub fn respond(_text: &str, memory: &mut Memory) -> Option<String> {
    let (thing, advice) = _the_return_advice(memory);
    if thing.is_empty() {
        return None;
    }
    let lead = &LEADS[memory.turns.len() % LEADS.len()];
    Some(format_lead(lead, &thing, &advice))
}

/// `lead.format(thing=thing, advice=advice)` for leads that hold only those two fields.
fn format_lead(lead: &str, thing: &str, advice: &str) -> String {
    let mut out = String::new();
    let mut rest = lead;
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        if let Some(tail) = after.strip_prefix("{thing}") {
            out.push_str(thing);
            rest = tail;
        } else if let Some(tail) = after.strip_prefix("{advice}") {
            out.push_str(advice);
            rest = tail;
        } else {
            out.push('{');
            rest = &after[1..];
        }
    }
    out.push_str(rest);
    out
}

/// Python: bot/modules/a_return_brings_back_the_advice.py:218 `CLAIMS_INSTEAD(text, memory)`: this
/// line is a bare return to a subject, and the advice is the answer to it.
#[allow(non_snake_case)]
pub fn CLAIMS_INSTEAD(_text: &str, memory: &Memory) -> bool {
    let (thing, advice) = _the_return_advice(memory);
    !thing.is_empty() && !advice.is_empty()
}
