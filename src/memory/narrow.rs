//! Port of `bot/memory/narrow.py`: the part of an answer that a follow-up asks for.
//!
//! A follow-up often asks about the answer the bot has just given: after "How do I roast a
//! chicken?", "what temperature should the oven be?" wants the oven temperature, not the whole
//! method again. This returns the sentence, clause or number that answers the property asked
//! for ("190C (170C fan).", "40 to 45 minutes.", "20 minutes per 500g.").
//!
//! It is deliberately conservative: it only acts when the line asks for a property it knows how
//! to find (time, heat, amount, money, distance, order), and it returns None rather than guess.
//!
//! Offsets: Python's string indices are code points and Rust's are bytes. Wherever Python only
//! slices a string at positions found in that same string the two agree, and the byte offsets
//! are used as they are; wherever Python does arithmetic on an index (`at - 24`, `end + 14`,
//! `text[-12:]`) or carries an index from one string into another (`flat(sentence)[:at]`), the
//! index is converted to characters first.
#![allow(non_snake_case)]

use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::understanding::pyuni;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex, RwLock};

macro_rules! data_re {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.memory.narrow.", stringify!($name)))); )*
    };
}
macro_rules! data_set {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.memory.narrow.", stringify!($name)))); )*
    };
}
macro_rules! inline_re {
    ($name:ident, $pattern:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pattern, $flags));
    };
}

data_re!(
    PLACE, COOKS, URGENT, DONENESS, ASKS_THE_DATE_ITSELF, A_LENGTH, CAUSE, REMEDY, SYMPTOM,
    REMEDY_CUT, REMEDY_JOINER, DATED_STRONG, RATE, FREQUENCY_AFTER,
    FREQUENCY_AFTER_TIMES, FREQUENCY_BEFORE, FREQUENCY_AFTER_AT_A_TIME, LENGTH_OF_THE_JOB,
    A_COUNTDOWN, A_MOMENT, RATE_LEAD, A_SET_OF_STEPS, SEQUENCE, WAIT, STEP, AFTER, BEFORE, RESULT,
    BARE_UNIT, KEEPS, QUALIFIER, SENTENCE, NUM_RANGE, THOUSANDS, WORD, OPTION, NOT_OPTION,
    FOLLOW_UP, UNIT, BARE_RANGE, ASKS_ABOUT_IT, YES_NO, STEP_VERB, _COUNTED_UNIT, CLAUSE_BREAK,
    PAPERWORK, A_VALIDITY_ASK, A_RATE_ON_A_FIGURE, A_PERIOD_IN_A_LINE, JOINS, _STATE_LENGTH_ASKS,
    _A_DOING, _A_CUT_OFF_BEFORE, _A_CUT_OFF_AFTER, _A_DURATION_VERB, _IN_MONTHS_OR_YEARS, _A_RANGE,
    _A_BARE_FIGURE, A_START_LEAD, _A_PERIOD_OF_EXTENT, A_WHEN_WORD, ASKS_FREQUENCY, FREQUENCY,
    ASKED_UNIT, RATE_FREQ, FREQ_CLAUSE, TRIGGER, RATE_INTERVAL, A_BARE_COUNT, ONLY_THE_INTERVAL_ASK,
    A_HOW_OFTEN_THE_DOING, FREQ_LIMIT, FREQ_PORTION, ASKS_FOR_A_HAND, OWN_SAVING, SAYS_SOMETHING,
    QUOTED_WORDS, QUOTED_WORDS_SINGLE, DURATION_ADJECTIVE, CARRIES_ON, EITHER_OR, FIGURE_MEASURE,
    A_UNIT_WORD, A_BARE_RANGE, A_FOR_A_THING, IN_A_RELATIVE, A_WEATHER_WINDOW, AT_A_TIME,
    IN_A_CONDITION, A_BOUNDED, A_STATE_FOR, A_RELATIVE_AFTER, ASIDE_ON, HELD, NAMED_BREAK, LEANING,
    CLAUSE_BREAK_COLON,
);

data_set!(
    LENGTH_FILLER, OVEN, WHERE, QUALIFIER_WORDS, QUALIFIER_FILLER, EMPTY,
    NAMING_IGNORE, ASKING_ONLY, FREQUENCY_IGNORE, POINTS_BACK,
);

static RANGE_JOIN: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.narrow.RANGE_JOIN"));
static THOUSANDS_JOIN: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.narrow.THOUSANDS_JOIN"));
static NO_SET_TIME: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.narrow.NO_SET_TIME"));
static NO_INTERVAL_SAID: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.narrow.NO_INTERVAL_SAID"));
static A_STAGE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.narrow.A_STAGE"));
static NAMES_A_PART: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.narrow.NAMES_A_PART").as_int().unwrap() as usize);
static BAREST_VERDICT: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.narrow.BAREST_VERDICT").as_int().unwrap() as usize);

/// `(kind, pattern)` pairs of a module tuple such as `ASKS` or `OUTCOME_ASKS`, in order.
fn kind_table(name: &str) -> Vec<(String, &'static Re)> {
    data::get(name)
        .items()
        .iter()
        .map(|pair| {
            let item = pair.items();
            (item[0].str().to_string(), item[1].as_re().expect("a compiled pattern"))
        })
        .collect()
}

static ASKS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| kind_table("bot.memory.narrow.ASKS"));
static OUTCOME_ASKS: LazyLock<Vec<(String, &'static Re)>> =
    LazyLock::new(|| kind_table("bot.memory.narrow.OUTCOME_ASKS"));

/// A `{str: pattern}` dict global, in its order.
fn pattern_map(name: &str) -> Vec<(String, &'static Re)> {
    data::get(name)
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.as_re().expect("a compiled pattern")))
        .collect()
}

static SIGNALS_MAP: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| pattern_map("bot.memory.narrow.SIGNALS"));
static SPANS_MAP: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| pattern_map("bot.memory.narrow.SPANS"));

/// `SIGNALS.get(kind)`.
fn signals(kind: &str) -> Option<&'static Re> {
    SIGNALS_MAP.iter().find(|(k, _)| k == kind).map(|(_, r)| *r)
}

/// `SIGNALS[kind]`: a kind with no signal raises KeyError in Python.
fn signal(kind: &str) -> &'static Re {
    signals(kind).unwrap_or_else(|| panic!("KeyError: {kind:?} not in narrow.SIGNALS"))
}

/// `SPANS.get(kind)`.
fn spans(kind: &str) -> Option<&'static Re> {
    SPANS_MAP.iter().find(|(k, _)| k == kind).map(|(_, r)| *r)
}

/// `SPANS[kind]`.
fn span(kind: &str) -> &'static Re {
    spans(kind).unwrap_or_else(|| panic!("KeyError: {kind:?} not in narrow.SPANS"))
}

/// `SAME`: the groups of words that mean the same thing. `_same` adds to it, as in Python.
static SAME: LazyLock<RwLock<HashMap<String, Vec<String>>>> = LazyLock::new(|| {
    RwLock::new(
        data::get("bot.memory.narrow.SAME")
            .pairs()
            .iter()
            .map(|(k, v)| (k.str().to_string(), v.strs()))
            .collect(),
    )
});

/// `SAME.get(word, ())`.
fn same_as(word: &str) -> Vec<String> {
    SAME.read().unwrap().get(word).cloned().unwrap_or_default()
}

/// `word in SAME`.
fn in_same(word: &str) -> bool {
    SAME.read().unwrap().contains_key(word)
}

/// `_UNITS`: what each unit word counts.
static _UNITS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.narrow._UNITS"));

/// A pattern built at run time (`re.compile` / `re.search` on a string made in the function),
/// compiled once per distinct pattern, as Python's own `re` cache does.
pub(crate) fn re_cached(pattern: &str, flags: u32) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let key = (pattern.to_string(), flags);
    if let Some(found) = CACHE.lock().unwrap().get(&key) {
        return found;
    }
    let made: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    CACHE.lock().unwrap().insert(key, made);
    made
}

/// `template.format(name=value, ...)`: plain `{name}` fields and `{{` / `}}`. A field the call
/// does not give, or a lone brace, raises in Python; here it panics, as the module would fail.
pub(crate) fn py_format(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(i) = rest.find(['{', '}']) {
        out.push_str(&rest[..i]);
        let c = rest.as_bytes()[i];
        let after = &rest[i + 1..];
        if c == b'{' && after.starts_with('{') {
            out.push('{');
            rest = &after[1..];
        } else if c == b'}' && after.starts_with('}') {
            out.push('}');
            rest = &after[1..];
        } else if c == b'{' {
            let end = after.find('}').unwrap_or_else(|| panic!("ValueError: unclosed field in {template:?}"));
            let name = &after[..end];
            match fields.iter().find(|(k, _)| *k == name) {
                Some((_, v)) => out.push_str(v),
                None => panic!("KeyError: {name:?} in str.format"),
            }
            rest = &after[end + 1..];
        } else {
            panic!("ValueError: single '}}' in {template:?}");
        }
    }
    out.push_str(rest);
    out
}

/// Python's `str(value)` for the values memory holds (`repr` inside containers).
pub(crate) fn py_str(value: &Value) -> String {
    match value {
        Value::Str(s) => s.to_string(),
        other => py_repr(other),
    }
}

/// `str(value or "")`: a falsy value is "".
pub(crate) fn py_str_or_empty(value: Option<&Value>) -> String {
    match value {
        Some(v) if v.truthy() => py_str(v),
        _ => String::new(),
    }
}

/// Python's `repr(value)`.
pub(crate) fn py_repr(value: &Value) -> String {
    match value {
        Value::None => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Str(s) => {
            let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
            let mut out = String::new();
            out.push(quote);
            for c in s.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c == quote => {
                        out.push('\\');
                        out.push(c);
                    }
                    c => out.push(c),
                }
            }
            out.push(quote);
            out
        }
        Value::List(v) => format!("[{}]", v.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Tuple(v) if v.len() == 1 => format!("({},)", py_repr(&v[0])),
        Value::Tuple(v) => format!("({})", v.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Set(v) if v.is_empty() => "set()".into(),
        Value::Set(v) => format!("{{{}}}", v.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Dict(d) => format!(
            "{{{}}}",
            d.iter().map(|(k, v)| format!("{}: {}", py_repr(k), py_repr(v))).collect::<Vec<_>>().join(", ")
        ),
        other => format!("{other:?}"),
    }
}

/// The memory as the Python dict a function that takes `memory` as a value reads
/// (`compose.pick` and its kind take the memory as a `Value` in the contract).
#[allow(dead_code)]
pub(crate) fn memory_value(memory: &crate::state::Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// Python's `try: ... except Exception:` around a call: a panic that stands for a Python
/// exception is caught and gives None. A stub that is not ported yet (`UNPORTED ...`) is not a
/// Python exception, so it is passed on and the module is skipped as unported.
pub(crate) fn py_try<T>(f: impl FnOnce() -> T) -> Option<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(value) => Some(value),
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            if message.starts_with("UNPORTED") {
                std::panic::resume_unwind(payload);
            }
            None
        }
    }
}

/// `len(s.split())`.
fn n_words(s: &str) -> usize {
    text::split_ref(s).len()
}

/// `s[a:b]` with a character start counted on from a byte offset: `s[at:at + n]` where `at`
/// is a match position (bytes) and `n` counts characters.
fn chars_from(s: &str, at: usize, n: usize) -> &str {
    let rest = &s[at..];
    let end = rest.char_indices().nth(n).map(|(b, _)| b).unwrap_or(rest.len());
    &rest[..end]
}

/// `s[:at]` where `at` is a Python character index (clamped like a slice).
fn chars_before(s: &str, at: usize) -> &str {
    &s[..text::byte_at(s, at)]
}

/// `s[-n:]` in characters.
fn last_chars(s: &str, n: usize) -> &str {
    let count = s.chars().count();
    if count <= n {
        return s;
    }
    &s[text::byte_at(s, count - n)..]
}

/// The set of `words_of(text)`.
fn word_set(text_: &str) -> HashSet<String> {
    words_of(text_).into_iter().collect()
}

inline_re!(RE_DIGIT, r"\d", 0);
inline_re!(RE_JOINER_OFF, r"^(?:and|but|so|then|also)\s+", pyre::I);
inline_re!(RE_SENTENCE_SPLIT, r"(?<=[.!?])\s+", 0);
inline_re!(RE_DEPENDS, r"\bdepends\b", pyre::I);
inline_re!(RE_CLAUSE_OF_TIME, r"\s*(?:;|,\s*(?:and|but|so)\b)", 0);
inline_re!(
    RE_SAYS_A_TIME,
    r"\b(?:long|longer|slow|quick|soon|never|until|till|months?|weeks?|years?|days?|hours?|minutes?)\b",
    pyre::I
);
inline_re!(RE_AGAIN, r"\bagain\b", pyre::I);
inline_re!(RE_COUNTS_ON, r"\b(?:the same|same count|that count|as above|as before)\b", pyre::I);
inline_re!(RE_JOINER_OPENS, r"(?:then|and|also|plus|after that|afterwards)\b", pyre::I);
inline_re!(RE_COMMA_SPACE, r",\s*", 0);
inline_re!(RE_NAMED_PIECES, r";\s*|(?<=[.!?])\s+|,\s+(?:and|but|so)\s+|\s+(?:and|but|so)\s+", 0);
inline_re!(RE_LIST_GOES_ON, r"\s*(?:[,;]|\band\b)", 0);
inline_re!(RE_HOW_DO_I, r"\bhow (?:do|can|should|would|could) (?:i|we|you)\b", 0);
inline_re!(RE_STEP_CUT, r"[:,]\s+", 0);
inline_re!(RE_TRAILING_JOIN, r"[\s,]*(?:and|or)?[\s,]*$", 0);
inline_re!(RE_PER_OR_A, r"\bper\b|\ba\b", pyre::I);
inline_re!(RE_TAKES_OR_SO, r"\b(?:takes|so)\b", pyre::I);
inline_re!(RE_OPTIONS, r"\b\d+(?:[^,.;:]{0,18}?)\bfor\b", pyre::I);
inline_re!(RE_LEAD_CUT, r"[:,;]\s+|\s+-\s+", 0);
inline_re!(RE_PER, r"\bper\b", pyre::I);
inline_re!(RE_WORDS_AZ, r"[A-Za-z]+", 0);

// ---------------------------------------------------------------------------------------------
// A line that only gives a length

/// Python: bot/memory/narrow.py:143 `only_a_length(text)`
///
/// Whether the line says nothing but how long the thing has been going ("its been like that for
/// a week").
pub fn only_a_length(text_: &str) -> bool {
    if text_.contains('?') {
        return false;
    }
    let Some(found) = A_LENGTH.search(text_) else { return false };
    let joined = format!("{} {}", &text_[..found.start()], &text_[found.end()..]).replace(',', " ");
    let rest = text::split_ref(&joined);
    if rest.len() > 6 {
        return false;
    }
    rest.iter().all(|word| LENGTH_FILLER.contains(&text::lower(word)))
}

/// Python: bot/memory/narrow.py:218 `_the_remedy_in(said)`
///
/// The clause of a symptom answer that says what to take, or None when there is none.
pub fn _the_remedy_in(said: &str) -> Option<String> {
    if !REMEDY.is_match(said) || !SYMPTOM.is_match(said) {
        return None;
    }
    let sentences: Vec<String> =
        SENTENCE.split(said).iter().map(|s| text::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    for sentence in sentences {
        let pieces: Vec<String> = REMEDY_CUT
            .split(&sentence)
            .iter()
            .map(|piece| text::strip(piece).to_string())
            .filter(|piece| !piece.is_empty())
            .collect();
        for (at, piece) in pieces.iter().enumerate() {
            if !REMEDY.is_match(piece) {
                continue;
            }
            let mut kept = vec![piece.clone()];
            while at + kept.len() < pieces.len() && n_words(&kept.join(" ")) < 6 {
                kept.push(pieces[at + kept.len()].clone());
            }
            let joined = kept.join(", ");
            let part = tidy(&REMEDY_JOINER.sub("", text::strip_chars(&joined, " ,;:")));
            if let Some(part) = part {
                if !part.is_empty() && text::count(&part, "(") == text::count(&part, ")") {
                    return Some(part);
                }
            }
        }
    }
    None
}

/// Python: bot/memory/narrow.py:244 `carries(reply, kind)`
///
/// Whether an answer holds the part a line of this kind asks for.
pub fn carries(reply: &str, kind: &str) -> bool {
    if kind == "remedy" {
        return REMEDY.is_match(&text::squash(&text::lower(reply)));
    }
    match spans(kind) {
        Some(span) => span.is_match(reply),
        None => false,
    }
}

/// Python: bot/memory/narrow.py:291 `has_a_date(sentence)`
///
/// Whether a sentence gives a date, counting a bare number only when it is not money.
pub fn has_a_date(sentence: &str) -> bool {
    if DATED_STRONG.is_match(sentence) {
        return true;
    }
    span("date").is_match(&span("money").sub(" ", sentence))
}

/// Python: bot/memory/narrow.py:623 `pieces_of(text, pattern)`
///
/// `text` cut into pieces on `pattern`, with a numeric range and a thousands comma kept whole.
/// (`pattern` is a compiled pattern; Python also takes the pattern's text.)
pub fn pieces_of(text_: &str, pattern: &Re) -> Vec<String> {
    let guarded = NUM_RANGE.sub_fn(text_, |found| found.as_str().replace(" and ", &RANGE_JOIN));
    let guarded = THOUSANDS.sub_fn(&guarded, |_| THOUSANDS_JOIN.to_string());
    pattern
        .split_opt(&guarded)
        .into_iter()
        .map(|piece| {
            // A group in the pattern that did not take part gives None, and None.replace raises.
            let piece = piece.expect("AttributeError: 'NoneType' object has no attribute 'replace'");
            piece.replace(RANGE_JOIN.as_str(), " and ").replace(THOUSANDS_JOIN.as_str(), ",")
        })
        .collect()
}

/// Python: bot/memory/narrow.py:659 `_same(words)`
///
/// Every word of the group means the same as the others (`SAME` is filled at import; the
/// bundle already holds the result, and this adds to it the way Python would).
pub fn _same(words: &str) {
    let mut group: Vec<String> = Vec::new();
    for word in text::split(words) {
        if !group.contains(&word) {
            group.push(word);
        }
    }
    group.sort();
    let mut same = SAME.write().unwrap();
    for word in &group {
        same.insert(word.clone(), group.clone());
    }
}

/// Python: bot/memory/narrow.py:731 `step_word(question)`
///
/// The verb a yes/no question names as its step, or None when it names none.
pub fn step_word(question: &str) -> Option<String> {
    let said = flat(question);
    STEP_VERB.search(&said).map(|found| text::lower(found.str(1)))
}

/// Python: bot/memory/narrow.py:737 `kind_of(question)`
///
/// The property the line asks for: money, distance, time, heat, amount, order or None.
pub fn kind_of(question: &str) -> Option<String> {
    let clean = flat(question);
    for (kind, pattern) in ASKS.iter() {
        if pattern.is_match(&clean) {
            // "when is it done" reads as a date, but a cooking answer has no year to cut out:
            // the line wants the doneness test, which is a sentence, not a figure.
            if kind == "date" && DONENESS.is_match(&clean) {
                return None;
            }
            return Some(kind.clone());
        }
    }
    None
}

/// Python: bot/memory/narrow.py:752 `asks_a_part(question)`
///
/// Whether a follow-up asks for a part of the answer: a figure, a step or a property.
pub fn asks_a_part(question: &str) -> bool {
    kind_of(question).is_some()
        || _property_ask(question)
        || _option_ask(question)
        || _named_follow_up(question)
        || YES_NO.is_match(&flat(question))
        || ASKS_FREQUENCY.is_match(&flat(question))
        || ASKS_FOR_A_HAND.is_match(&flat(question))
}

/// Python: bot/memory/narrow.py:760 `_named_follow_up(question)`
///
/// Whether the line follows up on a thing it names ("and if I use a loaf tin?").
pub fn _named_follow_up(question: &str) -> bool {
    let clean = flat(question);
    if !FOLLOW_UP.is_match(&clean) {
        return false;
    }
    let named = words_of(&clean).into_iter().filter(|word| !NAMING_IGNORE.contains(word) && word.chars().count() > 2).count();
    (1..=4).contains(&named)
}

/// Python: bot/memory/narrow.py:777 `_gives_a_figure_in(part, unit)`
///
/// Whether this part of an answer gives a figure in that unit ("about eight weeks").
pub fn _gives_a_figure_in(part: &str, unit: &str) -> bool {
    _COUNTED_UNIT.finditer(part).iter().any(|hit| _unit_stem(hit.str(1)) == unit)
}

/// Python: bot/memory/narrow.py:791 `_the_clause_in_the_unit(sentence, unit)`
///
/// The clause of that sentence that gives the figure, said on its own, or None.
pub fn _the_clause_in_the_unit(sentence: &str, unit: &str) -> Option<String> {
    for clause in CLAUSE_BREAK.split(sentence) {
        let part = text::strip(&clause);
        if part.is_empty() || !_gives_a_figure_in(part, unit) {
            continue;
        }
        let part = RE_JOINER_OFF.sub("", part);
        if n_words(&part) >= 3 {
            return Some(text::upper_first(&part));
        }
    }
    None
}

/// Python: bot/memory/narrow.py:808 `_the_sentence_in_the_unit(reply, unit)`
///
/// The answer's own sentence that gives a figure in that unit, or None.
pub fn _the_sentence_in_the_unit(reply: &str, unit: &str) -> Option<String> {
    for sentence in RE_SENTENCE_SPLIT.split(reply) {
        let part = text::strip(&sentence);
        if !part.is_empty() && _gives_a_figure_in(part, unit) {
            return Some(_the_clause_in_the_unit(part, unit).unwrap_or_else(|| part.to_string()));
        }
    }
    None
}

/// Python: bot/memory/narrow.py:824 `part_in_the_unit(reply, question)`
///
/// CONTRACT: the generated signature returns `()` (the trace only ever saw None); Python
/// returns the part or None, which is [`part_in_the_unit_text`]. This keeps the contract and
/// runs the same reading.
pub fn part_in_the_unit(reply: &str, question: &str) {
    let _ = part_in_the_unit_text(reply, question);
}

/// `part_in_the_unit(reply, question)` with its value: the part of an answer that gives a
/// figure in the unit the line asks for, or None.
pub fn part_in_the_unit_text(reply: &str, question: &str) -> Option<String> {
    let asked = ASKED_UNIT.search(question)?;
    let part = narrow(reply, question, "")?;
    if text::strip(&part).is_empty() {
        return None;
    }
    let unit = _unit_stem(asked.str(1));
    let said = RE_WORDS_AZ.finditer(&part).iter().any(|word| {
        let word = word.as_str();
        _unit_stem(word) == unit && _UNITS.contains_key(text::rstrip_chars(&text::lower(word), "s"))
    });
    if said { Some(part) } else { None }
}

/// Python: bot/memory/narrow.py:865 `asked_unit(question)`
///
/// The unit a "how many weeks" line asks for, as a stem, or None when it names none.
pub fn asked_unit(question: &str) -> Option<String> {
    ASKED_UNIT.search(question).map(|asked| _unit_stem(asked.str(1)))
}

/// Python: bot/memory/narrow.py:871 `_states_a_length(part)`
///
/// Whether this part of an answer gives a figure in any unit at all ("20 minutes").
pub fn _states_a_length(part: &str) -> bool {
    _COUNTED_UNIT.is_match(part)
}

/// Python: bot/memory/narrow.py:917 `the_outcome_asked(question)`
///
/// Which of the outcome asks the line makes, or "" when it makes none.
pub fn the_outcome_asked(question: &str) -> String {
    let said = flat(question);
    for (which, pattern) in OUTCOME_ASKS.iter() {
        if pattern.is_match(&said) {
            return which.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/narrow.py:926 `the_part_asked(reply, question, about='')`
///
/// The part of the answer the line asks for, with the unit it names respected.
pub fn the_part_asked(reply: &str, question: &str, about: &str) -> Option<String> {
    let part = narrow(reply, question, about);
    let said = part.as_deref().unwrap_or("");
    let Some(unit) = asked_unit(question) else { return part };
    if _gives_a_figure_in(said, &unit) || !_states_a_length(said) {
        return part;
    }
    _the_sentence_in_the_unit(reply, &unit).or(part)
}

/// Python: bot/memory/narrow.py:952 `_a_rate_cut_from_its_figure(reply, part, question)`
///
/// Whether `part` is a figure that left the period making it a rate behind.
pub fn _a_rate_cut_from_its_figure(reply: &str, part: &str, question: &str) -> bool {
    let whole = flat(reply);
    let piece = flat(part);
    let money = span("money");
    let Some(figure) = money.search(&piece) else { return false };
    let figure = text::lower(figure.as_str());
    let said = flat(question);
    let asked = A_PERIOD_IN_A_LINE.search(&said);
    let mut periods: Vec<String> = Vec::new();
    for hit in money.finditer(&whole) {
        if text::lower(hit.as_str()) != figure {
            continue;
        }
        // `whole[hit.end():hit.end() + 14]`: fourteen characters on.
        let after = chars_from(&whole, hit.end(), 14);
        match A_RATE_ON_A_FIGURE.match_(after) {
            None => {
                // The same figure is also written as a plain price somewhere in the answer, so
                // the part may be that price: leave it alone rather than guess.
                return false;
            }
            Some(found) => periods.push(text::lower(found.str(1))),
        }
    }
    if periods.is_empty() {
        return false;
    }
    match asked {
        None => true,
        Some(asked) => !periods.contains(&text::lower(asked.str(1))),
    }
}

/// Python: bot/memory/narrow.py:982 `narrow(reply, question, about='')`
///
/// The part of `reply` that answers `question`, or None when it cannot say safely. None also
/// means the part would be the whole reply: the caller must not repeat it.
pub fn narrow(reply: &str, question: &str, about: &str) -> Option<String> {
    let said = flat(question);
    if A_VALIDITY_ASK.is_match(&said) && (PAPERWORK.is_match(&said) || PAPERWORK.is_match(&flat(about))) {
        return None;
    }
    let part = _narrow(reply, question)?;
    // A part that starts on the word joining it to the rest of its sentence is not an answer
    // anyone can read ("Which is about four cups of coffee.").
    if JOINS.match_(&flat(&part)).is_some() {
        return None;
    }
    if kind_of(question).as_deref() == Some("money") && _a_rate_cut_from_its_figure(reply, &part, question) {
        return None;
    }
    Some(part)
}

/// Python: bot/memory/narrow.py:1029 `_a_length_of_a_doing(sentence, part, question)`
///
/// True when `part` is how long something the advice tells them to do goes on for.
pub fn _a_length_of_a_doing(sentence: &str, part: &str, question: &str) -> bool {
    if !_STATE_LENGTH_ASKS.is_match(&flat(question)) {
        return false;
    }
    let said = flat(sentence);
    let cut = flat(part);
    let at = if cut.is_empty() { None } else { said.find(&cut) };
    let Some(at) = at else { return false };
    let doing: HashSet<String> = _A_DOING.findall_str(&said).into_iter().collect();
    if doing.is_empty() {
        return false;
    }
    let asked: HashSet<String> = _A_DOING.findall_str(&flat(question)).into_iter().collect();
    if doing.intersection(&asked).next().is_some() {
        return false;
    }
    // The clause the figure sits in: the doing that owns a length is the one the clause opens
    // with.
    _A_DOING.is_match(&_a_clause_before(&said, at))
}

/// Python: bot/memory/narrow.py:1085 `_a_cut_off_not_a_length(sentence, part)`
///
/// True when the figure `part` is the cut-off a rule starts at, not a length of anything.
pub fn _a_cut_off_not_a_length(sentence: &str, part: &str) -> bool {
    let said = flat(sentence);
    let flat_part = flat(part);
    let cut = text::rstrip_chars(&flat_part, ". ");
    if cut.is_empty() {
        return false;
    }
    let Some(at) = said.find(cut) else { return false };
    let before = &said[..at];
    let after = &said[at + cut.len()..];
    if !(_A_CUT_OFF_BEFORE.is_match(before) || _A_CUT_OFF_AFTER.match_(after).is_some()) {
        return false;
    }
    if !(_IN_MONTHS_OR_YEARS.is_match(cut) || _A_BARE_FIGURE.fullmatch(cut).is_some()) {
        return false;
    }
    if _A_RANGE.is_match(cut) {
        return false; // "four to six months" is a span someone can ask the length of
    }
    let clause = _a_clause_before(&said, at);
    !_A_DURATION_VERB.is_match(&clause)
}

/// Python: bot/memory/narrow.py:1113 `_a_clause_before(said, at)`
///
/// The clause `at` sits in: back to the nearest comma, colon or semicolon. (`at` is a byte
/// offset into `said`, found in `said`.)
pub fn _a_clause_before(said: &str, at: usize) -> String {
    let mut start = 0;
    for mark in [",", ";", ":"] {
        if let Some(cut) = said[..at].rfind(mark) {
            start = start.max(cut + 1);
        }
    }
    said[start..at].to_string()
}

/// Python: bot/memory/narrow.py:1129 `the_doings_length(reply, question)`
///
/// The clause of `reply` that gives a time for something to keep doing, or "".
pub fn the_doings_length(reply: &str, question: &str) -> String {
    let whole = flat(reply);
    if whole.is_empty() || !_STATE_LENGTH_ASKS.is_match(&flat(question)) {
        return String::new();
    }
    let kind = kind_of(question);
    let kind = match kind.as_deref() {
        Some(k @ ("time" | "clock")) => k.to_string(),
        _ => return String::new(),
    };
    let mut clauses: Vec<String> = Vec::new();
    for sentence in SENTENCE.split(reply) {
        if text::strip(&sentence).is_empty() || !signal(&kind).is_match(&sentence) {
            continue;
        }
        let part = match _part(&sentence, &kind, question) {
            Part::Text(p) if !p.is_empty() => p,
            _ => continue,
        };
        if !_a_length_of_a_doing(&sentence, &part, question) {
            return String::new(); // the answer states the thing's own length
        }
        let said = flat(&sentence);
        let at = said.find(&flat(&part));
        let clause = _the_clause_around(&said, at);
        if !clause.is_empty() && flat(&clause) != whole {
            clauses.push(clause);
        }
    }
    clauses.into_iter().next().unwrap_or_default()
}

/// Python: bot/memory/narrow.py:1166 `_the_clause_around(said, at)`
///
/// The clause `at` sits in, back to the mark before it and up to the mark after it. (`at` is
/// `said.find(...)`: None for -1.)
pub fn _the_clause_around(said: &str, at: Option<usize>) -> String {
    let Some(at) = at else { return String::new() };
    let mut start = 0;
    for mark in [",", ";", ":"] {
        if let Some(cut) = said[..at].rfind(mark) {
            start = start.max(cut + 1);
        }
    }
    let mut end = said.len();
    for mark in [",", ";", ":"] {
        if let Some(cut) = said[at..].find(mark) {
            end = end.min(at + cut);
        }
    }
    if start >= end {
        return String::new();
    }
    text::strip_chars(&said[start..end], " .").to_string()
}

/// Python: bot/memory/narrow.py:1183 `_at_another_place(sentence, part, question)`
///
/// Whether this part gives the figure of a place other than the one the line asks about.
pub fn _at_another_place(sentence: &str, part: &str, question: &str) -> bool {
    let asked: HashSet<String> = PLACE.findall_str(&flat(question)).iter().map(|w| text::lower(w)).collect();
    if asked.is_empty() {
        return false;
    }
    let said = flat(sentence);
    let cut = flat(part);
    let at = if cut.is_empty() { None } else { said.find(&cut) };
    let Some(at) = at else { return false };
    let Some(found) = PLACE.search(&said[at + cut.len()..]) else { return false };
    !asked.contains(&text::lower(found.as_str()))
}

/// Python: bot/memory/narrow.py:1206 `_gives_another_places_figure(sentence, kind, question)`
///
/// Whether the sentence's own figure for this ask is written at another place.
pub fn _gives_another_places_figure(sentence: &str, kind: Option<&str>, question: &str) -> bool {
    let Some(kind) = kind else { return false };
    if spans(kind).is_none() {
        return false;
    }
    match _part(sentence, kind, question) {
        Part::Text(part) if !part.is_empty() => _at_another_place(sentence, &part, question),
        _ => false,
    }
}

/// Python: bot/memory/narrow.py:1216 `_narrow(reply, question)`
pub fn _narrow(reply: &str, question: &str) -> Option<String> {
    if text::strip(reply).is_empty() {
        return None;
    }
    let mut kind = kind_of(question);
    if kind.as_deref() == Some("remedy") {
        // What to take is a clause of a sentence, not a figure.
        let part = _the_remedy_in(reply);
        return match part {
            Some(part) if !part.is_empty() && flat(&part) != flat(reply) => Some(part),
            _ => None,
        };
    }
    if let Some(k) = kind.as_deref() {
        if spans(k).is_none() {
            // "can I make it the day before?" asks for a step, and there is no figure to cut
            // out for it: whoever reads whole sentences answers that line.
            return None;
        }
    }
    let whole = flat(reply);
    if URGENT.is_match(&whole) {
        return None;
    }
    let mut sentences: Vec<String> = SENTENCE
        .split(text::strip(reply))
        .iter()
        .map(|s| text::strip(s).to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut when_ask = false;
    if kind.as_deref() == Some("date") {
        if let Some(dated) = _dated(&sentences, question, &whole) {
            return Some(dated);
        }
        // A date ask with no dated sentence in the answer is the timing in it when the line
        // asks when something happens; not when it asks for the date itself.
        if ASKS_THE_DATE_ITSELF.is_match(&flat(question)) {
            return None;
        }
        when_ask = true;
        kind = if signal("time").is_match(reply) { Some("time".to_string()) } else { None };
        if kind.is_none() {
            return _the_answer_said_again(&sentences, &whole);
        }
    }
    // A question that names the thing it is about is answered by the sentence that names it
    // too, not by the first figure in the answer.
    let wanted: HashSet<String> = words_of(question).into_iter().filter(|w| !POINTS_BACK.contains(w)).collect();
    if !wanted.is_empty() {
        let naming: Vec<String> = sentences
            .iter()
            .filter(|one| word_set(one).intersection(&wanted).next().is_some())
            .cloned()
            .collect();
        if !naming.is_empty() {
            let kept: Vec<String> = naming
                .into_iter()
                .filter(|one| !_gives_another_places_figure(one, kind.as_deref(), question))
                .collect();
            if !kept.is_empty() {
                sentences = kept;
            }
        }
    }
    if kind.as_deref() == Some("cause") {
        // The reason is a whole sentence, not a figure.
        let sets: Vec<HashSet<String>> = sentences.iter().map(|one| word_set(one)).collect();
        let only: HashSet<String> =
            wanted.iter().filter(|word| sets.iter().filter(|set| set.contains(*word)).count() == 1).cloned().collect();
        if !only.is_empty() {
            let named: Vec<String> = sentences
                .iter()
                .filter(|one| word_set(one).intersection(&only).next().is_some())
                .cloned()
                .collect();
            if !named.is_empty() {
                sentences = named;
            }
        }
        let openings: &[String] =
            if text::strip_chars(&flat(question), " ?!.") == "why" { &sentences[..sentences.len().min(1)] } else { &sentences };
        for sentence in openings {
            if CAUSE.is_match(sentence) {
                return if flat(sentence) != whole { tidy(sentence) } else { None };
            }
        }
        return None;
    }
    if kind.as_deref() == Some("time") && PLACE.is_match(question) {
        sentences.retain(|one| PLACE.is_match(one) || COOKS.is_match(one));
    }
    if kind.as_deref() == Some("time") && BEFORE.is_match(question) {
        // "how long before i can play a song" asks when the result comes.
        let results: Vec<String> = sentences.iter().filter(|one| RESULT.is_match(one)).cloned().collect();
        for sentence in &results {
            let found: Vec<String> = RESULT.finditer(sentence).iter().map(|m| _figure(m, sentence)).collect();
            let part = found.last().and_then(|f| tidy(f));
            if let Some(part) = part {
                if !part.is_empty() && flat(&part) != whole {
                    return Some(part);
                }
            }
        }
    }
    // The clause of the answer that is about the thing the line names.
    let mut piece = _named_clause(reply, question);
    if let Some(p) = piece.clone() {
        let p = _with_unit(&p, reply);
        piece = Some(p.clone());
        if let Some(k) = kind.as_deref() {
            if spans(k).is_some() && signal(k).is_match(&p) {
                let mut part = match _part(&p, k, question) {
                    Part::Text(t) => Some(t),
                    // A clause of rates holds no length for a whole-job ask.
                    Part::NoLength | Part::Nothing => None,
                };
                if let Some(t) = &part {
                    if !t.is_empty() && _at_another_place(&p, t, question) {
                        part = None;
                    }
                }
                if let Some(t) = part {
                    if !t.is_empty() {
                        if let Some(t) = tidy(&t) {
                            if !t.is_empty() && flat(&t) != whole {
                                return Some(t);
                            }
                        }
                    }
                }
            }
        }
    }
    let Some(kind) = kind else {
        if let Some(often) = _how_often(&sentences, question, &whole) {
            if !often.is_empty() {
                return Some(often);
            }
        }
        if let Some(found) = _not_a_figure(&sentences, question, &whole, reply) {
            if !found.is_empty() {
                return Some(found);
            }
        }
        if let Some(named) = _the_part_they_named(&sentences, question, &whole) {
            if !named.is_empty() {
                return Some(named);
            }
        }
        // A follow-up that names a thing without asking its figure is still about the figure
        // that thing has.
        let source = match piece.as_deref() {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => reply.to_string(),
        };
        for guess in ["time", "money", "distance", "amount", "heat"] {
            if let Some(one) = _the_item_they_named(&source, guess, question) {
                if !one.is_empty() {
                    if let Some(part) = tidy(&one) {
                        if !part.is_empty() && flat(&part) != whole {
                            return Some(part);
                        }
                    }
                }
            }
        }
        if let Some(piece) = piece.as_deref().filter(|p| !p.is_empty()) {
            let fq = flat(question);
            if FOLLOW_UP.is_match(&fq) || YES_NO.is_match(&fq) || ASKS_ABOUT_IT.is_match(&fq) {
                // A yes/no question about a thing the answer names is answered by the clause that
                // speaks of that thing.
                let said = tidy(piece);
                if !_informative(piece) || _cuts_a_list(piece, reply) {
                    let holder: Vec<&String> = sentences.iter().filter(|one| flat(one).contains(&flat(piece))).collect();
                    let longer = holder.first().and_then(|h| tidy(h));
                    if let Some(longer) = longer.filter(|l| !l.is_empty()) {
                        let said_words = n_words(said.as_deref().expect("AttributeError: 'NoneType' object has no attribute 'split'"));
                        if n_words(&longer) > said_words && flat(&longer) != whole {
                            return Some(longer);
                        }
                    }
                }
                if let Some(said) = said {
                    if !said.is_empty() && flat(&said) != whole {
                        return Some(said);
                    }
                }
            }
        }
        return None;
    };
    let wait = WAIT.is_match(question);
    let mut skipped_a_cut_off = false;
    for sentence in &sentences {
        if !signal(&kind).is_match(sentence) {
            continue;
        }
        if let Some(both) = _a_figure_each(sentence, &kind) {
            // Two figures for two things in one sentence: a line that names one of them is
            // asking about that one.
            if let Some(one) = _the_half_they_named(&both, question) {
                if let Some(part) = tidy(&one) {
                    if !part.is_empty() && flat(&part) != whole {
                        return Some(part);
                    }
                }
            }
        }
        if wait && kind == "time" && !WAIT.is_match(sentence) {
            continue;
        }
        let part = match _part(sentence, &kind, question) {
            Part::NoLength => continue, // a sentence of rates: the length is in the next
            Part::Nothing => None,
            Part::Text(t) => Some(t).filter(|t| !t.is_empty()),
        };
        if let Some(p) = &part {
            if _at_another_place(sentence, p, question) {
                continue; // another place's figure
            }
            if _a_length_of_a_doing(sentence, p, question) {
                continue; // how long to keep doing it, not how long the thing lasts
            }
            if _a_cut_off_not_a_length(sentence, p) {
                skipped_a_cut_off = true;
                continue; // the age a rule starts at
            }
            if let Some(p) = tidy(&_as_a_length(p)) {
                if !p.is_empty() && flat(&p) != whole {
                    if let Some(holder) = _the_sentence_a_figure_hangs_on(&p, sentence, &whole, when_ask) {
                        return Some(holder);
                    }
                    return Some(p);
                }
            }
        }
        if when_ask {
            // The answer holds no sentence that says when: its opening is said again rather
            // than nothing.
            return _the_answer_said_again(&sentences, &whole);
        }
        if skipped_a_cut_off {
            continue;
        }
        return None;
    }
    if kind == "time" && (LENGTH_OF_THE_JOB.is_match(&flat(question)) || _STATE_LENGTH_ASKS.is_match(&flat(question))) {
        // A "how long" whose answer states no length at all: say how the time is spent.
        if let Some(said) = _the_time_it_does_keep(&sentences, &whole) {
            if !said.is_empty() {
                return Some(said);
            }
        }
    }
    None
}

/// Python: bot/memory/narrow.py:1438 `_the_start_age(sentence, figure, at)`
///
/// The figure as a start age ("around six months"). `at` is a Python character index into the
/// sentence the figure was found in, applied (as in Python) to the flattened sentence.
pub fn _the_start_age(sentence: &str, figure: &str, at: usize) -> Option<String> {
    let flat_sentence = flat(sentence);
    let from = text::byte_at(&flat_sentence, at.saturating_sub(24));
    let to = text::byte_at(&flat_sentence, at);
    let before = if from <= to { &flat_sentence[from..to] } else { "" };
    if let Some(lead) = A_START_LEAD.search(before) {
        return Some(text::squash(&format!("{} {}", text::strip(lead.as_str()), figure)));
    }
    Some(text::squash(figure))
}

/// Python: bot/memory/narrow.py:1452 `_the_time_it_does_keep(sentences, whole)`
///
/// The sentence of the answer that talks about time, said as the help it is.
pub fn _the_time_it_does_keep(sentences: &[String], whole: &str) -> Option<String> {
    let time = signal("time");
    let mut start_age: Option<String> = None;
    for sentence in sentences {
        if !time.is_match(sentence) {
            continue;
        }
        let mut timed: Vec<Match> = Vec::new();
        let mut last_end: Option<usize> = None;
        for found in time.finditer(sentence) {
            if last_end.is_some_and(|end| found.start() < end) {
                continue; // part of a rate, not its own figure
            }
            last_end = Some(found.end());
            if IN_A_RELATIVE.is_match(&sentence[..found.start()]) {
                continue; // a figure about something else
            }
            if FREQUENCY_AFTER.match_(found.as_str()).is_some()
                && !_a_period_of_extent(sentence, text::char_index(sentence, found.start()))
            {
                continue; // a period word on its own is the "a day" of a rate
            }
            if FREQUENCY_AFTER.match_(&sentence[found.end()..]).is_some() {
                continue; // the period word of a rate
            }
            timed.push(found);
        }
        if timed.is_empty() {
            continue;
        }
        if LENGTH_OF_THE_JOB.is_match(sentence) || RE_DEPENDS.is_match(sentence) {
            continue;
        }
        if start_age.is_none() && timed.iter().all(|one| _a_cut_off_not_a_length(sentence, one.as_str())) {
            // Every figure in this sentence is the age the advice starts at: kept in case the
            // answer has no length at all.
            let one = &timed[0];
            start_age = _the_start_age(&flat(sentence), one.as_str(), text::char_index(sentence, one.start()));
        }
        let mut clause: Option<String> = None;
        for one in RE_CLAUSE_OF_TIME.split(text::strip(sentence)) {
            let one = text::strip_chars(&one, " .,");
            if _only_a_cut_off(one) || !_says_a_time(one) {
                continue;
            }
            clause = Some(one.to_string());
            break;
        }
        let Some(clause) = clause else { continue };
        let words = n_words(&clause);
        if !(3..=24).contains(&words) {
            continue;
        }
        if flat(&clause) == whole {
            continue;
        }
        let mut chars = clause.chars();
        let first: String = chars.next().map(|c| c.to_lowercase().collect()).unwrap_or_default();
        return Some(format!("{}{}{}.", *NO_SET_TIME, first, chars.as_str()));
    }
    if let Some(start_age) = start_age.filter(|s| !s.is_empty()) {
        // The answer has no length in it at all: this much is where it starts.
        return Some(A_STAGE[0].replace("{when}", &start_age));
    }
    None
}

/// Python: bot/memory/narrow.py:1516 `_a_period_of_extent(sentence, at)`
///
/// Whether the time at `at` (a character index into `sentence`) is introduced by a word of
/// extent ("for a month"). As in Python, the index is applied to the flattened sentence.
pub fn _a_period_of_extent(sentence: &str, at: usize) -> bool {
    let said = flat(sentence);
    _A_PERIOD_OF_EXTENT.is_match(chars_before(&said, at))
}

/// Python: bot/memory/narrow.py:1521 `_says_a_time(clause)`
///
/// Whether this clause says anything about time at all.
pub fn _says_a_time(clause: &str) -> bool {
    signal("time").is_match(clause) || RE_SAYS_A_TIME.is_match(clause)
}

/// Python: bot/memory/narrow.py:1533 `_only_a_cut_off(clause)`
///
/// Whether every figure in this clause is the age a rule starts at, and none is a length.
pub fn _only_a_cut_off(clause: &str) -> bool {
    let figures: Vec<String> = signal("time").finditer(clause).iter().map(|m| m.as_str().to_string()).collect();
    if figures.is_empty() {
        return false;
    }
    figures.iter().all(|figure| _a_cut_off_not_a_length(clause, figure))
}

/// Python: bot/memory/narrow.py:1546 `_the_answer_said_again(sentences, whole)`
///
/// The sentence an answer opens with, when the answer has more to it than that.
pub fn _the_answer_said_again(sentences: &[String], whole: &str) -> Option<String> {
    if sentences.len() < 2 {
        return None;
    }
    let opening = tidy(&sentences[0])?;
    if opening.is_empty() || flat(&opening) == whole || n_words(&opening) > 30 {
        return None;
    }
    Some(opening)
}

/// Python: bot/memory/narrow.py:1568 `_the_sentence_a_figure_hangs_on(part, sentence, whole, when_ask=False)`
pub fn _the_sentence_a_figure_hangs_on(part: &str, sentence: &str, whole: &str, when_ask: bool) -> Option<String> {
    // Only a when ask: a length is a figure by design.
    if !when_ask {
        return None;
    }
    if A_WHEN_WORD.is_match(part) || n_words(part) > 4 {
        return None;
    }
    let holder = tidy(sentence)?;
    if holder.is_empty() || flat(&holder) == whole || n_words(&holder) > 25 {
        return None;
    }
    if !flat(&holder).contains(text::strip_chars(&flat(part), " .,;:!?")) {
        return None;
    }
    Some(holder)
}

/// Python: bot/memory/narrow.py:1621 `_unit_stem(word)`
///
/// The thing a unit word counts: "mins", "minute" and "minutes" are all a minute.
pub fn _unit_stem(word: &str) -> String {
    let bare = text::lower(text::strip_chars(word, ".,;:!?"));
    let bare = text::rstrip_chars(&bare, "s");
    _UNITS.get(bare).cloned().unwrap_or_else(|| bare.to_string())
}

/// Python: bot/memory/narrow.py:1638 `_same_word(one, other)`
///
/// Whether two words are the same word or one is the other with an ending on it.
pub fn _same_word(one: &str, other: &str) -> bool {
    if one == other {
        return true;
    }
    let short = if one.chars().count().min(other.chars().count()) >= 4 { 4 } else { 3 };
    one.chars().take(short).eq(other.chars().take(short))
}

/// Python: bot/memory/narrow.py:1646 `_names_one_of(wanted, words)`
pub fn _names_one_of(wanted: &HashSet<String>, words: &[String]) -> bool {
    wanted.iter().any(|word| words.iter().any(|said| _same_word(word, said)))
}

/// Python: bot/memory/narrow.py:1679 `_only_a_count(piece)`
///
/// True when the only rate words in the piece are counts with no period beside them.
pub fn _only_a_count(piece: &str) -> bool {
    let found = FREQUENCY.finditer(piece);
    if found.is_empty() {
        return false;
    }
    found.iter().all(|m| {
        A_BARE_COUNT.match_(m.as_str()).is_some()
            || ["once", "twice", "thrice"].contains(&text::strip(&text::lower(m.as_str())))
    }) && !RATE_INTERVAL.is_match(piece)
}

/// Python: bot/memory/narrow.py:1732 `_how_often(sentences, question, whole)`
///
/// The clause of the answer that says how often, when the line asks that.
pub fn _how_often(sentences: &[String], question: &str, whole: &str) -> Option<String> {
    if !ASKS_FREQUENCY.is_match(&flat(question)) {
        return None;
    }
    let wanted: HashSet<String> = words_of(question)
        .into_iter()
        .filter(|word| !NAMING_IGNORE.contains(word) && !FREQUENCY_IGNORE.contains(word) && word.chars().count() > 2)
        .collect();
    let nothing: HashSet<String> = HashSet::new();
    let mut rates: Vec<String> = Vec::new();
    let mut counted: Option<String> = None;
    for sentence in sentences {
        let about = !wanted.is_empty() && _names_one_of(&wanted, &words_of(sentence));
        let clause = _freq_clause(sentence, about || wanted.is_empty(), if about { &wanted } else { &nothing });
        let Some(clause) = clause.filter(|c| !c.is_empty()) else { continue };
        if !wanted.is_empty() && !about {
            continue;
        }
        let Some(said) = tidy(&clause) else { continue };
        if said.is_empty() || flat(&said) == whole {
            continue;
        }
        if FREQUENCY.is_match(&flat(&said)) && !_only_a_count(&said) {
            return Some(said); // "twice a year", "every other morning": the answer itself
        }
        if _only_a_count(&said) {
            if counted.is_none() {
                counted = Some(said);
            }
            continue;
        }
        rates.push(said);
    }
    for said in &rates {
        if _only_a_count(said) {
            if counted.is_none() {
                counted = Some(said.clone());
            }
            continue;
        }
        if !(FREQ_LIMIT.is_match(said) || FREQ_PORTION.is_match(said)) {
            return Some(said.clone());
        }
    }
    if counted.is_some() {
        return counted;
    }
    if let Some(first) = rates.first() {
        // A limit or a portion, when the answer has nothing better.
        return Some(first.clone());
    }
    // No rate anywhere, but a sentence that says when to do it again.
    for sentence in sentences {
        if TRIGGER.is_match(sentence) && RE_AGAIN.is_match(sentence) {
            if let Some(said) = tidy(sentence) {
                if !said.is_empty() && flat(&said) != whole {
                    return Some(said);
                }
            }
        }
    }
    // Nothing in the answer says how often: say so, but only for an answer that really is a set
    // of steps, and only for the bare ask or the ask about the answer's own opening doing.
    let opening = sentences.first().map(|s| text::strip(s).to_string()).unwrap_or_default();
    let first_word = text::split_ref(&opening)
        .first()
        .map(|w| text::lower(text::strip_chars(w, ",.;:")))
        .unwrap_or_default();
    let the_doing = A_HOW_OFTEN_THE_DOING.is_match(&flat(question))
        && !wanted.is_empty()
        && !first_word.is_empty()
        && _names_one_of(&wanted, &[first_word.clone()]);
    if (ONLY_THE_INTERVAL_ASK.match_(question).is_some() || the_doing)
        && !RATE_INTERVAL.is_match(whole)
        && A_SET_OF_STEPS.is_match(whole)
    {
        return Some(NO_INTERVAL_SAID.clone());
    }
    rates.first().cloned()
}

/// Python: bot/memory/narrow.py:1828 `_counts_on_the_answer(piece)`
///
/// Whether the clause counts something the answer already said rather than naming its rate.
pub fn _counts_on_the_answer(piece: &str) -> bool {
    RE_COUNTS_ON.is_match(piece)
}

/// Python: bot/memory/narrow.py:1833 `_freq_clause(sentence, weak=False, wanted=frozenset())`
///
/// The clause of one sentence that carries the frequency, or None.
pub fn _freq_clause(sentence: &str, weak: bool, wanted: &HashSet<String>) -> Option<String> {
    let pieces: Vec<String> = FREQ_CLAUSE
        .split(sentence)
        .iter()
        .map(|piece| text::strip_chars(piece, " ,;:").to_string())
        .filter(|piece| !piece.is_empty())
        .collect();
    let mut found: Vec<String> =
        pieces.into_iter().filter(|piece| FREQUENCY.is_match(piece) || (weak && RATE_FREQ.is_match(piece))).collect();
    if found.is_empty() {
        return None;
    }
    if n_words(sentence) <= 30 {
        // A sentence short enough to say back whole is said whole.
        return Some(sentence.to_string());
    }
    if !wanted.is_empty() {
        // The clause that is said back has to name the thing asked about too.
        found.retain(|piece| _names_one_of(wanted, &words_of(piece)));
        if found.is_empty() {
            return None;
        }
    }
    // A clause that counts something said earlier in the answer paints no picture on its own.
    let plain: Vec<String> = {
        let p: Vec<String> = found.iter().filter(|piece| !_counts_on_the_answer(piece)).cloned().collect();
        if p.is_empty() { found.clone() } else { p }
    };
    let mut piece = plain[0].clone();
    if plain.len() > 1 && RE_JOINER_OPENS.match_(&piece).is_some() {
        piece = plain[1].clone();
    }
    if n_words(&piece) <= 3 && found.len() > 1 {
        piece = found[found.len() - 2..].join(" ");
    }
    if n_words(&piece) > 14 {
        // Too long to say back whole: keep the part of it that names the frequency.
        let parts: Vec<String> = RE_COMMA_SPACE
            .split(&piece)
            .iter()
            .map(|part| text::strip_chars(part, " ,;:").to_string())
            .filter(|part| !part.is_empty())
            .collect();
        let at: Vec<usize> = parts.iter().enumerate().filter(|(_, part)| FREQUENCY.is_match(part)).map(|(i, _)| i).collect();
        if let Some(&last) = at.last() {
            piece = parts[last].clone();
            if n_words(&piece) <= 5 && last > 0 {
                piece = format!("{}, {}", parts[last - 1], piece);
            }
        }
    }
    if n_words(&piece) >= 2 { Some(piece) } else { None }
}

/// Python: bot/memory/narrow.py:1880 `_dated(sentences, question, whole)`
///
/// The sentence of the answer that says when, picking between dates by the words used.
pub fn _dated(sentences: &[String], question: &str, whole: &str) -> Option<String> {
    let mut wanted: HashSet<String> = word_set(question);
    for word in wanted.clone() {
        wanted.extend(same_as(&word));
    }
    let mut best: Option<&String> = None;
    let mut best_score: (i64, i64) = (-1, -1);
    for sentence in sentences {
        if !has_a_date(sentence) {
            continue;
        }
        let shared = words_of(sentence).iter().filter(|w| wanted.contains(*w)).collect::<HashSet<_>>().len() as i64;
        let score = (if DATED_STRONG.is_match(sentence) { 1 } else { 0 }, shared);
        if score > best_score {
            best = Some(sentence);
            best_score = score;
        }
    }
    let best = best?;
    if flat(best) == whole {
        return None;
    }
    tidy(best)
}

/// Python: bot/memory/narrow.py:1911 `_the_part_they_named(sentences, question, whole)`
///
/// The sentence of the answer about the thing this advice ask names, or None.
pub fn _the_part_they_named(sentences: &[String], question: &str, whole: &str) -> Option<String> {
    if !ASKS_FOR_A_HAND.is_match(&flat(question)) {
        return None;
    }
    let sentence = _by_words(sentences, question)?;
    let shared = word_set(question).intersection(&word_set(&sentence)).count();
    if shared < *NAMES_A_PART {
        return None;
    }
    let said = tidy(&sentence)?;
    if !said.is_empty() && flat(&said) != whole {
        return Some(said);
    }
    None
}

/// Python: bot/memory/narrow.py:1927 `_not_a_figure(sentences, question, whole, reply=None)`
///
/// The sentence that answers a step or a property question, when it is not a figure.
pub fn _not_a_figure(sentences: &[String], question: &str, whole: &str, reply: &str) -> Option<String> {
    if let Some(option) = _by_option(sentences, question).filter(|o| !o.is_empty()) {
        if flat(&option) != whole {
            return _with_the_script(&option, sentences, reply).filter(|s| !s.is_empty()).or_else(|| tidy(&option));
        }
    }
    if let Some(step) = _by_step(sentences, question).filter(|s| !s.is_empty()) {
        if flat(&step) != whole {
            return _with_the_script(&step, sentences, reply).filter(|s| !s.is_empty()).or_else(|| tidy(&step));
        }
    }
    if _property_ask(question) {
        if let Some(sentence) = _by_words(sentences, question).filter(|s| !s.is_empty()) {
            if flat(&sentence) != whole {
                return tidy(&sentence);
            }
        }
    }
    None
}

/// Python: bot/memory/narrow.py:1953 `_named_clause(answer, question)`
///
/// The clause of the answer about a thing the line names, or None.
pub fn _named_clause(answer: &str, question: &str) -> Option<String> {
    let named: Vec<String> =
        words_of(question).into_iter().filter(|word| !NAMING_IGNORE.contains(word) && word.chars().count() > 2).collect();
    if named.is_empty() {
        return None;
    }
    let pairs: Vec<String> = named.windows(2).map(|pair| format!("{} {}", pair[0], pair[1])).collect();
    let squashed = text::squash(answer);
    // Python cuts the answer this way first and then replaces the pieces with `clauses()`; the
    // first cut has no effect on the result.
    let _first_cut: Vec<String> = pieces_of(&squashed, &RE_NAMED_PIECES)
        .iter()
        .map(|piece| text::strip(piece).to_string())
        .filter(|piece| !piece.is_empty())
        .collect();
    let roots: HashSet<String> = named.iter().map(|word| _root(word)).collect();
    let pieces = clauses(&squashed, &NAMED_BREAK);
    let mut best: Option<String> = None;
    let mut best_score = 0usize;
    let mut best_count = 0usize;
    for piece in pieces {
        let said = flat(&piece);
        let mut score = if pairs.is_empty() { 0 } else { pairs.iter().filter(|pair| _says(&said, pair)).count() };
        if score == 0 {
            // The same word said another way: plural and verb endings.
            let piece_roots: HashSet<String> = words_of(&piece).iter().map(|word| _root(word)).collect();
            score = piece_roots.intersection(&roots).count();
        }
        if score > best_score {
            best = Some(piece);
            best_score = score;
            best_count = 1;
        } else if score == best_score && score > 0 {
            best_count += 1;
        }
    }
    let best = best.filter(|b| !b.is_empty())?;
    if best_count != 1 {
        return None;
    }
    // Every thing the line names has to be spoken of by the answer.
    let answer_roots: HashSet<String> = words_of(answer).iter().map(|word| _root(word)).collect();
    if !roots.is_subset(&answer_roots) {
        return None;
    }
    Some(text::rstrip_chars(&best, ".!?,;: ").to_string())
}

/// Python: bot/memory/narrow.py:2000 `_root(word)`
///
/// The stem a word shares with its other endings ("cleaning", "cleaned" -> "clean").
pub fn _root(word: &str) -> String {
    let mut word = word.to_string();
    for ending in ["ing", "ed", "es", "s"] {
        if word.chars().count() > ending.len() + 2 && word.ends_with(ending) {
            word.truncate(word.len() - ending.len());
            break;
        }
    }
    if word.chars().count() > 3 && word.ends_with('e') {
        word.pop();
    }
    word
}

/// Python: bot/memory/narrow.py:2009 `_says(text, phrase)`
///
/// Whether `text` says `phrase`, plurals included ("loaf tin" in "loaf tins").
pub fn _says(text_: &str, phrase: &str) -> bool {
    let words: Vec<String> = text::split(phrase).iter().map(|word| pyre::escape(word)).collect();
    let pattern = format!(r"\b{}[a-z]*\b", words.join(r"[a-z]*\s+"));
    re_cached(&pattern, 0).is_match(&flat(text_))
}

/// Python: bot/memory/narrow.py:2015 `_informative(piece)`
///
/// Whether this clause answers on its own: a figure in it, or a step to take.
pub fn _informative(piece: &str) -> bool {
    RE_DIGIT.is_match(piece) || COOKS.is_match(piece) || WAIT.is_match(piece)
}

/// Python: bot/memory/narrow.py:2020 `_cuts_a_list(piece, answer)`
///
/// Whether this clause is one item of a list the answer carries on with.
pub fn _cuts_a_list(piece: &str, answer: &str) -> bool {
    let said = text::squash(piece);
    let whole = text::squash(answer);
    if said.is_empty() {
        return false;
    }
    let Some(at) = whole.find(&said) else { return false };
    let rest = &whole[at + said.len()..];
    RE_LIST_GOES_ON.match_(rest).is_some()
}

/// Python: bot/memory/narrow.py:2036 `_with_unit(piece, reply)`
///
/// A clause that ends on a bare range gets the unit the answer used before it.
pub fn _with_unit(piece: &str, reply: &str) -> String {
    let text_ = text::strip(piece);
    if !BARE_RANGE.is_match(text_) || UNIT.is_match(last_chars(text_, 12)) {
        return text_.to_string();
    }
    let before = flat(reply);
    let looked = match before.find(&flat(text_)) {
        None => &before[..],
        Some(at) => &before[..at],
    };
    let units = UNIT.finditer(looked);
    match units.last() {
        None => text_.to_string(),
        Some(unit) => format!("{} {}", text_, unit.as_str()),
    }
}

/// Python: bot/memory/narrow.py:2053 `_property_ask(question)`
///
/// Whether the line asks how the thing turns out, in words the answer may say differently.
pub fn _property_ask(question: &str) -> bool {
    if !RE_HOW_DO_I.is_match(&flat(question)) {
        return false;
    }
    if OWN_SAVING.is_match(&flat(question)) {
        return false;
    }
    words_of(question).iter().any(|word| in_same(word))
}

/// Python: bot/memory/narrow.py:2075 `_option_ask(question)`
///
/// Whether the line is an elliptical ask about one named option ("and for a hard one?").
pub fn _option_ask(question: &str) -> bool {
    let clean = flat(question);
    OPTION.is_match(&clean) && QUALIFIER.is_match(&clean) && !NOT_OPTION.is_match(&clean) && words_of(&clean).len() <= 8
}

/// Python: bot/memory/narrow.py:2082 `_stands_alone(piece)`
///
/// Whether a clause cut out of an answer is an answer on its own, not half a sentence.
pub fn _stands_alone(piece: &str) -> bool {
    let text_ = text::strip(piece);
    if text_.is_empty() {
        return false;
    }
    if RE_DIGIT.is_match(text_) {
        return true;
    }
    words_of(text_).len() >= 2
}

/// Python: bot/memory/narrow.py:2099 `_by_option(sentences, question)`
///
/// The clause of the sentence that names the option the line asks about.
pub fn _by_option(sentences: &[String], question: &str) -> Option<String> {
    let qualifier = QUALIFIER.search(question)?;
    for sentence in sentences {
        if let Some(clause) = _clause_with(sentence, qualifier.as_str()).filter(|c| !c.is_empty()) {
            if flat(&clause) != flat(sentence) {
                if _only_a_qualifier(&clause) {
                    // a clause of nothing but the qualifier is the condition the sentence is
                    // about, not an option named
                    return Some(sentence.clone());
                }
                return Some(if _stands_alone(&clause) { clause } else { sentence.clone() });
            }
        }
    }
    None
}

/// Python: bot/memory/narrow.py:2118 `_only_a_qualifier(clause)`
///
/// Whether a clause cut out of an answer names nothing but the qualifier the line asked for.
pub fn _only_a_qualifier(clause: &str) -> bool {
    let words: Vec<String> = words_of(clause).into_iter().filter(|word| !QUALIFIER_FILLER.contains(word)).collect();
    !words.is_empty() && words.iter().all(|word| QUALIFIER_WORDS.contains(word))
}

/// Python: bot/memory/narrow.py:2138 `_by_step(sentences, question)`
///
/// The clause that starts with the step a yes/no question names, or None.
pub fn _by_step(sentences: &[String], question: &str) -> Option<String> {
    if !YES_NO.is_match(&flat(question)) {
        return None;
    }
    let mut wanted: HashSet<String> = word_set(question);
    // The step is the verb the question names, when it names one.
    if let Some(step) = step_word(question).filter(|s| !s.is_empty()) {
        wanted = HashSet::from([step.clone()]);
        wanted.extend(same_as(&step));
    }
    for sentence in sentences {
        // The sentence must *start* with the step.
        let raw = WORD.findall_str(&flat(sentence));
        let Some(first) = raw.first() else { continue };
        let mut first = first.clone();
        if first.chars().count() > 3 && first.ends_with('s') && !first.ends_with("ss") {
            first.pop();
        }
        if wanted.contains(&first) {
            let stripped = text::strip(sentence);
            let head = RE_STEP_CUT.split(stripped).into_iter().next().unwrap_or_default();
            let head = text::strip(&head);
            let cut = if head.is_empty() { sentence.as_str() } else { head };
            return Some(if _stands_alone(cut) { cut.to_string() } else { stripped.to_string() });
        }
    }
    None
}

/// Python: bot/memory/narrow.py:2164 `_with_the_script(part, sentences, reply=None)`
///
/// A bare verdict on the saying, with the words the person would use, and nothing else.
pub fn _with_the_script(part: &str, sentences: &[String], reply: &str) -> Option<String> {
    let head = text::strip_chars(text::strip(part), ".?! ");
    let said = flat(head);
    if said.is_empty() || said.chars().count() > *BAREST_VERDICT || !said.contains(' ') {
        return None;
    }
    if SAYS_SOMETHING.match_(&said).is_none() {
        return None;
    }
    // The answer's own sentences, not the ones the caller kept.
    let mut whole_answer: Vec<String> = SENTENCE
        .split(text::strip(reply))
        .iter()
        .map(|one| text::strip(one).to_string())
        .filter(|one| !one.is_empty())
        .collect();
    if whole_answer.is_empty() {
        whole_answer = sentences.to_vec();
    }
    for (at, sentence) in whole_answer.iter().enumerate() {
        if flat(sentence).starts_with(&said) {
            let words = _the_quotation(&_to_the_end_of_the_script(&whole_answer, at));
            return match words {
                Some(words) if !words.is_empty() => Some(format!("{}, like this: {}", head, words)),
                _ => None,
            };
        }
    }
    None
}

/// Python: bot/memory/narrow.py:2208 `_the_quotation(sentence)`
///
/// The words quoted in the sentence, marks and all, when it quotes any.
pub fn _the_quotation(sentence: &str) -> Option<String> {
    if let Some(found) = QUOTED_WORDS.search(sentence) {
        return Some(text::strip(found.as_str()).to_string());
    }
    if let Some(found) = QUOTED_WORDS_SINGLE.search(sentence) {
        return Some(format!("'{}'", text::strip(found.str(1))));
    }
    None
}

/// Python: bot/memory/narrow.py:2220 `_to_the_end_of_the_script(sentences, at)`
///
/// The sentence, plus what the splitter took for the next one when it broke inside a quote.
pub fn _to_the_end_of_the_script(sentences: &[String], at: usize) -> String {
    let mut said = text::strip(&sentences[at]).to_string();
    let mut index = at;
    while index + 1 < sentences.len() && text::count(&said, "\"") % 2 == 1 {
        index += 1;
        said = format!("{} {}", said, text::strip(&sentences[index]));
    }
    said
}

/// Python: bot/memory/narrow.py:2237 `_by_words(sentences, question)`
///
/// The sentence that shares the most words with the question, when it shares any.
pub fn _by_words(sentences: &[String], question: &str) -> Option<String> {
    let mut wanted: HashSet<String> = word_set(question);
    for word in wanted.clone() {
        wanted.extend(same_as(&word));
    }
    if wanted.is_empty() {
        return None;
    }
    let mut best: Option<&String> = None;
    let mut best_score = 0usize;
    let mut best_length = 0usize;
    for sentence in sentences {
        let words = words_of(sentence);
        let score = words.iter().filter(|w| wanted.contains(*w)).count();
        let length = words.len();
        if score > 0 && (score > best_score || (score == best_score && length < best_length)) {
            best = Some(sentence);
            best_score = score;
            best_length = length;
        }
    }
    best.cloned()
}

/// Python: bot/memory/narrow.py:2260 `_as_a_length(text)`
///
/// `a 20-minute job` read as `20 minutes`: the answer to how long is the time itself.
pub fn _as_a_length(text_: &str) -> String {
    DURATION_ADJECTIVE.sub_fn(text_, |found| {
        let number = found.str(1);
        let mut unit = text::lower(found.str(2));
        if number != "1" && number != "1.0" && !unit.ends_with('s') {
            unit.push('s');
        }
        format!("{} {}", number, unit)
    })
}

/// Python: bot/memory/narrow.py:2279 `_names_a_thing(question)`
///
/// Whether the line names something of its own beyond the property it asks for.
pub fn _names_a_thing(question: &str) -> bool {
    words_of(&flat(question)).iter().any(|word| !ASKING_ONLY.contains(word) && !POINTS_BACK.contains(word))
}

/// Python: bot/memory/narrow.py:2302 `_measure(span)`
///
/// The unit a figure is given in, or "" when it names none.
pub fn _measure(span_: &str) -> String {
    FIGURE_MEASURE.search(span_).map(|found| text::lower(found.str(1))).unwrap_or_default()
}

/// Python: bot/memory/narrow.py:2308 `_figure(match, text)`
///
/// The whole figure a span found, thousands separator and all.
pub fn _figure(found: &Match, text_: &str) -> String {
    let start = found.start();
    let mut left = start;
    while left > 0 {
        let c = text_[..left].chars().next_back().unwrap();
        if pyuni::ch_digit(c) || c == ',' || c == '.' {
            left -= c.len_utf8();
        } else {
            break;
        }
    }
    if left == start {
        return found.as_str().to_string();
    }
    text::lstrip_chars(&text_[left..found.end()], ".,").to_string()
}

/// Python: bot/memory/narrow.py:2329 `_a_figure_each(sentence, kind)`
///
/// The halves of a sentence that each state a figure for a thing, or None.
pub fn _a_figure_each(sentence: &str, kind: &str) -> Option<Vec<String>> {
    let figure = spans(kind)?;
    if !sentence.contains(';') {
        return None;
    }
    let halves: Vec<String> =
        sentence.split(';').map(|one| text::strip(one).to_string()).filter(|one| !one.is_empty()).collect();
    if halves.len() < 2 {
        return None;
    }
    // The second half of a pair gives its figure without the unit: a bare range counts when one
    // of the halves states the unit.
    let stated = halves.iter().any(|one| figure.is_match(one));
    if !stated {
        return None;
    }
    if halves.iter().any(|one| !(figure.is_match(one) || BARE_RANGE.is_match(text::rstrip_chars(one, ".!?")))) {
        return None;
    }
    Some(halves)
}

/// Python: bot/memory/narrow.py:2373 `_the_item_they_named(sentence, kind, question)`
///
/// The item of a one-figure-per-thing list that the line names, or None.
pub fn _the_item_they_named(sentence: &str, kind: &str, question: &str) -> Option<String> {
    let figure = spans(kind)?;
    let said = word_set(&flat(question));
    if said.is_empty() {
        return None;
    }
    // A sentence that opens with what the list is for says it before the colon.
    let body = match sentence.split_once(':') {
        Some((_, after)) => after,
        None => sentence,
    };
    let starts = re_cached(&format!("{}|{}", figure.pattern(), A_BARE_RANGE.pattern()), pyre::I);
    let found: Vec<Match> = starts.finditer(body).into_iter().filter(|m| !_a_frequency(body, m.end(), None)).collect();
    if found.len() < 2 {
        return None;
    }
    let mut items: Vec<(&str, &str)> = Vec::new();
    for (index, m) in found.iter().enumerate() {
        let stop = if index + 1 < found.len() { found[index + 1].start() } else { body.len() };
        items.push((m.as_str(), &body[m.start()..stop]));
    }
    // Every item of such a list says what its figure is for.
    let reads_as_a_list = items.iter().all(|(span_, piece)| {
        let at = span_.len().min(piece.len());
        A_FOR_A_THING.search_at(piece, at).is_some_and(|m| m.start() == at)
    });
    if !reads_as_a_list {
        return None;
    }
    let scored: Vec<(usize, &str)> =
        items.iter().map(|(_, piece)| (word_set(piece).intersection(&said).count(), *piece)).collect();
    let best = scored.iter().map(|(score, _)| *score).max().unwrap_or(0);
    let winners: Vec<&str> = scored.iter().filter(|(score, _)| *score == best).map(|(_, piece)| *piece).collect();
    if best == 0 || winners.len() != 1 {
        return None;
    }
    let mut item = RE_TRAILING_JOIN.sub("", text::strip(winners[0]));
    if !A_UNIT_WORD.is_match(&item) {
        // The unit is stated once, at the top of the list.
        let unit = items.iter().find_map(|(_, one)| A_UNIT_WORD.search(one).map(|m| m.as_str().to_string()))?;
        let split = A_BARE_RANGE.search(&item).map(|head| head.end()).unwrap_or(item.len());
        item = format!("{} {}{}", &item[..split], text::lower(&unit), &item[split..]);
    }
    Some(item)
}

/// Python: bot/memory/narrow.py:2427 `_the_half_they_named(halves, question)`
///
/// The half of a two-answer sentence the line names, or None when it names neither.
pub fn _the_half_they_named(halves: &[String], question: &str) -> Option<String> {
    let said = word_set(&flat(question));
    if said.is_empty() {
        return None;
    }
    let scored: Vec<(usize, &String)> = halves.iter().map(|one| (word_set(one).intersection(&said).count(), one)).collect();
    let best = scored.iter().map(|(score, _)| *score).max()?;
    let winners: Vec<&String> = scored.iter().filter(|(score, _)| *score == best).map(|(_, one)| *one).collect();
    if best == 0 || winners.len() != 1 {
        return None;
    }
    Some(winners[0].clone())
}

/// Python: bot/memory/narrow.py:2478 `_a_condition(sentence, match)`
///
/// True when the figure (at `start..end` of the sentence) counts something other than the
/// length of the whole job.
pub fn _a_condition(sentence: &str, start: usize, end: usize) -> bool {
    if AT_A_TIME.match_(&sentence[end..]).is_some() {
        return true;
    }
    if A_RELATIVE_AFTER.match_(&sentence[end..]).is_some() {
        return true;
    }
    let before = &sentence[..start];
    if IN_A_CONDITION.is_match(before) && A_BOUNDED.is_match(before) {
        return true;
    }
    if A_STATE_FOR.is_match(before) {
        return true;
    }
    if A_WEATHER_WINDOW.is_match(before) {
        return true;
    }
    IN_A_RELATIVE.is_match(before)
}

/// What `_part` hands back: a figure or clause, nothing, or `NO_LENGTH` (a sentence whose every
/// figure is a rate, which Python marks with a falsey sentinel).
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Nothing,
    NoLength,
    Text(String),
}

/// Python: bot/memory/narrow.py:2494 `_part(sentence, kind, question)`
///
/// The tightest safe answer inside one sentence.
pub fn _part(sentence: &str, kind: &str, question: &str) -> Part {
    let figure = spans(kind);
    let fq = flat(question);
    if let Some(qualifier) = QUALIFIER.search(question) {
        if let Some(clause) = _clause_with(sentence, qualifier.as_str()).filter(|c| !c.is_empty()) {
            let found: Vec<String> = match figure {
                Some(f) => f.finditer(&clause).iter().map(|m| _figure(m, &clause)).collect(),
                None => Vec::new(),
            };
            if let Some(last) = found.last() {
                return Part::Text(last.clone());
            }
            if RE_DIGIT.is_match(&clause) {
                return Part::Text(clause);
            }
        }
    }
    let mut sentence: String = sentence.to_string();
    let mut spans_: Vec<String> = match figure {
        Some(f) => f.finditer(&sentence).iter().map(|m| _figure(m, &sentence)).collect(),
        None => Vec::new(),
    };
    if kind == "time" && !spans_.is_empty() && LENGTH_OF_THE_JOB.is_match(&fq) {
        let f = figure.unwrap();
        let mut lengths: Vec<String> = f
            .finditer(&sentence)
            .iter()
            .filter(|m| {
                !_a_frequency(&sentence, m.end(), Some(m.start()))
                    && !(A_MOMENT.fullmatch(text::strip(m.as_str())).is_some() && !A_COUNTDOWN.is_match(&fq))
                    && !_a_condition(&sentence, m.start(), m.end())
            })
            .map(|m| _figure(m, &sentence))
            .collect();
        if lengths.is_empty() {
            // Every figure in the sentence is a rate: the next sentence may say the length.
            return Part::NoLength;
        }
        // A rate that opens the sentence is not part of a whole-job answer.
        let lead = RATE_LEAD.match_(&sentence).map(|lead| (f.is_match(lead.as_str()), lead.end()));
        if let Some((true, lead_end)) = lead {
            {
                let cut = text::strip(&sentence[lead_end..]).to_string();
                if !cut.is_empty() && f.is_match(&cut) {
                    sentence = cut;
                    lengths = f
                        .finditer(&sentence)
                        .iter()
                        .filter(|m| !_a_frequency(&sentence, m.end(), Some(m.start())))
                        .map(|m| _figure(m, &sentence))
                        .collect();
                }
            }
        }
        spans_ = lengths;
    }
    if kind == "time" && !spans_.is_empty() && !LENGTH_OF_THE_JOB.is_match(&fq) {
        // A rate ask wants the rate.
        let f = figure.unwrap();
        let all = f.finditer(&sentence);
        let rates: Vec<String> =
            all.iter().filter(|m| _a_frequency(&sentence, m.end(), Some(m.start()))).map(|m| _figure(m, &sentence)).collect();
        let lengths: Vec<String> =
            all.iter().filter(|m| !_a_frequency(&sentence, m.end(), Some(m.start()))).map(|m| _figure(m, &sentence)).collect();
        if !rates.is_empty() && !lengths.is_empty() {
            return Part::Text(rates[0].clone());
        }
    }
    if spans_.is_empty() {
        // A clock ask needs a clock in it.
        if kind == "clock" {
            return Part::Nothing;
        }
        return if n_words(&sentence) <= 28 { Part::Text(sentence) } else { Part::Nothing };
    }
    // "how many weeks should i train" names the unit it wants.
    if let Some(wanted_unit) = ASKED_UNIT.search(question) {
        if kind == "time" && spans_.len() > 1 {
            let unit = _unit_stem(wanted_unit.str(1));
            let same: Vec<&String> = spans_
                .iter()
                .filter(|s| _unit_stem(text::split_ref(s).last().copied().unwrap_or("")) == unit)
                .collect();
            if same.len() == 1 {
                return Part::Text(same[0].clone());
            }
        }
    }
    if kind == "amount" {
        return Part::Text(_clause(&sentence, question).filter(|c| !c.is_empty()).unwrap_or(sentence));
    }
    if kind == "money" || kind == "distance" {
        return Part::Text(spans_[0].clone());
    }
    if RATE.is_match(question) {
        let wanted: Vec<&String> = spans_.iter().filter(|s| RE_PER_OR_A.is_match(s)).collect();
        if wanted.is_empty() {
            return Part::Nothing;
        }
        // The rate worked into a total: keep the sentence.
        if spans_.len() > 1 && RE_TAKES_OR_SO.is_match(&sentence) {
            return Part::Text(sentence);
        }
        return Part::Text(wanted[0].clone());
    }
    // Several values given "for" different options: one of them would be a guess.
    if RE_OPTIONS.findall(&sentence).len() >= 2 {
        return Part::Nothing;
    }
    if kind == "time" && BEFORE.is_match(question) {
        let result: Vec<String> = RESULT.finditer(&sentence).iter().map(|m| _figure(m, &sentence)).collect();
        if let Some(last) = result.last() {
            return Part::Text(last.clone());
        }
    }
    // An answer that leads with the figure and then details it answers "how long" in that
    // opening clause.
    if spans_.len() > 1 && !SEQUENCE.is_match(&sentence) {
        let lead = RE_LEAD_CUT.split(text::strip(&sentence)).into_iter().next().unwrap_or_default();
        let lead_spans: Vec<String> = match spans(kind) {
            Some(f) => f.finditer(&lead).iter().map(|m| m.as_str().to_string()).collect(),
            None => panic!("AttributeError: 'tuple' object has no attribute 'finditer'"),
        };
        let rest: Vec<&String> = spans_.iter().filter(|s| !lead_spans.contains(s)).collect();
        if lead_spans.len() == 1 && rest.iter().any(|s| _measure(s) != _measure(&lead_spans[0])) {
            return Part::Text(lead_spans[0].clone());
        }
    }
    if spans_.len() == 1 {
        return Part::Text(spans_[0].clone());
    }
    if let Some(stepped) = _step_time(&sentence, question, figure).filter(|s| !s.is_empty()) {
        return Part::Text(stepped);
    }
    if SEQUENCE.is_match(&sentence) && !WAIT.is_match(question) {
        // "first this, then that" is read as steps: the last figure is the answer.
        return Part::Text(spans_.last().unwrap().clone());
    }
    if !_names_a_thing(question) && CARRIES_ON.is_match(question) && !RE_PER.is_match(&spans_[0]) && !EITHER_OR.is_match(&sentence)
    {
        // The follow-up asks for the figure again and nothing else: the clause carrying it.
        return Part::Text(_clause_holding(&sentence, &spans_[0]).filter(|c| !c.is_empty()).unwrap_or_else(|| spans_[0].clone()));
    }
    // Several times for different sizes or tins: hand back the sentence.
    Part::Text(_clause(&sentence, question).filter(|c| !c.is_empty()).unwrap_or(sentence))
}

/// Python: bot/memory/narrow.py:2631 `_canonical(word)`
///
/// The step a kitchen word names, so the oven is the oven however the person says it.
pub fn _canonical(word: &str) -> String {
    if OVEN.contains(word) { "oven".to_string() } else { word.to_string() }
}

/// Python: bot/memory/narrow.py:2636 `_a_frequency(sentence, end, start=None)`
///
/// Whether the time that ends at `end` (and starts at `start`; byte offsets into `sentence`)
/// is how often to do something, not how long it takes.
pub fn _a_frequency(sentence: &str, end: usize, start: Option<usize>) -> bool {
    let after = &sentence[end..];
    if let Some(start) = start {
        if FREQUENCY_BEFORE.is_match(&sentence[..start]) {
            // "every two or three years" is the interval the thing is done at.
            return true;
        }
    }
    FREQUENCY_AFTER.match_(after).is_some()
        || FREQUENCY_AFTER_TIMES.match_(after).is_some()
        || FREQUENCY_AFTER_AT_A_TIME.match_(after).is_some()
}

/// Python: bot/memory/narrow.py:2652 `_same_step(asked, words)`
///
/// True when the question and the clause name the same step.
pub fn _same_step(asked: &HashSet<String>, words: &HashSet<String>) -> bool {
    let mine: HashSet<String> = asked.iter().map(|w| _canonical(w)).collect();
    let theirs: HashSet<String> = words.iter().map(|w| _canonical(w)).collect();
    if mine.intersection(&theirs).next().is_some() {
        return true;
    }
    for mine in asked {
        for theirs in words {
            let (short, long_) = if mine.chars().count() <= theirs.chars().count() { (mine, theirs) } else { (theirs, mine) };
            let (ns, nl) = (short.chars().count(), long_.chars().count());
            if ns >= 4 && nl - ns >= 2 && long_.contains(short.as_str()) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/memory/narrow.py:2672 `_step_time(sentence, question, figure)`
///
/// The time in the clause about the step the question names, or None.
pub fn _step_time(sentence: &str, question: &str, figure: Option<&Re>) -> Option<String> {
    let asked: HashSet<String> = words_of(question)
        .iter()
        .filter(|word| word.chars().count() > 3)
        .map(|word| text::rstrip_chars(word, "s").to_string())
        .collect();
    let mut pieces: Vec<String> = Vec::new();
    for one in STEP.split(sentence) {
        let piece = text::strip(&one);
        if piece.is_empty() {
            continue;
        }
        // A figure in two units is one figure, so the bare unit goes back on the piece before.
        if !pieces.is_empty() && BARE_UNIT.match_(piece).is_some() {
            let last = pieces.last_mut().unwrap();
            *last = format!("{} and {}", last, piece);
        } else {
            pieces.push(piece.to_string());
        }
    }
    if !(SEQUENCE.is_match(sentence) || AFTER.is_match(sentence)) {
        // Two different places in the pieces is the exception: each half is a step of its own.
        let places: HashSet<String> =
            pieces.iter().flat_map(|piece| words_of(piece)).filter(|word| WHERE.contains(word)).collect();
        if places.len() < 2 {
            return None;
        }
    }
    if pieces.len() < 2 {
        return None;
    }
    let mut named: Vec<String> = Vec::new();
    let mut cooking: Vec<String> = Vec::new();
    let mut waiting: Vec<String> = Vec::new();
    for piece in &pieces {
        let found: Vec<String> = match figure {
            Some(f) => f.finditer(piece).iter().map(|m| _figure(m, piece)).collect(),
            None => Vec::new(),
        };
        let Some(last) = found.last() else { continue };
        if KEEPS.is_match(piece) {
            continue; // a shelf life in the clause is not a time the step takes
        }
        let words: HashSet<String> = words_of(piece)
            .iter()
            .filter(|word| word.chars().count() > 3)
            .map(|word| text::rstrip_chars(word, "s").to_string())
            .collect();
        let waiting_here = AFTER.is_match(piece);
        if waiting_here { waiting.push(last.clone()) } else { cooking.push(last.clone()) }
        // A clause about waiting is not the step the question names.
        if _same_step(&asked, &words) && !(waiting_here && !WAIT.is_match(question)) {
            named.push(last.clone());
        }
    }
    if !(SEQUENCE.is_match(sentence) || AFTER.is_match(sentence) || !named.is_empty()) {
        return None;
    }
    if !asked.is_empty() {
        if named.len() == 1 {
            return Some(named[0].clone());
        }
        if !named.is_empty() {
            return None; // the question fits two steps
        }
    }
    if WAIT.is_match(question) && !waiting.is_empty() {
        return waiting.last().cloned();
    }
    // Two steps that both take a different time and a question that names neither: a guess.
    if cooking.iter().collect::<HashSet<_>>().len() > 1 {
        return None;
    }
    if let Some(last) = cooking.last() {
        return Some(last.clone());
    }
    waiting.last().cloned()
}

/// Python: bot/memory/narrow.py:2764 `_aside_held(text)`
///
/// The text with each aside replaced by a placeholder, and the asides, to put back.
pub fn _aside_held(text_: &str) -> (String, Vec<String>) {
    let mut kept: Vec<String> = Vec::new();
    let held = ASIDE_ON.sub_fn(text_, |found| {
        kept.push(text::strip(found.as_str()).to_string());
        format!(" \x01{}\x01 ", kept.len() - 1)
    });
    (held, kept)
}

/// Python: bot/memory/narrow.py:2775 `_aside_back(piece, kept)`
pub fn _aside_back(piece: &str, kept: &[String]) -> String {
    let back = HELD.sub_fn(piece, |found| {
        let index: usize = found.str(1).parse().expect("an aside number");
        kept[index].clone()
    });
    text::squash(&back)
}

/// Python: bot/memory/narrow.py:2779 `clauses(text, pattern)`
///
/// The pieces of `text` cut at the separators in `pattern` that are not inside an aside.
pub fn clauses(text_: &str, pattern: &Re) -> Vec<String> {
    clauses_bare(text_, pattern).0
}

/// Python: bot/memory/narrow.py:2789 `clauses_bare(text, pattern)`
///
/// The pieces of `clauses()`, and the same pieces with their asides left out.
pub fn clauses_bare(text_: &str, pattern: &Re) -> (Vec<String>, Vec<String>) {
    let (held, kept) = _aside_held(text_);
    let mut pieces = Vec::new();
    let mut bare = Vec::new();
    for piece in pieces_of(&held, pattern) {
        let dressed_back = _aside_back(&piece, &kept);
        let dressed = text::strip_chars(&dressed_back, " ,;:");
        if dressed.is_empty() {
            continue;
        }
        pieces.push(dressed.to_string());
        let without = text::squash(&HELD.sub(" ", &piece));
        bare.push(text::strip_chars(&without, " ,;:").to_string());
    }
    (pieces, bare)
}

/// Python: bot/memory/narrow.py:2829 `_clause(text, question)`
///
/// The clause that shares the most words with the question, when that is a real narrowing.
pub fn _clause(text_: &str, question: &str) -> Option<String> {
    let wanted = word_set(question);
    if wanted.is_empty() {
        return None;
    }
    let (pieces, bare) = clauses_bare(text_, &CLAUSE_BREAK);
    if pieces.len() < 2 {
        return None;
    }
    let scores: Vec<usize> = bare.iter().map(|piece| word_set(piece).intersection(&wanted).count()).collect();
    let best = *scores.iter().max().unwrap();
    if best > 0 && scores.iter().filter(|&&s| s == best).count() == 1 {
        let index = scores.iter().position(|&s| s == best).unwrap();
        let piece = &pieces[index];
        return if LEANING.match_(piece).is_some() { None } else { Some(piece.clone()) };
    }
    None
}

/// Python: bot/memory/narrow.py:2845 `_clause_with(text, word)`
///
/// The clause of `text` that says `word`, when it says it in exactly one.
pub fn _clause_with(text_: &str, word: &str) -> Option<String> {
    let pieces = clauses(text_, &CLAUSE_BREAK_COLON);
    let says = re_cached(&format!(r"\b{}\b", pyre::escape(word)), pyre::I);
    let hits: Vec<String> = pieces.into_iter().filter(|piece| says.is_match(piece)).collect();
    if hits.len() == 1 { hits.into_iter().next() } else { None }
}

/// Python: bot/memory/narrow.py:2852 `_clause_holding(text, span)`
///
/// The one clause of the sentence that carries this figure, when one clause carries it alone.
pub fn _clause_holding(text_: &str, span_: &str) -> Option<String> {
    let pieces: Vec<String> = pieces_of(text_, &CLAUSE_BREAK_COLON)
        .iter()
        .filter(|piece| !piece.is_empty() && !text::strip(piece).is_empty())
        .map(|piece| text::strip(piece).to_string())
        .collect();
    let hits: Vec<String> = pieces.into_iter().filter(|piece| piece.contains(span_)).collect();
    if hits.len() == 1 { hits.into_iter().next() } else { None }
}

/// Python: bot/memory/narrow.py:2866 `words_of(text)`
///
/// The content words of `text`, without fillers or plurals.
pub fn words_of(text_: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for word in WORD.findall_str(&flat(text_)) {
        let word = text::strip_chars(&word, "'");
        if word.is_empty() || EMPTY.contains(word) {
            continue;
        }
        let mut word = word.to_string();
        if word.chars().count() > 3 && word.ends_with('s') && !word.ends_with("ss") {
            word.pop();
        }
        if !found.contains(&word) {
            found.push(word);
        }
    }
    found
}

/// Python: bot/memory/narrow.py:2880 `flat(text)`
pub fn flat(text_: &str) -> String {
    text::squash(&text::lower(text_))
}

/// Python: bot/memory/narrow.py:2884 `tidy(text)`
///
/// One clean line: no trailing comma, no lowercase start, no double spaces. None for a line
/// with nothing in it.
pub fn tidy(text_: &str) -> Option<String> {
    let squashed = text::squash(text_);
    let mut said = text::rstrip_chars(text::strip(&squashed), ",;:").to_string();
    if said.is_empty() {
        return None;
    }
    let first = said.chars().next().unwrap();
    if pyuni::ch_lower(first) {
        said = format!("{}{}", first.to_uppercase(), &said[first.len_utf8()..]);
    }
    if !said.ends_with(['.', '!', '?']) {
        said.push('.');
    }
    Some(said)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_readers() {
        assert!(only_a_length("its been like that for a week"));
        assert!(!only_a_length("its been like that for a week and now the freezer is warm"));
        assert_eq!(kind_of("how much does it cost"), Some("money".into()));
        assert_eq!(kind_of("when is it done"), None);
        assert_eq!(step_word("should i have a go"), Some("have".into()));
        assert_eq!(tidy("  hello there, "), Some("Hello there.".into()));
        assert_eq!(tidy(" ,;"), None);
        assert_eq!(_root("cleaning"), "clean");
        assert_eq!(_root("filters"), "filter");
        assert_eq!(_unit_stem("Mins."), "minute");
        assert_eq!(words_of("How do I get the potatoes crisp?"), vec!["potatoe", "crisp"]);
    }

    /// Expected values from PyBot (`harness/pycall.py`), for readers the corpus rarely reaches.
    #[test]
    fn the_part_asked_for() {
        let n = |reply: &str, question: &str| narrow(reply, question, "");
        assert_eq!(
            n("Roast at 190C (170C fan) for 20 minutes per 500g plus 20 minutes. Rest it for 15 minutes before carving.", "how long should i rest it"),
            Some("15 minutes.".into())
        );
        assert_eq!(
            n("Green tea wants water at about 80C for 2 to 3 minutes; black tea wants it just off the boil for 3 to 5.", "how long for black tea"),
            Some("Black tea wants it just off the boil for 3 to 5.".into())
        );
        assert_eq!(
            n("Rest it loosely under foil: 15 to 20 minutes for a chicken or a turkey, 10 to 20 for a joint of beef, 5 to 10 for a steak or chops, and 20 to 30 for a brisket.", "how long should i rest a steak"),
            None
        );
        assert_eq!(
            n("Most back pain is muscular and settles in two to six weeks; keep moving, and take paracetamol or ibuprofen if you can.", "what can i take for it"),
            Some("Take paracetamol or ibuprofen if you can.".into())
        );
        assert_eq!(
            n("Ice it for 20 minutes at a time with a cloth round it for the first two days. Most sprains settle in a couple of weeks.", "how long will it last"),
            Some("A couple of weeks.".into())
        );
        assert_eq!(
            n("The Romans arrived in 43 AD and the legions left in 410 AD. They held Britain for nearly 400 years.", "when did they leave"),
            Some("The Romans arrived in 43 AD and the legions left in 410 AD.".into())
        );
        assert_eq!(
            n("Descale it every month or two in a hard-water area. Half fill the kettle with equal parts water and white vinegar, boil it and leave it half an hour.", "how often"),
            Some("Descale it every month or two in a hard-water area.".into())
        );
        assert_eq!(
            n("For a baby over six months, a consistent wind-down helps; feedings at night are normal for much longer.", "how long will this last"),
            Some("No set time on that one - feedings at night are normal for much longer.".into())
        );
        assert_eq!(
            n("Paying within 14 days usually gets a 50% discount. You can appeal within 28 days if you think it is wrong.", "when do i have to pay"),
            Some("14 days.".into())
        );
        assert_eq!(n("A dog costs about £1,500 to £2,000 a year to keep, more for a big breed.", "and how much does one cost"), None);
        assert_eq!(
            the_doings_length("Ice it for 20 minutes at a time with a cloth round it for the first two days. Raise it when you sit.", "how long will it last"),
            "ice it for 20 minutes at a time with a cloth round it for the first two days"
        );
        assert_eq!(
            _how_often(&["Feed it twice a day.".into(), "Weigh the portion.".into()], "how often should i feed it", "feed it twice a day. weigh the portion."),
            Some("Feed it twice a day.".into())
        );
        assert_eq!(
            _freq_clause("Water it deeply once a week in summer and only when the compost is dry all the way down in winter, not on a fixed day, and feed it monthly from spring to autumn with a balanced liquid feed diluted to half strength.", false, &HashSet::new()),
            Some("Water it deeply once a week in summer and only when the compost is dry all the way down in winter, not on a fixed day".into())
        );
        assert_eq!(_clause("Soak the beans overnight, rinse them, and boil hard for ten minutes before simmering.", "how long do i boil them"), None);
        assert_eq!(_named_clause("A sponge in a round tin takes 20 to 25 minutes; a loaf tin needs 45 to 55, so check it with a skewer.", "and if i use a loaf tin?"), None);
        assert_eq!(_with_unit("A loaf tin needs 45 to 55", "A sponge takes 20 to 25 minutes; a loaf tin needs 45 to 55."), "A loaf tin needs 45 to 55 minutes");
    }
}
