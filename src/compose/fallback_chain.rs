//! Port of `bot/compose/fallback_chain.py`: the rows of `fallback` kept in their own file (the
//! composer's chain of readings, `reply_for`, and the small readers it is built from).
//!
//! The Python file is imported from the last line of `fallback.py`, so the two read as one
//! module; here the readings of `fallback.py` are called as `fallback::...`.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback;
use crate::data::{self, Value};
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Python glue shared by this workstream's files (fallback_chain, verdict, onscreen).
// ---------------------------------------------------------------------------------------------

/// `try: <f> except Exception: <fallback>`: a panic that stands for a Python exception gives
/// None. A stub that is not ported yet (`UNPORTED ...`) is not a Python exception, so it keeps
/// unwinding and the checkers still count it.
pub fn caught<T>(f: impl FnOnce() -> T) -> Option<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("");
            if msg.starts_with("UNPORTED") {
                std::panic::resume_unwind(payload);
            }
            None
        }
    }
}

/// `line.format(name=value, ...)` with keyword fields only: `{{` and `}}` are literal braces, and
/// a field that is not given raises (KeyError), as in Python.
pub fn py_format(line: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(line.len() + 16);
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(ch) => name.push(ch),
                        None => panic!("ValueError: expected '}}' before end of string"),
                    }
                }
                match fields.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => out.push_str(v),
                    None => panic!("KeyError: {name}"),
                }
            }
            '}' => panic!("ValueError: Single '}}' encountered in format string"),
            c => out.push(c),
        }
    }
    out
}

/// What a Python call gave back, read the way the caller reads it: its truth, the text it is, or
/// the pool it is. Other workstreams' functions still carry the skeleton's guess at a return type
/// (`()` where every traced call gave None); these readers accept the guess and the real type
/// alike, so a call site keeps compiling when the callee's signature is put right.
pub trait PyRet {
    fn truthy(&self) -> bool;
    fn into_text(self) -> Option<String>
    where
        Self: Sized,
    {
        None
    }
    fn into_pool(self) -> Option<Vec<String>>
    where
        Self: Sized,
    {
        None
    }
    /// The text with `None` and `""` kept apart (for the readers where Python tells them apart).
    fn into_opt_string(self) -> Option<String>
    where
        Self: Sized,
    {
        None
    }
    /// A pool with `None` and an empty pool kept apart (`x is not None`).
    fn into_pool_exact(self) -> Option<Vec<String>>
    where
        Self: Sized,
    {
        self.into_pool()
    }
    /// What it was, as a Python value (to hand on to a reader that takes the value itself).
    fn into_value(self) -> Value
    where
        Self: Sized;
}

impl PyRet for () {
    fn truthy(&self) -> bool {
        false
    }
    fn into_value(self) -> Value {
        Value::None
    }
}
impl PyRet for bool {
    fn truthy(&self) -> bool {
        *self
    }
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}
impl PyRet for i64 {
    fn truthy(&self) -> bool {
        *self != 0
    }
    fn into_value(self) -> Value {
        Value::Int(self)
    }
}
impl PyRet for usize {
    fn truthy(&self) -> bool {
        *self != 0
    }
    fn into_value(self) -> Value {
        Value::Int(self as i64)
    }
}
impl PyRet for String {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_text(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
    fn into_opt_string(self) -> Option<String> {
        Some(self)
    }
    fn into_value(self) -> Value {
        Value::from(self)
    }
}
impl PyRet for &str {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_text(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self.to_string()) }
    }
    fn into_opt_string(self) -> Option<String> {
        Some(self.to_string())
    }
    fn into_value(self) -> Value {
        Value::from(self)
    }
}
impl PyRet for Vec<String> {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_pool(self) -> Option<Vec<String>> {
        if self.is_empty() { None } else { Some(self) }
    }
    fn into_pool_exact(self) -> Option<Vec<String>> {
        Some(self)
    }
    fn into_value(self) -> Value {
        Value::from(self)
    }
}
impl PyRet for &[String] {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_pool(self) -> Option<Vec<String>> {
        if self.is_empty() { None } else { Some(self.to_vec()) }
    }
    fn into_pool_exact(self) -> Option<Vec<String>> {
        Some(self.to_vec())
    }
    fn into_value(self) -> Value {
        Value::from(self.to_vec())
    }
}
impl PyRet for &Vec<String> {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_pool(self) -> Option<Vec<String>> {
        if self.is_empty() { None } else { Some(self.clone()) }
    }
    fn into_pool_exact(self) -> Option<Vec<String>> {
        Some(self.clone())
    }
    fn into_value(self) -> Value {
        Value::from(self.clone())
    }
}
impl PyRet for Value {
    fn truthy(&self) -> bool {
        Value::truthy(self)
    }
    fn into_text(self) -> Option<String> {
        match &self {
            Value::Str(s) if !s.is_empty() => Some(s.to_string()),
            _ => None,
        }
    }
    fn into_pool(self) -> Option<Vec<String>> {
        match &self {
            Value::List(_) | Value::Tuple(_) | Value::Set(_) if self.truthy() => Some(self.strs()),
            _ => None,
        }
    }
    fn into_opt_string(self) -> Option<String> {
        match &self {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }
    fn into_pool_exact(self) -> Option<Vec<String>> {
        match &self {
            Value::List(_) | Value::Tuple(_) | Value::Set(_) => Some(self.strs()),
            _ => None,
        }
    }
    fn into_value(self) -> Value {
        self
    }
}
impl PyRet for &Value {
    fn truthy(&self) -> bool {
        Value::truthy(self)
    }
    fn into_text(self) -> Option<String> {
        self.clone().into_text()
    }
    fn into_pool(self) -> Option<Vec<String>> {
        self.clone().into_pool()
    }
    fn into_opt_string(self) -> Option<String> {
        self.clone().into_opt_string()
    }
    fn into_pool_exact(self) -> Option<Vec<String>> {
        self.clone().into_pool_exact()
    }
    fn into_value(self) -> Value {
        self.clone()
    }
}
impl<T: PyRet> PyRet for Option<T> {
    fn truthy(&self) -> bool {
        self.as_ref().is_some_and(|v| v.truthy())
    }
    fn into_text(self) -> Option<String> {
        self.and_then(|v| v.into_text())
    }
    fn into_pool(self) -> Option<Vec<String>> {
        self.and_then(|v| v.into_pool())
    }
    fn into_opt_string(self) -> Option<String> {
        self.and_then(|v| v.into_opt_string())
    }
    fn into_pool_exact(self) -> Option<Vec<String>> {
        self.and_then(|v| v.into_pool_exact())
    }
    fn into_value(self) -> Value {
        match self {
            Some(v) => v.into_value(),
            None => Value::None,
        }
    }
}

/// `bool(x)` for whatever a call gave back.
pub fn truthy<T: PyRet>(v: T) -> bool {
    v.truthy()
}
/// The text a call gave back, or None for None / "".
pub fn text_of<T: PyRet>(v: T) -> Option<String> {
    v.into_text()
}
/// The pool a call gave back, or None for None / an empty pool.
pub fn pool_of<T: PyRet>(v: T) -> Option<Vec<String>> {
    v.into_pool()
}

/// The text a call gave back with None and "" kept apart.
pub fn opt_string_of<T: PyRet>(v: T) -> Option<String> {
    v.into_opt_string()
}

/// A memory with nothing in it: what a Python reader that takes `memory=None` sees.
pub static NO_MEMORY: LazyLock<Memory> = LazyLock::new(Memory::new);

/// The list of strings a value holds, `str()` of each (an empty list for None).
pub fn str_items(v: Option<&Value>) -> Vec<String> {
    v.map(|v| v.items().iter().map(|x| x.str().to_string()).collect()).unwrap_or_default()
}

/// `" ".join(text.lower().split())`.
pub fn low_squash(s: &str) -> String {
    text::squash(&text::lower(s))
}

/// A word list as owned strings.
pub fn owned(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| w.to_string()).collect()
}

/// `w.strip(chars)` for each word of `s.split()`.
fn stripped_words(s: &str, chars: &str) -> Vec<String> {
    text::split_ref(s).into_iter().map(|w| text::strip_chars(w, chars).to_string()).collect()
}

/// A tuple of strings formatted with one keyword field each.
fn formatted(pool: &[String], fields: &[(&str, &str)]) -> Vec<String> {
    pool.iter().map(|line| py_format(line, fields)).collect()
}

/// Statics for this module's data, read from the bundle by Python name.
macro_rules! data_re {
    ($m:expr; $($n:ident),* $(,)?) => {
        $( static $n: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{}{}", $m, stringify!($n)))); )*
    };
}
macro_rules! data_strs {
    ($m:expr; $($n:ident),* $(,)?) => {
        $( static $n: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{}{}", $m, stringify!($n)))); )*
    };
}
macro_rules! data_set {
    ($m:expr; $($n:ident),* $(,)?) => {
        $( static $n: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{}{}", $m, stringify!($n)))); )*
    };
}
pub(crate) use {data_re, data_set, data_strs};

const M: &str = "bot.compose.fallback_chain.";

/// Python's `set(a) & b` is non-empty.
fn meets(words: &[String], set: &HashSet<String>) -> bool {
    words.iter().any(|w| set.contains(w))
}

// ---------------------------------------------------------------------------------------------
// The body states.
// ---------------------------------------------------------------------------------------------

data_set!(M; FEEL_COPULA, BODY_LEAD, BODY_PART_QUALIFIER, BODY_PART_TAIL, BODY_ALSO, BODY_TAIL);
data_re!(M; GOING_BOTH_WAYS, BODY_PART);
data_strs!(M; BODY_SITUATION);
static BODY_STATES: LazyLock<Vec<(String, HashSet<String>)>> = LazyLock::new(|| {
    data::get(&format!("{M}BODY_STATES")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
});

/// Python: bot/compose/fallback_chain.py:890 `_body_state(clean)`: which body state the whole
/// line is ("cold", "hot", "hungry", "thirsty"), or None. (The skeleton had this returning
/// nothing; Python returns the state's name or None.)
pub fn _body_state(clean: &str) -> Option<String> {
    if clean.contains('?') || clean.contains(" but ") || clean.contains(" not ") {
        return None;
    }
    let flat = clean.replace("light-headed", "lightheaded").replace("light headed", "lightheaded");
    let flat = flat.replace("bunged-up", "bungedup").replace("bunged up", "bungedup");
    let flat = flat.replace("blocked-up", "blockedup").replace("blocked up", "blockedup");
    let words: Vec<String> = stripped_words(&flat, ".,!?;:").into_iter().filter(|w| !w.is_empty()).collect();
    if words.is_empty() || fallback::_opens_a_question(&words) {
        return None;
    }
    for index in 0..words.len().saturating_sub(1) {
        let here = words[index].as_str();
        if !["im", "i'm", "am", "iam"].contains(&here)
            && !(here == "i" && (words[index + 1] == "am" || FEEL_COPULA.contains(&words[index + 1])))
        {
            continue;
        }
        if words[..index].iter().any(|w| !BODY_LEAD.contains(w)) {
            continue;
        }
        let start = if here == "i" { index + 2 } else { index + 1 };
        if GOING_BOTH_WAYS.search(clean).is_some() {
            return None;
        }
        let (state, _after) = _state_from(&words[start.min(words.len())..].join(" "));
        if state.is_some() {
            return state;
        }
    }
    if let Some(part_state) = _a_part_in_a_state(clean) {
        if !part_state.is_empty() {
            return Some(part_state);
        }
    }
    None
}

/// Python: bot/compose/fallback_chain.py:938 `_a_part_in_a_state(clean)`
pub fn _a_part_in_a_state(clean: &str) -> Option<String> {
    let part = BODY_PART.search(clean)?;
    let words: Vec<String> =
        stripped_words(part.name_str("rest"), ".,!?;:").into_iter().filter(|w| !w.is_empty()).collect();
    for (index, word) in words.iter().enumerate() {
        let Some(state) = _the_state(word) else {
            if !BODY_PART_QUALIFIER.contains(word) {
                return None;
            }
            continue;
        };
        if state == "hot" {
            return None;
        }
        let rest: Vec<&String> = words[index + 1..].iter().filter(|w| *w != "and").collect();
        if rest.iter().all(|w| BODY_PART_TAIL.contains(*w) || _the_state(w).as_deref() == Some(state.as_str())) {
            return Some(state);
        }
        return None;
    }
    None
}

/// Python: bot/compose/fallback_chain.py:976 `_a_part_is_cold(clean)`
pub fn _a_part_is_cold(clean: &str) -> bool {
    _a_part_in_a_state(clean).as_deref() == Some("cold")
}

/// Python: bot/compose/fallback_chain.py:985 `_state_from(rest)`: (state, words after it).
pub fn _state_from(rest: &str) -> (Option<String>, Vec<String>) {
    if rest.is_empty() {
        return (None, vec![]);
    }
    let words = stripped_words(rest, ".,!?;:");
    for (index, word) in words.iter().enumerate() {
        let Some(state) = _the_state(word) else { continue };
        let tail = &words[index + 1..];
        if !tail.is_empty() && tail[0] == "and" && tail.len() == 2 && BODY_ALSO.contains(&tail[1]) {
            return (Some(state), vec![]);
        }
        if tail.is_empty() || tail.iter().all(|w| BODY_TAIL.contains(w)) {
            return (Some(state), tail.to_vec());
        }
        if BODY_SITUATION.contains(&tail.join(" ")) {
            return (Some(state), tail.to_vec());
        }
    }
    (None, vec![])
}

/// Python: bot/compose/fallback_chain.py:1006 `_the_state(word)`
pub fn _the_state(word: &str) -> Option<String> {
    BODY_STATES.iter().find(|(_, pool)| pool.contains(word)).map(|(name, _)| name.clone())
}

// ---------------------------------------------------------------------------------------------
// A habit said flatly.
// ---------------------------------------------------------------------------------------------

data_re!(M; HABIT_STATED, HABIT_NOT, HABIT_MIND_IT, HABIT_PROBLEM_ACT, NOT_ABOUT_THEM);
data_set!(M; STRETCH_OF_TIME);

/// Python: bot/compose/fallback_chain.py:1052 `_habit_is_a_problem(clean)`
pub fn _habit_is_a_problem(clean: &str) -> bool {
    HABIT_PROBLEM_ACT.search(clean).is_some() || HABIT_MIND_IT.search(clean).is_some()
}

/// Python: bot/compose/fallback_chain.py:1057 `reads_as_a_flat_habit(text, understood=None)`
pub fn reads_as_a_flat_habit(text_: &str, understood: &Understood) -> bool {
    let mut said = vec![text_.to_string()];
    let repaired = understood.plain.as_str();
    if !repaired.is_empty() && repaired != said[0] {
        said.push(repaired.to_string());
    }
    said.iter().any(|line| _flat_habit_stated(line))
}

/// Python: bot/compose/fallback_chain.py:1081 `_flat_habit_stated(raw)`
pub fn _flat_habit_stated(raw: &str) -> bool {
    let clean = low_squash(raw);
    let words = text::split(&clean);
    caught(|| {
        HABIT_STATED.search(&clean).is_some()
            && HABIT_NOT.search(&clean).is_none()
            && _mood(&words, &clean) == "neutral"
            && !fallback::_reads_as_question(raw, &words)
            && _habit_is_a_problem(&clean)
    })
    .unwrap_or(false)
}

/// Python: bot/compose/fallback_chain.py:1102 `_a_claim_about_a_thing(before)`
pub fn _a_claim_about_a_thing(before: &str) -> bool {
    let probe = format!("{before} ");
    match NOT_ABOUT_THEM.search(&probe) {
        Some(found) => !STRETCH_OF_TIME.contains(found.str(1)),
        None => false,
    }
}

// ---------------------------------------------------------------------------------------------
// A job in the world asked of the bot.
// ---------------------------------------------------------------------------------------------

data_re!(M; ORDER_OPENER, ORDER_A_REPORT, ORDER_NOT_A_JOB, ORDER_A_CLOCK, ORDER_A_RECORD, ORDER_A_MESSAGE,
    ORDER_A_BUY, ORDER_A_DOCUMENT, POLITE_ASK, CAN_DO_ITSELF, ACTS_IN_A_DIARY, EMERGENCY_CALL, POLICE_CALL,
    NOT_A_THING_TO_GET, HANDS_IT_OVER_HERE, ACTS_AN_ERRAND, ACTS_ON_A_PHONE, KEEP_ME_POSTED,
    A_MESSAGE_OR_A_FORM, REMIND_ME, ACT_OPENER, ACT_REPORT_TAIL, ACT_FOR_ME, ACT_CHOICE, ACT_OWN_STEP,
    ACT_SETTING, ADDRESSED, BOOKING_THING, ACT_THING);
data_strs!(M; ACTS_ON_A_CLOCK_LINES, ACTS_IN_A_DIARY_LINES, ACTS_ON_A_PHONE_LINES, ACTS_AN_ERRAND_LINES,
    ACTS_ON_A_DOCUMENT_LINES);
data_set!(M; HOUSE_VERB);

/// `re.sub(r"[^a-z0-9 ']+", " ", ...)`, written inline in two readers below.
static NOT_A_WORD_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9 ']+", 0));

fn bare_of(clean: &str) -> String {
    text::squash(&NOT_A_WORD_CHAR.sub(" ", &text::lower(clean)))
}

/// Python: bot/compose/fallback_chain.py:1350 `_an_order_it_cannot_do(clean)`: the pool, or None.
pub fn _an_order_it_cannot_do(clean: &str) -> Option<Vec<String>> {
    let bare = bare_of(clean);
    if ORDER_OPENER.match_(&bare).is_none() || ORDER_A_REPORT.search(&bare).is_some() {
        return None;
    }
    if ORDER_NOT_A_JOB.search(&bare).is_some() {
        return None;
    }
    if ORDER_A_CLOCK.search(&bare).is_some() {
        return Some(ACTS_ON_A_CLOCK_LINES.clone());
    }
    if ORDER_A_RECORD.search(&bare).is_some() {
        return Some(ACTS_IN_A_DIARY_LINES.clone());
    }
    if ORDER_A_MESSAGE.search(&bare).is_some() {
        return Some(ACTS_ON_A_PHONE_LINES.clone());
    }
    if ORDER_A_BUY.search(&bare).is_some() {
        return Some(ACTS_AN_ERRAND_LINES.clone());
    }
    if ORDER_A_DOCUMENT.search(&bare).is_some() {
        return Some(ACTS_ON_A_DOCUMENT_LINES.clone());
    }
    None
}

/// Python: bot/compose/fallback_chain.py:1376 `_act_asked_of_me(clean)`: the pool, or None.
pub fn _act_asked_of_me(clean: &str) -> Option<Vec<String>> {
    let said = low_squash(clean);
    let found = POLITE_ASK.match_(&said)?;
    let rest = text::strip(found.name_str("rest"));
    if rest.is_empty() || CAN_DO_ITSELF.match_(rest).is_some() {
        return None;
    }
    if ACTS_IN_A_DIARY.search(rest).is_some() {
        return Some(ACTS_IN_A_DIARY_LINES.clone());
    }
    if EMERGENCY_CALL.search(rest).is_some() || POLICE_CALL.search(rest).is_some() {
        return None;
    }
    if NOT_A_THING_TO_GET.search(rest).is_some() || HANDS_IT_OVER_HERE.search(rest).is_some() {
        return None;
    }
    if ACTS_AN_ERRAND.search(rest).is_some() {
        return Some(ACTS_AN_ERRAND_LINES.clone());
    }
    if ACTS_ON_A_PHONE.search(rest).is_some() {
        return Some(ACTS_ON_A_PHONE_LINES.clone());
    }
    None
}

/// Python: bot/compose/fallback_chain.py:1403 `_the_store_has_the_recall(clean, memory)`. The
/// store's `respond` is handed the memory itself, as in Python (what it writes there stays), or
/// an empty one when there is none.
pub fn _the_store_has_the_recall(clean: &str, memory: Option<&mut Memory>) -> bool {
    let mut empty = Memory::new();
    let memory = match memory {
        Some(m) => m,
        None => &mut empty,
    };
    caught(|| truthy(crate::modules::memory_store::respond(clean, memory))).unwrap_or(false)
}

/// Python: bot/compose/fallback_chain.py:1423 `_acts_for_me(clean, memory=None)`
pub fn _acts_for_me(clean: &str, memory: Option<&mut Memory>) -> bool {
    let bare = bare_of(clean);
    if KEEP_ME_POSTED.search(&bare).is_some() {
        return false;
    }
    if A_MESSAGE_OR_A_FORM.search(&bare).is_some() {
        if crate::compose::no_eyes::writes_the_message(clean)
            || truthy(crate::compose::no_eyes::cannot_see_your_thing(clean))
            || truthy(crate::compose::no_eyes::look_at_a_body_lines(clean, Some(&bare), &NO_MEMORY))
        {
            return false;
        }
    }
    if REMIND_ME.match_(&bare).is_some() {
        return !_the_store_has_the_recall(clean, memory);
    }
    let Some(opener) = ACT_OPENER.match_(&bare) else {
        return false;
    };
    let tail = &bare[opener.end()..];
    if ACT_REPORT_TAIL.search(tail).is_some() {
        return false;
    }
    if ACT_FOR_ME.search(&bare).is_none() {
        if ACT_CHOICE.search(&bare).is_some() || ACT_OWN_STEP.match_(tail).is_some() || ACT_SETTING.search(&bare).is_some() {
            return false;
        }
    }
    if ACT_FOR_ME.search(&bare).is_some() {
        return true;
    }
    let verb = text::split(&bare).into_iter().next().unwrap_or_default();
    if !HOUSE_VERB.contains(&verb) && ADDRESSED.match_(&bare).is_none() {
        return false;
    }
    if ["call", "ring", "phone", "text", "email", "buy", "pay", "renew"].contains(&verb.as_str()) {
        return false;
    }
    if ["book", "reserve", "order"].contains(&verb.as_str()) {
        return BOOKING_THING.search(&bare).is_some();
    }
    ADDRESSED.match_(&bare).is_some() || ACT_THING.search(&bare).is_some()
}

// ---------------------------------------------------------------------------------------------
// Moods and the cues that carry them.
// ---------------------------------------------------------------------------------------------

data_set!(M; BAD_STRETCH, NEGATORS, DOUBTED, DOUBT, _WAS, ABOUT_WORDS, PERSON_WORDS, YOU_SUBJECTS, CROWD,
    THINGS_THAT_STOP, THINGS_THAT_STOP_AFTER_UP, THE_WORDS_BETWEEN, JUNK_CUES, _BREAK_WORDS, POSITIVE_CUES,
    NEGATIVE_CUES, NEARLY, PAST_STATE_CUES, STATE_CUES, A_WAIT_ENDING, AIM_HEADS, CONTRAST, NOTHING_AT_ALL);
data_re!(M; A_COMPARISON, NOTHING_THAT_LIMITS, PLEASURE_ABOUT_ONE_THING, FEELING_NOTHING, PLEASURE_GONE_OUT,
    NOTHING_AND_A_PLEASURE, NOTHING_NEGATED, ABOUT_THE_BOT, _A_HURT_DONE, _A_BODY_WORD, _A_HARM_HAD,
    _A_HARM_ILL, _A_HARM_ILL_NOT, _A_HARM_IDIOM, _A_HARM_KNOCK, _A_OUT_OF_SOCKET, _A_IN_AGONY, _A_PART_WENT,
    _A_HURT_BY_SOMEONE, _A_HURT_BY_SOMEONE_NOT, _A_PART_OUT, _A_PROGNOSIS, _A_PROGNOSIS_THAT_IS_NOT,
    _A_BREAK_THAT_IS_GOOD, SOMETHING_CAME_APART, SAYS_A_FEELING, A_FEELING_IN_THE_LINE);
static LOSS_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.loss.PEOPLE"));
static LOSS_SELF_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.loss.SELF_NOUNS"));
static A_MOOD_WITH_A_CATCH: LazyLock<Vec<(String, HashSet<String>)>> = LazyLock::new(|| {
    data::get(&format!("{M}A_MOOD_WITH_A_CATCH")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
});

/// Python: bot/compose/fallback_chain.py:1472 `_reads_as_a_bad_stretch(clean)`
pub fn _reads_as_a_bad_stretch(clean: &str) -> bool {
    let words = stripped_words(clean, ".,!?;:");
    for (index, word) in words.iter().enumerate() {
        if !BAD_STRETCH.contains(word) {
            continue;
        }
        let before = words[index.saturating_sub(4)..index].join(" ");
        if _a_claim_about_a_thing(&before) {
            continue;
        }
        let mut after = &words[index + 1..];
        if !after.is_empty() && (after[0] == "few" || after[0] == "couple") && after.len() > 1 {
            after = &after[1..];
        }
        if !after.is_empty() && STRETCH_OF_TIME.contains(&after[0]) {
            return true;
        }
        if after.is_empty() && text::split_ref(&before).iter().any(|w| STRETCH_OF_TIME.contains(*w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback_chain.py:1489 `_cue_negated(words, cue)`
pub fn _cue_negated(words: &[String], cue: &str) -> bool {
    for (index, word) in words.iter().enumerate() {
        if word != cue {
            continue;
        }
        if words[index.saturating_sub(2)..index].iter().any(|other| NEGATORS.contains(other)) {
            return true;
        }
        let before = &words[index.saturating_sub(3)..index];
        if meets(before, &DOUBTED) {
            return true;
        }
        if meets(before, &DOUBT) && crate::compose::turns_round(before) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback_chain.py:1511 `_a_state_in_the_past(words, index)`
pub fn _a_state_in_the_past(words: &[String], index: i64) -> bool {
    let index = index.max(0) as usize;
    meets(&words[index.saturating_sub(3)..index.min(words.len())], &_WAS)
}

/// Python: bot/compose/fallback_chain.py:1516 `_the_win_is_someone_elses(words, index)`
pub fn _the_win_is_someone_elses(words: &[String], index: i64) -> bool {
    let index = index as usize;
    let after = &words[(index + 1).min(words.len())..(index + 3).min(words.len())];
    if !after.is_empty() && ABOUT_WORDS.contains(&after[0]) && meets(&after[1..], &PERSON_WORDS) {
        return false;
    }
    // range(index - 1, max(-1, index - 7), -1)
    let stop = (index as i64 - 7).max(-1);
    let mut j = index as i64 - 1;
    while j > stop {
        let ju = j as usize;
        let word = &words[ju];
        if YOU_SUBJECTS.contains(word) {
            return false;
        }
        if CROWD.contains(word) {
            return true;
        }
        if LOSS_PEOPLE.contains(word)
            && word.ends_with('s')
            && words[ju.saturating_sub(3)..ju + 1].iter().any(|w| w == "all")
        {
            return true;
        }
        j -= 1;
    }
    false
}

/// Python: bot/compose/fallback_chain.py:1620 `reads_as_no_pleasure(clean)`
pub fn reads_as_no_pleasure(clean: &str) -> bool {
    let said = low_squash(clean);
    if said.is_empty() {
        return false;
    }
    if A_COMPARISON.search(&said).is_some() {
        return false;
    }
    if NOTHING_THAT_LIMITS.search(&said).is_some() || PLEASURE_ABOUT_ONE_THING.search(&said).is_some() {
        return false;
    }
    if FEELING_NOTHING.search(&said).is_some() || PLEASURE_GONE_OUT.search(&said).is_some() {
        return true;
    }
    if NOTHING_AND_A_PLEASURE.search(&said).is_some() {
        return true;
    }
    NOTHING_NEGATED.search(&said).is_some()
}

/// Python: bot/compose/fallback_chain.py:1664 `reads_as_a_question_about_the_bot(clean)`
pub fn reads_as_a_question_about_the_bot(clean: &str) -> bool {
    let said = low_squash(clean);
    !said.is_empty() && ABOUT_THE_BOT.match_(&said).is_some()
}

/// Python: bot/compose/fallback_chain.py:1693 `_a_thing_stopping_after(words, i)`
pub fn _a_thing_stopping_after(words: &[String], i: i64) -> bool {
    let i = i as usize;
    let mut after: Vec<&String> = words[(i + 1).min(words.len())..(i + 5).min(words.len())].iter().collect();
    while !after.is_empty() && THE_WORDS_BETWEEN.contains(after[0]) {
        after.remove(0);
    }
    if after.is_empty() {
        return false;
    }
    if THINGS_THAT_STOP.contains(after[0]) {
        return true;
    }
    if THINGS_THAT_STOP_AFTER_UP.contains(after[0]) && after.len() > 1 && ["up", "in", "out"].contains(&after[1].as_str()) {
        return after.len() == 2 || ["on", "it", "them", "me", "him", "her", "us"].contains(&after[2].as_str());
    }
    false
}

/// Python: bot/compose/fallback_chain.py:1906 `_a_harm_said(clean)`
pub fn _a_harm_said(clean: &str) -> bool {
    let said = low_squash(clean);
    if said.is_empty() {
        return false;
    }
    let s = said.as_str();
    if _A_HARM_IDIOM.search(s).is_some() {
        return false;
    }
    if _A_PART_OUT.search(s).is_some() {
        return true;
    }
    if _A_PROGNOSIS.search(s).is_some() && _A_PROGNOSIS_THAT_IS_NOT.search(s).is_none() {
        return true;
    }
    if _A_HURT_DONE.search(s).is_some() && _A_BODY_WORD.search(s).is_some() {
        return true;
    }
    if _A_HURT_BY_SOMEONE.search(s).is_some() && _A_HURT_BY_SOMEONE_NOT.search(s).is_none() {
        return true;
    }
    if _A_HARM_ILL.search(s).is_some() && _A_HARM_ILL_NOT.search(s).is_none() {
        return true;
    }
    _A_HARM_HAD.search(s).is_some()
        || _A_PART_WENT.search(s).is_some()
        || _A_HARM_KNOCK.search(s).is_some()
        || _A_OUT_OF_SOCKET.search(s).is_some()
        || _A_IN_AGONY.search(s).is_some()
}

/// Python: bot/compose/fallback_chain.py:1948 `_mood(words, clean='')`
pub fn _mood(words: &[String], clean: &str) -> String {
    if crate::memory::ack::only_a_dislike(clean, &NO_MEMORY) {
        return "neutral".into();
    }
    if crate::compose::limit::a_limit_on_the_answer(clean) {
        return "neutral".into();
    }
    if reads_as_no_pleasure(clean) {
        return "negative".into();
    }
    let (mut good, mut bad) = (false, false);
    if words.iter().any(|w| w == "ill") && fallback::_reads_as_illness(clean) {
        bad = true;
    }
    if !good && _a_harm_said(clean) && !crate::compose::badnews::reads_like_a_symptom(clean) {
        bad = true;
    }
    for (i, word) in words.iter().enumerate() {
        if fallback::_in_an_aim(clean, word) {
            continue;
        }
        if JUNK_CUES.contains(word) && fallback::_a_mood_word(words, word) {
            bad = true;
            continue;
        }
        if _BREAK_WORDS.contains(word) && _A_BREAK_THAT_IS_GOOD.search(clean).is_some() {
            good = true;
            continue;
        }
        if !POSITIVE_CUES.contains(word) && !NEGATIVE_CUES.contains(word) {
            continue;
        }
        if let Some((_, catch)) = A_MOOD_WITH_A_CATCH.iter().find(|(k, _)| k == word) {
            let before = &words[i.saturating_sub(3)..i];
            let after = words.get(i + 1).map(String::as_str).unwrap_or("");
            if after == "up" {
                good = true;
                continue;
            }
            if !meets(before, catch) && !["me", "him", "her", "them", "us"].contains(&after) {
                continue;
            }
        }
        let window = &words[i.saturating_sub(3)..i];
        if meets(window, &NEARLY) {
            continue;
        }
        let mut back = _cue_negated(words, word);
        if PAST_STATE_CUES.contains(word) && _a_state_in_the_past(words, i as i64) {
            continue;
        }
        if POSITIVE_CUES.contains(word) && _a_thing_stopping_after(words, i as i64) {
            continue;
        }
        if POSITIVE_CUES.contains(word) {
            if _no_such_thing_before(words, i as i64) {
                back = true;
            }
            if back {
                if STATE_CUES.contains(word) {
                    bad = true;
                }
            } else if A_WAIT_ENDING.contains(word) && SOMETHING_CAME_APART.search(clean).is_some() {
                continue;
            } else if _the_win_is_someone_elses(words, i as i64) {
                continue;
            } else {
                good = true;
            }
        } else if !back || meets(window, &AIM_HEADS) {
            bad = true;
        }
    }
    if bad {
        return "negative".into();
    }
    if good {
        return "positive".into();
    }
    "neutral".into()
}

/// Python: bot/compose/fallback_chain.py:2063 `_no_such_thing_before(words, i)`
pub fn _no_such_thing_before(words: &[String], i: i64) -> bool {
    for step in 1..9i64 {
        if i - step < 0 {
            return false;
        }
        let word = &words[(i - step) as usize];
        if CONTRAST.contains(word) {
            return false;
        }
        if NOTHING_AT_ALL.contains(word) || (word == "one" && i - step - 1 >= 0 && words[(i - step - 1) as usize] == "no") {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// Choosing a line.
// ---------------------------------------------------------------------------------------------

/// Python: bot/compose/fallback_chain.py:2082 `_mixed_seed(text)`
pub fn _mixed_seed(text_: &str) -> i128 {
    let mut value: i128 = 7;
    for c in text_.chars() {
        value = (value * 31 + c as i128) % 1000003;
    }
    value
}

/// `re.split(r"(?<=[.!?])\s+", ...)` as written inline in `_only_what_the_line_gives`.
static SENTENCE_BREAK: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));

/// Python: bot/compose/fallback_chain.py:2115 `_only_what_the_line_gives(pool, clean)`
pub fn _only_what_the_line_gives(pool: &[String], clean: &str) -> Vec<String> {
    if A_FEELING_IN_THE_LINE.search(clean).is_some() {
        return pool.to_vec();
    }
    let mut kept = Vec::new();
    for line in pool {
        let mut asserts = false;
        for sentence in SENTENCE_BREAK.split(line) {
            if text::rstrip(&sentence).ends_with('?') {
                continue;
            }
            if SAYS_A_FEELING.search(&sentence).is_some() {
                asserts = true;
                break;
            }
        }
        if !asserts {
            kept.push(line.clone());
        }
    }
    if kept.is_empty() { pool.to_vec() } else { kept }
}

/// Python: bot/compose/fallback_chain.py:2133 `_choose(options, memory, clean, seed_text=None)`:
/// `seed_text` None or a text, and None back when the conversation has used the pool up.
pub fn _choose(options: &[String], memory: &Memory, clean: &str, seed_text: Option<&str>) -> Option<String> {
    _choose_opt(options, memory, clean, seed_text)
}

/// `_choose` exactly: `seed_text` None or a text, and None back when nothing is fresh.
pub fn _choose_opt(options: &[String], memory: &Memory, clean: &str, seed_text: Option<&str>) -> Option<String> {
    let turns = &memory.turns;
    let recent: Vec<&str> =
        turns[turns.len().saturating_sub(5)..].iter().map(|t| t.bot.as_str()).filter(|b| !b.is_empty()).collect();
    let mut fresh: Vec<&String> = options.iter().filter(|o| !recent.contains(&o.as_str())).collect();
    if fresh.is_empty() {
        let last = recent.last().copied();
        fresh = options.iter().filter(|o| Some(o.as_str()) != last).collect();
        if fresh.is_empty() {
            fresh = options.iter().collect();
        }
    }
    if fresh.is_empty() {
        return None;
    }
    let seed = match seed_text {
        None => turns.len() as i128 * 7919 + crate::py::random::ord_sum(clean),
        Some(s) => turns.len() as i128 * 7919 + _mixed_seed(s),
    };
    Some(crate::py::random::Random::new(seed).choice(&fresh).to_string())
}

// ---------------------------------------------------------------------------------------------
// Birthdays, the day's routine, and the thread.
// ---------------------------------------------------------------------------------------------

data_strs!(M; A_BIRTHDAY_STILL_TO_COME, _NO_DAY_OF_THEIR_OWN, TOO_MUCH_PHRASES, TOO_MUCH, _START_ASK,
    UNKNOWN_MONEY, ROUTINE_PHRASES, ROUTINE, PUNT);
data_set!(M; ROUTINE_NOT, BEHIND_ON_MONEY, _DAY_WORDS, ROUTINE_WORDS, IMPERATIVE, FRAME_WORDS);
data_re!(M; _ROUTINE_NOT_SHAPE, _UNDERTAKING, A_LIST_ON_THE_TABLE, ASK_SHAPE, FRAME_PREFIX, NEAR_MISS,
    OWN_THING, A_MOOD, TAKING_SOMETHING_BACK, THE_OPENER, ASKS_AFTER_THE_OPENER, SAYS_A_THING,
    THE_THING_ITSELF);
static URGENT_ALREADY_GIVEN: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.memory.stuck.URGENT_ALREADY_GIVEN"));

/// `isinstance(v, int)` (a bool is an int in Python too).
fn as_py_int(v: Option<&Value>) -> Option<String> {
    match v {
        Some(Value::Int(i)) => Some(i.to_string()),
        Some(Value::Bool(b)) => Some(if *b { "True" } else { "False" }.to_string()),
        _ => None,
    }
}

/// Python's `str(value)` for the plain values memory holds.
pub fn py_str(v: Option<&Value>) -> String {
    match v {
        Some(Value::Str(s)) => s.to_string(),
        Some(other) => py_repr(other),
        None => "None".into(),
    }
}

/// Python's `repr(value)`.
pub fn py_repr(v: &Value) -> String {
    let join = |items: &[Value]| items.iter().map(py_repr).collect::<Vec<_>>().join(", ");
    match v {
        Value::None => "None".into(),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        Value::Str(s) => crate::understood::py_str_repr(s),
        Value::List(items) => format!("[{}]", join(items)),
        Value::Tuple(items) if items.len() == 1 => format!("({},)", py_repr(&items[0])),
        Value::Tuple(items) => format!("({})", join(items)),
        Value::Set(items) if items.is_empty() => "set()".into(),
        Value::Set(items) => format!("{{{}}}", join(items)),
        Value::Dict(pairs) => {
            let parts: Vec<String> = pairs.iter().map(|(k, x)| format!("{}: {}", py_repr(k), py_repr(x))).collect();
            format!("{{{}}}", parts.join(", "))
        }
        other => format!("{:?}", other.to_plain()),
    }
}

/// `str(x or "")` for a value from memory.
pub fn str_or_empty(v: Option<&Value>) -> String {
    match v {
        Some(v) if v.truthy() => py_str(Some(v)),
        _ => String::new(),
    }
}

/// Python: bot/compose/fallback_chain.py:2161 `_a_birthday_still_to_come(clean, memory)`: the one
/// entry of `memory["others"]` whose birthday to come this line gives, or None.
pub fn _a_birthday_still_to_come(clean: &str, memory: &Memory) -> Option<Value> {
    let said = low_squash(clean);
    if said.is_empty() {
        return None;
    }
    let mut met = Vec::new();
    if let Some(others) = memory.get("others") {
        for one in others.items() {
            if !one.is_dict() || one.get("pronoun").is_some_and(|p| p.truthy()) || !one.get("who").is_some_and(|w| w.truthy()) {
                continue;
            }
            let age = as_py_int(one.get("to_come"));
            let when = low_squash(&str_or_empty(one.get("turns")));
            let Some(age) = age else { continue };
            if when.is_empty() {
                continue;
            }
            if !said.contains(&age) || !said.contains(&when) {
                continue;
            }
            met.push(one.clone());
        }
    }
    if met.len() == 1 { met.pop() } else { None }
}

/// Python: bot/compose/fallback_chain.py:2186 `_the_birthday_said_back(birthday)`
pub fn _the_birthday_said_back(birthday: &Value) -> Vec<String> {
    let mut who = text::lower(&str_or_empty(birthday.get("who")));
    if !who.is_empty() && !(who.starts_with("your ") || who.starts_with("my ")) {
        who = format!("your {who}");
    }
    let age = py_str(birthday.get("to_come"));
    let when = text::squash(&str_or_empty(birthday.get("turns")));
    A_BIRTHDAY_STILL_TO_COME
        .iter()
        .map(|line| text::upper_first(&line.replace("{who}", &who).replace("{age}", &age).replace("{when}", &when)))
        .collect()
}

/// Python: bot/compose/fallback_chain.py:2211 `_somebody_elses_day_in_thread(memory)`
pub fn _somebody_elses_day_in_thread(memory: &Memory) -> bool {
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(1)..] {
        let said = text::lower(&turn.user);
        if _NO_DAY_OF_THEIR_OWN.iter().any(|word| said.contains(word.as_str()))
            || crate::compose::loss::reads_as_loss(&said, None, false)
        {
            return false;
        }
        let said_words = stripped_words(&said, ".,!?;:'\"()");
        if meets(&said_words, &ROUTINE_NOT) || _ROUTINE_NOT_SHAPE.search(&said).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback_chain.py:2244 `_the_person_named_a_list(memory)`
pub fn _the_person_named_a_list(memory: &Memory) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(2)..].iter().any(|turn| A_LIST_ON_THE_TABLE.search(&text::lower(&turn.user)).is_some())
}

/// Python: bot/compose/fallback_chain.py:2257 `_routine_pool(clean, words, memory=None)`
pub fn _routine_pool(clean: &str, words: &[String], memory: Option<&Memory>) -> Option<Vec<String>> {
    let words: HashSet<&str> = words.iter().map(String::as_str).collect();
    let swamped = ["overwhelmed", "drowning", "swamped"].iter().any(|w| words.contains(w));
    if let Some(memory) = memory {
        if crate::memory::stuck::the_advice_was_to_get_seen(memory)
            && (TOO_MUCH_PHRASES.iter().any(|phrase| clean.contains(phrase.as_str())) || swamped)
        {
            return Some(URGENT_ALREADY_GIVEN.clone());
        }
    }
    if TOO_MUCH_PHRASES.iter().filter(|p| !_START_ASK.contains(p)).any(|phrase| clean.contains(phrase.as_str())) {
        return Some(TOO_MUCH.clone());
    }
    if fallback::_money_decision(clean) && words.iter().any(|w| BEHIND_ON_MONEY.contains(*w)) {
        return Some(UNKNOWN_MONEY.clone());
    }
    if _UNDERTAKING.findall_str(clean).iter().any(|word| !_DAY_WORDS.contains(word)) {
        return None;
    }
    if words.iter().any(|w| ROUTINE_NOT.contains(*w)) || _ROUTINE_NOT_SHAPE.search(clean).is_some() {
        return None;
    }
    if let Some(memory) = memory {
        if _somebody_elses_day_in_thread(memory) {
            return None;
        }
    }
    if ROUTINE_PHRASES.iter().any(|phrase| clean.contains(phrase.as_str())) {
        return Some(ROUTINE.clone());
    }
    if words.iter().any(|w| ROUTINE_WORDS.contains(*w)) {
        return Some(if swamped { TOO_MUCH.clone() } else { ROUTINE.clone() });
    }
    if _START_ASK.iter().any(|phrase| clean.contains(phrase.as_str())) {
        let named = memory.is_some_and(_the_person_named_a_list);
        return if named { Some(TOO_MUCH.clone()) } else { None };
    }
    None
}

/// Python: bot/compose/fallback_chain.py:2298 `_asks_the_bot(understood)`
pub fn _asks_the_bot(understood: &Understood) -> bool {
    if understood.asks.iter().any(|a| a == "recommend" || a == "steps" || a == "options") {
        return true;
    }
    understood.norm.first().is_some_and(|w| IMPERATIVE.contains(w))
}

static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Python: bot/compose/fallback_chain.py:2306 `_words(text)`
pub fn _words(text_: &str) -> Vec<String> {
    A_WORD.findall_str(&text::lower(text_))
}

/// Python: bot/compose/fallback_chain.py:2311 `_reads(pred, clean, repaired, lower=False,
/// memory=None)`: the predicate (with memory already bound when there is one) on the typed line,
/// then on the repaired one.
pub fn _reads<T: PyRet>(pred: impl Fn(&str) -> T, clean: &str, repaired: &str, lower: bool) -> bool {
    let ask = |t: &str| -> bool {
        if lower { pred(&text::lower(t)).truthy() } else { pred(t).truthy() }
    };
    if ask(clean) {
        return true;
    }
    if !repaired.is_empty() && repaired != clean {
        return ask(repaired);
    }
    false
}

/// Python: bot/compose/fallback_chain.py:2341 `_restated(understood)`
pub fn _restated(understood: &Understood) -> String {
    if understood.intent != "correction" {
        return String::new();
    }
    let focus = text::strip(&understood.focus.join(" ")).to_string();
    let focus = text::strip_chars(&FRAME_PREFIX.sub("", &focus), " ,.!?").to_string();
    let words = text::split(&focus);
    if words.len() < 2 {
        return String::new();
    }
    if words.iter().all(|w| FRAME_WORDS.contains(&text::lower(w))) {
        return String::new();
    }
    if ASK_SHAPE.search(&text::lower(&focus)).is_none() {
        return String::new();
    }
    focus
}

/// Python: bot/compose/fallback_chain.py:2360 `_answers_the_correction(restated, memory)`
pub fn _answers_the_correction(restated: &str, memory: &Memory) -> Option<String> {
    let mut sub = memory.clone();
    // `sub["compose"] = dict(memory.get("compose") or {})`
    let compose = match memory.get("compose") {
        Some(c) if c.truthy() => c.clone(),
        _ => Value::dict(),
    };
    sub.set("compose", compose);
    sub.understood = Some(crate::understanding::understand(restated, Some(memory)));
    sub.set("correction_answered", true);
    let not_a_punt = |reply: &str| {
        let low = text::lower(reply);
        !PUNT.iter().any(|punt| low.contains(punt.as_str()))
    };
    if let Some(reply) = crate::compose::verdict::next_step(restated, &sub) {
        if !reply.is_empty() && not_a_punt(&reply) {
            return Some(reply);
        }
    }
    if let Some(reply) = crate::compose::verdict::reply_for(restated, &sub) {
        if !reply.is_empty() && not_a_punt(&reply) {
            return Some(reply);
        }
    }
    let reply = reply_for(restated, &mut sub, true);
    if !reply.is_empty() && not_a_punt(&reply) {
        return Some(reply);
    }
    None
}

/// Python: bot/compose/fallback_chain.py:2430 `_a_near_miss(clean)`
pub fn _a_near_miss(clean: &str) -> bool {
    NEAR_MISS.search(clean).is_some()
}

/// Python: bot/compose/fallback_chain.py:2435 `_a_things_state(memory, clean)`
pub fn _a_things_state(memory: &Memory, clean: &str) -> bool {
    let said = text::lower(clean);
    let owner = memory.understood.as_ref().and_then(|u| u.state_owner.as_deref());
    owner == Some("thing") && OWN_THING.search(&said).is_some() && A_MOOD.search(&said).is_some()
}

/// Python: bot/compose/fallback_chain.py:2487 `_a_chat_to_correct(memory)`
pub fn _a_chat_to_correct(memory: &Memory) -> bool {
    memory.turns.iter().any(|turn| !text::strip(&turn.bot).is_empty())
}

/// Python: bot/compose/fallback_chain.py:2496 `_takes_something_back(clean, memory=None)`
pub fn _takes_something_back(clean: &str, memory: &Memory) -> bool {
    let said = text::squash(clean);
    if said.is_empty() || said.contains('?') || TAKING_SOMETHING_BACK.match_(&said).is_none() {
        return false;
    }
    if !_a_chat_to_correct(memory) {
        return false;
    }
    let mut body = text::strip_chars(&THE_OPENER.subn("", &said, 1), " ,.;:-").to_string();
    if !body.is_empty() && TAKING_SOMETHING_BACK.match_(&body).is_some() {
        body = THE_OPENER.subn("", &TAKING_SOMETHING_BACK.subn("", &body, 1), 1);
    }
    if text::split(&body).len() < 2 || ASKS_AFTER_THE_OPENER.match_(&body).is_some() {
        return false;
    }
    SAYS_A_THING.search(&body).is_some() || THE_THING_ITSELF.match_(&body).is_some()
}

data_re!(M; LATE_ASKS, LATE_NOT_A_TIME, LATE_NOW, LATE_NOT_IT);

/// Python: bot/compose/fallback_chain.py:2520 `_a_lateness_report(clean, normalised='')`
pub fn _a_lateness_report(clean: &str, normalised: &str) -> bool {
    if clean.contains('?') || text::split(clean).len() > 12 {
        return false;
    }
    if LATE_ASKS.search(clean).is_some() || LATE_NOT_A_TIME.search(clean).is_some() {
        return false;
    }
    if _mood(&_words(clean), clean) != "neutral" {
        return false;
    }
    let mut read = clean.to_string();
    let mut hit = LATE_NOW.search(&read).map(|m| (m.str(1).to_string(), m.as_str().to_string(), m.end()));
    if hit.is_none() && !normalised.is_empty() {
        read = text::squash(&normalised.replace(" | ", " "));
        hit = LATE_NOW.search(&read).map(|m| (m.str(1).to_string(), m.as_str().to_string(), m.end()));
    }
    let Some((between, whole, end)) = hit else {
        return false;
    };
    if LATE_NOT_IT.search(&between).is_some() {
        return false;
    }
    if whole.ends_with("behind") {
        let after = &read[end..];
        if A_WORD.findall_str(after).iter().any(|word| LOSS_SELF_NOUNS.contains(word)) {
            return false;
        }
    }
    true
}

data_re!(M; HURT_SOMEBODY, HURT_DEATH, HURT_EVENT, HURT_ON_THE_SPEAKER, HURT_BUT_FINE, HURT_FIGURE,
    LOW_DAY_NOT_MINE, LOW_DAY, LOW_DAY_MINE, LOVES_A_GROUP, LOVES_PEOPLE, LOVES_SOMEONE, LOVE_START);
data_strs!(M; SOMEBODY_HURT);
static MY_OR_OUR: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:my|our)\b", 0));

/// Python: bot/compose/fallback_chain.py:2596 `_somebody_hurt(clean, memory, understood)`
pub fn _somebody_hurt(clean: &str, memory: &Memory, understood: &Understood) -> Option<String> {
    if !understood.asks.is_empty() {
        return None;
    }
    if fallback::_in_a_care_thread(memory) {
        return None;
    }
    let found = HURT_SOMEBODY.search(clean)?;
    if HURT_DEATH.search(clean).is_some() {
        return None;
    }
    if HURT_EVENT.search(clean).is_none() {
        return None;
    }
    if HURT_ON_THE_SPEAKER.search(clean).is_some() {
        return None;
    }
    if HURT_BUT_FINE.search(clean).is_some() {
        return None;
    }
    if HURT_FIGURE.search(clean).is_some() {
        return None;
    }
    let who = MY_OR_OUR.sub("your", found.as_str());
    _choose_opt(&formatted(&SOMEBODY_HURT, &[("who", &who)]), memory, clean, None)
}

/// Python: bot/compose/fallback_chain.py:2652 `_a_low_day(clean, understood)`
pub fn _a_low_day(clean: &str, understood: &Understood) -> bool {
    if understood.kind == "question" {
        return false;
    }
    if LOW_DAY_NOT_MINE.search(clean).is_some() {
        return false;
    }
    if clean.contains(',') || clean.contains(" because ") || clean.contains(" cos ") {
        return false;
    }
    LOW_DAY.search(clean).is_some() && LOW_DAY_MINE.match_(clean).is_some()
}

static A_POSSESSIVE_PERSON: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(?:mum|mom|dad|nan|gran|sister|brother|kid|child|parent|family)'?s\b|\band (?:my|our|his|her|their)\b",
        0,
    )
});

/// Python: bot/compose/fallback_chain.py:2697 `_loves_people(clean)`
pub fn _loves_people(clean: &str) -> i64 {
    if clean.contains('?') {
        return 0;
    }
    let said = text::strip(clean);
    let padded = format!(" {said} ");
    if padded.contains(" but ") || padded.contains(" though ") {
        return 0;
    }
    if LOVES_A_GROUP.match_(said).is_some() || LOVES_PEOPLE.match_(said).is_some() {
        return 2;
    }
    if LOVES_SOMEONE.match_(said).is_some() {
        return 1;
    }
    if LOVE_START.match_(said).is_some() && A_POSSESSIVE_PERSON.search(said).is_none() {
        return 1;
    }
    0
}

/// Python: bot/compose/fallback_chain.py:2717 `_loves_someone(clean)`
pub fn _loves_someone(clean: &str) -> bool {
    _loves_people(clean) != 0
}

data_strs!(M; AT_THE_CLOCK);
data_re!(M; A_TIME_SAID, CLOCK_THE_DEPARTURE, CLOCK_ASKED, CLOCK_NOT_THIS, CLOCK_SOMEONE_ELSE,
    CLOCK_ABOUT_THE_JOB, A_LENGTH, CLOCK_SHAPE);

/// Python: bot/compose/fallback_chain.py:2801 `_the_clock_pool(at)`
pub fn _the_clock_pool(at: &str) -> Vec<String> {
    let at = text::strip(at);
    if at.is_empty() {
        return AT_THE_CLOCK.clone();
    }
    let up = text::upper_first(at);
    formatted(&AT_THE_CLOCK, &[("at", at), ("At", &up)])
}

/// Python: bot/compose/fallback_chain.py:2809 `_the_time_said(clean)`: the match, and whether it
/// is `CLOCK_THE_DEPARTURE`'s (`said.re is CLOCK_THE_DEPARTURE`).
pub fn _the_time_said(clean: &str) -> Option<(Match<'_>, bool)> {
    if let Some(m) = A_TIME_SAID.search(clean) {
        return Some((m, false));
    }
    CLOCK_THE_DEPARTURE.search(clean).map(|m| (m, true))
}

/// Python: bot/compose/fallback_chain.py:2818 `_the_morning_has_a_clock(clean)`
pub fn _the_morning_has_a_clock(clean: &str) -> bool {
    if clean.contains('?') || CLOCK_ASKED.match_(clean).is_some() {
        return false;
    }
    if CLOCK_NOT_THIS.search(clean).is_some() || CLOCK_SOMEONE_ELSE.search(clean).is_some() {
        return false;
    }
    if CLOCK_ABOUT_THE_JOB.search(clean).is_some() {
        return false;
    }
    let Some((said, departure)) = _the_time_said(clean) else {
        return false;
    };
    if A_LENGTH.match_(&clean[said.end()..]).is_some() {
        return false;
    }
    CLOCK_SHAPE.search(clean).is_some() || departure
}

static NOTHING_SAID_YET: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.NOTHING_SAID_YET"));

/// Python: bot/compose/fallback_chain.py:2834 `reply_for(text, memory, verdict=True)`. Python's
/// None is "" here.
pub fn reply_for(text_: &str, memory: &mut Memory, verdict: bool) -> String {
    if verdict {
        let matchable = fallback::_matchable(text_);
        let mut alone = crate::memory::ack::a_person_named_alone(&matchable, memory);
        if alone.is_empty() {
            alone = crate::memory::ack::a_person_pointed_at(&matchable, memory);
        }
        if !alone.is_empty() {
            return alone;
        }
    }
    let said = _reply_for(text_, memory, None, verdict).unwrap_or_default();
    if verdict && !said.is_empty() && fallback::is_a_hedge(&said) {
        let clean = fallback::_matchable(text_);
        if let Some(pool) = pool_of(fallback::_a_verdict_on_their_plan(&clean, memory, true)) {
            return _choose_opt(&pool, memory, &clean, None).unwrap_or_default();
        }
    }
    if verdict && !said.is_empty() {
        if fallback::nothing_said_yet(memory)
            && fallback::brings_nothing_of_its_own(text_)
            && fallback::assumes_an_earlier_turn(&said)
        {
            return _choose_opt(&NOTHING_SAID_YET, memory, &fallback::_matchable(text_), None).unwrap_or_default();
        }
    }
    said
}

data_re!(M; OPTION_ON_THE_TABLE, COUNTDOWN_ASK, COUNTDOWN_TO, COUNTED);
data_strs!(M; COUNTDOWN_AFTER_CARE);

/// Python: bot/compose/fallback_chain.py:2898 `_options_already_up(memory)`
pub fn _options_already_up(memory: &Memory) -> bool {
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(2)..] {
        if !OPTION_ON_THE_TABLE.findall(&text::lower(&turn.bot)).is_empty() || text::lower(&turn.user).contains(" or ") {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback_chain.py:2912 `_reads_value(pred, clean, repaired, memory=None)`
/// (memory, when there is one, is bound into `pred`).
pub fn _reads_value<T: PyRet>(pred: impl Fn(&str) -> T, clean: &str, repaired: &str) -> T {
    let found = pred(clean);
    if found.truthy() || repaired.is_empty() || repaired == clean {
        return found;
    }
    pred(repaired)
}

/// Python: bot/compose/fallback_chain.py:2958 `_a_countdown_in_a_care_thread(memory, clean,
/// repaired, understood)`: the date said plainly, or None. (The skeleton had this returning
/// nothing; Python returns the line or None.)
pub fn _a_countdown_in_a_care_thread(memory: &Memory, clean: &str, repaired: &str, _understood: &Understood) -> Option<String> {
    if !crate::memory::topics::in_a_safety_thread(memory) {
        return None;
    }
    let said = low_squash(if !repaired.is_empty() { repaired } else { clean });
    if COUNTDOWN_ASK.search(&said).is_none() || COUNTDOWN_TO.search(&said).is_none() {
        return None;
    }
    let reading = Understood { clean: said.clone(), words: text::split(&said), kind: "question".into(), ..Default::default() };
    let count = crate::knowledge::maths::solve(&said, &reading, &mut Memory::new())?;
    if text::strip(&count).is_empty() {
        return None;
    }
    if COUNTED.match_(&text::lower(text::strip(&count))).is_none() {
        return None;
    }
    let count = format!("{}.", text::rstrip_chars(text::strip(&count), "."));
    _choose_opt(&formatted(&COUNTDOWN_AFTER_CARE, &[("count", &count)]), memory, clean, None)
}

// ---------------------------------------------------------------------------------------------
// The rows after the chain: wrongs, regrets, care homes, small days, names, states, sums.
// ---------------------------------------------------------------------------------------------

data_re!(M; A_WRONG_PLAIN, A_WRONG_SAID, A_WRONG_AIMED, A_WRONG_ABOUT_A_THING, FOUND_IT_OUT, WHO_DID_IT, FOUND_OUT,
    HARM_ON_ME, A_REGRET_DOING, NOT_A_REGRET, SNAPPED_A_PHOTO, THE_JOB_NAMED, A_MISTAKE, HARM_NAMED,
    HAVE_GOT_A_THING, A_CROWD, A_MILESTONE, CROWD_ORDER, NOT_A_CROWD_AHEAD, A_CHILDHOOD_MEMORY,
    SOMEONE_ELSE_NAMED, A_VISIT, THEIR_OWN_DOING, PAST_DOING, A_THING_GOT_AHEAD, GOT_AHEAD_GONE,
    GOT_AHEAD_NOT_A_DATE, A_MONTH, DATE_AHEAD, PROBLEM_STATED, SOMETHING_UP, NOTHING_TO_WEAR, WEAR_ASK,
    WEATHER_WET, COLD_FIGURE, WEATHER_COLD, HOT_FIGURE, WEATHER_HOT, WEATHER_SUN, A_LOOKING_BACK, DAY_PAST,
    A_DAY_SUBJECT, NOT_A_DAY_DONE, NOT_A_DAY_SAID, A_SYMPTOM_PHRASE, NEGATIVE_LINE, A_HURT_MYSELF,
    A_DAY_WENT_WRONG, SOMEONE_DONE_WRONG, A_BAD_THING_NAMED, A_HAND_OUT, BAD_DAY_ANYWAY, _A_DETAIL_OPENING,
    _A_SYMPATHY, HARM_ANYWHERE, _A_BODY_UNWELL, A_NAMES_TOLD, A_NAME_A_HOUSEHOLD_TAKES, NAMED_AFTER,
    STATE_NOT_A_DOING, BARE_STATE, STATE_THEN, STATE_IN_THE_PAST, A_DAY_SAID_BADLY, BOT_WAS_SORRY,
    MONEY_IN_SHAPES, A_ASKING_WORD, A_REPORTED_NOT_LIVED, A_NEWS_IS_WEATHER, A_NEWS_IS_A_CHANGE,
    A_BEING_DONE_NOW, A_DECISION_NOT_A_DAY, A_WISH, AN_ATTEMPT_AT_SOMEONE, A_BETRAYAL_FOUND_OUT,
    A_NAME_IS_NOT_A_DAY, RESULT_WENT_WRONG, PLAN_THAT_FELL_THROUGH, A_DOING_GONE_BADLY, A_THING_SKIPPED,
    A_MEAL_SKIPPED);
data_strs!(M; A_CARE_HOME_BRING, A_CARE_HOME_GUILT, A_CARE_HOME_VISITING, A_CARE_HOME_MOVED, A_CARE_HOME,
    A_BAD_TIME_TOLD, A_THING_THEY_REGRET, A_BETRAYAL, A_DAY_ASKED_PLAIN, A_SMALL_DAY, A_REST_OF_A_GOOD_DAY,
    A_REST_OF_A_HARD_DAY, A_GOOD_TIME, A_GOOD_TIME_IN_A_CARE_THREAD, LOW_DAY_REPLIES, A_HARD_DAY_MOOD,
    A_BOOKING_AHEAD_DATED, A_BOOKING_AHEAD, BARE_PICK_WEAR_WET, BARE_PICK_WEAR_COLD, BARE_PICK_WEAR_HOT,
    WEAR_WHEN_NOTHING_ANSWERS, A_NAMED_AFTER_THEM, A_NAMED_AFTER, A_NAME_STORY, A_NAME_TOLD, NEGATIVE,
    A_THING_CAME_OUT_WRONG, A_PLAN_THAT_FELL_THROUGH, A_NEWS_OF_WEATHER, A_NEWS_OF_SOMETHING_ELSE);
data_set!(M; NOT_A_JOB_BUT_A_PERSON, A_BARE_MODIFIER, DISH_PARTICIPLE, DISH_DETERMINER, FOOD_TOPICS, DONE_VERB,
    DONE_NOT_THIS, _THING_DONE_LINES, NOT_A_NAME_WORD, THE_PEOPLE, A_POSSESSIVE_OF_MINE, MONEY_IN_NOUNS,
    NO_THING_SAID, PRICE_NOUNS);

/// A tuple of pools as a value, for `fallback._in_a_pool(text, pools)`.
fn pools_value(pools: &[&Vec<String>]) -> Value {
    Value::Tuple(std::sync::Arc::new(pools.iter().map(|p| Value::Tuple(std::sync::Arc::new(p.iter().map(|s| Value::from(s.as_str())).collect()))).collect()))
}

fn in_a_pool(text_: &str, pools: &[&Vec<String>]) -> bool {
    fallback::_in_a_pool(text_, &pools_value(pools))
}

static AIMED_ELSEWHERE: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:to|with|from|on|at)\s+(?!me\b|us\b)(?:his|her|their|my|the|a|an|some)\b", 0));

/// Python: bot/compose/fallback_chain.py:5028 `_a_betrayal(clean)`
pub fn _a_betrayal(clean: &str) -> bool {
    let t = low_squash(clean);
    if t.is_empty() || text::split(&t).len() > 22 {
        return false;
    }
    let (wstart, wend) = match A_WRONG_PLAIN.search(&t) {
        Some(w) => (w.start(), w.end()),
        None => {
            let Some(w) = A_WRONG_SAID.search(&t) else { return false };
            if A_WRONG_AIMED.search(&t).is_none() {
                if A_WRONG_ABOUT_A_THING.match_(&t[w.end()..]).is_some() {
                    return false;
                }
                if FOUND_IT_OUT.search(&t).is_none() {
                    return false;
                }
            }
            (w.start(), w.end())
        }
    };
    if WHO_DID_IT.search(&t[..wstart]).is_none() && FOUND_OUT.search(&t[..wstart]).is_none() {
        return false;
    }
    if AIMED_ELSEWHERE.search(&t[wend..]).is_some() {
        return false;
    }
    if HARM_ON_ME.search(&t).is_some() && WHO_DID_IT.search(&t).is_none() {
        return false;
    }
    true
}

/// Python: bot/compose/fallback_chain.py:5068 `_a_thing_they_regret(clean)`
pub fn _a_thing_they_regret(clean: &str) -> bool {
    let t = low_squash(clean);
    if t.is_empty() || text::split(&t).len() > 20 {
        return false;
    }
    if A_REGRET_DOING.search(&t).is_none() {
        return false;
    }
    if NOT_A_REGRET.search(&t).is_some() || SNAPPED_A_PHOTO.search(&t).is_some() {
        return false;
    }
    if A_MISTAKE.search(&t).is_some() {
        if let Some(job) = THE_JOB_NAMED.search(&t) {
            if !NOT_A_JOB_BUT_A_PERSON.contains(job.str(1)) {
                return false;
            }
        }
    }
    if HARM_NAMED.search(&t).is_some() || HARM_ON_ME.search(&t).is_some() {
        return false;
    }
    true
}

/// Python: bot/compose/fallback_chain.py:5131 `_a_care_home_line(understood, memory)`
pub fn _a_care_home_line(understood: Option<&Understood>, memory: &Memory) -> Option<String> {
    let care = understood?.care_home.as_ref()?;
    let pronoun = if care.pronoun.is_empty() { "they".to_string() } else { care.pronoun.clone() };
    let several = pronoun == "they";
    let words: [(&str, &str); 4] = [("they", &pronoun), ("them", "them"), ("their", "their"), ("is", if several { "are" } else { "is" })];
    let pool: &Vec<String> = match care.shape.as_str() {
        "bring" => &A_CARE_HOME_BRING,
        "guilt" => &A_CARE_HOME_GUILT,
        "visiting" => &A_CARE_HOME_VISITING,
        "moved" => &A_CARE_HOME_MOVED,
        _ => &A_CARE_HOME,
    };
    _choose_opt(&formatted(pool, &words), memory, &care.who, None)
}

/// Python: bot/compose/fallback_chain.py:5167 `_a_day_told_instead_of_the_receipt(said_back, clean,
/// memory)`
pub fn _a_day_told_instead_of_the_receipt(said_back: Option<&str>, clean: &str, memory: &Memory) -> Option<String> {
    let said_back = said_back.filter(|s| !s.is_empty())?;
    if !crate::memory::ack::reads_as_a_receipt(said_back) {
        return None;
    }
    if let Some(named) = _a_name_told_reply(clean, memory) {
        return Some(named);
    }
    if _a_small_day_shape(clean) && _a_detail_of_a_bad_day(memory, Some(clean)) {
        return _choose_opt(&A_BAD_TIME_TOLD, memory, clean, None);
    }
    if _a_thing_they_regret(clean) {
        return _choose_opt(&A_THING_THEY_REGRET, memory, clean, None);
    }
    if _a_betrayal(clean) {
        return _choose_opt(&A_BETRAYAL, memory, clean, None);
    }
    if let Some(care) = _a_care_home_line(memory.understood.as_ref(), memory) {
        if !care.is_empty() {
            return Some(care);
        }
    }
    if HAVE_GOT_A_THING.search(clean).is_some() {
        return None;
    }
    if _just_asked_about_the_day(memory) {
        return None;
    }
    if _a_bad_time_told(clean, memory) {
        return _choose_opt(&A_BAD_TIME_TOLD, memory, clean, None);
    }
    if _a_day_asked_without_a_cheer(clean, memory) {
        return _choose_opt(&A_DAY_ASKED_PLAIN, memory, clean, None);
    }
    if !_a_small_day(clean, memory) {
        return None;
    }
    if let Some(wrecked) = pool_of(crate::compose::finding_out::the_wrecked_form_reply(clean)) {
        if let Some(said) = _choose_opt(&wrecked, memory, clean, None) {
            if !said.is_empty() {
                return Some(said);
            }
        }
    }
    if let Some(rest) = _the_rest_of_the_day(clean, memory, "neutral") {
        return _choose_opt(&rest, memory, clean, None);
    }
    _choose_opt(&A_SMALL_DAY, memory, clean, None)
}

/// Python: bot/compose/fallback_chain.py:5295 `_the_crowd_is_ahead(clean)`
pub fn _the_crowd_is_ahead(clean: &str) -> bool {
    let t = low_squash(clean);
    if t.is_empty() || t.contains('?') || text::split(&t).len() > 16 {
        return false;
    }
    if A_CROWD.search(&t).is_none() || A_MILESTONE.search(&t).is_none() {
        return false;
    }
    if CROWD_ORDER.search(&t).is_none() {
        return false;
    }
    NOT_A_CROWD_AHEAD.search(&t).is_none()
}

/// Python: bot/compose/fallback_chain.py:5319 `_the_rest_of_the_day(clean, memory, mood='neutral')`
pub fn _the_rest_of_the_day(clean: &str, memory: &Memory, mood: &str) -> Option<Vec<String>> {
    if mood == "negative" {
        return None;
    }
    if !_a_small_day(clean, memory) {
        return None;
    }
    let last = memory.turns.last()?;
    let was = fallback::_matchable(&last.user);
    if was.is_empty() {
        return None;
    }
    if fallback::_a_good_time(&was, &NO_MEMORY) || in_a_pool(&last.bot, &[&A_GOOD_TIME, &A_GOOD_TIME_IN_A_CARE_THREAD]) {
        return Some(A_REST_OF_A_GOOD_DAY.clone());
    }
    if in_a_pool(&last.bot, &[&LOW_DAY_REPLIES]) {
        return Some(A_REST_OF_A_HARD_DAY.clone());
    }
    if let Some(emotion) = _an_emotion(&was) {
        if A_HARD_DAY_MOOD.contains(&emotion) {
            return Some(A_REST_OF_A_HARD_DAY.clone());
        }
    }
    None
}

/// Python: bot/compose/fallback_chain.py:5348 `_an_emotion(clean)`
pub fn _an_emotion(clean: &str) -> Option<String> {
    caught(|| crate::understanding::kinds::emotion_of(clean)).flatten().filter(|e| !e.is_empty())
}

/// Python: bot/compose/fallback_chain.py:5356 `_just_asked_about_the_day(memory)`
pub fn _just_asked_about_the_day(memory: &Memory) -> bool {
    match memory.turns.last() {
        Some(last) => in_a_pool(&last.bot, &[&A_SMALL_DAY]),
        None => false,
    }
}

/// Python: bot/compose/fallback_chain.py:5452 `_a_day_of_someone_else(text)`
pub fn _a_day_of_someone_else(text_: &str) -> bool {
    if A_CHILDHOOD_MEMORY.search(text_).is_some() {
        return true;
    }
    let Some(who) = SOMEONE_ELSE_NAMED.search(text_) else { return false };
    if A_VISIT.search(text_).is_some() {
        return false;
    }
    match THEIR_OWN_DOING.search(text_) {
        None => true,
        Some(mine) => who.start() < mine.start(),
    }
}

/// Python: bot/compose/fallback_chain.py:5480 `_a_thing_happened(text)`
pub fn _a_thing_happened(text_: &str) -> bool {
    let rest = match PAST_DOING.search(text_) {
        None => text_.to_string(),
        Some(doing) => format!("{} {}", &text_[..doing.start()], &text_[doing.end()..]),
    };
    text::split_ref(&rest).iter().any(|w| text::len(w) > 2 && !A_BARE_MODIFIER.contains(*w))
}

/// Python: bot/compose/fallback_chain.py:5518 `_a_thing_got_ahead(clean)`: the pool, or None.
pub fn _a_thing_got_ahead(clean: &str) -> Option<Vec<String>> {
    let t = low_squash(clean);
    if t.is_empty() || t.contains('?') {
        return None;
    }
    A_THING_GOT_AHEAD.search(&t)?;
    if GOT_AHEAD_GONE.search(&t).is_some() || GOT_AHEAD_NOT_A_DATE.search(&t).is_some() {
        return None;
    }
    if A_MONTH.search(&t).is_some() || DATE_AHEAD.search(&t).is_some() {
        return Some(A_BOOKING_AHEAD_DATED.clone());
    }
    Some(A_BOOKING_AHEAD.clone())
}

static NOT_ALNUM_SPACE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9 ]+", 0));

/// Python: bot/compose/fallback_chain.py:5588 `_a_problem_stated(clean, repaired=None)`
pub fn _a_problem_stated(clean: &str, repaired: Option<&str>) -> bool {
    for line in [Some(clean), repaired] {
        let bare = text::squash(&NOT_ALNUM_SPACE.sub(" ", &text::lower(line.unwrap_or(""))));
        if !bare.is_empty() && (PROBLEM_STATED.match_(&bare).is_some() || SOMETHING_UP.match_(&bare).is_some()) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback_chain.py:5646 `_says_there_is_nothing_to_wear(clean, repaired=None,
/// text=None)`
pub fn _says_there_is_nothing_to_wear(clean: &str, repaired: &str, text_: &str) -> bool {
    [clean, repaired, text_].iter().any(|line| {
        let said = low_squash(line);
        !said.is_empty() && NOTHING_TO_WEAR.search(&said).is_some()
    })
}

static TWO_LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']{2,}", 0));

/// Python: bot/compose/fallback_chain.py:5655 `_a_wear_ask_about_a_day(clean, repaired=None,
/// text=None)`
pub fn _a_wear_ask_about_a_day(clean: &str, repaired: &str, text_: &str) -> bool {
    for line in [clean, repaired, text_] {
        let said = low_squash(line);
        if said.is_empty() || WEAR_ASK.search(&said).is_none() {
            continue;
        }
        let rest = WEAR_ASK.subn(" ", &said, 1);
        if TWO_LETTERS.search(&rest).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback_chain.py:5695 `_the_wear_pool_for_the_line(clean, repaired=None,
/// text=None)`
pub fn _the_wear_pool_for_the_line(clean: &str, repaired: &str, text_: &str) -> Vec<String> {
    let said = [clean, repaired, text_].iter().map(|l| low_squash(l)).collect::<Vec<_>>().join(" ");
    if WEATHER_WET.search(&said).is_some() {
        return BARE_PICK_WEAR_WET.clone();
    }
    if COLD_FIGURE.search(&said).is_some() || WEATHER_COLD.search(&said).is_some() {
        return BARE_PICK_WEAR_COLD.clone();
    }
    if HOT_FIGURE.search(&said).is_some() || WEATHER_HOT.search(&said).is_some() || WEATHER_SUN.search(&said).is_some() {
        return BARE_PICK_WEAR_HOT.clone();
    }
    WEAR_WHEN_NOTHING_ANSWERS.clone()
}

static REMARK_THEIR_NAME_AGE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.remark.THEIR_NAME_AGE"));
static REMARK_THEIR_NAME: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.remark.THEIR_NAME"));
static ONETHING_A_MEDS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.onething.A_MEDS"));
static ONETHING_A_MEDS_CHANGE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.onething.A_MEDS_CHANGE"));
static ONETHING_A_MEDS_MINE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.onething.A_MEDS_MINE"));
static ONETHING_A_MEDS_SYMPTOM: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.onething.A_MEDS_SYMPTOM"));

/// Python: bot/compose/fallback_chain.py:5708 `_a_small_day(clean, memory=None)`
pub fn _a_small_day(clean: &str, memory: &Memory) -> bool {
    if !_a_small_day_shape(clean) {
        return false;
    }
    let t = low_squash(clean);
    let read = memory.understood.as_ref();
    if read.is_some_and(|u| u.suggestion.is_some()) {
        return false;
    }
    if fallback::_a_good_time(clean, memory) {
        return false;
    }
    if _a_detail_about_what_was_said(&t, memory) {
        return false;
    }
    if A_LOOKING_BACK.match_(&t).is_some() {
        return false;
    }
    if read.is_some_and(|u| u.polarity == -1) {
        return false;
    }
    if _a_thing_got_ahead(&t).is_some() {
        return false;
    }
    if REMARK_THEIR_NAME_AGE.match_(&t).is_some() || REMARK_THEIR_NAME.match_(&t).is_some() {
        return false;
    }
    if _a_day_of_someone_else(&t) {
        return false;
    }
    if _a_problem_stated(&t, None) {
        return false;
    }
    if DAY_PAST.search(&t).is_none() && !_a_thing_happened(&t) {
        return false;
    }
    if !(A_DAY_SUBJECT.search(&t).is_some() && PAST_DOING.search(&t).is_some()) {
        return false;
    }
    if DAY_PAST.search(&t).is_none() && text::split(&t).len() > 10 {
        return false;
    }
    if NOT_A_DAY_DONE.search(&t).is_some() || NOT_A_DAY_SAID.search(&t).is_some() {
        return false;
    }
    if _a_thing_they_regret(&t) {
        return false;
    }
    if A_SYMPTOM_PHRASE.search(&t).is_some() {
        return false;
    }
    if NEGATIVE_LINE.search(&t).is_some() || HARM_NAMED.search(&t).is_some() || A_HURT_MYSELF.search(&t).is_some() {
        return false;
    }
    if A_DAY_WENT_WRONG.search(&t).is_some() || crate::compose::remark::done_to_me(&t) {
        return false;
    }
    if fallback::_a_mishap_to_ask_about(&t) {
        return false;
    }
    if SOMEONE_DONE_WRONG.search(&t).is_some() {
        return false;
    }
    if A_BAD_THING_NAMED.search(&t).is_some() || A_HAND_OUT.search(&t).is_some() {
        return false;
    }
    if _the_day_was_already_bad(memory) && DAY_PAST.search(&t).is_none() {
        return false;
    }
    if ONETHING_A_MEDS.search(&t).is_some()
        && ONETHING_A_MEDS_CHANGE.search(&t).is_some()
        && ONETHING_A_MEDS_MINE.match_(&t).is_some()
        && ONETHING_A_MEDS_SYMPTOM.search(&t).is_none()
    {
        return false;
    }
    BAD_DAY_ANYWAY.search(&t).is_none()
}

/// Python: bot/compose/fallback_chain.py:5900 `_a_detail_about_what_was_said(text, memory)`
pub fn _a_detail_about_what_was_said(text_: &str, memory: &Memory) -> bool {
    if _A_DETAIL_OPENING.match_(text_).is_none() {
        return false;
    }
    let Some(last) = memory.turns.last() else { return false };
    let last_bot = last.bot.as_str();
    if in_a_pool(last_bot, &[&A_SMALL_DAY]) {
        return false;
    }
    if _A_SYMPATHY.search(&text::lower(last_bot)).is_some() {
        return true;
    }
    let before = low_squash(&last.user);
    [&*HARM_ANYWHERE, &*NEGATIVE_LINE, &*BAD_DAY_ANYWAY, &*A_DAY_WENT_WRONG, &*_A_BODY_UNWELL]
        .iter()
        .any(|reading| reading.search(&before).is_some())
}

/// Python: bot/compose/fallback_chain.py:6019 `_a_dish_name(clean, words, memory)`
pub fn _a_dish_name(_clean: &str, words: &[String], memory: &Memory) -> bool {
    if words.len() < 2 || !DISH_PARTICIPLE.contains(&words[0]) {
        return false;
    }
    if DISH_DETERMINER.contains(&words[1]) {
        return false;
    }
    let Some(understood) = memory.understood.as_ref() else { return false };
    let mut topics: HashSet<&str> = understood.topics.iter().map(String::as_str).collect();
    if let Some(topic) = understood.topic.as_deref() {
        topics.insert(topic);
    }
    topics.iter().any(|t| FOOD_TOPICS.contains(*t))
}

/// Python: bot/compose/fallback_chain.py:6039 `_a_report_with_no_pronoun(clean, text, memory)`. The
/// composer is asked on the memory itself, as in Python (what it writes there stays).
pub fn _a_report_with_no_pronoun(clean: &str, text_: &str, memory: &mut Memory) -> bool {
    let words = A_WORD.findall_str(clean);
    if words.is_empty() || !DONE_VERB.contains(&words[0]) {
        return false;
    }
    if HARM_ANYWHERE.search(clean).is_some() {
        return false;
    }
    if words[1..words.len().min(3)].iter().any(|w| DONE_NOT_THIS.contains(w)) {
        return false;
    }
    if _a_dish_name(clean, &words, memory) {
        return false;
    }
    let scan = fallback::_thing_done_scan(clean);
    if scan.items().first().map(|v| v.str()) != Some("done") {
        return false;
    }
    match caught(|| reply_for(text_, memory, true)) {
        Some(reply) => _THING_DONE_LINES.contains(&reply),
        None => false,
    }
}

/// Python: bot/compose/fallback_chain.py:6080 `_repaired_of(text, memory)`
pub fn _repaired_of(text_: &str, memory: &Memory) -> String {
    caught(|| {
        if let Some(read) = memory.understood.as_ref() {
            if !read.plain.is_empty() {
                return read.plain.clone();
            }
        }
        crate::understanding::understand(text_, Some(memory)).plain
    })
    .unwrap_or_default()
}

/// Python: bot/compose/fallback_chain.py:6096 `takes_as_it_comes(text, memory)`
pub fn takes_as_it_comes(text_: &str, memory: &mut Memory) -> bool {
    let clean = low_squash(text_);
    caught(|| {
        if fallback::reads_as_a_report(text_, memory) {
            return true;
        }
        let repaired = _repaired_of(text_, memory);
        if truthy(crate::compose::remark::regret_reply_for(&clean, &text::split(&clean), memory, &repaired)) {
            return true;
        }
        if _a_report_with_no_pronoun(&clean, text_, memory) {
            return true;
        }
        if crate::compose::why::takes_as_it_comes(text_, memory) {
            return true;
        }
        if truthy(fallback::_a_verdict_line(&clean, memory)) {
            return true;
        }
        if truthy(crate::compose::safetyask::reply_for(&clean, memory)) {
            return !truthy(fallback::_after_safety(memory, &clean, memory.understood.as_ref()));
        }
        if truthy(fallback::_a_verdict_on_their_plan(&clean, memory, false)) {
            return true;
        }
        if truthy(fallback::_a_place_asked_about(&clean, memory)) {
            return true;
        }
        if fallback::reads_as_if_i_saw(text_, memory) {
            return true;
        }
        if !text_.contains('?') && crate::compose::trouble::answers_a_strain_question(&clean, memory) {
            return true;
        }
        let plain = fallback::_plain(&clean);
        if fallback::_a_declined_plan(&plain, true)
            && (fallback::_a_hard_week(memory) || !fallback::_the_turn_before_was_a_heavy_one(memory, &fallback::_plain(&clean)))
            && !(fallback::_a_plan_on_the_table(memory) && !fallback::_the_line_cancels(&fallback::_plain(&clean)))
        {
            return !truthy(fallback::_after_safety(memory, &clean, memory.understood.as_ref()));
        }
        if !crate::compose::handoff::takes_as_it_comes(text_, memory) {
            return false;
        }
        !truthy(fallback::_after_safety(memory, &clean, memory.understood.as_ref()))
    })
    .unwrap_or(false)
}

static NAME_JOIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*(?:,|and)\s*", 0));

/// Python: bot/compose/fallback_chain.py:6223 `_a_name_told(clean)`: (the names said, the person
/// word, more than one).
pub fn _a_name_told(clean: &str) -> Option<(String, String, bool)> {
    let said = text::strip_chars(&low_squash(clean), " .!,").to_string();
    if said.is_empty() || said.contains('?') {
        return None;
    }
    let (names_text, mut person) = match A_NAMES_TOLD.search(&said) {
        Some(m) => (m.name_str("names").to_string(), m.name_str("person").to_string()),
        None => {
            let m = A_NAME_A_HOUSEHOLD_TAKES.search(&said)?;
            (m.name_str("names").to_string(), String::new())
        }
    };
    let names: Vec<String> = NAME_JOIN.split(&names_text).iter().map(|w| text::strip(w).to_string()).collect();
    let mut kept = Vec::new();
    for word in names {
        if word.is_empty() || text::split(&word).iter().all(|w| NOT_A_NAME_WORD.contains(w)) {
            break;
        }
        kept.push(word);
    }
    if kept.is_empty() {
        return None;
    }
    let spoken = if kept.len() == 1 {
        text::title(&kept[0])
    } else {
        let front: Vec<String> = kept[..kept.len() - 1].iter().map(|w| text::title(w)).collect();
        format!("{} and {}", front.join(", "), text::title(kept.last().unwrap()))
    };
    if !person.is_empty() && !THE_PEOPLE.contains(&person) && !person.contains('-') {
        person = String::new();
    }
    Some((spoken, person, kept.len() > 1))
}

/// Python: bot/compose/fallback_chain.py:6277 `_named_after_reply(clean, memory)`
pub fn _named_after_reply(clean: &str, memory: &Memory) -> Option<String> {
    let said = low_squash(clean);
    let found = NAMED_AFTER.search(&said)?;
    let mut who = found.name_str("who").to_string();
    if ["me", "us", "you", "i"].contains(&who.as_str()) {
        return _choose_opt(&A_NAMED_AFTER_THEM, memory, clean, None).filter(|s| !s.is_empty());
    }
    let words = text::split(&who);
    let last_word = words.last().cloned().unwrap_or_default();
    if A_POSSESSIVE_OF_MINE.contains(&words[0]) {
        who = format!("your {}", words[1]);
    }
    let lines: Vec<String> = A_NAMED_AFTER
        .iter()
        .map(|line| text_of(crate::compose::remark::fill_person(&line.replace("{who}", &who), &last_word)).unwrap_or_default())
        .collect();
    _choose_opt(&lines, memory, clean, None).filter(|s| !s.is_empty())
}

/// Python: bot/compose/fallback_chain.py:6296 `_a_name_told_reply(clean, memory)`
pub fn _a_name_told_reply(clean: &str, memory: &Memory) -> Option<String> {
    let (spoken, person, plural) = _a_name_told(clean)?;
    if person.is_empty() {
        let lines: Vec<String> = A_NAME_STORY.iter().map(|line| line.replace("{names}", &spoken)).collect();
        return _choose_opt(&lines, memory, clean, None).filter(|s| !s.is_empty());
    }
    let word = if plural { "kids".to_string() } else { person };
    let lines: Vec<String> = A_NAME_TOLD
        .iter()
        .map(|line| text_of(crate::compose::remark::fill_person(&line.replace("{names}", &spoken), &word)).unwrap_or_default())
        .collect();
    _choose_opt(&lines, memory, clean, None).filter(|s| !s.is_empty())
}

/// Python: bot/compose/fallback_chain.py:6343 `_a_state_of_theirs(text)`: "now", "then" or None.
pub fn _a_state_of_theirs(text_: &str) -> Option<String> {
    let t = low_squash(text_);
    if t.is_empty() || A_DAY_SUBJECT.search(&t).is_none() {
        return None;
    }
    STATE_NOT_A_DOING.search(&t)?;
    let rest = BARE_STATE.sub(" ", &STATE_NOT_A_DOING.sub(" ", &t));
    if DAY_PAST.search(&t).is_some() || PAST_DOING.search(&rest).is_some() {
        return None;
    }
    if STATE_THEN.search(&t).is_some() || STATE_IN_THE_PAST.search(&t).is_some() {
        return Some("then".into());
    }
    Some("now".into())
}

/// Python: bot/compose/fallback_chain.py:6422 `_a_day_said_badly(text)`
pub fn _a_day_said_badly(text_: &str) -> bool {
    let said = text::strip_chars(&low_squash(text_), " .!,").to_string();
    if said.is_empty() || said.contains('?') || text::split(&said).len() > 12 {
        return false;
    }
    A_DAY_SAID_BADLY.search(&said).is_some()
}

/// Python: bot/compose/fallback_chain.py:6430 `_the_last_exchange(memory)`: the last turn with a
/// reply on it.
pub fn _the_last_exchange(memory: &Memory) -> Option<&crate::state::Turn> {
    memory.turns.iter().rev().find(|t| !text::strip(&t.bot).is_empty())
}

/// Python: bot/compose/fallback_chain.py:6446 `_the_day_was_already_bad(memory)`
pub fn _the_day_was_already_bad(memory: &Memory) -> bool {
    let Some(seen) = _the_last_exchange(memory) else { return false };
    let last = seen.bot.as_str();
    if in_a_pool(last, &[&LOW_DAY_REPLIES, &NEGATIVE, &A_BAD_TIME_TOLD, &A_THING_CAME_OUT_WRONG, &A_PLAN_THAT_FELL_THROUGH, &A_DAY_ASKED_PLAIN]) {
        return true;
    }
    if BOT_WAS_SORRY.search(&text::slice(last, 0, Some(80))).is_some() {
        return true;
    }
    _a_day_said_badly(&seen.user)
}

/// Python: bot/compose/fallback_chain.py:6478 `_a_detail_of_a_bad_day(memory, text=None)`
pub fn _a_detail_of_a_bad_day(memory: &Memory, text_: Option<&str>) -> bool {
    let Some(seen) = _the_last_exchange(memory) else { return false };
    if let Some(t) = text_.filter(|t| !t.is_empty()) {
        if DAY_PAST.search(&text::lower(t)).is_some() {
            return _a_detail_about_what_was_said(t, memory);
        }
    }
    _a_day_said_badly(&seen.user)
}

/// Python: bot/compose/fallback_chain.py:6567 `_money_came_to_them(text, words)`
pub fn _money_came_to_them(text_: &str, words: &[String]) -> bool {
    if meets(words, &MONEY_IN_NOUNS) {
        return true;
    }
    MONEY_IN_SHAPES.search(text_).is_some()
}

static A_CURRENCY_MARK: LazyLock<Re> = LazyLock::new(|| Re::new("[\u{00a3}$\u{20ac}]", 0));
static READING_A_PRICE_VERB: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.reading.A_PRICE_VERB"));

/// Python: bot/compose/fallback_chain.py:6574 `_a_sum_with_nothing_beside_it(clean, memory=None)`:
/// the figure as they wrote it, or None.
pub fn _a_sum_with_nothing_beside_it(clean: &str, memory: Option<&Memory>) -> Option<String> {
    let _ = memory;
    let t = low_squash(clean);
    if t.is_empty() || t.contains('?') {
        return None;
    }
    let words: Vec<String> = {
        let mut seen = HashSet::new();
        A_WORD.findall_str(&t).into_iter().filter(|w| seen.insert(w.clone())).collect()
    };
    if _money_came_to_them(&t, &words) {
        return None;
    }
    let marked = A_CURRENCY_MARK.search(&t).is_some();
    let named = words.iter().any(|w| !NO_THING_SAID.contains(w));
    if !(marked || meets(&words, &PRICE_NOUNS) || (named && READING_A_PRICE_VERB.search(&t).is_some())) {
        return None;
    }
    let probe = Memory { turns: vec![crate::state::Turn { user: t.clone(), bot: String::new(), module: None }], ..Memory::new() };
    let found = crate::compose::reading::_their_figure(&probe, &Value::Int(1));
    if !found.truthy() {
        return None;
    }
    let parts = found.items();
    let said = parts.get(2).cloned().unwrap_or(Value::None);
    let money = parts.get(3).cloned().unwrap_or(Value::None);
    if !money.truthy() || !said.truthy() {
        return None;
    }
    Some(py_str(Some(&said)))
}

/// Python: bot/compose/fallback_chain.py:6653 `_a_report_from_elsewhere(clean)`: the pool, or None.
pub fn _a_report_from_elsewhere(clean: &str) -> Option<Vec<String>> {
    let t = low_squash(clean);
    if t.is_empty() || t.contains('?') {
        return None;
    }
    if A_ASKING_WORD.match_(&t).is_some() {
        return None;
    }
    A_REPORTED_NOT_LIVED.search(&t)?;
    if A_NEWS_IS_WEATHER.search(&t).is_some() {
        return Some(A_NEWS_OF_WEATHER.clone());
    }
    if A_NEWS_IS_A_CHANGE.search(&t).is_some() {
        return Some(A_NEWS_OF_SOMETHING_ELSE.clone());
    }
    None
}

/// Python: bot/compose/fallback_chain.py:6679 `_a_small_day_shape(clean)`
pub fn _a_small_day_shape(clean: &str) -> bool {
    let t = low_squash(clean);
    if t.is_empty() || t.contains('?') {
        return false;
    }
    if text::split(&t).len() > 14 {
        return false;
    }
    if A_BEING_DONE_NOW.search(&t).is_some() || A_DECISION_NOT_A_DAY.search(&t).is_some() || A_WISH.search(&t).is_some() {
        return false;
    }
    if AN_ATTEMPT_AT_SOMEONE.search(&t).is_some() || A_BETRAYAL_FOUND_OUT.search(&t).is_some() {
        return false;
    }
    if _a_day_of_someone_else(&t) {
        return false;
    }
    if DAY_PAST.search(&t).is_none() && !_a_thing_happened(&t) {
        return false;
    }
    if !(A_DAY_SUBJECT.search(&t).is_some() && PAST_DOING.search(&t).is_some()) {
        return false;
    }
    if DAY_PAST.search(&t).is_none() && text::split(&t).len() > 10 {
        return false;
    }
    if NOT_A_DAY_DONE.search(&t).is_some() || NOT_A_DAY_SAID.search(&t).is_some() {
        return false;
    }
    if _a_thing_they_regret(&t) {
        return false;
    }
    if A_SYMPTOM_PHRASE.search(&t).is_some() {
        return false;
    }
    if NEGATIVE_LINE.search(&t).is_some() || HARM_NAMED.search(&t).is_some() {
        return false;
    }
    if A_DAY_WENT_WRONG.search(&t).is_some() {
        return false;
    }
    if fallback::_a_mishap_to_ask_about(&t) {
        return false;
    }
    if A_BAD_THING_NAMED.search(&t).is_some() || A_HAND_OUT.search(&t).is_some() {
        return false;
    }
    if A_NAME_IS_NOT_A_DAY.search(&t).is_some() {
        return false;
    }
    if _a_report_from_elsewhere(&t).is_some() {
        return false;
    }
    if _a_state_of_theirs(&t).is_some() {
        return false;
    }
    if RESULT_WENT_WRONG.search(&t).is_some() || PLAN_THAT_FELL_THROUGH.search(&t).is_some() {
        return false;
    }
    true
}

/// Python: bot/compose/fallback_chain.py:6787 `_a_bad_time_told(clean, memory=None)`
pub fn _a_bad_time_told(clean: &str, memory: &Memory) -> bool {
    if !_a_small_day_shape(clean) {
        return false;
    }
    let t = low_squash(clean);
    if NEGATIVE_LINE.search(&t).is_some() || HARM_NAMED.search(&t).is_some() || A_DAY_WENT_WRONG.search(&t).is_some() {
        return false;
    }
    if memory.understood.as_ref().is_some_and(|u| u.polarity == -1) {
        return true;
    }
    if _a_detail_of_a_bad_day(memory, Some(&t)) {
        return true;
    }
    if A_DOING_GONE_BADLY.search(&t).is_some() || (A_THING_SKIPPED.search(&t).is_some() && A_MEAL_SKIPPED.search(&t).is_none()) {
        return true;
    }
    A_DAY_SAID_BADLY.search(&t).is_some()
}

/// Python: bot/compose/fallback_chain.py:6827 `_a_day_asked_without_a_cheer(clean, memory=None)`
pub fn _a_day_asked_without_a_cheer(clean: &str, memory: &Memory) -> bool {
    if !_a_small_day_shape(clean) {
        return false;
    }
    let t = low_squash(clean);
    if NEGATIVE_LINE.search(&t).is_some() || HARM_NAMED.search(&t).is_some() || A_DAY_WENT_WRONG.search(&t).is_some() {
        return false;
    }
    if _a_bad_time_told(clean, memory) {
        return false;
    }
    if _just_asked_about_the_day(memory) {
        return false;
    }
    BAD_DAY_ANYWAY.search(&t).is_some()
}

// ---------------------------------------------------------------------------------------------
// The chain: `_reply_for`.
// ---------------------------------------------------------------------------------------------

data_re!(M; ANSWERING_A_QUOTED_ASK, A_BIT_PAST_IT, A_CHILD, A_DISMISSAL, A_GOODNESS_ASKED_OF_ME, A_HELP_WORD,
    A_SKILL_ASKED_OF_ME, A_SORRY_ON_ITS_OWN, A_WAY_OF_BEING, CANNOT_ANYTHING_ASK, CANNOT_READ_YOURS,
    CANNOT_SEE_ANYTHING, DITHER, DITHER_OPEN, DITHER_YES_NO, FLAT_LINE, FORGETS_AGAIN, FORGETS_A_CLASH,
    FORGET_A_DATE, FORGET_A_SECRET, FRIGHTENED, HANDS_IT_BACK, HAND_BACK, HEALTH_MODIFIER, HELP_SERVICE,
    I_RATE_A_SKILL, LEAVING_A_PERSON, LEFT_BEHIND, LOST_QUESTION, NOT_LEAVING_ANYONE, NOT_SELF, PLAN_IDIOM,
    PLAN_MEDICINE_SLOW, PLAN_NOT_YET, QUOTE_ASKS, RATES_A_SKILL, SELF_BLAME_ASKS, SELF_SUBJECT, SO_NOTHING_LIVE,
    TELL_HARD, THANKS_GOOD_AGAIN, THANKS_WENT_BADLY, THE_LEAVE_HAS_HAPPENED, WORDING_ASKS, WORDING_ASKS_WHO,
    WOULD_HAVE_BEEN, _A_CORRECTION, _CAUGHT, _ILL, _LUMP);
data_strs!(M; ABOUT_TALLY, ABOUT_THE_BOT_LINES, ACK, ACK_AFTER_TROUBLE, ACK_WARM, ACTS_FOR_ME, AGREEMENT,
    ASKED_A_RATING, ASKED_IF_GOOD, A_CHILD_LEAVING, A_CHILD_LEFT, A_COMPARISON_OF_STANDING, A_CORRECTION_TAKEN,
    A_COST_SOMEBODY_ASKS, A_CROWD_AHEAD, A_DREAD, A_LAUGH, A_LOVE_FOR_PEOPLE, A_LOVE_FOR_SOMEONE, A_STATE_NOW,
    A_STATE_PAST, A_SUGGESTED, A_SUGGESTED_BY_SOMEONE, A_SUM_STATED, A_THING_LOOKS_SAD, A_WANT_IT_OVER,
    BAD_STRETCH_REPLIES, CAME_BACK, CANNOT_ANYTHING, CANNOT_SEE_ANYTHING_LINES, CANNOT_YOUR_OWN, CARD_LINES,
    CARING, CARING_FOR_A_BODY, CHECK_PHRASES, CLOSED_DOOR, CONGRATS, DATES_KEEP_SLIPPING, DECLINED_PLAN,
    DECLINED_PLAN_GENTLE, DITHER_DECIDE, DITHER_OPEN_LINES, DITHER_TWO_WAY_LINES, DOUBLE_BOOKED,
    ELDER_AID_REPLIES, ELDER_CLINIC_REPLIES, ELDER_DOCTOR_REPLIES, ELDER_HELP_REPLIES, EMBARRASSED, EMPTY, FLAT,
    FOUND_IT, GREETING, GRIEF_AFTER, GRIEVANCE, HABIT_REPLY, HAND, HANDS_BACK, HELP_ALREADY, HOW_ARE_YOU,
    IDIOM_GLAD, IDIOM_SHY, INVITED, INVITED_TO_TALK, KEEPS_FORGETTING, KNOCKED, LANDED_BAD, LATE_BEHIND_LINES,
    LATE_NOW_LINES, LAUGHED_AT, LEFT_POOL, LEFT_POOL_GONE, LOCKED_OUT_LINES, LOCKED_OUT_OF_THE_CAR, LOST_A_THING,
    LOST_A_THING_AGAIN, LUMP, MILESTONE, MISHAP_BACK, NAMED_NEGATIVE, NEAR_MISS_REPLY, NEUTRAL, NEVER_MIND,
    NEWS_TO_COME, NOTHING_TO_WEAR_REPLIES, NOT_FIT_ENOUGH, NOT_TOO_OLD, NO_DIRECTION_REPLY, ONE_SKILL_RATED,
    PAUSE_IN_A_SAFETY_THREAD, PENDING_DOUBT, PENDING_LANDED, PENDING_WAITING, PENDING_WHEN,
    PET_CONDITION_TROUBLE, PET_LOW_REPLY, PET_TROUBLE, PLAN, PLAN_DONE, PLAN_DONE_ACK, PLAN_KNOCKED,
    PLAN_MEDICINE_REPLY, PLAN_MEDICINE_SLOW_REPLY, PLEASURE_GONE_LINES, POSITIVE, PREGNANCY_BLEEDING, PRESENT,
    QUOTE_LINES, REACTIONS_CARE, REACTION_REPEAT, RELATIONSHIP_DECISION, RELIEF, RETURNED_TO_A_THING,
    RETURNED_TO_IT, RETURNED_TO_IT_FIRST, RUMINATION_REPLIES, RUN_OUT_LINES, SECRETS_KEEP_SLIPPING, SELF_BLAME,
    SOMEONE_GONE, SOMEONE_LEAVING, SOMEONE_SERIOUSLY_ILL, SORRY_FOR_ASKING, SORRY_ON_ITS_OWN,
    STUCK_AFTER_TROUBLE, SUDDEN_LOSS, SYMPTOM, SYMPTOM_COLD, SYMPTOM_LINGERS, TELL_HARD_LINES, THANKS,
    THANKS_NO_BETTER, THANKS_THEN_SAID, THE_FIRST_HOUR_AND_A_CHILD, TIGHT_MONEY, TIGHT_MONEY_MORE, TIPS_LINES,
    UNKNOWN, UNKNOWN_HEALTH, UNKNOWN_WORD, WALKED_INTO_POOL, WIN_AND_A_COST, WIN_IN_A_SILENCE, WORDING_LINES);
data_set!(M; ACKS, ANIMALS, COMFORT_SAID, EXPERT_WORDS, HEALTH_WORDS);

/// A `{str: pool}` dict of this module, in order.
fn pool_map(name: &str) -> Vec<(String, Vec<String>)> {
    data::get(&format!("{M}{name}")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
}
static ABOUT: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| pool_map("ABOUT"));
static ABOUT_SELF: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| pool_map("ABOUT_SELF"));
static BODY_POOLS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| pool_map("BODY_POOLS"));
static BOUND_FOR: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| pool_map("BOUND_FOR"));
static CUT_FOR: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| pool_map("CUT_FOR"));
/// `REACTIONS_SAID`: a reaction pool -> the pool for when the person already said it.
static REACTIONS_SAID: LazyLock<Vec<(Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get(&format!("{M}REACTIONS_SAID")).pairs().iter().map(|(k, v)| (k.strs(), v.strs())).collect()
});
static VET_WHO: LazyLock<String> = LazyLock::new(|| data::string(&format!("{M}VET_WHO")));

/// `d[key]` for one of the `{str: pool}` dicts (KeyError as in Python).
fn pool_at<'a>(map: &'a [(String, Vec<String>)], key: &str) -> &'a Vec<String> {
    map.iter().find(|(k, _)| k == key).map(|(_, v)| v).unwrap_or_else(|| panic!("KeyError: {key}"))
}
fn has_key(map: &[(String, Vec<String>)], key: &str) -> bool {
    map.iter().any(|(k, _)| k == key)
}

static MORNING_AFTER: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.morning_after.A_MORNING_AFTER"));
static DRAFT_LEAD_INS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.draft.LEAD_INS"));
static DRAFT_NOT_YET: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.draft.NOT_YET"));
static LOSS_AFTER_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.loss.AFTER_REPLIES"));
static STUCK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.stuck.STUCK"));
static ACK_COMPLAINT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.ack.COMPLAINT"));
static UNSEEN_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.unseen.LINES"));
/// `childwords.THE_WORDS_FOR_A_CHILD`. The export did not load `bot/compose/childwords.py` (this
/// file imports it inside `_reply_for`), so the pool is not in the data bundle.
fn the_words_for_a_child() -> Vec<String> {
    match data::try_get("bot.compose.childwords.THE_WORDS_FOR_A_CHILD") {
        Some(v) => v.strs(),
        None => panic!("UNPORTED data bot.compose.childwords.THE_WORDS_FOR_A_CHILD (not in the bundle)"),
    }
}

/// The (axis, part) pairs `the_limits_said` gives back.
pub trait PyPairs {
    fn into_pairs(self) -> Vec<(String, String)>;
}
impl PyPairs for Vec<(String, String)> {
    fn into_pairs(self) -> Vec<(String, String)> {
        self
    }
}
impl PyPairs for Value {
    fn into_pairs(self) -> Vec<(String, String)> {
        self.items().iter().map(|p| (p.items().first().map(|x| x.str().to_string()).unwrap_or_default(), p.items().get(1).map(|x| x.str().to_string()).unwrap_or_default())).collect()
    }
}
impl PyPairs for Vec<String> {
    fn into_pairs(self) -> Vec<(String, String)> {
        // The skeleton's guess (a list of strings) cannot carry Python's (axis, part) tuples.
        panic!("UNPORTED contract bot.compose.fallback:the_limits_said returns (axis, part) pairs")
    }
}

/// `_choose(pool, memory, clean)` as the chain says it.
macro_rules! choose {
    ($pool:expr, $memory:expr, $clean:expr) => {
        _choose_opt(&$pool, &*$memory, &$clean, None)
    };
    ($pool:expr, $memory:expr, $clean:expr, $seed:expr) => {
        _choose_opt(&$pool, &*$memory, &$clean, Some(&$seed))
    };
}

/// `_choose(...).format(...)` / `.replace(...)` on a line that must be there: Python's
/// `None.format` raises.
fn chosen(line: Option<String>) -> String {
    line.unwrap_or_else(|| panic!("AttributeError: 'NoneType' object has no attribute 'format'"))
}

/// Python: bot/compose/fallback_chain.py:2986 `_reply_for(text, memory, understood=None,
/// verdict=True)`: the composer's reply to this turn, or None.
pub fn _reply_for(text_: &str, memory: &mut Memory, understood: Option<&Understood>, verdict: bool) -> Option<String> {
    let understood: Understood = match understood {
        Some(u) => u.clone(),
        None => match &memory.understood {
            Some(u) => u.clone(),
            None => crate::understanding::understand(text_, Some(&*memory)),
        },
    };
    let was_taste = {
        let state = memory.setdefault("compose", Value::dict());
        let was = state.get("taste").is_some_and(|v| v.truthy());
        state.set_key("taste", false);
        was
    };
    let mut kind = if understood.kind.is_empty() { "statement".to_string() } else { understood.kind.clone() };
    let words: Vec<String> = understood.words.clone();
    let clean = if understood.clean.is_empty() { low_squash(text_) } else { understood.clean.clone() };
    if let Some(counted) = _a_countdown_in_a_care_thread(memory, &clean, text_, &understood) {
        if !counted.is_empty() {
            return Some(counted);
        }
    }
    if let Some(after) = text_of(fallback::_after_safety(memory, &clean, Some(&understood))) {
        return Some(after);
    }
    let for_the_child = if text_.is_empty() { clean.as_str() } else { text_ };
    if crate::compose::childwords::asks_the_words_for_a_child(for_the_child, memory) {
        return choose!(the_words_for_a_child(), memory, clean);
    }
    if let Some(at_a_time) = text_of(crate::compose::talktime::answer_for(&clean, memory)) {
        return Some(at_a_time);
    }
    if crate::compose::morning_after::reads_as_a_morning_after(&clean) {
        return choose!(MORNING_AFTER, memory, clean);
    }
    if let Some(hurt) = _somebody_hurt(&clean, memory, &understood) {
        if !hurt.is_empty() {
            return Some(hurt);
        }
    }
    if !fallback::_a_care_thread_to_leave_alone(memory) {
        if let Some(safety_ask) = pool_of(crate::compose::safetyask::reply_for(&clean, memory)) {
            return choose!(safety_ask, memory, clean);
        }
    }
    if HANDS_IT_BACK.match_(&clean).is_some() && !_options_already_up(memory) {
        return choose!(HANDS_BACK, memory, clean);
    }
    let returned = opt_string_of(fallback::reads_as_a_return(&clean));
    if let Some(returned) = returned {
        if !fallback::_came_back(&clean) && !fallback::_grief_thread(memory) {
            if !returned.is_empty() {
                return Some(py_format(&chosen(choose!(RETURNED_TO_A_THING, memory, clean)), &[("thing", &returned)]));
            }
            if !fallback::_a_chat_behind_it(memory) {
                return choose!(RETURNED_TO_IT_FIRST, memory, clean);
            }
            return choose!(RETURNED_TO_IT, memory, clean);
        }
    }
    if let Some(drafted) = text_of(fallback::_named_draft(memory, text_, &words, &clean)) {
        return Some(drafted);
    }
    if understood.agreement.as_deref().is_some_and(|a| !a.is_empty()) {
        return choose!(AGREEMENT, memory, clean);
    }
    if understood.tight_on_money {
        let mut pool: Vec<String> = TIGHT_MONEY.clone();
        if understood.amount_left {
            pool.retain(|line| !text::lower(line).contains("what's left"));
        }
        return choose!(pool, memory, clean);
    }
    if fallback::a_continuation_of_the_tight_week(memory, &clean) {
        return choose!(TIGHT_MONEY_MORE, memory, clean);
    }
    if _a_low_day(&clean, &understood) {
        return choose!(LOW_DAY_REPLIES, memory, clean, text_);
    }
    if HAND_BACK.match_(&fallback::_self_line(text_)).is_some() && !memory.turns.is_empty() {
        return choose!(HOW_ARE_YOU, memory, clean, text_);
    }
    let mut verdict_pool: Option<Vec<String>> = if verdict { pool_of(fallback::_verdict_pool(&clean, memory)) } else { None };
    if verdict_pool.is_some() && fallback::in_a_safety_thread(memory) {
        verdict_pool = None;
    } else if verdict_pool.is_some() && fallback::a_happening_now(memory) {
        verdict_pool = None;
    } else if verdict_pool.is_some() && fallback::a_loss_in_the_thread(memory) {
        verdict_pool = pool_of(fallback::_the_heavy_pool(memory));
    }
    if verdict_pool.is_none() {
        if let Some(tried_reply) = text_of(fallback::_after_advice(memory, &clean, &words)) {
            return Some(tried_reply);
        }
    }
    if let Some(refused) = pool_of(fallback::_a_refusal_to_answer(&clean, memory, &words)) {
        let advice = fallback::the_advice(memory);
        let seed = if advice.is_empty() { clean.clone() } else { advice };
        return choose!(refused, memory, clean, seed);
    }
    if let Some(doable) = text_of(fallback::_a_doable(&clean, memory)) {
        return Some(doable);
    }
    if let Some(pool) = verdict_pool {
        return choose!(pool, memory, clean);
    }
    if let Some(answered) = text_of(crate::compose::reaction::reply_for(&clean, memory)) {
        return Some(answered);
    }
    let reaction: Option<Vec<String>> = pool_of(fallback::_reaction_pool(&clean, memory));
    let is_repeat = reaction.as_ref().is_some_and(|r| *r == *REACTION_REPEAT);
    if is_repeat {
        if let Some(again) = text_of(crate::compose::why::said_again(&clean, memory)) {
            return Some(again);
        }
    }
    if let Some(mut pool) = reaction.clone() {
        if !fallback::_a_care_thread_to_leave_alone(memory) {
            if fallback::_the_person_already_said_it(memory) {
                if let Some((_, said)) = REACTIONS_SAID.iter().find(|(k, _)| *k == pool) {
                    pool = said.clone();
                }
            }
            return choose!(pool, memory, clean);
        }
        if !is_repeat {
            return choose!(REACTIONS_CARE, memory, clean);
        }
    }
    if let Some(the_day) = pool_of(fallback::_the_day_described(&clean)) {
        if !fallback::in_a_safety_thread(memory)
            && !fallback::_a_care_thread_to_leave_alone(memory)
            && !fallback::_the_turn_before_was_a_heavy_one(memory, &clean)
        {
            return choose!(the_day, memory, clean);
        }
    }
    if !words.is_empty() && words.len() <= 6 && FLAT_LINE.match_(&words.join(" ")).is_some() {
        return choose!(FLAT, memory, clean);
    }
    if let Some(view) = text_of(fallback::_the_view_asked_for(&clean, memory)) {
        return Some(view);
    }
    if let Some(no_words) = pool_of(fallback::_no_words(&clean)) {
        return choose!(no_words, memory, clean);
    }
    if kind == "thanks" && text_.contains('?') {
        kind = "question".into();
    }
    let plain_clean = fallback::_plain(&clean);
    let asks_for_help = fallback::_asks_for_help(&plain_clean, Some(&understood));
    let opens_with_hello =
        kind == "statement" && fallback::_reads_as_greeting(&words, text_) && !fallback::_reads_as_question(text_, &words);
    if opens_with_hello
        && !asks_for_help
        && (crate::understanding::kinds::greeting_only(&words) || understood.intent == "request")
    {
        kind = "greeting".into();
    }
    if kind == "empty" {
        return choose!(EMPTY, memory, clean);
    }
    if kind == "greeting" && !asks_for_help {
        return choose!(GREETING, memory, clean);
    }
    if kind == "thanks" {
        if truthy(fallback::_thanks_news_went_badly(&clean)) {
            return choose!(THANKS_NO_BETTER, memory, clean);
        }
        if let Some(bye) = pool_of(crate::compose::leaving::pool(&clean, &words, true, memory, true)) {
            return choose!(bye, memory, clean);
        }
        return choose!(THANKS, memory, clean);
    }
    let loved = _loves_people(&clean);
    if kind != "question" && loved != 0 {
        return if loved > 1 { choose!(A_LOVE_FOR_PEOPLE, memory, clean) } else { choose!(A_LOVE_FOR_SOMEONE, memory, clean) };
    }
    if let Some(going_on) = text_of(crate::compose::handoff::reply_for(&clean, memory)) {
        return Some(going_on);
    }
    let restated = if memory.get("correction_answered").is_some_and(|v| v.truthy()) { String::new() } else { _restated(&understood) };
    if !restated.is_empty() {
        if let Some(reply) = _answers_the_correction(&restated, memory) {
            if !reply.is_empty() {
                return Some(reply);
            }
        }
    }
    if let Some(corrected) = text_of(crate::memory::ack::value_put_right(&clean, memory)) {
        return Some(corrected);
    }
    if let Some(wondered) = text_of(crate::memory::ack::_a_condition_they_wonder_about(&clean, memory)) {
        return Some(wondered);
    }
    let plain = fallback::_plain(&clean);
    let mut repaired = if understood.plain.is_empty() { clean.clone() } else { understood.plain.clone() };
    if fallback::_is_presence_check(&clean) {
        return choose!(PRESENT, memory, clean);
    }
    if fallback::_a_declined_plan(&plain, false) && !truthy(crate::compose::draft::drafter(memory, &clean, &[] as &[String])) {
        if fallback::_a_hard_week(memory) {
            return choose!(DECLINED_PLAN_GENTLE, memory, clean);
        }
        if fallback::_the_turn_before_was_a_heavy_one(memory, &plain) {
            // left to the readers that answer a feeling
        } else if fallback::_someone_else_dropped_it(&plain) {
            return choose!(PLAN_KNOCKED, memory, clean);
        } else {
            return choose!(DECLINED_PLAN, memory, clean);
        }
    }
    if let Some(help_topic) = pool_of(fallback::_help_topic(&clean, &understood)) {
        return choose!(help_topic, memory, clean);
    }
    if let Some(leaning) = text_of(fallback::_a_leaning_side(&plain, memory)) {
        return Some(leaning);
    }
    if let Some(mine) = text_of(fallback::_my_pick(&plain, memory)) {
        return Some(mine);
    }
    if let Some(pool) = pool_of(fallback::_a_verdict_on_their_plan(&plain, memory, false)) {
        return choose!(pool, memory, clean);
    }
    if let Some(place_ask) = pool_of(fallback::_a_place_asked_about(&plain, memory)) {
        return choose!(place_ask, memory, clean);
    }
    let body = fallback::_thread_is_about_the_body(memory, 4);
    if let Some(saying_ask) = text_of(crate::compose::why::respond_to_a_saying_ask(&clean, memory, &understood, body)) {
        return Some(saying_ask);
    }
    let mut about = text_of(fallback::_about(&plain, Some(&understood)));
    if about.as_deref().is_some_and(|a| a == "self_doing" || a == "clothes")
        && ANSWERING_A_QUOTED_ASK.search(&clean).is_some()
        && truthy(fallback::_the_answers_take_it(&clean))
    {
        about = None;
    }
    if about.as_deref() == Some("memory") && fallback::_a_tally_of_its_own_day(&understood) {
        return choose!(ABOUT_TALLY, memory, clean);
    }
    if about.as_deref() == Some("hand") {
        return choose!(HAND, memory, clean);
    }
    if let Some(a) = about.as_deref() {
        if has_key(&ABOUT, a) {
            return choose!(pool_at(&ABOUT, a), memory, clean);
        }
        return choose!(pool_at(&ABOUT_SELF, a), memory, clean);
    }
    if let Some(awkward) = pool_of(fallback::_their_own_slip(&clean, memory)) {
        let mut said = choose!(awkward, memory, clean);
        if said.is_none() {
            said = if awkward == *LAUGHED_AT { choose!(EMBARRASSED, memory, clean) } else { choose!(LAUGHED_AT, memory, clean) };
        }
        if let Some(said) = said.filter(|s| !s.is_empty()) {
            return Some(said);
        }
    }
    if SELF_SUBJECT.search(&clean).is_some()
        && SELF_BLAME_ASKS.search(&clean).is_none()
        && WORDING_ASKS.search(&clean).is_none()
        && RATES_A_SKILL.search(&clean).is_none()
        && !truthy(fallback::_own_feeling(&clean))
    {
        return choose!(SELF_BLAME, memory, clean);
    }
    let mut asking = kind == "question"
        || (kind == "statement"
            && (fallback::_reads_as_question(text_, &words)
                || (repaired != clean && fallback::_reads_as_a_question_when_repaired(&repaired))));
    if asking && truthy(fallback::_thanks_then_said(&clean)) {
        asking = false;
    }
    if asking && fallback::_acknowledges_alone(&clean) {
        asking = false;
    }
    if let Some(action) = text_of(fallback::_action(&clean, memory)) {
        return Some(action);
    }
    if WOULD_HAVE_BEEN.search(&clean).is_some() {
        return choose!(MILESTONE, memory, clean);
    }
    if let Some(bye) = pool_of(crate::compose::leaving::pool(&clean, &words, true, memory, true)) {
        return choose!(bye, memory, clean);
    }
    if crate::compose::leaving::reads_as_a_leave_with_a_body_in_it(&clean, &words) {
        if let Some(bye) = pool_of(crate::compose::leaving::pool(&clean, &words, false, memory, true)) {
            return choose!(bye, memory, clean);
        }
    }
    let mut asks_for_words =
        fallback::_asks_for_words(&clean, &words) || (repaired != clean && fallback::_asks_for_words(&repaired, &words));
    if WORDING_ASKS_WHO.search(&clean).is_some() && words.iter().any(|w| EXPERT_WORDS.contains(w)) {
        asks_for_words = true;
    }
    if QUOTE_ASKS.search(&clean).is_some() {
        return choose!(QUOTE_LINES, memory, clean);
    }
    if asks_for_words {
        if TELL_HARD.search(&clean).is_some() {
            return choose!(TELL_HARD_LINES, memory, clean);
        }
        if words.iter().any(|w| w == "card") {
            return choose!(CARD_LINES, memory, clean);
        }
        return choose!(WORDING_LINES, memory, clean);
    }
    if fallback::_asks_how_are_you(&plain, &understood.plain) || understood.asks_how_i_am {
        return choose!(HOW_ARE_YOU, memory, clean);
    }
    if let Some(taste) = text_of(fallback::_taste(&plain, memory, was_taste, asking, &repaired)) {
        if let Some(state) = memory.get_mut("compose") {
            state.set_key("taste", true);
        }
        return Some(taste);
    }
    if let Some(number) = pool_of(fallback::_their_contact_number(&plain, asking)) {
        return choose!(number, memory, clean);
    }
    if fallback::_a_no_from_the_bot(&plain) {
        let cannot = fallback::_live(&plain, true, memory).filter(|p| !p.is_empty()).or_else(|| pool_of(fallback::_the_live_thing_named(&plain)));
        if let Some(cannot) = cannot {
            return choose!(cannot, memory, clean);
        }
        if CANNOT_SEE_ANYTHING.search(&plain).is_some() {
            return choose!(CANNOT_SEE_ANYTHING_LINES, memory, clean);
        }
        if CANNOT_READ_YOURS.search(&plain).is_some() {
            let pool = pool_of(fallback::_what_no_eyes_already_says(&plain)).unwrap_or_else(|| CANNOT_YOUR_OWN.clone());
            return choose!(pool, memory, clean);
        }
        if SO_NOTHING_LIVE.search(&plain).is_some()
            || (fallback::_the_bot_says_it_cannot(&plain) && CANNOT_ANYTHING_ASK.search(&plain).is_some())
        {
            return choose!(CANNOT_ANYTHING, memory, clean);
        }
    }
    if let Some(live) = fallback::_live(&plain, asking, memory).filter(|p| !p.is_empty()) {
        return choose!(live, memory, clean);
    }
    if crate::compose::loss::found_what_was_lost(&clean) {
        let last = text_of(crate::memory::last_user(memory, 1)).unwrap_or_default();
        if crate::compose::loss::lost_thing_to_find(&last) {
            return choose!(FOUND_IT, memory, clean);
        }
    }
    if crate::compose::loss::locked_out(&clean) {
        if crate::compose::loss::locked_out_of_the_car(&clean) {
            return choose!(LOCKED_OUT_OF_THE_CAR, memory, clean);
        }
        return choose!(LOCKED_OUT_LINES, memory, clean);
    }
    if crate::compose::loss::keeps_losing_thing(&clean) {
        return choose!(LOST_A_THING_AGAIN, memory, clean);
    }
    if crate::compose::loss::lost_thing_to_find(&clean) {
        return choose!(LOST_A_THING, memory, clean);
    }
    if crate::compose::unseen::reads(&clean) {
        return choose!(UNSEEN_LINES, memory, clean);
    }
    let own = pool_of(fallback::_own_feeling(&clean)).or_else(|| {
        if LOST_QUESTION.search(&clean).is_some() && NOT_SELF.search(&clean).is_none() { Some(NO_DIRECTION_REPLY.clone()) } else { None }
    });
    if let Some(own) = own {
        return choose!(own, memory, clean);
    }
    if let Some(routine) = _routine_pool(&clean, &words, Some(&*memory)) {
        return choose!(routine, memory, clean);
    }
    if I_RATE_A_SKILL.search(&clean).is_some() && WORDING_ASKS.search(&clean).is_none() && A_WAY_OF_BEING.search(&clean).is_none() {
        return choose!(ONE_SKILL_RATED, memory, clean);
    }
    if SELF_BLAME_ASKS.search(&clean).is_none() {
        if A_SKILL_ASKED_OF_ME.search(&clean).is_some() && A_WAY_OF_BEING.search(&clean).is_none() {
            return choose!(ASKED_A_RATING, memory, clean);
        }
        if A_GOODNESS_ASKED_OF_ME.search(&clean).is_some() {
            return choose!(ASKED_IF_GOOD, memory, clean);
        }
    }
    let body = fallback::_thread_is_about_the_body(memory, 4);
    if let Some(why_follow_up) = text_of(crate::compose::why::respond(&clean, memory, &understood, body)) {
        return Some(why_follow_up);
    }
    if let Some(asked) = _act_asked_of_me(&clean) {
        return choose!(asked, memory, clean);
    }
    if _acts_for_me(&clean, Some(&mut *memory)) {
        return choose!(ACTS_FOR_ME, memory, clean);
    }
    if let Some(ordered) = _an_order_it_cannot_do(&clean) {
        return choose!(ordered, memory, clean);
    }
    let repaired_low = text::lower(&repaired);
    if crate::compose::badnews::reads_like_a_symptom(&repaired_low) && _LUMP.search(&repaired).is_some() {
        return choose!(LUMP, memory, clean);
    }
    let settled = format!(" {} ", NOT_AN_APOSTROPHE_WORD.sub(" ", &clean));
    let settled_words = text::split(&settled.replace('\'', ""));
    let not_sore = |memory: &Memory| -> bool {
        let _ = memory;
        !(opt_string_of(crate::compose::trouble::reads_as_trouble(&clean, Some(words.as_slice()), &repaired)).as_deref() == Some("sore")
            && _CAUGHT.search(&clean).is_none())
    };
    if crate::compose::badnews::reads_like_a_symptom(&repaired_low)
        && fallback::_who_for(&clean, &settled, &settled_words, None).filter(|s| !s.is_empty()).is_none()
        && not_sore(memory)
        && crate::compose::says_it_has_lingered(&repaired, _ILL.search(&repaired).is_some())
    {
        return choose!(SYMPTOM_LINGERS, memory, clean);
    }
    if asking
        && crate::compose::badnews::reads_like_a_symptom(&repaired_low)
        && fallback::_who_for(&clean, &settled, &settled_words, None).filter(|s| !s.is_empty()).is_none()
        && not_sore(memory)
    {
        return if _ILL.search(&repaired).is_some() { choose!(SYMPTOM_COLD, memory, clean) } else { choose!(SYMPTOM, memory, clean) };
    }
    if _a_lateness_report(&clean, &understood.norm_text) {
        let pool: &Vec<String> =
            if clean.contains("behind") || understood.norm_text.contains("behind") { &LATE_BEHIND_LINES } else { &LATE_NOW_LINES };
        return choose!(pool, memory, clean);
    }
    let invited = text_of(fallback::_an_invitation(&clean));
    if invited.as_deref() == Some("talk") {
        return choose!(INVITED_TO_TALK, memory, clean);
    }
    if invited.is_some() {
        return choose!(INVITED, memory, clean);
    }
    let mut limits: Vec<(String, String)> = fallback::the_limits_said(&clean).into_pairs();
    if limits.is_empty() {
        limits = vec![(fallback::reads_as_a_narrowing(&clean), String::new())];
    }
    for (axis, part) in &limits {
        if axis.is_empty() || !fallback::the_answer_to_cut(memory, &Value::from(axis.as_str())).truthy() {
            continue;
        }
        let bound = fallback::the_bound_said(if part.is_empty() { &clean } else { part });
        if !bound.is_empty() {
            return Some(py_format(&chosen(choose!(pool_at(&BOUND_FOR, axis), memory, clean)), &[("bound", &bound)]));
        }
        return choose!(pool_at(&CUT_FOR, axis), memory, clean);
    }
    let refused = text_of(fallback::_a_carer_refused(&clean)).or_else(|| text_of(fallback::_a_carer_refused(&repaired)));
    if let Some(refused) = refused {
        if !fallback::_hedged_recently(memory, 3)
            && !(fallback::_in_a_loss_thread(memory) || fallback::_grief_thread(memory) || fallback::in_a_safety_thread(memory))
        {
            let pool: &Vec<String> = match refused.as_str() {
                "aid" => &ELDER_AID_REPLIES,
                "clinic" => &ELDER_CLINIC_REPLIES,
                "doctor" => &ELDER_DOCTOR_REPLIES,
                "help" => &ELDER_HELP_REPLIES,
                other => panic!("KeyError: {other}"),
            };
            let said_before = memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default();
            if !in_a_pool(&said_before, &[&ELDER_AID_REPLIES, &ELDER_CLINIC_REPLIES, &ELDER_DOCTOR_REPLIES, &ELDER_HELP_REPLIES]) {
                let seed = format!("{clean} {}", fallback::_the_line_before(memory));
                return choose!(pool, memory, clean, seed);
            }
        }
    }
    if let Some(handed) = pool_of(crate::compose::no_eyes::handed_you_something(&clean)) {
        return choose!(handed, memory, clean);
    }
    if let Some(body_lines) = pool_of(crate::compose::no_eyes::look_at_a_body_lines(&clean, None, memory)) {
        return choose!(body_lines, memory, clean);
    }
    if let Some(cannot_see) = pool_of(crate::compose::no_eyes::cannot_see_your_thing(&clean)) {
        return choose!(cannot_see, memory, clean);
    }
    if fallback::_am_i_a_nuisance(&clean) && !fallback::_grief_thread(memory) && !fallback::in_a_safety_thread(memory) {
        return choose!(SORRY_FOR_ASKING, memory, clean);
    }
    if asking && !_takes_something_back(&clean, memory) {
        if reads_as_no_pleasure(&clean) || (!repaired.is_empty() && reads_as_no_pleasure(&repaired)) {
            return choose!(PLEASURE_GONE_LINES, memory, clean);
        }
        if let Some(step) = text_of(crate::compose::step_when_low(&clean, memory, &COMFORT_SAID.iter().cloned().collect::<Vec<_>>())) {
            return Some(step);
        }
        if reads_as_a_question_about_the_bot(&clean) || reads_as_a_question_about_the_bot(&repaired) {
            return choose!(ABOUT_THE_BOT_LINES, memory, clean);
        }
        let unknown = fallback::_unknown(&understood, memory, &clean);
        if _a_wear_ask_about_a_day(&clean, &repaired, text_) && fallback::is_a_hedge(&unknown) {
            return choose!(_the_wear_pool_for_the_line(&clean, &repaired, text_), memory, clean);
        }
        return Some(unknown);
    }
    if let Some(drafted) = text_of(crate::compose::draft::drafter(memory, text_, &words)) {
        let lines: Vec<String> = DRAFT_LEAD_INS.iter().map(|lead| format!("{lead} \"{drafted}\"")).collect();
        return choose!(lines, memory, clean);
    }
    if crate::compose::draft::not_the_message(memory, text_) {
        return choose!(DRAFT_NOT_YET, memory, clean);
    }
    use crate::compose::badnews;
    use crate::compose::goodnews;
    if _reads(badnews::reads_as_pregnancy_bleeding, &clean, &repaired, false) {
        return choose!(PREGNANCY_BLEEDING, memory, clean);
    }
    if fallback::a_fresh_death_and_the_words(&clean) {
        return choose!(THE_FIRST_HOUR_AND_A_CHILD, memory, clean);
    }
    if _reads(badnews::reads_as_sudden_loss, &clean, &repaired, false) {
        return choose!(SUDDEN_LOSS, memory, clean);
    }
    if _reads(goodnews::reads_as_recovery, &clean, &repaired, true) && !fallback::_in_a_loss_thread(memory) {
        return choose!(RELIEF, memory, clean);
    }
    if _reads(goodnews::reads_as_a_clear_result, &clean, &repaired, true) && !fallback::_in_a_loss_thread(memory) {
        return choose!(RELIEF, memory, clean);
    }
    if goodnews::reads_as_settled(&text::lower(&clean))
        && goodnews::worry_on_the_table(memory, &Value::Int(4))
        && !fallback::_in_a_loss_thread(memory)
    {
        return choose!(RELIEF, memory, clean);
    }
    if _reads(badnews::reads_as_a_closed_door, &clean, &repaired, false) {
        return choose!(CLOSED_DOOR, memory, clean);
    }
    if fallback::_came_back(&clean) && fallback::in_a_safety_thread(memory) {
        return choose!(PAUSE_IN_A_SAFETY_THREAD, memory, clean);
    }
    if fallback::_came_back(&clean) && !fallback::_grief_thread(memory) {
        return choose!(CAME_BACK, memory, clean);
    }
    if _reads(badnews::reads_as_the_leaving, &clean, &repaired, false)
        && !_reads(badnews::reads_as_the_speakers_own_plan, &clean, &repaired, false)
        && !_a_things_state(memory, &clean)
    {
        return if badnews::reads_as_a_leaving_already_gone(&clean) { choose!(LEFT_POOL_GONE, memory, clean) } else { choose!(LEFT_POOL, memory, clean) };
    }
    if let Some(situation) = pool_of(crate::compose::circumstance::pool(&clean)) {
        return choose!(situation, memory, clean);
    }
    if let Some(broken_into) = text_of(crate::compose::remark::break_in_reply_for(&clean, &words, memory)) {
        return Some(broken_into);
    }
    if _reads(badnews::reads_as_bad_news, &clean, &repaired, false)
        && !_reads(badnews::reads_as_the_speakers_own_plan, &clean, &repaired, false)
        && PLAN_IDIOM.search(&clean).is_none()
        && !(_reads(badnews::reads_as_an_award_lost, &clean, &repaired, false) && truthy(fallback::who_settles(&clean, memory)))
    {
        if _a_things_state(memory, &clean) {
            return choose!(A_THING_LOOKS_SAD, memory, clean);
        }
        if _reads(badnews::reads_as_the_leaving, &clean, &repaired, false) {
            return if badnews::reads_as_a_leaving_already_gone(&clean) { choose!(LEFT_POOL_GONE, memory, clean) } else { choose!(LEFT_POOL, memory, clean) };
        }
        if opt_string_of(crate::compose::remark::reads_as_report(&clean, &words)).as_deref() == Some("again") {
            if let Some(again) = text_of(crate::compose::remark::report_reply_for(&clean, &words, memory)) {
                return Some(again);
            }
        }
        if !asking && _reads(fallback::reads_as_someone_close_is_ill, &clean, &repaired, false) {
            return choose!(SOMEONE_SERIOUSLY_ILL, memory, clean);
        }
        return if _reads(badnews::reads_as_news_with_a_name, &clean, &repaired, false) { choose!(NAMED_NEGATIVE, memory, clean) } else { choose!(NEGATIVE, memory, clean) };
    }
    if !asking
        && !fallback::_in_a_loss_thread(memory)
        && _reads(|t: &str| LEAVING_A_PERSON.search(t).is_some(), &clean, &repaired, false)
        && _reads(|t: &str| LEFT_BEHIND.search(t).is_some(), &clean, &repaired, false)
        && !_reads(|t: &str| NOT_LEAVING_ANYONE.search(t).is_some(), &clean, &repaired, false)
    {
        let is_a_child = _reads(|t: &str| A_CHILD.search(t).is_some(), &clean, &repaired, false);
        let has_happened = _reads(|t: &str| THE_LEAVE_HAS_HAPPENED.search(t).is_some(), &clean, &repaired, false);
        let pool = if has_happened {
            if is_a_child { A_CHILD_LEFT.clone() } else { SOMEONE_GONE.clone() }
        } else {
            _only_what_the_line_gives(if is_a_child { &A_CHILD_LEAVING } else { &SOMEONE_LEAVING }, &clean)
        };
        return choose!(pool, memory, clean);
    }
    if _reads(|t: &str| ACK_COMPLAINT.search(t).is_some(), &clean, &repaired, false) {
        if _a_things_state(memory, &clean) {
            return choose!(A_THING_LOOKS_SAD, memory, clean);
        }
        return choose!(NEGATIVE, memory, clean);
    }
    let clean_low = text::lower(&clean);
    let repaired_low = text::lower(&repaired);
    let a_good_time_here = |memory: &Memory| -> bool { _reads_value(|t: &str| fallback::_a_good_time(t, memory), &clean, &repaired) };
    if _asks_the_bot(&understood)
        && !_takes_something_back(&clean, memory)
        && !a_good_time_here(memory)
        && !_reads(goodnews::reads_as_a_win_nobody_noticed, &clean_low, &repaired_low, false)
        && !fallback::_a_good_time(&clean, memory)
        && !(fallback::_reads_as_dither(&clean, &repaired) && fallback::_weighs_a_choice(&clean))
        && !fallback::_a_dead_end(&clean)
        && fallback::_the_person_they_are_stuck_on(&clean).is_empty()
        && !fallback::_acknowledges_alone(&clean)
    {
        let tips = fallback::_tips(memory, &clean, &repaired);
        if !tips.is_empty() {
            return Some(tips);
        }
        if let Some(free_time) = text_of(fallback::_the_stretch_of_time(&clean, memory)) {
            return Some(free_time);
        }
        if let Some(lost) = fallback::the_lost_ask_reply(memory, &clean, &repaired).filter(|s| !s.is_empty()) {
            return Some(lost);
        }
        let pool = fallback::unknown_pool(&clean, &words);
        if pool == *UNKNOWN_WORD {
            let drafting = pool_of(crate::compose::fallback_pools::_drafting_pool(&clean)).unwrap_or_default();
            return choose!(drafting, memory, clean);
        }
        if let Some(routed) = crate::compose::finding_out::for_the_ask(&clean, &repaired, &understood).filter(|p| !p.is_empty()) {
            if let Some(said) = choose!(routed, memory, clean).filter(|s| !s.is_empty()) {
                return Some(said);
            }
        }
        if _a_wear_ask_about_a_day(&clean, &repaired, text_) {
            return choose!(_the_wear_pool_for_the_line(&clean, &repaired, text_), memory, clean);
        }
        return choose!(UNKNOWN, memory, clean);
    }
    if _reads(fallback::_is_a_tips_ask, &clean, &repaired, false) {
        let tips = fallback::_tips(memory, &clean, &repaired);
        if !tips.is_empty() {
            return Some(tips);
        }
        return choose!(TIPS_LINES, memory, clean);
    }
    if let Some(told) = text_of(fallback::_trouble(memory, &repaired, &words)) {
        return Some(told);
    }
    if fallback::asks_for_more(&clean, memory) && !fallback::_hedged_recently(memory, 3) && fallback::_answered_recently(memory) {
        return choose!(HELP_ALREADY, memory, clean);
    }
    if fallback::_in_sudden_loss_thread(memory) {
        return choose!(LOSS_AFTER_REPLIES, memory, clean);
    }
    if words.len() <= 2 && meets(&words, &ACKS) {
        if fallback::_a_loss_recently(memory) {
            return choose!(LOSS_AFTER_REPLIES, memory, clean);
        }
        if fallback::_the_last_line_was_bad_news(memory) {
            return choose!(LOSS_AFTER_REPLIES, memory, clean);
        }
        if fallback::_the_last_line_was_a_fault(memory) {
            return choose!(ACK_AFTER_TROUBLE, memory, clean);
        }
        if fallback::_the_last_line_was_heavy(memory) {
            return choose!(ACK_WARM, memory, clean);
        }
        return choose!(ACK, memory, clean);
    }
    if !fallback::_in_a_loss_thread(memory) {
        if FORGETS_A_CLASH.search(&plain).is_some() {
            return choose!(DOUBLE_BOOKED, memory, clean);
        }
        if FORGET_A_DATE.search(&plain).is_some() {
            return choose!(DATES_KEEP_SLIPPING, memory, clean);
        }
        if FORGET_A_SECRET.search(&plain).is_some() {
            return choose!(SECRETS_KEEP_SLIPPING, memory, clean);
        }
    }
    if FORGETS_AGAIN.search(&plain).is_some() && !fallback::_in_a_loss_thread(memory) {
        return choose!(KEEPS_FORGETTING, memory, clean);
    }
    if fallback::_is_rumination(&clean) {
        return choose!(RUMINATION_REPLIES, memory, clean);
    }
    let mut mood = _mood(&words, &clean);
    if let Some(carried) = pool_of(crate::compose::trouble::carried_on(&clean, &words, memory)) {
        return choose!(carried, memory, clean);
    }
    if _a_near_miss(&clean) {
        return choose!(NEAR_MISS_REPLY, memory, clean);
    }
    if mood == "neutral" && !repaired.is_empty() && repaired != clean {
        mood = _mood(&_words(&repaired), &repaired);
    }
    if mood != "positive" && _reads(badnews::reads_for_a_pet, &clean_low, &repaired_low, false) {
        if _reads(badnews::reads_as_a_pet_low, &clean_low, &repaired_low, false) {
            return choose!(PET_LOW_REPLY, memory, clean);
        }
        let condition = _reads(badnews::reads_as_a_pet_condition, &clean_low, &repaired_low, false);
        return if condition { choose!(PET_CONDITION_TROUBLE, memory, clean) } else { choose!(PET_TROUBLE, memory, clean) };
    }
    if mood != "positive" && _reads(|t: &str| badnews::reads_as_a_carer_body(t, memory), &clean_low, &repaired_low, false) {
        return choose!(CARING_FOR_A_BODY, memory, clean);
    }
    if mood != "positive" && _reads(|t: &str| badnews::reads_for_a_carer(t, memory), &clean_low, &repaired_low, false) {
        return choose!(CARING, memory, clean);
    }
    if mood == "neutral" && _reads(badnews::reads_like_a_symptom, &clean_low, &repaired_low, false) && not_sore(memory) {
        return if _ILL.search(&repaired).is_some() { choose!(SYMPTOM_COLD, memory, clean) } else { choose!(SYMPTOM, memory, clean) };
    }
    if fallback::_a_relationship_ending(&clean) {
        return choose!(RELATIONSHIP_DECISION, memory, clean);
    }
    if fallback::a_comparison_of_standing(&clean) && !fallback::_in_a_loss_thread(memory) {
        return choose!(A_COMPARISON_OF_STANDING, memory, clean);
    }
    if mood != "negative" && !crate::compose::limit::a_limit_on_the_answer(&clean) && fallback::_something_funny(&clean, memory) {
        return choose!(A_LAUGH, memory, clean);
    }
    if !fallback::_in_a_care_thread(memory) && _the_crowd_is_ahead(&clean) {
        return choose!(A_CROWD_AHEAD, memory, clean);
    }
    if mood != "negative" && a_good_time_here(memory) {
        if fallback::_in_a_care_thread(memory) {
            return choose!(A_GOOD_TIME_IN_A_CARE_THREAD, memory, clean);
        }
        return choose!(A_GOOD_TIME, memory, clean);
    }
    if mood != "negative" && _reads(goodnews::reads_as_a_win_nobody_noticed, &clean_low, &repaired_low, false) {
        return choose!(WIN_IN_A_SILENCE, memory, clean);
    }
    if mood != "negative" && _reads(goodnews::reads_as_a_win_a_cost_came_with, &clean_low, &repaired_low, false) {
        return choose!(WIN_AND_A_COST, memory, clean);
    }
    let occasion = {
        let first = goodnews::reads_as_an_occasion(&clean_low);
        if truthy(first.clone()) {
            Some(first)
        } else if repaired != clean {
            let second = goodnews::reads_as_an_occasion(&repaired_low);
            if truthy(second.clone()) { Some(second) } else { None }
        } else {
            None
        }
    };
    if mood != "negative" {
        if let Some(occasion) = occasion {
            let pool = pool_of(fallback::_occasion_pool(&occasion_value(occasion), &clean)).unwrap_or_default();
            return choose!(pool, memory, clean);
        }
    }
    let ahead_pool = _reads_value(fallback::_a_day_ahead, &clean, &repaired);
    if mood != "negative" {
        if let Some(ahead) = ahead_pool {
            return choose!(ahead, memory, clean);
        }
    }
    if mood != "negative" && _reads(goodnews::reads_as_news_to_come, &clean_low, &repaired_low, false) {
        if let Some(birthday) = _a_birthday_still_to_come(&clean, memory) {
            return choose!(_the_birthday_said_back(&birthday), memory, clean);
        }
        return choose!(NEWS_TO_COME, memory, clean);
    }
    if truthy(crate::memory::facts::a_limit_on_the_plan(&clean)) {
        if let Some(said_back) = text_of(crate::memory::ack::a_limit_reply_text(text_, memory)) {
            return Some(said_back);
        }
    }
    if mood == "neutral" && fallback::_dreading(&clean) {
        return choose!(A_DREAD, memory, clean);
    }
    if mood == "neutral" && fallback::_wants_it_over(&clean) {
        return choose!(A_WANT_IT_OVER, memory, clean);
    }
    if mood != "negative" && _reads_value(fallback::_looking_forward, &clean, &repaired) {
        return choose!(NEWS_TO_COME, memory, clean);
    }
    if mood == "neutral" && _reads(goodnews::reads_as_good_news, &clean_low, &repaired_low, false) {
        return choose!(fallback::_says_only_what_they_said(&CONGRATS, &clean), memory, clean);
    }
    let arrived = goodnews::new_thing(&clean_low);
    if truthy(arrived.clone()) && mood != "negative" {
        let pool = pool_of(goodnews::new_thing_replies(&occasion_value(arrived))).unwrap_or_default();
        return choose!(pool, memory, clean);
    }
    if let Some(regretted) = text_of(crate::compose::remark::regret_reply_for(&clean, &words, memory, &understood.plain)) {
        return Some(regretted);
    }
    if let Some(idiom) = PLAN_IDIOM.search(&clean) {
        return if !idiom.as_str().contains("excit") { choose!(IDIOM_SHY, memory, clean) } else { choose!(IDIOM_GLAD, memory, clean) };
    }
    if fallback::_plan_reported(&clean, &mood) && crate::memory::ack::reads_as_a_trip(&clean).map_or(true, |t| t.is_empty()) {
        if fallback::reads_as_a_change_of_medicine(&clean) {
            let whole: &Vec<String> = if PLAN_MEDICINE_SLOW.search(&clean).is_some() { &PLAN_MEDICINE_SLOW_REPLY } else { &PLAN_MEDICINE_REPLY };
            return choose!(whole, memory, clean);
        }
        if PLAN_DONE.iter().any(|phrase| clean.contains(phrase.as_str())) && PLAN_NOT_YET.search(&clean).is_none() {
            return choose!(PLAN_DONE_ACK, memory, clean);
        }
        return choose!(PLAN, memory, clean);
    }
    let promise = opt_pool_exact(fallback::_a_promise_that_keeps_moving(&clean));
    if let Some(promise) = promise {
        if mood != "positive" {
            return choose!(promise, memory, clean);
        }
    }
    if mood != "positive" && fallback::_someone_else_minding(&clean, &words, fallback::_grievance_thread(memory)) {
        if fallback::_grief_thread(memory) {
            return choose!(GRIEF_AFTER, memory, clean);
        }
        return choose!(GRIEVANCE, memory, clean);
    }
    if mood == "neutral" && fallback::_sorry_for_asking(&clean) {
        return choose!(SORRY_FOR_ASKING, memory, clean);
    }
    let said = fallback::_thanks_then_said(&clean);
    if truthy(fallback::_thanks_news_went_badly(&clean)) {
        return choose!(THANKS_NO_BETTER, memory, clean);
    }
    if !said.is_empty() {
        let said_words = A_WORD.findall_str(&said);
        if THANKS_GOOD_AGAIN.search(&said).is_some() || (THANKS_WENT_BADLY.search(&said).is_none() && _mood(&said_words, &said) != "negative") {
            return choose!(THANKS_THEN_SAID, memory, clean);
        }
    }
    if mood == "neutral" && (fallback::_thanks_with_more(&clean) || fallback::_acknowledges_alone(&clean)) {
        return choose!(THANKS, memory, clean);
    }
    if mood == "neutral"
        && (DITHER.search(&clean).is_some() || DITHER.search(&repaired).is_some())
        && HELP_SERVICE.search(&clean).is_some()
    {
        if clean.contains("vet")
            || clean.contains("vets")
            || words.iter().any(|w| ANIMALS.contains(w) || ANIMALS.contains(text::rstrip_chars(w, "s")))
        {
            return Some(VET_WHO.clone());
        }
        return choose!(UNKNOWN_HEALTH, memory, clean);
    }
    if mood == "neutral" {
        if let Some(about_a_person) = text_of(fallback::_a_question_about_the_person(&clean, memory)) {
            return Some(about_a_person);
        }
    }
    if let Some(free_time) = text_of(fallback::_the_stretch_of_time(&clean, memory)) {
        return Some(free_time);
    }
    if mood == "neutral" && fallback::_reads_as_dither(&clean, &repaired) && fallback::_weighs_a_choice(&clean) {
        if (meets(&words, &HEALTH_WORDS) && HEALTH_MODIFIER.search(&clean).is_none())
            || CHECK_PHRASES.iter().any(|phrase| clean.contains(phrase.as_str()))
        {
            return choose!(UNKNOWN_HEALTH, memory, clean);
        }
        if DITHER_YES_NO.search(&clean).is_some() {
            return choose!(DITHER_DECIDE, memory, clean);
        }
        if DITHER_OPEN.search(&clean).is_some() {
            return choose!(DITHER_OPEN_LINES, memory, clean);
        }
        return choose!(DITHER_TWO_WAY_LINES, memory, clean);
    }
    if mood == "neutral" && fallback::_run_out(&clean) && !fallback::_in_a_care_thread(memory) {
        return choose!(RUN_OUT_LINES, memory, clean);
    }
    if mood == "neutral" && fallback::_only_the_stuck_phrase(&clean) && !fallback::_in_a_care_thread(memory) {
        if A_HELP_WORD.search(&clean).is_some() {
            return choose!(HAND, memory, clean);
        }
        if fallback::_news_above(memory, false) || fallback::_a_subject_above(memory) || fallback::_a_bad_line_above(memory) {
            return choose!(STUCK_AFTER_TROUBLE, memory, clean);
        }
        if !(STUCK.match_(&clean).is_some() || fallback::_a_start_ask(&clean)) {
            return choose!(DITHER_OPEN_LINES, memory, clean);
        }
    }
    if _a_problem_stated(&clean, Some(&repaired)) {
        return choose!(HAND, memory, clean);
    }
    if let Some(amount) = text_of(crate::compose::quantity::reply_for(&clean, memory)) {
        return Some(amount);
    }
    if let Some(named_after) = _named_after_reply(&clean, memory) {
        if !named_after.is_empty() {
            return Some(named_after);
        }
    }
    if mood == "neutral" {
        if let Some(own) = pool_of(fallback::_own_feeling(&clean)) {
            return choose!(own, memory, clean);
        }
        if fallback::_a_knock_on_oneself(&clean) {
            return choose!(KNOCKED, memory, clean);
        }
        if fallback::_a_knock_into_a_thing(&clean) {
            return choose!(WALKED_INTO_POOL, memory, clean);
        }
        if fallback::_a_mishap(&clean, &words) {
            return choose!(MISHAP_BACK, memory, clean);
        }
        if let Some(summons) = understood.summons.as_ref() {
            let summons_value = Value::from_plain(&serde_json::to_value(summons).unwrap_or_default());
            let pool = pool_of(fallback::_summons_pool(&summons_value, &clean)).unwrap_or_default();
            return choose!(pool, memory, clean);
        }
        let taking_up = understood.taking_up.as_ref().map(|t| Value::from_plain(&serde_json::to_value(t).unwrap_or_default())).unwrap_or(Value::None);
        if let Some(taken) = text_of(crate::compose::remark::interest_reply_for(&taking_up, memory)) {
            return Some(taken);
        }
        if let Some(planned) = crate::compose::remark::reply_for(&clean, &words, memory, &understood.topics).filter(|s| !s.is_empty()) {
            return Some(planned);
        }
        if let Some(arranged) = text_of(crate::compose::remark::arranged_reply_for(&clean, &words, memory)) {
            return Some(arranged);
        }
        let pool = fallback::_thing_done_pool(&clean).filter(|p| !p.is_empty());
        if mood != "negative" {
            if let Some(pool) = pool {
                return choose!(pool, memory, clean);
            }
        }
        if let Some(reported) = text_of(crate::compose::remark::report_reply_for(&clean, &words, memory)) {
            return Some(reported);
        }
        if !asking && reads_as_a_flat_habit(text_, &understood) {
            return choose!(HABIT_REPLY, memory, clean);
        }
        if let Some(together) = text_of(crate::compose::remark::the_two_of_them(&clean, memory)) {
            return Some(together);
        }
        if let Some(of_theirs) = text_of(crate::compose::remark::their_people_reply_for(&clean, &words, memory)) {
            return Some(of_theirs);
        }
        if let Some(got_ahead) = _a_thing_got_ahead(&clean) {
            return choose!(got_ahead, memory, clean);
        }
        if let Some(going_on) = text_of(crate::compose::remark::their_mood_going_on_reply_for(&clean, memory)) {
            return Some(going_on);
        }
        if let Some(mooded) = text_of(crate::compose::remark::their_mood_with_you_reply_for(&clean, &words, memory)) {
            return Some(mooded);
        }
        let wear = crate::compose::worn::reads_as_worn(&clean, &words, &repaired);
        if truthy(wear.clone()) {
            let pool = pool_of(crate::compose::worn::pool(&occasion_value(wear))).unwrap_or_default();
            return choose!(pool, memory, clean);
        }
        if let Some(disliked) = text_of(fallback::the_dislike_ask(&clean, memory)) {
            return Some(disliked);
        }
        let said_back = crate::memory::ack::reply(text_, memory);
        if crate::compose::remark::a_want_said_back(said_back.as_deref(), &clean) {
            if let Some(theirs) = text_of(crate::compose::remark::dependant_want_reply_for(&clean, &words, memory)) {
                return Some(theirs);
            }
        }
        if let Some(told) = _a_day_told_instead_of_the_receipt(said_back.as_deref(), &clean, memory) {
            if !told.is_empty() {
                return Some(told);
            }
        }
        if let Some(said_back) = said_back.filter(|s| !s.is_empty()) {
            return Some(said_back);
        }
    }
    if mood == "neutral" {
        if let Some(of_theirs) = text_of(crate::compose::remark::their_people_reply_for(&clean, &words, memory)) {
            return Some(of_theirs);
        }
        if _the_morning_has_a_clock(&clean) {
            let at = _the_time_said(&clean).map(|(m, _)| m.str(1).to_string()).unwrap_or_default();
            return choose!(_the_clock_pool(&at), memory, clean);
        }
        let said_back = crate::memory::ack::reply(text_, memory);
        if crate::compose::remark::a_want_said_back(said_back.as_deref(), &clean) {
            if let Some(theirs) = text_of(crate::compose::remark::dependant_want_reply_for(&clean, &words, memory)) {
                return Some(theirs);
            }
        }
        if let Some(told) = _a_day_told_instead_of_the_receipt(said_back.as_deref(), &clean, memory) {
            if !told.is_empty() {
                return Some(told);
            }
        }
        if let Some(said_back) = said_back.filter(|s| !s.is_empty()) {
            return Some(said_back);
        }
        let shape = crate::compose::trouble::reads_as_trouble(&clean, Some(words.as_slice()), &repaired);
        if truthy(shape.clone()) {
            let pool = pool_of(crate::compose::trouble::pool(&occasion_value(shape), &clean, memory)).unwrap_or_default();
            if let Some(said) = choose!(pool, memory, clean).filter(|s| !s.is_empty()) {
                return Some(said);
            }
        }
    }
    if mood == "negative" {
        let shape = crate::compose::trouble::reads_as_trouble(&clean, Some(words.as_slice()), &repaired);
        if opt_string_of(shape.clone()).as_deref() == Some("sore") && _CAUGHT.search(&clean).is_none() && FRIGHTENED.search(&clean).is_none() {
            let pool = pool_of(crate::compose::trouble::pool(&occasion_value(shape), &clean, memory)).unwrap_or_default();
            if let Some(said) = choose!(pool, memory, clean).filter(|s| !s.is_empty()) {
                return Some(said);
            }
        }
    }
    let mut landed: Option<String> = None;
    if !(fallback::_in_a_loss_thread(memory) || fallback::_grief_thread(memory)) {
        landed = opt_string_of(fallback::_the_news_landed(&clean));
    }
    match landed.as_deref() {
        Some("bad") => return choose!(LANDED_BAD, memory, clean),
        Some("good") => return choose!(fallback::_says_only_what_they_said(&CONGRATS, &clean), memory, clean),
        Some("landed") => return choose!(PENDING_LANDED, memory, clean),
        _ => {}
    }
    let answered = opt_string_of(crate::compose::toknow::ask_about_what_to_look_out_for(&clean, memory));
    if let Some(answered) = answered.as_ref().filter(|s| !s.is_empty()) {
        return Some(answered.clone());
    }
    if answered.is_none() && fallback::_an_open_result(&clean) {
        let pool: &Vec<String> = if fallback::_a_time_in_it(&clean) {
            &PENDING_WAITING
        } else if fallback::_a_guessed_result(&clean) {
            &PENDING_DOUBT
        } else {
            &PENDING_WHEN
        };
        return choose!(pool, memory, clean);
    }
    if opens_with_hello && mood == "neutral" {
        return choose!(GREETING, memory, clean);
    }
    if _reads_as_a_bad_stretch(&clean) {
        return choose!(BAD_STRETCH_REPLIES, memory, clean);
    }
    if _says_there_is_nothing_to_wear(&clean, &repaired, text_) {
        return choose!(NOTHING_TO_WEAR_REPLIES, memory, clean);
    }
    if _a_wear_ask_about_a_day(&clean, &repaired, text_) {
        return choose!(_the_wear_pool_for_the_line(&clean, &repaired, text_), memory, clean);
    }
    if mood == "neutral" && _A_CORRECTION.match_(&clean).is_none() && crate::memory::swap::rewritten(&clean, memory).is_some() {
        return choose!(UNKNOWN, memory, clean);
    }
    if crate::understanding::kinds::verdicts_a_thing(&clean) {
        return choose!(NEGATIVE, memory, clean);
    }
    let mut state = _body_state(&clean);
    if state.is_none() {
        repaired = understood.plain.clone();
        if !repaired.is_empty() && repaired != clean {
            state = _body_state(&repaired);
        }
    }
    if let Some(state) = state.filter(|s| !s.is_empty()) {
        return choose!(pool_at(&BODY_POOLS, &state), memory, clean);
    }
    let pool = fallback::_thing_done_pool(&clean).filter(|p| !p.is_empty());
    if mood != "negative" {
        if let Some(pool) = pool {
            if !_a_things_state(memory, &clean) {
                return choose!(pool, memory, clean);
            }
        }
    }
    if mood == "negative" && _a_things_state(memory, &clean) {
        return choose!(A_THING_LOOKS_SAD, memory, clean);
    }
    if A_BIT_PAST_IT.search(&clean).is_some()
        && fallback::_a_thing_they_started(memory, &Value::Int(3))
        && !fallback::_hedged_recently(memory, 3)
        && !fallback::_thread_is_about_the_body(memory, 4)
    {
        return if fallback::_names_their_age(&clean) { choose!(NOT_TOO_OLD, memory, clean) } else { choose!(NOT_FIT_ENOUGH, memory, clean) };
    }
    if crate::compose::leaving::reads_as_a_leave_with_a_trouble_in_it(&clean, &words) {
        if let Some(bye) = pool_of(crate::compose::leaving::pool(&clean, &words, true, memory, false)) {
            return choose!(bye, memory, clean);
        }
    }
    if fallback::_a_cost_somebody_asks(&clean) {
        return choose!(A_COST_SOMEBODY_ASKS, memory, clean);
    }
    if let Some(lost) = fallback::the_lost_ask_reply(memory, &clean, &repaired).filter(|s| !s.is_empty()) {
        return Some(lost);
    }
    if RESULT_WENT_WRONG.search(&clean).is_some() {
        return choose!(A_THING_CAME_OUT_WRONG, memory, clean);
    }
    if PLAN_THAT_FELL_THROUGH.search(&clean).is_some() {
        return choose!(A_PLAN_THAT_FELL_THROUGH, memory, clean);
    }
    let state = if RESULT_WENT_WRONG.search(&clean).is_some() || mood == "positive" { None } else { _a_state_of_theirs(&clean) };
    if let Some(state) = state {
        return if state == "then" { choose!(A_STATE_PAST, memory, clean) } else { choose!(A_STATE_NOW, memory, clean) };
    }
    if _a_bad_time_told(&clean, memory) {
        return choose!(A_BAD_TIME_TOLD, memory, clean);
    }
    if _a_day_asked_without_a_cheer(&clean, memory) {
        return choose!(A_DAY_ASKED_PLAIN, memory, clean);
    }
    if let Some(named) = _a_name_told_reply(&clean, memory) {
        if !named.is_empty() {
            return Some(named);
        }
    }
    if let Some(suggestion) = understood.suggestion.as_ref() {
        if !asking {
            let who = suggestion.who.clone();
            if !who.is_empty() {
                return choose!(formatted(&A_SUGGESTED_BY_SOMEONE, &[("who", &who)]), memory, clean);
            }
            return choose!(A_SUGGESTED, memory, clean);
        }
    }
    if _a_thing_they_regret(&clean) {
        return choose!(A_THING_THEY_REGRET, memory, clean);
    }
    if _a_betrayal(&clean) {
        return choose!(A_BETRAYAL, memory, clean);
    }
    if let Some(care) = _a_care_home_line(Some(&understood), memory) {
        if !care.is_empty() {
            return Some(care);
        }
    }
    if let Some(wrecked) = pool_of(crate::compose::finding_out::the_wrecked_form_reply(&clean)) {
        if let Some(said) = choose!(wrecked, memory, clean).filter(|s| !s.is_empty()) {
            return Some(said);
        }
    }
    if let Some(sum_said) = _a_sum_with_nothing_beside_it(&clean, Some(&*memory)) {
        if !sum_said.is_empty() {
            return choose!(formatted(&A_SUM_STATED, &[("said", &sum_said)]), memory, clean);
        }
    }
    if let Some(rest) = _the_rest_of_the_day(&clean, memory, &mood) {
        return choose!(rest, memory, clean);
    }
    if let Some(reported) = _a_report_from_elsewhere(&clean) {
        return choose!(reported, memory, clean);
    }
    if mood == "neutral" && _a_small_day(&clean, memory) && !_just_asked_about_the_day(memory) {
        return choose!(A_SMALL_DAY, memory, clean);
    }
    if A_SORRY_ON_ITS_OWN.match_(text::strip(&clean)).is_some() {
        return choose!(SORRY_ON_ITS_OWN, memory, clean);
    }
    if A_DISMISSAL.match_(text::strip(&clean)).is_some() {
        return choose!(NEVER_MIND, memory, clean);
    }
    if _takes_something_back(&clean, memory) {
        return choose!(A_CORRECTION_TAKEN, memory, clean);
    }
    let positive = fallback::_says_only_what_they_said(&POSITIVE, &clean);
    if mood == "neutral" {
        if let Some(one_thing) = pool_of(crate::compose::onething::pool(&clean, &understood, memory)) {
            return choose!(one_thing, memory, clean);
        }
        if let Some(narrowed) = text_of(crate::compose::limit::a_limit_the_reading_missed(&clean, memory)) {
            return Some(narrowed);
        }
    }
    if reads_as_no_pleasure(&clean) {
        return choose!(PLEASURE_GONE_LINES, memory, clean);
    }
    match mood.as_str() {
        "positive" => choose!(positive, memory, clean),
        "negative" => choose!(NEGATIVE, memory, clean),
        "neutral" => choose!(NEUTRAL, memory, clean),
        other => panic!("KeyError: {other}"),
    }
}

/// `str` pattern for "settled" in `_reply_for`: `re.sub(r"[^a-z0-9']", " ", clean)`.
static NOT_AN_APOSTROPHE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9']", 0));

/// A value a reader gave back, handed on to the reader that takes it (an occasion, a new thing, a
/// shape): whatever it was, as a Python value.
fn occasion_value<T: PyRet>(v: T) -> Value {
    v.into_value()
}

/// `x is not None` for a reader that gives a pool or None: an empty pool still counts.
fn opt_pool_exact<T: PyRet>(v: T) -> Option<Vec<String>> {
    v.into_pool_exact()
}

/// The memory as a plain Python value, for a reader whose contract takes one.
#[allow(dead_code)]
fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Turn;

    fn memory_with(bots: &[&str]) -> Memory {
        let mut m = Memory::new();
        m.turns = bots.iter().map(|b| Turn { user: String::new(), bot: b.to_string(), module: None }).collect();
        m
    }

    // Expected values from PyBot (bot.compose.fallback_chain), clock pinned to 2026-11-17.
    #[test]
    fn choose_like_python() {
        let pool = owned(&["a", "b", "c"]);
        assert_eq!(_choose_opt(&pool, &memory_with(&["a"]), "hello", None).as_deref(), Some("b"));
        assert_eq!(_choose_opt(&pool, &memory_with(&["a", "b", "c"]), "hello", None).as_deref(), Some("b"));
        assert_eq!(_choose_opt(&pool, &Memory::new(), "hello", Some("seed me")).as_deref(), Some("a"));
        assert_eq!(_choose_opt(&owned(&["a"]), &memory_with(&["a"]), "hello", None).as_deref(), Some("a"));
        assert_eq!(_choose_opt(&[], &Memory::new(), "hello", None), None);
        assert_eq!(_mixed_seed("abc"), 304891);
        assert_eq!(_mixed_seed("Ünïcode line"), 159460);
    }

    #[test]
    fn format_like_python() {
        assert_eq!(py_format("{who} said {{hi}}", &[("who", "your mum")]), "your mum said {hi}");
        assert!(caught(|| py_format("{nobody}", &[("who", "x")])).is_none());
    }

    #[test]
    fn names_and_birthdays() {
        assert_eq!(_a_name_told("my kids are called jack and emma"), Some(("Jack and Emma".into(), "kids".into(), true)));
        assert_eq!(_a_name_told("my car is called betsy"), Some(("Betsy".into(), String::new(), false)));
        let mut b = Value::dict();
        b.set_key("who", "mum");
        b.set_key("to_come", 73i64);
        b.set_key("turns", "next  month");
        let said = _the_birthday_said_back(&b);
        assert_eq!(said[0], "Your mum turns 73 next month \u{2014} that's one to look forward to. Are you doing anything for it?");
        assert_eq!(said[2], "Lovely \u{2014} your mum at 73 next month. Will you mark it somehow?");
    }
    fn w(line: &str) -> Vec<String> {
        text::split(line)
    }

    #[test]
    fn word_readers_like_python() {
        assert!(_the_win_is_someone_elses(&w("all my mates are sorted and i havent"), 3));
        assert!(!_the_win_is_someone_elses(&w("everyone is happy for us"), 2));
        assert!(!_the_win_is_someone_elses(&w("my friend said i should be proud"), 6));
        assert!(!_a_thing_stopping_after(&w("my boiler has finally died"), 1));
        assert!(!_a_thing_stopping_after(&w("i gave up smoking"), 0));
        assert!(_a_thing_stopping_after(&w("my laptop has given up on me"), 1));
        assert!(_no_such_thing_before(&w("nobody makes me feel good"), 4));
        assert!(!_no_such_thing_before(&w("nobody believed i could but i did good"), 7));
        assert_eq!(_state_from("really very cold"), (Some("cold".into()), vec![]));
        assert_eq!(_state_from("cold and tired"), (Some("cold".into()), vec![]));
        assert_eq!(_state_from("cold water"), (None, vec![]));
        assert_eq!(_a_part_in_a_state("my nose is freezing cold").as_deref(), Some("cold"));
        assert_eq!(_a_part_in_a_state("my hands are a bit clammy").as_deref(), Some("clammy"));
        assert_eq!(_a_part_in_a_state("my face is hot"), None);
        assert!(_a_claim_about_a_thing("the weather is"));
        assert!(!_a_claim_about_a_thing("the week was"));
        assert!(_an_order_it_cannot_do("set a timer for 10 minutes").unwrap()[0].starts_with("No clock on this end"));
        assert!(_act_asked_of_me("can you order me a pizza").unwrap()[0].starts_with("I can't book, buy or pay"));
    }

    #[test]
    fn inline_patterns_compile() {
        NOT_A_WORD_CHAR.check().unwrap();
        SENTENCE_BREAK.check().unwrap();
        A_WORD.check().unwrap();
        MY_OR_OUR.check().unwrap();
        A_POSSESSIVE_PERSON.check().unwrap();
        AIMED_ELSEWHERE.check().unwrap();
        NOT_ALNUM_SPACE.check().unwrap();
        TWO_LETTERS.check().unwrap();
        NAME_JOIN.check().unwrap();
        A_CURRENCY_MARK.check().unwrap();
        NOT_AN_APOSTROPHE_WORD.check().unwrap();
    }

}
