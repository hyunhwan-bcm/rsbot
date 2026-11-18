//! Port of `bot/compose/fallback.py`: the composer's safety net, a fit reply when no module
//! answered the line, and the many readings of a line and its thread the other composers share.
//!
//! Every module-level table, pool and compiled regex comes from the data bundle under its
//! Python name in this module's namespace (`bot.compose.fallback.X`), which is the binding the
//! running module sees, including the names re-imported at the end of the Python file.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, clippy::all)]

use crate::compose::fallback_chain::{_choose, _mood, _words};
use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understanding::rx;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.fallback.";

/// Compiled regexes of this module's namespace, by Python name.
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// String sets (frozensets, sets, dict keys) of this module's namespace, for membership tests.
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// String sequences (reply pools, ordered word lists) of this module's namespace, in order.
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// Plain strings of this module's namespace.
macro_rules! strings {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<String> =
            LazyLock::new(|| data::string(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// Any other value of this module's namespace (dicts, tuples of tuples).
macro_rules! values {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Value> =
            LazyLock::new(|| data::get(concat!("bot.compose.fallback.", stringify!($name))));)*
    };
}

/// `re.findall(r"[a-z']+", s)`: the runs of lowercase ASCII letters and apostrophes.
fn az_words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_ascii_lowercase() || c == '\'' {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// `re.findall(r"[a-z0-9']+", s)`.
fn az09_words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '\'' {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// `set(re.findall(r"[a-z']+", s))`.
fn az_set(s: &str) -> HashSet<String> {
    az_words(s).into_iter().collect()
}

/// Python's `seq[-n:]` for a count `n` (`seq[-0:]` is the whole sequence, as in Python).
fn last_n<T>(seq: &[T], n: i64) -> &[T] {
    let len = seq.len() as i64;
    let start = -n;
    let start = if start < 0 { (len + start).max(0) } else { start.min(len) };
    &seq[start as usize..]
}

/// A look-back count passed as a Python value (an int, or None for the default).
fn look_back_of(v: &Value, default: i64) -> i64 {
    v.as_int().unwrap_or(default)
}

/// `str(x or "").lower()`.
fn lower(s: &str) -> String {
    text::lower(s)
}

/// `any(word in s for word in set)` written as `set(words) & other`.
fn meets(words: &HashSet<String>, other: &HashSet<String>) -> bool {
    if words.len() <= other.len() { words.iter().any(|w| other.contains(w)) } else { other.iter().any(|w| words.contains(w)) }
}

/// `random.Random(seed).choice(...)`-free pick: `_choose` with no seed text.
fn choose(options: &[String], memory: &Memory, clean: &str) -> Option<String> {
    let picked = _choose(options, memory, clean, "");
    if picked.is_empty() { None } else { Some(picked) }
}

/// `_choose(options, memory, clean, seed_text)` with a seed text.
fn choose_seeded(options: &[String], memory: &Memory, clean: &str, seed_text: &str) -> Option<String> {
    let picked = _choose(options, memory, clean, seed_text);
    if picked.is_empty() { None } else { Some(picked) }
}

/// Python's `str.format` with named fields: `{name}` replaced, `{{`/`}}` unescaped.
fn format_named(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(i) = rest.find(['{', '}']) {
        out.push_str(&rest[..i]);
        let c = rest.as_bytes()[i];
        if c == b'{' && rest[i + 1..].starts_with('{') {
            out.push('{');
            rest = &rest[i + 2..];
            continue;
        }
        if c == b'}' {
            out.push('}');
            rest = if rest[i + 1..].starts_with('}') { &rest[i + 2..] } else { &rest[i + 1..] };
            continue;
        }
        let end = rest[i..].find('}').map(|j| i + j).unwrap_or(rest.len());
        let key = &rest[i + 1..end];
        match fields.iter().find(|(k, _)| *k == key) {
            Some((_, v)) => out.push_str(v),
            None => panic!("KeyError: {key}"),
        }
        rest = if end < rest.len() { &rest[end + 1..] } else { "" };
    }
    out.push_str(rest);
    out
}

/// Python: bot/compose/fallback.py:15975 `_advice_thread_is_about_the_body(memory)`
pub fn _advice_thread_is_about_the_body(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_advice_thread_is_about_the_body")
}

/// Python: bot/compose/fallback.py:15990 `_a_step_word_in(words, steps)` (never called on the trace corpus)
pub fn _a_step_word_in(words: &[String], steps: &Value) -> bool {
    crate::unported!("bot.compose.fallback:_a_step_word_in")
}

/// Python: bot/compose/fallback.py:15998 `_a_report_on_the_advice(clean, words, about_the_body=True, the_lot_ok=False, steps=NO_STEP)`
pub fn _a_report_on_the_advice(clean: &str, words: &[String], about_the_body: bool, the_lot_ok: bool, steps: &HashSet<String>) -> Option<Vec<String>> {
    crate::unported!("bot.compose.fallback:_a_report_on_the_advice")
}

/// Python: bot/compose/fallback.py:16055 `_the_outcome_of_the_advice(memory, clean, words)`
pub fn _the_outcome_of_the_advice(memory: &Memory, clean: &str, words: &[String]) {
    crate::unported!("bot.compose.fallback:_the_outcome_of_the_advice")
}

/// Python: bot/compose/fallback.py:16092 `the_advice(memory)`
pub fn the_advice(memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:the_advice")
}

/// Python: bot/compose/fallback.py:16102 `reads_as_a_report(text, memory)`
pub fn reads_as_a_report(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:reads_as_a_report")
}

/// Python: bot/compose/fallback.py:16115 `the_report_reply(text, memory)` (never called on the trace corpus)
pub fn the_report_reply(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:the_report_reply")
}

/// Python: bot/compose/fallback.py:16132 `a_report_that_says_something(text, memory)` (never called on the trace corpus)
pub fn a_report_that_says_something(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:a_report_that_says_something")
}

/// Python: bot/compose/fallback.py:16146 `_after_advice(memory, clean, words)`
pub fn _after_advice(memory: &Memory, clean: &str, words: &[String]) {
    crate::unported!("bot.compose.fallback:_after_advice")
}

/// Python: bot/compose/fallback.py:16161 `_the_answer_above_already_says_what_to_do(turns, memory)` (never called on the trace corpus)
pub fn _the_answer_above_already_says_what_to_do(turns: &[Turn], memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_answer_above_already_says_what_to_do")
}

/// Python: bot/compose/fallback.py:16176 `_a_thread_about_how_somebody_treats_them(turns)` (never called on the trace corpus)
pub fn _a_thread_about_how_somebody_treats_them(turns: &[Turn]) -> bool {
    crate::unported!("bot.compose.fallback:_a_thread_about_how_somebody_treats_them")
}

/// Python: bot/compose/fallback.py:16201 `_a_refusal_to_answer(clean, memory, words=None)`
pub fn _a_refusal_to_answer(clean: &str, memory: &Memory, words: &[String]) {
    crate::unported!("bot.compose.fallback:_a_refusal_to_answer")
}

/// Python: bot/compose/fallback.py:16264 `reads_as_a_refusal(text, memory)` (never called on the trace corpus)
pub fn reads_as_a_refusal(text: &str, memory: &Memory) -> Option<bool> {
    crate::unported!("bot.compose.fallback:reads_as_a_refusal")
}

/// Python: bot/compose/fallback.py:16273 `_the_next_move_after_the_advice(memory, clean, words)`
pub fn _the_next_move_after_the_advice(memory: &Memory, clean: &str, words: &[String]) {
    crate::unported!("bot.compose.fallback:_the_next_move_after_the_advice")
}

/// Python: bot/compose/fallback.py:16298 `_a_care_thread(memory)` (never called on the trace corpus)
pub fn _a_care_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_care_thread")
}

/// Python: bot/compose/fallback.py:16306 `in_a_safety_thread(memory)`
pub fn in_a_safety_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:in_a_safety_thread")
}

/// Python: bot/compose/fallback.py:16316 `_thread_answer(memory)`
pub fn _thread_answer(memory: &Memory) -> String {
    crate::unported!("bot.compose.fallback:_thread_answer")
}

/// Python: bot/compose/fallback.py:16362 `asks_about_the_last_answer(line, last, own=())` (never called on the trace corpus)
pub fn asks_about_the_last_answer(line: &str, last: &str, own: &str) -> bool {
    crate::unported!("bot.compose.fallback:asks_about_the_last_answer")
}

/// Python: bot/compose/fallback.py:16420 `asks_about_a_care_answer(line, last, memory, own=())` (never called on the trace corpus)
pub fn asks_about_a_care_answer(line: &str, last: &str, memory: &Memory, own: &str) -> bool {
    crate::unported!("bot.compose.fallback:asks_about_a_care_answer")
}

/// Python: bot/compose/fallback.py:16442 `how_much_of_the_answer_kept(reply, memory)` (never called on the trace corpus)
pub fn how_much_of_the_answer_kept(reply: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.compose.fallback:how_much_of_the_answer_kept")
}

/// Python: bot/compose/fallback.py:16471 `_words_kept(reply, answer)` (never called on the trace corpus)
pub fn _words_kept(reply: &str, answer: &str) -> i64 {
    crate::unported!("bot.compose.fallback:_words_kept")
}

/// Python: bot/compose/fallback.py:16492 `careful(text, memory)`
pub fn careful(text: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.compose.fallback:careful")
}

/// Python: bot/compose/fallback.py:16505 `_in_a_loss_thread(memory)`
pub fn _in_a_loss_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_in_a_loss_thread")
}

/// Python: bot/compose/fallback.py:16555 `_a_loss_in(text)` (never called on the trace corpus)
pub fn _a_loss_in(text: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_loss_in")
}

/// Python: bot/compose/fallback.py:16585 `_dead_is_the_only_death_word(said)` (never called on the trace corpus)
pub fn _dead_is_the_only_death_word(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:_dead_is_the_only_death_word")
}

/// Python: bot/compose/fallback.py:16593 `_we_condoled(line)` (never called on the trace corpus)
pub fn _we_condoled(line: &str) -> bool {
    crate::unported!("bot.compose.fallback:_we_condoled")
}

/// Python: bot/compose/fallback.py:16605 `_a_thing_stopped(clean)` (never called on the trace corpus)
pub fn _a_thing_stopped(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_thing_stopped")
}

/// Python: bot/compose/fallback.py:16610 `_last_turn(memory)` (never called on the trace corpus)
pub fn _last_turn(memory: &Memory) -> Value {
    crate::unported!("bot.compose.fallback:_last_turn")
}

/// Python: bot/compose/fallback.py:16616 `_the_last_line_was_bad_news(memory)` (never called on the trace corpus)
pub fn _the_last_line_was_bad_news(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_last_line_was_bad_news")
}

/// Python: bot/compose/fallback.py:16626 `_the_last_line_was_a_fault(memory)` (never called on the trace corpus)
pub fn _the_last_line_was_a_fault(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_last_line_was_a_fault")
}

/// Python: bot/compose/fallback.py:16641 `_a_loss_recently(memory)` (never called on the trace corpus)
pub fn _a_loss_recently(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_a_loss_recently")
}

/// Python: bot/compose/fallback.py:16671 `_the_last_line_was_heavy(memory)` (never called on the trace corpus)
pub fn _the_last_line_was_heavy(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_the_last_line_was_heavy")
}

/// Python: bot/compose/fallback.py:16691 `_in_sudden_loss_thread(memory)`
pub fn _in_sudden_loss_thread(memory: &Memory) -> bool {
    crate::unported!("bot.compose.fallback:_in_sudden_loss_thread")
}

/// Python: bot/compose/fallback.py:16701 `_reads_as_illness(clean)` (never called on the trace corpus)
pub fn _reads_as_illness(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_reads_as_illness")
}

/// Python: bot/compose/fallback.py:16734 `_in_an_aim(clean, word)`
pub fn _in_an_aim(clean: &str, word: &str) -> bool {
    crate::unported!("bot.compose.fallback:_in_an_aim")
}

/// Python: bot/compose/fallback.py:16753 `_a_mood_word(words, word)` (never called on the trace corpus)
pub fn _a_mood_word(words: &[String], word: &str) -> bool {
    crate::unported!("bot.compose.fallback:_a_mood_word")
}

/// Python: bot/compose/fallback.py:16812 `_is_rumination(clean)`
pub fn _is_rumination(clean: &str) -> bool {
    crate::unported!("bot.compose.fallback:_is_rumination")
}

/// Python: bot/compose/fallback.py:16832 `_opens_a_question(words)`
pub fn _opens_a_question(words: &[String]) -> bool {
    crate::unported!("bot.compose.fallback:_opens_a_question")
}

/// Python: bot/compose/fallback.py:16901 `is_a_report_reply(said)`
pub fn is_a_report_reply(said: &str) -> bool {
    crate::unported!("bot.compose.fallback:is_a_report_reply")
}

/// Python: bot/compose/fallback.py:16912 `_flat_pools()` (never called on the trace corpus)
pub fn _flat_pools() -> Value {
    crate::unported!("bot.compose.fallback:_flat_pools")
}

// ---- fallback.py:801-1770: phrase matching, the manners ask, the stuck and dithering lines ----

regexes!(HOW_WAS_YOUR_TIME, YOUR_DAY_ASK, YOUR_TIME_ASK);
pools!(HOW_ARE_YOU_ASKS, HOW_ARE_YOU_GOING_ON);
word_sets!(HOW_ARE_YOU_GOING_ON_AFTER, HOW_ARE_YOU_TAGS);

/// Python: bot/compose/fallback.py:801 `_bounded(phrase)`
/// A phrase matched as whole words, so "raining" is not "training". (Python keeps the compiled
/// patterns in `_BOUNDED`; `rx` is the same cache.)
pub fn _bounded(phrase: &str) -> &'static Re {
    rx(&format!("(?<![a-z]){}(?![a-z])", crate::py::re::escape(phrase)))
}

/// Python: bot/compose/fallback.py:810 `_has(clean, phrases)`
pub fn _has(clean: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|phrase| _bounded(phrase).is_match(clean))
}

/// Python: bot/compose/fallback.py:824 `_matchable(text)`
/// The line as a phrase table can see it: lowercase, no apostrophes, single spaces.
pub fn _matchable(text: &str) -> String {
    let t = lower(text).replace('\u{2019}', "'").replace('\'', "");
    text::squash(&t)
}

/// Python: bot/compose/fallback.py:835 `_asks_how_are_you(clean, repaired='')`
pub fn _asks_how_are_you(clean: &str, repaired: &str) -> bool {
    let mut forms = vec![_matchable(clean)];
    if !repaired.is_empty() {
        let r = _matchable(repaired);
        if r != forms[0] {
            forms.push(r);
        }
    }
    for form in &forms {
        for m in HOW_WAS_YOUR_TIME.finditer(form) {
            if _nothing_but_tags(form, m.end()) {
                return true;
            }
        }
        for phrase in HOW_ARE_YOU_ASKS.iter() {
            for m in _bounded(&_matchable(phrase)).finditer(form) {
                if _nothing_but_tags(form, m.end()) {
                    return true;
                }
            }
        }
        for phrase in HOW_ARE_YOU_GOING_ON.iter() {
            for m in _bounded(&_matchable(phrase)).finditer(form) {
                let tail = &form[m.end()..];
                // "my daughter is 7 going on 8": the figure after the phrase is an age.
                if rx(r"\d").is_match(tail) {
                    continue;
                }
                let after = az_words(tail).join(" ");
                if HOW_ARE_YOU_GOING_ON_AFTER.contains(&after) {
                    return true;
                }
            }
        }
        for pattern in [&*YOUR_DAY_ASK, &*YOUR_TIME_ASK] {
            for m in pattern.finditer(form) {
                if _nothing_but_tags(form, m.end()) {
                    return true;
                }
            }
        }
    }
    false
}

/// Python: bot/compose/fallback.py:876 `_nothing_but_tags(form, end)` (never called on the trace corpus)
/// True when what follows the phrase is only a tag. `end` is a byte offset into `form`.
pub fn _nothing_but_tags(form: &str, end: usize) -> bool {
    az_words(&form[end..]).iter().all(|w| HOW_ARE_YOU_TAGS.contains(w))
}

regexes!(STUCK_ON_A_PERSON, STUCK_PERSON_TAIL_PHRASE, STUCK_A_CLAUSE_ABOUT_THEM, STUCK_AND_ANOTHER_PERSON, STUCK_DESCRIBES_THE_PERSON);

/// Python: bot/compose/fallback.py:1124 `_the_person_ends_the_ask(rest)` (never called on the trace corpus)
/// True when what follows the person is about the person, not a thing of theirs.
pub fn _the_person_ends_the_ask(rest: &str) -> bool {
    let rest = rest.trim_start_matches(' ').trim_end_matches([' ', '?', '.', '!', ',']);
    if rest.trim_matches([' ', ',', '?', '!', '.']).is_empty() {
        return true;
    }
    if STUCK_AND_ANOTHER_PERSON.match_(rest).is_some() {
        return true;
    }
    if STUCK_DESCRIBES_THE_PERSON.match_(rest).is_some() {
        return true;
    }
    if STUCK_A_CLAUSE_ABOUT_THEM.match_(rest).is_some() {
        return true;
    }
    let mut left: &str = rest;
    loop {
        let Some(found) = STUCK_PERSON_TAIL_PHRASE.match_(left) else { break };
        left = left[found.end()..].trim_start_matches([' ', ',', '?', '!', '.']);
    }
    left.is_empty()
}

/// Python: bot/compose/fallback.py:1146 `_the_person_they_are_stuck_on(clean)`
/// The person named in "i dont know what to do about my mum", faced back at them ("" for none).
pub fn _the_person_they_are_stuck_on(clean: &str) -> String {
    let Some(found) = STUCK_ON_A_PERSON.search(clean) else { return String::new() };
    if !_the_person_ends_the_ask(found.name_str("rest")) {
        return String::new();
    }
    let mut who = _yours(found.name_str("who"));
    if lower(&who).starts_with("our ") {
        who = format!("your {}", text::slice(&who, 4, None));
    }
    who
}

word_sets!(STUCK_ONLY_WORDS, RUN_OUT_WORDS);
regexes!(RUN_OUT, RUN_OUT_DOUBT, RUN_OUT_EMPTY, STUCK_PHRASES);
pools!(DITHER_ABOUT_A_PERSON, A_FREE_TIME_ASK);

/// Python: bot/compose/fallback.py:1222 `_a_bad_line_above(memory)` (never called on the trace corpus)
/// True when the line before this one was a bad one by its own words.
pub fn _a_bad_line_above(memory: &Memory) -> bool {
    let Some(turn) = memory.turns.last() else { return false };
    let last = &turn.user;
    if text::strip(last).is_empty() {
        return false;
    }
    _mood(&_words(last), last) == "negative"
}

/// Python: bot/compose/fallback.py:1247 `_run_out(clean)`
pub fn _run_out(clean: &str) -> bool {
    let words = az_words(clean);
    if words.is_empty() || !RUN_OUT.is_match(clean) {
        return false;
    }
    if !(RUN_OUT_DOUBT.is_match(clean) || RUN_OUT_EMPTY.is_match(clean)) {
        return false;
    }
    words.iter().all(|w| RUN_OUT_WORDS.contains(text::strip_chars(w, "'")))
}

/// Python: bot/compose/fallback.py:1257 `_a_dead_end(clean)`
pub fn _a_dead_end(clean: &str) -> bool {
    _only_the_stuck_phrase(clean) || _run_out(clean)
}

/// Python: bot/compose/fallback.py:1262 `_only_the_stuck_phrase(clean)`
pub fn _only_the_stuck_phrase(clean: &str) -> bool {
    let words = az_words(clean);
    if words.is_empty() || !STUCK_PHRASES.is_match(clean) {
        return false;
    }
    words.iter().all(|w| STUCK_ONLY_WORDS.contains(text::strip_chars(w, "'")))
}

/// Python: bot/compose/fallback.py:1270 `_a_question_about_the_person(clean, memory)`
/// A question about the person they are stuck on, or None when no person is named.
/// (The skeleton's `()` return could not carry the reply; it returns what Python returns.)
pub fn _a_question_about_the_person(clean: &str, memory: &Memory) -> Option<String> {
    let who = _the_person_they_are_stuck_on(clean);
    if who.is_empty() {
        return None;
    }
    let first = text::slice(&who, 0, Some(1));
    let cap = format!("{}{}", text::upper(&first), text::slice(&who, 1, None));
    let lines: Vec<String> =
        DITHER_ABOUT_A_PERSON.iter().map(|line| format_named(line, &[("who", &who), ("Who", &cap)])).collect();
    choose(&lines, memory, clean)
}

regexes!(SAYS_THE_SIDES, DITHER_TWO_WAY, LIFE_DIRECTION, A_STRETCH_OF_TIME);
word_sets!(HOW_TO_DECIDE_FILLER);

/// Python: bot/compose/fallback.py:1303 `_nothing_to_choose(rest)` (never called on the trace corpus)
/// True when what follows "how do i decide/pick/choose" names no subject of its own.
pub fn _nothing_to_choose(rest: &str) -> bool {
    if SAYS_THE_SIDES.match_(rest).is_some() {
        return true;
    }
    !az09_words(&lower(rest)).iter().any(|w| !HOW_TO_DECIDE_FILLER.contains(w))
}

/// Python: bot/compose/fallback.py:1336 `_the_stretch_of_time(clean, memory)`
/// A question about the free afternoon, or None when the line is not that.
pub fn _the_stretch_of_time(clean: &str, memory: &Memory) -> Option<String> {
    if !A_STRETCH_OF_TIME.is_match(clean) {
        return None;
    }
    choose(&A_FREE_TIME_ASK, memory, clean)
}

/// Python: bot/compose/fallback.py:1350 `_weighs_a_choice(clean)`
pub fn _weighs_a_choice(clean: &str) -> bool {
    if DITHER_TWO_WAY.is_match(clean) {
        return true;
    }
    !(LIFE_DIRECTION.is_match(clean) || A_STRETCH_OF_TIME.is_match(clean))
}

regexes!(FACT_BETWEEN, HELP_SERVICE, PERSON_NOT_A_DECISION, PRONOUN_NOT_A_DECISION, DITHER, DITHER_STRONG, DITHER_SUBJECT);

/// Python: bot/compose/fallback.py:1411 `_reads_as_dither(clean, repaired=None)`
pub fn _reads_as_dither(clean: &str, repaired: &str) -> bool {
    let mut lines = vec![clean];
    if !repaired.is_empty() && repaired != clean {
        lines.push(repaired);
    }
    for line in lines {
        if FACT_BETWEEN.is_match(line) {
            continue;
        }
        if PERSON_NOT_A_DECISION.is_match(line) || PRONOUN_NOT_A_DECISION.is_match(line) || HELP_SERVICE.is_match(line) {
            continue;
        }
        if DITHER.is_match(line) && (DITHER_STRONG.is_match(line) || DITHER_SUBJECT.is_match(line)) {
            return true;
        }
    }
    false
}

regexes!(PLAN_WORDS, TAKING_UP);

/// Python: bot/compose/fallback.py:1634 `_a_plan_in_the_thread(clean, memory)` (never called on the trace corpus)
/// True when the plan the doubt is about is in the line or the turn before it.
pub fn _a_plan_in_the_thread(clean: &str, memory: &Memory) -> bool {
    for t in [clean.to_string(), _the_line_before(memory)] {
        let said = lower(&t);
        if !said.is_empty() && (PLAN_WORDS.is_match(&said) || TAKING_UP.is_match(&said)) {
            return true;
        }
    }
    _a_thing_they_started(memory, &Value::None)
}

regexes!(REFUSES_IT, A_CHILD, SOMEONE_CLOSE, SOMEONE_ELSE, A_CLINIC, AN_AID, THE_DOCTOR, HELP_IN);

/// Python: bot/compose/fallback.py:1696 `_a_carer_refused(clean)`
/// Which of the refusals "my mum wont ..." is ("clinic", "aid", "doctor", "help"), or None.
pub fn _a_carer_refused(clean: &str) -> Option<String> {
    if !REFUSES_IT.is_match(clean) || A_CHILD.is_match(clean) {
        return None;
    }
    if !(SOMEONE_CLOSE.is_match(clean) || SOMEONE_ELSE.is_match(clean)) {
        return None;
    }
    let kind = if A_CLINIC.is_match(clean) {
        "clinic"
    } else if AN_AID.is_match(clean) {
        "aid"
    } else if THE_DOCTOR.is_match(clean) {
        "doctor"
    } else if HELP_IN.is_match(clean) {
        "help"
    } else {
        return None;
    };
    Some(kind.to_string())
}

/// Python: bot/compose/fallback.py:1713 `_a_plan_doubt(pred, clean, repaired)`
/// `pred` is a compiled regex's `search` (the only predicate PyBot passes, `REALISTIC.search`).
pub fn _a_plan_doubt(pred: &Value, clean: &str, repaired: &str) -> bool {
    let rx_ = pred.as_re().unwrap_or_else(|| panic!("_a_plan_doubt: predicate is not a regex: {pred:?}"));
    _a_plan_doubt_with(|t| rx_.is_match(t), clean, repaired)
}

/// `_a_plan_doubt` with any predicate.
pub fn _a_plan_doubt_with(pred: impl Fn(&str) -> bool, clean: &str, repaired: &str) -> bool {
    for t in [clean, repaired] {
        if pred(t) {
            return true;
        }
        let tail = if t.contains(',') { text::strip(t.rsplit(',').next().unwrap_or("")) } else { "" };
        if !tail.is_empty() && pred(tail) {
            return true;
        }
    }
    false
}

regexes!(SHOULD_WORRY, SHOULD_WORRY_FILLER);
word_sets!(BODY_PARTS, HEALTH_WORDS, SYMPTOM_WORDS);
pools!(CHECK_PHRASES);

/// Python: bot/compose/fallback.py:1759 `_a_bare_worry(clean)` (never called on the trace corpus)
/// True when "should i worry" is the whole ask, not attached to a thing they describe.
pub fn _a_bare_worry(clean: &str) -> bool {
    let Some(found) = SHOULD_WORRY.search(clean) else { return false };
    let rest = format!("{} {}", &clean[..found.start()], &clean[found.end()..]);
    let rest = SHOULD_WORRY_FILLER.sub(" ", &rest);
    !rest.chars().any(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// Python: bot/compose/fallback.py:1769 `_worry_is_about_the_body(clean, words)`
pub fn _worry_is_about_the_body(clean: &str, words: &[String]) -> bool {
    words.iter().any(|w| BODY_PARTS.contains(w) || HEALTH_WORDS.contains(w) || SYMPTOM_WORDS.contains(w))
        || CHECK_PHRASES.iter().any(|p| clean.contains(p.as_str()))
}

// ---- fallback.py:1782-2760: whose trouble the thread is, the worry about somebody, unknown pools ----

pools!(
    UNKNOWN_HOW_IT_GOES_FOR_SOMEONE,
    UNKNOWN_HOW_LONG_IN_HOSPITAL,
    UNKNOWN_HOW_IT_GOES_FOR_A_HARD_TIME,
    UNKNOWN_HOW_IT_GOES_FOR_A_PET,
    UNKNOWN_IF_THEY_STILL_CAN,
);
word_sets!(THE_PEOPLE, VISIT_PRONOUNS, NOT_PEOPLE_WORDS, ANIMALS, WILD_BIRDS, MESS_PLACES);
regexes!(IN_HOSPITAL);

/// `{w[:-2] if w.endswith("'s") else w for w in re.findall(r"[a-z']+", said)}`.
fn words_without_possessive(said: &str) -> HashSet<String> {
    az_words(said)
        .into_iter()
        .map(|w| if w.ends_with("'s") { w[..w.len() - 2].to_string() } else { w })
        .collect()
}

/// Python: bot/compose/fallback.py:1845 `_somebody_to_visit(clean, turns, look_back=3)` (never called on the trace corpus)
/// True when the line or the last few turns name a person the visit could be to.
pub fn _somebody_to_visit(clean: &str, turns: &[Turn], look_back: i64) -> bool {
    let is_person = |w: &String| THE_PEOPLE.contains(w) || VISIT_PRONOUNS.contains(w);
    if az_words(clean).iter().any(is_person) {
        return true;
    }
    for turn in last_n(turns, look_back) {
        if az_words(&lower(&turn.user)).iter().any(is_person) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:1857 `_someone_is_a_patient(clean, turns, look_back=4)` (never called on the trace corpus)
pub fn _someone_is_a_patient(clean: &str, turns: &[Turn], look_back: i64) -> bool {
    if IN_HOSPITAL.is_match(clean) {
        return true;
    }
    last_n(turns, look_back).iter().any(|turn| IN_HOSPITAL.is_match(&lower(&turn.user)))
}

regexes!(AN_ANIMAL_DID_IT, AN_ANIMAL_DID_IT_TO_ANYONE, A_PERSON_ELSE_IS_HURT, A_PERSONS_TREATMENT, A_PERSON_IS_BLEEDING, AN_A_HARM_VERB, AN_ANIMAL_WORD);

/// Python: bot/compose/fallback.py:1938 `_the_animal_is_the_patient(said)`
pub fn _the_animal_is_the_patient(said: &str) -> bool {
    let t = lower(said);
    let animals: Vec<usize> = AN_ANIMAL_WORD.finditer(&t).iter().map(|m| m.start()).collect();
    if animals.is_empty() {
        return false;
    }
    if AN_ANIMAL_DID_IT.is_match(&t) {
        return false;
    }
    let Some(verb) = AN_A_HARM_VERB.search(&t) else { return false };
    if *animals.iter().min().unwrap() > verb.start() {
        return false;
    }
    // tail & (THE_PEOPLE - ANIMALS)
    !az_words(&t[verb.end()..]).iter().any(|w| THE_PEOPLE.contains(w) && !ANIMALS.contains(w))
}

/// Python: bot/compose/fallback.py:1958 `_the_person_is_the_patient(said)`
pub fn _the_person_is_the_patient(said: &str) -> bool {
    if said.is_empty() || _the_animal_is_the_patient(said) {
        return false;
    }
    AN_ANIMAL_DID_IT.is_match(said)
        || AN_ANIMAL_DID_IT_TO_ANYONE.is_match(said)
        || A_PERSON_ELSE_IS_HURT.is_match(said)
        || A_PERSONS_TREATMENT.is_match(said)
        || A_PERSON_IS_BLEEDING.is_match(said)
}

/// Python: bot/compose/fallback.py:1975 `_the_thread_is_an_animal(clean, turns, look_back=4)`
pub fn _the_thread_is_an_animal(clean: &str, turns: &[Turn], look_back: i64) -> bool {
    let said = lower(clean);
    let recent: Vec<String> = last_n(turns, look_back).iter().map(|t| lower(&t.user)).collect();
    if _the_person_is_the_patient(&said) || recent.iter().any(|x| _the_person_is_the_patient(x)) {
        return false;
    }
    if az_words(&said).iter().any(|w| NOT_PEOPLE_WORDS.contains(w)) {
        return true;
    }
    recent.iter().any(|x| az_words(x).iter().any(|w| NOT_PEOPLE_WORDS.contains(w)))
}

static TROUBLE_OTHERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.trouble.OTHERS"));

/// Python: bot/compose/fallback.py:1993 `_thread_is_about_someone_else(memory, look_back=3)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _thread_is_about_someone_else(memory: &Memory, look_back: &Value) -> bool {
    for turn in last_n(&memory.turns, look_back_of(look_back, 3)) {
        let words = words_without_possessive(&lower(&turn.user));
        if words.iter().any(|w| TROUBLE_OTHERS.contains(w) && !NOT_PEOPLE_WORDS.contains(w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2008 `_thread_is_about_a_pet(memory, look_back=3)`
pub fn _thread_is_about_a_pet(memory: &Memory, look_back: i64) -> bool {
    for turn in last_n(&memory.turns, look_back).iter().rev() {
        let said = lower(&turn.user);
        let words = words_without_possessive(&said);
        let animals: Vec<&String> = words.iter().filter(|w| ANIMALS.contains(*w)).collect();
        if animals.is_empty() {
            if _the_speaker_is_the_subject(&said) {
                return false;
            }
            continue;
        }
        if animals.iter().all(|a| WILD_BIRDS.contains(*a)) && words.iter().any(|w| MESS_PLACES.contains(w)) {
            continue;
        }
        if crate::compose::loss::reads_as_loss(&said, None, false) || crate::compose::badnews::reads_as_sudden_loss(&said) {
            return false;
        }
        return true;
    }
    false
}

regexes!(
    ASKED_ABOUT,
    WHETHER_IT_GOES_WELL,
    WHETHER_THEY_CAN_STILL,
    WHETHER_IT_GOES_WELL_ITSELF,
    WHETHER_IT_LONGER_HAS_GOT,
    AN_ILLNESS,
    A_PET_IN_TROUBLE,
    WHETHER_IT_IS_THEIR_AGE,
    A_PATIENT_STATE,
    A_HARD_TIME,
    A_MEDICAL_THREAD,
);
word_sets!(_MY_PEOPLE);

/// Python: bot/compose/fallback.py:2145 `_the_thread_said(memory, look_back=3)` (never called on the trace corpus)
/// The last few things the person said, as one lowercase string.
pub fn _the_thread_said(memory: &Memory, look_back: i64) -> String {
    last_n(&memory.turns, look_back).iter().map(|t| lower(&t.user)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/fallback.py:2151 `_a_person_in_the_thread(memory, look_back=3)`
pub fn _a_person_in_the_thread(memory: &Memory, look_back: i64) -> bool {
    for turn in last_n(&memory.turns, look_back) {
        let said = lower(&turn.user);
        let words = words_without_possessive(&said);
        if words.iter().any(|w| _MY_PEOPLE.contains(w.trim_end_matches('s'))) || A_PATIENT_STATE.is_match(&said) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2167 `_a_hard_time_and_not_a_body(memory, line='')` (never called on the trace corpus)
pub fn _a_hard_time_and_not_a_body(memory: &Memory, line: &str) -> bool {
    let said = format!("{} {}", _the_thread_said(memory, 3), line);
    if A_MEDICAL_THREAD.is_match(&said) || AN_ILLNESS.is_match(&said) {
        return false;
    }
    A_HARD_TIME.is_match(&_the_thread_said(memory, 3))
}

/// Python: bot/compose/fallback.py:2181 `_a_pet_in_trouble(line, memory)`
pub fn _a_pet_in_trouble(line: &str, memory: &Memory) -> bool {
    if !_thread_is_about_a_pet(memory, 3) || _a_person_in_the_thread(memory, 2) {
        return false;
    }
    let said = _the_thread_said(memory, 3);
    A_PET_IN_TROUBLE.is_match(line) || A_PET_IN_TROUBLE.is_match(&said) || A_HARD_TIME.is_match(&said)
}

/// Python: bot/compose/fallback.py:2197 `_the_pool_for_a_worry_about_them(clean, memory)`
/// The pool for "is he going to be ok" and its kind, or None. (The skeleton's `()` return could
/// not carry the pool.)
pub fn _the_pool_for_a_worry_about_them(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let line = rx(r"[^a-z0-9' ]").sub(" ", &lower(clean));
    if an_outcome_asked_about_someone(clean, memory).is_some() || the_threads_answer(clean, memory).is_some() {
        return None;
    }
    let still = WHETHER_THEY_CAN_STILL.is_match(&line);
    let itself = WHETHER_IT_GOES_WELL_ITSELF.is_match(&line) || WHETHER_IT_LONGER_HAS_GOT.is_match(&line);
    if itself {
        if _a_pet_in_trouble(&line, memory) {
            return Some(UNKNOWN_HOW_IT_GOES_FOR_A_PET.clone());
        }
        if !_a_person_in_the_thread(memory, 3) {
            return None;
        }
        if !(WHETHER_IT_LONGER_HAS_GOT.is_match(&line) || AN_ILLNESS.is_match(&_the_thread_said(memory, 3))) {
            return None;
        }
    }
    if !(still || itself || WHETHER_IT_GOES_WELL.is_match(&line) || WHETHER_IT_IS_THEIR_AGE.is_match(&line)) {
        return None;
    }
    let named = ASKED_ABOUT.search(&line).map(|m| m.as_str().to_string()).unwrap_or_default();
    if text::split_ref(&named).iter().any(|w| ANIMALS.contains(*w) || NOT_PEOPLE_WORDS.contains(*w)) {
        return Some(UNKNOWN_HOW_IT_GOES_FOR_A_PET.clone());
    }
    if named.is_empty() || rx(r"(?:he|she|they)\b").match_(&named).is_some() {
        if _a_pet_in_trouble(&line, memory) {
            return Some(UNKNOWN_HOW_IT_GOES_FOR_A_PET.clone());
        }
        if !_a_person_in_the_thread(memory, 3) {
            return None;
        }
    }
    if still {
        return Some(UNKNOWN_IF_THEY_STILL_CAN.clone());
    }
    if _a_hard_time_and_not_a_body(memory, &line) {
        return Some(UNKNOWN_HOW_IT_GOES_FOR_A_HARD_TIME.clone());
    }
    if _someone_is_a_patient(&line, &memory.turns, 4) {
        return Some(UNKNOWN_HOW_LONG_IN_HOSPITAL.clone());
    }
    Some(UNKNOWN_HOW_IT_GOES_FOR_SOMEONE.clone())
}

regexes!(THEIR_OWN_STATE);

/// Python: bot/compose/fallback.py:2262 `_the_speaker_is_the_subject(said)`
pub fn _the_speaker_is_the_subject(said: &str) -> bool {
    THEIR_OWN_STATE.is_match(said)
}

/// Python: bot/compose/fallback.py:2271 `_a_thing_they_started(memory, look_back=3)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _a_thing_they_started(memory: &Memory, look_back: &Value) -> bool {
    last_n(&memory.turns, look_back_of(look_back, 3)).iter().any(|t| TAKING_UP.is_match(&lower(&t.user)))
}

regexes!(_AGE_DOUBT, WORRY_SELF, WEIRD_ASK, FEELING_SAID);
pools!(UNKNOWN_WORRY, UNKNOWN_WEIRD_BARE);

/// Python: bot/compose/fallback.py:2285 `_names_their_age(clean)` (never called on the trace corpus)
pub fn _names_their_age(clean: &str) -> bool {
    rx(r"\b\d{1,2}\b").is_match(clean) || _AGE_DOUBT.is_match(clean)
}

/// Python: bot/compose/fallback.py:2291 `_reads_as_worry(clean, words=())`
pub fn _reads_as_worry(clean: &str, words: &[String]) -> bool {
    let split;
    let words = if words.is_empty() {
        split = text::split(clean);
        &split[..]
    } else {
        words
    };
    if _worry_is_about_the_body(clean, words) {
        return false;
    }
    if WORRY_SELF.is_match(clean) {
        return true;
    }
    SHOULD_WORRY.is_match(clean) && _a_bare_worry(clean)
}

/// Python: bot/compose/fallback.py:2301 `_the_weird_ask_pool(clean, memory)`
/// The pool for "is that weird / normal", or None when the line is a question of fact.
pub fn _the_weird_ask_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let found = WEIRD_ASK.search(clean)?;
    if _worry_is_about_the_body(clean, &text::split(clean)) || _thread_is_about_the_body(memory, 4) {
        return None;
    }
    if FEELING_SAID.is_match(clean) || FEELING_SAID.is_match(&_the_line_before(memory)) {
        return Some(UNKNOWN_WORRY.clone());
    }
    if _the_line_before(memory).is_empty() && _nothing_but_the_ask(clean, &found) {
        return Some(UNKNOWN_WEIRD_BARE.clone());
    }
    None
}

/// Python: bot/compose/fallback.py:2324 `_nothing_but_the_ask(clean, found)`
pub fn _nothing_but_the_ask(clean: &str, found: &Match) -> bool {
    let rest = format!("{} {}", &clean[..found.start()], &clean[found.end()..]);
    !SHOULD_WORRY_FILLER.sub(" ", &rest).chars().any(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

static IDIOM_PHRASES: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get("bot.compose.fallback.IDIOM_PHRASES").items().iter().map(Value::strs).collect());

/// Python: bot/compose/fallback.py:2330 `_without_the_idioms(said)`
/// The words of a line with the phrases in `IDIOM_PHRASES` taken out (a set).
pub fn _without_the_idioms(said: &str) -> HashSet<String> {
    let words = az_words(said);
    if words.is_empty() {
        return HashSet::new();
    }
    let mut taken = vec![false; words.len()];
    for phrase in IDIOM_PHRASES.iter() {
        let size = phrase.len();
        if size > words.len() {
            continue;
        }
        for at in 0..=(words.len() - size) {
            if words[at..at + size] == phrase[..] {
                for t in &mut taken[at..at + size] {
                    *t = true;
                }
            }
        }
    }
    words.into_iter().enumerate().filter(|(at, _)| !taken[*at]).map(|(_, w)| w).collect()
}

/// Python: bot/compose/fallback.py:2351 `_thread_is_about_the_body(memory, look_back=4)`
pub fn _thread_is_about_the_body(memory: &Memory, look_back: i64) -> bool {
    for turn in last_n(&memory.turns, look_back) {
        let said = lower(&turn.user);
        if said.is_empty() {
            continue;
        }
        let said = _without_the_things_named_after_a_body(&_without_the_forecast_verb(&said));
        let words = _without_the_idioms(&said);
        if words.iter().any(|w| SYMPTOM_WORDS.contains(w) || BODY_PARTS.contains(w) || HEALTH_WORDS.contains(w)) {
            return true;
        }
    }
    false
}

strings!(HEALTH_WHO);

/// Python: bot/compose/fallback.py:2366 `_the_threads_who(memory, look_back=2)`
/// The person the thread's own line hands the question to, or None.
pub fn _the_threads_who(memory: &Memory, look_back: i64) -> Option<String> {
    for turn in last_n(&memory.turns, look_back).iter().rev() {
        let said = format!(" {} ", rx(r"[^a-z0-9']").sub(" ", &lower(&turn.user)));
        let words = text::split(&said.replace('\'', ""));
        if words.is_empty() {
            continue;
        }
        let who = _who_for(text::strip(&said), &said, &words, None);
        if let Some(who) = who {
            if !who.is_empty() && who != *HEALTH_WHO {
                return Some(who);
            }
        }
    }
    None
}

regexes!(_LEANS_ON_THE_THREAD, _A_CANDIDATE_OF_ITS_OWN, ASK_SHAPE);
pools!(UNKNOWN_WHO, UNKNOWN_HEALTH);

/// `understand(text, memory)` inside Python's `try: ... except Exception: understood = {}`.
fn understand_or_empty(text_: &str, memory: Option<&Memory>) -> Understood {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::understanding::understand(text_, memory)))
        .unwrap_or_default()
}

/// Python: bot/compose/fallback.py:2397 `the_threads_answer(text, memory)`
/// The line for a follow-up that only points at the thread, or None. (The skeleton's `()`
/// return could not carry the line.)
pub fn the_threads_answer(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(text_);
    if clean.is_empty() || _hedged_recently(memory, 3) {
        return None;
    }
    let understood = understand_or_empty(&clean, Some(memory));
    if !_the_ask_is_the_threads(&clean, &understood) {
        return None;
    }
    if let Some(who) = _the_threads_who(memory, 2) {
        let lines: Vec<String> = UNKNOWN_WHO.iter().map(|l| format_named(l, &[("who", &who)])).collect();
        return choose(&lines, memory, &clean);
    }
    let leans = understood.leans_on.as_ref().and_then(|l| l.topic.as_deref());
    if leans == Some("health") || _thread_is_about_the_body(memory, 1) {
        return choose(&UNKNOWN_HEALTH, memory, &clean);
    }
    None
}

word_sets!(_THREAD_MACHINES);

/// Python: bot/compose/fallback.py:2449 `_thread_names_a_machine(memory, look_back=4)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _thread_names_a_machine(memory: &Memory, look_back: &Value) -> bool {
    let body = |w: &str| BODY_PARTS.contains(w) || SYMPTOM_WORDS.contains(w) || HEALTH_WORDS.contains(w);
    for turn in last_n(&memory.turns, look_back_of(look_back, 4)).iter().rev() {
        let tokens = az_words(&lower(&turn.user));
        let machine_at: Vec<usize> = tokens.iter().enumerate().filter(|(_, w)| _THREAD_MACHINES.contains(*w)).map(|(i, _)| i).collect();
        if machine_at.is_empty() {
            if tokens.iter().any(|w| body(w)) {
                return false;
            }
            continue;
        }
        let body_at: Vec<usize> = tokens.iter().enumerate().filter(|(_, w)| body(w)).map(|(i, _)| i).collect();
        if body_at.iter().all(|&i| machine_at.iter().any(|&j| (i as i64 - j as i64).abs() == 1)) {
            return true;
        }
    }
    false
}

static _BODY_TOPICS: LazyLock<Vec<Option<String>>> = LazyLock::new(|| {
    data::get("bot.compose.fallback._BODY_TOPICS").items().iter().map(|v| v.as_str().map(str::to_string)).collect()
});

/// Python: bot/compose/fallback.py:2490 `_the_ask_is_the_threads(clean, understood)`
pub fn _the_ask_is_the_threads(clean: &str, understood: &Understood) -> bool {
    if !(ASK_SHAPE.is_match(clean) && _LEANS_ON_THE_THREAD.is_match(clean)) {
        return false;
    }
    if _A_CANDIDATE_OF_ITS_OWN.is_match(&lower(clean)) {
        return false;
    }
    _BODY_TOPICS.contains(&understood.topic)
}

word_sets!(_THREAD_PEOPLE, _NOT_AN_OWN_THING, MONEY_WORDS, _MONEY_NOUN, SORE_WORDS, SORE_AMBIGUOUS);
regexes!(REPAIR_PERSON, REPAIR_DECISION);

/// Python: bot/compose/fallback.py:2503 `_thread_is_about_a_person(memory, look_back=4)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _thread_is_about_a_person(memory: &Memory, look_back: &Value) -> bool {
    for turn in last_n(&memory.turns, look_back_of(look_back, 4)) {
        if az_words(&lower(&turn.user)).iter().any(|w| _THREAD_PEOPLE.contains(w)) || REPAIR_PERSON.is_match(&turn.user) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2513 `_a_thing_was_named(memory, look_back=4)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _a_thing_was_named(memory: &Memory, look_back: &Value) -> bool {
    for turn in last_n(&memory.turns, look_back_of(look_back, 4)) {
        let said = &turn.user;
        if text::strip(said).is_empty() {
            continue;
        }
        if crate::understanding::understand(said, None).entities.iter().any(|e| e.kind == "thing") {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2535 `_names_its_own_thing(clean)` (never called on the trace corpus)
pub fn _names_its_own_thing(clean: &str) -> bool {
    let Some(m) = REPAIR_DECISION.search(clean) else { return false };
    for word in az_words(&clean[m.end()..]) {
        if _NOT_AN_OWN_THING.contains(&word) {
            continue;
        }
        if BODY_PARTS.contains(&word) || HEALTH_WORDS.contains(&word) || SYMPTOM_WORDS.contains(&word) {
            continue;
        }
        if REPAIR_PERSON.is_match(&word) {
            continue;
        }
        return true;
    }
    false
}

regexes!(_PAY_DEBT, _CARD_MONEY, _VALUE_OF_THING, SORE_DOING, SORE_PICKING, SORE_ONLY_A_BODY, SORE_PLACE);

/// Python: bot/compose/fallback.py:2554 `_money_decision(clean)`
pub fn _money_decision(clean: &str) -> bool {
    let said = az_set(clean);
    if !said.iter().any(|w| MONEY_WORDS.contains(w)) && !_PAY_DEBT.is_match(clean) && !_CARD_MONEY.is_match(clean) {
        return false;
    }
    if _VALUE_OF_THING.is_match(clean) && !said.iter().any(|w| _MONEY_NOUN.contains(w)) {
        return false;
    }
    true
}

/// Python: bot/compose/fallback.py:2564 `_about_a_sore(clean, words)`
pub fn _about_a_sore(clean: &str, words: &[String]) -> bool {
    if !SORE_DOING.is_match(clean) {
        return false;
    }
    let named: HashSet<&String> = words.iter().filter(|w| SORE_WORDS.contains(*w)).collect();
    if named.is_empty() {
        return false;
    }
    if named.iter().any(|w| !SORE_AMBIGUOUS.contains(*w)) {
        return true;
    }
    if !SORE_PICKING.is_match(clean) || SORE_ONLY_A_BODY.is_match(clean) {
        return true;
    }
    SORE_PLACE.is_match(clean)
}

regexes!(
    _ASKS_TO_ACT,
    _A_PLACE_TO_BE_SEEN,
    _A_CALL_WORD,
    DANGER_SYMPTOM,
    MINOR_HARM,
    EMERGENCY_CALL,
    POLICE_CALL,
    REACHING_THE_POLICE,
    _ASKING_FOR_THE_POLICE,
    SOMETHING_HAPPENING,
);
pools!(RING_FOR_HELP, GO_TO_BE_SEEN, RING_THE_POLICE);

/// Python: bot/compose/fallback.py:2583 `_a_call_for_help(clean)`
/// The pool for a line asking whether to ring for help, or None when it is another question.
pub fn _a_call_for_help(clean: &str) -> Option<Vec<String>> {
    if !_ASKS_TO_ACT.is_match(clean) {
        return None;
    }
    if _A_PLACE_TO_BE_SEEN.is_match(clean) && !_A_CALL_WORD.is_match(clean) {
        if DANGER_SYMPTOM.is_match(clean) {
            return Some(RING_FOR_HELP.clone());
        }
        if MINOR_HARM.is_match(clean) {
            return None;
        }
        return Some(GO_TO_BE_SEEN.clone());
    }
    if EMERGENCY_CALL.is_match(clean) {
        if MINOR_HARM.is_match(clean) && !DANGER_SYMPTOM.is_match(clean) {
            return None;
        }
        return Some(RING_FOR_HELP.clone());
    }
    if POLICE_CALL.is_match(clean)
        && (REACHING_THE_POLICE.is_match(clean) || _ASKING_FOR_THE_POLICE.is_match(clean) || SOMETHING_HAPPENING.is_match(clean))
    {
        return Some(RING_THE_POLICE.clone());
    }
    None
}

regexes!(REPAIR_HOW, ASKED_OUT, AN_APOLOGY, _CONTACT, _CALLING, _GOING, _NOT_INVITED);
pools!(REPAIR_ANSWER, ASK_OUT_ANSWER, APOLOGY_ANSWER, CALL_ANSWER, CONTACT_ANSWER, GO_ANSWER);
word_sets!(_A_PERSON, _A_SOCIAL_THING);

/// Python: bot/compose/fallback.py:2627 `_a_small_social_decide(clean, words)`
/// The pool for an everyday social decision, or None when the line is a different question.
pub fn _a_small_social_decide(clean: &str, words: &[String]) -> Option<Vec<String>> {
    if ASKED_OUT.is_match(clean) {
        return Some(ASK_OUT_ANSWER.clone());
    }
    if AN_APOLOGY.is_match(clean) {
        return Some(APOLOGY_ANSWER.clone());
    }
    if _CONTACT.is_match(clean) && words.iter().any(|w| _A_PERSON.contains(w)) {
        if _CALLING.is_match(clean) {
            return Some(CALL_ANSWER.clone());
        }
        return Some(CONTACT_ANSWER.clone());
    }
    if words.iter().any(|w| _A_SOCIAL_THING.contains(w)) && _GOING.is_match(clean) && !_NOT_INVITED.is_match(clean) {
        return Some(GO_ANSWER.clone());
    }
    None
}

regexes!(METHOD_OPENING, HOW_TO_TELL, NOT_A_METHOD, _DEFINITION);
word_sets!(_A_PRODUCT);
pools!(HELP_ALREADY, TIPS_LINES);

/// Python: bot/compose/fallback.py:2691 `method_ask_pool(clean, memory=None)`
/// The pool for a method asked for when nothing else could answer it, else None. (The
/// skeleton's `()` return could not carry the pool.)
pub fn method_ask_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    if METHOD_OPENING.match_(clean).is_none() {
        return None;
    }
    if HOW_TO_TELL.is_match(clean) || NOT_A_METHOD.is_match(clean) {
        return None;
    }
    if az_words(clean).iter().any(|w| _A_PRODUCT.contains(w)) {
        return None;
    }
    if _DEFINITION.is_match(clean) {
        return None;
    }
    if _answered_already(memory) {
        return Some(HELP_ALREADY.clone());
    }
    Some(TIPS_LINES.clone())
}

regexes!(HANDS_IT_BACK, PLASTER_ON, DECIDE_OPENING, DECIDE_SANITY);
pools!(HANDS_BACK, SORE_HANDS, PLASTER_IT, WORD_ANYWHERE, UNKNOWN_WORD, ORDER_OF_WORK, UNKNOWN_MONEY, UNKNOWN_DECIDE, UNKNOWN);
word_sets!(SORE_SERIOUS, _CREATIVE);

/// Python: bot/compose/fallback.py:2719 `unknown_pool(clean, words=())`
/// The pool for a question the bot cannot answer, chosen by the kind of question it is.
pub fn unknown_pool(clean: &str, words: &[String]) -> Vec<String> {
    let split;
    let words = if words.is_empty() {
        split = text::split(clean);
        &split[..]
    } else {
        words
    };
    if HANDS_IT_BACK.match_(clean).is_some() {
        return HANDS_BACK.clone();
    }
    if let Some(calling) = _a_call_for_help(clean) {
        if !calling.is_empty() {
            return calling;
        }
    }
    if REPAIR_HOW.is_match(clean) && words.iter().any(|w| _A_PERSON.contains(w)) {
        return REPAIR_ANSWER.clone();
    }
    if _reads_as_worry(clean, words) {
        return UNKNOWN_WORRY.clone();
    }
    if _about_a_sore(clean, words) {
        if words.iter().any(|w| SORE_SERIOUS.contains(w)) {
            return UNKNOWN_HEALTH.clone();
        }
        return SORE_HANDS.clone();
    }
    if PLASTER_ON.is_match(clean) {
        return PLASTER_IT.clone();
    }
    if WORD_ANYWHERE.iter().any(|p| clean.contains(p.as_str())) && !words.iter().any(|w| _CREATIVE.contains(w)) {
        return UNKNOWN_WORD.clone();
    }
    if !_DEFINITION.is_match(clean) && _an_order_of_work(_choice_sides(clean).as_deref(), clean) {
        return ORDER_OF_WORK.clone();
    }
    if (DECIDE_OPENING.is_match(clean) || DECIDE_SANITY.is_match(clean)) && !_DEFINITION.is_match(clean) {
        if words.iter().any(|w| HEALTH_WORDS.contains(w) || SORE_SERIOUS.contains(w)) || CHECK_PHRASES.iter().any(|p| clean.contains(p.as_str())) {
            return UNKNOWN_HEALTH.clone();
        }
        if _money_decision(clean) {
            return UNKNOWN_MONEY.clone();
        }
        if let Some(social) = _a_small_social_decide(clean, words) {
            if !social.is_empty() {
                return social;
            }
        }
        return UNKNOWN_DECIDE.clone();
    }
    UNKNOWN.clone()
}

// ---- fallback.py:2795-3420: a choking thing, the line before, switching off, "did you see" ----

pools!(THING_STATES);
word_sets!(BODY_SUBJECT, _COPULA, THING_SUBJECT);

/// Python: bot/compose/fallback.py:2798 `_the_thing_is_choking(clean, words)` (never called on the trace corpus)
/// True when a choking state belongs to a thing: the engine, the drain, the bike chain.
pub fn _the_thing_is_choking(clean: &str, words: &[String]) -> bool {
    if !THING_STATES.iter().any(|p| clean.contains(p.as_str())) {
        return false;
    }
    if words.iter().any(|w| BODY_SUBJECT.contains(w)) {
        return false;
    }
    let tokens = text::split(&clean.replace('\'', ""));
    for (i, word) in tokens.iter().enumerate() {
        if word != "choking" {
            continue;
        }
        let mut before: Vec<&String> = tokens[i.saturating_sub(4)..i].iter().collect();
        while before.last().is_some_and(|w| _COPULA.contains(*w)) {
            before.pop();
        }
        if before.last().is_some_and(|w| THING_SUBJECT.contains(*w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:2932 `_the_line_before(memory)`
/// What the person said on the turn before this one, or "" on the first turn.
pub fn _the_line_before(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// Python: bot/compose/fallback.py:2940 `_reads_as_heavy_news(text)`
/// True when a line is heavy news - a death, a diagnosis, a loss - rather than an ordinary one.
pub fn _reads_as_heavy_news(text_: &str) -> bool {
    let said = lower(text_);
    !said.is_empty()
        && (crate::compose::loss::reads_as_loss(&said, None, false)
            || crate::compose::badnews::reads_as_bad_news(&said)
            || crate::compose::badnews::reads_as_sudden_loss(&said))
}

/// Python: bot/compose/fallback.py:2954 `_the_thread_is_heavy(memory)`
pub fn _the_thread_is_heavy(memory: &Memory) -> bool {
    _reads_as_heavy_news(&_the_line_before(memory))
}

regexes!(SWITCH_OFF_ASK, SWITCH_OFF_GET_OUT, SMOKE_ABOUT, ALREADY_SAID_OFF);
word_sets!(SWITCH_OFF_THINGS, KEEP_IT_RUNNING, SWITCH_OFF_FAULT);
pools!(SWITCH_OFF_ALREADY, SWITCH_OFF_ADVISE, SWITCH_OFF_BATTERY);

/// Python: bot/compose/fallback.py:3047 `_a_switch_off_asked(memory, clean)`
/// The pool for "should i turn it off" about a machine in a thread the bot has just answered,
/// or None. (The skeleton's `()` return could not carry the pool.)
pub fn _a_switch_off_asked(memory: &Memory, clean: &str) -> Option<Vec<String>> {
    if !SWITCH_OFF_ASK.is_match(clean) {
        return None;
    }
    let said = format!(" {} ", lower(&_the_line_before(memory)));
    let answer = format!(" {} ", lower(&memory_bot(memory)));
    let thread = format!("{said}{answer}");
    let things = az_set(&thread);
    if !things.iter().any(|w| SWITCH_OFF_THINGS.contains(w)) {
        return None;
    }
    if things.iter().any(|w| KEEP_IT_RUNNING.contains(w)) {
        return None;
    }
    if clean.contains("battery") && ["alarm", "alarms", "smoke"].iter().any(|w| things.contains(*w)) {
        if ["beeping", "beeps", "chirping", "chirps", "chirp", "beep"].iter().any(|w| things.contains(*w)) {
            return Some(SWITCH_OFF_BATTERY.clone());
        }
        return None;
    }
    if SWITCH_OFF_GET_OUT.is_match(&said) {
        return None;
    }
    if SWITCH_OFF_GET_OUT.is_match(&answer) || answer.contains("999") || answer.contains("911") {
        return None;
    }
    if ALREADY_SAID_OFF.is_match(&answer) {
        return Some(SWITCH_OFF_ALREADY.clone());
    }
    if SMOKE_ABOUT.is_match(&said) {
        return None;
    }
    if things.iter().any(|w| SWITCH_OFF_FAULT.contains(w)) {
        return Some(SWITCH_OFF_ADVISE.clone());
    }
    None
}

regexes!(ASKED_IF_I_SAW, NOT_THE_NEWS, LOOKED_UP, SAW_IT, WATCHABLE, SAW_NEWS, SAW_IT_OR_THE);

/// Python: bot/compose/fallback.py:3144 `names_the_thing_worth_watching(clean)` (never called on the trace corpus)
pub fn names_the_thing_worth_watching(clean: &str) -> bool {
    let Some(said) = SAW_IT_OR_THE.match_(clean) else { return false };
    let tail = text::split_ref(&clean[said.end()..]).into_iter().take(5).collect::<Vec<_>>().join(" ");
    WATCHABLE.is_match(&tail) || SAW_NEWS.is_match(&tail)
}

/// Python: bot/compose/fallback.py:3165 `reads_as_if_i_saw(text, memory=None)`
pub fn reads_as_if_i_saw(text_: &str, memory: &Memory) -> bool {
    let clean = text::squash(&lower(text_).replace('\u{2019}', "'"));
    if clean.is_empty() || NOT_THE_NEWS.is_match(&clean) || LOOKED_UP.is_match(&clean) {
        return false;
    }
    if SAW_IT.match_(&clean).is_some() {
        let last = crate::memory::last_user(memory, 1);
        return WATCHABLE.is_match(&lower(last.as_str().unwrap_or("")));
    }
    if ASKED_IF_I_SAW.match_(&clean).is_some() {
        return names_the_thing_worth_watching(&clean);
    }
    false
}

regexes!(
    WAS_THEY_THERE,
    WAS_A_NAMED_ONE_THERE,
    DID_THEY_GO,
    A_THING_OR_A_PERSON,
    A_LOOK_BACK,
    WAS_IT_ANY_GOOD,
    WENT_AND_WERE_ASKED,
    WHO_WAS_THERE,
    HURT_IN_THE_LINE,
);
word_sets!(PEOPLE_WHO_WENT, NOT_A_NAME);

/// Python: bot/compose/fallback.py:3251 `_asked_about_the_day_they_had(clean)`
pub fn _asked_about_the_day_they_had(clean: &str) -> bool {
    if HURT_IN_THE_LINE.is_match(clean) || LOOKED_UP.is_match(clean) {
        return false;
    }
    if WAS_THEY_THERE.match_(clean).is_some() || WAS_IT_ANY_GOOD.match_(clean).is_some() || WHO_WAS_THERE.match_(clean).is_some() {
        return true;
    }
    if let Some(went) = DID_THEY_GO.match_(clean) {
        let verb = clean[went.start()..went.end()].trim_end_matches(text::is_space);
        if verb.ends_with("think") {
            if verb.starts_with("do") || verb.starts_with("does") {
                let said = &clean[went.end()..];
                if !A_LOOK_BACK.is_match(said) || !A_THING_OR_A_PERSON.is_match(said) {
                    return false;
                }
            }
        }
        return true;
    }
    if let Some(m) = WENT_AND_WERE_ASKED.match_(clean) {
        if PEOPLE_WHO_WENT.contains(m.name_str("noun")) {
            return true;
        }
    }
    let Some(named) = WAS_A_NAMED_ONE_THERE.match_(clean) else { return false };
    let who = named.name_str("who");
    let also = named.name("also");
    if NOT_A_NAME.contains(who) || also.is_some_and(|a| !a.is_empty() && NOT_A_NAME.contains(a)) {
        return false;
    }
    true
}

regexes!(BARE_PICK, BARE_PICK_LOST, BARE_PICK_LOST_FIRST, BARE_PICK_START, BARE_PICK_START_CLAUSE);
pools!(LOST_IN_A_THREAD);

/// Python: bot/compose/fallback.py:3342 `_a_lost_ask(clean)`
pub fn _a_lost_ask(clean: &str) -> bool {
    BARE_PICK_LOST.match_(clean).is_some() || BARE_PICK_LOST_FIRST.match_(clean).is_some()
}

/// Python: bot/compose/fallback.py:3394 `_a_start_ask(clean)`
pub fn _a_start_ask(clean: &str) -> bool {
    BARE_PICK_START.match_(clean).is_some() || BARE_PICK_START_CLAUSE.is_match(clean)
}

/// Python: bot/compose/fallback.py:3400 `_said_what_it_is_about(memory)` (never called on the trace corpus)
/// True when a line before this one gave a reply something to name.
pub fn _said_what_it_is_about(memory: &Memory) -> bool {
    for turn in last_n(&memory.turns, 2) {
        let said = text::squash(&lower(&turn.user));
        if said.split(' ').filter(|w| w.chars().count() > 2).count() >= 3 {
            return true;
        }
    }
    false
}

// ---- fallback.py:3424-3990: the lost ask, the bare pick, asking for a person or a hand ----

pools!(START_ASK, BARE_PICK_AFTER_NEWS, BARE_PICK_DO);

/// Python's `try: ... except Exception: ...`: a panic is caught and given back as None, except
/// an UNPORTED one (a function another workstream has not ported yet), which keeps unwinding so
/// the checkers count it.
fn py_try<T>(f: impl FnOnce() -> T) -> Option<T> {
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

/// Python: bot/compose/fallback.py:3424 `the_pool_for_a_lost_ask(memory, clean='')`
/// Which offer "i dont know what to do" gets: the news one, the thread one, or the bare one.
pub fn the_pool_for_a_lost_ask(memory: &Memory, clean: &str) -> Vec<String> {
    if _news_above(memory, false) {
        return BARE_PICK_AFTER_NEWS.clone();
    }
    if _a_start_ask(clean) {
        return START_ASK.clone();
    }
    if _said_what_it_is_about(memory) {
        return LOST_IN_A_THREAD.clone();
    }
    BARE_PICK_DO.clone()
}

/// Python: bot/compose/fallback.py:3435 `the_lost_ask_reply(memory, clean, repaired=None)`
/// The offer for "i dont know what to do", or None when this line is not that ask.
pub fn the_lost_ask_reply(memory: &Memory, clean: &str, repaired: &str) -> Option<String> {
    if !the_advice(memory).is_empty() {
        return None;
    }
    if !(_a_lost_ask(clean) || _a_lost_ask(repaired) || _a_start_ask(clean) || _a_start_ask(repaired)) {
        return None;
    }
    let pick = the_pool_for_a_lost_ask(memory, if _a_start_ask(clean) { clean } else { "" });
    if pick.is_empty() {
        return None;
    }
    choose(&pick, memory, clean)
}

regexes!(WEATHER_COLD, WEATHER_HOT, WEATHER_WET, HEAVY_ABOVE, WORK_NEWS);

/// Python: bot/compose/fallback.py:3499 `_news_above(memory, heavy=False)`
/// True when the line before this one said something heavy.
pub fn _news_above(memory: &Memory, heavy: bool) -> bool {
    if _in_a_care_thread(memory) {
        return true;
    }
    for turn in last_n(&memory.turns, 2) {
        let low = text::squash(&lower(&turn.user));
        if low.is_empty() {
            continue;
        }
        if HEAVY_ABOVE.is_match(&low) {
            return true;
        }
        let news = if heavy {
            crate::compose::badnews::reads_as_heavy_news(&low)
        } else {
            crate::compose::badnews::reads_as_bad_news(&low) && !WORK_NEWS.is_match(&low)
        };
        if news {
            return true;
        }
        if py_try(|| crate::compose::loss::reads_as_loss(&low, None, false)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:3535 `_weather_above(memory)` (never called on the trace corpus)
/// The weather the person named in the last two lines, as "cold", "hot", "wet" or None.
pub fn _weather_above(memory: &Memory) -> Option<String> {
    for turn in last_n(&memory.turns, 2).iter().rev() {
        let said = &turn.user;
        if said.is_empty() {
            continue;
        }
        if WEATHER_WET.is_match(said) {
            return Some("wet".into());
        }
        if WEATHER_COLD.is_match(said) {
            return Some("cold".into());
        }
        if WEATHER_HOT.is_match(said) {
            return Some("hot".into());
        }
    }
    None
}

regexes!(OCCASION_ABOVE, THE_DAY_IS_NAMED, AN_OCCASION_AHEAD, TRIP_ABOVE);

/// Python: bot/compose/fallback.py:3560 `_an_occasion_above(memory)` (never called on the trace corpus)
pub fn _an_occasion_above(memory: &Memory) -> bool {
    last_n(&memory.turns, 2).iter().any(|t| OCCASION_ABOVE.is_match(&t.user))
}

/// Python: bot/compose/fallback.py:3591 `_an_occasion_in_the_thread(memory, back=4)`
pub fn _an_occasion_in_the_thread(memory: &Memory, back: i64) -> bool {
    last_n(&memory.turns, back).iter().any(|t| AN_OCCASION_AHEAD.is_match(&t.user))
}

/// Python: bot/compose/fallback.py:3623 `_a_trip_above(memory)`
pub fn _a_trip_above(memory: &Memory) -> bool {
    last_n(&memory.turns, 2).iter().any(|t| TRIP_ABOVE.is_match(&t.user))
}

word_sets!(BARE_PICK_NEEDS_THE_THING, THREAD_NOTHING);

/// Python: bot/compose/fallback.py:3647 `_a_subject_above(memory)` (never called on the trace corpus)
/// True when the last line the person sent named a thing, so a bare ask is a follow-up. (The
/// second of the two definitions in the Python file, the one the module keeps.)
pub fn _a_subject_above(memory: &Memory) -> bool {
    for turn in last_n(&memory.turns, 2) {
        let said = &turn.user;
        if text::strip(said).is_empty() {
            continue;
        }
        let words = az_words(&lower(said));
        if words.iter().any(|w| !THREAD_NOTHING.contains(text::strip_chars(w, "'")) && w.chars().count() > 2) {
            return true;
        }
    }
    false
}

regexes!(_AFTER_A_CLAUSE);

/// Python: bot/compose/fallback.py:3676 `_the_ask_clause(clean, memory=None)`
/// The ask a line ends with when something sits in front of it, or "" when nothing does.
pub fn _the_ask_clause(clean: &str, memory: &Memory) -> String {
    let said = text::squash(&lower(clean));
    if said.is_empty() {
        return String::new();
    }
    let mut ask = String::new();
    if let Some(reading) = &memory.understood {
        let focus = text::split(&reading.focus.join(" "));
        let joined = focus.join(" ");
        if focus.len() >= 3 && said.ends_with(&joined) && joined != said {
            ask = joined;
        }
    }
    if ask.is_empty() {
        ask = _AFTER_A_CLAUSE.search(&said).map(|m| text::squash(m.str(1))).unwrap_or_default();
    }
    if text::split_ref(&ask).len() >= 3 { ask } else { String::new() }
}

static BARE_PICK_KIND: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.BARE_PICK_KIND").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
pools!(BARE_PICK_WHICH, BARE_PICK_WEAR, BARE_PICK_WEAR_WET, BARE_PICK_WEAR_COLD, BARE_PICK_WEAR_HOT, BARE_PICK_BRING, BARE_PICK_TAKE);

/// Python: bot/compose/fallback.py:3693 `_bare_pick(clean, memory=None)`
/// The pool for a line that asks the bot to choose and names nothing to choose between, or None.
/// (The skeleton's `()` return could not carry the pool.) Python tells the pools apart by
/// identity (`pool is BARE_PICK_WEAR`); every pool here has its own lines, so equality is the same.
pub fn _bare_pick(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let mut found = BARE_PICK.match_(clean);
    let mut from_the_clause = false;
    let ask;
    if found.is_none() {
        ask = _the_ask_clause(clean, memory);
        found = if ask.is_empty() { None } else { BARE_PICK.match_(&ask) };
        from_the_clause = found.is_some();
    }
    let found = found?;
    if lower(text::strip(found.as_str())).starts_with("which") {
        return Some(BARE_PICK_WHICH.clone());
    }
    let verb = text::squash(&lower(found.name_str("verb")));
    let pool = BARE_PICK_KIND.iter().find(|(k, _)| *k == verb).map(|(_, v)| v.clone()).unwrap_or_else(|| BARE_PICK_DO.clone());
    if from_the_clause && (OCCASION_ABOVE.is_match(clean) || THE_DAY_IS_NAMED.is_match(clean) || _an_occasion_in_the_thread(memory, 4)) {
        return None;
    }
    if from_the_clause && pool == *BARE_PICK_DO {
        return None;
    }
    if BARE_PICK_NEEDS_THE_THING.contains(&verb) && _a_subject_above(memory) {
        return None;
    }
    if pool == *BARE_PICK_WEAR && !_an_occasion_above(memory) {
        match _weather_above(memory).as_deref() {
            Some("wet") => return Some(BARE_PICK_WEAR_WET.clone()),
            Some("cold") => return Some(BARE_PICK_WEAR_COLD.clone()),
            Some("hot") => return Some(BARE_PICK_WEAR_HOT.clone()),
            _ => {}
        }
    }
    if pool == *BARE_PICK_BRING && _a_trip_above(memory) {
        return Some(BARE_PICK_TAKE.clone());
    }
    if pool == *BARE_PICK_DO && _news_above(memory, true) {
        return Some(BARE_PICK_AFTER_NEWS.clone());
    }
    Some(pool)
}

regexes!(HUMAN_LEAD, HUMAN_QUOTED, HUMAN_HELPS, HUMAN_TAIL, HUMAN_TALK_ASK, HUMAN_TO_ME, HUMAN_OTHER_END, HAND_ASK);
word_sets!(HUMAN_EMPTY);

/// Python: bot/compose/fallback.py:3792 `_opens_the_line(prefix)` (never called on the trace corpus)
/// True when what comes before the human/person ask is only a greeting or a question opener.
pub fn _opens_the_line(prefix: &str) -> bool {
    HUMAN_LEAD.match_(prefix).is_some() || HUMAN_QUOTED.is_match(prefix)
}

/// Python: bot/compose/fallback.py:3835 `_asks_elsewhere(tail)` (never called on the trace corpus)
pub fn _asks_elsewhere(tail: &str) -> bool {
    if HUMAN_HELPS.match_(tail).is_some() {
        return false;
    }
    if HUMAN_TAIL.match_(tail).is_some() {
        return false;
    }
    if HUMAN_OTHER_END.match_(tail).is_some() {
        return false;
    }
    rx("[a-z]+").findall_str(&lower(tail)).iter().any(|w| !HUMAN_EMPTY.contains(w))
}

/// Python: bot/compose/fallback.py:3846 `_asks_for_a_person_to_talk_to(clean)`
pub fn _asks_for_a_person_to_talk_to(clean: &str) -> bool {
    let Some(matched) = HUMAN_TALK_ASK.search(clean) else { return false };
    if !_opens_the_line(&clean[..matched.start()]) {
        return false;
    }
    let tail = &clean[matched.end()..];
    HUMAN_TAIL.match_(tail).is_some() || HUMAN_TO_ME.match_(tail).is_some() || HUMAN_OTHER_END.match_(tail).is_some()
}

/// Python: bot/compose/fallback.py:3887 `_asks_for_help(clean, understood=None)`
/// A bare ask for a hand as the whole line, commas and all ("hi, can you help me").
pub fn _asks_for_help(clean: &str, understood: Option<&Understood>) -> bool {
    let mut lines = vec![clean.to_string()];
    if let Some(u) = understood {
        if !u.plain.is_empty() && u.plain != clean {
            lines.push(u.plain.clone());
        }
    }
    for line in &lines {
        let bare = text::squash(&rx("[^a-z0-9 ]+").sub(" ", &lower(line)));
        if HAND_ASK.match_(&bare).is_some() {
            return true;
        }
    }
    false
}

// ---- fallback.py:4158-4316: warm lines that only say what was said, the occasion, a slip ----

static DETAILS_IN_A_WARM_LINE: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.DETAILS_IN_A_WARM_LINE")
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].strs()))
        .collect()
});

/// Python: bot/compose/fallback.py:4151 `_says_only_what_they_said(lines, clean)`
/// The warm lines whose own detail the person's line carries, or the plain ones if none do.
pub fn _says_only_what_they_said(lines: &[String], clean: &str) -> Vec<String> {
    let said = az_set(&lower(clean));
    let unsupported = |line: &String| -> bool {
        let low = lower(line);
        for (phrase, support) in DETAILS_IN_A_WARM_LINE.iter() {
            if low.contains(phrase.as_str()) {
                return !support.iter().any(|w| said.contains(w));
            }
        }
        false
    };
    let kept: Vec<String> = lines.iter().filter(|l| !unsupported(l)).cloned().collect();
    if !kept.is_empty() {
        return kept;
    }
    lines.to_vec()
}

regexes!(HAPPENING_NOW, DATE_GIVEN, _APART, _PLACES_THEM);
word_sets!(_SHE, _HE);

/// Python: bot/compose/fallback.py:4222 `_the_pronoun(clean)` (never called on the trace corpus)
/// her or him for the one person the day belongs to, "" when the line names none or both.
pub fn _the_pronoun(clean: &str) -> String {
    let words = az_words(&lower(clean).replace('\u{2019}', "'"));
    let (mut she, mut he) = (false, false);
    for word in &words {
        let cut = &word[..word.len().saturating_sub(1)];
        let stem: &str = if word.ends_with('s') && (_SHE.contains(cut) || _HE.contains(cut)) { cut } else { word };
        if _SHE.contains(stem) {
            she = true;
        } else if _HE.contains(stem) {
            he = true;
        }
    }
    if she && !he {
        return "her".into();
    }
    if he && !she {
        return "him".into();
    }
    String::new()
}

static SOMEONE_ELSES_DAY: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.SOMEONE_ELSES_DAY").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static OCCASION: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.OCCASION").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});

/// Python: bot/compose/fallback.py:4239 `_occasion_pool(occasion, clean)` (never called on the trace corpus)
/// The lines for this occasion (a name such as "birthday"), as a tuple value.
pub fn _occasion_pool(occasion: &Value, clean: &str) -> Value {
    Value::Tuple(std::sync::Arc::new(occasion_pool(occasion.str(), clean).into_iter().map(Value::from).collect()))
}

/// `_occasion_pool` with the occasion as a name and the pool as lines.
pub fn occasion_pool(occasion: &str, clean: &str) -> Vec<String> {
    if crate::compose::goodnews::someone_elses_occasion(clean) {
        if let Some((_, pool)) = SOMEONE_ELSES_DAY.iter().find(|(k, _)| k == occasion) {
            if !pool.is_empty() {
                let apart = _APART.is_match(clean);
                let mut kept: Vec<String> = pool.iter().filter(|l| apart || !_PLACES_THEM.is_match(l)).cloned().collect();
                let pronoun = _the_pronoun(clean);
                if !pronoun.is_empty() {
                    kept = kept.iter().map(|l| rx(r"\bthem\b").sub(&pronoun, l)).collect();
                }
                return if kept.is_empty() { pool.clone() } else { kept };
            }
        }
    }
    let pool = OCCASION
        .iter()
        .find(|(k, _)| k == occasion)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| panic!("KeyError: {occasion:?} (OCCASION)"));
    let mut drop: Vec<&str> = Vec::new();
    if HAPPENING_NOW.is_match(clean) {
        drop.push("when it comes");
    }
    if DATE_GIVEN.is_match(clean) {
        drop.push("how far off");
    }
    if !drop.is_empty() {
        let kept: Vec<String> = pool.iter().filter(|l| !drop.iter().any(|p| lower(l).contains(p))).cloned().collect();
        if !kept.is_empty() {
            return kept;
        }
    }
    pool
}

regexes!(
    SERIOUS_NEWS,
    BABY_LOSS,
    SELF_BLAME_ASKS,
    A_VERDICT_ON_THE_PERSON,
    GOING_TO_DIE,
    HARM_IN_A_LAUGH,
    LAUGH_STOPPED,
    A_CROWD_LAUGHED,
    A_PERSON_LAUGHED,
    A_OWN_LAUGH,
    LAUGHED_AT_THEM,
    SOMEONE_ELSES_SHAME,
    DIE_OF_IT,
    THE_GROUND_OPENS,
    EMBARRASSMENT_WORD,
    THE_MOMENT_IS_THEIRS,
    A_SLIP_IS_NOT_A_FALL,
    A_SOCIAL_SLIP,
    A_SLIP,
    IN_A_ROOM,
);
pools!(LAUGHED_AT, EMBARRASSED);

/// Python: bot/compose/fallback.py:4265 `_their_own_slip(clean, memory=None)`
/// Which pool an awkward moment gets: "at them" or "of their own", or None. (The skeleton's
/// `()` return could not carry the pool.)
pub fn _their_own_slip(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    if clean.contains('?') {
        return None;
    }
    if HEAVY_ABOVE.is_match(clean) || SERIOUS_NEWS.is_match(clean) || BABY_LOSS.is_match(clean) {
        return None;
    }
    if SELF_BLAME_ASKS.is_match(clean) || A_VERDICT_ON_THE_PERSON.is_match(clean) {
        return None;
    }
    if GOING_TO_DIE.is_match(clean) {
        return None;
    }
    if HARM_IN_A_LAUGH.is_match(clean) || LAUGH_STOPPED.is_match(clean) {
        return None;
    }
    if _in_a_care_thread(memory) {
        return None;
    }
    if A_CROWD_LAUGHED.is_match(clean) {
        return Some(LAUGHED_AT.clone());
    }
    if A_PERSON_LAUGHED.is_match(clean) || A_OWN_LAUGH.is_match(clean) {
        return None;
    }
    if LAUGHED_AT_THEM.is_match(clean) {
        return Some(LAUGHED_AT.clone());
    }
    if SOMEONE_ELSES_SHAME.is_match(clean) {
        return None;
    }
    if DIE_OF_IT.is_match(clean) || THE_GROUND_OPENS.is_match(clean) {
        return Some(EMBARRASSED.clone());
    }
    if EMBARRASSMENT_WORD.is_match(clean) && THE_MOMENT_IS_THEIRS.is_match(clean) {
        return Some(EMBARRASSED.clone());
    }
    if A_SLIP_IS_NOT_A_FALL.is_match(clean) {
        return None;
    }
    if A_SOCIAL_SLIP.is_match(clean) || (A_SLIP.is_match(clean) && IN_A_ROOM.is_match(clean)) {
        return Some(EMBARRASSED.clone());
    }
    None
}

// ---- fallback.py:4374-4720: serious news about someone close, a death and the words, results ----

regexes!(CLOSE_OWNER, NOT_THEIR_NEWS, NOT_YET_NEWS);

/// Python: bot/compose/fallback.py:4371 `reads_as_someone_close_is_ill(clean)` (never called on the trace corpus)
/// Whether a cleaned, lowercased line is serious news about someone the person loves.
pub fn reads_as_someone_close_is_ill(clean: &str) -> bool {
    if clean.is_empty() || NOT_THEIR_NEWS.is_match(clean) {
        return false;
    }
    if !CLOSE_OWNER.is_match(clean) {
        return false;
    }
    let Some(serious) = SERIOUS_NEWS.search(clean) else { return false };
    if let Some(doubt) = NOT_YET_NEWS.search(clean) {
        if doubt.start() < serious.start() {
            return false;
        }
    }
    true
}

regexes!(TELLING_A_CHILD, A_DEATH_FROM_TODAY, A_CHILD_DIED, A_DEATH_IN_THE_LINE, A_SUICIDE_IN_THE_LINE);

/// Python: bot/compose/fallback.py:4501 `a_fresh_death_and_the_words(clean)`
pub fn a_fresh_death_and_the_words(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    TELLING_A_CHILD.is_match(&t)
        && A_DEATH_IN_THE_LINE.is_match(&t)
        && A_DEATH_FROM_TODAY.is_match(&t)
        && !A_CHILD_DIED.is_match(&t)
        && !A_SUICIDE_IN_THE_LINE.is_match(&t)
}

/// Python: bot/compose/fallback.py:4517 `asks_what_to_tell_a_child(clean)`
pub fn asks_what_to_tell_a_child(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    TELLING_A_CHILD.is_match(&t) && A_DEATH_IN_THE_LINE.is_match(&t) && !A_DEATH_FROM_TODAY.is_match(&t) && !A_CHILD_DIED.is_match(&t)
}

regexes!(_RESULT, _WAITING, _DOUBT, _GUESSED_RESULT, _RESOLVED, _A_TIME, _ANSWER_GOOD, _ANSWER_BAD, _LANDED);

/// Python: bot/compose/fallback.py:4622 `_an_open_result(clean)`
/// True when the line reports a result the person has not heard yet.
pub fn _an_open_result(clean: &str) -> bool {
    if _RESOLVED.is_match(clean) {
        return false;
    }
    if _WAITING.is_match(clean) {
        return true;
    }
    _RESULT.is_match(clean) && _DOUBT.is_match(clean)
}

/// Python: bot/compose/fallback.py:4651 `_the_news_landed(clean)`
/// "bad", "good" or "landed" when the answer came back, None while it has not. (The skeleton's
/// `()` return could not carry the answer.)
pub fn _the_news_landed(clean: &str) -> Option<String> {
    if !_LANDED.is_match(clean) {
        return None;
    }
    if _ANSWER_BAD.is_match(clean) {
        return Some("bad".into());
    }
    if _ANSWER_GOOD.is_match(clean) {
        return Some("good".into());
    }
    Some("landed".into())
}

/// Python: bot/compose/fallback.py:4668 `reads_as_a_result_line(text)`
pub fn reads_as_a_result_line(text_: &str) -> bool {
    let clean = _plain(&_matchable(text_));
    if clean.is_empty() {
        return false;
    }
    _the_news_landed(&clean).is_some() || _an_open_result(&clean) || _a_guessed_result(&clean)
}

/// Python: bot/compose/fallback.py:4683 `_a_guessed_result(clean)`
pub fn _a_guessed_result(clean: &str) -> bool {
    _GUESSED_RESULT.is_match(clean)
}

/// Python: bot/compose/fallback.py:4688 `_a_time_in_it(clean)` (never called on the trace corpus)
pub fn _a_time_in_it(clean: &str) -> bool {
    _A_TIME.is_match(clean)
}

regexes!(A_COST, SOMEBODY_ASKS, A_ROLE_IN_SOMEBODY_ELSES_DAY);

/// Python: bot/compose/fallback.py:4712 `_a_cost_somebody_asks(clean)`
/// True for a bill somebody else has put on the person for a shared occasion.
pub fn _a_cost_somebody_asks(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    if t.is_empty() || t.contains('?') || t.split(' ').count() > 24 {
        return false;
    }
    if !A_COST.is_match(&t) {
        return false;
    }
    if !(SOMEBODY_ASKS.is_match(&t) || A_ROLE_IN_SOMEBODY_ELSES_DAY.is_match(&t)) {
        return false;
    }
    !rx(r"\bmy\s+(?:own\s+)?(?:wedding|holiday|party|trip)\b").is_match(&t)
}

// ---- fallback.py:4867-5590: the day remarked on, a plan dropped, no words, invitations, limits ----

static DAY_WORDS: LazyLock<Vec<(String, HashSet<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.DAY_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
});
static THE_DAY: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.THE_DAY").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static THE_DAY_AHEAD: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.THE_DAY_AHEAD").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static DAY_SAID_AS_A_KIND: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.DAY_SAID_AS_A_KIND").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
word_sets!(
    DAY_GLUE,
    NEGATIONS,
    NEGATION_SOFTENERS,
    DAY_TO_COME,
    DAY_VERDICTS,
    DAY_ASKS,
    DAY_THEIR_OWN_DAY,
    DAY_JOINED_ON,
    DAY_NEEDS_A_DAY_NAMED,
    DAY_NAMED,
);
pools!(DAY_ON_THE_WAY, DAY_AS_ANYTHING, DAY_TO_COME_PHRASES, DAY_TO_COME_PAST, DAY_NOT_THE_DAY, DAY_IN_HERE);
regexes!(DAY_MET_ME);

/// `dict.get(key)` on a table of pools.
fn pool_of(table: &[(String, Vec<String>)], key: &str) -> Option<Vec<String>> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

/// `re.sub(r"\b" + r"\s+".join(phrase.split()) + r"\b", repl, line)`.
fn sub_phrase(line: &str, phrase: &str, repl: &str) -> String {
    let pattern = format!(r"\b{}\b", text::split_ref(phrase).join(r"\s+"));
    rx(&pattern).sub(repl, line)
}

/// Python: bot/compose/fallback.py:4867 `_the_day_described(clean)`
/// The pool for a line that says what the day is like, or None. (The skeleton's `()` return
/// could not carry the pool.)
pub fn _the_day_described(clean: &str) -> Option<Vec<String>> {
    let mut line = text::squash(&lower(clean));
    if line.is_empty() || line.contains('?') {
        return None;
    }
    line = rx(r"^what\s+an?\s+").sub("", &line);
    if DAY_MET_ME.match_(&line).is_some() {
        line = DAY_MET_ME.sub("", &line);
        for phrase in DAY_ON_THE_WAY.iter() {
            line = sub_phrase(&line, phrase, " ");
        }
        line = text::squash(&line);
    }
    for phrase in DAY_AS_ANYTHING.iter() {
        line = sub_phrase(&line, phrase, " ");
    }
    for (phrase, word) in DAY_SAID_AS_A_KIND.iter() {
        line = sub_phrase(&line, phrase, word);
    }
    line = text::squash(&line);
    let words: Vec<String> = text::split_ref(&line)
        .iter()
        .map(|w| w.trim_matches(['.', ',', '!', ';', ':', '\'', '\u{2019}', '"']).to_string())
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() || words.len() > 8 {
        return None;
    }
    if DAY_ASKS.contains(&words[0]) || DAY_THEIR_OWN_DAY.contains(&words[0]) || DAY_JOINED_ON.contains(&words[0]) {
        return None;
    }
    if DAY_NOT_THE_DAY.iter().any(|p| line.contains(p.as_str())) {
        return None;
    }
    if DAY_IN_HERE.iter().any(|p| line.contains(p.as_str())) {
        return None;
    }
    let in_a_pool = |w: &str| DAY_WORDS.iter().any(|(_, pool)| pool.contains(w));
    if !words.iter().any(|w| DAY_NAMED.contains(w) || DAY_TO_COME.contains(w)) {
        let day_words: Vec<&String> = words.iter().filter(|w| in_a_pool(w)).collect();
        if !day_words.is_empty() && day_words.iter().all(|w| DAY_NEEDS_A_DAY_NAMED.contains(*w)) {
            return None;
        }
    }
    if words.iter().any(|w| !DAY_GLUE.contains(w) && !in_a_pool(w)) {
        return None;
    }
    if DAY_TO_COME_PAST.iter().any(|p| line.contains(p.as_str())) {
        return None;
    }
    for (index, word) in words.iter().enumerate() {
        if index > 0 && index + 1 < words.len() && in_a_pool(word) {
            if words[index - 1] == "the" && (words[index + 1] == "one" || words[index + 1] == "ones") {
                return None;
            }
        }
    }
    let day_words: Vec<&String> = words.iter().filter(|w| in_a_pool(w)).collect();
    if (words.iter().any(|w| w == "was") || words.iter().any(|w| w == "were"))
        && !day_words.is_empty()
        && day_words.iter().all(|w| DAY_VERDICTS.contains(*w))
        && !words.iter().any(|w| DAY_TO_COME.contains(w))
    {
        return None;
    }
    let ruled_out = |index: usize| -> bool {
        let mut back = index as i64 - 1;
        while back >= 0 && NEGATION_SOFTENERS.contains(&words[back as usize]) {
            back -= 1;
        }
        back >= 0 && NEGATIONS.contains(&words[back as usize])
    };
    let (mut best, mut best_count): (Option<&str>, usize) = (None, 0);
    let (mut ruled, mut ruled_count): (Option<&str>, usize) = (None, 0);
    for (kind, pool) in DAY_WORDS.iter() {
        let count = words.iter().enumerate().filter(|(i, w)| pool.contains(*w) && !ruled_out(*i)).count();
        if count > best_count {
            best = Some(kind);
            best_count = count;
        }
        let cleared = words.iter().enumerate().filter(|(i, w)| pool.contains(*w) && ruled_out(*i)).count();
        if cleared > ruled_count {
            ruled = Some(kind);
            ruled_count = cleared;
        }
    }
    let Some(best) = best else {
        if ruled.is_some() {
            return pool_of(&THE_DAY, "mild");
        }
        return None;
    };
    if words.iter().any(|w| DAY_TO_COME.contains(w)) || DAY_TO_COME_PHRASES.iter().any(|p| line.contains(p.as_str())) {
        return pool_of(&THE_DAY_AHEAD, best).filter(|p| !p.is_empty()).or_else(|| pool_of(&THE_DAY, best));
    }
    pool_of(&THE_DAY, best)
}

/// Python: bot/compose/fallback.py:4961 `the_day_remarked(text)`
/// The pool for a remark about the day as it is or as it is coming, or None. (The skeleton's
/// `()` return could not carry the pool.)
pub fn the_day_remarked(text_: &str) -> Option<Vec<String>> {
    let line = text::squash(&lower(text_));
    let pool = _the_day_described(&line)?;
    if pool.is_empty() {
        return None;
    }
    if rx(r"\b(?:was|were)\b").is_match(&line) {
        return None;
    }
    Some(pool)
}

regexes!(
    DECLINE_ABOUT_THE_BOT,
    DECLINE_SOMEONE_ELSE,
    DECLINE_VERB,
    DECLINE_MUST,
    DECLINE_ASK,
    DECLINE_TIME,
    DECLINE_OCCASION,
    DECLINE_DATE,
    DECLINE_OTHER,
    DECLINE_OTHER_SUBJECT,
    DECLINE_BE_THERE,
    DECLINE_AFTER,
    BODY_SAYS_WHY_PHRASE,
    BABY_ON_THE_WAY,
    VERB_PARTICLE,
    LATE_NOT_A_TIME,
    PLAN_ON_THE_TABLE,
    LINE_CANCELS,
);
word_sets!(BODY_SAYS_WHY);

/// Python: bot/compose/fallback.py:5070 `_the_reason_is_on_the_body(text)` (never called on the trace corpus)
pub fn _the_reason_is_on_the_body(text_: &str) -> bool {
    if text_.is_empty() {
        return false;
    }
    let words: Vec<String> = az_set(text_).into_iter().collect();
    words.iter().any(|w| BODY_SAYS_WHY.contains(w))
        || _worry_is_about_the_body(text_, &words)
        || LATE_NOT_A_TIME.is_match(text_)
        || BABY_ON_THE_WAY.is_match(text_)
        || BODY_SAYS_WHY_PHRASE.is_match(text_)
}

/// Python: bot/compose/fallback.py:5104 `_the_line_cancels(line)` (never called on the trace corpus)
pub fn _the_line_cancels(line: &str) -> bool {
    LINE_CANCELS.is_match(line)
}

/// Python: bot/compose/fallback.py:5109 `_a_plan_on_the_table(memory)` (never called on the trace corpus)
pub fn _a_plan_on_the_table(memory: &Memory) -> bool {
    last_n(&memory.turns, 3).iter().any(|t| !t.user.is_empty() && PLAN_ON_THE_TABLE.is_match(&t.user))
}

/// Python: bot/compose/fallback.py:5124 `_hesitant_news_before(memory, turns=3)` (never called on the trace corpus)
/// (`turns` is the count of the person's lines to read, as in Python.)
pub fn _hesitant_news_before(memory: &Memory, turns: i64) -> bool {
    last_n(&memory.turns, turns).iter().any(|t| {
        let said = lower(&t.user);
        !said.is_empty() && crate::compose::badnews::reads_as_bad_news(&said)
    })
}

/// Python: bot/compose/fallback.py:5140 `_a_hard_week(memory)` (never called on the trace corpus)
pub fn _a_hard_week(memory: &Memory) -> bool {
    _in_a_loss_thread(memory) || _a_loss_recently(memory) || _hesitant_news_before(memory, 3)
}

/// Python: bot/compose/fallback.py:5150 `_the_turn_before_was_a_heavy_one(memory, line='')` (never called on the trace corpus)
pub fn _the_turn_before_was_a_heavy_one(memory: &Memory, line: &str) -> bool {
    let plain = if line.is_empty() { String::new() } else { _plain(line) };
    if !plain.is_empty() && !(_a_declined_plan(&plain, true) && !DECLINE_TIME.is_match(&plain)) {
        return false;
    }
    let Some(last) = memory.turns.last() else { return false };
    let said = lower(text::strip(&last.user));
    if said.is_empty() {
        return false;
    }
    let read = py_try(|| crate::understanding::understand(&said, None));
    if let Some(read) = &read {
        if read.intent == "question" || read.intent == "ask" || read.kind == "question" {
            return false;
        }
    }
    if let Some(note) = memory.get("heavy_reply") {
        if note.is_dict() && note.get("turn").and_then(Value::as_int) == Some(memory.turns.len() as i64 - 1) {
            return true;
        }
    }
    if crate::compose::badnews::reads_like_a_symptom(&said) || _the_reason_is_on_the_body(&said) {
        return true;
    }
    let Some(read) = read else { return false };
    if read.intent == "feeling" && read.polarity < 0 {
        return true;
    }
    read.emotion.as_deref().is_some_and(|e| !e.is_empty()) || read.body.as_deref().is_some_and(|b| !b.is_empty())
}

/// The line `_a_declined_plan` and `_someone_else_dropped_it` read: no apostrophes, one space.
fn decline_line(clean: &str) -> String {
    text::squash(text::strip(&clean.replace('\u{2019}', "'").replace('\'', "")))
}

/// The first decline in the line that is not about the bot: (is it `DECLINE_MUST`, the match).
fn the_decline<'t>(line: &'t str) -> Option<(bool, Match<'t>)> {
    for (must, pattern) in [(false, &*DECLINE_VERB), (true, &*DECLINE_MUST)] {
        for m in pattern.finditer(line) {
            if DECLINE_ABOUT_THE_BOT.is_match(&line[..m.start()]) {
                continue;
            }
            return Some((must, m));
        }
    }
    None
}

/// Python: bot/compose/fallback.py:5207 `_a_declined_plan(clean, without_a_thing=False)`
/// True when the line is the person saying they cannot make a plan.
pub fn _a_declined_plan(clean: &str, without_a_thing: bool) -> bool {
    let line = decline_line(clean);
    if line.is_empty() || DECLINE_ASK.match_(&line).is_some() || line.ends_with('?') {
        return false;
    }
    let Some((must, found)) = the_decline(&line) else { return false };
    let group1 = found.str(1);
    if _the_reason_is_on_the_body(&line[..found.start()]) || _the_reason_is_on_the_body(&VERB_PARTICLE.sub(" ", group1)) {
        return false;
    }
    let stripped = group1.trim_matches([' ', ',', '.', '-', '!', '?']);
    let tail_owned = DECLINE_AFTER.sub("", stripped);
    let tail = tail_owned.trim_matches([' ', ',', '.', '-', '!', '?']);
    if tail.is_empty() {
        return true;
    }
    if tail.starts_with("for ") {
        let head = &line[..found.end() - group1.len()];
        if DECLINE_BE_THERE.is_match(head.trim_end_matches(text::is_space)) {
            return false;
        }
    }
    if must && !DECLINE_TIME.is_match(tail) {
        return false;
    }
    if DECLINE_DATE.is_match(tail) {
        return true;
    }
    if DECLINE_TIME.is_match(tail) {
        if DECLINE_OTHER.match_(tail).is_none() || DECLINE_OTHER_SUBJECT.match_(&line).is_none() {
            return true;
        }
    }
    if DECLINE_OTHER.match_(tail).is_some() {
        return false;
    }
    if tail == "it" || tail == "there" || tail == "along" {
        return true;
    }
    if without_a_thing {
        return false;
    }
    if ["to ", "for ", "it to ", "it on ", "it for ", "it at ", "it next "].iter().any(|p| tail.starts_with(p)) {
        return true;
    }
    DECLINE_OCCASION.is_match(tail)
}

/// Python: bot/compose/fallback.py:5275 `_someone_else_dropped_it(clean)` (never called on the trace corpus)
pub fn _someone_else_dropped_it(clean: &str) -> bool {
    let line = decline_line(clean);
    if line.is_empty() || !_a_declined_plan(clean, false) {
        return false;
    }
    match the_decline(&line) {
        Some((_, m)) => DECLINE_SOMEONE_ELSE.is_match(&line[..m.start()]),
        None => false,
    }
}

regexes!(NO_WORDS, NO_WORDS_NOT, NO_WORDS_TO, NO_WORDS_SHE, NO_WORDS_CROWD, NO_WORDS_ONE, NO_WORDS_CARE, NO_WORDS_ASK);
pools!(NO_WORDS_ASKING, NO_WORDS_AT_THE_CARE, NO_WORDS_SOMEONE, NO_WORDS_STUCK);

/// Python: bot/compose/fallback.py:5336 `_no_words(clean)`
/// The pool for a line that cannot find the words, or None. (The skeleton's `()` return could
/// not carry the pool.)
pub fn _no_words(clean: &str) -> Option<Vec<String>> {
    if !NO_WORDS.is_match(clean) || NO_WORDS_NOT.is_match(clean) {
        return None;
    }
    if NO_WORDS_ASK.is_match(clean) {
        return Some(NO_WORDS_ASKING.clone());
    }
    if NO_WORDS_CARE.is_match(clean) {
        return Some(NO_WORDS_AT_THE_CARE.clone());
    }
    if NO_WORDS_TO.is_match(clean) || NO_WORDS_SHE.is_match(clean) {
        if NO_WORDS_CROWD.is_match(clean) && !NO_WORDS_ONE.is_match(clean) {
            return None;
        }
        return Some(NO_WORDS_SOMEONE.clone());
    }
    Some(NO_WORDS_STUCK.clone())
}

regexes!(INVITE_TALK, INVITE_OPENER, INVITE_COMPANY, INVITE_NOT_A_DOING, INVITE_WHEN);
word_sets!(INVITE_NEVER, INVITE_FILLER, INVITE_PRONOUN, INVITE_ACTIVITY);

/// Python: bot/compose/fallback.py:5377 `_an_invitation(clean)`
/// 'talk', 'go' or None: which invitation this line is, if it is one. (The skeleton's `()`
/// return could not carry the answer.)
pub fn _an_invitation(clean: &str) -> Option<String> {
    let line = text::squash(&rx("[^a-z0-9 ']+").sub(" ", &lower(clean)));
    let split = text::split_ref(&line);
    if line.is_empty() || split.len() < 2 {
        return None;
    }
    if split.iter().any(|w| INVITE_NEVER.contains(*w)) {
        return None;
    }
    let talk_or_go = || Some(if INVITE_TALK.is_match(&line) { "talk".to_string() } else { "go".to_string() });
    if INVITE_COMPANY.match_(&line).is_some() {
        return talk_or_go();
    }
    let found = INVITE_OPENER.match_(&line)?;
    let rest = text::split_ref(found.str(1));
    let mut tail: Vec<&str> = rest.iter().copied().filter(|w| !INVITE_FILLER.contains(*w)).collect();
    if tail.is_empty() {
        tail = rest.iter().copied().filter(|w| !["tonight", "today", "tomorrow", "later", "now", "soon", "after"].contains(w)).collect();
    }
    let said = tail.join(" ");
    if tail.is_empty() || INVITE_NOT_A_DOING.match_(&said).is_some() || INVITE_WHEN.match_(&said).is_some() {
        return None;
    }
    if tail.iter().all(|w| INVITE_PRONOUN.contains(*w)) {
        return None;
    }
    if !tail.iter().any(|w| INVITE_ACTIVITY.contains(*w)) && !INVITE_TALK.is_match(&line) {
        return None;
    }
    talk_or_go()
}

regexes!(BOUND_SAID, NARROW_JOIN, A_FAULT);
static NARROW_AXES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.NARROW_AXES")
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().unwrap()))
        .collect()
});
static NARROW_WORDS: LazyLock<i64> = LazyLock::new(|| data::get("bot.compose.fallback.NARROW_WORDS").as_int().unwrap());
static NARROW_SUBSTANCE: LazyLock<i64> = LazyLock::new(|| data::get("bot.compose.fallback.NARROW_SUBSTANCE").as_int().unwrap());
word_sets!(NARROW_ANSWERED);
static AXIS_SAID: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.AXIS_SAID").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().unwrap())).collect()
});
static AXIS_MIN: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.AXIS_MIN").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect()
});

/// Python: bot/compose/fallback.py:5449 `the_bound_said(clean)` (never called on the trace corpus)
/// The bound the line itself names, said back the way they said it, or "".
pub fn the_bound_said(clean: &str) -> String {
    let line = text::squash(text::strip(clean));
    let Some(found) = BOUND_SAID.search(&line) else { return String::new() };
    let rest = &line[found.start()..];
    let rest = rx(r"[^\w\s£$€.,-]").sub(" ", rest);
    let rest = rest.trim_matches([' ', '.', ',']);
    let words: Vec<&str> = text::split_ref(rest).into_iter().take(5).collect();
    words.join(" ").trim_end_matches([' ', '.', ',']).to_string()
}

/// Python: bot/compose/fallback.py:5461 `reads_as_a_narrowing(clean)`
/// The axis this line narrows on, or "" when it is not only a constraint.
pub fn reads_as_a_narrowing(clean: &str) -> String {
    the_limits_said(clean).into_iter().next().map(|(axis, _)| axis).unwrap_or_default()
}

/// Python: bot/compose/fallback.py:5484 `the_limits_said(clean)`
/// The limits a line of joined limits names, as `(axis, part)` pairs in the order a reply should
/// try them. (The skeleton's `Vec<String>` could not carry the pairs Python returns.)
pub fn the_limits_said(clean: &str) -> Vec<(String, String)> {
    let line = text::squash(text::strip(clean));
    if line.is_empty() || text::rstrip(&line).ends_with('?') || text::split_ref(&line).len() as i64 > *NARROW_WORDS {
        return Vec::new();
    }
    let parts: Vec<String> = NARROW_JOIN.split(&line).into_iter().filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Vec::new();
    }
    let axes: Vec<String> = parts.iter().map(|p| _one_axis(p)).collect();
    if axes.iter().any(|a| a.is_empty()) {
        return Vec::new();
    }
    let mut with_bound = Vec::new();
    let mut without = Vec::new();
    for (axis, part) in axes.into_iter().zip(parts) {
        if the_bound_said(&part).is_empty() {
            without.push((axis, part));
        } else {
            with_bound.push((axis, part));
        }
    }
    with_bound.reverse();
    without.reverse();
    with_bound.extend(without);
    with_bound
}

/// Python: bot/compose/fallback.py:5514 `_one_axis(line)`
/// The one axis a piece of a line narrows on, or "".
pub fn _one_axis(line: &str) -> String {
    for (axis, pattern) in NARROW_AXES.iter() {
        if pattern.match_(line).is_some() {
            return axis.clone();
        }
    }
    String::new()
}

/// Python's `str.isupper()` for one character.
fn char_is_upper(c: char) -> bool {
    c.is_uppercase()
}

/// Python: bot/compose/fallback.py:5545 `_names_things_to_choose(said)` (never called on the trace corpus)
/// True when the answer puts two or more named things in front of the person, to pick from.
pub fn _names_things_to_choose(said: &str) -> bool {
    if !rx_flags_i(r"\bor\b").is_match(said) {
        return false;
    }
    let words = text::split_ref(said);
    let mut named: HashSet<&str> = HashSet::new();
    for (index, word) in words.iter().enumerate() {
        let bare = word.trim_matches(['.', ',', ';', ':', '!', '?', '*', '(', ')', '\u{2014}', '-', '\u{2019}', '\'', '"']);
        let mut chars = bare.chars();
        let Some(first) = chars.next() else { continue };
        let rest = chars.as_str();
        if bare.chars().count() < 2 || !char_is_upper(first) || !text::is_lower(rest) {
            continue;
        }
        if index == 0 {
            continue;
        }
        let before = text::rstrip(words[index - 1]);
        if before.ends_with(['.', '!', '?', ':']) && !before.ends_with("...") {
            continue;
        }
        named.insert(bare);
    }
    named.len() >= 2
}

/// `re.compile(pattern, re.I)` through the shared cache.
fn rx_flags_i(pattern: &str) -> &'static Re {
    crate::understanding::rx_flags(pattern, crate::py::re::I)
}

/// Python: bot/compose/fallback.py:5596 `the_answer_to_cut(memory, axis='')` (never called on the trace corpus)
/// The answer still on the screen (the last turn, as a dict of user, bot and module), or {}.
pub fn the_answer_to_cut(memory: &Memory, axis: &Value) -> Value {
    match answer_to_cut(memory, axis.as_str().unwrap_or("")) {
        Some(turn) => {
            let mut d = Value::dict();
            d.set_key("user", turn.user.as_str());
            d.set_key("bot", turn.bot.as_str());
            d.set_key("module", turn.module.as_deref().map(Value::from).unwrap_or(Value::None));
            d
        }
        None => Value::dict(),
    }
}

/// `the_answer_to_cut` as the turn itself (None for Python's `{}`).
pub fn answer_to_cut<'m>(memory: &'m Memory, axis: &str) -> Option<&'m Turn> {
    let last = memory.turns.last()?;
    if !last.module.as_deref().is_some_and(|m| NARROW_ANSWERED.contains(m)) {
        return None;
    }
    let said = &last.bot;
    if (text::split_ref(said).len() as i64) < *NARROW_SUBSTANCE {
        return None;
    }
    if text::rstrip(said).ends_with('?') {
        return None;
    }
    if !axis.is_empty() {
        let raised = match AXIS_SAID.iter().find(|(k, _)| k == axis) {
            Some((_, rx_)) => rx_.finditer(said).len() as i64,
            None => 0,
        };
        let min = AXIS_MIN.iter().find(|(k, _)| k == axis).map(|(_, n)| *n).unwrap_or(1);
        if raised < min && !_names_things_to_choose(said) {
            return None;
        }
    }
    if A_FAULT.is_match(&last.user) {
        return None;
    }
    Some(last)
}

// ---- fallback.py:5683-6127: reactions and verdicts on the thing before ----

/// `" ".join(str(turn.get("user") or "").lower().split())` for the person's last `n` lines.
fn last_lines(memory: &Memory, n: i64) -> Vec<String> {
    last_n(&memory.turns, n).iter().map(|t| text::squash(&lower(&t.user))).collect()
}

/// Python: bot/compose/fallback.py:5683 `_the_person_already_said_it(memory)` (never called on the trace corpus)
pub fn _the_person_already_said_it(memory: &Memory) -> bool {
    let last = crate::memory::last_user(memory, 1);
    let said = text::strip(last.as_str().unwrap_or(""));
    if said.is_empty() || text::rstrip(said).ends_with('?') {
        return false;
    }
    text::split_ref(said).len() >= 4
}

/// Python: bot/compose/fallback.py:5697 `_a_care_thread_to_leave_alone(memory)`
pub fn _a_care_thread_to_leave_alone(memory: &Memory) -> bool {
    if _in_a_care_thread(memory) || _in_sudden_loss_thread(memory) || _in_a_loss_thread(memory) {
        return true;
    }
    _the_last_few_lines(memory).iter().any(|said| baby_loss(said))
}

/// Python: bot/compose/fallback.py:5711 `_the_last_few_lines(memory)`
/// The person's own last few lines, as the thread left them.
pub fn _the_last_few_lines(memory: &Memory) -> Vec<String> {
    last_n(&memory.turns, 4).iter().filter(|t| !text::strip(&t.user).is_empty()).map(|t| text::squash(&lower(&t.user))).collect()
}

/// Python: bot/compose/fallback.py:5718 `baby_loss(clean)`
pub fn baby_loss(clean: &str) -> bool {
    BABY_LOSS.is_match(clean)
}

regexes!(VERDICT_SAID, VERDICT_BARE, JOKE_WANTED, JOKE_UNWANTED);
pools!(REACTION_OUCH);

/// Python: bot/compose/fallback.py:5756 `_the_answer_above_was_a_joke(memory)` (never called on the trace corpus)
pub fn _the_answer_above_was_a_joke(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let asked = text::squash(&lower(&last.user));
    if JOKE_UNWANTED.is_match(&asked) {
        return false;
    }
    JOKE_WANTED.is_match(&asked)
}

/// Python: bot/compose/fallback.py:5767 `_a_verdict_line(text, memory=None)`
/// The pool for a line that is only a verdict on the thing before it, or None. (The skeleton's
/// `()` return could not carry the pool.)
pub fn _a_verdict_line(text_: &str, memory: &Memory) -> Option<Vec<String>> {
    let t = text::squash(&lower(text_));
    let found = VERDICT_SAID.match_(&t);
    if found.is_none() && VERDICT_BARE.match_(&t).is_none() {
        return None;
    }
    if let Some(found) = &found {
        if found.name("noun") == Some("joke") && _the_answer_above_was_a_joke(memory) {
            return None;
        }
    }
    Some(REACTION_OUCH.clone())
}

regexes!(VERDICT_LINE, STATE_ABOVE, WAITING_NEWS, JOB_GONE_NEWS, GRIEF_ABOVE, VERDICT_POINTS_BACK, HEAVY_NEWS, HARD_WEEK);

/// Python: bot/compose/fallback.py:5807 `a_death_just_said(memory=None)` (never called on the trace corpus)
pub fn a_death_just_said(memory: &Memory) -> bool {
    for line in last_lines(memory, 2) {
        if line.is_empty() {
            continue;
        }
        if py_try(|| crate::compose::loss::reads_as_loss(&line, None, false)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:5825 `a_verdict_on_the_person(memory=None)` (never called on the trace corpus)
pub fn a_verdict_on_the_person(memory: &Memory) -> bool {
    last_lines(memory, 2).iter().any(|l| !l.is_empty() && STATE_ABOVE.is_match(l))
}

/// Python: bot/compose/fallback.py:5848 `_the_last_two_lines(memory=None)` (never called on the trace corpus)
/// The person's own lines for this turn and the one before - never the bot's wording.
pub fn _the_last_two_lines(memory: &Memory) -> Vec<String> {
    last_lines(memory, 2).into_iter().filter(|l| !l.is_empty()).collect()
}

/// Python: bot/compose/fallback.py:5858 `_a_loss_in_the_same_breath(memory=None, lines=())` (never called on the trace corpus)
pub fn _a_loss_in_the_same_breath(memory: &Memory, lines: &[String]) -> bool {
    if a_death_just_said(memory) || a_heavy_verdict(memory) {
        return true;
    }
    for line in lines {
        if HEAVY_ABOVE.is_match(line) {
            return true;
        }
        let heavy = py_try(|| {
            crate::compose::badnews::reads_as_sudden_loss(line)
                || crate::compose::badnews::reads_as_the_leaving(line)
                || crate::compose::loss::reads_as_loss(line, None, false)
        });
        if heavy == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:5879 `a_wait_with_no_loss(memory=None)` (never called on the trace corpus)
pub fn a_wait_with_no_loss(memory: &Memory) -> bool {
    let lines = _the_last_two_lines(memory);
    if !lines.iter().any(|l| WAITING_NEWS.is_match(l)) {
        return false;
    }
    !_a_loss_in_the_same_breath(memory, &lines)
}

/// Python: bot/compose/fallback.py:5887 `a_setback_with_no_loss(memory=None)` (never called on the trace corpus)
pub fn a_setback_with_no_loss(memory: &Memory) -> bool {
    let lines = _the_last_two_lines(memory);
    if !lines.iter().any(|l| JOB_GONE_NEWS.is_match(l)) {
        return false;
    }
    !_a_loss_in_the_same_breath(memory, &lines)
}

pools!(VERDICT_NUMB, VERDICT_QUEUE, VERDICT_SETBACK, VERDICT_HEAVY);

/// Python: bot/compose/fallback.py:5895 `_the_heavy_pool(memory=None)` (never called on the trace corpus)
/// The pool for a verdict on news with no repair in it, as a tuple value.
pub fn _the_heavy_pool(memory: &Memory) -> Value {
    Value::from(the_heavy_pool(memory))
}

/// `_the_heavy_pool` as lines.
pub fn the_heavy_pool(memory: &Memory) -> Vec<String> {
    if a_death_just_said(memory) {
        return VERDICT_NUMB.clone();
    }
    if a_wait_with_no_loss(memory) {
        return VERDICT_QUEUE.clone();
    }
    if a_setback_with_no_loss(memory) {
        return VERDICT_SETBACK.clone();
    }
    VERDICT_HEAVY.clone()
}

/// Python: bot/compose/fallback.py:5917 `a_heavy_verdict(memory=None)` (never called on the trace corpus)
pub fn a_heavy_verdict(memory: &Memory) -> bool {
    last_lines(memory, 2).iter().any(|l| !l.is_empty() && GRIEF_ABOVE.is_match(l))
}

/// Python: bot/compose/fallback.py:5940 `a_happening_now(memory=None)` (never called on the trace corpus)
pub fn a_happening_now(memory: &Memory) -> bool {
    last_lines(memory, 2).iter().any(|l| !l.is_empty() && HAPPENING_NOW.is_match(l))
}

/// Python: bot/compose/fallback.py:5959 `_a_verdict_on_something(text, memory)` (never called on the trace corpus)
pub fn _a_verdict_on_something(text_: &str, memory: &Memory) -> bool {
    if VERDICT_POINTS_BACK.match_(text_).is_some() {
        return true;
    }
    !memory.turns.is_empty()
}

pools!(VERDICT_SHAME, VERDICT_LUCK, VERDICT_ROUGH, VERDICT_ANNOY);

/// Python: bot/compose/fallback.py:5976 `_verdict_pool(clean, memory=None)`
/// The pool for a verdict on the news, or None when the line is anything more. (The skeleton's
/// `()` return could not carry the pool.)
pub fn _verdict_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let t = text::squash(&lower(clean));
    if t.is_empty() || VERDICT_LINE.match_(&t).is_none() || t.contains('?') {
        return None;
    }
    if !_a_verdict_on_something(&t, memory) {
        return None;
    }
    if memory.turns.is_empty() {
        return None;
    }
    if rx(r"\b(?:shame|letdown)\b").is_match(&t) {
        return Some(VERDICT_SHAME.clone());
    }
    if rx(r"\b(?:typical|figures|luck|law|of course)\b").is_match(&t) {
        return Some(VERDICT_LUCK.clone());
    }
    if a_verdict_on_the_person(memory) {
        return Some(VERDICT_ROUGH.clone());
    }
    Some(VERDICT_ANNOY.clone())
}

/// Python: bot/compose/fallback.py:6001 `a_heavy_news(memory=None)` (never called on the trace corpus)
pub fn a_heavy_news(memory: &Memory) -> bool {
    for line in last_lines(memory, 2) {
        if line.is_empty() {
            continue;
        }
        if HEAVY_NEWS.is_match(&line) {
            return true;
        }
        if py_try(|| crate::compose::badnews::reads_as_the_leaving(&line)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:6021 `a_loss_in_the_thread(memory=None)` (never called on the trace corpus)
pub fn a_loss_in_the_thread(memory: &Memory) -> bool {
    if a_heavy_verdict(memory) || a_heavy_news(memory) {
        return true;
    }
    match py_try(|| _in_a_loss_thread(memory) || _grief_thread(memory)) {
        Some(true) => return true,
        Some(false) => {}
        None => return false,
    }
    for line in last_lines(memory, 2) {
        if line.is_empty() {
            continue;
        }
        if HEAVY_ABOVE.is_match(&line) {
            return true;
        }
        if py_try(|| crate::compose::badnews::reads_as_sudden_loss(&line)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:6052 `a_verdict_stands_down(memory=None)`
/// (Python's `if not memory` is never true of a turn's memory, which always holds its turns.)
pub fn a_verdict_stands_down(memory: &Memory) -> bool {
    match py_try(|| in_a_safety_thread(memory) || _in_a_loss_thread(memory) || _grief_thread(memory)) {
        Some(true) => return true,
        Some(false) => {}
        None => return false,
    }
    for line in last_lines(memory, 2) {
        if line.is_empty() {
            continue;
        }
        if HARD_WEEK.is_match(&line) || HEAVY_ABOVE.is_match(&line) {
            return true;
        }
        if py_try(|| crate::compose::badnews::reads_as_sudden_loss(&line)) == Some(true) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:6093 `reads_as_a_verdict(text, memory=None)`
pub fn reads_as_a_verdict(text_: &str, memory: &Memory) -> bool {
    py_try(|| _verdict_pool(text_, memory).is_some()).unwrap_or(false)
}

regexes!(FIRST_TURN_VERDICT, REACTION_LINE);
pools!(VERDICT_FIRST, REACTION_REPEAT, REACTION_REPEAT_FIRST, REACTIONS_CARE);
static REACTION_SIGNS: LazyLock<Vec<(Vec<String>, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.REACTION_SIGNS").items().iter().map(|p| (p.items()[0].strs(), p.items()[1].as_re().unwrap())).collect()
});
static REACTIONS_SAID: LazyLock<Vec<(Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.REACTIONS_SAID").pairs().iter().map(|(k, v)| (k.strs(), v.strs())).collect()
});

/// Python: bot/compose/fallback.py:6122 `_reaction_pool(clean, memory=None)`
/// The pool for a bare reaction, or None when the line is anything more than one. (The
/// skeleton's `()` return could not carry the pool.)
pub fn _reaction_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let t = text::squash(&lower(clean));
    if let Some(verdict) = _a_verdict_line(&t, memory) {
        if !verdict.is_empty() {
            return Some(verdict);
        }
    }
    if FIRST_TURN_VERDICT.match_(&t).is_some() && memory.turns.is_empty() {
        return Some(VERDICT_FIRST.clone());
    }
    if t.is_empty() || REACTION_LINE.match_(&t).is_none() {
        return None;
    }
    for (pool, sign) in REACTION_SIGNS.iter() {
        if sign.is_match(&t) {
            return Some(pool.clone());
        }
    }
    if rx(r"[\w']*\s*(?:what|pardon)\W*").fullmatch(&t).is_some() {
        if crate::memory::last_bot(memory).is_empty() {
            return Some(REACTION_REPEAT_FIRST.clone());
        }
        return Some(REACTION_REPEAT.clone());
    }
    None
}

/// Python: bot/compose/fallback.py:6148 `reaction_line(text, memory=None)` (never called on the trace corpus)
/// The composer's own line for a verdict on the thing before it, or None.
pub fn reaction_line(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&lower(text_));
    let mut pool = _a_verdict_line(&clean, memory)?;
    if pool.is_empty() {
        return None;
    }
    if _a_care_thread_to_leave_alone(memory) {
        pool = REACTIONS_CARE.clone();
    } else if _the_person_already_said_it(memory) {
        if let Some((_, said)) = REACTIONS_SAID.iter().find(|(k, _)| *k == pool) {
            pool = said.clone();
        }
    }
    choose(&pool, memory, &clean)
}

// ---- fallback.py:6250-6505: a thing done ----

/// Python truthiness of what another module's function returns, whatever type its port gives
/// it (a reader whose skeleton says `()` is still a stub, and never returns).
pub trait Truthy {
    fn truthy(&self) -> bool;
}
impl Truthy for () {
    fn truthy(&self) -> bool {
        false
    }
}
impl Truthy for bool {
    fn truthy(&self) -> bool {
        *self
    }
}
impl Truthy for String {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
}
impl Truthy for &str {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
}
impl<T> Truthy for Vec<T> {
    fn truthy(&self) -> bool {
        !self.is_empty()
    }
}
impl<T: Truthy> Truthy for Option<T> {
    fn truthy(&self) -> bool {
        self.as_ref().is_some_and(Truthy::truthy)
    }
}
impl Truthy for Value {
    fn truthy(&self) -> bool {
        Value::truthy(self)
    }
}

pools!(A_THING_ON, A_THING_AHEAD, A_THING_GONE, A_THING_SEEN_TO, A_THING_BOUGHT, A_THING_DONE);
word_sets!(GONE_VERB, BOUGHT_VERB, ALIVE, NOT_CHOSEN, SEEN_TO_VERB);

/// Python: bot/compose/fallback.py:6247 `_a_thing_done(clean)` (never called on the trace corpus)
pub fn _a_thing_done(clean: &str) -> bool {
    _thing_done_shape(clean).is_some()
}

/// Python: bot/compose/fallback.py:6252 `_thing_done_shape(clean)` (never called on the trace corpus)
/// Just the shape: see `_thing_done_scan`.
pub fn _thing_done_shape(clean: &str) -> Option<String> {
    thing_done_scan(clean).0
}

/// Python: bot/compose/fallback.py:6257 `_thing_done_pool(clean)`
/// The pool for a thing done, or None when the line is not one.
pub fn _thing_done_pool(clean: &str) -> Option<Vec<String>> {
    if crate::compose::fallback_chain::_a_thing_they_regret(clean) {
        return None;
    }
    let (shape, done_verb) = thing_done_scan(clean);
    let shape = shape?;
    if shape.is_empty() {
        return None;
    }
    if shape == "going" {
        return Some(A_THING_ON.clone());
    }
    if shape == "soon" || shape == "at" {
        return Some(A_THING_AHEAD.clone());
    }
    if GONE_VERB.contains(&done_verb) {
        return Some(A_THING_GONE.clone());
    }
    if BOUGHT_VERB.contains(&done_verb) {
        let split = text::split_ref(clean);
        if split.iter().any(|w| ALIVE.contains(*w)) {
            return None;
        }
        if split.iter().any(|w| NOT_CHOSEN.contains(*w)) {
            return Some(A_THING_SEEN_TO.clone());
        }
        return Some(A_THING_BOUGHT.clone());
    }
    if SEEN_TO_VERB.contains(&done_verb) {
        return Some(A_THING_SEEN_TO.clone());
    }
    Some(A_THING_DONE.clone())
}

regexes!(
    HARM_AFTER,
    TAIL_JOIN,
    TAIL_COMPLAINT,
    TAIL_ASK,
    TAIL_GRUMBLE,
    TAIL_TROUBLE,
    NEGATIVE_LINE,
    HARM_NAMED,
    HARM_ON_ME,
    FEELING_OFF,
    A_TROUBLE_IN_THE_TAIL,
    HARM_ANYWHERE,
    WENT_AT_IT,
    DID_AT_IT,
    SPENT_TIME,
    BEEN_AT_IT,
    TIME_SPAN,
    AT_IT,
    HABIT_TIME,
    FUTURE_TIME,
);
word_sets!(
    QUESTION_OPENERS,
    DONE_VERB,
    DONE_NOT_THIS,
    DONE_ONLY_A_THING,
    DONE_NOT_A_PERSON,
    DONE_ING,
    AT_IT_EXTRA,
    BECOMING,
    BECOMING_STATE,
    AT_IT_NOT,
    THING_AFTER,
);

/// Python: bot/compose/fallback.py:6305 `_tail_complains(clean)`
pub fn _tail_complains(clean: &str) -> bool {
    match TAIL_JOIN.search(clean) {
        Some(first) => TAIL_COMPLAINT.is_match(&clean[first.end()..]),
        None => false,
    }
}

/// Python: bot/compose/fallback.py:6319 `_thing_done_scan(clean)`
/// `(shape, verb)` for this line, or `(None, "")` when it is not a thing done, as a tuple value.
pub fn _thing_done_scan(clean: &str) -> Value {
    let (shape, verb) = thing_done_scan(clean);
    Value::Tuple(std::sync::Arc::new(vec![shape.map(Value::from).unwrap_or(Value::None), Value::from(verb)]))
}

/// `_thing_done_scan` as a pair.
pub fn thing_done_scan(clean: &str) -> (Option<String>, String) {
    let none = (None, String::new());
    if clean.contains('?')
        || clean.contains(" but ")
        || TAIL_ASK.is_match(clean)
        || TAIL_GRUMBLE.is_match(clean)
        || TAIL_TROUBLE.is_match(clean)
        || _tail_complains(clean)
    {
        return none;
    }
    if QUESTION_OPENERS.contains(text::split_ref(clean).first().copied().unwrap_or("")) {
        return none;
    }
    if NEGATIVE_LINE.is_match(clean) {
        return none;
    }
    let tokens = az_words(clean);
    if crate::compose::trouble::reads_as_trouble(clean, Some(&tokens[..]), clean).truthy() {
        return none;
    }
    if HARM_AFTER.is_match(clean) {
        return none;
    }
    if HARM_NAMED.is_match(clean) || HARM_ON_ME.is_match(clean) {
        return none;
    }
    if FEELING_OFF.is_match(clean) || A_TROUBLE_IN_THE_TAIL.is_match(clean) {
        return none;
    }
    let t = rx(r"\b(?:ive|i've)\b").sub("i have", clean);
    let t = rx(r"\b(?:weve|we've)\b").sub("we have", &t);
    let t = rx(r"\b(i|we)'d\b").sub(r"\1 had", &t);
    let words = az_words(&t);
    for i in 0..words.len().saturating_sub(1) {
        if words[i] != "i" && words[i] != "we" {
            continue;
        }
        let mut j = i + 1;
        while j < words.len() && ["just", "also", "finally", "only", "actually", "have", "had", "been"].contains(&words[j].as_str()) {
            j += 1;
        }
        if j >= words.len() || !DONE_VERB.contains(&words[j]) {
            continue;
        }
        let rest = &words[j + 1..(j + 3).min(words.len())];
        if rest.iter().any(|w| DONE_NOT_THIS.contains(w)) {
            return none;
        }
        if DONE_ONLY_A_THING.contains(&words[j]) && words[(j + 1).min(words.len())..(j + 4).min(words.len())].iter().any(|w| DONE_NOT_A_PERSON.contains(w)) {
            return none;
        }
        return (Some("done".into()), words[j].clone());
    }
    if !words.is_empty() && DONE_VERB.contains(&words[0]) && !HARM_ANYWHERE.is_match(clean) {
        let rest = &words[1..3.min(words.len())];
        if !rest.iter().any(|w| DONE_NOT_THIS.contains(w)) {
            return (Some("done".into()), words[0].clone());
        }
    }
    if WENT_AT_IT.is_match(&t) {
        return (Some("gone".into()), "went".into());
    }
    if DID_AT_IT.is_match(&t) {
        return (Some("gone".into()), "did".into());
    }
    if let Some(spent) = SPENT_TIME.search(&t) {
        if DONE_ING.contains(spent.str(1)) {
            return (Some("going".into()), spent.str(1).to_string());
        }
    }
    if let Some(been) = BEEN_AT_IT.search(&t) {
        if DONE_ING.contains(been.str(1)) && TIME_SPAN.is_match(&t) {
            return (Some("going".into()), been.str(1).to_string());
        }
    }
    if let Some(at_it) = AT_IT.search(&t) {
        let verb = at_it.str(1);
        if DONE_ING.contains(verb) || AT_IT_EXTRA.contains(verb) {
            let rest = &t[at_it.end()..];
            if BECOMING.contains(verb) && BECOMING_STATE.contains(text::split_ref(rest).first().copied().unwrap_or("")) {
                return none;
            }
            if _a_thing_after(rest) && !HABIT_TIME.is_match(rest) && !az_words(rest).iter().any(|w| AT_IT_NOT.contains(w)) {
                let shape = if FUTURE_TIME.is_match(rest) { "soon" } else { "at" };
                return (Some(shape.into()), verb.to_string());
            }
        }
    }
    none
}

/// Python: bot/compose/fallback.py:6477 `_a_thing_after(rest)`
/// True when the words after the verb name the thing the person is working on.
pub fn _a_thing_after(rest: &str) -> bool {
    az_words(rest).iter().any(|w| !THING_AFTER.contains(w) && w.chars().count() > 2)
}

/// Python: bot/compose/fallback.py:6494 `_done_ing()` (never called on the trace corpus)
/// The "-ing" forms of DONE_VERB (the module keeps the result as `DONE_ING`).
pub fn _done_ing() -> HashSet<String> {
    let mut out = HashSet::new();
    for verb in DONE_VERB.iter() {
        let stem: &str = if verb.ends_with("ed") { &verb[..verb.len() - 2] } else { verb };
        out.insert(format!("{stem}ing"));
        out.insert(format!("{verb}ing"));
        if stem.chars().count() > 2 {
            let last = stem.chars().last().unwrap();
            out.insert(format!("{stem}{last}ing"));
        }
    }
    for w in [
        "running", "swimming", "making", "writing", "cutting", "digging", "hanging", "building", "sweeping", "freezing", "sitting", "getting",
        "decorating", "repainting", "reorganising", "reorganizing",
    ] {
        out.insert(w.to_string());
    }
    out
}

// ---- fallback.py:6568-7130: knocks and mishaps, dislikes, own feeling, summons, promises, doable ----

/// What another module's reader returns, as Python's `str | None`, whatever type its port gives
/// it (a reader whose skeleton says `()` is still a stub, and never returns).
pub trait PyOptStr {
    fn py_opt_str(self) -> Option<String>;
}
impl PyOptStr for () {
    fn py_opt_str(self) -> Option<String> {
        None
    }
}
impl PyOptStr for String {
    fn py_opt_str(self) -> Option<String> {
        Some(self)
    }
}
impl PyOptStr for Option<String> {
    fn py_opt_str(self) -> Option<String> {
        self
    }
}
impl PyOptStr for Value {
    fn py_opt_str(self) -> Option<String> {
        self.as_str().map(str::to_string)
    }
}

regexes!(KNOCK_ON_ME, A_SPAN_OF_TIME, WALKED_INTO, MISHAP_FELL, MISHAP_UNDERFOOT, SENSE_GONE, MISHAP_ANYWHERE, MISHAP_OVER);

/// Python: bot/compose/fallback.py:6568 `_a_knock_on_oneself(clean)`
pub fn _a_knock_on_oneself(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') || clean.contains(" but ") {
        return false;
    }
    if !KNOCK_ON_ME.is_match(clean) {
        return false;
    }
    !A_SPAN_OF_TIME.is_match(clean)
}

/// Python: bot/compose/fallback.py:6591 `_a_knock_into_a_thing(clean)`
pub fn _a_knock_into_a_thing(clean: &str) -> bool {
    if clean.is_empty() || clean.contains('?') || clean.contains(" but ") {
        return false;
    }
    WALKED_INTO.is_match(clean)
}

/// Python: bot/compose/fallback.py:6691 `_a_mishap_to_ask_about(text)`
pub fn _a_mishap_to_ask_about(text_: &str) -> bool {
    if SENSE_GONE.is_match(text_) {
        return true;
    }
    if !MISHAP_ANYWHERE.is_match(text_) {
        return false;
    }
    !MISHAP_OVER.is_match(text_)
}

/// Python: bot/compose/fallback.py:6707 `_a_mishap(clean, words)`
pub fn _a_mishap(clean: &str, words: &[String]) -> bool {
    if clean.is_empty() || clean.contains('?') || words.len() > 24 {
        return false;
    }
    if MISHAP_OVER.is_match(clean) {
        return false;
    }
    MISHAP_FELL.is_match(clean) || MISHAP_UNDERFOOT.is_match(clean)
}

regexes!(NOT_ENOUGH, NO_DIRECTION, NOT_SELF, DISLIKE_HAS_TWO_HALVES, DISLIKE_TAIL, DISLIKE_WITHOUT_AN_I);
pools!(DISLIKE_ASKS, NOT_ENOUGH_REPLY, NO_DIRECTION_REPLY);
word_sets!(NOT_A_THING_ASKED);
static DISLIKE_TOO_LONG: LazyLock<usize> = LazyLock::new(|| data::get("bot.compose.fallback.DISLIKE_TOO_LONG").as_int().unwrap() as usize);

/// `[frame.format(thing=thing, Thing=thing[:1].upper() + thing[1:]) for frame in DISLIKE_ASKS]`.
fn dislike_asks(thing: &str) -> Vec<String> {
    let cap = format!("{}{}", text::upper(&text::slice(thing, 0, Some(1))), text::slice(thing, 1, None));
    DISLIKE_ASKS.iter().map(|f| format_named(f, &[("thing", thing), ("Thing", &cap)])).collect()
}

/// Python: bot/compose/fallback.py:6831 `the_dislike_ask(clean, memory)`
/// A question about the thing the person has just said they do not like, or None. (The
/// skeleton's `()` return could not carry the question.)
pub fn the_dislike_ask(clean: &str, memory: &Memory) -> Option<String> {
    let mut thing = crate::memory::obstacle::the_thing(clean).py_opt_str().unwrap_or_default();
    let strip_ends = |s: &str| s.trim_matches([' ', ',', '.', ';', ':', '-']).to_string();
    if thing.is_empty() {
        if let Some(found) = DISLIKE_WITHOUT_AN_I.match_(&text::squash(clean)) {
            let mut t = strip_ends(found.name_str("thing"));
            if t.is_empty() || NOT_A_THING_ASKED.contains(&lower(&t)) {
                return None;
            }
            t = strip_ends(&DISLIKE_TAIL.sub("", &t));
            if t.is_empty() || text::split_ref(&t).len() > *DISLIKE_TOO_LONG {
                return None;
            }
            if DISLIKE_HAS_TWO_HALVES.is_match(&t) {
                return None;
            }
            let t = _yours(&t);
            return choose(&dislike_asks(&t), memory, clean);
        }
    }
    if !thing.is_empty() {
        thing = _yours(&strip_ends(&DISLIKE_TAIL.sub("", &thing)));
    }
    if thing.is_empty() || text::split_ref(&thing).len() > *DISLIKE_TOO_LONG {
        return None;
    }
    if DISLIKE_HAS_TWO_HALVES.is_match(&thing) {
        return None;
    }
    choose(&dislike_asks(&thing), memory, clean)
}

/// Python: bot/compose/fallback.py:6862 `_own_feeling(clean)`
/// The pool for a statement about the speaker's own worth or direction, or None. (The
/// skeleton's `()` return could not carry the pool.)
pub fn _own_feeling(clean: &str) -> Option<Vec<String>> {
    if text::rstrip(clean).ends_with('?') || NOT_SELF.is_match(clean) {
        return None;
    }
    if NOT_ENOUGH.is_match(clean) {
        return Some(NOT_ENOUGH_REPLY.clone());
    }
    if NO_DIRECTION.is_match(clean) {
        return Some(NO_DIRECTION_REPLY.clone());
    }
    None
}

regexes!(SUMMONS_ASKS_A_CALL);
pools!(SUMMONS_RANG, SUMMONS_WRITTEN, SUMMONS_SEEN);

/// Python: bot/compose/fallback.py:6882 `_summons_pool(summons, line='')` (never called on the trace corpus)
/// The lines for a summons, by shape: a meeting, a letter, or a phone call (as a tuple value).
/// `summons` is the reading's `summons` dict.
pub fn _summons_pool(summons: &Value, line: &str) -> Value {
    let shape = if summons.is_dict() { summons.get("shape").and_then(Value::as_str) } else { None };
    Value::from(summons_pool(shape, line))
}

/// `_summons_pool` by the summons' shape.
pub fn summons_pool(shape: Option<&str>, line: &str) -> Vec<String> {
    match shape {
        Some("written") => {
            if SUMMONS_ASKS_A_CALL.is_match(line) {
                SUMMONS_RANG.clone()
            } else {
                SUMMONS_WRITTEN.clone()
            }
        }
        Some("rang") => SUMMONS_RANG.clone(),
        _ => SUMMONS_SEEN.clone(),
    }
}

regexes!(
    PROMISE_OFFICE,
    PROMISE_A_PERSON_NAMED,
    PROMISE_REPORTED,
    PROMISE_FUTURE,
    PROMISE_BUT_NEVER,
    PROMISE_NEVER_WHAT,
    PROMISE_RISK,
    PROMISE_RECUR,
    PROMISE_NOT_A_PERSON,
    PROMISE_MONEY,
    PROMISE_A_HE_OR_SHE,
);
pools!(PROMISE_A_PERSON, PROMISE_AN_OFFICE);
word_sets!(GRIEVANCE_HEALTH, GRIEVANCE_HARM);

/// Python: bot/compose/fallback.py:7011 `_a_promise_that_keeps_moving(clean)`
/// The reply pool for a promise that keeps not happening, or None. (The skeleton's `()` return
/// could not carry the pool.)
pub fn _a_promise_that_keeps_moving(clean: &str) -> Option<Vec<String>> {
    let reported = PROMISE_REPORTED.search(clean);
    if let Some(device) = PROMISE_NOT_A_PERSON.search(clean) {
        if reported.as_ref().is_none_or(|r| device.start() < r.start()) {
            return None;
        }
    }
    if PROMISE_MONEY.is_match(clean) || PROMISE_RISK.is_match(clean) {
        return None;
    }
    let said = az_set(&clean.replace('\u{2019}', "'"));
    let office_subject = PROMISE_OFFICE.is_match(clean) && !PROMISE_A_HE_OR_SHE.is_match(clean) && !PROMISE_A_PERSON_NAMED.is_match(clean);
    if !office_subject && said.iter().any(|w| GRIEVANCE_HEALTH.contains(w) || GRIEVANCE_HARM.contains(w)) {
        return None;
    }
    let broken = PROMISE_BUT_NEVER.search(clean);
    let promised = PROMISE_NEVER_WHAT.is_match(clean)
        || reported.as_ref().is_some_and(|r| {
            (PROMISE_RECUR.is_match(clean) && PROMISE_FUTURE.is_match(clean)) || broken.as_ref().is_some_and(|b| b.start() > r.end())
        });
    if !promised {
        return None;
    }
    Some(if office_subject { PROMISE_AN_OFFICE.clone() } else { PROMISE_A_PERSON.clone() })
}

regexes!(DOABLE, DOABLE_NOT, DOABLE_FOLD);
word_sets!(DOABLE_WORDS, DOABLE_BREATHERS);
pools!(DOABLE_REPLIES);

/// Python: bot/compose/fallback.py:7087 `_reads_as_a_judgement(text)`
/// True when the line is nothing but a judgement that what was said is doable.
pub fn _reads_as_a_judgement(text_: &str) -> bool {
    let mut clean = text::squash(&lower(text_));
    while DOABLE_FOLD.match_(&clean).is_some() {
        clean = text::strip(&DOABLE_FOLD.subn("", &clean, 1)).to_string();
    }
    if clean.is_empty() || clean.contains('?') {
        return false;
    }
    let words = text::split_ref(&clean);
    if words.len() > 8 {
        return false;
    }
    if !DOABLE.is_match(&clean) || DOABLE_NOT.is_match(&clean) {
        return false;
    }
    words.iter().map(|w| w.trim_matches(['.', ',', '!', '?', ';', ':', '\'', '\u{2019}'])).all(|w| DOABLE_WORDS.contains(w) || DOABLE_BREATHERS.contains(w))
}

/// Python: bot/compose/fallback.py:7111 `_a_doable(clean, memory)`
/// The reply for a short "that's doable" about the advice just given, or None. (The skeleton's
/// `()` return could not carry the reply.)
pub fn _a_doable(clean: &str, memory: &Memory) -> Option<String> {
    if !_reads_as_a_judgement(clean) {
        return None;
    }
    if the_advice(memory).is_empty() {
        return None;
    }
    choose(&DOABLE_REPLIES, memory, clean)
}

/// Python: bot/compose/fallback.py:7121 `a_doable(text, memory)`
/// Read by greeting: the reply when this line is a judgement of the answer just given, or None.
/// (The skeleton's `()` return could not carry the reply.)
pub fn a_doable(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&lower(text_));
    _a_doable(&clean, memory)
}

// ---- fallback.py:7134-7640: judgement, a change of medicine, plans, standing, advice words ----

/// Python: bot/compose/fallback.py:7133 `reads_as_a_judgement(text)`
/// Read by answers: is this line a judgement rather than a thing?
pub fn reads_as_a_judgement(text_: &str) -> bool {
    _reads_as_a_judgement(text_)
}

regexes!(PLAN_MEDICINE, _SPLITTING_A_TABLET, _A_QUESTION, _A_QUALIFIED_PERSON_DOES_IT);

/// Python: bot/compose/fallback.py:7268 `reads_as_a_change_of_medicine(text)`
/// Is this line the person's own plan to change what they take?
pub fn reads_as_a_change_of_medicine(text_: &str) -> bool {
    if !PLAN_MEDICINE.is_match(text_) {
        return false;
    }
    if _SPLITTING_A_TABLET.is_match(text_) {
        return false;
    }
    if _A_QUESTION.is_match(text::strip(text_)) {
        return false;
    }
    !_A_QUALIFIED_PERSON_DOES_IT.is_match(text_)
}

regexes!(PLAN_NOT, PLAN_IDIOM, PLAN_OPENING, PLAN_DECIDED, RELATIONSHIP_ENDING);
pools!(PLAN_DONE);

/// Python: bot/compose/fallback.py:7345 `_plan_reported(clean, mood)`
/// True when the line is the person saying what they will do next, not a trouble.
pub fn _plan_reported(clean: &str, mood: &str) -> bool {
    if mood == "negative" || PLAN_NOT.is_match(clean) || PLAN_IDIOM.is_match(clean) {
        return false;
    }
    PLAN_OPENING.match_(clean).is_some() || PLAN_DECIDED.match_(clean).is_some() || PLAN_DONE.iter().any(|p| clean.contains(p.as_str()))
}

regexes!(A_CROWD_SUBJECT, A_MARKER_OF_STANDING, A_PARENT_STILL_THERE, MY_OWN_AGE, THE_PERSON_IS_THE_SUBJECT, AN_ASK_ON_TOP);

/// Python: bot/compose/fallback.py:7411 `a_comparison_of_standing(clean)`
/// True when the line measures the person's life against a crowd's - the feeling of being behind.
pub fn a_comparison_of_standing(clean: &str) -> bool {
    let line = text::squash(clean);
    if line.is_empty() || line.contains('?') {
        return false;
    }
    if text::split_ref(&line).len() > 14 {
        return false;
    }
    if A_PARENT_STILL_THERE.is_match(&line) && rx_flags_i(r"\bstill\b").is_match(&line) {
        return false;
    }
    let mut comparison: &str = &line;
    if THE_PERSON_IS_THE_SUBJECT.match_(&line).is_some() {
        // re.split(r"(?:,|\band\b|\bbut\b)", line, maxsplit=1)
        let Some(turn) = rx(r"(?:,|\band\b|\bbut\b)").search(&line) else { return false };
        comparison = &line[turn.end()..];
        if !A_CROWD_SUBJECT.is_match(comparison) {
            return false;
        }
    }
    if !A_CROWD_SUBJECT.is_match(comparison) {
        return rx_flags_i(r"\bat (?:my|our) age\b").is_match(comparison);
    }
    if AN_ASK_ON_TOP.is_match(&line) {
        return false;
    }
    if MY_OWN_AGE.is_match(comparison) {
        return true;
    }
    A_MARKER_OF_STANDING.is_match(comparison)
}

/// Python: bot/compose/fallback.py:7466 `_a_relationship_ending(clean)`
pub fn _a_relationship_ending(clean: &str) -> bool {
    RELATIONSHIP_ENDING.is_match(clean)
}

regexes!(ADVICE_CUE);
static ADVICE_LENGTH: LazyLock<usize> = LazyLock::new(|| data::get("bot.compose.fallback.ADVICE_LENGTH").as_int().unwrap() as usize);
static IRREGULAR_STEP: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.IRREGULAR_STEP").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

/// Python: bot/compose/fallback.py:7586 `_advice_in(last)`
/// Was the last answer something the person could do and report back on?
pub fn _advice_in(last: &str) -> bool {
    text::len(last) >= *ADVICE_LENGTH || ADVICE_CUE.is_match(&lower(last))
}

/// Python: bot/compose/fallback.py:7625 `_the_words_in(advice)`
/// The words of an answer, with the past forms a report would write its steps in (a set).
pub fn _the_words_in(advice: &str) -> HashSet<String> {
    if advice.is_empty() {
        return HashSet::new();
    }
    let said = az_set(&lower(advice));
    if said.is_empty() {
        return HashSet::new();
    }
    let mut words = said.clone();
    for word in &said {
        let n = word.chars().count();
        if n < 3 {
            continue;
        }
        if word.ends_with("ing") && n > 4 {
            let stem = &word[..word.len() - 3];
            words.insert(stem.to_string());
            words.insert(format!("{stem}e"));
            let b = stem.as_bytes();
            // stem[-1:] == stem[-2:-1]
            let last = &stem[stem.len().saturating_sub(1)..];
            let before = if stem.len() >= 2 { &stem[stem.len() - 2..stem.len() - 1] } else { "" };
            if last == before && !b.is_empty() {
                words.insert(stem[..stem.len() - 1].to_string());
            }
        }
    }
    let mut forms = words.clone();
    for word in &words {
        if word.is_empty() {
            continue;
        }
        let last = word.chars().last().unwrap();
        forms.insert(format!("{word}ed"));
        forms.insert(format!("{word}d"));
        forms.insert(format!("{word}{last}ed"));
    }
    for (past, base) in IRREGULAR_STEP.iter() {
        if words.contains(base) {
            forms.insert(past.clone());
        }
    }
    forms
}

// ---- fallback.py:7764-7990: the advice asked about, easing, arrivals, presence, the store ----

regexes!(ADVICE_COOL_ASK, ADVICE_TIME_WORDS, ADVICE_WORSE_ASK, ADVICE_TIME_ASK, ADVICE_AGAIN_ASK);
pools!(ADVICE_COOL_WORDS, ADVICE_COOL, ADVICE_WORSE, ADVICE_TIME, ADVICE_AGAIN);

/// Python: bot/compose/fallback.py:7779 `_asks_about_the_advice(clean, last)`
/// The pool for a question about the advice just given, or None. (The skeleton's `()` return
/// could not carry the pool.)
pub fn _asks_about_the_advice(clean: &str, last: &str) -> Option<Vec<String>> {
    if ADVICE_COOL_ASK.is_match(clean) && ADVICE_COOL_WORDS.iter().any(|w| last.contains(w.as_str())) {
        return Some(ADVICE_COOL.clone());
    }
    if !ADVICE_TIME_WORDS.is_match(last) {
        return None;
    }
    if ADVICE_WORSE_ASK.is_match(clean) {
        return Some(ADVICE_WORSE.clone());
    }
    if ADVICE_TIME_ASK.is_match(clean) {
        return Some(ADVICE_TIME.clone());
    }
    if ADVICE_AGAIN_ASK.is_match(clean) {
        return Some(ADVICE_AGAIN.clone());
    }
    None
}

regexes!(FEELS_BETTER, FEELS_NOT_BETTER, MOVE_ON, NO_RELIEF, ROUTE_NAMED, WITH_YOU, ARRIVING, STILL_HERE);
word_sets!(EASING_FILLER, PRESENCE_LEFTOVER);
pools!(CRISIS_WORDS);

/// Python: bot/compose/fallback.py:7817 `_feels_better(clean)`
/// True for a line that says they are easing, and says nothing else.
pub fn _feels_better(clean: &str) -> bool {
    if !FEELS_BETTER.is_match(clean) || FEELS_NOT_BETTER.is_match(clean) {
        return false;
    }
    if clean.contains('?') || MOVE_ON.is_match(clean) {
        return false;
    }
    az_words(clean)
        .iter()
        .all(|w| EASING_FILLER.contains(w) || SYMPTOM_WORDS.contains(w) || BODY_PARTS.contains(w) || HEALTH_WORDS.contains(w))
}

/// Python: bot/compose/fallback.py:7848 `_no_relief(memory, last='')` (never called on the trace corpus)
/// True when the thread is one where feeling better does not mean being out of danger.
pub fn _no_relief(memory: &Memory, last: &str) -> bool {
    let mut lines: Vec<&str> = vec![last];
    for past in &memory.turns {
        lines.push(&past.user);
        lines.push(&past.bot);
    }
    if lines.iter().any(|l| !l.is_empty() && NO_RELIEF.is_match(&lower(l))) {
        return true;
    }
    for past in &memory.turns {
        let said = &past.bot;
        let low = lower(said);
        if CRISIS_WORDS.iter().any(|w| low.contains(w.as_str())) {
            continue;
        }
        // re.split(r"(?<=[.?!])\s", said.strip(), 1)[0].lower()
        let stripped = text::strip(said);
        let opening = match rx(r"(?<=[.?!])\s").search(stripped) {
            Some(m) => &stripped[..m.start()],
            None => stripped,
        };
        let opening = lower(opening);
        if !opening.is_empty() && ROUTE_NAMED.is_match(&opening) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:7874 `_someone_is_arriving(clean)`
pub fn _someone_is_arriving(clean: &str) -> bool {
    if WITH_YOU.is_match(clean) {
        return true;
    }
    rx(r"\b(?:come|coming|let)\s+(?:in|round)\b").is_match(clean) && ARRIVING.is_match(clean)
}

/// Python: bot/compose/fallback.py:8030 `_is_presence_check(clean)`
/// True when the line is only "are you there / can you hear me", and nothing more.
pub fn _is_presence_check(clean: &str) -> bool {
    let Some(m) = STILL_HERE.search(clean) else { return false };
    let rest = az_words(&format!("{} {}", &clean[..m.start()], &clean[m.end()..]));
    rest.len() <= 2 && rest.iter().all(|w| PRESENCE_LEFTOVER.contains(w))
}

static MEMORY_STORE_FIRST_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.memory_store.FIRST_ASK"));

/// Python: bot/compose/fallback.py:8049 `_the_store_takes_it(clean)` (never called on the trace corpus)
/// The card stands down only on a line the memory store will answer (`FIRST_ASK`).
pub fn _the_store_takes_it(clean: &str) -> bool {
    MEMORY_STORE_FIRST_ASK.is_match(clean)
}

// ---- fallback.py:8016-9018: the identity check, choices, the person's own records and days ----

/// Python: bot/compose/fallback.py:8102 `_the_answers_take_it(clean)` (never called on the trace corpus)
/// Whether the knowledge module will answer this line itself (always a bool in Python; the
/// contract's `Option` is kept and is always `Some`).
pub fn _the_answers_take_it(clean: &str) -> Option<bool> {
    let taken = py_try(|| crate::modules::answers::respond(clean, &mut Memory::new()).is_some()).unwrap_or(false);
    Some(taken)
}

word_sets!(HELP_TAIL);

/// Python: bot/compose/fallback.py:8166 `_free_tail(tail)` (never called on the trace corpus)
/// True when nothing but subject-free words follow the offer.
pub fn _free_tail(tail: &str) -> bool {
    az_words(tail).iter().all(|w| HELP_TAIL.contains(w))
}

regexes!(CHOICE, CHOICE_WORRY, CHOICE_FRAME, CHOICE_ARTICLE, JOINT_FRAME, CHOICE_REFUSED);

/// Python: bot/compose/fallback.py:8208 `_choice_advice(clean)`
/// `clean` with a joint question frame taken off, for the advice-word test.
pub fn _choice_advice(clean: &str) -> String {
    if JOINT_FRAME.match_(text::strip(clean)).is_some() { _unframed(clean) } else { clean.to_string() }
}

/// Python: bot/compose/fallback.py:8213 `_unframed(clean)`
/// `clean` without the question frame in front of a choice, and without its article.
pub fn _unframed(clean: &str) -> String {
    let line = CHOICE_FRAME.sub("", text::strip(clean));
    let line = CHOICE_ARTICLE.sub("", text::strip(&line));
    text::strip(&line).to_string()
}

/// Python: bot/compose/fallback.py:8234 `_refuses_the_pair(clean)`
/// True when the line turns both things down instead of asking which to take.
pub fn _refuses_the_pair(clean: &str) -> bool {
    let head = clean.split(" or ").next().unwrap_or("");
    CHOICE_REFUSED.is_match(head)
}

/// Python: bot/compose/fallback.py:8240 `_is_a_choice(clean)`
/// True for a bare "x or y" that names two things, not a question with a clause in it.
pub fn _is_a_choice(clean: &str) -> bool {
    let asked = CHOICE.match_(clean).is_some() || CHOICE.match_(&_unframed(clean)).is_some();
    asked && !CHOICE_WORRY.is_match(clean) && !crate::understanding::kinds::two_ends_of_one_scale(clean) && !_refuses_the_pair(clean)
}

regexes!(ORDER_SIDE_END, ORDER_BEFORE_AFTER, ORDER_WORD_TAIL, ORDER_WORD_LEAD, CHOICE_SIDE_LEAD, CHOICE_SIDE_TAIL, CHOICE_SIDE_WHEN, CHOICE_PERSON_SIDE);
word_sets!(ORDER_ONLY);

/// Python: bot/compose/fallback.py:8401 `_an_order_of_work(sides, clean)`
/// True when the two options are the steps of one job rather than two things to pick from.
pub fn _an_order_of_work(sides: Option<&[String]>, clean: &str) -> bool {
    if ORDER_BEFORE_AFTER.is_match(clean) {
        return true;
    }
    let sides = sides.unwrap_or(&[]);
    if !sides.iter().any(|s| ORDER_SIDE_END.is_match(s)) {
        return false;
    }
    for side in sides {
        let core = ORDER_WORD_TAIL.sub("", &ORDER_WORD_LEAD.sub("", side));
        let core = CHOICE_ARTICLE.sub("", text::strip(&core));
        let core = text::strip(&core);
        if ORDER_ONLY.contains(core) || text::len(core) < 2 {
            return false;
        }
    }
    true
}

/// Python: bot/compose/fallback.py:8419 `_choice_sides(clean)`
/// The two things a bare choice names, as the person typed them, or None.
pub fn _choice_sides(clean: &str) -> Option<Vec<String>> {
    let unframed = _unframed(clean);
    let head = unframed.trim_end_matches('?');
    let parts: Vec<&str> = head.split(" or ").collect();
    if parts.len() != 2 {
        return None;
    }
    let mut sides = Vec::new();
    for part in parts {
        let side = CHOICE_SIDE_TAIL.sub("", &CHOICE_SIDE_LEAD.sub("", text::strip(part)));
        let side = CHOICE_SIDE_WHEN.sub("", text::strip(&side));
        let side = CHOICE_ARTICLE.sub("", text::strip(&side));
        let side = text::strip(&side).to_string();
        if side.is_empty() || !side.chars().any(|c| c.is_ascii_lowercase()) {
            return None;
        }
        sides.push(side);
    }
    Some(sides)
}

/// Python: bot/compose/fallback.py:8441 `_capital(side)`
/// `side` with its first letter as a capital, which is how a name or an option starts.
pub fn _capital(side: &str) -> String {
    let side = text::squash(side);
    if side.is_empty() {
        return side;
    }
    format!("{}{}", text::upper(&text::slice(&side, 0, Some(1))), text::slice(&side, 1, None))
}

pools!(NAMED_TYPES);

/// Python: bot/compose/fallback.py:8447 `_named(side, memory=None)`
/// `side` with any name in it spelled the way the person spelled it.
pub fn _named(side: &str, memory: &Memory) -> String {
    let mut side = side.to_string();
    let Some(understood) = &memory.understood else { return side };
    for entity in &understood.entities {
        if !NAMED_TYPES.contains(&entity.kind) {
            continue;
        }
        let name = text::strip(&entity.text).to_string();
        if text::len(&name) < 2 || !side.contains(&lower(&name)) {
            continue;
        }
        let pattern = format!(r"\b{}\b", crate::py::re::escape(&lower(&name)));
        side = rx(&pattern).sub(&name, &side);
    }
    side
}

/// Python: bot/compose/fallback.py:8460 `_a_pick(sides, clean, memory=None)`
/// One side of the choice, said as a reply, and the other one quoted beside it: `[take, other]`.
pub fn _a_pick(sides: &[String], clean: &str, memory: &Memory) -> Vec<String> {
    let (first, second) = (&sides[0], &sides[1]);
    let seed: i128 = clean.chars().map(|c| c as i128).sum();
    let (take, other) = if crate::py::random::Random::new(seed).getrandbits(1) != 0 { (first, second) } else { (second, first) };
    vec![_capital(&_named(take, memory)), _named(other, memory)]
}

word_sets!(CHOICE_PEOPLE, CHOICE_HEALTH, CHOICE_THERAPY, CHOICE_STAKES);
pools!(CHOICE_LIGHT, CHOICE_BETWEEN_PEOPLE, CHOICE_LIGHT_PICK, CHOICE_WEIGH, CHOICE_PHARMACIST);

/// Python: bot/compose/fallback.py:8474 `_between_people(sides, clean)`
/// True when the two things are two people rather than two things.
pub fn _between_people(sides: &[String], clean: &str) -> bool {
    for side in sides {
        if text::split_ref(side).iter().any(|w| CHOICE_PEOPLE.contains(*w)) {
            return true;
        }
    }
    CHOICE_PERSON_SIDE.is_match(clean)
}

/// Python: bot/compose/fallback.py:8487 `_choice_light(clean, memory=None)`
pub fn _choice_light(clean: &str, memory: &Memory) -> Vec<String> {
    let Some(sides) = _choice_sides(clean) else { return CHOICE_LIGHT.clone() };
    if _between_people(&sides, clean) {
        return CHOICE_BETWEEN_PEOPLE.clone();
    }
    let pick = _a_pick(&sides, clean, memory);
    CHOICE_LIGHT_PICK.iter().map(|l| format_named(l, &[("pick", &pick[0]), ("other", &pick[1])])).collect()
}

/// Python: bot/compose/fallback.py:8499 `_choice_pool(clean, memory=None)`
pub fn _choice_pool(clean: &str, memory: &Memory) -> Vec<String> {
    let words = text::split_ref(clean);
    if words.iter().any(|w| CHOICE_HEALTH.contains(*w)) {
        if words.iter().any(|w| CHOICE_THERAPY.contains(*w)) {
            return CHOICE_WEIGH.clone();
        }
        return CHOICE_PHARMACIST.clone();
    }
    if _an_order_of_work(_choice_sides(clean).as_deref(), clean) {
        return ORDER_OF_WORK.clone();
    }
    if words.iter().any(|w| CHOICE_STAKES.contains(*w)) {
        return CHOICE_WEIGH.clone();
    }
    _choice_light(clean, memory)
}

regexes!(OWN_RECORD_NOT, RECORD_STATE, RECORD_QUALIFIED, OFFICE_STATE, MY_ANY_THING, A_TILL_ASK, NOT_AN_OFFICE, RECORD_CHECK, MY_OWN_STATE, CARD_VALIDITY_ASK, CARD_BALANCE_ASK, CARD_NOT_A_BALANCE);

/// Python: bot/compose/fallback.py:8535 `_their_own_phrase(clean, things)`
/// The possessive phrase naming one of the person's own things, or None. (Python keeps the
/// compiled patterns in `MY_OWN`; `rx` is the same cache.)
pub fn _their_own_phrase<'t>(clean: &'t str, things: &[String]) -> Option<Match<'t>> {
    for thing in things {
        let pattern = format!(r"\b(?:my|our)\s+(?:[a-z']+\s+){{0,2}}{}(?![a-z])", crate::py::re::escape(thing));
        let re_: &'static Re = rx(&pattern);
        if let Some(m) = re_.search(clean) {
            return Some(m);
        }
    }
    None
}

/// `pattern.match(text, pos)`: a match that starts exactly at `pos` (a byte offset).
fn match_at<'t>(pattern: &'t Re, text_: &'t str, pos: usize) -> Option<Match<'t>> {
    pattern.search_at(text_, pos).filter(|m| m.start() == pos)
}

static _A_MY_PERSON: LazyLock<Re> = LazyLock::new(|| {
    let mut words: Vec<String> = data::get("bot.compose.loss.PEOPLE").strs().iter().map(|w| crate::py::re::escape(w)).collect();
    words.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    Re::new(
        &format!(
            concat!(
                r"\b(?:my|our|his|her|their|the)\s+(?:{})(?:\'s|s\')?",
                r"(?=\s*(?:$|[,.!?;]|(?:has|have|had|is|are|was|were|did|does|do|not|never|just|now|",
                r"still|yet|already|arrived|arrive|arrives|coming|come|came|returned|returns|got|",
                r"gotten|been|being|at|home|there|back|in|out|safe|ok|alright|fine|well|on|off|to|",
                r"from|away|left|gone|late|early|waiting)\b))"
            ),
            words.join("|")
        ),
        crate::py::re::I,
    )
});

/// Python: bot/compose/fallback.py:8593 `_a_person_is_the_thing(clean)`
/// True when the person is the subject of the state, not the owner of a record.
pub fn _a_person_is_the_thing(clean: &str) -> bool {
    _A_MY_PERSON.is_match(clean)
}

regexes!(MY_BILL_THING, MY_BILL_MINE, MY_BILL_ASK, MY_BILL_NOT, POINTS_SCHEME, POINTS_ASK, POINTS_NOT);
pools!(MY_BILL_LINES, POINTS_LINES, AT_THE_TILL, OFFICE_RECORD_LINES, CARD_VALIDITY, CARD_RECORD_THINGS, RECORD_THINGS, HEALTH_RECORD, HEALTH_WORLD, HEALTH_RECORD_LINES);
static MY_RECORDS: LazyLock<Vec<(String, Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.MY_RECORDS")
        .items()
        .iter()
        .map(|r| (r.items()[0].str().to_string(), r.items()[1].strs(), r.items()[2].strs()))
        .collect()
});
/// `MY_CARD_ROW`, `MY_MONEY_ROW`, `MY_APPLICATION_ROW`: a row's options, or None.
fn row_of(name: &str) -> Option<Vec<String>> {
    let v = data::get(&format!("{M}{name}"));
    if v.is_none() { None } else { Some(v.strs()) }
}
static MY_CARD_ROW: LazyLock<Option<Vec<String>>> = LazyLock::new(|| row_of("MY_CARD_ROW"));
static MY_MONEY_ROW: LazyLock<Option<Vec<String>>> = LazyLock::new(|| row_of("MY_MONEY_ROW"));
static MY_APPLICATION_ROW: LazyLock<Option<Vec<String>>> = LazyLock::new(|| row_of("MY_APPLICATION_ROW"));

/// Python: bot/compose/fallback.py:8711 `_their_own_bill(clean)`
/// The honest line for the price of something the person already pays for, or None.
pub fn _their_own_bill(clean: &str) -> Option<Vec<String>> {
    if !MY_BILL_MINE.is_match(clean) || !MY_BILL_THING.is_match(clean) {
        return None;
    }
    if !MY_BILL_ASK.is_match(clean) || MY_BILL_NOT.is_match(clean) {
        return None;
    }
    Some(MY_BILL_LINES.clone())
}

/// Python: bot/compose/fallback.py:8720 `_their_own_points(clean)`
/// The honest line for a points or stamps balance, or None.
pub fn _their_own_points(clean: &str) -> Option<Vec<String>> {
    if !POINTS_SCHEME.is_match(clean) || POINTS_NOT.is_match(clean) {
        return None;
    }
    if !POINTS_ASK.is_match(clean) {
        return None;
    }
    Some(POINTS_LINES.clone())
}

/// Python: bot/compose/fallback.py:8733 `_an_office_record(clean, asking)`
/// The honest line for a record only an office can see, or None.
pub fn _an_office_record(clean: &str, asking: bool) -> Option<Vec<String>> {
    if !asking || NOT_AN_OFFICE.is_match(clean) {
        return None;
    }
    if A_TILL_ASK.is_match(clean) {
        return Some(AT_THE_TILL.clone());
    }
    if _a_person_is_the_thing(clean) {
        return None;
    }
    if OFFICE_STATE.is_match(clean) && MY_ANY_THING.is_match(clean) {
        return Some(OFFICE_RECORD_LINES.clone());
    }
    if MY_OWN_STATE.is_match(clean) {
        let money: Vec<String> = ["paid", "pay", "wages", "salary", "money", "expenses"].iter().map(|s| s.to_string()).collect();
        if _has(clean, &money) {
            if let Some(row) = MY_MONEY_ROW.as_ref() {
                return Some(row.clone());
            }
        }
        if let Some(row) = MY_APPLICATION_ROW.as_ref() {
            return Some(row.clone());
        }
    }
    None
}

/// Python: bot/compose/fallback.py:8763 `_their_own_record(clean, asking)`
/// The honest line for one of the person's own records, or None.
pub fn _their_own_record(clean: &str, asking: bool) -> Option<Vec<String>> {
    if let Some(said) = _their_own_week(clean) {
        if !said.is_empty() {
            return Some(said);
        }
    }
    if let Some(said) = _their_own_points(clean) {
        if !said.is_empty() {
            return Some(said);
        }
    }
    let checking = RECORD_CHECK.is_match(clean) && _their_own_phrase(clean, &RECORD_THINGS).is_some();
    let validity = CARD_VALIDITY_ASK.is_match(clean) && _their_own_phrase(clean, &CARD_RECORD_THINGS).is_some();
    if !(asking || checking || validity) || OWN_RECORD_NOT.is_match(clean) {
        return None;
    }
    if validity {
        return Some(CARD_VALIDITY.clone());
    }
    if !(RECORD_STATE.is_match(clean) || checking) {
        return _an_office_record(clean, asking);
    }
    let mut rows: Vec<&(String, Vec<String>, Vec<String>)> = MY_RECORDS.iter().collect();
    if _their_own_phrase(clean, &CARD_RECORD_THINGS).is_some() {
        rows.sort_by_key(|row| row.0 != "card");
    }
    for (_, things, options) in rows {
        if let Some(phrase) = _their_own_phrase(clean, things) {
            if match_at(&RECORD_QUALIFIED, clean, phrase.end()).is_none() {
                return Some(options.clone());
            }
        }
    }
    if let Some(row) = MY_CARD_ROW.as_ref() {
        if CARD_BALANCE_ASK.is_match(clean) && !CARD_NOT_A_BALANCE.is_match(clean) && _their_own_phrase(clean, &["card".to_string()]).is_some() {
            return Some(row.clone());
        }
    }
    if let Some(phrase) = _their_own_phrase(clean, &HEALTH_RECORD) {
        if match_at(&RECORD_QUALIFIED, clean, phrase.end()).is_none() && _has(clean, &HEALTH_WORLD) {
            return Some(HEALTH_RECORD_LINES.clone());
        }
    }
    _an_office_record(clean, asking)
}

regexes!(A_DAY_AGO, MY_OWN_DOING, NOT_ABOUT_THEIR_DAY);
pools!(MY_OWN_DAY_LINES);

/// Python: bot/compose/fallback.py:8852 `_their_own_week(clean)`
/// The honest line for what the person did on a day gone by, or None.
pub fn _their_own_week(clean: &str) -> Option<Vec<String>> {
    let said = text::squash(&lower(clean));
    if said.is_empty() {
        return None;
    }
    if NOT_ABOUT_THEIR_DAY.is_match(&said) {
        return None;
    }
    if A_DAY_AGO.is_match(&said) && MY_OWN_DOING.is_match(&said) {
        return Some(MY_OWN_DAY_LINES.clone());
    }
    None
}

regexes!(NUMBER_WANTED, NUMBER_OF, NUMBER_OF_PHONE, NUMBER_REPORT);
pools!(NUMBER_NOT);
static NUMBER_PLACES: LazyLock<Vec<(Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.NUMBER_PLACES").items().iter().map(|p| (p.items()[0].strs(), p.items()[1].strs())).collect()
});

/// Python: bot/compose/fallback.py:8906 `_names_place(clean, places)`
/// True when the line names one of these places, plural or not ("the councils number"). (Python
/// keeps the compiled patterns in `NUMBER_PLACE_PATTERNS`; `rx` is the same cache.)
pub fn _names_place(clean: &str, places: &[String]) -> bool {
    places.iter().any(|place| rx(&format!(r"\b{}(?:s|es)?\b", crate::py::re::escape(place))).is_match(clean))
}

/// Python: bot/compose/fallback.py:8918 `_their_contact_number(clean, asking)`
/// The honest line for the phone number of a place the person wants to ring, or None. (The
/// skeleton's `()` return could not carry the pool.)
pub fn _their_contact_number(clean: &str, asking: bool) -> Option<Vec<String>> {
    if !NUMBER_WANTED.is_match(clean) || _has(clean, &NUMBER_NOT) {
        return None;
    }
    if NUMBER_REPORT.is_match(clean) {
        return None;
    }
    if NUMBER_OF.is_match(clean) && !NUMBER_OF_PHONE.is_match(clean) {
        return None;
    }
    for (places, options) in NUMBER_PLACES.iter() {
        if _names_place(clean, places) {
            return Some(options.clone());
        }
    }
    None
}

regexes!(MY_DAY_ASK, MY_DAY_NOT);
word_sets!(MY_DAY_SAID);
pools!(MY_OWN_DAY);

/// Python: bot/compose/fallback.py:8966 `_their_own_day(clean)`
/// The honest line for their own shift, or None. (The skeleton's `()` return could not carry
/// the pool.)
pub fn _their_own_day(clean: &str) -> Option<Vec<String>> {
    let m = MY_DAY_ASK.search(clean)?;
    if MY_DAY_NOT.is_match(clean) {
        return None;
    }
    let work: Vec<String> = ["work", "works", "working", "job", "shift", "shifts", "rota", "hours"].iter().map(|s| s.to_string()).collect();
    if !(_has(clean, &work) || rx(r"\bclock(?:ed|s)? (?:in|off|on)\b").is_match(clean)) {
        return None;
    }
    let rest = format!("{} {}", &clean[..m.start()], &clean[m.end()..]);
    if az_words(&rest).iter().any(|w| !MY_DAY_SAID.contains(w)) {
        return None;
    }
    Some(MY_OWN_DAY.clone())
}

pools!(LIVE_TAKE_ASK, LIVE_WEAR, LIVE_SOON);
word_sets!(LIVE_TAKE_ALLOWED);

/// Python: bot/compose/fallback.py:8999 `_a_bare_wear_ask(clean)` (never called on the trace corpus)
/// True when a take/wear ask is the whole of the line and nothing else is attached.
pub fn _a_bare_wear_ask(clean: &str) -> bool {
    let mut t = clean.to_string();
    let mut asks: Vec<&String> = LIVE_TAKE_ASK.iter().collect();
    asks.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    for phrase in asks {
        t = t.replace(phrase.as_str(), " ");
    }
    for word in LIVE_WEAR.iter().chain(LIVE_SOON.iter()) {
        t = t.replace(word.as_str(), " ");
    }
    az_words(&t).iter().all(|w| LIVE_TAKE_ALLOWED.contains(w))
}

// ---- fallback.py:9081-9395: asks, care threads, the bot's own pick, a trade-off leaned on ----

pools!(REQUEST_CUES, OPINION_ASKS, TRADE_OFF_LEAN, MY_PICK_NOTE, MY_PICK, MY_PICK_ASK);
regexes!(OWN_LIFE, PICK_ASK, PICK_WORD, PICK_TITLE, PICK_NOTE, PICK_ADVICE, LEAN_ASK, PICK_BACK);
regexes!(TRADE_OFF, NOT_A_SIDE, NOT_A_CHOICE, SIDE_HAS, SIDE_IS, SIDE_SUBJECT, SIDE_VERB, SIDE_GOOD, SIDE_BAD, SIDE_ONE, SIDE_COPULA, SIDE_TAIL, SIDE_OTHER);
regexes!(DESIRE, OBSTACLE, LEAN_NOT);
word_sets!(CARE_MODULES, QUIET_MODULES, PICK_ONLY_WORDS, WORRY_WORDS);
strings!(FALLBACK_MODULE);
static THREAD_TURNS: LazyLock<i64> = LazyLock::new(|| data::get("bot.compose.fallback.THREAD_TURNS").as_int().unwrap());

/// Python: bot/compose/fallback.py:9080 `_asks_for_something(text)` (never called on the trace corpus)
/// True when the line is more than a greeting: it wants an answer.
pub fn _asks_for_something(text_: &str) -> bool {
    let low = lower(text_);
    REQUEST_CUES.iter().any(|cue| low.contains(cue.as_str()))
}

/// Python: bot/compose/fallback.py:9086 `_reads_as_why(clean)`
pub fn _reads_as_why(clean: &str) -> bool {
    clean.starts_with("why") || clean.contains("why that") || clean.contains("how come") || clean.contains("what makes you say")
}

/// Python: bot/compose/fallback.py:9098 `_is_about_the_persons_own_life(clean)`
pub fn _is_about_the_persons_own_life(clean: &str) -> bool {
    OWN_LIFE.is_match(clean)
}

/// Python: bot/compose/fallback.py:9103 `_in_a_care_thread(memory)`
/// True when the bot's last answer, reading past its own continuations, was care.
pub fn _in_a_care_thread(memory: &Memory) -> bool {
    for turn in last_n(&memory.turns, *THREAD_TURNS).iter().rev() {
        let module = turn.module.as_deref();
        if module.is_some_and(|m| CARE_MODULES.contains(m)) {
            return true;
        }
        if !module.is_some_and(|m| QUIET_MODULES.contains(m)) {
            return false;
        }
    }
    false
}

/// `PICK_WORD.findall(s)`: the runs of `[a-z0-9']`.
fn pick_words(s: &str) -> Vec<String> {
    PICK_WORD.findall_str(s)
}

/// Python: bot/compose/fallback.py:9121 `_a_lean_only_line(clean)`
/// True when the line is only the ask, so the trade-off is what it is about.
pub fn _a_lean_only_line(clean: &str) -> bool {
    pick_words(clean).iter().all(|w| PICK_ONLY_WORDS.contains(w))
}

/// Python: bot/compose/fallback.py:9126 `_a_bare_pick(clean)`
/// True when the line asks for the bot's own pick and names nothing of its own.
pub fn _a_bare_pick(clean: &str) -> bool {
    if !PICK_ASK.is_match(clean) {
        return false;
    }
    if clean.contains(" or ") {
        return false;
    }
    if _has(clean, &OPINION_ASKS) {
        return false;
    }
    pick_words(clean).iter().all(|w| PICK_ONLY_WORDS.contains(w))
}

/// Python: bot/compose/fallback.py:9142 `_the_side_taken(reply, sides)`
/// The side of `sides` the bot's own reply went to, or None when it named neither.
pub fn _the_side_taken(reply: &str, sides: Option<&[String]>) -> Option<String> {
    let said = lower(reply);
    let mut best: Option<(usize, &String)> = None;
    for side in sides.unwrap_or(&[]) {
        let found = rx(&format!(r"\b{}", crate::py::re::escape(&lower(side)))).search(&said);
        if let Some(found) = found {
            let at = text::char_index(&said, found.start());
            if best.is_none_or(|(b, _)| at < b) {
                best = Some((at, side));
            }
        }
    }
    best.map(|(_, side)| side.clone())
}

/// Python: bot/compose/fallback.py:9161 `_picked_a_side(reply, sides)`
pub fn _picked_a_side(reply: &str, sides: Option<&[String]>) -> bool {
    _the_side_taken(reply, sides).is_some()
}

/// Python: bot/compose/fallback.py:9172 `_offered(memory)`
/// The options the bot offered for what the conversation is on, as `(name, note)` pairs. (The
/// skeleton's `Vec<String>` could not carry the pairs Python returns.)
pub fn _offered(memory: &Memory) -> Vec<(String, String)> {
    for turn in memory.turns.iter().rev() {
        let said = pick_words(&lower(&turn.user));
        if said.is_empty() || said.iter().all(|w| PICK_ONLY_WORDS.contains(w)) {
            continue;
        }
        let joined = said.join(" ");
        if turn.module.as_deref() == Some(FALLBACK_MODULE.as_str()) && !_picked_a_side(&turn.bot, _choice_sides(&joined).as_deref()) {
            return Vec::new();
        }
        let found = _titles(&turn.bot);
        if found.len() >= 2 {
            return found;
        }
        let Some(sides) = _choice_sides(&joined) else { return Vec::new() };
        if sides.is_empty() {
            return Vec::new();
        }
        if let Some(taken) = _the_side_taken(&turn.bot, Some(&sides)) {
            return vec![(_capital(&taken), String::new())];
        }
        return sides.iter().map(|s| (_capital(s), String::new())).collect();
    }
    Vec::new()
}

/// Python: bot/compose/fallback.py:9212 `_titles(said)`
/// The options a reply offered, as `(name, note)` pairs.
pub fn _titles(said: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for m in PICK_TITLE.finditer(said) {
        let name = text::squash(m.str(1));
        let n_words = text::split_ref(&name).len();
        if name.is_empty() || n_words > 8 || name.trim_matches('.').is_empty() {
            continue;
        }
        let first = text::slice(&name, 0, Some(1));
        if n_words == 1 && text::is_lower(&first) && !name.chars().any(|c| text::is_digit(&c.to_string())) {
            continue;
        }
        let tail = text::slice(&said[m.end()..], 0, Some(90));
        let mut note = PICK_NOTE.match_(&tail).map(|n| text::squash(n.str(1))).unwrap_or_default();
        if !note.is_empty() && rx(r"\b(?:you|your|i|we|me|us)\b").is_match(&note) {
            note = String::new();
        }
        found.push((name, note));
    }
    found
}

/// Python: bot/compose/fallback.py:9232 `_names_in(said)` (never called on the trace corpus)
/// The words of a line that name something, with the plumbing and the ask taken out (a set).
pub fn _names_in(said: &str) -> HashSet<String> {
    pick_words(&lower(said)).into_iter().filter(|w| !PICK_ONLY_WORDS.contains(w)).collect()
}

/// Python: bot/compose/fallback.py:9237 `_a_named_side(after)` (never called on the trace corpus)
/// The side after the "but" as the thing to pick, or "" when it is not one.
pub fn _a_named_side(after: &str) -> String {
    let strip_ends = |s: &str| s.trim_matches([' ', ',', '.', ';', ':']).to_string();
    let mut t = strip_ends(&text::squash(after));
    t = strip_ends(&SIDE_TAIL.sub("", &t));
    if t.is_empty() || NOT_A_CHOICE.is_match(&t) || SIDE_BAD.is_match(&t) {
        return String::new();
    }
    if !SIDE_GOOD.is_match(&t) {
        return String::new();
    }
    t = SIDE_OTHER.sub("", &t).trim_matches([' ', ',']).to_string();
    if crate::understanding::rx_flags(r"^(?:is|are|was|were)\s+", crate::py::re::I).match_(&t).is_some() {
        t = format!("that {t}");
    }
    let first = crate::understanding::rx_flags(r"^(?:i|we|im|i'm)\b", crate::py::re::I).match_(&t).is_some();
    if let Some(has) = SIDE_HAS.match_(&t) {
        return format!("the one with {}", has.str(1).trim_matches([' ', ',', '.']));
    }
    if let Some(is_) = SIDE_IS.match_(&t) {
        return format!("the one that is {}", is_.str(1).trim_matches([' ', ',', '.']));
    }
    if first {
        return String::new();
    }
    if let Some(one) = SIDE_ONE.match_(&t) {
        if SIDE_GOOD.is_match(one.str(2)) {
            return one.str(1).to_string();
        }
    }
    if SIDE_SUBJECT.match_(&t).is_some() && SIDE_COPULA.is_match(&t) {
        return format!("the one where {t}");
    }
    if crate::understanding::rx_flags(r"^(?:more|fewer|less|better|cheaper|bigger|newer|nicer)\b", crate::py::re::I).match_(&t).is_some() {
        return format!("the one with {t}");
    }
    if SIDE_VERB.is_match(&t) && SIDE_SUBJECT.match_(&t).is_none() {
        return format!("the one that {t}");
    }
    if SIDE_SUBJECT.match_(&t).is_none() {
        return format!("the one {t}");
    }
    String::new()
}

/// Python: bot/compose/fallback.py:9286 `_a_trade_off(memory)` (never called on the trace corpus)
/// The side the person put after their "but", named as a thing to pick, or "" (Python's `()`).
pub fn _a_trade_off(memory: &Memory) -> String {
    let mut pointed: Vec<String> = Vec::new();
    for turn in memory.turns.iter().rev() {
        let said = text::squash(&turn.user);
        let found = pick_words(&lower(&said));
        if found.is_empty() || found.iter().all(|w| PICK_ONLY_WORDS.contains(w)) {
            continue;
        }
        let Some(m) = TRADE_OFF.search(&said) else {
            if text::split_ref(&said).len() <= 3 && !said.ends_with('?') {
                pointed.push(said);
                continue;
            }
            return String::new();
        };
        let before = m.str(1).trim_matches([' ', ',', '.']);
        let after = m.str(2).trim_matches([' ', ',', '.']);
        if text::split_ref(before).len() < 2 || text::split_ref(after).len() < 2 || NOT_A_SIDE.is_match(after) || DESIRE.is_match(before) || OBSTACLE.is_match(after) {
            return String::new();
        }
        if _is_a_body_or_care_trade_off(&said) {
            return String::new();
        }
        let named = _a_named_side(after);
        if named.is_empty() {
            return String::new();
        }
        let names = _names_in(&said);
        if pointed.iter().any(|line| !_names_in(line).iter().any(|w| names.contains(w))) {
            return String::new();
        }
        return named;
    }
    String::new()
}

/// Python: bot/compose/fallback.py:9328 `_is_a_body_or_care_trade_off(said)` (never called on the trace corpus)
pub fn _is_a_body_or_care_trade_off(said: &str) -> bool {
    let words = az_set(&lower(said));
    if words.iter().any(|w| BODY_PARTS.contains(w) || SYMPTOM_WORDS.contains(w) || HEALTH_WORDS.contains(w) || CHOICE_HEALTH.contains(w) || WORRY_WORDS.contains(w)) {
        return true;
    }
    LEAN_NOT.is_match(said)
}

/// Python: bot/compose/fallback.py:9341 `_a_leaning_side(clean, memory)`
/// The bot's own side of a trade-off the person has just put down, or None. (The skeleton's
/// `()` return could not carry the reply.)
pub fn _a_leaning_side(clean: &str, memory: &Memory) -> Option<String> {
    if !_a_bare_pick(clean) && !LEAN_ASK.is_match(clean) {
        return None;
    }
    if !_a_lean_only_line(clean) {
        return None;
    }
    if _in_a_care_thread(memory) {
        return None;
    }
    let side = _a_trade_off(memory);
    if side.is_empty() {
        return None;
    }
    let line = choose(&TRADE_OFF_LEAN, memory, clean).unwrap_or_else(|| panic!("AttributeError: 'NoneType' object has no attribute 'format'"));
    Some(format_named(&line, &[("side", &side)]))
}

/// Python: bot/compose/fallback.py:9364 `_my_pick(clean, memory)`
/// The bot's own pick, when the line asks for it and names nothing of its own. (The skeleton's
/// `()` return could not carry the reply.)
pub fn _my_pick(clean: &str, memory: &Memory) -> Option<String> {
    let offered = _offered(memory);
    if !_a_bare_pick(clean) && !(!offered.is_empty() && PICK_BACK.match_(text::strip(clean)).is_some()) {
        return None;
    }
    if _in_a_care_thread(memory) {
        return None;
    }
    if !offered.is_empty() {
        let seed = memory.turns.len() as i128 * 7919 + clean.chars().map(|c| c as i128).sum::<i128>();
        let (name, note) = &offered[crate::py::random::Random::new(seed).randrange(offered.len())];
        let pool: Vec<String> = if !note.is_empty() {
            MY_PICK_NOTE.iter().map(|l| format_named(l, &[("name", name), ("note", note)])).collect()
        } else {
            MY_PICK.iter().map(|l| format_named(l, &[("name", name)])).collect()
        };
        return choose(&pool, memory, clean);
    }
    if PICK_ADVICE.is_match(clean) {
        return None;
    }
    choose(&MY_PICK_ASK, memory, clean)
}

// ---- fallback.py:9568-9935: the plan on the table and the verdict on it ----

regexes!(JUDGE_ASK, JUDGE_ASK_BARE, JUDGE_PLAN_OPENING, JUDGE_SHOULD_OPENING, JUDGE_HOPED_FOR, MIND_WORRY, JUDGE_NAMES_A_HOLE);
regexes!(JUDGE_ACT_AT_SOMEONE, JUDGE_ANSWERS_FIRST, JUDGE_A_CARD_HEADING, JUDGE_NOT_A_PLAN_TO_BLESS, JUDGE_TAKING_SOMETHING, JUDGE_LYING, JUDGE_A_TARGET, JUDGE_TAKING);
regexes!(JUDGE_WISHES_HARM, JUDGE_JUDGES_A_NOUN, JUDGE_DECISION_LEAD, JUDGE_HOLE_SAFETY, JUDGE_JUDGEMENT, JUDGE_MOVING_TO, MINE);
word_sets!(JUDGE_ASKING_FOR_A_CAUTION, JUDGE_NOT_A_GERUND, A_NOUN_PHRASE_OPENS, JUDGE_PLAN_STOP, JUDGE_NOD_WORDS);
word_sets!(JUDGE_BARE_FACES, JUDGE_NOT_A_VERDICT, JUDGE_SELF_JUDGING, JUDGE_ONLY_A_PLAN, JUDGE_SOUNDS_FINE);
pools!(JUDGE_PLAN_HAS_A_HOLE, JUDGE_WISH_ANGRY, JUDGE_NO_PLAN_MIND, JUDGE_NO_PLAN_MAD, JUDGE_NO_PLAN, JUDGE_NO_PLAN_FINE, JUDGE_SANE, JUDGE_FINE, JUDGE_WORTH);
static YOURS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.YOURS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

/// Python: bot/compose/fallback.py:9574 `_the_opening_before(match)` (never called on the trace corpus)
/// The words of the line in front of the plan the opener matched: "im going to".
pub fn _the_opening_before(m: &Match) -> String {
    for group in ["plan", "plan2", "plan3", "plan9"] {
        if let Some((start, _)) = m.name_span(group) {
            if !m.name_str(group).is_empty() {
                return m.text[..start].to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/compose/fallback.py:9582 `_a_gerund(word)` (never called on the trace corpus)
/// True when the word is an "-ing" form a sentence can open on ("getting", "moving").
pub fn _a_gerund(word: &str) -> bool {
    let word = word.trim_matches(['.', ',', '!', '?', ';', ':', '\'', '"']);
    text::len(word) >= 5 && word.ends_with("ing") && !JUDGE_NOT_A_GERUND.contains(word)
}

/// Python: bot/compose/fallback.py:9598 `_a_plan_said_back(phrase)` (never called on the trace corpus)
/// "wanting to quit your job", or "wanting a cat" where the words are a thing and not a step.
pub fn _a_plan_said_back(phrase: &str) -> String {
    let words = text::split_ref(phrase);
    let opening = if words.first().is_some_and(|w| A_NOUN_PHRASE_OPENS.contains(&lower(w))) { "wanting " } else { "wanting to " };
    format!("{opening}{}", _yours(phrase))
}

/// Python: bot/compose/fallback.py:9605 `_a_plan_phrase(said)` (never called on the trace corpus)
/// The person's plan as a phrase to say back ("wanting to ask for a pay rise"), or "".
pub fn _a_plan_phrase(said: &str) -> String {
    let t = _matchable(said);
    if let Some(m) = JUDGE_PLAN_OPENING.search(&t) {
        let group = ["plan", "plan2", "plan3", "plan9"].iter().map(|g| m.name_str(g)).find(|s| !s.is_empty()).unwrap_or("");
        let words = text::split_ref(group);
        if words.is_empty() || words.len() > 7 || words.iter().any(|w| JUDGE_PLAN_STOP.contains(*w)) {
            return String::new();
        }
        let phrase = words.join(" ");
        if !phrase.chars().any(|c| c.is_ascii_lowercase()) || !rx("[a-z]{3}").is_match(&phrase) {
            return String::new();
        }
        if _a_gerund(words[0]) {
            return _yours(&phrase);
        }
        if JUDGE_MOVING_TO.is_match(&_the_opening_before(&m)) && !crate::compose::stepwords::a_step(words[0]) {
            return _yours(&format!("going to {phrase}"));
        }
        return _a_plan_said_back(&phrase);
    }
    if let Some(m) = JUDGE_SHOULD_OPENING.search(&t) {
        let words = text::split_ref(m.name_str("plan"));
        if words.is_empty() || words.len() > 7 || words.iter().any(|w| JUDGE_PLAN_STOP.contains(*w)) {
            return String::new();
        }
        if !rx("[a-z]{3}").is_match(&words.join(" ")) {
            return String::new();
        }
        return _a_plan_said_back(&words.join(" "));
    }
    String::new()
}

/// Python: bot/compose/fallback.py:9646 `_yours(phrase)` (never called on the trace corpus)
/// The person's own phrase turned to face them: "quit my job" -> "quit your job". (The second of
/// the two definitions in the Python file, the one the module keeps.)
pub fn _yours(phrase: &str) -> String {
    MINE.sub_fn(phrase, |found| {
        let g = found.str(1);
        let low = lower(g);
        YOURS.iter().find(|(k, _)| *k == low).map(|(_, v)| v.clone()).unwrap_or_else(|| g.to_string())
    })
}

/// Python: bot/compose/fallback.py:9651 `_the_plan_on_the_table(memory)` (never called on the trace corpus)
/// The plan the person has just named, or "" when the last thing they said named none (a str).
pub fn _the_plan_on_the_table(memory: &Memory) -> Value {
    Value::from(the_plan_and_its_line(memory).0)
}

/// Python: bot/compose/fallback.py:9656 `_the_plan_is_the_last_thing_said(plan_line, memory)` (never called on the trace corpus)
/// True when nothing but a nod has been said since the plan was named.
pub fn _the_plan_is_the_last_thing_said(plan_line: &str, memory: &Memory) -> bool {
    let said: Vec<String> = memory.turns.iter().map(|t| _matchable(&t.user)).filter(|l| !l.is_empty()).collect();
    let content_of = |line: &str| -> Vec<String> {
        text::split(&rx("[^a-z0-9 ]").sub(" ", line)).into_iter().filter(|w| w.chars().count() > 2 && !JUDGE_NOD_WORDS.contains(w)).collect()
    };
    let about: HashSet<String> = content_of(plan_line).into_iter().collect();
    let Some(stop) = (0..said.len()).rev().find(|&i| said[i] == plan_line) else { return true };
    for line in &said[stop + 1..] {
        let content = content_of(line);
        if content.is_empty() || content.iter().any(|w| about.contains(w)) {
            continue;
        }
        return false;
    }
    true
}

/// Python: bot/compose/fallback.py:9687 `_the_plan_and_its_line(memory)` (never called on the trace corpus)
/// The plan, and the line that named it, as a tuple value.
pub fn _the_plan_and_its_line(memory: &Memory) -> Value {
    let (plan, line) = the_plan_and_its_line(memory);
    Value::Tuple(std::sync::Arc::new(vec![Value::from(plan), Value::from(line)]))
}

/// `_the_plan_and_its_line` as a pair.
pub fn the_plan_and_its_line(memory: &Memory) -> (String, String) {
    for turn in last_n(&memory.turns, 4).iter().rev() {
        let said = _matchable(&turn.user);
        if said.is_empty() || text::split_ref(&said).iter().all(|w| JUDGE_NOD_WORDS.contains(*w)) {
            continue;
        }
        let plan = _a_plan_phrase(&said);
        if !plan.is_empty() {
            return (plan, said);
        }
        if JUDGE_PLAN_OPENING.is_match(&said) || JUDGE_SHOULD_OPENING.is_match(&said) {
            return (String::new(), said);
        }
    }
    (String::new(), String::new())
}

/// Python: bot/compose/fallback.py:9708 `_the_reply_above_judged(memory, past_a_question=False)` (never called on the trace corpus)
/// True when the bot's last answer already said what the person should do about the plan.
pub fn _the_reply_above_judged(memory: &Memory, past_a_question: bool) -> bool {
    let said = text::strip(&crate::memory::last_bot(memory)).to_string();
    if said.is_empty() {
        return false;
    }
    if JUDGE_A_CARD_HEADING.match_(&said).is_some() {
        return false;
    }
    if said.ends_with('?') && !past_a_question {
        return true;
    }
    if JUDGE_JUDGEMENT.is_match(&said) || JUDGE_ANSWERS_FIRST.match_(&said).is_some() {
        return true;
    }
    let sentences: Vec<String> = rx(r"(?<=[.!?])\s+").split(&said).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    let last = sentences.last().cloned().unwrap_or_else(|| said.clone());
    text::split_ref(&last).len() <= 20 && JUDGE_DECISION_LEAD.match_(&last).is_some()
}

regexes!(TRIP_NAMED, NOT_A_PLACE, PLACE_EVALUATIVE, PLACE_OPENER);
pools!(A_PLACE_I_DONT_KNOW);
static A_FARE_OR_A_TICKET: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.the_rule_for_it.A_FARE_OR_A_TICKET"));
static RULE_PRICE_VERDICT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.the_rule_for_it.PRICE_VERDICT"));

/// Python: bot/compose/fallback.py:9767 `_a_place_asked_about(clean, memory)`
/// The pool for "is it nice" about a trip the person has just named, or None. (The skeleton's
/// `()` return could not carry the pool.)
pub fn _a_place_asked_about(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let said = _matchable(clean).trim_end_matches([' ', '?', '.', '!', ',']).to_string();
    if said.is_empty() || text::split_ref(&said).len() > 6 {
        return None;
    }
    if !PLACE_EVALUATIVE.is_match(&said) {
        return None;
    }
    if A_FARE_OR_A_TICKET.is_match(&said) && RULE_PRICE_VERDICT.is_match(&said) {
        return None;
    }
    if !(PLACE_OPENER.match_(&said).is_some() || clean.contains('?')) {
        return None;
    }
    for turn in last_n(&memory.turns, 3).iter().rev() {
        let mentioned = _matchable(&turn.user);
        if mentioned.is_empty() || text::split_ref(&mentioned).iter().all(|w| JUDGE_NOD_WORDS.contains(*w)) {
            continue;
        }
        if NOT_A_PLACE.is_match(&mentioned) {
            return None;
        }
        return if TRIP_NAMED.is_match(&mentioned) { Some(A_PLACE_I_DONT_KNOW.clone()) } else { None };
    }
    None
}

/// Python: bot/compose/fallback.py:9810 `a_verdict_on_their_plan(clean, memory, past_a_question=False)` (never called on the trace corpus)
/// The composer's verdict on the plan the person has just put down (a list of lines), or None.
pub fn a_verdict_on_their_plan(clean: &str, memory: &Memory, past_a_question: &Value) -> Value {
    match _a_verdict_on_their_plan(clean, memory, past_a_question.truthy()) {
        Some(lines) => Value::from(lines),
        None => Value::None,
    }
}

/// `[line.format(plan=plan, Plan=plan[:1].upper() + plan[1:]) for line in pool]`.
fn plan_lines(pool: &[String], plan: &str) -> Vec<String> {
    let cap = format!("{}{}", text::upper(&text::slice(plan, 0, Some(1))), text::slice(plan, 1, None));
    pool.iter().map(|l| format_named(l, &[("plan", plan), ("Plan", &cap)])).collect()
}

/// Python: bot/compose/fallback.py:9822 `_a_verdict_on_their_plan(clean, memory, past_a_question=False)`
/// The pool for "is that a stupid idea", or None when the line is not asking for a verdict. (The
/// skeleton's `()` return could not carry the pool.)
pub fn _a_verdict_on_their_plan(clean: &str, memory: &Memory, past_a_question: bool) -> Option<Vec<String>> {
    let said = _matchable(clean).trim_end_matches([' ', '?', '.', '!', ',']).to_string();
    let asked = match JUDGE_ASK.match_(&said) {
        Some(m) => m,
        None => {
            let bare = JUDGE_ASK_BARE.match_(&said)?;
            if bare.name_str("bare_noun").is_empty() && !JUDGE_BARE_FACES.contains(bare.name_str("face")) {
                return None;
            }
            if the_plan_and_its_line(memory).0.is_empty() {
                return None;
            }
            bare
        }
    };
    if _in_a_care_thread(memory) {
        return None;
    }
    let face = ["face", "face2", "face3"].iter().map(|g| asked.name_str(g)).find(|s| !s.is_empty()).map(lower).unwrap_or_default();
    if JUDGE_NOT_A_VERDICT.contains(&face) && !JUDGE_JUDGES_A_NOUN.is_match(&said) {
        return None;
    }
    let (mut plan, mut plan_line) = the_plan_and_its_line(memory);
    let stale = !plan_line.is_empty() && !_the_plan_is_the_last_thing_said(&plan_line, memory);
    if stale {
        plan = String::new();
        plan_line = String::new();
    }
    if !plan_line.is_empty() && (JUDGE_NAMES_A_HOLE.is_match(&plan) || JUDGE_NAMES_A_HOLE.is_match(&plan_line)) {
        if JUDGE_HOLE_SAFETY.is_match(&plan_line) || JUDGE_HOLE_SAFETY.is_match(&plan) {
            return None;
        }
        if JUDGE_ASKING_FOR_A_CAUTION.contains(&face) {
            return Some(plan_lines(&JUDGE_PLAN_HAS_A_HOLE, &plan));
        }
        return None;
    }
    if !plan.is_empty() || !plan_line.is_empty() {
        let wish = [plan.as_str(), plan_line.as_str()].iter().filter(|p| !p.is_empty()).copied().collect::<Vec<_>>().join(" ");
        if !JUDGE_A_TARGET.is_match(&wish) && (JUDGE_ACT_AT_SOMEONE.is_match(&wish) || JUDGE_TAKING.is_match(&wish)) {
            let said_back = if plan_line.is_empty() { Value::None } else { crate::compose::harm::reply_for(&plan_line, memory) };
            return Some(if said_back.truthy() { vec![said_back.str().to_string()] } else { JUDGE_WISH_ANGRY.clone() });
        }
        if JUDGE_WISHES_HARM.is_match(&wish) || JUDGE_TAKING_SOMETHING.is_match(&wish) || JUDGE_LYING.is_match(&wish) || JUDGE_NOT_A_PLAN_TO_BLESS.is_match(&wish) {
            return None;
        }
    }
    if !plan_line.is_empty() && plan.is_empty() {
        return None;
    }
    if !plan.is_empty() && _the_reply_above_judged(memory, past_a_question) {
        if !JUDGE_HOPED_FOR.is_match(&_matchable(&plan_line)) {
            return None;
        }
        plan = String::new();
    }
    if plan.is_empty() && !asked.name_str("face3").is_empty() {
        let last_user = memory.turns.last().map(|t| t.user.as_str()).unwrap_or("");
        if MIND_WORRY.is_match(&_matchable(last_user)) {
            return Some(JUDGE_NO_PLAN_MIND.clone());
        }
        return if stale { Some(JUDGE_NO_PLAN_MAD.clone()) } else { None };
    }
    if plan.is_empty() {
        if JUDGE_SELF_JUDGING.contains(&face) {
            return Some(JUDGE_NO_PLAN_MAD.clone());
        }
        if JUDGE_ONLY_A_PLAN.contains(&face) {
            return Some(JUDGE_NO_PLAN.clone());
        }
        if JUDGE_SOUNDS_FINE.contains(&face) {
            return Some(JUDGE_NO_PLAN_FINE.clone());
        }
        return None;
    }
    let pool: &[String] = if ["mad", "crazy", "insane", "nuts", "mental", "unhinged"].contains(&face.as_str()) {
        &JUDGE_SANE
    } else if ["good", "great", "sensible", "wise", "smart", "clever", "reasonable"].contains(&face.as_str()) {
        &JUDGE_FINE
    } else {
        &JUDGE_WORTH
    };
    Some(plan_lines(pool, &plan))
}

// ---- fallback.py:9953-10180: their favourite, the bot's tastes, venue hours, the day's news ----

regexes!(OWN_FAVOURITE, A_FAVOURITE_SAID);
pools!(OWN_FAVOURITE_LINES, THEIR_FAVOURITE_BACK, FAVOURITE_WORDS, ADVICE_WORDS, WHY_AFTER_TASTE, FAVOURITE_GENERAL, TASTE_GENERAL);
static TASTES: LazyLock<Vec<(Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.TASTES").items().iter().map(|p| (p.items()[0].strs(), p.items()[1].strs())).collect()
});

/// Python: bot/compose/fallback.py:9953 `_their_favourite_said(thing, memory)`
/// What the person said their favourite <thing> was in this chat, or None.
pub fn _their_favourite_said(thing: &str, memory: &Memory) -> Option<String> {
    if thing.is_empty() {
        return None;
    }
    for turn in last_n(&memory.turns, 12).iter().rev() {
        let said = _matchable(&turn.user);
        if let Some(found) = A_FAVOURITE_SAID.search(&said) {
            if found.str(1) == thing {
                let value = text::strip(found.str(2)).to_string();
                if !value.is_empty() && !text::slice(&value, 0, Some(1)).contains(' ') {
                    return Some(value);
                }
            }
        }
    }
    None
}

/// Python: bot/compose/fallback.py:9968 `_their_favourite_back(thing, value)` (never called on the trace corpus)
/// The pool above with the thing and the value the person gave filled in.
pub fn _their_favourite_back(thing: &str, value: &str) -> Vec<String> {
    let cap = format!("{}{}", text::upper(&text::slice(value, 0, Some(1))), text::slice(value, 1, None));
    THEIR_FAVOURITE_BACK.iter().map(|l| format_named(l, &[("value", value), ("Value", &cap)])).collect()
}

/// Python: bot/compose/fallback.py:9974 `_taste(clean, memory, was_taste, asking, repaired='')`
/// The bot's own answer to a question about its tastes, or None if it is not one. (The
/// skeleton's `()` return could not carry the reply.)
pub fn _taste(clean: &str, memory: &Memory, was_taste: bool, asking: bool, repaired: &str) -> Option<String> {
    if was_taste && _reads_as_why(clean) {
        return choose(&WHY_AFTER_TASTE, memory, clean);
    }
    if asking {
        if let Some(found) = OWN_FAVOURITE.search(clean) {
            let thing = found.get(1).filter(|s| !s.is_empty()).or_else(|| found.get(2)).unwrap_or("");
            if let Some(value) = _their_favourite_said(thing, memory) {
                return choose(&_their_favourite_back(thing, &value), memory, clean);
            }
            return choose(&OWN_FAVOURITE_LINES, memory, clean);
        }
    }
    let mut clean = clean;
    if !repaired.is_empty() && repaired != clean && !_has(clean, &OPINION_ASKS) && _has(repaired, &OPINION_ASKS) {
        clean = repaired;
    }
    let mut asked = _has(clean, &OPINION_ASKS);
    if asked && _is_about_the_persons_own_life(clean) {
        return None;
    }
    if !asked && _is_a_choice(clean) && !_has(&_choice_advice(clean), &ADVICE_WORDS) {
        asked = true;
    }
    if !asked && asking && TASTES.iter().any(|(phrases, _)| _has(clean, phrases)) {
        asked = true;
    }
    if !asked {
        return None;
    }
    if _is_about_the_persons_own_life(clean) && _has(clean, &FAVOURITE_WORDS) {
        return None;
    }
    for (phrases, options) in TASTES.iter() {
        if _has(clean, phrases) {
            return choose(options, memory, clean);
        }
    }
    if _is_a_choice(clean) {
        return choose(&_choice_pool(clean, memory), memory, clean);
    }
    if _has(clean, &FAVOURITE_WORDS) {
        return choose(&FAVOURITE_GENERAL, memory, clean);
    }
    choose(&TASTE_GENERAL, memory, clean)
}

pools!(LIVE_VENUE, LIVE_OPEN_CLOSE);
static LIVE_KINDS: LazyLock<Vec<(String, Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.LIVE_KINDS")
        .items()
        .iter()
        .map(|r| (r.items()[0].str().to_string(), r.items()[1].strs(), r.items()[2].strs()))
        .collect()
});
static VENUE_RE: LazyLock<Re> = LazyLock::new(|| {
    let mut words: Vec<String> = LIVE_VENUE.iter().map(|w| crate::py::re::escape(w)).collect();
    // sorted(LIVE_VENUE, key=len, reverse=True): longest first, ties in the tuple's order
    let mut order: Vec<(usize, usize)> = LIVE_VENUE.iter().enumerate().map(|(i, w)| (i, text::len(w))).collect();
    order.sort_by(|a, b| b.1.cmp(&a.1));
    words = order.iter().map(|(i, _)| crate::py::re::escape(&LIVE_VENUE[*i])).collect();
    Re::new(&format!("(?<![a-z])(?:{})(?:s|es)?(?![a-z])", words.join("|")), 0)
});

/// Python: bot/compose/fallback.py:10034 `_names_a_venue(clean)`
/// True when the line names one of the places in `LIVE_VENUE`, singular or plural.
pub fn _names_a_venue(clean: &str) -> bool {
    VENUE_RE.is_match(clean)
}

/// Python: bot/compose/fallback.py:10043 `venue_hours(text, memory=None)`
/// True when the line asks when a named place opens, closes or is open.
pub fn venue_hours(text_: &str, memory: &Memory) -> bool {
    let clean = lower(text::strip(text_));
    if !_names_a_venue(&clean) || !_has(&clean, &LIVE_OPEN_CLOSE) {
        return false;
    }
    for (kind, _phrases, options) in LIVE_KINDS.iter() {
        if kind == "hours" {
            return _live(&clean, true, memory).as_ref() == Some(options);
        }
    }
    false
}

/// Python: bot/compose/fallback.py:10061 `venue_hours_lines()` (never called on the trace corpus)
/// The composer's honest lines for a venue's opening hours (a frozenset).
pub fn venue_hours_lines() -> Value {
    for (kind, _phrases, options) in LIVE_KINDS.iter() {
        if kind == "hours" {
            let mut seen: Vec<Value> = Vec::new();
            for o in options {
                let v = Value::from(o.as_str());
                if !seen.contains(&v) {
                    seen.push(v);
                }
            }
            return Value::Set(std::sync::Arc::new(seen));
        }
    }
    Value::Set(std::sync::Arc::new(Vec::new()))
}

regexes!(LIVE_RESULT_ASK, LIVE_BARE_RESULTS, LIVE_HOW_IT_WENT, LIVE_TABLE_ASK, LIVE_LISTING_ASK, LIVE_WORLD_ASK, _WORLD_AS_SCOPE, _A_NEWS_WORD_BESIDE);
regexes!(LIVE_NOT_WORLDS, LIVE_A_YEAR, LIVE_A_BARE_WIN, LIVE_A_BARE_WINNING, LIVE_ANY_ACCIDENT);
pools!(LIVE_SPORT, LIVE_OWN_DRAW, LIVE_NEWS_LINES, LIVE_LISTING_LINES, LIVE_WHICH_RESULTS);

/// Python: bot/compose/fallback.py:10109 `_the_scope_of_a_standing_ask(clean)` (never called on the trace corpus)
pub fn _the_scope_of_a_standing_ask(clean: &str) -> bool {
    if !_WORLD_AS_SCOPE.is_match(clean) {
        return false;
    }
    !_A_NEWS_WORD_BESIDE.is_match(&_WORLD_AS_SCOPE.sub(" ", clean))
}

/// Python: bot/compose/fallback.py:10153 `_the_scores_line()` (never called on the trace corpus)
/// The one honest line about a score or a table (`sport._RESULTS_REPLY`), as a pool.
pub fn _the_scores_line() -> Vec<String> {
    vec![data::string("bot.knowledge.sport._RESULTS_REPLY")]
}

/// Python: bot/compose/fallback.py:10170 `_the_days_news(clean, asking)`
/// The honest line for the day's news, a result or a listing, or None. (The skeleton's `()`
/// return could not carry the pool.)
pub fn _the_days_news(clean: &str, asking: bool) -> Option<Vec<String>> {
    if LIVE_A_YEAR.is_match(clean) || LIVE_NOT_WORLDS.is_match(clean) || _has(clean, &LIVE_OWN_DRAW) || LIVE_A_BARE_WIN.is_match(clean) {
        return None;
    }
    let played = _has(clean, &LIVE_SPORT);
    if LIVE_LISTING_ASK.is_match(clean) && played {
        return Some(LIVE_LISTING_LINES.clone());
    }
    if LIVE_TABLE_ASK.is_match(clean) && played {
        return Some(_the_scores_line());
    }
    if LIVE_RESULT_ASK.is_match(clean) || (played && LIVE_HOW_IT_WENT.is_match(clean)) {
        if LIVE_A_BARE_WINNING.is_match(clean) {
            return Some(_the_scores_line());
        }
        if played {
            return Some(_the_scores_line());
        }
        if LIVE_BARE_RESULTS.is_match(clean) {
            return Some(LIVE_WHICH_RESULTS.clone());
        }
        return Some(LIVE_NEWS_LINES.clone());
    }
    if played && LIVE_TABLE_ASK.is_match(clean) {
        return Some(_the_scores_line());
    }
    if LIVE_WORLD_ASK.is_match(clean) && !_the_scope_of_a_standing_ask(clean) {
        return Some(LIVE_NEWS_LINES.clone());
    }
    None
}

// ---- fallback.py:10276-10735: what the bot cannot see, tastes as a reading, live information ----

/// What another module's reader returns, as Python's pool-or-None (a tuple of lines), whatever
/// type its port gives it (a reader whose skeleton says `()` is still a stub, and never returns).
pub trait PyOptPool {
    fn py_opt_pool(self) -> Option<Vec<String>>;
}
impl PyOptPool for () {
    fn py_opt_pool(self) -> Option<Vec<String>> {
        None
    }
}
impl PyOptPool for Vec<String> {
    fn py_opt_pool(self) -> Option<Vec<String>> {
        Some(self)
    }
}
impl PyOptPool for Option<Vec<String>> {
    fn py_opt_pool(self) -> Option<Vec<String>> {
        self
    }
}
impl PyOptPool for Value {
    fn py_opt_pool(self) -> Option<Vec<String>> {
        if self.is_none() { None } else { Some(self.strs()) }
    }
}

static SEEING_FAMILIES: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.no_eyes.SEEING_FAMILIES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static NO_EYES_CANNOT_SEE: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.no_eyes.CANNOT_SEE").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});

/// Python: bot/compose/fallback.py:10275 `_what_no_eyes_already_says(clean)` (never called on the trace corpus)
/// The line no_eyes already has for a named thing of theirs (a tuple value), or None.
pub fn _what_no_eyes_already_says(clean: &str) -> Value {
    match what_no_eyes_already_says(clean) {
        Some(pool) => Value::from(pool),
        None => Value::None,
    }
}

/// `_what_no_eyes_already_says` as lines.
pub fn what_no_eyes_already_says(clean: &str) -> Option<Vec<String>> {
    for (family, things) in SEEING_FAMILIES.iter() {
        let alts: Vec<String> = things.iter().map(|t| crate::py::re::escape(t)).collect();
        if rx(&format!(r"\b(?:{})\b", alts.join("|"))).is_match(clean) {
            return pool_of(&NO_EYES_CANNOT_SEE, family);
        }
    }
    None
}

regexes!(BOT_CANNOT, BOT_CANNOT_DO, BOT_CANNOT_LOOK, CANNOT_ANYTHING_ASK, SO_NOTHING_LIVE, CANNOT_READ_YOURS, CANNOT_SEE_ANYTHING);

/// Python: bot/compose/fallback.py:10289 `_a_no_from_the_bot(clean)`
/// True when the line only asks the bot to confirm it cannot do something.
pub fn _a_no_from_the_bot(clean: &str) -> bool {
    if SO_NOTHING_LIVE.is_match(clean) {
        return true;
    }
    BOT_CANNOT.is_match(clean) && BOT_CANNOT_DO.is_match(clean)
}

static CANNOT_THINGS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.CANNOT_THINGS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].strs())).collect()
});

/// Python: bot/compose/fallback.py:10307 `_the_live_thing_named(clean)` (never called on the trace corpus)
/// The live pool for the bare thing a denial names (a tuple value), or None.
pub fn _the_live_thing_named(clean: &str) -> Value {
    for (kind, words) in CANNOT_THINGS.iter() {
        if _has(clean, words) {
            for (name, _phrases, options) in LIVE_KINDS.iter() {
                if name == kind {
                    return Value::from(options.clone());
                }
            }
        }
    }
    Value::None
}

/// Python: bot/compose/fallback.py:10317 `_the_bot_says_it_cannot(clean)` (never called on the trace corpus)
pub fn _the_bot_says_it_cannot(clean: &str) -> bool {
    BOT_CANNOT.is_match(clean) && BOT_CANNOT_LOOK.is_match(clean)
}

/// Python: bot/compose/fallback.py:10327 `_taste_lines()` (never called on the trace corpus)
/// The taste lines that may take a favourite question off a card (the module keeps the result
/// as `TASTE_LINES`).
pub fn _taste_lines() -> HashSet<String> {
    let mut lines: HashSet<String> = FAVOURITE_GENERAL.iter().cloned().collect();
    for (phrases, options) in TASTES.iter() {
        if phrases.iter().any(|p| p.contains("favourite") || p.contains("favorite")) {
            lines.extend(options.iter().cloned());
        }
    }
    lines
}

word_sets!(TASTE_LINES);
regexes!(BESIDE_THEIR_OWN);

/// Python: bot/compose/fallback.py:10352 `reads_as_a_taste(text, memory=None)`
/// True when the line asks what the bot itself likes and the composer answers it as a taste.
pub fn reads_as_a_taste(text_: &str, memory: &Memory) -> bool {
    let mut clean = _matchable(text_);
    if clean.is_empty() {
        return false;
    }
    if !_has(&clean, &OPINION_ASKS) {
        let Some(read) = py_try(|| crate::understanding::understand(text_, Some(memory))) else { return false };
        clean = _matchable(&read.plain);
        if clean.is_empty() || !_has(&clean, &OPINION_ASKS) {
            return false;
        }
    }
    if _is_about_the_persons_own_life(&clean) {
        return false;
    }
    if BESIDE_THEIR_OWN.is_match(&clean) {
        return false;
    }
    if !_has(&clean, &FAVOURITE_WORDS) {
        return false;
    }
    let Some(reply) = py_try(|| _taste(&clean, memory, false, false, "")) else { return false };
    reply.is_some_and(|r| !r.is_empty() && TASTE_LINES.contains(&r))
}

regexes!(TRIP_ON_TABLE);

/// Python's `str()` of what `last_user(memory, n)` gives: the line, or the repr of the list.
fn py_str_of_lines(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::List(items) | Value::Tuple(items) => {
            let parts: Vec<String> = items.iter().map(|x| crate::understood::py_str_repr(x.as_str().unwrap_or(""))).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::None => String::new(),
        other => other.as_str().unwrap_or("").to_string(),
    }
}

/// Python: bot/compose/fallback.py:10397 `_a_trip_is_on_the_table(memory)` (never called on the trace corpus)
/// True when the ends of the trip are already in the chat.
pub fn _a_trip_is_on_the_table(memory: &Memory) -> bool {
    for step in [1i64, 2] {
        let said = lower(&py_str_of_lines(&crate::memory::last_user(memory, step)));
        if !said.is_empty() && TRIP_ON_TABLE.is_match(&said) {
            return true;
        }
    }
    false
}

word_sets!(LIVE_SKY_NOUNS, LIVE_SKY_WORDS, LIVE_SKY_PLACE, ROAD_TRAFFIC_WORDS);
pools!(LIVE_INDOOR);
regexes!(A_REPORT_OF_WHAT_HAPPENED, LIVE_ROAD_NAME);

/// Python: bot/compose/fallback.py:10442 `_the_sky_and_nothing_else(clean)`
pub fn _the_sky_and_nothing_else(clean: &str) -> bool {
    let words: Vec<String> = az_words(clean).iter().map(|w| text::strip_chars(w, "'").to_string()).collect();
    let nouns: Vec<String> = LIVE_SKY_NOUNS.iter().cloned().collect();
    if words.is_empty() || !_has(clean, &nouns) {
        return false;
    }
    let mut unknown = 0;
    let mut after_place = false;
    for word in &words {
        if LIVE_SKY_PLACE.contains(word) {
            after_place = true;
            unknown = 0;
            continue;
        }
        if LIVE_SKY_WORDS.contains(word) {
            continue;
        }
        if !after_place || LIVE_INDOOR.contains(word) {
            return false;
        }
        unknown += 1;
        if unknown > 2 {
            return false;
        }
    }
    true
}

/// Python: bot/compose/fallback.py:10497 `_a_road_named_with_traffic(clean)`
pub fn _a_road_named_with_traffic(clean: &str) -> bool {
    if !LIVE_ROAD_NAME.is_match(clean) {
        return false;
    }
    if A_REPORT_OF_WHAT_HAPPENED.is_match(clean) {
        return false;
    }
    text::split_ref(clean).iter().any(|w| ROAD_TRAFFIC_WORDS.contains(*w))
}

pools!(THEIR_OWN_COUNT_WORDS, LIVE_SITE_THING);
regexes!(A_COUNT_ASKED, THEIR_OWN_COUNT_HAPPENED, THEIR_OWN_COUNT_REPORT, A_COUNT_FELL);

/// Python: bot/compose/fallback.py:10526 `_their_own_count(clean)`
pub fn _their_own_count(clean: &str) -> bool {
    if !_has(clean, &LIVE_SITE_THING) || !_has(clean, &THEIR_OWN_COUNT_WORDS) {
        return false;
    }
    if A_COUNT_ASKED.is_match(clean) {
        return true;
    }
    !(THEIR_OWN_COUNT_HAPPENED.is_match(clean) || THEIR_OWN_COUNT_REPORT.is_match(clean))
}

/// Python: bot/compose/fallback.py:10555 `_their_own_thing(clean)` (never called on the trace corpus)
pub fn _their_own_thing(clean: &str) -> bool {
    let mine: Vec<String> = ["my", "our", "mine", "the shop", "the site"].iter().map(|s| s.to_string()).collect();
    _has(clean, &mine) || A_COUNT_ASKED.is_match(clean)
}

/// Python: bot/compose/fallback.py:10561 `_their_own_count_fell(clean)`
pub fn _their_own_count_fell(clean: &str) -> bool {
    if A_COUNT_ASKED.is_match(clean) {
        return false;
    }
    let mut words = THEIR_OWN_COUNT_WORDS.clone();
    words.push("sales".into());
    words.push("orders".into());
    _has(clean, &LIVE_SITE_THING) && _has(clean, &words) && A_COUNT_FELL.is_match(clean)
}

pools!(LIVE_NOT, LIVE_STRONG, LIVE_WEATHER_WORDS, LIVE_SKY, LIVE_SKY_OFF, LIVE_SETTING, LIVE_ROADS_NOT, LIVE_PAPER_WORDS, LIVE_ROAD_WORDS, LIVE_DELAY_WORDS);
pools!(LIVE_JOURNEY_ENDS, LIVE_TRAVEL_WORDS, LIVE_NOT_HOURS, OWN_PLACE, OWN_PERSON);
regexes!(LIVE_SKY_WHY, LIVE_HOURS_ASK, LIVE_YES_NO_OPEN, OWN_TIME_ASK, MY_TIME_ASK, OWN_VERB_OPEN, LIVE_WEATHER_DAY);

/// Python: bot/compose/fallback.py:10571 `_live(clean, asking, memory=None)`
/// The pool of honest replies for live information, or None.
pub fn _live(clean: &str, asking: bool, memory: &Memory) -> Option<Vec<String>> {
    if let Some(place) = crate::compose::place_facts::the_line(clean).py_opt_pool() {
        if !place.is_empty() {
            return Some(place);
        }
    }
    if _has(clean, &LIVE_NOT) {
        return None;
    }
    if memory.understood.as_ref().is_some_and(|u| u.over_time == Some(false)) {
        return None;
    }
    if let Some(lookup) = crate::compose::postcode::a_postcode_lookup(clean).py_opt_pool() {
        if !lookup.is_empty() {
            return Some(lookup);
        }
    }
    if ["news", "the news", "and the news"].contains(&clean.trim_matches([' ', '?', '!', '.'])) {
        return Some(LIVE_NEWS_LINES.clone());
    }
    if crate::compose::news_ask::reads_as_an_ask_for_the_news(clean) {
        return Some(LIVE_NEWS_LINES.clone());
    }
    if !asking
        && !_has(clean, &LIVE_STRONG)
        && !LIVE_ANY_ACCIDENT.is_match(clean)
        && !LIVE_A_BARE_WINNING.is_match(clean)
        && !_the_sky_and_nothing_else(clean)
        && !_a_road_named_with_traffic(clean)
        && !_their_own_count(clean)
        && !_their_own_count_fell(clean)
    {
        if RECORD_CHECK.is_match(clean) && _their_own_phrase(clean, &RECORD_THINGS).is_some() {
            if let Some(record) = _their_own_record(clean, false) {
                if !record.is_empty() {
                    return Some(record);
                }
            }
        }
        return None;
    }
    if let Some(mine) = _their_own_bill(clean).filter(|p| !p.is_empty()).or_else(|| _their_own_points(clean)) {
        if !mine.is_empty() {
            return Some(mine);
        }
    }
    for (kind, phrases, options) in LIVE_KINDS.iter() {
        let kind = kind.as_str();
        if kind == "site-stats" || kind == "site-stats-down" {
            if kind == "site-stats-down" {
                if _their_own_count_fell(clean) && _their_own_thing(clean) {
                    return Some(options.clone());
                }
                continue;
            }
            if _their_own_count(clean) && _their_own_thing(clean) {
                return Some(options.clone());
            }
            continue;
        }
        if kind == "weather" {
            if _has(clean, &LIVE_INDOOR) && !_has(clean, &LIVE_WEATHER_WORDS) {
                continue;
            }
            let wear = _has(clean, &LIVE_WEAR)
                && (if _has(clean, &LIVE_TAKE_ASK) { _a_bare_wear_ask(clean) } else { _has(clean, &LIVE_SOON) })
                && !_has(clean, &LIVE_SETTING);
            if _has(clean, phrases)
                || LIVE_WEATHER_DAY.is_match(clean)
                || (_has(clean, &LIVE_SKY) && !_has(clean, &LIVE_SKY_OFF) && !LIVE_SKY_WHY.is_match(clean))
                || wear
            {
                return Some(options.clone());
            }
            continue;
        }
        if kind == "news" && LIVE_A_BARE_WINNING.is_match(clean) {
            continue;
        }
        if kind == "news" && LIVE_NOT_WORLDS.is_match(clean) {
            continue;
        }
        if kind == "roads" {
            if _has(clean, &LIVE_ROADS_NOT) {
                continue;
            }
            if _has(clean, phrases) || (LIVE_ROAD_NAME.is_match(clean) && !_has(clean, &LIVE_PAPER_WORDS)) {
                return Some(options.clone());
            }
            if _has(clean, &LIVE_ROAD_WORDS) && _has(clean, &LIVE_DELAY_WORDS) {
                return Some(options.clone());
            }
        }
        if kind == "journey" {
            if _has(clean, phrases) && (_has(clean, &LIVE_JOURNEY_ENDS) || _a_trip_is_on_the_table(memory)) {
                return Some(options.clone());
            }
            continue;
        }
        if kind == "travel" {
            if _has(clean, &LIVE_ROADS_NOT) {
                continue;
            }
            if _has(clean, phrases) || (_has(clean, &LIVE_TRAVEL_WORDS) && _has(clean, &LIVE_DELAY_WORDS)) {
                return Some(options.clone());
            }
            if _has(clean, &LIVE_TRAVEL_WORDS) && (OWN_TIME_ASK.is_match(clean) || MY_TIME_ASK.is_match(clean)) {
                return Some(options.clone());
            }
            continue;
        }
        if kind == "prices" {
            let rate = crate::compose::quantity::the_rate_they_asked_about(clean, memory).py_opt_str().filter(|r| !r.is_empty());
            if let Some(rate) = &rate {
                if !crate::compose::quantity::the_thing_they_pay_for(clean).truthy() {
                    return Some(vec![rate.clone()]);
                }
            }
            if let Some(verdict) = _a_price_wants_a_verdict(clean, memory) {
                if !verdict.is_empty() {
                    return Some(verdict);
                }
            }
            let last = crate::memory::last_user(memory, 1);
            let before = lower(last.as_str().unwrap_or(""));
            if let Some(priced) = crate::compose::prices::the_price_reply(clean, phrases, &before, options).py_opt_pool() {
                if !priced.is_empty() {
                    return Some(priced);
                }
            }
            if let Some(rate) = rate {
                return Some(vec![rate]);
            }
            continue;
        }
        if kind == "hours" {
            if _has(clean, &LIVE_NOT_HOURS) {
                continue;
            }
            if _names_a_venue(clean) && (LIVE_HOURS_ASK.is_match(clean) || LIVE_YES_NO_OPEN.is_match(clean)) {
                return Some(options.clone());
            }
            if OWN_TIME_ASK.is_match(clean) {
                if !(_has(clean, &OWN_PLACE) || OWN_VERB_OPEN.is_match(clean)) || _has(clean, &OWN_PERSON) {
                    continue;
                }
            }
            if _has(clean, &LIVE_SPORT) && LIVE_LISTING_ASK.is_match(clean) {
                continue;
            }
        }
        if _has(clean, phrases) {
            return Some(options.clone());
        }
    }
    _the_days_news(clean, asking).filter(|p| !p.is_empty()).or_else(|| _their_own_record(clean, asking))
}

// ---- fallback.py:10822-10960 and 12984-13013: the ask for words, greetings, an emergency named ----

regexes!(WORDING_ASKS, WORDING_SAYS, WORDING_ASKS_WHO, WORDING_TELL, WORDING_SOMEONE, A_DOUBT_IN_FRONT, TELL_NOT_WORDS, WORDING_WHERE, GREETING_OPEN);
static THE_SAME_ASK_AS_A_STATEMENT: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.THE_SAME_ASK_AS_A_STATEMENT")
        .items()
        .iter()
        .map(|p| (p.items()[0].as_re().unwrap(), p.items()[1].str().to_string()))
        .collect()
});

/// Python: bot/compose/fallback.py:10818 `_the_ask_for_words_behind_a_doubt(clean)`
/// True when a doubt about how to say something is really the ask for words.
pub fn _the_ask_for_words_behind_a_doubt(clean: &str) -> bool {
    if !A_DOUBT_IN_FRONT.is_match(clean) || _reads_as_heavy_news(clean) {
        return false;
    }
    let mut said = clean.to_string();
    for (pattern, instead) in THE_SAME_ASK_AS_A_STATEMENT.iter() {
        said = pattern.sub(instead, &said);
    }
    if said == clean || TELL_NOT_WORDS.is_match(&said) {
        return false;
    }
    WORDING_ASKS.is_match(&said)
        || WORDING_SOMEONE.is_match(&said)
        || WORDING_TELL.is_match(&said)
        || (WORDING_SAYS.is_match(&said) && WORDING_WHERE.is_match(&said))
}

/// Python: bot/compose/fallback.py:10839 `_asks_for_words(said, words)`
/// True when this line is a person asking for the words to say.
pub fn _asks_for_words(said: &str, words: &[String]) -> bool {
    WORDING_ASKS.is_match(said)
        || WORDING_SOMEONE.is_match(said)
        || (WORDING_TELL.is_match(said) && !TELL_NOT_WORDS.is_match(said))
        || _the_ask_for_words_behind_a_doubt(said)
        || (WORDING_SAYS.is_match(said) && (WORDING_WHERE.is_match(said) || words.len() <= 6))
}

word_sets!(GREETING_WORDS, QUESTION_WORDS);

/// Python: bot/compose/fallback.py:10890 `_reads_as_greeting(words, text='')`
/// True when the line opens with a greeting.
pub fn _reads_as_greeting(words: &[String], text_: &str) -> bool {
    let plain: Vec<String> = words.iter().map(|w| w.replace('\'', "")).collect();
    if plain.is_empty() {
        return false;
    }
    if GREETING_WORDS.contains(&plain[0]) {
        return true;
    }
    let joined = plain.join(" ");
    let line = if text_.is_empty() { joined.as_str() } else { text_ };
    if GREETING_OPEN.match_(line).is_some() {
        let low = lower(text_);
        return !(crate::compose::goodnews::reads_as_good_news(&low) || crate::compose::badnews::reads_as_bad_news(&low));
    }
    let first_two = plain.iter().take(2).cloned().collect::<Vec<_>>().join(" ");
    ["good morning", "good afternoon", "good evening", "good day"].contains(&first_two.as_str())
}

/// Python: bot/compose/fallback.py:10911 `_reads_as_taste(clean, words)`
/// 'Cats or dogs?' is a choice between two things, not a fact anyone can settle.
pub fn _reads_as_taste(clean: &str, words: &[String]) -> bool {
    if !clean.contains(" or ") {
        return false;
    }
    if words.len() > 6 {
        return false;
    }
    !text::split_ref(clean).iter().any(|w| QUESTION_WORDS.contains(*w))
}

word_sets!(DARK_THING);
static BLACKED_OUT_SIGHT: LazyLock<Vec<(String, bool)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.BLACKED_OUT_SIGHT").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].truthy())).collect()
});
regexes!(BLACKED_OUT_A_THING, BLACKOUT_PERSON);

/// Python: bot/compose/fallback.py:10920 `_blackout_words(clean, words, said)`
/// True when the line describes a faint in words other than "blacked out".
pub fn _blackout_words(clean: &str, words: &[String], said: &str) -> bool {
    if _names(words, said, DARK_THING.iter()) || BLACKED_OUT_A_THING.is_match(clean) {
        return false;
    }
    let someone = BLACKOUT_PERSON.is_match(clean);
    for (shape, needs_person) in BLACKED_OUT_SIGHT.iter() {
        if rx(shape).is_match(clean) {
            return someone || !needs_person;
        }
    }
    false
}

regexes!(CANT_WAKE, ROUTINE_WAKE, UNRESPONSIVE_TAIL, FEED_TAIL, CHOKE_MACHINE, CHOKE_BODY, OVERDOSE_FOOD, OVERDOSE_DRUG, CHOKE_ON_A_WORD, OVERDOSE_OF_A_TREAT, OVERDOSE_A_PILL, FAINT_COMING);
regexes!(PASSED_OUT, PASSING_OUT_A_THING);
word_sets!(GADGET_WORDS, EMERGENCY_WORDS, FLOPPY_WAKE, TIRED, LANGUAGE_FLUENCY, GAS_WORDS, FUME_WORDS, FUEL_WORDS, PETROL_WORDS, LAUGHING_WORDS, CHILD_FAINT);
strings!(EMERGENCY_ANSWER, GAS_ANSWER, CHILD_FAINT_ANSWER, FAINT_COMING_ANSWER, PASSED_OUT_ANSWER);

/// Python: bot/compose/fallback.py:10936 `_action(clean, memory)`
/// A line with a right action gets it whatever its grammar: the line, or None. (The skeleton's
/// `()` return could not carry the reply.)
pub fn _action(clean: &str, memory: &Memory) -> Option<String> {
    let said = format!(" {} ", rx(r"[^a-z0-9']").sub(" ", clean));
    let words = text::split(&said.replace('\'', ""));
    let routine = ROUTINE_WAKE.is_match(clean);
    let story = crate::compose::urgent::reads_as_a_story(clean);
    if CANT_WAKE.is_match(clean) && !_names(&words, &said, GADGET_WORDS.iter()) && !routine && !story {
        return Some(EMERGENCY_ANSWER.clone());
    }
    let tail_only = UNRESPONSIVE_TAIL.is_match(clean) && !FEED_TAIL.is_match(clean);
    let choking_thing = CHOKE_MACHINE.is_match(clean) && !CHOKE_BODY.is_match(clean);
    if _names(&words, &said, EMERGENCY_WORDS.iter())
        && !_names(&words, &said, GADGET_WORDS.iter())
        && !choking_thing
        && !tail_only
        && !routine
        && !story
        && !(OVERDOSE_FOOD.is_match(clean) && !OVERDOSE_DRUG.is_match(clean))
        && !CHOKE_ON_A_WORD.is_match(clean)
        && !(OVERDOSE_OF_A_TREAT.is_match(clean) && !OVERDOSE_A_PILL.is_match(clean))
        && !_the_thing_is_choking(clean, &words)
    {
        return Some(EMERGENCY_ANSWER.clone());
    }
    if _names(&words, &said, FLOPPY_WAKE.iter()) && _names(&words, &said, TIRED.iter()) && !_names(&words, &said, GADGET_WORDS.iter()) {
        return Some(EMERGENCY_ANSWER.clone());
    }
    if !_names(&words, &said, LANGUAGE_FLUENCY.iter())
        && (_names(&words, &said, GAS_WORDS.iter()) || (_names(&words, &said, FUME_WORDS.iter()) && _names(&words, &said, FUEL_WORDS.iter())))
        && !_names(&words, &said, PETROL_WORDS.iter())
    {
        return Some(GAS_ANSWER.clone());
    }
    let animal = words.iter().any(|w| ANIMALS.contains(w) || ANIMALS.contains(w.trim_end_matches('s')));
    let went_under = (PASSED_OUT.is_match(clean) && !BLACKED_OUT_A_THING.is_match(clean) && !PASSING_OUT_A_THING.is_match(clean))
        || _blackout_words(clean, &words, &said);
    if went_under && !animal && !_names(&words, &said, LAUGHING_WORDS.iter()) {
        if _names(&words, &said, CHILD_FAINT.iter()) {
            return choose(&[CHILD_FAINT_ANSWER.clone()], memory, clean);
        }
        if FAINT_COMING.is_match(clean) {
            return choose(&[format!("{}.", *FAINT_COMING_ANSWER)], memory, clean);
        }
        return choose(&[format!("{}.", *PASSED_OUT_ANSWER)], memory, clean);
    }
    None
}

word_sets!(NOT_A_FORM, ENDINGS);

/// Python: bot/compose/fallback.py:12984 `_is_a_form(word, evidence_word)`
/// True when `word` is `evidence_word` with a grammatical ending on it.
pub fn _is_a_form(word: &str, evidence_word: &str) -> bool {
    let tail = text::slice(word, text::len(evidence_word) as isize, None);
    if tail.is_empty() {
        return true;
    }
    if NOT_A_FORM.contains(word) {
        return false;
    }
    if ENDINGS.contains(&tail) {
        return true;
    }
    let last = evidence_word.chars().last().map(String::from).unwrap_or_default();
    text::len(&tail) > 1 && text::slice(&tail, 0, Some(1)) == last && ENDINGS.contains(&text::slice(&tail, 1, None))
}

/// Python: bot/compose/fallback.py:13001 `_names(words, said, evidence)`
/// True when the line uses one of these words: whole, a phrase, or a longer form of it.
pub fn _names<'a>(words: &[String], said: &str, evidence: impl IntoIterator<Item = &'a String>) -> bool {
    for word in evidence {
        if word.contains(' ') {
            if said.contains(word.as_str()) {
                return true;
            }
        } else if words.iter().any(|w| w == word || (text::len(word) >= 4 && w.starts_with(word.as_str()) && _is_a_form(w, word))) {
            return true;
        }
    }
    false
}

// ---- fallback.py:11027-11203: a name given before, the move asked for, said it already ----

static GAVE_A_NAME: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| data::get("bot.compose.fallback.GAVE_A_NAME").items().iter().map(|v| v.as_re().unwrap()).collect());
static RECALL_NAME: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| data::get("bot.compose.fallback.RECALL_NAME").items().iter().map(|v| v.as_re().unwrap()).collect());
word_sets!(ASK_ONLY, POINTING, RULED_OUT_DETERMINER, PARTICLE);
regexes!(RECALL_BARE, MOVE_ASK);
pools!(SAME_AGAIN);

/// Python: bot/compose/fallback.py:11027 `_name_given_before(clean, memory)` (never called on the trace corpus)
/// The name the person gave a turn or two ago for this, as they wrote it, or "".
pub fn _name_given_before(clean: &str, memory: &Memory) -> String {
    let turns = last_n(&memory.turns, 2);
    if turns.is_empty() {
        return String::new();
    }
    let asked: HashSet<String> = text::split(&rx("[^a-z0-9 ]").sub(" ", &lower(clean))).into_iter().filter(|w| !ASK_ONLY.contains(w)).collect();
    for turn in turns.iter().rev() {
        let said = text::strip(&turn.user).to_string();
        if said.is_empty() {
            continue;
        }
        let mut name = String::new();
        let mut subject: HashSet<String> = HashSet::new();
        for rule in GAVE_A_NAME.iter() {
            let Some(found) = rule.search(&said) else { continue };
            let tail = if found.group_count() == 2 {
                subject = [lower(found.str(1))].into_iter().collect();
                text::strip(found.str(2)).to_string()
            } else {
                text::strip(found.str(1)).to_string()
            };
            let words = text::split(&tail.replace(',', " "));
            if words.is_empty() {
                continue;
            }
            name = words[words.len() - 1].clone();
            for w in &words[..words.len() - 1] {
                subject.insert(lower(w));
            }
            if words.len() > 1 && ["mr", "mrs", "ms", "miss", "dr"].contains(&lower(&words[0]).as_str()) {
                name = words[words.len() - 2..].join(" ");
            }
            break;
        }
        let lowered = rx("[^a-z]").sub("", &lower(&name));
        if name.is_empty() || NOT_A_NAME.contains(&lowered) || ASK_ONLY.contains(&lowered) {
            continue;
        }
        if asked.is_empty() || subject.is_empty() || asked.iter().any(|w| subject.contains(w)) {
            if !said.contains(&lower(&name)) {
                continue;
            }
            let bits: Vec<String> =
                text::split_ref(&name).iter().map(|b| format!("{}{}", text::upper(&text::slice(b, 0, Some(1))), text::slice(b, 1, None))).collect();
            return bits.join(" ");
        }
    }
    String::new()
}

/// Python: bot/compose/fallback.py:11071 `_asks_a_name_back(clean, words=())`
/// True when the line asks for a name of the person's own that nothing here knows.
pub fn _asks_a_name_back(clean: &str, words: &[String]) -> bool {
    let t = text::squash(&lower(clean));
    let split;
    let mine: &[String] = if words.is_empty() {
        split = text::split(&rx("[^a-z0-9' ]").sub(" ", &t));
        &split
    } else {
        words
    };
    if !mine.iter().any(|w| ["i", "we", "my", "our"].contains(&w.as_str())) {
        return false;
    }
    if mine.iter().any(|w| ["should", "shall", "could"].contains(&w.as_str())) {
        return false;
    }
    if WORD_ANYWHERE.iter().any(|p| t.contains(p.as_str())) {
        return false;
    }
    if RECALL_NAME.iter().any(|r| r.is_match(&t)) {
        return true;
    }
    RECALL_BARE.is_match(&t) && t.contains('?')
}

/// Python: bot/compose/fallback.py:11099 `_asks_for_the_move(line)`
/// True when the line asks what to do, rather than what the last answer said.
pub fn _asks_for_the_move(line: &str) -> bool {
    MOVE_ASK.is_match(line)
}

/// Python: bot/compose/fallback.py:11104 `_says_it_already(clean, words, memory)`
/// True when this line only asks again what the reply just above it answered.
pub fn _says_it_already(clean: &str, words: &[String], memory: &Memory) -> bool {
    if _asks_for_the_move(clean) {
        return false;
    }
    if !_answered_already(memory) || _hedged_recently(memory, 3) {
        return false;
    }
    if words.is_empty() || words.len() > 10 {
        return false;
    }
    let said: HashSet<String> = _flat_words(&_the_answer_above(memory)).into_iter().filter(|w| !POINTING.contains(w)).collect();
    if said.is_empty() {
        return false;
    }
    let mine: Vec<&String> = words.iter().filter(|w| !POINTING.contains(*w) && w.chars().count() > 2).collect();
    if mine.is_empty() {
        return false;
    }
    let missing: Vec<&String> = mine
        .iter()
        .copied()
        .filter(|w| !(said.contains(*w) || said.contains(w.trim_end_matches('s')) || said.contains(&format!("{w}s"))))
        .collect();
    let here = mine.len() - missing.len();
    if here == 0 || here * 2 < mine.len() {
        return false;
    }
    !missing.iter().any(|w| _names_a_new_thing(w, clean))
}

/// Python: bot/compose/fallback.py:11154 `_names_a_new_thing(word, clean)`
/// True when the line names `word` as a thing (or a plurality of something) of its own.
pub fn _names_a_new_thing(word: &str, clean: &str) -> bool {
    let words = text::split_ref(clean);
    for (i, here) in words.iter().enumerate() {
        if *here != word && here.trim_end_matches('s') != word {
            continue;
        }
        let before = if i > 0 { words[i - 1] } else { "" };
        let after = words.get(i + 1).copied().unwrap_or("");
        if RULED_OUT_DETERMINER.contains(before) && !PARTICLE.contains(after) {
            return true;
        }
        if here.ends_with('s') && text::len(here) > 3 && *here != words[0] {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:11170 `content_words(text)`
/// The words a line actually asks with: no pointing words, nothing of three letters or less.
pub fn content_words(text_: &str) -> Vec<String> {
    _flat_words(text_).into_iter().filter(|w| !POINTING.contains(w) && w.chars().count() > 2).collect()
}

/// Python: bot/compose/fallback.py:11176 `_flat_words(text)`
/// The words of a reply, lowercased and without punctuation, for comparing two of them.
pub fn _flat_words(text_: &str) -> Vec<String> {
    text::split(&rx("[^a-z0-9 ]").sub(" ", &lower(text_)))
}

/// Python: bot/compose/fallback.py:11181 `memory_bot(memory)` (never called on the trace corpus)
/// The reply the bot gave on the turn just before this one.
pub fn memory_bot(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|t| !t.bot.is_empty()).map(|t| t.bot.clone()).unwrap_or_default()
}

/// Python: bot/compose/fallback.py:11187 `says_it_already(text, memory)`
/// True when the composer's reply to this line is about the repeat, not a stock shrug.
pub fn says_it_already(text_: &str, memory: &Memory) -> bool {
    let read;
    let understood = match &memory.understood {
        Some(u) => u,
        None => {
            read = crate::understanding::understand(text_, Some(memory));
            &read
        }
    };
    let clean = if understood.clean.is_empty() { text::squash(&lower(text_)) } else { understood.clean.clone() };
    let words = if understood.words.is_empty() { text::split(&clean) } else { understood.words.clone() };
    py_try(|| _says_it_already(&clean, &words, memory)).unwrap_or(false)
}

/// Python: bot/compose/fallback.py:11206 `is_the_answer_shorter(reply, last)`
/// True when `reply` is the last answer said shorter by the composer.
pub fn is_the_answer_shorter(reply: &str, last: &str) -> bool {
    py_try(|| crate::compose::why::is_the_answer_shorter(reply, last)).unwrap_or(false)
}

/// Python: bot/compose/fallback.py:11221 `is_said_it_already(reply)`
/// True when `reply` is one of the said-it-already lines, in any of its wordings.
pub fn is_said_it_already(reply: &str) -> bool {
    let flat = _flat_words(reply).join(" ");
    HELP_ALREADY.iter().chain(SAME_AGAIN.iter()).any(|line| flat == _flat_words(line).join(" "))
}

// ---- fallback.py:11269-11690: a pointer at nothing, forecasts, the hedge's floor, in a pool ----

word_sets!(POINTING_WORDS, POINTERS, MEAN_WORDS, POINTING_ASK, POINTING_IDIOMS);

/// Python's `not memory` for a memory dict: nothing at all in it.
fn memory_is_empty(memory: &Memory) -> bool {
    memory.understood.is_none()
        && memory.intent_wants.is_none()
        && !memory.extra_had_intent
        && memory.asked_now.is_none()
        && memory.turns.is_empty()
        && memory.extra.is_empty()
}

/// Python: bot/compose/fallback.py:11269 `_points_at_nothing(clean, memory, repaired='')`
/// True when the line is only a pointer and there is nothing in the chat for it to be.
pub fn _points_at_nothing(clean: &str, memory: &Memory, repaired: &str) -> bool {
    for line in [clean, repaired] {
        let said = rx("[^a-z0-9' ]").sub(" ", &lower(line));
        let words = text::split(&said.replace('\'', ""));
        if words.is_empty() || words.len() > 6 {
            continue;
        }
        if !words.iter().all(|w| POINTING_WORDS.contains(w)) {
            continue;
        }
        let points = words.iter().any(|w| POINTERS.contains(w));
        let asks = words.iter().any(|w| MEAN_WORDS.contains(w));
        if !points && !asks && !POINTING_IDIOMS.contains(&words.join(" ")) {
            continue;
        }
        if !words.iter().any(|w| POINTING_ASK.contains(w)) && !asks {
            continue;
        }
        if (words.iter().any(|w| w == "you") || words.iter().any(|w| w == "u")) && !asks {
            continue;
        }
        return _nothing_to_point_at(memory);
    }
    false
}

/// Python: bot/compose/fallback.py:11292 `_nothing_to_point_at(memory)`
/// True when the chat has said nothing the pointer could be about.
pub fn _nothing_to_point_at(memory: &Memory) -> bool {
    if memory_is_empty(memory) {
        return true;
    }
    if in_a_safety_thread(memory) || _grief_thread(memory) || _thread_is_about_the_body(memory, 4) {
        return false;
    }
    if _answered_recently(memory) {
        return false;
    }
    if _a_previous_line_named_something(memory, &Value::None) {
        return false;
    }
    if crate::memory::last_bot(memory).contains('?') {
        return false;
    }
    true
}

/// Python: bot/compose/fallback.py:11318 `_a_previous_line_named_something(memory, look_back=3)` (never called on the trace corpus)
/// (`look_back`: an int, or None for the default.)
pub fn _a_previous_line_named_something(memory: &Memory, look_back: &Value) -> bool {
    last_n(&memory.turns, look_back_of(look_back, 3)).iter().any(|t| !crate::memory::topics::keywords(&t.user).is_empty())
}

regexes!(OFFERS_TO_FIX_THEMSELVES);

/// Python: bot/compose/fallback.py:11338 `_offers_to_fix_themselves(clean)` (never called on the trace corpus)
pub fn _offers_to_fix_themselves(clean: &str) -> bool {
    OFFERS_TO_FIX_THEMSELVES.is_match(clean)
}

static FORECAST_POOLS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get("bot.compose.fallback.FORECAST_POOLS").items().iter().map(Value::strs).collect());

/// Python: bot/compose/fallback.py:11342 `is_a_forecast(said)`
/// True when a reply is one of the forecast's own lines, which say no one can call it.
pub fn is_a_forecast(said: &str) -> bool {
    let head = text::strip(said);
    if head.is_empty() {
        return false;
    }
    FORECAST_POOLS.iter().flatten().any(|line| head.starts_with(line.split('{').next().unwrap_or("")))
}

regexes!(FORECAST_I_ASK, FORECAST_RELATION, FORECAST_HE_SHE, BODY_WORD_FOR_A_THING, FORECAST, FORECAST_OFF_THEIR_FOOD, FORECAST_A_FALL, FORECAST_STRUGGLE);
regexes!(FORECAST_PERMISSION, FORECAST_DURATION, FORECAST_PET_HARM);
word_sets!(FORECAST_BODY_WORDS, FORECAST_ILL);
pools!(FORECAST_AFTER_SAFETY, FORECAST_PET, FORECAST_PERSON, FORECAST_SOMEONE_STRUGGLING, FORECAST_SELF_STRUGGLING, FORECAST_SELF);

/// Python: bot/compose/fallback.py:11358 `_the_forecast_subject(clean, memory)` (never called on the trace corpus)
/// `(kind, who)`: ('person'|'self'|'pet', "your dad") for the one a forecast is about, or
/// (None, "").
pub fn _the_forecast_subject(clean: &str, memory: &Memory) -> (Option<String>, String) {
    let recent: Vec<String> = last_n(&memory.turns, 3).iter().map(|t| lower(&t.user)).collect();
    let asked_about_self = FORECAST_I_ASK.is_match(clean);
    let mut lines: Vec<&str> = vec![clean];
    if !asked_about_self {
        lines.extend(recent.iter().rev().map(String::as_str));
    }
    for said in lines {
        if let Some(found) = FORECAST_RELATION.search(said) {
            let relation = found.str(1);
            let kind = if ANIMALS.contains(relation) { "pet" } else { "person" };
            return (Some(kind.into()), format!("your {relation}"));
        }
    }
    if asked_about_self {
        return (Some("self".into()), "you".into());
    }
    if _thread_is_about_a_pet(memory, 3) {
        return (Some("pet".into()), "them".into());
    }
    let whose = _whose_body(clean, &recent);
    if whose == "mine" {
        return (Some("self".into()), "you".into());
    }
    if whose == "theirs" {
        return (Some("person".into()), "them".into());
    }
    if FORECAST_HE_SHE.is_match(clean) {
        return (Some("person".into()), "them".into());
    }
    if _a_body_in_the_thread(memory, 4) {
        return (Some("self".into()), "you".into());
    }
    if _thread_is_about_someone_else(memory, &Value::None) || _thread_is_about_a_person(memory, &Value::None) {
        return (Some("person".into()), "them".into());
    }
    (None, String::new())
}

/// Python: bot/compose/fallback.py:11397 `_without_the_things_named_after_a_body(text)`
/// The line with "back door" and "foot of the stairs" taken out before a body is read.
pub fn _without_the_things_named_after_a_body(text_: &str) -> String {
    BODY_WORD_FOR_A_THING.sub(" ", text_)
}

/// Python: bot/compose/fallback.py:11402 `_whose_body(clean, recent)` (never called on the trace corpus)
/// 'mine', 'theirs' or "" for the body a forecast is about. `recent` is the person's last
/// lines, oldest first.
pub fn _whose_body(clean: &str, recent: &[String]) -> String {
    let mut lines: Vec<&str> = vec![clean];
    lines.extend(recent.iter().rev().map(String::as_str));
    for said in lines {
        let tokens = az_words(&_without_the_things_named_after_a_body(&_without_the_forecast_verb(said)));
        for at in 0..tokens.len().saturating_sub(1) {
            if tokens[at] == "my" && FORECAST_BODY_WORDS.contains(&tokens[at + 1]) {
                return "mine".into();
            }
        }
        if tokens.iter().any(|t| FORECAST_BODY_WORDS.contains(t))
            && (tokens.iter().any(|t| _THREAD_PEOPLE.contains(t)) || FORECAST_HE_SHE.is_match(said))
        {
            return "theirs".into();
        }
    }
    String::new()
}

/// Python: bot/compose/fallback.py:11422 `_without_the_forecast_verb(line)`
/// The line with the forecast's own words taken out, so they cannot pass for a subject.
pub fn _without_the_forecast_verb(line: &str) -> String {
    match FORECAST.search(line) {
        Some(found) => text::strip(&format!("{} {}", &line[..found.start()], &line[found.end()..])).to_string(),
        None => line.to_string(),
    }
}

/// Python: bot/compose/fallback.py:11435 `_thread_said(memory, look_back=4)` (never called on the trace corpus)
/// The last few things the person said, as one line. (`look_back`: an int, or None for the
/// default.)
pub fn _thread_said(memory: &Memory, look_back: &Value) -> String {
    last_n(&memory.turns, look_back_of(look_back, 4)).iter().map(|t| lower(&t.user)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/fallback.py:11441 `_a_body_in_the_thread(memory, look_back=4)` (never called on the trace corpus)
/// True when the last few things said were about a body or a health worry.
pub fn _a_body_in_the_thread(memory: &Memory, look_back: i64) -> bool {
    for turn in last_n(&memory.turns, look_back) {
        let said = lower(&turn.user);
        let said = _without_the_things_named_after_a_body(&_without_the_forecast_verb(&said));
        let words = _without_the_idioms(&said);
        if words.iter().any(|w| BODY_PARTS.contains(w) || SYMPTOM_WORDS.contains(w) || HEALTH_WORDS.contains(w) || FORECAST_ILL.contains(w))
            || FORECAST_OFF_THEIR_FOOD.is_match(&said)
            || FORECAST_A_FALL.is_match(&said)
        {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:11462 `_a_hard_patch_in_the_thread(clean, memory, look_back=3)` (never called on the trace corpus)
/// True when the thread is somebody having a hard time rather than somebody ill.
pub fn _a_hard_patch_in_the_thread(clean: &str, memory: &Memory, look_back: i64) -> bool {
    let mut said = vec![lower(clean)];
    said.extend(last_n(&memory.turns, look_back).iter().map(|t| lower(&t.user)));
    said.iter().any(|l| !l.is_empty() && FORECAST_STRUGGLE.is_match(l))
}

/// `[phrase.format(who=who) for phrase in pool]`.
fn with_who(pool: &[String], who: &str) -> Vec<String> {
    pool.iter().map(|p| format_named(p, &[("who", who)])).collect()
}

/// Python: bot/compose/fallback.py:11470 `_forecast_pool(clean, repaired, memory)`
/// The pool for a line asking how something will turn out, or None. (The skeleton's `()`
/// return could not carry the pool.)
pub fn _forecast_pool(clean: &str, repaired: &str, memory: &Memory) -> Option<Vec<String>> {
    let line = [clean, repaired].into_iter().find(|s| !s.is_empty() && FORECAST.is_match(s)).unwrap_or("");
    if line.is_empty() || FORECAST_PERMISSION.is_match(line) || FORECAST_DURATION.is_match(line) {
        return None;
    }
    if _in_a_loss_thread(memory) || _grief_thread(memory) {
        return None;
    }
    let (kind, who) = _the_forecast_subject(line, memory);
    if in_a_safety_thread(memory) {
        return Some(with_who(&FORECAST_AFTER_SAFETY, if who.is_empty() { "you" } else { &who }));
    }
    match kind.as_deref() {
        Some("pet") => {
            if !(_a_body_in_the_thread(memory, 4) || FORECAST_PET_HARM.is_match(&_thread_said(memory, &Value::None))) {
                return None;
            }
            Some(with_who(&FORECAST_PET, &who))
        }
        Some("person") => {
            if an_outcome_asked_about_someone(clean, memory).is_some() {
                return None;
            }
            if _a_body_in_the_thread(memory, 4) {
                return Some(with_who(&FORECAST_PERSON, &who));
            }
            if _a_hard_patch_in_the_thread(line, memory, 3) {
                if _a_hard_time_and_not_a_body(memory, line) {
                    return None;
                }
                return Some(with_who(&FORECAST_SOMEONE_STRUGGLING, &who));
            }
            None
        }
        Some("self") => {
            let recent: Vec<String> = last_n(&memory.turns, 3).iter().map(|t| lower(&t.user)).collect();
            if _the_threads_who(memory, 2).is_some_and(|w| !w.is_empty()) {
                return None;
            }
            if _whose_body(clean, last_n(&recent, 1)) != "mine" && (_a_hard_patch_in_the_thread(line, memory, 3) || _thread_is_about_someone_else(memory, &Value::None)) {
                return Some(FORECAST_SELF_STRUGGLING.clone());
            }
            Some(FORECAST_SELF.clone())
        }
        _ => None,
    }
}

regexes!(POINTER_HEDGE, NAMES_WHERE_TO_ASK);

/// Python: bot/compose/fallback.py:11532 `_unknown(understood, memory, clean)`
/// The hedge for a line no pool could answer, with the floors that do better than a hedge.
pub fn _unknown(understood: &Understood, memory: &Memory, clean: &str) -> String {
    let reply = crate::compose::fallback_pools::_unknown(understood, memory, clean);
    let a_hedge = _is_a_hedge(&reply);
    if a_hedge || crate::compose::not_knowing::_asks_about_this_chat(clean) {
        let about_me = _a_question_about_the_bot(clean, memory);
        if !about_me.is_empty() {
            return about_me;
        }
        if !a_hedge {
            return reply;
        }
        let stuck = _what_to_do_about(clean, memory);
        if !stuck.is_empty() {
            return stuck;
        }
        let no_subject = crate::compose::not_knowing::_an_ask_with_no_subject(clean, memory);
        if !no_subject.is_empty() {
            return no_subject;
        }
        let lead = if POINTER_HEDGE.is_match(&reply) && !NAMES_WHERE_TO_ASK.is_match(&reply) {
            crate::compose::not_knowing::_the_lead_and_the_route(clean, memory)
        } else {
            String::new()
        };
        if !lead.is_empty() {
            return lead;
        }
    }
    reply
}

regexes!(TIPS_ASKS, NOT_A_TIPS_ASK);

/// Python: bot/compose/fallback.py:11585 `_is_a_tips_ask(line)`
/// True for a plea for tips in this spelling: the frame, without the phrase that is not one.
pub fn _is_a_tips_ask(line: &str) -> bool {
    TIPS_ASKS.is_match(line) && !NOT_A_TIPS_ASK.is_match(line)
}

pools!(PUNT, NOT_AN_ANSWER);

/// Python: bot/compose/fallback.py:11615 `_hedged_recently(memory, look_back=3)`
/// True when the bot has said it does not know, on any of the last few turns.
pub fn _hedged_recently(memory: &Memory, look_back: i64) -> bool {
    for turn in last_n(&memory.turns, look_back) {
        let said = text::strip(&turn.bot);
        if said.is_empty() {
            continue;
        }
        let head = text::slice(said, 0, Some(40));
        if UNKNOWN.iter().any(|line| head == text::slice(line, 0, Some(40))) {
            return true;
        }
        let low = lower(said);
        if PUNT.iter().any(|p| low.contains(p.as_str())) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:11668 `_in_a_pool(text, pools=(NOT_AN_ANSWER,))`
/// True when a reply is one of the pool lines, with or without its question after it. `pools`
/// is a tuple of pools (None for the default, `(NOT_AN_ANSWER,)`).
pub fn _in_a_pool(text_: &str, pools: &Value) -> bool {
    if pools.is_none() {
        return in_a_pool(text_, &[&NOT_AN_ANSWER[..]]);
    }
    let owned: Vec<Vec<String>> = pools.items().iter().map(Value::strs).collect();
    let refs: Vec<&[String]> = owned.iter().map(Vec::as_slice).collect();
    in_a_pool(text_, &refs)
}

/// `_in_a_pool` with the pools as lines.
pub fn in_a_pool(text_: &str, pools: &[&[String]]) -> bool {
    let said = _plain(text_);
    !said.is_empty()
        && pools.iter().any(|pool| {
            pool.iter().any(|line| {
                let p = _plain(line);
                !p.is_empty() && said.starts_with(&p)
            })
        })
}

// ---- fallback.py:11737-12095: what to do about it, questions about the bot, the floor, views ----

regexes!(WHAT_TO_DO_ABOUT, NOT_WHAT_TO_DO_ABOUT, THING_TAIL, ABOUT_THE_BOT_NAMED, ABOUT_THE_BOT_BORN, ABOUT_THE_BOT_TALK, ABOUT_THE_BOT_CLOCK);
pools!(HEALTH_STEMS, DITHER_OPEN_LINES, WHAT_TO_DO_ABOUT_FRAMES, BOT_FEELING_NAMED_LINES, BOT_BORN_LINES, BOT_TALK_LIMIT_LINES, BOT_TALK_CLOCK_LINES);

/// Python: bot/compose/fallback.py:11732 `_the_thing_alone(thing)` (never called on the trace corpus)
/// The thing as it can be said back: "the holiday i booked in august" -> "the holiday".
pub fn _the_thing_alone(thing: &str) -> String {
    let said = _yours(text::squash(thing).trim_matches([' ', ',', '.', ';', ':', '-']));
    let cut = THING_TAIL.sub("", &said).trim_matches([' ', ',', '.', ';', ':', '-']).to_string();
    if cut.is_empty() { said } else { cut }
}

/// Python: bot/compose/fallback.py:11749 `_what_to_do_about(clean, memory=None)`
/// The questions for a "what should i do about X" line, or "".
pub fn _what_to_do_about(clean: &str, memory: &Memory) -> String {
    let line = text::squash(clean);
    let Some(found) = WHAT_TO_DO_ABOUT.match_(&line) else { return String::new() };
    let thing = found.name_str("thing").trim_matches([' ', ',', '.', ';', ':', '-']).to_string();
    if thing.is_empty() || NOT_WHAT_TO_DO_ABOUT.is_match(&line) {
        return String::new();
    }
    let words = az_set(&lower(&line));
    if words.iter().any(|w| SYMPTOM_WORDS.contains(w) || BODY_PARTS.contains(w) || HEALTH_WORDS.contains(w) || HEALTH_STEMS.iter().any(|s| w.starts_with(s.as_str()))) {
        return String::new();
    }
    if text::split_ref(&lower(&thing)).first().is_some_and(|w| NOT_A_THING_ASKED.contains(*w)) {
        return String::new();
    }
    if text::split_ref(&thing).len() > *DISLIKE_TOO_LONG {
        return String::new();
    }
    let named = _capital(&_the_thing_alone(&thing));
    if named.is_empty() || text::split_ref(&lower(&named)).first().is_some_and(|w| NOT_A_THING_ASKED.contains(*w)) {
        return choose(&DITHER_OPEN_LINES, memory, clean).unwrap_or_default();
    }
    let low = lower(&named);
    let lines: Vec<String> = WHAT_TO_DO_ABOUT_FRAMES
        .iter()
        .map(|f| format_named(f, &[("thing", if f.starts_with("{thing}") { &named } else { &low })]))
        .collect();
    choose(&lines, memory, clean).unwrap_or_default()
}

/// Python: bot/compose/fallback.py:11860 `_a_question_about_the_bot(clean, memory=None)`
/// What the bot can say when the line puts a feeling or a start to the bot itself, or "".
pub fn _a_question_about_the_bot(clean: &str, memory: &Memory) -> String {
    let line = text::squash(clean);
    if line.is_empty() {
        return String::new();
    }
    if ABOUT_THE_BOT_CLOCK.match_(&line).is_some() {
        return choose(&BOT_TALK_CLOCK_LINES, memory, clean).unwrap_or_default();
    }
    if ABOUT_THE_BOT_TALK.match_(&line).is_some() {
        return choose(&BOT_TALK_LIMIT_LINES, memory, clean).unwrap_or_default();
    }
    if ABOUT_THE_BOT_BORN.match_(&line).is_some() {
        return choose(&BOT_BORN_LINES, memory, clean).unwrap_or_default();
    }
    let Some(found) = ABOUT_THE_BOT_NAMED.match_(&line) else { return String::new() };
    let raw_group = if !found.name_str("thing").is_empty() { found.name_str("thing") } else { found.name_str("thing2") };
    let raw = raw_group.trim_matches([' ', ',', '.', ';', ':', '-']);
    if text::split_ref(&lower(raw)).first().is_some_and(|w| ["your", "ur", "you", "yourself"].contains(w)) {
        return String::new();
    }
    let thing = _the_thing_alone(raw);
    if thing.is_empty() || text::split_ref(&lower(&thing)).first().is_some_and(|w| NOT_A_THING_ASKED.contains(*w)) {
        return String::new();
    }
    if text::split_ref(&thing).len() > *DISLIKE_TOO_LONG {
        return String::new();
    }
    let said = lower(&thing);
    let lines: Vec<String> = BOT_FEELING_NAMED_LINES.iter().map(|f| format_named(f, &[("thing", &said)])).collect();
    choose(&lines, memory, clean).unwrap_or_default()
}

regexes!(FLOOR_CLAUSE, FLOOR_COURTESY, FLOOR_WAITING);

/// Python: bot/compose/fallback.py:11891 `_only_the_floor(said)`
/// True when every part of the line is the floor or a courtesy on its way to it.
pub fn _only_the_floor(said: &str) -> bool {
    let said = _plain(said);
    if said.is_empty() {
        return false;
    }
    let parts: Vec<String> = rx(r"[.!?;,—–]|\s-\s")
        .split(&said)
        .iter()
        .map(|p| p.trim_matches([' ', ',', '.', ';', ':', '!', '?', '\u{2014}', '\u{2013}', '-', '\t']).to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() || parts.len() > 4 {
        return false;
    }
    for part in &parts {
        if FLOOR_COURTESY.match_(part).is_some() {
            continue;
        }
        if FLOOR_CLAUSE.match_(part).is_none() {
            return false;
        }
        if text::split_ref(part).len() > 8 && !FLOOR_WAITING.is_match(part) {
            return false;
        }
    }
    true
}

/// Python: bot/compose/fallback.py:11921 `_plain(text)`
/// The line in one case, with the dashes people type made the same. (The second of the two
/// definitions in the Python file, the one the module keeps.)
pub fn _plain(text_: &str) -> String {
    text::squash(&lower(text_).replace('\u{2014}', "-").replace('\u{2013}', "-"))
}

word_sets!(_ECHO_FILLER);

/// Python: bot/compose/fallback.py:11931 `_the_words(text)`
/// The words of a line that say something about its subject. (A set in Python; here in the order
/// the line says them, each once.)
pub fn _the_words(text_: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for w in az_words(&lower(text_)) {
        if !_ECHO_FILLER.contains(&w) && w.chars().count() > 2 && !out.contains(&w) {
            out.push(w);
        }
    }
    out
}

/// Python: bot/compose/fallback.py:11937 `_an_echo_of(question, sentence)` (never called on the trace corpus)
/// True when a sentence is only the person's own line said back, not a claim.
pub fn _an_echo_of(question: &str, sentence: &str) -> bool {
    let mine: HashSet<String> = _the_words(question).into_iter().collect();
    let theirs = _the_words(sentence);
    if theirs.len() < 2 || mine.is_empty() {
        return false;
    }
    let both = theirs.iter().filter(|w| mine.contains(*w)).count();
    both as f64 / theirs.len() as f64 >= 0.75
}

pools!(PRAISE, NEUTRAL, BARE_FOLLOW_UP_REPLIES);
word_sets!(VIEW_ONLY_WORDS);
regexes!(VIEW_ASKS, VIEW_NOT_AN_ANSWER, VIEW_NAMES_SOMEBODY);

/// Python: bot/compose/fallback.py:11959 `_only_a_reaction(module, said)` (never called on the trace corpus)
pub fn _only_a_reaction(module: &Value, said: &str) -> bool {
    if module.as_str() != Some("chat") {
        return false;
    }
    let said = text::strip(said);
    if !said.ends_with('?') || text::split_ref(said).len() > 14 {
        return false;
    }
    let low = lower(said);
    PRAISE.iter().any(|p| low.contains(p.as_str()))
}

/// Python: bot/compose/fallback.py:11968 `_the_answer_in_reply(question, said)` (never called on the trace corpus)
/// The reply without the receipt or reaction sentences in front of it (a str).
pub fn _the_answer_in_reply(question: &str, said: &str) -> Value {
    Value::from(the_answer_in_reply(question, said))
}

/// `_the_answer_in_reply` as a string.
pub fn the_answer_in_reply(question: &str, said: &str) -> String {
    let mut said = text::strip(said).to_string();
    for _ in 0..3 {
        let sentences = rx(r"(?<=[.!?])\s+").split(&said);
        let first = sentences.first().map(|s| text::strip(s).to_string()).unwrap_or_default();
        let rest = text::strip(&sentences[1..].join(" ")).to_string();
        if rest.is_empty() {
            return said;
        }
        if !(crate::memory::ack::reads_as_a_receipt(&first) || in_a_pool(&first, &[&NOT_AN_ANSWER[..]]) || _an_echo_of(question, &first)) {
            return said;
        }
        said = rest;
    }
    said
}

/// Python: bot/compose/fallback.py:11993 `_reads_as_a_claim(said)` (never called on the trace corpus)
/// True when the reply made a claim, so a view of it is the bot's own view.
pub fn _reads_as_a_claim(said: &str) -> bool {
    let said = text::strip(said);
    if said.is_empty() || VIEW_NOT_AN_ANSWER.match_(&lower(said)).is_some() {
        return false;
    }
    if crate::memory::ack::reads_as_a_receipt(said) {
        return false;
    }
    if in_a_pool(said, &[&NOT_AN_ANSWER[..]]) {
        return false;
    }
    if _only_the_floor(said) {
        return false;
    }
    let sentences = rx(r"(?<=[.!?])\s+").split(said);
    let first = text::strip(&sentences[0]);
    if first.ends_with('?') && first.contains(':') {
        return false;
    }
    true
}

/// Python: bot/compose/fallback.py:12029 `_nothing_to_stand_behind(said)`
/// True when the bot's line left the answer where it was, so the view is the one behind it.
pub fn _nothing_to_stand_behind(said: &str) -> bool {
    let said = text::strip(said);
    !said.is_empty() && (crate::memory::ack::reads_as_a_receipt(said) || in_a_pool(said, &[&NEUTRAL[..]]) || _only_the_floor(said))
}

/// Python: bot/compose/fallback.py:12048 `_is_a_view_ask(clean)`
/// True when the line asks for a view and names nothing of its own.
pub fn _is_a_view_ask(clean: &str) -> bool {
    if !VIEW_ASKS.is_match(clean) {
        return false;
    }
    let words = az_words(clean);
    !words.is_empty() && words.iter().all(|w| VIEW_ONLY_WORDS.contains(text::strip_chars(w, "'")))
}

/// Python: bot/compose/fallback.py:12062 `_the_view_asked_for(clean, memory)`
/// (Moved to fallback_pools; the reply or None. The skeleton's `()` return could not carry it.)
pub fn _the_view_asked_for(clean: &str, memory: &Memory) -> Option<String> {
    crate::compose::fallback_pools::_the_view_asked_for(clean, memory).py_opt_str()
}

/// Python: bot/compose/fallback.py:12067 `_after_a_colon(said)` (never called on the trace corpus)
/// The answer's first word, after a colon: "My view: find the water..." reads as one line (a str).
pub fn _after_a_colon(said: &str) -> Value {
    let mut chars = said.chars();
    let out = match chars.next() {
        Some(first) if text::len(said) > 1 && first.is_uppercase() && !said.starts_with("I ") && !said.starts_with("I'") => {
            format!("{}{}", text::lower(&first.to_string()), chars.as_str())
        }
        _ => said.to_string(),
    };
    Value::from(out)
}

/// Python: bot/compose/fallback.py:12095 `_is_an_ask_which_bit(said)`
/// True when a reply is the "tell me which bit" line rather than an answer or a hedge.
pub fn _is_an_ask_which_bit(said: &str) -> bool {
    let head = text::strip(said);
    !head.is_empty() && BARE_FOLLOW_UP_REPLIES.iter().any(|r| r == head)
}

// ---- fallback.py:12104-12659: hedges, an outcome asked about, views, more of it, tips, prices ----

static HEDGE_HEADS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.HEDGE_HEADS"));
static HEDGE_PREFIXES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.HEDGE_PREFIXES"));

/// Python: bot/compose/fallback.py:12104 `_is_a_hedge(said)`
/// True when a reply is one of the pools that mean the bot cannot answer.
pub fn _is_a_hedge(said: &str) -> bool {
    let head = text::slice(said, 0, Some(40));
    HEDGE_HEADS.iter().any(|line| head == *line) || HEDGE_PREFIXES.iter().any(|p| said.starts_with(p.as_str()))
}

regexes!(THE_OUTCOME_ASK, WHO_IS_ILL);
pools!(SAFETY_WORDS, OUTCOME_LINES);

/// The memory as the plain Python dict `compose.pick` takes in its contract.
fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// Python: bot/compose/fallback.py:12145 `an_outcome_asked_about_someone(text, memory)`
/// The line behind "is she going to be ok", answered honestly instead of hedged, or None. (The
/// skeleton's `()` return could not carry the line.)
pub fn an_outcome_asked_about_someone(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&lower(text_));
    if clean.is_empty() || az_words(&clean).len() > 9 {
        return None;
    }
    if THE_OUTCOME_ASK.match_(&clean).is_none() {
        return None;
    }
    if rx(r"\b(?:i|me|my|we|us|our)\b").is_match(&clean) {
        return None;
    }
    let turns = last_n(&memory.turns, 3);
    let last_turn = turns.last()?;
    let last = &last_turn.bot;
    if !last.is_empty() {
        let low = lower(last);
        if CRISIS_WORDS.iter().any(|w| low.contains(w.as_str())) || SAFETY_WORDS.iter().any(|w| low.contains(w.as_str())) {
            return None;
        }
    }
    let mut parts: Vec<&str> = turns.iter().map(|t| t.user.as_str()).collect();
    parts.push(last);
    if !WHO_IS_ILL.is_match(&parts.join(" ")) {
        return None;
    }
    Some(crate::compose::pick(&OUTCOME_LINES, &memory_value(memory), &clean))
}

/// Python: bot/compose/fallback.py:12174 `is_a_hedge(said)`
/// The public name for the test above, for the arbiter (compared without capitals).
pub fn is_a_hedge(said: &str) -> bool {
    let head = lower(&text::slice(said, 0, Some(40)));
    let low = lower(said);
    HEDGE_HEADS.iter().any(|line| head == lower(line)) || HEDGE_PREFIXES.iter().any(|p| low.starts_with(&lower(p)))
}

regexes!(VIEW_MIRROR, VIEW_A_CASE, ASKS_BACK);
word_sets!(VIEW_LEAD_IN);

/// Python: bot/compose/fallback.py:12210 `_the_view_of(said)` (never called on the trace corpus)
/// `(claim or None, mirrored)` as a tuple value: the first part of the answer that says
/// something, cut at a colon, and whether an opener was skipped.
pub fn _the_view_of(said: &str) -> Value {
    let (claim, mirrored) = the_view_of(said);
    Value::Tuple(std::sync::Arc::new(vec![claim.map(Value::from).unwrap_or(Value::None), Value::Bool(mirrored)]))
}

/// `_the_view_of` as a pair.
pub fn the_view_of(said: &str) -> (Option<String>, bool) {
    let parts: Vec<String> = rx(r"(?<=[.!?])\s+").split(text::strip(said)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    let mut mirrored = false;
    for part in parts {
        let (head, colon, tail) = text::partition(&part, ":");
        let mut chosen = part.clone();
        if !colon.is_empty() && !text::strip(tail).is_empty() {
            let words: Vec<String> = text::split_ref(head).iter().map(|w| lower(w.trim_matches([',', '.']))).collect();
            let swap = (words.len() < 7 && words.iter().any(|w| VIEW_LEAD_IN.contains(w)))
                || words.len() <= 3
                || VIEW_A_CASE.match_(&lower(text::strip(head))).is_some();
            chosen = if swap { tail.to_string() } else { head.to_string() };
        }
        let part = chosen;
        let part = text::strip(&part).to_string();
        if VIEW_MIRROR.match_(&lower(&part)).is_some() {
            mirrored = true;
            continue;
        }
        if let Some(claim) = _a_said_claim(&part) {
            return (Some(claim), mirrored);
        }
        mirrored = true;
    }
    (None, mirrored)
}

/// Python: bot/compose/fallback.py:12255 `_a_said_claim(first)` (never called on the trace corpus)
/// One sentence read as a claim to stand behind, or None when it says nothing.
pub fn _a_said_claim(first: &str) -> Option<String> {
    if first.is_empty() || first.ends_with(':') {
        return None;
    }
    if first.ends_with('?') {
        return if text::split_ref(first).len() >= 4 { Some(first.to_string()) } else { None };
    }
    if ASKS_BACK.match_(&lower(first)).is_some() {
        return None;
    }
    let words = text::split_ref(first);
    let mut first = first.to_string();
    if words.len() > 30 {
        let mut cut = 30;
        for index in 20..words.len().min(44) {
            if words[index].ends_with(',') {
                cut = index + 1;
                break;
            }
        }
        first = format!("{}.", words[..cut].join(" ").trim_end_matches(','));
    }
    if text::split_ref(&first).len() < 3 {
        return None;
    }
    Some(format!("{}.", first.trim_end_matches([' ', '.', ';', ',', ':'])))
}

regexes!(MORE_ON_THIS, WHAT_ABOUT, PERSON_NAMED);

/// Python: bot/compose/fallback.py:12313 `asks_for_more(text, memory)`
/// True when the line asks for more of the answer just given, about the same thing.
pub fn asks_for_more(text_: &str, memory: &Memory) -> bool {
    let said = lower(text::strip(text_));
    if !MORE_ON_THIS.is_match(&said) {
        return false;
    }
    let Some(tail) = WHAT_ABOUT.search(&said) else { return true };
    let named = tail.str(1);
    if PERSON_NAMED.match_(named).is_some() {
        return false;
    }
    _named_recently(named, memory, 3)
}

/// Python: bot/compose/fallback.py:12332 `_named_recently(named, memory, look_back=3)`
/// True when the words of the named subject are in the last few turns, either side.
pub fn _named_recently(named: &str, memory: &Memory, look_back: i64) -> bool {
    let wanted: HashSet<String> = crate::memory::topics::keywords(named).into_iter().collect();
    if wanted.is_empty() {
        return true;
    }
    let said: Vec<&str> = last_n(&memory.turns, look_back).iter().flat_map(|t| [t.user.as_str(), t.bot.as_str()]).collect();
    crate::memory::topics::keywords(&said.join(" ")).iter().any(|w| wanted.contains(w))
}

word_sets!(NO_MORE_OF);
regexes!(ACK_ONLY, REACTED_ONLY);

/// Python: bot/compose/fallback.py:12344 `_answered_recently(memory)`
/// True when the bot answered on the turn just before this one, from any module.
pub fn _answered_recently(memory: &Memory) -> bool {
    let turns = &memory.turns;
    if turns.is_empty() {
        return false;
    }
    let mut index = turns.len() - 1;
    while index > 0 && _only_takes_the_answer(&turns[index].bot) {
        index -= 1;
    }
    let last = &turns[index];
    if NO_MORE_OF.contains(last.module.as_deref().unwrap_or("")) {
        return false;
    }
    if !_told_them_something(&last.bot) {
        return false;
    }
    !crate::memory::history::not_a_subject(&last.user)
}

/// Python: bot/compose/fallback.py:12398 `_only_takes_the_answer(said)`
/// True when the bot's line only acknowledged what it was told, saying nothing of its own.
pub fn _only_takes_the_answer(said: &str) -> bool {
    let t = text::squash(said);
    if t.is_empty() || text::split_ref(&t).len() > 8 {
        return false;
    }
    ACK_ONLY.match_(&t).is_some()
}

/// Python: bot/compose/fallback.py:12416 `_told_them_something(said)`
/// True when the bot's last line said something, rather than asking back or only reacting.
pub fn _told_them_something(said: &str) -> bool {
    let t = text::strip(said);
    if t.is_empty() || t.ends_with('?') {
        return false;
    }
    REACTED_ONLY.match_(t).is_none()
}

/// The turns with the fact-back notes at the end taken off (`verdict.takes_a_fact_back`).
fn turns_past_the_notes(memory: &Memory) -> &[Turn] {
    let mut turns: &[Turn] = &memory.turns;
    while let Some(last) = turns.last() {
        if !crate::compose::verdict::takes_a_fact_back(&last.bot) {
            break;
        }
        turns = &turns[..turns.len() - 1];
    }
    turns
}

/// Python: bot/compose/fallback.py:12428 `_the_answer_above(memory)`
/// The bot's last real answer, as "the reply above" for a bare ask or a repeat.
pub fn _the_answer_above(memory: &Memory) -> String {
    turns_past_the_notes(memory).last().map(|t| t.bot.clone()).unwrap_or_default()
}

/// Python: bot/compose/fallback.py:12448 `_answered_already(memory)`
/// True when the bot answered on the turn just before this one.
pub fn _answered_already(memory: &Memory) -> bool {
    turns_past_the_notes(memory).last().is_some_and(|t| t.module.as_deref() == Some("answers"))
}

static TIPS_SPOKEN: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.TIPS_SPOKEN").items().iter().map(|p| (p.items()[0].as_re().unwrap(), p.items()[1].str().to_string())).collect()
});
regexes!(TIPS_NUMBER_TAIL, TIPS_SUBJECT_FOR, TIPS_SUBJECT_TAIL, TIPS_LEAD, TIPS_ANY_IDEA, TIPS_WORD, TIPS_REQUEST, TIPS_SOON);
word_sets!(TIPS_CLAUSE, TIPS_VERB, FILLERS);
pools!(TIPS_AHEAD, TIPS_THING);
static TIPS_OWNER: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.TIPS_OWNER").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

/// Python: bot/compose/fallback.py:12475 `_tips_spoken(text)`
/// The line with its chat-speak read as words, leaving a number that counts something alone.
pub fn _tips_spoken(text_: &str) -> String {
    let mut words = text::split(text_);
    for i in 0..words.len() {
        let tail = words.get(i + 1).cloned().unwrap_or_default();
        if text::is_digit(&words[i]) && TIPS_NUMBER_TAIL.match_(&tail).is_some() {
            continue;
        }
        let mut word = words[i].clone();
        for (pattern, said) in TIPS_SPOKEN.iter() {
            word = pattern.sub(said, &word);
        }
        words[i] = word;
    }
    words.join(" ")
}

/// Python: bot/compose/fallback.py:12488 `_tips_subject(clean)`
/// The thing a plea for tips is about, said back, or None when the line names no thing.
pub fn _tips_subject(clean: &str) -> Option<String> {
    let t = _tips_spoken(&text::squash(&lower(clean)));
    if !TIPS_WORD.is_match(&t) && !TIPS_ANY_IDEA.is_match(&t) {
        return None;
    }
    let spaced = format!(" {t}");
    let mut subject = match TIPS_SUBJECT_FOR.search(&spaced) {
        Some(found) => found.name_str("subject").to_string(),
        None => TIPS_SUBJECT_TAIL.sub("", &format!(" {t} ")),
    };
    subject = rx(r"[,.!?;:].*$").sub("", subject.trim_matches([' ', ',', '.', '!', '?', ';', ':']));
    subject = TIPS_LEAD.sub("", &text::squash(&subject));
    subject = rx(r"^(?:i|we)\s+(?:really\s+)?(?:need|want|would\s+like|am\s+looking\s+for)\s+").sub("", &subject);
    subject = TIPS_REQUEST.sub("", &TIPS_LEAD.sub("", &subject));
    subject = TIPS_LEAD.sub("", &subject);
    if subject.is_empty()
        || subject.starts_with("to ")
        || ["it", "that", "this", "some", "anything", "something", "a", "an", "the", "advice", "tips", "help"].contains(&subject.as_str())
    {
        return None;
    }
    let kept: Vec<&str> = text::split_ref(&subject).into_iter().filter(|w| !FILLERS.contains(*w)).collect();
    if kept.is_empty() || kept.len() > 6 || text::len(&kept.join(" ")) > 40 {
        return None;
    }
    if kept.iter().any(|w| TIPS_CLAUSE.contains(*w)) || TIPS_VERB.contains(kept[0]) {
        return None;
    }
    let owned: Vec<String> = kept.iter().map(|w| TIPS_OWNER.iter().find(|(k, _)| k == w).map(|(_, v)| v.clone()).unwrap_or_else(|| w.to_string())).collect();
    Some(owned.join(" "))
}

/// Python: bot/compose/fallback.py:12515 `_tips(memory, clean, repaired=None)`
/// A reply that turns a plea for tips into the one question that makes a tip possible, or "".
pub fn _tips(memory: &Memory, clean: &str, repaired: &str) -> String {
    let subject = _tips_subject(clean).or_else(|| if repaired != clean { _tips_subject(repaired) } else { None });
    let Some(subject) = subject else { return String::new() };
    let said = format!(" {} ", rx(r"[^a-z0-9']").sub(" ", &lower(clean)));
    let words = text::split(&said.replace('\'', ""));
    if _who_for(&lower(clean), &said, &words, None).is_some_and(|w| !w.is_empty()) {
        return String::new();
    }
    let pool: &[String] = if TIPS_SOON.is_match(&subject) { &TIPS_AHEAD } else { &TIPS_THING };
    choose(pool, memory, clean).unwrap_or_default()
}

regexes!(PRICE_VERDICT, MONEY_IN_A_LINE, A_BARE_FIGURE, A_MONEY_VERB, A_MEASURE, FOR_A_PERSON, SPLIT_BETWEEN_PEOPLE, A_PRICE_ASK);
pools!(PRICE_VERDICT_LINES);

/// Python: bot/compose/fallback.py:12625 `_the_figure_in(text)`
/// The sum as the person wrote it, or "" when there is no money in the line.
pub fn _the_figure_in(text_: &str) -> String {
    MONEY_IN_A_LINE.search(text_).map(|m| text::squash(m.as_str())).unwrap_or_default()
}

/// Python: bot/compose/fallback.py:12631 `_a_price_wants_a_verdict(clean, memory)`
/// The pool for a price asked to be judged, or None for any other line. (The skeleton's `()`
/// return could not carry the pool.)
pub fn _a_price_wants_a_verdict(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let said = text::squash(&lower(clean));
    if said.is_empty() || text::split_ref(&said).len() > 7 {
        return None;
    }
    if !A_PRICE_ASK.is_match(&said) || !PRICE_VERDICT.is_match(&said) {
        return None;
    }
    if FOR_A_PERSON.is_match(&said) || SPLIT_BETWEEN_PEOPLE.is_match(&said) || A_MEASURE.is_match(&said) {
        return None;
    }
    let mut figure = _the_figure_in(&said);
    if figure.is_empty() {
        figure = A_BARE_FIGURE.search(&said).map(|m| m.as_str().to_string()).unwrap_or_default();
    }
    if figure.is_empty() {
        let before = _the_line_before(memory);
        figure = _the_figure_in(&before);
        if figure.is_empty() {
            let squashed = text::squash(&before);
            let paid = A_MONEY_VERB.search(&squashed);
            figure = paid.and_then(|p| A_BARE_FIGURE.search(p.as_str()).map(|b| b.as_str().to_string())).unwrap_or_default();
        }
    }
    if figure.is_empty() {
        return None;
    }
    Some(PRICE_VERDICT_LINES.iter().map(|l| l.replace("{figure}", &figure)).collect())
}

// ---- fallback.py:12691-12981: who settles a line, the bad stretch, who to ring about it ----

word_sets!(GROWN_UP_WORDS, BABY_FOOD_NAMES, SELF_EATING_WORDS, CHILD_AT_HOME, GIFT_WORDS, MEDS_WORDS, BUYING_WORDS, NETWORK_WORDS);
word_sets!(SCHOOL_WORDS, SCHOOL_TROUBLE, PASSWORD_TROUBLE, COMPLAINT_WORDS, TRADER_WORDS, FILLING_WORDS, MOUTH_WORDS, FOOD_TOPICS, FILLING_FOODS);
word_sets!(WHERE_SOMETHING_WRONG, SICKNESS, JOINT_WORDS, WORK_ABOUT, HEALTH_VISIT_WORDS);
strings!(SCHOOL_WHO, DEVICE_WHO, COMPLAIN_WHO, DENTIST_WHO, BABY_WHO, PEST_WHO, VET_WHO, TRADES_WHO, TENANT_WHO, WORK_WHO);
regexes!(_ASKS_DEFINITION, _ASKS_WHERE, _SCHEDULING, _ASKS_ARITHMETIC, _ASKS_DIY, _CHOOSING_BETWEEN_TWO, _ASKS_ABOUT_THE_JOINT);
static WHO_TO_ASK: LazyLock<Vec<(HashSet<String>, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.WHO_TO_ASK").items().iter().map(|p| (p.items()[0].str_set(), p.items()[1].str().to_string())).collect()
});

/// Python: bot/compose/fallback.py:12691 `_a_grown_up_asks(clean, words, said)`
/// True when the baby in the line is a food's name or the line is about a grown-up.
pub fn _a_grown_up_asks(clean: &str, words: &[String], said: &str) -> bool {
    if _names(words, said, GROWN_UP_WORDS.iter()) {
        return true;
    }
    let mut left = format!(" {clean} ");
    let mut dropped = false;
    for name in BABY_FOOD_NAMES.iter() {
        let spaced = format!(" {name} ");
        if left.contains(&spaced) {
            left = left.replace(&spaced, " ");
            dropped = true;
        }
    }
    if !dropped || !_names(words, said, SELF_EATING_WORDS.iter()) {
        return false;
    }
    let left_words: Vec<String> = text::split(&left).into_iter().collect::<HashSet<String>>().into_iter().collect();
    !_names(&left_words, &left, CHILD_AT_HOME.iter())
}

/// Python: bot/compose/fallback.py:12712 `_who_for(clean, said, words, topic=None)`
/// Which person settles this line, or None.
pub fn _who_for(clean: &str, said: &str, words: &[String], topic: Option<&str>) -> Option<String> {
    if _ASKS_DEFINITION.is_match(clean) || _ASKS_ARITHMETIC.is_match(clean) {
        return None;
    }
    let diy = _ASKS_DIY.is_match(clean);
    let gift = _names(words, said, GIFT_WORDS.iter());
    let meds = _names(words, said, MEDS_WORDS.iter());
    let buying = _names(words, said, BUYING_WORDS.iter());
    if _names(words, said, SCHOOL_WORDS.iter()) && _names(words, said, SCHOOL_TROUBLE.iter()) {
        return Some(SCHOOL_WHO.clone());
    }
    if _names(words, said, [String::from("password")].iter()) && _names(words, said, PASSWORD_TROUBLE.iter()) {
        return Some(DEVICE_WHO.clone());
    }
    if _names(words, said, COMPLAINT_WORDS.iter()) && _names(words, said, TRADER_WORDS.iter()) {
        return Some(COMPLAIN_WHO.clone());
    }
    let mut animal = words.iter().any(|w| ANIMALS.contains(w) || ANIMALS.contains(w.trim_end_matches('s')));
    if animal && _names(words, said, WILD_BIRDS.iter()) && _names(words, said, MESS_PLACES.iter()) {
        animal = false;
    }
    let machine_mouse = crate::understanding::topics::is_a_machine_word(clean);
    if !animal && meds {
        return Some(HEALTH_WHO.clone());
    }
    if !animal
        && _names(words, said, FILLING_WORDS.iter())
        && (_names(words, said, MOUTH_WORDS.iter()) || !(topic.is_some_and(|t| FOOD_TOPICS.contains(t)) || _names(words, said, FILLING_FOODS.iter())))
    {
        return Some(DENTIST_WHO.clone());
    }
    let where_is_a_thing = _ASKS_WHERE.is_match(clean) && !_names(words, said, WHERE_SOMETHING_WRONG.iter());
    for (evidence, who) in WHO_TO_ASK.iter() {
        let who = who.as_str();
        if where_is_a_thing && (who == *BABY_WHO || who == *HEALTH_WHO) {
            continue;
        }
        if machine_mouse && who == *PEST_WHO {
            continue;
        }
        if animal && who != *VET_WHO && evidence.iter().any(|w| SICKNESS.contains(w)) {
            continue;
        }
        if animal && who == *DENTIST_WHO {
            continue;
        }
        if who == *HEALTH_WHO && _SCHEDULING.is_match(clean) {
            continue;
        }
        if who == *HEALTH_WHO && _CHOOSING_BETWEEN_TWO.is_match(clean) {
            continue;
        }
        if who == *HEALTH_WHO
            && _names(words, said, JOINT_WORDS.iter())
            && !(_names(words, said, evidence.iter().filter(|w| !JOINT_WORDS.contains(*w))) || _ASKS_ABOUT_THE_JOINT.is_match(clean))
        {
            continue;
        }
        if diy && (who == *TRADES_WHO || who == *DEVICE_WHO) {
            continue;
        }
        if who == *BABY_WHO && _a_grown_up_asks(clean, words, said) {
            continue;
        }
        if who == *BABY_WHO && !_names(words, said, CHILD_AT_HOME.iter()) {
            continue;
        }
        if gift && (who == *BABY_WHO || who == *HEALTH_WHO) {
            continue;
        }
        if meds && who == *DEVICE_WHO {
            continue;
        }
        if who == *DEVICE_WHO && _names(words, said, NETWORK_WORDS.iter()) {
            continue;
        }
        if buying && who == *TENANT_WHO {
            continue;
        }
        if who == *WORK_WHO && !_names(words, said, WORK_ABOUT.iter()) {
            continue;
        }
        if _names(words, said, evidence.iter()) {
            return Some(who.to_string());
        }
    }
    if animal && (meds || _names(words, said, SICKNESS.iter())) {
        return Some(VET_WHO.clone());
    }
    None
}

/// Python: bot/compose/fallback.py:12799 `who_settles(text, memory)` (never called on the trace corpus)
/// Which person settles this line - "the school office", "a vet" - or None (a str value).
pub fn who_settles(text_: &str, memory: &Memory) -> Value {
    match who_settles_line(text_, memory) {
        Some(who) => Value::from(who),
        None => Value::None,
    }
}

/// `who_settles` as an optional string.
pub fn who_settles_line(text_: &str, memory: &Memory) -> Option<String> {
    let read;
    let understood = match &memory.understood {
        Some(u) if u.text == text_ => u,
        _ => {
            let mut borrowed = memory.clone();
            borrowed.understood = None;
            read = crate::understanding::understand(text_, Some(&borrowed));
            &read
        }
    };
    let clean = if understood.clean.is_empty() { text::squash(&lower(text_)) } else { understood.clean.clone() };
    let words = understood.words.clone();
    let said = format!(" {} ", rx(r"[^a-z0-9']").sub(" ", &clean));
    _who_for(&clean, &said, &words, understood.topic.as_deref())
}

/// Python: bot/compose/fallback.py:12821 `_advisor(memory, clean, said, words)`
/// Name the person whose job settles this line, when the line names that world, or None.
pub fn _advisor(memory: &Memory, clean: &str, said: &str, words: &[String]) -> Option<String> {
    let topic = memory.understood.as_ref().and_then(|u| u.topic.as_deref());
    let who = _who_for(clean, said, words, topic)?;
    if who.is_empty() {
        return None;
    }
    let lines: Vec<String> = UNKNOWN_WHO.iter().map(|l| format_named(l, &[("who", &who)])).collect();
    choose(&lines, memory, clean)
}

regexes!(A_ROUGH_STRETCH, A_HARD_THING_HAPPENED, WHO_FOR_IT, WHO_TO_TALK_ASK);

/// Python: bot/compose/fallback.py:12859 `_a_bad_stretch_above(memory, look_back=3)` (never called on the trace corpus)
/// True when the last few things the person said were about a bad stretch of their own.
pub fn _a_bad_stretch_above(memory: &Memory, look_back: i64) -> bool {
    last_n(&memory.turns, look_back).iter().any(|t| {
        let said = lower(&t.user);
        !said.is_empty() && (A_ROUGH_STRETCH.is_match(&said) || A_HARD_THING_HAPPENED.is_match(&said))
    })
}

/// Python: bot/compose/fallback.py:12877 `_thread_has_a_health_visit(memory, look_back=3)` (never called on the trace corpus)
/// True when the last few things the person said were about a visit or a result coming.
pub fn _thread_has_a_health_visit(memory: &Memory, look_back: i64) -> bool {
    last_n(&memory.turns, look_back).iter().any(|t| rx("[a-z'-]+").findall_str(&lower(&t.user)).iter().any(|w| HEALTH_VISIT_WORDS.contains(w)))
}

/// Python: bot/compose/fallback.py:12887 `_in_a_crisis_thread(memory)`
/// True when the bot's own last word in this stretch named a crisis line.
pub fn _in_a_crisis_thread(memory: &Memory) -> bool {
    let answer = _thread_answer(memory);
    !answer.is_empty() && CRISIS_WORDS.iter().any(|w| answer.contains(w.as_str()))
}

static NAMED_BODIES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.NAMED_BODIES").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string())).collect()
});
pools!(WHO_FOR_IT_SAID_VOICE, WHO_NAMED_BEFORE, WHO_FOR_IT_SAID, WHO_FOR_IT_BODY, WHO_FOR_IT_ROUTES, WHO_TO_TALK_TO_A_BAD_STRETCH, WHO_TO_TALK_TO);

/// Python: bot/compose/fallback.py:12900 `_who_for_the_thread(memory, clean, understood=None)`
/// The person to take a bare "who do i call about it" to, or a reply asking what it is (lines),
/// or None. (The skeleton's `()` return could not carry the pool.)
pub fn _who_for_the_thread(memory: &Memory, clean: &str, understood: &Understood) -> Option<Vec<String>> {
    let typed = understood.text.as_str();
    let line = if typed.is_empty() { clean } else { typed };
    let a_voice = WHO_TO_TALK_ASK.is_match(line)
        && !rx(r"\babout\s+(?!it\b|that\b|this\b|them\b)\w|\b(?:talk|speak|chat)\s+(?:to|with)\s+(?:the|a|an|my|your|his|her|their)\b").is_match(line);
    let a_call = if !a_voice { WHO_FOR_IT.match_(line).is_some() } else { false };
    if !(a_call || a_voice) {
        return None;
    }
    let answer = lower(&crate::memory::last_bot(memory));
    if !answer.is_empty() {
        for (word, name) in NAMED_BODIES.iter() {
            if rx(&format!(r"\b{}\b", crate::py::re::escape(word))).is_match(&answer) {
                let shapes: &[String] = if a_voice { &WHO_FOR_IT_SAID_VOICE } else { &WHO_NAMED_BEFORE };
                return Some(with_who(shapes, name));
            }
        }
    }
    for turn in last_n(&memory.turns, 3).iter().rev() {
        let said = &turn.user;
        if said.is_empty() {
            continue;
        }
        let who = py_try(|| who_settles_line(said, memory)).flatten();
        if let Some(who) = who.filter(|w| !w.is_empty()) {
            let shapes: &[String] = if a_voice { &WHO_FOR_IT_SAID_VOICE } else { &WHO_FOR_IT_SAID };
            return Some(with_who(shapes, &who));
        }
    }
    if a_call && (_thread_is_about_the_body(memory, 4) || _thread_has_a_health_visit(memory, 3)) {
        return Some(WHO_FOR_IT_BODY.clone());
    }
    if a_call {
        return Some(WHO_FOR_IT_ROUTES.clone());
    }
    Some(if _a_bad_stretch_above(memory, 3) { WHO_TO_TALK_TO_A_BAD_STRETCH.clone() } else { WHO_TO_TALK_TO.clone() })
}

pools!(TROUBLE_CUES);

/// Python: bot/compose/fallback.py:12961 `_trouble(memory, clean, words)`
/// A statement of trouble still deserves the person who can settle it: the reply, or None. (The
/// skeleton's `()` return could not carry the reply.)
pub fn _trouble(memory: &Memory, clean: &str, words: &[String]) -> Option<String> {
    let said = format!(" {} ", words.join(" "));
    if !words.iter().any(|w| TROUBLE_CUES.iter().any(|cue| w.starts_with(cue.as_str()))) {
        return None;
    }
    if !_reads_as_question(clean, words) {
        let topic = memory.understood.as_ref().and_then(|u| u.topic.as_deref());
        if _who_for(clean, &said, words, topic).as_deref() == Some(HEALTH_WHO.as_str()) {
            return None;
        }
    }
    _advisor(memory, clean, &said, words)
}

// ---- fallback.py:13016-13654 (and 16191): questions, the bot itself, small talk, a tight week ----

static WORD_FIXES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.WORD_FIXES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});
word_sets!(_OPENING_QUESTIONS);
regexes!(INDIRECT, LATER_QUESTION, ASK_SHAPES);

/// `WORD_FIXES.get(word, word)`.
fn word_fixed(word: &str) -> String {
    WORD_FIXES.iter().find(|(k, _)| k == word).map(|(_, v)| v.clone()).unwrap_or_else(|| word.to_string())
}

/// Python: bot/compose/fallback.py:13016 `_reads_as_question(text, words)`
/// True when the line asks something, even if the understanding engine called it a statement.
pub fn _reads_as_question(text_: &str, words: &[String]) -> bool {
    if text_.contains('?') {
        return true;
    }
    let plain: Vec<String> = words.iter().map(|w| w.replace('\'', "")).collect();
    if let Some(first) = plain.first() {
        if QUESTION_WORDS.contains(&word_fixed(first)) {
            return true;
        }
    }
    let joined = INDIRECT.sub(" ", &format!(" {} ", plain.join(" ")));
    let later = INDIRECT.sub(" ", &lower(text_));
    LATER_QUESTION.is_match(&later) || ASK_SHAPES.is_match(&joined)
}

/// Python: bot/compose/fallback.py:13037 `_reads_as_a_question_when_repaired(text)`
/// The opening shape of the repaired line: a question word first, or an inversion.
pub fn _reads_as_a_question_when_repaired(text_: &str) -> bool {
    let plain: Vec<String> = text::split_ref(text_).iter().map(|w| w.replace('\'', "")).collect();
    if let Some(first) = plain.first() {
        if _OPENING_QUESTIONS.contains(&word_fixed(first)) {
            return true;
        }
    }
    ASK_SHAPES.is_match(&format!(" {} ", plain.join(" ")))
}

regexes!(HELP_TOPIC_ASK);
static HELP_TOPIC: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.HELP_TOPIC").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
pools!(HELP_TOPIC_ANY);

/// Python: bot/compose/fallback.py:13052 `_help_topic(clean, understood)`
/// A capability ask with a topic gets a line inside that topic (a pool), or None. (The
/// skeleton's `()` return could not carry the pool.)
pub fn _help_topic(clean: &str, understood: &Understood) -> Option<Vec<String>> {
    if !HELP_TOPIC_ASK.is_match(clean) {
        return None;
    }
    let topic = understood.topic_own.as_deref().unwrap_or("");
    if topic.is_empty() || ["about-the-bot", "small-talk", "opinions"].contains(&topic) {
        return None;
    }
    Some(pool_of(&HELP_TOPIC, topic).filter(|p| !p.is_empty()).unwrap_or_else(|| HELP_TOPIC_ANY.clone()))
}

pools!(ABOUT_ITSELF_KINDS);

/// Python: bot/compose/fallback.py:13069 `reads_as_about_itself(text, memory=None)`
/// True when the line is a question about the bot itself, for the arbiter's use.
pub fn reads_as_about_itself(text_: &str, memory: &Memory) -> bool {
    let clean = _self_line(text_);
    _about(&clean, None).is_some_and(|k| ABOUT_ITSELF_KINDS.contains(&k)) || FORGETTING_ASK.is_match(&clean)
}

regexes!(QUESTION_CLAUSE, A_THING_THAT_IS_SHORT, BARE_AND, _NOT_MANAGING, YOUR_OWN_TIME, HAND_BACK);
pools!(TIGHT_MONEY, TIGHT_MONEY_MORE);

/// Python: bot/compose/fallback.py:13113 `reads_as_a_tight_week(text, memory=None)`
/// True when the line only says the money has run out, for the arbiter's use.
pub fn reads_as_a_tight_week(text_: &str, memory: &Memory) -> bool {
    let mut context = memory.clone();
    context.understood = None;
    let Some(understood) = py_try(|| crate::understanding::understand(text_, Some(&context))) else { return false };
    if !understood.tight_on_money {
        return false;
    }
    let kind = if understood.kind.is_empty() { None } else { Some(understood.kind.as_str()) };
    match py_try(|| crate::understanding::topics::tight_on_money(&understood.norm, kind, "")) {
        Some(true) => {}
        _ => return false,
    }
    if QUESTION_CLAUSE.is_match(&_self_line(text_)) {
        return false;
    }
    if A_THING_THAT_IS_SHORT.is_match(&_self_line(text_)) {
        return false;
    }
    if py_try(|| crate::memory::more::asks_for_more(text_)) == Some(true) {
        return false;
    }
    true
}

/// Python: bot/compose/fallback.py:13169 `a_continuation_of_the_tight_week(memory, text)`
/// True when a bare "go on" follows the composer's own tight-week line.
pub fn a_continuation_of_the_tight_week(memory: &Memory, text_: &str) -> bool {
    if BARE_AND.match_(text_).is_none() {
        match py_try(|| crate::memory::more::asks_for_more(text_)) {
            Some(true) => {}
            _ => return false,
        }
    }
    let Some(last) = memory.turns.last() else { return false };
    let said = text::strip(&last.bot);
    TIGHT_MONEY.iter().chain(TIGHT_MONEY_MORE.iter()).any(|l| l == said)
}

/// Python: bot/compose/fallback.py:13186 `reads_as_small_talk(text, memory=None)`
/// True when the line is the manners question about the bot's own day, for the arbiter's use.
pub fn reads_as_small_talk(text_: &str, memory: &Memory) -> bool {
    let clean = _self_line(text_);
    if _about(&clean, None).is_some() {
        return false;
    }
    if _a_low_turn_above(memory) {
        return false;
    }
    if HAND_BACK.match_(&clean).is_some() {
        return !memory.turns.is_empty();
    }
    if !_asks_how_are_you(&clean, "") {
        return false;
    }
    if YOUR_DAY_ASK.is_match(&clean) && !(YOUR_OWN_TIME.is_match(&clean) || YOUR_TIME_ASK.is_match(&clean)) {
        // Python calls `_an_answer_above(memory)` here, and the name is bound to the later
        // definition, `_an_answer_above(turns)`: with a memory dict for `turns`, `turns[-1]` is a
        // KeyError for any memory that holds anything (and False for an empty one).
        if memory_is_empty(memory) {
            return false;
        }
        panic!("KeyError: -1");
    }
    YOUR_OWN_TIME.is_match(&clean) || YOUR_TIME_ASK.is_match(&clean)
}

/// Python: bot/compose/fallback.py:13257 `_a_low_turn_above(memory)`
/// True when the last turn or two were heavy, so the cheerful pool must not take the turn.
pub fn _a_low_turn_above(memory: &Memory) -> bool {
    if _hard_week_in_the_thread(memory) || _in_a_loss_thread(memory) || _grief_thread(memory) || _news_above(memory, false) {
        return true;
    }
    for earlier in last_n(&memory.turns, 2) {
        let line = text::squash(&lower(&earlier.user));
        if line.is_empty() {
            continue;
        }
        let read = crate::understanding::understand(&line, Some(&Memory::new()));
        if read.polarity < 0 {
            return true;
        }
        if _mood(&_words(&line), &line) == "negative" || _NOT_MANAGING.is_match(&line) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:16191 `_an_answer_above(turns)` (never called on the trace corpus)
/// Did the bot answer above this line, so that the no is how that answer went? (The second of
/// the two definitions in the Python file, the one the module keeps.)
pub fn _an_answer_above(turns: &[Turn]) -> bool {
    let bot = turns.last().map(|t| t.bot.as_str()).unwrap_or("");
    text::split_ref(bot).len() >= 8
}

/// Python: bot/compose/fallback.py:13307 `_a_repair_already_spoken_of(memory)` (never called on the trace corpus)
/// True when the bot's last line already said what a repair would involve.
pub fn _a_repair_already_spoken_of(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    if last.module.as_deref() != Some("answers") {
        return false;
    }
    rx(r"\b(?:repair|repairs|repaired|repairing|mend\w*|fix\w*|replace\w*|replacement|part|parts|quote|quoted|engineer|optician|servic\w+)\b").is_match(&lower(&last.bot))
}

regexes!(LEAD_IN);

/// Python: bot/compose/fallback.py:13343 `_self_line(text)`
/// The line as these readings see it: lowercase, no apostrophes, single spaces, no lead-in.
pub fn _self_line(text_: &str) -> String {
    let clean = text::squash(&lower(text_).replace('\u{2019}', "'").replace('\'', ""));
    let out = LEAD_IN.sub("", &clean);
    if out.is_empty() { clean } else { out }
}

/// Python: bot/compose/fallback.py:13349 `_a_tally_of_its_own_day(understood)`
pub fn _a_tally_of_its_own_day(understood: &Understood) -> bool {
    understood.counts_the_bots_own_day
}

static _SPELLED_SHORT: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback._SPELLED_SHORT").items().iter().map(|p| (p.items()[0].as_re().unwrap(), p.items()[1].str().to_string())).collect()
});

/// Python: bot/compose/fallback.py:13375 `_spelled_out(clean)`
/// The line with chat-speak spelled out, for the phrase tables in this file.
pub fn _spelled_out(clean: &str) -> String {
    let mut out = clean.to_string();
    for (pattern, word) in _SPELLED_SHORT.iter() {
        out = pattern.sub(word, &out);
    }
    out
}

regexes!(PRIVACY_ASK, PRIVACY_SHAPES, BETWEEN_US_ASK, NOT_WHAT_I_SAID, TRUST_ASK, SECRET_ASK, PROMISE_ASK, JUDGING_ASK, PRIVACY_NOT_THIS);
regexes!(LIMITS_ASK, ONLINE_ASK, OWN_DEVICE, A_BARE_ONLINE, WHAT_ARE_YOU, WHAT_ARE_YOU_NOUN, GOOD_AT_ASK, CLOTHES_ASK, LIKE_ASK, WORKINGS_ASK, ASKING_ASK);
regexes!(FORGETTING_ASK, RECALL_TAIL, HUMAN_ASK, HELP_ELSE, MODEL_NAMED);
pools!(IDENTITY, MEMORY);
/// The keys of a dict global (for `key in DICT`).
fn dict_keys(name: &str) -> HashSet<String> {
    data::get(name).pairs().iter().map(|(k, _)| k.str().to_string()).collect()
}
static ABOUT: LazyLock<HashSet<String>> = LazyLock::new(|| dict_keys("bot.compose.fallback.ABOUT"));
static ABOUT_SELF: LazyLock<HashSet<String>> = LazyLock::new(|| dict_keys("bot.compose.fallback.ABOUT_SELF"));
static MODEL_NAMED_WORDS: LazyLock<usize> = LazyLock::new(|| data::get("bot.compose.fallback.MODEL_NAMED_WORDS").as_int().unwrap() as usize);

/// Python's `string.punctuation`.
const PUNCTUATION: &[char] = &[
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
];

/// Python: bot/compose/fallback.py:13383 `_about(clean, understood=None)`
/// Which about-the-bot question this is, or None. (The skeleton's `()` return could not carry
/// the kind.)
pub fn _about(clean: &str, understood: Option<&Understood>) -> Option<String> {
    let clean = _self_line(clean);
    let self_ = _self_line(&clean);
    if (PRIVACY_ASK.is_match(&self_) || PRIVACY_SHAPES.is_match(&self_) || BETWEEN_US_ASK.is_match(&self_)) && !NOT_WHAT_I_SAID.is_match(&self_) {
        return Some("privacy".into());
    }
    if TRUST_ASK.is_match(&self_) {
        return Some("trust".into());
    }
    if SECRET_ASK.is_match(&self_) || PROMISE_ASK.is_match(&self_) {
        return Some("secret".into());
    }
    if JUDGING_ASK.is_match(&self_) {
        return Some("judging".into());
    }
    if PRIVACY_ASK.is_match(&clean) && !PRIVACY_NOT_THIS.is_match(&clean) && !NOT_WHAT_I_SAID.is_match(&clean) {
        return Some("privacy".into());
    }
    if TRUST_ASK.is_match(&clean) && !PRIVACY_NOT_THIS.is_match(&clean) {
        return Some("trust".into());
    }
    let selfed = _self_line(&clean);
    if LIMITS_ASK.is_match(&selfed) {
        return Some("limits".into());
    }
    if ONLINE_ASK.is_match(&selfed) && !OWN_DEVICE.is_match(&selfed) && !A_BARE_ONLINE.is_match(&clean) {
        return Some("online".into());
    }
    let manners: Vec<String> = text::split_ref(&selfed).iter().map(|w| w.trim_matches(PUNCTUATION).to_string()).collect();
    if crate::understanding::kinds::the_manners_question(&manners) {
        if let Some(u) = understood {
            if u.intent != "smalltalk" {
                return Some("help".into());
            }
        }
        return None;
    }
    let spelled = _spelled_out(&clean);
    if WHAT_ARE_YOU.is_match(&clean) || WHAT_ARE_YOU.is_match(&spelled) {
        return Some("identity".into());
    }
    if rx(r"\bwhat are you\b").is_match(&spelled) && _the_answers_take_it(&clean) != Some(true) {
        if WHAT_ARE_YOU_NOUN.is_match(&clean) {
            return Some("identity".into());
        }
        if GOOD_AT_ASK.is_match(&clean) {
            return Some("help".into());
        }
        if CLOTHES_ASK.is_match(&clean) {
            return Some("clothes".into());
        }
        if LIKE_ASK.is_match(&clean) {
            return Some("identity".into());
        }
        return Some("self_doing".into());
    }
    if WORKINGS_ASK.is_match(&clean) {
        return Some("workings".into());
    }
    if ASKING_ASK.is_match(&self_) {
        return Some("asking".into());
    }
    for phrase in IDENTITY.iter() {
        if clean.contains(phrase.as_str()) || spelled.contains(phrase.as_str()) {
            return Some("identity".into());
        }
    }
    if FORGETTING_ASK.is_match(&clean) {
        return Some("memory".into());
    }
    for phrase in MEMORY.iter() {
        if clean.contains(phrase.as_str()) || spelled.contains(phrase.as_str()) {
            if RECALL_TAIL.is_match(&clean) && _the_store_takes_it(&clean) {
                return None;
            }
            return Some("memory".into());
        }
    }
    if let Some(u) = understood {
        if let Some(kind) = u.about_bot_kind.as_deref() {
            if ABOUT.contains(kind) || ABOUT_SELF.contains(kind) {
                return Some(kind.to_string());
            }
        }
    }
    if let Some(asked) = HUMAN_ASK.search(&clean) {
        if _opens_the_line(&clean[..asked.start()]) && !_asks_elsewhere(&clean[asked.end()..]) {
            return Some("identity".into());
        }
    }
    if _asks_for_a_person_to_talk_to(&clean) {
        return Some("identity".into());
    }
    if let Some(matched) = HELP_ELSE.search(&clean) {
        let tail = matched.get(1).filter(|s| !s.is_empty()).or_else(|| matched.get(2)).unwrap_or("");
        if _free_tail(tail) {
            return Some("help".into());
        }
    }
    if _asks_for_help(&clean, understood) {
        return Some("hand".into());
    }
    _the_bot_itself(&clean)
}

/// Python: bot/compose/fallback.py:13520 `_names_the_make(clean)`
/// Whether the line names what the bot is made of, in a line short enough to be only that.
pub fn _names_the_make(clean: &str) -> bool {
    let said = az09_words(&lower(clean));
    said.len() <= *MODEL_NAMED_WORDS && MODEL_NAMED.is_match(&said.join(" "))
}

/// Python: bot/compose/fallback.py:13526 `about_the_bot(clean)`
/// The composer's reading of the line as a question about the bot itself, or None. (The
/// skeleton's `()` return could not carry the kind.)
pub fn about_the_bot(clean: &str) -> Option<String> {
    _the_bot_itself(clean)
}

regexes!(DOUBT_OPENER, DOUBT_TAIL, DOUBT_NOT, DOUBT_NOT_BE);

/// Python: bot/compose/fallback.py:13561 `_a_doubt_about_the_bot(clean)`
/// Which question about the bot this doubt is asking, or None. (The skeleton's `()` return
/// could not carry the kind.)
pub fn _a_doubt_about_the_bot(clean: &str) -> Option<String> {
    let mut line = clean.to_string();
    for _ in 0..3 {
        let stripped = DOUBT_OPENER.sub("", &line);
        if stripped == line {
            break;
        }
        line = stripped;
    }
    line = DOUBT_TAIL.sub("", &line);
    let asked = if let Some(notted) = DOUBT_NOT.search(&line) {
        Some(format!("can you {}", text::strip(&line[notted.end()..])))
    } else {
        DOUBT_NOT_BE.search(&line).map(|n| format!("are you {}", text::strip(&line[n.end()..])))
    };
    let asked = asked?;
    if asked.is_empty() || asked.ends_with("you") || asked.ends_with("you can") || asked.ends_with("you are") {
        return None;
    }
    _about(&asked, None)
}

/// Python: bot/compose/fallback.py:13583 `_the_bot_itself(clean)`
/// The bots-own-kind reading, with the doubt shape read as the question it is asking. (The
/// skeleton's `()` return could not carry the kind.)
pub fn _the_bot_itself(clean: &str) -> Option<String> {
    let clean = _self_line(clean);
    _about_itself(&clean).or_else(|| _a_doubt_about_the_bot(&clean))
}

regexes!(ABOUT_THE_PERSON, REFERS_BACK, NAMES_THE_BOT, SELF_ASK, A_VIEW_ASKED_FOR, SELF_OPENER, REGRET_ASK, SELF_PURPOSE, MADE_ASK, MODEL_ASK);
word_sets!(SELF_FILLER, PICKS_A_MECHANISM, SELF_WORDS);
static SELF_GROUP: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.SELF_GROUP").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

/// Python: bot/compose/fallback.py:13603 `_about_itself(clean)`
/// Which question about the bot's own nature this is, or None. (The skeleton's `()` return
/// could not carry the kind.)
pub fn _about_itself(clean: &str) -> Option<String> {
    if REFERS_BACK.is_match(clean) && !NAMES_THE_BOT.is_match(clean) {
        return None;
    }
    let said: Vec<String> = text::split_ref(clean).iter().map(|w| w.trim_matches(['?', '!', '.', ',', ';', ':', '"', '\'', '(', ')']).to_string()).collect();
    if REGRET_ASK.is_match(clean) {
        return Some("regret".into());
    }
    if SELF_PURPOSE.is_match(clean) {
        return Some("purpose".into());
    }
    if MADE_ASK.is_match(clean) {
        return Some("agency".into());
    }
    if MODEL_ASK.is_match(clean) || _names_the_make(clean) || WORKINGS_ASK.is_match(clean) {
        return Some("model".into());
    }
    let selfed = _self_line(clean);
    if LIMITS_ASK.is_match(&selfed) {
        return Some("limits".into());
    }
    if ONLINE_ASK.is_match(&selfed) && !OWN_DEVICE.is_match(&selfed) && !A_BARE_ONLINE.is_match(clean) {
        return Some("online".into());
    }
    if !SELF_ASK.is_match(clean) {
        return None;
    }
    if let Some(view) = A_VIEW_ASKED_FOR.match_(clean) {
        let you = rx(r"\s*(?:you|u|youre|you're|ur|yourself|for\s+yourself|about\s+yourself|by\s+yourself|on\s+your\s+own)\b");
        if you.match_(view.str(1)).is_none() {
            return None;
        }
    }
    if ABOUT_THE_PERSON.is_match(clean) {
        return None;
    }
    let rest: Vec<&String> = said.iter().filter(|w| !SELF_FILLER.contains(*w) && !["you", "u", "youre", "you're", "ur"].contains(&w.as_str())).collect();
    if rest.is_empty() {
        if rx(r"\bwhy\b").is_match(clean) {
            return Some("purpose".into());
        }
        return if SELF_OPENER.is_match(clean) { Some("where".into()) } else { None };
    }
    if rest.iter().any(|w| !SELF_WORDS.contains(*w)) {
        return None;
    }
    for word in rest {
        if PICKS_A_MECHANISM.contains(word) && !rx(r"\bhow\b").is_match(clean) {
            continue;
        }
        let group = SELF_GROUP.iter().find(|(k, _)| k == word).map(|(_, v)| v.as_str());
        if group == Some("where") && (word == "here" || word == "there") && rx(r"\bwhy\b").is_match(clean) {
            return Some("purpose".into());
        }
        if let Some(group) = group {
            if !group.is_empty() {
                return Some(group.to_string());
            }
        }
    }
    None
}

// ---- fallback.py:13665-14003: a burn, a hazard, keeping a care thread, fear, somebody else ----

regexes!(BURN_BLISTER_LINE, BURN_ON_IT, POP_A_BARE_IT, BURN_WORD, BURN_THING, BURN_ASK);

/// Python: bot/compose/fallback.py:13674 `_burn_line(clean, thread='')`
/// True for a line inside a burn thread that is about the burn.
pub fn _burn_line(clean: &str, thread: &str) -> bool {
    if BURN_BLISTER_LINE.is_match(clean) || BURN_ON_IT.is_match(clean) {
        return true;
    }
    if POP_A_BARE_IT.is_match(clean) && (BURN_WORD.is_match(&lower(thread)) || BURN_WORD.is_match(clean)) {
        return true;
    }
    BURN_THING.is_match(clean) && BURN_ASK.is_match(clean)
}

regexes!(_HAZARD_WORD, _FETCH_FROM_INSIDE, _GAS_WORD, _FIRE_WORD, _ASKS_TO_LEAVE, _THE_WAY_OUT, HOUSE_FAULT, CARE_ASKS, _POINTER_OWN_OBJECT, ALREADY_RANG, CARE_SEEN_ASKS);

/// Python: bot/compose/fallback.py:13762 `_the_hazard_is_gas(line)`
/// True when the emergency this thread opened on is a leak rather than a fire.
pub fn _the_hazard_is_gas(line: &str) -> bool {
    _GAS_WORD.is_match(line) && !_FIRE_WORD.is_match(line)
}

/// Python: bot/compose/fallback.py:13768 `_goes_back_for(clean, thread='')`
/// True when the line asks to fetch something out of the place a hazard is burning in.
pub fn _goes_back_for(clean: &str, thread: &str) -> bool {
    if !(_HAZARD_WORD.is_match(clean) || _HAZARD_WORD.is_match(thread)) {
        return false;
    }
    _FETCH_FROM_INSIDE.is_match(clean)
}

static TROUBLE_BODY_PART: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.trouble.BODY_PART"));
word_sets!(CARE_POINTER, CARE_PRONOUNS, CARE_HURTS, CARE_INJURY, CARE_WORDS, CARE_RESOLVED, CARE_NOT_RESOLVED, REACTION_WORDS, GENERIC_WORDS, AGREEMENTS, ASKING_WORDS);
pools!(CARE_PHRASES);

/// Python: bot/compose/fallback.py:13775 `_names_a_new_part(words, clean, thread)` (never called on the trace corpus)
/// True when the line complains about a part of the body this thread never mentioned. (`words`
/// is a set in Python; the thread's own line is `thread`.)
pub fn _names_a_new_part(words: &[String], clean: &str, thread: &str) -> bool {
    let part: Vec<&String> = words.iter().filter(|w| TROUBLE_BODY_PART.contains(*w)).collect();
    if part.is_empty() {
        return false;
    }
    let mut sorted: Vec<String> = words.to_vec();
    sorted.sort();
    sorted.dedup();
    if !crate::compose::trouble::reads_as_trouble(clean, Some(&sorted[..]), "").truthy() {
        return false;
    }
    if words.iter().any(|w| CARE_POINTER.contains(w) || CARE_PRONOUNS.contains(w)) {
        return false;
    }
    let seen: HashSet<String> = az09_words(&lower(thread)).into_iter().collect();
    !part.iter().any(|p| seen.contains(*p))
}

/// Python: bot/compose/fallback.py:13794 `_keeps_thread(understood, clean, thread='', crisis=False)`
/// True when the line is still about the emergency, rather than a new subject.
pub fn _keeps_thread(understood: &Understood, clean: &str, thread: &str, crisis: bool) -> bool {
    // `norm` is always a list on a reading (`isinstance(norm, (list, tuple))`).
    let mut words: Vec<String> = understood.norm.clone();
    words.sort();
    words.dedup();
    let has = |set: &HashSet<String>| words.iter().any(|w| set.contains(w));
    if !crisis && _names_a_new_part(&words, clean, thread) {
        return false;
    }
    if _ASKS_TO_LEAVE.is_match(clean) && SOMEBODY_HURT_ME.is_match(thread) {
        return true;
    }
    if _THE_WAY_OUT.is_match(clean) {
        return true;
    }
    if has(&CARE_HURTS) {
        return true;
    }
    if !crisis && !has(&CARE_INJURY) && HOUSE_FAULT.is_match(clean) {
        return false;
    }
    if words.iter().any(|w| CARE_WORDS.contains(w) && !CARE_POINTER.contains(w)) || CARE_ASKS.is_match(clean) {
        return true;
    }
    let own = understood.topic_own.as_deref();
    if has(&CARE_RESOLVED) && has(&CARE_NOT_RESOLVED) && (own.is_none() || own == Some("feelings") || own == Some("health")) {
        return true;
    }
    if has(&REACTION_WORDS) {
        return true;
    }
    if has(&CARE_POINTER) && own.is_none_or(str::is_empty) && !_POINTER_OWN_OBJECT.is_match(clean) {
        return true;
    }
    if CARE_PHRASES.iter().any(|p| clean.contains(p.as_str())) || CARE_SEEN_ASKS.is_match(clean) {
        return true;
    }
    if _someone_is_arriving(clean) {
        return true;
    }
    if ALREADY_RANG.is_match(clean) {
        return true;
    }
    if _burn_line(clean, thread) {
        return true;
    }
    if understood.presence || STILL_HERE.is_match(clean) {
        return true;
    }
    if understood.kind == "thanks" {
        return words.len() <= 5;
    }
    if crisis && has(&TROUBLE_BODY_PART) {
        return true;
    }
    if understood.kind == "question" || clean.contains('?') {
        return understood.keywords.iter().all(|w| GENERIC_WORDS.contains(&lower(w)));
    }
    if words.len() <= 2 && has(&AGREEMENTS) {
        return true;
    }
    if own.is_none_or(str::is_empty) {
        let mine: Vec<&String> = words.iter().filter(|w| w.chars().count() > 3 && !ASKING_WORDS.contains(*w)).collect();
        let thread_words: HashSet<String> = az09_words(&lower(thread)).into_iter().collect();
        return mine.iter().any(|w| thread_words.contains(*w)) || mine.is_empty();
    }
    false
}

regexes!(ABOUT_THE_SPEAKER, NOT_FRIGHTENED, FRIGHTENED);
word_sets!(WORRY_FILLER);

/// Python: bot/compose/fallback.py:13905 `_reads_as_fear(understood, clean)`
/// True when the line is the speaker's own fear, not a practical question about the incident.
pub fn _reads_as_fear(understood: &Understood, clean: &str) -> bool {
    if understood.kind == "question" || clean.contains('?') {
        return false;
    }
    if CARE_ASKS.is_match(clean) {
        return false;
    }
    if !ABOUT_THE_SPEAKER.is_match(clean) {
        return false;
    }
    if NOT_FRIGHTENED.is_match(clean) {
        return false;
    }
    if FRIGHTENED.is_match(clean) {
        return true;
    }
    _only_a_worry(clean)
}

/// Python: bot/compose/fallback.py:13927 `_only_a_worry(clean)` (never called on the trace corpus)
/// True when the whole line is the speaker's worry and nothing else.
pub fn _only_a_worry(clean: &str) -> bool {
    if rx(r"\babout\b").is_match(clean) {
        return false;
    }
    let words = az_words(clean);
    if words.is_empty() || words.len() > 6 {
        return false;
    }
    if !words.iter().any(|w| WORRY_WORDS.contains(w)) {
        return false;
    }
    words.iter().all(|w| WORRY_FILLER.contains(w) || WORRY_WORDS.contains(w))
}

regexes!(A_PERSON_OF_THEIRS, SELF_IN_DANGER, SOMEBODY_HURT_ME, A_PRONOUN_FOR_SOMEBODY);
word_sets!(SHE, HE);

/// Python: bot/compose/fallback.py:13947 `_about_somebody_else(safety, memory)`
/// The words for the person in danger when the crisis was about somebody else (a dict of
/// `subject`, `object`, `possessive`, `be` and `who`), else None. `safety` is the safety turn.
/// (The skeleton's `()` return could not carry the words.)
pub fn _about_somebody_else(safety: &Value, memory: &Memory) -> Option<Value> {
    let opener = safety.get("user").and_then(Value::as_str).unwrap_or("").to_string();
    if SELF_IN_DANGER.is_match(&opener) {
        return None;
    }
    if SOMEBODY_HURT_ME.is_match(&opener) {
        return None;
    }
    let mut who: Option<String> = A_PERSON_OF_THEIRS.search(&opener).map(|m| m.name_str("who").to_string());
    if who.is_none() {
        if !A_PRONOUN_FOR_SOMEBODY.is_match(&opener) {
            return None;
        }
        for turn in last_n(&memory.turns, 4).iter().rev() {
            let said = &turn.user;
            if *said == opener || SELF_IN_DANGER.is_match(said) {
                continue;
            }
            if let Some(found) = A_PERSON_OF_THEIRS.search(said) {
                who = Some(found.name_str("who").to_string());
                break;
            }
        }
    }
    let who = who?;
    let word = rx(r"\s+").sub(" ", &lower(text::strip(&who)));
    let (subject, object, possessive, be) = if SHE.contains(&word) {
        ("she", "her", "her", "is")
    } else if HE.contains(&word) {
        ("he", "him", "his", "is")
    } else {
        ("they", "them", "their", "are")
    };
    let mut forms = Value::dict();
    forms.set_key("subject", subject);
    forms.set_key("object", object);
    forms.set_key("possessive", possessive);
    forms.set_key("be", be);
    forms.set_key("who", format!("your {word}"));
    Some(forms)
}

/// Python: bot/compose/fallback.py:13999 `_wrote_for_them(pool, forms)` (never called on the trace corpus)
/// One pool's lines, filled with the words for the person they are about (a tuple value).
/// `forms` is the dict `_about_somebody_else` gives. (The skeleton typed it `&[String]`, which
/// cannot carry the dict.)
pub fn _wrote_for_them(pool: &Value, forms: &Value) -> Value {
    let get = |k: &str| forms.get(k).and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| panic!("KeyError: {k:?}"));
    let (who, subj, obj, poss, be) = (get("who"), get("subject"), get("object"), get("possessive"), get("be"));
    let lines: Vec<Value> = pool
        .strs()
        .iter()
        .map(|l| Value::from(format_named(l, &[("who", &who), ("subj", &subj), ("obj", &obj), ("poss", &poss), ("be", &be)])))
        .collect();
    Value::Tuple(std::sync::Arc::new(lines))
}

// ---- fallback.py:14054-14345: red flags, the threshold crossed, the route named, a draft ----

regexes!(RED_FLAG_SHARED, RED_FLAG_EVENTS, THRESHOLD_CLAUSE, A_CONDITION, ALREADY_OR_NOT, SENTENCE_END, CLAUSE_END, RESOLVED, STAKES_WORDS, FALL_FIRST_AID, CALL_THEM_NOW);
word_sets!(THRESHOLD_NOT_A_SIGN);
pools!(A_FEELING_OF, THE_CALL_POOLS);
static CARE_ROUTE_WORDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.CARE_ROUTE_WORDS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().unwrap())).collect()
});

/// The last `n` words of `s.split()`.
fn last_words(s: &str, n: usize) -> Vec<&str> {
    let w = text::split_ref(s);
    let start = w.len().saturating_sub(n);
    w[start..].to_vec()
}

/// Python: bot/compose/fallback.py:14053 `_red_flags_in(line)`
/// The flag words in the line, leaving out the ones the line itself says are gone (a set).
pub fn _red_flags_in(line: &str) -> HashSet<String> {
    let low = text::squash(&lower(line));
    let mut flags = HashSet::new();
    for m in RED_FLAG_SHARED.finditer(&low) {
        let before = last_words(&low[..m.start()], 4);
        let after: Vec<&str> = text::split_ref(&low[m.end()..]).into_iter().take(4).collect();
        if before.iter().any(|w| ["not", "no", "never", "isnt", "arent", "havent", "hasnt", "dont", "didnt"].contains(w)) {
            continue;
        }
        if after.contains(&"stopped")
            || (after.contains(&"any") && after.contains(&"more"))
            || (after.contains(&"no") && (after.contains(&"more") || after.contains(&"longer")))
        {
            continue;
        }
        flags.insert(_flag_stem(m.as_str()));
    }
    flags
}

/// Python: bot/compose/fallback.py:14087 `_flag_events_in(line)`
/// The signs in the line that are an emergency by themselves, leaving out the denied ones.
pub fn _flag_events_in(line: &str) -> HashSet<String> {
    let low = text::squash(&lower(line));
    let mut found = HashSet::new();
    for m in RED_FLAG_EVENTS.finditer(&low) {
        let before = last_words(&low[..m.start()], 4);
        if before.iter().any(|w| ["not", "no", "never", "isnt", "arent", "havent", "hasnt", "dont", "didnt", "wasnt", "without"].contains(w)) {
            continue;
        }
        found.insert(_flag_stem(text::split_ref(m.as_str())[0]));
    }
    found
}

/// Python: bot/compose/fallback.py:14140 `_the_threshold_words(sentence)`
/// The signs the answer names in its own if-clause, without the ending that varies.
pub fn _the_threshold_words(sentence: &str) -> HashSet<String> {
    let low = text::squash(&lower(sentence));
    let mut words = HashSet::new();
    for m in THRESHOLD_CLAUSE.finditer(&low) {
        if lower(text::lstrip(m.as_str())).starts_with("even if") {
            continue;
        }
        for word in az_words(m.str(1)) {
            if word.chars().count() < 4 || THRESHOLD_NOT_A_SIGN.contains(&word) {
                continue;
            }
            words.insert(_flag_stem(&word));
        }
    }
    words
}

/// Python: bot/compose/fallback.py:14154 `_it_is_the_answer_word(line, word)` (never called on the trace corpus)
/// True when the line says the answer's own sign as a feeling of theirs.
pub fn _it_is_the_answer_word(line: &str, word: &str) -> bool {
    let low = text::squash(&lower(line));
    let esc = crate::py::re::escape(word);
    if !rx(&format!(r"\b{esc}\w*\b")).is_match(&low) {
        return false;
    }
    for shape in A_FEELING_OF.iter() {
        let said = format!(r"(?:the\s+|a\s+|some\s+)?{esc}\w*");
        if rx(&format!(r"\b{}\b", shape.replace("{w}", &said))).is_match(&low) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:14166 `_the_flags_still_standing(line)`
/// The flag words in the line, leaving out the clauses that say it has passed.
pub fn _the_flags_still_standing(line: &str) -> HashSet<String> {
    let clauses: Vec<String> = CLAUSE_END.split(&lower(line)).into_iter().filter(|c| !text::strip(c).is_empty()).collect();
    let settled: Vec<&String> = clauses.iter().filter(|c| _feels_better(c) || RESOLVED.is_match(c)).collect();
    if settled.is_empty() {
        return _red_flags_in(line);
    }
    let mut flags = HashSet::new();
    for clause in &clauses {
        if settled.contains(&clause) || rx(r"\b(?:was|were|had)\b").is_match(clause) {
            continue;
        }
        flags.extend(_red_flags_in(clause));
    }
    flags
}

/// Python: bot/compose/fallback.py:14188 `_the_threshold_was_crossed(last, clean, turns)`
/// True when the answer named 999 for something worse, and this line says that thing.
pub fn _the_threshold_was_crossed(last: &str, clean: &str, turns: &[Turn]) -> bool {
    let said = lower(last);
    if said.is_empty() || !CALL_THEM_NOW.is_match(&said) {
        return false;
    }
    let line = clean;
    if _the_thread_is_an_animal(line, turns, 4) {
        return false;
    }
    if !_flag_events_in(line).is_empty() {
        return true;
    }
    let flags = _the_flags_still_standing(line);
    for sentence in SENTENCE_END.split(&said) {
        if !CALL_THEM_NOW.is_match(&sentence) {
            continue;
        }
        if _red_flags_in(&sentence).iter().any(|f| flags.contains(f)) {
            return true;
        }
        if _feels_better(line) || RESOLVED.is_match(&lower(clean)) {
            continue;
        }
        for word in _the_threshold_words(&sentence) {
            if _it_is_the_answer_word(line, &word) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/fallback.py:14225 `_flag_stem(word)` (never called on the trace corpus)
/// The word without the ending that makes "sweaty" and "sweating" the same flag.
pub fn _flag_stem(word: &str) -> String {
    let mut word = word.to_string();
    for tail in ["ing", "ness", "ed", "y", "s"] {
        if word.ends_with(tail) && text::len(&word) - text::len(tail) >= 4 {
            word = text::slice(&word, 0, Some(-(text::len(tail) as isize)));
        }
    }
    word
}

/// Python: bot/compose/fallback.py:14240 `_it_is_this_files_call(said)` (never called on the trace corpus)
/// True when the line is one of this file's own call pools, said word for word.
pub fn _it_is_this_files_call(said: &str) -> bool {
    let said = text::squash(&lower(said));
    THE_CALL_POOLS.iter().any(|line| said.starts_with(&text::slice(&text::squash(&lower(line)), 0, Some(40))))
}

/// Python: bot/compose/fallback.py:14249 `_a_newer_answer_asked_them_to_call(last, turns)` (never called on the trace corpus)
/// True when this fallback's own later line already told them to ring, so the route is stale.
pub fn _a_newer_answer_asked_them_to_call(last: &str, turns: &[Turn]) -> bool {
    let last_low = lower(text::strip(last));
    for turn in last_n(turns, *THREAD_TURNS).iter().rev() {
        let said = &turn.bot;
        if said.is_empty() {
            continue;
        }
        if lower(text::strip(said)) == last_low {
            return false;
        }
        if turn.module.as_deref() != Some(FALLBACK_MODULE.as_str()) {
            return false;
        }
        if _the_answer_asked_them_to_call(said) || _it_is_this_files_call(said) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:14278 `_the_route_the_answer_named(last)` (never called on the trace corpus)
/// The place the bot's own last answer told them to go, or None (a str value).
pub fn _the_route_the_answer_named(last: &str) -> Value {
    the_route_the_answer_named(last).map(Value::from).unwrap_or(Value::None)
}

/// `_the_route_the_answer_named` as an optional name.
pub fn the_route_the_answer_named(last: &str) -> Option<String> {
    if last.is_empty() {
        return None;
    }
    let gas = rx(r"0800\s*111\s*999");
    let t = if gas.is_match(last) { gas.sub("the gas emergency line", last) } else { last.to_string() };
    CARE_ROUTE_WORDS.iter().find(|(_, p)| p.is_match(&t)).map(|(name, _)| name.clone())
}

/// Python: bot/compose/fallback.py:14296 `_the_answer_asked_them_to_call(last)`
/// True when the bot's own last answer told the person to ring the emergency services now.
pub fn _the_answer_asked_them_to_call(last: &str) -> bool {
    for sentence in SENTENCE_END.split(last) {
        let low = ALREADY_OR_NOT.sub("already", &lower(&sentence));
        let mut led_by_condition = false;
        for clause in CLAUSE_END.split(&low) {
            let opens_with_condition = A_CONDITION.is_match(&clause) && !rx(r"\b(?:call|ring|ringing|dial|phone|text|press|try)\b").is_match(&clause);
            if CALL_THEM_NOW.is_match(&clause) && !A_CONDITION.is_match(&clause) && !led_by_condition {
                return true;
            }
            led_by_condition = opens_with_condition;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:14321 `_raises_the_stakes(clean, last)` (never called on the trace corpus)
/// True when the person's line meets the red flag their first-aid card named.
pub fn _raises_the_stakes(clean: &str, last: &str) -> bool {
    STAKES_WORDS.is_match(clean) && FALL_FIRST_AID.is_match(&lower(last))
}

/// Python: bot/compose/fallback.py:14331 `_releasable(memory)`
/// True when the thread the composer is holding can be let go by a new subject.
pub fn _releasable(memory: &Memory) -> bool {
    for turn in last_n(&memory.turns, *THREAD_TURNS).iter().rev() {
        if turn.module.as_deref() == Some("safety") {
            let last = lower(&turn.bot);
            return !(CRISIS_WORDS.iter().any(|w| last.contains(w.as_str())) || _the_answer_asked_them_to_call(&last));
        }
        if turn.module.as_deref() != Some(FALLBACK_MODULE.as_str()) {
            return false;
        }
    }
    true
}

static DRAFT_LEAD_INS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.draft.LEAD_INS"));

/// Python: bot/compose/fallback.py:14350 `_named_draft(memory, text, words, clean)`
/// The message the bot asked for, on a line that says it is the message - or None. (The
/// skeleton's `()` return could not carry the reply.)
pub fn _named_draft(memory: &Memory, text_: &str, words: &[String], clean: &str) -> Option<String> {
    if !crate::compose::draft::says_it_is_the_message(clean) {
        return None;
    }
    if _action(clean, memory).is_some() || crate::compose::badnews::reads_as_pregnancy_bleeding(clean) || crate::compose::badnews::reads_as_sudden_loss(clean) {
        return None;
    }
    let drafted = crate::compose::draft::drafter(memory, text_, words).py_opt_str().filter(|d| !d.is_empty())?;
    let lines: Vec<String> = DRAFT_LEAD_INS.iter().map(|lead| format!("{lead} \"{drafted}\"")).collect();
    choose(&lines, memory, clean)
}

// ---- fallback.py:14369-14962: the continuation after a safety answer and its helpers ----

pools!(AFRAID_AFTER, HELLO_WITH_YOU, HELLO_STILL_THERE, GREETING_AFTER, CARE_HERE, MOVE_ON_AFTER, ABUSE_AFTER, TRIED_NO_CHANGE_ELSEWHERE, TRIED_WORSE_ELSEWHERE);
pools!(CRISIS_AFTER_THEM, CRISIS_AFTER, EMERGENCY_NOT_WORKED, THANKS_AFTER, RESOLVED_AFTER, AFTER_RESOLVED, THE_CALL_STANDS_ANYWAY, RESOLVED_AFTER_THEM);
pools!(HELPLINE_NOT_AN_AMBULANCE, STILL_HERE_AFTER, HOLD_AFTER, CARE_VISIT_WARD, CARE_VISIT, CARE_SEEN_ANIMAL, CARE_SEEN, EMERGENCY_BETTER_THEM, EMERGENCY_BETTER);
pools!(CRISIS_BETTER_THEM, CRISIS_BETTER, CARE_BETTER, BURN_BLISTER, BURN_AFTER, WITH_YOU_AFTER, HAZARD_AFTER, EMERGENCY_AFTER, CARE_AFTER_ANIMAL_STILL);
pools!(CARE_AFTER_ANIMAL, CARE_AFTER_STILL, CARE_AFTER, SORTED_AND_NO_CALL_YET, THRESHOLD_CROSSED, THE_CALL_STANDS, CALL_WORDS);
regexes!(THING_SORTED, NEEDS_SEEN, A_HELPLINE, VISIT_ASK, MEDICAL_PLACE, NOT_A_VISIT, SAID_AS_A_DEATH, STILL_GOING, IMMEDIATE_CALL, CONDITIONAL_OPENING, ABUSE_CARD, HAZARD_LINE, A_BACK_AGAIN, THE_THING_ITSELF_LOOKS_FINE);
word_sets!(THANKS_WORDS, ONGOING_WORDS);
static THE_THREAD_KEPT: LazyLock<i64> = LazyLock::new(|| data::get("bot.compose.fallback.THE_THREAD_KEPT").as_int().unwrap());
static CARE_ROUTE_BACK: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback.CARE_ROUTE_BACK").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});

/// `_wrote_for_them(pool, forms)` as lines.
fn wrote_for_them(pool: &[String], forms: &Value) -> Vec<String> {
    _wrote_for_them(&Value::from(pool.to_vec()), forms).strs()
}

/// `_choose(_wrote_for_them(THEM, them) if them else OWN, memory, clean)`.
fn for_them_or(them: &Option<Value>, them_pool: &[String], own: &[String], memory: &Memory, clean: &str) -> Option<String> {
    match them {
        Some(forms) => choose(&wrote_for_them(them_pool, forms), memory, clean),
        None => choose(own, memory, clean),
    }
}

/// Python: bot/compose/fallback.py:14369 `_after_safety(memory, clean, understood=None)`
/// A careful continuation when the bot has just answered an emergency, or None. (The skeleton's
/// `()` return could not carry the reply.)
pub fn _after_safety(memory: &Memory, clean: &str, understood: Option<&Understood>) -> Option<String> {
    let turns = &memory.turns;
    let mut window = last_n(turns, *THREAD_TURNS);
    let mut safety: Option<&Turn> = None;
    for index in (0..window.len()).rev() {
        if window[index].module.as_deref() == Some("safety") {
            safety = Some(&window[index]);
            window = &window[index + 1..];
            break;
        }
    }
    let safety = safety?;
    let read_owned;
    let read: &Understood = match understood.or(memory.understood.as_ref()) {
        Some(u) => u,
        None => {
            read_owned = crate::understanding::understand(clean, Some(memory));
            &read_owned
        }
    };
    let mut last = lower(&safety.bot);
    for turn in window {
        if turn.module.as_deref() == Some(FALLBACK_MODULE.as_str()) {
            continue;
        }
        if crate::memory::history::not_a_subject(&turn.user) {
            continue;
        }
        if _words_kept(&turn.bot, &last) >= *THE_THREAD_KEPT {
            continue;
        }
        return None;
    }
    let in_crisis = CRISIS_WORDS.iter().any(|w| last.contains(w.as_str()));
    if _reads_as_fear(read, clean) && !in_crisis {
        return choose(&AFRAID_AFTER, memory, clean);
    }
    let words: Vec<String> = read.words.clone();
    let has = |set: &HashSet<String>| words.iter().any(|w| set.contains(w));
    let live = in_crisis || _the_answer_asked_them_to_call(&last) || _raises_the_stakes(clean, &last);
    if read.kind == "greeting" && !has(&CARE_WORDS) {
        if live {
            return choose(if in_crisis { &HELLO_WITH_YOU } else { &HELLO_STILL_THERE }, memory, clean);
        }
        return choose(&GREETING_AFTER, memory, clean);
    }
    let plain = if read.plain.is_empty() { clean } else { read.plain.as_str() };
    if _asks_how_are_you(clean, plain) && !_burn_line(clean, "") {
        return choose(&CARE_HERE, memory, clean);
    }
    if !live && A_BACK_AGAIN.match_(text::strip(clean)).is_some() {
        return choose(&GREETING_AFTER, memory, clean);
    }
    if _short_ack(&words) && !live && _thread_was_closed(turns) {
        return None;
    }
    if MOVE_ON.is_match(clean) {
        return choose(&MOVE_ON_AFTER, memory, clean);
    }
    if !in_crisis {
        if let Some(advice) = _asks_about_the_advice(clean, &last) {
            if !advice.is_empty() {
                return choose(&advice, memory, clean);
            }
        }
    }
    if crate::compose::outlook::asks_the_outlook(clean, clean) {
        let cards = _the_safety_cards(turns);
        let in_a_crisis = cards.iter().any(|card| CRISIS_WORDS.iter().any(|w| card.contains(w.as_str())));
        let told_to_call = cards.iter().any(|card| _the_answer_asked_them_to_call(card));
        let abuse = cards.iter().any(|card| _the_answer_was_an_abuse_card(card));
        let pool = crate::compose::outlook::the_pool(
            &Value::Bool(in_a_crisis),
            &Value::Bool(_somebody_is_coming(turns, clean)),
            &Value::Bool(told_to_call),
            &Value::Bool(abuse),
        );
        let pool = if pool.is_none() { with_who(&FORECAST_AFTER_SAFETY, "you") } else { pool.strs() };
        return choose(&pool, memory, clean);
    }
    let safety_turn = {
        let mut d = Value::dict();
        d.set_key("user", safety.user.as_str());
        d.set_key("bot", safety.bot.as_str());
        d.set_key("module", safety.module.as_deref().map(Value::from).unwrap_or(Value::None));
        d
    };
    let them = _about_somebody_else(&safety_turn, memory);
    if _the_answer_was_an_abuse_card(&last) && (_THE_WAY_OUT.is_match(clean) || _ASKS_TO_LEAVE.is_match(clean)) {
        return choose(&ABUSE_AFTER, memory, clean);
    }
    if live {
        // `in (TRIED_NO_CHANGE, TRIED_NO_CHANGE_ELSEWHERE, TRIED_WORSE, TRIED_WORSE_ELSEWHERE)`:
        // the two bare names are the module's compiled patterns, which no pool equals.
        let report = _a_report_on_the_advice(clean, &words, false, false, &HashSet::new());
        if report.as_ref().is_some_and(|r| *r == *TRIED_NO_CHANGE_ELSEWHERE || *r == *TRIED_WORSE_ELSEWHERE) {
            if in_crisis && them.is_some() {
                return for_them_or(&them, &CRISIS_AFTER_THEM, &CRISIS_AFTER, memory, clean);
            }
            return choose(if in_crisis { &CRISIS_AFTER } else { &EMERGENCY_NOT_WORKED }, memory, clean);
        }
    }
    if THING_SORTED.is_match(clean) {
        let pool = if in_crisis { CRISIS_AFTER.clone() } else { _hold_pool(turns, clean, &last, true) };
        return choose(&pool, memory, clean);
    }
    if !in_crisis {
        let reading = crate::compose::verdict::reply_for(clean, memory).py_opt_str();
        if let Some(reading) = reading.filter(|r| !r.is_empty()) {
            return Some(reading);
        }
    }
    if in_crisis && (_a_lost_ask(clean) || _a_start_ask(clean)) {
        return choose(&CRISIS_AFTER, memory, clean);
    }
    if _the_hazard_is_gas(&safety.user) && (_THE_WAY_OUT.is_match(clean) || _goes_back_for(clean, &safety.user)) {
        return Some(GAS_ANSWER.clone());
    }
    if _the_threshold_was_crossed(&last, clean, turns) {
        return choose(&THRESHOLD_CROSSED, memory, clean);
    }
    if !_feels_better(clean)
        && !(_goes_back_for(clean, &safety.user) || _keeps_thread(read, clean, &safety.user, !_releasable(memory) || _raises_the_stakes(clean, &last)))
    {
        return None;
    }
    if (read.kind == "thanks" || has(&THANKS_WORDS)) && !_thanks_carries_emergency(&words, clean) {
        return choose(&THANKS_AFTER, memory, clean);
    }
    if _short_ack(&words) {
        if in_crisis {
            return choose(&CRISIS_AFTER, memory, clean);
        }
        return choose(&_hold_pool(turns, clean, &last, false), memory, clean);
    }
    last = lower(&safety.bot);
    let asked = read.kind == "question" || clean.contains('?');
    if !_reads_as_fear(read, clean) && has(&REACTION_WORDS) && last_n(turns, 2).iter().any(|t| RESOLVED_AFTER.contains(&t.bot)) {
        return choose(&AFTER_RESOLVED, memory, clean);
    }
    if !asked && RESOLVED.is_match(clean) && !NEEDS_SEEN.is_match(clean) && !_someone_is_arriving(clean) {
        if them.is_none() && _a_newer_answer_asked_them_to_call(&last, turns) && !ALREADY_RANG.is_match(clean) {
            return choose(&THE_CALL_STANDS_ANYWAY, memory, clean);
        }
        return for_them_or(&them, &RESOLVED_AFTER_THEM, &RESOLVED_AFTER, memory, clean);
    }
    if ALREADY_RANG.is_match(clean) {
        if A_HELPLINE.is_match(clean) {
            if CRISIS_WORDS.iter().any(|w| last.contains(w.as_str())) {
                return for_them_or(&them, &CRISIS_AFTER_THEM, &CRISIS_AFTER, memory, clean);
            }
            if SAFETY_WORDS.iter().any(|w| last.contains(w.as_str())) {
                return choose(&HELPLINE_NOT_AN_AMBULANCE, memory, clean);
            }
            return choose(&STILL_HERE_AFTER, memory, clean);
        }
        if CRISIS_WORDS.iter().any(|w| last.contains(w.as_str())) {
            return for_them_or(&them, &CRISIS_AFTER_THEM, &CRISIS_AFTER, memory, clean);
        }
        return choose(&HOLD_AFTER, memory, clean);
    }
    if _reads_as_fear(read, clean) && !CRISIS_WORDS.iter().any(|w| last.contains(w.as_str())) {
        return choose(&AFRAID_AFTER, memory, clean);
    }
    if has(&REACTION_WORDS) {
        if CRISIS_WORDS.iter().any(|w| last.contains(w.as_str())) {
            return for_them_or(&them, &CRISIS_AFTER_THEM, &CRISIS_AFTER, memory, clean);
        }
        return choose(&_hold_pool(turns, clean, &last, false), memory, clean);
    }
    if VISIT_ASK.is_match(clean) && !MEDICAL_PLACE.is_match(clean) && !NOT_A_VISIT.is_match(clean) && _somebody_to_visit(clean, turns, 3) {
        let pool = if _someone_is_a_patient(clean, turns, 4) { &CARE_VISIT_WARD } else { &CARE_VISIT };
        return choose(pool, memory, clean);
    }
    if CARE_SEEN_ASKS.is_match(clean) {
        if _the_thread_is_an_animal(clean, turns, 4) {
            return choose(&CARE_SEEN_ANIMAL, memory, clean);
        }
        return choose(&CARE_SEEN, memory, clean);
    }
    if _feels_better(clean) {
        if _no_relief(memory, &last) {
            return for_them_or(&them, &EMERGENCY_BETTER_THEM, &EMERGENCY_BETTER, memory, clean);
        }
        if CRISIS_WORDS.iter().any(|w| last.contains(w.as_str())) {
            return for_them_or(&them, &CRISIS_BETTER_THEM, &CRISIS_BETTER, memory, clean);
        }
        return choose(&CARE_BETTER, memory, clean);
    }
    let opener = lower(&safety.user);
    if BURN_WORD.is_match(&last) || BURN_WORD.is_match(&opener) {
        if BURN_BLISTER_LINE.is_match(clean) || POP_A_BARE_IT.is_match(clean) {
            return choose(&BURN_BLISTER, memory, clean);
        }
        if BURN_ON_IT.is_match(clean) || (BURN_THING.is_match(clean) && BURN_ASK.is_match(clean)) {
            return choose(&BURN_AFTER, memory, clean);
        }
    }
    if them.is_some() && SAID_AS_A_DEATH.is_match(clean) {
        return None;
    }
    if CRISIS_WORDS.iter().any(|w| last.contains(w.as_str())) {
        return for_them_or(&them, &CRISIS_AFTER_THEM, &CRISIS_AFTER, memory, clean);
    }
    if _someone_is_arriving(clean) {
        return choose(&WITH_YOU_AFTER, memory, clean);
    }
    if _already_rang(turns) {
        return choose(&HOLD_AFTER, memory, clean);
    }
    if _the_answer_asked_them_to_call(&last) || _raises_the_stakes(clean, &last) {
        if _is_a_hazard(&last) {
            return choose(&HAZARD_AFTER, memory, clean);
        }
        return choose(&EMERGENCY_AFTER, memory, clean);
    }
    if _the_thread_is_an_animal(clean, turns, 4) {
        return choose(if STILL_GOING.is_match(clean) { &CARE_AFTER_ANIMAL_STILL } else { &CARE_AFTER_ANIMAL }, memory, clean);
    }
    if _a_newer_answer_asked_them_to_call(&last, turns) {
        return choose(&THE_CALL_STANDS, memory, clean);
    }
    if let Some(route) = the_route_the_answer_named(&last) {
        if !THE_THING_ITSELF_LOOKS_FINE.is_match(clean) {
            let pool = pool_of(&CARE_ROUTE_BACK, &route).unwrap_or_else(|| panic!("KeyError: {route:?} (CARE_ROUTE_BACK)"));
            return choose(&pool, memory, clean);
        }
    }
    choose(if STILL_GOING.is_match(clean) { &CARE_AFTER_STILL } else { &CARE_AFTER }, memory, clean)
}

/// Python: bot/compose/fallback.py:14739 `_is_a_hazard(last)` (never called on the trace corpus)
/// True when the emergency the bot answered is a fire, a gas leak or an alarm going off.
pub fn _is_a_hazard(last: &str) -> bool {
    HAZARD_LINE.is_match(&lower(last))
}

/// Python: bot/compose/fallback.py:14752 `_the_safety_cards(turns)` (never called on the trace corpus)
/// The safety answers still in the thread window, newest last.
pub fn _the_safety_cards(turns: &[Turn]) -> Vec<String> {
    last_n(turns, *THREAD_TURNS).iter().filter(|t| t.module.as_deref() == Some("safety")).map(|t| lower(&t.bot)).collect()
}

/// Python: bot/compose/fallback.py:14763 `_somebody_is_coming(turns, clean)` (never called on the trace corpus)
/// True when the person has rung, or help is with them, anywhere in the thread.
pub fn _somebody_is_coming(turns: &[Turn], clean: &str) -> bool {
    for turn in last_n(turns, *THREAD_TURNS) {
        let said = lower(&turn.user);
        if !said.is_empty() && (ALREADY_RANG.is_match(&said) || _someone_is_arriving(&said)) {
            return true;
        }
    }
    ALREADY_RANG.is_match(clean) || _someone_is_arriving(clean)
}

/// Python: bot/compose/fallback.py:14772 `_the_answer_was_an_abuse_card(last)`
pub fn _the_answer_was_an_abuse_card(last: &str) -> bool {
    ABUSE_CARD.is_match(last)
}

/// Python: bot/compose/fallback.py:14782 `_thread_was_closed(turns)` (never called on the trace corpus)
/// True when this fallback's last line in the thread was a close, not advice.
pub fn _thread_was_closed(turns: &[Turn]) -> bool {
    for turn in last_n(turns, *THREAD_TURNS).iter().rev() {
        if turn.module.as_deref() != Some(FALLBACK_MODULE.as_str()) {
            continue;
        }
        return THANKS_AFTER.iter().any(|line| turn.bot.starts_with(&text::slice(line, 0, Some(24))));
    }
    false
}

/// Python: bot/compose/fallback.py:14796 `_hold_pool(turns, clean, last, done_report=False)` (never called on the trace corpus)
/// The hold pool: one that may promise a call only when this thread already has one.
pub fn _hold_pool(turns: &[Turn], clean: &str, last: &str, done_report: bool) -> Vec<String> {
    if !_a_call_is_known(turns, clean, last) {
        return STILL_HERE_AFTER.clone();
    }
    if done_report && !(ALREADY_RANG.is_match(clean) || _already_rang(turns)) {
        return SORTED_AND_NO_CALL_YET.clone();
    }
    HOLD_AFTER.clone()
}

/// Python: bot/compose/fallback.py:14824 `_a_call_is_known(turns, clean, last)` (never called on the trace corpus)
/// True when somebody has been rung in this thread: said here, said before, or in the advice.
pub fn _a_call_is_known(turns: &[Turn], clean: &str, last: &str) -> bool {
    if ALREADY_RANG.is_match(clean) || _already_rang(turns) {
        return true;
    }
    if !CALL_WORDS.iter().any(|w| last.contains(w.as_str())) {
        return false;
    }
    if last.contains("111") && !IMMEDIATE_CALL.is_match(last) {
        return false;
    }
    !_the_call_is_conditional(last)
}

/// Python: bot/compose/fallback.py:14851 `_the_call_is_conditional(text)` (never called on the trace corpus)
/// True when every sentence of the answer that names a call puts it behind a condition.
pub fn _the_call_is_conditional(text_: &str) -> bool {
    let sentences = rx(r"(?<=[.!?])\s+").split(text_);
    let carrying: Vec<&String> = sentences.iter().filter(|s| {
        let low = lower(s);
        CALL_WORDS.iter().any(|w| low.contains(w.as_str()))
    }).collect();
    if carrying.is_empty() {
        return false;
    }
    carrying.iter().all(|s| CONDITIONAL_OPENING.is_match(text::strip(s)))
}

/// Python: bot/compose/fallback.py:14860 `_already_rang(turns)` (never called on the trace corpus)
/// True when the person said, in the last few turns, that somebody has been rung.
pub fn _already_rang(turns: &[Turn]) -> bool {
    last_n(turns, 3).iter().any(|t| ALREADY_RANG.is_match(&lower(&t.user)))
}

/// Python: bot/compose/fallback.py:14866 `_thanks_carries_emergency(words, clean)` (never called on the trace corpus)
/// True when a thanks also says the emergency is still going on.
pub fn _thanks_carries_emergency(words: &[String], clean: &str) -> bool {
    let content: Vec<&String> = words
        .iter()
        .filter(|w| CARE_WORDS.contains(*w) && !THANKS_WORDS.contains(*w) && !CARE_PRONOUNS.contains(*w) && !CARE_POINTER.contains(*w))
        .collect();
    if !content.is_empty() && words.iter().any(|w| ONGOING_WORDS.contains(w)) {
        return true;
    }
    if words.iter().any(|w| CARE_RESOLVED.contains(w)) {
        return words.iter().any(|w| CARE_NOT_RESOLVED.contains(w));
    }
    !content.is_empty() || CARE_ASKS.is_match(clean) || CARE_SEEN_ASKS.is_match(clean) || CARE_PHRASES.iter().any(|p| clean.contains(p.as_str()))
}

regexes!(A_DATE_OF_OURS, A_WORRY_ABOUT_THEM);

/// Python: bot/compose/fallback.py:14911 `a_date_of_ours_forgotten(clean)`
/// True when the line is somebody forgetting a date of ours - an anniversary, a birthday.
pub fn a_date_of_ours_forgotten(clean: &str) -> bool {
    let said = text::squash(&lower(clean));
    A_DATE_OF_OURS.is_match(&said) && !A_WORRY_ABOUT_THEM.is_match(&said)
}

word_sets!(GRIEVANCE_IMPACT, NOT_THEIR_VERB, FIRST_PERSON, FORGETTING, OURS, EATING, EATEN_KIND);

/// Python: bot/compose/fallback.py:14922 `_impact_said(clean)`
/// The words of what somebody did, where the somebody is the one doing them (a set).
pub fn _impact_said(clean: &str) -> HashSet<String> {
    let tokens = az_words(clean);
    let mut found = HashSet::new();
    for (position, token) in tokens.iter().enumerate() {
        if !GRIEVANCE_IMPACT.contains(token) {
            continue;
        }
        let before = &tokens[position.saturating_sub(2)..position];
        if before.last().is_some_and(|w| NOT_THEIR_VERB.contains(w)) {
            continue;
        }
        if before.iter().any(|w| FIRST_PERSON.contains(w)) {
            continue;
        }
        if FORGETTING.contains(token) && !tokens[(position + 1).min(tokens.len())..(position + 4).min(tokens.len())].iter().any(|w| OURS.contains(w)) {
            continue;
        }
        if EATING.contains(token) && position + 1 < tokens.len() && EATEN_KIND.contains(&tokens[position + 1]) {
            continue;
        }
        found.insert(token.clone());
    }
    found
}

static LOSS_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.loss.LOSS_REPLIES"));

/// Python: bot/compose/fallback.py:14948 `_grief_thread(memory)`
/// True when the last thing the bot said about this was a condolence.
pub fn _grief_thread(memory: &Memory) -> bool {
    for turn in last_n(&memory.turns, *THREAD_TURNS).iter().rev() {
        let line = &turn.bot;
        if line.is_empty() {
            continue;
        }
        return LOSS_REPLIES.iter().any(|reply| line.starts_with(&text::slice(reply, 0, Some(30))));
    }
    false
}

// ---- fallback.py:14963-15975: grievances, good times, dread, pauses, returns, manners, reading ----

pools!(GRIEVANCE, GRIEVANCE_HARM_PHRASES, GRIEVANCE_HEALTH_PHRASES, GRIEVANCE_PHRASES, GRIEVANCE_STRAIN);
regexes!(GRIEVANCE_PRONOUN, GRIEVANCE_MINE, GRIEVANCE_RECUR, GRIEVANCE_MINE_THINGS, GRIEVANCE_STRAIN_SAID);
word_sets!(GRIEVANCE_PEOPLE, THREAD_PEOPLE, GRIEVANCE_NOT_A_THING, ACKS);

/// Python: bot/compose/fallback.py:14963 `_grievance_thread(memory)`
/// True when the last thing the bot said about this was the offer to talk it over.
pub fn _grievance_thread(memory: &Memory) -> bool {
    for turn in last_n(&memory.turns, *THREAD_TURNS).iter().rev() {
        let line = &turn.bot;
        if line.is_empty() {
            continue;
        }
        return GRIEVANCE.iter().any(|reply| line.starts_with(&text::slice(reply, 0, Some(30))));
    }
    false
}

/// Python: bot/compose/fallback.py:14981 `_someone_else_minding(clean, words, after=False)`
/// True when the line is about what another person keeps doing, and the person minds it.
pub fn _someone_else_minding(clean: &str, words: &[String], after: bool) -> bool {
    let said: HashSet<&str> = words.iter().filter(|w| text::is_alpha(w) || w.contains('\'')).map(String::as_str).collect();
    let a_person_pronoun = GRIEVANCE_PRONOUN.is_match(clean);
    if said.iter().any(|w| GRIEVANCE_PEOPLE.contains(*w)) {
        if !["my", "our", "the"].iter().any(|w| said.contains(w)) {
            return false;
        }
    } else if !(after && said.iter().any(|w| THREAD_PEOPLE.contains(*w))) && !a_person_pronoun {
        return false;
    }
    let spare = clean.replace("thank you", "thanks").replace("thank u", "thanks");
    if format!(" {spare} ").contains(" you ") {
        return false;
    }
    if said.iter().any(|w| GRIEVANCE_HEALTH.contains(*w)) || said.iter().any(|w| GRIEVANCE_HARM.contains(*w)) {
        if !(a_date_of_ours_forgotten(clean) && !said.iter().any(|w| GRIEVANCE_HARM.contains(*w))) {
            return false;
        }
    }
    if GRIEVANCE_HARM_PHRASES.iter().any(|p| clean.contains(p.as_str())) {
        return false;
    }
    if GRIEVANCE_HEALTH_PHRASES.iter().any(|p| clean.contains(p.as_str())) {
        return false;
    }
    let mine: HashSet<String> = GRIEVANCE_MINE.findall_str(clean).into_iter().collect();
    if _impact_said(clean).iter().any(|w| !mine.contains(w)) {
        return true;
    }
    if GRIEVANCE_RECUR.is_match(clean) {
        if after && !GRIEVANCE_MINE.is_match(clean) {
            return true;
        }
        if GRIEVANCE_MINE_THINGS.findall_str(clean).iter().any(|w| !GRIEVANCE_PEOPLE.contains(w) && !GRIEVANCE_NOT_A_THING.contains(w)) {
            return true;
        }
    }
    if GRIEVANCE_PHRASES.iter().any(|p| clean.contains(p.as_str())) {
        return true;
    }
    if GRIEVANCE_STRAIN.iter().any(|p| clean.contains(p.as_str())) {
        return true;
    }
    GRIEVANCE_STRAIN_SAID.is_match(clean)
}

/// Python: bot/compose/fallback.py:15035 `_short_ack(words)`
/// True for a bare "ok", "right", "got it" — a nod, not a new subject.
pub fn _short_ack(words: &[String]) -> bool {
    words.len() <= 2 && words.iter().any(|w| ACKS.contains(w))
}

regexes!(GOOD_TIME_SUBJECT, GOOD_TIME_OURS, GOOD_DAY_OURS, GOOD_DAY_BARE, GOOD_DAY_WENT, GOOD_TIME_IT, GOOD_VERDICT);
word_sets!(GOOD_TIME_WORDS, NOT_A_VERDICT_SUBJECT);

/// Python: bot/compose/fallback.py:15135 `_hard_week_in_the_thread(memory)`
/// True when the thread still holds a death or an accident the bot was gentle about.
pub fn _hard_week_in_the_thread(memory: &Memory) -> bool {
    if memory_is_empty(memory) {
        return false;
    }
    if crate::compose::loss::_loss_is_the_thread(memory) {
        return true;
    }
    for line in last_lines(memory, 2) {
        if !line.is_empty() && (crate::compose::badnews::reads_as_bad_news(&line) || crate::compose::badnews::reads_as_sudden_loss(&line) || HARD_WEEK.is_match(&line)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/fallback.py:15156 `_a_verdict(clean, memory=None)`
/// True when the line is a good verdict on a thing, with nothing asked and nothing after it.
pub fn _a_verdict(clean: &str, memory: &Memory) -> bool {
    if _hard_week_in_the_thread(memory) {
        return false;
    }
    let Some(found) = GOOD_VERDICT.search(clean) else { return false };
    let said = text::split(&lower(found.name_str("subject")));
    let head = if said.len() > 1 { &said[1] } else { &said[0] };
    if GRIEVANCE_PEOPLE.contains(head) || TROUBLE_OTHERS.contains(head) {
        return false;
    }
    !NOT_A_VERDICT_SUBJECT.contains(head)
}

regexes!(LOOKING_FORWARD, A_CANT_WAIT, A_WAITING_IN, A_WAITING_FOR_A_TURN_UP, A_WORKING_DAY, NOT_LOOKED_FORWARD_TO, A_WAIT_FOR_A_LIFE_TO_END, DREADING, NOT_A_MILD_DREAD);

/// Python: bot/compose/fallback.py:15260 `_dreading(clean)`
/// True for a mild thing coming that the person is not looking forward to.
pub fn _dreading(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    if t.is_empty() || t.contains('?') || t.split(' ').count() > 20 {
        return false;
    }
    DREADING.is_match(&t) && !NOT_A_MILD_DREAD.is_match(&t)
}

/// Python: bot/compose/fallback.py:15270 `_wants_it_over(clean)`
/// True when the thing being waited for is the end of a bad stretch.
pub fn _wants_it_over(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    if t.is_empty() || t.contains('?') || A_WAIT_FOR_A_LIFE_TO_END.is_match(&t) {
        return false;
    }
    LOOKING_FORWARD.is_match(&t) && NOT_LOOKED_FORWARD_TO.is_match(&t)
}

regexes!(DATE_AHEAD, DATE_AHEAD_OWN, AHEAD_NICE, AHEAD_TENSE, NOT_A_PLAN);
pools!(A_DAY_AHEAD, A_DAY_AHEAD_TENSE);

/// Python: bot/compose/fallback.py:15315 `_a_day_ahead(clean)`
/// The pool for a dated thing ahead of the person, or None.
pub fn _a_day_ahead(clean: &str) -> Option<Vec<String>> {
    let t = text::squash(&lower(clean));
    if t.is_empty() || t.contains('?') {
        return None;
    }
    if !DATE_AHEAD.is_match(&t) || !DATE_AHEAD_OWN.is_match(&t) {
        return None;
    }
    if NOT_A_PLAN.is_match(&t) {
        return None;
    }
    if AHEAD_TENSE.is_match(&t) {
        return Some(A_DAY_AHEAD_TENSE.clone());
    }
    if AHEAD_NICE.is_match(&t) {
        return Some(A_DAY_AHEAD.clone());
    }
    None
}

/// Python: bot/compose/fallback.py:15336 `_the_wait_is_for_someone_to_turn_up(text)` (never called on the trace corpus)
/// True when the only looking forward in the line is a "cant wait" that means being at home.
pub fn _the_wait_is_for_someone_to_turn_up(text_: &str) -> bool {
    let hits = A_CANT_WAIT.finditer(text_);
    if hits.is_empty() {
        return false;
    }
    let mut left = text_.to_string();
    // Python cuts each hit out of the progressively shortened string, at the original offsets
    // (code points), clamped the way a slice is.
    for hit in &hits {
        let (s, e) = (text::char_index(text_, hit.start()) as isize, text::char_index(text_, hit.end()) as isize);
        left = format!("{} {}", text::slice(&left, 0, Some(s)), text::slice(&left, e, None));
    }
    if LOOKING_FORWARD.is_match(&left) {
        return false;
    }
    let working = A_WORKING_DAY.is_match(text_);
    for hit in &hits {
        let after = &text_[hit.end()..];
        if A_WAITING_IN.match_(after).is_some() {
            continue;
        }
        if working && A_WAITING_FOR_A_TURN_UP.is_match(after) {
            continue;
        }
        return false;
    }
    true
}

/// Python: bot/compose/fallback.py:15361 `_looking_forward(clean)`
/// True when the line is looking forward to something still to come.
pub fn _looking_forward(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    if t.is_empty() || t.contains('?') {
        return false;
    }
    if !LOOKING_FORWARD.is_match(&t) || A_WAIT_FOR_A_LIFE_TO_END.is_match(&t) {
        return false;
    }
    if _the_wait_is_for_someone_to_turn_up(&t) {
        return false;
    }
    !NOT_LOOKED_FORWARD_TO.is_match(&t)
}

/// Python: bot/compose/fallback.py:15373 `_a_good_time(clean, memory=None)`
/// True when the line reports a good time had by a group or by an event.
pub fn _a_good_time(clean: &str, memory: &Memory) -> bool {
    if clean.contains('?') || clean.contains(" but ") || clean.contains(" until ") {
        return false;
    }
    if GOOD_DAY_OURS.is_match(clean) || GOOD_DAY_WENT.is_match(clean) || GOOD_DAY_BARE.is_match(clean) || GOOD_TIME_IT.match_(clean).is_some() {
        return true;
    }
    if _a_verdict(clean, memory) {
        return true;
    }
    if QUESTION_OPENERS.contains(text::split_ref(clean).first().copied().unwrap_or("")) {
        return false;
    }
    if GOOD_TIME_OURS.is_match(clean) {
        return true;
    }
    if !GOOD_TIME_SUBJECT.is_match(clean) {
        return false;
    }
    rx("[a-z]+").findall_str(clean).iter().any(|w| GOOD_TIME_WORDS.contains(w))
}

regexes!(A_FILM, A_LAUGH_ITSELF, A_ROOMS_LAUGH, LAUGHING, LAUGHING_AT);

/// Python: bot/compose/fallback.py:15458 `_something_funny(clean, memory=None)`
/// True when the line says something was funny and nothing in it says a problem.
pub fn _something_funny(clean: &str, memory: &Memory) -> bool {
    if clean.contains('?') || clean.contains(" but ") {
        return false;
    }
    if LAUGHING_AT.is_match(clean) || A_FILM.is_match(clean) || HARM_IN_A_LAUGH.is_match(clean) || LAUGH_STOPPED.is_match(clean) || A_LAUGH_ITSELF.is_match(clean) {
        return false;
    }
    if A_ROOMS_LAUGH.is_match(clean) && !A_OWN_LAUGH.is_match(clean) {
        return false;
    }
    if _in_a_care_thread(memory) {
        return false;
    }
    LAUGHING.is_match(clean)
}

regexes!(STEPPED_AWAY, GOT_DRAWN_AWAY, A_TURN_THE_PAGE, ABSENCE, _SORRY_LEAD, SPEAKER_ABSENCE, AN_APOLOGY_IN_THE_LINE);
word_sets!(A_GAP_FOR_ILLNESS, A_GAP_NAMED_PERSON, A_PAUSE_WORD, A_HOW_THE_TIME_WENT);

/// Python: bot/compose/fallback.py:15554 `_the_gap_is_all_there_is(clean)` (never called on the trace corpus)
/// True when nothing is left of the line once the apology for the gap is taken off.
pub fn _the_gap_is_all_there_is(clean: &str) -> bool {
    let mut t = text::squash(&lower(clean).replace('\u{2019}', "'"));
    if t.is_empty() {
        return false;
    }
    let errand = STEPPED_AWAY.is_match(&t);
    let spare = if errand { 1 } else { 0 };
    for pattern in [&*STEPPED_AWAY, &*ABSENCE, &*GOT_DRAWN_AWAY, &*_SORRY_LEAD] {
        for _ in 0..4 {
            let Some(m) = pattern.search(&t) else { break };
            if m.as_str().is_empty() {
                break;
            }
            let joined = format!("{} {}", &t[..m.start()], &t[m.end()..]);
            t = text::strip(&joined).to_string();
        }
    }
    for m in A_TURN_THE_PAGE.finditer(&t) {
        if rx("[a-z']").is_match(&t[m.end()..]) {
            return false;
        }
    }
    let left: Vec<String> = az_words(&t).into_iter().filter(|w| !A_PAUSE_WORD.contains(w)).collect();
    let rest = &left[spare.min(left.len())..];
    if rest.is_empty() {
        return true;
    }
    if !rest.iter().any(|w| A_HOW_THE_TIME_WENT.contains(w) || A_GAP_FOR_ILLNESS.contains(w)) {
        return false;
    }
    left.len() <= spare + 6 && rest.iter().all(|w| A_HOW_THE_TIME_WENT.contains(w) || A_GAP_FOR_ILLNESS.contains(w) || A_GAP_NAMED_PERSON.contains(w))
}

/// Python: bot/compose/fallback.py:15608 `reads_as_pause(clean)`
/// True when the line is an apology for the conversation having paused.
pub fn reads_as_pause(clean: &str) -> bool {
    let t = text::squash(&lower(clean).replace('\u{2019}', "'"));
    if STEPPED_AWAY.is_match(&t) || GOT_DRAWN_AWAY.is_match(&t) {
        return _the_gap_is_all_there_is(&t);
    }
    if AN_APOLOGY_IN_THE_LINE.is_match(&t) && ABSENCE.is_match(&t) {
        return _the_gap_is_all_there_is(&t);
    }
    if SPEAKER_ABSENCE.is_match(&t) {
        return _the_gap_is_all_there_is(&t);
    }
    false
}

/// Python: bot/compose/fallback.py:15637 `_came_back(clean)`
pub fn _came_back(clean: &str) -> bool {
    reads_as_pause(clean)
}

regexes!(POINTS_BACK);
word_sets!(A_POINTING_FILLER, A_THING_MARKER);
pools!(RETURNED_TO_IT_FIRST);

/// Python: bot/compose/fallback.py:15670 `_a_chat_behind_it(memory)`
/// True when the person has already said something, so there is a thread to point back at.
pub fn _a_chat_behind_it(memory: &Memory) -> bool {
    crate::memory::history::an_earlier_turn(memory, "")
}

/// Python: bot/compose/fallback.py:15683 `nothing_said_yet(memory)`
/// True on a chat's first line.
pub fn nothing_said_yet(memory: &Memory) -> bool {
    !crate::memory::history::an_earlier_turn(memory, "")
}

/// Python: bot/compose/fallback.py:15688 `nothing_yet_line()` (never called on the trace corpus)
/// The line for a first line whose every candidate answers a turn that never happened.
pub fn nothing_yet_line() -> String {
    RETURNED_TO_IT_FIRST.first().cloned().unwrap_or_default()
}

/// Python: bot/compose/fallback.py:15693 `brings_nothing_of_its_own(text)`
/// True when the line has no subject of its own, only a point back at the conversation.
pub fn brings_nothing_of_its_own(text_: &str) -> bool {
    crate::memory::history::brings_nothing_of_its_own(text_)
}

/// Python: bot/compose/fallback.py:15707 `assumes_an_earlier_turn(reply)`
/// Whether the reply answers something the person has said earlier in this chat.
pub fn assumes_an_earlier_turn(reply: &str) -> bool {
    crate::memory::history::a_past_that_is_not_there(reply, &Value::dict(), Some(""))
}

/// Python: bot/compose/fallback.py:15723 `reads_as_a_return(clean)`
/// What is left of a line that picks the thread back up: None, "" or the thing it names. (The
/// skeleton's `()` return could not carry it.)
pub fn reads_as_a_return(clean: &str) -> Option<String> {
    let t = text::squash(&lower(clean).replace('\u{2019}', "'"));
    if t.is_empty() || !POINTS_BACK.is_match(&t) {
        return None;
    }
    let mut left = t.clone();
    for _ in 0..3 {
        let Some(m) = POINTS_BACK.search(&left) else { break };
        if m.as_str().is_empty() {
            break;
        }
        let joined = format!("{} {}", &left[..m.start()], &left[m.end()..]);
        left = text::strip(&joined).to_string();
    }
    let tokens: Vec<String> = text::split_ref(&left).iter().map(|w| rx("[^a-z0-9']").sub("", w)).filter(|w| !w.is_empty()).collect();
    let marker = tokens.iter().position(|w| A_THING_MARKER.contains(w));
    let Some(marker) = marker else {
        return if tokens.iter().all(|w| A_POINTING_FILLER.contains(w)) { Some(String::new()) } else { None };
    };
    if !tokens[..marker].iter().all(|w| A_POINTING_FILLER.contains(w)) {
        return None;
    }
    let thing = tokens[marker + 1..].iter().filter(|w| !w.is_empty()).cloned().collect::<Vec<_>>().join(" ");
    let thing = thing.trim_matches([' ', ',', '.', '-', '\u{2026}']).to_string();
    if thing.is_empty() || text::split_ref(&thing).len() > 4 {
        return None;
    }
    Some(thing)
}

regexes!(SORRY_ASKING, AM_I_A_NUISANCE, THANKS_OPENS, THANKS_WENT_BADLY, THANKS_GOOD_AGAIN, THANKS_TAKES_A_QUESTION, _ACKNOWLEDGES_ALONE);
word_sets!(MANNERS_ONLY, NUISANCE_ONLY, THANKS_ONLY, THANK_WORDS);

/// Python: bot/compose/fallback.py:15768 `_only_manners(clean, allowed)`
/// True when every word of the line is in the allowed set, so it names nothing of its own.
pub fn _only_manners(clean: &str, allowed: &HashSet<String>) -> bool {
    let words = az_words(&lower(clean));
    !words.is_empty() && words.iter().all(|w| allowed.contains(w))
}

/// Python: bot/compose/fallback.py:15774 `_sorry_for_asking(clean)`
/// True when the line is an apology for asking and nothing else.
pub fn _sorry_for_asking(clean: &str) -> bool {
    let t = lower(clean);
    SORRY_ASKING.is_match(&t) && _only_manners(&t, &MANNERS_ONLY)
}

/// Python: bot/compose/fallback.py:15803 `_am_i_a_nuisance(clean)`
/// True when the line only checks whether the person is being a nuisance.
pub fn _am_i_a_nuisance(clean: &str) -> bool {
    let t = text::squash(&lower(clean));
    AM_I_A_NUISANCE.is_match(&t) && _only_manners(&t, &NUISANCE_ONLY)
}

/// Python: bot/compose/fallback.py:15859 `_thanks_then_said(clean)`
/// The outcome they came back to tell me after a thank-you, or "" when there is none.
pub fn _thanks_then_said(clean: &str) -> String {
    let line = text::strip(text::strip(clean).trim_end_matches(['.', '!']));
    let Some(m) = THANKS_OPENS.match_(line) else { return String::new() };
    let said = text::strip(m.name_str("said")).trim_matches([' ', '.', ',', '!', ';', ':']).to_string();
    if az_words(&said).len() < 2 {
        return String::new();
    }
    if THANKS_TAKES_A_QUESTION.is_match(&THANKS_GOOD_AGAIN.sub(" ", &said)) {
        return String::new();
    }
    said
}

/// Python: bot/compose/fallback.py:15878 `_thanks_news_went_badly(clean)`
/// The outcome when a thank-you came back with bad news, or "" when it did not.
pub fn _thanks_news_went_badly(clean: &str) -> String {
    let said = _thanks_then_said(clean);
    if said.is_empty() || THANKS_GOOD_AGAIN.is_match(&said) {
        return String::new();
    }
    if THANKS_WENT_BADLY.is_match(&said) {
        return said;
    }
    let words = az_words(&said);
    if _mood(&words, &said) == "negative" { said } else { String::new() }
}

/// Python: bot/compose/fallback.py:15914 `_acknowledges_alone(clean)`
/// True when the whole line is "good to know" and nothing else.
pub fn _acknowledges_alone(clean: &str) -> bool {
    _ACKNOWLEDGES_ALONE.match_(&text::squash(&lower(clean))).is_some()
}

/// Python: bot/compose/fallback.py:15924 `_thanks_with_more(clean)`
/// True when the line is thanks and nothing else, however many words it takes to say it.
pub fn _thanks_with_more(clean: &str) -> bool {
    let t = lower(clean);
    if t.contains('?') {
        return false;
    }
    if !az_words(&t).iter().any(|w| THANK_WORDS.contains(w)) {
        return false;
    }
    _only_manners(&t, &THANKS_ONLY)
}

strings!(CARE_THREAD, THREAD, PAUSE);

/// Python: bot/compose/fallback.py:15935 `reading(text, memory)`
/// `(kind, target)` for a line the composer answers itself, or `("", ())` for an ordinary one,
/// as a tuple value.
pub fn reading(text_: &str, memory: &Memory) -> Value {
    let (kind, target) = reading_of(text_, memory);
    let target = Value::Tuple(std::sync::Arc::new(target.into_iter().map(Value::from).collect()));
    Value::Tuple(std::sync::Arc::new(vec![Value::from(kind), target]))
}

/// `reading` as a kind and its targets.
pub fn reading_of(text_: &str, memory: &Memory) -> (String, Vec<String>) {
    let continuation = py_try(|| careful(text_, memory)).flatten();
    if let Some(continuation) = continuation.filter(|c| !c.is_empty()) {
        let kind = if _a_care_thread(memory) { CARE_THREAD.clone() } else { THREAD.clone() };
        return (kind, vec![continuation]);
    }
    match py_try(|| crate::compose::leaving::reads_as_leaving(&text::squash(&lower(text_)), None)) {
        Some(true) => return (data::string("bot.compose.leaving.LEAVING"), Vec::new()),
        Some(false) => {}
        None => return (String::new(), Vec::new()),
    }
    let rest = py_try(|| {
        if !_grief_thread(memory) && !in_a_safety_thread(memory) && reads_as_pause(&text::squash(&lower(text_))) {
            return Some(PAUSE.clone());
        }
        if !_grief_thread(memory) && !in_a_safety_thread(memory) && _a_chat_behind_it(memory) && reads_as_a_return(text_).is_some() {
            return Some(PAUSE.clone());
        }
        None
    });
    match rest {
        Some(Some(kind)) => (kind, Vec::new()),
        _ => (String::new(), Vec::new()),
    }
}
