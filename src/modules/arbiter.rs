//! Port of `bot/modules/arbiter.py`: chooses each turn's reply among the modules' candidates.
//!
//! Every function of the Python file is here, in the file's order, with the same names. The
//! seed rule stays the floor: candidates arrive highest PRIORITY first, and a fallback (PRIORITY
//! below 0) answers only when nothing else does; the rules on top of it are read from the Python
//! docstrings (kept short here) and ported branch for branch.
//!
//! How the Python is carried over:
//!
//! * `memory` is the turn's real memory dict in Python, and the hooks the arbiter asks
//!   (`TAKES_AS_IT_COMES`, `CLAIMS_INSTEAD`, `ASKS_ABOUT_THE_CHAT`) may write to it; what they
//!   write is seen by every later reading in the same call. So `choose` works on one copy of the
//!   memory in a `RefCell` ([`Mem`]) that every reading borrows and every hook borrows mutably.
//!   The copy is dropped when `choose` returns, as Python's runtime replaces the memory with the
//!   chosen module's copy.
//! * A Python `try: ... except Exception` is [`guarded`]: a panic inside it (a Python exception,
//!   or a stub another workstream has not ported) takes the except branch. An exception Python
//!   does not catch is a panic that reaches `choose`, which falls back to the seed rule.
//! * Candidates are `&[Candidate]`; an index is a `usize`.
//! * Readings that belong to other lanes are called through small adapters at the end of the
//!   file ("Readings from the other lanes"), one per Python call, so the order of calls (and with
//!   it the global random generator and memory writes) is Python's.

use crate::data::{self, Value};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::runtime::{guarded, Candidate};
use crate::state::Memory;
use crate::understood::Understood;
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// The turn's memory as the arbiter's readings and the modules' hooks share it (see the module
/// documentation).
pub type Mem = RefCell<Memory>;

type Set = HashSet<String>;

// ---------------------------------------------------------------------------------------------
// Module-level data (bot.modules.arbiter.*), read from the exported bundle.

fn name(n: &str) -> String {
    format!("bot.modules.arbiter.{n}")
}
fn bre(n: &str) -> &'static Re {
    data::re(&name(n))
}
fn bset(n: &str) -> Set {
    data::set(&name(n))
}
fn bint(n: &str) -> i64 {
    data::get(&name(n)).as_int().unwrap_or_else(|| panic!("{n} is not an int"))
}
fn bf64(n: &str) -> f64 {
    data::get(&name(n)).as_f64().unwrap_or_else(|| panic!("{n} is not a number"))
}

static THREAD: LazyLock<String> = LazyLock::new(|| data::string(&name("THREAD")));
static CARE_THREAD: LazyLock<String> = LazyLock::new(|| data::string(&name("CARE_THREAD")));
static PAUSE: LazyLock<String> = LazyLock::new(|| data::string(&name("PAUSE")));
static STEP_WHEN_LOW: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("_STEP_WHEN_LOW")));
static CHOICE: LazyLock<String> = LazyLock::new(|| data::string(&name("CHOICE")));
static SAFETY: LazyLock<f64> = LazyLock::new(|| bf64("SAFETY"));
static CHATTER: LazyLock<f64> = LazyLock::new(|| bf64("CHATTER"));
static RECALL: LazyLock<f64> = LazyLock::new(|| bf64("RECALL"));
static MOVE_WORDS: LazyLock<usize> = LazyLock::new(|| bint("MOVE_WORDS") as usize);
static MOVE_SHARE: LazyLock<f64> = LazyLock::new(|| bf64("MOVE_SHARE"));
static MOVE_WHOLE: LazyLock<f64> = LazyLock::new(|| bf64("MOVE_WHOLE"));
static MOVE_ANSWER_WORDS: LazyLock<usize> = LazyLock::new(|| bint("MOVE_ANSWER_WORDS") as usize);
static WHOLE_LINE_BAND: LazyLock<f64> = LazyLock::new(|| bf64("WHOLE_LINE_BAND"));
static WORD: LazyLock<&'static Re> = LazyLock::new(|| bre("WORD"));
static LABEL_KEYS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("LABEL_KEYS")));
static SAID_BEFORE_WORDS: LazyLock<usize> = LazyLock::new(|| bint("_SAID_BEFORE_WORDS") as usize);
static MEMORY_MODULES: LazyLock<Set> = LazyLock::new(|| bset("MEMORY_MODULES"));
static REFUSAL_UNDONE: LazyLock<Set> = LazyLock::new(|| bset("_REFUSAL_UNDONE"));
static ALLERGY_LIFTED: LazyLock<&'static Re> = LazyLock::new(|| bre("_ALLERGY_LIFTED"));
static A_CEILING: LazyLock<&'static Re> = LazyLock::new(|| bre("A_CEILING"));
static GENERIC: LazyLock<Set> = LazyLock::new(|| bset("GENERIC"));
static A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| bre("A_FIGURE"));
static A_WORKED_OUT: LazyLock<&'static Re> = LazyLock::new(|| bre("A_WORKED_OUT"));
static A_SUM_TO_WORK_OUT: LazyLock<&'static Re> = LazyLock::new(|| bre("A_SUM_TO_WORK_OUT"));
static AN_AMENDMENT: LazyLock<&'static Re> = LazyLock::new(|| bre("_AN_AMENDMENT"));
static MONTHS: LazyLock<&'static Re> = LazyLock::new(|| bre("_MONTHS"));
static DOING_WANTS: LazyLock<Set> = LazyLock::new(|| bset("DOING_WANTS"));
static SAID_BACK: LazyLock<&'static Re> = LazyLock::new(|| bre("SAID_BACK"));
static NUMBER_WORDS: LazyLock<HashMap<String, f64>> = LazyLock::new(|| {
    data::get(&name("_NUMBER_WORDS"))
        .pairs()
        .iter()
        .map(|(k, v)| (k.str().to_string(), v.as_f64().unwrap_or(0.0)))
        .collect()
});
static FIGURES: LazyLock<&'static Re> = LazyLock::new(|| bre("_FIGURES"));
static A_MY_OWN_REPORT: LazyLock<&'static Re> = LazyLock::new(|| bre("A_MY_OWN_REPORT"));
static A_REPORT_ASKS: LazyLock<&'static Re> = LazyLock::new(|| bre("A_REPORT_ASKS"));
static A_DEFINITION: LazyLock<&'static Re> = LazyLock::new(|| bre("A_DEFINITION"));
static A_REPORT_NOD: LazyLock<&'static Re> = LazyLock::new(|| bre("A_REPORT_NOD"));
static A_CONTENT_WORD: LazyLock<&'static Re> = LazyLock::new(|| bre("A_CONTENT_WORD"));
static REPORT_STOP: LazyLock<Set> = LazyLock::new(|| bset("REPORT_STOP"));
static LONG_ENOUGH: LazyLock<usize> = LazyLock::new(|| bint("LONG_ENOUGH") as usize);
static LETTERS: LazyLock<&'static Re> = LazyLock::new(|| bre("LETTERS"));
static FIRST_VERB: LazyLock<Set> = LazyLock::new(|| bset("FIRST_VERB"));
static CARRIES_NO_SUBJECT: LazyLock<Set> = LazyLock::new(|| bset("CARRIES_NO_SUBJECT"));
static INFLECTED_ENDS: LazyLock<Set> = LazyLock::new(|| bset("INFLECTED_ENDS"));
static DOUBT_WORDS: LazyLock<Set> = LazyLock::new(|| bset("DOUBT_WORDS"));
static POINTS_BACK: LazyLock<&'static Re> = LazyLock::new(|| bre("POINTS_BACK"));
static REPEAT_FRAME: LazyLock<Set> = LazyLock::new(|| bset("REPEAT_FRAME"));
static THE_ANSWER_BANDS: LazyLock<Set> = LazyLock::new(|| bset("THE_ANSWER_BANDS"));
static POOL_MODULES: LazyLock<Set> = LazyLock::new(|| bset("POOL_MODULES"));
static THE_ASK_OPENS: LazyLock<&'static Re> = LazyLock::new(|| bre("THE_ASK_OPENS"));
static ECHO_STOP: LazyLock<Set> = LazyLock::new(|| bset("ECHO_STOP"));
static THEIR_VOICE: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map(&name("THEIR_VOICE")));
static A_SUBJECT_WORD: LazyLock<&'static Re> = LazyLock::new(|| bre("A_SUBJECT_WORD"));
static A_SUBJECT_PHRASE: LazyLock<&'static Re> = LazyLock::new(|| bre("A_SUBJECT_PHRASE"));
static WORD_ENDINGS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("WORD_ENDINGS")));
static A_MEASURE_ASKED: LazyLock<&'static Re> = LazyLock::new(|| bre("A_MEASURE_ASKED"));
static NOT_A_SUBJECT: LazyLock<Set> = LazyLock::new(|| bset("NOT_A_SUBJECT"));
static WHOLE_MOVE_WORDS: LazyLock<usize> = LazyLock::new(|| bint("WHOLE_MOVE_WORDS") as usize);
static CUT_MARKS: LazyLock<String> = LazyLock::new(|| data::string(&name("CUT_MARKS")));
static TOKEN: LazyLock<&'static Re> = LazyLock::new(|| bre("TOKEN"));
static THE_CAUSE: LazyLock<&'static Re> = LazyLock::new(|| bre("THE_CAUSE"));
static PEOPLE: LazyLock<Set> = LazyLock::new(|| bset("_PEOPLE"));
static CONDOLENCE: LazyLock<&'static Re> = LazyLock::new(|| bre("_CONDOLENCE"));
static THE_NAMELESS: LazyLock<Set> = LazyLock::new(|| bset("_THE_NAMELESS"));
static KEEPS_ITS_OWN_ANSWER: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("KEEPS_ITS_OWN_ANSWER")));
static ANSWERS_THE_NEWS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&name("ANSWERS_THE_NEWS")));
static A_VERDICT_BACK: LazyLock<&'static Re> = LazyLock::new(|| bre("A_VERDICT_BACK"));
static NOT_A_SUBJECT_WORD: LazyLock<Set> = LazyLock::new(|| bset("NOT_A_SUBJECT_WORD"));
static ASK_CLAUSE_SPLIT: LazyLock<&'static Re> = LazyLock::new(|| bre("ASK_CLAUSE_SPLIT"));
static ASKS_FOR_A_THING: LazyLock<&'static Re> = LazyLock::new(|| bre("ASKS_FOR_A_THING"));
static A_HOW_TO: LazyLock<&'static Re> = LazyLock::new(|| bre("_A_HOW_TO"));
static THE_ONE_NAMED: LazyLock<&'static Re> = LazyLock::new(|| bre("_THE_ONE_NAMED"));
static A_PERSON_NAMED: LazyLock<&'static Re> = LazyLock::new(|| bre("_A_PERSON_NAMED"));
static ABOUT_SOMEONE_ELSE: LazyLock<&'static Re> = LazyLock::new(|| bre("_ABOUT_SOMEONE_ELSE"));
static REFUSAL: LazyLock<&'static Re> = LazyLock::new(|| bre("REFUSAL"));
static CHITCHAT: LazyLock<Set> = LazyLock::new(|| bset("CHITCHAT"));
static AGAIN_FIRST_WORDS: LazyLock<usize> = LazyLock::new(|| bint("AGAIN_FIRST_WORDS") as usize);
static AGAIN_TAIL_WORDS: LazyLock<usize> = LazyLock::new(|| bint("AGAIN_TAIL_WORDS") as usize);
static A_POINTS_BACK: LazyLock<&'static Re> = LazyLock::new(|| bre("A_POINTS_BACK"));
static NOT_A_NEW_WORD: LazyLock<Set> = LazyLock::new(|| bset("NOT_A_NEW_WORD"));
static A_PLAINER_ASK: LazyLock<&'static Re> = LazyLock::new(|| bre("A_PLAINER_ASK"));
static PLAIN_ASK_FILLER: LazyLock<Set> = LazyLock::new(|| bset("PLAIN_ASK_FILLER"));

// The regexes Python writes inline in a function body.
/// `re.split(r"(?<=[.!?])\s+", ..., maxsplit=1)` in `_offers_the_food`.
static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", pyre::U));
/// `re.findall(r"[a-z']+", ...)`.
static LETTERS_AND_QUOTE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", pyre::U));
/// `re.search(r"\bhow much\b", line, re.I)`.
static HOW_MUCH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow much\b", pyre::I | pyre::U));
/// `re.sub(r"[^a-z0-9' ]+", " ", ...)`.
static NOT_A_WORD_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", pyre::U));
/// `re.search(r"\d", said)` in `_places_the_figure`.
static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", pyre::U));
/// The money or rate words in `_places_the_figure`.
static A_PRICE: LazyLock<Re> =
    LazyLock::new(|| Re::new("[£$\u{20ac}]|\\b(?:a|an|per|pounds?|quid|dollars?|euros?)\\b", pyre::U));

/// Python's `READ`: readings already made of the conversation's stored lines (`_read_topic`).
static READ: LazyLock<std::sync::Mutex<HashMap<String, Set>>> = LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

// ---------------------------------------------------------------------------------------------
// Small Python idioms.

/// `_priority(candidate)`: the candidate's PRIORITY (always a number here).
fn _priority(candidate: &Candidate) -> f64 {
    candidate.priority
}

/// `int(candidate.get("priority") or 0)`: the PRIORITY truncated toward zero.
fn int_priority(candidate: &Candidate) -> f64 {
    candidate.priority.trunc()
}

/// The memory, borrowed for one reading.
fn m(mem: &Mem) -> Ref<'_, Memory> {
    mem.borrow()
}

/// `WORD.findall(s)`.
fn word_findall(s: &str) -> Vec<String> {
    WORD.findall_str(s)
}

/// `s.split()` as owned words.
fn split(s: &str) -> Vec<String> {
    text::split(s)
}

/// `len(s)` in characters.
fn clen(s: &str) -> usize {
    text::len(s)
}

/// `s[:-n]` with Python's character indices.
fn drop_last(s: &str, n: usize) -> String {
    let count = clen(s);
    s.chars().take(count.saturating_sub(n)).collect()
}

/// `str(value)` for the plain values memory holds (a string is itself).
fn py_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::None => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        other => format!("{:?}", other.to_plain()),
    }
}

/// `str(x or "")` of a value.
fn py_str_or_empty(v: Option<&Value>) -> String {
    match v {
        Some(v) if v.truthy() => py_str(v),
        _ => String::new(),
    }
}

/// The candidate's module name.
fn module_of(candidates: &[Candidate], index: usize) -> &str {
    candidates[index].module
}

// ---------------------------------------------------------------------------------------------
// The hook.

/// What `_choose` hands back: an index, or the empty frozenset `_the_venue_hours_turn` returns
/// when the composer's pool cannot be read (Python passes it through both guards unchanged, and
/// the runtime reads anything that is not an index as 0).
enum Pick {
    At(usize),
    Nonsense,
}

/// Python: bot/modules/arbiter.py:204 `choose(text, memory, candidates)`: the index of the chosen
/// candidate.
pub fn choose(text: &str, memory: &Memory, candidates: &[Candidate]) -> usize {
    let mem: Mem = RefCell::new(memory.clone());
    let index = match guarded(|| _choose(text, &mem, candidates)) {
        Some(Pick::At(index)) => index,
        Some(Pick::Nonsense) => return 0,
        None => _seed(candidates),
    };
    let index = _a_past_that_is_not_there(index, candidates, text, &mem);
    _an_answer_it_already_gave(index, candidates, text, &mem)
}

/// Python: bot/modules/arbiter.py:218 `_a_past_that_is_not_there(index, candidates, text, memory)`
///
/// The chosen candidate, unless it answers a chat that has not started: a reply about turns
/// behind this one gives way to the best real candidate that is not, never out of the safety
/// band. Everything is inside one `try`, so the guard cannot lose the turn.
pub fn _a_past_that_is_not_there(index: usize, candidates: &[Candidate], text: &str, mem: &Mem) -> usize {
    if candidates.is_empty() {
        return index;
    }
    guarded(|| {
        let memory = memory_value(mem);
        let speaks_of_a_past = |reply: &str| crate::memory::history::a_past_that_is_not_there(reply, &memory, Some(text));
        let reply = if index < candidates.len() { candidates[index].reply.as_str() } else { "" };
        if !speaks_of_a_past(reply) {
            return index;
        }
        if _priority(&candidates[index]) >= *SAFETY {
            return index;
        }
        for (other, candidate) in candidates.iter().enumerate() {
            if _priority(candidate) < 0.0 {
                continue;
            }
            if !speaks_of_a_past(&candidate.reply) {
                return other;
            }
        }
        index
    })
    .unwrap_or(index)
}

/// Python: bot/modules/arbiter.py:273 `_an_answer_it_already_gave(index, candidates, text, memory)`
///
/// The chosen candidate, unless it is a whole answer this chat has already been given (to a
/// different question): then the first other candidate outside the safety band that says
/// something of its own takes the turn. Where none does, the repeat stands.
pub fn _an_answer_it_already_gave(index: usize, candidates: &[Candidate], text: &str, mem: &Mem) -> usize {
    if candidates.is_empty() {
        return index;
    }
    guarded(|| {
        if _priority(&candidates[index]) >= *SAFETY {
            return index;
        }
        let heard = _said_before(mem, text);
        if heard.is_empty() || !heard.contains(&_content(&candidates[index].reply)) {
            return index;
        }
        let last = _last_reply(mem);
        for (other, candidate) in candidates.iter().enumerate() {
            if other == index {
                continue;
            }
            if _priority(candidate) >= *SAFETY {
                continue;
            }
            let reply = candidate.reply.as_str();
            if heard.contains(&_content(reply)) || _echoes(reply, &last, false) {
                continue;
            }
            if lanes::is_a_hedge(reply) || _points_back(reply) {
                continue;
            }
            if split(&_content(reply)).len() < *SAID_BEFORE_WORDS {
                // A stock line, a receipt or a nod is not a new answer: the paragraph again is
                // better than that (STYLE.md).
                continue;
            }
            return other;
        }
        index
    })
    .unwrap_or(index)
}

/// Python: bot/modules/arbiter.py:350 `_a_recall_from_an_earlier_turn(text, memory)`
///
/// Whether this line asks for something said earlier, and there is an earlier turn to hold it:
/// any of the memory lane's three readings of such an ask.
pub fn _a_recall_from_an_earlier_turn(text: &str, mem: &Mem) -> bool {
    if !lanes::there_is_a_chat(mem) {
        return false;
    }
    if guarded(|| lanes::recall_ask(text)).unwrap_or(false) {
        return true;
    }
    if guarded(|| lanes::repeat_asked_back(text, mem)).unwrap_or(false) {
        return true;
    }
    guarded(|| lanes::lineback_an_earlier_turn(text, mem)).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:378 `_the_kind_line_they_gave(text, memory)`
///
/// The earlier line of theirs that gives the kind of detail this line asks for, or None (the
/// memory lane's reading).
pub fn _the_kind_line_they_gave(text: &str, mem: &Mem) -> Value {
    if !lanes::there_is_a_chat(mem) {
        return Value::None;
    }
    guarded(|| lanes::lineback_the_kind_they_gave(text, mem)).unwrap_or(Value::None)
}

/// Python: bot/modules/arbiter.py:392 `_seed(candidates)`: the first real candidate, or the
/// first fallback.
pub fn _seed(candidates: &[Candidate]) -> usize {
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) >= 0.0 {
            return index;
        }
    }
    0
}

/// Python: bot/modules/arbiter.py:426 `_the_food_the_line_refused(text, memory)`
///
/// The foods the line says somebody at the table will not eat, read by the kitchen; an empty set
/// for a cupboard that ran out or an allergy taken back.
pub fn _the_food_the_line_refused(text: &str, mem: &Mem) -> Set {
    // `understood = memory.get("understood")`, or `{"clean": text}` when there is none.
    let understood = m(mem).understood.clone().unwrap_or_else(|| Understood { clean: text.to_string(), ..Understood::default() });
    let said = text::lower(if !understood.clean.is_empty() {
        &understood.clean
    } else if !understood.text.is_empty() {
        &understood.text
    } else {
        text
    });
    if ALLERGY_LIFTED.is_match(&said) {
        return Set::new();
    }
    if lanes::cooking_shortage_words().iter().any(|phrase| said.contains(phrase.as_str())) {
        return Set::new();
    }
    guarded(|| crate::knowledge::cooking_b::refused_foods(&understood)).unwrap_or_default()
}

/// Python: bot/modules/arbiter.py:448 `_offers_the_food(reply, refused)`
///
/// True when the first sentence of the reply offers a food the line refused: the first mention
/// of each food, with no refusal word within five words of it.
pub fn _offers_the_food(reply: &str, refused: &Set) -> bool {
    let stripped = text::strip(reply);
    let first = match SENTENCE_END.search(stripped) {
        Some(found) => &stripped[..found.start()],
        None => stripped,
    };
    let first = text::lower(first);
    let words: Vec<String> = LETTERS_AND_QUOTE.findall_str(&first).iter().map(|word| word.replace('\'', "")).collect();
    for food in refused {
        let mut names: Set = [food.clone(), format!("{food}s")].into_iter().collect();
        if food.ends_with('y') {
            names.insert(format!("{}ies", drop_last(food, 1)));
        }
        if food.ends_with('f') {
            names.insert(format!("{}ves", drop_last(food, 1)));
        }
        for (place, word) in words.iter().enumerate() {
            if !names.contains(word) {
                continue;
            }
            let near = &words[place.saturating_sub(5)..(place + 5).min(words.len())];
            if !near.iter().any(|undone| REFUSAL_UNDONE.contains(undone)) {
                return true;
            }
            break; // the first mention is the one the answer opens on
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:474 `_offers_what_the_line_ruled_out(candidate, text, memory)`
///
/// True when a candidate hands back the very thing the person has just ruled out.
pub fn _offers_what_the_line_ruled_out(candidate: &Candidate, text: &str, mem: &Mem) -> bool {
    // `thing, _kept, _opening = ...` inside the try; `if not thing: return False`.
    let Some(Some(thing)) = guarded(|| lanes::ruled_out_thing(text, mem)) else {
        return false;
    };
    _offers_the_food(&candidate.reply, &[thing].into_iter().collect())
}

/// Python: bot/modules/arbiter.py:496 `_the_thing_the_line_ruled_out(text, memory)`
///
/// The thing the line rules out of the answer above it, in the person's own words, or "".
pub fn _the_thing_the_line_ruled_out(text: &str, mem: &Mem) -> String {
    match guarded(|| lanes::ruled_out_thing(text, mem)) {
        Some(Some(thing)) => thing,
        _ => String::new(),
    }
}

/// Python: bot/modules/arbiter.py:507 `_every_real_answer_offers(thing, candidates, index)`
///
/// True when every answer above the floor (and under the safety band) hands back the thing the
/// line just ruled out.
pub fn _every_real_answer_offers(thing: &str, candidates: &[Candidate], index: usize) -> bool {
    let others: Vec<&Candidate> = candidates
        .iter()
        .enumerate()
        .filter(|(other, candidate)| {
            *other != index && _priority(candidate) >= 0.0 && _priority(candidate) < *SAFETY && !text::strip(&candidate.reply).is_empty()
        })
        .map(|(_, candidate)| candidate)
        .collect();
    if others.is_empty() {
        return false;
    }
    let refused: Set = [thing.to_string()].into_iter().collect();
    others.iter().all(|candidate| _offers_the_food(&candidate.reply, &refused))
}

/// Python: bot/modules/arbiter.py:533 `_the_cut_the_line_asked_for(memory, candidates)`
///
/// The candidate that is the composer's cut of the answer above (the thing the line ruled out
/// taken out of it), when a module offered it.
pub fn _the_cut_the_line_asked_for(mem: &Mem, candidates: &[Candidate]) -> Option<usize> {
    let constrained = m(mem).understood.as_ref().map(|u| u.constraint).unwrap_or(false);
    if !constrained {
        return None;
    }
    let wanted = guarded(|| lanes::limit_reply_for(mem))?;
    if !wanted.truthy() {
        return None;
    }
    let wanted = py_str(&wanted);
    let wanted = text::strip(&wanted);
    candidates.iter().position(|candidate| text::strip(&candidate.reply) == wanted)
}

/// Python: bot/modules/arbiter.py:575 `_the_figures_they_gave(memory)`: the numbers in the
/// person's own last eight lines.
pub fn _the_figures_they_gave(mem: &Mem) -> Set {
    let memory = m(mem);
    let turns = &memory.turns;
    let mut said = Set::new();
    for one in &turns[turns.len().saturating_sub(8)..] {
        said.extend(A_FIGURE.findall_str(&one.user));
    }
    said
}

/// Python: bot/modules/arbiter.py:588 `_says_back_one_of(reply, figures)`
pub fn _says_back_one_of(reply: &str, figures: &Set) -> bool {
    if figures.is_empty() {
        return false;
    }
    A_FIGURE.findall_str(reply).iter().any(|one| figures.contains(one))
}

/// Python: bot/modules/arbiter.py:595 `_the_candidates_with_a_figure(text, memory, candidates, pool)`
///
/// On a how-much line that asks for an amount, the candidates whose reply carries the figure (or
/// says the person's own figure back), best first; the pool unchanged otherwise.
pub fn _the_candidates_with_a_figure(text: &str, mem: &Mem, candidates: &[Candidate], pool: &[usize]) -> Vec<usize> {
    let wants_amount = m(mem).understood.as_ref().map(|u| u.wants.as_deref() == Some("amount")).unwrap_or(false);
    if !wants_amount {
        return pool.to_vec();
    }
    let line = text;
    if !HOW_MUCH.is_match(line) {
        return pool.to_vec();
    }
    if A_CEILING.is_match(line) {
        // A ceiling line is answered by a card, and a candidate that only hands the last answer
        // back under a repeat frame says nothing new.
        let frame = lanes::limit_frame();
        let plain: Vec<usize> = pool.iter().copied().filter(|&index| frame.match_(text::strip(&candidates[index].reply)).is_none()).collect();
        return if plain.is_empty() { pool.to_vec() } else { plain };
    }
    let money = lanes::narrow_signal("money");
    let figure = [money, lanes::narrow_signal("amount")];
    let theirs = _the_figures_they_gave(mem);
    let mut subject: Set = word_findall(&text::lower(line)).into_iter().collect();
    subject.extend(_carried_words(mem));
    let subject: Set = subject.into_iter().filter(|word| clen(word) >= 5 && !GENERIC.contains(word)).collect();
    let to_work_out = A_SUM_TO_WORK_OUT.is_match(line);
    let (mut with_figure, mut says_it_back, mut bare_echo) = (Vec::new(), Vec::new(), Vec::new());
    for &index in pool {
        let reply = candidates[index].reply.as_str();
        let has_figure = figure.iter().any(|signal| signal.map(|s| s.is_match(reply)).unwrap_or(false));
        let quotes = _says_back_one_of(reply, &theirs);
        if !(has_figure || quotes) {
            continue;
        }
        if quotes {
            let new_figures: Set = A_FIGURE.findall_str(reply).into_iter().filter(|one| !theirs.contains(one)).collect();
            // `money.search(reply)`: SIGNALS always has "money" (a None would raise here).
            let money_said = || money.expect("SIGNALS has money").is_match(reply);
            if !new_figures.is_empty() && (money_said() || A_WORKED_OUT.is_match(reply)) {
                says_it_back.push(index);
            } else if new_figures.is_empty() && !to_work_out {
                bare_echo.push(index);
            }
            continue;
        }
        if !subject.is_empty() {
            let said: Vec<String> = word_findall(&text::lower(reply)).into_iter().filter(|word| clen(word) >= 5).collect();
            if !subject.iter().any(|word| said.iter().any(|one| _the_same_word(word, one))) {
                continue;
            }
        }
        with_figure.push(index);
    }
    for best in [bare_echo, says_it_back, with_figure] {
        if !best.is_empty() && best.len() < pool.len() {
            return best;
        }
    }
    pool.to_vec()
}

/// Python: bot/modules/arbiter.py:689 `_amends_a_detail(text, memory)`
///
/// The months a line amends ("actually we fly in september"), or an empty set when the line is
/// not an amendment of one month in a chat already going.
pub fn _amends_a_detail(text: &str, mem: &Mem) -> Set {
    let said = text::strip(text);
    if said.is_empty() || !AN_AMENDMENT.is_match(said) {
        return Set::new();
    }
    let months: Set = MONTHS.findall_str(said).iter().map(|month| text::lower(month)).collect();
    if months.len() == 1 && !m(mem).turns.is_empty() {
        months
    } else {
        Set::new()
    }
}

/// Python: bot/modules/arbiter.py:704 `_the_candidates_that_hand_the_detail_back(months, candidates, pool)`
///
/// The one candidate that says the amended month first, then the rest of the pool; the pool
/// unchanged when none or more than one says it.
pub fn _the_candidates_that_hand_the_detail_back(months: &Set, candidates: &[Candidate], pool: &[usize]) -> Vec<usize> {
    let holding: Vec<usize> = pool
        .iter()
        .copied()
        .filter(|&index| MONTHS.findall_str(&candidates[index].reply).iter().any(|month| months.contains(&text::lower(month))))
        .collect();
    if holding.len() != 1 {
        return pool.to_vec();
    }
    let mut out = holding.clone();
    out.extend(pool.iter().copied().filter(|&index| index != holding[0]));
    out
}

/// Python: bot/modules/arbiter.py:720 `_asks_it_back(reply)`: the reply ends by asking the
/// person something.
pub fn _asks_it_back(reply: &str) -> bool {
    let said = text::strip(reply);
    if said.is_empty() {
        return false;
    }
    let last = said.replace('\n', " ");
    let last = text::rstrip(&last);
    let cut = [last.rfind(". "), last.rfind("! "), last.rfind("? ")].into_iter().flatten().max();
    match cut {
        Some(cut) => last.get(cut + 2..).unwrap_or("").contains('?'),
        None => last.contains('?'),
    }
}

/// Python: bot/modules/arbiter.py:736 `_the_candidates_that_give_the_shape(text, memory, candidates, pool)`
///
/// The candidates whose reply gives the shape of answer the line asked for (`compose/fit.py`),
/// in front of the rest, when the one in front definitely misses it; the pool otherwise.
pub fn _the_candidates_that_give_the_shape(text: &str, mem: &Mem, candidates: &[Candidate], pool: &[usize]) -> Vec<usize> {
    let want = match m(mem).understood.as_ref() {
        None => return pool.to_vec(),
        Some(u) => u.wants.clone().unwrap_or_default(),
    };
    if want.is_empty() {
        return pool.to_vec();
    }
    if want == "amount" {
        return _the_candidates_with_a_figure(text, mem, candidates, pool);
    }
    if lanes::fit_soft().contains(&want) {
        return pool.to_vec();
    }
    if want == "yesno" && _names_the_answers_figure(&candidates[pool[0]].reply, mem) {
        return pool.to_vec();
    }
    if _a_check_on_the_answer(&candidates[pool[0]].reply) {
        return pool.to_vec(); // a check on the answer above is the shape the line asked for
    }
    if pool.is_empty() || lanes::fit_gives(&candidates[pool[0]].reply, &want, text) != Some(false) {
        return pool.to_vec();
    }
    // A line that hands the question back is not the answer to it, so it does not take the turn
    // off an entry that does answer the thing, whatever shape its line gives.
    let words: Set = word_findall(&text::lower(text)).into_iter().collect();
    let subject_known = if pool.is_empty() { true } else { _names_the_subject(&candidates[pool[0]], mem, &words) };
    let mut giving = Vec::new();
    for &index in &pool[1..] {
        let reply = candidates[index].reply.as_str();
        if lanes::fit_gives(reply, &want, text) == Some(true)
            && (!subject_known || _names_the_subject(&candidates[index], mem, &words) || !_asks_it_back(reply))
        {
            giving.push(index);
        }
    }
    if giving.is_empty() {
        return pool.to_vec();
    }
    let mut out = giving.clone();
    out.extend(pool.iter().copied().filter(|index| !giving.contains(index)));
    out
}

/// Python: bot/modules/arbiter.py:794 `_the_line_asks_for_a_doing(memory)`: the line wants the
/// steps, the options, the pick, the advice or a how-to.
pub fn _the_line_asks_for_a_doing(mem: &Mem) -> bool {
    m(mem).understood.as_ref().and_then(|u| u.wants.as_ref()).map(|want| DOING_WANTS.contains(want)).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:808 `_says_what_they_already_said(reply)`: a reply that opens
/// on what the person said ("You said Friday.").
pub fn _says_what_they_already_said(reply: &str) -> bool {
    SAID_BACK.match_(text::strip(reply)).is_some()
}

/// Python: bot/modules/arbiter.py:830 `_the_numbers_in(line)`: the numbers a line names, digits
/// and words counting the same.
pub fn _the_numbers_in(line: &str) -> Vec<f64> {
    let mut said: Vec<f64> = Vec::new();
    let mut add = |value: f64| {
        if !said.contains(&value) {
            said.push(value);
        }
    };
    for word in FIGURES.findall_str(&text::lower(line)) {
        if let Some(value) = NUMBER_WORDS.get(&word) {
            add(*value);
            continue;
        }
        if let Ok(value) = word.replace(',', "").parse::<f64>() {
            add(value);
        }
    }
    said
}

/// Python: bot/modules/arbiter.py:844 `_names_the_answers_figure(reply, memory)`: the reply takes
/// up a number the answer above named.
pub fn _names_the_answers_figure(reply: &str, mem: &Mem) -> bool {
    let mine = _the_numbers_in(reply);
    if mine.is_empty() {
        return false;
    }
    let theirs = _the_numbers_in(&_last_reply(mem));
    mine.iter().any(|one| theirs.contains(one))
}

/// Python: bot/modules/arbiter.py:857 `_explains_the_method(reply)`: the reply says why a thing
/// works.
pub fn _explains_the_method(reply: &str) -> bool {
    let flat = text::lower(reply);
    flat.contains(" because ") && flat.contains("work")
}

/// Python: bot/modules/arbiter.py:868 `_the_figure_they_asked_back(candidates, real, memory, text)`
///
/// On a line that points back at a figure of the person's own (`looks_back`), the first real
/// candidate that carries the figure.
pub fn _the_figure_they_asked_back(candidates: &[Candidate], real: &[usize], mem: &Mem, text: &str) -> Option<usize> {
    let looks_back = m(mem).understood.as_ref().map(|u| u.looks_back.is_some()).unwrap_or(false);
    if !looks_back {
        return None;
    }
    let figure = guarded(|| lanes::repeat_the_figure_they_gave(text, mem))?;
    if !figure.truthy() {
        return None;
    }
    real.iter().copied().find(|&index| crate::memory::repeat::carries_the_figure(&figure, &candidates[index].reply))
}

/// Python: bot/modules/arbiter.py:943 `_the_words_the_report_names(text)`: the content words of
/// a report line.
pub fn _the_words_the_report_names(text: &str) -> Vec<String> {
    let said = text::squash(&text::lower(text));
    A_CONTENT_WORD.findall_str(&said).into_iter().filter(|word| !REPORT_STOP.contains(word)).collect()
}

/// Python: bot/modules/arbiter.py:949 `_a_report_and_no_ask(text)`: the line only reports that
/// the person has something.
pub fn _a_report_and_no_ask(text: &str) -> bool {
    let said = text::squash(&text::lower(text));
    A_MY_OWN_REPORT.is_match(&said) && !A_REPORT_ASKS.is_match(&said)
}

/// `" " + re.sub(r"[^a-z0-9' ]+", " ", said.lower()) + " "`.
fn padded_letters(said: &str) -> String {
    format!(" {} ", NOT_A_WORD_CHAR.sub(" ", &text::lower(said)))
}

/// The distinct words, in first-seen order (`set(words)` where only membership counts).
fn distinct(words: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in words {
        if !out.contains(word) {
            out.push(word.clone());
        }
    }
    out
}

/// Python: bot/modules/arbiter.py:955 `_is_advice_about(reply, words)`: the reply answers the
/// report (about its thing, and not a definition or a nod).
pub fn _is_advice_about(reply: &str, words: &[String]) -> bool {
    let said = text::squash(reply);
    if said.is_empty() || A_REPORT_NOD.match_(&said).is_some() || A_DEFINITION.match_(&said).is_some() {
        return false;
    }
    if split(&said).len() < 10 {
        return false;
    }
    let low = padded_letters(&said);
    distinct(words).iter().any(|word| low.contains(&format!(" {word}")))
}

/// Python: bot/modules/arbiter.py:968 `_how_much_it_names(reply, words)`: how many of the
/// report's words the reply names.
pub fn _how_much_it_names(reply: &str, words: &[String]) -> usize {
    let low = padded_letters(reply);
    distinct(words).iter().filter(|word| low.contains(&format!(" {word}"))).count()
}

/// Python: bot/modules/arbiter.py:974 `_the_advice_over_a_definition(candidates, pool, text, chosen)`
///
/// The candidate that advises about the thing the line reports, when the turn went to the
/// definition of that thing.
pub fn _the_advice_over_a_definition(candidates: &[Candidate], pool: &[usize], text: &str, chosen: usize) -> Option<usize> {
    if pool.len() < 2 || !_a_report_and_no_ask(text) {
        return None;
    }
    let words = _the_words_the_report_names(text);
    if words.is_empty() {
        return None;
    }
    let won = candidates[chosen].reply.as_str();
    let low = padded_letters(won);
    if A_DEFINITION.match_(won).is_none() || !words.iter().any(|word| low.contains(&format!(" {word}"))) {
        return None;
    }
    let (mut best_index, mut best_score, mut best_chat): (Option<usize>, usize, bool) = (None, 0, true);
    for &index in pool {
        if index == chosen {
            continue;
        }
        let reply = candidates[index].reply.as_str();
        if !_is_advice_about(reply, &words) {
            continue;
        }
        let score = _how_much_it_names(reply, &words);
        let is_chat = module_of(candidates, index) == "chat";
        if score > best_score || (score == best_score && (best_index.is_none() || (best_chat && !is_chat))) {
            best_index = Some(index);
            best_score = score;
            best_chat = is_chat;
        }
    }
    best_index
}

/// Python: bot/modules/arbiter.py:1014 `_choose(text, memory, candidates)`
///
/// The routing rules, in Python's order. Each rule either returns the turn or narrows the race
/// (`real`, then `pool`) for the rules after it; the specialist rule and the seed rule close it.
fn _choose(text: &str, mem: &Mem, candidates: &[Candidate]) -> Pick {
    use Pick::At;
    let mut real: Vec<usize> = (0..candidates.len()).filter(|&index| _priority(&candidates[index]) >= 0.0).collect();
    if real.is_empty() {
        return At(0);
    }
    let last = _last_reply(mem);
    if !last.is_empty() && _the_line_is_news(text) {
        // The line reports the thing the last answer was about coming back, so an answer under a
        // lead-in that says it is the same one misreads it: any candidate that does not announce
        // itself as the old answer is better, a stock line included.
        let hears_the_news = |index: usize| !_announces_a_repeat(&candidates[index].reply);
        let mut hearing_it: Vec<usize> = real.iter().copied().filter(|&index| hears_the_news(index)).collect();
        if hearing_it.is_empty() {
            hearing_it = (0..candidates.len()).filter(|&index| hears_the_news(index)).collect();
        }
        if !hearing_it.is_empty() && hearing_it.len() < candidates.len() {
            real = hearing_it;
        }
    }
    let mut best = real[0];
    if _priority(&candidates[best]) >= *SAFETY {
        return At(best); // the safety band is answered as it comes, before the never-repeat rule
    }
    let amended = _amends_a_detail(text, mem);
    if !amended.is_empty() {
        // A detail of their own plan is being changed: the reply that carries the new detail
        // answers it, and it goes into the race as well as to the front.
        let every: Vec<usize> = (0..candidates.len()).collect();
        let handed = _the_candidates_that_hand_the_detail_back(&amended, candidates, &every);
        if handed[0] != best {
            let front = handed[0];
            real = std::iter::once(front).chain(real.iter().copied().filter(|&index| index != front)).collect();
            best = front;
        }
    }
    // A reply that talks about the conversation needs one to have happened.
    if !lanes::onscreen_earlier_turn(mem) {
        let fresh: Vec<usize> =
            real.iter().copied().filter(|&index| !lanes::talks_about_the_conversation(&candidates[index].reply)).collect();
        if !fresh.is_empty() {
            best = fresh[0];
            real = fresh;
        }
    }
    // A check on the answer above keeps the turn the composer gave it, unless a reply explains
    // why the method works.
    if _a_check_on_the_answer(&candidates[best].reply) {
        let explained: Vec<usize> =
            real.iter().copied().filter(|&index| index != best && _explains_the_method(&candidates[index].reply)).collect();
        if explained.is_empty() {
            return At(best);
        }
        best = explained[0];
    }
    // A line pointing back at a figure the person gave gets the reply that carries it.
    if let Some(back) = _the_figure_they_asked_back(candidates, &real, mem, text) {
        return At(back);
    }
    // A condolence is not the answer to a question about why a living thing is failing.
    if let Some(cause) = _the_cause_asked_for(candidates, &real, text, best) {
        return At(cause);
    }
    // A verdict on the news has no subject of its own: the composer's line takes it.
    if let Some(verdict) = _verdict_reply(candidates, text, mem, &last, text, best) {
        return At(verdict);
    }
    // An ask to have the answer put a plainer way is answered by the candidate that explains it.
    if let Some(plainer) = _a_plainer_reading_of_the_same_answer(candidates, best, text, mem) {
        return At(plainer);
    }
    if let Some(handed_back) = _the_answer_handed_back(candidates, text, mem, &last) {
        return At(handed_back);
    }
    // A line that asks for the kind of detail an earlier line of theirs gives is answered by that
    // line (said_back), over the small talk and a card that merely names a word of it.
    if _the_kind_line_they_gave(text, mem).truthy() {
        let front = module_of(candidates, real[0]);
        for &index in &real {
            if module_of(candidates, index) == "said_back" {
                if ["chat", "greeting", "answers"].contains(&front) || index == real[0] {
                    return At(index);
                }
            }
        }
    }
    if let Some(mut held) = _as_it_comes(text, mem, candidates, &last) {
        // A how-much line is owed a figure even when the claim is a recall of the card.
        let race: Vec<usize> = std::iter::once(held).chain(real.iter().copied().filter(|&index| index != held)).collect();
        let with_figure = _the_candidates_that_give_the_shape(text, mem, candidates, &race);
        if !with_figure.contains(&held) {
            held = with_figure[0];
        } else if with_figure[0] != held && _the_line_asks_for_a_doing(mem) && _says_what_they_already_said(&candidates[held].reply) {
            // The claim hands back something the person has already said, and the filter put a
            // reply that gives the kind of answer asked for in front of it.
            held = with_figure[0];
        }
        // A claim is only given up to a module above it that says the line is its own answer.
        let mut instead = _claims_instead(text, mem, candidates, &last, Some(_priority(&candidates[held])));
        if let Some(other) = instead {
            if _offers_what_the_line_ruled_out(&candidates[other], text, mem) {
                instead = None;
            }
        }
        return At(instead.unwrap_or(held));
    }
    let mut pool = _the_candidates_that_give_the_shape(text, mem, candidates, &real);
    real = pool.clone();
    // A reply made of nothing but the words the person has just used stands down for one that
    // answers the ask after them.
    if let Some(answering) = _the_candidates_that_answer_the_ask(text, candidates, &pool) {
        pool = answering.clone();
        real = answering;
    }
    // The candidate in front offers the very food the line refused, and another does not.
    let refused = _the_food_the_line_refused(text, mem);
    if !refused.is_empty() {
        let respecting: Vec<usize> = pool.iter().copied().filter(|&index| !_offers_the_food(&candidates[index].reply, &refused)).collect();
        if !respecting.is_empty() && respecting.len() < pool.len() {
            pool = respecting.clone();
            real = respecting;
        }
    }
    if let Some(cut) = _the_cut_the_line_asked_for(mem, candidates) {
        // The answer above with the ruled-out thing taken out of it: a repeat the person asked
        // for, so the never-repeat rule below does not apply to it.
        return At(cut);
    }
    if !last.is_empty() {
        // The never-repeat rule: a reply that says the last answer again (with a nod in front, or
        // a slice of it), a hedge, or a pointer back at the answer above is passed over while a
        // real candidate has something new; so is a whole answer given two or more turns back.
        let heard = _said_before(mem, text);
        // A cut of an answer already read is not a fresh answer: the words of the candidate sit
        // inside an earlier answer as one run and are fewer than it.
        let mut cut: Vec<(usize, Vec<String>)> = Vec::new();
        for &index in &pool {
            let words = split(&_content(&candidates[index].reply));
            if words.len() < 8 {
                continue;
            }
            for line in &heard {
                let was = split(line);
                if words.len() < was.len() && _a_run_of(&was, &words) {
                    cut.push((index, was));
                    break;
                }
            }
        }
        let mut fresh: Vec<usize> = pool
            .iter()
            .copied()
            .filter(|&index| {
                let reply = candidates[index].reply.as_str();
                _content(reply) != last && !_echoes(reply, &last, false) && !lanes::is_a_hedge(reply) && !_points_back(reply)
            })
            .collect();
        if !cut.is_empty() {
            let held: Vec<usize> = fresh.iter().copied().filter(|index| !cut.iter().any(|(gone, _)| gone == index)).collect();
            // The turn may only go to a line made out of the same answer's words.
            if !held.is_empty()
                && held.iter().any(|&take| {
                    let taken = split(&_content(&candidates[take].reply));
                    cut.iter().any(|(_, was)| _a_run_of(was, &taken))
                })
            {
                fresh = held;
            }
        }
        if !fresh.is_empty() && !heard.contains(&_content(&candidates[pool[0]].reply)) {
            let unheard: Vec<usize> = fresh.iter().copied().filter(|&index| !heard.contains(&_content(&candidates[index].reply))).collect();
            if !unheard.is_empty() {
                fresh = unheard;
            }
        }
        if !fresh.is_empty() && _a_repeat_to_hold(mem, text) && _echoes(&candidates[pool[0]].reply, &last, false) {
            // A line that asks to hear the answer again is answered by that answer, so an echo of
            // it is not the same thing twice.
            fresh = Vec::new();
        }
        if !fresh.is_empty() {
            pool = fresh;
        } else {
            if pool.len() > 1 {
                // Every candidate says the last answer again: the one that repeats least wins, if
                // it is an echo of the answer and clearly shorter than the longest repeat.
                let length = |index: usize| _content(&candidates[index].reply).len();
                let mut shortest = pool[0];
                for &index in &pool[1..] {
                    if length(index) < length(shortest) {
                        shortest = index;
                    }
                }
                let longest = pool.iter().map(|&index| length(index)).max().unwrap_or(0);
                if _echoes(&candidates[shortest].reply, &last, false) && length(shortest) * 3 <= longest * 2 {
                    return At(shortest);
                }
            }
            // The composer's own short form of that answer is the one line here with something
            // the person has not read.
            if let Some(shorter) = _a_shorter_said_again(candidates, &last) {
                return At(shorter);
            }
            if let Some(said) = _said_it_already(candidates, text, mem, &last) {
                pool = vec![said];
            } else if let Some(choice) = _a_choice_the_answer_did_not_make(candidates, text, mem, &last) {
                pool = vec![choice];
            } else if !pool.iter().any(|&index| _content(&candidates[index].reply) == last) {
                // Every real candidate is a slice of the last answer: it is only given up to a
                // fallback line that says what to do about the thing.
                let unheard: Vec<usize> = (0..candidates.len())
                    .filter(|&index| !_echoes(&candidates[index].reply, &last, false) && _answers_this_line(&candidates[index].reply))
                    .collect();
                if !unheard.is_empty() {
                    pool = unheard;
                }
                // Every real candidate only points back at the answer above, and a line that
                // carries a piece of that answer is on the table: that line takes the turn.
                if pool.iter().all(|&index| _points_back(&candidates[index].reply)) {
                    let says: Vec<usize> = (0..candidates.len())
                        .filter(|&index| {
                            let reply = candidates[index].reply.as_str();
                            !_points_back(reply) && !lanes::is_a_hedge(reply) && _carries_the_answer(reply, mem)
                        })
                        .collect();
                    if !says.is_empty() {
                        pool = says;
                    }
                }
            }
        }
    }
    // A detail added to news the bot has just cheered is the news again.
    if let Some(detail) = _a_detail_added_to_the_news(candidates, &pool, text, mem) {
        return At(detail);
    }
    if let Some(named) = _a_card_that_names_the_line(candidates, &pool, text, mem) {
        return At(named);
    }
    // A follow-up that adds a thing the answer above never covered is not answered by that
    // answer again.
    if let Some(added) = _the_answer_that_names_what_the_line_added(candidates, &pool, text, &last) {
        return At(added);
    }
    let mut best = pool[0];
    let (own, carried, words, clean) = _about(text, mem);
    // The same ask one turn later, in the thread of a condolence: "how do i tell a 4 year old".
    if let Some(child_words) = _the_words_for_the_child(candidates, &clean, mem, &last) {
        if !_a_whole_line_module_spoke(candidates) {
            return At(child_words);
        }
    }
    // A thank-you or a check that the bot is here names no topic of its own: the topic rule is
    // skipped and the highest PRIORITY takes the turn.
    let polite = _only_acknowledges(mem);
    let here = _a_presence_check(mem);
    if let Some(ask) = _the_ask_about_a_child(candidates, &clean, &last) {
        return At(ask);
    }
    if let Some(gave) = _the_name_they_gave(candidates, text, mem) {
        return At(gave);
    }
    if let Some(drafted) = _draft_index(candidates, &last, &clean) {
        return At(drafted);
    }
    // The card for the speaker's own life does not answer a how-to about somebody else.
    if let Some(about_them) = _about_the_person_the_line_names(candidates, &pool, &clean, &last) {
        return At(about_them);
    }
    if let Some(instead) = _instead(text, candidates, &last, &own, &clean) {
        return At(instead);
    }
    // A line that says the person is going is not a question about its reason clause.
    if let Some(bye) = _leaving_reply(candidates, &own, &carried, &words, &clean, text, mem, &last) {
        return At(bye);
    }
    if let Some(left) = _a_leaving_said_plain(candidates, &pool, &clean, mem) {
        return At(left);
    }
    if let Some(sudden) = _sudden_loss_reply(candidates, &pool, &clean) {
        return At(sudden);
    }
    if let Some(bot_answer) = _about_the_bot(candidates, &clean, &last) {
        return At(bot_answer);
    }
    // When a named place opens is not a fact any entry can hold.
    match _the_venue_hours_turn(candidates, &clean, &last, text) {
        Some(Pick::At(hours)) => return At(hours),
        Some(Pick::Nonsense) => return Pick::Nonsense,
        None => {}
    }
    // A question about the bot's own taste has no entry in the knowledge base.
    if let Some(own_taste) = _the_bots_own_taste(candidates, &clean, mem, &last) {
        return At(own_taste);
    }
    if let Some(corrected) = _store_correcting(candidates, &pool, mem, text, &last) {
        return At(corrected);
    }
    // The store taking its own suggestion apart when the line says what is out of it.
    if let Some(revised) = _the_store_revises_the_answer(candidates, &pool, mem, text, &last) {
        return At(revised);
    }
    // A bare "where do i start" asks for the one step out of the answer in hand.
    if let Some(step) = _the_move_in_hand(candidates, &real, text, &last) {
        if !_points_back(&candidates[step].reply) {
            return At(step);
        }
    }
    let written = _the_answer_written_out(mem);
    if let Some(whole) = _the_whole_move(candidates, &real, text, &last, &written) {
        if !_points_back(&candidates[whole].reply) {
            return At(whole);
        }
    }
    // "what else" after a step asks for the next part of the same answer.
    if let Some(more) = _the_next_part(candidates, text, &last) {
        if !_points_back(&candidates[more].reply) {
            return At(more);
        }
    }
    // "when do we fly": the store's date is the answer, not a card about the subject.
    if let Some(dated) = _the_date_fact(candidates, &real, text) {
        return At(dated);
    }
    // The next line of a hard story is not a request about its words.
    if let Some(hurt) = _hurting_reply(candidates, &pool, &clean) {
        return At(hurt);
    }
    if let Some(story) = _story_goes_on_reply(text, mem, candidates, &last) {
        return At(story);
    }
    if let Some(paused) = _pause_reply(candidates, &pool, text, mem, &last) {
        return At(paused);
    }
    // The line as typed, not the normalised one: the composer picks its continuation from the
    // words in front of it.
    let (kind, target) = _composer_reading(text, mem);
    let in_a_thread = kind == *THREAD || kind == *CARE_THREAD;
    if own.is_empty() && carried.is_empty() && words.is_empty() && !in_a_thread {
        // Nothing in the line to match on, so the best candidate stands.
        return At(best);
    }
    if in_a_thread {
        let care_about = if kind == *CARE_THREAD { clean.as_str() } else { "" };
        if let Some(keeping) = _thread_index(candidates, &real, &_named_topics(mem), &target, &last, care_about, mem, text) {
            return At(keeping);
        }
    }
    if polite || here {
        return At(pool[0]);
    }
    let (fit_own, fit_carried) =
        _fit(&own, &carried, mem, _priority(&candidates[best]), &candidates[best].reply, module_of(candidates, best), text);
    let mut best_score = _match(module_of(candidates, best), &fit_own, &fit_carried, &words, &clean);
    // A follow-up that asks what to do next is answered from the answer given a turn ago: an
    // entry that answers a topic the line does not name and none of the answer's words does not
    // take the turn off a reply that carries it.
    let kept = _carries_the_answer(&candidates[best].reply, mem);
    let named = _named_topics(mem);
    // A generic step from the composer stands down for an entry that already tells them what to
    // do.
    if _is_a_generic_step(&candidates[best].reply) {
        for &index in &pool[1..] {
            let entry = candidates[index].reply.as_str();
            if !entry.is_empty() && !_is_a_generic_step(entry) && _opens_on_a_job(entry) {
                return At(index);
            }
        }
    }
    let recall_from_earlier = _a_recall_from_an_earlier_turn(text, mem);
    // The closest reply to a doubt is the entry that gave the answer being doubted.
    if _reads_the_doubt(&candidates[pool[0]]) {
        for &index in &pool[1..] {
            if _carries_the_answer(&candidates[index].reply, mem) {
                return At(index);
            }
        }
    }
    for &index in &pool[1..] {
        if recall_from_earlier && MEMORY_MODULES.contains(module_of(candidates, best)) {
            // The store has answered a line that asks what was said earlier.
            continue;
        }
        if !_names_the_subject(&candidates[index], mem, &words) && _reads_the_doubt(&candidates[pool[0]]) {
            continue;
        }
        let score = _match(module_of(candidates, index), &fit_own, &fit_carried, &words, &clean);
        if score <= best_score {
            continue; // ties keep the higher PRIORITY, which comes first
        }
        // About the line's subject, and not only about one of its words.
        if _about_the_line(&candidates[best].reply, &words, &clean)
            && !_says_only_the_line(&candidates[best].reply, &words)
            && !_about_the_line(&candidates[index].reply, &words, &clean)
        {
            continue;
        }
        // A report of a step the person has just taken is not a request for that step again.
        if lanes::tells_them_the_step_again(&candidates[index].reply, text, mem)
            && !lanes::tells_them_the_step_again(&candidates[best].reply, text, mem)
        {
            continue;
        }
        if kept && _match(module_of(candidates, index), &named, &carried, &words, &clean) <= 0 {
            let entry = candidates[index].reply.as_str();
            if !(_carries_the_answer(entry, mem) || _opens_on_a_job(entry)) {
                continue;
            }
        }
        best = index;
        best_score = score;
    }
    // The person asked the bot to rule on their plan and the front of the list rules on nothing:
    // a card that names the subject of the thread takes the turn off a hand-back.
    let front = candidates[best].reply.as_str();
    if (_hands_the_decision_back(front) || (module_of(candidates, best) == "chat" && _ack_lines().contains(&_content(front))))
        && (_asks_a_question(mem) || _asks_for_a_judgement(text, mem))
    {
        let already_read = _said_before(mem, text);
        for index in 0..candidates.len() {
            if index == best || _priority(&candidates[index]) >= *SAFETY {
                continue;
            }
            let reply = candidates[index].reply.as_str();
            // A card that hands back an answer the person has already read is not an answer to
            // a hand-back either.
            if already_read.contains(&_content(reply)) || (!last.is_empty() && _echoes(reply, &last, false)) {
                continue;
            }
            if _names_what_they_said(reply, text, mem) {
                return At(index);
            }
        }
    }
    // A report of a thing is answered by what to do about it, not by the fact that explains it.
    if let Some(advice) = _the_advice_over_a_definition(candidates, &pool, text, best) {
        return At(advice);
    }
    At(_the_card_that_places_the_figure(candidates, best, text, mem))
}

/// Python: bot/modules/arbiter.py:1636 `_a_bare_doubt_reply(candidate, text, memory)`
///
/// Is this candidate the composer's own reply to a doubt with no range behind it (the call the
/// module makes, compared reply to reply)?
pub fn _a_bare_doubt_reply(candidate: &Candidate, text: &str, mem: &Mem) -> bool {
    let reply = _content(&candidate.reply);
    if reply.is_empty() {
        return false;
    }
    let Some(own) = guarded(|| lanes::reply_to_a_bare_doubt(text, mem)) else {
        return false;
    };
    let own = _content(&py_str_or_empty(Some(&own)));
    !own.is_empty() && own == reply
}

/// Python: bot/modules/arbiter.py:1660 `_places_the_figure(reply)`: the reply puts a number on
/// the thing (a range, a price, a total).
pub fn _places_the_figure(reply: &str) -> bool {
    if !A_DIGIT.is_match(reply) {
        return false;
    }
    A_PRICE.is_match(reply)
}

/// Python: bot/modules/arbiter.py:1673 `_the_card_that_places_the_figure(candidates, best, text, memory)`
///
/// A card that puts the figure against a range beats the composer's ask for a second figure.
pub fn _the_card_that_places_the_figure(candidates: &[Candidate], best: usize, text: &str, mem: &Mem) -> usize {
    if !_a_bare_doubt_reply(&candidates[best], text, mem) {
        return best;
    }
    for index in 0..candidates.len() {
        if index == best || _priority(&candidates[index]) >= *SAFETY {
            continue;
        }
        let reply = candidates[index].reply.as_str();
        if _places_the_figure(reply) && _names_what_they_said(reply, text, mem) {
            return index;
        }
    }
    best
}

/// Python: bot/modules/arbiter.py:1696 `_says_only_the_line(reply, words)`
///
/// Whether a reply is the line handed back: at most eight long words, and at least half of them
/// words the line already has.
pub fn _says_only_the_line(reply: &str, words: &Set) -> bool {
    let said: Vec<String> = LETTERS.findall_str(&text::lower(reply)).into_iter().filter(|word| clen(word) >= *LONG_ENOUGH).collect();
    if said.is_empty() {
        return false;
    }
    let had: Vec<&String> = words.iter().filter(|word| clen(word) >= *LONG_ENOUGH).collect();
    if had.is_empty() {
        return false;
    }
    let fresh = said.iter().filter(|word| !had.iter().any(|one| word.starts_with(one.as_str()) || one.starts_with(word.as_str()))).count();
    if said.len() > 8 {
        return false; // a long reply carries something of its own somewhere
    }
    fresh * 2 <= said.len()
}

/// Python: bot/modules/arbiter.py:1725 `_about_the_line(reply, words, clean='')`
///
/// Whether a reply names something of the subject the line is on, compared as subjects (the
/// crude stem of `_subject_words`), a short or filler word matched loosely only from a longer
/// one.
pub fn _about_the_line(reply: &str, words: &Set, _clean: &str) -> bool {
    let said = _subject_words(reply);
    if said.is_empty() {
        return false;
    }
    for word in words {
        let stems = _subject_words(word);
        if stems.is_empty() {
            // A short or filler word: only a longer word may be matched loosely on it.
            if clen(word) < *LONG_ENOUGH {
                continue;
            }
            for one in word_findall(&text::lower(reply)) {
                if one.starts_with(word.as_str()) || word.starts_with(one.as_str()) {
                    return true;
                }
            }
            continue;
        }
        for stem in &stems {
            for one in &said {
                if one == stem {
                    return true;
                }
                if clen(stem) >= *LONG_ENOUGH && clen(one) >= *LONG_ENOUGH && (one.starts_with(stem.as_str()) || stem.starts_with(one.as_str())) {
                    return true;
                }
            }
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:1771 `_names_the_subject(candidate, memory, words)`
///
/// Whether an entry may take a bare yes-or-no doubt that names no subject of its own: on such a
/// line it has to come back to the answer, or name the thing the conversation is on.
pub fn _names_the_subject(candidate: &Candidate, mem: &Mem, words: &Set) -> bool {
    if !_a_bare_yes_no(mem) {
        return true;
    }
    if _carries_the_answer(&candidate.reply, mem) {
        // The entry comes back to the answer the bot gave a turn ago, so the subject is its own.
        return true;
    }
    let carried = _carried_words(mem);
    let subject: Vec<&String> = carried.iter().filter(|word| !words.contains(*word) && !DOUBT_WORDS.contains(*word)).collect();
    if subject.is_empty() {
        return true;
    }
    let reply = candidate.reply.as_str();
    let line_words: Set = carried.union(words).cloned().collect();
    for word in subject {
        if clen(word) < 4 {
            continue;
        }
        if _names_the_thing(word, reply, &line_words) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:1826 `_names_the_thing(word, reply, line_words)`
///
/// Whether the reply is about the thing `word` names, not just holding the word: a word the
/// reply opens on may not be the front of a fuller name the line never said ("blood test").
pub fn _names_the_thing(word: &str, reply: &str, line_words: &Set) -> bool {
    let tokens = split(&_content(reply));
    let mut opening: Vec<&String> = Vec::new();
    for token in &tokens {
        if FIRST_VERB.contains(token) {
            break;
        }
        opening.push(token);
    }
    for (index, token) in tokens.iter().enumerate() {
        if !_the_same_plain_word(word, token) {
            continue;
        }
        if index >= opening.len() || token != word {
            // Later in the reply it describes the thing, and an inflected form is the thing.
            return true;
        }
        let fuller = opening[index + 1..]
            .iter()
            .any(|after| clen(after) >= 3 && !CARRIES_NO_SUBJECT.contains(*after) && !line_words.contains(*after));
        if !fuller {
            return true;
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:1868 `_the_same_plain_word(one, other)`
///
/// One word said the same way twice: the same letters, or the same word with a plural, a
/// possessive or "ache" on it.
pub fn _the_same_plain_word(one: &str, other: &str) -> bool {
    if one == other {
        return true;
    }
    for (word, token) in [(one, other), (other, one)] {
        if word.is_empty() || token.is_empty() {
            continue;
        }
        if let Some(rest) = token.strip_prefix(word) {
            if INFLECTED_ENDS.contains(rest) {
                return true;
            }
        }
        if word.ends_with('y') && token == format!("{}ies", drop_last(word, 1)) {
            return true; // berry / berries
        }
        if (word.ends_with("'s") || word.ends_with("s'")) && token == drop_last(word, 2) {
            return true; // boiler's / boiler
        }
        if word.ends_with('s') && !word.ends_with("ss") && token == drop_last(word, 1) {
            return true; // boilers / boiler
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:1917 `_reads_the_doubt(candidate)`
///
/// True when the reply is the composer's own ruling on the doubt (the pools `compose/verdict.py`
/// names), or its said-it-already line.
pub fn _reads_the_doubt(candidate: &Candidate) -> bool {
    guarded(|| {
        let said = candidate.reply.as_str();
        crate::compose::verdict::rules_on_the_doubt(said) || lanes::is_said_it_already(said)
    })
    .unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:1957 `_a_bare_yes_no(memory)`: the line asks yes or no and
/// names no topic of its own.
pub fn _a_bare_yes_no(mem: &Mem) -> bool {
    let memory = m(mem);
    let Some(understood) = memory.understood.as_ref() else { return false };
    if !_line_topics_of(understood).is_empty() {
        return false;
    }
    _labels_strs(&understood.asks).contains("yesno")
}

/// Python: bot/modules/arbiter.py:1967 `_carried_words(memory)`: the words of the line the
/// conversation is on, which a bare follow-up inherits.
pub fn _carried_words(mem: &Mem) -> Set {
    let topic = m(mem).get("topic").cloned();
    if let Some(topic) = topic {
        if topic.is_dict() {
            let labels = _labels(topic.get("words").unwrap_or(&Value::None));
            if !labels.is_empty() {
                return labels;
            }
            if let Some(Value::Str(said)) = topic.get("text") {
                if !text::strip(said).is_empty() {
                    return word_findall(text::strip(&text::lower(said))).into_iter().collect();
                }
            }
        }
    }
    Set::new()
}

/// Python: bot/modules/arbiter.py:1980 `_fit(own, carried, memory, best_priority, reply='', claimant='', text='')`
///
/// The topics the specialist rule may count, and the ones it may only count weakly: in front of
/// the recall band, a repeat ask, a line about the chat, or a line with no topic of its own
/// counts nothing (or only the conversation's topic) against it.
pub fn _fit(own: &Set, carried: &Set, mem: &Mem, best_priority: f64, reply: &str, claimant: &str, text: &str) -> (Set, Set) {
    if best_priority >= *RECALL {
        // Only when the recall band has the part they asked for.
        let repeat = _asks_for_the_answer_again(mem, "");
        if repeat && !_echoes(reply, &_last_reply(mem), false) {
            return (Set::new(), Set::new());
        }
        // A line that asks about the conversation is the recall band's.
        if !repeat && !claimant.is_empty() && _asks_about_the_chat(claimant, text, mem) {
            return (Set::new(), Set::new());
        }
        if _line_topics(mem).is_empty() {
            return (Set::new(), carried.clone());
        }
    }
    (own.clone(), carried.clone())
}

/// Python: bot/modules/arbiter.py:2033 `_line_topics(memory)`: the topic labels the
/// understanding engine read from this line by itself.
pub fn _line_topics(mem: &Mem) -> Set {
    match m(mem).understood.as_ref() {
        Some(understood) => _line_topics_of(understood),
        None => Set::new(),
    }
}

fn _line_topics_of(understood: &Understood) -> Set {
    _labels_strs(&understood.topics)
}

/// Python: bot/modules/arbiter.py:2072 `_an_answer_behind_the_ask(memory)`: the last thing the
/// bot said came from an answer band, so a repeat ask has an answer to hand back.
pub fn _an_answer_behind_the_ask(mem: &Mem) -> bool {
    let memory = m(mem);
    let said: Vec<&crate::state::Turn> = memory.turns.iter().filter(|one| !text::strip(&one.bot).is_empty()).collect();
    match said.last() {
        None => false,
        Some(one) => one.module.as_ref().map(|module| THE_ANSWER_BANDS.contains(module)).unwrap_or(false),
    }
}

/// Python: bot/modules/arbiter.py:2090 `_asks_for_the_answer_again(memory, text='')`
///
/// Whether the whole line is a pointer back at what the bot said, with no subject of its own.
pub fn _asks_for_the_answer_again(mem: &Mem, text: &str) -> bool {
    let line = {
        let memory = m(mem);
        let Some(understood) = memory.understood.as_ref() else { return false };
        let said = if !text.is_empty() {
            text
        } else if !understood.clean.is_empty() {
            &understood.clean
        } else {
            &understood.text
        };
        text::lower(said)
    };
    if crate::memory::repeat::asks_for_less(&line) {
        // An ask for less is an ask for the answer in shorter words, not for the same answer.
        return false;
    }
    if _the_store_reads_a_repeat(&line, mem) {
        return true;
    }
    if _the_store_reads_an_explain_again_ask(&line) {
        return true;
    }
    if !POINTS_BACK.is_match(&line) {
        return false;
    }
    let frame = _the_repeat_frame();
    LETTERS_AND_QUOTE.findall_str(&line).iter().all(|word| frame.contains(word))
}

/// Python: bot/modules/arbiter.py:2128 `_the_repeat_frame()`: the words a repeat ask is made of,
/// this file's and every word the store reads one with.
pub fn _the_repeat_frame() -> &'static Set {
    static FRAME: LazyLock<Set> = LazyLock::new(|| {
        let mut frame = REPEAT_FRAME.clone();
        frame.extend(lanes::store_repeat_words());
        frame
    });
    &FRAME
}

/// Python: bot/modules/arbiter.py:2150 `_the_store_reads_a_repeat(text, memory)`: the store read
/// the whole line as an ask for its last reply again.
pub fn _the_store_reads_a_repeat(text: &str, mem: &Mem) -> bool {
    guarded(|| crate::memory::store_replies::asks_again(text, &m(mem))).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:2174 `_the_store_reads_an_explain_again_ask(text)`: the line
/// is the ask to have the last turn put another way (`bot/memory/meant.py`).
pub fn _the_store_reads_an_explain_again_ask(text: &str) -> bool {
    let line = text::lower(text::strip(text));
    if line.is_empty() {
        return false;
    }
    for name in ["MEAN", "REPEAT"] {
        if lanes::meant(name).match_(&line).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:2202 `_a_repeat_to_hold(memory, text='')`
///
/// Whether this line is an ask to hear the last answer again, with that answer to give back.
pub fn _a_repeat_to_hold(mem: &Mem, text: &str) -> bool {
    _asks_for_the_answer_again(mem, text) && _an_answer_behind_the_ask(mem) && !_names_a_figure(text)
}

/// Python: bot/modules/arbiter.py:2218 `_a_choice_the_answer_did_not_make(candidates, text, memory, last='')`
///
/// On a line that asks to choose between options, the first fallback with words of its own (not
/// the last answer, a hedge or a pointer back) when every real candidate repeats.
pub fn _a_choice_the_answer_did_not_make(candidates: &[Candidate], _text: &str, mem: &Mem, last: &str) -> Option<usize> {
    if last.is_empty() {
        return None;
    }
    let asks = m(mem).understood.as_ref()?.asks.clone();
    if !_labels_strs(&asks).contains(CHOICE.as_str()) {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) >= 0.0 {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || reply == last || _echoes(&reply, last, false) {
            continue;
        }
        if lanes::is_a_hedge(&candidate.reply) || _points_back(&candidate.reply) {
            continue;
        }
        return Some(index);
    }
    None
}

/// Python: bot/modules/arbiter.py:2258 `_a_shorter_said_again(candidates, last)`: the composer's
/// own shorter form of the last answer, from the fallback band.
pub fn _a_shorter_said_again(candidates: &[Candidate], last: &str) -> Option<usize> {
    if last.is_empty() {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) >= 0.0 {
            continue;
        }
        if lanes::is_the_answer_shorter(&candidate.reply, last) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:2277 `_said_it_already(candidates, text, memory, last='')`
///
/// The composer's said-it-already reply, when the real candidates only repeat the opening of the
/// last answer and what they lost is what the line asked about.
pub fn _said_it_already(candidates: &[Candidate], text: &str, mem: &Mem, last: &str) -> Option<usize> {
    if last.is_empty() || !lanes::says_it_already(text, mem) {
        return None;
    }
    let real: Vec<&Candidate> = candidates.iter().filter(|candidate| _priority(candidate) >= 0.0).collect();
    if real.is_empty() {
        return None;
    }
    let first = _content(&real[0].reply);
    if !_echoes(&first, last, false) || split(&first).len() as i64 >= split(last).len() as i64 - 3 {
        return None;
    }
    // The words of a figure ask are the asking, not the thing asked about: every ask the line
    // matches is taken out of the whole line, and the shortest result is the subject.
    let mut subject = text.to_string();
    for figure in lanes::figure_asks() {
        let items = figure.items();
        if !matches!(figure, Value::List(_) | Value::Tuple(_)) || items.len() != 3 {
            continue; // `_name, ask, _says = figure` raises: the ask is skipped
        }
        let Some(ask) = items[1].as_re() else { continue }; // AttributeError: skipped
        if ask.is_match(text) {
            let shorter = ask.sub(" ", text);
            if clen(&shorter) < clen(&subject) {
                subject = shorter;
            }
        }
    }
    // A bare figure ask names no subject.
    let left: Vec<String> = LETTERS_AND_QUOTE
        .findall_str(&subject)
        .into_iter()
        .filter(|word| !lanes::figure_words().contains(word) && !lanes::figure_saying().contains(word))
        .collect();
    if left.is_empty() {
        return None;
    }
    let last_words: Set = split(last).into_iter().collect();
    let in_words = |words: &Set, word: &str| {
        words.contains(word) || words.contains(word.trim_end_matches('s')) || words.contains(&format!("{word}s"))
    };
    let asked: Vec<String> = lanes::content_words(&subject).into_iter().filter(|word| in_words(&last_words, word)).collect();
    if asked.is_empty() {
        return None;
    }
    let said: Set = split(&first).into_iter().collect();
    let missing = asked.iter().filter(|word| !in_words(&said, word)).count();
    if missing * 2 < asked.len() {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) < 0.0 && lanes::is_said_it_already(&candidate.reply) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:2355 `_ack_lines()`: the lines `chat` closes a conversation
/// with, reduced the way a reply is compared.
pub fn _ack_lines() -> &'static Set {
    static ACK_POOL: LazyLock<Set> = LazyLock::new(|| {
        guarded(|| {
            let pools = data::get("bot.modules.chat.REPLIES");
            pools.get("ack").map(|lines| lines.items().iter().map(|line| _content(&py_str(line))).collect()).unwrap_or_default()
        })
        .unwrap_or_default()
    });
    &ACK_POOL
}

/// Python: bot/modules/arbiter.py:2368 `_asks_a_question(memory)`: the understanding engine read
/// the line as a question.
pub fn _asks_a_question(mem: &Mem) -> bool {
    match m(mem).understood.as_ref() {
        Some(understood) => understood.kind == "question" || understood.intent == "question",
        None => false,
    }
}

/// Python: bot/modules/arbiter.py:2382 `_asks_for_a_judgement(text, memory=None)`: the line asks
/// the bot to rule on the plan on the table (the composer's reading).
pub fn _asks_for_a_judgement(text: &str, mem: &Mem) -> bool {
    guarded(|| crate::compose::judgement::asks_for_a_judgement(text, &m(mem))).flatten().unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:2395 `_hands_the_decision_back(reply)`: the reply hands the
/// choice back rather than answering the line (`chat.hands_it_back`).
pub fn _hands_the_decision_back(reply: &str) -> bool {
    guarded(|| crate::modules::chat::hands_it_back(reply)).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:2417 `_the_reply_is_the_report(candidate, text, memory)`: is
/// this candidate the composer's own reply to a report on the advice just given?
pub fn _the_reply_is_the_report(candidate: &Candidate, text: &str, mem: &Mem) -> bool {
    let reply = _content(&candidate.reply);
    if reply.is_empty() {
        return false;
    }
    let Some(own) = guarded(|| _content(&py_str_or_empty(Some(&crate::compose::fallback::the_report_reply(text, &m(mem)))))) else {
        return false;
    };
    !own.is_empty() && own == reply
}

/// Python: bot/modules/arbiter.py:2438 `_real_answers_besides(candidates, last='')`: the
/// candidates above the floor (not at it) that said something to the line, not chat's sign-off
/// and not the last reply again.
pub fn _real_answers_besides(candidates: &[Candidate], last: &str) -> Vec<usize> {
    let mut said = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) <= 0.0 {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == _content(last)) {
            continue;
        }
        if candidate.module == "chat" && _ack_lines().contains(&reply) {
            continue;
        }
        said.push(index);
    }
    said
}

/// Python: bot/modules/arbiter.py:2487 `_the_ask_after_the_facts(text)`: the ask at the end of a
/// line that states something first, or "".
pub fn _the_ask_after_the_facts(text: &str) -> String {
    let lowered = text::lower(text);
    let low = text::rstrip_chars(&lowered, " .?!");
    let cut = [low.rfind(','), low.rfind(';'), low.rfind('.')].into_iter().flatten().max();
    let Some(cut) = cut else { return String::new() };
    if cut == 0 {
        return String::new();
    }
    if word_findall(&low[..cut]).len() < 3 {
        return String::new();
    }
    let ask = text::strip_chars(&low[cut + 1..], " .?!");
    if THE_ASK_OPENS.match_(ask).is_some() {
        ask.to_string()
    } else {
        String::new()
    }
}

/// Python: bot/modules/arbiter.py:2503 `_only_says_the_line_back(reply, text)`: a reply made of
/// nothing but the words the person has just used (said back in the second person).
pub fn _only_says_the_line_back(reply: &str, text: &str) -> bool {
    let words = word_findall(&_content(reply));
    if words.is_empty() || words.len() > 14 {
        return false;
    }
    let mut said: Set = word_findall(&text::lower(text)).into_iter().collect();
    let voiced: Vec<String> = said.iter().map(|word| THEIR_VOICE.get(word).cloned().unwrap_or_else(|| word.clone())).collect();
    said.extend(voiced);
    let body: Vec<&String> = words.iter().filter(|word| !ECHO_STOP.contains(*word) && clen(word) > 2).collect();
    if body.is_empty() {
        return false;
    }
    body.iter().all(|word| said.contains(*word))
}

/// Python: bot/modules/arbiter.py:2522 `_the_candidates_that_answer_the_ask(text, candidates, pool)`
///
/// The pool without the replies that only say the person's own line back, or None to keep it.
pub fn _the_candidates_that_answer_the_ask(text: &str, candidates: &[Candidate], pool: &[usize]) -> Option<Vec<usize>> {
    if _the_ask_after_the_facts(text).is_empty() {
        return None;
    }
    let answering: Vec<usize> =
        pool.iter().copied().filter(|&index| !_only_says_the_line_back(&_content(&candidates[index].reply), text)).collect();
    if answering.is_empty() || answering.len() == pool.len() {
        return None;
    }
    if !answering.iter().any(|&index| _priority(&candidates[index]) >= 0.0) {
        return None;
    }
    Some(answering)
}

/// Python: bot/modules/arbiter.py:2541 `_a_card_besides(candidates, index)`: a module with a
/// subject of its own (not a pool module) proposed something beside this candidate.
pub fn _a_card_besides(candidates: &[Candidate], index: usize) -> bool {
    candidates
        .iter()
        .enumerate()
        .any(|(other, candidate)| other != index && !POOL_MODULES.contains(candidate.module) && !_content(&candidate.reply).is_empty())
}

/// Python: bot/modules/arbiter.py:2560 `_a_real_answer_besides(candidates, last='')`
pub fn _a_real_answer_besides(candidates: &[Candidate], last: &str) -> bool {
    !_real_answers_besides(candidates, last).is_empty()
}

/// Python: bot/modules/arbiter.py:2573 `_only_a_hand_back_besides(candidates, last='')`: the only
/// answer at or above the floor hands the decision back.
pub fn _only_a_hand_back_besides(candidates: &[Candidate], last: &str) -> bool {
    let mut found = false;
    for candidate in candidates {
        if _priority(candidate) < 0.0 {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == _content(last)) {
            continue;
        }
        if candidate.module == "chat" && _ack_lines().contains(&reply) {
            continue;
        }
        if !_hands_the_decision_back(&candidate.reply) {
            return false;
        }
        found = true;
    }
    found
}

/// Python: bot/modules/arbiter.py:2611 `_names_what_they_said(reply, text, memory)`: the reply
/// says a word (or a form of one) from what the person has told the bot.
pub fn _names_what_they_said(reply: &str, text: &str, mem: &Mem) -> bool {
    let mut said: Set = A_SUBJECT_WORD.findall_str(&_content(text)).into_iter().collect();
    {
        let memory = m(mem);
        for turn in memory.turns.iter().rev() {
            said.extend(A_SUBJECT_WORD.findall_str(&_content(&turn.user)));
            if said.len() > 80 {
                break;
            }
        }
    }
    let said: Set = said.into_iter().filter(|word| !NOT_A_SUBJECT_WORD.contains(word)).collect();
    let words: Set = A_SUBJECT_WORD.findall_str(&_content(reply)).into_iter().filter(|word| !NOT_A_SUBJECT_WORD.contains(word)).collect();
    if said.iter().any(|word| words.contains(word)) {
        return true;
    }
    said.iter().any(|one| words.iter().any(|other| _one_word(one, other)))
}

/// Python: bot/modules/arbiter.py:2646 `_one_word(one, other)`: the two words are one word with
/// an ending on.
pub fn _one_word(one: &str, other: &str) -> bool {
    let (shorter, longer) = if clen(one) <= clen(other) { (one, other) } else { (other, one) };
    if shorter == longer {
        return true;
    }
    if clen(shorter) < 4 {
        return false; // "go" is not the subject of "going"
    }
    let tail = longer.strip_prefix(shorter).unwrap_or("");
    let last_of_shorter = shorter.chars().last().unwrap_or(' ');
    if WORD_ENDINGS.iter().any(|ending| ending == tail) {
        let vowel = |c: char| "aeiou".contains(c);
        return !(tail.chars().next().map(vowel).unwrap_or(false) && vowel(last_of_shorter));
    }
    // The last letter doubled before the ending: "shop"/"shopping".
    let mut tail_chars = tail.chars();
    if let Some(first) = tail_chars.next() {
        let rest = tail_chars.as_str();
        if first == last_of_shorter && WORD_ENDINGS.iter().any(|ending| ending == rest) {
            return true;
        }
    }
    // The stem's "e" dropped before "-ing" or "-ed": "shine"/"shining".
    if shorter.ends_with('e') && clen(shorter) >= 5 {
        let stem = drop_last(shorter, 1);
        for ending in ["ing", "ed"] {
            if longer == format!("{stem}{ending}") {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:2690 `_the_answer_handed_back(candidates, text, memory, last)`
///
/// The store's hand-back (the store's band, any memory module) when the line asks to hear that
/// answer again: an echo of it first, then a reply made of its words.
pub fn _the_answer_handed_back(candidates: &[Candidate], text: &str, mem: &Mem, last: &str) -> Option<usize> {
    if last.is_empty() || !_a_repeat_to_hold(mem, text) {
        return None;
    }
    let band: Vec<usize> =
        (0..candidates.len()).filter(|&index| MEMORY_MODULES.contains(candidates[index].module) && _priority(&candidates[index]) >= 0.0).collect();
    for &index in &band {
        let reply = _content(&candidates[index].reply);
        if !reply.is_empty() && _echoes(&reply, last, false) {
            return Some(index);
        }
    }
    let seen: Set = split(&_content(last)).into_iter().collect();
    for &index in &band {
        let reply = _content(&candidates[index].reply);
        if !reply.is_empty() && _made_of_the_answer(&reply, &seen) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:2726 `_names_a_figure(text)`: the line asks for a number, time
/// or amount (the store's own figure asks) rather than to hear it all again.
pub fn _names_a_figure(text: &str) -> bool {
    if text::strip(text).is_empty() {
        return false;
    }
    if A_MEASURE_ASKED.is_match(text) {
        return true;
    }
    for figure in lanes::figure_asks() {
        let items = figure.items();
        if !matches!(figure, Value::List(_) | Value::Tuple(_)) || items.len() != 3 {
            continue;
        }
        let Some(ask) = items[1].as_re() else { continue };
        if ask.is_match(text) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:2750 `_as_it_comes(text, memory, candidates, last='')`
///
/// The first candidate whose module says (`TAKES_AS_IT_COMES`) this line is one to answer as it
/// comes, unless one of the conditions in the body stands it down.
pub fn _as_it_comes(text: &str, mem: &Mem, candidates: &[Candidate], last: &str) -> Option<usize> {
    for (index, candidate) in candidates.iter().enumerate() {
        let name = candidate.module;
        if !_declares_as_it_comes(name, text, mem) {
            continue;
        }
        // ...unless the answers beside it hand back the very thing the line ruled out.
        let thing = _the_thing_the_line_ruled_out(text, mem);
        let vetoed = !thing.is_empty() && _every_real_answer_offers(&thing, candidates, index);
        if _priority(candidate) < 0.0 && _a_real_answer_besides(candidates, last) && !vetoed {
            // A hand-back above the floor, against a take about what the person said...
            let hand_back = _only_a_hand_back_besides(candidates, last) && _names_what_they_said(&candidate.reply, text, mem);
            // ...or the composer's answer to a report on the advice it just gave.
            let report = _the_reply_is_the_report(candidate, text, mem)
                && (lanes::a_report_that_says_something(text, mem) || !_a_card_besides(candidates, index))
                && !_a_declaring_specialist_besides(candidates, index, mem, text);
            if !(hand_back || report) {
                continue;
            }
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == last) {
            continue;
        }
        // A module that declares the topic the line was read as outranks a generalist's take.
        if !vetoed && _priority(candidate) < *CHATTER && _a_declaring_specialist_besides(candidates, index, mem, text) {
            continue;
        }
        // A stock line that says nothing of the line stands down for a card that names it.
        if _priority(candidate) < *CHATTER && _a_card_names_the_line(candidates, index, text, mem) {
            continue;
        }
        // A claim whose answer is the last answer under a lead is not a second answer.
        if !last.is_empty()
            && (_echoes(&reply, last, true) || _the_answer_again_with_a_tail(&reply, last))
            && _a_fresh_answer_besides(candidates, index, last)
            && !_a_repeat_to_hold(mem, text)
        {
            continue;
        }
        // A claim that only says the person's own line back is not an answer to the rest of it.
        if !_the_ask_after_the_facts(text).is_empty() && _only_says_the_line_back(&reply, text) && _a_card_besides(candidates, index) {
            continue;
        }
        return Some(index);
    }
    None
}

/// Python: bot/modules/arbiter.py:2884 `_a_question_about_the_bot(text, memory)`: the line asks
/// about the bot itself (the composer's reading).
pub fn _a_question_about_the_bot(text: &str, mem: &Mem) -> bool {
    guarded(|| crate::compose::fallback::reads_as_about_itself(text, &m(mem))).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:2900 `_a_declaring_specialist_besides(candidates, index, memory, text='')`
///
/// True when another candidate's module declares a topic this line was read as, and its reply
/// is not the step the person has just reported taking.
pub fn _a_declaring_specialist_besides(candidates: &[Candidate], index: usize, mem: &Mem, text: &str) -> bool {
    let topics: Set = _line_topics(mem).into_iter().filter(|topic| !NOT_A_SUBJECT.contains(topic)).collect();
    if topics.is_empty() {
        return false;
    }
    // A question about the bot itself is not a subject, whatever topic the reading hung on it.
    if !text.is_empty() && _a_question_about_the_bot(text, mem) {
        return false;
    }
    for (other, candidate) in candidates.iter().enumerate() {
        if other == index {
            continue;
        }
        if !_declared(candidate.module).iter().any(|topic| topics.contains(topic)) {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() {
            continue;
        }
        if lanes::tells_them_the_step_again(&reply, text, mem) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/modules/arbiter.py:2936 `_a_fresh_answer_besides(candidates, index, last)`: a real
/// candidate has an answer that is not the last one said again.
pub fn _a_fresh_answer_besides(candidates: &[Candidate], index: usize, last: &str) -> bool {
    for (other, candidate) in candidates.iter().enumerate() {
        if other == index || _priority(candidate) < 0.0 {
            continue;
        }
        let reply = _content(&candidate.reply);
        if !reply.is_empty() && reply != last && !_echoes(&reply, last, true) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:2947 `_claims_instead(text, memory, candidates, last='', above=None)`
///
/// The first real candidate above `above` whose module says (`CLAIMS_INSTEAD`) the claimed line
/// is its own answer, and that says something other than the last reply.
pub fn _claims_instead(text: &str, mem: &Mem, candidates: &[Candidate], last: &str, above: Option<f64>) -> Option<usize> {
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) < 0.0 {
            continue;
        }
        if let Some(above) = above {
            if _priority(candidate) <= above {
                continue;
            }
        }
        if !_declares_claims_instead(candidate.module, text, mem) {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == last) {
            continue;
        }
        return Some(index);
    }
    None
}

/// Python: bot/modules/arbiter.py:2983 `_declares_claims_instead(module_name, text, memory)`: the
/// module's `CLAIMS_INSTEAD` hook says yes; a module without one, or one that raises, says no.
pub fn _declares_claims_instead(module_name: &str, text: &str, mem: &Mem) -> bool {
    guarded(|| hooks::claims_instead(module_name, text, mem)).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:2993 `_declares_as_it_comes(module_name, text, memory)`: the
/// module's `TAKES_AS_IT_COMES` hook says yes.
pub fn _declares_as_it_comes(module_name: &str, text: &str, mem: &Mem) -> bool {
    guarded(|| hooks::takes_as_it_comes(module_name, text, mem)).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:3003 `_asks_about_the_chat(module_name, text, memory)`: the
/// module's `ASKS_ABOUT_THE_CHAT` hook (or its own `_about_this_chat`) says the line is about the
/// conversation.
pub fn _asks_about_the_chat(module_name: &str, text: &str, mem: &Mem) -> bool {
    guarded(|| hooks::asks_about_the_chat(module_name, text, mem)).unwrap_or(false)
}

/// The share of `words` that are in `seen`.
fn share_of(words: &[String], seen: &Set) -> f64 {
    words.iter().filter(|word| seen.contains(*word)).count() as f64 / words.len() as f64
}

/// `_repeat.names_nothing(text)` and `_repeat.MOVE_ASK.search(text)` inside a try: the line asks
/// for a move out of the answer in hand. None when the reading raised.
fn asks_for_the_move(text: &str) -> Option<bool> {
    guarded(|| crate::memory::repeat::names_nothing(text) && lanes::move_ask().is_match(text))
}

/// Python: bot/modules/arbiter.py:3020 `_the_move_in_hand(candidates, real, text, last)`
///
/// On "where do i start" after an answer, the shortest candidate made of the answer's own words
/// (one clause long), where another candidate re-tells the whole answer.
pub fn _the_move_in_hand(candidates: &[Candidate], real: &[usize], text: &str, last: &str) -> Option<usize> {
    if last.is_empty() {
        return None;
    }
    if asks_for_the_move(text) != Some(true) {
        return None;
    }
    if real.len() < 2 {
        return None;
    }
    let said = split(last);
    let seen: Set = said.iter().cloned().collect();
    let mut forms: Vec<(usize, usize, bool)> = Vec::new();
    let mut whole = false;
    for &index in real {
        let words = split(&_content(&candidates[index].reply));
        if words.is_empty() {
            continue;
        }
        let of_the_answer = share_of(&words, &seen) >= *MOVE_SHARE;
        let re_tells = said.len() >= *MOVE_ANSWER_WORDS && words.len() as f64 >= said.len() as f64 * *MOVE_WHOLE;
        if re_tells {
            // A reply as long as the answer is the answer again, whatever words it picks.
            whole = true;
        }
        if of_the_answer || re_tells {
            forms.push((words.len(), index, of_the_answer));
        }
    }
    if !whole {
        return None;
    }
    // The step has to be made of the answer's own words.
    let sharing: Vec<&(usize, usize, bool)> = forms.iter().filter(|form| form.2).collect();
    if forms.len() < 2 || sharing.is_empty() {
        return None;
    }
    let &&(shortest, index, _) = sharing.iter().min().expect("sharing is not empty");
    let longest = forms.iter().map(|form| form.0).max().unwrap_or(0);
    if shortest > *MOVE_WORDS || shortest * 3 > longest * 2 {
        return None; // nothing here is a step, or the step is not clearly shorter than the re-tell
    }
    Some(index)
}

/// Python: bot/modules/arbiter.py:3097 `_the_whole_move(candidates, real, text, last, written='')`
///
/// On the same ask, the step that keeps the answer's sentence where another stops at the
/// answer's own comma; and, where nothing stops at a comma, the reply made of the answer over a
/// card about something else, or the quiet repeat over an announced one.
pub fn _the_whole_move(candidates: &[Candidate], real: &[usize], text: &str, last: &str, written: &str) -> Option<usize> {
    if last.is_empty() || real.len() < 2 {
        return None;
    }
    if asks_for_the_move(text) != Some(true) {
        return None;
    }
    // The answer as it was written: the comma a step was cut at is a comma of the answer as the
    // person read it.
    let answer = text::squash(if written.is_empty() { last } else { written });
    let lowered = text::lower(&answer);
    // `(match.group(0), match.end())` with Python's character offsets into `answer.lower()`.
    let tokens: Vec<(String, usize)> =
        TOKEN.finditer(&lowered).iter().map(|found| (found.as_str().to_string(), text::char_index(&lowered, found.end()))).collect();
    if tokens.len() < 4 {
        return None;
    }
    let seen: Set = tokens.iter().map(|(word, _)| word.clone()).collect();
    let halves = |skip_repeats: bool| {
        let (mut cut, mut whole): (Vec<(Vec<String>, Vec<String>)>, Vec<Vec<String>>) = (Vec::new(), Vec::new());
        for &index in real {
            let words = split(&_content(&candidates[index].reply));
            if words.is_empty() || words.len() > *WHOLE_MOVE_WORDS {
                continue;
            }
            if skip_repeats && _echoes(&candidates[index].reply, last, false) {
                continue;
            }
            if share_of(&words, &seen) < *MOVE_SHARE {
                continue;
            }
            let run = _run_in_the_answer(&words, &tokens, &answer);
            let at_a_comma = run.as_ref().map(|(_, end, next)| {
                // `answer[end:end + 1] in CUT_MARKS`: "" (past the end) is in every string.
                let mark = text::slice(&answer, *end as isize, Some(*end as isize + 1));
                CUT_MARKS.contains(mark.as_str()) && tokens.len() as i64 - *next as i64 >= 2
            });
            match (run, at_a_comma) {
                (Some((tail, _, _)), Some(true)) => cut.push((words, tail)),
                _ => whole.push(words),
            }
        }
        (cut, whole)
    };
    let (cut, whole) = halves(true);
    if cut.is_empty() && !whole.is_empty() {
        // Nothing here stops at the answer's comma: the reply made of the answer beats a card
        // about something else.
        let absent = real.iter().any(|&index| !_made_of_the_answer(&candidates[index].reply, &seen));
        if absent {
            for &index in real {
                if _made_of_the_answer(&candidates[index].reply, &seen) {
                    return Some(index);
                }
            }
        }
    }
    if cut.is_empty() || whole.is_empty() {
        // Every candidate says the answer over again: the one that does not announce the repeat
        // takes the turn, in the pool's own order.
        let repeats: Vec<usize> = real
            .iter()
            .copied()
            .filter(|&index| _echoes(&candidates[index].reply, last, false) && _made_of_the_answer(&candidates[index].reply, &seen))
            .collect();
        let quiet: Vec<usize> = repeats.iter().copied().filter(|&index| !_announces_a_repeat(&candidates[index].reply)).collect();
        if !quiet.is_empty() && quiet.len() < repeats.len() {
            return Some(quiet[0]);
        }
        return None;
    }
    for &index in real {
        let words = split(&_content(&candidates[index].reply));
        if words.is_empty() || !whole.contains(&words) {
            continue;
        }
        if cut.iter().any(|(_, run)| _carries_on(&words, run)) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:3203 `_made_of_the_answer(reply, seen)`: the reply is the
/// answer's own words and next to nothing else.
pub fn _made_of_the_answer(reply: &str, seen: &Set) -> bool {
    let words = split(&_content(reply));
    !words.is_empty() && share_of(&words, seen) >= *MOVE_SHARE
}

/// Python: bot/modules/arbiter.py:3209 `_the_line_is_news(text)`: the line reports the thing the
/// last answer was about coming back (`returned.says_it_happened_again`).
pub fn _the_line_is_news(text: &str) -> bool {
    crate::memory::returned::says_it_happened_again(text)
}

/// Python: bot/modules/arbiter.py:3223 `_announces_a_repeat(reply)`: the reply opens on one of
/// the lead-ins the other lanes give an answer again in.
pub fn _announces_a_repeat(reply: &str) -> bool {
    let said = text::lower(text::strip(reply));
    if said.is_empty() {
        return false;
    }
    _repeat_leads().iter().any(|lead| said.starts_with(lead.as_str()))
}

/// Python: bot/modules/arbiter.py:3239 `_repeat_leads()`: the lead-ins the other lanes put in
/// front of an answer they hand over again (`memory/topics.py` REPEAT_LEADS, `answers.AGAIN`).
pub fn _repeat_leads() -> &'static Vec<String> {
    static LEADS: LazyLock<Vec<String>> = LazyLock::new(|| {
        let mut leads = Vec::new();
        if let Some(found) = guarded(|| {
            data::get("bot.memory.topics.REPEAT_LEADS").items().iter().map(|lead| lead.items().iter().map(py_str).collect::<Vec<_>>().join(" ")).collect::<Vec<_>>()
        }) {
            leads.extend(found);
        }
        if let Some(found) = guarded(|| {
            data::get("bot.modules.answers.AGAIN")
                .items()
                .iter()
                .map(|frame| text::lower(text::strip(py_str(frame).split('{').next().unwrap_or(""))))
                .filter(|head| !head.is_empty())
                .collect::<Vec<_>>()
        }) {
            leads.extend(found);
        }
        leads
    });
    &LEADS
}

/// Python: bot/modules/arbiter.py:3259 `_carries_on(words, run)`: `run` is a stretch of `words`
/// with at least one more word after it.
pub fn _carries_on(words: &[String], run: &[String]) -> bool {
    let length = run.len();
    (0..words.len().saturating_sub(length)).any(|start| words[start..start + length] == *run)
}

/// Python: bot/modules/arbiter.py:3268 `_run_in_the_answer(words, tokens, answer)`
///
/// The longest tail of these words (eight to three) that the answer says in one go: (the tail,
/// the character offset where it ends in the answer, the index of the token after it).
pub fn _run_in_the_answer(words: &[String], tokens: &[(String, usize)], _answer: &str) -> Option<(Vec<String>, usize, usize)> {
    let mut length = words.len().min(8);
    while length > 2 {
        let tail = &words[words.len() - length..];
        if tokens.len() + 1 > length {
            for start in 0..tokens.len() - length + 1 {
                if (0..length).all(|offset| tokens[start + offset].0 == tail[offset]) {
                    return Some((tail.to_vec(), tokens[start + length - 1].1, start + length));
                }
            }
        }
        length -= 1;
    }
    None
}

/// Python: bot/modules/arbiter.py:3307 `_a_condolence(reply)`: the reply opens as comfort for a
/// loss.
pub fn _a_condolence(reply: &str) -> bool {
    let said = text::lower(&_content(reply));
    !said.is_empty() && CONDOLENCE.match_(&said).is_some()
}

/// Python: bot/modules/arbiter.py:3313 `_says_the_subject(reply, line)`: the reply says a word of
/// the line that names something (four letters or more).
pub fn _says_the_subject(reply: &str, line: &str) -> bool {
    let said: Vec<String> = word_findall(&text::lower(&_content(reply))).into_iter().filter(|word| clen(word) >= 4).collect();
    if said.is_empty() {
        return false;
    }
    for word in word_findall(line) {
        if clen(&word) < 4 || THE_NAMELESS.contains(&word) {
            continue;
        }
        if said.iter().any(|other| other.starts_with(word.as_str()) || word.starts_with(other.as_str())) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:3327 `_the_cause_asked_for(candidates, real, text, best)`
///
/// On "why is this thing failing", the entry that names the thing beats a condolence that does
/// not (and never on a line about a person).
pub fn _the_cause_asked_for(candidates: &[Candidate], real: &[usize], text: &str, best: usize) -> Option<usize> {
    if !_a_condolence(&candidates[best].reply) {
        return None;
    }
    let line = text::lower(&_content(text));
    if !THE_CAUSE.is_match(&line) {
        return None;
    }
    if _says_the_subject(&candidates[best].reply, &line) {
        // A condolence that names the thing is about it, and keeps the turn.
        return None;
    }
    if word_findall(&line).iter().any(|word| PEOPLE.contains(word)) {
        return None;
    }
    real.iter().copied().find(|&index| index != best && _says_the_subject(&candidates[index].reply, &line))
}

/// Python: bot/modules/arbiter.py:3358 `_the_date_fact(candidates, real, text)`
///
/// On a "when is it" ask, the store's date (at the recall band) beats a card about the subject,
/// unless another candidate already names the same date.
pub fn _the_date_fact(candidates: &[Candidate], real: &[usize], text: &str) -> Option<usize> {
    let asked = _content(text);
    if !crate::compose::facts::asks_for_a_when(&asked) {
        return None;
    }
    let mut store = None;
    for &index in real {
        let candidate = &candidates[index];
        if candidate.module != "memory_store" {
            continue;
        }
        if _priority(candidate) < *RECALL {
            continue;
        }
        if crate::compose::facts::the_fact_asked_for(&asked, &_content(&candidate.reply)) {
            store = Some(index);
            break;
        }
    }
    let store = store?;
    let said = Value::from(_content(&candidates[store].reply));
    for &index in real {
        if index == store {
            continue;
        }
        if crate::compose::facts::the_same_date(&_content(&candidates[index].reply), &said) {
            return None;
        }
    }
    Some(store)
}

/// Python: bot/modules/arbiter.py:3394 `_the_next_part(candidates, text, last)`
///
/// On "what else" after a step, the memory lane's next part of the answer in hand (a line that
/// opens on one of its NEXT/ALSO leads).
pub fn _the_next_part(candidates: &[Candidate], text: &str, last: &str) -> Option<usize> {
    if last.is_empty() {
        return None;
    }
    let leads = guarded(|| {
        if !crate::memory::repeat::asks_for_more(text) {
            return None;
        }
        let mut leads: Vec<String> = data::strs("bot.memory.repeat.NEXT_LEADS");
        leads.extend(data::strs("bot.memory.repeat.ALSO_LEADS"));
        Some(leads)
    })
    .flatten()?;
    _a_lead_line(candidates, &leads, last)
}

/// Python: bot/modules/arbiter.py:3419 `_a_lead_line(candidates, leads, last)`
///
/// The longest candidate that opens on one of the leads (the last such on a tie), not the reply
/// just given.
pub fn _a_lead_line(candidates: &[Candidate], leads: &[String], last: &str) -> Option<usize> {
    let openers: Vec<String> = leads
        .iter()
        .filter(|lead| !lead.is_empty())
        .map(|lead| text::lower(text::strip(lead.split('{').next().unwrap_or(""))))
        .collect();
    if openers.is_empty() {
        return None;
    }
    let mut found: Vec<(usize, usize)> = Vec::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let reply = text::strip(&candidate.reply);
        if reply.is_empty() || !openers.iter().any(|opener| text::lower(reply).starts_with(opener.as_str())) {
            continue;
        }
        if !last.is_empty() && _content(reply) == last {
            continue;
        }
        found.push((split(&_content(reply)).len(), index));
    }
    found.sort();
    found.last().map(|&(_, index)| index)
}

/// Python: bot/modules/arbiter.py:3444 `_only_acknowledges(memory)`: the line is politeness and
/// nothing else (thanks, or an ack with no ask and no topic of its own).
pub fn _only_acknowledges(mem: &Mem) -> bool {
    let memory = m(mem);
    let Some(understood) = memory.understood.as_ref() else { return false };
    if !_labels_strs(&understood.topics).is_empty() {
        return false; // "thank you for the spinach tip" is thanks and about cooking: not this
    }
    if understood.kind == "thanks" {
        return true;
    }
    understood.intent == "ack" && understood.asks.is_empty() && understood.kind != "question"
}

/// Python: bot/modules/arbiter.py:3467 `_a_presence_check(memory)`: the line checks the bot is
/// here ("you there?"), names no topic, and is not inside a loss or safety thread.
pub fn _a_presence_check(mem: &Mem) -> bool {
    let memory = m(mem);
    let Some(understood) = memory.understood.as_ref() else { return false };
    if !_labels_strs(&understood.topics).is_empty() {
        return false; // names a subject of its own, so it is about that subject
    }
    if !understood.presence {
        return false;
    }
    if let Some(turn) = memory.turns.last() {
        if matches!(turn.module.as_deref().unwrap_or(""), "loss" | "safety") {
            return false;
        }
    }
    true
}

/// Python: bot/modules/arbiter.py:3497 `_about_the_bot(candidates, clean, last='')`
///
/// On a line the composer reads as asking how the bot works (`MY_WORKINGS`), the first fallback
/// that declares no topics, when a topic-declaring module is in front.
pub fn _about_the_bot(candidates: &[Candidate], clean: &str, last: &str) -> Option<usize> {
    let kind = guarded(|| lanes::about_the_bot(clean))?;
    if !kind.as_ref().map(|kind| lanes::my_workings().contains(kind)).unwrap_or(false) {
        return None;
    }
    if candidates.is_empty() || !_declares_topics(candidates[0].module) {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) >= 0.0 || _declares_topics(candidate.module) {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == last) {
            continue;
        }
        return Some(index);
    }
    None
}

/// Python: bot/modules/arbiter.py:3533 `_the_venue_hours_turn(candidates, clean, last='', text='')`
///
/// On a question about a named venue's hours, the composer's honest hours line from the
/// fallback band. A pool that cannot be read gives Python's empty frozenset ([`Pick::Nonsense`]).
fn _the_venue_hours_turn(candidates: &[Candidate], clean: &str, last: &str, _text: &str) -> Option<Pick> {
    if !guarded(|| lanes::venue_hours(clean)).unwrap_or(false) {
        return None;
    }
    if candidates.is_empty() || !_declares_topics(candidates[0].module) {
        return None;
    }
    let Some(lines) = guarded(|| lanes::venue_hours_lines().iter().map(|line| _content(line)).collect::<Set>()) else {
        return Some(Pick::Nonsense);
    };
    if lines.is_empty() {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) >= 0.0 || _declares_topics(candidate.module) {
            continue;
        }
        let reply = _content(&candidate.reply);
        if !reply.is_empty() && lines.contains(&reply) && !(!last.is_empty() && reply == last) {
            return Some(Pick::At(index));
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:3572 `_the_bots_own_taste(candidates, clean, memory, last='')`
///
/// On a question about the bot's own taste, the composer's taste line from the fallback band,
/// when a topic-declaring module is in front.
pub fn _the_bots_own_taste(candidates: &[Candidate], clean: &str, mem: &Mem, last: &str) -> Option<usize> {
    if candidates.is_empty() || !_declares_topics(candidates[0].module) {
        return None;
    }
    let asked = guarded(|| crate::compose::fallback::reads_as_a_taste(clean, &m(mem)))?;
    if !asked {
        return None;
    }
    let lines: Set = guarded(|| data::get("bot.compose.fallback.TASTE_LINES").items().iter().map(|line| _content(&py_str(line))).collect())?;
    if lines.is_empty() {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) >= 0.0 || _declares_topics(candidate.module) {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == last) {
            continue;
        }
        if lines.contains(&reply) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:3606 `_my_workings()`: the kinds the composer claims as
/// questions about how it works.
pub fn _my_workings() -> &'static Set {
    lanes::my_workings()
}

/// Python: bot/modules/arbiter.py:3614 `_declares_topics(module_name)`: the module declares the
/// topics it answers (`ANSWERS`).
pub fn _declares_topics(module_name: &str) -> bool {
    !_answered(module_name).is_empty()
}

/// Python: bot/modules/arbiter.py:3623 `_reads_as_a_verdict(text, memory=None)`: the composer
/// reads the line as only a verdict on the news.
pub fn _reads_as_a_verdict(text: &str, mem: &Mem) -> bool {
    guarded(|| crate::compose::fallback::reads_as_a_verdict(text, &m(mem))).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:3634 `_verdict_lines()`: the composer's own lines for a
/// verdict on the news, reduced the way a reply is.
pub fn _verdict_lines() -> &'static Set {
    static LINES: LazyLock<Set> = LazyLock::new(|| {
        let mut lines = Set::new();
        for pool in ["VERDICT_ANNOY", "VERDICT_LUCK", "VERDICT_SHAME", "VERDICT_ROUGH", "VERDICT_HEAVY", "VERDICT_NUMB"] {
            if let Some(found) = data::try_get(&format!("bot.compose.fallback.{pool}")) {
                lines.extend(found.items().iter().map(py_str));
            }
        }
        lines.iter().map(|line| _content(line)).collect()
    });
    &LINES
}

/// Python: bot/modules/arbiter.py:3650 `_the_verdict_turn(candidates, lines)`: the candidate
/// whose reply IS the composer's verdict line.
pub fn _the_verdict_turn(candidates: &[Candidate], lines: &Set) -> Option<usize> {
    candidates.iter().position(|candidate| lines.contains(&_content(&candidate.reply)))
}

/// Python: bot/modules/arbiter.py:3666 `_in_a_careful_thread(clean, memory)`: the conversation
/// is inside a live emergency or its next line.
pub fn _in_a_careful_thread(clean: &str, mem: &Mem) -> bool {
    let (kind, _target) = _composer_reading(clean, mem);
    if kind == *THREAD || kind == *CARE_THREAD {
        return true;
    }
    guarded(|| crate::compose::fallback::in_a_safety_thread(&m(mem))).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:3725 `_subject_words(text)`: the words of a line that could be
/// what it is about, roughly singular.
pub fn _subject_words(text: &str) -> Set {
    let mut words = Set::new();
    for word in word_findall(&text::lower(text)) {
        if clen(&word) < 3 || NOT_A_SUBJECT_WORD.contains(&word) {
            continue;
        }
        let mut word = word;
        for ending in ["ies", "es", "s"] {
            if word.ends_with(ending) && clen(&word) > ending.len() + 2 {
                word = format!("{}{}", drop_last(&word, ending.len()), if ending == "ies" { "y" } else { "" });
                break;
            }
        }
        words.insert(word);
    }
    words
}

/// Python: bot/modules/arbiter.py:3743 `_about_the_news(reply, memory)`: a module's answer names
/// something in the news (the last turn) the verdict is about.
pub fn _about_the_news(reply: &str, mem: &Mem) -> bool {
    let mut news = Set::new();
    if let Some(turn) = m(mem).turns.last() {
        news.extend(_subject_words(&turn.user));
        news.extend(_subject_words(&turn.bot));
    }
    let words = _subject_words(reply);
    news.iter().any(|word| words.contains(word))
}

/// Python: bot/modules/arbiter.py:3762 `_says_the_same_thing_again(reply, last)`: most of what
/// the candidate says is already in the last answer.
pub fn _says_the_same_thing_again(reply: &str, last: &str) -> bool {
    if last.is_empty() {
        return false;
    }
    let words = _subject_words(reply);
    let said = _subject_words(last);
    if words.is_empty() || said.is_empty() {
        return false;
    }
    let shared = words.iter().filter(|word| said.contains(*word)).count();
    shared >= 2 && shared * 2 >= words.len()
}

/// Python: bot/modules/arbiter.py:3782 `_the_subject_was_answered(candidate, memory)`: the module
/// that answered last is saying about this subject again.
pub fn _the_subject_was_answered(candidate: &Candidate, mem: &Mem) -> bool {
    let memory = m(mem);
    let Some(last) = memory.turns.last() else { return false };
    if last.module.as_deref().unwrap_or("") != candidate.module {
        return false;
    }
    let said: Set = _subject_words(&last.bot).into_iter().filter(|word| clen(word) >= 5).collect();
    let words: Set = _subject_words(&candidate.reply).into_iter().filter(|word| clen(word) >= 5).collect();
    said.iter().filter(|word| words.contains(*word)).count() >= 2
}

/// Python: bot/modules/arbiter.py:3808 `_the_verdict_back(candidates, last='', memory=None)`
///
/// The module's own answer to the verdict: the store or chat taking the verdict up, or an answer
/// (from those or a card) about the news that is not the last module saying it again.
pub fn _the_verdict_back(candidates: &[Candidate], last: &str, mem: &Mem) -> Option<usize> {
    for module_name in KEEPS_ITS_OWN_ANSWER.iter().chain(ANSWERS_THE_NEWS.iter()) {
        let is_a_card = ANSWERS_THE_NEWS.contains(module_name);
        for (index, candidate) in candidates.iter().enumerate() {
            if candidate.module != module_name {
                continue;
            }
            if _priority(candidate) < 0.0 {
                continue;
            }
            let reply = _content(&candidate.reply);
            if reply.is_empty() {
                continue;
            }
            if !last.is_empty() && (reply == last || _echoes(&candidate.reply, last, false)) {
                continue;
            }
            if _says_the_same_thing_again(&candidate.reply, last) {
                continue;
            }
            if !is_a_card && A_VERDICT_BACK.match_(&reply).is_some() {
                return Some(index);
            }
            if _about_the_news(&candidate.reply, mem) {
                if !_the_subject_was_answered(candidate, mem) {
                    return Some(index);
                }
                continue;
            }
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:3844 `_verdict_reply(candidates, text, memory, last='', clean='', best=0)`
///
/// The candidate a bare verdict on the news is answered with, outside a careful thread and below
/// the whole-line band.
pub fn _verdict_reply(candidates: &[Candidate], text: &str, mem: &Mem, last: &str, clean: &str, best: usize) -> Option<usize> {
    if !_reads_as_a_verdict(text, mem) {
        return None;
    }
    if _priority(&candidates[best]) >= *WHOLE_LINE_BAND {
        return None;
    }
    if _in_a_careful_thread(if clean.is_empty() { text } else { clean }, mem) {
        return None;
    }
    if let Some(back) = _the_verdict_back(candidates, last, mem) {
        return Some(back);
    }
    _the_verdict_turn(candidates, _verdict_lines())
}

/// Python: bot/modules/arbiter.py:3874 `_instead(text, candidates, last='', own=(), clean='')`
///
/// The first candidate whose module answers in place of the topic entries (`ANSWERS_INSTEAD`),
/// unless an entry it defers to, a closer answer to a changed subject, or the entry that answers
/// the ask clause keeps the turn.
pub fn _instead(text: &str, candidates: &[Candidate], last: &str, own: &Set, clean: &str) -> Option<usize> {
    for (index, candidate) in candidates.iter().enumerate() {
        let module = candidate.module;
        if !_declares_instead(module) {
            continue;
        }
        if !last.is_empty() && _content(&candidate.reply) == last {
            continue; // the never-repeat rule holds in every band
        }
        if let Some(keeps) = _defers(module, candidates, index, last, own, clean, text) {
            return Some(keeps); // the entry that already answers this line better keeps the turn
        }
        // A line that puts one subject down and takes another up is answered on the new one.
        let subject = crate::compose::aboutness::after_a_change_of_subject(text);
        if !subject.is_empty() {
            let closer = crate::compose::aboutness::the_better_answer(
                &candidate.reply,
                candidates,
                &subject,
                &Value::from(index),
                &Value::from(1i64),
            );
            if !closer.is_none() {
                continue;
            }
        }
        if let Some(keeps) = _the_entry_that_answers_the_ask(candidates, index, if clean.is_empty() { text } else { clean }) {
            return Some(keeps); // the ask clause is the question, and that entry answers it
        }
        return Some(index);
    }
    None
}

/// Python: bot/modules/arbiter.py:3945 `_the_ask_clause(line)`: the long words of the last
/// clause of a two-part line, when that clause asks for something.
pub fn _the_ask_clause(line: &str) -> Vec<String> {
    if line.is_empty() {
        return Vec::new();
    }
    let parts: Vec<String> =
        ASK_CLAUSE_SPLIT.split(line).iter().map(|part| text::strip(part).to_string()).filter(|part| !part.is_empty()).collect();
    if parts.len() < 2 {
        return Vec::new();
    }
    let last = text::lower(&parts[parts.len() - 1]);
    if !ASKS_FOR_A_THING.is_match(&last) {
        return Vec::new();
    }
    let words: Vec<String> = word_findall(&last).into_iter().filter(|word| clen(word) >= *LONG_ENOUGH).collect();
    if words.len() < 2 {
        return Vec::new();
    }
    words
}

/// Python: bot/modules/arbiter.py:3967 `_the_entry_that_answers_the_ask(candidates, skip, line)`
///
/// A real, non-declaring reply that names the ask clause, where the declaring module's own reply
/// does not.
pub fn _the_entry_that_answers_the_ask(candidates: &[Candidate], skip: usize, line: &str) -> Option<usize> {
    let words: Set = _the_ask_clause(line).into_iter().collect();
    if words.is_empty() {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if index == skip || _declares_instead(candidate.module) {
            continue;
        }
        if _priority(candidate) < 0.0 {
            continue;
        }
        let reply = candidate.reply.as_str();
        if reply.is_empty() || !_about_the_line(reply, &words, "") {
            continue;
        }
        if _about_the_line(&candidates[skip].reply, &words, "") {
            return None; // both name the ask, so the module's own reading stands
        }
        return Some(index);
    }
    None
}

/// Python: bot/modules/arbiter.py:4000 `_defers(module_name, candidates, skip, last='', own=(), clean='', text='')`
///
/// The first real, non-declaring candidate on the line's topic whose reply passes the declaring
/// module's `DEFERS_TO` test.
pub fn _defers(module_name: &str, candidates: &[Candidate], skip: usize, last: &str, own: &Set, clean: &str, text: &str) -> Option<usize> {
    if !hooks::has_defers_to(module_name) {
        return None;
    }
    let none = Set::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if index == skip || _declares_instead(candidate.module) {
            continue;
        }
        if int_priority(candidate) < 0.0 {
            continue; // the fallback band apologises for everything: it cannot keep a real turn
        }
        if !own.is_empty() && _match(candidate.module, own, &none, &none, clean) <= 0 {
            continue; // only an entry that answers the topic this line names can keep the turn
        }
        if !last.is_empty() && _content(&candidate.reply) == last {
            continue; // the entry cannot keep the turn with the reply just given
        }
        let reply = candidate.reply.as_str();
        // A test that raises defers to nobody.
        if guarded(|| hooks::defers_to(module_name, reply, text)).unwrap_or(false) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4041 `_defers_to(module_name)`: whether the module declares a
/// `DEFERS_TO` test (the test itself is [`hooks::defers_to`]).
pub fn _defers_to(module_name: &str) -> bool {
    hooks::has_defers_to(module_name)
}

/// Python: bot/modules/arbiter.py:4051 `_declares_instead(module_name)`: the module answers in
/// place of the topic entries (`ANSWERS_INSTEAD`).
pub fn _declares_instead(module_name: &str) -> bool {
    !declared_attr(module_name, "ANSWERS_INSTEAD").is_empty()
}

/// Python: bot/modules/arbiter.py:4060 `_pause_reply(candidates, pool, text, memory, last='')`
///
/// On an apology for the chat pausing (or a line that only points back at it), the composer's
/// line for it, unless a whole-line module is in the pool.
pub fn _pause_reply(candidates: &[Candidate], pool: &[usize], text: &str, mem: &Mem, last: &str) -> Option<usize> {
    let said = guarded(|| {
        if _composer_reading(text, mem).0 != *PAUSE {
            return None;
        }
        let mut said: Set = data::get("bot.compose.fallback.CAME_BACK").items().iter().map(|line| _content(&py_str(line))).collect();
        said.extend(data::get("bot.compose.fallback.RETURNED_TO_IT").items().iter().map(|line| _content(&py_str(line))));
        let named = lanes::reads_as_a_return(text);
        if let Some(named) = named.filter(|named| !named.is_empty()) {
            said.extend(
                data::get("bot.compose.fallback.RETURNED_TO_A_THING")
                    .items()
                    .iter()
                    .map(|line| _content(&py_str(line).replace("{thing}", &named))),
            );
        }
        Some(said)
    })
    .flatten()?;
    let mut kept = None;
    for &index in pool {
        if _priority(&candidates[index]) >= *WHOLE_LINE_BAND {
            return None; // a module that read the whole line has the better claim
        }
    }
    for (index, candidate) in candidates.iter().enumerate() {
        let said_back = _content(&candidate.reply);
        if !said.contains(&said_back) {
            continue;
        }
        if last.is_empty() || said_back != last {
            return Some(index);
        }
        kept = kept.or(Some(index));
    }
    kept
}

/// Python: bot/modules/arbiter.py:4106 `_leaving_reply(candidates, own, carried, words, clean, text, memory, last='')`
///
/// On a line that says the person is going: small talk's own sign-off, else the composer's
/// goodbye, else the first real reply that is not about a topic of the line.
#[allow(clippy::too_many_arguments)]
pub fn _leaving_reply(
    candidates: &[Candidate],
    own: &Set,
    carried: &Set,
    words: &Set,
    clean: &str,
    text: &str,
    mem: &Mem,
    last: &str,
) -> Option<usize> {
    // The line as typed: the composer's reading needs the punctuation `clean` has lost.
    let typed = text::squash(&text::lower(text));
    let said: Set = guarded(|| {
        if !crate::compose::leaving::reads_as_leaving(&typed, None) {
            return None;
        }
        Some(lanes::leaving_lines().iter().map(|line| _content(line)).collect::<Set>())
    })
    .flatten()?;
    if _composer_reading(text, mem).0 == *THREAD {
        return None;
    }
    let small_talk = _labels_strs(&["small-talk".to_string()]);
    let reasoned = guarded(|| lanes::leaving_pool_is_the_reason_pool(&typed)).unwrap_or(false);
    for (index, candidate) in candidates.iter().enumerate() {
        // ... but only a sign-off: it asks nothing, and where the leave has a reason the
        // composer's vocabulary decides whether the reply is one.
        if _declared(candidate.module).iter().any(|topic| small_talk.contains(topic))
            && !_asks(&candidate.reply)
            && (!reasoned || _a_sign_off(&candidate.reply))
        {
            return Some(index);
        }
    }
    let mut spare = None;
    for (index, candidate) in candidates.iter().enumerate() {
        let said_back = _content(&candidate.reply);
        if said.contains(&said_back) {
            if last.is_empty() || said_back != last {
                return Some(index);
            }
            spare = spare.or(Some(index));
        }
    }
    if spare.is_some() {
        return spare;
    }
    // No sign-off among the candidates: the first reply that is not about a topic.
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) < 0.0 {
            continue;
        }
        if _match(candidate.module, own, carried, words, clean) <= 0 {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4170 `_asks(reply)`: the reply is a question.
pub fn _asks(reply: &str) -> bool {
    reply.contains('?')
}

/// Python: bot/modules/arbiter.py:4175 `_a_sign_off(reply)`: the reply lets the person go
/// (`compose/leaving.py`).
pub fn _a_sign_off(reply: &str) -> bool {
    guarded(|| crate::compose::leaving::reads_as_a_sign_off(reply)).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:4183 `_declared(module_name)`: the topics a module declares.
pub fn _declared(module_name: &str) -> Set {
    _answered(module_name)
}

/// Python: bot/modules/arbiter.py:4192 `_the_store_revises_the_answer(candidates, pool, memory, text, last='')`
///
/// The store's reply that hands the last answer back with a disliked thing taken out, on the line
/// that names the dislike.
pub fn _the_store_revises_the_answer(candidates: &[Candidate], pool: &[usize], mem: &Mem, text: &str, last: &str) -> Option<usize> {
    if last.is_empty() {
        return None;
    }
    let dislikes: Vec<String> = {
        let memory = m(mem);
        let mut held: Vec<Value> = vec![memory.get("user").cloned().unwrap_or(Value::None)];
        if let Some(others @ Value::List(_)) = memory.get("others") {
            held.extend(others.items().iter().cloned());
        }
        let mut dislikes = Vec::new();
        for person in held.iter().filter(|person| person.is_dict()) {
            if let Some(things) = person.get("dislikes").filter(|things| things.truthy()) {
                for thing in things.items() {
                    if thing.truthy() {
                        dislikes.push(text::lower(&py_str(thing)));
                    }
                }
            }
        }
        dislikes
    };
    if dislikes.is_empty() {
        return None;
    }
    let said: Set = split(&_content(text)).into_iter().collect();
    if !dislikes.iter().any(|thing| said.contains(thing)) {
        return None; // the line did not name the thing they do not eat
    }
    let previous = split(last);
    for &index in pool {
        if candidates[index].module != "memory_store" {
            continue;
        }
        let reply = candidates[index].reply.as_str();
        if _a_clause_was_cut_in_half(reply) {
            continue;
        }
        let words = split(&_content(reply));
        for start in 0..words.len().saturating_sub(4) {
            let run = &words[start..(start + 5).min(words.len())];
            if run.len() == 5 && _a_run_of(&previous, run) {
                return Some(index);
            }
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4248 `_a_clause_was_cut_in_half(reply)`: the store's revision
/// spliced a describing clause ("which", "whose") onto a list.
pub fn _a_clause_was_cut_in_half(reply: &str) -> bool {
    split(&_content(reply)).iter().any(|word| word == "which" || word == "whose")
}

/// Python: bot/modules/arbiter.py:4264 `_store_correcting(candidates, pool, memory, text, last='')`
///
/// The store's reply on the turn the person corrects what they just said
/// (`memory["correction"]` names the line).
pub fn _store_correcting(candidates: &[Candidate], pool: &[usize], mem: &Mem, text: &str, last: &str) -> Option<usize> {
    let fixed = m(mem).get("correction").cloned()?;
    if !fixed.is_dict() || !fixed.get("value").map(Value::truthy).unwrap_or(false) {
        return None;
    }
    if py_str_or_empty(fixed.get("said")) != text::squash(&text::lower(text)) {
        return None; // this is not the line that made the correction
    }
    for &index in pool {
        if candidates[index].module != "memory_store" {
            continue;
        }
        if !last.is_empty() && _content(&candidates[index].reply) == last {
            return None; // the reply just given is no answer to a correction
        }
        return Some(index);
    }
    None
}

/// Python: bot/modules/arbiter.py:4288 `_draft_index(candidates, last, clean='')`
///
/// The composer's promised draft (or its offer to write one, on a writing ask no other candidate
/// answers with words), read from the first candidate of the composer's module.
pub fn _draft_index(candidates: &[Candidate], last: &str, clean: &str) -> Option<usize> {
    use crate::compose::draft;
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.module != lanes::fallback_module() {
            continue;
        }
        let reply = candidate.reply.as_str();
        if _content(reply).is_empty() || _content(reply) == last {
            return None;
        }
        if draft::names_a_draft(reply) {
            return Some(index);
        }
        if !clean.is_empty()
            && draft::offers_a_draft(reply)
            && draft::reads_as_a_writing_ask(clean)
            && !candidates.iter().enumerate().filter(|(other, _)| *other != index).any(|(_, other)| {
                draft::offers_a_draft(&other.reply) || draft::hands_over_words(&other.reply) || draft::hands_over_the_shape(&other.reply)
            })
        {
            return Some(index);
        }
        return None;
    }
    None
}

/// Python: bot/modules/arbiter.py:4342 `_a_leaving_said_plain(candidates, pool, clean, memory)`
///
/// "my husband has left me", said as news and nothing else: the composer's line that hears it
/// beats a reply in front that hands over a job.
pub fn _a_leaving_said_plain(candidates: &[Candidate], pool: &[usize], clean: &str, mem: &Mem) -> Option<usize> {
    if clean.is_empty() {
        return None;
    }
    let leaving = guarded(|| {
        crate::compose::badnews::reads_as_the_leaving(clean) || crate::compose::badnews::reads_as_a_leaving_already_gone(clean)
    })?;
    if !leaving {
        return None;
    }
    let asks = match m(mem).understood.as_ref() {
        Some(understood) => {
            understood.kind == "question"
                || understood.intent == "question"
                || understood.intent == "request"
                || understood.help_ask
                || understood.asks_a_verdict
        }
        None => false,
    };
    if asks {
        return None;
    }
    let pools = [data::strs("bot.compose.fallback.LEFT_POOL"), data::strs("bot.compose.fallback.LEFT_POOL_GONE")];
    let index = _is_loss_reply_in(candidates, pool, &pools)?;
    let best = pool.first().copied()?;
    if !_opens_on_a_job(&candidates[best].reply) {
        return None;
    }
    Some(index)
}

/// Python: bot/modules/arbiter.py:4381 `_sudden_loss_reply(candidates, pool, clean)`
///
/// A sudden loss (or bleeding in pregnancy) gets the composer's own line for it.
pub fn _sudden_loss_reply(candidates: &[Candidate], pool: &[usize], clean: &str) -> Option<usize> {
    let pools = guarded(|| {
        if clean.is_empty() {
            return None;
        }
        if !(crate::compose::badnews::reads_as_sudden_loss(clean) || crate::compose::badnews::reads_as_pregnancy_bleeding(clean)) {
            return None;
        }
        if crate::compose::fallback::asks_what_to_tell_a_child(clean) && _a_knowledge_entry_has_it(candidates) {
            return None;
        }
        Some([
            data::strs("bot.compose.fallback.SUDDEN_LOSS"),
            data::strs("bot.compose.fallback.PREGNANCY_BLEEDING"),
            data::strs("bot.compose.fallback.THE_FIRST_HOUR_AND_A_CHILD"),
        ])
    })
    .flatten()?;
    _is_loss_reply_in(candidates, pool, &pools)
}

/// Python: bot/modules/arbiter.py:4414 `_the_words_for_the_child(candidates, clean, memory, last='')`
///
/// After a condolence, "how do i tell a 4 year old": the composer's words for a child.
pub fn _the_words_for_the_child(candidates: &[Candidate], clean: &str, mem: &Mem, last: &str) -> Option<usize> {
    let asks = guarded(|| !clean.is_empty() && crate::compose::childwords::asks_the_words_for_a_child(clean, &m(mem))).unwrap_or(false);
    if !asks {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.module != lanes::fallback_module() {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == last) {
            continue;
        }
        if _the_words_for_a_child_pool().contains(&reply) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4441 `_a_whole_line_module_spoke(candidates)`: a module at the
/// whole-line band (grief, loss) has an answer of its own.
pub fn _a_whole_line_module_spoke(candidates: &[Candidate]) -> bool {
    candidates.iter().any(|candidate| _priority(candidate) >= *WHOLE_LINE_BAND && candidate.module != lanes::fallback_module())
}

/// Python: bot/modules/arbiter.py:4457 `_the_words_for_a_child_pool()`: the composer's pool for a
/// child's words (`compose/childwords.py` THE_WORDS_FOR_A_CHILD), reduced the way the candidates
/// are.
pub fn _the_words_for_a_child_pool() -> Vec<String> {
    data::get("bot.compose.childwords.THE_WORDS_FOR_A_CHILD").items().iter().map(|line| _content(&py_str(line))).collect()
}

/// Python: bot/modules/arbiter.py:4467 `_the_ask_about_a_child(candidates, clean, last='')`
///
/// "how do I tell the child", read whole by the composer: the knowledge entry answers it.
pub fn _the_ask_about_a_child(candidates: &[Candidate], clean: &str, last: &str) -> Option<usize> {
    if !guarded(|| crate::compose::fallback::asks_what_to_tell_a_child(clean)).unwrap_or(false) {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.module != "answers" || _priority(candidate) < 0.0 {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == last) {
            continue;
        }
        return Some(index);
    }
    None
}

/// Python: bot/modules/arbiter.py:4531 `_the_name_they_gave(candidates, text, memory)`
///
/// The line that answers the bot's "what's its name?" goes to the first real reply that says
/// the name back.
pub fn _the_name_they_gave(candidates: &[Candidate], text: &str, mem: &Mem) -> Option<usize> {
    let name = crate::memory::named::the_name_given(text, &m(mem));
    if name.is_empty() {
        return None;
    }
    let said = text::lower(&name);
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) < 0.0 {
            continue;
        }
        if text::lower(&candidate.reply).contains(&said) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4554 `_about_the_person_the_line_names(candidates, pool, clean, last='')`
///
/// On a how-to about somebody the line names first ("how do i stop my teenager ..."), an answers
/// card about them beats a reply in front (below the recall band) that says nothing of them.
pub fn _about_the_person_the_line_names(candidates: &[Candidate], pool: &[usize], clean: &str, last: &str) -> Option<usize> {
    if clean.is_empty() || A_HOW_TO.match_(clean).is_none() {
        return None;
    }
    let named = THE_ONE_NAMED.search(clean)?;
    let first = A_PERSON_NAMED.search(clean);
    match first {
        Some(first) if first.start() == named.start() => {}
        // the line names somebody else first ("how do i console a friend who lost their dad")
        _ => return None,
    }
    let word = text::lower(split(named.as_str()).last().map(String::as_str).unwrap_or(""));
    if _priority(&candidates[pool[0]]) >= *RECALL {
        return None; // the recall band and the safety band are answered as they come
    }
    let front = _content(&candidates[pool[0]].reply);
    if front.is_empty() {
        return None;
    }
    let says_them = Re::new(&format!(r"\b{}s?\b", pyre::escape(word.trim_end_matches('s'))), pyre::I | pyre::U);
    if says_them.is_match(&front) || ABOUT_SOMEONE_ELSE.is_match(&front) {
        return None; // the reply in front already says something about them
    }
    for &index in &pool[1..] {
        let candidate = &candidates[index];
        if _priority(candidate) >= *RECALL || candidate.module != "answers" {
            continue;
        }
        let reply = _content(&candidate.reply);
        if reply.is_empty() || (!last.is_empty() && reply == last) {
            continue;
        }
        if lanes::is_a_hedge(&reply) {
            continue; // "I don't know enough to answer that" is not an improvement on the card
        }
        if says_them.is_match(&reply) || ABOUT_SOMEONE_ELSE.is_match(&reply) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4597 `_a_knowledge_entry_has_it(candidates)`: the knowledge
/// base (`answers`, at or above the floor) proposed an answer to this line.
pub fn _a_knowledge_entry_has_it(candidates: &[Candidate]) -> bool {
    candidates.iter().any(|candidate| int_priority(candidate) >= 0.0 && candidate.module == "answers")
}

/// Python: bot/modules/arbiter.py:4611 `_hurting_reply(candidates, pool, clean)`
///
/// A wish to hurt somebody gets the composer's own line for it (`compose/harm.py`).
pub fn _hurting_reply(candidates: &[Candidate], pool: &[usize], clean: &str) -> Option<usize> {
    if clean.is_empty() {
        return None;
    }
    if !guarded(|| crate::compose::harm::reads_as_hurting_someone(clean)).unwrap_or(false) {
        return None;
    }
    for index in pool.iter().copied().chain(0..candidates.len()) {
        // A reply we cannot read answers nothing.
        if guarded(|| crate::compose::harm::reads_as_our_reply(&candidates[index].reply)).unwrap_or(false) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4636 `_story_goes_on_reply(text, memory, candidates, last='')`
///
/// The next line of a hard story (after a death, a leaving, or a refusal) gets the memory lane's
/// listening line.
pub fn _story_goes_on_reply(text: &str, mem: &Mem, candidates: &[Candidate], last: &str) -> Option<usize> {
    guarded(|| {
        if !crate::memory::ack::carries_the_story_on(text, &m(mem)) {
            return None;
        }
        if !_a_story_opened(mem) && !_a_refusal(text) && candidates.iter().any(|candidate| _priority(candidate) >= 0.0) {
            // Named news about somebody living: the module that answered it keeps the turn.
            return None;
        }
        let pool = data::strs("bot.memory.ack.THE_STORY_GOES_ON");
        for (index, candidate) in candidates.iter().enumerate() {
            if int_priority(candidate) >= *SAFETY {
                continue; // a real emergency is never passed over for a listening line
            }
            let reply = text::strip(&candidate.reply);
            if reply.is_empty() || (!last.is_empty() && _content(reply) == last) {
                continue;
            }
            if pool.iter().any(|line| line == reply) {
                return Some(index);
            }
        }
        None
    })
    .flatten()
}

/// Python: bot/modules/arbiter.py:4680 `_a_story_opened(memory)`: the last turn or two said a
/// death or a leaving (not a dream of one), rather than news about somebody living.
pub fn _a_story_opened(mem: &Mem) -> bool {
    let said_lines: Vec<String> = {
        let memory = m(mem);
        let turns = &memory.turns;
        turns[turns.len().saturating_sub(2)..].iter().map(|turn| text::lower(text::strip(&turn.user))).collect()
    };
    for said in said_lines {
        if said.is_empty() {
            continue;
        }
        if crate::memory::dream::named(&said) && !crate::memory::dream::a_death_came_first(&said) {
            // A dream is not a story of a death that happened.
            continue;
        }
        if crate::compose::loss::reads_as_loss(&said, None, false) {
            return true;
        }
        match guarded(|| crate::compose::badnews::reads_as_the_leaving(&said) || crate::compose::badnews::reads_as_a_leaving_already_gone(&said)) {
            Some(true) => return true,
            _ => continue,
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:4721 `_a_refusal(text)`: somebody refusing something.
pub fn _a_refusal(text: &str) -> bool {
    REFUSAL.is_match(&text::lower(text).replace('\u{2019}', "'"))
}

/// Python: bot/modules/arbiter.py:4726 `_is_loss_reply_in(candidates, pool, pools)`: the first
/// candidate (the pool first, then every candidate) whose reply is one of these pools' lines.
pub fn _is_loss_reply_in(candidates: &[Candidate], pool: &[usize], pools: &[Vec<String>]) -> Option<usize> {
    for index in pool.iter().copied().chain(0..candidates.len()) {
        let reply = text::strip(&candidates[index].reply);
        if !reply.is_empty() && pools.iter().any(|wanted| wanted.iter().any(|line| line == reply)) {
            return Some(index);
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4735 `_composer_reading(clean, memory)`
///
/// What the composer says this line is: (kind, target), or ("", ()) for an ordinary line or a
/// reading that raised.
pub fn _composer_reading(clean: &str, mem: &Mem) -> (String, Value) {
    let reading = guarded(|| {
        let reading = crate::compose::fallback::reading(clean, &m(mem));
        // `kind, target = ...`: anything but a pair raises.
        let items = reading.items();
        assert!(matches!(reading, Value::List(_) | Value::Tuple(_)) && items.len() == 2, "reading is not a pair");
        (items[0].clone(), items[1].clone())
    });
    match reading {
        Some((kind, target)) if kind.truthy() => (py_str(&kind), target),
        _ => (String::new(), Value::Tuple(Default::default())),
    }
}

/// Python: bot/modules/arbiter.py:4752 `_carries_the_answer(reply, memory)`: the reply comes back
/// to the answer the bot gave a turn ago (`compose/verdict.py`).
pub fn _carries_the_answer(reply: &str, mem: &Mem) -> bool {
    if reply.is_empty() {
        return false;
    }
    guarded(|| crate::compose::verdict::carries_the_answer(reply, &m(mem))).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:4768 `_a_check_on_the_answer(reply)`: the reply is the
/// composer's answer to a check on the answer above.
pub fn _a_check_on_the_answer(reply: &str) -> bool {
    if reply.is_empty() {
        return false;
    }
    guarded(|| crate::compose::verdict::a_check_on_the_reply(reply)).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:4784 `_opens_on_a_job(reply)`: the reply opens on an
/// instruction.
pub fn _opens_on_a_job(reply: &str) -> bool {
    if reply.is_empty() {
        return false;
    }
    guarded(|| crate::compose::verdict::opens_on_a_job(reply)).unwrap_or(false)
}

/// Python: bot/modules/arbiter.py:4795 `_is_a_generic_step(reply)`: one of the composer's own
/// generic first-step lines (`compose.STEP_WHEN_LOW`).
pub fn _is_a_generic_step(reply: &str) -> bool {
    if reply.is_empty() || STEP_WHEN_LOW.is_empty() {
        return false;
    }
    let reply = text::strip(reply);
    STEP_WHEN_LOW.iter().any(|line| line == reply)
}

/// Python: bot/modules/arbiter.py:4808 `_named_topics(memory)`: the topics the line itself names
/// (`understood["topics"]`), small talk left out.
pub fn _named_topics(mem: &Mem) -> Set {
    match m(mem).understood.as_ref() {
        Some(understood) => _labels_strs(&understood.topics).into_iter().filter(|topic| !CHITCHAT.contains(topic)).collect(),
        None => Set::new(),
    }
}

/// Python: bot/modules/arbiter.py:4832 `_thread_index(candidates, real, own, target, last, care_about='', memory=None, line='')`
///
/// The composer's continuation for a safety thread (the composer's module in the fallback band),
/// unless a module that declares the line's own topic answers it; inside a care thread the reply
/// that keeps more of the thread's answer wins.
#[allow(clippy::too_many_arguments)]
pub fn _thread_index(
    candidates: &[Candidate],
    real: &[usize],
    own: &Set,
    target: &Value,
    last: &str,
    care_about: &str,
    mem: &Mem,
    line: &str,
) -> Option<usize> {
    // `text = target[0] if target else ""`
    let said = if target.truthy() { target.items().first().map(py_str).unwrap_or_default() } else { String::new() };
    if _content(&said).is_empty() {
        return None;
    }
    let asks = lanes::asks_about_a_care_answer(care_about, last, mem, own);
    let pointed_at = crate::compose::outlook::points_at_the_thread(if line.is_empty() { &said } else { line });
    if !pointed_at
        && real.iter().any(|&index| !own.is_empty() && _labels_set(&_answered(candidates[index].module)).iter().any(|topic| own.contains(topic)))
        && !asks
    {
        return None;
    }
    for (index, candidate) in candidates.iter().enumerate() {
        if _priority(candidate) < 0.0 && candidate.module == lanes::fallback_module() {
            let reply = _content(&candidate.reply);
            if reply.is_empty() || reply == last {
                return None; // nothing to say, or the same line again
            }
            if asks {
                // The reply that says more of the answer in hand continues the conversation.
                let mine = crate::compose::fallback::how_much_of_the_answer_kept(&candidate.reply, &m(mem));
                for &other in real {
                    let closer = crate::compose::fallback::how_much_of_the_answer_kept(&candidates[other].reply, &m(mem));
                    if closer > mine {
                        return Some(other);
                    }
                }
            }
            return Some(index); // the composer's continuation for this thread
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:4906 `_answers_this_line(reply)`: a line from below the floor
/// that tells the person something to do (not the said-it-already line).
pub fn _answers_this_line(reply: &str) -> bool {
    if reply.is_empty() || lanes::is_said_it_already(reply) {
        return false;
    }
    _opens_on_a_job(reply)
}

/// Python: bot/modules/arbiter.py:4922 `_content(reply)`: a reply reduced to what it says, for the
/// never-repeat rule.
pub fn _content(reply: &str) -> String {
    word_findall(&text::lower(reply)).join(" ")
}

/// Python: bot/modules/arbiter.py:4927 `_echoes(reply, last, announced=False)`
///
/// True when a reply is only words of the last answer: up to six words of lead-in and then a run
/// of twelve or more to the end; with `announced`, also a lead the lanes write for a repeat
/// followed by eight or more of the answer's words.
pub fn _echoes(reply: &str, last: &str, announced: bool) -> bool {
    let words = split(&_content(reply));
    let previous = split(&_content(last));
    if previous.len() < 12 || words.len() < 12 {
        return false;
    }
    for lead in 0..=(words.len() - 12).min(6) {
        let run = &words[lead..];
        if run.len() >= 12 && _a_run_of(&previous, run) {
            return true;
        }
    }
    if announced {
        if let Some(handed) = _the_quoted_repeat(reply) {
            if !handed.is_empty() {
                let run = split(&handed);
                if run.len() >= 8 && _a_run_of(&previous, &run) {
                    return true;
                }
            }
        }
    }
    false
}

/// Python: bot/modules/arbiter.py:4976 `_the_answer_again_with_a_tail(reply, last)`: the reply
/// opens on the whole last answer (sixteen words or more) and adds at most twelve of its own.
pub fn _the_answer_again_with_a_tail(reply: &str, last: &str) -> bool {
    let words = split(&_content(reply));
    let previous = split(&_content(last));
    if previous.len() < *AGAIN_FIRST_WORDS {
        return false;
    }
    let mut same = 0;
    while same < words.len() && same < previous.len() && words[same] == previous[same] {
        same += 1;
    }
    same >= *AGAIN_FIRST_WORDS && words.len() - same <= *AGAIN_TAIL_WORDS
}

/// Python: bot/modules/arbiter.py:5007 `_the_quoted_repeat(reply)`: the part of the last answer a
/// repeat lead says it is handing back, or None.
pub fn _the_quoted_repeat(reply: &str) -> Option<String> {
    let said = _content(reply);
    if said.is_empty() {
        return None;
    }
    let mut heads: Vec<String> = _repeat_leads().iter().map(|lead| _content(lead)).collect();
    if let Some(found) = guarded(|| {
        data::get("bot.memory.repeat.LEADS").items().iter().map(|frame| _content(py_str(frame).split('{').next().unwrap_or(""))).collect::<Vec<_>>()
    }) {
        heads.extend(found);
    }
    for head in heads {
        if !head.is_empty() && said.starts_with(head.as_str()) {
            return Some(text::strip(&said[head.len()..]).to_string());
        }
    }
    None
}

/// Python: bot/modules/arbiter.py:5052 `_points_back(reply)`: the reply only says the answer above
/// covers the thing.
pub fn _points_back(reply: &str) -> bool {
    A_POINTS_BACK.is_match(reply)
}

/// Python: bot/modules/arbiter.py:5057 `_a_run_of(previous, run)`: `run` is a continuous run of
/// words inside `previous`.
pub fn _a_run_of(previous: &[String], run: &[String]) -> bool {
    let size = run.len();
    if size > previous.len() {
        return false;
    }
    (0..=previous.len() - size).any(|start| previous[start..start + size] == *run)
}

/// Python: bot/modules/arbiter.py:5064 `_the_answer_written_out(memory)`: the last answer as the
/// person read it.
pub fn _the_answer_written_out(mem: &Mem) -> String {
    m(mem).turns.last().map(|turn| turn.bot.clone()).unwrap_or_default()
}

/// Python: bot/modules/arbiter.py:5072 `_last_reply(memory)`: the last answer, reduced.
pub fn _last_reply(mem: &Mem) -> String {
    m(mem).turns.last().map(|turn| _content(&turn.bot)).unwrap_or_default()
}

/// Python: bot/modules/arbiter.py:5079 `_said_before(memory, text)`
///
/// Whole answers (eight words or more) the bot has already given this person, reduced, leaving
/// out a turn whose question is this line word for word. Python's set; kept here in the order the
/// turns gave them (see `_choose`, the only reader that walks it).
pub fn _said_before(mem: &Mem, text: &str) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    let asked = _content(text);
    for turn in &m(mem).turns {
        if !asked.is_empty() && _content(&turn.user) == asked {
            continue;
        }
        let reply = _content(&turn.bot);
        if split(&reply).len() >= *SAID_BEFORE_WORDS && !said.contains(&reply) {
            said.push(reply);
        }
    }
    said
}

/// Python: bot/modules/arbiter.py:5107 `_about(text, memory)`
///
/// What this turn is about: (the topics the reading gives it, the topic only a stored record
/// names, the line's words, the whole line), all lower case.
pub fn _about(text: &str, mem: &Mem) -> (Set, Set, Set, String) {
    let (own, mut words, stored) = {
        let memory = m(mem);
        let (mut own, mut words) = (Set::new(), Set::new());
        if let Some(understood) = memory.understood.as_ref() {
            own.extend(_labels_strs(&understood.topics));
            if let Some(topic) = understood.topic.as_deref() {
                own.extend(_labels_strs(&[topic.to_string()]));
            }
            for word in &understood.words {
                words.extend(word_findall(&text::lower(word)));
            }
        }
        (own, words, memory.get("topic").cloned().unwrap_or(Value::None))
    };
    let carried: Set = _labels(&stored).into_iter().filter(|label| !own.contains(label)).collect();
    let clean = word_findall(&text::lower(text)).join(" ");
    words.extend(split(&clean));
    (own, carried, words, clean)
}

/// Python: bot/modules/arbiter.py:5134 `_strings(value)`: the strings in a value, one level deep.
pub fn _strings(value: &Value) -> Vec<String> {
    match value {
        Value::Str(s) => vec![s.to_string()],
        Value::List(_) | Value::Tuple(_) | Value::Set(_) => value.items().iter().filter_map(|item| item.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

/// Python: bot/modules/arbiter.py:5143 `_label(value)`: one topic label, canonical ("small-talk",
/// "Small talk" and "small_talk" are "small talk").
pub fn _label(value: &str) -> String {
    word_findall(&text::lower(value)).join(" ")
}

/// Python: bot/modules/arbiter.py:5149 `_labels(value)`
///
/// The topic labels in a value: a label, a list of them, a record (`_stored`'s reading of its line
/// plus its label key), or a mapping of topic to score.
pub fn _labels(value: &Value) -> Set {
    let mut found = Set::new();
    match value {
        Value::Str(s) => {
            found.insert(_label(s));
        }
        Value::Dict(_) => {
            found.extend(_stored(value));
            let mut labelled = false;
            for key in LABEL_KEYS.iter() {
                if let Some(Value::Str(label)) = value.get(key) {
                    found.insert(_label(label));
                    labelled = true;
                    break;
                }
            }
            if !labelled {
                if value.pairs().iter().any(|(key, _)| key.as_str() == Some("topics")) {
                    found.extend(_labels(value.get("topics").unwrap_or(&Value::None)));
                } else if !value.pairs().is_empty()
                    && value.pairs().iter().all(|(_, item)| !matches!(item, Value::Dict(_) | Value::List(_) | Value::Tuple(_)))
                {
                    for (key, _) in value.pairs() {
                        // a mapping of topic -> score
                        if let Some(key) = key.as_str() {
                            if !LABEL_KEYS.iter().any(|label_key| label_key == key) {
                                found.insert(_label(key));
                            }
                        }
                    }
                }
            }
        }
        Value::List(_) | Value::Tuple(_) | Value::Set(_) => {
            for item in value.items() {
                found.extend(_labels(item));
            }
        }
        _ => {}
    }
    found.remove("");
    found
}

/// `_labels` of a list of strings (the reading's typed lists).
fn _labels_strs(values: &[String]) -> Set {
    let mut found: Set = values.iter().map(|value| _label(value)).collect();
    found.remove("");
    found
}

/// `_labels` of a set of labels (re-reading labels already made).
fn _labels_set(values: &Set) -> Set {
    let mut found: Set = values.iter().map(|value| _label(value)).collect();
    found.remove("");
    found
}

/// Python: bot/modules/arbiter.py:5182 `_stored(record)`: the topic of the line a record keeps
/// (its "text", else its "words"), read with the understanding engine.
pub fn _stored(record: &Value) -> Set {
    let mut said = match record.get("text") {
        Some(Value::Str(s)) => s.to_string(),
        _ => String::new(),
    };
    if text::strip(&said).is_empty() {
        said = _strings(record.get("words").unwrap_or(&Value::None)).join(" ");
    }
    _read_topic(&said)
}

/// Python: bot/modules/arbiter.py:5194 `_read_topic(text)`: the topic id the understanding engine
/// gives a line (score 2 or more), cached by the line's words (`READ`).
pub fn _read_topic(text: &str) -> Set {
    let key = word_findall(&text::lower(text)).join(" ");
    if key.is_empty() {
        return Set::new();
    }
    if let Some(found) = READ.lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return found.clone();
    }
    let found = guarded(|| {
        use crate::understanding::{normalize, topics};
        let read = normalize::normalise(&key);
        let entities = topics::detect_entities(&read, None);
        let (topic, score) = topics::detect_topic(&read, None, Some(&entities), None);
        match topic {
            Some(topic) if !topic.is_empty() && score >= 2 => [_label(&topic)].into_iter().collect(),
            _ => Set::new(),
        }
    })
    .unwrap_or_default();
    READ.lock().unwrap_or_else(|e| e.into_inner()).insert(key, found.clone());
    found
}

/// The labels a module declares under one attribute (`ANSWERS`, `ANSWERS_INSTEAD`), read once.
fn declared_attr(module_name: &str, attr: &str) -> Set {
    static CACHE: LazyLock<std::sync::Mutex<HashMap<(String, String), Set>>> = LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
    let key = (module_name.to_string(), attr.to_string());
    if let Some(found) = CACHE.lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return found.clone();
    }
    // `getattr(module, attr, ())`: the bundle holds every respond module's declaration.
    let found = match data::try_get(&format!("bot.modules.{module_name}.{attr}")) {
        Some(value) => _labels(value),
        None => Set::new(),
    };
    CACHE.lock().unwrap_or_else(|e| e.into_inner()).insert(key, found.clone());
    found
}

/// Python: bot/modules/arbiter.py:5213 `_answered(module_name)`: the topic labels a module
/// declares it answers (`ANSWERS`).
pub fn _answered(module_name: &str) -> Set {
    declared_attr(module_name, "ANSWERS")
}

/// Python: bot/modules/arbiter.py:5223 `_match(module_name, own, carried, words, clean)`
///
/// How well a module's declared topics fit this turn: 2 for a topic the reading gives, 1 for one
/// only a stored record names, 1 for a declared phrase or word the line contains.
pub fn _match(module_name: &str, own: &Set, carried: &Set, words: &Set, clean: &str) -> i64 {
    let mut score = 0;
    let padded = format!(" {clean} ");
    for label in _answered(module_name) {
        if own.contains(&label) {
            score += 2;
        } else if carried.contains(&label) {
            score += 1;
        } else if label.contains(' ') {
            if padded.contains(&format!(" {label} ")) {
                score += 1;
            }
        } else if words.contains(&label) {
            score += 1;
        }
    }
    score
}

/// Python: bot/modules/arbiter.py:5248 `_the_subject_words(text)`: the words the line says its
/// subject with (what follows a determiner).
pub fn _the_subject_words(text: &str) -> Set {
    let mut words = Set::new();
    for phrase in A_SUBJECT_PHRASE.findall_str(&text::lower(text)) {
        words.extend(A_SUBJECT_WORD.findall_str(&phrase).into_iter().filter(|word| !NOT_A_SUBJECT_WORD.contains(word)));
    }
    words
}

/// Python: bot/modules/arbiter.py:5257 `_the_lines_own_words(text, memory)`: the words of the
/// line (and of the reading's subject) that say what it is about.
pub fn _the_lines_own_words(text: &str, mem: &Mem) -> Set {
    let mut words = _the_subject_words(text);
    if let Some(understood) = m(mem).understood.as_ref() {
        let subject = understood.subject.clone().unwrap_or_default();
        if !subject.is_empty() {
            words.extend(A_SUBJECT_WORD.findall_str(&text::lower(&subject)));
        }
        words.extend(_the_subject_words(&subject).into_iter().filter(|word| !NOT_A_SUBJECT_WORD.contains(word)));
    }
    words.into_iter().filter(|word| !NOT_A_SUBJECT_WORD.contains(word)).collect()
}

/// Python: bot/modules/arbiter.py:5270 `_the_forms_of(word)`: the word and the endings English
/// puts on it.
pub fn _the_forms_of(word: &str) -> Set {
    let mut forms: Set = [word.to_string()].into_iter().collect();
    let n = clen(word);
    if word.ends_with('s') && n > 3 {
        forms.insert(drop_last(word, 1));
        if word.ends_with("es") && n > 4 {
            forms.insert(drop_last(word, 2));
        }
    }
    if word.ends_with("ed") && n > 4 {
        forms.insert(drop_last(word, 2));
        forms.insert(drop_last(word, 1));
    }
    if word.ends_with("ing") && n > 5 {
        let stem = drop_last(word, 3);
        forms.insert(format!("{stem}e"));
        forms.insert(stem);
    }
    forms.into_iter().filter(|form| clen(form) >= 3).collect()
}

/// Python: bot/modules/arbiter.py:5284 `_the_same_word(said, word)`: `said` is a form of `word`.
/// (The later of the file's two definitions of this name, the one Python binds.)
pub fn _the_same_word(said: &str, word: &str) -> bool {
    _the_forms_of(word).contains(said)
}

/// Python: bot/modules/arbiter.py:5289 `_names_a_word_of(a_reply, words)`: the reply says one of
/// `words` as a word of its own.
pub fn _names_a_word_of(a_reply: &str, words: &Set) -> bool {
    let said = A_SUBJECT_WORD.findall_str(&_content(a_reply));
    said.iter().any(|one| words.iter().any(|word| _the_same_word(one, word)))
}

/// Python: bot/modules/arbiter.py:5295 `_a_card_names_the_line(candidates, index, text, memory)`:
/// another real candidate names the line's own words and this one names none.
pub fn _a_card_names_the_line(candidates: &[Candidate], index: usize, text: &str, mem: &Mem) -> bool {
    let words = _the_lines_own_words(text, mem);
    if words.is_empty() {
        return false;
    }
    if _names_a_word_of(&candidates[index].reply, &words) {
        return false;
    }
    candidates
        .iter()
        .enumerate()
        .any(|(other, candidate)| other != index && _priority(candidate) >= 0.0 && _names_a_word_of(&candidate.reply, &words))
}

/// Python: bot/modules/arbiter.py:5317 `_a_detail_added_to_the_news(candidates, pool, text, memory)`
///
/// On a detail added to news the bot has just cheered, the real candidate that is the composer's
/// detail line (`compose/goodnews.py`).
pub fn _a_detail_added_to_the_news(candidates: &[Candidate], pool: &[usize], text: &str, mem: &Mem) -> Option<usize> {
    if !crate::compose::goodnews::reads_as_a_detail_of_the_news(text, &m(mem)) {
        return None;
    }
    pool.iter()
        .copied()
        .find(|&index| _priority(&candidates[index]) >= 0.0 && crate::compose::goodnews::reads_as_the_detail_line(&candidates[index].reply))
}

/// Python: bot/modules/arbiter.py:5337 `_shares_a_stem(one, other)`: the same first five letters.
pub fn _shares_a_stem(one: &str, other: &str) -> bool {
    clen(one) >= 5 && clen(other) >= 5 && one.chars().take(5).eq(other.chars().take(5))
}

/// Python: bot/modules/arbiter.py:5349 `_the_word_the_line_added(text, last)`: the longest word
/// of the line (five letters or more, not glue) that the answer above does not have, else "".
pub fn _the_word_the_line_added(text: &str, last: &str) -> String {
    let said = word_findall(&text::lower(last));
    let words = A_SUBJECT_WORD
        .findall_str(&text::lower(text))
        .into_iter()
        .filter(|word| clen(word) >= 5 && !NOT_A_SUBJECT_WORD.contains(word) && !NOT_A_NEW_WORD.contains(word));
    let mut best = String::new();
    for word in words {
        if said.iter().any(|one| _shares_a_stem(&word, one)) {
            continue;
        }
        if best.is_empty() || clen(&word) > clen(&best) {
            best = word;
        }
    }
    best
}

/// Python: bot/modules/arbiter.py:5366 `_says_nothing_new(reply, last)`: at most one word of
/// substance in the reply is not in the answer above.
pub fn _says_nothing_new(reply: &str, last: &str) -> bool {
    let said: Set = word_findall(&_content(last)).into_iter().collect();
    let fresh = word_findall(&_content(reply))
        .into_iter()
        .filter(|word| clen(word) >= 4 && !NOT_A_SUBJECT_WORD.contains(word) && !NOT_A_NEW_WORD.contains(word))
        .filter(|word| !said.contains(word) && !said.iter().any(|one| _shares_a_stem(word, one)))
        .count();
    fresh <= 1
}

/// Python: bot/modules/arbiter.py:5385 `_names_the_word(a_reply, word)`: the reply says the word,
/// endings and all.
pub fn _names_the_word(a_reply: &str, word: &str) -> bool {
    word_findall(&_content(a_reply)).iter().any(|said| _shares_a_stem(said, word))
}

/// Python: bot/modules/arbiter.py:5390 `_the_answer_that_names_what_the_line_added(candidates, pool, text, last)`
///
/// The real candidate that names the word the line added, where the one in front (below the
/// recall band) says nothing new and leaves the word out.
pub fn _the_answer_that_names_what_the_line_added(candidates: &[Candidate], pool: &[usize], text: &str, last: &str) -> Option<usize> {
    if pool.len() < 2 || last.is_empty() {
        return None;
    }
    let added = _the_word_the_line_added(text, last);
    if added.is_empty() {
        return None;
    }
    let front = &candidates[pool[0]];
    if !_says_nothing_new(&front.reply, last) {
        return None;
    }
    if _names_the_word(&front.reply, &added) {
        return None;
    }
    if _priority(front) >= *RECALL {
        return None;
    }
    pool[1..].iter().copied().find(|&index| _priority(&candidates[index]) >= 0.0 && _names_the_word(&candidates[index].reply, &added))
}

/// Python: bot/modules/arbiter.py:5421 `_a_card_that_names_the_line(candidates, pool, text, memory)`
///
/// A real card that names the line where the take in front (below the chatter band, and not the
/// store answering a recall) names nothing of it.
pub fn _a_card_that_names_the_line(candidates: &[Candidate], pool: &[usize], text: &str, mem: &Mem) -> Option<usize> {
    if pool.len() < 2 || _priority(&candidates[pool[0]]) >= *CHATTER {
        return None;
    }
    if MEMORY_MODULES.contains(candidates[pool[0]].module) && _a_recall_from_an_earlier_turn(text, mem) {
        return None;
    }
    let words = _the_lines_own_words(text, mem);
    if words.is_empty() || _names_a_word_of(&candidates[pool[0]].reply, &words) {
        return None;
    }
    pool[1..].iter().copied().find(|&index| _priority(&candidates[index]) >= 0.0 && _names_a_word_of(&candidates[index].reply, &words))
}

/// Python: bot/modules/arbiter.py:5484 `_a_line_asking_for_plainer_words(text)`: the line is only
/// the ask to have the last answer put a plainer way.
pub fn _a_line_asking_for_plainer_words(text: &str) -> bool {
    let clean = _content(text);
    if clean.is_empty() || !A_PLAINER_ASK.is_match(&clean) {
        return false;
    }
    let rest = A_PLAINER_ASK.sub(" ", &clean);
    A_SUBJECT_WORD.findall_str(&text::lower(&rest)).iter().all(|word| PLAIN_ASK_FILLER.contains(word))
}

/// Python: bot/modules/arbiter.py:5499 `_says_the_answer_again(reply, last)`: the reply is the
/// previous answer, with a lead in front of it or without one.
pub fn _says_the_answer_again(reply: &str, last: &str) -> bool {
    if reply.is_empty() || last.is_empty() {
        return false;
    }
    let said = _content(reply);
    if !said.is_empty() && said == _content(last) {
        return true;
    }
    if _the_quoted_repeat(reply).is_some() {
        return true;
    }
    _echoes(reply, last, true)
}

/// Python: bot/modules/arbiter.py:5511 `_a_plainer_reading_of_the_same_answer(candidates, best, text, memory)`
///
/// On an ask to have the answer put plainer, where the turn in hand is the answer again: the
/// highest real candidate that says something new about the answer's subject.
pub fn _a_plainer_reading_of_the_same_answer(candidates: &[Candidate], best: usize, text: &str, mem: &Mem) -> Option<usize> {
    if !_a_line_asking_for_plainer_words(text) {
        return None;
    }
    let last = _last_reply(mem);
    if last.is_empty() || !_says_the_answer_again(&candidates[best].reply, &last) {
        return None;
    }
    let about = _the_subject_words(&last);
    if about.is_empty() {
        return None;
    }
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by(|&a, &b| _priority(&candidates[b]).partial_cmp(&_priority(&candidates[a])).unwrap_or(std::cmp::Ordering::Equal));
    for index in order {
        if index == best || _priority(&candidates[index]) < 0.0 {
            continue;
        }
        let reply = candidates[index].reply.as_str();
        if reply.is_empty() || _says_the_answer_again(reply, &last) {
            continue;
        }
        if lanes::is_a_hedge(reply) || _points_back(reply) {
            continue;
        }
        if _names_a_word_of(reply, &about) {
            return Some(index);
        }
    }
    None
}

/// The memory as a Python-like value, for readings whose contract takes one.
fn memory_value(mem: &Mem) -> Value {
    Value::from_plain(&serde_json::Value::Object(m(mem).to_plain()))
}

// ---------------------------------------------------------------------------------------------
// Readings from the other lanes.
//
// One adapter per Python call the arbiter makes into another file, each called where Python calls
// it. Where the other file's Rust contract cannot carry what Python returns (a stub typed `()`
// because the traced calls only ever returned None), the adapter still makes the call, so a panic
// or a draw on the random generator happens as in Python, and reads the result as None; those are
// marked CONTRACT and listed in the workstream report.
mod lanes {
    use super::{m, py_str, Mem, Set};
    use crate::data::{self, Value};
    use crate::py::re::Re;
    use crate::state::Memory;
    use std::sync::LazyLock;

    /// `fallback.is_a_hedge(reply)`.
    pub fn is_a_hedge(reply: &str) -> bool {
        crate::compose::fallback::is_a_hedge(reply)
    }

    /// `fallback.is_said_it_already(reply)`.
    pub fn is_said_it_already(reply: &str) -> bool {
        crate::compose::fallback::is_said_it_already(reply)
    }

    /// `fallback.is_the_answer_shorter(reply, last)`.
    pub fn is_the_answer_shorter(reply: &str, last: &str) -> bool {
        crate::compose::fallback::is_the_answer_shorter(reply, last)
    }

    /// `fallback.says_it_already(text, memory)`.
    pub fn says_it_already(text: &str, mem: &Mem) -> bool {
        crate::compose::fallback::says_it_already(text, &m(mem))
    }

    /// `fallback.content_words(text)`.
    pub fn content_words(text: &str) -> Vec<String> {
        crate::compose::fallback::content_words(text)
    }

    /// `fallback.a_report_that_says_something(text, memory)`.
    pub fn a_report_that_says_something(text: &str, mem: &Mem) -> bool {
        crate::compose::fallback::a_report_that_says_something(text, &m(mem))
    }

    /// `fallback.FALLBACK_MODULE`: the module whose whole reply is the composer's.
    pub fn fallback_module() -> &'static str {
        static NAME: LazyLock<String> = LazyLock::new(|| data::string("bot.compose.fallback.FALLBACK_MODULE"));
        &NAME
    }

    /// `fallback.MY_WORKINGS`.
    pub fn my_workings() -> &'static Set {
        static KINDS: LazyLock<Set> = LazyLock::new(|| data::set("bot.compose.fallback.MY_WORKINGS"));
        &KINDS
    }

    /// `fallback.about_the_bot(clean)`: the kind of question about the bot, or None.
    /// CONTRACT: `compose::fallback::about_the_bot` returns `()`, so no kind can come back.
    pub fn about_the_bot(clean: &str) -> Option<String> {
        #[allow(clippy::let_unit_value)]
        let _kind = crate::compose::fallback::about_the_bot(clean);
        None
    }

    /// `fallback.venue_hours(clean, None)`.
    /// CONTRACT: the Rust signature takes `&Memory` where Python passes None; an empty memory
    /// stands in for it.
    pub fn venue_hours(clean: &str) -> bool {
        crate::compose::fallback::venue_hours(clean, &Memory::new())
    }

    /// `fallback.venue_hours_lines()`: the composer's honest lines for a venue's opening hours.
    pub fn venue_hours_lines() -> Vec<String> {
        crate::compose::fallback::venue_hours_lines().items().iter().map(py_str).collect()
    }

    /// `fallback.reads_as_a_return(text)`: the thing named after "as i was saying about", or None.
    /// CONTRACT: `compose::fallback::reads_as_a_return` returns `()`.
    pub fn reads_as_a_return(text: &str) -> Option<String> {
        #[allow(clippy::let_unit_value)]
        let _named = crate::compose::fallback::reads_as_a_return(text);
        None
    }

    /// `fallback.asks_about_a_care_answer(care_about, last, memory, own)`.
    /// CONTRACT: Python passes `own` as the set of topic labels the line names; the Rust signature
    /// takes `&str`, so the labels are handed over joined by a space (empty when there are none).
    pub fn asks_about_a_care_answer(care_about: &str, last: &str, mem: &Mem, own: &Set) -> bool {
        let mut labels: Vec<&String> = own.iter().collect();
        labels.sort();
        let own: Vec<&str> = labels.iter().map(|label| label.as_str()).collect();
        crate::compose::fallback::asks_about_a_care_answer(care_about, last, &m(mem), &own.join(" "))
    }

    /// `_there_is_a_chat(memory)` (`bot/memory/__init__.py`).
    pub fn there_is_a_chat(mem: &Mem) -> bool {
        crate::memory::there_is_a_chat(&m(mem))
    }

    /// `_recall.ask(text)` read as a truth value.
    pub fn recall_ask(text: &str) -> bool {
        crate::memory::recall::ask(text, false).map(|found| !found.is_empty()).unwrap_or(false)
    }

    /// `_repeat.asked_back(text, memory)` read as a truth value.
    /// CONTRACT: `memory::repeat::asked_back` returns `()`.
    pub fn repeat_asked_back(text: &str, mem: &Mem) -> bool {
        #[allow(clippy::let_unit_value)]
        let _asked = crate::memory::repeat::asked_back(text, &m(mem));
        false
    }

    /// `_repeat.the_figure_they_gave(text, memory)`.
    /// CONTRACT: `memory::repeat::the_figure_they_gave` returns `()`.
    pub fn repeat_the_figure_they_gave(text: &str, mem: &Mem) -> Value {
        #[allow(clippy::let_unit_value)]
        let _figure = crate::memory::repeat::the_figure_they_gave(text, &m(mem));
        Value::None
    }

    /// `_repeat.MOVE_ASK`.
    pub fn move_ask() -> &'static Re {
        data::re("bot.memory.repeat.MOVE_ASK")
    }

    /// `_repeat.FIGURE_ASKS`: (name, ask, says) triples.
    pub fn figure_asks() -> &'static [Value] {
        data::get("bot.memory.repeat.FIGURE_ASKS").items()
    }

    /// `_repeat.FIGURE_WORDS`.
    pub fn figure_words() -> &'static Set {
        static WORDS: LazyLock<Set> = LazyLock::new(|| data::set("bot.memory.repeat.FIGURE_WORDS"));
        &WORDS
    }

    /// `_repeat.FIGURE_SAYING`.
    pub fn figure_saying() -> &'static Set {
        static WORDS: LazyLock<Set> = LazyLock::new(|| data::set("bot.memory.repeat.FIGURE_SAYING"));
        &WORDS
    }

    /// `_lineback.an_earlier_turn(text, memory)` read as a truth value.
    /// CONTRACT: `memory::lineback::an_earlier_turn` returns `()`.
    pub fn lineback_an_earlier_turn(text: &str, mem: &Mem) -> bool {
        #[allow(clippy::let_unit_value)]
        let _found = crate::memory::lineback::an_earlier_turn(text, &m(mem));
        false
    }

    /// `_lineback.the_kind_they_gave(text, memory)`.
    /// CONTRACT: `memory::lineback::the_kind_they_gave` returns `()`.
    pub fn lineback_the_kind_they_gave(text: &str, mem: &Mem) -> Value {
        #[allow(clippy::let_unit_value)]
        let _line = crate::memory::lineback::the_kind_they_gave(text, &m(mem));
        Value::None
    }

    /// `_cooking.SHORTAGE_WORDS`.
    pub fn cooking_shortage_words() -> &'static Vec<String> {
        static WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.cooking.SHORTAGE_WORDS"));
        &WORDS
    }

    /// `thing, _kept, _opening = _ruledout._rules_out_what_i_never_said(text, memory)`: the thing
    /// when it is a non-empty string, None when it is falsy. Unpacking anything but a triple
    /// raises (a panic, as in Python).
    pub fn ruled_out_thing(text: &str, mem: &Mem) -> Option<String> {
        let found = crate::compose::ruledout::_rules_out_what_i_never_said(text, &m(mem));
        let items = found.items();
        assert!(matches!(found, Value::List(_) | Value::Tuple(_)) && items.len() == 3, "not three values to unpack");
        match &items[0] {
            Value::Str(thing) if !thing.is_empty() => Some(thing.to_string()),
            thing if !thing.truthy() => None,
            // A truthy thing that is not a string: Python's `food + "s"` raises on it.
            _ => panic!("the thing ruled out is not a string"),
        }
    }

    /// `_limit.reply_for(understood, memory)` (the caller has checked there is a reading).
    pub fn limit_reply_for(mem: &Mem) -> Value {
        let memory = m(mem);
        let understood = memory.understood.as_ref().expect("a reading");
        crate::compose::limit::reply_for(understood, &memory)
    }

    /// `_limit.FRAME`.
    pub fn limit_frame() -> &'static Re {
        data::re("bot.compose.limit.FRAME")
    }

    /// `_narrow.SIGNALS.get(name)`.
    pub fn narrow_signal(name: &str) -> Option<&'static Re> {
        data::get("bot.memory.narrow.SIGNALS").get(name).and_then(Value::as_re)
    }

    /// `fit.SOFT`.
    pub fn fit_soft() -> &'static Set {
        static SOFT: LazyLock<Set> = LazyLock::new(|| data::set("bot.compose.fit.SOFT"));
        &SOFT
    }

    /// `fit.gives(reply, want, text)`: True, False, or None when the words cannot tell.
    pub fn fit_gives(reply: &str, want: &str, text: &str) -> Option<bool> {
        crate::compose::fit::gives(reply, want, text)
    }

    /// `_reading.reply_to_a_bare_doubt(text, memory)`.
    /// CONTRACT: `compose::reading::reply_to_a_bare_doubt` returns `()`.
    pub fn reply_to_a_bare_doubt(text: &str, mem: &Mem) -> Value {
        #[allow(clippy::let_unit_value)]
        let _reply = crate::compose::reading::reply_to_a_bare_doubt(text, &m(mem));
        Value::None
    }

    /// `memory_store.REPEAT_WORDS`.
    pub fn store_repeat_words() -> Vec<String> {
        data::try_get("bot.modules.memory_store.REPEAT_WORDS").map(Value::strs).unwrap_or_default()
    }

    /// `meant.MEAN` / `meant.REPEAT`.
    pub fn meant(name: &str) -> &'static Re {
        data::re(&format!("bot.memory.meant.{name}"))
    }

    /// `_already.tells_them_the_step_again(reply, text, memory)`.
    pub fn tells_them_the_step_again(reply: &str, text: &str, mem: &Mem) -> bool {
        crate::memory::already::tells_them_the_step_again(reply, text, &m(mem))
    }

    /// `onscreen.earlier_turn(memory)`.
    pub fn onscreen_earlier_turn(mem: &Mem) -> bool {
        crate::compose::onscreen::earlier_turn(&m(mem))
    }

    /// `onscreen.talks_about_the_conversation(reply)`.
    pub fn talks_about_the_conversation(reply: &str) -> bool {
        crate::compose::onscreen::talks_about_the_conversation(reply)
    }

    /// `leaving.lines()`: every goodbye the composer can say.
    pub fn leaving_lines() -> Vec<String> {
        crate::compose::leaving::lines().items().iter().map(py_str).collect()
    }

    /// `leaving.pool(typed) is leaving.REASON_POOL`.
    /// CONTRACT: `compose::leaving::pool` returns `()` (and takes `&[String]`/`&Memory` where
    /// Python passes None), so the pool it picks cannot be compared: read as not the reason pool.
    pub fn leaving_pool_is_the_reason_pool(typed: &str) -> bool {
        #[allow(clippy::let_unit_value)]
        let _pool = crate::compose::leaving::pool(typed, &[], true, &Memory::new(), true);
        false
    }
}

// ---------------------------------------------------------------------------------------------
// The modules' declarations (README.md, "Interface"), by module name.
//
// Python reads them with `getattr(importlib.import_module(...), NAME, None)`; here each one is a
// table from the candidate's module to the Rust port of the attribute. The tables list exactly the
// respond modules that declare the hook in PyBot (introspected from the running bot); every other
// module declares nothing.
mod hooks {
    use super::{m, Mem};
    use crate::modules as md;

    /// `TAKES_AS_IT_COMES(text, memory)` of a module, False for a module without one.
    pub fn takes_as_it_comes(module: &str, text: &str, mem: &Mem) -> bool {
        match module {
            "the_month_they_have" => md::the_month_they_have::TAKES_AS_IT_COMES(text, &mut mem.borrow_mut()),
            "since_it_began" => md::since_it_began::TAKES_AS_IT_COMES(text, &m(mem)),
            "keeping" => md::keeping::TAKES_AS_IT_COMES(text, &m(mem)),
            "an_age_ahead" => md::an_age_ahead::TAKES_AS_IT_COMES(text, &m(mem)),
            "group_ages" => md::group_ages::TAKES_AS_IT_COMES(text, &m(mem)).unwrap_or(false),
            "household" => md::household::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "said_fact_back" => md::said_fact_back::TAKES_AS_IT_COMES(text, &mut mem.borrow_mut()),
            "live_times" => md::live_times::TAKES_AS_IT_COMES(text, &m(mem)),
            "favourites" => md::favourites::TAKES_AS_IT_COMES(text, &m(mem)),
            "memory_store" => md::memory_store::TAKES_AS_IT_COMES(text, &mut mem.borrow_mut()),
            "attribute_said_again" => md::attribute_said_again::takes_as_it_comes(text, &mut mem.borrow_mut()),
            "thing_i_did_with" => md::thing_i_did_with::takes_as_it_comes(text, &m(mem)),
            "name_taken_back" => md::name_taken_back::takes_as_it_comes(text, &m(mem)),
            "who_to_see" => md::who_to_see::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "pace" => md::pace::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "words" => md::words::TAKES_AS_IT_COMES(text, &m(mem)),
            "my_work" => md::my_work::TAKES_AS_IT_COMES(text, &m(mem)),
            "years_in" => md::years_in::takes_as_it_comes(text, &m(mem)),
            "aboutachild" => md::aboutachild::TAKES_AS_IT_COMES(text, &m(mem)),
            "familyask" => md::familyask::TAKES_AS_IT_COMES(text, &m(mem)),
            "saving_followup" => md::saving_followup::TAKES_AS_IT_COMES(text, &mut mem.borrow_mut()),
            "no_advice" => md::no_advice::TAKES_AS_IT_COMES(text, &m(mem)),
            "a_figure_beside_the_state" => md::a_figure_beside_the_state::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "a_measure_asked_again" => md::a_measure_asked_again::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "a_shock_at_a_bill" => md::a_shock_at_a_bill::TAKES_AS_IT_COMES(text, &m(mem)),
            "a_will_it_work" => md::a_will_it_work::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "figure_asked_again" => md::figure_asked_again::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "taking_asked_again" => md::taking_asked_again::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "a_limit_over_the_answer" => md::a_limit_over_the_answer::TAKES_AS_IT_COMES(text, &m(mem)),
            "already" => crate::memory::already::takes_as_it_comes(text, &m(mem)).truthy(),
            "chat" => md::chat::TAKES_AS_IT_COMES(text, &mut mem.borrow_mut()),
            "answers" => md::answers::TAKES_AS_IT_COMES(text, &mut mem.borrow_mut()),
            "ruled_out" => md::ruled_out::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "nudge" => md::nudge::takes_as_it_comes(text, &m(mem)).unwrap_or(false),
            "the_trouble_named" => md::the_trouble_named::TAKES_AS_IT_COMES(text, &m(mem)),
            "greeting" => md::greeting::takes_as_it_comes(text, &m(mem)),
            _ => false,
        }
    }

    /// `CLAIMS_INSTEAD(text, memory)` of a module, False for a module without one.
    pub fn claims_instead(module: &str, text: &str, mem: &Mem) -> bool {
        match module {
            "an_age_ahead" => md::an_age_ahead::CLAIMS_INSTEAD(text, &m(mem)),
            "a_return_brings_back_the_advice" => md::a_return_brings_back_the_advice::CLAIMS_INSTEAD(text, &m(mem)),
            "dreams" => md::dreams::CLAIMS_INSTEAD(text, &m(mem)).unwrap_or(false),
            "chat" => md::chat::claims_instead(text, &mut mem.borrow_mut()),
            "callback_about" => md::callback_about::CLAIMS_INSTEAD(text, &mut mem.borrow_mut()),
            "answers" => md::answers::claims_instead(text, &mut mem.borrow_mut()).unwrap_or(false),
            _ => false,
        }
    }

    /// `ASKS_ABOUT_THE_CHAT(text, memory)`, or the module's own `_about_this_chat`; False for a
    /// module with neither.
    pub fn asks_about_the_chat(module: &str, text: &str, mem: &Mem) -> bool {
        match module {
            "recap" => md::recap::ASKS_ABOUT_THE_CHAT(text, &m(mem)),
            // memory_store's `_about_this_chat` is `bot.memory.store_replies._about_this_chat`.
            "memory_store" => crate::memory::store_replies::_about_this_chat(text, &mut mem.borrow_mut()),
            _ => false,
        }
    }

    /// Whether a module declares a `DEFERS_TO` test.
    pub fn has_defers_to(module: &str) -> bool {
        matches!(module, "grief" | "loss" | "invitation" | "talkto")
    }

    /// The module's `DEFERS_TO(reply, text)`; every declared test takes the reply and the line.
    pub fn defers_to(module: &str, reply: &str, text: &str) -> bool {
        match module {
            "grief" => md::grief::defers_to(reply, text),
            "loss" => md::loss::defers_to(reply, text),
            "invitation" => md::invitation::DEFERS_TO(reply, text),
            "talkto" => md::talkto::DEFERS_TO(reply, text),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    //! The arbiter's own readings checked against PyBot's on the lines and candidates of a turn
    //! oracle (`scratch/mod3/arbiter_ref.py` writes the reference). Run with
    //! `ARBITER_REF=<reference jsonl> ARBITER_INPUT=<oracle .gz>`.
    use super::*;
    use serde_json::{json, Value as J};
    use std::io::BufRead;

    /// Each oracle record's post-observer memory and its candidates, by record index.
    fn records(path: &str) -> HashMap<usize, (Memory, Vec<Candidate>)> {
        let reader = std::io::BufReader::new(flate2::read::GzDecoder::new(std::fs::File::open(path).unwrap()));
        let mut out = HashMap::new();
        for (idx, line) in reader.lines().enumerate() {
            let Ok(line) = line else { break };
            let Ok(rec) = serde_json::from_str::<J>(&line) else { break };
            let mut mem = rec["mem_in"].as_object().cloned().unwrap_or_default();
            for o in rec["observe"].as_array().unwrap() {
                for (k, v) in o["diff"]["set"].as_object().unwrap() {
                    mem.insert(k.clone(), v.clone());
                }
                for k in o["diff"]["del"].as_array().unwrap() {
                    mem.shift_remove(k.as_str().unwrap());
                }
            }
            let cands: Vec<Candidate> = rec["respond"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["reply"].as_str().map(|s| !text::strip(s).is_empty()).unwrap_or(false))
                .map(|r| {
                    let def = crate::modules::registry().iter().find(|d| d.name == r["module"].as_str().unwrap()).unwrap();
                    Candidate { module: def.name, priority: def.priority, reply: r["reply"].as_str().unwrap().to_string() }
                })
                .collect();
            if cands.len() >= 2 {
                out.insert(idx, (Memory::from_plain(&mem), cands));
            }
        }
        out
    }

    fn sorted(set: impl IntoIterator<Item = String>) -> J {
        let mut v: Vec<String> = set.into_iter().collect();
        v.sort();
        json!(v)
    }

    fn strs(v: &J) -> Vec<String> {
        v.as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect()
    }

    fn set(v: &J) -> Set {
        strs(v).into_iter().collect()
    }

    fn num(x: f64) -> J {
        if x.fract() == 0.0 && x.abs() < 1e15 { json!(x as i64) } else { json!(x) }
    }

    fn cand(c: &Candidate) -> J {
        json!({"module": c.module, "priority": num(c.priority), "reply": c.reply})
    }

    fn opt(i: Option<usize>) -> J {
        i.map(|i| json!(i)).unwrap_or(J::Null)
    }

    fn run(f: &str, a: &[J], mem: &Mem, cands: &[Candidate]) -> J {
        let s = |i: usize| a[i].as_str().unwrap_or("").to_string();
        let idx = |i: usize| a[i].as_u64().unwrap() as usize;
        let list = |i: usize| a[i].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as usize).collect::<Vec<usize>>();
        match f {
            "_the_ask_after_the_facts" => json!(_the_ask_after_the_facts(&s(0))),
            "_a_refusal" => json!(_a_refusal(&s(0))),
            "_a_line_asking_for_plainer_words" => json!(_a_line_asking_for_plainer_words(&s(0))),
            "_a_report_and_no_ask" => json!(_a_report_and_no_ask(&s(0))),
            "_the_words_the_report_names" => json!(_the_words_the_report_names(&s(0))),
            "_names_a_figure" => json!(_names_a_figure(&s(0))),
            "_the_store_reads_an_explain_again_ask" => json!(_the_store_reads_an_explain_again_ask(&s(0))),
            "_subject_words" => sorted(_subject_words(&s(0))),
            "_the_subject_words" => sorted(_the_subject_words(&s(0))),
            "_the_ask_clause" => json!(_the_ask_clause(&s(0))),
            "_content" => json!(_content(&s(0))),
            "_the_numbers_in" => {
                let mut v = _the_numbers_in(&s(0));
                v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                json!(v)
            }
            "_label" => json!(_label(&s(0))),
            "_read_topic" => sorted(_read_topic(&s(0))),
            "_amends_a_detail" => sorted(_amends_a_detail(&s(0), mem)),
            "_the_lines_own_words" => sorted(_the_lines_own_words(&s(0), mem)),
            "_said_before" => sorted(_said_before(mem, &s(1))),
            "_carried_words" => sorted(_carried_words(mem)),
            "_about" => {
                let (own, carried, words, clean) = _about(&s(0), mem);
                json!([sorted(own), sorted(carried), sorted(words), clean])
            }
            "_line_topics" => sorted(_line_topics(mem)),
            "_named_topics" => sorted(_named_topics(mem)),
            "_only_acknowledges" => json!(_only_acknowledges(mem)),
            "_a_presence_check" => json!(_a_presence_check(mem)),
            "_a_bare_yes_no" => json!(_a_bare_yes_no(mem)),
            "_an_answer_behind_the_ask" => json!(_an_answer_behind_the_ask(mem)),
            "_the_figures_they_gave" => sorted(_the_figures_they_gave(mem)),
            "_the_word_the_line_added" => json!(_the_word_the_line_added(&s(0), &s(1))),
            "_the_answer_written_out" => json!(_the_answer_written_out(mem)),
            "_announces_a_repeat" => json!(_announces_a_repeat(&s(0))),
            "_points_back" => json!(_points_back(&s(0))),
            "_the_quoted_repeat" => json!(_the_quoted_repeat(&s(0))),
            "_asks_it_back" => json!(_asks_it_back(&s(0))),
            "_a_condolence" => json!(_a_condolence(&s(0))),
            "_places_the_figure" => json!(_places_the_figure(&s(0))),
            "_explains_the_method" => json!(_explains_the_method(&s(0))),
            "_says_what_they_already_said" => json!(_says_what_they_already_said(&s(0))),
            "_a_clause_was_cut_in_half" => json!(_a_clause_was_cut_in_half(&s(0))),
            "_asks" => json!(_asks(&s(0))),
            "_echoes" => json!(_echoes(&s(0), &s(1), a[2].as_bool().unwrap())),
            "_the_answer_again_with_a_tail" => json!(_the_answer_again_with_a_tail(&s(0), &s(1))),
            "_says_nothing_new" => json!(_says_nothing_new(&s(0), &s(1))),
            "_says_the_answer_again" => json!(_says_the_answer_again(&s(0), &s(1))),
            "_says_the_same_thing_again" => json!(_says_the_same_thing_again(&s(0), &s(1))),
            "_only_says_the_line_back" => json!(_only_says_the_line_back(&s(0), &s(1))),
            "_says_the_subject" => json!(_says_the_subject(&s(0), &s(1))),
            "_about_the_line" => json!(_about_the_line(&s(0), &set(&a[1]), &s(2))),
            "_says_only_the_line" => json!(_says_only_the_line(&s(0), &set(&a[1]))),
            "_is_advice_about" => json!(_is_advice_about(&s(0), &strs(&a[1]))),
            "_how_much_it_names" => json!(_how_much_it_names(&s(0), &strs(&a[1]))),
            "_names_what_they_said" => json!(_names_what_they_said(&s(0), &s(1), mem)),
            "_names_a_word_of" => json!(_names_a_word_of(&s(0), &set(&a[1]))),
            "_names_the_answers_figure" => json!(_names_the_answers_figure(&s(0), mem)),
            "_about_the_news" => json!(_about_the_news(&s(0), mem)),
            "_offers_the_food" => json!(_offers_the_food(&s(0), &set(&a[1]))),
            "_says_back_one_of" => json!(_says_back_one_of(&s(0), &set(&a[1]))),
            "_names_the_thing" => json!(_names_the_thing(&s(0), &s(1), &set(&a[2]))),
            "_names_the_word" => json!(_names_the_word(&s(0), &s(1))),
            "_one_word" => json!(_one_word(&s(0), &s(1))),
            "_the_same_plain_word" => json!(_the_same_plain_word(&s(0), &s(1))),
            "_the_same_word" => json!(_the_same_word(&s(0), &s(1))),
            "_shares_a_stem" => json!(_shares_a_stem(&s(0), &s(1))),
            "_the_forms_of" => sorted(_the_forms_of(&s(0))),
            "_a_card_besides" => json!(_a_card_besides(cands, idx(1))),
            "_a_fresh_answer_besides" => json!(_a_fresh_answer_besides(cands, idx(1), &s(2))),
            "_a_card_names_the_line" => json!(_a_card_names_the_line(cands, idx(1), &s(2), mem)),
            "_every_real_answer_offers" => json!(_every_real_answer_offers(&s(0), cands, idx(2))),
            "_real_answers_besides" => json!(_real_answers_besides(cands, &s(1)).iter().map(|&i| cand(&cands[i])).collect::<Vec<_>>()),
            "_the_candidates_that_answer_the_ask" => {
                _the_candidates_that_answer_the_ask(&s(0), cands, &list(2)).map(|v| json!(v)).unwrap_or(J::Null)
            }
            "_the_candidates_with_a_figure" => json!(_the_candidates_with_a_figure(&s(0), mem, cands, &list(3))),
            "_the_advice_over_a_definition" => opt(_the_advice_over_a_definition(cands, &list(1), &s(2), idx(3))),
            "_the_answer_that_names_what_the_line_added" => opt(_the_answer_that_names_what_the_line_added(cands, &list(1), &s(2), &s(3))),
            "_the_cause_asked_for" => opt(_the_cause_asked_for(cands, &list(1), &s(2), idx(3))),
            "_the_verdict_back" => opt(_the_verdict_back(cands, &s(1), mem)),
            "_the_verdict_turn" => opt(_the_verdict_turn(cands, &set(&a[1]))),
            "_a_lead_line" => opt(_a_lead_line(cands, &strs(&a[1]), &s(2))),
            "_a_shorter_said_again" => opt(_a_shorter_said_again(cands, &s(1))),
            "_a_whole_line_module_spoke" => json!(_a_whole_line_module_spoke(cands)),
            "_a_knowledge_entry_has_it" => json!(_a_knowledge_entry_has_it(cands)),
            "_seed" => json!(_seed(cands)),
            "_the_candidates_that_hand_the_detail_back" => json!(_the_candidates_that_hand_the_detail_back(&set(&a[0]), cands, &list(2))),
            "_match" => json!(_match(&s(0), &set(&a[1]), &set(&a[2]), &set(&a[3]), &s(4))),
            "_declares_topics" => json!(_declares_topics(&s(0))),
            "_declares_instead" => json!(_declares_instead(&s(0))),
            "_declared" => sorted(_declared(&s(0))),
            other => panic!("no dispatch for {other}"),
        }
    }

    #[test]
    #[ignore]
    fn arbiter_readings_against_the_reference() {
        let (Ok(reference), Ok(input)) = (std::env::var("ARBITER_REF"), std::env::var("ARBITER_INPUT")) else { return };
        crate::runtime::quiet_panics();
        let recs = records(&input);
        let (mut ok, mut bad, mut raised) = (0, 0, 0);
        let mut per: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
        let mut shown = 0;
        for line in std::fs::read_to_string(&reference).unwrap().lines() {
            let want: J = serde_json::from_str(line).unwrap();
            let idx = want["rec"].as_u64().unwrap() as usize;
            let Some((memory, cands)) = recs.get(&idx) else { continue };
            let f = want["f"].as_str().unwrap();
            let args = want["args"].as_array().unwrap();
            let mem: Mem = RefCell::new(memory.clone());
            let got = guarded(|| run(f, args, &mem, cands));
            let entry = per.entry(f.to_string()).or_default();
            match got {
                None => {
                    raised += 1;
                    entry.1 += 1;
                    if !want["err"].is_null() {
                        continue;
                    }
                    if shown < 30 {
                        shown += 1;
                        eprintln!("#{idx} {f} {args:?}\n   rust raised {:?}\n   py: {}", crate::runtime::take_last_panic(), want["out"]);
                    }
                }
                Some(got) => {
                    if got == want["out"] && want["err"].is_null() {
                        ok += 1;
                        entry.0 += 1;
                    } else {
                        bad += 1;
                        entry.1 += 1;
                        if shown < 30 {
                            shown += 1;
                            eprintln!("#{idx} {f} {}\n   rust: {got}\n   py:   {} {}", J::Array(args.clone()).to_string().chars().take(300).collect::<String>(), want["out"], want["err"]);
                        }
                    }
                }
            }
        }
        for (f, (good, wrong)) in &per {
            if *wrong > 0 {
                eprintln!("  {f}: {good} same, {wrong} different");
            }
        }
        eprintln!("arbiter readings against the reference: {ok} same, {bad} different, {raised} raised");
    }
}
