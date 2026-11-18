//! Port of `bot/memory/condition.py`: the branch of the last answer that a short line of the
//! person's own words chooses.
//!
//! "Dogs, by a whisker ... A cat is the kinder choice if you're out all day" / "i work all day":
//! the conditions are read out of the answer, and the consequence of the one the line matches
//! (on a pair of words, a place that agrees, and a number that passes the case's test) is handed
//! back. A line about somebody else takes no case.
#![allow(non_snake_case, clippy::all)]

use crate::memory::mem7::{self, inline_re, lists, res, sets, values};
use crate::py::re;
use crate::py::text as pt;
use std::collections::HashSet;

res!("condition"; SENTENCE, OPENS, CONDITION_WORD, BREAK, RUNS_ON, MY_PERSON, A_PLAN);
sets!("condition"; EMPTY, PLACE, PERSON, A_PERSON_IN_IT, UNITS);
values!("condition"; MOST_LETTERS, MOST_CASE, NUMBERS, MOST_WORDS_BEFORE);
lists!("condition"; BELOW, ABOVE);
inline_re! {
    LETTERS = r"[a-z]+", 0;
    PIECES = "(?:,\\s+|\\s+-\\s+|\\s+\u{2014}\\s+)", 0;
    JUNCTION = r"(?:[\s,;:]+)(?:and|but|or|so|then|with|plus)$", re::I;
    AN_ARTICLE = r"\s+(?=(?:a|an|the)\s)", 0;
    A_COLON = r"[:;]", 0;
    AFTER_LEAD = "^\\s*(?:,|;|:|\u{2014}|\\s-\\s|\\band\\b|\\bthen\\b)\\s*", re::I;
    MEASURE_WORDS = r"[a-z0-9.]+", 0;
    HALVES = r"\bor\b|\bwhen\b", re::I;
}

const EDGES: &str = " ,;:-\u{2014}";

fn int_of(v: &crate::data::Value) -> i64 {
    v.as_int().expect("an int global")
}

/// Python: bot/memory/condition.py:69 `_places(words)`
pub fn _places(words: &[String]) -> HashSet<String> {
    words.iter().filter(|w| PLACE.contains(*w)).cloned().collect()
}

/// Python: bot/memory/condition.py:73 `_words(text)`
///
/// The words of a line, with the apostrophe taken out so "you're" meets "youre".
pub fn _words(text: &str) -> Vec<String> {
    LETTERS.findall_str(&pt::lower(text).replace('\u{2019}', "'"))
}

/// Python: bot/memory/condition.py:78 `_pairs(words)`
///
/// Every two words that sit next to each other and are not both empty ones.
pub fn _pairs(words: &[String]) -> HashSet<String> {
    words.windows(2).filter(|w| !EMPTY.contains(&w[0]) || !EMPTY.contains(&w[1])).map(|w| format!("{} {}", w[0], w[1])).collect()
}

/// `word.rstrip("'s")`.
fn unpossessed(word: &str) -> &str {
    pt::rstrip_chars(word, "'s")
}

/// Python: bot/memory/condition.py:114 `about_someone_else(text)`
///
/// Whether the line is about one of the people in it rather than the person typing.
pub fn about_someone_else(text: &str) -> bool {
    let low = pt::lower(pt::strip(text)).replace('\u{2019}', "'");
    MY_PERSON.findall_str(&low).iter().any(|word| PERSON.contains(unpossessed(word)))
}

/// Python: bot/memory/condition.py:126 `_sentences(said)`
pub fn _sentences(said: &str) -> Vec<String> {
    SENTENCE.finditer(said).iter().map(|m| pt::strip(m.as_str()).to_string()).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/memory/condition.py:130 `_one_move(consequence)`
///
/// The first move of a consequence: a long one is cut at a clause boundary, not dropped.
pub fn _one_move(consequence: &str) -> String {
    let text = pt::strip_chars(consequence, " ,;:-\u{2014}.");
    let most = int_of(&MOST_LETTERS) as usize;
    let mut kept = text.to_string();
    if pt::len(text) > most {
        kept = String::new();
        for piece in PIECES.split(text) {
            let trial = if kept.is_empty() { piece.clone() } else { format!("{kept}, {piece}") };
            if pt::len(&trial) > most && !kept.is_empty() {
                break;
            }
            kept = trial;
        }
    }
    let kept = pt::strip_chars(&kept, EDGES).to_string();
    // A clause never ends on the word that was going to join it to the next one.
    let kept = JUNCTION.sub("", &kept);
    pt::strip_chars(&kept, EDGES).to_string()
}

/// Python: bot/memory/condition.py:158 `conditions(said)`
///
/// [(the case, what the answer says happens in that case)] for every "if" in the answer.
pub fn conditions(said: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for sentence in _sentences(said) {
        for opening in CONDITION_WORD.finditer(&sentence) {
            let before = pt::strip_chars(&sentence[..opening.start()], EDGES);
            // A condition nested inside another one's clause is part of the first.
            if CONDITION_WORD.is_match(before) {
                continue;
            }
            let rest = pt::strip(&sentence[opening.end()..]);
            let rest = OPENS.sub("", rest);
            let rest = pt::strip(&rest);
            let mut cut = BREAK.search(rest).map(|m| m.start());
            if cut.is_none() && !before.is_empty() && RUNS_ON.is_match(rest) {
                continue;
            }
            if cut.is_none() && before.is_empty() {
                // A sentence that opens with the case and never pauses: an article starts the answer.
                cut = AN_ARTICLE.search(rest).map(|m| m.start());
            }
            // Nothing before a colon is the answer to the "if" after it.
            if !before.is_empty() && A_COLON.is_match(before) {
                continue;
            }
            let case = pt::strip_chars(cut.map_or(rest, |c| &rest[..c]), " ,;:-\u{2014}.").to_string();
            let after = cut.map_or(String::new(), |c| AFTER_LEAD.subn("", &rest[c..], 1));
            // The consequence is on the side of the "if" that still says something.
            let consequence = if _words(before).len() >= 2 { before.to_string() } else { pt::strip_chars(&after, " ,;:-\u{2014}.").to_string() };
            if case.is_empty() || consequence.is_empty() {
                continue;
            }
            if pt::len(&case) as i64 > int_of(&MOST_CASE) {
                continue;
            }
            let consequence = _one_move(&consequence);
            if consequence.is_empty() {
                continue;
            }
            found.push((case, consequence));
        }
    }
    found
}

/// Python: bot/memory/condition.py:242 `_number(word)`
pub fn _number(word: &str) -> Option<f64> {
    let word = pt::strip_chars(word, ".,");
    if pt::is_digit(word) {
        return mem7::py_float(word);
    }
    NUMBERS.get(word).and_then(|v| v.as_int()).map(|v| v as f64)
}

/// `unit.rstrip("s")`.
fn unit_of(word: &str) -> &str {
    pt::rstrip_chars(word, "s")
}

/// Python: bot/memory/condition.py:249 `_measures(text)`
///
/// {unit: number} for every "<number> <unit>" in a line, digits or words alike (first wins).
pub fn _measures(text: &str) -> Vec<(String, f64)> {
    let mut found: Vec<(String, f64)> = Vec::new();
    let words = MEASURE_WORDS.findall_str(&pt::lower(text));
    for index in 1..words.len() {
        let unit = unit_of(&words[index]);
        if !UNITS.contains(unit) {
            continue;
        }
        let mut value = _number(&words[index - 1]);
        if value.is_none() && index > 1 && matches!(words[index - 1].as_str(), "a" | "an") {
            value = Some(1.0);
        }
        if let Some(value) = value {
            if !found.iter().any(|(u, _)| u == unit) {
                found.push((unit.to_string(), value));
            }
        }
    }
    found
}

/// Python: bot/memory/condition.py:265 `_tests(case)`
///
/// [(unit, number, "below"|"above")] for every number a case tests.
pub fn _tests(case: &str) -> Vec<(String, f64, &'static str)> {
    let mut found = Vec::new();
    let words = MEASURE_WORDS.findall_str(&pt::lower(case));
    let most = int_of(&MOST_WORDS_BEFORE) as usize;
    for index in 0..words.len() {
        let unit = unit_of(&words[index]);
        let value = if index > 0 { _number(&words[index - 1]) } else { None };
        let (true, Some(value)) = (UNITS.contains(unit), value) else {
            continue;
        };
        let from = (index - 1).saturating_sub(most);
        let around = words[from..index - 1].join(" ");
        let side = if BELOW.iter().any(|lead| around.contains(lead.as_str())) {
            "below"
        } else if ABOVE.iter().any(|lead| around.contains(lead.as_str())) {
            "above"
        } else {
            ""
        };
        if !side.is_empty() {
            found.push((unit.to_string(), value, side));
        }
    }
    found
}

/// Python: bot/memory/condition.py:281 `_meets(case, text)`
///
/// Whether the numbers the line gives pass the numbers the case tests: Some(true) when every
/// test is passed, Some(false) when one is failed, None when the case tests nothing it gives.
pub fn _meets(case: &str, text: &str) -> Option<bool> {
    let mine = _measures(text);
    let mut judged = false;
    for (unit, value, side) in _tests(case) {
        let Some((_, theirs)) = mine.iter().find(|(u, _)| *u == unit) else {
            continue;
        };
        judged = true;
        if side == "below" && !(*theirs < value) {
            return Some(false);
        }
        if side == "above" && !(*theirs > value) {
            return Some(false);
        }
    }
    if judged {
        Some(true)
    } else {
        None
    }
}

/// Python: bot/memory/condition.py:303 `chosen(text, said)`
///
/// The consequence the last answer gave for the case this line states, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the consequence or None.
pub fn chosen(text: &str, said: &str) -> Option<String> {
    _candidates(text, said).into_iter().find(|(case, _)| _meets(case, text) != Some(false)).map(|(_, consequence)| consequence)
}

/// Python: bot/memory/condition.py:319 `refused(text, said)`
///
/// Whether the last answer put a test on the number this line gives, and their number fails.
pub fn refused(text: &str, said: &str) -> bool {
    _candidates(text, said).iter().any(|(case, _)| _meets(case, text) == Some(false))
}

/// Python: bot/memory/condition.py:330 `_carried(case_words, words)`
///
/// Whether the line carries what the case turns on, when no pair of words can be shared.
pub fn _carried(case_words: &[String], words: &[String]) -> bool {
    let want: HashSet<&String> = case_words.iter().filter(|w| !EMPTY.contains(*w)).collect();
    if want.is_empty() {
        return false;
    }
    let met = want.iter().filter(|w| words.contains(w)).count();
    if met == 0 {
        return false;
    }
    met == want.len() || (want.len() >= 3 && met * 2 >= want.len())
}

/// Python: bot/memory/condition.py:349 `_about_a_person(case_words)`
///
/// Whether the case itself names somebody, so a line about theirs can be that case.
pub fn _about_a_person(case_words: &[String]) -> bool {
    case_words.iter().any(|w| A_PERSON_IN_IT.contains(unpossessed(w)))
}

/// Python: bot/memory/condition.py:354 `_candidates(text, said)`
///
/// The (case, consequence) pairs of the answer this line can be matched to in words.
pub fn _candidates(text: &str, said: &str) -> Vec<(String, String)> {
    if A_PLAN.is_match(text) {
        return Vec::new();
    }
    let words = _words(text);
    if words.is_empty() || words.len() > 8 {
        return Vec::new();
    }
    let mine = _pairs(&words);
    let mut found = Vec::new();
    for (case, consequence) in conditions(said) {
        let case_words = _words(&case);
        let halves: Vec<Vec<String>> = HALVES.split(&case).iter().filter(|one| !pt::strip(one).is_empty()).map(|one| _words(one)).collect();
        let ours = _places(&words);
        let hit = halves.iter().any(|half| {
            if mine.is_disjoint(&_pairs(half)) && !_carried(half, &words) {
                return false;
            }
            let theirs = _places(half);
            // The case says one place and their line says another: not their case.
            !(!theirs.is_empty() && !ours.is_empty() && theirs.is_disjoint(&ours))
        });
        if !hit {
            continue;
        }
        if about_someone_else(text) && !_about_a_person(&case_words) {
            continue; // her afternoon is not their case
        }
        found.push((case, consequence));
    }
    found
}
