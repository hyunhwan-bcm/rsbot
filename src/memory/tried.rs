//! Port of `bot/memory/tried.py`: a follow-up that says the last answer did not work.
//!
//! "i tried that and it didn't work", "still blocked", "no luck" is about the answer the bot has
//! just given. This module reads the line as what it is and says the next thing: the step after
//! the one the person has done, or - when the home fix has done all it can - who does it for a
//! living. The trade is looked up from the words of the answer that did not work, not from the
//! new line. Module data (patterns, pools, word lists) comes from the exported bundle.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::facts::py_str;
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

const I: u32 = crate::py::re::I;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.tried.{name}"))
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("bot.memory.tried.{name} is not a regex"))
}

macro_rules! patterns {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<&'static Re> = LazyLock::new(|| re(stringify!($name)));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(pub static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($name)).str_set());)*
    };
}

patterns!(
    NOT_WORKED, STILL_ONLY, TRIED, MADE_IT_WORSE, POINTS_AT_THE_THING, ELSEWHERE, YET, IMPROVED,
    WORSE, WORD, STEP_ALREADY, STEP_TRIED, STEP_LAST, STEP_DONE, STEP_DID, PAST_WORD,
    PAST_STEP_ED, DOING_WORD, TAIL_WORD, PRONOUN_TAIL, CLAUSE, STILL_STATE, STILL_ASKS,
    PROBLEM_OR_FIX, FAULT, HEALTH, MARK, BUYING, STILL_SPECIFIC, NAMED_TRY, ACTED_SAID,
    SAY_ADVICE, TOLD_THE_BOT, STEP_TAIL_NOUN, PAST_STEP, MOVES, A_CONDITION, KEEPS_IT_RIGHT,
    IN_CARE,
);
pools!(
    ANSWER_MODULES, ASK_WHAT_TRIED, STILL_GOING_ON, SAME_STILL, ASK_NEXT, STILL_ASK, END, LEADS,
    STEP_LEADS, ALREADY_LEADS, DONE_LEADS, PRESENT_LEADS, ACTED_REPLIES, NEXT_LEADS, IN_CARE_ASK,
    IN_CARE_ASK_UNNAMED, IN_CARE_SELF,
);
word_sets!(
    POINTS_AT_IT, STEP_VAGUE, STEP_EMPTY, SHORT_PAST, PAST_IRREGULAR, MORE_STEP_VERBS,
    NOT_A_STATE, SAME_STEP, STEP_TIMING, NEXT_ASK_WORDS, SHE, HE, STILL_ABOUT_A_PERSON, POINTING,
    A_PERSON, PEOPLE_AGED,
);

/// `STEP_WORDS`: a step longer than this many words is a paragraph, not a step.
static STEP_WORDS: LazyLock<usize> = LazyLock::new(|| g("STEP_WORDS").as_int().unwrap_or(22) as usize);

/// One row of `TRADES`: the words an answer can only be about, who does it for a living, and
/// the kind of evidence the person's own words have to carry (`None`: no evidence needed).
pub struct Trade {
    pub entry_words: Vec<String>,
    pub trade: String,
    pub needs: Option<String>,
}
pub static TRADES: LazyLock<Vec<Trade>> = LazyLock::new(|| {
    g("TRADES")
        .items()
        .iter()
        .map(|row| {
            let row = row.items();
            Trade {
                entry_words: row[0].strs(),
                trade: row[1].str().to_string(),
                needs: row[2].as_str().map(str::to_string),
            }
        })
        .collect()
});
/// `EVIDENCE`: the pattern each kind of evidence is read with.
static EVIDENCE: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    g("EVIDENCE").pairs().iter().filter_map(|(k, v)| Some((k.str().to_string(), v.as_re()?))).collect()
});

// The other modules' data this file reads.
static IMPERATIVES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.verdict.IMPERATIVES"));
static REPEAT_SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.SENTENCE"));
static REPEAT_ASKING_THEM: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.ASKING_THEM"));
static REPEAT_NEXT_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.NEXT_ASK"));
static TOPICS_RED_FLAG: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.RED_FLAG"));
static FACTS_KIN: LazyLock<HashSet<String>> = LazyLock::new(|| key_set(data::get("bot.memory.facts.KIN")));
static FACTS_PET_ANIMALS: LazyLock<HashSet<String>> =
    LazyLock::new(|| key_set(data::get("bot.memory.facts.PET_ANIMALS")));
static NAMED_PEOPLE_WORDS: LazyLock<HashSet<String>> =
    LazyLock::new(|| key_set(data::get("bot.memory.named.PEOPLE_WORDS")));

// The patterns PyBot writes inline.
static ALREADY_AT_THE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+already$", 0));
static TRYING_ON_THE_FRONT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:tried|used)\s+", 0));
static CONTRAST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+(but|and|though|although|yet)\s+", I));
static PAST_CONTRAST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+(?:but|and|though|although|yet)\s+", 0));
static OPENING_JOINT: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:and|but|or)\s+", I));
static THEN_AT_THE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r",?\s*\bthen$", 0));
static WITH_SOMEONE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwith\s+(?:my\s+|our\s+|the\s+)?([a-z']+)", 0));
static I_AM_IN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi(?:'?m| am)\b.*\bin\b", 0));
static OPENS_ON_IM: LazyLock<Re> = LazyLock::new(|| Re::new(r"^im\b", 0));

/// The characters a piece of an answer is trimmed of.
const PIECE_ENDS: &str = " .,;:";

/// Python truthiness of what another file's function returns, whichever type its contract has.
trait PyTrue {
    fn py_true(&self) -> bool;
}
impl PyTrue for bool {
    fn py_true(&self) -> bool {
        *self
    }
}
impl PyTrue for Value {
    fn py_true(&self) -> bool {
        self.truthy()
    }
}

/// `repeat._says_what_to_do(part)`.
fn says_what_to_do(part: &str) -> bool {
    crate::memory::repeat::_says_what_to_do(part).py_true()
}

/// `set(value)`: a dict's keys, or the strings of a list, tuple or set.
fn key_set(value: &Value) -> HashSet<String> {
    match value {
        Value::Dict(d) => d.iter().filter_map(|(k, _)| k.as_str().map(str::to_string)).collect(),
        other => other.str_set(),
    }
}

/// What iterating a Python value gives: the items of a list, tuple or set, a dict's keys, a
/// string's characters; nothing for a value that is falsy (`value or ()`).
fn py_iter(value: Option<&Value>) -> Vec<Value> {
    match value {
        Some(Value::Dict(d)) => d.iter().map(|(k, _)| k.clone()).collect(),
        Some(Value::Str(s)) => s.chars().map(|c| Value::from(c.to_string())).collect(),
        Some(v) => v.items().to_vec(),
        None => Vec::new(),
    }
}

/// The memory as a Python-like dict value (for the contract functions that take one).
fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// `pick(options, memory, seed_text)` from `bot.compose`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// `not memory`: the dict has no key at all.
fn is_empty(memory: &Memory) -> bool {
    memory.understood.is_none()
        && memory.intent_wants.is_none()
        && !memory.extra_had_intent
        && memory.asked_now.is_none()
        && memory.extra.is_empty()
        && memory.turns.is_empty()
}

/// `template.format(name=value, ...)` for the `{name}` fields the pools use, read in one pass
/// so a value is never itself read as a field.
fn fmt(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 64);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) => {
                let name = &after[..close];
                match fields.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => out.push_str(v),
                    None => panic!("KeyError: '{name}'"),
                }
                rest = &after[close + 1..];
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// `s[:1].lower() + s[1:]`.
fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// `len(line.split())`.
fn word_count(line: &str) -> usize {
    pt::split_ref(line).len()
}

/// `str(turn.get("module") or "")`.
fn module_of(turn: &Turn) -> &str {
    turn.module.as_deref().unwrap_or("")
}

/// `memory.get(key) == n` for an int `n`.
fn is_the_int(value: Option<&Value>, n: i64) -> bool {
    match value {
        Some(Value::Int(i)) => *i == n,
        Some(Value::Bool(b)) => *b as i64 == n,
        Some(Value::Float(f)) => *f == n as f64,
        _ => false,
    }
}

/// Python: bot/memory/tried.py:354 `words_of(text)`
pub fn words_of(text: &str) -> Vec<String> {
    WORD.findall_str(&pt::lower(text).replace('\u{2019}', "'"))
}

/// `" ".join(words_of(text))`.
fn the_words(text: &str) -> String {
    words_of(text).join(" ")
}

/// Python: bot/memory/tried.py:358 `reads_as_not_worked(text)`
///
/// True when the line is a report that the last answer did not do the job.
pub fn reads_as_not_worked(text: &str) -> bool {
    let line = the_words(text);
    if line.is_empty() || word_count(&line) > 14 {
        return false;
    }
    if ELSEWHERE.is_match(&line) {
        return false;
    }
    // A verdict of its own - it worked, or it made things worse - is not this report.
    if IMPROVED.is_match(&line) || WORSE.is_match(&line) {
        return false;
    }
    // "it didnt work yet" is a fix still in progress, so the trade has not been earned.
    if YET.is_match(&line) {
        return false;
    }
    if NOT_WORKED.is_match(&line) {
        return true;
    }
    if MADE_IT_WORSE.is_match(&line) && POINTS_AT_THE_THING.is_match(&line) {
        return true;
    }
    TRIED.is_match(&line)
}

/// Python: bot/memory/tried.py:406 `the_still_state(text, memory)`
///
/// The state a "still ..." line reports on the thing, or "" when it is not that report. The
/// state has to be one the chat has already named - in the person's own line or in the words
/// their line was answered by.
pub fn the_still_state(text: &str, memory: &Memory) -> String {
    let said = the_words(text);
    if said.is_empty() || word_count(&said) > 7 || said.ends_with('?') {
        return String::new();
    }
    if STILL_ASKS.match_(&said).is_some() {
        return String::new();
    }
    let Some(found) = STILL_STATE.search(&said) else {
        return String::new();
    };
    let state: Vec<String> =
        words_of(found.name_str("state")).into_iter().filter(|word| !STEP_EMPTY.contains(word)).collect();
    if state.is_empty() || state.iter().any(|word| NOT_A_STATE.contains(word)) {
        return String::new();
    }
    let mut named: HashSet<String> = words_of(&person_said(memory)).iter().map(|word| _stem(word)).collect();
    named.extend(last_answer_words(memory).iter().map(|word| _stem(word)));
    if !state.iter().any(|word| named.contains(&_stem(word))) {
        return String::new();
    }
    // A sore back that is "still" sore is the complaint again, not a report on a fix, and the
    // same goes for feeling ill, tired or low: no home fix is what was being tried.
    if HEALTH.is_match(&state.join(" ")) {
        return String::new();
    }
    state.join(" ")
}

/// Python: bot/memory/tried.py:437 `still_the_state(text, memory)`
pub fn still_the_state(text: &str, memory: &Memory) -> bool {
    !the_still_state(text, memory).is_empty()
}

/// Python: bot/memory/tried.py:447 `might_be_a_report(text)`
///
/// A cheap look before the readers below: does the line have any word that could make it a
/// report on the answer, a step already done, or an ask about the step handed back?
pub fn might_be_a_report(text: &str) -> bool {
    let said = the_words(text);
    if said.is_empty() || word_count(&said) > 20 {
        return false;
    }
    NOT_WORKED.is_match(&said)
        || MADE_IT_WORSE.is_match(&said)
        || TRIED.is_match(&said)
        || STILL_STATE.is_match(&said)
        || STEP_DONE.is_match(&said)
        || STEP_DID.is_match(&said)
        || YET.is_match(&said)
        || NAMED_TRY.is_match(&said)
}

/// `str(trade or "").split(",")[0].strip().lower()`: the trade's name without its reason.
fn trade_name(trade: &str) -> String {
    pt::lower(pt::strip(trade.split(',').next().unwrap_or("")))
}

/// Python: bot/memory/tried.py:456 `names_a_trade(text)`
///
/// True when the line already hands the thing over to a trade.
pub fn names_a_trade(text: &str) -> bool {
    let said = the_words(text);
    TRADES.iter().any(|row| {
        let name = trade_name(&row.trade);
        !name.is_empty() && said.contains(&name)
    })
}

/// Python: bot/memory/tried.py:466 `after_an_answer(memory)`
///
/// True when the bot's last turn was an answer, so "still blocked" is about that answer.
pub fn after_an_answer(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let Some(last) = turns.last() else {
        return false;
    };
    let module = module_of(last);
    if ANSWER_MODULES.iter().any(|m| m == module) {
        return true;
    }
    if module != "chat" {
        return false;
    }
    // A step handed back out of the answer just given is that answer carrying on, so a report
    // on it is a report on the answer.
    if is_the_int(memory.get("handed_back_step"), turns.len() as i64 - 1) {
        return true;
    }
    // A reply that already named the trade, or asked what the thing is doing, is the answer
    // this report is about.
    names_a_trade(&last.bot) || asked_what_its_doing(memory)
}

/// Python: bot/memory/tried.py:491 `last_answer_words(memory)`
///
/// The words the person used for the thing, so the trade can be found from them. Only the
/// person's own words: the question that was answered and the words the entry was reached by.
pub fn last_answer_words(memory: &Memory) -> HashSet<String> {
    let mut said: Vec<String> = _the_lines(memory);
    if let Some(last) = memory.get("last_answer").filter(|last| last.is_dict()) {
        if let Some(keys @ (Value::List(_) | Value::Tuple(_) | Value::Set(_))) = last.get("keys") {
            said.extend(keys.items().iter().map(py_str));
        }
    }
    words_of(&said.join(" ")).into_iter().collect()
}

/// Python: bot/memory/tried.py:526 `asks_for_a_fix(memory)`
///
/// True when the turn that was answered was a problem or a how-to, not a recommendation.
pub fn asks_for_a_fix(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let Some(last) = turns.last() else {
        return false;
    };
    let asked = the_words(&last.user);
    if asked.is_empty() {
        return false;
    }
    if PROBLEM_OR_FIX.is_match(&asked) {
        return true;
    }
    if turns.len() > 1 {
        let before = the_words(&turns[turns.len() - 2].user);
        return PROBLEM_OR_FIX.is_match(&before);
    }
    false
}

/// Python: bot/memory/tried.py:580 `buying(memory)`
///
/// True when the turn that was answered was about choosing or buying, not about a fault.
pub fn buying(memory: &Memory) -> bool {
    match memory.turns.last() {
        Some(last) => BUYING.is_match(&the_words(&last.user)),
        None => false,
    }
}

/// Python: bot/memory/tried.py:588 `person_said(memory)`
///
/// The person's own last two lines, which are the only evidence for a fault.
pub fn person_said(memory: &Memory) -> String {
    _the_lines(memory).iter().filter(|line| !line.is_empty()).map(|line| the_words(line)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/tried.py:598 `_the_lines(memory)`
///
/// The person's own lines that can name the thing, newest first. The answered line always
/// counts; the line before it only when the answered line is itself a report that a fix did
/// not work, because that is when the thing was named a turn earlier.
pub fn _the_lines(memory: &Memory) -> Vec<String> {
    let turns = &memory.turns;
    let Some(last) = turns.last() else {
        return Vec::new();
    };
    let mut said = vec![last.user.clone()];
    if turns.len() > 1 && reads_as_not_worked(&the_words(&last.user)) {
        said.push(turns[turns.len() - 2].user.clone());
    }
    said
}

/// Python: bot/memory/tried.py:618 `_names(said, blob, entry_words)`
///
/// True when the words name the thing: the word itself, its plural or -ing, or a phrase.
pub fn _names(said: &HashSet<String>, blob: &str, entry_words: &[String]) -> bool {
    if entry_words.iter().any(|word| said.contains(word)) {
        return true;
    }
    for word in entry_words {
        if word.contains(' ') {
            if blob.contains(word.as_str()) {
                return true;
            }
        } else if pt::len(word) >= 5 && said.iter().any(|token| token.starts_with(word.as_str())) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/tried.py:647 `names_a_thing(memory)`
///
/// True when the person's own words name something a home fix could be about: the things that
/// can be waiting on a trade are the ones `TRADES` is made of.
pub fn names_a_thing(memory: &Memory) -> bool {
    let said: HashSet<String> = words_of(&person_said(memory)).into_iter().collect();
    if said.is_empty() {
        return false;
    }
    let mut sorted: Vec<&str> = said.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    let blob = sorted.join(" ");
    TRADES.iter().any(|row| _names(&said, &blob, &row.entry_words))
}

/// Python: bot/memory/tried.py:667 `has_evidence(text, memory)`
///
/// True when there is something for "what is it doing now" to be about: the turn that was
/// answered names a fault, or the report says what is still wrong, or names what was tried.
pub fn has_evidence(text: &str, memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else {
        return false;
    };
    let asked = the_words(&last.user);
    if FAULT.is_match(&asked) || MARK.is_match(&asked) || NAMED_TRY.is_match(&asked) {
        return true;
    }
    let report = the_words(text);
    if STILL_SPECIFIC.is_match(&report) || NAMED_TRY.is_match(&report) {
        return true;
    }
    MADE_IT_WORSE.is_match(&report) && POINTS_AT_THE_THING.is_match(&report)
}

/// Python: bot/memory/tried.py:687 `asked_what_its_doing(memory)`
///
/// True when the bot's last turn asked what the thing is doing now.
pub fn asked_what_its_doing(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else {
        return false;
    };
    let said = the_words(&last.bot);
    said.contains("doing now") || said.contains("happens now when you try")
}

/// Python: bot/memory/tried.py:696 `already_named(trade, memory)`
///
/// True when the last turn already said this trade, so saying it again is a loop. Only a reply
/// this module produced counts.
pub fn already_named(trade: &str, memory: &Memory) -> bool {
    let turns = &memory.turns;
    let name = trade_name(trade);
    if name.is_empty() {
        return false;
    }
    let said = turns[turns.len().saturating_sub(2)..]
        .iter()
        .filter(|t| module_of(t) == "chat")
        .map(|t| the_words(&t.bot))
        .collect::<Vec<_>>()
        .join(" ");
    said.contains(&name)
}

/// Python: bot/memory/tried.py:714 `_the_answered_line(memory)`
///
/// The line the last answer answered, which says what the turn was about.
pub fn _the_answered_line(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// Python: bot/memory/tried.py:720 `trade_for(memory)`
///
/// The trade that takes this over, as a phrase, or None when none fits.
pub fn trade_for(memory: &Memory) -> Option<String> {
    let said = last_answer_words(memory);
    let words = person_said(memory);
    let mut sorted: Vec<&str> = said.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    let blob = format!("{} {}", sorted.join(" "), words);
    // A fix that made the thing worse is evidence of the fault the trade deals with, in the
    // person's own words. It only counts when the answer that was followed was about a fault.
    let worse = MADE_IT_WORSE.is_match(&words) && POINTS_AT_THE_THING.is_match(&words);
    let about_a_fault = FAULT.is_match(&the_words(&_the_answered_line(memory)));
    for row in TRADES.iter() {
        if !_names(&said, &blob, &row.entry_words) {
            continue;
        }
        let Some(needs) = row.needs.as_deref() else {
            return Some(row.trade.clone());
        };
        let Some((_, pattern)) = EVIDENCE.iter().find(|(kind, _)| kind == needs) else {
            continue;
        };
        if pattern.is_match(&words) || (needs == "fault" && worse && about_a_fault) {
            return Some(row.trade.clone());
        }
    }
    None
}

/// Python: bot/memory/tried.py:773 `acted_pool(text, memory)`
///
/// The line after advice that was acted on: ask how it went, not what the advice was.
pub fn acted_pool(text: &str, memory: &Memory) -> Option<Vec<String>> {
    let line = the_words(text);
    if ACTED_SAID.match_(&line).is_none() || TOLD_THE_BOT.is_match(&line) {
        return None;
    }
    let last = memory.turns.last()?;
    // The advice can come from any module. A safety answer is the one exception, because a
    // line after it belongs to that thread.
    if module_of(last) == "safety" {
        return None;
    }
    let asked = the_words(&last.user);
    if !SAY_ADVICE.is_match(&asked) {
        return None;
    }
    Some(ACTED_REPLIES.clone())
}

/// Python: bot/memory/tried.py:806 `_up_to_the_contrast(what)`
///
/// The step the line names, with a second object kept and a change of subject dropped.
pub fn _up_to_the_contrast(what: &str) -> String {
    let Some(cut) = CONTRAST.search(what) else {
        return what.to_string();
    };
    let (head, joint, tail) = (&what[..cut.start()], cut.str(1), &what[cut.end()..]);
    if pt::lower(joint) == "and" && STEP_TAIL_NOUN.match_(pt::strip(tail)).is_some() {
        return what.to_string();
    }
    head.to_string()
}

/// The shape a step was said in: which of the `STEP_*` patterns read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepShape {
    Already,
    Tried,
    Last,
    Done,
    Did,
}

/// `" ".join(str(text or "").lower().split()).strip().strip(".,!")`.
fn the_line(text: &str) -> String {
    pt::strip_chars(pt::strip(&pt::squash(&pt::lower(text))), ".,!").to_string()
}

/// Python: bot/memory/tried.py:817 `_read_step_tried(text)`
///
/// (the step the person says they have done, the shape it was said in), or None when the line
/// names no step ("tried that" is the report, not a step of the answer).
pub fn _read_step_tried(text: &str) -> Option<(String, StepShape)> {
    let line = the_line(text);
    if line.is_empty() || line.contains('?') || word_count(&line) > 14 {
        return None;
    }
    // A line that also says the answer failed is the report, which has the trade and the "what
    // is it doing now" question; only the attempt on its own asks for the next step.
    if NOT_WORKED.is_match(&line) || IMPROVED.is_match(&line) || WORSE.is_match(&line) {
        return None;
    }
    let shapes: [(&Re, StepShape); 5] = [
        (*STEP_ALREADY, StepShape::Already),
        (*STEP_TRIED, StepShape::Tried),
        (*STEP_LAST, StepShape::Last),
        (*STEP_DONE, StepShape::Done),
        (*STEP_DID, StepShape::Did),
    ];
    for (pattern, shape) in shapes {
        let Some(found) = pattern.match_(&line) else {
            continue;
        };
        let what = pt::squash(found.name_str("what"));
        let what = pt::strip_chars(&what, " .,!'");
        // "i tried the cream already": the step is what the doing word is about, not the rest
        // of the sentence.
        let what = ALREADY_AT_THE_END.sub("", what);
        // "i tried everything" reaches the plain "i <doing word>" shape with the trying still
        // on the front, and it names no step.
        let what = TRYING_ON_THE_FRONT.sub("", &what);
        let what = _up_to_the_contrast(&what);
        let what = pt::strip_chars(&what, " .,!'");
        let words: Vec<String> = words_of(what).into_iter().filter(|w| !STEP_EMPTY.contains(w)).collect();
        if words.is_empty() || words.iter().all(|w| STEP_VAGUE.contains(w)) {
            continue;
        }
        return Some((what.to_string(), shape));
    }
    None
}

/// Python: bot/memory/tried.py:850 `the_step_tried(text)`
///
/// The step the person says they have done, as they put it, or None.
pub fn the_step_tried(text: &str) -> Option<String> {
    _read_step_tried(text).map(|(what, _)| what)
}

/// Python: bot/memory/tried.py:863 `_the_past_step(text, said)`
///
/// The step the line reports in the plain past tense, when it is one of the answer's own.
pub fn _the_past_step(text: &str, said: &str) -> Option<String> {
    let line = the_line(text);
    if line.is_empty() || said.is_empty() || line.contains('?') {
        return None;
    }
    if NOT_WORKED.is_match(&line) || IMPROVED.is_match(&line) || WORSE.is_match(&line) {
        return None;
    }
    let found = PAST_STEP.match_(&line)?;
    let what = pt::squash(found.name_str("what"));
    let what = pt::strip_chars(&what, " .,!'");
    let what = ALREADY_AT_THE_END.sub("", what);
    let what = PAST_CONTRAST.split(&what).into_iter().next().unwrap_or_default();
    let what = pt::strip_chars(&what, " .,!'");
    let words: Vec<String> = words_of(what).into_iter().filter(|w| !STEP_EMPTY.contains(w)).collect();
    if words.len() < 2 {
        return None;
    }
    let first = &words[0];
    if !PAST_IRREGULAR.contains(first) && !(PAST_STEP_ED.match_(first).is_some() || SHORT_PAST.contains(first)) {
        return None;
    }
    // The report has to be about the answer's own steps, and it has to share something real
    // with them: a past-tense line about another thing ("i went to the shop") is not this
    // answer.
    if !in_the_answer(what, said) {
        return None;
    }
    Some(what.to_string())
}

/// Python: bot/memory/tried.py:890 `_the_last_answer(memory)`
///
/// The knowledge base's last reply, or "" - the answer a report on it is about.
pub fn _the_last_answer(memory: &Memory) -> String {
    match memory.turns.last() {
        Some(last) if module_of(last) == "answers" => last.bot.clone(),
        _ => String::new(),
    }
}

/// Python: bot/memory/tried.py:901 `_stem(word)`
///
/// The word without its ending, so "topping", "topped" and "top" are the same step.
pub fn _stem(word: &str) -> String {
    for ending in ["ing", "ed", "es", "s"] {
        if pt::len(word) as isize - ending.len() as isize >= 3 && word.ends_with(ending) {
            let mut stem: Vec<char> = word[..word.len() - ending.len()].chars().collect();
            // "topp" and "bleedd": a doubled letter is where the ending was joined on.
            let n = stem.len();
            if (ending == "ing" || ending == "ed") && n > 3 && stem[n - 1] == stem[n - 2] {
                stem.pop();
            }
            return stem.into_iter().collect();
        }
    }
    word.to_string()
}

/// Python: bot/memory/tried.py:913 `_step_stems(text)`
pub fn _step_stems(text: &str) -> HashSet<String> {
    words_of(text).iter().filter(|w| !STEP_EMPTY.contains(*w) && !STEP_VAGUE.contains(*w)).map(|w| _stem(w)).collect()
}

/// The two words name the same step: equal, or one is the other with an ending that leaves
/// the step what it was ("clean" and "cleaned", but not "clean" and "cleaner").
fn same_step(got: &str, tried: &str) -> bool {
    if got == tried {
        return true;
    }
    if pt::len(got) >= 5 && tried.starts_with(got) && SAME_STEP.contains(&tried[got.len()..]) {
        return true;
    }
    pt::len(tried) >= 5 && got.starts_with(tried) && SAME_STEP.contains(&got[tried.len()..])
}

/// Python: bot/memory/tried.py:922 `_names_it(part, stems)`
///
/// True when this piece of the answer is the step the person has already done.
pub fn _names_it(part: &str, stems: &HashSet<String>) -> bool {
    words_of(part).iter().any(|word| {
        let got = _stem(word);
        stems.iter().any(|tried| same_step(&got, tried))
    })
}

/// Python: bot/memory/tried.py:947 `_the_move_after(part, stems)`
///
/// The move left in this part once the step the person has done is taken out of it.
pub fn _the_move_after(part: &str, stems: &HashSet<String>) -> String {
    _the_move_pieces(part, stems).1
}

/// Python: bot/memory/tried.py:952 `_the_move_pieces(part, stems)`
///
/// The piece that names the step they did, and the moves left after it in the same part.
pub fn _the_move_pieces(part: &str, stems: &HashSet<String>) -> (String, String) {
    if A_CONDITION.match_(pt::strip(part)).is_some() {
        // "if the phone is two years old, have a look at battery health" is one step: the
        // condition and the doing belong together, so there is no move after it inside it.
        return (String::new(), String::new());
    }
    let pieces: Vec<String> = MOVES
        .split(part)
        .iter()
        .map(|p| pt::strip_chars(p, PIECE_ENDS).to_string())
        .filter(|p| !p.is_empty())
        .collect();
    // The move that holds most of what they said, and the last of those.
    let mut best = 0usize;
    let mut last: Option<usize> = None;
    for (i, piece) in pieces.iter().enumerate() {
        let words: HashSet<String> = words_of(piece).into_iter().collect();
        // Not `_stem`med here: the answer's words are matched as they are written.
        let hits = stems.iter().filter(|tried| words.iter().any(|w| same_step(w, tried))).count();
        if hits > 0 && hits >= best {
            best = hits;
            last = Some(i);
        }
    }
    let Some(last) = last else {
        return (String::new(), String::new());
    };
    let rest = &pieces[last + 1..];
    if rest.is_empty() {
        return (pieces[last].clone(), String::new());
    }
    let left = rest.join(" and ");
    if PRONOUN_TAIL.match_(&left).is_some() || (TAIL_WORD.match_(&left).is_some() && !_opens_a_step(&left)) {
        // "put it as central as the phone line allows" is the end of the step before it: on its
        // own the "it" points at a sentence that is not there.
        return (pieces[last].clone(), String::new());
    }
    let after = if _opens_a_step(&left) { left } else { String::new() };
    (pieces[last].clone(), after)
}

/// Python: bot/memory/tried.py:987 `_clauses(text)`
///
/// The move cut where one thing to do ends and the next begins; a cut inside brackets is not
/// one.
pub fn _clauses(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut piece: Vec<String> = Vec::new();
    let mut depth: i64 = 0;
    for chunk in CLAUSE.split(&pt::squash(text)) {
        depth += pt::count(&chunk, "(") as i64 - pt::count(&chunk, ")") as i64;
        if !piece.is_empty() && depth > 0 {
            piece.push(chunk);
            continue;
        }
        if !piece.is_empty() {
            out.push(piece.join(", "));
        }
        piece = vec![chunk];
    }
    if !piece.is_empty() {
        out.push(piece.join(", "));
    }
    out.iter().map(|p| pt::strip_chars(p, PIECE_ENDS).to_string()).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/memory/tried.py:1003 `_a_step(piece)`
///
/// One clause as a step: the next thing to do, not the paragraph it came out of ("" when it is
/// too long to be one).
pub fn _a_step(piece: &str) -> String {
    let mut piece = pt::strip_chars(piece, PIECE_ENDS).to_string();
    if word_count(&piece) > *STEP_WORDS {
        // What to do comes before the examples that follow a colon.
        let head = pt::strip_chars(piece.split(": ").next().unwrap_or(""), PIECE_ENDS).to_string();
        if !head.is_empty() && head != piece {
            piece = head;
        }
    }
    if !piece.is_empty() && word_count(&piece) > *STEP_WORDS {
        // "If the phone is two or more years old, have a look at battery health, because a worn
        // battery drains no matter what you switch off" is one step with its reason after it.
        let (head, comma, tail) = pt::partition(&piece, ", ");
        if !comma.is_empty() && A_CONDITION.match_(head).is_some() {
            let step = pt::strip_chars(tail.split(", ").next().unwrap_or(""), PIECE_ENDS);
            if !step.is_empty() && _opens_a_step(step) {
                piece = format!("{head}, {step}");
            }
        }
    }
    if piece.is_empty() || word_count(&piece) > *STEP_WORDS || pt::len(&piece) > 180 {
        return String::new();
    }
    piece
}

/// Python: bot/memory/tried.py:1033 `_opens_a_step(part)`
///
/// True when the piece opens on a doing word, which is what a step looks like.
pub fn _opens_a_step(part: &str) -> bool {
    let is_a_doing_word = |word: &String| IMPERATIVES.contains(word) || MORE_STEP_VERBS.contains(word);
    let words = words_of(part);
    let Some(first) = words.first() else {
        return false;
    };
    if is_a_doing_word(first) {
        return true;
    }
    // "if <condition>, <step>": the doing word is in the second clause.
    let (head, comma, tail) = pt::partition(pt::strip(part), ", ");
    if !comma.is_empty() && A_CONDITION.match_(head).is_some() {
        if let Some(first) = words_of(tail).first() {
            return is_a_doing_word(first);
        }
    }
    false
}

/// Python: bot/memory/tried.py:1052 `_the_step_after(thing, memory)`
///
/// The next step of the answer just given, in its own words, or "". Only an answer from the
/// knowledge base is walked.
pub fn _the_step_after(thing: &str, memory: &Memory) -> String {
    match memory.turns.last() {
        Some(last) if module_of(last) == "answers" => _walk_answer(&last.bot, thing, true),
        _ => String::new(),
    }
}

/// Python: bot/memory/tried.py:1068 `_the_steps_of(said)`
///
/// The answer cut into the pieces that could be a step, in the order it was written.
pub fn _the_steps_of(said: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    if said.is_empty() {
        return parts;
    }
    // The composer's own "move" is not used: it is cut down to the first instruction, and the
    // step after the one they tried is often the second half of that same sentence.
    for sentence in REPEAT_SENTENCE.split(pt::strip(said)) {
        for clause in _clauses(&sentence) {
            let piece = _a_step(&clause);
            if !piece.is_empty() && !parts.contains(&piece) {
                parts.push(piece);
            }
        }
    }
    parts
}

/// Python: bot/memory/tried.py:1086 `_list_heads(said)`
///
/// The pieces that only introduce the list that follows them: a sentence that ends at a colon
/// is a heading, not a step.
pub fn _list_heads(said: &str) -> HashSet<String> {
    let mut heads = HashSet::new();
    for sentence in REPEAT_SENTENCE.split(pt::strip(said)) {
        let (head, colon, rest) = pt::partition(&sentence, ":");
        if colon.is_empty() || pt::strip(rest).is_empty() {
            continue;
        }
        for clause in _clauses(head) {
            let piece = _a_step(&clause);
            if !piece.is_empty() {
                heads.insert(_flat(&piece));
            }
        }
    }
    heads
}

/// Python: bot/memory/tried.py:1115 `_object_stems(text)`
///
/// The words of a step that name the thing it is done to, leaving out the doing word.
pub fn _object_stems(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for w in words_of(text) {
        if STEP_EMPTY.contains(&w) || STEP_VAGUE.contains(&w) || STEP_TIMING.contains(&w) {
            continue;
        }
        if PAST_IRREGULAR.contains(&w) || SHORT_PAST.contains(&w) || PAST_STEP_ED.match_(&w).is_some() {
            continue;
        }
        if DOING_WORD.match_(&w).is_some() {
            continue;
        }
        out.insert(_stem(&w));
    }
    out
}

/// Python: bot/memory/tried.py:1129 `in_the_answer(thing, said)`
///
/// True when one of the answer's pieces names what the person says they have done.
pub fn in_the_answer(thing: &str, said: &str) -> bool {
    let stems = _step_stems(thing);
    if stems.is_empty() {
        return false;
    }
    let parts = _the_steps_of(said);
    if !parts.iter().any(|part| _names_it(part, &stems)) {
        return false;
    }
    // Sharing the doing word is not enough to name a step: the thing the step is done to has to
    // be in the answer too, when the line names one.
    let objects = _object_stems(thing);
    if !objects.is_empty() {
        return parts.iter().any(|part| _names_it(part, &objects));
    }
    true
}

/// Python: bot/memory/tried.py:1155 `_flat(text)`
pub fn _flat(text: &str) -> String {
    pt::strip_chars(&pt::squash(&pt::lower(text)), PIECE_ENDS).to_string()
}

/// A part that holds a step already handed over is that step's part, not a new one.
fn was_handed_over(flat: &str, seen: &[String]) -> bool {
    seen.iter().any(|was| flat == was || flat.contains(was.as_str()))
}

/// Python: bot/memory/tried.py:1159 `_the_first_step_left(said, seen, after='')`
///
/// The answer's first step that has not been handed back yet, or "". `after` is the step the
/// bot has just given: asked what comes next, only the parts after it are the answer.
pub fn _the_first_step_left(said: &str, seen: &[String], after: &str) -> String {
    let heads = _list_heads(said);
    let mut parts = _the_steps_of(said);
    if !after.is_empty() {
        let place = _flat(after);
        for i in 0..parts.len() {
            let flat = _flat(&parts[i]);
            if !place.is_empty() && (flat == place || flat.contains(&place) || place.contains(&flat)) {
                parts = parts[i + 1..].to_vec();
                break;
            }
        }
    }
    for part in &parts {
        let flat = _flat(part);
        if heads.contains(&flat) {
            continue;
        }
        if was_handed_over(&flat, seen) {
            continue;
        }
        if _opens_a_step(part) && says_what_to_do(part) {
            return part.clone();
        }
        // The answer's last move can be written as a condition ("and if it carries on get a
        // heating engineer to look at the pump"), which is a step like any other.
        let mut joined = OPENING_JOINT.sub("", &flat);
        if A_CONDITION.match_(&joined).is_some() && says_what_to_do(&joined) {
            if joined != flat {
                joined = pt::upper_first(&joined);
            }
            return joined;
        }
    }
    // A part after the step just given can say what to do without opening on the doing word ("A
    // kettle of hot water with a spoonful of bicarbonate and vinegar clears the rest"), and
    // that is what comes next.
    if !after.is_empty() {
        for part in &parts {
            let flat = _flat(part);
            if heads.contains(&flat) || was_handed_over(&flat, seen) {
                continue;
            }
            if REPEAT_ASKING_THEM.is_match(part) {
                continue; // a question is not a step
            }
            if says_what_to_do(part) {
                return part.clone();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/tried.py:1218 `_with_its_tail(part, parts, i)`
///
/// The step and the part after it, when the answer wrote them as one instruction. Only the
/// answer's own last part is joined, and only into one sentence.
pub fn _with_its_tail(part: &str, parts: &[String], i: usize) -> String {
    if i + 2 != parts.len() {
        return part.to_string();
    }
    let nxt = &parts[i + 1];
    if !says_what_to_do(nxt) || REPEAT_ASKING_THEM.is_match(nxt) {
        return part.to_string();
    }
    if A_CONDITION.match_(nxt).is_some() || _list_heads(nxt).contains(&_flat(nxt)) {
        return part.to_string();
    }
    if words_of(part).len() + words_of(nxt).len() > 40 {
        return part.to_string();
    }
    format!("{}, and {}", pt::rstrip_chars(part, PIECE_ENDS), lower_first(nxt))
}

/// A piece that opens on a pronoun object ("put it as central as the line allows") is the end
/// of the step before it, so it is never handed over on its own. A "then"/"if" piece is only
/// the tail when it does not hold a step of its own.
fn is_a_tail(part: &str) -> bool {
    PRONOUN_TAIL.match_(part).is_some() || (TAIL_WORD.match_(part).is_some() && !_opens_a_step(part))
}

/// Python: bot/memory/tried.py:1242 `_the_advice_after(parts)`
///
/// The answer's own line on keeping the thing right, among the parts still to be used.
pub fn _the_advice_after(parts: &[String]) -> String {
    for part in parts.iter().rev() {
        if words_of(part).len() < 5 {
            continue;
        }
        if REPEAT_ASKING_THEM.is_match(part) {
            continue; // a question is not a step
        }
        if TOPICS_RED_FLAG.match_(part).is_some() {
            continue;
        }
        if is_a_tail(part) {
            continue;
        }
        if KEEPS_IT_RIGHT.is_match(part) {
            return part.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/tried.py:1264 `_walk_answer(said, thing, wrap=True)`
///
/// The first step of this answer after the part that names what they have done, or "". `wrap`
/// is off for the ask after a step the bot has already handed over: two steps on, the answer's
/// first step is one the person has had or has done.
pub fn _walk_answer(said: &str, thing: &str, wrap: bool) -> String {
    let parts = _the_steps_of(said);
    let stems = _step_stems(thing);
    if parts.is_empty() || stems.is_empty() {
        return String::new();
    }
    let heads = _list_heads(said);
    // The answer has to be about the thing they are working on.
    if !parts.iter().any(|part| _names_it(part, &stems)) {
        return String::new();
    }
    // Everything up to the step they have done has been read and tried: the next step is the
    // one after the first part that names it. The first, not the last, because a good answer
    // names the step again at the end, and walking past that leaves nothing.
    let mut start = 0usize;
    let mut theirs_is_a_step = false;
    for (i, part) in parts.iter().enumerate() {
        if _names_it(part, &stems) {
            // The part that names it can hold the step they did and the one after it.
            let (named, the_move) = _the_move_pieces(part, &stems);
            if !the_move.is_empty() && says_what_to_do(&the_move) {
                return the_move;
            }
            // The piece that names it is the step of the answer, even when the part is written
            // round it.
            theirs_is_a_step = _opens_a_step(part) || _opens_a_step(&named);
            start = i + 1;
            break;
        }
    }
    if !theirs_is_a_step {
        // The part that names it is the answer saying why, not one of its steps, so the steps
        // after it are the answer's whole sequence rather than what is left of theirs.
        return if !wrap { _the_advice_after(&parts[start..]) } else { String::new() };
    }
    for (i, part) in parts.iter().enumerate().skip(start) {
        if _names_it(part, &stems) {
            // The same part can hold the step they did and the one after it.
            let the_move = _the_move_after(part, &stems);
            if !the_move.is_empty() && says_what_to_do(&the_move) {
                return the_move;
            }
            continue;
        }
        if REPEAT_ASKING_THEM.is_match(part) {
            continue; // a question is not a step
        }
        if TOPICS_RED_FLAG.match_(part).is_some() {
            // The warning is where the answer ends, not the next thing to do.
            continue;
        }
        // A piece that opens on "and", "not" or "if" is the tail of the step before it, not a
        // step of its own.
        if is_a_tail(part) {
            continue;
        }
        if heads.contains(&_flat(part)) {
            continue;
        }
        // A step opens on the doing word: "it's a 20-minute job and saves a lot of water" is
        // the answer explaining itself, not the next thing to do.
        if _opens_a_step(part) {
            return _with_its_tail(part, &parts, i);
        }
    }
    // Nothing follows the step they did: an answer gives its steps in one order and people
    // start in the middle of it.
    if !wrap {
        // The ask after a step the bot handed back, with nothing behind it that opens on a
        // doing word: the answer's own line on keeping the thing right is what comes next.
        return _the_advice_after(&parts[start..]);
    }
    for part in &parts[..start.saturating_sub(1)] {
        if is_a_tail(part) {
            continue;
        }
        if heads.contains(&_flat(part)) {
            continue;
        }
        if _opens_a_step(part) && says_what_to_do(part) {
            return part.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/tried.py:1368 `reads_as_a_step_done(text, memory)`
///
/// True when the line says a step of the answer just given is done, whatever comes next.
pub fn reads_as_a_step_done(text: &str, memory: &Memory) -> bool {
    if is_empty(memory) {
        return false;
    }
    let said = _the_last_answer(memory);
    let thing = match _read_step_tried(text) {
        // A step said as a plain past-tense report is read against the answer it reports on.
        None => match _the_past_step(text, &said) {
            Some(thing) => thing,
            None => return false,
        },
        Some((thing, _shape)) => {
            if !said.is_empty() && !in_the_answer(&thing, &said) {
                // "ive cleaned the bathroom" after the washing machine answer names a thing of
                // its own, and the answer never mentions it.
                return false;
            }
            thing
        }
    };
    if said.is_empty() {
        return false;
    }
    // The walk decides what is handed over; what it decides here is whether the turn is kept.
    if !_walk_answer(&said, &thing, true).is_empty() {
        return true;
    }
    // Nothing after their step, so the answer has been used up. The turn is kept only when the
    // whole answer was that one step; an answer with more steps than that keeps the line for
    // the knowledge base, which hands over the rest of its own answer.
    _the_steps_of(&said).len() <= 1
}

/// Python: bot/memory/tried.py:1410 `_as_a_lead(lead, thing, step)`
///
/// The lead filled in: the step opens lower down when it follows a comma.
pub fn _as_a_lead(lead: &str, thing: &str, step: &str) -> String {
    // The person is being addressed, so their own "my" has to turn into "your" before it goes
    // into a sentence about them.
    let thing = crate::memory::you(thing);
    let step = pt::strip_chars(step, PIECE_ENDS);
    // A step that ends on the joiner it was cut at hands over a "then" with nothing after it;
    // the word belongs to the part that comes next.
    let step = THEN_AT_THE_END.sub("", step);
    let mut step = pt::rstrip_chars(&step, " ,;:").to_string();
    let first = pt::split_ref(&step).first().copied().unwrap_or("").to_string();
    // The person's own "I", and a name in capitals ("GP", "NHS"), keep their shape.
    let keeps_its_shape =
        matches!(first.as_str(), "I" | "I'm" | "I've" | "I'd") || (pt::is_upper(&first) && pt::len(&first) > 1);
    if !keeps_its_shape {
        step = lower_first(&step);
    }
    let reply = fmt(
        lead,
        &[("thing", &thing), ("Thing", &pt::upper_first(&thing)), ("step", &step), ("Step", &pt::upper_first(&step))],
    );
    format!("{}.", pt::rstrip_chars(&reply, " ."))
}

/// Python: bot/memory/tried.py:1432 `next_step_pool(text, memory)`
///
/// The step after the one the person has tried, as a pool of one, or None (which leaves the
/// line to the report, which hands over the trade).
pub fn next_step_pool(text: &str, memory: &mut Memory) -> Option<Vec<String>> {
    if is_empty(memory) {
        return None;
    }
    let mut said_as_the_past = false;
    let (thing, shape) = match _read_step_tried(text) {
        None => {
            let thing = _the_past_step(text, &_the_last_answer(memory))?;
            said_as_the_past = true;
            (thing, None)
        }
        Some((thing, shape)) => {
            let answer = _the_last_answer(memory);
            if !answer.is_empty() && !in_the_answer(&thing, &answer) {
                // The answer is in hand and it never names what they cleaned.
                return None;
            }
            (thing, Some(shape))
        }
    };
    // A recommendation is not a set of steps: "ive tried the one i have" after advice about
    // which laptop to buy is not partway through the answer.
    if buying(memory) {
        return None;
    }
    let step = _the_step_after(&thing, memory);
    if step.is_empty() {
        return None;
    }
    // "ive tried the plunger" names the thing, so the lead has to say the trying; "ive cleaned
    // the filter" names the doing, so the lead can say it as done. Which one it is is read from
    // the phrase, not from the marker word the line happened to use.
    let says_the_doing = DOING_WORD.match_(&thing).is_some() && shape != Some(StepShape::Tried);
    let first_word = pt::lower(pt::split_ref(&thing).first().copied().unwrap_or(""));
    let as_a_habit =
        says_the_doing && !(PAST_WORD.match_(&first_word).is_some() || PAST_IRREGULAR.contains(&first_word));
    let leads: &[String] = if said_as_the_past {
        // "i ran a hot wash with vinegar": a plain past-tense report, so the lead says it back
        // with their own verb rather than "you've already ran".
        &DONE_LEADS
    } else if as_a_habit {
        &PRESENT_LEADS
    } else if says_the_doing {
        &ALREADY_LEADS
    } else {
        &STEP_LEADS
    };
    let reply = _as_a_lead(&pick(leads, memory, text), &thing, &step);
    // Which turn was the answer's continuation, so a report on it is read as a report on the
    // answer (`after_an_answer`). The step and the answer it came out of are kept too, so the
    // ask after it ("and then what") is answered with the part that comes next.
    let turn_count = memory.turns.len();
    let mut seen: Vec<Value> = py_iter(memory.get("handed_back_seen").filter(|v| v.truthy()));
    seen.push(Value::from(_flat(&step)));
    memory.set("handed_back_step", turn_count);
    memory.set("handed_back_text", step);
    memory.set("handed_back_seen", seen);
    if let Some(last) = memory.turns.last() {
        let answer = last.bot.clone();
        memory.set("handed_back_answer", answer);
    }
    Some(vec![reply])
}

/// Python: bot/memory/tried.py:1507 `_asks_for_the_next_part(text)`
///
/// True for "and then what" / "what else can i try", and not for a line naming a thing.
pub fn _asks_for_the_next_part(text: &str) -> bool {
    let line = the_line(text);
    if line.is_empty() || word_count(&line) > 7 {
        return false;
    }
    if REPEAT_NEXT_ASK.match_(&line).is_none() {
        return false;
    }
    words_of(&line).iter().all(|word| NEXT_ASK_WORDS.contains(word))
}

/// Python: bot/memory/tried.py:1519 `next_step_ask_pool(text, memory)`
///
/// The part of the answer after the step the bot just handed back, or None.
///
/// PyBot keeps `handed_back_seen` as `list(seen | {flat})`, a set written out as a list: its
/// order follows CPython's per-process string hashing, so it is not a fact about the chat.
/// The order kept here is the order the steps were handed back in.
pub fn next_step_ask_pool(text: &str, memory: &mut Memory) -> Option<Vec<String>> {
    if is_empty(memory) || !_asks_for_the_next_part(text) {
        return None;
    }
    let text_of = |key: &str| memory.get(key).filter(|v| v.truthy()).map(py_str).unwrap_or_default();
    let step = pt::strip(&text_of("handed_back_text")).to_string();
    let said = text_of("handed_back_answer");
    if step.is_empty() || said.is_empty() {
        return None;
    }
    let turn_count = memory.turns.len();
    // Only the turn right after it: two turns on, the conversation has moved.
    if !is_the_int(memory.get("handed_back_step"), turn_count as i64 - 1) {
        return None;
    }
    let mut seen: Vec<String> = Vec::new();
    for was in py_iter(memory.get("handed_back_seen").filter(|v| v.truthy())) {
        let was = py_str(&was);
        if !seen.contains(&was) {
            seen.push(was);
        }
    }
    let mut more = _walk_answer(&said, &step, false);
    if more.is_empty() {
        more = _the_first_step_left(&said, &seen, &step);
    }
    // The answer's line on keeping it right is handed back once. Asked again there is no next
    // part of the answer to give, and saying the same sentence with another lead on it is the
    // repeat this module exists to avoid.
    if more.is_empty() || pt::squash(&pt::lower(&more)) == pt::squash(&pt::lower(&step)) {
        return None;
    }
    let reply = _as_a_lead(&pick(&NEXT_LEADS, memory, text), "", &more);
    let flat = _flat(&more);
    if !seen.contains(&flat) {
        seen.push(flat);
    }
    memory.set("handed_back_step", turn_count);
    memory.set("handed_back_text", more);
    memory.set("handed_back_seen", seen);
    memory.set("handed_back_answer", said);
    Some(vec![reply])
}

/// Python: bot/memory/tried.py:1557 `_somebody_is_the_subject(memory)`
///
/// True when the chat is about a person or an animal rather than the person's own thing.
pub fn _somebody_is_the_subject(memory: &Memory) -> bool {
    let Some(topic) = memory.get("topic").filter(|topic| topic.is_dict()) else {
        return false;
    };
    let text = topic.get("text").filter(|v| v.truthy()).map(py_str).unwrap_or_default();
    let mut words: HashSet<String> = words_of(&text).into_iter().collect();
    for word in py_iter(topic.get("words").filter(|v| v.truthy())) {
        if word.truthy() {
            words.insert(pt::lower(&py_str(&word)));
        }
    }
    if words.is_empty() {
        return false;
    }
    if words.iter().any(|w| FACTS_KIN.contains(w) || NAMED_PEOPLE_WORDS.contains(w) || FACTS_PET_ANIMALS.contains(w)) {
        return true;
    }
    // A person or an animal the chat has already named: "bella is at the vet" then "still
    // there".
    let user = memory.get("user").filter(|v| v.truthy());
    let name_of = |entry: &Value| -> Option<String> {
        let name = entry.get("name").filter(|name| name.truthy())?;
        Some(pt::lower(&py_str(name)))
    };
    let mut names: HashSet<String> = HashSet::new();
    for entry in py_iter(user.and_then(|u| u.get("pets")).filter(|v| v.truthy())) {
        if entry.is_dict() {
            names.extend(name_of(&entry));
        }
    }
    for entry in py_iter(user.and_then(|u| u.get("others")).filter(|v| v.truthy())) {
        if entry.is_dict() && entry.get("name").is_some_and(Value::truthy) {
            names.extend(name_of(&entry));
        } else if let Some(name) = entry.as_str() {
            names.insert(pt::lower(name));
        }
    }
    words.iter().any(|w| names.contains(w))
}

/// Python: bot/memory/tried.py:1584 `still_tail(text, memory)`
///
/// The thing said again with "still" on it, or "": a state the chat has already named, or the
/// thing said whole with a word that only points at it ("still there" after a headache).
pub fn still_tail(text: &str, memory: &Memory) -> String {
    let state = the_still_state(text, memory);
    if !state.is_empty() {
        return state;
    }
    let line = the_words(text);
    if line.is_empty() || word_count(&line) > 6 {
        return String::new();
    }
    let Some(found) = STILL_ONLY.match_(&line) else {
        return String::new();
    };
    let tail = pt::strip(found.name_str("tail"));
    if !POINTS_AT_IT.contains(tail) {
        return String::new();
    }
    // The tail is the speaker's own thing: a chat about a person or an animal keeps its reply.
    if _somebody_is_the_subject(memory) {
        return String::new();
    }
    tail.to_string()
}

/// Python: bot/memory/tried.py:1617 `a_card_is_not_the_answer(text, memory)`
///
/// True when the card one word of this line belongs to must not answer it.
pub fn a_card_is_not_the_answer(text: &str, memory: &Memory) -> bool {
    let tail = still_tail(text, memory);
    if tail.is_empty() {
        return false;
    }
    if POINTS_AT_IT.contains(&tail) {
        // "still there" is the thing only when there is an answer behind it: on its own it is
        // "are you there?", and that is a card of nobody's.
        return after_an_answer(memory);
    }
    names_a_thing(memory) || after_an_answer(memory)
}

/// Python: bot/memory/tried.py:1639 `still_going_on(text, memory)`
///
/// The pool for a line that is only the thing again with "still" on it, or None.
pub fn still_going_on(text: &str, memory: &mut Memory) -> Option<Vec<String>> {
    let tail = still_tail(text, memory);
    if tail.is_empty() || buying(memory) {
        return None;
    }
    if POINTS_AT_IT.contains(&tail) {
        // "still there" is only the thing on the line before: on its own it is "are you there".
        if !after_an_answer(memory) {
            return None;
        }
        // A body or a feeling is not a machine, so the question SAME_STILL asks is the wrong
        // one for a headache. It is still going on, and that is the thing to ask about.
        if !names_a_thing(memory) {
            return Some(STILL_GOING_ON.clone());
        }
    } else if !names_a_thing(memory) {
        // A body or a feeling is not a thing with a next step of its own.
        return None;
    }
    // The store keeps out of every line the report readers can carry: a trade to hand over, a
    // step that carries on, the end of the home fixes. What is left is a line with no fix in
    // hand, and there the thing keeps the turn with the one question that tells the two apart.
    // (`pool is not ASK_WHAT_TRIED and pool is not SAME_STILL`: no other pool has their lines.)
    if let Some(pool) = pool_for(text, memory) {
        if pool != *ASK_WHAT_TRIED && pool != *SAME_STILL {
            return None;
        }
    }
    Some(SAME_STILL.clone())
}

/// Python: bot/memory/tried.py:1676 `reply_for(text, memory)`
///
/// The reply to a line that is only the thing again with "still" on it, or None.
pub fn reply_for(text: &str, memory: &mut Memory) -> Option<String> {
    let pool = still_going_on(text, memory).filter(|pool| !pool.is_empty())?;
    Some(pick(&pool, memory, text))
}

/// Python: bot/memory/tried.py:1692 `pool_for(text, memory)`
///
/// The pool the next step comes from, or None when the line is not a report on the answer.
pub fn pool_for(text: &str, memory: &mut Memory) -> Option<Vec<String>> {
    if let Some(care) = in_care_pool(text, memory).filter(|pool| !pool.is_empty()) {
        // A person of theirs is in a place of care and the line says they are still in it.
        return Some(care);
    }
    // `still_the_state` reads "still cold" after the shower; it deliberately drops a state that
    // is a body or a feeling, and it drops "still there", which is the thing said whole. Those
    // are read here so the report path stays shut and the store's own pool answers the line.
    if !(reads_as_not_worked(text) || still_the_state(text, memory) || !still_tail(text, memory).is_empty())
        || !after_an_answer(memory)
    {
        // The named step is read first: "ive tried the plunger" is not a report that the answer
        // failed, it is being partway through it, and the turn after it is the next step.
        if let Some(step) = next_step_pool(text, memory).filter(|pool| !pool.is_empty()) {
            return Some(step);
        }
        // The ask after a step the bot handed back out of the answer: "and then what" wants the
        // part that comes after it, not the answer's first step again.
        return next_step_ask_pool(text, memory).filter(|pool| !pool.is_empty());
    }
    // A "then I did X and now it is worse" is only a report on a fix when the turn that was
    // answered was a problem or a how-to.
    let report = the_words(text);
    let made_it_worse = MADE_IT_WORSE.is_match(&report) && POINTS_AT_THE_THING.is_match(&report);
    if made_it_worse && !asks_for_a_fix(memory) {
        return None;
    }
    // "that didnt help" about a purchase is a recommendation that was not to their taste.
    if buying(memory) {
        return None;
    }
    // The question has been asked and the answer is "still the same": what is left to say is
    // that a home fix is the wrong place to keep looking.
    if asked_what_its_doing(memory) {
        return Some(END.clone());
    }
    // A trade found in the person's own words is proof enough that the thing is theirs to fix.
    if let Some(trade) = trade_for(memory).filter(|trade| !trade.is_empty()) {
        if already_named(&trade, memory) {
            // Naming it twice is not a next step: the second report is a question about the
            // thing, not about who to ring.
            return Some(ASK_NEXT.clone());
        }
        return Some(LEADS.iter().map(|lead| fmt(lead, &[("trade", &trade)])).collect());
    }
    // A "still cold" line with no fix in hand is a report that the thing has not moved, and the
    // person has not said they tried anything, so the reply asks about the thing itself and
    // claims nothing about what has been done.
    let state = the_still_state(text, memory);
    if !state.is_empty() && !(asks_for_a_fix(memory) && has_evidence(text, memory)) {
        return Some(STILL_ASK.iter().map(|line| fmt(line, &[("state", &state)])).collect());
    }
    if !asks_for_a_fix(memory) || !has_evidence(text, memory) {
        // A line that is only the thing again with "still" on it says the same problem is going
        // on, so it is not nothing. A body is not a thing with a next step, so "still tired"
        // keeps the answer it had.
        let tail = still_tail(text, memory);
        if !tail.is_empty() && (POINTS_AT_IT.contains(&tail) || names_a_thing(memory)) {
            return Some(SAME_STILL.clone());
        }
        return None;
    }
    // "and now it is worse" has already said what the thing does, so asking "what is it doing
    // now" asks for the line back.
    if made_it_worse && !(NOT_WORKED.is_match(&report) || TRIED.is_match(&report)) {
        return None;
    }
    if !names_a_thing(memory) {
        // Nothing in this chat is a thing that can be doing something, so there is no trade to
        // hand over and no "what is it doing now" to ask: ask what has been tried.
        return Some(ASK_WHAT_TRIED.clone());
    }
    Some(ASK_NEXT.clone())
}

// --------------------------------------------------------------------------------------------
// A person of theirs in a place of care.

/// Python: bot/memory/tried.py:1829 `_the_person_word(token)`
///
/// The person the word names ("mum", "dad"), or "" for a word that names nobody.
pub fn _the_person_word(token: &str) -> String {
    let mut word = token.strip_suffix("'s").unwrap_or(token);
    if let Some(one) = word.strip_suffix('s') {
        if SHE.contains(one) || HE.contains(one) {
            word = one;
        }
    }
    if SHE.contains(word) || HE.contains(word) { word.to_string() } else { String::new() }
}

/// Python: bot/memory/tried.py:1837 `_a_report_on_a_person(text)`
///
/// True when the line is only "still" and pointing words: "still there", "still the same",
/// "still in hospital", "she's still in".
pub fn _a_report_on_a_person(text: &str) -> bool {
    let said = the_words(text);
    let tokens = pt::split_ref(&said);
    if tokens.is_empty() || tokens.len() > 6 || said.ends_with('?') {
        return false;
    }
    let Some(at) = tokens.iter().position(|token| *token == "still") else {
        return false;
    };
    let after = &tokens[at + 1..];
    !after.is_empty()
        && after.iter().all(|word| STILL_ABOUT_A_PERSON.contains(*word))
        && tokens[..at].iter().all(|word| POINTING.contains(*word))
}

/// The pronouns a person word settles: (subject, object).
fn pronouns_of(person: &str) -> (&'static str, &'static str) {
    if SHE.contains(person) {
        ("she", "her")
    } else if HE.contains(person) {
        ("he", "him")
    } else {
        ("they", "them")
    }
}

/// Python: bot/memory/tried.py:1853 `_in_care(memory)`
///
/// (the person as "your mum" wants it, subject, object) for the person in a place of care, or
/// None. The person's own lines only, newest first, six turns back; a line that names a thing a
/// home fix is about is where the scan stops.
pub fn _in_care(memory: &Memory) -> Option<(String, String, String)> {
    let turns = &memory.turns;
    let lines = &turns[turns.len().saturating_sub(6)..];
    let found = |word: &str, subj: &str, obj: &str| Some((word.to_string(), subj.to_string(), obj.to_string()));
    for line in lines.iter().rev().map(|t| &t.user).filter(|line| !line.is_empty()) {
        let said = the_words(line);
        let tokens: HashSet<String> = pt::split(&said).into_iter().collect();
        if TRADES.iter().any(|row| _names(&tokens, &said, &row.entry_words)) {
            return None;
        }
        if !IN_CARE.is_match(&said) {
            continue;
        }
        let with_someone = WITH_SOMEONE.search(&said);
        let person_with = with_someone.as_ref().map(|m| _the_person_word(m.str(1))).unwrap_or_default();
        if let Some(with_someone) = with_someone.as_ref().filter(|_| person_with.is_empty()) {
            // "with my partner" and "with my friend" name a person with no pronoun to fix. A
            // `with` clause that names a person at all is not the user being the patient.
            let word_with = pt::rstrip_chars(with_someone.str(1), "'s");
            if A_PERSON.contains(word_with)
                || A_PERSON.contains(pt::rstrip_chars(word_with, "s"))
                || PEOPLE_AGED.contains(word_with)
            {
                return found("", "they", "them");
            }
        }
        if !person_with.is_empty() {
            // "im at the hospital with mum" is about the mum, not about the person sitting in
            // the waiting room.
            let (subj, obj) = pronouns_of(&person_with);
            return found(&person_with, subj, obj);
        }
        if I_AM_IN.is_match(&said) || OPENS_ON_IM.match_(&said).is_some() {
            return found("", "you", "you");
        }
        for word in pt::split_ref(&said) {
            let person = _the_person_word(word);
            if !person.is_empty() {
                let (subj, obj) = pronouns_of(&person);
                return found(&person, subj, obj);
            }
        }
        // The place is named and no person is: the answer can ask about them without a word
        // for who they are.
        return found("", "they", "them");
    }
    None
}

/// Python: bot/memory/tried.py:1905 `in_care_pool(text, memory)`
///
/// The lines that answer a bare "still ..." about a person of theirs who is in a place of care,
/// or None for every other line.
pub fn in_care_pool(text: &str, memory: &Memory) -> Option<Vec<String>> {
    if is_empty(memory) || !_a_report_on_a_person(text) {
        return None;
    }
    let (word, subj, obj) = _in_care(memory)?;
    if word.is_empty() {
        if subj == "you" {
            return Some(IN_CARE_SELF.clone());
        }
        return Some(IN_CARE_ASK_UNNAMED.clone());
    }
    // A line that still prints a brace is a line with a word in it this branch cannot fill.
    let filled: Vec<String> = IN_CARE_ASK
        .iter()
        .map(|line| fmt(line, &[("subj", &subj), ("obj", &obj), ("word", &word)]))
        .filter(|line| !line.contains('{') && !line.contains('}'))
        .collect();
    if filled.is_empty() { Some(IN_CARE_ASK_UNNAMED.clone()) } else { Some(filled) }
}
