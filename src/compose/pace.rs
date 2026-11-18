//! Port of `bot/compose/pace.py`: "how slowly should i build up" or "how long should i keep it
//! up" after an answer, answered with the answer's own pace or length, read out of it.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, bundle_vals, format_named, inline_re};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

bundle_res!("bot.compose.pace"; PACE_ASK, KEEP_UP_ASK, _A_LENGTH, _A_RATE_LEAD, _STEP, _PACE, _TEST, _CONTINUES, _NOT_A_TAIL,
    _DANGLES, _A_PORTION);
bundle_sets!("bot.compose.pace"; _ALLOWED, _KEEP_ALLOWED, _STOPPED, _ASK_WORDS);
bundle_strs!("bot.compose.pace"; _REST, _NEXT, _TESTED, _KEEP_LEADS, _KEEP_MARK);
bundle_vals!("bot.compose.pace"; _LEADS);
inline_re!(CLAUSE_BREAK, r"(?<=[.;!?])\s+|;\s*|,\s*|:\s+|\s+-\s+");
inline_re!(A_JOINER, r"^(?:and|but|then|so)\s+", crate::py::re::I);
inline_re!(NOT_A_LETTER, r"[^a-z0-9]");
inline_re!(A_WORD, r"[a-z']+");
inline_re!(A_LONG_WORD, r"[a-z]{4,}");
inline_re!(A_LEAD_IN, r"^(?:to start with|first|first of all|for a start)\b", crate::py::re::I);
inline_re!(A_PREPOSITION, r"^(?:of|in|on|for|at)\s+");
inline_re!(A_VERB_BREAK, r"\b(?:is|are|was|were|turns?|helps?|makes?|works?|should|will|but)\b");

/// Python: bot/compose/pace.py:134 `_clean(text)`
pub fn _clean(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

/// The words of the line other than the ask, plain (shared by the two ask readers).
fn the_rest(said: &str, start: usize, end: usize) -> Vec<String> {
    let rest = format!("{} {}", &said[..start], &said[end..]).replace("'s", " ");
    text::split(&rest).iter().map(|w| NOT_A_LETTER.sub("", w)).filter(|w| !w.is_empty()).collect()
}

/// Python: bot/compose/pace.py:138 `asks_the_pace(text, answer='')`
///
/// True when the line asks how slowly or how fast and names no subject of its own (a word the
/// answer above already named is not one).
pub fn asks_the_pace(text_: &str, answer: &str) -> bool {
    let cleaned = _clean(text_);
    let said = text::strip_chars(&cleaned, " ?!.,;:'\"");
    let Some(found) = PACE_ASK.search(said) else { return false };
    let words = the_rest(said, found.start(), found.end());
    if words.is_empty() {
        return true; // "how fast?" on its own is about the answer above
    }
    let in_the_answer: HashSet<String> = A_WORD.findall_str(&answer.to_lowercase()).into_iter().collect();
    words.iter().all(|w| _ALLOWED.contains(w) || in_the_answer.contains(w))
}

/// Python: bot/compose/pace.py:159 `_last_answer(memory)`
pub fn _last_answer(memory: &Memory) -> String {
    memory.turns.iter().rev().map(|t| text::strip(&t.bot)).find(|s| !s.is_empty()).unwrap_or("").to_string()
}

/// Python: bot/compose/pace.py:169 `_clauses(answer)`
///
/// The answer cut into the short pieces a pace is said in: on sentence ends and commas.
pub fn _clauses(answer: &str) -> Vec<String> {
    let mut out = Vec::new();
    for piece in CLAUSE_BREAK.split(answer) {
        let t = text::squash(text::strip_chars(&piece, " ,.;"));
        let t = A_JOINER.sub("", &t);
        if !t.is_empty() {
            out.push(t);
        }
    }
    out
}

/// Python: bot/compose/pace.py:184 `_trim(text, most=96)`
///
/// A clause short enough to read out, cut at a comma rather than mid-phrase.
pub fn _trim(text_: &str, most: usize) -> String {
    if text::len(text_) <= most {
        return text_.to_string();
    }
    let mut cut = text::slice(text_, 0, Some(most as isize));
    let comma = text::rfind(&cut, ",");
    if comma > (most / 2) as isize {
        cut = text::slice(&cut, 0, Some(comma));
    } else {
        let space = text::rfind(&cut, " ");
        if space > (most / 2) as isize {
            cut = text::slice(&cut, 0, Some(space)); // never a word cut in half
        }
    }
    text::rstrip_chars(&text::squash(&cut), ",;:").to_string()
}

/// Python: bot/compose/pace.py:199 `_parts(answer)`
///
/// The answer's own pace, best first, its own test, and the step that comes after it.
pub fn _parts(answer: &str) -> (Vec<String>, String, String) {
    let clauses = _clauses(answer);
    let (mut stepped, mut timed, mut test, mut after): (Vec<String>, Vec<String>, String, String) =
        (Vec::new(), Vec::new(), String::new(), String::new());
    for (index, clause) in clauses.iter().enumerate() {
        if _STEP.search(clause).is_some() && stepped.len() < 2 {
            stepped.push(_trim(clause, 96));
        } else if _PACE.search(clause).is_some() && timed.len() < 2 {
            timed.push(_trim(clause, 96));
        } else if test.is_empty() {
            if let Some(found) = _TEST.search(clause) {
                test = _trim(text::strip(found.as_str()), 96);
            }
        }
        if (!stepped.is_empty() || !timed.is_empty()) && after.is_empty() && !_a_lead_in(clause) {
            after = _the_next_step(&clauses[index + 1..]);
        }
    }
    let pace: Vec<String> = stepped.into_iter().chain(timed).collect();
    if pace.contains(&after) {
        after = String::new();
    }
    (pace, test, after)
}

/// Python: bot/compose/pace.py:225 `_the_next_step(clauses)`
///
/// The first clause after the pace that names something to do, rather than carrying on.
pub fn _the_next_step<S: AsRef<str>>(clauses: &[S]) -> String {
    for clause in clauses {
        let clause = _trim(clause.as_ref(), 96);
        if clause.is_empty() || _a_lead_in(&clause) || _CONTINUES.match_(&clause).is_some() {
            continue;
        }
        if _PACE.search(&clause).is_some() || _STEP.search(&clause).is_some() {
            continue;
        }
        return clause;
    }
    String::new()
}

/// Python: bot/compose/pace.py:237 `_a_lead_in(clause)`
pub fn _a_lead_in(clause: &str) -> bool {
    A_LEAD_IN.match_(clause).is_some()
}

/// Python: bot/compose/pace.py:242 `_a_pace_reply(answer)`
///
/// True when the answer above is this file's own reply, so the rate has been given once.
pub fn _a_pace_reply(answer: &str) -> bool {
    let opening = text::slice(&text::squash(answer), 0, Some(80)).to_lowercase();
    _LEADS.pairs().iter().any(|(_, leads)| {
        leads.strs().iter().any(|lead| opening.starts_with(&text::slice(&lead.to_lowercase(), 0, Some(24))))
    })
}

/// Python: bot/compose/pace.py:252 `asks_the_keep_up(text, answer='')`
///
/// True when the line asks how long to keep going and names no subject of its own.
pub fn asks_the_keep_up(text_: &str, answer: &str) -> bool {
    let cleaned = _clean(text_);
    let said = text::strip_chars(&cleaned, " ?!.,;:'\"");
    let Some(found) = KEEP_UP_ASK.search(said) else { return false };
    let words = the_rest(said, found.start(), found.end());
    if words.is_empty() {
        return true;
    }
    let in_the_answer: HashSet<String> = A_WORD.findall_str(&answer.to_lowercase()).into_iter().collect();
    words.iter().all(|w| _ALLOWED.contains(w) || _KEEP_ALLOWED.contains(w) || in_the_answer.contains(w))
}

/// Python: bot/compose/pace.py:274 `_the_doing(after)`
///
/// The words the answer puts after its length: "of doing it the same way every single time".
pub fn _the_doing(after: &str) -> String {
    let after = A_PREPOSITION.sub("", text::strip_chars(after, " ,.;"));
    let mut cut = match A_VERB_BREAK.search(&after) {
        Some(m) => after[..m.start()].to_string(),
        None => after.clone(),
    };
    // A piece that stops mid-clause is not a sentence to hand back.
    cut = _DANGLES.sub("", &cut);
    if cut.matches('(').count() > cut.matches(')').count() {
        let at = cut.rfind('(').unwrap();
        cut.truncate(at);
    }
    let cut = _trim(text::strip_chars(&text::squash(&cut), " ,.;-"), 80);
    if text::split_ref(&cut).len() < 2 || _NOT_A_TAIL.match_(&cut).is_some() {
        return String::new();
    }
    cut
}

/// Python: bot/compose/pace.py:303 `_the_length(answer, asked=frozenset())`
///
/// The answer's own length of time, and what it says to do over it. The clause that names what
/// the person asked about wins; otherwise the first length in the answer.
pub fn _the_length(answer: &str, asked: &HashSet<String>) -> (String, String) {
    let mut first = (String::new(), String::new());
    for clause in _clauses(answer) {
        let Some(found) = _A_LENGTH.search(&clause) else { continue };
        if _A_PORTION.search(&clause).is_some() || _A_RATE_LEAD.search(&clause[..found.start()]).is_some() {
            continue;
        }
        let length = text::strip(found.as_str()).to_string();
        let doing = _the_doing(&clause[found.end()..]);
        if !asked.is_empty() && _names_it(&clause, asked) {
            return (length, doing);
        }
        if first.0.is_empty() {
            first = (length, doing);
        }
    }
    first
}

/// Python: bot/compose/pace.py:326 `_names_it(clause, asked)`
///
/// Whether the clause says the thing the person asked about, near enough by its stem.
pub fn _names_it(clause: &str, asked: &HashSet<String>) -> bool {
    _stems(clause).iter().any(|w| asked.contains(w))
}

/// Python: bot/compose/pace.py:332 `_stems(text)`
///
/// The content words of a line, cut back so "feeding", "feeds" and "feed" are one word.
pub fn _stems(text_: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for word in A_LONG_WORD.findall_str(&text_.to_lowercase()) {
        if _STOPPED.contains(&word) {
            continue;
        }
        let mut word = word;
        for ending in ["ing", "ed", "es", "s"] {
            if word.len() > ending.len() + 3 && word.ends_with(ending) {
                word.truncate(word.len() - ending.len());
                break;
            }
        }
        out.insert(word);
    }
    out
}

/// Python: bot/compose/pace.py:355 `_content(answer)`
pub fn _content(answer: &str) -> HashSet<String> {
    A_LONG_WORD.findall_str(&answer.to_lowercase()).into_iter().filter(|w| !_STOPPED.contains(w)).collect()
}

/// Python: bot/compose/pace.py:359 `_about_the_same_thing(one, other)`
///
/// Two bot answers on one subject share the words the subject is made of.
pub fn _about_the_same_thing(one: &str, other: &str) -> bool {
    let theirs = _content(other);
    _content(one).iter().filter(|w| theirs.contains(*w)).count() >= 2
}

/// Python: bot/compose/pace.py:364 `_the_advice_length(memory, answers=3, asked=frozenset())`
///
/// The length of time in the advice in hand - the newest answer, or one before it on the same
/// thing.
pub fn _the_advice_length(memory: &Memory, answers: usize, asked: &HashSet<String>) -> (String, String) {
    let said: Vec<String> = _bot_turns(memory).into_iter().rev().take(answers.max(1)).collect();
    if said.is_empty() {
        return (String::new(), String::new());
    }
    let (length, action) = _the_length(&said[0], asked);
    if !length.is_empty() {
        return (length, action);
    }
    for older in &said[1..] {
        let (length, action) = _the_length(older, asked);
        if !length.is_empty() && _about_the_same_thing(older, &said[0]) {
            return (length, action);
        }
    }
    (String::new(), String::new())
}

/// Python: bot/compose/pace.py:389 `_asked_stems(text)`
///
/// What the person asked about, from the line itself, as stems.
pub fn _asked_stems(text_: &str) -> HashSet<String> {
    _stems(text_).into_iter().filter(|w| !_ASK_WORDS.contains(w)).collect()
}

/// Python: bot/compose/pace.py:401 `_bot_turns(memory)`
///
/// Every answer the bot has given, oldest first, that is not one of this file's own.
pub fn _bot_turns(memory: &Memory) -> Vec<String> {
    memory
        .turns
        .iter()
        .map(|t| text::strip(&t.bot).to_string())
        .filter(|s| !s.is_empty() && !_a_pace_reply(s) && !_a_keep_reply(s))
        .collect()
}

/// Python: bot/compose/pace.py:421 `_a_keep_reply(answer)`
///
/// True when the answer above is this file's own length reply.
pub fn _a_keep_reply(answer: &str) -> bool {
    let opening = text::slice(&text::squash(answer), 0, Some(60)).to_lowercase();
    _KEEP_LEADS.iter().any(|lead| opening.starts_with(&text::strip(lead.split("{length}").next().unwrap_or("")).to_lowercase()))
}

/// Python: bot/compose/pace.py:428 `the_keep_up_line(text, memory)`
///
/// True only when this line is the one the length reply answers.
pub fn the_keep_up_line(text_: &str, memory: &Memory) -> bool {
    if KEEP_UP_ASK.search(&_clean(text_)).is_none() {
        return false;
    }
    let answer = _last_answer(memory);
    if answer.is_empty() || _a_pace_reply(&answer) || _a_keep_reply(&answer) {
        return false;
    }
    if !asks_the_keep_up(text_, &answer) {
        return false;
    }
    !_the_advice_length(memory, 3, &_asked_stems(text_)).0.is_empty()
}

/// Python: bot/compose/pace.py:441 `_keep_up_reply_for(text, memory)`
pub fn _keep_up_reply_for(text_: &str, memory: &Memory) -> Option<String> {
    if !the_keep_up_line(text_, memory) {
        return None;
    }
    let (length, action) = _the_advice_length(memory, 3, &_asked_stems(text_));
    if length.is_empty() {
        return None;
    }
    let first = format!(
        "{} - {}",
        format_named(&pick(&_KEEP_LEADS, memory, text_), &[("length", &_as_clause(&length))]),
        pick(&_KEEP_MARK, memory, text_)
    );
    if !action.is_empty() {
        return Some(format!("{first}, {}.", _as_clause(&action)));
    }
    Some(format!("{first}."))
}

/// Python: bot/compose/pace.py:454 `the_pace_line(text, memory)`
///
/// True only when this line is the one `reply_for` answers: a rate, and a pace above it.
pub fn the_pace_line(text_: &str, memory: &Memory) -> bool {
    if PACE_ASK.search(&_clean(text_)).is_none() {
        return false;
    }
    let answer = _last_answer(memory);
    if _a_pace_reply(&answer) || !asks_the_pace(text_, &answer) {
        return false;
    }
    let (pace, test, after) = _parts(&answer);
    !pace.is_empty() && (!test.is_empty() || pace.len() > 1 || !after.is_empty())
}

/// Python: bot/compose/pace.py:465 `reply_for(text, memory)`
///
/// The pace or the length of the answer just given, or None when this is not that line.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    if KEEP_UP_ASK.search(&_clean(text_)).is_some() {
        return _keep_up_reply_for(text_, memory);
    }
    if PACE_ASK.search(&_clean(text_)).is_none() {
        return None;
    }
    let answer = _last_answer(memory);
    if answer.is_empty() || _a_pace_reply(&answer) || !asks_the_pace(text_, &answer) {
        return None;
    }
    let (pace, test, after) = _parts(&answer);
    if pace.is_empty() {
        return None;
    }
    let said = _clean(text_);
    let word = if said.contains("slowly") {
        "slowly"
    } else if said.contains("fast") {
        "fast"
    } else if said.contains("soon") {
        "soon"
    } else {
        "quickly"
    };
    let lead = pick(&_LEADS.get(word).unwrap_or_else(|| panic!("KeyError: {word:?}")).strs(), memory, text_);
    let first = format!("{lead}: {}.", _as_clause(&pace[0]));
    let second = if !test.is_empty() {
        // The frame is dropped when its own lead-in says the test's first word already.
        let clause = _as_clause(&test);
        let opening = text::split_ref(&clause).first().map(|w| text::strip_chars(&w.to_lowercase(), ",.;:").to_string()).unwrap_or_default();
        let frames: Vec<String> = _TESTED
            .iter()
            .filter(|f| {
                !text::split_ref(f.split("{test}").next().unwrap_or(""))
                    .iter()
                    .any(|w| text::strip_chars(&w.to_lowercase(), ",.;:") == opening)
            })
            .cloned()
            .collect();
        let pool = if frames.is_empty() { _TESTED.clone() } else { frames };
        format_named(&pick(&pool, memory, text_), &[("test", &clause)])
    } else if pace.len() > 1 {
        format_named(&pick(&_REST, memory, text_), &[("rest", &_as_clause(&pace[1]))])
    } else if !after.is_empty() {
        format_named(&pick(&_NEXT, memory, text_), &[("next", &_as_clause(&after))])
    } else {
        return None;
    };
    Some(format!("{first} {second}"))
}

/// Python: bot/compose/pace.py:499 `_as_clause(text)`
///
/// A piece of the answer with its final stop taken off, so it can sit inside a sentence.
pub fn _as_clause(text_: &str) -> String {
    let said = text::strip(text::rstrip_chars(text::strip(text_), ".")).to_string();
    let head = text::slice(&said, 0, Some(2));
    if head != "I " && head != "I'" {
        let mut c = said.chars();
        return match c.next() {
            Some(f) => f.to_lowercase().chain(c).collect(),
            None => String::new(),
        };
    }
    said
}
