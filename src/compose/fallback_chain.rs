//! Port of `bot/compose/fallback_chain.py`: the rows of `fallback` kept in their own file (the
//! composer's chain of readings, `reply_for`, and the small readers it is built from).
//!
//! The Python file is imported from the last line of `fallback.py`, so the two read as one
//! module; here the readings of `fallback.py` are called as `fallback::...`.
#![allow(non_snake_case, clippy::all, unused_variables)]

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
}

impl PyRet for () {
    fn truthy(&self) -> bool {
        false
    }
}
impl PyRet for bool {
    fn truthy(&self) -> bool {
        *self
    }
}
impl PyRet for i64 {
    fn truthy(&self) -> bool {
        *self != 0
    }
}
impl PyRet for usize {
    fn truthy(&self) -> bool {
        *self != 0
    }
}
impl PyRet for String {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_text(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
}
impl PyRet for &str {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_text(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self.to_string()) }
    }
}
impl PyRet for Vec<String> {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_pool(self) -> Option<Vec<String>> {
        if self.is_empty() { None } else { Some(self) }
    }
}
impl PyRet for &[String] {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_pool(self) -> Option<Vec<String>> {
        if self.is_empty() { None } else { Some(self.to_vec()) }
    }
}
impl PyRet for &Vec<String> {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
    fn into_pool(self) -> Option<Vec<String>> {
        if self.is_empty() { None } else { Some(self.clone()) }
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

/// Python: bot/compose/fallback_chain.py:2133 `_choose(options, memory, clean, seed_text=None)`,
/// as the contract has it: `seed_text` "" stands for None, and the None Python gives back when the
/// conversation has used the pool up is "".
pub fn _choose(options: &[String], memory: &Memory, clean: &str, seed_text: &str) -> String {
    _choose_opt(options, memory, clean, if seed_text.is_empty() { None } else { Some(seed_text) }).unwrap_or_default()
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
        None | Some(Value::None) => "None".into(),
        Some(Value::Bool(b)) => if *b { "True" } else { "False" }.into(),
        Some(Value::Int(i)) => i.to_string(),
        Some(Value::Float(f)) => text::float_repr(*f),
        Some(Value::Str(s)) => s.to_string(),
        Some(other) => format!("{:?}", other.to_plain()),
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
pub fn _a_countdown_in_a_care_thread(memory: &Memory, clean: &str, repaired: &str, understood: &Understood) -> Option<String> {
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
pub fn _a_dish_name(clean: &str, words: &[String], memory: &Memory) -> bool {
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
// Not ported yet.
// ---------------------------------------------------------------------------------------------


/// Python: bot/compose/fallback_chain.py:2986 `_reply_for(text, memory, understood=None, verdict=True)`
pub fn _reply_for(text: &str, memory: &mut Memory, understood: Option<&Understood>, verdict: bool) -> Option<String> {
    crate::unported!("bot.compose.fallback_chain:_reply_for")
}

