//! Port of `bot/modules/a_limit_over_the_answer.py`: a limit of theirs, said over an answer
//! already given ("we rent the flat", "we have no car"), changes that answer - the answer's own
//! first move is said back with the limit worked into it. Only after a card from `answers` (the
//! only writer of `memory["last_answer"]`), and only when that card is about the same kind of
//! thing as the limit.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.modules.a_limit_over_the_answer.{n}")
}

static LEAD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_LEAD")));
static TAIL: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_TAIL")));
static ASKS: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_ASKS")));
static INSTRUCTION: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_INSTRUCTION")));
static DANGLING: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&name("_DANGLING")));
static JOINER: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_JOINER")));
static LEAD_INS: LazyLock<&'static Re> = LazyLock::new(|| data::re(&name("_LEAD_INS")));
static MOVE_WORDS: LazyLock<usize> = LazyLock::new(|| data::get(&name("_MOVE_WORDS")).as_int().unwrap_or(45) as usize);
static MOVE_CUT: LazyLock<usize> = LazyLock::new(|| data::get(&name("_MOVE_CUT")).as_int().unwrap_or(34) as usize);

/// `re.split(r"(?<=[.!?])\s", text)` in `_the_move`.
static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s", crate::py::re::U));
/// `re.split(r"[,;]\s", first)` in `_the_move`.
static CLAUSE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;]\s", crate::py::re::U));

/// One entry of `_KINDS`: the kind, the shapes it is said in, the words its answer would be
/// about, and the three lead sentences.
pub struct Kind {
    pub kind: String,
    pub shapes: Vec<&'static Re>,
    pub about: Vec<String>,
    pub leads: Vec<String>,
}

static KINDS: LazyLock<Vec<Kind>> = LazyLock::new(|| {
    data::get(&name("_KINDS"))
        .items()
        .iter()
        .map(|entry| {
            let items = entry.items();
            Kind {
                kind: items[0].str().to_string(),
                shapes: items[1].items().iter().filter_map(Value::as_re).collect(),
                about: items[2].strs(),
                leads: items[3].strs(),
            }
        })
        .collect()
});

/// Python: bot/modules/a_limit_over_the_answer.py:58 `_clean(text)`: the line with the tone and
/// the lead-in taken off, lower case, no end punctuation.
pub fn _clean(text: &str) -> String {
    let squashed = text::squash(&text.replace('\u{2019}', "'"));
    let mut low = text::strip_chars(&squashed, " .!,").to_string();
    for _ in 0..3 {
        let before = low.clone();
        low = LEAD.sub("", &low);
        low = TAIL.sub("", &low);
        if low == before {
            break;
        }
    }
    text::strip_chars(&text::lower(&low), " .,!").to_string()
}

/// Python: bot/modules/a_limit_over_the_answer.py:175 `_a_limit(text)`: the kind a line that is
/// only a limit says (its kind, the words its answer is about, its leads), or None.
pub fn _a_limit(text: &str) -> Option<&'static Kind> {
    let low = _clean(text);
    if low.is_empty() || ASKS.is_match(&low) {
        return None;
    }
    KINDS.iter().find(|kind| kind.shapes.iter().any(|shape| shape.match_(&low).is_some()))
}

/// Python: bot/modules/a_limit_over_the_answer.py:187 `_the_answer(memory)`
///
/// The answer in hand (`memory["last_answer"]`, written by `answers`), while the chat is still
/// about it: its turn is one of the last three. `Value::None` otherwise.
pub fn _the_answer(memory: &Memory) -> Value {
    let Some(record) = memory.get("last_answer") else { return Value::None };
    if !record.is_dict() || !matches!(record.get("reply"), Some(Value::Str(_))) {
        return Value::None;
    }
    let turns = &memory.turns;
    if turns.is_empty() {
        return Value::None;
    }
    // `isinstance(turn, int)`: a bool is an int in Python.
    let turn = match record.get("turn") {
        Some(Value::Int(turn)) => *turn,
        Some(Value::Bool(turn)) => *turn as i64,
        _ => return Value::None,
    };
    if turn < 0 || turns.len() as i64 - 1 - turn > 2 {
        return Value::None;
    }
    record.clone()
}

/// Python: bot/modules/a_limit_over_the_answer.py:210 `_what_the_answer_is_about(record)`: the
/// words the answer is about, from the record's own keys (lower case).
pub fn _what_the_answer_is_about(record: &Value) -> HashSet<String> {
    let keys = match record.get("keys") {
        Some(Value::Str(keys)) => text::split(keys),
        Some(keys @ (Value::List(_) | Value::Tuple(_) | Value::Set(_))) => keys.items().iter().map(py_str).collect(),
        _ => return HashSet::new(),
    };
    keys.iter().map(|key| text::lower(key)).collect()
}

/// `str(value)` for the plain values a record's keys hold.
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

/// Python: bot/modules/a_limit_over_the_answer.py:225 `_the_answer_is_about(record, about)`
///
/// The answer is about the same kind of thing as the limit: one of the kind's words among its
/// keys, or two of them in its prose.
pub fn _the_answer_is_about(record: &Value, about: &[String]) -> bool {
    if about.is_empty() {
        return true;
    }
    let keys = _what_the_answer_is_about(record);
    if about.iter().any(|word| keys.contains(word)) {
        return true;
    }
    let low = text::lower(record.get("reply").map(Value::str).unwrap_or(""));
    let found: HashSet<&String> = about.iter().filter(|word| low.contains(word.as_str())).collect();
    found.len() >= 2
}

/// `len(s.split())`.
fn word_count(s: &str) -> usize {
    text::split_ref(s).len()
}

/// Python: bot/modules/a_limit_over_the_answer.py:247 `_the_move(answer)`
///
/// The answer's first instruction, said back as a whole sentence (cut at a whole clause only when
/// it is too long to say back), or "" when nothing in it tells anyone to do anything.
pub fn _the_move(answer: &str) -> String {
    let squashed = text::squash(&answer.replace('\u{2014}', "- "));
    let said = LEAD_INS.sub("", &squashed);
    let sentence = match SENTENCE_END.search(&said) {
        Some(found) => said[..found.start()].to_string(),
        None => said.clone(),
    };
    let mut first = text::strip_chars(&sentence, " .;:,-").to_string();
    if first.is_empty() {
        return String::new();
    }
    // The instruction usually comes after the explanation, so the colon's tail is said back.
    if let Some((_head, tail)) = first.split_once(':') {
        if word_count(tail) >= 4 {
            first = text::strip(tail).to_string();
        }
    }
    let mut parts: Vec<String> =
        CLAUSE_END.split(&first).iter().map(|part| text::strip(part).to_string()).filter(|part| !part.is_empty()).collect();
    // A clause that opens with a verb is the instruction; the clauses before it are the why.
    let opens_on_a_verb = |part: &str| INSTRUCTION.match_(&JOINER.sub("", part)).is_some();
    if !parts.is_empty() && !opens_on_a_verb(&parts[0]) {
        match (1..parts.len()).find(|&index| opens_on_a_verb(&parts[index])) {
            Some(index) => parts = parts[index..].to_vec(),
            // No clause tells anyone to do anything: the turn is left to what answers it today.
            None => return String::new(),
        }
    }
    let whole: Vec<String> = parts.iter().map(|part| JOINER.sub("", part)).collect();
    // As much of the sentence as can be said back, but never half of a clause.
    let mut out: Vec<String> = Vec::new();
    for part in &whole {
        let mut tried = out.clone();
        tried.push(part.clone());
        if !out.is_empty() && word_count(&tried.join(" ")) > *MOVE_WORDS {
            break;
        }
        out.push(part.clone());
    }
    if out.len() > 1 && word_count(&out.join(" ")) > *MOVE_CUT {
        // A sentence too long to repeat in full: whole clauses up to the cut, and no more.
        let mut kept: Vec<String> = Vec::new();
        for part in &out {
            let mut tried = kept.clone();
            tried.push(part.clone());
            if !kept.is_empty() && word_count(&tried.join(" ")) > *MOVE_CUT {
                break;
            }
            kept.push(part.clone());
        }
        out = if kept.is_empty() { out[..1].to_vec() } else { kept };
    }
    let cut = if out.is_empty() { first.clone() } else { out.join(", ") };
    let mut cut = text::strip_chars(&cut, " ,;:-.").to_string();
    // Nothing left dangling: "check the site for the trains, buses and" is not a sentence.
    loop {
        let words = text::split(&cut);
        let Some(last) = words.last() else { break };
        if !DANGLING.contains(text::strip_chars(&text::lower(last), " ,.;:-")) {
            break;
        }
        cut = words[..words.len() - 1].join(" ");
    }
    let mut cut = text::strip_chars(&cut, " ,;:-.").to_string();
    // Never hand back half a bracket.
    if cut.matches('(').count() > cut.matches(')').count() {
        let at = cut.find('(').unwrap_or(cut.len());
        cut = text::strip_chars(&cut[..at], " .;:-, ").to_string();
    }
    let mut chars = cut.chars();
    let Some(head) = chars.next() else { return String::new() };
    format!("{}{}.", head.to_uppercase(), chars.as_str())
}

/// Python: bot/modules/a_limit_over_the_answer.py:314 `TAKES_AS_IT_COMES(text, memory)`: this
/// line is the limit itself, and there is an answer above it for the limit to change.
#[allow(non_snake_case)]
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    _a_limit(text).is_some() && _the_answer(memory).truthy()
}

/// Python: bot/modules/a_limit_over_the_answer.py:327 `observe(text, memory)`: nothing to keep.
pub fn observe(_text: &str, _memory: &mut Memory) {}

/// `lead.format(move=move, move_lower=move_lower)` for leads that hold only those two fields.
fn format_lead(lead: &str, said: &str, said_lower: &str) -> String {
    let mut out = String::new();
    let mut rest = lead;
    while let Some(at) = rest.find('{') {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        if let Some(tail) = after.strip_prefix("{move_lower}") {
            out.push_str(said_lower);
            rest = tail;
        } else if let Some(tail) = after.strip_prefix("{move}") {
            out.push_str(said);
            rest = tail;
        } else {
            out.push('{');
            rest = &after[1..];
        }
    }
    out.push_str(rest);
    out
}

/// Python: bot/modules/a_limit_over_the_answer.py:332 `respond(text, memory)`: the answer in hand
/// with the limit worked into it, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let read = _a_limit(text)?;
    let record = _the_answer(memory);
    if record.is_none() {
        return None;
    }
    let answer = record.get("reply").map(Value::str).unwrap_or("").to_string();
    let low = text::lower(&answer);
    // The answer has to be about the same kind of thing as the limit.
    if !_the_answer_is_about(&record, &read.about) {
        return None;
    }
    let said = _the_move(&answer);
    if said.is_empty() {
        return None;
    }
    // The answer already said the limit itself ("if you rent, tell the landlord").
    if read.kind == "rent" && low.contains("landlord") {
        return None;
    }
    let pick_from = memory.turns.len();
    let lead = &read.leads[pick_from % read.leads.len()];
    let mut chars = said.chars();
    let said_lower = match chars.next() {
        Some(head) => format!("{}{}", head.to_lowercase(), chars.as_str()),
        None => String::new(),
    };
    Some(text::squash(&format_lead(lead, &said, &said_lower)))
}
