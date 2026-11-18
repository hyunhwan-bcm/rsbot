//! Port of `bot/memory/carried_on.py`: a follow-up that says which step of the last answer they
//! have done gets the step after it.
//!
//! "how do i get red wine out of a cream carpet" / "it happened last night and i blotted it" ->
//! "Good - blotting first is right. Now pour on cold water and blot again." The reading is
//! narrow: the turn before was a how-to answer from `answers`, the line is short with no
//! question, the person is in it naming the step they did, it is not a lack, a mishap or a
//! feeling, and something comes after that step.
#![allow(non_snake_case, clippy::all)]

use crate::memory::mem7::{self, inline_re, res, sets};
use crate::py::re;
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

res!("carried_on"; WORD, HOW_TO, A_QUESTION, FORBADE, CLAUSE, THEY_DID, FINISHED_IT, NOT_A_STEP, TENSE, QUALIFIER);
sets!("carried_on"; FRAME, DANGLING, POINTS_BACK, JOINING);
inline_re! {
    JOINER_FIRST = r"^(?:and|then|but|so|also|plus|next)\s+", re::I;
}
static SHORT: LazyLock<usize> = LazyLock::new(|| mem7::int_global("bot.memory.carried_on.SHORT") as usize);
static STEP_WORDS: LazyLock<usize> = LazyLock::new(|| mem7::int_global("bot.memory.carried_on.STEP_WORDS") as usize);

/// Python: bot/memory/carried_on.py:93 `_a_whole_step(said)`
///
/// The step, cut to STEP_WORDS words without cutting a sentence in the middle.
pub fn _a_whole_step(said: &str) -> String {
    let mut words: Vec<String> = pt::split(said);
    if words.len() <= *STEP_WORDS {
        return pt::strip_chars(said, " ,;:").to_string();
    }
    words.truncate(*STEP_WORDS);
    let mut cut = false;
    for at in (3..words.len()).rev() {
        if words[at - 1].ends_with([':', ';', ',']) {
            words.truncate(at);
            words[at - 1] = pt::rstrip_chars(&words[at - 1], ":;,").to_string();
            cut = true;
            break;
        }
    }
    if !cut {
        while words.len() > 2 && {
            let last = words.last().expect("more than two");
            DANGLING.contains(&pt::lower(pt::strip_chars(last, ",;:"))) || last.ends_with([',', ':', ';'])
        } {
            words.pop();
        }
    }
    pt::strip_chars(&words.join(" "), " ,;:").to_string()
}

/// Python: bot/memory/carried_on.py:154 `_stem(word)`
///
/// "blotted" -> "blot", "pouring" -> "pour", "boils" -> "boil".
pub fn _stem(word: &str) -> String {
    let mut word = pt::lower(word);
    if pt::len(&word) > 4 && word.ends_with("ied") {
        word = format!("{}y", &word[..word.len() - 3]);
    }
    let word = TENSE.sub("", &word);
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    if n > 3 && chars[n - 1] == chars[n - 2] && !"lsz".contains(chars[n - 1]) {
        return chars[..n - 1].iter().collect();
    }
    word
}

/// Python: bot/memory/carried_on.py:166 `_words(text)`
pub fn _words(text: &str) -> Vec<String> {
    WORD.findall_str(&pt::lower(text).replace('\u{2019}', "'"))
}

/// Python: bot/memory/carried_on.py:198 `_steps(answer)`
///
/// The answer cut at its clause breaks, which is where one instruction stops and the next starts.
pub fn _steps(answer: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in CLAUSE.split(answer) {
        let part = pt::strip_chars(&part, " .;,-");
        // The joining word the cut was made on is not part of the step.
        let mut words = pt::split_ref(part);
        while words.last().is_some_and(|last| JOINING.contains(&pt::lower(pt::strip_chars(last, " ,;:")))) {
            words.pop();
        }
        let part = pt::strip_chars(&words.join(" "), " ,;:").to_string();
        if !part.is_empty() {
            out.push(part);
        }
    }
    out
}

/// Python: bot/memory/carried_on.py:216 `_doing(word)`
///
/// "blotted" -> "blotting", so the lead-in reads like a person wrote it.
pub fn _doing(word: &str) -> String {
    let low = pt::lower(word);
    let n = pt::len(&low);
    if n < 3 {
        return String::new();
    }
    if low.ends_with("ing") {
        return low;
    }
    let base = if low.ends_with("ied") && n > 4 {
        format!("{}y", &low[..low.len() - 3])
    } else if low.ends_with("ed") && n > 3 {
        low[..low.len() - 2].to_string()
    } else if low.ends_with('s') && n > 3 {
        low[..low.len() - 1].to_string()
    } else {
        low.clone()
    };
    let chars: Vec<char> = base.chars().collect();
    let m = chars.len();
    if m > 3 && chars[m - 1] == chars[m - 2] && !"lsz".contains(chars[m - 1]) {
        return format!("{base}ing");
    }
    if base.ends_with('e') && m > 3 {
        return format!("{}ing", &base[..base.len() - 1]);
    }
    if m > 2 && !"aeiouwxy".contains(chars[m - 1]) && "aeiou".contains(chars[m - 2]) && !"aeiou".contains(chars[m - 3]) {
        return format!("{base}{}ing", chars[m - 1]);
    }
    format!("{base}ing")
}

/// Python: bot/memory/carried_on.py:242 `_the_action(line)`
///
/// The action the person says they have done, as a stem, or None.
pub fn _the_action(line: &str) -> Option<String> {
    let words: Vec<String> = _words(line).into_iter().filter(|w| !FRAME.contains(w)).collect();
    // The last action word in the line is the one they are reporting.
    for word in words.iter().rev() {
        if pt::len(word) < 4 || !TENSE.is_match(word) {
            continue;
        }
        let stem = _stem(word);
        if !stem.is_empty() && !FRAME.contains(&stem) && pt::len(&stem) > 2 {
            return Some(stem);
        }
    }
    None
}

/// The stems of a text's words.
fn stems(text: &str) -> HashSet<String> {
    _words(text).iter().map(|w| _stem(w)).collect()
}

/// Python: bot/memory/carried_on.py:263 `reply_for(text, memory)`
///
/// The next step of the last answer, when the line says which step they are on.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let line = _words(text).join(" ");
    if line.is_empty() || mem7::word_count(&line) > *SHORT {
        return None;
    }
    if A_QUESTION.is_match(text) {
        return None;
    }
    // The answer this line is a report on is the last turn the store holds.
    let last = memory.turns.last()?;
    if last.module.as_deref().unwrap_or("") != "answers" {
        return None;
    }
    if !HOW_TO.is_match(&last.user) {
        return None;
    }
    let steps = _steps(&last.bot);
    if steps.len() < 2 {
        return None;
    }
    // The line has to have the person in it, and not be a lack, a mishap or a feeling.
    if !THEY_DID.is_match(text) || NOT_A_STEP.is_match(text) {
        return None;
    }
    let action = _the_action(text);
    // A noun alone is not a step: only a line with the verb that says it is finished may match
    // the answer on its things.
    let things: HashSet<String> = if FINISHED_IT.is_match(text) {
        _words(text).iter().filter(|w| !FRAME.contains(*w) && pt::len(w) > 3).map(|w| _stem(w)).collect()
    } else {
        HashSet::new()
    };
    if action.is_none() && things.is_empty() {
        return None;
    }
    // The step they are on: the one whose clause uses their action, and is not a warning.
    let mut done = None;
    if let Some(action) = &action {
        done = steps.iter().position(|step| !FORBADE.is_match(step) && stems(step).contains(action));
    }
    if done.is_none() {
        done = steps.iter().position(|step| !FORBADE.is_match(step) && !things.is_disjoint(&stems(step)));
    }
    let done = done?;
    if done >= steps.len() - 1 {
        return None;
    }
    // The step after theirs. A clause that only says how, or forbids a way, belongs to the step
    // before it.
    let said = steps[done + 1..].iter().find(|step| !QUALIFIER.is_match(step))?;
    let said = JOINER_FIRST.subn("", said, 1);
    if mem7::word_count(&said) < 2 {
        return None;
    }
    // A clause that only points at something said already is not a step.
    if _words(&said).iter().all(|w| POINTS_BACK.contains(w)) {
        return None;
    }
    let mut said = _a_whole_step(&said);
    // The step is spliced after a comma, so its first letter comes down unless it names
    // something (a word longer than one letter all in capitals, or "I").
    let first_word = pt::split_ref(&said).first().map(|w| w.to_string()).unwrap_or_default();
    let named_word = mem7::is_upper(&first_word) && (pt::len(&first_word) > 1 || first_word == "I");
    if mem7::is_upper(mem7::first_chars(&said, 1)) && !named_word {
        said = mem7::low1(&said);
    }
    // "Blotting first is right" only when their action is the step's own verb.
    let first = _words(&steps[done]).into_iter().next();
    let mut named = String::new();
    if let (Some(action), Some(first)) = (&action, &first) {
        if _stem(first) == *action {
            named = _doing(action);
        }
    }
    if matches!(named.as_str(), "do" | "doing" | "go" | "going" | "did") {
        named.clear();
    }
    // Three ways to say the same thing, so two of these in a row do not read as a stamped card.
    let shapes = [
        "Good - {doing} first is right. Now {step}.",
        "Right - {doing} is the first bit done. Now {step}.",
        "That was the right first move. Next, {step}.",
    ];
    let which = if named.is_empty() { shapes[2] } else { shapes[(mem7::ord_sum(&said) % 3) as usize] };
    let lead = mem7::fmt(which, &[("doing", &named), ("step", &said)]);
    // A step that starts with "then" reads back as "now then", which is not English.
    Some(lead.replace("Now then ", "Now "))
}
