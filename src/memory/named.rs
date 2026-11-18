//! Port of `bot/memory/named.py`: a follow-up that names one thing takes the ask before it with
//! it ("and a pudding" after "what should i cook" is "ideas for a pudding"), and the other
//! readers of what a short line names: the name given, the thing a measure is asked of again, the
//! aim a length is asked of, a part of the trip, the subject's own word.

use crate::data::Value;
use crate::knowledge::entry::Entry;
use crate::memory::topics::{carried_topic as the_carried_topic, is_ack, subject as the_subject};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::memory::topics::deps;
use crate::understanding::t;
use crate::understood::Understood;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.named.";

static CONTINUES: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "CONTINUES"));
static LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LEAD"));
static QUANT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "QUANT"));
static ASKING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKING"));
static POINTS_BACK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POINTS_BACK"));
static ASKS_FOR_AN_ANSWER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASKS_FOR_AN_ANSWER"));
static VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VERBS"));
static TAKES_BACK: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TAKES_BACK"));
static ASKED_FOR_IDEAS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKED_FOR_IDEAS"));
static MOST_WORDS: LazyLock<i64> = LazyLock::new(|| t::int(M, "MOST_WORDS"));
static SAID_NOT_A_FOOD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SAID_NOT_A_FOOD"));
static A_THANK_YOU: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_THANK_YOU"));
static PREP: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PREP"));
static TAKE_BACK_LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "TAKE_BACK_LEAD"));
static TAKE_BACK_WHOLE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "TAKE_BACK_WHOLE"));
static THE_ASKING: LazyLock<Vec<&'static Re>> = LazyLock::new(|| t::res(M, "THE_ASKING"));
static THE_ASKING_TAIL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THE_ASKING_TAIL"));
static WHO_X_IS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHO_X_IS"));
static MOST_WORDS_ASKED_ABOUT: LazyLock<i64> = LazyLock::new(|| t::int(M, "MOST_WORDS_ASKED_ABOUT"));
static POINTED_AT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POINTED_AT"));
static THE_ASKING_VERB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THE_ASKING_VERB"));
static NAME_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAME_WORD"));
static IN_FRONT_OF_THE_NAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "IN_FRONT_OF_THE_NAME"));
static AFTER_THE_NAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AFTER_THE_NAME"));
static A_NAME_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_NAME_ASK"));
static JUST_A_NAME: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JUST_A_NAME"));
static NOT_A_NAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_NAME"));
static SPAN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "SPAN_WORDS"));
static NOT_GOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOT_GOT"));
static NOT_A_SHELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_SHELF"));
static PART_WORDS: LazyLock<i64> = LazyLock::new(|| t::int(M, "PART_WORDS"));
static PART_LEAD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PART_LEAD"));
static A_MEASURE_AGAIN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_MEASURE_AGAIN"));
static A_FITTING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FITTING"));
static WANTS_A_NEW_ONE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WANTS_A_NEW_ONE"));
static ASKED_THE_MEASURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKED_THE_MEASURE"));
static _THE_LIFE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_THE_LIFE"));
static NAMED_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NAMED_A_THING"));
static NOT_THE_THING_ITSELF: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_THE_THING_ITSELF"));
static A_GOAL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_GOAL"));
static GOAL_VERB: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "GOAL_VERB"));
static OUTCOME_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "OUTCOME_ASK"));
static AIM_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "AIM_VERB"));
static A_DEADLINE_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_DEADLINE_CLAUSE"));
static LENGTH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LENGTH_WORDS"));
static DISTRESS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DISTRESS"));
static ACTIVITY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ACTIVITY_WORDS"));
static LENGTH_KEY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LENGTH_KEY_WORDS"));
static A_LENGTH_STATED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_LENGTH_STATED"));
static A_PACE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PACE"));
static NO_THANKS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NO_THANKS"));
static BARE_ANOTHER: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BARE_ANOTHER"));
static VIEW_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VIEW_FRAME"));
static VIEW_MANNER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VIEW_MANNER"));
static VIEW_POSSESSIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "VIEW_POSSESSIVE"));
static FEELING_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FEELING_FILLER"));
static FEELING_SAID_AS_A_PHRASE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "FEELING_SAID_AS_A_PHRASE"));
static MEASURE_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "MEASURE_ASK"));
static PROGRESS_ASK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PROGRESS_ASK"));
static A_GERUND: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_GERUND"));
static A_DOING_NAMED: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_DOING_NAMED"));
static A_FIGURE_ON_THE_END: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FIGURE_ON_THE_END"));
static A_PLAN_TO_THE_DOING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PLAN_TO_THE_DOING"));
static A_HOW_GOOD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_HOW_GOOD"));
static PROGRESS_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PROGRESS_WORDS"));
static MEASURE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEASURE_VERBS"));
static MEASURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "MEASURE_WORDS"));
static A_FAULT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_FAULT"));
static NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_THE_THING"));
static A_GETTING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_GETTING"));
static ASPECT_ASKS: LazyLock<Vec<(HashSet<String>, String)>> = LazyLock::new(|| {
    t::value(M, "ASPECT_ASKS").items().iter().map(|p| (p.items()[0].str_set(), p.items()[1].str().to_string())).collect()
});
static FED_THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FED_THINGS"));
static ASPECT_FILLERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASPECT_FILLERS"));
static ASPECT_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "ASPECT_NOUNS"));
static IN_PROGRESS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "IN_PROGRESS"));
static THE_LENGTH_FOR_IT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "THE_LENGTH_FOR_IT"));
static LENGTH_NOT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LENGTH_NOT"));
static LENGTH_FILLERS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LENGTH_FILLERS"));
static JOURNEY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "JOURNEY"));
static BEST_TIME_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "BEST_TIME_TO_DO"));
static POINTS_AT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POINTS_AT_A_THING"));
static A_VAGUE_JOB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_VAGUE_JOB"));
static A_BARE_JOB: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_BARE_JOB"));
static NOT_THE_JOB: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_THE_JOB"));
static _AND_AT_THE_START: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_AND_AT_THE_START"));
static PLACE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_WORDS"));
static PLACE_PAIRS: LazyLock<HashSet<(String, String)>> = LazyLock::new(|| {
    t::value(M, "PLACE_PAIRS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
static LIGHT_DOING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "LIGHT_DOING"));
static PLACE_PREPOSITIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_PREPOSITIONS"));
static PLACE_ARTICLES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLACE_ARTICLES"));
static DOING_BASE: LazyLock<IndexMap<String, String>> = LazyLock::new(|| t::map(M, "DOING_BASE"));
static DOING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "DOING"));
static STATE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "STATE_WORDS"));
static A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_FIGURE"));
static A_PLACE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PLACE"));
static CARE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CARE_VERBS"));
static A_HOW_OFTEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_HOW_OFTEN"));
static CARE_NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "CARE_NOT_THE_THING"));
static WHERE_IT_STANDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHERE_IT_STANDS"));
static _NOT_THE_PRACTICE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_NOT_THE_PRACTICE"));
static PERSON_OPEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "PERSON_OPEN"));
static FACT_OPEN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "FACT_OPEN"));
static FACT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FACT_FILLER"));
static TIMES_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "TIMES_ONLY"));
static FACT_MOST_WORDS: LazyLock<i64> = LazyLock::new(|| t::int(M, "FACT_MOST_WORDS"));
static A_CHOICE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_CHOICE"));
static COUNT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COUNT_WORDS"));
static PEOPLE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PEOPLE_WORDS"));
static NOT_THE_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_THE_SUBJECT"));
static _NOT_A_PRONOUN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_NOT_A_PRONOUN"));
static _A_CONNECTIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_CONNECTIVE"));
static _A_TIME_IN_A_REPLY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_A_TIME_IN_A_REPLY"));
static A_PURPOSE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_PURPOSE"));
static NOT_A_THING_END: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "NOT_A_THING_END"));
static A_TRIP_PART: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| t::re_str_pairs(M, "A_TRIP_PART"));
static _ASK_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_ASK_FILLER"));
static _GOING_AWAY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_GOING_AWAY"));
static _LIVING_THERE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "_LIVING_THERE"));
static _A_TIME_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_TIME_WORD"));
static LIMIT: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "LIMIT"));
static IRREGULAR_PARTICIPLES: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.normalize.", "IRREGULAR_PARTICIPLES"));
static VERBISH: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "VERBISH"));
static VERB_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "VERB_FORMS"));
static FEELING: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.compose.draft.", "FEELING"));
static FEELING_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| t::strs("bot.compose.draft.", "FEELING_PHRASES"));

static WHAT_HOW_ABOUT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*(?:what|how)\s+about\b", crate::py::re::I));
static SPACE_OR_COMMA: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s,]+", 0));
static A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]", 0));
static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));
static NOT_A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]", 0));
static LOWER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static LOWER_WORDS_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));
static HOW_LONG_OR_SOON: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow\s+(?:long|soon|many\s+(?:weeks|months|days|hours|years))\b", 0));
static HOW_LONG_SOON_MANY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow\s+(?:long|soon|many)\b", crate::py::re::I));
static TO_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^to\s+", crate::py::re::I));
static THREE_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]{3}", 0));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));
static A_WORD_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"^[a-z]+\s+", 0));
static ING_DOUBLING: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^aeiou][aeiou][bdgklmnprst]$", 0));
static FIRST_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"(\w+)", 0));
static A_POINTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:it|its|them|that|this|those|these|one|ones)\b", crate::py::re::I));
static A_POINTER_WRITTEN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:it|that|this|they|them|those|these|one|ones)\b", crate::py::re::I));
static ITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:its|it's)\b", crate::py::re::I));
static IT_THEY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:it|they|them|their)\b", crate::py::re::I));
static I_OR_WE_LEAD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*(?:i|we|im|i'm|ive|i've)\s+", crate::py::re::I));
static NOT_A_LETTER_RUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z]+", 0));
static NOT_A_LETTER_OR_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9]+", 0));
static NOT_A_WORD_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z']+", 0));
static NOT_A_WORD_ANY_CASE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^A-Za-z']+", 0));
static OPENING_IN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*in\s+(.{2,40}?)(?=[,:.!?]|\s(?:the|a|an|is|it|you|we)\b)", crate::py::re::I));
static A_DETERMINER_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?i)(?:the|a|an|your|my|his|her|their|our|this|that)\b", 0));

/// What another workstream's function returned, as Python's truth (its contract type may still
/// be `()` while it is a stub).
trait PyTruth {
    fn py_truth(&self) -> bool;
}
impl PyTruth for () {
    fn py_truth(&self) -> bool {
        false
    }
}
impl PyTruth for bool {
    fn py_truth(&self) -> bool {
        *self
    }
}
impl PyTruth for String {
    fn py_truth(&self) -> bool {
        !self.is_empty()
    }
}
impl<T: PyTruth> PyTruth for Option<T> {
    fn py_truth(&self) -> bool {
        self.as_ref().is_some_and(PyTruth::py_truth)
    }
}

/// `str(x or "")` for a value.
fn vstr(v: Option<&Value>) -> String {
    match v {
        Some(Value::Str(s)) => s.to_string(),
        Some(Value::Int(i)) => i.to_string(),
        Some(Value::Float(f)) => text::float_repr(*f),
        Some(Value::Bool(true)) => "True".to_string(),
        _ => String::new(),
    }
}

/// `" ".join(str(s).lower().split())`.
fn low_squash(s: &str) -> String {
    text::squash(&s.to_lowercase())
}

/// Python's `repr()` of a value, as `str(dict)` prints it.
fn py_repr(v: &Value) -> String {
    use crate::understood::py_str_repr;
    match v {
        Value::None => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Str(s) => py_str_repr(s),
        Value::List(items) => format!("[{}]", items.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Tuple(items) if items.len() == 1 => format!("({},)", py_repr(&items[0])),
        Value::Tuple(items) => format!("({})", items.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Set(items) if items.is_empty() => "set()".to_string(),
        Value::Set(items) => format!("{{{}}}", items.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Dict(pairs) => {
            format!("{{{}}}", pairs.iter().map(|(k, v)| format!("{}: {}", py_repr(k), py_repr(v))).collect::<Vec<_>>().join(", "))
        }
        other => format!("{other:?}"),
    }
}

/// `str(memory.get(key) or "")`, with a dict or list printed the way Python prints it.
fn mem_str(memory: &Memory, key: &str) -> String {
    match memory.get(key) {
        None => String::new(),
        Some(v) if !v.truthy() => String::new(),
        Some(Value::Str(s)) => s.to_string(),
        Some(v) => py_repr(v),
    }
}

/// `_answers_the_bot(text)`: the line is a short reply to the bot's last turn.
pub fn _answers_the_bot(text_: &str) -> bool {
    is_ack(text_) && CONTINUES.match_(text_).is_none()
}

/// `_previous_ask(memory)`: the line the person put before this one, when it asked for
/// suggestions.
pub fn _previous_ask(memory: &Memory) -> String {
    let Some(last) = memory.turns.last() else { return String::new() };
    if ASKED_FOR_IDEAS.is_match(&last.user) { last.user.clone() } else { String::new() }
}

/// `takes_back(text)`: the line is the person taking something back off the record.
pub fn takes_back(text_: &str) -> bool {
    let said = low_squash(text::strip(text_));
    if said.is_empty() {
        return false;
    }
    let rest = TAKE_BACK_LEAD.sub("", &said);
    let rest = text::strip_chars(&rest, " .,;:!?");
    if rest.is_empty() {
        return false;
    }
    if TAKE_BACK_WHOLE.iter().any(|w| rest.starts_with(w.as_str())) {
        return true;
    }
    if text::split(rest).len() > 6 {
        return false;
    }
    let first = SPACE_OR_COMMA.split(rest).into_iter().next().unwrap_or_default();
    TAKES_BACK.contains(text::strip_chars(&first, ".,;:!?"))
}

/// `the_one_asked_about(text)`: the person or thing a "tell me about X" line asks about, or "".
pub fn the_one_asked_about(text_: &str) -> String {
    let squashed = text::squash(&text_.replace('\u{2019}', "'"));
    let mut said = text::strip_chars(&squashed, " ?!.,;:").to_string();
    if said.is_empty() {
        return String::new();
    }
    if let Some(m) = WHO_X_IS.match_(&said) {
        said = text::strip_chars(m.name_str("thing"), " ?!.,;:").to_string();
    }
    let mut changed = true;
    while changed {
        changed = false;
        for frame in THE_ASKING.iter() {
            let new = frame.subn("", &said, 1);
            let new = text::strip_chars(&new, " ?!.,;:").to_string();
            if !new.is_empty() && new != said {
                said = new;
                changed = true;
            }
        }
        let trimmed = THE_ASKING_TAIL.sub("", &said);
        let trimmed = text::strip_chars(&trimmed, " ?!.,;:").to_string();
        if !trimmed.is_empty() && trimmed != said {
            said = trimmed;
            changed = true;
        }
    }
    if POINTED_AT.contains(&said.to_lowercase()) {
        return said;
    }
    if text::split(&said).len() as i64 > *MOST_WORDS_ASKED_ABOUT {
        return String::new();
    }
    if THE_ASKING_VERB.match_(&said).is_some() {
        return String::new();
    }
    if text::split(&said).iter().any(|w| VERBS.contains(&w.to_lowercase())) {
        return String::new();
    }
    if !_looks_like_a_thing(&said) {
        return String::new();
    }
    said
}

/// `thing(text)`: the short noun phrase a carrying-on line names ("a pudding"), or "".
pub fn thing(text_: &str) -> String {
    let said = text::squash(text::strip(text_));
    if said.is_empty() || text::split(&said).len() as i64 > *MOST_WORDS + 4 {
        return String::new();
    }
    if ASKING.match_(&said).is_some() && WHAT_HOW_ABOUT.match_(&said).is_none() {
        return String::new();
    }
    let rest = LEAD.sub("", &said);
    let rest = QUANT.sub("", text::strip_chars(&rest, " .,;:!?"));
    let rest = text::strip_chars(&rest, " .,;:!?");
    if rest.is_empty() {
        return String::new();
    }
    let found: Vec<String> = SPACE_OR_COMMA.split(rest).into_iter().filter(|w| !w.is_empty()).collect();
    if found.len() as i64 > *MOST_WORDS {
        return String::new();
    }
    for word in &found {
        let bare = text::strip_chars(&word.to_lowercase(), ".,;:!?").to_string();
        if POINTS_BACK.contains(&bare)
            || VERBS.contains(&bare)
            || TAKES_BACK.contains(&bare)
            || ASKS_FOR_AN_ANSWER.contains(&bare)
            || ["what", "how", "why", "when"].contains(&bare.as_str())
        {
            return String::new();
        }
        if !A_LETTER.is_match(&bare) {
            return String::new();
        }
    }
    if !_looks_like_a_thing(&found.join(" ")) {
        return String::new();
    }
    found.join(" ")
}

/// `_looks_like_a_thing(phrase)`.
pub fn _looks_like_a_thing(phrase: &str) -> bool {
    let found = text::split(phrase);
    if found.is_empty() {
        return false;
    }
    if found.iter().any(|w| A_DIGIT.is_match(w)) {
        return false;
    }
    found.iter().any(|w| text::len(&NOT_A_LETTER.sub("", &w.to_lowercase())) >= 3)
}

/// `carries_a_name(text)`.
pub fn carries_a_name(text_: &str) -> bool {
    NAME_WORD.is_match(text_)
}

/// `the_name_in(value)`: the name a line gives after "called" / "named", or "".
pub fn the_name_in(value: &Value) -> String {
    let said = match value {
        Value::Str(s) => s.to_string(),
        Value::None => String::new(),
        other if !other.truthy() => String::new(),
        other => py_repr(other),
    };
    let said = text::squash(&said.replace('\u{2019}', "'"));
    if !carries_a_name(&said) {
        return String::new();
    }
    let found = text::split(&said);
    for (index, word) in found.iter().enumerate() {
        let bare = text::strip_chars(word, ".,!?;:").to_lowercase();
        if !["called", "named", "name", "names", "name's"].contains(&bare.as_str()) {
            continue;
        }
        let mut rest: Vec<String> = found[index + 1..]
            .iter()
            .filter(|one| !IN_FRONT_OF_THE_NAME.contains(&text::strip_chars(one, ".,!?;:").to_lowercase()))
            .cloned()
            .collect();
        while rest.last().is_some_and(|w| AFTER_THE_NAME.contains(&text::strip_chars(w, ".,!?;:").to_lowercase())) {
            rest.pop();
        }
        if !rest.is_empty() {
            return text::strip_chars(&rest.join(" "), " .,!?;:").to_string();
        }
        return String::new();
    }
    String::new()
}

/// `asked_for_a_name(memory)`: the bot's last real line asked what something is called.
pub fn asked_for_a_name(memory: &Memory) -> bool {
    for turn in memory.turns.iter().rev() {
        if !text::strip(&turn.bot).is_empty() {
            return A_NAME_ASK.is_match(&text::squash(&turn.bot));
        }
    }
    false
}

/// `the_name_given(text, memory=None)`: the name a line gives after the bot asked for one, or "".
pub fn the_name_given(text_: &str, memory: &Memory) -> String {
    let said = text::squash(&text_.replace('\u{2019}', "'"));
    if said.is_empty() || !asked_for_a_name(memory) {
        return String::new();
    }
    let framed = the_name_in(&Value::from(said.as_str()));
    if !framed.is_empty() {
        return framed;
    }
    let Some(found) = JUST_A_NAME.match_(&said) else { return String::new() };
    let name = found.name_str("name");
    if NOT_A_NAME.contains(&name.to_lowercase()) {
        return String::new();
    }
    name.to_string()
}

/// `_not_got(text, memory=None)`: the food a line says they have not got, or "".
pub fn _not_got(text_: &str, memory: &Memory) -> String {
    let _ = memory;
    let line = text::squash(text_);
    let Some(m) = NOT_GOT.match_(&line) else { return String::new() };
    let food = text::strip(m.name_str("food")).to_string();
    if carries_a_name(&food) {
        return String::new();
    }
    if !_looks_like_a_thing(&food) {
        return String::new();
    }
    let found = text::split(&food.to_lowercase());
    if found.iter().any(|w| NOT_A_SHELF.contains(w)) {
        return String::new();
    }
    let after = found.get(1).cloned().unwrap_or_default();
    if text::strip(m.name_str("holding")).is_empty() && IRREGULAR_PARTICIPLES.contains(&found[0]) && (after.is_empty() || SPAN_WORDS.contains(&after)) {
        return String::new();
    }
    if found.iter().any(|w| POINTS_BACK.contains(w)) || ASKING.match_(&food).is_some() {
        return String::new();
    }
    if found.iter().any(|w| SAID_NOT_A_FOOD.contains(w)) {
        return String::new();
    }
    if found.iter().any(|w| A_THANK_YOU.contains(w)) {
        return String::new();
    }
    if text::split(&food).len() as i64 > *MOST_WORDS {
        return String::new();
    }
    food
}

/// `_said_the_food(memory, food)`: the bot's last reply named the food.
pub fn _said_the_food(memory: &Memory, food: &str) -> bool {
    let said = _last_reply(memory).to_lowercase();
    let pattern = format!(r"\b{}", crate::py::re::escape(&food.to_lowercase()));
    !said.is_empty() && Re::new(&pattern, 0).is_match(&said)
}

/// `_last_reply(memory)`: the bot's last reply (`holds.last_reply`).
pub fn _last_reply(memory: &Memory) -> String {
    crate::memory::holds::last_reply(memory)
}

/// `the_thing_named(memory)`: the thing the last few lines named as theirs.
pub fn the_thing_named(memory: &Memory) -> String {
    _the_thing_they_named(memory)
}

/// `about_the_thing(text, memory, owner="")`: "the price of the boiler" for "and the price" after
/// the boiler, or "".
pub fn about_the_thing(text_: &str, memory: &Memory, owner: &str) -> String {
    let said = text::squash(text::strip(text_));
    let said = text::strip_chars(&said, " .!?").to_string();
    if said.is_empty() || text::split(&said).len() as i64 > *PART_WORDS + 5 {
        return String::new();
    }
    if ASKING.match_(&said).is_some() && WHAT_HOW_ABOUT.match_(&said).is_none() {
        return String::new();
    }
    if !follow_up(text_, memory).is_empty() {
        return String::new();
    }
    let rest = LEAD.sub("", &said);
    let rest = text::strip_chars(&rest, " .,;:!?");
    if PART_LEAD.match_(rest).is_none() {
        return String::new();
    }
    let rest = QUANT.sub("", rest);
    let rest = text::strip_chars(&rest, " .,;:!?");
    let found: Vec<String> = SPACE_OR_COMMA
        .split(rest)
        .into_iter()
        .map(|w| text::strip_chars(&w, ".,;:!?").to_string())
        .filter(|w| !w.is_empty())
        .collect();
    if found.is_empty() || found.len() as i64 > *PART_WORDS {
        return String::new();
    }
    let mut part: Vec<String> = Vec::new();
    for word in &found {
        let bare = word.to_lowercase();
        if POINTS_BACK.contains(&bare) {
            continue;
        }
        if VERBS.contains(&bare) || ["what", "how", "why", "when", "where", "which", "who"].contains(&bare.as_str()) {
            return String::new();
        }
        if part.is_empty() && (PREP.contains(&bare) || ["another", "more", "other"].contains(&bare.as_str())) {
            return String::new();
        }
        part.push(word.clone());
    }
    if part.is_empty() || !_looks_like_a_thing(&part.join(" ")) {
        return String::new();
    }
    if text::split(&part[0].to_lowercase()).iter().any(|w| w == "what" || w == "how") {
        return String::new();
    }
    let owner = text::strip(owner);
    let thing_said = if owner.is_empty() { the_subject(memory) } else { owner.to_string() };
    if thing_said.is_empty() {
        return String::new();
    }
    let said_words: HashSet<String> = text::split(&part.join(" ")).iter().map(|w| w.to_lowercase()).collect();
    let thing_words: HashSet<String> =
        text::split(&thing_said).iter().map(|w| text::strip_chars(&w.to_lowercase(), ".,;:!?").to_string()).collect();
    if said_words.iter().any(|w| thing_words.contains(w)) {
        return String::new();
    }
    format!("{} of {}", part.join(" "), thing_said)
}

/// `_the_thing_they_named(memory)`: the thing the last three lines named as theirs ("my boiler is
/// ..."), or "".
pub fn _the_thing_they_named(memory: &Memory) -> String {
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        let said = low_squash(&turn.user);
        for m in NAMED_A_THING.finditer(&said) {
            let found: Vec<String> =
                text::split(m.str(1)).into_iter().filter(|w| !NOT_THE_THING_ITSELF.contains(w) && text::len(w) > 2).collect();
            if !found.is_empty() {
                return found[found.len().saturating_sub(2)..].join(" ");
            }
        }
    }
    String::new()
}

/// `a_measure_asked_again(text, memory)`: "how long does it take to fit a new <thing>" for "and a
/// new one fitted?", or "".
pub fn a_measure_asked_again(text_: &str, memory: &Memory) -> String {
    let said = text::squash(text::strip(text_));
    let said = text::strip_chars(&said, " .!?").to_string();
    let n = text::split(&said).len();
    if said.is_empty() || n > 12 {
        return String::new();
    }
    if A_MEASURE_AGAIN.match_(&said).is_some() && n > 8 {
        return String::new();
    }
    if _THE_LIFE.is_match(&said) {
        return String::new();
    }
    if !WANTS_A_NEW_ONE.is_match(&said) {
        return String::new();
    }
    let adds_on = A_MEASURE_AGAIN.match_(&said).is_some();
    if adds_on && !(A_FITTING.is_match(&said) || ASKED_THE_MEASURE.is_match(&said)) {
        return String::new();
    }
    if !adds_on && !ASKED_THE_MEASURE.is_match(&said) {
        return String::new();
    }
    let Some(last) = memory.turns.last() else { return String::new() };
    if adds_on && !ASKED_THE_MEASURE.is_match(&low_squash(&last.user)) {
        return String::new();
    }
    let thing_ = _the_thing_they_named(memory);
    if thing_.is_empty() {
        return String::new();
    }
    format!("how long does it take to fit a new {thing_}")
}

/// `_a_length_ask(text)`: a bare "how long will it take".
pub fn _a_length_ask(text_: &str) -> bool {
    let found = LOWER_WORDS.findall_str(&low_squash(text_));
    if found.is_empty() || found.len() > 8 {
        return false;
    }
    if !HOW_LONG_OR_SOON.is_match(&found.join(" ")) {
        return false;
    }
    found.iter().all(|w| LENGTH_WORDS.contains(w))
}

/// `_a_length_ask_of_the_aim(text)`: a length ask naming only the practice for the aim.
pub fn _a_length_ask_of_the_aim(text_: &str) -> bool {
    let found = LOWER_WORDS.findall_str(&low_squash(text_));
    if found.is_empty() || found.len() > 10 {
        return false;
    }
    if !HOW_LONG_OR_SOON.is_match(&found.join(" ")) {
        return false;
    }
    found.iter().all(|w| LENGTH_WORDS.contains(w) || ACTIVITY_WORDS.contains(w) || w == "of")
}

/// `_answers_a_pace(ask, memory)`: the entry the ask reaches gives a pace.
pub fn _answers_a_pace(ask: &str, memory: &Memory) -> bool {
    match _the_entry(ask, memory) {
        Some(found) => A_PACE.is_match(&crate::memory::topics::_reply_as_str(found)),
        None => false,
    }
}

/// `_the_entry(ask, memory)`: the entry the ask reaches, read against the chat; a lookup that
/// fails is no entry.
pub fn _the_entry(ask: &str, memory: &Memory) -> Option<&'static Entry> {
    crate::runtime::guarded(|| {
        let read = crate::understanding::understand(ask, Some(memory));
        crate::knowledge::lookup::find(&read, None, None, false)
    })
    .flatten()
}

/// The lower-cased words of every alternative of an entry's key (`str(phrase).lower().split()`).
fn _key_said(found: &Entry) -> Vec<HashSet<String>> {
    let mut out = Vec::new();
    for part in &found.key {
        for phrase in crate::knowledge::lookup::alternatives(part) {
            out.push(text::split(&phrase.to_lowercase()).into_iter().collect());
        }
    }
    out
}

/// `_answers_a_length(ask, memory)`: the entry the ask reaches states a length and is keyed on
/// one.
pub fn _answers_a_length(ask: &str, memory: &Memory) -> bool {
    let Some(found) = _the_entry(ask, memory) else { return false };
    if !A_LENGTH_STATED.is_match(&crate::memory::topics::_reply_as_str(found)) {
        return false;
    }
    _key_said(found).iter().any(|said| said.iter().any(|w| LENGTH_KEY_WORDS.contains(w)))
}

/// `_the_length_entry_about(ask, memory, aim_words)`: the entry the ask reaches, when it states a
/// length and is keyed on a word of the aim.
pub fn _the_length_entry_about(ask: &str, memory: &Memory, aim_words: &[String]) -> Option<&'static Entry> {
    let found = _the_entry(ask, memory)?;
    if !A_LENGTH_STATED.is_match(&crate::memory::topics::_reply_as_str(found)) {
        return None;
    }
    let keyed: HashSet<String> = _key_said(found).into_iter().flatten().collect();
    if !aim_words.iter().any(|w| text::len(w) > 1 && keyed.contains(w)) {
        return None;
    }
    Some(found)
}

/// `_answers_a_length_about(ask, memory, aim_words)`.
pub fn _answers_a_length_about(ask: &str, memory: &Memory, aim_words: &[String]) -> bool {
    _the_length_entry_about(ask, memory, aim_words).is_some()
}

/// `_an_aim_before_the_side_question(turns, memory)`: an aim said a turn or two before a side
/// question, or "".
pub fn _an_aim_before_the_side_question(turns: &[Turn], memory: &Memory) -> String {
    let said = low_squash(&mem_str(memory, "last_answer"));
    if A_LENGTH_STATED.is_match(&said) || A_PACE.is_match(&said) {
        return String::new();
    }
    let before = &turns[..turns.len().saturating_sub(1)];
    for row in before[before.len().saturating_sub(3)..].iter().rev() {
        let line = text::squash(text::strip(&row.user));
        if !line.is_empty() && A_GOAL.match_(&line).is_some() {
            return line;
        }
    }
    String::new()
}

/// `_the_aim_and_its_length_question(text, memory)`: (the length question written out with the
/// aim, whether the aim was said against a deadline).
pub fn _the_aim_and_its_length_question(text_: &str, memory: &Memory) -> (String, bool) {
    let none = (String::new(), false);
    let lowered = low_squash(text_);
    let outcome = OUTCOME_ASK.search(&lowered);
    if !_a_length_ask(text_) && !_a_length_ask_of_the_aim(text_) && outcome.is_none() {
        return none;
    }
    let turns = &memory.turns;
    let Some(last) = turns.last() else { return none };
    let mut before = text::squash(text::strip(&last.user));
    if before.is_empty() {
        return none;
    }
    if A_GOAL.match_(&before).is_none() {
        before = _an_aim_before_the_side_question(turns, memory);
        if before.is_empty() || A_GOAL.match_(&before).is_none() {
            return none;
        }
    }
    let m = A_GOAL.match_(&before).unwrap();
    if HOW_LONG_SOON_MANY.is_match(&before) {
        return none;
    }
    let verb = GOAL_VERB.get(&m.name_str("how").to_lowercase()).cloned().unwrap_or_default();
    let what = TO_LEAD.subn("", m.name_str("what"), 0);
    let deadline = A_DEADLINE_CLAUSE.is_match(&what);
    let goal = A_DEADLINE_CLAUSE.sub("", &what);
    let goal = text::strip_chars(&goal, " .,;:-").to_string();
    let n = text::split(&goal).len();
    if !(1..=6).contains(&n) {
        return none;
    }
    if !THREE_LETTERS.is_match(&goal) {
        return none;
    }
    let goal_low = goal.to_lowercase();
    if LOWER_WORDS.findall_str(&goal_low).iter().any(|w| DISTRESS.contains(w)) {
        return none;
    }
    if let Some(outcome) = &outcome {
        let state = text::strip(&SPACES.sub(" ", outcome.name_str("state"))).to_string();
        let mut verbs: Vec<&String> = AIM_VERB.iter().collect();
        verbs.sort();
        let joined: Vec<&str> = verbs.iter().map(|s| s.as_str()).collect();
        let lead = crate::understanding::rx(&format!(r"^(?:{})\s+", joined.join("|")));
        if state != lead.sub("", &goal_low) {
            return none;
        }
    }
    let aim = text::strip(&format!("{verb}{goal}").to_lowercase()).to_string();
    let goal_words = text::split(&goal_low);
    let activity: Vec<String> = LOWER_WORDS.findall_str(&lowered).into_iter().filter(|w| ACTIVITY_WORDS.contains(w)).collect();
    if !activity.is_empty() {
        let as_it_stands = LOWER_WORDS.findall_str(&lowered).join(" ");
        if _answers_a_length_about(&as_it_stands, memory, &goal_words) {
            return (String::new(), deadline);
        }
        let said = text::strip(&last.bot).to_lowercase();
        let short_aim = A_WORD_FIRST.sub("", &aim);
        for with_the_activity in [
            format!("how long does {} for {} take", activity[0], aim),
            format!("how long does {} for {} take", activity[0], short_aim),
        ] {
            let Some(found) = _the_length_entry_about(&with_the_activity, memory, &goal_words) else { continue };
            let reply = text::strip(&crate::memory::topics::_reply_as_str(found)).to_lowercase();
            if !reply.is_empty() && said.contains(&text::slice(&reply, 0, Some(70))) {
                continue;
            }
            return (with_the_activity, deadline);
        }
    }
    let asked = format!("how long does it take to {aim}");
    if _answers_a_length(&asked, memory) {
        return (asked, deadline);
    }
    let paced = format!("how fast can i {}", format!("{verb}{goal}").to_lowercase());
    if _answers_a_pace(&paced, memory) {
        return (paced, deadline);
    }
    if _answers_a_pace(&asked, memory) {
        return (asked, deadline);
    }
    (String::new(), deadline)
}

/// `a_goal_measured(text, memory)`: "how long does it take to get fit" for "how long will it
/// take" after an aim, or "".
pub fn a_goal_measured(text_: &str, memory: &Memory) -> String {
    _the_aim_and_its_length_question(text_, memory).0
}

/// `a_length_about_an_aim_on_a_deadline(text, memory)`: the same, only when the aim was said
/// against a deadline.
pub fn a_length_about_an_aim_on_a_deadline(text_: &str, memory: &Memory) -> String {
    let (asked, deadline) = _the_aim_and_its_length_question(text_, memory);
    if deadline { asked } else { String::new() }
}

/// `_no_thanks(text)`.
pub fn _no_thanks(text_: &str) -> bool {
    NO_THANKS.match_(&text::squash(text_)).is_some()
}

/// `follow_up(text, memory)`: the ask a line that names one thing carries, or "".
pub fn follow_up(text_: &str, memory: &Memory) -> String {
    let aim = a_goal_measured(text_, memory);
    if !aim.is_empty() {
        return aim;
    }
    let again = a_measure_asked_again(text_, memory);
    if !again.is_empty() {
        return again;
    }
    let missing = _not_got(text_, memory);
    if !missing.is_empty() {
        let ask = _previous_ask(memory);
        if !ask.is_empty() {
            if _said_the_food(memory, &missing) {
                return format!("{} without {missing}", text::rstrip_chars(&ask, " .,!?"));
            }
            return String::new();
        }
        return format!("what can i make without {missing}");
    }
    if _answers_the_bot(text_) {
        return String::new();
    }
    if _no_thanks(text_) {
        return String::new();
    }
    if deps::truth("bot.compose.trouble:reads_as_trouble", deps::args1(text_), || crate::compose::trouble::reads_as_trouble(text_, None, "").py_truth()) {
        return String::new();
    }
    if BARE_ANOTHER.match_(text_).is_some() {
        return if _previous_ask(memory).is_empty() { String::new() } else { "ideas for anything else".to_string() };
    }
    let said = thing(text_);
    if said.is_empty() {
        return String::new();
    }
    if _previous_ask(memory).is_empty() {
        return String::new();
    }
    format!("ideas for {said}")
}

/// `names_nothing_to_be_about(said)`: "your honest view", "your take": no thing at all.
pub fn names_nothing_to_be_about(said: &str) -> bool {
    let found: Vec<String> = LOWER_WORDS.findall_str(&said.to_lowercase()).into_iter().filter(|w| !VIEW_POSSESSIVE.contains(w)).collect();
    !found.is_empty() && found.iter().all(|w| VIEW_FRAME.contains(w) || VIEW_MANNER.contains(w))
}

/// `_only_how_they_feel(text, memory)`: the line only says how they feel.
pub fn _only_how_they_feel(text_: &str, memory: &Memory) -> bool {
    let Some(read) = memory.understood.as_ref() else { return false };
    if read.emotion.as_deref().unwrap_or("").is_empty() {
        return false;
    }
    if memory.turns.last().is_some_and(|t_| t_.module.as_deref() == Some("answers")) {
        return false;
    }
    let mut low = low_squash(text_);
    if low.is_empty() || low.contains('?') || read.kind == "question" {
        return false;
    }
    for phrase in FEELING_PHRASES.iter().chain(FEELING_SAID_AS_A_PHRASE.iter()) {
        low = low.replace(phrase.as_str(), " ");
    }
    let found = LOWER_WORDS.findall_str(&low);
    if found.is_empty() {
        return false;
    }
    found.iter().all(|w| FEELING_FILLER.contains(w) || FEELING.contains(w))
}

/// `about_it(text, memory)`: "what about <thing>" for a line that names one thing after another
/// line, or "".
pub fn about_it(text_: &str, memory: &Memory) -> String {
    if _answers_the_bot(text_) {
        return String::new();
    }
    if deps::truth("bot.compose.trouble:reads_as_trouble", deps::args1(text_), || crate::compose::trouble::reads_as_trouble(text_, None, "").py_truth()) {
        return String::new();
    }
    if deps::truth("bot.compose.fallback:reads_as_a_judgement", deps::args1(text_), || crate::compose::fallback::reads_as_a_judgement(text_)) {
        return String::new();
    }
    if _only_how_they_feel(text_, memory) {
        return String::new();
    }
    let corrected = deps::opt_string("bot.memory.facts:without_the_correction_opener", deps::args1(text_), || Some(crate::memory::facts::without_the_correction_opener(text_))).unwrap_or_default();
    let corrected = if corrected.is_empty() { text_.to_string() } else { corrected };
    let said = thing(&corrected);
    if said.is_empty() {
        return String::new();
    }
    let Some(last) = memory.turns.last() else { return String::new() };
    let was = low_squash(&last.user);
    if was.is_empty() || was == low_squash(text_) {
        return String::new();
    }
    if names_nothing_to_be_about(&said) {
        return String::new();
    }
    format!("what about {said}")
}

/// `a_figure_on_the_doing(memory, word)`: the figure said right after the doing ("5k" in "run
/// 5k"), or "".
pub fn a_figure_on_the_doing(memory: &Memory, word: &str) -> Value {
    if word.is_empty() {
        return Value::from("");
    }
    let said = _the_plan_said(memory, word);
    if said.is_empty() {
        return Value::from("");
    }
    let found = LOWER_WORDS_DIGITS.findall_str(&said);
    let low = word.to_lowercase();
    for index in 0..found.len().saturating_sub(1) {
        if found[index] == low && A_FIGURE_ON_THE_END.match_(&found[index + 1]).is_some() {
            return Value::from(found[index + 1].as_str());
        }
    }
    Value::from("")
}

/// `is_a_progress_ask(text)`.
pub fn is_a_progress_ask(text_: &str) -> bool {
    PROGRESS_ASK.is_match(text_)
}

/// `_the_doing(word, memory)`: the doing the plan named for this word, as an -ing, or "".
pub fn _the_doing(word: &str, memory: &Memory) -> String {
    let said = _the_plan_said(memory, word);
    if said.is_empty() || word.is_empty() {
        return String::new();
    }
    let word = word.to_lowercase();
    let mut named = false;
    for found in A_DOING_NAMED.finditer(&said) {
        let doing = found.name("doing").filter(|d| !d.is_empty()).or_else(|| found.name("also")).unwrap_or("");
        if doing == word {
            named = true;
            break;
        }
        if doing.ends_with("ing") && text::rstrip_chars(&doing[..doing.len() - 3], "n") == text::rstrip_chars(&word, "n") {
            named = true;
            break;
        }
    }
    if !named {
        named = the_doing_the_plan_named(&word, memory);
    }
    if !named {
        return String::new();
    }
    _the_ing(&word)
}

/// `the_doing_the_plan_named(word, memory)`: the plan said they want to do the word.
pub fn the_doing_the_plan_named(word: &str, memory: &Memory) -> bool {
    let said = _the_plan_said(memory, word);
    if said.is_empty() || word.is_empty() {
        return false;
    }
    let pattern = A_PLAN_TO_THE_DOING.pattern().replacen("%s", &crate::py::re::escape(&word.to_lowercase()), 1);
    Re::new(&pattern, crate::py::re::I).is_match(&said)
}

/// `_the_plan_said(memory, word)`: the lines of the last four turns that say the word, and the
/// subject, joined by " | ".
pub fn _the_plan_said(memory: &Memory, word: &str) -> String {
    let turns = &memory.turns;
    let mut said: Vec<String> = Vec::new();
    for row in &turns[turns.len().saturating_sub(4)..] {
        let line = low_squash(&row.user);
        if !line.is_empty() && LOWER_WORDS_DIGITS.findall_str(&line).contains(&word.to_lowercase()) {
            said.push(line);
        }
    }
    let topic = low_squash(&the_subject(memory));
    if !topic.is_empty() {
        said.push(topic);
    }
    said.join(" | ")
}

/// `_the_ing(word)`: the -ing form of a word.
pub fn _the_ing(word: &str) -> String {
    if word.is_empty() || word.ends_with("ing") {
        return word.to_string();
    }
    let c: Vec<char> = word.chars().collect();
    if c.len() > 3 && word.ends_with('e') && !"aeiou".contains(c[c.len() - 2]) {
        return format!("{}ing", &word[..word.len() - 1]);
    }
    if c.len() <= 5 && ING_DOUBLING.is_match(word) {
        return format!("{word}{}ing", c[c.len() - 1]);
    }
    format!("{word}ing")
}

/// `_the_doing_in_hand(memory)`: what the newest line says they are doing, or "".
pub fn _the_doing_in_hand(memory: &Memory) -> String {
    for turn in memory.turns.iter().rev() {
        let said = text::squash(&turn.user);
        if said.is_empty() {
            continue;
        }
        return match IN_PROGRESS.search(&said) {
            Some(found) => text::strip_chars(found.str(1), " ,;:-").to_string(),
            None => String::new(),
        };
    }
    String::new()
}

/// `the_duration_for_the_doing(text, memory)`: "how long does it take to <doing>" for a bare
/// length ask, or "".
pub fn the_duration_for_the_doing(text_: &str, memory: &Memory) -> String {
    let said = text::squash(text_);
    let said = text::strip_chars(&said, " .!?,").to_string();
    if said.is_empty() || text::split(&said).len() > 8 {
        return String::new();
    }
    if THE_LENGTH_FOR_IT.match_(&said).is_none() || LENGTH_NOT.is_match(&said) {
        return String::new();
    }
    for word in LOWER_WORDS.findall_str(&said.to_lowercase()) {
        if MEASURE_WORDS.contains(&word) || MEASURE_VERBS.contains(&word) || POINTS_BACK.contains(&word) {
            continue;
        }
        if LENGTH_FILLERS.contains(&word) || PLACE_WORDS.contains(&word) {
            continue;
        }
        if PLACE_PREPOSITIONS.contains(&word) || PLACE_ARTICLES.contains(&word) {
            continue;
        }
        return String::new();
    }
    let doing = _the_doing_in_hand(memory);
    if doing.is_empty() || text::split(&doing).len() > 7 || JOURNEY.match_(&doing).is_some() {
        return String::new();
    }
    if _the_place_belongs_to_the_doing(&said, &doing, memory) {
        return String::new();
    }
    format!("how long does it take to {doing}")
}

/// `_the_place_belongs_to_the_doing(said, doing, memory)`: the place the line names is not of the
/// doing's kind.
pub fn _the_place_belongs_to_the_doing(said: &str, doing: &str, memory: &Memory) -> bool {
    let place = crate::understanding::understand(said, Some(memory)).subject.unwrap_or_default();
    if place.is_empty() {
        return false;
    }
    let Some(item) = crate::memory::topics::_entry(&format!("how long does it take to {doing}")) else { return false };
    let domain = item.domain.clone();
    if domain.is_empty() {
        return false;
    }
    let kind = crate::understanding::understand(&place, Some(memory)).topic.unwrap_or_default();
    if kind.is_empty() {
        return false;
    }
    let of_domain = crate::knowledge::by_topic().iter().find(|(d, _)| *d == domain).map(|(_, v)| v.as_slice()).unwrap_or(&[]);
    !of_domain.contains(&kind)
}

/// `_the_aspect_asked(low, thing)`: the ask shape for a line that only names an aspect of the
/// thing ("and the cost"), or "".
pub fn _the_aspect_asked(low: &[String], thing_: &str) -> String {
    let rest: Vec<&String> = low.iter().filter(|w| !ASPECT_FILLERS.contains(*w)).collect();
    if rest.len() != 1 {
        return String::new();
    }
    for (nouns, shape) in ASPECT_ASKS.iter() {
        if !nouns.contains(rest[0]) {
            continue;
        }
        if shape.contains("{thing}") && shape.starts_with("what do i feed") && !FED_THINGS.contains(thing_) {
            continue;
        }
        return shape.clone();
    }
    String::new()
}

/// `_the_job_before(memory)`: the job the line before names ("painting" -> "paint"), or "".
pub fn _the_job_before(memory: &Memory) -> String {
    let Some(last) = memory.turns.last() else { return String::new() };
    let said = low_squash(&last.user);
    for word in LOWER_WORDS.findall_str(&said).iter().rev() {
        if text::len(word) < 4 || NOT_THE_JOB.contains(word) {
            continue;
        }
        if word.ends_with("ing") || (word.ends_with("ed") && text::len(word) > 4) {
            let mut stem: String = if word.ends_with("ing") { word[..word.len() - 3].to_string() } else { word[..word.len() - 2].to_string() };
            let c: Vec<char> = stem.chars().collect();
            if c.len() > 3 && c[c.len() - 1] == c[c.len() - 2] && !"lsz".contains(c[c.len() - 1]) {
                stem = c[..c.len() - 1].iter().collect();
            }
            if text::len(&stem) > 2 && !NOT_THE_JOB.contains(&stem) {
                return stem;
            }
        }
    }
    if let Some(found) = A_BARE_JOB.search(&said) {
        if !NOT_THE_JOB.contains(found.str(1)) {
            return found.str(1).to_string();
        }
    }
    String::new()
}

/// `the_best_time_to_do_it(text, understood, memory=None)`: "when is the best time to <do the
/// thing>" with the pronoun filled, or "".
pub fn the_best_time_to_do_it(text_: &str, understood: &Understood, memory: &Memory) -> String {
    let said = text::squash(text_);
    let said = text::strip_chars(&said, " .!?").to_string();
    if said.is_empty() || text::split(&said).len() > 14 {
        return String::new();
    }
    let Some(found) = BEST_TIME_TO_DO.match_(&said) else { return String::new() };
    let rest_said = found.name_str("rest").to_string();
    let found_words = LOWER_WORDS.findall_str(&rest_said.to_lowercase());
    if !found_words.iter().any(|w| POINTS_AT_A_THING.contains(w)) {
        return String::new();
    }
    if found_words.iter().any(|w| !POINTS_AT_A_THING.contains(w) && !A_VAGUE_JOB.contains(w) && text::len(w) > 2) {
        return String::new();
    }
    let r = &understood.reference;
    let mut target = String::new();
    let none: Option<String> = None;
    for slot in [&r.it, &r.they, &none, &r.that, &none, &none, &none] {
        let said_target = text::squash(slot.as_deref().unwrap_or(""));
        if !said_target.is_empty() && !POINTS_AT_A_THING.contains(&said_target.to_lowercase()) {
            target = said_target;
            break;
        }
    }
    if target.is_empty() {
        return String::new();
    }
    let mut rest = rest_said.clone();
    if let Some(job) = FIRST_WORD.match_(&rest) {
        if A_VAGUE_JOB.contains(&job.str(1).to_lowercase()) {
            let before = _the_job_before(memory);
            if before.is_empty() {
                return String::new();
            }
            let after = text::strip(&rest[job.end()..]).to_string();
            let filled = A_POINTER.sub_with(&after, 1, |_| target.clone());
            return format!("when is the best time to {before} {filled}");
        }
    }
    let rest_start = found.name_span("rest").map(|s| s.0).unwrap_or(said.len());
    let head = said[..rest_start].to_string();
    rest = A_POINTER.sub_with(&rest, 1, |_| target.clone());
    format!("{head}{rest}")
}

/// `_only_the_measure(low)`: every word is a measure, a light doing or a place word.
pub fn _only_the_measure(low: &[String]) -> bool {
    low.iter().enumerate().all(|(index, word)| {
        MEASURE_WORDS.contains(word)
            || LIGHT_DOING.contains(word)
            || PLACE_WORDS.contains(word)
            || PLACE_PREPOSITIONS.contains(word)
            || PLACE_ARTICLES.contains(word)
            || (index + 1 < low.len() && PLACE_PAIRS.contains(&(word.clone(), low[index + 1].clone())))
    })
}

/// `about_the_subject(text, memory)`: a measure ask written out with the subject in it, or "".
pub fn about_the_subject(text_: &str, memory: &Memory) -> String {
    let said = text::squash(text::strip(text_));
    let said = text::strip_chars(&said, " .!?").to_string();
    let mut said = _without_and_at_the_start(&said);
    if said.is_empty() || text::split(&said).len() > 12 {
        return String::new();
    }
    let mut low: Vec<String> = text::split(&said).iter().map(|w| text::strip_chars(&w.to_lowercase(), ".,;:!?").to_string()).collect();
    let timed = the_duration_for_the_doing(&said, memory);
    if !timed.is_empty() {
        return timed;
    }
    let said_the_thing = subject_word(memory);
    let asked = _the_aspect_asked(&low, &said_the_thing);
    if !asked.is_empty() {
        return if said_the_thing.is_empty() { String::new() } else { asked.replace("{thing}", &said_the_thing) };
    }
    if !MEASURE_ASK.is_match(&said) {
        return String::new();
    }
    let place = _the_place_phrase(&low);
    if place.is_empty() && !low.iter().any(|w| POINTS_BACK.contains(w)) && !_only_the_measure(&low) {
        return String::new();
    }
    let faulty = a_faulty_thing(memory);
    while faulty && low.first().is_some_and(|w| _A_CONNECTIVE.contains(w)) {
        low.remove(0);
    }
    let bare = !low.iter().any(|w| POINTS_BACK.contains(w));
    let progress = PROGRESS_ASK.is_match(&said);
    for (index, word) in low.iter().enumerate() {
        if MEASURE_WORDS.contains(word) || POINTS_BACK.contains(word) || (bare && LIGHT_DOING.contains(word)) {
            continue;
        }
        if PLACE_WORDS.contains(word) {
            continue;
        }
        if faulty && MEASURE_VERBS.contains(word) {
            continue;
        }
        if progress && PROGRESS_WORDS.contains(word) {
            continue;
        }
        if PLACE_PREPOSITIONS.contains(word) || PLACE_ARTICLES.contains(word) {
            continue;
        }
        if index + 1 < low.len() && PLACE_PAIRS.contains(&(word.clone(), low[index + 1].clone())) {
            continue;
        }
        return String::new();
    }
    let said_the_thing = subject_word(memory);
    if !place.is_empty() {
        let mut in_hand = _the_thing_in_hand(memory, &said);
        if in_hand.is_empty() {
            in_hand = said_the_thing.clone();
        }
        if in_hand.is_empty() {
            return String::new();
        }
        return _about_the_thing_in_the_place(&low, &place, &in_hand, &said, memory);
    }
    if said_the_thing.is_empty() {
        return String::new();
    }
    let parts = text::split(&said);
    if !low.is_empty() && low.len() < parts.len() {
        said = parts[parts.len() - low.len()..].join(" ");
    }
    if !low.iter().any(|w| POINTS_BACK.contains(w)) {
        return format!("{said} {said_the_thing}");
    }
    let mut written = format!("the {said_the_thing}");
    let doing = if progress { _the_doing(&said_the_thing, memory) } else { String::new() };
    if !doing.is_empty()
        && !A_HOW_GOOD.is_match(&said)
        && !vstr(Some(&a_figure_on_the_doing(memory, &said_the_thing))).is_empty()
        && crate::understanding::rx_flags(r"\bhow\s+long\b", crate::py::re::I).is_match(&said)
    {
        return String::new();
    }
    if !doing.is_empty() || (progress && A_GERUND.match_(&said_the_thing).is_some()) {
        written = if doing.is_empty() { said_the_thing.clone() } else { doing };
    }
    A_POINTER_WRITTEN.sub_with(&said, 1, |_| written.clone())
}

/// `_without_and_at_the_start(said)`.
pub fn _without_and_at_the_start(said: &str) -> String {
    _AND_AT_THE_START.subn("", said, 1)
}

/// `_the_forms(base)`: the forms of a cooking verb.
pub fn _the_forms(base: &str) -> HashSet<String> {
    let mut forms = HashSet::new();
    for f in [base.to_string(), format!("{base}s"), format!("{base}ing"), format!("{base}ed")] {
        forms.insert(f);
    }
    let last = base.chars().last().unwrap_or(' ');
    if "aeiou".contains(last) {
        forms.insert(format!("{}ing", &base[..base.len() - last.len_utf8()]));
    } else if base.ends_with('y') {
        forms.insert(format!("{}ies", &base[..base.len() - 1]));
        forms.insert(format!("{}ied", &base[..base.len() - 1]));
    } else if !"aeiouwxy".contains(last) {
        forms.insert(format!("{base}{last}ing"));
        forms.insert(format!("{base}{last}ed"));
    }
    forms
}

/// `_the_doing_word(text)`: the cooking verb a line says, as its base, or "".
pub fn _the_doing_word(text_: &str) -> String {
    for word in text::split(text_) {
        let bare = text::strip_chars(&word.to_lowercase(), ".,;:!?").to_string();
        if let Some(base) = DOING_BASE.get(&bare) {
            return base.clone();
        }
    }
    String::new()
}

/// `_a_doing_word(text)`.
pub fn _a_doing_word(text_: &str) -> bool {
    !_the_doing_word(text_).is_empty()
}

/// `_what_the_chat_is_on(memory, this_line)`: the newest line before this one, or the topic's
/// text.
pub fn _what_the_chat_is_on(memory: &Memory, this_line: &str) -> String {
    let mine = low_squash(this_line);
    for turn in memory.turns.iter().rev() {
        let was = &turn.user;
        if text::strip(was).is_empty() || low_squash(was) == mine {
            continue;
        }
        return was.clone();
    }
    match memory.get("topic") {
        Some(record @ Value::Dict(_)) if record.get("text").is_some_and(Value::truthy) => vstr(record.get("text")),
        _ => String::new(),
    }
}

/// `_the_thing_in_hand(memory, this_line)`: the last word of the line the chat is on that names a
/// thing, or "".
pub fn _the_thing_in_hand(memory: &Memory, this_line: &str) -> String {
    let was = _what_the_chat_is_on(memory, this_line);
    for word in NOT_A_WORD_LETTER.split(&was.to_lowercase()).iter().rev().filter(|p| !p.is_empty()) {
        if text::len(word) < 3 || MEASURE_WORDS.contains(word) || PLACE_WORDS.contains(word) {
            continue;
        }
        if DOING.contains(word) || POINTS_BACK.contains(word) || PLACE_PREPOSITIONS.contains(word) {
            continue;
        }
        return word.clone();
    }
    String::new()
}

/// `_the_place_phrase(low)`: the place a line names, with its preposition and article ("in the
/// oven"), or "".
pub fn _the_place_phrase(low: &[String]) -> String {
    for (index, word) in low.iter().enumerate() {
        if !PLACE_WORDS.contains(word) {
            continue;
        }
        let mut first = index;
        if index > 0 && PLACE_PAIRS.contains(&(low[index - 1].clone(), word.clone())) {
            first = index - 1;
        }
        let mut start = first;
        while start > 0 && (PLACE_PREPOSITIONS.contains(&low[start - 1]) || PLACE_ARTICLES.contains(&low[start - 1])) {
            start -= 1;
        }
        return low[start..=index].join(" ");
    }
    String::new()
}

/// `_about_the_thing_in_the_place(low, place, said_the_thing, this_line, memory)`: "how long does
/// it take to cook <thing> in the oven", or "".
pub fn _about_the_thing_in_the_place(low: &[String], place: &str, said_the_thing: &str, this_line: &str, memory: &Memory) -> String {
    if PLACE_WORDS.contains(said_the_thing) {
        return String::new();
    }
    let doing = _the_doing_word(&_what_the_chat_is_on(memory, this_line));
    if doing.is_empty() {
        return String::new();
    }
    let mut measure: Vec<&str> = Vec::new();
    for word in low {
        if PLACE_WORDS.contains(word) || PLACE_PREPOSITIONS.contains(word) || PLACE_ARTICLES.contains(word) {
            break;
        }
        if POINTS_BACK.contains(word) {
            break;
        }
        measure.push(word);
    }
    if measure.is_empty() || (!measure.contains(&"how") && !measure.contains(&"what")) {
        return String::new();
    }
    if measure != ["how", "long"] {
        return String::new();
    }
    let written = format!("how long does it take to {doing} {said_the_thing} {place}");
    if !_the_rewrite_earns_its_place(&low.join(" "), &written, memory) {
        return String::new();
    }
    written
}

/// `_the_rewrite_earns_its_place(alone, written, memory)`: the rewritten line reaches another
/// entry than the line alone, and not the one on the screen.
pub fn _the_rewrite_earns_its_place(alone: &str, written: &str, memory: &Memory) -> bool {
    let was = crate::memory::topics::_entry(alone);
    let became = crate::memory::topics::_entry(written);
    let (Some(was), Some(became)) = (was, became) else { return false };
    if std::ptr::eq(was, became) {
        return false;
    }
    if let Some(last @ Value::Dict(_)) = memory.get("last_answer") {
        if let Some(id) = last.get("id").filter(|i| i.truthy()) {
            if *id == Value::from(became.id.as_str()) {
                return false;
            }
        }
    }
    true
}

/// `_thing_in_hand(memory)`: the words of the topic that name the thing the answer is about.
pub fn _thing_in_hand(memory: &Memory) -> Vec<String> {
    let topic = memory.get("topic").filter(|t_| matches!(t_, Value::Dict(_)));
    let said: Vec<String> = match topic.and_then(|t_| t_.get("words")) {
        Some(Value::List(l)) if !l.is_empty() => l.iter().map(|w| vstr(Some(w))).collect(),
        _ => {
            // Python returns the subject word itself here, a str and not a list, so a caller that
            // walks the things walks its letters: kept as the letters
            return subject_word(memory).chars().map(String::from).collect();
        }
    };
    let answer = memory.get("last_answer").cloned().unwrap_or(Value::None);
    let keys: Vec<String> = answer.get("keys").map(|k| k.items().iter().map(|w| vstr(Some(w)).to_lowercase()).collect()).unwrap_or_default();
    let subject_keys = _the_answers_own_subject(&answer);
    let topic_text = topic.map(|t_| vstr(t_.get("text"))).unwrap_or_default();
    let mut found: Vec<String> = Vec::new();
    for wanted in [&subject_keys, &keys] {
        for word in said.iter().rev() {
            let bare = text::strip_chars(&word.to_lowercase(), ".,;:!?").to_string();
            if text::len(&bare) < 3 || NOT_THE_THING.contains(&bare) || STATE_WORDS.contains(&bare) {
                continue;
            }
            if found.contains(&bare) {
                continue;
            }
            if _only_a_particle(&bare, &topic_text) {
                continue;
            }
            if wanted.iter().any(|key| text::slice(key, 0, Some(4)) == text::slice(&bare, 0, Some(4))) {
                found.push(bare);
            }
        }
    }
    if found.is_empty() {
        let only = subject_word(memory);
        return if only.is_empty() { Vec::new() } else { vec![only] };
    }
    found
}

/// `_only_a_particle(word, text)`: every time the topic says the word, it comes after a verb.
pub fn _only_a_particle(word: &str, text_: &str) -> bool {
    let said = format!(" {} ", low_squash(text_));
    let pattern = format!(r"\b{}\b", crate::py::re::escape(word));
    let rx = Re::new(&pattern, 0);
    let found = rx.finditer(&said);
    if found.is_empty() {
        return false;
    }
    for m in &found {
        let before = text::split(&said[..m.start()]);
        let Some(last) = before.last() else { return false };
        let last = text::strip_chars(last, ".,;:!?()");
        if !(VERBISH.contains(last) || VERB_FORMS.contains(last) || last.ends_with("ing") || last.ends_with("ed")) {
            return false;
        }
    }
    true
}

/// `_the_answers_own_subject(answer)`: the first key part of the entry the last answer came from.
pub fn _the_answers_own_subject(answer: &Value) -> Vec<String> {
    let entry_id = if matches!(answer, Value::Dict(_)) { vstr(answer.get("id")) } else { String::new() };
    if entry_id.is_empty() {
        return Vec::new();
    }
    for item in crate::knowledge::lookup::all_entries() {
        if item.id == entry_id && !item.key.is_empty() {
            return crate::knowledge::lookup::alternatives(&item.key[0]).iter().map(|p| p.to_lowercase()).collect();
        }
    }
    Vec::new()
}

/// `reported_states(text, memory)`: the line with the thing in hand put in place of its pronoun,
/// when it reports a state of the thing.
pub fn reported_states(text_: &str, memory: &Memory) -> Vec<String> {
    let said = text::squash(text::strip(text_));
    let said = text::strip_chars(&said, " .!?").to_string();
    if said.is_empty() || text::split(&said).len() > 16 {
        return Vec::new();
    }
    let low: Vec<String> = text::split(&said).iter().map(|w| text::strip_chars(&w.to_lowercase(), ".,;:!?()").to_string()).collect();
    if !low.iter().any(|w| POINTS_BACK.contains(w)) {
        return Vec::new();
    }
    if ASKING.match_(&said).is_some() || MEASURE_ASK.is_match(&said) {
        return Vec::new();
    }
    let things = _thing_in_hand(memory);
    if things.is_empty() {
        return Vec::new();
    }
    let reported = A_FIGURE.is_match(&said) || A_PLACE.is_match(&said) || low.iter().any(|w| STATE_WORDS.contains(w));
    if !reported {
        return Vec::new();
    }
    let mut reads: Vec<String> = Vec::new();
    for thing_ in &things {
        if low.contains(thing_) {
            continue;
        }
        let first = format!("my {thing_} is");
        let written = ITS.sub_with(&said, 1, |_| first.clone());
        let second = format!("my {thing_}");
        let written = IT_THEY.sub_with(&written, 1, |_| second.clone());
        if !reads.contains(&written) {
            reads.push(written);
        }
    }
    reads
}

/// `_the_persons_own_words(memory)`: the words of theirs that name the thing a care report is
/// about.
pub fn _the_persons_own_words(memory: &Memory) -> Vec<String> {
    let answer = memory.get("last_answer").cloned().unwrap_or(Value::None);
    let wanted = _the_answers_own_subject(&answer);
    let mut own: Vec<String> = Vec::new();
    for turn in memory.turns.iter().rev() {
        for word in NOT_A_WORD_ANY_CASE.split(&turn.user.to_lowercase()) {
            if text::len(&word) > 3 && !own.contains(&word) {
                own.push(word);
            }
        }
    }
    let in_hand: Vec<String> = _thing_in_hand(memory).iter().map(|w| w.to_lowercase()).collect();
    let mut found: Vec<String> = Vec::new();
    for source in [&wanted, &in_hand] {
        for key in source.iter() {
            for word in &own {
                if text::len(word) < 4 || found.contains(word) || _NOT_THE_PRACTICE.contains(word) {
                    continue;
                }
                if NOT_THE_THING.contains(word) || MEASURE_WORDS.contains(word) || STATE_WORDS.contains(word) || WHERE_IT_STANDS.contains(word) {
                    continue;
                }
                if text::slice(key, 0, Some(4)) == text::slice(word, 0, Some(4)) {
                    found.push(word.clone());
                }
            }
        }
        if !found.is_empty() {
            break;
        }
    }
    if !found.is_empty() {
        return found;
    }
    _thing_in_hand(memory)
        .iter()
        .filter(|w| text::len(w) > 3 && !_NOT_THE_PRACTICE.contains(&w.to_lowercase()) && !WHERE_IT_STANDS.contains(&w.to_lowercase()))
        .map(|w| w.to_lowercase())
        .collect()
}

/// `stated_about_the_person(text, memory)`: a fact about them read with the subject's words in
/// front of it.
pub fn stated_about_the_person(text_: &str, memory: &Memory) -> Vec<String> {
    let said = text::squash(text::strip(text_));
    let said = text::strip_chars(&said, " .!?").to_string();
    if said.is_empty() || text::split(&said).len() as i64 > *FACT_MOST_WORDS {
        return Vec::new();
    }
    if text_.contains('?') || ASKING.match_(&said).is_some() || MEASURE_ASK.is_match(&said) {
        return Vec::new();
    }
    if A_CHOICE.is_match(&said) {
        return Vec::new();
    }
    if _a_place_of_its_own(memory) {
        return Vec::new();
    }
    let low: Vec<String> = text::split(&said).iter().map(|w| text::strip_chars(&w.to_lowercase(), ".,;:!?()").to_string()).collect();
    if !(PERSON_OPEN.match_(&said).is_some() || FACT_OPEN.match_(&said).is_some() || A_FIGURE.is_match(&said)) {
        return Vec::new();
    }
    let named: Vec<&String> = low.iter().filter(|w| text::len(w) > 2 && !TIMES_ONLY.contains(*w) && !FACT_FILLER.contains(*w)).collect();
    if named.is_empty() {
        return Vec::new();
    }
    if named.iter().all(|w| PEOPLE_WORDS.contains(*w) || COUNT_WORDS.contains(*w)) {
        return Vec::new();
    }
    let mut reads: Vec<String> = Vec::new();
    for word in subject_words(memory, text_) {
        if low.contains(&word) {
            continue;
        }
        let written = text::squash(&format!("{word} {said}"));
        if !reads.contains(&written) {
            reads.push(written);
        }
    }
    reads
}

/// `_a_place_of_its_own(memory)`: the reading found a place in the line.
pub fn _a_place_of_its_own(memory: &Memory) -> bool {
    memory.understood.as_ref().is_some_and(|r| r.entities.iter().any(|e| e.kind == "place"))
}

/// `subject_words(memory, text="")`: the words of the thing the chat is on, at most four.
pub fn subject_words(memory: &Memory, text_: &str) -> Vec<String> {
    let mut was = String::new();
    if !text_.is_empty() {
        if let Some(record) = the_carried_topic(memory, text_) {
            if matches!(record, Value::Dict(_)) {
                was = vstr(record.get("text"));
            }
        }
    }
    let source = if was.is_empty() { the_subject(memory) } else { was };
    let said = text::squash(&source).to_lowercase();
    let mut out: Vec<String> = Vec::new();
    for word in NOT_A_LETTER_OR_DIGIT.split(&said) {
        if text::len(&word) < 3 || out.contains(&word) {
            continue;
        }
        if NOT_THE_THING.contains(&word) || MEASURE_WORDS.contains(&word) || STATE_WORDS.contains(&word) {
            continue;
        }
        if _NOT_A_PRONOUN.contains(&word) {
            continue;
        }
        if VERBISH.contains(&word) || VERB_FORMS.contains(&word) || NOT_THE_SUBJECT.contains(&word) {
            continue;
        }
        out.push(word);
    }
    out.truncate(4);
    out
}

/// `care_reports(text, memory)`: a care report ("i water it every week") with the thing put in.
pub fn care_reports(text_: &str, memory: &Memory) -> Vec<String> {
    let said = text::squash(text::strip(text_));
    let said = text::strip_chars(&said, " .!?").to_string();
    if said.is_empty() || text::split(&said).len() > 12 {
        return Vec::new();
    }
    let low: Vec<String> = text::split(&said).iter().map(|w| text::strip_chars(&w.to_lowercase(), ".,;:!?()").to_string()).collect();
    if !low.iter().any(|w| CARE_VERBS.contains(w)) {
        return Vec::new();
    }
    if !A_HOW_OFTEN.is_match(&said) {
        return Vec::new();
    }
    if ASKING.match_(&said).is_some() || MEASURE_ASK.is_match(&said) {
        return Vec::new();
    }
    if low.iter().any(|w| CARE_NOT_THE_THING.contains(w)) {
        return Vec::new();
    }
    let mut rest = I_OR_WE_LEAD.sub("", &said);
    let mut pointers: Vec<&String> = POINTS_BACK.iter().collect();
    pointers.sort();
    pointers.sort_by_key(|w| std::cmp::Reverse(text::len(w)));
    for word in pointers {
        let pattern = format!(r"\b{}\b", crate::py::re::escape(word));
        rest = crate::understanding::rx_flags(&pattern, crate::py::re::I).subn("", &rest, 1);
    }
    let rest = text::squash(&rest);
    if rest.is_empty() {
        return Vec::new();
    }
    let things = _the_persons_own_words(memory);
    if things.is_empty() || low.contains(&things[0]) {
        return Vec::new();
    }
    vec![text::squash(&format!("{} {rest}", things[0]))]
}

/// `a_faulty_thing(memory)`: the subject is a thing with a fault, and the answer gave no time.
pub fn a_faulty_thing(memory: &Memory) -> bool {
    let said = text::squash(&the_subject(memory)).to_lowercase();
    if said.is_empty() {
        return false;
    }
    if _the_answer_gave_a_time(memory) {
        return false;
    }
    NOT_A_LETTER_RUN.split(&said).iter().any(|w| A_FAULT.contains(w))
}

/// `_the_answer_gave_a_time(memory)`.
pub fn _the_answer_gave_a_time(memory: &Memory) -> bool {
    memory.turns.last().is_some_and(|t_| _A_TIME_IN_A_REPLY.is_match(&t_.bot))
}

/// `the_thing(memory)`: the subject without its fault and measure words, at most four words.
pub fn the_thing(memory: &Memory) -> String {
    let said = text::squash(&the_subject(memory)).to_lowercase();
    if said.is_empty() {
        return String::new();
    }
    let kept: Vec<String> = NOT_A_LETTER_OR_DIGIT
        .split(&said)
        .into_iter()
        .filter(|w| {
            !w.is_empty()
                && !A_FAULT.contains(w)
                && !NOT_THE_THING.contains(w)
                && !MEASURE_WORDS.contains(w)
                && !["the", "and", "for", "with"].contains(&w.as_str())
        })
        .collect();
    kept.into_iter().take(4).collect::<Vec<_>>().join(" ")
}

/// `subject_word(memory)`: the thing the chat is on, in one word, or "".
pub fn subject_word(memory: &Memory) -> String {
    let said = text::squash(&the_subject(memory)).to_lowercase();
    let mut found = _the_noun_in(&said);
    if !found.is_empty() && found.iter().all(|w| A_GETTING.contains(w)) {
        let own = match memory.get("topic") {
            Some(newest @ Value::Dict(_)) => vstr(newest.get("text")),
            _ => String::new(),
        };
        found = _the_noun_in(&text::squash(&own).to_lowercase());
    }
    if found.last().is_some_and(|w| PLACE_WORDS.contains(w)) {
        if let Some(before) = _the_record_before(memory) {
            let again = _the_noun_in(&text::squash(&vstr(before.get("text"))).to_lowercase());
            if !again.is_empty() {
                found = again;
            }
        }
    }
    found.last().cloned().unwrap_or_default()
}

/// `_the_noun_in(said)`: the content words of a phrase, in order.
pub fn _the_noun_in(said: &str) -> Vec<String> {
    if said.is_empty() {
        return Vec::new();
    }
    NOT_A_LETTER_RUN
        .split(said)
        .into_iter()
        .filter(|w| {
            text::len(w) > 2
                && !NOT_THE_THING.contains(w)
                && !MEASURE_WORDS.contains(w)
                && !A_FAULT.contains(w)
                && !["the", "and", "for", "with", "some", "any", "new"].contains(&w.as_str())
        })
        .collect()
}

/// `_the_record_before(memory)`: the newest topic record whose text is not the current topic's.
pub fn _the_record_before(memory: &Memory) -> Option<Value> {
    let now = match memory.get("topic") {
        Some(v @ Value::Dict(_)) => vstr(v.get("text")),
        _ => String::new(),
    };
    let records = memory.get("topics").map(Value::items).unwrap_or(&[]);
    records
        .iter()
        .filter(|r| matches!(r, Value::Dict(_)) && r.get("text").is_some_and(Value::truthy))
        .rev()
        .find(|r| vstr(r.get("text")) != now)
        .cloned()
}

/// `a_newer_thing(memory, word)`: the thing the newest line ends on, when it is not the word in
/// hand, or "".
pub fn a_newer_thing(memory: &Memory, word: &str) -> String {
    let Some(last) = memory.turns.last() else { return String::new() };
    if word.is_empty() {
        return String::new();
    }
    let said = low_squash(&last.user);
    let said = text::strip_chars(&said, " .!?").to_string();
    if said.is_empty() || A_PURPOSE.is_match(&said) {
        return String::new();
    }
    let tail = text::strip_chars(text::split(&said).last().map(String::as_str).unwrap_or(""), ".,;:!?").to_string();
    if text::len(&tail) < 4 || VERBS.contains(&tail) || NOT_A_THING_END.contains(&tail) || POINTS_BACK.contains(&tail) || A_FAULT.contains(&tail) {
        return String::new();
    }
    if ASPECT_NOUNS.contains(&tail) {
        return String::new();
    }
    if tail == word || tail.contains(word) || word.contains(tail.as_str()) {
        return String::new();
    }
    tail
}

/// `_the_trip_place(memory)`: the place of the trip the chat is on, or "".
pub fn _the_trip_place(memory: &Memory) -> String {
    let Some(known) = memory.get("user").filter(|u| matches!(u, Value::Dict(_))) else { return String::new() };
    let places: Vec<String> = known
        .get("places")
        .map(Value::items)
        .unwrap_or(&[])
        .iter()
        .map(|p| text::strip(&vstr(Some(p))).to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if places.is_empty() {
        return _the_place_the_bot_just_named(memory);
    }
    let place = places[places.len() - 1].clone();
    let said = place.to_lowercase();
    let turns = &memory.turns;
    let mut named_at: i64 = -1;
    let said_there = Re::new(&format!(r"(?<![a-z]){}(?![a-z])", crate::py::re::escape(&said)), 0);
    for index in (0..turns.len()).rev() {
        let line = low_squash(&turns[index].user);
        if !said_there.is_match(&line) {
            continue;
        }
        if !_GOING_AWAY.is_match(&line) {
            return String::new();
        }
        named_at = index as i64;
        break;
    }
    if named_at < 0 {
        let plan = low_squash(&vstr(known.get("plan")));
        if !(plan.contains(&said) && _GOING_AWAY.is_match(&plan)) {
            return String::new();
        }
    }
    let (card, card_at) = _the_opening_place_at(memory);
    if !card.is_empty() && card_at > named_at {
        return card;
    }
    place
}

/// `_the_opening_place(memory)`.
pub fn _the_opening_place(memory: &Memory) -> String {
    _the_opening_place_at(memory).0
}

/// `_the_opening_place_at(memory)`: the place the bot's newest card opened on ("In Lisbon:"), and
/// the turn it was given on, or ("", -1).
pub fn _the_opening_place_at(memory: &Memory) -> (String, i64) {
    let turns = &memory.turns;
    if turns.is_empty() {
        return (String::new(), -1);
    }
    for (back, turn) in turns.iter().rev().enumerate() {
        let Some(opening) = OPENING_IN.match_(&turn.bot) else { continue };
        let place = text::strip(opening.str(1)).to_string();
        if _A_TIME_WORD.contains(&place.to_lowercase()) {
            continue;
        }
        if A_DETERMINER_FIRST.match_(&place).is_some() {
            continue;
        }
        if !place.chars().next().is_some_and(|c| text::is_upper(&c.to_string())) {
            continue;
        }
        return (place, (turns.len() - 1 - back) as i64);
    }
    (String::new(), -1)
}

/// `_the_place_the_bot_just_named(memory)`: the card's place, unless they live there.
pub fn _the_place_the_bot_just_named(memory: &Memory) -> String {
    let place = _the_opening_place(memory);
    if place.is_empty() {
        return String::new();
    }
    if memory.turns.iter().any(|t_| _LIVING_THERE.is_match(&t_.user)) {
        return String::new();
    }
    place
}

/// `a_part_of_the_trip(text, memory)`: the trip's part asked with the place in it ("where should i
/// eat in rome"), or "".
pub fn a_part_of_the_trip(text_: &str, memory: &Memory) -> String {
    let place = _the_trip_place(memory);
    if place.is_empty() {
        return String::new();
    }
    let said = low_squash(text_);
    let said = text::strip_chars(&said, " .!?").to_string();
    if said.is_empty() || text::split(&said).len() > 8 {
        return String::new();
    }
    let mut part = String::new();
    let mut found_words: HashSet<String> = HashSet::new();
    let mut earliest: Option<usize> = None;
    for (pattern, asked) in A_TRIP_PART.iter() {
        let matches = pattern.finditer(&said);
        if matches.is_empty() {
            continue;
        }
        let found: HashSet<String> = matches.iter().map(|m| m.as_str().to_string()).collect();
        let at = matches.iter().map(|m| m.start()).min().unwrap();
        if earliest.is_none_or(|e| at < e) {
            part = asked.clone();
            earliest = Some(at);
            let joined: Vec<&str> = found.iter().map(String::as_str).collect();
            let mut words: HashSet<String> = NOT_A_LETTER_RUN.split(&joined.join(" ")).into_iter().collect();
            words.extend(found.iter().cloned());
            found_words = words;
        }
    }
    if part.is_empty() {
        return String::new();
    }
    for matched in &found_words {
        let pattern = format!(r"\b(?:a|an)\s+{}\b", crate::py::re::escape(matched));
        if crate::understanding::rx(&pattern).is_match(&said) {
            return String::new();
        }
    }
    for word in NOT_A_WORD_LETTER.split(&said) {
        if text::len(&word) < 3 || _ASK_FILLER.contains(&word) || found_words.contains(&word) {
            continue;
        }
        if (word == "day" || word == "days") && part.starts_with("how many days") {
            continue;
        }
        return String::new();
    }
    let shown = if text::is_upper(&place) { place.to_lowercase() } else { place };
    part.replace("{place}", &shown)
}

/// `asks_a_part_of_the_trip(text, memory)`: the line asks a part of the trip, with or without the
/// place named.
pub fn asks_a_part_of_the_trip(text_: &str, memory: &Memory) -> bool {
    if !a_part_of_the_trip(text_, memory).is_empty() {
        return true;
    }
    let place = _the_trip_place(memory);
    let said = low_squash(text_);
    let name = place.to_lowercase();
    if name.is_empty() || !said.contains(&name) {
        return false;
    }
    !a_part_of_the_trip(&said.replace(&name, " "), memory).is_empty()
}

/// `the_food_it_is_about(understood, memory)`: the food the line leans on, when the line does not
/// name it, or "".
pub fn the_food_it_is_about(understood: &Understood, memory: &Memory) -> String {
    let _ = memory;
    let Some(record) = understood.leans_on.as_ref() else { return String::new() };
    let food = text::squash(record.subject.as_deref().unwrap_or("")).to_lowercase();
    if food.is_empty() {
        return String::new();
    }
    let line = text::squash(if understood.clean.is_empty() { &understood.text } else { &understood.clean }).to_lowercase();
    if line.is_empty() || line.contains(&food) {
        return String::new();
    }
    let raised = text::squash(&record.text).to_lowercase();
    if !raised.is_empty() && LIMIT.is_match(&raised) {
        return String::new();
    }
    food
}
