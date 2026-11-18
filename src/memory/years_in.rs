//! Port of `bot/memory/years_in.py`: how long they have been doing the job they told me about,
//! said back as a person would.
//!
//! "im a teacher" / "i have been doing it for 10 years" -> "Ten years as a teacher - that's a long
//! stretch." Narrow: the line is only the length statement, there is a job (filed in this chat),
//! and the length is years or months. The same for a person of theirs whose job the chat holds.
#![allow(non_snake_case, clippy::all)]

use crate::data::Value;
use crate::memory::facts::{HE, SHE};
use crate::memory::mem7::{self, inline_re, lists, res, sets};
use crate::py::re;
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;

res!("years_in"; BEEN_AT_IT, BEEN_AT_IT_BARE, NOT_A_STINT, _A_GERUND, THEIR_LENGTH);
sets!("years_in"; THE_JOB_NOUNS);
lists!("years_in"; _NUMBERS, _TENS);
inline_re! {
    NOT_A_LETTER = r"[^a-z]", 0;
    AN_ARTICLE = r"^(?:a|an|the)\s+", re::I;
    IT = r"(?:doing\s+)?(?:it|that|this|the\s+job)", re::I;
}

/// Python: bot/memory/years_in.py:55 `_a_job_word(doing)`
///
/// True when the doing is a job word or the gerund of one.
pub fn _a_job_word(doing: &str) -> bool {
    let low = pt::lower(pt::strip(doing));
    if low.is_empty() {
        return false;
    }
    if THE_JOB_NOUNS.contains(&low) {
        return true;
    }
    match _A_GERUND.match_(&low) {
        Some(m) => {
            let stem = m.name_str("stem");
            !stem.is_empty() && THE_JOB_NOUNS.iter().any(|word| word.starts_with(stem))
        }
        None => false,
    }
}

/// Python: bot/memory/years_in.py:69 `_alike(doing, job)`
///
/// The line's doing and the filed job are the same work: "teaching" and "teacher".
pub fn _alike(doing: &str, job: &str) -> bool {
    let a = NOT_A_LETTER.sub("", &pt::lower(doing));
    let b = NOT_A_LETTER.sub("", &pt::lower(job));
    if a.is_empty() || b.is_empty() {
        return false;
    }
    if b.contains(&a) || a.contains(&b) {
        return true;
    }
    mem7::first_chars(&a, 4) == mem7::first_chars(&b, 4)
}

/// Python: bot/memory/years_in.py:80 `_the_filed_job(memory)`
///
/// The job this chat filed, or "".
pub fn _the_filed_job(memory: &Memory) -> String {
    if let Some(Value::Str(job)) = memory.get("user").filter(|u| u.is_dict()).and_then(|u| u.get("job")) {
        if !pt::strip(job).is_empty() {
            return pt::strip(job).to_string();
        }
    }
    for fact in mem7::iterate_or_empty(memory.get("facts_log")).iter().rev() {
        if fact.is_dict() && mem7::get(fact, "key").as_str() == Some("job") {
            if let Value::Str(value) = mem7::get(fact, "value") {
                if !pt::strip(value).is_empty() {
                    return pt::strip(value).to_string();
                }
            }
        }
    }
    String::new()
}

/// "it", "doing it", "that", "the job": the doing named by a pronoun, not by itself.
fn names_the_doing(doing: &str) -> bool {
    IT.fullmatch(doing).is_none()
}

/// Python: bot/memory/years_in.py:96 `_the_doing_and_the_length(text, memory)`
///
/// The job and the length this line is the length of, or ("", 0, "years").
pub fn _the_doing_and_the_length(text: &str, memory: &Memory) -> (String, i64, String) {
    let nothing = || (String::new(), 0, "years".to_string());
    let said = pt::strip(text);
    if said.is_empty() || said.contains('?') || NOT_A_STINT.is_match(said) {
        return nothing();
    }
    let Some(m) = BEEN_AT_IT.fullmatch(said).or_else(|| BEEN_AT_IT_BARE.fullmatch(said)) else {
        return nothing();
    };
    let doing = AN_ARTICLE.subn("", pt::strip(m.name_str("doing")), 1);
    let filed = _the_filed_job(memory);
    if filed.is_empty() {
        // A length on its own is not a job.
        return nothing();
    }
    if names_the_doing(&doing) && !_alike(&doing, &filed) {
        // The length is of something else.
        return nothing();
    }
    let count = _the_count(m.name_str("count"));
    let unit = pt::lower(m.name_str("unit"));
    let unit = if unit.starts_with("month") || unit.starts_with("mnth") { "months" } else { "years" };
    (filed, count, unit.to_string())
}

/// Python: bot/memory/years_in.py:135 `_the_count(word)`
///
/// The figure a person said, as a number.
pub fn _the_count(word: &str) -> i64 {
    let low = pt::lower(pt::strip(word));
    if pt::is_digit(&low) {
        return mem7::int_str(&low).unwrap_or(0);
    }
    if let Some(at) = _NUMBERS.iter().position(|n| *n == low) {
        return at as i64;
    }
    match low.as_str() {
        "a" | "an" => 1,
        "a few" => 3,
        "a couple of" => 2,
        "several" => 4,
        "many" => 8,
        _ => 0,
    }
}

/// Python: bot/memory/years_in.py:145 `_the_figure_in_words(count)`
///
/// Ten, twelve, fifteen: a length as a person says it.
pub fn _the_figure_in_words(count: i64) -> String {
    let whole = count;
    if 0 <= whole && (whole as usize) < _NUMBERS.len() {
        return pt::capitalize(&_NUMBERS[whole as usize]);
    }
    if whole < 100 {
        let (tens, ones) = (whole.div_euclid(10), whole.rem_euclid(10));
        let said = format!("{}{}", TENS_WORD(tens), if ones != 0 { format!("-{}", _NUMBERS[ones as usize]) } else { String::new() });
        return pt::capitalize(&said);
    }
    whole.to_string()
}

/// Python: bot/memory/years_in.py:156 `TENS_WORD(tens)`
///
/// The tens a figure is said in: 2 -> "twenty".
pub fn TENS_WORD(tens: i64) -> String {
    if 0 <= tens && (tens as usize) < _TENS.len() {
        _TENS[tens as usize].clone()
    } else {
        tens.to_string()
    }
}

/// Python: bot/memory/years_in.py:161 `_with_an(word)`
///
/// "an engineer", "a teacher".
pub fn _with_an(word: &str) -> String {
    let first = pt::lower(mem7::first_chars(word, 1));
    format!("{}{}", if "aeiou".contains(first.as_str()) { "an " } else { "a " }, word)
}

/// `pick([frame.format(...) for frame in frames], memory, seed)`.
fn pick_of(frames: &[String], memory: &Memory, seed: &str) -> String {
    mem7::pick(frames, memory, seed)
}

/// Python: bot/memory/years_in.py:166 `_the_stint(job, count, memory=None, said="", unit="years")`
///
/// The sentence the length deserves, in the words the job is said in.
pub fn _the_stint(job: &str, count: i64, memory: &Memory, said: &str, unit: &str) -> String {
    let seed = format!("{said} {job}");
    let at_it = _as_a(job);
    if unit == "months" && count < 24 {
        // Said in months, answered in months.
        let figure = _the_figure_in_words(count);
        let frames: Vec<String> = if count < 12 {
            vec![
                format!("{figure} months {at_it} - the first year is the hardest stretch of it."),
                format!("{figure} months {at_it}. That is still the part where everyone is finding their feet."),
                format!("{figure} months {at_it} - how are you finding it so far?"),
            ]
        } else {
            vec![
                format!("{figure} months {at_it} - past the first year, then."),
                format!("{figure} months {at_it}. The first year is the one that tests everyone."),
                format!("{figure} months {at_it} - what has got easier?"),
            ]
        };
        return pick_of(&frames, memory, &seed);
    }
    let count = if unit == "months" { count.div_euclid(12) } else { count };
    let figure = _the_figure_in_words(count);
    if (count as f64) < 1.5 {
        let frames = vec![
            format!("A year {at_it} - the first one is the hardest."),
            "A year in already. It gets easier from here, or at least more familiar.".to_string(),
            format!("A year {at_it}. You've got past the bit where everyone feels useless."),
        ];
        return pick_of(&frames, memory, &seed);
    }
    let frames = vec![
        format!("{figure} years {at_it} - that's a long stretch."),
        format!("{figure} years {at_it}. You must know it inside out by now."),
        format!("{figure} years {at_it} is more than most people stick at anything."),
        format!("{figure} years {at_it} - what keeps you at it?"),
    ];
    pick_of(&frames, memory, &seed)
}

/// Python: bot/memory/years_in.py:208 `_as_a(job)`
pub fn _as_a(job: &str) -> String {
    format!("as {}", _with_an(job))
}

/// Python: bot/memory/years_in.py:213 `the_years_in_the_doing(text, memory)`
///
/// The reply for a line that is only how long they have been at the job, or "".
pub fn the_years_in_the_doing(text: &str, memory: &Memory) -> String {
    let (job, count, unit) = _the_doing_and_the_length(text, memory);
    if job.is_empty() || count <= 0 {
        return String::new();
    }
    _the_stint(&job, count, memory, text, &unit)
}

/// Python: bot/memory/years_in.py:235 `_the_person_holding_a_job(memory, who)`
///
/// The most recent person of theirs with a job filed, whose pronoun fits, or ("", "").
pub fn _the_person_holding_a_job(memory: &Memory, who: &str) -> (String, String) {
    let fits: HashSet<String> = match who {
        "she" => SHE.clone(),
        "he" => HE.clone(),
        _ => SHE.union(&HE).cloned().collect(),
    };
    for fact in mem7::iterate_or_empty(memory.get("facts_log")).iter().rev() {
        if !fact.is_dict() || mem7::get(fact, "key").as_str() != Some("other") {
            continue;
        }
        let value = mem7::get(fact, "value");
        if !value.is_dict() {
            continue;
        }
        let relation = pt::lower(pt::strip(&mem7::text_of(value.get("who"))));
        let job = pt::strip(&mem7::text_of(value.get("job"))).to_string();
        if relation.is_empty() || job.is_empty() {
            continue;
        }
        if fits.contains(&relation) {
            return (relation, job);
        }
    }
    (String::new(), String::new())
}

/// Python: bot/memory/years_in.py:254 `_their_stint(relation, job, count, memory=None, said="", unit="years")`
///
/// The sentence their length of service deserves.
pub fn _their_stint(relation: &str, job: &str, count: i64, memory: &Memory, said: &str, unit: &str) -> String {
    let theirs = format!("Your {relation}");
    let seed = format!("{said} {job}");
    if count == 0 {
        let frames = vec![
            format!("{theirs} has been at it a while, then. That is not nothing."),
            format!("{theirs} has put the years in - no wonder it comes up."),
        ];
        return pick_of(&frames, memory, &seed);
    }
    let span = if unit == "months" && count < 24 { "months" } else { "years" };
    let count = if span == "years" && unit == "months" { count.div_euclid(12) } else { count };
    let figure = _the_figure_in_words(count);
    let low = pt::lower(&figure);
    let theirs_lower = pt::lower(&theirs);
    let a_job = _with_an(job);
    let frames: Vec<String> = if span == "months" && count < 12 {
        vec![
            format!("{theirs} has been {a_job} for {low} months - early days, then."),
            format!("{low} months in as {a_job} for {theirs_lower}. The first year is the hard one."),
            format!("{theirs} has been {a_job} for {low} months - how is it going?"),
        ]
    } else {
        vec![
            format!("{theirs} has been {a_job} for {low} {span} - that's a long stretch."),
            format!("{figure} {span} is a long time for {theirs_lower} to stay in one job."),
            format!("{theirs} has been {a_job} for {low} {span}. That is a lot of years behind them."),
        ]
    };
    pick_of(&frames, memory, &seed)
}

/// Python: bot/memory/years_in.py:286 `the_years_in_their_doing(text, memory)`
///
/// The reply for a line that is only how long a person of theirs has been at their job, or "".
pub fn the_years_in_their_doing(text: &str, memory: &Memory) -> String {
    let said = pt::strip(text);
    if said.is_empty() || said.contains('?') {
        return String::new();
    }
    let Some(m) = THEIR_LENGTH.fullmatch(said) else {
        return String::new();
    };
    let unit = pt::lower(m.name("unit").filter(|u| !u.is_empty()).unwrap_or("years"));
    let (relation, job) = _the_person_holding_a_job(memory, &pt::lower(m.name_str("who")));
    if relation.is_empty() {
        return String::new();
    }
    let doing = AN_ARTICLE.subn("", pt::strip(m.name_str("doing")), 1);
    if names_the_doing(&doing) && !_alike(&doing, &job) {
        return String::new();
    }
    let count = _the_count(m.name_str("count"));
    let months = unit.starts_with("month") || unit.starts_with("mnth");
    if count == 0 && pt::lower(m.name_str("count")) != "years" {
        return String::new();
    }
    _their_stint(&relation, &job, count, memory, said, if months { "months" } else { "years" })
}
