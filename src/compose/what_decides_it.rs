//! Port of `bot/compose/what_decides_it.py`: a short qualifier after a knowledge answer ("its not
//! that bad", "only a bit") answered with the test the answer itself gave.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_str, bundle_strs, format_named, inline_re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::HashSet;

bundle_str!("bot.compose.what_decides_it"; ANSWER_MODULE);
bundle_res!("bot.compose.what_decides_it"; _CONDITION, _HANDOFF, _LIMIT, _MORE_THAN, _NEGATION, _AMOUNT, _CORRECTION);
bundle_sets!("bot.compose.what_decides_it"; _QUESTION_START, STOP);
bundle_strs!("bot.compose.what_decides_it"; LEAD, POOL);
inline_re!(A_WORD, r"[a-z']+");
inline_re!(SENTENCE_BREAK, r"(?<=[.!?])\s+");

/// Python: bot/compose/what_decides_it.py:79 `_turns(memory)`
pub fn _turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// Python: bot/compose/what_decides_it.py:83 `_clean(text)`
pub fn _clean(text_: &str) -> String {
    text::squash(&com4::straight(&text_.to_lowercase()))
}

/// Python: bot/compose/what_decides_it.py:87 `_answer_above(memory)`
///
/// The bot's own last reply, when it was a knowledge answer.
pub fn _answer_above(memory: &Memory) -> String {
    let Some(last) = _turns(memory).last() else { return String::new() };
    if last.module.as_deref().unwrap_or("") != ANSWER_MODULE.as_str() {
        return String::new();
    }
    text::strip(&last.bot).to_string()
}

/// Python: bot/compose/what_decides_it.py:98 `_is_a_qualifier(clean)`
///
/// A short statement about themselves, not a question and not a new subject.
pub fn _is_a_qualifier(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') {
        return false;
    }
    let words = text::split_ref(clean);
    if words.len() > 10 || _QUESTION_START.contains(text::strip_chars(words[0], ",.")) {
        return false;
    }
    if _CORRECTION.search(clean).is_some() {
        return false;
    }
    _NEGATION.search(clean).is_some() || _AMOUNT.search(clean).is_some() || clean.contains("kind") || clean.contains("sort of")
}

/// Python: bot/compose/what_decides_it.py:115 `_content(text)`
pub fn _content(text_: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for word in A_WORD.findall_str(&_clean(text_)) {
        let word = text::strip_chars(&word, "'").to_string();
        if text::len(&word) < 4 || STOP.contains(&word) {
            continue;
        }
        out.insert(if word.ends_with('s') && text::len(&word) > 4 { word[..word.len() - 1].to_string() } else { word });
    }
    out
}

/// Python: bot/compose/what_decides_it.py:125 `_sentences(answer)`
pub fn _sentences(answer: &str) -> Vec<String> {
    SENTENCE_BREAK.split(text::strip(answer)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/compose/what_decides_it.py:130 `_is_a_test(sentence)`
pub fn _is_a_test(sentence: &str) -> bool {
    let lowered = format!(" {} ", sentence.to_lowercase());
    if _CONDITION.search(&lowered).is_none() || text::split_ref(sentence).len() < 5 {
        return false;
    }
    let limit = _LIMIT.search(&lowered).is_some();
    _HANDOFF.search(&lowered).is_some() || limit || (_MORE_THAN.search(&lowered).is_some() && limit)
}

/// Python: bot/compose/what_decides_it.py:138 `_the_test(answer)`
///
/// The answer's own deciding sentence: the last one that carries a condition.
pub fn _the_test(answer: &str) -> String {
    let Some(found) = _sentences(answer).into_iter().filter(|s| _is_a_test(s)).last() else { return String::new() };
    let mut words: Vec<String> = text::split(text::rstrip_chars(&found, " .;:,"));
    if words.len() > 34 {
        // cut at the last comma that still leaves a clause worth quoting, never mid-phrase
        let cut = words[..34].join(" ");
        let comma = text::rfind(&cut, ",");
        let kept = if comma > 60 { text::slice(&cut, 0, Some(comma)) } else { cut };
        words = text::split(&kept);
    }
    let mut found = text::rstrip_chars(&words.join(" "), " ,;:").to_string();
    // "See a GP" reads as "see a GP" mid-line; an acronym or a name keeps its case.
    let first = found.split(' ').next().unwrap_or("").to_string();
    let head: String = first.chars().take(1).collect();
    let rest: String = first.chars().skip(1).collect();
    if text::is_upper(&head) && text::is_lower(&rest) {
        let mut c = found.chars();
        let f = c.next().unwrap();
        found = f.to_lowercase().chain(c).collect();
    }
    found
}

/// Python: bot/compose/what_decides_it.py:163 `_in_contact(clean, test)`
///
/// True when the person's own words bear on the test: then the reply is not ours to give.
pub fn _in_contact(clean: &str, test: &str) -> bool {
    let theirs = _content(test);
    _content(clean).iter().any(|w| theirs.contains(w))
}

/// Python: bot/compose/what_decides_it.py:168 `answer_for(text, memory=None)`
///
/// One line naming the test the answer above gave, or None when the line is not a qualifier.
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    let clean = _clean(text_);
    let answer = _answer_above(memory);
    if answer.is_empty() || !_is_a_qualifier(&clean) {
        return None;
    }
    let test = _the_test(&answer);
    if test.is_empty() || _in_contact(&clean, &test) {
        return None;
    }
    let lead = &LEAD[(com4::ord_sum(&clean) % LEAD.len() as i128) as usize];
    let line = &POOL[(com4::ord_sum(&format!("{clean}{test}")) % POOL.len() as i128) as usize];
    Some(format_named(line, &[("lead", lead), ("clause", &test)]))
}
