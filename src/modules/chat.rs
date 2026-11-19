//! Port of `bot/modules/chat.py`: small talk - greetings, thanks, short replies, goodbyes and
//! simple feelings.
//!
//! Reads the line the way a friend would. A short reply ("ok", "lol", "nah") answers the bot's
//! last turn, so it is read in that light and answered in kind. A feeling ("I'm knackered",
//! "long day") gets a warm, concrete line back. Everything else is left to the modules that know
//! about it: this one passes unless the whole line is small talk.
#![allow(non_snake_case, non_upper_case_globals, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

pub use self::bridge::*;

/// Calls into other workstreams' files go through here: Python's return values are converted
/// to what this file reads off them, whatever Rust type the callee's contract gives today.
pub mod bridge {
    use crate::data::Value;
    use crate::state::Memory;
    use std::sync::Arc;

    /// A callee's return value as the Python value it stands for.
    pub trait Ret {
        fn py(self) -> Value;
    }
    impl Ret for () {
        fn py(self) -> Value {
            Value::None
        }
    }
    impl Ret for Value {
        fn py(self) -> Value {
            self
        }
    }
    impl Ret for &Value {
        fn py(self) -> Value {
            self.clone()
        }
    }
    impl Ret for bool {
        fn py(self) -> Value {
            Value::Bool(self)
        }
    }
    impl Ret for i64 {
        fn py(self) -> Value {
            Value::Int(self)
        }
    }
    impl Ret for usize {
        fn py(self) -> Value {
            Value::Int(self as i64)
        }
    }
    impl Ret for String {
        fn py(self) -> Value {
            Value::Str(Arc::from(self.as_str()))
        }
    }
    impl Ret for &str {
        fn py(self) -> Value {
            Value::Str(Arc::from(self))
        }
    }
    impl Ret for Vec<String> {
        fn py(self) -> Value {
            Value::from(self)
        }
    }
    impl<T: Ret> Ret for Option<T> {
        fn py(self) -> Value {
            match self {
                Some(v) => v.py(),
                None => Value::None,
            }
        }
    }

    /// The Python value a call returned.
    pub fn py<R: Ret>(r: R) -> Value {
        r.py()
    }
    /// `bool(result)`.
    pub fn truthy<R: Ret>(r: R) -> bool {
        r.py().truthy()
    }
    /// The result when it is a non-empty str (`if reply: return reply`).
    pub fn said<R: Ret>(r: R) -> Option<String> {
        match r.py() {
            Value::Str(s) if !s.is_empty() => Some(s.to_string()),
            _ => None,
        }
    }
    /// The result when it is a str, empty or not (`is not None` on a str-or-None).
    pub fn opt_str<R: Ret>(r: R) -> Option<String> {
        match r.py() {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }

    /// The memory as the Python dict it is, for a callee whose contract takes a `Value`.
    pub fn mem_value(memory: &Memory) -> Value {
        Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
    }
}

// ---------------------------------------------------------------------------------------------
// Helpers for the Python idioms this file repeats.

/// `catch_unwind` for a Python `try: ... except Exception:` block: a real failure gives None (the
/// caller's fallback), while a call into a function nobody has ported yet still propagates, so
/// it is counted as unported rather than read as a Python exception.
pub fn attempt<T>(f: impl FnOnce() -> T) -> Option<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Some(v),
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| e.downcast_ref::<&str>().copied())
                .unwrap_or("");
            if msg.starts_with("UNPORTED") {
                std::panic::resume_unwind(e)
            }
            None
        }
    }
}

macro_rules! res {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.chat.", stringify!($n))));)* };
}
macro_rules! sets {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(concat!("bot.modules.chat.", stringify!($n))));)* };
}
macro_rules! vals {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Value> = LazyLock::new(|| data::get(concat!("bot.modules.chat.", stringify!($n))));)* };
}
/// An inline `re.compile(...)`/`re.search(r"...")` pattern, compiled once.
macro_rules! inline_re {
    ($name:ident, $pat:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, 0));
    };
    ($name:ident, $pat:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
    };
}

/// `[turn for turn in (memory.get("turns") or []) if isinstance(turn, dict)]`.
fn turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

fn in_set(set: &HashSet<String>, word: &str) -> bool {
    set.contains(word)
}

/// `" ".join(text.split())`.
fn squash(s: &str) -> String {
    text::squash(s)
}

inline_re!(AZ_APOS_WORDS, r"[a-z']+");

/// `re.findall(r"[a-z']+", text)`.
fn az_words(s: &str) -> Vec<String> {
    AZ_APOS_WORDS.findall_str(s)
}

/// `seq[-n:]` for a slice.
fn last_n<T>(v: &[T], n: usize) -> &[T] {
    &v[v.len().saturating_sub(n)..]
}

// ---------------------------------------------------------------------------------------------
// Python lines 89-2290: tokens, refusals, judgements, reactions.

res!(
    TOKEN, SAYS_TOO_MUCH, NOT_A_REFUSAL, KEEPS_FAILING, LET_SOMEONE_DOWN, NOT_FAIR, NOT_GREAT,
    SORRY_MET, NEWS_MET, AGREEMENT_ONLY, GRUMBLE, WEATHER, GOOD_NEWS_TURN, OWN_WIN, OWN_SETBACK,
    ASKED_AFTER, OWN_HARSH, OWN_WORTH, OWN_TRAIT, SOCIAL_TRAIT_SETTING, EVERYTHING_WRONG,
    LOST_JOB, JOB_STEP_ASK, POOR_THING,
);
sets!(
    NO_WORDS, NO_REFUSAL_CONTENT, NOT_THE_PERSONS_OWN, SELF_SUBJECT, SELF_WORDS,
    LEAD_IN_JUDGEMENT, FIRST_COPULA, NOT_FAIR_WORDS, OTHER_SUBJECT, REACTION_LEAD, REACTION_TAIL,
    REACTION_SUBJECT, REACTION_COPULA, REACTION_HOW, REACTION_SIGH, REACTION_SIGH_WORD,
    REACTION_BARE_GOOD, REACTION_BARE_FUNNY, REACTION_CHEERFUL, NEWS_MODULES, SELF_COPULA,
    JUDGEMENT_NOUNS, JUDGEMENT_AFTER, JUDGEMENT_TIME, HARSH_WORDS, ROLE_YOU_CAN_FAIL,
    ASKS_A_QUESTION, NOT_A_NIGHT_OUT,
);
vals!(REACTION_WORDS, REACTION_WHAT_A, NEWS_REPLIES, HANDS_IT_BACK);

/// Python: bot/modules/chat.py:164 `_only_a_refusal(clean, tokens)`
pub fn _only_a_refusal(clean: &str, tokens: &[String]) -> bool {
    if !tokens.iter().any(|t| in_set(&NO_WORDS, t)) {
        return false;
    }
    if NOT_A_REFUSAL.match_(clean).is_some() {
        return false;
    }
    !crate::memory::topics::keywords(clean)
        .iter()
        .any(|word| !in_set(&NO_REFUSAL_CONTENT, word) && !in_set(&NO_WORDS, word))
}

/// Python: bot/modules/chat.py:414 `_keeps_failing_theirs(clean)`: the phrase's span, when what
/// keeps failing is the person rather than something of theirs.
pub fn _keeps_failing_theirs(clean: &str) -> Option<(usize, usize)> {
    for m in KEEPS_FAILING.finditer(clean) {
        let all = az_words(&clean[..m.start()].to_lowercase());
        let before = last_n(&all, 3);
        if !before.iter().any(|w| in_set(&NOT_THE_PERSONS_OWN, w)) {
            return Some((m.start(), m.end()));
        }
        if before.iter().any(|w| in_set(&SELF_SUBJECT, w)) {
            return Some((m.start(), m.end()));
        }
    }
    None
}

/// Python: bot/modules/chat.py:431 `_letting_someone_down(clean)`: the match's span, or None.
pub fn _letting_someone_down(clean: &str) -> Option<(usize, usize)> {
    if let Some(m) = LET_SOMEONE_DOWN.search(clean) {
        return Some((m.start(), m.end()));
    }
    _keeps_failing_theirs(clean)
}

/// Python: bot/modules/chat.py:465 `_about_themselves(tokens)`
pub fn _about_themselves(tokens: &[String]) -> bool {
    if tokens.len() <= 4 {
        return true;
    }
    tokens.iter().any(|t| in_set(&SELF_WORDS, t))
}

/// Python: bot/modules/chat.py:488 `_themselves_unfair(tokens)`
pub fn _themselves_unfair(tokens: &[String]) -> bool {
    for (at, word) in tokens.iter().enumerate() {
        if word != "fair" && word != "unfair" {
            continue;
        }
        let mut window = &tokens[at.saturating_sub(4)..at];
        while !window.is_empty() && in_set(&LEAD_IN_JUDGEMENT, &window[0]) {
            window = &window[1..];
        }
        if window.is_empty() || !in_set(&FIRST_COPULA, &window[0]) {
            continue;
        }
        if window[1..].iter().any(|w| in_set(&OTHER_SUBJECT, w)) {
            continue;
        }
        if word == "unfair" || window.iter().any(|w| in_set(&NOT_FAIR_WORDS, w)) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/chat.py:566 `_the_answer_met_it_as_news(memory)`
pub fn _the_answer_met_it_as_news(memory: &Memory) -> bool {
    for turn in turns(memory).iter().rev() {
        let line = &turn.bot;
        if !line.is_empty() {
            return SORRY_MET.search(line).is_some() || NEWS_MET.search(line).is_some();
        }
    }
    false
}

/// Python: bot/modules/chat.py:576 `_a_loss_before(said, memory=None)`
pub fn _a_loss_before(said: &str, memory: &Memory) -> bool {
    attempt(|| {
        let last = turns(memory);
        if let Some(turn) = last.last() {
            let module = turn.module.as_deref().unwrap_or("");
            if !module.is_empty() {
                return in_set(&NEWS_MODULES, module);
            }
        }
        if crate::compose::loss::reads_as_loss(said, None, false) || crate::compose::loss::a_pet_died(said) {
            return true;
        }
        crate::compose::badnews::reads_as_sudden_loss(said)
    })
    .unwrap_or(false)
}

/// Python: bot/modules/chat.py:600 `_a_thread_to_come_back_to(memory)`
pub fn _a_thread_to_come_back_to(memory: &Memory) -> bool {
    truthy(crate::memory::thread::earlier_turn(memory))
}

/// Python: bot/modules/chat.py:612 `_news_before(memory)`
pub fn _news_before(memory: &Memory) -> String {
    turns(memory).last().map(|t| t.user.to_lowercase()).unwrap_or_default()
}

/// Python: bot/modules/chat.py:619 `_nobody_would_call_it_nice(said)`
pub fn _nobody_would_call_it_nice(said: &str) -> bool {
    if said.is_empty() {
        return false;
    }
    crate::compose::loss::reads_as_loss(said, None, false)
        || crate::compose::loss::a_pet_died(said)
        || crate::compose::badnews::reads_as_sudden_loss(said)
        || crate::compose::badnews::reads_as_bad_news(said)
}

/// Python: bot/modules/chat.py:666 `_the_agreement_pool(memory)`
pub fn _the_agreement_pool(memory: &Memory) -> Option<String> {
    let said = _news_before(memory);
    if said.is_empty() {
        return None;
    }
    if truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    if crate::compose::loss::reads_as_loss(&said, None, false)
        || crate::compose::loss::a_pet_died(&said)
        || crate::compose::badnews::reads_as_sudden_loss(&said)
    {
        return Some("agree_loss".into());
    }
    if crate::compose::goodnews::reads_as_good_news(&said) || crate::compose::goodnews::reads_as_a_pass(&said) {
        return Some("agree_good".into());
    }
    if crate::compose::badnews::reads_as_bad_news(&said) || GRUMBLE.search(&said).is_some() {
        if WEATHER.search(&said).is_some() {
            return Some("agree_weather".into());
        }
        return Some("agree_grim".into());
    }
    Some("agree_plain".into())
}

/// Python: bot/modules/chat.py:692 `_reaction(clean, tokens, memory)`
pub fn _reaction(clean: &str, tokens: &[String], memory: &Memory) -> Option<String> {
    let pool = _reaction_pool(clean, tokens, memory);
    if let Some(p) = &pool {
        if in_set(&REACTION_CHEERFUL, p) && _nobody_would_call_it_nice(&_news_before(memory)) {
            return Some("react_sigh".into());
        }
    }
    pool
}

/// Python: bot/modules/chat.py:700 `_reaction_pool(clean, tokens, memory)`
pub fn _reaction_pool(clean: &str, tokens: &[String], memory: &Memory) -> Option<String> {
    if clean.contains('?') || tokens.is_empty() || tokens.len() > 7 {
        return None;
    }
    if turns(memory).is_empty() {
        return None;
    }
    if attempt(|| truthy(crate::compose::reading::judges_a_figure(clean, memory))).unwrap_or(false) {
        return None;
    }
    let mut words: Vec<String> = tokens.iter().map(|w| text::strip_chars(w, ".,!?;:").to_string()).collect();
    while words.len() > 2 && in_set(&REACTION_LEAD, &words[0]) {
        words.remove(0);
    }
    while words.last().is_some_and(|w| in_set(&REACTION_TAIL, w)) {
        words.pop();
    }
    if words.is_empty() {
        return None;
    }
    if AGREEMENT_ONLY.match_(clean).is_some() {
        return _the_agreement_pool(memory);
    }
    if (words.len() == 3 || words.len() == 4) && words[0] == "what" && (words[1] == "a" || words[1] == "an") {
        let mut rest = &words[2..];
        const FILL: [&str; 9] = ["bloody", "right", "proper", "total", "absolute", "real", "complete", "utter", "such"];
        while !rest.is_empty() && FILL.contains(&rest[0].as_str()) {
            rest = &rest[1..];
        }
        let thing = rest.join(" ");
        if let Some(kind) = REACTION_WHAT_A.get(&thing) {
            let said = _news_before(memory);
            let kind = kind.str();
            if kind == "good" {
                if !said.is_empty() && crate::compose::goodnews::reads_as_good_news(&said) {
                    return Some("react_good".into());
                }
                return None;
            }
            if !said.is_empty() && !_a_loss_before(&said, memory) && _the_answer_met_it_as_news(memory) {
                return Some(format!("react_{kind}"));
            }
        }
        return None;
    }
    if words.len() == 2 && in_set(&REACTION_HOW, &words[0]) {
        if let Some(kind) = REACTION_WORDS.get(&words[1]) {
            return Some(format!("react_{}", kind.str()));
        }
    }
    if words.len() <= 3 && in_set(&REACTION_SIGH, &words[0]) && words[1..].iter().all(|w| in_set(&REACTION_SIGH_WORD, w)) {
        return Some("react_sigh".into());
    }
    let said = _news_before(memory);
    if words.len() <= 2 && words.iter().all(|w| in_set(&REACTION_BARE_GOOD, w)) {
        if !said.is_empty() && crate::compose::goodnews::reads_as_good_news(&said) {
            return Some("react_good".into());
        }
        if !said.is_empty() && _nobody_would_call_it_nice(&said) {
            return Some("react_sigh".into());
        }
        return None;
    }
    if words.len() <= 2 && words.iter().all(|w| in_set(&REACTION_BARE_FUNNY, w)) {
        return Some("react_funny".into());
    }
    if in_set(&REACTION_SUBJECT, &words[0]) {
        let mut rest = &words[1..];
        while !rest.is_empty() && in_set(&REACTION_COPULA, &rest[0]) {
            rest = &rest[1..];
        }
        while !rest.is_empty() && ["so", "really", "very", "proper", "well"].contains(&rest[0].as_str()) {
            rest = &rest[1..];
        }
        if rest.len() == 1 {
            if let Some(kind) = REACTION_WORDS.get(&rest[0]) {
                return Some(format!("react_{}", kind.str()));
            }
        }
        if rest == ["a", "lot"] || rest == ["a", "hell", "of", "a", "lot"] {
            return Some("react_bad".into());
        }
    }
    None
}

/// Python: bot/modules/chat.py:825 `_turns_on_news(clean)`
pub fn _turns_on_news(clean: &str) -> bool {
    let reads = |s: &str| {
        crate::compose::goodnews::reads_as_a_pass(s)
            || crate::compose::goodnews::reads_as_good_news(s)
            || OWN_WIN.search(s).is_some()
            || OWN_SETBACK.search(s).is_some()
            || crate::compose::badnews::reads_as_bad_news(s)
    };
    if reads(clean) {
        return true;
    }
    for m in GOOD_NEWS_TURN.finditer(clean) {
        if reads(m.str(1)) {
            return true;
        }
    }
    false
}

/// What `_judgement` holds as `judged`: a match (by its span), or a value with no `.end()`.
#[derive(Clone, Copy, Debug)]
pub enum Judged {
    Span(usize, usize),
    /// `True`, or the int position `topics.reads_as_unfit` returns: neither has `.end()`.
    NoEnd,
}

/// Python: bot/modules/chat.py:849 `_asks_after(clean, judged)`
pub fn _asks_after(clean: &str, judged: &Judged) -> bool {
    let Some(ask) = ASKED_AFTER.search(clean) else { return false };
    match judged {
        Judged::Span(_, end) => ask.start() > *end,
        // Python: `judged.end()` on an int raises AttributeError, which nothing in the chain
        // catches: the module is skipped for the turn.
        Judged::NoEnd => panic!("AttributeError: object has no attribute 'end'"),
    }
}

/// Python: bot/modules/chat.py:871 `_judged_as_a_noun(tokens)`
pub fn _judged_as_a_noun(tokens: &[String]) -> bool {
    for (at, word) in tokens.iter().enumerate() {
        if !in_set(&JUDGEMENT_NOUNS, word) {
            continue;
        }
        let before = &tokens[at.saturating_sub(4)..at];
        let has = |w: &str| before.iter().any(|b| b == w);
        if !before.iter().any(|b| in_set(&SELF_COPULA, b)) {
            if !((has("life") || has("everything") || has("it")) && has("is")) {
                continue;
            }
        }
        if !(has("a") || has("an") || has("the")) {
            continue;
        }
        if has("not") || has("never") || has("no") {
            continue;
        }
        let after = &tokens[(at + 1).min(tokens.len())..(at + 2).min(tokens.len())];
        if !after.is_empty() && !in_set(&JUDGEMENT_AFTER, &after[0]) {
            continue;
        }
        if _when_not_why(&tokens[(at + 1).min(tokens.len())..]) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/modules/chat.py:895 `_when_not_why(rest)`
pub fn _when_not_why(rest: &[String]) -> bool {
    if rest.len() < 2 || !["at", "on", "after", "before", "during", "every"].contains(&rest[0].as_str()) {
        return false;
    }
    let named: Vec<&String> = rest[1..].iter().filter(|w| !["the", "a", "an"].contains(&w.as_str())).collect();
    match named.first() {
        Some(first) => in_set(&JUDGEMENT_TIME, first) || in_set(&JUDGEMENT_TIME, &text::slice(first, 0, Some(-1))),
        None => false,
    }
}

/// Python: bot/modules/chat.py:909 `_judged_a_role(clean)`
pub fn _judged_a_role(clean: &str) -> bool {
    let Some(m) = OWN_HARSH.search(clean) else { return true };
    let words = az_words(&m.as_str().to_lowercase());
    for (index, word) in words.iter().enumerate() {
        if in_set(&HARSH_WORDS, word) && index + 1 < words.len() {
            return in_set(&ROLE_YOU_CAN_FAIL, &words[index + 1]);
        }
    }
    true
}

/// Python: bot/modules/chat.py:927 `_judgement(clean, tokens)`
pub fn _judgement(clean: &str, tokens: &[String]) -> Option<String> {
    if clean.contains('?') || tokens.len() > 14 || tokens.first().is_some_and(|t| in_set(&ASKS_A_QUESTION, t)) {
        return None;
    }
    if crate::compose::said_about_me::reads_as_somebody_elses_words(&clean.replace('\'', "")) {
        return None;
    }
    if let Some(kind) = _feel_like(clean) {
        if kind != "feel_like" {
            return None;
        }
    }
    if OWN_HARSH.search(clean).is_some() && _about_themselves(tokens) {
        return Some(if _judged_a_role(clean) { "own_harsh" } else { "own_described" }.into());
    }
    let span = |m: Option<Match>| m.map(|m| Judged::Span(m.start(), m.end()));
    let judged: Option<Judged> = span(OWN_WORTH.search(clean))
        .or_else(|| {
            if OWN_TRAIT.search(clean).is_some() && SOCIAL_TRAIT_SETTING.search(clean).is_none() {
                // `OWN_TRAIT.search(clean) and not SOCIAL_TRAIT_SETTING.search(clean)` is `True`.
                Some(Judged::NoEnd)
            } else {
                None
            }
        })
        .or_else(|| span(EVERYTHING_WRONG.search(clean)))
        .or_else(|| match py(crate::understanding::topics::reads_as_unfit(clean)) {
            v if v.truthy() => Some(Judged::NoEnd),
            _ => None,
        })
        .or_else(|| _letting_someone_down(clean).map(|(s, e)| Judged::Span(s, e)));
    if !(judged.is_some() && _asks_after(clean, judged.as_ref().unwrap())) {
        if _judged_as_a_noun(tokens) {
            return Some("not_enough".into());
        }
        if judged.is_some() && _about_themselves(tokens) && !_turns_on_news(clean) {
            return Some("not_enough".into());
        }
    }
    if NOT_FAIR.search(clean).is_some() {
        if _themselves_unfair(tokens) {
            return Some("own_fault".into());
        }
        return Some("not_fair".into());
    }
    if tokens.len() <= 8 && NOT_GREAT.search(clean).is_some() {
        return Some("not_great".into());
    }
    None
}

/// Python: bot/modules/chat.py:1313 `_a_job_that_went(said)`
pub fn _a_job_that_went(said: &str) -> bool {
    LOST_JOB.search(said).is_some() || crate::compose::badnews::reads_as_a_sacking(said)
}

/// Python: bot/modules/chat.py:1327 `_the_ask_after_a_job_that_went(clean, memory)`
pub fn _the_ask_after_a_job_that_went(clean: &str, memory: &Memory) -> Option<String> {
    if JOB_STEP_ASK.match_(text::strip(clean)).is_none() {
        return None;
    }
    for turn in last_n(turns(memory), 3).iter().rev() {
        let said = squash(&turn.user.to_lowercase());
        if said.is_empty() {
            continue;
        }
        if _a_job_that_went(&said) {
            return Some(pick(&data::strs("bot.memory.stuck.JOB"), memory, clean));
        }
        if text::split(&said).len() > 2 {
            return None;
        }
    }
    None
}

/// Python: bot/modules/chat.py:1467 `_not_a_night_out(place)`
pub fn _not_a_night_out(place: &str) -> bool {
    let low = squash(&place.to_lowercase().replace('&', " and "));
    if in_set(&NOT_A_NIGHT_OUT, &low) {
        return true;
    }
    let bare: Vec<&str> = text::split_ref(&low).into_iter().filter(|w| !["the", "a", "an"].contains(w)).collect();
    let joined = bare.join(" ");
    in_set(&NOT_A_NIGHT_OUT, &joined) || (!joined.is_empty() && in_set(&NOT_A_NIGHT_OUT, bare[0]))
}

/// Python: bot/modules/chat.py:1510 `_poor_word(flat)`
pub fn _poor_word(flat: &str) -> Option<String> {
    let m = POOR_THING.search(flat)?;
    Some(squash(m.name_str("poor")))
}

/// Python: bot/modules/chat.py:1652 `_asked_about_the_effort(memory)`
pub fn _asked_about_the_effort(memory: &Memory) -> bool {
    let Some(last) = turns(memory).last() else { return false };
    let said = &last.bot;
    for kind in ["at_it", "on_a_form", "on_the_thing", "the_long_haul"] {
        if NEWS_REPLIES.get(kind).is_some_and(|pool| pool.contains_str(said)) {
            return true;
        }
    }
    false
}

inline_re!(PLACEHOLDER_SPLIT, r"(\{[A-Za-z_]+\})");

/// Python: bot/modules/chat.py:2267 `_as_a_pattern(line)`
pub fn _as_a_pattern(line: &str) -> Re {
    let mut out = String::new();
    for part in PLACEHOLDER_SPLIT.split(line) {
        if part.starts_with('{') && part.ends_with('}') {
            out.push_str(".+");
        } else if !part.is_empty() {
            out.push_str(&crate::py::re::escape(&part));
        }
    }
    Re::new(&out, 0)
}

/// Python: bot/modules/chat.py:2283 `hands_it_back(reply)`
pub fn hands_it_back(reply: &str) -> bool {
    let said = text::strip(reply);
    HANDS_IT_BACK.items().iter().any(|p| p.as_re().is_some_and(|re| re.fullmatch(said).is_some()))
}

/// `compose.pick(options, memory, seed_text)`: its contract takes the memory as a `Value`.
pub fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, memory, seed_text)
}

// ---------------------------------------------------------------------------------------------
// More helpers.

static NO_MEMORY: LazyLock<Memory> = LazyLock::new(Memory::new);

static DYN_RE: LazyLock<std::sync::Mutex<std::collections::HashMap<(String, u32), std::sync::Arc<Re>>>> =
    LazyLock::new(Default::default);

/// A pattern built at run time (`re.search(r"\b" + re.escape(word) + r"\b", ...)`), compiled
/// once per pattern, as Python's `re` caches it.
pub fn dyn_re(pattern: &str, flags: u32) -> std::sync::Arc<Re> {
    let mut cache = DYN_RE.lock().unwrap_or_else(|e| e.into_inner());
    if cache.len() > 4096 {
        cache.clear();
    }
    cache.entry((pattern.to_string(), flags)).or_insert_with(|| std::sync::Arc::new(Re::new(pattern, flags))).clone()
}

/// `re.compile(r"\b" + re.escape(word) + r"\b")`.
fn word_re(word: &str) -> std::sync::Arc<Re> {
    dyn_re(&format!(r"\b{}\b", crate::py::re::escape(word)), 0)
}

/// Python's `template.format(**fields)` for `{name}` fields (and `{{`/`}}`); a field the
/// template names that is not given raises KeyError.
pub fn py_format(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut chars = template.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '{' if chars.peek().map(|p| p.1) == Some('{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek().map(|p| p.1) == Some('}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                let rest = &template[i + 1..];
                let close = rest.find('}').unwrap_or_else(|| panic!("ValueError: single '{{' in format string"));
                let name = &rest[..close];
                let value = fields
                    .iter()
                    .find(|(k, _)| *k == name)
                    .map(|(_, v)| *v)
                    .unwrap_or_else(|| panic!("KeyError: {name:?}"));
                out.push_str(value);
                for _ in 0..name.chars().count() + 1 {
                    chars.next();
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// The understood reading as the `Value` some contracts take.
pub fn understood_value(u: &Understood) -> Value {
    Value::from_plain(&serde_json::to_value(u).unwrap_or(serde_json::Value::Null))
}

// ---------------------------------------------------------------------------------------------
// Python lines 2290-4450.

res!(
    BUG_NAMED, SYMPTOM_IN_LINE, LOSS, BODY_FEELING_SHAPE, BODY_PART_FEELING, BODY_AT_THOUGHT,
    WORRY_WHEN, ASKS_IF_NORMAL, _HOW_I_READ_IT, _A_LIFE, _A_LOOSE_VERDICT, _THEIR_OWN_LIFE,
    _NOT_A_VERDICT_YET, AVOIDED_FEELING, NOT_FOND, FOND_OF_ME, BEFORE_PRAISE, THANKS_AFTER,
    PRAISED_ANSWER, SAME_COLD, A_STRETCH_THAT_DRAGS, VENTED, STUCK_DOES_SOMETHING,
    STUCK_A_DEGREE_OR_A_TIME, STUCK_AN_OBJECT, STUCK_LOSS, PLANS_ASK, WHAT_ARE_YOU_CAN_DO,
    BOT_FEELING_ASK, BOT_MOOD_ASK, ANSWERING_A_QUOTED_ASK,
);
sets!(
    THING_WORDS, THOUGHT_UNNAMED, NOT_WORDS, NOT_FILLERS, CORRECTION_MARKERS, BARE_PRAISE,
    NO_NOSE_WORDS, PLANS_FILLER, FEELING_NAMES, SELF_AFTER, AHEAD_TIME, BEHIND_TIME,
    BEEN_UP_TO_SPAN,
);
vals!(
    BODY_FEELING_WORDS, BOT_FEELING_REPLIES, BOT_FEELING_NAME_OF, BOT_FEELING_NAMED, BOT_LIFE,
    BOT_LIFE_AREA, BOT_DOINGS, BOT_WHEREABOUTS, BOT_DOINGS_AHEAD, BOT_NEVER, BOT_DOINGS_BARE,
    BOT_ABOUT, PLANS_REPLIES, SELF_ABOUT, OPINIONS, CONTEXT_CHAT,
);

inline_re!(A_DETERMINER_LAST, r"\b(?:a|an|the|this|that|my|your|our|their|his|her|its|any|no|another)\s+$");

/// Python: bot/modules/chat.py:2495 `_a_thing_in_front(clean, word)`
pub fn _a_thing_in_front(clean: &str, word: &str) -> bool {
    if !in_set(&THING_WORDS, word) {
        return false;
    }
    // `clean.find(word)` is -1 when the word is not there, and `clean[:-1]` is all but the last
    // character.
    let before = match clean.find(word) {
        Some(at) => &clean[..at],
        None => {
            let n = clean.char_indices().last().map(|(i, _)| i).unwrap_or(0);
            &clean[..n]
        }
    };
    A_DETERMINER_LAST.search(before).is_some()
}

/// Python: bot/modules/chat.py:2657 `_body_is_a_feeling(clean)`
pub fn _body_is_a_feeling(clean: &str) -> Option<String> {
    let text_ = clean;
    if BUG_NAMED.search(text_).is_some() {
        return None;
    }
    if SYMPTOM_IN_LINE.search(text_).is_some() {
        return None;
    }
    if LOSS.search(text_).is_some() || crate::compose::badnews::reads_as_someones_death(text_) {
        return None;
    }
    for pattern in [&*BODY_FEELING_SHAPE, &*BODY_PART_FEELING] {
        if let Some(found) = pattern.search(text_) {
            let said = text::strip(found.name("feeling").unwrap_or(""));
            let kind = BODY_FEELING_WORDS.get(said).map(|v| v.str()).unwrap_or("worry");
            return Some(_the_kind_of_worry(kind, text_));
        }
    }
    if let Some(found) = BODY_AT_THOUGHT.search(text_) {
        let pointed_at = text::strip(found.name("object").unwrap_or(""));
        if pointed_at.is_empty() || in_set(&THOUGHT_UNNAMED, pointed_at) {
            return Some(_the_kind_of_worry("worry", text_));
        }
    }
    None
}

/// Python: bot/modules/chat.py:2687 `_the_kind_of_worry(kind, text)`
pub fn _the_kind_of_worry(kind: &str, text_: &str) -> String {
    if kind != "worry" {
        return kind.to_string();
    }
    if WORRY_WHEN.search(text_).is_some() || ASKS_IF_NORMAL.search(text_).is_some() {
        return "anxious".to_string();
    }
    kind.to_string()
}

/// Python: bot/modules/chat.py:2701 `_line_as_read(text)`
pub fn _line_as_read(text_: &str) -> String {
    let clean = squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    crate::understanding::normalize::without_the_figure_of_speech(&clean)
}

/// Python: bot/modules/chat.py:2707 `_a_limit(text, memory)`
pub fn _a_limit(text_: &str, memory: &mut Memory) -> Option<String> {
    let read = memory.understood.clone()?;
    if !read.constraint {
        return None;
    }
    attempt(|| {
        if crate::knowledge::lookup::find(&read, None, None, false).is_some()
            && !truthy(crate::memory::limit::only_a_limit(text_, None))
        {
            return None;
        }
        let reply = py(crate::compose::limit::reply_for(&read, memory));
        if reply.truthy() {
            let steps = py(crate::compose::limit::steps_left(&read, memory));
            let mut limit = Value::dict();
            limit.set_key("said", reply.clone());
            limit.set_key("steps", steps);
            memory.set("limit", limit);
        }
        // `return reply`: a str or None (an empty str is returned as it is).
        reply.as_str().map(str::to_string)
    })
    .flatten()
}

inline_re!(VERDICT_JOIN_TAIL, r"(?:\b(?:and|but|so|though|honestly|tbh|well))\W*$");

/// Python: bot/modules/chat.py:2816 `_their_own_call(text)`
pub fn _their_own_call(text_: &str) -> Option<String> {
    let said = squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    if said.is_empty() || said.contains('?') {
        return None;
    }
    for m in _HOW_I_READ_IT.finditer(&said) {
        let fact = text::strip_chars(&said[..m.start()], " ,;:-");
        let fact = VERDICT_JOIN_TAIL.sub("", fact);
        let fact = text::strip_chars(&fact, " ,;:-");
        if text::split(fact).len() < 2 || _THEIR_OWN_LIFE.search(fact).is_none() {
            continue;
        }
        if _NOT_A_VERDICT_YET.search(fact).is_some() {
            continue;
        }
        let verdict = text::strip_chars(m.as_str(), " ,").to_string();
        if verdict.contains(" by me") {
            return Some(verdict);
        }
        if _A_LOOSE_VERDICT.search(&verdict).is_some() && _A_LIFE.search(fact).is_none() {
            continue;
        }
        return Some(verdict);
    }
    None
}

/// Python: bot/modules/chat.py:2850 `_without_the_feeling_avoided(clean, understood)`
pub fn _without_the_feeling_avoided(clean: &str, understood: Option<&Understood>) -> String {
    let Some(u) = understood else { return clean.to_string() };
    if u.kind != "question" {
        return clean.to_string();
    }
    if !u.asks.iter().any(|a| a == "how" || a == "steps") {
        return clean.to_string();
    }
    AVOIDED_FEELING.sub(" ", clean)
}

inline_re!(CLAUSE_STOP, r"[.!?,;:]");

/// Python: bot/modules/chat.py:2877 `_negated(clean, word)`
pub fn _negated(clean: &str, word: &str) -> bool {
    let mut found = false;
    for m in word_re(word).finditer(clean) {
        found = true;
        let parts = CLAUSE_STOP.split(&clean[..m.start()]);
        let before = text::split(parts.last().map(String::as_str).unwrap_or(""));
        let mut index = before.len() as isize - 1;
        let mut skipped: Vec<&str> = Vec::new();
        while index >= 0 && in_set(&NOT_FILLERS, &before[index as usize]) {
            skipped.push(&before[index as usize]);
            index -= 1;
        }
        if index < 0 {
            return false;
        }
        let at = &before[index as usize];
        if !(in_set(&NOT_WORDS, at) || at.ends_with("n't")) {
            return false;
        }
        if at == "never" && skipped.iter().any(|w| ["so", "this", "more", "such"].contains(w)) {
            return false;
        }
    }
    found
}

/// Python: bot/modules/chat.py:2910 `_denies_an_earlier_word(clean, word)`
pub fn _denies_an_earlier_word(clean: &str, word: &str) -> bool {
    for m in word_re(word).finditer(clean) {
        let mut held = false;
        let all = text::split(&clean[..m.start()]);
        for token in last_n(&all, 4) {
            if in_set(&NOT_WORDS, token) || token.ends_with("n't") {
                held = true;
            } else if held && in_set(&CORRECTION_MARKERS, token) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/chat.py:2992 `_fond_of_the_bot(clean)`
pub fn _fond_of_the_bot(clean: &str) -> bool {
    let line = squash(&clean.to_lowercase());
    let line = text::strip_chars(&line, " .,!?:;");
    if line.is_empty() || NOT_FOND.search(line).is_some() {
        return false;
    }
    FOND_OF_ME.match_(line).is_some()
}

/// Python: bot/modules/chat.py:3091 `_praised_the_answer(clean)`
pub fn _praised_the_answer(clean: &str) -> bool {
    let line = squash(&clean.to_lowercase());
    let line = text::strip_chars(&line, " .,!?:;");
    let mut line = BEFORE_PRAISE.sub("", line);
    if line.is_empty() {
        return false;
    }
    loop {
        let core = text::strip_chars(&THANKS_AFTER.sub("", &line), " .,!?:;").to_string();
        if core == line {
            break;
        }
        line = core;
    }
    if line.is_empty() {
        return false;
    }
    if in_set(&BARE_PRAISE, &line) {
        return true;
    }
    PRAISED_ANSWER.finditer(&line).iter().any(|found| found.end() == line.len())
}

/// Python: bot/modules/chat.py:3254 `_the_same_cold_again(clean, memory)`
pub fn _the_same_cold_again(clean: &str, memory: &Memory) -> bool {
    let turns = turns(memory);
    if turns.is_empty() {
        return false;
    }
    let words = az_words(&clean.to_lowercase());
    if !words.iter().any(|w| in_set(&NO_NOSE_WORDS, &w.to_lowercase())) {
        return false;
    }
    SAME_COLD.search(&turns[turns.len() - 1].bot).is_some()
}

/// Python: bot/modules/chat.py:3346 `_a_stretch_that_drags(clean)`
pub fn _a_stretch_that_drags(clean: &str) -> bool {
    let text_ = squash(&clean.to_lowercase());
    if text_.is_empty() || text_.contains('?') {
        return false;
    }
    if A_STRETCH_THAT_DRAGS.search(&text_).is_none() {
        return false;
    }
    if VENTED.search(&text_).is_some() {
        return false;
    }
    true
}

/// Python: bot/modules/chat.py:3550 `_doing_something_to_a_thing(clean)`
pub fn _doing_something_to_a_thing(clean: &str) -> bool {
    let Some(found) = STUCK_DOES_SOMETHING.search(clean) else { return false };
    let Some(degree) = STUCK_A_DEGREE_OR_A_TIME.search_at(clean, found.start()) else { return true };
    STUCK_AN_OBJECT.search_at(clean, degree.end()).is_some()
}

/// Python: bot/modules/chat.py:3592 `said_loss(memory)`
pub fn said_loss(memory: &Memory) -> bool {
    for turn in last_n(turns(memory), 8) {
        if STUCK_LOSS.search(&squash(&turn.user).to_lowercase()).is_some() {
            return true;
        }
    }
    false
}

inline_re!(PLANS_STOP, r"[.!?,;]\s*");

/// Python: bot/modules/chat.py:3697 `_asks_about_plans(said)`
pub fn _asks_about_plans(said: &str) -> bool {
    let line = squash(said);
    if line.is_empty() {
        return false;
    }
    if PLANS_ASK.match_(&line).is_some() {
        return true;
    }
    for stop in PLANS_STOP.finditer(&line) {
        let tail = &line[stop.end()..];
        if !tail.is_empty() && PLANS_ASK.match_(tail).is_some() {
            if text::split_ref(&line[..stop.start()]).iter().all(|w| in_set(&PLANS_FILLER, w)) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/chat.py:3827 `_a_feeling_named(word)`
pub fn _a_feeling_named(word: &str) -> bool {
    let word = text::strip(word).to_lowercase();
    in_set(&FEELING_NAMES, &word) || ["ness", "tion", "ment", "dom"].iter().any(|e| word.ends_with(e))
}

/// Python: bot/modules/chat.py:3861 `_a_feeling_about_me(word)`: a pool of lines.
pub fn _a_feeling_about_me(word: &str) -> Vec<String> {
    let word = text::strip(word).to_lowercase();
    if word.is_empty() {
        return BOT_FEELING_REPLIES.strs();
    }
    let pool = if _a_feeling_named(&word) { &*BOT_FEELING_NAME_OF } else { &*BOT_FEELING_NAMED };
    pool.strs().iter().map(|line| py_format(line, &[("word", &word)])).collect()
}

/// Python: bot/modules/chat.py:4185 `_about_bot_life(understood)`
pub fn _about_bot_life(understood: Option<&Understood>) -> Option<Vec<String>> {
    let u = understood?;
    if u.about_bot_kind.as_deref() != Some("life") {
        return None;
    }
    let words: &[String] = if !u.words.is_empty() { &u.words } else { &u.norm };
    for (area, pool) in BOT_LIFE.pairs() {
        let wanted = BOT_LIFE_AREA.get(area.str()).unwrap_or_else(|| panic!("KeyError: {:?}", area.str()));
        if words.iter().any(|w| wanted.contains_str(w)) {
            return Some(pool.strs());
        }
    }
    None
}

/// Python: bot/modules/chat.py:4196 `_as_meant(clean, understood)`
pub fn _as_meant(clean: &str, understood: Option<&Understood>) -> String {
    let Some(u) = understood else { return clean.to_string() };
    let theirs = if u.clean.is_empty() { clean } else { u.clean.as_str() };
    if clean != theirs {
        return clean.to_string();
    }
    let plain = &u.plain;
    if plain.is_empty() || plain == clean {
        return clean.to_string();
    }
    if u.signals.question_mark && !plain.contains('?') {
        return format!("{plain}?");
    }
    plain.clone()
}

/// Python: bot/modules/chat.py:4278 `_a_shape_no_list_knows(words)`
pub fn _a_shape_no_list_knows(words: &HashSet<String>) -> bool {
    if words.contains("keeps") || words.contains("keeping") {
        return true;
    }
    words.contains("been") && words.iter().any(|w| in_set(&BEEN_UP_TO_SPAN, w))
}

/// Python: bot/modules/chat.py:4285 `_doings_pool(words)`
pub fn _doings_pool(words: &HashSet<String>) -> Vec<String> {
    if words.iter().any(|w| in_set(&AHEAD_TIME, w)) && !words.iter().any(|w| in_set(&BEHIND_TIME, w)) {
        return BOT_DOINGS_AHEAD.strs();
    }
    const TIMES: [&str; 12] =
        ["weekend", "weekends", "yesterday", "night", "morning", "evening", "day", "week", "today", "tonight", "lately", "days"];
    if TIMES.iter().any(|t| words.contains(*t)) {
        return BOT_DOINGS.strs();
    }
    BOT_DOINGS_BARE.strs()
}

/// Python: bot/modules/chat.py:4300 `_a_sign_off_with_a_time(understood)`
pub fn _a_sign_off_with_a_time(understood: Option<&Understood>) -> bool {
    let text_ = match understood {
        Some(u) if !u.text.is_empty() => u.text.as_str(),
        Some(u) => u.clean.as_str(),
        None => "",
    };
    !text_.is_empty() && truthy(crate::compose::leaving::reads_as_leaving(text_, None))
}

/// Python: bot/modules/chat.py:4305 `_about_bot_doings(understood)`
pub fn _about_bot_doings(understood: Option<&Understood>) -> Option<Vec<String>> {
    let u = understood?;
    if u.about_bot_kind.as_deref() != Some("doings") {
        return None;
    }
    if _a_sign_off_with_a_time(understood) {
        return None;
    }
    let words: HashSet<String> = u.norm.iter().cloned().collect();
    if words.contains("ever") || words.contains("once") {
        return Some(BOT_NEVER.strs());
    }
    if words.contains("where") {
        return Some(BOT_WHEREABOUTS.strs());
    }
    Some(_doings_pool(&words))
}

/// Python: bot/modules/chat.py:4327 `_about_bot(clean, understood=None, memory=None)`: a pool.
pub fn _about_bot(clean: &str, understood: Option<&Understood>, memory: Option<&Memory>) -> Option<Vec<String>> {
    if let Some(life) = _about_bot_life(understood) {
        if !life.is_empty() {
            return Some(life);
        }
    }
    if let Some(doings) = _about_bot_doings(understood) {
        if !doings.is_empty() {
            return Some(doings);
        }
    }
    let norms: Vec<String> = understood.map(|u| u.norm.clone()).unwrap_or_default();
    if truthy(crate::understanding::kinds::the_manners_question(&norms)) {
        let intent = understood.map(|u| u.intent.as_str()).unwrap_or("");
        let set: HashSet<String> = norms.iter().cloned().collect();
        if intent == "smalltalk" || _a_shape_no_list_knows(&set) {
            return Some(_doings_pool(&set));
        }
    }
    let said = _as_meant(clean, understood);
    let named = memory
        .and_then(|m| m.get("user"))
        .and_then(|u| u.get("bot_name"))
        .map(|v| text::strip(&py_str(v)).to_string())
        .unwrap_or_default();
    if !named.is_empty() && data::re("bot.memory.facts.BOT_NAME_ASK").search(&said).is_some() {
        return Some(vec![
            format!("{named} \u{2014} the name you gave me. What can I help with?"),
            format!("You call me {named}. What can I do for you?"),
            format!("I'm {named}, since you named me."),
        ]);
    }
    let quoted = ANSWERING_A_QUOTED_ASK.search(&said).is_some();
    for (index, row) in BOT_ABOUT.items().iter().enumerate() {
        if quoted && index == 2 {
            continue;
        }
        let (pattern, replies) = (&row.items()[0], &row.items()[1]);
        if pattern.as_re().is_some_and(|p| p.search(&said).is_some()) {
            return Some(replies.strs());
        }
    }
    if !quoted && _asks_about_plans(&said) {
        return Some(PLANS_REPLIES.strs());
    }
    for row in SELF_ABOUT.items() {
        let (pattern, replies) = (&row.items()[0], &row.items()[1]);
        if let Some(m) = pattern.as_re().and_then(|p| p.search(&said)) {
            if _self_only(&said, &m) {
                return Some(replies.strs());
            }
        }
    }
    if WHAT_ARE_YOU_CAN_DO.match_(clean).is_some() {
        return None;
    }
    if let Some(m) = BOT_FEELING_ASK.search(clean) {
        if _self_only(clean, &m) {
            return Some(_a_feeling_about_me(m.name("mood").unwrap_or("")));
        }
    }
    if let Some(m) = BOT_MOOD_ASK.search(clean) {
        if _self_only(clean, &m) {
            return Some(_a_feeling_about_me("moods"));
        }
    }
    None
}

/// `str(value or "")` for a memory value.
fn py_str(v: &Value) -> String {
    match v {
        Value::None => String::new(),
        Value::Str(s) => s.to_string(),
        Value::Bool(false) => String::new(),
        Value::Bool(true) => "True".into(),
        Value::Int(0) => String::new(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) if *f == 0.0 => String::new(),
        Value::Float(f) => text::float_repr(*f),
        other if !other.truthy() => String::new(),
        other => panic!("str() of {other:?} is not ported"),
    }
}

/// Python: bot/modules/chat.py:4405 `_self_only(clean, match)`
pub fn _self_only(clean: &str, m: &Match) -> bool {
    let rest = text::strip_chars(&clean[m.end()..], " ?!.,;");
    text::split(&rest.replace(',', " ")).iter().all(|w| in_set(&SELF_AFTER, w))
}

inline_re!(
    OPINION_ASK,
    r"\b(do you like|do you prefer|what do you think of|what do you think about|which do you prefer|what(?:'s| is) your favourite|what's one|which one|favourite one)\b"
);

/// Python: bot/modules/chat.py:4416 `_opinion(clean, tokens, memory)`
pub fn _opinion(clean: &str, tokens: &[String], memory: &Memory) -> Option<String> {
    if OPINION_ASK.search(clean).is_none() || tokens.len() > 12 {
        return None;
    }
    let subject = clean;
    for row in OPINIONS.items() {
        let (keys, reply) = (&row.items()[0], &row.items()[1]);
        if keys.items().iter().any(|k| word_re(k.str()).search(subject).is_some()) {
            return Some(pick(&reply.strs(), memory, ""));
        }
    }
    let record = py(crate::memory::topics::current(memory));
    if record.truthy() && tokens.len() <= 9 {
        let words = record.get("words").cloned().unwrap_or(Value::None);
        for row in OPINIONS.items() {
            let (keys, reply) = (&row.items()[0], &row.items()[1]);
            if keys.items().iter().any(|k| words.contains_str(k.str())) {
                return Some(pick(&reply.strs(), memory, ""));
            }
        }
    }
    None
}

/// Python: bot/modules/chat.py:4436 `_context_follow_up(clean, memory)`
pub fn _context_follow_up(clean: &str, memory: &Memory) -> Option<String> {
    let mut near: HashSet<String> = HashSet::new();
    if let Some(Value::List(records)) = memory.get("topics") {
        for record in last_n(records, 3) {
            if record.is_dict() {
                if let Some(words) = record.get("words") {
                    near.extend(words.strs());
                }
            }
        }
    }
    if near.is_empty() {
        return None;
    }
    for row in CONTEXT_CHAT.items() {
        let (keys, cues, replies) = (&row.items()[0], &row.items()[1], &row.items()[2]);
        if !keys.items().iter().any(|k| near.contains(k.str())) {
            continue;
        }
        if cues.items().iter().any(|cue| dyn_re(cue.str(), 0).search(clean).is_some()) {
            return Some(pick(&replies.strs(), memory, ""));
        }
    }
    None
}

// ---------------------------------------------------------------------------------------------
// Python lines 4450-6450: plans, the hand-back, short talk, pools, goodbyes, nights.

res!(
    _ON_THE_JOB, _THE_RUN_OF_IT, A_RATE_OF_MONEY, A_JUDGEMENT_ON_A_FIGURE, OVERLOAD, LOAD_ASK,
    LOAD_TURNED, LOAD_SOMEONE_ELSE, LOAD_FEELS, JUDGED, _CONSIDERING, _CONSIDERING_TO,
    _BARE_DOING, _STATEMENT, HOW_TO_ASKED, SORRY_TALK, BUSY_RE, PLAN_RE, PLAN_WORRY,
    PLAN_TROUBLE, GO_AWAY, AHEAD, HOW_ARE_YOU, DESCRIBES_THE_DAY, FREQUENCY_ONLY, WORRY,
    NOT_MY_NIGHT, SYMPTOM, END_OF_ROPE, OTHER_AWAKE, OTHER_SLEPT, SPEAKER_LOST, LOW_MOOD,
    WORRY_EVENT, WORRY_PERSON, WORRY_ABOUT_THING, SKILL, _A_CHILD_OR_THEM, EVENT_RE,
    EVENT_OUTING, EVENT_TIRED, EVENT_STATE, NEED_FOR, INTENT_RE, NEED_RE, NEXT_UP, NEXT_REST,
    NEXT_ACT, WANTED, FEELING_WANT, LEAVE_SOMEONE, DOUBT, INTENT_CONDITION,
);
sets!(
    BUSY_WORK, PLAN_STOP, FEELING_THING, INTENT_STOP, QUESTION_STARTERS, VAGUE_WORDS,
    AGAIN_ONLY, NOT_AGAIN, LOOKING_FORWARD, HEAVY_FEELINGS, CHEER_KINDS, PLAN_CLAUSE_OPENS,
    PLAN_THINGS_OPEN, PLAN_VERB_OPENS, DENIAL_FILLER, POSITIVE, A_NOD_BEFORE, THE_MAKE,
    LEAVING_WORDS, NIGHT_SIGNOFF, _NOTHING_MUCH_FILLER, _NOTHING_MUCH_BETWEEN, _NOTHING_MUCH_DO,
    _NOTHING_MUCH_AFTER, _NOTHING_MUCH_TAIL, GREET_CORE, GREET_FILLER, THANKS_CORE,
    THANKS_FILLER, THANKS_LOOSE, BYE_FILLER, BYE_TIME, BYE_VERB,
);
vals!(
    NEEDS_AHEAD, HOW_ARE_YOU_CONTENT, MOOD_NIGHTS, SORE_NIGHTS, EARLY_NIGHTS, THE_STEPS_AGAIN,
    SOMEONE_TROUBLE, CHEER_IN_LOSS, FIRST_LINE_POOL, ASKED_HOW_ARE_YOU, A_FIRM_NO, LEAVES_ONLY,
    NIGHT_FREQUENCY, NIGHT_FREQUENCY_KEPT, BODY_FREQUENCY, BODY_FREQUENCY_KEPT, SMILES, REPLIES,
);

/// `REPLIES[kind]`: a pool of lines (KeyError when there is none).
fn replies(kind: &str) -> Vec<String> {
    REPLIES.get(kind).unwrap_or_else(|| panic!("KeyError: {kind:?}")).strs()
}

/// `s[:1].upper() + s[1:]`.
fn upper1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// `s[:1].lower() + s[1:]`.
fn lower1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn word_set(tokens: &[String]) -> HashSet<String> {
    tokens.iter().cloned().collect()
}

/// Python: bot/modules/chat.py:4775 `_a_trip(flat, memory)`
pub fn _a_trip(flat: &str, memory: &Memory) -> bool {
    if truthy(crate::memory::ack::reads_as_a_trip(flat)) {
        return true;
    }
    let places = memory.get("user").and_then(|u| u.get("places"));
    places.is_some_and(|p| p.truthy()) && GO_AWAY.search(flat).is_some()
}

/// `answers._offer(lambda reject: answers._in_context(text, memory, reject=reject), memory,
/// understood)`: (item is not None, reply). The reader is `answers._offer_with`'s closure form,
/// handed the same memory Python's lambda closes over.
fn answers_offer_in_context(text_: &str, memory: &mut Memory, understood: &Understood) -> (bool, Value) {
    let (item, reply) = crate::modules::answers::_offer_with(
        &mut |m: &mut Memory, reject| crate::modules::answers::_in_context_with(text_, m, reject),
        memory,
        understood,
    );
    (item.is_some(), py(reply))
}

/// Python: bot/modules/chat.py:4791 `_answers_it(text, understood, memory)`: the entry's reply
/// is worked out on this module's own memory, which an entry's reply function may write to.
pub fn _answers_it(text_: &str, understood: &Understood, memory: &mut Memory) -> bool {
    let read = attempt(|| {
        if let Some(found) = crate::knowledge::lookup::find(understood, None, None, false) {
            let reply = crate::modules::answers::_payload_mut(found, memory, understood);
            if !text::strip(&reply).is_empty() {
                return Some(true);
            }
        }
        let (item, reply) = answers_offer_in_context(text_, memory, understood);
        Some(item && reply.as_str().is_some_and(|r| !text::strip(r).is_empty()))
    });
    match read {
        None => true,
        Some(v) => v.unwrap_or(true),
    }
}

/// Python: bot/modules/chat.py:4818 `_intent(clean, tokens, memory)`
pub fn _intent(clean: &str, tokens: &[String], memory: &mut Memory) -> Option<String> {
    if tokens.is_empty() || tokens.len() > 22 || clean.contains('?') || tokens.len() < 3 {
        return None;
    }
    if crate::compose::fallback_chain::_the_morning_has_a_clock(&tokens.join(" ")) {
        return None;
    }
    let flat = tokens.join(" ");
    if _a_trip(&flat, memory) {
        return None;
    }
    if crate::compose::fallback::reads_as_a_change_of_medicine(&flat) {
        return None;
    }
    if truthy(crate::memory::facts::a_limit_on_the_plan(&flat)) {
        return None;
    }
    let words = word_set(tokens);
    if words.iter().any(|w| in_set(&INTENT_STOP, w)) || PLAN_WORRY.search(&flat).is_some() {
        return None;
    }
    if crate::compose::harm::reads_as_hurting_someone(&flat) {
        return None;
    }
    if let Some(quoted) = said(crate::compose::saidby::reply_for(&flat, memory)) {
        return Some(quoted);
    }
    let last = tokens[tokens.len() - 1].as_str();
    if last == "though" || last == "tho" {
        return None;
    }
    if let Some(wanted) = WANTED.search(&flat) {
        let thing = text::strip_chars(wanted.str(1), " .,!;:");
        let first = text::split_ref(thing).first().copied().unwrap_or("").to_string();
        if in_set(&FEELING_THING, &first) || FEELING_WANT.search(thing).is_some() || LEAVE_SOMEONE.search(thing).is_some() {
            return None;
        }
    }
    let head: Vec<&String> =
        tokens.iter().filter(|t| !["ok", "okay", "so", "and", "but", "also", "well"].contains(&t.as_str())).collect();
    if head.first().is_some_and(|h| in_set(&QUESTION_STARTERS, h)) {
        return None;
    }
    if DOUBT.search(&flat).is_some() {
        return None;
    }
    if INTENT_CONDITION.search(&flat).is_some() {
        return None;
    }
    if let Some(u) = memory.understood.clone() {
        let u = &u;
        if _answers_it(clean, u, memory) {
            return None;
        }
        if u.tight_on_money {
            return None;
        }
        if !crate::knowledge::cooking::negated_ingredients(u).is_empty() {
            return None;
        }
    }
    if truthy(crate::memory::ack::a_job_to_do(clean, memory)) {
        return None;
    }
    if EVENT_RE.search(&flat).is_some() {
        if EVENT_OUTING.search(&flat).is_some() || EVENT_TIRED.search(&flat).is_some() || EVENT_STATE.search(&flat).is_some() {
            return None;
        }
        return Some("event".into());
    }
    if NEED_FOR.search(&flat).is_some() {
        return None;
    }
    if INTENT_RE.search(&flat).is_some() || NEED_RE.search(&flat).is_some() {
        if NEXT_UP.search(&flat).is_some() && !_my_night(&flat) {
            if NEXT_REST.search(&flat).is_some() && NEXT_ACT.search(&flat).is_none() {
                return Some("next_rest".into());
            }
            if NEXT_ACT.search(&flat).is_some() {
                return Some("next_act".into());
            }
            return Some("next_up".into());
        }
        return Some("intent".into());
    }
    None
}

/// Python: bot/modules/chat.py:4918 `_as_typed(flat, memory)`
pub fn _as_typed(flat: &str, memory: &Memory) -> String {
    if let Some(u) = memory.understood.as_ref() {
        if !u.words.is_empty() && u.words.iter().all(|w| !w.is_empty()) {
            return u.words.join(" ");
        }
    }
    flat.to_string()
}

/// Python: bot/modules/chat.py:4940 `_a_detail_on_the_job(flat, memory)`
pub fn _a_detail_on_the_job(flat: &str, memory: &Memory) -> bool {
    match turns(memory).last() {
        Some(t) if t.module.as_deref() == Some("answers") => {}
        _ => return false,
    }
    let leans = memory.understood.as_ref().and_then(|u| u.leans_on.as_ref());
    match leans {
        Some(l) if !text::strip(&l.text).is_empty() => {}
        _ => return false,
    }
    if _ON_THE_JOB.search(flat).is_none() {
        return false;
    }
    _THE_RUN_OF_IT.search(flat).is_none()
}

/// Python: bot/modules/chat.py:4980 `_overloaded(flat, memory)`
pub fn _overloaded(flat: &str, memory: &Memory) -> bool {
    if OVERLOAD.search(flat).is_none() {
        return false;
    }
    if A_RATE_OF_MONEY.search(flat).is_some() && A_JUDGEMENT_ON_A_FIGURE.search(flat).is_some() {
        return false;
    }
    let said = _as_typed(flat, memory);
    if LOAD_ASK.search(&said).is_some() || LOAD_TURNED.search(&said).is_some() || LOAD_SOMEONE_ELSE.search(&said).is_some() {
        return false;
    }
    if _a_detail_on_the_job(flat, memory) {
        return false;
    }
    LOAD_FEELS.search(&said).is_none()
}

/// Python: bot/modules/chat.py:5027 `_judged(clean, memory)`
pub fn _judged(clean: &str, memory: &Memory) -> Option<String> {
    if JUDGED.match_(clean).is_none() {
        return None;
    }
    if attempt(|| truthy(crate::compose::fallback::a_verdict_on_their_plan(clean, memory, &Value::Bool(false)))).unwrap_or(false) {
        return None;
    }
    let thing = _judged_thing(clean, memory);
    let options: Vec<String> = if !thing.is_empty() {
        let big = upper1(&thing);
        NEWS_REPLIES
            .get("judged")
            .unwrap_or_else(|| panic!("KeyError: 'judged'"))
            .strs()
            .iter()
            .map(|o| py_format(o, &[("thing", &thing), ("Thing", &big)]))
            .collect()
    } else {
        NEWS_REPLIES.get("judged_bare").unwrap_or_else(|| panic!("KeyError: 'judged_bare'")).strs()
    };
    Some(pick(&options, memory, clean))
}

inline_re!(JUDGED_THING_CUT, r"[,;]|\b(?:because|but|so|though|although)\b");

/// Python: bot/modules/chat.py:5076 `_judged_thing(clean, memory)`
pub fn _judged_thing(clean: &str, memory: &Memory) -> String {
    let record = py(crate::memory::topics::current(memory));
    let said = if record.is_dict() { squash(&py_str(record.get("text").unwrap_or(&Value::None))) } else { String::new() };
    let said = text::strip_chars(&said, " .!?,").to_string();
    if said.is_empty() || said.to_lowercase() == clean.to_lowercase() {
        return String::new();
    }
    let to_form = _CONSIDERING_TO.match_(&said).is_some();
    let mut said = if to_form { _CONSIDERING_TO.sub("", &said) } else { _CONSIDERING.sub("", &said) };
    if to_form {
        said = crate::compose::stepwords::said_after_a_to(&said, &Value::None);
    }
    if let Some(doing) = _BARE_DOING.match_(&said) {
        let joined = format!("{}{}", doing.name_str("doing"), doing.name_str("rest"));
        said = text::strip_chars(&joined, " .,!?:;").to_string();
    }
    let cut = match JUDGED_THING_CUT.search(&said) {
        Some(m) => said[..m.start()].to_string(),
        None => said.clone(),
    };
    let said = text::strip_chars(&cut, " .,!?:;").to_string();
    let n = text::split(&said).len();
    if !(2..=6).contains(&n) || _STATEMENT.search(&said).is_some() {
        return String::new();
    }
    said
}

/// Python: bot/modules/chat.py:5134 `_the_steps_again(memory)`
pub fn _the_steps_again(memory: &Memory) -> Option<String> {
    let last = turns(memory).last()?;
    let asked = &last.user;
    let answer = text::strip(&last.bot);
    if HOW_TO_ASKED.search(asked).is_none() || text::split(answer).len() < 8 {
        return None;
    }
    Some(pick(&THE_STEPS_AGAIN.strs(), memory, "") + answer)
}

/// Python: bot/modules/chat.py:5155 `_a_care_turn(memory)`
pub fn _a_care_turn(memory: &Memory) -> bool {
    let last = py_str(&py(crate::memory::last_user(memory, 1)));
    if !last.is_empty() {
        let low = last.to_lowercase();
        if SOMEONE_TROUBLE.items().iter().any(|row| row.items()[1].as_re().is_some_and(|p| p.search(&low).is_some())) {
            return true;
        }
    }
    crate::compose::fallback::_in_a_care_thread(memory)
}

/// Python: bot/modules/chat.py:5169 `_short_talk(clean, tokens, memory)`
pub fn _short_talk(clean: &str, tokens: &[String], memory: &Memory) -> Option<String> {
    if tokens.is_empty() || tokens.len() > 12 || clean.contains('?') {
        return None;
    }
    let flat = tokens.join(" ");
    if _a_trip(&flat, memory) {
        return None;
    }
    let words = word_set(tokens);
    if words.iter().all(|w| in_set(&AGAIN_ONLY, w)) && words.contains("again") {
        return Some(if words.iter().any(|w| in_set(&NOT_AGAIN, w)) { "again" } else { "again_what" }.into());
    }
    if in_set(&LOOKING_FORWARD, &flat) {
        return if _a_care_turn(memory) { None } else { Some("keen".into()) };
    }
    if SORRY_TALK.search(&flat).is_some() {
        return Some("sorry_talk".into());
    }
    if BUSY_RE.search(&flat).is_some() && words.iter().any(|w| in_set(&BUSY_WORK, w)) {
        return Some("busy".into());
    }
    if _overloaded(&flat, memory) {
        return Some("overloaded".into());
    }
    if flat.contains("everything at once") || flat.contains("one thing after another") {
        return Some("busy".into());
    }
    if tokens.len() <= 8
        && words.iter().all(|w| in_set(&VAGUE_WORDS, w))
        && (words.contains("everything")
            || (words.contains("those") && words.contains("one"))
            || (words.contains("dont") && words.contains("know")))
    {
        if truthy(crate::memory::stuck::_the_answer_gave_a_step(memory)) {
            return if PLAN_RE.search(&flat).is_some() && !words.iter().any(|w| in_set(&PLAN_STOP, w)) {
                Some("plan".into())
            } else {
                None
            };
        }
        return Some("vague".into());
    }
    if crate::compose::fallback::reads_as_a_change_of_medicine(&_without_the_might(&flat)) {
        let slow = data::re("bot.compose.fallback.PLAN_MEDICINE_SLOW").search(&flat).is_some();
        return Some(if slow { "plan_medicine_slow" } else { "plan_medicine" }.into());
    }
    if PLAN_RE.search(&_without_the_might(&flat)).is_some()
        && !words.iter().any(|w| in_set(&PLAN_STOP, w))
        && PLAN_WORRY.search(&flat).is_none()
        && PLAN_TROUBLE.search(&flat).is_none()
        && !_a_forecast(memory)
    {
        return Some("plan".into());
    }
    None
}

/// Python: bot/modules/chat.py:5226 `_a_forecast(memory)`
pub fn _a_forecast(memory: &Memory) -> bool {
    let Some(u) = memory.understood.as_ref() else { return false };
    if u.topic.as_deref().unwrap_or("") != "nature-and-weather" {
        return false;
    }
    u.kind == "question"
}

/// Python: bot/modules/chat.py:5268 `_tokens(text)`
pub fn _tokens(text_: &str) -> Vec<String> {
    let mut t = text_.to_lowercase().replace('\u{2019}', "'");
    for (smile, word) in SMILES.pairs() {
        t = t.replace(smile.str(), word.str());
    }
    let t = t.replace('\'', "");
    TOKEN.findall_str(&t).into_iter().filter(|w| !w.is_empty()).collect()
}

/// Python: bot/modules/chat.py:5276 `_name(memory)`
pub fn _name(memory: &Memory) -> Option<String> {
    let facts = memory.get("user")?;
    if !facts.is_dict() {
        return None;
    }
    match facts.get("name") {
        Some(Value::Str(name)) if !name.is_empty() => Some(upper1(name)),
        _ => None,
    }
}

/// Python: bot/modules/chat.py:5285 `_the_lost_thread_asked(memory, kind)`
pub fn _the_lost_thread_asked(memory: &mut Memory, kind: &str) {
    if kind != "confused" {
        return;
    }
    let mut note = Value::dict();
    note.set_key("turn", memory.turns.len());
    memory.set("lost_thread", note);
}

inline_re!(LOWER_AZ_APOS, r"[a-z']+");

/// Python: bot/modules/chat.py:5297 `_after_a_lost_thread(clean, understood, memory)`
pub fn _after_a_lost_thread(clean: &str, understood: Option<&Understood>, memory: &mut Memory) -> Option<String> {
    let note = memory.get("lost_thread")?;
    if !note.is_dict() {
        return None;
    }
    if memory.turns.is_empty() {
        return None;
    }
    if note.get("turn").cloned().unwrap_or(Value::None) != Value::Int(memory.turns.len() as i64 - 1) {
        return None;
    }
    let u = understood?;
    if u.kind != "statement" || u.emotion.as_deref().is_some_and(|e| !e.is_empty()) || !u.asks.is_empty() {
        return None;
    }
    if _confusion_names_a_thing(u) {
        return None;
    }
    let words = LOWER_AZ_APOS.findall_str(&clean.to_lowercase());
    if words.len() < 3 || words.len() > 14 {
        return None;
    }
    Some(_reply("lost_thread_more", memory, None, &words.join(" ")))
}

/// Python: bot/modules/chat.py:5329 `_the_weight_said(memory, feeling)`
pub fn _the_weight_said(memory: &mut Memory, feeling: &str) {
    let index = memory.turns.len();
    let mut said_kind = Value::dict();
    said_kind.set_key("kind", feeling);
    said_kind.set_key("turn", index);
    memory.set("said_kind", said_kind);
    if !in_set(&HEAVY_FEELINGS, feeling) {
        return;
    }
    let mut heavy = Value::dict();
    heavy.set_key("feeling", feeling);
    heavy.set_key("turn", index);
    memory.set("heavy_reply", heavy);
}

/// Python: bot/modules/chat.py:5432 `_nothing_much_span(words)`
pub fn _nothing_much_span(words: &[String]) -> Option<(usize, usize)> {
    for (i, word) in words.iter().enumerate() {
        if ["notmuch", "nm", "nmu"].contains(&word.as_str()) {
            return Some((i, i + 1));
        }
    }
    for opener in ["nothing", "not"] {
        let Some(start) = words.iter().position(|w| w == opener) else { continue };
        let Some(rel) = words[start + 1..].iter().position(|w| w == "much") else { continue };
        let stop = start + 1 + rel;
        if words[start + 1..stop].iter().all(|w| in_set(&_NOTHING_MUCH_BETWEEN, w)) {
            return Some((start, stop + 1));
        }
        return None;
    }
    if let Some(at) = words.iter().position(|w| w == "much") {
        if words[..at].iter().any(|w| in_set(&_NOTHING_MUCH_DO, w)) {
            return Some((at, at + 1));
        }
    }
    None
}

inline_re!(HOW_MUCH_MANY, r"\bhow\s+(?:much|many)\b");
inline_re!(FIRST_CLAUSE_CUT, r"[,;:]|\s-\s");

/// Python: bot/modules/chat.py:5459 `_says_nothing_much(tokens, clean="")`
pub fn _says_nothing_much(tokens: &[String], clean: &str) -> bool {
    let said = word_set(tokens);
    let has = |w: &str| said.contains(w);
    if !(has("much") || has("notmuch") || has("nm") || has("nmu")) {
        return false;
    }
    if HOW_MUCH_MANY.search(clean).is_some() {
        return false;
    }
    if has("much")
        && !((has("not") || has("nothing") || has("notmuch") || has("nm") || has("nmu"))
            || said.iter().any(|w| in_set(&_NOTHING_MUCH_DO, w)))
    {
        return false;
    }
    if said.iter().all(|w| in_set(&_NOTHING_MUCH_FILLER, w)) {
        return true;
    }
    let clause = match FIRST_CLAUSE_CUT.search(clean) {
        Some(m) => &clean[..m.start()],
        None => clean,
    };
    let words = _tokens(clause);
    let Some((start, stop)) = _nothing_much_span(&words) else { return false };
    let before = &words[..start];
    let mut after = &words[stop..];
    for (at, word) in after.iter().enumerate() {
        if in_set(&_NOTHING_MUCH_TAIL, word) {
            after = &after[..at];
            break;
        }
    }
    if !after.iter().all(|w| in_set(&_NOTHING_MUCH_FILLER, w) || in_set(&_NOTHING_MUCH_AFTER, w) || in_set(&_NOTHING_MUCH_BETWEEN, w)) {
        return false;
    }
    if !before.iter().all(|w| in_set(&_NOTHING_MUCH_FILLER, w) || in_set(&_NOTHING_MUCH_DO, w)) {
        return false;
    }
    true
}

/// Python: bot/modules/chat.py:5507 `_the_line_on_the_screen(memory)`
pub fn _the_line_on_the_screen(memory: &Memory) -> String {
    for turn in turns(memory).iter().rev() {
        let said = text::strip(&turn.bot);
        if !said.is_empty() {
            return said.to_string();
        }
    }
    String::new()
}

/// Python: bot/modules/chat.py:5517 `_thanks_that_fit(options, memory)`
pub fn _thanks_that_fit(options: Vec<String>, memory: &Memory) -> Vec<String> {
    let said = _the_line_on_the_screen(memory);
    if !said.is_empty() && !text::rstrip(&said).ends_with('?') {
        return options;
    }
    let left: Vec<String> = options.iter().filter(|o| !o.contains("helped")).cloned().collect();
    if left.is_empty() {
        options
    } else {
        left
    }
}

/// Python: bot/modules/chat.py:5540 `_reply(kind, memory, name=None, seed="")`
pub fn _reply(kind: &str, memory: &mut Memory, name: Option<&str>, seed: &str) -> String {
    let mut kind = kind.to_string();
    if !_mid_chat(memory) {
        if let Some(mapped) = FIRST_LINE_POOL.get(&kind) {
            kind = mapped.str().to_string();
        }
    }
    if kind == "greeting" && _mid_chat(memory) && !_says_the_time_of_day(memory) {
        if let Some(name) = name.filter(|n| !n.is_empty()) {
            let options: Vec<String> = replies("check_in_name").iter().map(|o| py_format(o, &[("name", name)])).collect();
            return pick(&options, memory, seed);
        }
        return pick(&replies("check_in"), memory, seed);
    }
    if kind == "howareyou" && _mid_chat(memory) {
        if said_loss(memory) {
            return pick(&replies("howareyou_loss"), memory, seed);
        }
        return pick(&replies("howareyou_mid"), memory, seed);
    }
    let part = _the_part_of_day(memory);
    if kind == "greeting" {
        if let Some(part) = part {
            return pick(&replies(&format!("greeting_{part}")), memory, seed);
        }
    }
    _the_weight_said(memory, &kind);
    _the_lost_thread_asked(memory, &kind);
    let mut options = replies(&kind);
    if !_mid_chat(memory) {
        let fresh: Vec<String> = options.iter().filter(|o| !crate::compose::onscreen::talks_about_the_conversation(o)).cloned().collect();
        if !fresh.is_empty() {
            options = fresh;
        }
    }
    if kind == "thanks" {
        options = _thanks_that_fit(options, memory);
    }
    if name.is_some_and(|n| !n.is_empty()) && kind == "greeting" {
        options = replies("greeting_name");
    }
    if in_set(&CHEER_KINDS, &kind) && _a_death_just_said(memory) {
        options = CHEER_IN_LOSS.strs();
    } else if kind == "happy" || kind == "anxious" {
        let ahead = NEEDS_AHEAD.strs();
        let fits: Vec<String> = options.iter().filter(|o| ahead.contains(o)).cloned().collect();
        if AHEAD.search(seed).is_some() {
            if !fits.is_empty() {
                options = fits;
            }
        } else {
            let rest: Vec<String> = options.iter().filter(|o| !ahead.contains(o)).cloned().collect();
            if !rest.is_empty() {
                options = rest;
            }
        }
    }
    let name_text = name.unwrap_or("None");
    let formatted: Vec<String> = options.iter().map(|o| py_format(o, &[("name", name_text)])).collect();
    pick(&formatted, memory, seed)
}

/// The words of the reading's `plain`, lower-cased, for the time-of-day reads.
fn plain_words(memory: &Memory) -> Vec<String> {
    match memory.understood.as_ref() {
        Some(u) => text::split(&u.plain.to_lowercase()),
        None => Vec::new(),
    }
}

/// Python: bot/modules/chat.py:5598 `_the_part_of_day(memory)`
pub fn _the_part_of_day(memory: &Memory) -> Option<&'static str> {
    let words = plain_words(memory);
    ["morning", "afternoon", "evening"].into_iter().find(|part| words.iter().any(|w| w == part))
}

/// Python: bot/modules/chat.py:5608 `_says_the_time_of_day(memory)`
pub fn _says_the_time_of_day(memory: &Memory) -> bool {
    _the_part_of_day(memory).is_some()
}

/// Python: bot/modules/chat.py:5620 `_mid_chat(memory)`
pub fn _mid_chat(memory: &Memory) -> bool {
    truthy(crate::compose::onscreen::earlier_turn(memory))
}

/// Python: bot/modules/chat.py:5629 `_just_met(memory)`: (name, plan); (None, None) when there
/// is no facts log.
pub fn _just_met(memory: &Memory) -> (Option<String>, Option<String>) {
    let Some(Value::List(log)) = memory.get("facts_log") else { return (None, None) };
    let turn = Value::Int(memory.turns.len() as i64);
    let (mut name, mut plan) = (String::new(), String::new());
    for entry in last_n(log, 6) {
        if !entry.is_dict() || entry.get("turn").cloned().unwrap_or(Value::None) != turn {
            continue;
        }
        let key = entry.get("key").and_then(|k| k.as_str());
        if let Some(Value::Str(v)) = entry.get("value") {
            if key == Some("name") {
                name = v.to_string();
            }
            if key == Some("plan") {
                plan = v.to_string();
            }
        }
    }
    (Some(name), Some(plan))
}

inline_re!(ING_DOUBLE, r"[^aeiou]*[aeiou][^aeiouwxy]");

/// Python: bot/modules/chat.py:5669 `_ing(word)`
pub fn _ing(word: &str) -> String {
    let low = word.to_lowercase();
    if low.ends_with("ie") {
        return format!("{}ying", text::slice(&low, 0, Some(-2)));
    }
    if low.ends_with('e') && !low.ends_with("ee") && text::len(&low) > 3 {
        return format!("{}ing", text::slice(&low, 0, Some(-1)));
    }
    if text::len(&low) <= 4 && ING_DOUBLE.fullmatch(&low).is_some() {
        let last = low.chars().last().map(String::from).unwrap_or_default();
        return format!("{low}{last}ing");
    }
    format!("{low}ing")
}

inline_re!(PLAN_IM, r"\b(im|ive|ill)\b", crate::py::re::I);
inline_re!(PLAN_WERE, r"\bwere\b", crate::py::re::I);
inline_re!(PLAN_WEVE, r"\bweve\b", crate::py::re::I);

/// Python: bot/modules/chat.py:5681 `_plan_clause(plan)`
pub fn _plan_clause(plan: &str) -> String {
    let squashed = squash(plan);
    let text_ = text::strip_chars(&squashed, " .!?,");
    let first = text::split_ref(text_).first().map(|w| w.to_lowercase()).unwrap_or_default();
    if text_.is_empty() {
        return String::new();
    }
    let said = PLAN_IM.sub_fn(text_, |m| format!("i'{}", text::slice(m.as_str(), 1, None)));
    let said = PLAN_WERE.sub("we're", &said);
    let said = PLAN_WEVE.sub("we've", &said);
    let clause = crate::memory::you(&said);
    if clause.to_lowercase().starts_with("you") && in_set(&PLAN_CLAUSE_OPENS, &first) {
        return lower1(&clause);
    }
    if in_set(&PLAN_THINGS_OPEN, &first) {
        let clause = lower1(&clause);
        return text::rstrip(text::rstrip_chars(&clause, ".!?")).to_string();
    }
    let mut words = text::split(&clause);
    if first.ends_with("ing") && text::len(&first) > 4 {
        return lower1(&clause);
    }
    if in_set(&PLAN_VERB_OPENS, &first) && !words.is_empty() {
        words[0] = _ing(&words[0]);
        return words.join(" ");
    }
    if first == "for" {
        return format!("saving {}", lower1(&clause));
    }
    String::new()
}

/// Python: bot/modules/chat.py:5710 `_is_greeting(tokens)`
pub fn _is_greeting(tokens: &[String]) -> bool {
    !tokens.is_empty() && tokens.iter().all(|t| in_set(&GREET_FILLER, t)) && tokens.iter().any(|t| in_set(&GREET_CORE, t))
}

/// Python: bot/modules/chat.py:5785 `_a_hedge_answering_the_question(memory)`
pub fn _a_hedge_answering_the_question(memory: &Memory) -> bool {
    if memory.turns.is_empty() {
        return true;
    }
    let Some(note) = memory.get("said_kind").filter(|n| n.is_dict()) else { return false };
    if note.get("turn").cloned().unwrap_or(Value::None) != Value::Int(memory.turns.len() as i64 - 1) {
        return false;
    }
    let kind = note.get("kind").cloned().unwrap_or(Value::None);
    ASKED_HOW_ARE_YOU.items().iter().any(|k| *k == kind)
}

/// Python: bot/modules/chat.py:5827 `_denies_a_good_word(tokens)`
pub fn _denies_a_good_word(tokens: &[String]) -> bool {
    for (index, token) in tokens.iter().enumerate() {
        if !["not", "never", "barely", "hardly"].contains(&token.as_str()) {
            continue;
        }
        for word in &tokens[index + 1..] {
            if in_set(&DENIAL_FILLER, word) {
                continue;
            }
            if in_set(&POSITIVE, word) {
                return true;
            }
            break;
        }
    }
    false
}

/// Python: bot/modules/chat.py:5847 `_is_thanks(tokens)`
pub fn _is_thanks(tokens: &[String]) -> bool {
    !tokens.is_empty()
        && tokens.iter().any(|t| in_set(&THANKS_CORE, t))
        && tokens.iter().all(|t| in_set(&THANKS_FILLER, t) || in_set(&THANKS_LOOSE, t) || in_set(&THANKS_CORE, t))
}

/// Python: bot/modules/chat.py:5866 `_without_the_at_all(tokens)`
pub fn _without_the_at_all(tokens: &[String]) -> Vec<String> {
    let firm = A_FIRM_NO.strs();
    let mut kept: Vec<String> = Vec::new();
    for token in tokens {
        if *token == firm[1] && kept.last().is_some_and(|k| *k == firm[0]) {
            kept.pop();
            continue;
        }
        kept.push(token.clone());
    }
    kept
}

/// Python: bot/modules/chat.py:5877 `_without_the_nod(tokens)`
pub fn _without_the_nod(tokens: &[String]) -> Vec<String> {
    let (mut start, mut end) = (0usize, tokens.len());
    while start + 1 < end && in_set(&A_NOD_BEFORE, &tokens[start]) && !in_set(&THANKS_CORE, &tokens[start]) {
        start += 1;
    }
    while end > start + 1 && in_set(&A_NOD_BEFORE, &tokens[end - 1]) && !in_set(&THANKS_CORE, &tokens[end - 1]) {
        end -= 1;
    }
    tokens[start..end].to_vec()
}

inline_re!(WITH_THEIR_THING, r"with\s+(?:my|our|your|his|her|their|the)\s+\w");

/// Python: bot/modules/chat.py:5929 `_is_howareyou(tokens)`
pub fn _is_howareyou(tokens: &[String]) -> bool {
    if !(1..=7).contains(&tokens.len()) {
        return false;
    }
    let line = tokens.join(" ");
    let Some(m) = HOW_ARE_YOU.search(&line) else { return false };
    if _asks_about_the_make(&line) {
        return false;
    }
    let rest = text::strip(&line[m.end()..]);
    if m.as_str().ends_with("feeling") && rest.starts_with("about ") {
        return false;
    }
    if WITH_THEIR_THING.match_(rest).is_some() {
        return false;
    }
    let rest_words = text::split(rest);
    let two = last_n_first(&rest_words, 2).join(" ");
    !HOW_ARE_YOU_CONTENT.contains_str(&two)
}

/// `seq[:n]`.
fn last_n_first<T>(v: &[T], n: usize) -> &[T] {
    &v[..n.min(v.len())]
}

/// Python: bot/modules/chat.py:5965 `_asks_about_the_make(line)`
pub fn _asks_about_the_make(line: &str) -> bool {
    attempt(|| match py(crate::compose::fallback::about_the_bot(line)) {
        Value::Str(kind) => in_set(&THE_MAKE, &kind),
        _ => false,
    })
    .unwrap_or(false)
}

/// Python: bot/modules/chat.py:5973 `_is_bye(tokens)`
pub fn _is_bye(tokens: &[String]) -> bool {
    if tokens.is_empty() || !tokens.iter().all(|t| in_set(&BYE_FILLER, t)) {
        return false;
    }
    _leaves(tokens)
}

inline_re!(
    A_DAY_OFF,
    r"\b(?:day|days|time|week|weeks|weekend|afternoon|morning|evening|night|shift|today|tomorrow|tonight|monday|tuesday|wednesday|thursday|friday|saturday|sunday)\s+off\b"
);

/// Python: bot/modules/chat.py:6022 `_leaves(tokens)`
pub fn _leaves(tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let as_tuple = Value::Tuple(std::sync::Arc::new(tokens.iter().map(Value::from).collect()));
    if LEAVES_ONLY.items().iter().any(|t| *t == as_tuple) || tokens == ["good", "night"] {
        return true;
    }
    let any_of = |set: &[&str]| tokens.iter().any(|t| set.contains(&t.as_str()));
    if any_of(&["bye", "byebye", "goodbye", "goodnight", "cya", "cheerio", "toodles", "nightnight"]) {
        return true;
    }
    if tokens.iter().any(|t| in_set(&BYE_TIME, t)) && tokens.iter().any(|t| in_set(&BYE_VERB, t)) {
        return true;
    }
    if tokens[0] == "see" && any_of(&["you", "u", "ya"]) {
        return true;
    }
    let joined = tokens.join(" ");
    if A_DAY_OFF.search(&joined).is_some() {
        return false;
    }
    let going = any_of(&["gotta", "got", "gonna", "gunna", "have", "heading", "head", "off", "leaving", "going", "need", "must", "should"]);
    let away = any_of(&["go", "sleep", "bed", "now", "out", "soon", "off"]);
    if going && away && tokens.iter().all(|t| in_set(&LEAVING_WORDS, t)) {
        return true;
    }
    if tokens.iter().any(|t| in_set(&BYE_TIME, t))
        && tokens.iter().all(|t| in_set(&BYE_TIME, t) || ["back", "in", "a", "see", "you", "u", "ya"].contains(&t.as_str()))
    {
        return true;
    }
    if tokens.iter().any(|t| t == "night" || t == "later") {
        return DESCRIBES_THE_DAY.search(&joined).is_none();
    }
    false
}

/// Python: bot/modules/chat.py:6070 `_bye_kind(tokens)`
pub fn _bye_kind(tokens: &[String]) -> &'static str {
    if tokens.iter().any(|t| in_set(&NIGHT_SIGNOFF, t)) {
        "bye_night"
    } else {
        "bye"
    }
}

inline_re!(SENTENCE_STOPS, r"[.!?]+");

/// Python: bot/modules/chat.py:6075 `_ending_bye(clean, memory)`
pub fn _ending_bye(clean: &str, memory: &Memory) -> bool {
    if clean.ends_with('?') {
        return false;
    }
    let mut clauses: Vec<Vec<String>> = memory.understood.as_ref().map(|u| u.clauses.clone()).unwrap_or_default();
    if clauses.is_empty() {
        clauses = SENTENCE_STOPS.split(clean).iter().filter(|p| !text::strip(p).is_empty()).map(|p| _tokens(p)).collect();
    }
    clauses.len() > 1 && _is_bye(&clauses[clauses.len() - 1])
}

inline_re!(NIGHT_PHRASE, r"\b(?:night|nights|morning|mornings|nightly|all\s+(?:the\s+)?time|constantly)\b", crate::py::re::I);
inline_re!(ASKED_THE_LENGTH, r"\b(?:how long|how often|weeks?|months?)\b", crate::py::re::I);

/// Python: bot/modules/chat.py:6259 `_frequency_of_the_last_trouble(clean, memory=None)`: a pool.
pub fn _frequency_of_the_last_trouble(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let frequency = FREQUENCY_ONLY.match_(clean)?;
    let often = upper1(text::strip(frequency.str(1)));
    let last = memory.turns.last()?;
    let said = squash(&last.user.to_lowercase());
    if said.is_empty() || _loss_recent(memory) || WORRY.search(&said).is_some() {
        return None;
    }
    let night_phrase = NIGHT_PHRASE.search(&often).is_some();
    let night = night_phrase && _my_night(&said) && NOT_MY_NIGHT.search(&said).is_none();
    let body = !night
        && NOT_MY_NIGHT.search(&said).is_none()
        && (SYMPTOM.search(&said).is_some() || crate::compose::badnews::reads_like_a_symptom(&said));
    if !(night || body) {
        return None;
    }
    let kept = ASKED_THE_LENGTH.search(&last.bot).is_some();
    let pool = if night && kept {
        &*NIGHT_FREQUENCY_KEPT
    } else if night {
        &*NIGHT_FREQUENCY
    } else if kept {
        &*BODY_FREQUENCY_KEPT
    } else {
        &*BODY_FREQUENCY
    };
    Some(pool.strs().iter().map(|line| py_format(line, &[("often", &often)])).collect())
}

/// Python: bot/modules/chat.py:6308 `_answers_has_the_frequency(clean, memory)`
pub fn _answers_has_the_frequency(clean: &str, memory: &Memory) -> bool {
    let Some(said_) = attempt(|| {
        let mut guess = memory.clone();
        crate::modules::answers::respond(clean, &mut guess)
    }) else {
        return false;
    };
    let Some(said_) = said_.filter(|s| !s.is_empty()) else { return false };
    let flat = squash(&said_.to_lowercase());
    if flat.is_empty() || _A_CHILD_OR_THEM.search(&flat).is_some() {
        return false;
    }
    let before = memory.turns.last().map(|t| squash(&t.bot.to_lowercase())).unwrap_or_default();
    if flat.starts_with("same answer") || (!before.is_empty() && flat.contains(&text::slice(&before, 0, Some(60)))) {
        return false;
    }
    true
}

/// Python: bot/modules/chat.py:6338 `_frequency_is_the_turn(clean, memory=None)`
pub fn _frequency_is_the_turn(clean: &str, memory: &Memory) -> bool {
    if END_OF_ROPE.search(clean).is_some() || OTHER_AWAKE.search(clean).is_some() || LOSS.search(clean).is_some() {
        return false;
    }
    attempt(|| {
        if _answers_has_the_frequency(clean, memory) {
            return false;
        }
        _frequency_of_the_last_trouble(clean, memory).is_some_and(|p| !p.is_empty())
    })
    .unwrap_or(false)
}

/// Python: bot/modules/chat.py:6358 `after(text, reply, memory=None)` - a runtime hook.
pub fn after(text_: &str, reply: &str, memory: &mut Memory) {
    let noted = attempt(|| {
        let clean = squash(&text_.to_lowercase().replace('\u{2019}', "'"));
        let pool = _frequency_of_the_last_trouble(&clean, memory);
        let frequency_reply = pool.is_some_and(|p| !p.is_empty() && p.iter().any(|line| line == reply));
        let behind = memory.turns.last();
        let answer = match behind {
            Some(t) if frequency_reply && t.module.as_deref() == Some("answers") => t.bot.clone(),
            _ => String::new(),
        };
        (frequency_reply, answer)
    });
    let (frequency_reply, answer) = noted.unwrap_or((false, String::new()));
    memory.set("frequency_reply", frequency_reply);
    memory.set("frequency_answer", answer);
}

/// Python: bot/modules/chat.py:6387 `_early_start(clean, memory=None)`: a pool.
pub fn _early_start(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let ours = SPEAKER_LOST.search(clean).is_some();
    if END_OF_ROPE.search(clean).is_some() || OTHER_AWAKE.search(clean).is_some() {
        return None;
    }
    if OTHER_SLEPT.search(clean).is_some() && !ours {
        return None;
    }
    if LOSS.search(clean).is_some() || _loss_recent(memory) || WORRY.search(clean).is_some() {
        return None;
    }
    if let Some(frequency) = _frequency_of_the_last_trouble(clean, memory) {
        if !frequency.is_empty() {
            return Some(frequency);
        }
    }
    for row in EARLY_NIGHTS.items() {
        let (pattern, replies_) = (&row.items()[0], &row.items()[1]);
        if pattern.as_re().is_some_and(|p| p.search(clean).is_some()) {
            if SYMPTOM.search(clean).is_some() {
                return Some(SORE_NIGHTS.strs());
            }
            if LOW_MOOD.search(clean).is_some() || _low_mood_recent(memory) {
                return Some(MOOD_NIGHTS.strs());
            }
            return Some(replies_.strs());
        }
    }
    None
}

/// Python: bot/modules/chat.py:6412 `_worry_kind(clean)`
pub fn _worry_kind(clean: &str) -> &'static str {
    if WORRY_EVENT.search(clean).is_some() {
        return "anxious";
    }
    if WORRY_PERSON.search(clean).is_some() {
        return "worry_person";
    }
    if WORRY_ABOUT_THING.search(clean).is_some() {
        if WORRY_WHEN.search(clean).is_some() {
            return "anxious";
        }
        return "worry";
    }
    "anxious"
}

/// Python: bot/modules/chat.py:6440 `_without_skills(clean)`
pub fn _without_skills(clean: &str) -> String {
    SKILL.sub(" ", clean)
}

// ---------------------------------------------------------------------------------------------
// Python lines 6450-7470: whose feeling a word is, tears, not coping.

res!(
    A_THANK_YOU_AT_THE_END, RELIEF_SAID, A_LABEL_ON_THE_PERSON, OWN_BEEN, COPULA,
    LIFE_THING_HARD, _OTHER_SUBJECT, OTHER_PERSON, FIRST_PERSON, SOMEONE_ELSE, SOMEBODY_ELSE,
    NOT_THEIR_TEARS, NOT_WANTING_IT, MOVED_BY, MY_TEARS, A_NAMED_LOW, NOT_COPING,
    NOT_COPING_SHAPES, NOT_COPING_FEEL, NOT_COPING_ANYWHERE, GOING_MAD, STRUGGLING_ABOUT_A_THING,
    HOW_I_GET_THROUGH, FEAR_BEFORE, SCATTERED_MIND, WANT_TO_WEEP, WEEPING_WITH_JOY,
    WHY_THE_TEARS, CANT_STOP_IT, SOMEONE_IN_THEIR_OWN_TEARS, GUILT_ABOUT, ASKS_WHY,
    A_NIGHT_NOT_SLEPT, FIRST_PERSON_BEFORE, IDIOM_PLACE,
);
sets!(
    BARE_FILLER, CAUSES_THE_FEELING, CLAUSE_JOIN, CLAUSE_SUBJECT, BARE_ACKS, BARE_CARRY_ON,
    _WEEP_WORDS, PERSON_WORDS, CONTRACTIONS, HABIT_VERBS, PLAIN_GOOD_WORDS, PLAIN_GOOD_ASKED,
    PLAIN_GOOD_FRAME, PLAIN_GOOD_FILLERS, PLAIN_GOOD_TAIL, GOOD_POOLS, HARD_ON_THE_PERSON,
    WEARS_OTHER_PEOPLE_OUT, INTENSIFIERS, INTENSIFIER_PAIRS, DETERMINERS, STATE_NOUNS, OWN_STATES,
    FUNCTION_WORDS,
);
vals!(FEELINGS, GOOD_NEWS);

/// `word.strip(".,!?;:")`.
fn bare(word: &str) -> &str {
    text::strip_chars(word, ".,!?;:")
}

/// Python: bot/modules/chat.py:6548 `_the_bot_just_asked_them(memory)`
pub fn _the_bot_just_asked_them(memory: &Memory) -> bool {
    match memory.turns.last() {
        Some(t) => text::rstrip(&t.bot).ends_with('?'),
        None => false,
    }
}

/// Python: bot/modules/chat.py:6569 `_a_thank_you_taken_off(clean, memory=None)`
pub fn _a_thank_you_taken_off(clean: &str, memory: &Memory) -> String {
    let folded = A_THANK_YOU_AT_THE_END.sub("", clean);
    let folded = text::strip_chars(&folded, " ,.!?").to_string();
    if folded.is_empty() || folded == clean || text::split(&folded).len() > 4 {
        return String::new();
    }
    let padded = format!(" {folded} ");
    for row in FEELINGS.items() {
        let (pool, words) = (row.items()[0].str(), &row.items()[1]);
        for word in words.items() {
            if padded.contains(&format!(" {} ", word.str())) {
                if in_set(&GOOD_POOLS, pool) && !_the_bot_just_asked_them(memory) {
                    return String::new();
                }
                return folded;
            }
        }
    }
    if _a_plain_good_answer(&folded, memory) {
        return if _the_bot_just_asked_them(memory) { folded } else { String::new() };
    }
    String::new()
}

inline_re!(NOT_BAD, r"\bnot (?:too |half |so )?bad\b");

/// Python: bot/modules/chat.py:6597 `_a_plain_good_answer(clean, memory=None)`
pub fn _a_plain_good_answer(clean: &str, memory: &Memory) -> bool {
    if clean.contains('?') {
        return false;
    }
    if NOT_BAD.search(clean).is_some() {
        return true;
    }
    let words: Vec<&str> = text::split_ref(clean).into_iter().map(bare).filter(|w| !w.is_empty()).collect();
    if words.is_empty() || words.len() > 7 {
        return false;
    }
    let good = |w: &str| in_set(&PLAIN_GOOD_WORDS, w);
    let asked = |w: &str| in_set(&PLAIN_GOOD_ASKED, w);
    if !words.iter().any(|w| good(w) || asked(w)) {
        return false;
    }
    if !words.iter().all(|w| good(w) || asked(w) || in_set(&PLAIN_GOOD_FILLERS, w) || in_set(&PLAIN_GOOD_TAIL, w)) {
        return false;
    }
    let soft = words.iter().any(|w| asked(w));
    if soft && !words.iter().any(|w| in_set(&PLAIN_GOOD_FRAME, w)) && !_the_bot_just_asked_them(memory) {
        return words.iter().all(|w| good(w) || in_set(&PLAIN_GOOD_FILLERS, w));
    }
    true
}

/// Python: bot/modules/chat.py:6630 `_bare_feeling(clean)`
pub fn _bare_feeling(clean: &str) -> Option<String> {
    let rest: Vec<&str> = text::split_ref(clean).into_iter().map(bare).filter(|w| !w.is_empty() && !in_set(&BARE_FILLER, w)).collect();
    if rest.is_empty() || rest.len() > 2 || rest.iter().any(|w| in_set(&NOT_WORDS, w)) {
        return None;
    }
    for row in FEELINGS.items() {
        let (name, words) = (row.items()[0].str(), &row.items()[1]);
        if rest.iter().all(|w| words.contains_str(w)) {
            return Some(name.to_string());
        }
    }
    None
}

/// Python: bot/modules/chat.py:6679 `_says_relief(clean)`
pub fn _says_relief(clean: &str) -> bool {
    RELIEF_SAID.search(clean).is_some() && !truthy(crate::compose::negated(clean, "relieved", 3))
}

/// Python: bot/modules/chat.py:6706 `_starts_a_clause(after)`
pub fn _starts_a_clause(after: &[&str]) -> bool {
    after.iter().any(|w| in_set(&CLAUSE_SUBJECT, &bare(w).to_lowercase()))
}

/// Python: bot/modules/chat.py:6711 `_in_its_clause(before)`
pub fn _in_its_clause<'a>(before: &'a [&'a str]) -> &'a [&'a str] {
    for index in (0..before.len()).rev() {
        let word = before[index];
        if [',', ';', ':', '.', '!', '?'].iter().any(|c| word.ends_with(*c)) {
            return &before[index + 1..];
        }
        if in_set(&CLAUSE_JOIN, &bare(word).to_lowercase()) && _starts_a_clause(&before[index + 1..]) {
            return &before[index + 1..];
        }
    }
    before
}

/// Python: bot/modules/chat.py:6722 `_owned_by_a_thing(clean, match)`
pub fn _owned_by_a_thing(clean: &str, m: (usize, usize)) -> bool {
    if in_set(&CAUSES_THE_FEELING, &clean[m.0..m.1].to_lowercase()) {
        return A_LABEL_ON_THE_PERSON.search(clean).is_some();
    }
    let words = text::split_ref(&clean[..m.0]);
    let before = _in_its_clause(&words);
    let owner = _owner(before);
    if let Some(owner) = &owner {
        if in_set(&PERSON_WORDS, owner) {
            return false;
        }
        return !in_set(&OWN_STATES, owner);
    }
    let Some(last) = before.last() else { return false };
    let word = bare(last).to_lowercase();
    let bare_word = word.replace('\'', "").replace('\u{2019}', "");
    let pair = last_n(before, 2).iter().map(|p| bare(p)).collect::<Vec<_>>().join(" ").to_lowercase();
    if in_set(&FUNCTION_WORDS, &word)
        || in_set(&INTENSIFIERS, &word)
        || in_set(&CONTRACTIONS, &bare_word)
        || in_set(&HABIT_VERBS, &word)
        || COPULA.match_(&word).is_some()
        || (text::len(&word) > 4 && (word.ends_with("ing") || word.ends_with("ly")))
        || in_set(&INTENSIFIER_PAIRS, &pair)
    {
        return false;
    }
    !in_set(&OWN_STATES, &word)
}

/// Python: bot/modules/chat.py:6757 `_own_state(clean)`
pub fn _own_state(clean: &str) -> bool {
    if OWN_BEEN.search(clean).is_some() {
        return true;
    }
    let words: Vec<String> = text::split_ref(clean).into_iter().map(|w| bare(w).to_lowercase()).collect();
    for (index, word) in words.iter().enumerate() {
        if (word == "my" || word == "our")
            && words[(index + 1).min(words.len())..(index + 3).min(words.len())].iter().any(|n| in_set(&STATE_NOUNS, n))
        {
            return true;
        }
    }
    false
}

/// Python: bot/modules/chat.py:6768 `_said_of_the_person(clean, match)`
pub fn _said_of_the_person(clean: &str, m: (usize, usize)) -> bool {
    let before = text::split_ref(&clean[..m.0]);
    let after = text::split_ref(&clean[m.1..]);
    let owner = _owner(&before);
    let stated = owner.as_deref().is_some_and(|o| in_set(&STATE_NOUNS, o))
        || before.last().is_some_and(|w| {
            let w = bare(w).to_lowercase();
            w == "my" || w == "our"
        });
    if after.first().is_some_and(|w| in_set(&STATE_NOUNS, &bare(w).to_lowercase())) {
        return true;
    }
    if stated {
        return after.is_empty() || _carries_on(after[0]);
    }
    let intensified = before.last().is_some_and(|w| in_set(&INTENSIFIERS, w))
        || (!before.is_empty() && in_set(&INTENSIFIER_PAIRS, &last_n(&before, 2).join(" ")));
    if !intensified {
        return false;
    }
    if !after.is_empty() && !_carries_on(after[0]) {
        return false;
    }
    match owner {
        None => true,
        Some(o) => in_set(&OWN_STATES, &o),
    }
}

/// Python: bot/modules/chat.py:6796 `_carries_on(word)`
pub fn _carries_on(word: &str) -> bool {
    let word = bare(word).to_lowercase();
    if in_set(&FUNCTION_WORDS, &word) {
        return true;
    }
    text::len(&word) > 4 && word.ends_with("ing")
}

/// Python: bot/modules/chat.py:6804 `_owner(before)`
pub fn _owner(before: &[&str]) -> Option<String> {
    for index in (0..before.len()).rev() {
        if COPULA.match_(bare(before[index])).is_none() {
            continue;
        }
        let subject: Vec<&str> = before[..index]
            .iter()
            .map(|w| bare(w))
            .filter(|w| {
                let low = w.to_lowercase();
                !in_set(&DETERMINERS, &low) && !in_set(&FUNCTION_WORDS, &low) && !in_set(&INTENSIFIERS, &low)
            })
            .collect();
        if subject.is_empty() {
            return None;
        }
        for word in subject.iter().rev() {
            if in_set(&STATE_NOUNS, &word.to_lowercase()) {
                return Some(word.to_lowercase());
            }
        }
        return Some(subject[subject.len() - 1].to_lowercase());
    }
    None
}

/// Python: bot/modules/chat.py:6845 `_hard_about_their_thing(clean)`
pub fn _hard_about_their_thing(clean: &str) -> bool {
    for word in HARD_ON_THE_PERSON.iter() {
        if let Some(at) = clean.find(word.as_str()) {
            if at > 0 && LIFE_THING_HARD.search(&clean[..at]).is_some() {
                return true;
            }
        }
    }
    false
}

inline_re!(SPEAKER_WORD, r"\b(?:i|im|i'm|we)\b");

/// Python: bot/modules/chat.py:6861 `_a_person_who_wears_you_out(clean, word, match)`
pub fn _a_person_who_wears_you_out(clean: &str, word: &str, m: (usize, usize)) -> bool {
    if !in_set(&WEARS_OTHER_PEOPLE_OUT, word) {
        return false;
    }
    let before = &clean[..m.0];
    if OTHER_PERSON.search(before).is_some() {
        return true;
    }
    _OTHER_SUBJECT.search(before).is_some() && SPEAKER_WORD.search(before).is_none()
}

/// Python: bot/modules/chat.py:6871 `_says_how_the_person_is(clean, word, match)`
pub fn _says_how_the_person_is(clean: &str, word: &str, m: (usize, usize)) -> bool {
    if _a_person_who_wears_you_out(clean, word, m) {
        return false;
    }
    if _owned_by_a_thing(clean, m) {
        return _said_of_the_person(clean, m);
    }
    if in_set(&HARD_ON_THE_PERSON, word) && LIFE_THING_HARD.search(&clean[..m.0]).is_some() {
        return true;
    }
    truthy(crate::understanding::kinds::cue_about_person(clean, word)) || _said_of_the_person(clean, m)
}

/// Python: bot/modules/chat.py:7006 `_my_own_tears(clean)`
pub fn _my_own_tears(clean: &str) -> bool {
    let before = match MOVED_BY.search(clean) {
        Some(moved) => &clean[..moved.start()],
        None => clean,
    };
    let Some(mine) = FIRST_PERSON.search(before) else { return false };
    match SOMEONE_ELSE.search(before) {
        None => true,
        Some(someone) => mine.start() < someone.start(),
    }
}

/// Python: bot/modules/chat.py:7022 `_a_low_word(clean)`
pub fn _a_low_word(clean: &str) -> bool {
    for row in FEELINGS.items() {
        for word in row.items()[1].items() {
            let word = word.str();
            if in_set(&_WEEP_WORDS, word) {
                continue;
            }
            if word_re(word).search(clean).is_some() {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/chat.py:7033 `_good_news(clean)`
pub fn _good_news(clean: &str) -> bool {
    let flat = clean.replace('\'', "");
    if crate::compose::goodnews::asks_for_news(&flat) {
        return false;
    }
    GOOD_NEWS.items().iter().any(|p| p.as_re().is_some_and(|re| re.search(&flat).is_some()))
}

/// Python: bot/modules/chat.py:7071 `_weeping(clean)`
pub fn _weeping(clean: &str) -> bool {
    if WANT_TO_WEEP.search(clean).is_none() {
        return false;
    }
    if NOT_WANTING_IT.search(clean).is_some() || NOT_THEIR_TEARS.search(clean).is_some() || SOMEBODY_ELSE.search(clean).is_some() {
        return false;
    }
    if WEEPING_WITH_JOY.search(clean).is_some() || WHY_THE_TEARS.search(clean).is_some() {
        return false;
    }
    if A_NAMED_LOW.search(clean).is_some() {
        return false;
    }
    !_good_news(clean)
}

/// Python: bot/modules/chat.py:7084 `_tears_said_as_a_verb(clean)`
pub fn _tears_said_as_a_verb(clean: &str) -> Option<&'static str> {
    let m = MY_TEARS.search(clean)?;
    if SOMEONE_IN_THEIR_OWN_TEARS.search(m.as_str()).is_some() {
        return None;
    }
    if WANT_TO_WEEP.search(clean).is_some()
        || NOT_WANTING_IT.search(clean).is_some()
        || CANT_STOP_IT.search(clean).is_some()
        || NOT_THEIR_TEARS.search(clean).is_some()
        || SOMEBODY_ELSE.search(clean).is_some()
        || WEEPING_WITH_JOY.search(clean).is_some()
        || WHY_THE_TEARS.search(clean).is_some()
        || A_NAMED_LOW.search(clean).is_some()
    {
        return None;
    }
    if _good_news(clean) {
        return None;
    }
    Some(if _going_on(clean) { "sad" } else { "tears" })
}

/// Python: bot/modules/chat.py:7279 `_not_coping(clean)`
pub fn _not_coping(clean: &str) -> bool {
    let t = squash(clean);
    if GUILT_ABOUT.search(&t).is_some() {
        return false;
    }
    if !(NOT_COPING.search(&t).is_some()
        || NOT_COPING_SHAPES.search(&t).is_some()
        || NOT_COPING_ANYWHERE.search(&t).is_some()
        || _speaker_going_mad(&t)
        || HOW_I_GET_THROUGH.search(&t).is_some())
    {
        return false;
    }
    STRUGGLING_ABOUT_A_THING.search(&t).is_none()
}

inline_re!(AT_OR_ABOUT_REST, r"\b(?:at|about)\s+(.*)$");

/// Python: bot/modules/chat.py:7321 `_speaker_going_mad(text)`
pub fn _speaker_going_mad(t: &str) -> bool {
    for at in GOING_MAD.finditer(t) {
        let before = &t[..at.start()];
        let after = AT_OR_ABOUT_REST.search(&t[at.end()..]);
        if let Some(after) = &after {
            if IDIOM_PLACE.match_(after.str(1)).is_none() {
                continue;
            }
            if FEAR_BEFORE.search(before).is_none() {
                continue;
            }
        }
        if FIRST_PERSON_BEFORE.search(before).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/modules/chat.py:7340 `_bad_day(clean)`
pub fn _bad_day(clean: &str) -> bool {
    let t = squash(clean);
    if ASKS_WHY.search(&t).is_some() || GUILT_ABOUT.search(&t).is_some() || A_NIGHT_NOT_SLEPT.search(&t).is_some() {
        return false;
    }
    if NOT_COPING_FEEL.search(&t).is_none() {
        return false;
    }
    STRUGGLING_ABOUT_A_THING.search(&t).is_none()
}

/// Python: bot/modules/chat.py:7378 `_scattered(clean)`
pub fn _scattered(clean: &str) -> bool {
    let Some(m) = SCATTERED_MIND.search(clean) else { return false };
    let parts = CLAUSE_STOP.split(&clean[..m.start()]);
    let words = text::split(parts.last().map(String::as_str).unwrap_or(""));
    !last_n(&words, 3).iter().any(|w| in_set(&NOT_WORDS, &bare(w).to_lowercase()))
}

/// Python: bot/modules/chat.py:7448 `_a_bare_acknowledgement(clean)`
pub fn _a_bare_acknowledgement(clean: &str) -> bool {
    let mut words = az_words(&squash(&clean.to_lowercase()));
    while words.len() > 1 && ["ok", "okay", "right", "well", "so", "and", "but", "now"].contains(&words[0].as_str()) {
        words.remove(0);
    }
    let joined = words.join(" ");
    in_set(&BARE_ACKS, &joined) || in_set(&BARE_CARRY_ON, &joined)
}

// ---------------------------------------------------------------------------------------------
// Python lines 7470-8400: heavy lines, "i dont know", the feeling reader, nods.

res!(
    NOTHING_HELPING, MIGHT_BE, A_QUESTION_ABOUT_A_FIGURE, STATE_WHEN, LOW_THING, FLAT_IDIOM,
    ANNOUNCED_ONE, NEWS_AFTER, MOOD_FOR, MOVED_BY_PART, ROUGH_DAY, WHAT_A_DAY,
    ASKED_ABOUT_THE_BOT, SELF_INSULT, FEELING_SELF, FEELING_LEAD, PANIC_NOW, PANIC_SOMEONE_ELSE,
    NOT_MYSELF, SELF_CUE, LOST_THE_THREAD, A_FACULTY_GETTING_WORSE, OWN_MEMORY_OFF, THING_OF,
    MAD_ABOUT_THE_DAY, GUILT_DEED, SICK_DEED, REGRET_DEED, NOT_HAPPY_ABOUT, WORRY_FEELING,
    TIRED_PERSISTENT, TIRED_DESPITE_SLEEP, SLEEP_WORD, SPENT_MONEY, NOT_LANDED, LEFT_BEHIND,
    NOT_A_NOD, ASKED_HOW_IT_WENT, A_JUDGEMENT_ASKED, NOTHING_CONTINUED, APART_FROM_PEOPLE,
    APART_SOMEONE_ELSE,
);
sets!(SORRY_LINES, LONELY_STEP_WORDS, ABILITY_WORDS, A_SUBJECT_ALREADY_ASKED, HEAVY_MODULES);
vals!(EMOTION_FEELING, GUILT_WORDS, JUDGEMENT_POOLS);

/// `note.get("turn") == len(turns) - 1` for a note kept in memory.
fn note_is_last_turn(note: &Value, memory: &Memory) -> bool {
    note.get("turn").cloned().unwrap_or(Value::None) == Value::Int(memory.turns.len() as i64 - 1)
}

/// Python: bot/modules/chat.py:7483 `_the_heavy_line(memory)`
pub fn _the_heavy_line(memory: &Memory) -> String {
    let turns = turns(memory);
    if turns.is_empty() {
        return String::new();
    }
    if let Some(note) = memory.get("heavy_reply").filter(|n| n.is_dict()) {
        if note_is_last_turn(note, memory) {
            if let Some(Value::Str(line)) = note.get("line") {
                if !text::strip(line).is_empty() {
                    return line.to_string();
                }
            }
            return turns[turns.len() - 1].user.clone();
        }
    }
    let mut last: &Turn = &turns[turns.len() - 1];
    for turn in turns.iter().rev() {
        last = turn;
        if !_a_bare_acknowledgement(&squash(&turn.user.to_lowercase())) {
            break;
        }
    }
    let seed = last.user.clone();
    if last.module.as_deref().is_some_and(|m| in_set(&HEAVY_MODULES, m)) {
        return seed;
    }
    let said = &last.bot;
    if in_set(&SORRY_LINES, text::strip(said)) || SORRY_MET.search(said).is_some() {
        return seed;
    }
    if !text::strip(&seed).is_empty()
        && (crate::compose::loss::reads_as_loss(&seed, None, false) || crate::compose::badnews::reads_as_bad_news(&seed))
    {
        return seed;
    }
    String::new()
}

/// Python: bot/modules/chat.py:7536 `_apart_from_people(clean)`
pub fn _apart_from_people(clean: &str) -> bool {
    let t = squash(clean);
    if APART_FROM_PEOPLE.search(&t).is_none() {
        return false;
    }
    APART_SOMEONE_ELSE.search(&t).is_none()
}

/// Python: bot/modules/chat.py:7552 `_after_lonely(clean, memory)`
pub fn _after_lonely(clean: &str, memory: &mut Memory) -> Option<String> {
    let note = memory.get("heavy_reply")?.clone();
    if !note.is_dict() {
        return None;
    }
    let feeling = note.get("feeling").and_then(|f| f.as_str()).unwrap_or("");
    if !(feeling == "lonely" || feeling == "lonely_apart") || note.get("feeling").map_or(true, |f| !f.is_str()) {
        return None;
    }
    if memory.turns.is_empty() {
        return None;
    }
    if !note_is_last_turn(&note, memory) {
        return None;
    }
    let words = az_words(&squash(&clean.to_lowercase()));
    if words.is_empty() || words.len() > 7 {
        return None;
    }
    if words.iter().any(|w| !in_set(&LONELY_STEP_WORDS, w)) {
        return None;
    }
    if !words.iter().any(|w| w == "what" || w == "how") || !words.iter().any(|w| w == "do") {
        return None;
    }
    Some(_reply("lonely_step", memory, None, &words.join(" ")))
}

/// Python: bot/modules/chat.py:7586 `_idk_kind(memory)`
pub fn _idk_kind(memory: &Memory) -> &'static str {
    let turns = turns(memory);
    if turns.is_empty() {
        return "idk";
    }
    if let Some(note) = memory.get("heavy_reply").filter(|n| n.is_dict()) {
        let heavy = note.get("feeling").and_then(|f| f.as_str()).is_some_and(|f| in_set(&HEAVY_FEELINGS, f));
        if heavy && note_is_last_turn(note, memory) {
            return "idk_heavy";
        }
    }
    let said = &turns[turns.len() - 1].bot;
    if text::strip(said).is_empty() {
        return "idk";
    }
    if _offered_a_choice(said) {
        return "idk_choice";
    }
    if text::rstrip(said).ends_with('?') {
        return "idk_asked";
    }
    "idk"
}

inline_re!(AFTER_A_SENTENCE, r"(?<=[.!?])\s+");
inline_re!(A_SHORTLIST, r",\s*[^,.;:]{2,60},\s*or\s+[^,.;:]{2,60}\s*\.?\s*$", crate::py::re::I);

/// Python: bot/modules/chat.py:7619 `_offered_a_choice(said)`
pub fn _offered_a_choice(said: &str) -> bool {
    AFTER_A_SENTENCE.split(said).iter().any(|clause| A_SHORTLIST.search(clause).is_some())
}

/// Python: bot/modules/chat.py:7635 `_a_low_just_before(memory)`
pub fn _a_low_just_before(memory: &Memory) -> bool {
    let turns = turns(memory);
    if turns.is_empty() {
        return false;
    }
    if let Some(note) = memory.get("heavy_reply").filter(|n| n.is_dict()) {
        let heavy = note.get("feeling").and_then(|f| f.as_str()).is_some_and(|f| in_set(&HEAVY_FEELINGS, f));
        if heavy && note_is_last_turn(note, memory) {
            return true;
        }
    }
    let said = &turns[turns.len() - 1].user;
    if text::strip(said).is_empty() {
        return false;
    }
    let Some(read) = attempt(|| crate::understanding::understand(said, None)) else { return false };
    if read.intent != "feeling" || read.polarity > 0 {
        return false;
    }
    !matches!(read.emotion.as_deref(), Some("happy" | "relieved" | "excited" | "proud"))
}

/// Python: bot/modules/chat.py:7678 `_nothing_helped_just_before(memory, clean)`
pub fn _nothing_helped_just_before(memory: &Memory, clean: &str) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said = squash(&last.bot);
    if said.is_empty() || text::split(clean).len() > 12 {
        return false;
    }
    replies("nothing_helps").contains(&said) && NOTHING_CONTINUED.search(clean).is_some()
}

/// Python: bot/modules/chat.py:7695 `_nothing_helps(clean, memory)`
pub fn _nothing_helps(clean: &str, memory: &mut Memory) -> Option<String> {
    if NOTHING_HELPING.search(clean).is_none() {
        return None;
    }
    if !((_a_low_just_before(memory) && text::split(clean).len() <= 8) || _nothing_helped_just_before(memory, clean)) {
        return None;
    }
    Some(_reply("nothing_helps", memory, None, clean))
}

/// Python: bot/modules/chat.py:7705 `_as_said(clean, understood)`
pub fn _as_said(clean: &str, understood: Option<&Understood>) -> String {
    if let Some(u) = understood {
        if !u.plain.is_empty() {
            return squash(&u.plain.to_lowercase());
        }
    }
    clean.to_string()
}

/// Python: bot/modules/chat.py:7735 `_without_the_might(clean)`
pub fn _without_the_might(clean: &str) -> String {
    if MIGHT_BE.search(clean).is_none() {
        return clean.to_string();
    }
    MIGHT_BE.sub_with(clean, 1, |m| {
        let state = m.name_str("state").to_lowercase();
        if _feeling(&format!("im {state}"), None, None, false).is_none() {
            return m.as_str().to_string();
        }
        format!("im {state}")
    })
}

/// Python: bot/modules/chat.py:7775 `_feeling(clean, understood=None, bare=None, kept=False)`
pub fn _feeling(clean: &str, understood: Option<&Understood>, bare: Option<String>, kept: bool) -> Option<String> {
    let clean = _as_said(clean, understood);
    let mut clean = _without_the_feeling_avoided(&clean, understood);
    if crate::compose::harm::reads_as_hurting_someone(&clean) {
        return None;
    }
    if understood.is_some_and(|u| u.a_load_carried) {
        return Some("carrying".into());
    }
    if truthy(crate::memory::facts::a_limit_on_the_plan(&clean)) {
        return None;
    }
    if A_QUESTION_ABOUT_A_FIGURE.search(&_as_said(&clean, understood)).is_some() {
        return None;
    }
    clean = STATE_WHEN.sub(" ", &clean);
    if let Some(u) = understood {
        if u.state_owner.as_deref().is_some_and(|o| o != "self") && FEELING_SELF.search(&clean).is_none() {
            return None;
        }
    }
    let mut bare = match bare {
        Some(b) => Some(b),
        None => _bare_feeling(&clean),
    };
    if LOW_THING.search(&clean).is_some() {
        clean = LOW_THING.sub(" ", &clean);
    }
    if understood.is_some_and(|u| u.shortage.as_deref().is_some_and(|s| !s.is_empty()))
        || truthy(crate::understanding::kinds::shortage_of(&clean))
    {
        return None;
    }
    clean = _without_skills(&clean);
    if FLAT_IDIOM.search(&clean).is_some() {
        clean = FLAT_IDIOM.sub(" ", &clean);
    }
    if ANNOUNCED_ONE.search(&clean).is_some() {
        let rest = ANNOUNCED_ONE.sub(" ", &clean);
        if NEWS_AFTER.search(&rest).is_some() {
            clean = rest;
            bare = _bare_feeling(&clean);
        }
    }
    if MOOD_FOR.search(&clean).is_some() {
        clean = MOOD_FOR.sub(" ", &clean);
    }
    if (MOVED_BY.search(&clean).is_some() || (MY_TEARS.search(&clean).is_some() && MOVED_BY_PART.search(&clean).is_some()))
        && !_a_low_word(&clean)
    {
        if _my_own_tears(&clean) && NOT_WANTING_IT.search(&clean).is_none() && !_bad_day(&clean) && ROUGH_DAY.search(&clean).is_none() {
            return Some("moved".into());
        }
        return None;
    }
    if let Some(tears) = _tears_said_as_a_verb(&clean) {
        return Some(tears.into());
    }
    if _weeping(&clean) {
        return Some(if _going_on(&clean) { "sad" } else { "tears" }.into());
    }
    if NOT_WANTING_IT.search(&clean).is_some() && !_a_low_word(&clean) {
        return None;
    }
    let asked = ASKED_ABOUT_THE_BOT.search(&clean);
    let mine = FEELING_SELF.search(&clean);
    if let Some(asked) = &asked {
        if mine.as_ref().map_or(true, |m| asked.start() < m.start()) {
            return None;
        }
    }
    if SELF_INSULT.search(&clean).is_some() && FEELING_SELF.search(&clean).is_some() {
        return Some("sad".into());
    }
    if _not_coping(&clean) {
        return Some("not_coping".into());
    }
    if _bad_day(&clean) {
        return Some("bad_day".into());
    }
    if _apart_from_people(&clean) {
        return Some("lonely_apart".into());
    }
    if _a_stretch_that_drags(&clean) {
        return Some("long_stretch".into());
    }
    if ROUGH_DAY.search(&clean).is_some() || WHAT_A_DAY.search(&clean).is_some() {
        return Some("rough_day".into());
    }
    if _scattered(&clean) {
        return Some("scattered".into());
    }
    if let Some(b) = bare.as_ref().filter(|b| !b.is_empty()) {
        if PANIC_NOW.search(&clean).is_none() {
            return Some(b.clone());
        }
    }
    if NOT_MYSELF.search(&clean).is_some() {
        return Some("sad".into());
    }
    if PANIC_NOW.search(&clean).is_some()
        && PANIC_SOMEONE_ELSE.search(&clean).is_none()
        && !truthy(crate::compose::negated(&clean, "panic", 3))
    {
        return Some("panic".into());
    }
    let mut read: Option<String> = None;
    if let Some(u) = understood {
        if SELF_CUE.search(&clean).is_some() || LOST_THE_THREAD.search(&clean).is_some() || u.state_owner.as_deref() == Some("self") {
            read = u
                .emotion
                .as_deref()
                .and_then(|e| EMOTION_FEELING.get(e))
                .map(|v| py_str(v))
                .filter(|s| !s.is_empty());
            if read.is_some()
                && (A_FACULTY_GETTING_WORSE.search(&clean).is_some() || OWN_MEMORY_OFF.search(&clean).is_some())
                && _answers_knows(&clean, &mut Memory::new())
            {
                read = None;
            }
            if read.as_deref() == Some("confused") && _confusion_names_a_thing(u) {
                read = None;
            }
            if read.is_some() && u.state_owner.as_deref().is_some_and(|o| o != "self") {
                read = None;
            }
            if let Some(r) = read.clone() {
                if truthy(crate::compose::nearly(&clean, &r, 2)) {
                    read = None;
                }
            }
            if read.as_deref() == Some("anxious") && THING_OF.search(&clean).is_some() {
                read = Some(_worry_kind(&clean).into());
            }
            if read.as_deref() == Some("happy") && _says_relief(&clean) {
                read = Some("relieved".into());
            } else if read.as_deref() == Some("happy")
                && RELIEF_SAID.search(&clean).is_some()
                && truthy(crate::compose::negated(&clean, "relieved", 3))
            {
                read = Some("sad".into());
            }
            if read.as_deref() == Some("annoyed") && MAD_ABOUT_THE_DAY.search(&clean).is_some() {
                read = Some("busy".into());
            }
        }
    }
    if !(FEELING_SELF.search(&clean).is_some()
        || FEELING_LEAD.search(&clean).is_some()
        || read.is_some()
        || _own_state(&clean)
        || kept
        || _hard_about_their_thing(&clean)
        || REGRET_DEED.search(&clean).is_some())
    {
        return None;
    }
    if !GUILT_WORDS.items().iter().any(|w| _negated(&clean, w.str())) {
        if GUILT_DEED.search(&clean).is_some() || SICK_DEED.search(&clean).is_some() {
            return Some("guilty".into());
        }
    }
    if REGRET_DEED.search(&clean).is_some() && !_negated(&clean, "regret") {
        return Some("guilty".into());
    }
    let mut denied = false;
    let mut denied_names: HashSet<String> = HashSet::new();
    for row in FEELINGS.items() {
        let name = row.items()[0].str();
        for word in row.items()[1].items() {
            let word = word.str();
            let Some(m) = word_re(word).search(&clean).map(|m| (m.start(), m.end())) else { continue };
            if _a_thing_in_front(&clean, word) || !_says_how_the_person_is(&clean, word, m) {
                continue;
            }
            if text::split_ref(word).first().is_some_and(|w| in_set(&NOT_WORDS, w)) {
                return Some(name.into());
            }
            if _negated(&clean, word) {
                if name == "happy" {
                    return Some(if NOT_HAPPY_ABOUT.search(&clean).is_some() { "annoyed" } else { "sad" }.into());
                }
                denied = true;
                denied_names.insert(name.to_string());
                continue;
            }
            if in_set(&ABILITY_WORDS, word)
                && dyn_re(&format!(r"\b{}\s+(?:at|with)\b", crate::py::re::escape(word)), 0).search(&clean).is_some()
            {
                continue;
            }
            if name == "anxious" && WORRY_FEELING.search(&clean).is_some() {
                let mut kind = _worry_kind(&clean);
                if crate::compose::badnews::reads_like_a_symptom(&clean) {
                    continue;
                }
                if kind == "anxious" && _own_state(&clean) && WORRY_EVENT.search(&clean).is_none() {
                    kind = "worry";
                }
                return Some(kind.into());
            }
            if (name == "anxious" || name == "stressed") && WORRY_PERSON.search(&clean).is_some() {
                return Some("worry_person".into());
            }
            if name == "annoyed" && word == "mad" && MAD_ABOUT_THE_DAY.search(&clean).is_some() {
                continue;
            }
            if ["tired", "bored", "done"].contains(&word)
                && dyn_re(&format!(r"\b{}\s+(of|with)\b", crate::py::re::escape(word)), 0).search(&clean).is_some()
            {
                let whole = format!(
                    r"\b{}\s+(of|with)\s+(everything|anything|it all|life|the lot|the world|myself)\b",
                    crate::py::re::escape(word)
                );
                if dyn_re(&whole, 0).search(&clean).is_some() {
                    return Some("sad".into());
                }
                return Some(if word != "bored" { "annoyed" } else { "bored" }.into());
            }
            if name == "tired"
                && (TIRED_PERSISTENT.search(&clean).is_some()
                    || (TIRED_DESPITE_SLEEP.search(&clean).is_some() && SLEEP_WORD.search(&clean).is_some()))
            {
                return None;
            }
            if word == "spent" && SPENT_MONEY.search(&clean).is_some() {
                continue;
            }
            if name == "happy" && truthy(crate::compose::negated(&clean, word, 3)) {
                return Some("sad".into());
            }
            if truthy(crate::compose::negated(&clean, word, 3)) && !_denies_an_earlier_word(&clean, word) {
                denied = true;
                denied_names.insert(name.to_string());
                continue;
            }
            if truthy(crate::compose::nearly(&clean, word, 2)) {
                continue;
            }
            if name == "happy" && NOT_LANDED.search(&clean).is_some() {
                return Some("sad".into());
            }
            if name == "happy" && crate::memory::ack::has_news(&clean) && LEFT_BEHIND.search(&clean).is_none() {
                return None;
            }
            return Some(name.into());
        }
    }
    if denied && !read.as_ref().is_some_and(|r| !denied_names.contains(r)) {
        return Some("not_feeling".into());
    }
    read
}

/// Python: bot/modules/chat.py:8250 `_asked_how_it_went(line)`
pub fn _asked_how_it_went(line: &str) -> bool {
    ASKED_HOW_IT_WENT.match_(&squash(&line.to_lowercase())).is_some()
}

inline_re!(ANY_GOOD, r"\bany good\b");
inline_re!(TOO_MUCH_LITTLE, r"\btoo (?:much|little)\b");
inline_re!(FOR_YOU, r"\b(?:with|for|by|to) (?:you|me|us)\b|\bsuits? (?:you|me)\b|\bworks? for you\b");

/// Python: bot/modules/chat.py:8292 `_a_judgement_asked(line)`
pub fn _a_judgement_asked(line: &str) -> String {
    let flat = squash(&line.to_lowercase());
    let Some(found) = A_JUDGEMENT_ASKED.match_(text::strip_chars(&flat, " .,!?")) else { return String::new() };
    let mut word = found.name_str("word").to_lowercase();
    if word.ends_with(" enough") {
        word = "enough".into();
    }
    if in_set(&A_SUBJECT_ALREADY_ASKED, &word) {
        return String::new();
    }
    if word == "good" && ANY_GOOD.search(&flat).is_some() {
        return String::new();
    }
    if TOO_MUCH_LITTLE.search(&flat).is_some() {
        return "too_much_no_subject".into();
    }
    if word == "suit" || FOR_YOU.search(&flat).is_some() {
        return "for_you_no_subject".into();
    }
    for row in JUDGEMENT_POOLS.items() {
        if row.items()[1].contains_str(&word) {
            return row.items()[0].str().to_string();
        }
    }
    "judge_no_subject".into()
}

/// Python: bot/modules/chat.py:8322 `_is_a_nod(line)`
pub fn _is_a_nod(line: &str) -> bool {
    let said = line.to_lowercase().replace('\'', "").replace(',', " ");
    let words = text::split(&said);
    if NOT_A_NOD.match_(&said).is_some() {
        return false;
    }
    let acks = data::get("bot.compose.fallback.ACKS");
    words.len() <= 3 && words.iter().any(|w| acks.contains_str(w))
}

/// Python: bot/modules/chat.py:8331 `_the_nod(memory)`: (kind, seed).
pub fn _the_nod(memory: &Memory) -> (&'static str, String) {
    let recent = last_n(turns(memory), 3);
    for index in (0..recent.len()).rev() {
        let turn = &recent[index];
        if index + 1 < recent.len() && !_is_a_nod(&recent[index + 1].user) {
            break;
        }
        if turn.module.as_deref().is_some_and(|m| in_set(&NEWS_MODULES, m)) {
            return ("nod_after_news", turn.user.clone());
        }
        if NEWS_MET.search(&turn.bot).is_some() {
            return ("nod_after_news", turn.user.clone());
        }
        if crate::compose::badnews::reads_as_bad_news(&turn.user) {
            return ("nod_after_news", turn.user.clone());
        }
    }
    ("ack", String::new())
}

// ---------------------------------------------------------------------------------------------
// Python lines 8400-10660: work threads, "i feel like", worries about people, rows, stuck.

res!(
    A_DOUBT_ABOUT_WORK, A_TOO_OLD, WORK_IN_THE_THREAD, AN_AGE_SAID, GOING_ON, KEEPS_HOW_I_AM,
    FEEL_LIKE, FEEL_LIKE_WANT, FEEL_LIKE_NOT_HEARD, OTHER_FEELS_LIKE, FEEL_LIKE_NOT_ENOUGH,
    FEEL_LIKE_WORTH, FEEL_LIKE_ROLE, FEEL_LIKE_COPE, FEEL_LIKE_AT_HOME, FEEL_LIKE_LOAD,
    FEEL_LIKE_HOME_LOAD, FEEL_LIKE_GOOD, ARRIVAL_WORDS, FEEL_LIKE_MAIN_ANSWERS,
    FEEL_LIKE_OVERLOAD, FEEL_LIKE_ADRIFT, LIFE_ADRIFT, IDEATION, EVENT_DONE, FEEL_LIKE_EVERYTHING,
    FEEL_LIKE_PHYSICAL, FEEL_LIKE_ILLNESS, FEEL_LIKE_LEFT, FEEL_LIKE_JUDGEMENT, _STATE_OF_THEIRS,
    AGAINST_ME, AGAINST_ME_SELF, AGAINST_ME_ITS_SCOPE, JUST_A_CONTINUATION, WORRY_SOMEONE, CARE,
    NIGHT_SAID, MAYBE_SOMEONE, SOMEONE_TROUBLE_ANY, SERIOUS_DIAGNOSIS, THING_BEFORE_CANNOT,
    SOMEONE_IS_IT_NORMAL, SOMEONE_AGES_IT, SOMEONE_MORE, SOMEONE_NEXT_STEP, YOUNG, FEAR, AT_IT,
    THEY_ARE_NOT_TALKING, ASKS_FOR_THE_WORDS, A_ROW, A_ROW_CLOSE, ROW_ASK, _ASKS_AFTER, ROW_HOW,
    WANTS_AN_APOLOGY, ASKS_FOR_WORDS, ROW_ENDING, ROW_CONTACT, ROW_CONTACT_ASK, ROW_HER, ROW_HIM,
    FALLING_OUT, A_ROW_NOT_CLOSE, ROW_EVENT_ID, ROW_ABOUT, ROW_SPACE, ROW_SAID_SPACE,
    A_QUIET_AT_ME, A_A_CHILD, A_TEN_OR_OVER, A_WHAT_HAPPENED, A_BARE_MOVE_ASK, ROW_HUNG_UP,
    LIMIT, PLAN_ASK, WHO_SAID, CANT, CANT_OPENING, ITS_BEEN, STUCK_SAFETY, STUCK_IDIOM,
    STUCK_REASON, STUCK_SELF, STUCK_FIXABLE, STUCK_OBJECT, JOINED_PERSON, MY_NIGHT,
);
sets!(GOOD_TAIL_LEADS, AGAINST_ME_THE_WEEK, ROW_ABOUT_A_PERSON, HOW_MUCH_OF_IT, PLURAL_WORDS);
vals!(
    TAKEN_ON_REPLIES, TOO_OLD_REPLIES, AGE_IN_A_WORK_THREAD, WORRY_SOMEONE_NIGHT, WORRY_PRONOUN,
    WORRY_SOMEONE_DAY, SOMEONE_REPLY, SOMEONE_NORMAL_ANY, SOMEONE_NORMAL, SOMEONE_AGAIN,
    SOMEONE_NEXT, SOMEONE_ASK, SAID_WRONG_DEED, ROW_REPLY, ROW_FELL_OUT, ROW_NEXT, ROW_HOW_REPLY,
    ROW_SPACE_REPLY, ROW_ENDING_REPLY, ROW_ENDING_REPLY_HUNG_UP, STUCK,
);

/// Python's `int(value)` for a value kept in memory (an int, a bool, a float or a digit string).
fn py_int(v: &Value) -> i64 {
    match v {
        Value::Int(i) => *i,
        Value::Bool(b) => *b as i64,
        Value::Float(f) => f.trunc() as i64,
        Value::Str(s) => text::strip(s).parse().unwrap_or_else(|_| panic!("ValueError: invalid literal for int(): {s:?}")),
        other => panic!("TypeError: int() argument {other:?}"),
    }
}

/// `int(d.get(key) or 0)`.
fn int_or_zero(v: Option<&Value>) -> i64 {
    match v {
        Some(v) if v.truthy() => py_int(v),
        _ => 0,
    }
}

/// `text.split()` as owned words.
fn n_words(s: &str) -> usize {
    text::split_ref(s).len()
}

/// Python: bot/modules/chat.py:8445 `_a_doubt_about_being_taken_on(clean, memory)`
pub fn _a_doubt_about_being_taken_on(clean: &str, memory: &Memory) -> Option<String> {
    let found = A_DOUBT_ABOUT_WORK.search(clean).or_else(|| A_TOO_OLD.search(clean))?;
    if !_in_a_work_thread(memory) {
        return None;
    }
    let pool = if A_TOO_OLD.search(found.as_str()).is_some() { &*TOO_OLD_REPLIES } else { &*TAKEN_ON_REPLIES };
    Some(pick(&pool.strs(), memory, clean))
}

/// Python: bot/modules/chat.py:8463 `_in_a_work_thread(memory)`
pub fn _in_a_work_thread(memory: &Memory) -> bool {
    for turn in memory.turns.iter().rev().take(3) {
        let said = &turn.user;
        if said.is_empty() || WORK_IN_THE_THREAD.search(said).is_none() {
            continue;
        }
        if crate::compose::badnews::reads_as_bad_news(said)
            || crate::compose::badnews::reads_like_a_symptom(said)
            || AN_AGE_SAID.match_(text::strip(said)).is_some()
        {
            return true;
        }
    }
    false
}

/// Python: bot/modules/chat.py:8484 `_the_age_in_a_work_thread(clean, memory)`
pub fn _the_age_in_a_work_thread(clean: &str, memory: &Memory) -> Option<String> {
    let found = AN_AGE_SAID.match_(text::strip(clean))?;
    let age: i64 = found.str(1).parse().unwrap_or_else(|_| panic!("ValueError"));
    if !(18..=99).contains(&age) {
        return None;
    }
    if _in_a_work_thread(memory) {
        return Some(pick(&AGE_IN_A_WORK_THREAD.strs(), memory, clean).replace("{age}", &age.to_string()));
    }
    None
}

/// Python: bot/modules/chat.py:8505 `_going_on(clean)`
pub fn _going_on(clean: &str) -> bool {
    if GOING_ON.search(clean).is_none() && KEEPS_HOW_I_AM.search(clean).is_none() {
        return false;
    }
    let Some(other) = SOMEONE_ELSE.search(clean) else { return true };
    match FIRST_PERSON.search(clean) {
        Some(mine) => mine.start() < other.start(),
        None => false,
    }
}

inline_re!(THERE_WORD, r"\bthere\b");
inline_re!(FEELS_WORD, r"\bfeels?\b");

/// Python: bot/modules/chat.py:8773 `_a_journey_not_progress(clean, match)`
pub fn _a_journey_not_progress(clean: &str, m: &Match) -> bool {
    if THERE_WORD.search(m.as_str()).is_none() {
        return false;
    }
    // `clean[max(0, match.start() - 60):match.start()]`, in characters.
    let at = text::char_index(clean, m.start());
    let from = text::byte_at(clean, at.saturating_sub(60));
    let before = &clean[from..m.start()];
    if FEELS_WORD.search(before).is_some() {
        return false;
    }
    ARRIVAL_WORDS.search(before).is_some()
}

inline_re!(FEEL_WORDS, r"\b(?:feel|feels|feeling|felt)\b");
inline_re!(OWN_REPORT_END, r"\b(?:i'?m|i am|i'?ve|ive|we'?re|we are|were|been)$");

/// Python: bot/modules/chat.py:8808 `_said_about_themselves(before)`
pub fn _said_about_themselves(before: &str) -> bool {
    let mut words = az_words(&before.to_lowercase());
    while words.last().is_some_and(|w| in_set(&GOOD_TAIL_LEADS, w)) {
        words.pop();
    }
    if words.is_empty() {
        return false;
    }
    if FEEL_WORDS.search(&last_n(&words, 4).join(" ")).is_some() {
        return true;
    }
    if _STATE_OF_THEIRS.search(&last_n(&words, 3).join(" ")).is_some() {
        return true;
    }
    OWN_REPORT_END.search(&last_n(&words, 2).join(" ")).is_some()
}

inline_re!(SPEAKER_BEFORE, r"\b(?:i|im|i'm|i've|ive|me|my|myself|we|us)\b");

/// Python: bot/modules/chat.py:8831 `_feel_like(clean)`
pub fn _feel_like(clean: &str) -> Option<String> {
    if FEEL_LIKE_PHYSICAL.search(clean).is_some() || FEEL_LIKE_ILLNESS.search(clean).is_some() || IDEATION.search(clean).is_some() {
        return None;
    }
    if FEEL_LIKE_WANT.search(clean).is_some() || FEEL_LIKE_NOT_HEARD.search(clean).is_some() {
        return None;
    }
    if crate::compose::unseen::reads(clean) {
        return None;
    }
    if FEEL_LIKE_OVERLOAD.search(clean).is_some() {
        return None;
    }
    if LIFE_ADRIFT.search(clean).is_some() || FEEL_LIKE_MAIN_ANSWERS.search(clean).is_some() {
        return None;
    }
    if EVENT_DONE.search(clean).is_some() {
        return None;
    }
    if FEEL_LIKE_HOME_LOAD.search(clean).is_some() {
        return None;
    }
    if FEEL_LIKE_WORTH.search(clean).is_some() {
        return None;
    }
    if FEEL_LIKE_ROLE.search(clean).is_some() || FEEL_LIKE_COPE.search(clean).is_some() {
        return None;
    }
    if let Some(good) = FEEL_LIKE_GOOD.search(clean) {
        let before = &clean[..good.start()];
        if !_a_journey_not_progress(clean, &good)
            && (_said_about_themselves(before) || (text::strip(before).is_empty() && good.as_str().starts_with("getting")))
        {
            return Some("feel_like_good".into());
        }
    }
    let m = FEEL_LIKE.search(clean)?;
    if FEEL_LIKE_EVERYTHING.search(clean).is_some() {
        return Some("against_me".into());
    }
    if OTHER_FEELS_LIKE.search(clean).is_some() {
        return None;
    }
    if SPEAKER_BEFORE.search(&clean[..m.end()]).is_none() {
        return None;
    }
    if FEEL_LIKE_NOT_ENOUGH.search(clean).is_some() {
        if _letting_someone_down(clean).is_some() {
            return None;
        }
        return Some("feel_like_not_enough".into());
    }
    if FEEL_LIKE_LOAD.search(clean).is_some() {
        return Some("feel_like_load".into());
    }
    if FEEL_LIKE_LEFT.search(clean).is_some() {
        if FEEL_LIKE_AT_HOME.search(clean).is_some() {
            return None;
        }
        return Some("feel_like_left".into());
    }
    if FEEL_LIKE_ADRIFT.search(clean).is_some() {
        return Some("feel_like_adrift".into());
    }
    if crate::compose::fallback::_someone_else_minding(clean, &text::split(clean), false) {
        return None;
    }
    if FEEL_LIKE_JUDGEMENT.search(&clean[m.end()..]).is_none() {
        return None;
    }
    Some("feel_like".into())
}

/// Python: bot/modules/chat.py:8958 `_a_feeling_riding_on_the_thread(clean, memory)`
pub fn _a_feeling_riding_on_the_thread(clean: &str, memory: &Memory) -> bool {
    if JUST_A_CONTINUATION.search(clean).is_none() {
        return false;
    }
    if memory.turns.is_empty() {
        return false;
    }
    attempt(|| truthy(crate::memory::topics::carried_topic(memory, clean))).unwrap_or(false)
}

/// Python: bot/modules/chat.py:8971 `_against_me(clean)`
pub fn _against_me(clean: &str) -> Option<&'static str> {
    AGAINST_ME.search(clean)?;
    AGAINST_ME_SELF.search(clean)?;
    for found in AGAINST_ME_ITS_SCOPE.finditer(clean) {
        if !in_set(&AGAINST_ME_THE_WEEK, &found.str(1).to_lowercase()) {
            return None;
        }
    }
    Some("against_me")
}

/// What `_worry_someone` gives: the replies, or "pass" (a problem another part answers).
#[derive(Clone, Debug, PartialEq)]
pub enum WorrySomeone {
    Pool(Vec<String>),
    Pass,
}

/// Python: bot/modules/chat.py:9059 `_worry_someone(clean, memory=None)`
pub fn _worry_someone(clean: &str, memory: &Memory) -> Option<WorrySomeone> {
    let found = WORRY_SOMEONE.search(clean)?;
    if LOSS.search(clean).is_some() || _loss_recent(memory) || END_OF_ROPE.search(clean).is_some() {
        return None;
    }
    let who = found.name_str("who");
    if SYMPTOM.search(clean).is_some() || CARE.search(clean).is_some() || OTHER_AWAKE.search(clean).is_some() {
        return Some(WorrySomeone::Pass);
    }
    let pool = if NIGHT_SAID.search(clean).is_some() { &*WORRY_SOMEONE_NIGHT } else { &*WORRY_SOMEONE_DAY };
    let them = WORRY_PRONOUN.get(who).map(|v| v.str()).unwrap_or("them");
    Some(WorrySomeone::Pool(pool.strs().iter().map(|r| py_format(r, &[("who", who), ("them", them)])).collect()))
}

/// Python: bot/modules/chat.py:9083 `_loss_recent(memory)`
pub fn _loss_recent(memory: &Memory) -> bool {
    last_n(turns(memory), 3).iter().any(|t| LOSS.search(&t.user.to_lowercase()).is_some())
}

/// Python: bot/modules/chat.py:9094 `_low_mood_recent(memory)`
pub fn _low_mood_recent(memory: &Memory) -> bool {
    last_n(turns(memory), 3).iter().any(|t| LOW_MOOD.search(&t.user.to_lowercase()).is_some())
}

/// Python: bot/modules/chat.py:9154 `_a_thing_cannot(clean)`
pub fn _a_thing_cannot(clean: &str) -> bool {
    SOMEONE_TROUBLE_ANY.search(&THING_BEFORE_CANNOT.sub(" ", clean)).is_none()
}

/// Python: bot/modules/chat.py:9392 `_checks_the_fact(text, memory)`
pub fn _checks_the_fact(text_: &str, memory: &Memory) -> bool {
    attempt(|| !py(crate::memory::thread::checking_a_fact(text_, memory)).is_none()).unwrap_or(false)
}

/// Python: bot/modules/chat.py:9399 `_a_follow_up_about_their_person(clean, memory)`
pub fn _a_follow_up_about_their_person(clean: &str, memory: &Memory) -> bool {
    if n_words(clean) > 9 {
        return false;
    }
    let record = py(crate::memory::topics::about(memory));
    if !record.truthy() {
        return false;
    }
    let said = squash(&py_str(record.get("said").unwrap_or(&Value::None)).to_lowercase());
    if said.is_empty() || said == clean {
        return false;
    }
    attempt(|| _about_someone(clean, memory).is_some()).unwrap_or(false)
}

/// Python: bot/modules/chat.py:9427 `TAKES_AS_IT_COMES(text, memory=None)`: the hook the arbiter
/// asks (it writes memory as Python's does: `limits.reply` and `_a_limit` note what they read).
pub fn TAKES_AS_IT_COMES(text_: &str, memory: &mut Memory) -> bool {
    if _checks_the_fact(text_, memory) {
        return true;
    }
    if crate::compose::occasionplan::takes_as_it_comes(text_, memory) {
        return true;
    }
    if attempt(|| crate::memory::repeat::asks_what_that_costs(text_, memory)).unwrap_or(false) {
        return true;
    }
    let clean = squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    if _a_follow_up_about_their_person(&clean, memory) {
        return true;
    }
    if _frequency_is_the_turn(&clean, memory) {
        return true;
    }
    if LOSS.search(&clean).is_some() || truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return false;
    }
    if AT_IT.match_(&clean).is_some() {
        return true;
    }
    if attempt(|| !py(crate::memory::limits::reply(text_, memory)).is_none()).unwrap_or(false) {
        return true;
    }
    if attempt(|| truthy(crate::compose::reading::judges_a_figure(text_, memory))).unwrap_or(false) {
        return true;
    }
    let line = _line_as_read(text_);
    let about_the_bot = attempt(|| {
        let named = memory
            .get("user")
            .and_then(|u| u.get("bot_name"))
            .map(|v| text::strip(&py_str(v)).to_string())
            .unwrap_or_default();
        !(!named.is_empty() && data::re("bot.memory.facts.BOT_NAME_ASK").search(&line).is_some())
            && _about_bot(&line, memory.understood.as_ref(), Some(memory)).is_some_and(|p| !p.is_empty())
    });
    if about_the_bot == Some(true) {
        return true;
    }
    if _body_is_a_feeling(&line).is_some_and(|f| !f.is_empty()) {
        return true;
    }
    if _a_limit(text_, memory).is_some_and(|r| !r.is_empty()) {
        return true;
    }
    if _their_own_call(&line).is_some_and(|v| !v.is_empty()) {
        return true;
    }
    if truthy(crate::compose::repair::amends_to(&clean)) {
        return true;
    }
    if _a_repair_is_open(memory) && THEY_ARE_NOT_TALKING.search(&clean).is_some() {
        return true;
    }
    if ASKS_FOR_THE_WORDS.search(&clean).is_some() && _after_a_repair(memory) && _the_deed_before(memory) != **SAID_WRONG_DEED {
        return true;
    }
    if _said_wrong(&clean) || _said_unkindly(&clean) {
        return !_an_entry_answers_the_deed(&clean, memory);
    }
    if let Some(said) = A_ROW.search(&clean) {
        if A_ROW_CLOSE.search(&clean).is_some()
            && ROW_ASK.match_(&clean).is_none()
            && !text::rstrip(&clean).ends_with('?')
            && _ASKS_AFTER.search(&clean[said.end()..]).is_none()
            && n_words(&clean) <= 18
        {
            return true;
        }
    }
    let Some(row) = memory.get("row").filter(|r| r.is_dict()).cloned() else { return false };
    if !row.get("them").is_some_and(|t| t.truthy()) || n_words(&clean) > 14 {
        return false;
    }
    if _the_base_answers_the_row(&clean, memory) {
        return false;
    }
    let how = ROW_HOW.search(&clean).is_some();
    if how && row.get("handed_over").is_some_and(|h| h.truthy()) {
        return false;
    }
    if how
        && (WANTS_AN_APOLOGY.search(&clean).is_some() || ASKS_FOR_WORDS.search(&clean).is_some())
        && _the_base_has_an_answer(&clean, &memory.clone()).is_some()
    {
        return false;
    }
    if !(ROW_ENDING.search(&clean).is_some()
        || how
        || (ROW_CONTACT.search(&clean).is_some() && ROW_CONTACT_ASK.search(&clean).is_some() && _names_the_person(&clean, Some(&row))))
    {
        return false;
    }
    let waited = memory.turns.len() as i64 - int_or_zero(row.get("turn"));
    0 < waited && waited <= 3
}

/// Python: bot/modules/chat.py:9649 `_close_nouns(text)`
pub fn _close_nouns(text_: &str) -> HashSet<String> {
    A_ROW_CLOSE.findall_str(&text_.to_lowercase()).iter().map(|w| text::slice(w, 0, Some(4))).collect()
}

inline_re!(A_PRONOUN, r"\b(?:her|him|them|she|he|they)\b");

/// Python: bot/modules/chat.py:9654 `_names_the_person(clean, row=None)`
pub fn _names_the_person(clean: &str, row: Option<&Value>) -> bool {
    if A_PRONOUN.search(clean).is_some() {
        return true;
    }
    let named = _close_nouns(clean);
    if named.is_empty() {
        return false;
    }
    let row_text = match row {
        Some(r) if r.is_dict() => py_str(r.get("text").unwrap_or(&Value::None)),
        _ => String::new(),
    };
    let with_them = _close_nouns(&row_text);
    if with_them.is_empty() {
        return true;
    }
    named.is_subset(&with_them)
}

/// Python: bot/modules/chat.py:9690 `_a_row_person(clean)`
pub fn _a_row_person(clean: &str) -> bool {
    if A_ROW_CLOSE.search(clean).is_some() {
        return true;
    }
    A_ROW_NOT_CLOSE.search(clean).is_none()
}

/// Python: bot/modules/chat.py:9701 `_keep_the_row(clean, memory)`
pub fn _keep_the_row(clean: &str, memory: &mut Memory) -> Option<Value> {
    if A_ROW.search(clean).is_none() || !_a_row_person(clean) {
        return None;
    }
    if ROW_ASK.match_(clean).is_some() || text::rstrip(clean).ends_with('?') {
        return None;
    }
    if n_words(clean) > 18 || LOSS.search(clean).is_some() || truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    let mut row = Value::dict();
    row.set_key("them", _row_pronoun(clean));
    row.set_key("they", _row_subject(clean));
    row.set_key("turn", memory.turns.len());
    row.set_key("text", clean);
    row.set_key("handed_over", FALLING_OUT.search(clean).is_some());
    memory.set("row", row.clone());
    Some(row)
}

/// Python: bot/modules/chat.py:9726 `observe(text, memory)` - a runtime hook.
pub fn observe(text_: &str, memory: &mut Memory) {
    let _ = attempt(|| _keep_the_row(text_, memory));
}

/// Python: bot/modules/chat.py:9750 `_the_base_has_an_answer(clean, memory)`
pub fn _the_base_has_an_answer(clean: &str, memory: &Memory) -> Option<&'static crate::knowledge::entry::Entry> {
    attempt(|| crate::knowledge::lookup::find(&crate::understanding::understand(clean, Some(memory)), None, None, false)).flatten()
}

/// Python: bot/modules/chat.py:9776 `_the_row_names_its_subject(clean)`
pub fn _the_row_names_its_subject(clean: &str) -> bool {
    ROW_ABOUT.finditer(clean).iter().any(|hit| !in_set(&ROW_ABOUT_A_PERSON, hit.str(1)))
}

/// Python: bot/modules/chat.py:9784 `_the_base_answers_the_row(clean, memory)`
pub fn _the_base_answers_the_row(clean: &str, memory: &Memory) -> bool {
    let Some(found) = _the_base_has_an_answer(clean, memory) else { return false };
    let ident = found.id.as_str();
    if ROW_EVENT_ID.search(ident).is_some() || ident.contains(ROW_FELL_OUT.str()) {
        return true;
    }
    _the_row_names_its_subject(clean)
}

/// The row's lines filled with the person it was with (`line.format(them=, they=, their=)`).
fn row_lines(pool: &[String], them: &str, they: &str) -> Vec<String> {
    let their = _their_of(them);
    pool.iter().map(|line| py_format(line, &[("them", them), ("they", they), ("their", &their)])).collect()
}

/// Python: bot/modules/chat.py:9803 `_a_row(clean, memory=None)`
pub fn _a_row(clean: &str, memory: &mut Memory) -> Option<String> {
    let said = A_ROW.search(clean)?;
    if !_a_row_person(clean) {
        return None;
    }
    if ROW_ASK.match_(clean).is_some() || text::rstrip(clean).ends_with('?') {
        return None;
    }
    if _ASKS_AFTER.search(&clean[said.end()..]).is_some() {
        return None;
    }
    if n_words(clean) > 18 || LOSS.search(clean).is_some() || truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    let them = _row_pronoun(clean);
    _keep_the_row(clean, memory);
    if _the_base_answers_the_row(clean, memory) {
        return None;
    }
    let they = _row_subject(clean);
    Some(pick(&row_lines(&ROW_REPLY.strs(), them, they), memory, clean))
}

/// Python: bot/modules/chat.py:9939 `_a_teen_has_gone_quiet(text)`
pub fn _a_teen_has_gone_quiet(text_: &str) -> bool {
    let said = text_.to_lowercase();
    if A_QUIET_AT_ME.search(&said).is_none() {
        return false;
    }
    if A_WHAT_HAPPENED.search(&said).is_some() {
        return false;
    }
    A_A_CHILD.search(&said).is_some() && A_TEN_OR_OVER.search(&said).is_some()
}

/// Python: bot/modules/chat.py:9949 `_a_quarrel_since(memory, turn)`
pub fn _a_quarrel_since(memory: &Memory, turn: Option<&Value>) -> bool {
    let from = int_or_zero(turn).max(0) as usize;
    let rows = turns(memory);
    rows[from.min(rows.len())..].iter().any(|r| A_WHAT_HAPPENED.search(&r.user.to_lowercase()).is_some())
}

/// Python: bot/modules/chat.py:9958 `_the_ask_in_the_line(clean)`
pub fn _the_ask_in_the_line(clean: &str) -> String {
    if let Some(said) = A_ROW.search(clean) {
        let tail = text::strip(&clean[said.end()..]);
        if !tail.is_empty() && _ASKS_AFTER.search(tail).is_some() {
            return tail.to_string();
        }
    }
    clean.to_string()
}

/// The row kept in memory, when it names who it was with.
fn the_row(memory: &Memory) -> Option<Value> {
    let row = memory.get("row")?;
    if !row.is_dict() || !row.get("them").is_some_and(|t| t.truthy()) {
        return None;
    }
    Some(row.clone())
}

/// `row["them"]` as the str it is written as.
fn row_them(row: &Value) -> String {
    py_str(row.get("them").unwrap_or_else(|| panic!("KeyError: 'them'")))
}

/// Python: bot/modules/chat.py:9973 `_a_row_next(clean, memory)`
pub fn _a_row_next(clean: &str, memory: &Memory) -> Option<String> {
    let clean = _the_ask_in_the_line(clean);
    let row = the_row(memory)?;
    let turn = memory.turns.len() as i64;
    if turn - int_or_zero(row.get("turn")) > 3 {
        return None;
    }
    let how = ROW_HOW.search(&clean).is_some();
    if A_BARE_MOVE_ASK.match_(&clean).is_some()
        && _a_teen_has_gone_quiet(&py_str(row.get("text").unwrap_or(&Value::None)))
        && !_a_quarrel_since(memory, row.get("turn"))
    {
        return None;
    }
    let space_said = ROW_SAID_SPACE.search(&clean).is_some();
    if _the_base_answers_the_row(&clean, memory) {
        return None;
    }
    if how && row.get("handed_over").is_some_and(|h| h.truthy()) {
        return None;
    }
    if how
        && (WANTS_AN_APOLOGY.search(&clean).is_some() || ASKS_FOR_WORDS.search(&clean).is_some())
        && _the_base_has_an_answer(&clean, memory).is_some()
    {
        return None;
    }
    if !how && !space_said && !(ROW_CONTACT.search(&clean).is_some() && ROW_CONTACT_ASK.search(&clean).is_some()) {
        return None;
    }
    if !how && !space_said && !_names_the_person(&clean, Some(&row)) {
        return None;
    }
    if n_words(&clean) > 14 || LOSS.search(&clean).is_some() || truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    let pool = if how {
        &*ROW_HOW_REPLY
    } else if space_said || ROW_SPACE.search(&clean).is_some() {
        &*ROW_SPACE_REPLY
    } else {
        &*ROW_NEXT
    };
    let them = row_them(&row);
    Some(pick(&row_lines(&pool.strs(), &them, &_they_of(Some(&row))), memory, &clean))
}

/// Python: bot/modules/chat.py:10055 `_a_row_ending(clean, memory)`
pub fn _a_row_ending(clean: &str, memory: &Memory) -> Option<String> {
    let clean = _the_ask_in_the_line(clean);
    let row = the_row(memory)?;
    let turn = memory.turns.len() as i64;
    if turn - int_or_zero(row.get("turn")) > 3 {
        return None;
    }
    if ROW_ENDING.search(&clean).is_none() || n_words(&clean) > 14 {
        return None;
    }
    if LOSS.search(&clean).is_some() || truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    let mut pool = ROW_ENDING_REPLY.strs();
    if ROW_HUNG_UP.search(&clean).is_some() {
        pool.extend(ROW_ENDING_REPLY_HUNG_UP.strs());
    }
    let them = row_them(&row);
    Some(pick(&row_lines(&pool, &them, &_they_of(Some(&row))), memory, &clean))
}

/// Python: bot/modules/chat.py:10080 `_row_pronoun(clean)`
pub fn _row_pronoun(clean: &str) -> &'static str {
    if ROW_HER.search(clean).is_some() {
        return "her";
    }
    if ROW_HIM.search(clean).is_some() {
        return "him";
    }
    "them"
}

/// Python: bot/modules/chat.py:10089 `_their_of(them)`
pub fn _their_of(them: &str) -> String {
    let them = if them.is_empty() { "them".to_string() } else { them.to_lowercase() };
    match them.as_str() {
        "her" => "her",
        "him" => "his",
        _ => "their",
    }
    .to_string()
}

/// Python: bot/modules/chat.py:10099 `_they_of(row)`
pub fn _they_of(row: Option<&Value>) -> String {
    let they = row.and_then(|r| r.get("they")).map(py_str).unwrap_or_default();
    if they.is_empty() {
        "they".to_string()
    } else {
        they
    }
}

/// Python: bot/modules/chat.py:10104 `_row_subject(clean)`
pub fn _row_subject(clean: &str) -> &'static str {
    if ROW_HER.search(clean).is_some() {
        return "she";
    }
    if ROW_HIM.search(clean).is_some() {
        return "he";
    }
    "they"
}

/// Python: bot/modules/chat.py:10113 `_the_deed_before(memory)`: the deed (a str) or None.
pub fn _the_deed_before(memory: &Memory) -> Value {
    match memory.turns.last() {
        Some(t) => py(crate::compose::repair::deed_of(&Value::from(t.user.as_str()))),
        None => Value::None,
    }
}

/// Python: bot/modules/chat.py:10121 `_after_a_repair(memory)`
pub fn _after_a_repair(memory: &Memory) -> bool {
    let said = memory.turns.last().map(|t| squash(&t.user.to_lowercase())).unwrap_or_default();
    !said.is_empty() && (_said_wrong(&said) || _said_unkindly(&said) || truthy(crate::compose::repair::amends_to(&said)))
}

/// Python: bot/modules/chat.py:10138 `_a_repair_is_open(memory)`
pub fn _a_repair_is_open(memory: &Memory) -> bool {
    truthy(crate::compose::repair::a_repair_is_open(memory))
}

/// `pool[kind]` from a `{str: str}` table (KeyError when missing).
fn table_str(table: &Value, kind: &str) -> String {
    table.get(kind).map(py_str).unwrap_or_else(|| panic!("KeyError: {kind:?}"))
}

/// Python: bot/modules/chat.py:10148 `_about_someone(clean, memory)`
pub fn _about_someone(clean: &str, memory: &Memory) -> Option<String> {
    if truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    if crate::compose::loss::reads_as_loss(clean, None, false) || _grieving(memory, clean) {
        return None;
    }
    if _after_a_repair(memory) && SOMEONE_TROUBLE_ANY.search(clean).is_none() {
        return None;
    }
    let record = py(crate::memory::topics::about(memory));
    let record = if record.truthy() { Some(record) } else { None };
    let read = memory.understood.as_ref();
    if record.is_none() && SOMEONE_AGES_IT.search(clean).is_some() {
        let said = read.and_then(|u| u.leans_on.as_ref()).map(|l| l.text.clone()).unwrap_or_default();
        let kept = if said.is_empty() { None } else { crate::memory::topics::trouble(&said) };
        if let Some(kept) = kept {
            if let Some(line) = SOMEONE_NORMAL.get(&kept) {
                return Some(py_str(line));
            }
        }
    }
    if let Some(record) = &record {
        if n_words(clean) <= 9 {
            let kept = record.get("trouble").cloned().unwrap_or(Value::None);
            let kept_str = kept.as_str().unwrap_or("");
            let asks_normal = SOMEONE_IS_IT_NORMAL.search(clean).is_some();
            let young_night = kept_str == "sleep"
                && !asks_normal
                && YOUNG.search(&py_str(record.get("said").unwrap_or(&Value::None))).is_some();
            let in_next = kept.as_str().is_some_and(|k| SOMEONE_NEXT.get(k).is_some());
            if in_next && (asks_normal || SOMEONE_NEXT_STEP.search(clean).is_some()) && !young_night {
                let line = if asks_normal { table_str(&SOMEONE_NORMAL, kept_str) } else { table_str(&SOMEONE_NEXT, kept_str) };
                let who = record.get("who").filter(|w| w.truthy()).map(py_str).unwrap_or_else(|| "them".into());
                return Some(py_format(&line, &[("who", &who)]));
            }
            if SOMEONE_AGES_IT.search(clean).is_some() {
                return Some(SOMEONE_NORMAL_ANY.str().to_string());
            }
        }
    }
    let mine_now = memory.turns.len() as i64;
    if let Some(record) = &record {
        let first_said = match record.get("trouble_turn") {
            None => -1,
            Some(Value::Int(i)) => *i,
            Some(Value::Bool(b)) => *b as i64,
            Some(other) => panic!("TypeError: '<' not supported for {other:?}"),
        };
        if first_said < mine_now && n_words(clean) <= 12 {
            let kept = record.get("trouble").cloned().unwrap_or(Value::None);
            let pattern = kept.as_str().and_then(|k| {
                SOMEONE_TROUBLE.items().iter().rev().find(|row| row.items()[0].str() == k).map(|row| row.items()[1].clone())
            });
            let more = pattern.as_ref().and_then(|p| p.as_re()).is_some_and(|p| p.search(clean).is_some())
                || (SOMEONE_MORE.search(clean).is_some() && n_words(clean) <= 8);
            if let Some(line) = kept.as_str().and_then(|k| SOMEONE_AGAIN.get(k)) {
                if more {
                    return Some(py_str(line));
                }
            }
        }
    }
    if (LIMIT.search(clean).is_some() || truthy(crate::memory::facts::a_limit_on_the_plan(clean))) && _after_a_plan(memory, clean) {
        return None;
    }
    if _said_wrong(clean) || _said_unkindly(clean) {
        return None;
    }
    if MAYBE_SOMEONE.search(clean).is_none() {
        return None;
    }
    if LOSS.search(clean).is_some() || END_OF_ROPE.search(clean).is_some() {
        return None;
    }
    let mut who = record.as_ref().and_then(|r| r.get("who")).filter(|w| w.truthy()).map(py_str);
    if who.is_none() {
        who = crate::memory::topics::person_named(clean, memory).filter(|w| !w.is_empty());
    }
    let who = who?;
    let young = YOUNG.search(clean).is_some();
    if FEAR.search(clean).is_some() {
        return None;
    }
    if crate::compose::fallback::a_date_of_ours_forgotten(clean) {
        return None;
    }
    let kind = crate::memory::topics::trouble(clean);
    let kind_str = kind.as_deref().unwrap_or("");
    if kind_str == "hospital" && SERIOUS_DIAGNOSIS.search(clean).is_some() {
        let pool = SOMEONE_REPLY.get("diagnosed").unwrap_or_else(|| panic!("KeyError: 'diagnosed'")).strs();
        return Some(py_format(&pick(&pool, memory, ""), &[("who", &who)]));
    }
    if let Some(pool) = kind.as_deref().and_then(|k| SOMEONE_REPLY.get(k)) {
        if !(SYMPTOM.search(clean).is_some() && kind_str == "ill") && !((kind_str == "low" || kind_str == "sleep") && young) {
            return Some(py_format(&pick(&pool.strs(), memory, ""), &[("who", &who)]));
        }
    }
    if read.is_some_and(|u| u.kind == "question") {
        return None;
    }
    if SOMEONE_TROUBLE_ANY.search(clean).is_some() && !_a_thing_cannot(clean) && !_still_about_me(clean) && !_better_answer_exists(clean, memory) {
        return Some(py_format(&pick(&SOMEONE_ASK.strs(), memory, ""), &[("who", &who)]));
    }
    None
}

/// Python: bot/modules/chat.py:10263 `_after_a_plan(memory, clean)`
pub fn _after_a_plan(memory: &Memory, clean: &str) -> bool {
    let turns = turns(memory);
    let mine = squash(&clean.to_lowercase());
    for turn in turns.iter().rev() {
        let said = squash(&turn.user.to_lowercase());
        if said.is_empty() || said == mine {
            continue;
        }
        if PLAN_ASK.search(&said).is_some() {
            return true;
        }
        break;
    }
    if let Some(Value::List(log)) = memory.get("facts_log") {
        let newest = turns.len() as i64;
        for entry in log.iter().rev() {
            if !entry.is_dict() {
                continue;
            }
            let key = entry.get("key").and_then(|k| k.as_str());
            if key != Some("plan") && key != Some("plans") {
                continue;
            }
            let when = match entry.get("turn") {
                Some(Value::Int(i)) => *i,
                Some(Value::Bool(b)) => *b as i64,
                Some(Value::Float(f)) => f.trunc() as i64,
                Some(Value::Str(s)) => match text::strip(s).parse::<i64>() {
                    Ok(i) => i,
                    Err(_) => return false,
                },
                _ => return false,
            };
            return newest - when <= 3;
        }
    }
    false
}

/// Python: bot/modules/chat.py:10298 `_grieving(memory, clean)`
pub fn _grieving(memory: &Memory, clean: &str) -> bool {
    if truthy(crate::compose::loss::continues(memory)) || crate::compose::badnews::reads_as_sudden_loss(clean) {
        return true;
    }
    let mine = squash(&clean.to_lowercase());
    for turn in turns(memory).iter().rev() {
        let user = squash(&turn.user.to_lowercase());
        if user.is_empty() || user == mine {
            continue;
        }
        let said = &turn.bot;
        if crate::compose::loss::reads_as_loss(&user, None, false) || crate::compose::badnews::reads_as_sudden_loss(&user) {
            return true;
        }
        let pools = data::get("bot.compose.loss.POOLS");
        return pools.items().iter().any(|pool| pool.contains_str(said))
            || data::get("bot.compose.fallback.SUDDEN_LOSS").contains_str(said);
    }
    false
}

/// Python: bot/modules/chat.py:10322 `_better_answer_exists(clean, memory)`
pub fn _better_answer_exists(clean: &str, memory: &Memory) -> bool {
    attempt(|| {
        if crate::knowledge::lookup::find(&crate::understanding::understand(clean, Some(memory)), None, None, false).is_some() {
            return true;
        }
        !py(crate::compose::fallback::who_settles(clean, memory)).is_none()
    })
    .unwrap_or(false)
}

/// Python: bot/modules/chat.py:10338 `_still_about_me(clean)`
pub fn _still_about_me(clean: &str) -> bool {
    FIRST_PERSON.search(&WHO_SAID.sub(" ", clean)).is_some()
}

/// Python: bot/modules/chat.py:10349 `_about_the_person(understood)`
pub fn _about_the_person(understood: Option<&Understood>) -> bool {
    !understood.is_some_and(|u| u.state_owner.as_deref() == Some("thing"))
}

inline_re!(ANY_SPACE, r"\s+");

/// Python: bot/modules/chat.py:10357 `_my_night(clean)`
pub fn _my_night(clean: &str) -> bool {
    let t = ANY_SPACE.sub(" ", &clean.to_lowercase());
    if MY_NIGHT.search(&t).is_none() {
        return false;
    }
    NOT_MY_NIGHT.search(&t).is_none()
}

/// `the_body_is_short_of_sleep(clean)` on a bare line: Python's `_the_whole_line` reads a str as
/// itself lower-cased, which is the reading's `words` joined when `text` and `clean` are empty.
fn the_body_is_short_of_sleep_said(clean: &str) -> bool {
    let u = Understood { words: vec![clean.to_string()], ..Understood::default() };
    crate::knowledge::lookup::the_body_is_short_of_sleep(&u)
}

/// Python: bot/modules/chat.py:10365 `_stuck(clean, memory=None)`
pub fn _stuck(clean: &str, memory: &Memory) -> Option<String> {
    let sleepy = the_body_is_short_of_sleep_said(clean);
    if !(CANT.search(clean).is_some() || CANT_OPENING.match_(clean).is_some() || ITS_BEEN.search(clean).is_some() || sleepy)
        || STUCK_SAFETY.search(clean).is_some()
    {
        return None;
    }
    if STUCK_IDIOM.search(clean).is_some()
        || STUCK_REASON.search(clean).is_some()
        || STUCK_SELF.search(clean).is_some()
        || _doing_something_to_a_thing(clean)
    {
        return None;
    }
    if STUCK_FIXABLE.search(clean).is_some() {
        return None;
    }
    if STUCK_LOSS.search(clean).is_some() || said_loss(memory) {
        return None;
    }
    if the_body_is_short_of_sleep_said(clean)
        && !(CANT.search(clean).is_some() || CANT_OPENING.match_(clean).is_some() || ITS_BEEN.search(clean).is_some())
    {
        return Some("sleep".into());
    }
    for row in STUCK.items() {
        let name = row.items()[0].str();
        if name == "work" && STUCK_OBJECT.search(clean).is_some() {
            continue;
        }
        for pattern in row.items()[1].items() {
            if dyn_re(pattern.str(), 0).search(clean).is_some() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Python: bot/modules/chat.py:10465 `_names_a_kind_of_thing(between, after)`
pub fn _names_a_kind_of_thing(between: &str, after: &str) -> bool {
    if !text::strip(text::strip_chars(after, " .,!?;:")).is_empty() {
        return false;
    }
    let words: Vec<String> = text::split_ref(between).into_iter().map(|w| bare(w).to_lowercase()).filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return false;
    }
    let articles = ["a", "an", "the"];
    if articles.contains(&words[words.len() - 1].as_str()) {
        return true;
    }
    let Some(last_article) = words.iter().rposition(|w| articles.contains(&w.as_str())) else { return false };
    let of_the_kind = &words[last_article + 1..];
    if of_the_kind.iter().any(|w| in_set(&HOW_MUCH_OF_IT, w)) {
        return false;
    }
    !of_the_kind.is_empty()
}

/// Python: bot/modules/chat.py:10639 `_recent_plural(memory)`
pub fn _recent_plural(memory: &Memory) -> bool {
    let said = last_n(turns(memory), 3).iter().map(|t| t.user.to_lowercase()).collect::<Vec<_>>().join(" ");
    if said.is_empty() {
        return false;
    }
    let words = text::split_ref(&said);
    if PLURAL_WORDS.iter().any(|w| words.contains(&w.as_str())) {
        return true;
    }
    JOINED_PERSON.search(&said).is_some()
}

// ---------------------------------------------------------------------------------------------
// Python lines 10650-12380: somebody else's feelings, fed up, places, news, nights out.

res!(
    OTHER_BAD_NIGHT, OTHER_PRONOUN, CRY_WOLF, OTHER_STOP, OTHER_QUESTION, _A_HELP_ASK_AT_END,
    OTHER_ADVERB, OTHER_VERB, OTHER_NEG, THING_THAT_STAYS, FED_UP_THING, FED_UP_STOPS,
    FED_UP_BARE, SECOND_PERSON_RE, FEDUP_SAID_PHRASE, PLACE_TIME_OPENS, PLACE_THING_OPENS,
    PLACE_LAST, PLACE_TIME, PLACE_NOBODY, STUCK_ON_A_FORM, STUCK_ON_A_THING, STUCK_FEELING,
    _HOW_LONG_AT_IT, LOST_ONLY, PRESENCE, PAUSE, RETURNED, AWAY_WORDS, BID, HOME_AGAIN, LEAVING,
    UNDERWAY, IN_BED, AWAKE_SINCE, ERRAND, PUT_ON, _A_DRESSING, TAKEN_OUT, HAD_A_THING_DONE,
    RESULT_CLEAR, DOCTOR_SAID, NIGHT, LONG_AGO, PART_FINE, REST_WORD, GOT_RESULTS, WEARER,
    BACK_FROM_CARE, WENT_TO_CARE, WAS_IN_CARE, THIS_WEEK, WISH_LUCK, RETIRED, RETIRED_ASKING,
    OUT_OF_WORK, AGE_BEFORE_A_ROLE, ROLE_WORK, ROLE, ROLE_TO_PEOPLE, COMPARED_ROLE, PLACE,
    PLACE_SPAN, GROWN_IN, MOVING, TRIP, PET, PET_ANY, LOST_WEIGHT, STATUS, CAME_BACK,
    JUST_STARTED, JUST_DONE, A_MOVE, VERDICT_THING, WENT_TO_SEE, WENT_TO_SEE_COMPANY, WENT_TO,
    WENT_TO_COMPANY, VERDICT_WELL, WENT_BAD_WEATHER, VERDICT_POOR, WENT_WRONG, WENT_WELL,
    TRIP_TIME_TAIL, SOME_WHEN, TRIP_NOT_YET, BACK_FROM, BEEN_TO, WENT_TO_ANY, GOOD_TIME,
    WANTS_A_THING, ASKS_ALOUD, JOKE_ADVICE, SELF_JOKE, JOKE_ASKED, RUNTIME_LIMIT, JOKE_ASK,
);
sets!(
    SHE_WORDS, HE_WORDS, OTHER_FEELING_VERB, FED_UP_THIN, NOT_A_THING, NOT_A_THING_FIRST,
    NOTHING_NAMED, CONFUSION_IDIOM_WORDS, FED_UP_FILLER, PLACE_IDIOM, PLACE_POINTS,
    ARTICLE_PLACES, NOT_A_SUBJECT, A_FEELING_SAID, RESTED, NOT_A_DIAGNOSIS, A_BODY, ROLE_STOP,
    PLACE_NONE, PLACE_POSITION, WATCHED_THINGS, WATCHED_DESCRIBER, SAID_POOR, KNOWN_PLACES,
    TRIP_SHORT_COUNTRIES, NOT_A_PLACE_START, PICK_WORDS,
);
vals!(
    GENERIC_OTHER_REPLIES, OTHER_REPLIES, ABOUT_REPLIES, FEDUP_LEAD, FEDUP_REPLIES,
    FEDUP_GERUND_SAFE, SECOND_PERSON, OTHER_REPLIES_SMALL_CHILD, JOKES, SHORT_JOKES, PLACE_TAIL,
    PLACE_MOST, COUNTRY_CAPS, INITIALISMS, BAD_NEWS, GOOD_TIME_LINES, DETAIL_LINES,
);

/// A dict's value for an int key (`KID_NUMBER_WORDS.get(n)`).
fn get_int_key(table: &Value, key: i64) -> Option<&Value> {
    table.pairs().iter().find(|(k, _)| k.as_int() == Some(key)).map(|(_, v)| v)
}

/// The `(name, value)` fields a format call takes, borrowed.
fn kv<'a>(fields: &'a [(&'a str, String)]) -> Vec<(&'a str, &'a str)> {
    fields.iter().map(|(k, v)| (*k, v.as_str())).collect()
}

/// Python: bot/modules/chat.py:10653 `_pronouns(word)`: (object, possessive, plural).
pub fn _pronouns(word: &str) -> (&'static str, &'static str, bool) {
    let word = word.to_lowercase();
    if in_set(&PLURAL_WORDS, &word) {
        return ("them", "their", true);
    }
    if in_set(&SHE_WORDS, &word) {
        return ("her", "her", false);
    }
    if in_set(&HE_WORDS, &word) {
        return ("him", "his", false);
    }
    ("them", "their", true)
}

/// What `_other_feeling` reads: the feeling, the person's pronouns, and the thing it is about.
#[derive(Clone, Debug, PartialEq)]
pub struct OtherFeeling {
    pub feeling: String,
    pub obj: String,
    pub poss: String,
    pub plural: bool,
    pub thing: String,
}

fn other(feeling: &str, p: (&str, &str, bool), thing: &str) -> OtherFeeling {
    OtherFeeling { feeling: feeling.into(), obj: p.0.into(), poss: p.1.into(), plural: p.2, thing: thing.into() }
}

inline_re!(OTHER_FED_TAIL, r"\b(?:sick of|tired of|fed up with|fed up of|done with|had enough of|had it with)\s+[^.!?]*$");
inline_re!(BE_VERB_END, r"(?:'s|'re|s|re)$");
inline_re!(AT_THEM_ME, r"\b(?:with|at|about|of|by)\s+(?:me|us)\b");
inline_re!(SHE_LATER, r"\b(she|her|hers)\b");
inline_re!(HE_LATER, r"\b(he|him|his)\b");
inline_re!(OTHER_FED, r"\b(?:sick of|done with|had enough of|had it with)\b\s*(.*)$");
inline_re!(SPEAKER_BETWEEN, r"\b(?:i'm|im|i am|i feel|i've|i have|me|my)\b");
inline_re!(SPEAKER_HAD, r"\b(?:i'm|im|i am|i feel|i've|i had)\b");

/// Python: bot/modules/chat.py:10680 `_other_feeling(clean, memory=None)`
pub fn _other_feeling(clean: &str, memory: &Memory) -> Option<OtherFeeling> {
    let body = OTHER_FED_TAIL.sub("", clean);
    let asked_ = _A_HELP_ASK_AT_END.sub("", clean);
    let asked_ = text::strip(&asked_);
    if clean.ends_with('?') && asked_ == clean {
        return None;
    }
    if OTHER_STOP.search(&body).is_some() || OTHER_QUESTION.search(asked_).is_some() {
        return None;
    }
    if CRY_WOLF.search(clean).is_some() {
        return None;
    }
    let mut be_verb = false;
    let (mut word, start) = match OTHER_PERSON.search(clean) {
        Some(m) => (m.str(1).to_string(), m.end()),
        None => {
            let m = OTHER_PRONOUN.match_(clean)?;
            let pronoun = m.as_str();
            let word = text::rstrip_chars(pronoun.split('\'').next().unwrap_or(""), "s").to_string();
            if ["they", "them", "their", "we", "us", "you"].contains(&word.to_lowercase().as_str()) && !_recent_plural(memory) {
                return None;
            }
            be_verb = BE_VERB_END.search(pronoun).is_some() && text::len(pronoun) > 2;
            (word, m.end())
        }
    };
    let rest = &clean[start..];
    if AT_THEM_ME.search(rest).is_some() {
        return None;
    }
    let low = word.to_lowercase();
    if !in_set(&SHE_WORDS, &low) && !in_set(&HE_WORDS, &low) {
        if SHE_LATER.search(rest).is_some() {
            word = "she".into();
        } else if HE_LATER.search(rest).is_some() {
            word = "he".into();
        }
    }
    if let Some(fed) = OTHER_FED.search(rest) {
        let thing = _the_thing(fed.str(1));
        let pronouns = _pronouns(&word);
        if !thing.is_empty() && _thing_that_stays(&thing) {
            return Some(other("fedup", pronouns, &thing));
        }
        return Some(other("annoyed", pronouns, ""));
    }
    if OTHER_BAD_NIGHT.search(rest).is_some() {
        return Some(other("tired", _pronouns(&word), ""));
    }
    for row in FEELINGS.items() {
        let name = row.items()[0].str();
        for feeling in row.items()[1].items() {
            let feeling = feeling.str();
            let Some(hit) = word_re(feeling).search(rest).map(|h| (h.start(), h.end())) else { continue };
            let between = &rest[..hit.0];
            if _names_a_kind_of_thing(between, &rest[hit.0 + feeling.len()..]) {
                continue;
            }
            let glue_ = OTHER_ADVERB.sub("", between);
            let glue = text::strip(&glue_);
            let their_verb = glue.is_empty() && !be_verb && in_set(&OTHER_FEELING_VERB, feeling);
            if !(their_verb || OTHER_VERB.search(between).is_some() || (be_verb && glue.is_empty())) {
                continue;
            }
            if !their_verb && !truthy(crate::understanding::kinds::cue_about_person(clean, feeling)) {
                continue;
            }
            if in_set(&WEARS_OTHER_PEOPLE_OUT, feeling) {
                continue;
            }
            if SPEAKER_BETWEEN.search(between).is_some() {
                return None;
            }
            if OTHER_NEG.search(between).is_some() {
                continue;
            }
            let after = &rest[hit.0..];
            let esc = crate::py::re::escape(feeling);
            if dyn_re(&format!(r"\b{esc}\s+with\s+(?:me|us|him|her|them)\b"), 0).match_(after).is_some() {
                return None;
            }
            let caused_re = dyn_re(&format!(r"\b{esc}\s+(?:of|about|with)\s+(.+)$"), 0);
            if let Some(caused) = caused_re.match_(after) {
                let thing = _the_thing(caused.str(1));
                if ["tired", "bored", "sick", "worn out", "done in", "fed up", "browned off", "peeved"].contains(&feeling) {
                    let pronouns = _pronouns(&word);
                    if !thing.is_empty() && _thing_that_stays(&thing) {
                        return Some(other("fedup", pronouns, &thing));
                    }
                    return Some(other(if feeling == "bored" { "bored" } else { "annoyed" }, pronouns, ""));
                }
                if ABOUT_REPLIES.get(name).is_some() {
                    return Some(other(&format!("{name}_about"), _pronouns(&word), &thing));
                }
                continue;
            }
            if feeling == "low" && dyn_re(r"\blow\s+(?:on|in)\b", 0).search(after).is_some() {
                continue;
            }
            if feeling == "spent" && dyn_re(r"\bspent\s+(?:up|all|it|my|the|every)\b", 0).search(after).is_some() {
                continue;
            }
            let joined = &clean[..start + hit.0];
            if JOINED_PERSON.search(joined).is_some() {
                return Some(other(name, ("them", "their", true), ""));
            }
            return Some(other(name, _pronouns(&word), ""));
        }
    }
    if ROUGH_DAY.search(rest).is_some() && SPEAKER_HAD.search(rest).is_none() {
        return Some(other("tired", _pronouns(&word), ""));
    }
    None
}

inline_re!(THING_CUT, r"\b(?:because|but|so|and|since|when|after|before|which|that)\b");

/// Python: bot/modules/chat.py:10825 `_the_thing(phrase)`
pub fn _the_thing(phrase: &str) -> String {
    let squashed = squash(phrase);
    let phrase = text::strip_chars(&squashed, " .,!?;:");
    let first = match THING_CUT.search(phrase) {
        Some(m) => &phrase[..m.start()],
        None => phrase,
    };
    let phrase = text::strip(first);
    let n = n_words(phrase);
    if n == 0 || n > 5 {
        return String::new();
    }
    phrase.to_string()
}

/// Python: bot/modules/chat.py:10848 `_thing_that_stays(thing)`
pub fn _thing_that_stays(thing: &str) -> bool {
    THING_THAT_STAYS.match_(thing).is_some()
}

/// Python: bot/modules/chat.py:10902 `_confusion_names_a_thing(understood)`
pub fn _confusion_names_a_thing(understood: &Understood) -> bool {
    for word in &understood.keywords {
        let word = text::strip_chars(word, ".,!?;:'\"()").to_lowercase();
        if !word.is_empty() && !in_set(&NOTHING_NAMED, &word) && !in_set(&CONFUSION_IDIOM_WORDS, &word) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/chat.py:10975 `_fedup_phrase(said, at)` (`at` a byte offset into `said`).
pub fn _fedup_phrase(said: &str, at: usize) -> String {
    match FEDUP_SAID_PHRASE.match_(&said[at.min(said.len())..]) {
        Some(hit) => hit.str(1).to_string(),
        None => "tired of".into(),
    }
}

inline_re!(MY_OR_OUR, r"^(?:my|our)\b(.*)$");

/// Python: bot/modules/chat.py:10981 `_fed_up_with(clean)`
pub fn _fed_up_with(clean: &str) -> String {
    let line = squash(clean);
    let Some(hit) = FED_UP_THING.search(&line) else { return String::new() };
    let parts0 = FED_UP_STOPS.split(hit.str(1));
    let tail = text::strip_chars(&parts0[0], " .");
    if tail.is_empty() {
        return String::new();
    }
    let mut parts: Vec<&str> = text::split_ref(tail);
    while parts.first().is_some_and(|p| in_set(&FED_UP_FILLER, p)) {
        parts.remove(0);
    }
    while parts.last().is_some_and(|p| in_set(&FED_UP_FILLER, p)) {
        parts.pop();
    }
    let mut tail = parts.join(" ");
    if tail.is_empty() {
        return String::new();
    }
    let owner = MY_OR_OUR.match_(&tail).map(|m| m.str(1).to_string());
    if owner.is_none() && FED_UP_BARE.match_(&tail).is_none() {
        return String::new();
    }
    let first = text::split_ref(&tail)[0].to_string();
    if in_set(&NOT_A_THING, &tail) || in_set(&NOT_A_THING_FIRST, &first) {
        return String::new();
    }
    if text::split_ref(&tail).iter().all(|w| in_set(&FED_UP_THIN, w)) {
        return String::new();
    }
    if let Some(rest) = owner {
        tail = format!("your{rest}");
    }
    SECOND_PERSON_RE.sub_fn(&tail, |hit| table_str(&SECOND_PERSON, hit.str(1)))
}

/// Python: bot/modules/chat.py:11026 `_the_small_child(memory)`: the age as it is kept (an int,
/// or a bool, which Python's `isinstance(age, int)` lets through).
pub fn _the_small_child(memory: &Memory) -> Option<Value> {
    let others: Vec<&Value> = match memory.get("others") {
        Some(v) => v.items().iter().filter(|e| e.is_dict()).collect(),
        None => Vec::new(),
    };
    if others.len() != 1 {
        return None;
    }
    let kept = others[0].get("age")?;
    let age = match kept {
        Value::Int(i) => *i,
        Value::Bool(b) => *b as i64,
        _ => return None,
    };
    if 0 < age && age < 9 {
        Some(kept.clone())
    } else {
        None
    }
}

inline_re!(FED_LEAD_SAID, r"\b(fed up with|fed up of|fed up|sick and tired of|sick of|tired of|done with|had enough of|had it with)\b");

/// Python: bot/modules/chat.py:11045 `_someone_reply(memory, feeling, obj, poss, plural, thing="", said="")`
pub fn _someone_reply(memory: &Memory, feeling: &str, obj: &str, poss: &str, plural: bool, thing: &str, said: &str) -> String {
    let subj = match obj {
        "her" => "she",
        "him" => "he",
        _ => "they",
    };
    let they = subj == "they";
    let mut values: Vec<(&str, String)> = vec![
        ("obj", obj.into()),
        ("poss", poss.into()),
        ("plate", if plural { "plates" } else { "plate" }.into()),
        ("target", if plural { "to carry" } else { "for one person to be carrying" }.into()),
        ("thing", thing.into()),
        ("subj", subj.into()),
        ("subj_is", if they { "are" } else { "is" }.into()),
        ("Subj_is", if they { "Are" } else { "Is" }.into()),
        ("subj_does", if they { "do" } else { "does" }.into()),
        ("Subj_has", if they { "Have" } else { "Has" }.into()),
    ];
    if feeling == "sad" && _the_small_child(memory).is_some() {
        return pick(&OTHER_REPLIES_SMALL_CHILD.strs(), memory, said);
    }
    let pool: Vec<String> = if feeling == "fedup" {
        let fed = FED_LEAD_SAID.search(said).map(|m| m.str(1).to_string()).unwrap_or_else(|| "fed up".into());
        let lead = FEDUP_LEAD.get(&fed).map(py_str).unwrap_or_else(|| "Being fed up with".into());
        values.push(("lead", lead));
        if thing.starts_with("being ") || thing.starts_with("having ") {
            FEDUP_GERUND_SAFE.strs()
        } else {
            FEDUP_REPLIES.strs()
        }
    } else if let Some(base) = feeling.strip_suffix("_about") {
        ABOUT_REPLIES.get(base).filter(|p| p.truthy()).map(|p| p.strs()).unwrap_or_else(|| GENERIC_OTHER_REPLIES.strs())
    } else {
        OTHER_REPLIES.get(feeling).filter(|p| p.truthy()).map(|p| p.strs()).unwrap_or_else(|| GENERIC_OTHER_REPLIES.strs())
    };
    let fields = kv(&values);
    let options: Vec<String> = pool.iter().map(|o| py_format(o, &fields)).collect();
    pick(&options, memory, "").replace("  ", " ")
}

/// Python: bot/modules/chat.py:11084 `_last_was_joke(memory)`
pub fn _last_was_joke(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    if last.module.as_deref() != Some("chat") {
        return false;
    }
    JOKES.pairs().iter().any(|(_, v)| v.as_str() == Some(last.bot.as_str())) || SHORT_JOKES.contains_str(&last.bot)
}

inline_re!(DANGLING_PREPOSITION, r"\s+(?:in|to|at|near|from|of|on)\s*$");

/// Python: bot/modules/chat.py:11091 `_drop_tail(phrase)`
pub fn _drop_tail(phrase: &str) -> String {
    let mut phrase = squash(phrase);
    let mut changed = true;
    while !phrase.is_empty() && changed {
        changed = false;
        for tail in PLACE_TAIL.items() {
            let Some(re) = tail.as_re() else { continue };
            let cut = re.sub("", &phrase);
            let shorter = text::strip_chars(&cut, " ,.;:-'").to_string();
            if shorter != phrase {
                phrase = shorter;
                changed = true;
            }
        }
    }
    DANGLING_PREPOSITION.sub("", &phrase)
}

const PLACE_STRIP: &str = " ,.;:-'";

/// Python: bot/modules/chat.py:11108 `_place_of(phrase)`
pub fn _place_of(phrase: &str) -> String {
    let mut phrase = _drop_tail(phrase);
    if PLACE_TIME_OPENS.match_(text::strip_chars(&phrase, PLACE_STRIP)).is_some() {
        return String::new();
    }
    phrase = text::strip_chars(&py_str(&py(crate::memory::facts::take_back_cut(&phrase))), PLACE_STRIP).to_string();
    if PLACE_THING_OPENS.match_(&phrase).is_some() {
        if let Some(inner) = PLACE_LAST.match_(&phrase) {
            let inside = inner.str(1).to_string();
            phrase = _drop_tail(&inside);
        }
    } else {
        phrase = text::strip_chars(&PLACE_TIME.sub("", &phrase), PLACE_STRIP).to_string();
    }
    if ["in", "on", "at", "to"].contains(&text::strip_chars(&phrase, PLACE_STRIP).to_lowercase().as_str()) {
        return String::new();
    }
    if PLACE_NOBODY.match_(&phrase).is_some() {
        return String::new();
    }
    if phrase.is_empty() || phrase.chars().any(|c| text::is_digit(&c.to_string())) {
        return String::new();
    }
    let words = text::split_ref(&phrase);
    if words.len() as i64 > PLACE_MOST.as_int().unwrap_or(4) {
        return String::new();
    }
    if in_set(&PLACE_IDIOM, &words[0].to_lowercase()) || in_set(&PLACE_IDIOM, &phrase.to_lowercase()) {
        return String::new();
    }
    if in_set(&PLACE_POINTS, &phrase.to_lowercase()) {
        return String::new();
    }
    phrase
}

/// Python: bot/modules/chat.py:11151 `_tidy(phrase)`
pub fn _tidy(phrase: &str) -> String {
    let words = text::split_ref(phrase);
    if let Some(first) = words.first() {
        if let Some(caps) = COUNTRY_CAPS.get(&first.to_lowercase()) {
            return format!("{}{}", caps.str(), text::slice(phrase, text::len(first) as isize, None));
        }
        if first.to_lowercase() == "the" {
            let rest = words[1..].join(" ");
            if in_set(&ARTICLE_PLACES, &rest.to_lowercase()) {
                return format!("The {}", _tidy(&rest));
            }
            return if rest.is_empty() { "The".into() } else { format!("The {rest}") };
        }
        if ["a", "an", "my", "our", "this"].contains(first) {
            let tail = if words.len() > 1 { format!(" {}", words[1..].join(" ")) } else { String::new() };
            return format!("{}{}", upper1(first), tail);
        }
    }
    const SMALL: [&str; 12] = ["in", "of", "the", "and", "at", "on", "by", "near", "upon", "under", "next", "to"];
    words.iter().map(|w| if SMALL.contains(w) { w.to_string() } else { upper1(w) }).collect::<Vec<_>>().join(" ")
}

/// Python: bot/modules/chat.py:11169 `_numbers(values)`
pub fn _numbers(values: &[Value]) -> String {
    let table = data::get("bot.memory.facts.KID_NUMBER_WORDS");
    let words: Vec<String> = values
        .iter()
        .map(|v| {
            let n = py_int(v);
            get_int_key(table, n).map(py_str).unwrap_or_else(|| match v {
                Value::Str(s) => s.to_string(),
                other => py_str(other),
            })
        })
        .collect();
    if words.len() <= 1 {
        return words.concat();
    }
    format!("{} and {}", words[..words.len() - 1].join(", "), words[words.len() - 1])
}

/// Python: bot/modules/chat.py:11177 `_place_values(named)`
pub fn _place_values(named: &str) -> Vec<(&'static str, String)> {
    let words = text::split_ref(named);
    let inside = match words.first() {
        Some(first) if first.chars().next().is_some_and(|c| c.is_uppercase()) && ["the", "a", "an"].contains(&first.to_lowercase().as_str()) => {
            let rest = if words.len() > 1 { format!(" {}", words[1..].join(" ")) } else { String::new() };
            format!("{}{}", first.to_lowercase(), rest)
        }
        _ => named.to_string(),
    };
    vec![("X", named.to_string()), ("x", inside)]
}

/// `NEWS_REPLIES[kind]` (KeyError when there is none).
fn news_replies(kind: &str) -> Vec<String> {
    NEWS_REPLIES.get(kind).unwrap_or_else(|| panic!("KeyError: {kind:?}")).strs()
}

/// Python: bot/modules/chat.py:11194 `_say(kind, memory, **values)`
pub fn _say(kind: &str, memory: &mut Memory, values: &[(&str, &str)]) -> String {
    if !_mid_chat(memory) {
        if let Some(opening) = FIRST_LINE_POOL.get(kind).filter(|o| o.truthy()) {
            let opening = py_str(opening);
            return _reply(&opening, memory, None, "");
        }
        let all = news_replies(kind);
        let fresh: Vec<&String> = all.iter().filter(|o| !crate::compose::onscreen::talks_about_the_conversation(o)).collect();
        if !fresh.is_empty() && fresh.len() < all.len() {
            let options: Vec<String> = fresh.iter().map(|o| py_format(o, values)).collect();
            return pick(&options, memory, "");
        }
    }
    if in_set(&CHEER_KINDS, kind) && _a_death_just_said(memory) {
        return pick(&CHEER_IN_LOSS.strs(), memory, "");
    }
    let options: Vec<String> = news_replies(kind).iter().map(|o| py_format(o, values)).collect();
    if kind == "good_news" {
        _note_the_pool(memory, kind);
    }
    pick(&options, memory, "")
}

/// Python: bot/modules/chat.py:11216 `_cheer_or_the_question(flat, memory)`
pub fn _cheer_or_the_question(flat: &str, memory: &mut Memory) -> String {
    if let Some(years_back) = said(crate::compose::goodnews::reply_for_a_win_from_a_past_year(flat)) {
        _note_the_pool(memory, "good_news_gone_by");
        return years_back;
    }
    _say("good_news", memory, &[])
}

/// Python: bot/modules/chat.py:11231 `_note_the_pool(memory, kind)`
pub fn _note_the_pool(memory: &mut Memory, kind: &str) {
    let mut note = Value::dict();
    note.set_key("kind", kind);
    note.set_key("turn", memory.turns.len());
    memory.set("said_kind", note);
}

/// Python: bot/modules/chat.py:11242 `_just_named(memory, name)`
pub fn _just_named(memory: &mut Memory, name: &str) -> Option<Value> {
    if name.is_empty() {
        return None;
    }
    let user = crate::memory::user(memory).clone();
    let pets: Vec<Value> = user.get("pets").map(|p| p.items().iter().filter(|p| p.is_dict()).cloned().collect()).unwrap_or_default();
    let low = name.to_lowercase();
    pets.into_iter().filter(|p| py_str(p.get("name").unwrap_or(&Value::None)).to_lowercase() == low).last()
}

/// Python: bot/modules/chat.py:11257 `_name_said(name)`
pub fn _name_said(name: &str) -> String {
    let name = text::strip(name);
    if !name.is_empty() && !name.chars().any(|c| c.is_uppercase()) {
        return text::split_ref(name).iter().map(|w| upper1(w)).collect::<Vec<_>>().join(" ");
    }
    name.to_string()
}

inline_re!(CALLED_NAME, r"\b(?:called|named)\s+(?:([a-z][a-z'-]+)\s+)?([a-z][a-z'-]+)");

/// `facts._with_title(match)`: its contract takes the match as a `Value`, given here as the
/// match's groups by name.
fn with_title(m: &Match) -> String {
    let mut groups = Value::dict();
    groups.set_key("title", m.name("title").map(Value::from).unwrap_or(Value::None));
    groups.set_key("name", m.name("name").map(Value::from).unwrap_or(Value::None));
    py_str(&py(crate::memory::facts::_with_title(&groups)))
}

/// Python: bot/modules/chat.py:11268 `_pet_name(flat, match)`
pub fn _pet_name(flat: &str, m: &Match) -> String {
    let rest = &flat[m.end()..];
    let mut stop = rest.len();
    for other in PET_ANY.finditer(rest) {
        stop = stop.min(other.start());
    }
    let window = &rest[..stop];
    let found = CALLED_NAME.search(window);
    if let Some(f) = &found {
        if let Some(first) = f.get(1) {
            if data::get("bot.memory.facts.NOT_A_PRONOUN_NAME").contains_str(&first.to_lowercase()) {
                return f.str(2).to_string();
            }
        }
    }
    if let Some(titled) = data::re("bot.memory.facts.PET_NAME").search(window) {
        return with_title(&titled);
    }
    if let Some(f) = &found {
        return f.str(2).to_string();
    }
    let start = text::char_index(flat, m.start()) as i64;
    said(crate::memory::facts::pet_appositive(flat, start)).unwrap_or_default()
}

/// Python: bot/modules/chat.py:11295 `_question_coming(understood)`
pub fn _question_coming(understood: Option<&Understood>) -> bool {
    understood.is_some_and(|u| u.asks.iter().any(|a| a == "question_coming"))
}

/// Python: bot/modules/chat.py:11318 `_the_not_knowing_has_a_subject(memory)`
pub fn _the_not_knowing_has_a_subject(memory: &Memory) -> bool {
    let Some(u) = memory.understood.as_ref() else { return false };
    if u.topic_own.as_deref().is_some_and(|t| in_set(&A_FEELING_SAID, t)) {
        return false;
    }
    match u.topic.as_deref() {
        Some(topic) if !topic.is_empty() => !in_set(&NOT_A_SUBJECT, topic),
        _ => false,
    }
}

inline_re!(BED_WORDS, r"\b(?:bed|sleep|sleeping|kip|nap)\b");

/// Python: bot/modules/chat.py:11337 `_conversation(clean, tokens, memory)`
pub fn _conversation(clean: &str, tokens: &[String], memory: &mut Memory) -> Option<String> {
    if tokens.len() > 8 {
        return None;
    }
    let replaced = clean.replace('\'', "");
    let flat = text::strip_chars(&replaced, " .,!;:").to_string();
    if truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    if STUCK_ON_A_FORM.match_(&flat).is_some() {
        return Some(_say("on_a_form", memory, &[]));
    }
    if STUCK_ON_A_THING.match_(&flat).is_some() {
        return Some(_say("on_the_thing", memory, &[]));
    }
    if STUCK_FEELING.match_(&flat).is_some() {
        return Some(_say("a_stuck_feeling", memory, &[]));
    }
    if AT_IT.match_(&flat).is_some() {
        return Some(_say("at_it", memory, &[]));
    }
    if _HOW_LONG_AT_IT.match_(&flat).is_some() && _asked_about_the_effort(memory) {
        return Some(_say("the_long_haul", memory, &[]));
    }
    if LOST_ONLY.match_(&flat).is_some()
        && !_the_not_knowing_has_a_subject(memory)
        && !truthy(crate::memory::stuck::_the_answer_gave_a_step(memory))
    {
        return Some(_say("lost", memory, &[]));
    }
    if PRESENCE.search(&flat).is_some() {
        if !truthy(crate::memory::tried::still_going_on(clean, memory)) {
            return Some(_say("presence", memory, &[]));
        }
    }
    if PAUSE.search(&flat).is_some() {
        return Some(_say("pause", memory, &[]));
    }
    if RETURNED.search(&flat).is_some() && AWAY_WORDS.search(&flat).is_none() && _a_thread_to_come_back_to(memory) {
        return Some(_say("returned", memory, &[]));
    }
    if BID.search(&flat).is_some() {
        return Some(_say("bid", memory, &[]));
    }
    if HOME_AGAIN.search(&flat).is_some() && AWAY_WORDS.search(&flat).is_none() {
        return Some(_say("home_again", memory, &[]));
    }
    if LEAVING.search(&flat).is_some() {
        if BED_WORDS.search(&flat).is_some() {
            return Some(_say("leaving_bed", memory, &[]));
        }
        return Some(_say("leaving", memory, &[]));
    }
    if UNDERWAY.search(&flat).is_some() {
        return Some(_say("underway", memory, &[]));
    }
    if IN_BED.search(&flat).is_some() {
        return Some(_say("in_bed", memory, &[]));
    }
    if let Some(awake) = AWAKE_SINCE.search(&flat) {
        let when = text::strip(awake.name_str("when")).to_string();
        return Some(_say("awake", memory, &[("when", &when)]));
    }
    if let Some(chore) = ERRAND.search(&flat) {
        let what = text::strip(chore.name_str("what")).to_string();
        return Some(_say("chore", memory, &[("what", &what)]));
    }
    None
}

/// Python: bot/modules/chat.py:11426 `_the_visit_was_this_week(flat)`
pub fn _the_visit_was_this_week(flat: &str) -> bool {
    if THIS_WEEK.search(flat).is_some() {
        return true;
    }
    LONG_AGO.search(flat).is_none() && n_words(flat) <= 8
}

inline_re!(DOCTORS, r"\bdoctors\b", crate::py::re::I);
inline_re!(NOT_A_TO_Z_AMP, r"[^a-z&]");

/// Python: bot/modules/chat.py:11525 `_a_thing_of_care(phrase, article=True)`
pub fn _a_thing_of_care(phrase: &str, article: bool) -> Vec<(&'static str, String)> {
    let pieces = ANY_SPACE_SPLIT.split(text::strip(phrase));
    let words: Vec<&str> = pieces.iter().map(String::as_str).filter(|w| !w.is_empty()).collect();
    let bare_ = words.iter().filter(|w| !["the", "a", "an"].contains(&w.to_lowercase().as_str())).cloned().collect::<Vec<_>>().join(" ");
    let mut bare_ = DOCTORS.sub("doctor", &bare_);
    if bare_.is_empty() {
        bare_ = "visit".into();
    }
    if ["a&e", "aande", "ande", "ae"].contains(&NOT_A_TO_Z_AMP.sub("", &bare_.to_lowercase()).as_str()) {
        return vec![
            ("X", "A&E".into()),
            ("x", "A&E".into()),
            ("Xb", "A&E".into()),
            ("xb", "A&E".into()),
            ("A", "An".into()),
            ("a", "an".into()),
        ];
    }
    let inside = if article { format!("the {bare_}") } else { bare_.clone() };
    let first = text::slice(&bare_, 0, Some(1)).to_lowercase();
    let low = bare_.to_lowercase();
    let vowel = (!first.is_empty() && "aeiouw".contains(first.as_str()) && !["uni", "one", "once", "use"].iter().any(|p| low.starts_with(p)))
        || ["MRI", "ECG", "EEG", "EKG", "x-ray"].contains(&_keep_the_letters(&bare_).as_str());
    vec![
        ("X", _keep_the_letters(&upper1(&inside))),
        ("x", _keep_the_letters(&inside.to_lowercase())),
        ("Xb", _keep_the_letters(&upper1(&bare_))),
        ("xb", _keep_the_letters(&bare_.to_lowercase())),
        ("A", if vowel { "An" } else { "A" }.into()),
        ("a", if vowel { "an" } else { "a" }.into()),
    ]
}

inline_re!(ANY_SPACE_SPLIT, r"\s+");

/// Python: bot/modules/chat.py:11550 `_ends_plural(phrase)`
pub fn _ends_plural(phrase: &str) -> bool {
    let word = text::split_ref(phrase).last().map(|w| w.to_lowercase()).unwrap_or_default();
    word.ends_with('s') && !(word.ends_with("ss") || word.ends_with("us") || word.ends_with("is"))
}

/// Python: bot/modules/chat.py:11556 `_keep_the_letters(text)`
pub fn _keep_the_letters(text_: &str) -> String {
    let mut out = text_.to_string();
    for (word, letters) in INITIALISMS.pairs() {
        let re = dyn_re(&format!(r"\b{}\b", crate::py::re::escape(word.str())), crate::py::re::I);
        out = re.sub(letters.str(), &out);
    }
    out
}

inline_re!(POSSESSIVE_S, r"'?s$", crate::py::re::I);

/// Python: bot/modules/chat.py:11578 `_wearer_is_themselves(flat, match)`
pub fn _wearer_is_themselves(flat: &str, m: &Match) -> bool {
    // `match.end() - 1`, one character back.
    let at = text::byte_at(flat, text::char_index(flat, m.end()).saturating_sub(1));
    let Some(found) = WEARER.search_at(flat, at) else { return true };
    if found.name("owner").unwrap_or("").to_lowercase() != "my" {
        return false;
    }
    let owner_end = found.name_span("owner").map(|s| s.1).unwrap_or(found.end());
    let rest: Vec<String> = text::split_ref(&flat[owner_end..]).iter().map(|w| text::strip_chars(w, ".,!?").to_string()).take(2).collect();
    if rest.len() >= 2 && SOMEBODY_ELSE.match_(&POSSESSIVE_S.sub("", &rest[0])).is_some() {
        return false;
    }
    !rest.is_empty() && in_set(&A_BODY, &rest[0].to_lowercase())
}

/// Python: bot/modules/chat.py:11596 `_names_a_person(word)`
pub fn _names_a_person(word: &str) -> bool {
    let low = text::strip(word).to_lowercase();
    if low.is_empty() {
        return false;
    }
    if data::get("bot.memory.facts.RELATIONS").contains_str(&low)
        || data::get("bot.memory.facts.KIN_WORDS").contains_str(&low)
        || data::get("bot.memory.facts._BUILT_KIN").contains_str(&low)
    {
        return true;
    }
    SOMEBODY_ELSE.match_(&low).is_some()
}

/// Python: bot/modules/chat.py:11611 `_own_medical_news(flat, memory, clean="")`
pub fn _own_medical_news(flat: &str, memory: &mut Memory, clean: &str) -> Option<String> {
    if _reads_as_distress(if clean.is_empty() { flat } else { clean }) {
        return None;
    }
    if let Some(m) = PART_FINE.search(flat) {
        let part = squash(m.name("part").unwrap_or(""));
        if !part.is_empty()
            && n_words(&part) <= 2
            && !data::get("bot.memory.facts.NOT_A_PRONOUN_NAME").contains_str(&part.to_lowercase())
            && !_names_a_person(&part)
        {
            let last = text::split_ref(&part).last().map(|w| w.to_lowercase()).unwrap_or_default();
            let pool = if REST_WORD.search(flat).is_some() || in_set(&RESTED, &last) { "med_part_fine" } else { "med_part_fine_result" };
            let fields = _a_thing_of_care(&part, false);
            return Some(_say(pool, memory, &kv(&fields)));
        }
    }
    if let Some(m) = PUT_ON.search(flat) {
        if LONG_AGO.search(flat).is_none() && _wearer_is_themselves(flat, &m) {
            let thing = m.name_str("thing");
            let pool = if _A_DRESSING.search(thing).is_some() { "med_dressing" } else { "med_wearing" };
            let fields = _a_thing_of_care(thing, true);
            return Some(_say(pool, memory, &kv(&fields)));
        }
    }
    if SOMEBODY_ELSE.search(flat).is_some() {
        return None;
    }
    if let Some(m) = RESULT_CLEAR.search(flat) {
        let thing = m.name_str("thing");
        let plural = _ends_plural(thing);
        let fields = _a_thing_of_care(thing, !plural);
        return Some(_say(if plural { "med_clear_plural" } else { "med_clear" }, memory, &kv(&fields)));
    }
    if let Some(m) = DOCTOR_SAID.search(flat) {
        let dx_ = squash(m.name("dx").unwrap_or(""));
        let dx = text::strip_chars(&dx_, " .,;!");
        if !dx.is_empty() && !in_set(&NOT_A_DIAGNOSIS, &dx.to_lowercase()) && n_words(dx) <= 3 {
            let fields = _a_thing_of_care(dx, false);
            return Some(_say("med_diagnosis", memory, &kv(&fields)));
        }
    }
    let visit = if NIGHT.search(flat).is_some() { "med_visit_night" } else { "med_visit" };
    if let Some(m) = BACK_FROM_CARE.search(flat) {
        let fields = _a_thing_of_care(m.name_str("place"), true);
        return Some(_say(visit, memory, &kv(&fields)));
    }
    if let Some(m) = WENT_TO_CARE.search(flat) {
        if _the_visit_was_this_week(flat) {
            let fields = _a_thing_of_care(m.name_str("place"), true);
            return Some(_say(visit, memory, &kv(&fields)));
        }
    }
    if let Some(m) = WAS_IN_CARE.search(flat) {
        if _the_visit_was_this_week(flat) {
            let fields = _a_thing_of_care(m.name_str("place"), true);
            return Some(_say(visit, memory, &kv(&fields)));
        }
    }
    if let Some(m) = TAKEN_OUT.search(flat) {
        if LONG_AGO.search(flat).is_none() {
            let fields = _a_thing_of_care(m.name_str("thing"), true);
            return Some(_say("med_out", memory, &kv(&fields)));
        }
    }
    if let Some(m) = GOT_RESULTS.search(flat) {
        let thing = m.name("thing").filter(|t| !t.is_empty()).unwrap_or("results");
        let fields = _a_thing_of_care(thing, true);
        return Some(_say("med_results", memory, &kv(&fields)));
    }
    if let Some(m) = HAD_A_THING_DONE.search(flat) {
        if LONG_AGO.search(flat).is_none() {
            let fields = _a_thing_of_care(m.name_str("thing"), true);
            return Some(_say(visit, memory, &kv(&fields)));
        }
    }
    None
}

inline_re!(NOTHING_ON, r"\bnothing (on|to watch)\b|\bnothing on (the )?(tv|telly)\b");

/// Python: bot/modules/chat.py:11685 `_news(clean, tokens, memory)`
pub fn _news(clean: &str, tokens: &[String], memory: &mut Memory) -> Option<String> {
    if clean.ends_with('?') || !(1..=14).contains(&tokens.len()) {
        return None;
    }
    let better = py(crate::memory::ack::a_person_getting_better_pool(clean, memory));
    if better.truthy() {
        return Some(pick(&better.strs(), memory, clean));
    }
    if truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    if memory.understood.as_ref().is_some_and(|u| u.kind == "question") {
        return None;
    }
    let clean = _as_said(clean, memory.understood.as_ref());
    if crate::compose::loss::reads_as_loss(&clean, None, false) {
        return None;
    }
    if crate::compose::loss::lost_thing_to_find(&clean) {
        return None;
    }
    let replaced = clean.replace('\'', "");
    let flat = text::strip_chars(&replaced, " .,!;:").to_string();
    if let Some(medical) = _own_medical_news(&flat, memory, &clean) {
        if !medical.is_empty() {
            return Some(medical);
        }
    }
    if WISH_LUCK.search(&flat).is_some() {
        return Some(_say("wish_luck", memory, &[]));
    }
    if data::re("bot.memory.facts.CONDITION").search(&flat).is_some() {
        return None;
    }
    if RETIRED.search(&flat).is_some() && RETIRED_ASKING.search(&flat).is_none() {
        return Some(_say("retired", memory, &[]));
    }
    if OUT_OF_WORK.search(&flat).is_some() {
        return Some(_say("out_of_work", memory, &[]));
    }
    if truthy(crate::compose::circumstance::reads_as_situation(&flat)) {
        return None;
    }
    if crate::compose::said_about_me::reads_as_somebody_elses_words(&flat) {
        return None;
    }
    let role_line = AGE_BEFORE_A_ROLE.subn("im ", &flat, 1);
    let (m, is_role) = match ROLE_WORK.search(&role_line) {
        Some(m) => (Some(m), false),
        None => (ROLE.search(&role_line), true),
    };
    if let Some(m) = m {
        let mut phrase = py_str(&py(crate::memory::facts::no_tone(m.str(1))));
        if phrase.is_empty() {
            return None;
        }
        let mut words: Vec<String> = text::split(&phrase);
        if !(!words.is_empty() && words.iter().all(|w| !in_set(&ROLE_STOP, w))) {
            return None;
        }
        if ROLE_TO_PEOPLE.search(&phrase).is_some() || (COMPARED_ROLE.search(&flat).is_some() && is_role) {
            return None;
        }
        let mut rest = if is_role { m.get(2).map(str::to_string) } else { None };
        if let Some(r) = rest.take().filter(|r| !r.is_empty()) {
            let place = _place_of(&r);
            if !place.is_empty() {
                rest = Some(place);
            }
        }
        if let Some(r) = rest.filter(|r| !r.is_empty()) {
            phrase = format!("{phrase} in {}", _tidy(&r));
            words = text::split(&phrase);
        }
        if words.len() <= 5 && !words.iter().any(|w| ["of", "with", "about", "that", "too", "and"].contains(&w.as_str())) {
            let big = upper1(&phrase);
            return Some(_say("job", memory, &[("x", &phrase), ("X", &big)]));
        }
        return None;
    }
    if let Some(m) = PLACE.search(&clean) {
        let said_ = m.str(1).to_string();
        let note = py(crate::memory::facts::qualified_by(&clean));
        let source = if note.truthy() {
            match data::re("bot.memory.facts.TAKE_BACK_SPLIT").search(&said_) {
                Some(cut) => said_[..cut.start()].to_string(),
                None => said_.clone(),
            }
        } else {
            said_.clone()
        };
        let mut place = _place_of(&source);
        if !place.is_empty() && PLACE_SPAN.match_(&text::strip(&place).to_lowercase()).is_some() {
            place.clear();
        }
        let grown = GROWN_IN.search(&flat).is_some() && !truthy(crate::memory::facts::grown_up_in_a_place(&place));
        let low = place.to_lowercase();
        let ordinary = !in_set(&PLACE_NONE, &low) && !in_set(&PLACE_POSITION, &low);
        if note.truthy() && !place.is_empty() && ordinary && !grown {
            let tidy = _tidy(&place);
            let near = py_str(&note);
            return Some(_say("place_near", memory, &[("place", &tidy), ("near", &near)]));
        }
        if !place.is_empty() && ordinary && !grown {
            let tidy = _tidy(&place);
            if MOVING.search(&clean).is_some() {
                return Some(_say("moving", memory, &[("place", &tidy)]));
            }
            return Some(_say("place", memory, &[("place", &tidy)]));
        }
    }
    if TRIP.search(&flat).is_some() {
        return Some(_say("trip", memory, &[]));
    }
    let kids = py(crate::memory::facts::kid_count(&flat));
    if kids.truthy() {
        let n = py_int(&kids);
        let word = get_int_key(data::get("bot.memory.facts.KID_NUMBER_WORDS"), n).map(py_str).unwrap_or_else(|| n.to_string());
        let kids_word = if n == 1 { "kid" } else { "kids" };
        let ages = py(crate::memory::facts::kid_ages(&flat));
        if ages.truthy() && ages.len() as i64 == n {
            let said_ages = _numbers(ages.items());
            let x = format!("{word} {kids_word}, {said_ages}");
            let big = format!("{} {kids_word}, {said_ages}", upper1(&word));
            return Some(_say("kids_aged", memory, &[("x", &x), ("X", &big)]));
        }
        let x = format!("{word} {kids_word}");
        let big = format!("{} {kids_word}", upper1(&word));
        return Some(_say("kids", memory, &[("x", &x), ("X", &big)]));
    }
    if let Some(m) = PET.search(&flat) {
        let pet = m.str(1).to_string();
        // `flat[match.end():match.end() + 20].lstrip()`, in characters.
        let at = text::char_index(&flat, m.end());
        let window = text::slice(&flat, at as isize, Some(at as isize + 20));
        let after = data::re("bot.memory.facts.AFTER_ANIMAL").match_(text::lstrip(&window)).map(|a| a.as_str().to_lowercase());
        let compound = after.is_some_and(|a| data::get("bot.memory.facts.PET_COMPOUND").contains_str(&a));
        if !compound && !truthy(crate::memory::facts::another_animal(&flat, &pet)) {
            let named = _pet_name(&flat, &m);
            if !named.is_empty() {
                let name = _name_said(&named);
                return Some(_say("pet_named", memory, &[("pet", &pet), ("name", &name)]));
            }
            return Some(_say("pet", memory, &[("pet", &pet)]));
        }
    }
    let name = said(crate::memory::facts::pet_name_only(&flat, false))
        .or_else(|| said(crate::memory::facts::pet_name_only(&flat, true)))
        .or_else(|| said(crate::memory::facts::bare_name(&flat)))
        .unwrap_or_default();
    if let Some(named_pet) = _just_named(memory, &name) {
        let tidy = _tidy(&py_str(named_pet.get("name").unwrap_or_else(|| panic!("KeyError: 'name'"))));
        let animal = named_pet.get("animal").map(py_str).unwrap_or_else(|| "pet".into());
        let said_name = _name_said(&tidy);
        return Some(_say("pet_named", memory, &[("pet", &animal), ("name", &said_name)]));
    }
    if LOST_WEIGHT.search(&flat).is_some() {
        let planned = py(crate::memory::goals::the_loss_that_was_the_plan(&flat, memory));
        if planned.truthy() {
            return Some(py_str(&planned));
        }
        return Some(_say("lost_weight", memory, &[]));
    }
    if _a_job_that_went(&flat) {
        return Some(_say("lost_job", memory, &[]));
    }
    if BAD_NEWS.items().iter().any(|p| p.as_re().is_some_and(|re| re.search(&flat).is_some())) {
        return Some(_say("bad_news", memory, &[]));
    }
    if !crate::compose::badnews::reads_as_bad_news(&flat)
        && !crate::compose::goodnews::reads_as_a_win_nobody_noticed(&flat)
        && !crate::compose::goodnews::reads_as_a_prize_in_a_message(&flat)
    {
        if crate::compose::goodnews::reads_as_a_pass(&flat) {
            return Some(_cheer_or_the_question(&flat, memory));
        }
        for pattern in GOOD_NEWS.items() {
            let Some(re) = pattern.as_re() else { continue };
            if let Some(m) = re.search(&flat) {
                if !truthy(crate::compose::negated(&flat, m.as_str(), 3)) && !truthy(crate::compose::doubted(&flat, 4)) {
                    return Some(_cheer_or_the_question(&flat, memory));
                }
            }
        }
    }
    if STATUS.search(&flat).is_some() {
        return Some(_say("status", memory, &[]));
    }
    if NOTHING_ON.search(&flat).is_some() {
        return Some(_say("nothing_on", memory, &[]));
    }
    if CAME_BACK.search(&flat).is_some() {
        if let Some(trip) = _told_a_trip(&flat, memory) {
            return Some(trip);
        }
        return Some(_say("came_back", memory, &[]));
    }
    if let Some(trip) = _told_a_trip(&flat, memory) {
        return Some(trip);
    }
    if let Some(night) = _night_out(&flat, memory) {
        return Some(night);
    }
    if JUST_STARTED.search(&flat).is_some() {
        return Some(_say("just_started", memory, &[]));
    }
    if JUST_DONE.search(&flat).is_some() {
        if truthy(crate::memory::ack::nobody_around_pool(&flat.to_lowercase(), memory)) {
            return None;
        }
        return Some(_say(if A_MOVE.search(&flat).is_some() { "move_done" } else { "just_done" }, memory, &[]));
    }
    None
}

inline_re!(NOT_A_NAME_CHAR, r"[^a-z'-]+");

/// Python: bot/modules/chat.py:11925 `_head_word(phrase)`
pub fn _head_word(phrase: &str) -> String {
    NOT_A_NAME_CHAR.split(&phrase.to_lowercase()).into_iter().filter(|w| !w.is_empty()).last().unwrap_or_default()
}

/// Python: bot/modules/chat.py:11931 `_the_poor_describer(phrase)`
pub fn _the_poor_describer(phrase: &str) -> String {
    let low = phrase.to_lowercase();
    text::split_ref(&low).into_iter().find(|w| in_set(&SAID_POOR, w)).map(str::to_string).unwrap_or_default()
}

/// Python: bot/modules/chat.py:11939 `_the_thing_this_line_names(clean)`
pub fn _the_thing_this_line_names(clean: &str) -> String {
    let Some(m) = VERDICT_THING.search(clean) else { return String::new() };
    let thing = m.name_str("thing");
    let singular = if thing.ends_with('s') { text::slice(thing, 0, Some(-1)) } else { String::new() };
    for word in [thing.to_string(), singular] {
        if in_set(&WATCHED_THINGS, &word) {
            return word;
        }
    }
    String::new()
}

/// `memory[key]["turn"] + 1 == len(turns)` for a note this module keeps (`.get("turn", -9)`).
fn noted_last_turn(note: &Value, memory: &Memory) -> bool {
    let turn = match note.get("turn") {
        None => -9,
        Some(v) => py_int(v),
    };
    memory.turns.len() as i64 == turn + 1
}

/// Python: bot/modules/chat.py:11951 `_about_the_night_out(clean, tokens, memory)`
pub fn _about_the_night_out(clean: &str, tokens: &[String], memory: &mut Memory) -> Option<String> {
    let seen = memory.get("night_out")?.clone();
    if !seen.is_dict() || !seen.get("place").is_some_and(|p| p.truthy()) {
        return None;
    }
    if !noted_last_turn(&seen, memory) {
        return None;
    }
    if clean.ends_with('?') || !(1..=8).contains(&tokens.len()) {
        return None;
    }
    if WENT_TO_SEE.search(clean).is_some()
        || WENT_TO_SEE_COMPANY.search(clean).is_some()
        || WENT_TO.search(clean).is_some()
        || WENT_TO_COMPANY.search(clean).is_some()
    {
        return None;
    }
    let own_thing = _the_thing_this_line_names(clean);
    let named = if own_thing.is_empty() { py_str(seen.get("place").unwrap()) } else { format!("the {own_thing}") };
    let values = _place_values(&named);
    if VERDICT_WELL.search(clean).is_some() {
        return Some(_say("night_out_well", memory, &kv(&values)));
    }
    if WENT_BAD_WEATHER.search(clean).is_some() {
        return Some(_say("night_out_weather", memory, &kv(&values)));
    }
    if VERDICT_POOR.search(clean).is_some() {
        return Some(_say("night_out_poor", memory, &kv(&values)));
    }
    None
}

fn note_night_out(memory: &mut Memory, place: &str) {
    let mut note = Value::dict();
    note.set_key("place", place);
    note.set_key("turn", memory.turns.len());
    memory.set("night_out", note);
}

/// Python: bot/modules/chat.py:11983 `_saw_something(flat, memory)`
pub fn _saw_something(flat: &str, memory: &mut Memory) -> Option<String> {
    let m = WENT_TO_SEE.search(flat).or_else(|| WENT_TO_SEE_COMPANY.search(flat))?;
    let thing_ = squash(m.str(1));
    let thing = text::strip_chars(&thing_, " .,!;:").to_string();
    let mut words: Vec<&str> = text::split_ref(&thing).into_iter().filter(|w| !["the", "a", "an"].contains(w)).collect();
    let mut said_how = 0;
    while words.len() > 1 && in_set(&WATCHED_DESCRIBER, words[0]) {
        words.remove(0);
        said_how += 1;
    }
    if words.is_empty() || !in_set(&WATCHED_THINGS, words[0]) {
        return None;
    }
    if said_how > 0 {
        let named = _tidy(&text::split_ref(&thing).into_iter().filter(|w| !in_set(&WATCHED_DESCRIBER, w)).collect::<Vec<_>>().join(" "));
        note_night_out(memory, &named);
        let poor_describer = _the_poor_describer(&thing);
        let lower = lower1(&named);
        if !poor_describer.is_empty() {
            return Some(_say("saw_it_poor", memory, &[("p", &poor_describer), ("X", &named), ("x", &lower)]));
        }
        return Some(_say("saw_it_told", memory, &[("X", &named), ("x", &lower)]));
    }
    let named = _tidy(&thing);
    let lower = lower1(&named);
    if let Some(poor) = _poor_word(flat).filter(|p| !p.is_empty()) {
        let big = upper1(&poor);
        return Some(_say("night_out_poor_said", memory, &[("p", &poor), ("P", &big), ("X", &named), ("x", &lower)]));
    }
    if WENT_WRONG.search(flat).is_some() || WENT_WELL.search(flat).is_some() {
        return None;
    }
    note_night_out(memory, &named);
    Some(_say("saw_it", memory, &[("X", &named), ("x", &lower)]))
}

fn is_a_place_word(word: &str) -> bool {
    in_set(&KNOWN_PLACES, word)
        || data::get("bot.memory.facts.COUNTRIES").contains_str(word)
        || data::get("bot.knowledge.travel.CITIES").contains_str(word)
}

/// Python: bot/modules/chat.py:12031 `_trip_place(phrase, memory)`
pub fn _trip_place(phrase: &str, memory: &Memory) -> String {
    let name = _place_of(phrase);
    if name.is_empty() {
        return String::new();
    }
    let words = text::split_ref(&name);
    if words.len() > 4 {
        return String::new();
    }
    let low: Vec<String> = words.iter().map(|w| text::strip_chars(&w.to_lowercase(), "-'\u{2019}").to_string()).collect();
    if !low.iter().all(|w| text::is_alpha(w)) {
        return String::new();
    }
    let article = ["the", "a", "an"].contains(&low[0].as_str());
    let bare_ = low.iter().filter(|w| !["the", "a", "an"].contains(&w.as_str())).cloned().collect::<Vec<_>>().join(" ");
    if is_a_place_word(&bare_) {
        return name;
    }
    if article && in_set(&TRIP_SHORT_COUNTRIES, &bare_) {
        return name;
    }
    let typed = memory.understood.as_ref().map(|u| u.text.clone()).unwrap_or_default();
    if text::split_ref(&typed).iter().any(|w| text::is_upper(&text::slice(w, 0, Some(1))) && low.contains(&w.to_lowercase())) {
        return name;
    }
    let shorter = _place_of(&TRIP_TIME_TAIL.sub("", &name));
    if !shorter.is_empty() && is_a_place_word(&shorter.to_lowercase()) {
        return shorter;
    }
    if SOME_WHEN.search(phrase).is_some() {
        if low.len() == 1 && !article {
            return name;
        }
        let short_ = _place_of(&TRIP_TIME_TAIL.sub("", &name));
        let short = text::split_ref(&short_);
        if short.len() == 1 && !["the", "a", "an"].contains(&short[0].to_lowercase().as_str()) {
            return short[0].to_string();
        }
    }
    String::new()
}

/// Python: bot/modules/chat.py:12078 `_told_a_trip(flat, memory)`
pub fn _told_a_trip(flat: &str, memory: &mut Memory) -> Option<String> {
    if TRIP_NOT_YET.search(flat).is_some() {
        return None;
    }
    for (pattern, kind) in [(&*BACK_FROM, "back_from"), (&*BEEN_TO, "been_to"), (&*WENT_TO_ANY, "day_out")] {
        let Some(m) = pattern.search(flat) else { continue };
        let place = _trip_place(m.str(1), memory);
        if place.is_empty() {
            continue;
        }
        if _not_a_night_out(&place) {
            continue;
        }
        let low = place.to_lowercase();
        let bare_: Vec<&str> = text::split_ref(&low).into_iter().filter(|w| !["the", "a", "an"].contains(w)).collect();
        let first = bare_.first().copied().unwrap_or_else(|| panic!("IndexError: list index out of range"));
        if in_set(&NOT_A_PLACE_START, first) {
            continue;
        }
        let named = _tidy(&place);
        note_night_out(memory, &named);
        return Some(_say(kind, memory, &kv(&_place_values(&named))));
    }
    None
}

/// Python: bot/modules/chat.py:12221 `_the_thread_is_heavy_here(memory)`
pub fn _the_thread_is_heavy_here(memory: &Memory) -> bool {
    attempt(|| truthy(crate::compose::fallback::_the_thread_is_heavy(memory))).unwrap_or(false)
}

/// Python: bot/modules/chat.py:12229 `_a_good_time_they_had(flat, memory)` (the second definition,
/// which replaces the first).
pub fn _a_good_time_they_had(flat: &str, memory: &mut Memory) -> Option<String> {
    let m = GOOD_TIME.search(flat)?;
    if _the_thread_is_heavy_here(memory) {
        return None;
    }
    let said_ = format!("a {} {}", m.name_str("good"), m.name_str("thing"));
    let mut note = Value::dict();
    note.set_key("thing", m.name_str("thing"));
    note.set_key("turn", memory.turns.len());
    memory.set("good_time", note);
    let x = _tidy(&said_);
    let options: Vec<String> = GOOD_TIME_LINES.strs().iter().map(|l| py_format(l, &[("X", &x)])).collect();
    Some(pick(&options, memory, ""))
}

inline_re!(TURNS_ROUND, r"\b(?:but|however|though|except|apart from)\b");

/// Python: bot/modules/chat.py:12244 `_the_detail_of_the_good_time(flat, clean, tokens, memory)`
/// (the second definition, which replaces the first).
pub fn _the_detail_of_the_good_time(flat: &str, clean: &str, tokens: &[String], memory: &Memory) -> Option<String> {
    let seen = memory.get("good_time")?;
    if !seen.is_dict() || !seen.get("thing").is_some_and(|t| t.truthy()) {
        return None;
    }
    if !noted_last_turn(seen, memory) {
        return None;
    }
    if clean.ends_with('?') || !(3..=30).contains(&tokens.len()) {
        return None;
    }
    if TURNS_ROUND.search(clean).is_some() {
        return None;
    }
    if _the_thread_is_heavy_here(memory) {
        return None;
    }
    if BAD_NEWS.items().iter().any(|p| p.as_re().is_some_and(|re| re.search(flat).is_some())) {
        return None;
    }
    let thing = py_str(seen.get("thing").unwrap());
    let article = if "aeiou".contains(&text::slice(&thing, 0, Some(1)) as &str) { "an" } else { "a" };
    let options: Vec<String> = DETAIL_LINES.strs().iter().map(|l| py_format(l, &[("thing", &thing), ("article", article)])).collect();
    Some(pick(&options, memory, ""))
}

/// Python: bot/modules/chat.py:12266 `_night_out(flat, memory)`
pub fn _night_out(flat: &str, memory: &mut Memory) -> Option<String> {
    let tokens = text::split(flat);
    if let Some(detail) = _the_detail_of_the_good_time(flat, flat, &tokens, memory) {
        return Some(detail);
    }
    if let Some(good_time) = _a_good_time_they_had(flat, memory) {
        return Some(good_time);
    }
    if let Some(detail) = _the_detail_of_the_good_time(flat, flat, &tokens, memory) {
        return Some(detail);
    }
    if let Some(good_time) = _a_good_time_they_had(flat, memory) {
        return Some(good_time);
    }
    if let Some(seen) = _saw_something(flat, memory) {
        return Some(seen);
    }
    let m = WENT_TO.search(flat).or_else(|| WENT_TO_COMPANY.search(flat))?;
    let place_ = squash(m.str(1));
    let place = text::strip_chars(&place_, " .,!;:").to_string();
    let words = text::split_ref(&place);
    if place.is_empty() || ["my", "our", "his", "her", "their"].contains(&words[0]) {
        return None;
    }
    if _not_a_night_out(&place) {
        return None;
    }
    let bare_: Vec<&str> = words.iter().copied().filter(|w| !["the", "a", "an"].contains(w)).collect();
    let first = bare_.first().copied().unwrap_or_else(|| panic!("IndexError: list index out of range"));
    if in_set(&NOT_A_PLACE_START, first) {
        return None;
    }
    let named = _tidy(&place);
    let values = vec![("X", named.clone()), ("x", lower1(&named))];
    let poor_describer = _the_poor_describer(&place);
    if !poor_describer.is_empty() && _poor_word(flat).is_none_or(|p| p.is_empty()) {
        let bare_place = _tidy(&words.iter().copied().filter(|w| !in_set(&SAID_POOR, w)).collect::<Vec<_>>().join(" "));
        if !bare_place.is_empty() {
            note_night_out(memory, &bare_place);
            let mut fields = vec![("p", poor_describer.clone())];
            fields.extend(_place_values(&bare_place));
            return Some(_say("day_out_poor", memory, &kv(&fields)));
        }
    }
    if let Some(poor) = _poor_word(flat).filter(|p| !p.is_empty()) {
        let mut fields = vec![("p", poor.clone()), ("P", upper1(&poor))];
        fields.extend(values.clone());
        return Some(_say("night_out_poor_said", memory, &kv(&fields)));
    }
    if WENT_WRONG.search(flat).is_some() {
        return None;
    }
    note_night_out(memory, &named);
    if WENT_WELL.search(flat).is_some() {
        return Some(_say("day_out_well", memory, &kv(&values)));
    }
    Some(_say("day_out", memory, &kv(&values)))
}

/// Python: bot/modules/chat.py:12333 `_a_pick_is_in_hand(memory)`
pub fn _a_pick_is_in_hand(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said_ = format!("{} {}", last.user, last.bot).to_lowercase();
    az_words(&said_).iter().any(|w| in_set(&PICK_WORDS, w))
}

/// Python: bot/modules/chat.py:12350 `_asks_for_a_joke(clean, tokens, memory=None)`
pub fn _asks_for_a_joke(clean: &str, tokens: &[String], memory: &Memory) -> bool {
    if WANTS_A_THING.search(clean).is_some() && ASKS_ALOUD.search(clean).is_none() && _a_pick_is_in_hand(memory) {
        return false;
    }
    if JOKE_ADVICE.search(clean).is_some() {
        return false;
    }
    if SELF_JOKE.search(clean).is_some() {
        return false;
    }
    if _fond_of_the_bot(clean) {
        return false;
    }
    if JOKE_ASKED.search(clean).is_some() {
        return true;
    }
    if RUNTIME_LIMIT.search(clean).is_some() {
        return false;
    }
    if JOKE_ASK.search(clean).is_none() {
        return false;
    }
    !tokens.iter().any(|t| in_set(&PICK_WORDS, t))
}

// ---------------------------------------------------------------------------------------------
// Python lines 12510-15620: their thing, jokes, standing lengths, repairs, respond, follow-ups.

res!(
    CORRECTION_TAIL, NEGATED_TAIL, PAIN_PART, ALWAYS, DECISION, STATE, RELATIVE, LEAVE,
    _A_NEED_ASK, PERSON_WORK, _PERSON_IN_THE_TAIL, INTENTION, WANT, QUESTION, THING_SKIP, KEEPS,
    KEEP_ABOUT_SOMEONE, A_THOUGHT_THAT_IS_NOT_A_THING, A_DOUBT_ABOUT_IT, A_THOUGHT_ABOUT_OTHERS,
    A_HOPELESS_THOUGHT, ASK_TAIL, MEDICINE, KEEPS_SYMPTOM, TIME_TAIL, STOPS_TOGETHER,
    REPORTS_A_JOKE, STANDING_LINE, A_THING_THAT_BREAKS, ASKED_HOW_LONG, TIRED_SAID, DONT_KNOW,
    CHAT_SCATTER, CHAT_SCATTER_ASKS, CHAT_SCATTER_CTX, CHAT_SCATTER_NOT, SAID_WRONG,
    SAID_WRONG_NOT, SAID_WRONG_LOST, SAID_WRONG_TO_AN_ANIMAL, SAID_WRONG_TO_A_PROFESSIONAL, I_SAID,
    CLAUSE_BOUNDARY, FIRST_PERSON_DEED, SAID_UNKINDLY, FEELING_CLAUSE, A_VERDICT,
    A_DOES_SOMETHING, A_STATE_WORD, A_REASSURES, A_NEW_CONDITION, A_ASKS_WHERE,
    A_WHEN_TO_GET_HELP, ASKS_A_PRICE, A_REPEAT_FRAME, A_STATE_ASK, ASKS_A_FIGURE,
    EVERY_YES_NO_OPEN, DISTRESS, TIME_ASK, TOPIC_WORD, QUESTION_OPEN, YES_NO_OPEN, OFFERING,
    OWN_THING, LOOKING_FOR, FOUND_THING, AN_ARTICLE, ASKS_ABOUT_A_THING, ASKS_WHAT_TO_DO_WITH,
    A_REPEAT_LEAD, WORK_PERSON, WORK_FAULT, WORK_ASK, WORK_UNSEEN, WORK_CONTEXT, WORK_RELATIVE,
    LAUGH_REPORT, LAUGH_TOGETHER, LAUGH_TOGETHER_NOT, LAUGH_MINE, LAUGH_AT_ME,
    LAUGH_NOT_A_PERSON, LAUGH_BAD_SETTING, LAUGH_CARRIED_ON, LAUGH_CARRIED_ON_NOT,
    SOMEBODY_THINKS, EVERYONE_THINKS, THEIR_OPINION_IS_A_HEALTH_MATTER, A_REPORT_A_THING,
    A_HEARD_A_THING, A_PRAISING_VERB, A_ASKED_NOT_TOLD, A_PRAISE_OF_A_THING,
    A_RECOMMENDATION_IS_NOT, _NO_REASON,
);
sets!(
    FILLER_TAIL, INDEFINITE, INDEFINITE_SKIP, DOING, ONE_OFF, CONTINUING, BARE_FEELING,
    NOT_A_GOOD, NOT_THE_THING, _A_PERSON_ALREADY_IN_THE_LINE, NOT_A_THING_WITH_KINDS,
    GENERIC_THING_WORDS, _HOW_STOP, KEEP_INTENT, KEEP_A_BELIEF, NEED_NOW, SAYS_NOTHING,
    _NOT_A_SUBJECT, DONT_KNOW_FILLER, CHAT_SCATTER_LEFTOVER, DEEDS_WITH_A_CARD, A_VERDICT_FILLER,
    HEARD_NOTHING, A_CONTAINER,
);
vals!(
    FILLER_TAIL_PHRASES, ASK_KIND, TASK_ONLY, KINDLESS, A_BREAK_FROM_SOMEONE, CHOOSING,
    THING_REPLIES, _VERB_ENDINGS, STANDING_WHAT_REPLIES, STANDING_REPLIES,
    STANDING_THING_REPLIES, STANDING_FEELING_REPLIES, CHAT_SCATTER_REPLIES, CHAT_SCATTER_ALONE,
    CHAT_ASKING_REPLIES, CHAT_ASKING_ALONE, FORGOT_DATE_REPLIES, LEFT_IT_TOO_LONG_REPLIES,
    DEED_REPLIES, FORGOT_DATE_WORDS, DEED_WORDS, NOT_AN_ANSWER, YES_NO_WORDS, A_SHORT_LINE,
    MEANT_LEADS, WORK_GRUMBLE, WORK_UNSEEN_REPLY, LAUGH_CARRIED_ON_REPLY, THINKS_REPLIES,
);

/// Python: bot/modules/chat.py:12515 `_without_a_filler_tail(thing)`
pub fn _without_a_filler_tail(thing: &str) -> String {
    let mut said = squash(thing);
    let (head0, _, tail) = text::partition(&said, ",");
    let head = text::strip_chars(head0, " ,;:").to_string();
    let tail = tail.to_string();
    if !tail.is_empty() {
        let fixed = CORRECTION_TAIL.match_(&tail).map(|f| f.name_str("t").to_string());
        if NEGATED_TAIL.match_(&tail).is_some() {
            said = if !head.is_empty() { head } else { fixed.unwrap_or(said) };
        } else if let Some(f) = fixed {
            said = f;
        } else if !head.is_empty() {
            said = head;
        }
    }
    let mut words: Vec<String> = text::split(&said);
    let mut changed = true;
    while changed && words.len() > 1 {
        changed = false;
        for phrase in FILLER_TAIL_PHRASES.items() {
            let phrase = phrase.strs();
            if words.len() > phrase.len()
                && words[words.len() - phrase.len()..]
                    .iter()
                    .zip(phrase.iter())
                    .all(|(w, want)| text::strip_chars(&w.to_lowercase(), ".!?") == want)
            {
                let keep = words.len() - phrase.len();
                words.truncate(keep);
                changed = true;
                break;
            }
        }
    }
    while words.len() > 1 && in_set(&FILLER_TAIL, text::strip_chars(&words[words.len() - 1].to_lowercase(), ".!?")) {
        words.pop();
    }
    words.join(" ")
}

/// Python: bot/modules/chat.py:12670 `_kindless(word)`
pub fn _kindless(word: &str) -> bool {
    let w = text::strip_chars(&word.to_lowercase(), " .,!?;:'\u{2019}\"()").to_string();
    if in_set(&NOT_A_THING_WITH_KINDS, &w) {
        return true;
    }
    if in_set(&NOT_A_GOOD, &w) || in_set(&NOT_THE_THING, &w) {
        return true;
    }
    if crate::modules::particles::a_job_word(&w) {
        return true;
    }
    if w.ends_with("ies") && in_set(&NOT_A_GOOD, &format!("{}y", &w[..w.len() - 3])) {
        return true;
    }
    for tail in ["es", "s"] {
        if w.ends_with(tail) && in_set(&NOT_A_GOOD, &w[..w.len() - tail.len()]) {
            return true;
        }
    }
    truthy(crate::understanding::breeds::the_kind_of(&w))
}

/// Python: bot/modules/chat.py:12753 `_a_death_just_said(memory)`
pub fn _a_death_just_said(memory: &Memory) -> bool {
    for turn in last_n(turns(memory), 2) {
        let said_ = &turn.user;
        if text::strip(said_).is_empty() {
            continue;
        }
        if !crate::compose::loss::reads_as_loss(said_, None, false) {
            continue;
        }
        if crate::compose::urgent::reads_as_a_story(&crate::compose::loss::_spoken(said_)) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/modules/chat.py:12779 `_in_loss(memory)`
pub fn _in_loss(memory: &Memory) -> bool {
    last_n(turns(memory), 6).iter().any(|t| crate::compose::loss::reads_as_loss(&t.user, None, false))
}

/// Python: bot/modules/chat.py:12788 `_answers_knows(text, memory)`: asks `answers.respond`, which
/// writes the memory it is handed, as Python's does.
pub fn _answers_knows(text_: &str, memory: &mut Memory) -> bool {
    attempt(|| crate::modules::answers::respond(text_, memory).is_some_and(|r| !text::strip(&r).is_empty())).unwrap_or(false)
}

/// Python: bot/modules/chat.py:12819 `_verb_stem(word)`
pub fn _verb_stem(word: &str) -> String {
    let mut word = text::strip_chars(&word.to_lowercase(), "'").to_string();
    if word.ends_with("ied") {
        word = format!("{}y", &word[..word.len() - 3]);
    }
    for ending in _VERB_ENDINGS.strs() {
        if text::len(&word) > text::len(&ending) + 2 && word.ends_with(ending.as_str()) {
            word.truncate(word.len() - ending.len());
            break;
        }
    }
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    if n > 3 && chars[n - 1] == chars[n - 2] && !"aeiou".contains(chars[n - 1]) {
        word = chars[..n - 1].iter().collect();
    }
    word
}

/// Python: bot/modules/chat.py:12833 `_answers_is_about_the_thing(text, thing, memory, verbs=())`
pub fn _answers_is_about_the_thing(text_: &str, thing: &str, memory: &Memory, verbs: &[String]) -> bool {
    attempt(|| {
        let Some(entry) = crate::knowledge::lookup::find(&crate::understanding::understand(text_, Some(memory)), None, None, false) else {
            return false;
        };
        let mut said_: HashSet<String> = HashSet::new();
        for part in &entry.key {
            for piece in part.words() {
                said_.extend(az_words(&piece.to_lowercase()));
            }
        }
        let line = format!(" {} ", squash(&text_.to_lowercase()));
        for part in &entry.key {
            for piece in part.words() {
                let piece = squash(&piece.to_lowercase());
                if piece.matches(' ').count() >= 2 && line.contains(&piece) {
                    return true;
                }
            }
        }
        let stems: HashSet<String> = verbs.iter().filter(|v| !v.is_empty()).map(|v| _verb_stem(v)).collect();
        for word in az_words(&thing.to_lowercase()) {
            if in_set(&GENERIC_THING_WORDS, &word) || stems.contains(&_verb_stem(&word)) {
                continue;
            }
            if said_.contains(&word) || said_.contains(&text::slice(&word, 0, Some(-1))) || said_.contains(&format!("{word}s")) {
                return true;
            }
        }
        false
    })
    .unwrap_or(false)
}

/// Python: bot/modules/chat.py:12887 `_answers_has_the_how(thing, memory)`
pub fn _answers_has_the_how(thing: &str, memory: &Memory) -> bool {
    let _ = memory;
    attempt(|| {
        let Some(found) = crate::knowledge::lookup::find(&crate::understanding::understand(&format!("how do i {thing}"), None), None, None, false) else {
            return false;
        };
        let content: HashSet<String> =
            az_words(&thing.to_lowercase()).into_iter().filter(|w| !in_set(&_HOW_STOP, w) && text::len(w) > 1).collect();
        let keys = crate::knowledge::lookup::key_words(found);
        !content.is_empty() && content.iter().all(|w| keys.contains(w))
    })
    .unwrap_or(false)
}

/// Python: bot/modules/chat.py:12908 `_fallback_has_its_own_pool(text)`
pub fn _fallback_has_its_own_pool(text_: &str) -> bool {
    let plain = squash(&text_.to_lowercase());
    attempt(|| {
        if crate::compose::loss::keeps_losing_thing(&plain) {
            return true;
        }
        for name in ["FORGET_A_DATE", "FORGET_A_SECRET", "FORGETS_A_CLASH", "FORGETS_AGAIN"] {
            if data::re(&format!("bot.compose.fallback.{name}")).search(&plain).is_some() {
                return true;
            }
        }
        crate::compose::fallback::_is_rumination(&plain)
    })
    .unwrap_or(false)
}

/// Python: bot/modules/chat.py:12931 `_the_words_they_meant(clean, understood)`
pub fn _the_words_they_meant(clean: &str, understood: Option<&Understood>) -> String {
    let plain = understood.map(|u| squash(&u.plain.to_lowercase().replace('\u{2019}', "'"))).unwrap_or_default();
    if plain.is_empty() || plain == clean {
        return clean.to_string();
    }
    let (typed, meant) = (text::split_ref(clean), text::split_ref(&plain));
    if typed.len() != meant.len() {
        return clean.to_string();
    }
    for (was, is_) in typed.iter().zip(meant.iter()) {
        if was == is_ {
            continue;
        }
        if *was == "2" || *was == "4" || was.starts_with(is_) || is_.starts_with(was) {
            continue;
        }
        return clean.to_string();
    }
    plain
}

inline_re!(SO_THAT, r"\bso that\b|\bso (?:i|we) (?:can|could|will|would)\b");
inline_re!(THING_CLAUSE_CUT, r"\b(?:because|but|so|though|although)\b");
inline_re!(SPEAKER_IN_THING, r"\b(?:i|i'm|im|i've|i'd|we|we're|us)\b");
inline_re!(PURPOSE_WORDS, r"\b(?:for|on|in|at|with|to|about|from)\b");

/// Python: bot/modules/chat.py:12961 `_their_thing(clean, tokens, memory, understood=None)`
pub fn _their_thing(clean: &str, tokens: &[String], memory: &mut Memory, understood: Option<&Understood>) -> Option<String> {
    let clean = _the_words_they_meant(clean, understood);
    let clean = clean.as_str();
    if clean.ends_with('?') || !(3..=16).contains(&tokens.len()) {
        return None;
    }
    if QUESTION.match_(clean).is_some() || THING_SKIP.search(clean).is_some() {
        return None;
    }
    if MEDICINE.search(clean).is_some() {
        return None;
    }
    if crate::compose::loss::reads_as_loss(clean, None, false) || _in_loss(memory) {
        return None;
    }
    if crate::compose::harm::reads_as_hurting_someone(clean) {
        return opt_str(crate::compose::harm::reply_for(clean, memory));
    }
    if let Some(quoted) = said(crate::compose::saidby::reply_for(clean, memory)) {
        return Some(quoted);
    }
    if crate::compose::fallback_chain::_the_morning_has_a_clock(clean) {
        return None;
    }
    if data::re("bot.memory.ack.WANTS_ANSWER").search(clean).is_some() || ASK_TAIL.search(clean).is_some() {
        return None;
    }
    let padded = format!(" {clean} ");
    if padded.contains(" but ") {
        return None;
    }
    if padded.contains(" without ") {
        return None;
    }
    let flat = text::strip_chars(&clean.replace('\u{2019}', "'"), " .,!;:").to_string();
    let mut verbs: Vec<String> = Vec::new();
    for (kind0, pattern) in [("intention", &*INTENTION), ("want", &*WANT), ("always", &*ALWAYS), ("keeps", &*KEEPS)] {
        let Some(m) = pattern.search(&flat) else { continue };
        let mut kind = kind0;
        let mut thing: String;
        if kind == "keeps" {
            let said_ = text::strip(m.get(2).unwrap_or("")).to_string();
            if in_set(&KEEP_A_BELIEF, &m.str(1).to_lowercase()) && A_THOUGHT_THAT_IS_NOT_A_THING.match_(&said_).is_some() {
                if A_HOPELESS_THOUGHT.search(&flat).is_some() {
                    return Some(pick(&replies("not_coping"), memory, ""));
                }
                if A_THOUGHT_ABOUT_OTHERS.match_(&said_).is_some() || A_DOUBT_ABOUT_IT.search(&said_).is_some() {
                    return None;
                }
            }
            if in_set(&KEEP_INTENT, &m.str(1).to_lowercase()) {
                kind = "want";
                thing = text::strip(m.get(2).unwrap_or("")).to_string();
            } else {
                verbs = vec![m.str(1).to_string()];
                thing = text::strip(&format!("{} {}", m.str(1), m.get(2).unwrap_or(""))).to_string();
            }
        } else {
            thing = text::strip(m.str(1)).to_string();
        }
        if SO_THAT.search(&flat).is_some() {
            return None;
        }
        let cut = match THING_CLAUSE_CUT.search(&thing) {
            Some(c) => thing[..c.start()].to_string(),
            None => thing.clone(),
        };
        thing = text::strip_chars(&cut, " ,;:").to_string();
        thing = text::strip_chars(&TIME_TAIL.sub("", &thing), " ,;:").to_string();
        let first = text::split_ref(&thing).first().copied().unwrap_or("").to_string();
        if in_set(&BARE_FEELING, &thing.to_lowercase()) {
            return Some(pick(&replies("not_coping"), memory, ""));
        }
        if in_set(&FEELING_THING, &first)
            || FEELING_WANT.search(&thing).is_some()
            || LEAVE_SOMEONE.search(&thing).is_some()
            || in_set(&CONTINUING, &thing.to_lowercase())
        {
            return None;
        }
        if let Some(stopping) = STOPS_TOGETHER.match_(&thing) {
            if in_set(&SAYS_NOTHING, &squash(stopping.name_str("subject")).to_lowercase()) {
                return Some(pick(&replies("not_coping"), memory, ""));
            }
            return None;
        }
        thing = _without_a_filler_tail(&thing);
        if SPEAKER_IN_THING.search(&thing).is_some() {
            return None;
        }
        let words: Vec<String> = text::split(&thing);
        if !(1..=8).contains(&words.len()) {
            return None;
        }
        if ["go", "be", "do", "get", "have", "try", "say", "know", "see", "it"].contains(&words[0].as_str()) && words.len() <= 2 {
            return None;
        }
        if LEAVE.search(clean).is_some() {
            return None;
        }
        if (kind == "want" || kind == "need") && _A_NEED_ASK.search(clean).is_some() {
            return None;
        }
        if truthy(crate::memory::facts::a_limit_on_the_plan(clean)) {
            return None;
        }
        let indefinite = in_set(&INDEFINITE, &words[0]);
        let problem = if indefinite {
            words[1..].iter().filter(|w| !in_set(&INDEFINITE_SKIP, w)).cloned().collect::<Vec<_>>().join(" ")
        } else {
            thing.clone()
        };
        if indefinite {
            let knows = _answers_knows(clean, memory)
                || ((SYMPTOM.search(&problem).is_some()
                    || PAIN_PART.search(&problem).is_some()
                    || SYMPTOM.search(&thing).is_some()
                    || PAIN_PART.search(&thing).is_some())
                    && (_answers_knows(&format!("what helps {problem}"), memory)
                        || _answers_knows(&format!("what helps {thing}"), memory)));
            if knows {
                return None;
            }
        }
        if indefinite && SYMPTOM.search(&problem).is_some() {
            let problem_read = crate::understanding::understand(&problem, Some(memory));
            if let Some(part) = problem_read.body.filter(|b| !b.is_empty()) {
                let kind_ = format!("ill_{part}");
                if REPLIES.get(&kind_).is_some() {
                    let name = _name(memory);
                    return Some(_reply(&kind_, memory, name.as_deref(), clean));
                }
            }
        }
        if let Some(someone) = _PERSON_IN_THE_TAIL.search(clean) {
            let who = someone.name_str("who").to_string();
            if !_answers_knows(clean, memory) {
                let said_ = crate::memory::thread::your_words(&thing);
                let line = pick(&A_BREAK_FROM_SOMEONE.strs(), memory, "");
                return Some(py_format(&line, &[("X", &upper1(&said_)), ("them", &who)]));
            }
        }
        if indefinite {
            let rest: Vec<&String> = words[1..].iter().filter(|w| !in_set(&INDEFINITE_SKIP, w)).collect();
            let said_ = crate::memory::thread::your_words(&thing);
            let line = pick(&ASK_KIND.strs(), memory, "");
            let adj = rest.first().map(|s| s.as_str()).unwrap_or("thing");
            return Some(py_format(&line, &[("x", &upper1(&said_)), ("adj", adj)]));
        }
        if (kind == "want" || kind == "need") && words.iter().any(|w| in_set(&NEED_NOW, &w.to_lowercase())) {
            return None;
        }
        if (kind == "keeps" || kind == "always") && KEEPS_SYMPTOM.search(&thing).is_some() {
            return None;
        }
        if (kind == "keeps" || kind == "always") && _answers_knows(clean, memory) {
            return None;
        }
        if (kind == "keeps" || kind == "always") && _fallback_has_its_own_pool(clean) {
            return None;
        }
        if ["want", "keeps", "always", "intention"].contains(&kind)
            && _answers_is_about_the_thing(clean, &thing, memory, &verbs)
            && _answers_knows(clean, memory)
        {
            return None;
        }
        if ["a", "an", "some", "the", "my", "another", "one"].contains(&words[0].as_str()) {
            let joined = words.join(" ");
            let first_piece = match PURPOSE_WORDS.search(&joined) {
                Some(p) => joined[..p.start()].to_string(),
                None => joined.clone(),
            };
            let head = text::split_ref(&first_piece).last().map(|h| h.to_string()).unwrap_or_else(|| panic!("IndexError"));
            if PERSON_WORK.search(&head).is_some() || RELATIVE.search(&thing).is_some() || _answers_knows(clean, memory) {
                return None;
            }
            let said_ = crate::modules::particles::a_thing_without_its_tail(&crate::memory::thread::your_words(&thing));
            let label = upper1(&said_);
            if in_set(&_A_PERSON_ALREADY_IN_THE_LINE, &head.to_lowercase()) {
                return None;
            }
            if _kindless(&head) {
                let options: Vec<String> = KINDLESS.strs().iter().map(|o| py_format(o, &[("x", &said_), ("X", &label)])).collect();
                return Some(pick(&options, memory, ""));
            }
            let options: Vec<String> = CHOOSING.strs().iter().map(|o| py_format(o, &[("X", &label), ("head", &head)])).collect();
            return Some(pick(&options, memory, ""));
        }
        if data::re("bot.memory.ack.DEATH_WORDS").search(clean).is_some() {
            return None;
        }
        if STATE.search(&thing).is_some() || in_set(&ONE_OFF, &words[0]) {
            return None;
        }
        if _answers_has_the_how(&thing, memory) && _answers_knows(clean, memory) {
            return None;
        }
        let mut pool = THING_REPLIES.get(kind).unwrap_or_else(|| panic!("KeyError: {kind:?}")).strs();
        if kind == "keeps" && KEEP_ABOUT_SOMEONE.search(clean).is_some() {
            pool = THING_REPLIES.get("keeps_someone").unwrap_or_else(|| panic!("KeyError: 'keeps_someone'")).strs();
        }
        if DECISION.search(&thing).is_some() || (kind == "want" && !in_set(&DOING, &words[0])) {
            let bad = TASK_ONLY.strs();
            pool.retain(|o| !bad.iter().any(|b| o.contains(b.as_str())));
        }
        if pool.is_empty() {
            return None;
        }
        let said_ = crate::memory::thread::your_words(&thing);
        let big = upper1(&said_);
        let options: Vec<String> = pool.iter().map(|o| py_format(o, &[("x", &said_), ("X", &big)])).collect();
        return Some(pick(&options, memory, ""));
    }
    None
}

/// Python: bot/modules/chat.py:13250 `_asks_outright(clean)`
pub fn _asks_outright(clean: &str) -> bool {
    JOKE_ASK.search(clean).is_some() || text::strip(clean).starts_with("joke")
}

/// Python: bot/modules/chat.py:13261 `_joined_read(memory)`
pub fn _joined_read(memory: &Memory) -> bool {
    memory.get("joined_read").is_some_and(|v| v.truthy())
}

inline_re!(JOKE_ABOUT, r"\babout\s+(?:(?:a|an|the|my|your|his|her|their|our|its)\s+)?([a-z]+)");

/// Python: bot/modules/chat.py:13274 `_joke(clean, tokens, memory)`
pub fn _joke(clean: &str, tokens: &[String], memory: &Memory) -> Option<String> {
    if truthy(crate::understanding::kinds::verdicts_a_thing(clean)) {
        return None;
    }
    if _joined_read(memory) {
        return None;
    }
    let asked_by_name = _asks_for_a_joke(clean, tokens, memory);
    if !_asks_outright(clean) && REPORTS_A_JOKE.search(clean).is_some() {
        return None;
    }
    if !_asks_outright(clean) && memory.get("loss").is_some_and(|l| l.truthy()) {
        return None;
    }
    let after_joke = _last_was_joke(memory)
        && tokens.len() <= 8
        && tokens.iter().any(|t| ["another", "more", "next", "again", "else", "any", "got", "one"].contains(&t.as_str()));
    let joke_word = tokens.iter().any(|t| t == "joke")
        && tokens.len() <= 10
        && !(JOKE_ADVICE.search(clean).is_some() || tokens.iter().any(|t| in_set(&PICK_WORDS, t)))
        && SELF_JOKE.search(clean).is_none();
    if !(asked_by_name || after_joke || joke_word) {
        return None;
    }
    let mut subject: Option<String> = None;
    if let Some(m) = JOKE_ABOUT.search(clean) {
        let s = text::rstrip_chars(m.str(1), "s").to_string();
        subject = if in_set(&_NOT_A_SUBJECT, &s) { None } else { Some(s) };
    }
    let words: HashSet<String> = tokens.iter().map(|t| text::rstrip_chars(t, "s").to_string()).collect();
    let subject_text = subject.clone().unwrap_or_default();
    for (key, joke) in JOKES.pairs() {
        if words.contains(key.str()) || subject_text.contains(key.str()) {
            return Some(py_str(joke));
        }
    }
    for (key, joke) in JOKES.pairs() {
        if dyn_re(&format!(r"\b{}s?\b", key.str()), 0).search(clean).is_some() {
            return Some(py_str(joke));
        }
    }
    if let Some(subject) = subject.filter(|s| !s.is_empty()) {
        return Some(format!("I don't know a good {subject} one, sorry. {}", pick(&SHORT_JOKES.strs(), memory, "")));
    }
    Some(pick(&SHORT_JOKES.strs(), memory, ""))
}

/// Python: bot/modules/chat.py:13319 `_normalised_line(memory)`
pub fn _normalised_line(memory: &Memory) -> Option<String> {
    let u = memory.understood.as_ref()?;
    if u.norm.is_empty() || u.norm == u.words {
        return None;
    }
    Some(u.norm.join(" "))
}

/// Python: bot/modules/chat.py:13505 `_a_constant_tiredness(clean)`
pub fn _a_constant_tiredness(clean: &str) -> bool {
    TIRED_SAID.search(clean).is_some() && TIRED_PERSISTENT.search(clean).is_some()
}

inline_re!(LEN_DAYS, r"\b(?:days?)\b");
inline_re!(LEN_WEEKS, r"\bweeks?\b");
inline_re!(LEN_MONTHS, r"\bmonths?\b");
inline_re!(LEN_SINCE, r"\bsince\b");
inline_re!(
    LEN_LONG_SINCE,
    r"\b(?:year|years|christmas|summer|winter|spring|autumn|january|february|march|april|may|june|july|august|september|october|november|december)\b"
);

/// `pool[bucket]` from one of the standing tables.
fn bucket_pool(table: &Value, bucket: &str) -> Vec<String> {
    table.get(bucket).unwrap_or_else(|| panic!("KeyError: {bucket:?}")).strs()
}

/// Python: bot/modules/chat.py:13510 `_standing(clean, memory)`
pub fn _standing(clean: &str, memory: &Memory) -> Option<String> {
    let m = STANDING_LINE.match_(clean)?;
    let length = squash(m.as_str());
    let bucket = if LEN_DAYS.search(&length).is_some() {
        "days"
    } else if LEN_WEEKS.search(&length).is_some() {
        "weeks"
    } else if LEN_MONTHS.search(&length).is_some() {
        "months"
    } else if LEN_SINCE.search(&length).is_some() && LEN_LONG_SINCE.search(&length).is_none() {
        "days"
    } else {
        "long"
    };
    let turns = turns(memory);
    let last_bot = turns.iter().rev().find(|t| !t.bot.is_empty()).map(|t| t.bot.clone()).unwrap_or_default();
    let said_ = match memory.get("topic") {
        Some(topic) if topic.is_dict() => py_str(topic.get("text").unwrap_or(&Value::None)),
        _ => String::new(),
    };
    let mut subjects: Vec<String> = if said_.is_empty() { Vec::new() } else { vec![said_] };
    for turn in turns.iter().rev() {
        if !turn.user.is_empty() {
            subjects.push(turn.user.clone());
            if subjects.len() >= 3 {
                break;
            }
        }
    }
    let mut about = String::new();
    for subject in &subjects {
        if !subject.is_empty()
            && (_feeling(subject, None, None, false).is_some_and(|f| !f.is_empty())
                || _stuck(subject, &NO_MEMORY).is_some_and(|s| !s.is_empty())
                || SYMPTOM.search(subject).is_some()
                || crate::compose::badnews::reads_like_a_symptom(subject)
                || _a_constant_tiredness(subject))
        {
            about = subject.clone();
            break;
        }
    }
    if about.is_empty() && A_THING_THAT_BREAKS.search(&subjects.join(" ")).is_some() {
        return Some(pick(&bucket_pool(&STANDING_THING_REPLIES, bucket), memory, ""));
    }
    if !(ASKED_HOW_LONG.search(&last_bot).is_some() || !about.is_empty()) {
        return Some(pick(&bucket_pool(&STANDING_WHAT_REPLIES, bucket), memory, ""));
    }
    if !about.is_empty() && _feeling(&about, None, None, false).is_some_and(|f| !f.is_empty()) && STANDING_FEELING_REPLIES.get(bucket).is_some() {
        return Some(pick(&bucket_pool(&STANDING_FEELING_REPLIES, bucket), memory, ""));
    }
    Some(pick(&bucket_pool(&STANDING_REPLIES, bucket), memory, ""))
}

/// Python: bot/modules/chat.py:13572 `_repaired_line(memory)`
pub fn _repaired_line(memory: &Memory) -> Option<String> {
    let u = memory.understood.as_ref()?;
    if text::strip(&u.plain).is_empty() {
        return None;
    }
    Some(u.plain.clone())
}

/// Python: bot/modules/chat.py:13603 `_says_dont_know(clean, tokens)`
pub fn _says_dont_know(clean: &str, tokens: &[String]) -> bool {
    if tokens.is_empty() || tokens.len() > 12 {
        return false;
    }
    if DONT_KNOW.search(clean).is_none() {
        return false;
    }
    tokens.iter().all(|t| in_set(&DONT_KNOW_FILLER, t))
}

/// Python: bot/modules/chat.py:13612 `claims_instead(text, memory)`: the arbiter's
/// `CLAIMS_INSTEAD` hook.
pub fn claims_instead(text_: &str, memory: &mut Memory) -> bool {
    let clean = squash(&text_.to_lowercase());
    if _a_bare_acknowledgement(&clean) && !_the_heavy_line(memory).is_empty() {
        return true;
    }
    if crate::memory::portions::an_ask(&clean, memory)
        || crate::memory::portions::a_count_to_fill(&clean, memory)
        || crate::memory::portions::a_bare_count_to_fill(&clean, memory)
    {
        return true;
    }
    if attempt(|| truthy(crate::memory::tried::pool_for(&clean, memory))).unwrap_or(false) {
        return true;
    }
    if _a_repair_is_open(memory) && THEY_ARE_NOT_TALKING.search(&clean).is_some() {
        return true;
    }
    REGRET_DEED.search(&clean).is_some()
}

/// Python: `CLAIMS_INSTEAD = claims_instead`.
pub fn CLAIMS_INSTEAD(text_: &str, memory: &mut Memory) -> bool {
    claims_instead(text_, memory)
}

/// Python: bot/modules/chat.py:13716 `_only_the_apology(clean, match)`
pub fn _only_the_apology(clean: &str, m: Option<&Match>) -> bool {
    let rest = match m {
        Some(m) => &clean[m.end()..],
        None => clean,
    };
    let words: Vec<String> = az_words(rest).into_iter().filter(|w| text::len(w) > 2).collect();
    words.iter().filter(|w| !in_set(&CHAT_SCATTER_LEFTOVER, w)).count() < 2
}

/// Python: bot/modules/chat.py:13750 `_chat_scatter(clean, memory)`
pub fn _chat_scatter(clean: &str, memory: &Memory) -> Option<String> {
    let shape = CHAT_SCATTER.search(clean);
    let asks = CHAT_SCATTER_ASKS.search(clean);
    if shape.is_none() && asks.is_none() {
        return None;
    }
    if CHAT_SCATTER_NOT.search(clean).is_some() || CHAT_SCATTER_CTX.search(clean).is_none() {
        return None;
    }
    if !_only_the_apology(clean, shape.as_ref().or(asks.as_ref())) {
        return None;
    }
    let (with_a_list, alone) = if shape.is_some() {
        (&*CHAT_SCATTER_REPLIES, &*CHAT_SCATTER_ALONE)
    } else {
        (&*CHAT_ASKING_REPLIES, &*CHAT_ASKING_ALONE)
    };
    let held = py(crate::memory::thread::on_the_table(memory, 3));
    if held.len() >= 2 {
        let said_ = py_str(&py(crate::memory::thread::in_a_list(&held)));
        let options: Vec<String> = with_a_list.strs().iter().map(|o| py_format(o, &[("list", &said_)])).collect();
        return Some(pick(&options, memory, ""));
    }
    if held.truthy() || shape.is_some() {
        return Some(pick(&alone.strs(), memory, ""));
    }
    None
}

/// Python: bot/modules/chat.py:13999 `_the_deed_pool(deed)`
pub fn _the_deed_pool(deed: &Value) -> Vec<String> {
    match deed.as_str() {
        Some("forgot a date") => FORGOT_DATE_REPLIES.strs(),
        Some("left it too long") => LEFT_IT_TOO_LONG_REPLIES.strs(),
        _ => DEED_REPLIES.strs(),
    }
}

/// Python: bot/modules/chat.py:14008 `_the_words_pool(deed)`
pub fn _the_words_pool(deed: &Value) -> Vec<String> {
    if deed.as_str() == Some("forgot a date") {
        FORGOT_DATE_WORDS.strs()
    } else {
        DEED_WORDS.strs()
    }
}

/// Python: bot/modules/chat.py:14092 `_an_entry_answers_the_deed(clean, memory)`
pub fn _an_entry_answers_the_deed(clean: &str, memory: &Memory) -> bool {
    let deed = py(crate::compose::repair::deed_of(&Value::from(clean)));
    if !deed.as_str().is_some_and(|d| in_set(&DEEDS_WITH_A_CARD, d)) {
        return false;
    }
    let Some(read) = memory.understood.as_ref() else { return false };
    attempt(|| crate::knowledge::lookup::find(read, None, None, false).is_some()).unwrap_or(false)
}

/// Python: bot/modules/chat.py:14111 `_said_wrong(clean)`
pub fn _said_wrong(clean: &str) -> bool {
    if truthy(crate::compose::repair::owns_the_deed(&Value::from(clean))) {
        return true;
    }
    if SAID_WRONG.search(clean).is_none() || SAID_WRONG_NOT.search(clean).is_some() {
        return false;
    }
    !(SAID_WRONG_LOST.search(clean).is_some()
        || SAID_WRONG_TO_AN_ANIMAL.search(clean).is_some()
        || SAID_WRONG_TO_A_PROFESSIONAL.search(clean).is_some())
}

/// Python: bot/modules/chat.py:14140 `_own_deed(clean, start)` (`start` a byte offset).
pub fn _own_deed(clean: &str, start: usize) -> bool {
    let from = CLAUSE_BOUNDARY.finditer(&clean[..start]).iter().map(|b| b.end()).max().unwrap_or(0);
    let opening = &clean[from..start];
    FIRST_PERSON_DEED.match_(opening).is_some() || text::strip(opening).is_empty()
}

/// Python: bot/modules/chat.py:14149 `_said_unkindly(clean)`
pub fn _said_unkindly(clean: &str) -> bool {
    if _said_wrong(clean) || I_SAID.search(clean).is_some() {
        return false;
    }
    for matched in SAID_UNKINDLY.finditer(clean) {
        // `matched.start("word")` is -1 when the group took no part, and `clean[:-1]` is all
        // but the last character.
        let before = match matched.name_span("word") {
            Some((s, _)) => &clean[..s],
            None => &clean[..clean.char_indices().last().map(|(i, _)| i).unwrap_or(0)],
        };
        if FEELING_CLAUSE.search(before).is_some() && _own_deed(clean, matched.start()) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/modules/chat.py:14163 `respond(text, memory)`, before the earlier-turn check
/// `again.replied` wraps it in.
fn respond_unwrapped(text_: &str, memory: &mut Memory) -> Option<String> {
    if crate::memory::when::asks_what_today_is(text_) {
        return None;
    }
    let step = py(crate::memory::plan::carry_on(text_, memory));
    if !step.is_none() {
        return opt_str(step);
    }
    if !memory.get("greet").is_some_and(|g| g.truthy()) {
        let answer = py(crate::memory::obstacle::answer_for(text_, memory));
        if !answer.is_none() {
            return opt_str(answer);
        }
    }
    if !memory.get("greet").is_some_and(|g| g.truthy()) {
        let narrowed = py(crate::memory::ack::a_limit_for_the_plan_text(text_, memory));
        if !narrowed.is_none() {
            return opt_str(narrowed);
        }
    }
    if let Some(theirs) = said(crate::memory::matters::what_they_say(text_, memory)) {
        return Some(theirs);
    }
    if let Some(hoped) = said(crate::memory::hope::answer_for(text_, memory)) {
        return Some(hoped);
    }
    if let Some(resized) = said(crate::memory::size::answer_for(text_, memory)) {
        return Some(resized);
    }
    let mut reply = _respond(text_, memory, true);
    if reply.is_none() || _the_same_answer_again(reply.as_deref().unwrap_or(""), memory, text_) {
        if let Some(fixed) = said(crate::compose::verdict::the_fix_asked(text_, memory)) {
            reply = Some(fixed);
        }
    }
    if let Some(r) = &reply {
        if _the_same_answer_again(r, memory, text_)
            && !(crate::memory::repeat::asks_to_put_it_right(text_) && A_REPEAT_LEAD.match_(r).is_some())
        {
            reply = None;
        }
    }
    let greeted = memory.remove("greet");
    if let Some(greeted) = greeted.filter(|g| g.truthy()) {
        let greeted = py_str(&greeted);
        let Some(r) = reply else {
            return Some(_reply("meet", memory, Some(&greeted), ""));
        };
        reply = opt_str(crate::compose::greet_with(&greeted, &r));
    }
    if reply.is_some() {
        return reply;
    }
    let typed = squash(&text_.to_lowercase());
    let retries = [_repaired_line(memory), _normalised_line(memory)];
    for retry in retries.into_iter().flatten() {
        if text::strip(&retry) == typed {
            continue;
        }
        if _tokens(&retry).len() > 8 {
            continue;
        }
        if let Some(r) = _respond(&retry, memory, false) {
            return Some(r);
        }
    }
    _follow_up_reading(text_, memory)
}

/// Python: `respond = _never_twice.replied(respond)` - the runtime hook: whatever the module
/// answers, the earlier-turn check (`again.instead`) frames a paragraph the chat already had.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    let reply = respond_unwrapped(text_, memory)?;
    if text::strip(&reply).is_empty() {
        return Some(reply);
    }
    match attempt(|| opt_str(crate::memory::again::instead(&reply, text_, memory))) {
        Some(Some(framed)) if !framed.is_empty() => Some(framed),
        _ => Some(reply),
    }
}

/// Python: bot/modules/chat.py:14288 `_a_mood_pool_line(said)`
pub fn _a_mood_pool_line(said_: &str) -> bool {
    let flat = _words_only(said_);
    if flat.is_empty() {
        return false;
    }
    for kind in HEAVY_FEELINGS.iter() {
        if let Some(pool) = REPLIES.get(kind) {
            if pool.items().iter().any(|line| line.as_str().is_some_and(|l| _words_only(l) == flat)) {
                return true;
            }
        }
    }
    false
}

/// The bot's last reply and the person's last line (folded), walking back from the end.
fn last_and_previous(memory: &Memory) -> (String, String) {
    let mut last = String::new();
    let mut previous = String::new();
    for turn in turns(memory).iter().rev() {
        if last.is_empty() && !turn.bot.is_empty() {
            last = turn.bot.clone();
        }
        if !turn.user.is_empty() {
            previous = squash(&turn.user.to_lowercase());
            break;
        }
    }
    (last, previous)
}

inline_re!(A_DIGIT, r"\d");

/// `set(_the_rates_in(text))`.
fn rates_in(text_: &str) -> HashSet<String> {
    py(crate::memory::topics::_the_rates_in(text_)).strs().into_iter().collect()
}

/// Python: bot/modules/chat.py:14306 `_a_part_of_the_answer_they_have(clean, memory)`
pub fn _a_part_of_the_answer_they_have(clean: &str, memory: &Memory) -> Option<String> {
    let (last, previous) = last_and_previous(memory);
    if text::strip(&last).is_empty() {
        return None;
    }
    let asked = squash(&clean.to_lowercase());
    if !asked.is_empty() && !previous.is_empty() && (asked == previous || previous.contains(&asked)) {
        return None;
    }
    if let Some(cause) = said(crate::memory::again::the_cause_said_again(&last, memory)) {
        return Some(cause);
    }
    let asked_rates = rates_in(clean);
    if !asked_rates.is_empty() && asked_rates.is_disjoint(&rates_in(&last)) {
        return None;
    }
    let part = attempt(|| crate::memory::narrow::narrow(&last, clean, "")).flatten()?;
    if text::strip(&part).is_empty() {
        return None;
    }
    if squash(&part.to_lowercase()) == squash(&last.to_lowercase()) {
        return None;
    }
    let no_set_time = data::string("bot.memory.narrow.NO_SET_TIME");
    if A_DIGIT.search(&part).is_none() && !part.starts_with(&no_set_time) {
        return None;
    }
    let last_rates = rates_in(&last);
    if !last_rates.is_empty() && rates_in(&part).is_disjoint(&last_rates) {
        return None;
    }
    Some(text::strip(&part).to_string())
}

/// Python: bot/modules/chat.py:14377 `_the_newest_line_started_the_topic(memory)`
pub fn _the_newest_line_started_the_topic(memory: &Memory) -> bool {
    let Some(Value::List(records)) = memory.get("topics") else { return false };
    let Some(record) = records.last() else { return false };
    if !record.is_dict() {
        return false;
    }
    record.get("turn").cloned().unwrap_or(Value::None) == Value::Int(memory.turns.len() as i64 - 1)
}

inline_re!(SENTENCE_SPLIT, r"(?<=[.!?])\s+");
inline_re!(FOUR_LETTER_WORDS, r"[a-z']{4,}");

fn with_a_stop(said_: &str) -> String {
    if said_.ends_with('.') || said_.ends_with('!') || said_.ends_with('?') {
        said_.to_string()
    } else {
        format!("{said_}.")
    }
}

/// Python: bot/modules/chat.py:14480 `_the_verdict_in(clean, memory)`
pub fn _the_verdict_in(clean: &str, memory: &Memory) -> Option<String> {
    let asked = squash(&clean.to_lowercase());
    if asked.is_empty() || EVERY_YES_NO_OPEN.match_(&asked).is_none() || ASKS_A_FIGURE.search(&asked).is_some() {
        return None;
    }
    if A_NEW_CONDITION.search(&asked).is_some() {
        return None;
    }
    if A_ASKS_WHERE.search(&asked).is_some() {
        return None;
    }
    let (last, previous) = last_and_previous(memory);
    if text::strip(&last).is_empty() {
        return None;
    }
    if !previous.is_empty() && (asked == previous || previous.contains(&asked)) {
        return None;
    }
    let sentences: Vec<String> = SENTENCE_SPLIT.split(text::strip(&last)).iter().map(|one| squash(one)).collect();
    for said_ in sentences.iter().skip(1) {
        if n_words(said_) < 4 || A_VERDICT.search(said_).is_none() {
            continue;
        }
        if A_WHEN_TO_GET_HELP.search(said_).is_some() {
            continue;
        }
        return Some(with_a_stop(said_));
    }
    if ASKS_A_PRICE.search(&asked).is_some() {
        return None;
    }
    if A_REPEAT_FRAME.search(&last).is_some() {
        return None;
    }
    if A_STATE_ASK.match_(&asked).is_none() {
        return None;
    }
    let named: HashSet<String> = FOUR_LETTER_WORDS.findall_str(&previous).into_iter().filter(|w| !in_set(&A_VERDICT_FILLER, w)).collect();
    for said_ in sentences.iter().skip(1).take(2) {
        if n_words(said_) < 4 {
            continue;
        }
        if A_DOES_SOMETHING.match_(said_).is_some() || !(A_STATE_WORD.search(said_).is_some() || A_REASSURES.search(said_).is_some()) {
            continue;
        }
        if FOUR_LETTER_WORDS.findall_str(&said_.to_lowercase()).iter().any(|w| named.contains(w)) {
            return Some(with_a_stop(said_));
        }
    }
    None
}

/// The topic records' turn when it is an int (`isinstance(x, int)`, bools included).
fn record_turn(record: &Value) -> Option<i64> {
    match record.get("turn") {
        Some(Value::Int(i)) => Some(*i),
        Some(Value::Bool(b)) => Some(*b as i64),
        _ => None,
    }
}

/// Python: bot/modules/chat.py:14571 `_a_topic_still_in_this_exchange(topic, memory)`
pub fn _a_topic_still_in_this_exchange(topic: &str, memory: &Memory) -> bool {
    let Some(Value::List(records)) = memory.get("topics") else { return false };
    for record in records.iter().rev() {
        if !record.is_dict() {
            continue;
        }
        if squash(&py_str(record.get("text").unwrap_or(&Value::None)).to_lowercase()) != topic {
            continue;
        }
        let Some(began) = record_turn(record) else { return false };
        return memory.turns.len() as i64 - began <= 3;
    }
    false
}

/// Python: bot/modules/chat.py:14598 `_follow_up_reading(text, memory)`
pub fn _follow_up_reading(text_: &str, memory: &Memory) -> Option<String> {
    if truthy(crate::memory::topics::in_a_safety_thread(memory)) || _reads_as_distress(text_) {
        return None;
    }
    if let Some(verdict) = _the_verdict_in(text_, memory).filter(|v| !v.is_empty()) {
        return Some(verdict);
    }
    if !_bare_ask(text_) {
        return None;
    }
    let no_set_time = data::string("bot.memory.narrow.NO_SET_TIME");
    if let Some(part) = _a_part_of_the_answer_they_have(text_, memory) {
        if !part.is_empty() && part.starts_with(&no_set_time) {
            return Some(part);
        }
    }
    let understood = memory.understood.as_ref();
    if understood.is_some_and(|u| u.about_bot_kind.as_deref().is_some_and(|k| !k.is_empty())) {
        return None;
    }
    if understood.is_some_and(|u| u.someone_new) {
        return None;
    }
    let clean = squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    let topics_said: Vec<String> =
        py(crate::memory::topics::context(memory, 3)).strs().iter().rev().map(|line| squash(&line.to_lowercase())).collect();
    let recent_said: Vec<String> = last_n(turns(memory), 3).iter().map(|t| squash(&t.user.to_lowercase())).collect();
    if understood.is_some_and(|u| u.kind == "question")
        && topics_said.iter().chain(recent_said.iter()).any(|line| _NO_REASON.match_(line).is_some())
    {
        return None;
    }
    if !topics_said.is_empty() && _nothing_was_heard(memory, &topics_said[0]) {
        return None;
    }
    if topics_said.iter().any(|t| _reads_as_distress(t)) {
        return None;
    }
    let mut readings: Vec<(String, String)> = Vec::new();
    for topic in &topics_said {
        if topic.is_empty() || *topic == clean || truthy(crate::memory::topics::names_nothing(topic)) {
            continue;
        }
        if _names_its_own_thing(text_, topic) {
            continue;
        }
        let stay = py(crate::memory::topics::a_stay_with_the_place(&clean, topic));
        if !stay.is_none() {
            readings.push((topic.clone(), py_str(&stay)));
        } else if truthy(crate::memory::topics::asks_how_long_to_stay(&clean)) && truthy(crate::memory::topics::the_city_of(topic)) {
            continue;
        } else {
            readings.push((topic.clone(), format!("{topic} {clean}")));
        }
    }
    if readings.is_empty() {
        return None;
    }
    let carries_past_the_newest = truthy(crate::memory::topics::names_nothing(&clean))
        && (topics_said.is_empty()
            || truthy(crate::memory::topics::names_nothing(&topics_said[0]))
            || !_the_newest_line_started_the_topic(memory));
    for (index, (topic, reading)) in readings.iter().enumerate() {
        if index > 0 && !carries_past_the_newest {
            return None;
        }
        if *reading == clean {
            continue;
        }
        let mut reading_memory = memory.clone();
        reading_memory.understood = Some(crate::understanding::understand(reading, Some(&reading_memory)));
        reading_memory.set("joined_read", true);
        let mut said_ = crate::modules::answers::respond(reading, &mut reading_memory);
        if said_.as_deref().is_none_or(|s| s.is_empty()) || _said_before(said_.as_deref().unwrap_or(""), memory) {
            if memory.get("entry_answer").is_some_and(|v| v.truthy()) || memory.get("entry_answers").is_some_and(|v| v.truthy()) {
                if let Some(part) = _a_part_of_the_answer_they_have(&clean, memory).filter(|p| !p.is_empty()) {
                    return Some(part);
                }
                continue;
            }
            said_ = _respond(reading, &mut reading_memory, true);
            if said_.as_deref().is_some_and(|s| !s.is_empty())
                && memory.understood.as_ref().is_some_and(|u| u.kind == "question")
                && _a_mood_pool_line(said_.as_deref().unwrap())
            {
                said_ = None;
            }
            if said_.as_deref().is_some_and(|s| !s.is_empty() && _says_nothing_about_the_ask(s)) {
                said_ = None;
            }
        }
        let Some(said_) = said_.filter(|s| !s.is_empty()) else { continue };
        let unframed = crate::memory::again::unframed(&said_).filter(|u| !u.is_empty()).unwrap_or_else(|| said_.clone());
        if _said_before(&said_, memory) || _said_before(&unframed, memory) {
            if let Some(part) = _a_part_of_the_answer_they_have(&clean, memory).filter(|p| !p.is_empty()) {
                return Some(part);
            }
            if index == 0 && topics_said.len() > 1 && _a_topic_still_in_this_exchange(&topics_said[1], memory) {
                continue;
            }
            return None;
        }
        if !_says_the_kind(&clean, &said_) {
            return None;
        }
        if _asks_what_they_said(&said_, memory) {
            continue;
        }
        if !_about_the_topic(reading, &reading_memory, topic) {
            continue;
        }
        return Some(said_);
    }
    None
}

/// Python: bot/modules/chat.py:14846 `_reads_as_distress(line)`
pub fn _reads_as_distress(line: &str) -> bool {
    DISTRESS.search(&format!(" {} ", squash(&line.to_lowercase()))).is_some()
}

/// Python: bot/modules/chat.py:14857 `_nothing_was_heard(memory, topic)`
pub fn _nothing_was_heard(memory: &Memory, topic: &str) -> bool {
    let Some(Value::List(records)) = memory.get("topics") else { return false };
    for record in records.iter().rev() {
        if !record.is_dict() {
            continue;
        }
        if squash(&py_str(record.get("text").unwrap_or(&Value::None)).to_lowercase()) != topic {
            continue;
        }
        let Some(index) = record_turn(record) else { return false };
        if index < 0 || index as usize >= memory.turns.len() {
            return false;
        }
        let module = memory.turns[index as usize].module.as_deref().unwrap_or("");
        return memory.turns[index as usize].module.is_some() && in_set(&HEARD_NOTHING, module);
    }
    false
}

/// Python: bot/modules/chat.py:14879 `_asks_what_they_said(said, memory)`
pub fn _asks_what_they_said(said_: &str, memory: &Memory) -> bool {
    if TIME_ASK.search(said_).is_none() {
        return false;
    }
    let duration = data::re("bot.compose.trouble.DURATION");
    for turn in turns(memory).iter().rev() {
        let line = squash(&turn.user.to_lowercase());
        if !line.is_empty() && duration.search(&format!(" {line} ")).is_some() {
            return true;
        }
    }
    false
}

/// `word.rstrip("s") or word`.
fn rstrip_s(word: &str) -> String {
    let s = text::rstrip_chars(word, "s");
    if s.is_empty() {
        word.to_string()
    } else {
        s.to_string()
    }
}

/// Python: bot/modules/chat.py:14904 `_the_entrys_own_name(item)`
pub fn _the_entrys_own_name(item: &crate::knowledge::entry::Entry) -> HashSet<String> {
    let mut found = HashSet::new();
    if let Some(first) = item.key.first() {
        for option in first.words() {
            for word in text::split_ref(&option.to_lowercase()) {
                found.insert(rstrip_s(word));
            }
        }
    }
    found
}

/// Python: bot/modules/chat.py:14916 `_about_the_topic(reading, memory, topic)`
pub fn _about_the_topic(reading: &str, memory: &Memory, topic: &str) -> bool {
    let Some(item) = crate::knowledge::lookup::find(&crate::understanding::understand(reading, Some(memory)), None, None, false) else {
        return true;
    };
    let said_: HashSet<String> = TOPIC_WORD.findall_str(topic).iter().map(|w| rstrip_s(w)).collect();
    let read = crate::understanding::understand(topic, Some(memory));
    let named: HashSet<String> = TOPIC_WORD.findall_str(read.subject.as_deref().unwrap_or("")).iter().map(|w| rstrip_s(w)).collect();
    let keys = crate::knowledge::lookup::key_words(item);
    if keys.iter().any(|k| named.contains(k)) {
        return true;
    }
    let name = _the_entrys_own_name(item);
    let things: HashSet<String> = read
        .entities
        .iter()
        .filter(|e| e.kind == "thing" || e.kind == "person")
        .flat_map(|e| TOPIC_WORD.findall_str(&e.text))
        .map(|w| rstrip_s(&w))
        .collect();
    let asked: HashSet<String> = TOPIC_WORD.findall_str(reading).iter().map(|w| rstrip_s(w)).filter(|w| !said_.contains(w)).collect();
    name.iter().any(|w| things.contains(w) && !asked.contains(w))
}

/// Python: bot/modules/chat.py:14965 `_says_nothing_about_the_ask(said)`
pub fn _says_nothing_about_the_ask(said_: &str) -> bool {
    NOT_AN_ANSWER.strs().iter().any(|kind| REPLIES.get(kind).is_some_and(|pool| pool.contains_str(said_)))
}

/// Python: bot/modules/chat.py:15036 `_names_its_own_thing(text, topic="")`
pub fn _names_its_own_thing(text_: &str, topic: &str) -> bool {
    let clean = squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    let mut named: Vec<String> = OWN_THING.findall_str(&clean);
    if LOOKING_FOR.match_(&clean).is_some() {
        named.extend(FOUND_THING.findall_str(&clean).into_iter().filter(|w| !in_set(&NOT_A_THING, w)));
    }
    if named.is_empty()
        && ASKS_ABOUT_A_THING.match_(&clean).is_some()
        && OFFERING.match_(&clean).is_none()
        && ASKS_WHAT_TO_DO_WITH.match_(&clean).is_none()
    {
        named.extend(AN_ARTICLE.findall_str(&clean).into_iter().filter(|w| !in_set(&NOT_A_THING, w) && !in_set(&A_CONTAINER, w)));
    }
    if named.is_empty() {
        return false;
    }
    let said_: HashSet<String> = az_words(&topic.to_lowercase()).into_iter().collect();
    let bare_: HashSet<String> = said_.iter().map(|w| text::rstrip_chars(w, "s").to_string()).collect();
    !named.iter().any(|w| said_.contains(w) || bare_.contains(text::rstrip_chars(w, "s")))
}

/// Python: bot/modules/chat.py:15063 `_bare_ask(text)`
pub fn _bare_ask(text_: &str) -> bool {
    let clean = squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    let words = n_words(&clean);
    if words > 8 {
        return false;
    }
    if OFFERING.match_(&clean).is_some() {
        return false;
    }
    if QUESTION_OPEN.match_(&clean).is_some() {
        return true;
    }
    YES_NO_OPEN.match_(&clean).is_some() && words as i64 <= YES_NO_WORDS.as_int().unwrap_or(5)
}

inline_re!(NOT_WORD_CHARS, r"[^0-9a-z]+");

/// Python: bot/modules/chat.py:15081 `_words_only(text)`
pub fn _words_only(text_: &str) -> String {
    squash(&NOT_WORD_CHARS.sub(" ", &text_.to_lowercase()))
}

/// Python: bot/modules/chat.py:15101 `_the_same_answer_again(said, memory, line="")`
pub fn _the_same_answer_again(said_: &str, memory: &Memory, line: &str) -> bool {
    if crate::memory::meant::reads_it(line) {
        return false;
    }
    if n_words(&squash(line)) as i64 > A_SHORT_LINE.as_int().unwrap_or(5) {
        return false;
    }
    let flat = _words_only(said_);
    if n_words(&flat) < 12 {
        return false;
    }
    let typed = said_.to_lowercase();
    if MEANT_LEADS.strs().iter().any(|lead| typed.starts_with(lead.as_str())) {
        return false;
    }
    let mut readings = vec![flat];
    if let Some(lead) = A_REPEAT_LEAD.match_(said_) {
        let after = _words_only(&said_[lead.end()..]);
        if n_words(&after) >= 12 {
            readings.push(after);
        }
    }
    for turn in last_n(turns(memory), 4) {
        let was = _words_only(&turn.bot);
        for reading in &readings {
            if was == *reading || (n_words(&was) > n_words(reading) && was.starts_with(reading.as_str())) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/chat.py:15165 `_said_before(said, memory)`
pub fn _said_before(said_: &str, memory: &Memory) -> bool {
    let said_ = squash(said_);
    if said_.is_empty() {
        return true;
    }
    let flat = _words_only(&said_);
    if flat.is_empty() {
        return true;
    }
    let pieces: Vec<String> = SENTENCE_SPLIT.split(&said_).iter().map(|p| _words_only(p)).filter(|p| n_words(p) >= 4).collect();
    for turn in turns(memory) {
        let bot = squash(&turn.bot);
        if bot.is_empty() {
            continue;
        }
        let said_bot = _words_only(&bot);
        if said_bot.is_empty() {
            continue;
        }
        if flat == said_bot {
            return true;
        }
        if n_words(&said_bot) >= 4 && (said_bot.contains(&flat) || flat.contains(&said_bot)) {
            return true;
        }
        if pieces.iter().any(|p| said_bot.contains(p.as_str())) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/chat.py:15202 `_says_the_kind(ask, said)`
pub fn _says_the_kind(ask: &str, said_: &str) -> bool {
    let Some(kind) = crate::memory::narrow::kind_of(ask) else { return true };
    let spans = data::get("bot.memory.narrow.SPANS");
    match spans.get(&kind).and_then(|p| p.as_re()) {
        None => true,
        Some(re) => re.search(said_).is_some(),
    }
}

/// Python: bot/modules/chat.py:15279 `_work_grumble(clean, tokens, memory)`
pub fn _work_grumble(clean: &str, tokens: &[String], memory: &mut Memory) -> Option<String> {
    if tokens.len() < 4 || clean.contains(" but ") || truthy(crate::memory::topics::in_a_safety_thread(memory)) {
        return None;
    }
    let a_fault = WORK_PERSON.search(clean).is_some() && WORK_FAULT.search(clean).is_some();
    let unseen = WORK_UNSEEN.search(clean).is_some() && WORK_CONTEXT.search(clean).is_some();
    if !(a_fault || unseen) {
        return None;
    }
    if WORK_ASK.search(&WORK_RELATIVE.sub(" ", clean)).is_some() {
        return None;
    }
    if _answers_knows(clean, memory) {
        return None;
    }
    let pool = if unseen && !a_fault { &*WORK_UNSEEN_REPLY } else { &*WORK_GRUMBLE };
    Some(pick(&pool.strs(), memory, clean))
}

inline_re!(HOWL, r"\bhowl");

/// Python: bot/modules/chat.py:15360 `_laughing_at(clean, memory=None)`
pub fn _laughing_at(clean: &str, memory: &mut Memory) -> Option<&'static str> {
    if clean.contains('?') {
        return None;
    }
    let together = LAUGH_TOGETHER.search(clean).is_some() && LAUGH_TOGETHER_NOT.search(clean).is_none();
    if LAUGH_REPORT.search(clean).is_none() && !together {
        return None;
    }
    if LAUGH_AT_ME.search(clean).is_some() || LAUGH_BAD_SETTING.search(clean).is_some() {
        return None;
    }
    if !together && LAUGH_MINE.search(clean).is_none() {
        return None;
    }
    if LAUGH_NOT_A_PERSON.search(clean).is_some() && HOWL.search(clean).is_some() {
        return None;
    }
    if crate::compose::badnews::reads_as_bad_news(clean) {
        return None;
    }
    memory.set("laughed", memory.turns.len());
    Some("laughing")
}

/// Python: bot/modules/chat.py:15394 `_the_condolence_is_the_last_thing(memory)`
pub fn _the_condolence_is_the_last_thing(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    if matches!(last.module.as_deref(), Some("loss" | "grief")) {
        return true;
    }
    if data::get("bot.compose.loss.AFTER_REPLIES").contains_str(&squash(&last.bot)) {
        return true;
    }
    crate::compose::loss::reads_as_loss(&last.user, None, false)
}

/// Python: bot/modules/chat.py:15442 `_laughing_still(clean, memory)`
pub fn _laughing_still(clean: &str, memory: &Memory) -> Option<String> {
    let was = match memory.get("laughed") {
        Some(Value::Int(i)) => *i,
        _ => return None,
    };
    if memory.turns.len() as i64 != was + 1 {
        return None;
    }
    if clean.contains('?') || LAUGH_CARRIED_ON.search(clean).is_none() {
        return None;
    }
    if crate::compose::badnews::reads_as_bad_news(clean) || LAUGH_BAD_SETTING.search(clean).is_some() || LAUGH_CARRIED_ON_NOT.search(clean).is_some() {
        return None;
    }
    Some(pick(&LAUGH_CARRIED_ON_REPLY.strs(), memory, clean))
}

/// Python: bot/modules/chat.py:15498 `_somebody_thinks_i_should(clean)`: (who, named).
pub fn _somebody_thinks_i_should(clean: &str) -> Option<(String, String)> {
    let t = squash(&clean.to_lowercase());
    if t.is_empty() || t.contains('?') || n_words(&t) > 26 {
        return None;
    }
    let found = SOMEBODY_THINKS.search(&t).or_else(|| EVERYONE_THINKS.search(&t))?;
    let who = squash(found.name_str("who"));
    let named_ = squash(found.name_str("named"));
    let named = text::strip_chars(&named_, " .,!").to_string();
    if named.is_empty() || n_words(&named) > 10 {
        return None;
    }
    if THEIR_OPINION_IS_A_HEALTH_MATTER.search(&t).is_some() {
        return None;
    }
    Some((who, named))
}

/// Python: bot/modules/chat.py:15520 `_thinks_pool(who)`
pub fn _thinks_pool(who: &str) -> Vec<String> {
    let who = if !["everyone", "everybody", "people"].contains(&who) && !who.starts_with("my ") { format!("your {who}") } else { who.to_string() };
    let cap = upper1(&who);
    THINKS_REPLIES.strs().iter().map(|line| py_format(line, &[("who", &who), ("who_cap", &cap)])).collect()
}

inline_re!(TOLD_ME_TO, r"\b(?:told|tells?)\s+me\s+to\b");
inline_re!(I_AM, r"\bi(?:'m|\s+am)\b");

/// Python: bot/modules/chat.py:15593 `_somebody_recommended_a_thing(clean)`
pub fn _somebody_recommended_a_thing(clean: &str) -> Option<String> {
    let t = squash(&clean.to_lowercase().replace('\u{2019}', "'"));
    if t.is_empty() || t.contains('?') || n_words(&t) > 24 {
        return None;
    }
    let found = A_REPORT_A_THING.search(&t).or_else(|| A_HEARD_A_THING.search(&t))?;
    let rest_ = squash(found.name_str("rest"));
    let rest = text::strip_chars(&rest_, " .,!").to_string();
    if rest.is_empty() || n_words(&rest) > 12 {
        return None;
    }
    if A_ASKED_NOT_TOLD.search(&t).is_some() {
        return None;
    }
    if A_PRAISING_VERB.search(&t).is_none() && A_PRAISE_OF_A_THING.search(&rest).is_none() {
        return None;
    }
    if TOLD_ME_TO.search(&t).is_some() {
        return None;
    }
    if I_AM.search(&rest).is_some() {
        return None;
    }
    if A_RECOMMENDATION_IS_NOT.search(&rest).is_some() {
        return None;
    }
    Some(rest)
}

// ---------------------------------------------------------------------------------------------
// Python lines 15620-16731: `_respond`, the reading of one line.

res!(SOMEONE_BRING_SOMEONE, ASKS_WHAT_TO_DO, SAID_WRONG_AT_THEMSELVES, FAMILY_NAMED);
vals!(
    AMENDS_REPLIES, NOT_TALKING_AFTER_A_REPAIR, SAID_WRONG_NEXT, DEED_NEXT, AT_MYSELF_REPLIES,
    SAID_WRONG_REPLIES, SAID_UNKINDLY_FAMILY, SAID_UNKINDLY_REPLIES,
);

/// Python: bot/modules/chat.py:15620 `_respond(text, memory, as_said=True)`
pub fn _respond(text_: &str, memory: &mut Memory, as_said: bool) -> Option<String> {
    let planned = py(crate::compose::occasionplan::reply_for(text_, memory));
    if !planned.is_none() {
        return opt_str(planned);
    }
    let mut text_: String = text_.to_string();
    let mut clean = squash(&text_.to_lowercase().replace('\u{2019}', "'"));
    clean = crate::understanding::normalize::without_the_figure_of_speech(&clean);
    clean = _without_the_might(&clean);
    let mut tokens = _tokens(&text_);
    if tokens.is_empty() {
        return None;
    }
    if _asked_how_it_went(&clean) {
        return None;
    }
    let still_in = py(crate::memory::tried::in_care_pool(&clean, memory));
    if still_in.truthy() {
        return Some(pick(&still_in.strs(), memory, ""));
    }
    let news_detail = crate::compose::goodnews::a_detail_of_the_news(&clean, memory);
    if !news_detail.is_empty() && !_a_death_just_said(memory) {
        let said_detail = squash(&clean);
        let kind = match news_detail.as_str() {
            "attempt" => "more_of_the_news",
            "arrival" => "more_of_the_news_arrival",
            _ => "more_of_the_news_score",
        };
        let big = upper1(&said_detail);
        return Some(_say(kind, memory, &[("X", &big)]));
    }
    if SOMEONE_BRING_SOMEONE.search(&clean).is_some() {
        let record = py(crate::memory::topics::about(memory));
        if record.truthy() && record.get("trouble").and_then(|t| t.as_str()) == Some("hospital") {
            return None;
        }
    }
    if let Some(doubt) = _a_doubt_about_being_taken_on(&clean, memory).filter(|d| !d.is_empty()) {
        return Some(doubt);
    }
    if let Some(aged) = _the_age_in_a_work_thread(&clean, memory).filter(|a| !a.is_empty()) {
        return Some(aged);
    }
    if truthy(crate::compose::repair::amends_to(&clean)) {
        return Some(pick(&AMENDS_REPLIES.strs(), memory, &clean));
    }
    if _a_repair_is_open(memory) && THEY_ARE_NOT_TALKING.search(&clean).is_some() {
        return Some(pick(&NOT_TALKING_AFTER_A_REPAIR.strs(), memory, &clean));
    }
    if _after_a_repair(memory) && ASKS_FOR_THE_WORDS.search(&clean).is_some() {
        let deed = _the_deed_before(memory);
        if deed != **SAID_WRONG_DEED {
            return Some(pick(&_the_words_pool(&deed), memory, &clean));
        }
    }
    if _after_a_repair(memory) && ASKS_WHAT_TO_DO.search(&clean).is_some() {
        if _the_deed_before(memory) == **SAID_WRONG_DEED {
            return Some(pick(&SAID_WRONG_NEXT.strs(), memory, &clean));
        }
        return Some(pick(&DEED_NEXT.strs(), memory, &clean));
    }
    if _said_wrong(&clean) {
        if SAID_WRONG_AT_THEMSELVES.search(&clean).is_some() {
            return Some(pick(&AT_MYSELF_REPLIES.strs(), memory, &clean));
        }
        let deed = py(crate::compose::repair::deed_of(&Value::from(clean.as_str())));
        if deed.truthy() && deed != **SAID_WRONG_DEED && SAID_WRONG.search(&clean).is_none() {
            let line = pick(&_the_deed_pool(&deed), memory, &clean);
            return opt_str(crate::compose::repair::with_the_feeling_named(&line, &clean));
        }
        return Some(pick(&SAID_WRONG_REPLIES.strs(), memory, &clean));
    }
    if _said_unkindly(&clean) {
        let pool = if FAMILY_NAMED.search(&clean).is_some() { &*SAID_UNKINDLY_FAMILY } else { &*SAID_UNKINDLY_REPLIES };
        return Some(pick(&pool.strs(), memory, &clean));
    }
    if let Some(addressed) = said(crate::memory::warmth::reply(&text_, memory)) {
        return Some(addressed);
    }
    if let Some(company) = said(crate::memory::passing_the_time::reply(&text_, memory)) {
        return Some(company);
    }
    if let Some(years_turn) = said(crate::memory::the_years::reply(&text_, memory)) {
        return Some(years_turn);
    }
    if let Some(a_limit) = said(crate::memory::limits::reply(&text_, memory)) {
        return Some(a_limit);
    }
    let thanks_off = _a_thank_you_taken_off(&clean, memory);
    if !thanks_off.is_empty() && _a_plain_good_answer(&thanks_off, memory) {
        let name = _name(memory);
        return Some(_reply("positive", memory, name.as_deref(), ""));
    }
    if !thanks_off.is_empty() {
        clean = thanks_off.clone();
        text_ = thanks_off;
        tokens = _tokens(&clean);
    }
    tokens = _without_the_at_all(&tokens);
    if let Some(step) = _after_lonely(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(step);
    }
    let read_now = memory.understood.clone();
    if let Some(picked_up) = _after_a_lost_thread(&clean, read_now.as_ref(), memory).filter(|s| !s.is_empty()) {
        return Some(picked_up);
    }
    if let Some(still_at_it) = _laughing_still(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(still_at_it);
    }
    if let Some(laughing) = _laughing_at(&clean, memory) {
        let name = _name(memory);
        return Some(_reply(laughing, memory, name.as_deref(), &clean));
    }
    if let Some(nothing) = _nothing_helps(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(nothing);
    }
    if let Some(held) = _a_limit(&text_, memory).filter(|s| !s.is_empty()) {
        return Some(held);
    }
    if _their_own_call(&clean).is_some_and(|c| !c.is_empty()) {
        let name = _name(memory);
        return Some(_reply("own_call", memory, name.as_deref(), &clean));
    }
    if let Some(body_feeling) = _body_is_a_feeling(&clean).filter(|s| !s.is_empty()) {
        let name = _name(memory);
        return Some(_reply(&body_feeling, memory, name.as_deref(), ""));
    }
    let read = memory.understood.clone();
    let asks = read.as_ref().is_some_and(|u| u.kind == "question");
    let symptom = COLD_SYMPTOM.search(&clean).is_some();
    let cold = symptom || COLD_NAMED.search(&clean).is_some();
    let part = read.as_ref().and_then(|u| u.body.clone()).filter(|p| !p.is_empty());
    let bare_complaint = COMPLAINT_ONLY.match_(&clean).is_some() && !asks;
    if (ILL_SELF.search(&clean).is_some() || (symptom && !asks) || part.is_some() || bare_complaint)
        && SICK_DEED.search(&clean).is_none()
        && BODY_BEFORE_AN_EVENT.search(&clean).is_none()
    {
        if COLD_SORE.search(&clean).is_some() && COLD_SYMPTOM.search(&clean).is_none() && COLD_SORE_FEELING.search(&clean).is_none() {
            if COLD_NAMED.search(&COLD_SORE.sub(" ", &clean)).is_none() {
                return None;
            }
        }
        if let Some(part) = &part {
            let kind = format!("ill_{part}");
            if REPLIES.get(&kind).is_some() {
                let name = _name(memory);
                return Some(_reply(&kind, memory, name.as_deref(), &text_));
            }
        }
        if cold && _the_same_cold_again(&clean, memory) {
            return None;
        }
        let name = _name(memory);
        if truthy(crate::compose::says_it_has_lingered(&clean, true)) {
            return Some(_reply("ill_lasting", memory, name.as_deref(), &text_));
        }
        return Some(_reply(if cold { "cold" } else { "ill" }, memory, name.as_deref(), &text_));
    }
    if crate::compose::badnews::reads_like_a_symptom(&clean)
        && truthy(crate::compose::says_it_has_lingered(&clean, false))
        && SICK_DEED.search(&clean).is_none()
    {
        return None;
    }
    if let Some(scattered) = _chat_scatter(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(scattered);
    }
    let understood = memory.understood.clone();
    let mut said_ = if as_said { _as_said(&clean, understood.as_ref()) } else { clean.clone() };
    let live_emergency = truthy(crate::memory::topics::in_a_safety_thread(memory));
    if understood.as_ref().is_some_and(|u| u.presence) {
        if live_emergency {
            return None;
        }
        if truthy(crate::memory::tried::still_going_on(&text_, memory)) {
            return None;
        }
        return Some(_say("presence", memory, &[]));
    }
    if understood.as_ref().is_some_and(|u| u.intent == "opening") {
        let name = _name(memory);
        return Some(_reply("opening", memory, name.as_deref(), ""));
    }
    if understood.as_ref().is_some_and(|u| u.asks_back) && tokens.len() <= 12 && !live_emergency {
        let polarity = understood.as_ref().map(|u| u.polarity).unwrap_or(0);
        let low = polarity < 0 || _a_low_word(&clean);
        let name = _name(memory);
        return Some(_reply(if low { "back_low" } else { "back" }, memory, name.as_deref(), ""));
    }
    let (met, plan) = _just_met(memory);
    if let Some(met) = met.filter(|m| !m.is_empty()) {
        said_ = _plan_clause(plan.as_deref().unwrap_or(""));
        if !said_.is_empty() {
            let join = if said_.to_lowercase().starts_with("you") { " \u{2014} that" } else { "" };
            return Some(format!(
                "Hi {met} \u{2014} good to meet you, and {said_}{join} sounds like a good one. What do you need a hand with?"
            ));
        }
        let kind = understood.as_ref().map(|u| u.kind.as_str());
        if matches!(kind, None | Some("greeting") | Some("empty")) {
            return Some(_reply("meet", memory, Some(&met), ""));
        }
        memory.set("greet", met);
    }
    if tokens.len() <= 6 && _says_nothing_much(&tokens, &clean) {
        let name = _name(memory);
        return Some(_reply("notmuch", memory, name.as_deref(), ""));
    }
    let laughed = understood.as_ref().is_some_and(|u| u.intent == "laugh")
        && !tokens.is_empty()
        && tokens.len() <= 8
        && tokens.iter().all(|t| {
            truthy(crate::understanding::lexicon::is_a_laugh(t)) || in_set(&ACK_WORDS, t) || in_set(&THANKS_FILLER, t)
        });
    let chatty = laughed
        || (tokens.len() <= 8
            && SAYS_TOO_MUCH.search(&clean).is_none()
            && tokens.iter().all(|t| in_set(&ACK_WORDS, t) || in_set(&LAUGH_WORDS, t) || in_set(&THANKS_FILLER, t)));
    if _is_bye(&tokens) || _ending_bye(&clean, memory) {
        let name = _name(memory);
        return Some(_reply(_bye_kind(&tokens), memory, name.as_deref(), ""));
    }
    let bare: Vec<String> = tokens.iter().filter(|t| !in_set(&ADDRESSED, t) && !in_set(&THANKS_NOISE, t)).cloned().collect();
    if _a_hedge_answering_the_question(memory) {
        if HEDGED_LOW_LINE.match_(&clean).is_some() {
            let name = _name(memory);
            return Some(_reply("hedged_low", memory, name.as_deref(), ""));
        }
        if HEDGED_OK_LINE.match_(&clean).is_some() {
            let name = _name(memory);
            return Some(_reply("hedged_ok", memory, name.as_deref(), ""));
        }
    }
    if _is_thanks(&bare) || _is_thanks(&tokens) || _is_thanks(&_without_the_nod(&tokens)) {
        let name = _name(memory);
        if ANSWERS_HOW_ARE_YOU.match_(&clean).is_some() && THANKS_FOR.search(&clean).is_none() {
            return Some(_reply("positive", memory, name.as_deref(), ""));
        }
        return Some(_reply("thanks", memory, name.as_deref(), ""));
    }
    if let Some(moved) = said(crate::memory::limit::move_after_a_limit(&clean, memory)) {
        return Some(moved);
    }
    if let Some(limited) = said(crate::memory::limit::reply_for(&clean, memory)) {
        return Some(limited);
    }
    if let Some(piled) = said(crate::memory::pile::reply_for(&clean, memory)) {
        return Some(piled);
    }
    if let Some(total) = said(crate::memory::portions::the_total_for(&clean, memory)) {
        return Some(total);
    }
    if let Some(portion) = said(crate::memory::portions::reply_for(&clean, memory)) {
        return Some(portion);
    }
    if let Some(came_back) = said(crate::memory::returned::pool_for(&clean, memory)) {
        let whole = said(crate::memory::returned::the_whole_of_it(&clean, memory)).unwrap_or_else(|| came_back.clone());
        let _ = crate::memory::repeat::handed_back(memory, &whole);
        return Some(came_back);
    }
    let not_worked = py(crate::memory::tried::pool_for(&clean, memory));
    if not_worked.truthy() {
        return Some(pick(&not_worked.strs(), memory, ""));
    }
    if let Some(attempt_) = said(crate::memory::remedy::reply_for(&clean, memory)) {
        return Some(attempt_);
    }
    let acted = py(crate::memory::tried::acted_pool(&clean, memory));
    if acted.truthy() {
        return Some(pick(&acted.strs(), memory, ""));
    }
    if let Some(next_step) = said(crate::memory::carried_on::reply_for(&clean, memory)) {
        return Some(next_step);
    }
    if let Some(verdict) = _about_the_night_out(&clean, &tokens, memory).filter(|s| !s.is_empty()) {
        return Some(verdict);
    }
    if let Some(the_hours) = said(crate::memory::hours::answer_for(&clean, memory)) {
        return Some(the_hours);
    }
    if let Some(conversation) = _conversation(&clean, &tokens, memory).filter(|s| !s.is_empty()) {
        return Some(conversation);
    }
    if let Some(grumble) = _work_grumble(&clean, &tokens, memory).filter(|s| !s.is_empty()) {
        return Some(grumble);
    }
    if KICK_MYSELF.search(&clean).is_some() {
        return Some(pick(&AT_MYSELF_REPLIES.strs(), memory, &clean));
    }
    if _is_greeting(&tokens) {
        let name = _name(memory);
        return Some(_reply("greeting", memory, name.as_deref(), ""));
    }
    if _is_howareyou(&tokens) {
        let name = _name(memory);
        return Some(_reply("howareyou", memory, name.as_deref(), ""));
    }
    if let Some(joke) = _joke(&clean, &tokens, memory).filter(|s| !s.is_empty()) {
        return Some(joke);
    }
    let other = _other_feeling(&clean, memory);
    let fedup_at = FEDUP_AT.search(&said_).map(|m| m.start());
    let named = OTHER_PERSON.search(&said_).map(|m| m.start());
    let corrected = FED_UP_DENIED.search(&said_).is_some()
        && _bare_feeling(&STATE_WHEN.sub(" ", &FED_UP_DENIED.sub(" ", &said_))).is_some_and(|f| !f.is_empty());
    if let Some(at) = fedup_at {
        if other.is_none()
            && !corrected
            && FED_UP_ABOUT_THE_BOT.search(&said_[..at]).is_none()
            && FED_UP_WITH_A_PERSON.search(&said_).is_none()
            && !named.is_some_and(|n| n < at)
        {
            let name = _name(memory);
            if FED_UP_DENIED.search(&said_).is_some() {
                return Some(_reply("not_feeling", memory, name.as_deref(), ""));
            }
            if AT_MYSELF.search(&clean).is_some() {
                return Some(pick(&AT_MYSELF_REPLIES.strs(), memory, &clean));
            }
            if _not_coping(&clean) {
                return Some(_reply("not_coping", memory, name.as_deref(), ""));
            }
            if FED_UP_WHOLE_OF_IT.search(&said_).is_some() {
                return Some(_reply("not_coping", memory, name.as_deref(), ""));
            }
            _the_weight_said(memory, "annoyed");
            let tired_of = _fed_up_with(&said_);
            if !tired_of.is_empty() {
                let options = if tired_of.starts_with("being ") || tired_of.starts_with("having ") {
                    FEDUP_GERUND_SAFE.strs()
                } else {
                    FEDUP_REPLIES.strs()
                };
                let lead = FEDUP_LEAD.get(&_fedup_phrase(&said_, at)).map(py_str).unwrap_or_else(|| "Being tired of".into());
                let lines: Vec<String> = options.iter().map(|o| py_format(o, &[("thing", &tired_of), ("obj", "you"), ("lead", &lead)])).collect();
                return Some(pick(&lines, memory, &text_));
            }
            return Some(pick(&replies("venting"), memory, &text_));
        }
    }
    if MISS_ME.search(&clean).is_some() {
        return Some(pick(&MISS_YOU_REPLIES.strs(), memory, &text_));
    }
    if STILL_ON_THEM.search(&clean).is_some() {
        let name = _name(memory);
        return Some(_reply("missing", memory, name.as_deref(), ""));
    }
    if _fond_of_the_bot(&clean) {
        return Some(pick(&FOND_REPLIES.strs(), memory, &text_));
    }
    if (KIND_TO_ME.search(&clean).is_some() || _praised_the_answer(&clean)) && DOUBT_FRAME.search(&clean).is_none() {
        return Some(pick(&KIND_REPLIES.strs(), memory, &text_));
    }
    if let Some(judgement) = _judgement(&_as_said(&clean, understood.as_ref()), &tokens) {
        let seed = if judgement == "own_harsh" || judgement == "own_described" { clean.clone() } else { String::new() };
        let name = _name(memory);
        return Some(_reply(&judgement, memory, name.as_deref(), &seed));
    }
    if let Some(reaction) = _reaction(&clean, &tokens, memory) {
        let seed = if !reaction.starts_with("agree") { clean.clone() } else { format!("{clean}|{}", _news_before(memory)) };
        let name = _name(memory);
        return Some(_reply(&reaction, memory, name.as_deref(), &seed));
    }
    if VENTED.search(&clean).is_some() {
        let name = _name(memory);
        return Some(_reply("venting", memory, name.as_deref(), ""));
    }
    if crate::compose::goodnews::asks_for_news(&clean)
        && !(ANY_WORLD_NEWS_ON.search(&clean).is_some() && data::re("bot.compose.fallback.LIVE_NOT_WORLDS").search(&clean).is_none())
    {
        return Some(_reply("no_news", memory, None, ""));
    }
    if GOOD_NEWS_SAID.search(&clean).is_some() {
        let name = _name(memory);
        if NOT_LANDED.search(&clean).is_some() || NOT_GLAD_WORDS.strs().iter().any(|w| truthy(crate::compose::negated(&clean, w, 3))) {
            return Some(_reply("good_news_not_landed", memory, name.as_deref(), &clean));
        }
        return Some(_reply("happy", memory, name.as_deref(), ""));
    }
    let now = memory.understood.clone();
    if let Some(about) = _about_bot(&clean, now.as_ref(), Some(memory)).filter(|p| !p.is_empty()) {
        return Some(pick(&about, memory, ""));
    }
    if now.as_ref().is_some_and(|u| u.about_bot_kind.as_deref().is_some_and(|k| !k.is_empty())) {
        return None;
    }
    if let Some(opinion) = _opinion(&clean, &tokens, memory).filter(|s| !s.is_empty()) {
        return Some(opinion);
    }
    match _worry_someone(&said_, memory) {
        Some(WorrySomeone::Pass) => return None,
        Some(WorrySomeone::Pool(pool)) if !pool.is_empty() => return Some(pick(&pool, memory, "")),
        _ => {}
    }
    if let Some(early) = _early_start(&clean, memory).filter(|p| !p.is_empty()) {
        return Some(pick(&early, memory, ""));
    }
    if let Some(row) = _a_row(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(row);
    }
    if let Some(row_next) = _a_row_next(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(row_next);
    }
    if let Some(row_ending) = _a_row_ending(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(row_ending);
    }
    let someone = _about_someone(&clean, memory).filter(|s| !s.is_empty());
    let follow_up = _context_follow_up(&clean, memory).filter(|s| !s.is_empty());
    if let Some(someone) = &someone {
        let ordinary = SOMEONE_NORMAL.pairs().iter().any(|(_, v)| v.as_str() == Some(someone.as_str()));
        if ordinary && (SOMEONE_IS_IT_NORMAL.search(&clean).is_some() || SOMEONE_NEXT_STEP.search(&clean).is_some()) {
            return Some(someone.clone());
        }
    }
    if let Some(follow_up) = follow_up {
        return Some(follow_up);
    }
    if let Some(other) = &other {
        if tokens.len() <= 16 && !crate::compose::badnews::reads_as_news_with_a_name(&clean) {
            return Some(_someone_reply(memory, &other.feeling, &other.obj, &other.poss, other.plural, &other.thing, &said_));
        }
    }
    if let Some(standing) = _standing(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(standing);
    }
    let bare = _bare_feeling(&STATE_WHEN.sub(" ", &clean));
    let read_as = if as_said { understood.clone() } else { None };
    let mut feeling = _feeling(&clean, read_as.as_ref(), bare.clone(), false).filter(|f| !f.is_empty());
    if feeling.is_none() && KEEPS_HOW_THEY_ARE.search(&clean).is_some() && !_answers_knows(&text_, memory) {
        feeling = _feeling(&clean, read_as.as_ref(), bare.clone(), true).filter(|f| !f.is_empty());
    }
    let mood = |f: &Option<String>| f.as_deref().is_some_and(|f| in_set(&MOOD_KINDS, f));
    if mood(&feeling) && _going_on(&clean) {
        feeling = Some(if GOING_ON_LONG.search(&clean).is_some() { "ongoing" } else { "recurring" }.into());
    }
    let asks = understood.as_ref().is_some_and(|u| u.kind == "question");
    if feeling.as_deref() == Some("carrying") {
        feeling = if _answers_knows(&text_, memory) { None } else { Some("not_coping".into()) };
    }
    if mood(&feeling) && A_VERDICT_ASKED.search(&clean).is_some() {
        feeling = None;
    }
    if mood(&feeling) && _answers_knows(&text_, memory) {
        feeling = None;
    }
    if feeling.as_deref() == Some("rough_day") && asks && _answers_knows(&text_, memory) {
        feeling = None;
    }
    let mut like = _feel_like(&clean).filter(|l| !l.is_empty());
    if like.is_some() && _a_feeling_riding_on_the_thread(&clean, memory) {
        like = None;
    }
    if let Some(l) = like.as_deref().filter(|l| *l != "feel_like") {
        let name = _name(memory);
        return Some(_reply(l, memory, name.as_deref(), &clean));
    }
    if let Some(someone) = &someone {
        if asks && tokens.len() <= 22 {
            return Some(someone.clone());
        }
    }
    if let Some(mut f) = feeling.clone() {
        if (bare.as_deref().is_some_and(|b| !b.is_empty()) || _about_the_person(understood.as_ref()))
            && (chatty || tokens.len() <= 12 || !asks)
        {
            if f == "ongoing" || f == "recurring" {
                _the_weight_said(memory, &f);
                let turn_no = memory.turns.len() as i64;
                let advice = memory.get("mood_advice").cloned().unwrap_or(Value::None);
                if advice == Value::Int(turn_no - 1) {
                    memory.set("mood_advice", turn_no);
                    return Some(pick(&replies("recurring_next"), memory, &clean));
                }
                memory.set("mood_advice", turn_no);
                return Some(pick(&replies(&f), memory, &clean));
            }
            if f == "annoyed" && AT_MYSELF.search(&clean).is_some() {
                return Some(pick(&AT_MYSELF_REPLIES.strs(), memory, &clean));
            }
            if f == "happy" && NOT_PROUD.search(&clean).is_some() {
                f = "guilty".into();
            } else if f == "happy" && PROUD_OF_MYSELF.search(&clean).is_some() && PROUD_OF_ANOTHER.search(&clean).is_none() {
                f = "proud".into();
            }
            let mut f = Some(f);
            if f.as_deref() == Some("happy") && DOUBT_FRAME.search(&clean).is_some() && _praised_the_answer(&clean) {
                f = None;
            }
            if f.as_deref() == Some("confused") {
                if let Some(plainer) = said(crate::memory::meant::answer_for(&clean, memory)) {
                    let low = plainer.to_lowercase();
                    if MEANT_LEADS.strs().iter().any(|lead| low.starts_with(lead.as_str())) {
                        return Some(plainer);
                    }
                }
            }
            // Python: `_reply(feeling, ...)` with `feeling` None after the doubt check raises
            // TypeError in `REPLIES[None]`: the module is skipped for the turn.
            let f = f.unwrap_or_else(|| panic!("KeyError: None"));
            let name = _name(memory);
            if f == "happy" || f == "anxious" || f == "proud" {
                return Some(_reply(&f, memory, name.as_deref(), &clean));
            }
            let seed = if ["not_coping", "bad_day", "tears", "long_stretch"].contains(&f.as_str()) { clean.clone() } else { String::new() };
            return Some(_reply(&f, memory, name.as_deref(), &seed));
        }
    }
    if let Some(l) = like {
        let name = _name(memory);
        return Some(_reply(&l, memory, name.as_deref(), &clean));
    }
    if let Some(against) = _against_me(&clean) {
        let name = _name(memory);
        return Some(_reply(against, memory, name.as_deref(), &clean));
    }
    if let Some(stuck) = _stuck(&clean, memory).filter(|s| !s.is_empty()) {
        if tokens.len() <= 16 && !(stuck == "effort" && _answers_knows(&text_, memory)) {
            let own = format!("stuck_{stuck}");
            let mut kind = if REPLIES.get(&own).is_some() { own } else { "stuck".to_string() };
            if ["stuck_eating", "stuck_sleep", "stuck"].contains(&kind.as_str()) && LONG_SPAN.search(&clean).is_some() {
                for days in [format!("stuck_{stuck}_days"), "stuck_sleep_days".to_string()] {
                    if REPLIES.get(&days).is_some() {
                        kind = days;
                        break;
                    }
                }
            }
            let name = _name(memory);
            return Some(_reply(&kind, memory, name.as_deref(), ""));
        }
    }
    if _my_night(&clean) && tokens.len() <= 18 {
        return Some(pick(&replies("stuck_sleep"), memory, &clean));
    }
    if let Some(someone) = _about_someone(&clean, memory).filter(|s| !s.is_empty()) {
        if tokens.len() <= 22 {
            return Some(someone);
        }
    }
    if let Some(news) = _news(&clean, &tokens, memory).filter(|s| !s.is_empty()) {
        return Some(news);
    }
    if _question_coming(understood.as_ref()) {
        let name = _name(memory);
        return Some(_reply("opening", memory, name.as_deref(), ""));
    }
    if let Some(thing) = _their_thing(&clean, &tokens, memory, understood.as_ref()).filter(|s| !s.is_empty()) {
        return Some(thing);
    }
    if SAY_SOMETHING.search(&THOUGH_AT_THE_END.sub("", &clean)).is_some() && ASK_WHETHER_TO_SPEAK.search(&clean).is_none() {
        let name = _name(memory);
        return Some(_reply("say_something", memory, name.as_deref(), ""));
    }
    if _says_dont_know(&clean, &tokens) {
        let name = _name(memory);
        return Some(_reply(_idk_kind(memory), memory, name.as_deref(), ""));
    }
    if let Some(carried_on) = said(crate::memory::more::reply_for(&text_, memory)) {
        return Some(carried_on);
    }
    if let Some(plan) = _intent(&clean, &tokens, memory).filter(|p| !p.is_empty()) {
        if plan == "next_up" || plan == "next_rest" || plan == "next_act" {
            return Some(pick(&replies(&plan), memory, &clean));
        }
        let name = _name(memory);
        return Some(_reply(&plan, memory, name.as_deref(), ""));
    }
    if let Some(reading) = said(crate::compose::reading::reply_for(&clean, memory)) {
        return Some(reading);
    }
    if let Some(doubted) = said(crate::compose::doubt::reply_for(&clean, memory)) {
        return Some(doubted);
    }
    let verdict = said(crate::compose::verdict::reply_for(&clean, memory)).or_else(|| said(crate::compose::verdict::next_step(&clean, memory)));
    if let Some(v) = &verdict {
        if !data::get("bot.compose.verdict.QUESTION_BACK_LINES").contains_str(v) {
            return Some(v.clone());
        }
    }
    let mut comfort: Vec<String> = FEELING_SAID.iter().cloned().collect();
    for line in data::strs("bot.compose.fallback.COMFORT_SAID") {
        if !FEELING_SAID.contains(&line) {
            comfort.push(line);
        }
    }
    if let Some(step) = said(crate::compose::step_when_low(&clean, memory, &comfort)) {
        return Some(step);
    }
    if let Some(v) = verdict {
        return Some(v);
    }
    if let Some(held) = said(crate::memory::thread::checking_the_verdict(&clean, memory)) {
        return Some(held);
    }
    if let Some(checked) = said(crate::memory::thread::checking_a_fact(&clean, memory)) {
        return Some(checked);
    }
    if let Some(judged) = _judged(&clean, memory).filter(|s| !s.is_empty()) {
        return Some(judged);
    }
    if tokens.iter().any(|t| t == "help") && tokens.iter().all(|t| in_set(&HELP_ONLY, t)) {
        let (nod, _) = _the_nod(memory);
        if nod == "ack" {
            let name = _name(memory);
            return Some(_reply("help_ask", memory, name.as_deref(), &clean));
        }
    }
    if !live_emergency {
        if let Some(job_step) = _the_ask_after_a_job_that_went(&clean, memory).filter(|s| !s.is_empty()) {
            return Some(job_step);
        }
    }
    let steps = if live_emergency { Value::None } else { py(crate::memory::stuck::pool(&clean, memory)) };
    if steps.truthy() {
        if let Some(again_steps) = said(crate::memory::repeat::answer_for_text(&text_, memory)) {
            return Some(again_steps);
        }
        return Some(pick(&steps.strs(), memory, &clean));
    }
    if data::re("bot.memory.repeat.A_TOLD_BACK").search(&clean).is_some() {
        if let Some(told) = said(crate::memory::repeat::the_figure_they_gave_text(&text_, memory)) {
            return Some(told);
        }
    }
    if let Some(short) = _short_talk(&clean, &tokens, memory).filter(|s| !s.is_empty()) {
        if short == "again_what" {
            if let Some(steps) = _the_steps_again(memory).filter(|s| !s.is_empty()) {
                return Some(steps);
            }
        }
        let name = _name(memory);
        let seed = if short == "overloaded" { clean.clone() } else { String::new() };
        return Some(_reply(&short, memory, name.as_deref(), &seed));
    }
    if let Some(again) = said(crate::memory::repeat::answer_for_text(&text_, memory)) {
        return Some(again);
    }
    if let Some(what) = said(crate::memory::meant::answer_for(&clean, memory)) {
        return Some(what);
    }
    let heavy = if _a_bare_acknowledgement(&clean) { _the_heavy_line(memory) } else { String::new() };
    if !heavy.is_empty() {
        let name = _name(memory);
        return Some(_reply("nod_after_news", memory, name.as_deref(), &heavy));
    }
    if _a_plain_good_answer(&clean, memory) {
        let name = _name(memory);
        return Some(_reply("positive", memory, name.as_deref(), ""));
    }
    if HOW_THEY_ARE_LINE.match_(&clean).is_some() && !_denies_a_good_word(&tokens) {
        let name = _name(memory);
        return Some(_reply("positive", memory, name.as_deref(), ""));
    }
    if NOT_TOO_BAD.search(&clean).is_some() && tokens.len() <= 8 {
        let (nod, nod_seed) = _the_nod(memory);
        let name = _name(memory);
        return Some(_reply(nod, memory, name.as_deref(), &nod_seed));
    }
    if understood.as_ref().is_some_and(|u| u.agrees_to_try) && tokens.len() <= 10 {
        let name = _name(memory);
        return Some(_reply("try_ack", memory, name.as_deref(), &clean));
    }
    if !_mid_chat(memory) {
        let asked = _a_judgement_asked(&clean);
        if !asked.is_empty() {
            let name = _name(memory);
            return Some(_reply(&asked, memory, name.as_deref(), &clean));
        }
    }
    if let Some(u) = understood.as_ref() {
        if u.asks_a_verdict {
            let on = if u.clean.is_empty() { text_.as_str() } else { u.clean.as_str() };
            if crate::compose::verdict::answers_the_ask(on, memory) {
                return None;
            }
        }
    }
    if let Some((who, _)) = _somebody_thinks_i_should(&clean) {
        if !crate::compose::fallback::_a_relationship_ending(&clean) {
            return Some(pick(&_thinks_pool(&who), memory, &text_));
        }
    }
    if _somebody_recommended_a_thing(&clean).is_some_and(|r| !r.is_empty()) {
        return Some(pick(&SECONDHAND_PRAISE_REPLIES.strs(), memory, &text_));
    }
    if !chatty || live_emergency {
        if !live_emergency {
            if let Some(carried) = said(crate::memory::repeat::carried_on_text(&text_, memory)) {
                return Some(carried);
            }
        }
        return None;
    }
    if understood.as_ref().is_some_and(|u| u.agreement.as_deref().is_some_and(|a| !a.is_empty())) {
        return None;
    }
    if understood.as_ref().is_some_and(|u| u.about_bot_kind.as_deref().is_some_and(|k| !k.is_empty())) {
        return None;
    }
    let name = _name(memory);
    if laughed || tokens.iter().any(|t| in_set(&LAUGH_WORDS, t)) {
        return Some(_reply("laugh", memory, name.as_deref(), ""));
    }
    if tokens.iter().any(|t| t == "worries" || t == "problem") {
        let (nod, nod_seed) = _the_nod(memory);
        return Some(_reply(nod, memory, name.as_deref(), &nod_seed));
    }
    if _only_a_refusal(&clean, &tokens) {
        return Some(_reply("no", memory, name.as_deref(), ""));
    }
    let has = |w: &str| tokens.iter().any(|t| t == w);
    if has("idk") || has("dunno") || (has("not") && has("sure")) || (has("dont") && has("know")) {
        return Some(_reply(_idk_kind(memory), memory, name.as_deref(), ""));
    }
    if NOT_GOOD.match_(&clean).is_some() {
        return Some(_reply("bad_day", memory, name.as_deref(), ""));
    }
    const SELF_POSITIVE: [&str; 15] =
        ["i", "im", "am", "so", "very", "really", "doing", "today", "feeling", "feel", "too", "not", "bad", "ok", "okay"];
    let self_positive = (has("i") || has("im"))
        && !_denies_a_good_word(&tokens)
        && tokens.iter().all(|t| in_set(&POSITIVE, t) || SELF_POSITIVE.contains(&t.as_str()));
    if self_positive {
        return Some(_reply("positive", memory, name.as_deref(), ""));
    }
    if let Some(answered) = said(crate::compose::reaction::reply_for(&clean, memory)) {
        return Some(answered);
    }
    let (nod, nod_seed) = _the_nod(memory);
    if NOT_A_NOD.match_(&clean).is_some() && crate::compose::fallback::_a_good_time(&clean, memory) {
        return None;
    }
    Some(_reply(nod, memory, name.as_deref(), &nod_seed))
}

inline_re!(FEDUP_AT, r"\b(done with|fed up|browned off|peeved|sick of|tired of|had enough|had it with)\b");
inline_re!(THOUGH_AT_THE_END, r"\s+(?:though|tho)$");
inline_re!(NOT_TOO_BAD, r"\bnot (?:too |half )?bad\b");
res!(
    COLD_SYMPTOM, COLD_NAMED, COMPLAINT_ONLY, ILL_SELF, COLD_SORE, COLD_SORE_FEELING,
    BODY_BEFORE_AN_EVENT, HEDGED_LOW_LINE, HEDGED_OK_LINE, ANSWERS_HOW_ARE_YOU, THANKS_FOR,
    KICK_MYSELF, MISS_ME, STILL_ON_THEM, KIND_TO_ME, DOUBT_FRAME, ANY_WORLD_NEWS_ON,
    GOOD_NEWS_SAID, AT_MYSELF, FED_UP_DENIED, FED_UP_ABOUT_THE_BOT, FED_UP_WITH_A_PERSON,
    FED_UP_WHOLE_OF_IT, KEEPS_HOW_THEY_ARE, GOING_ON_LONG, A_VERDICT_ASKED, NOT_PROUD,
    PROUD_OF_MYSELF, PROUD_OF_ANOTHER, LONG_SPAN, HOW_THEY_ARE_LINE, NOT_GOOD, SAY_SOMETHING,
    ASK_WHETHER_TO_SPEAK,
);
sets!(ADDRESSED, THANKS_NOISE, ACK_WORDS, LAUGH_WORDS, MOOD_KINDS, HELP_ONLY, FEELING_SAID);
vals!(MISS_YOU_REPLIES, FOND_REPLIES, KIND_REPLIES, NOT_GLAD_WORDS, SECONDHAND_PRAISE_REPLIES);

#[cfg(test)]
#[path = "chat_difftest.rs"]
mod diff_tests;
