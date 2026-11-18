//! Port of `bot/modules/answers.py`.
//!
//! GENERATED SKELETON: every function is a stub until its workstream ports it. The
//! signatures are the contract other files call against; keep them.
#![allow(unused_variables, unused_imports, non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::py::re::Re;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use crate::{compose, knowledge, memory, understanding};
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Python behaviour shared by this workstream's modules (answers, greeting, their_life).

/// What another file's function returned, read the way the Python caller reads `str or None`.
///
/// Several contract signatures in other workstreams' files return `()` where the trace only ever
/// saw None (the function returns a reply or None in Python). Calling them through `.py()` keeps
/// this file compiling against the stub today and picks the value up unchanged once the owner
/// gives the function its real return type (`Option<String>`, `String`, a `Value`).
pub trait PyRet {
    fn py(self) -> Option<String>;
}
impl PyRet for () {
    fn py(self) -> Option<String> {
        None
    }
}
impl PyRet for Option<String> {
    fn py(self) -> Option<String> {
        self
    }
}
impl PyRet for String {
    /// A `str` contract maps Python's None to "".
    fn py(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
}
impl PyRet for Option<&str> {
    fn py(self) -> Option<String> {
        self.map(str::to_string)
    }
}
impl PyRet for &str {
    fn py(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self.to_string()) }
    }
}
impl PyRet for Value {
    fn py(self) -> Option<String> {
        match self {
            Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    }
}
impl PyRet for Option<Value> {
    fn py(self) -> Option<String> {
        self.and_then(PyRet::py)
    }
}

/// A pool (a tuple of lines) another file's function returned, or None; read like [`PyRet`].
pub trait PyPool {
    fn pool(self) -> Option<Vec<String>>;
}
impl PyPool for () {
    fn pool(self) -> Option<Vec<String>> {
        None
    }
}
impl PyPool for Vec<String> {
    fn pool(self) -> Option<Vec<String>> {
        if self.is_empty() { None } else { Some(self) }
    }
}
impl PyPool for Option<Vec<String>> {
    fn pool(self) -> Option<Vec<String>> {
        self.filter(|p| !p.is_empty())
    }
}
impl PyPool for Value {
    fn pool(self) -> Option<Vec<String>> {
        if self.truthy() { Some(self.strs()) } else { None }
    }
}

/// `try: ... except Exception: <fallback>` - a panic (a Python exception) is caught and None
/// returned, except a not-yet-ported stub's ("UNPORTED ..."), which is passed on so the checkers can count it.
pub fn py_try<T>(f: impl FnOnce() -> T) -> Option<T> {
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

/// Python `int(s)` for a string of digits: an optional sign, surrounding whitespace, Unicode
/// decimal digits (as `\d` captures them and Python's `int` reads them) and single underscores
/// between digits. None where Python's `int()` would raise.
pub fn py_int(s: &str) -> Option<i64> {
    let t = crate::py::text::strip(s);
    let (neg, digits) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    if digits.is_empty() || digits.starts_with('_') || digits.ends_with('_') || digits.contains("__") {
        return None;
    }
    let mut v: i64 = 0;
    for c in digits.chars() {
        if c == '_' {
            continue;
        }
        let d = decimal_value(c)?;
        v = v.saturating_mul(10).saturating_add(d as i64);
    }
    Some(if neg { -v } else { v })
}

/// The value of one Unicode decimal digit (`unicodedata.decimal(c)`): decimal digits come in
/// runs of ten from zero, so the value is the distance from the start of the run, mod 10.
fn decimal_value(c: char) -> Option<u32> {
    if c.is_ascii_digit() {
        return Some(c as u32 - '0' as u32);
    }
    use crate::understanding::pyuni::ch_decimal;
    if !ch_decimal(c) {
        return None;
    }
    let mut start = c as u32;
    while start > 0 {
        match char::from_u32(start - 1) {
            Some(p) if ch_decimal(p) => start -= 1,
            _ => break,
        }
    }
    Some((c as u32 - start) % 10)
}

/// A pool global of another module, as `(POOL,)` is passed to `fallback._in_a_pool`.
pub fn pools(names: &[&str]) -> Value {
    Value::Tuple(std::sync::Arc::new(names.iter().map(|n| data::get(n).clone()).collect()))
}

// ---------------------------------------------------------------------------------------------
// Module data (bot/modules/answers.py), read from the bundle by Python name.

const A: &str = "bot.modules.answers.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{A}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{A}{name}"))
}
fn set_of(name: &str) -> HashSet<String> {
    data::set(&format!("{A}{name}"))
}
fn strs_of(name: &str) -> Vec<String> {
    data::strs(&format!("{A}{name}"))
}

macro_rules! re_static {
    ($($name:ident),* $(,)?) => { $(static $name: LazyLock<&'static Re> = LazyLock::new(|| rx(stringify!($name)));)* };
}
macro_rules! set_static {
    ($($name:ident),* $(,)?) => { $(static $name: LazyLock<HashSet<String>> = LazyLock::new(|| set_of(stringify!($name)));)* };
}
macro_rules! strs_static {
    ($($name:ident),* $(,)?) => { $(static $name: LazyLock<Vec<String>> = LazyLock::new(|| strs_of(stringify!($name)));)* };
}

/// An inline `re.compile(pattern, flags)` / `re.search(pattern, ...)`, compiled once.
macro_rules! inline_re {
    ($pat:expr) => {{
        static R: LazyLock<Re> = LazyLock::new(|| Re::new($pat, 0));
        &*R
    }};
    ($pat:expr, $flags:expr) => {{
        static R: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
        &*R
    }};
}

use crate::py::re::I as RE_I;
use crate::py::re::S as RE_S;
use crate::py::text;
use std::collections::HashSet;

re_static!(_BARE_VERDICT, WORD, SENTENCE, PICKED, A_SENTENCE_BREAK, A_CLAUSE_FINE, A_CLAUSE_BREAK, A_OPENS_A_CLAUSE,
    A_SAYS_SOMETHING, ASKS_ABOUT_HOW_IT_TURNED_OUT, STEP_END, SPOKEN, TIMEISH, _JOINED_ON, ASK_VERB, A_PLEA_FOR_TIPS,
    NUMBER_IN_TEXT);
set_static!(REFERRING, NOT_A_DETAIL, BACK_POINTS, DETERMINER_BEFORE_DETAIL, DETERMINERS, STOP_BEFORE_A_PART, PRAISE_WORD,
    NOTHING_WORD, SUBJECT_WORD, FILLER_WORD, A_QUALITY_WORD, A_PLACE_IN_IT, A_JOINING_WORD, NOT_IN_A_NAME,
    A_SAYS_NOTHING, BARE, A_CORRECTION_WORD, CLAUSE_OPENER, CLAUSE_SUBJECT, NUMBER_OR_OWNER, ASK_ONLY);
strs_static!(AGAIN, SAID_ABOVE, NOTHING_SEPARATE, GENERIC_KEEPING);

static NO_FIGURE: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| g("NO_FIGURE").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect());
static NO_STEP: LazyLock<String> = LazyLock::new(|| g("NO_STEP").str().to_string());

/// `lead._CAUSE_SENTENCE` (bot/compose/lead.py).
static LEAD_CAUSE_SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.lead._CAUSE_SENTENCE"));

fn lookup<'a>(table: &'a [(String, String)], key: &str) -> Option<&'a str> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// `WORD.findall(s)` (WORD has no group).
fn words_in(s: &str) -> Vec<String> {
    WORD.findall_str(s)
}

/// `str[:1].isupper()`: the first character is an upper-case letter.
fn first_is_upper(s: &str) -> bool {
    s.chars().next().map_or(false, |c| crate::understanding::pyuni::is_upper(&c.to_string()))
}

/// `s[:1].upper() + s[1:]`
fn up1(s: &str) -> String {
    text::upper_first(s)
}

/// `s[:1].lower() + s[1:]`
fn low1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// The bot lines of the last `keep` turns (`turns[-keep:]`).
fn last_turns(memory: &Memory, keep: usize) -> &[Turn] {
    let t = &memory.turns;
    &t[t.len().saturating_sub(keep)..]
}

/// `difflib.SequenceMatcher(None, a, b).ratio()` over characters, CPython's algorithm (autojunk
/// on, so a character in more than 1% of a long `b` is "popular" and only reached by extension).
pub fn sequence_ratio(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (la, lb) = (a.len(), b.len());
    if la + lb == 0 {
        return 1.0;
    }
    let mut b2j: std::collections::HashMap<char, Vec<usize>> = std::collections::HashMap::new();
    for (j, c) in b.iter().enumerate() {
        b2j.entry(*c).or_default().push(j);
    }
    if lb >= 200 {
        let ntest = lb / 100 + 1;
        b2j.retain(|_, idxs| idxs.len() <= ntest);
    }
    let find_longest = |alo: usize, ahi: usize, blo: usize, bhi: usize| -> (usize, usize, usize) {
        let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0usize);
        let mut j2len: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for i in alo..ahi {
            let mut newj2len = std::collections::HashMap::new();
            if let Some(js) = b2j.get(&a[i]) {
                for &j in js {
                    if j < blo {
                        continue;
                    }
                    if j >= bhi {
                        break;
                    }
                    let k = j.checked_sub(1).and_then(|p| j2len.get(&p)).copied().unwrap_or(0) + 1;
                    newj2len.insert(j, k);
                    if k > bestsize {
                        besti = i + 1 - k;
                        bestj = j + 1 - k;
                        bestsize = k;
                    }
                }
            }
            j2len = newj2len;
        }
        // no junk: extend over equal elements on both sides (popular ones included)
        while besti > alo && bestj > blo && a[besti - 1] == b[bestj - 1] {
            besti -= 1;
            bestj -= 1;
            bestsize += 1;
        }
        while besti + bestsize < ahi && bestj + bestsize < bhi && a[besti + bestsize] == b[bestj + bestsize] {
            bestsize += 1;
        }
        (besti, bestj, bestsize)
    };
    let mut queue = vec![(0usize, la, 0usize, lb)];
    let mut matches = 0usize;
    while let Some((alo, ahi, blo, bhi)) = queue.pop() {
        let (i, j, k) = find_longest(alo, ahi, blo, bhi);
        if k > 0 {
            matches += k;
            if alo < i && blo < j {
                queue.push((alo, i, blo, j));
            }
            if i + k < ahi && j + k < bhi {
                queue.push((i + k, ahi, j + k, bhi));
            }
        }
    }
    2.0 * matches as f64 / (la + lb) as f64
}

// ---------------------------------------------------------------------------------------------

/// Python: bot/modules/answers.py:101 `_the_new_part(reply, memory, keep=3)`
/// The part of the reply the last answer did not already say, when it stands on its own.
pub fn _the_new_part(reply: &str, memory: &Memory, keep: i64) -> Option<String> {
    let said = text::strip(reply);
    let mine = memory::narrow::flat(said);
    if mine.is_empty() {
        return None;
    }
    for turn in last_turns(memory, keep as usize) {
        let before = text::strip(&turn.bot);
        let opening = memory::narrow::flat(before);
        if opening.is_empty() || !mine.starts_with(&opening) || text::len(&mine) <= text::len(&opening) + 15 {
            continue;
        }
        if !said.starts_with(before) {
            continue;
        }
        let added = text::strip(&said[before.len()..]);
        if first_is_upper(added) && text::ends_with_any(added, &[".", "!", "?"]) {
            return Some(added.to_string());
        }
    }
    None
}

/// Python: bot/modules/answers.py:137 `_without_the_verdict(text)`
/// The reply without a sentence-long verdict in front of it.
pub fn _without_the_verdict(text_: &str) -> String {
    let said = text::squash(text_);
    let Some(found) = _BARE_VERDICT.match_(&said) else { return said };
    let head = text::strip(found.as_str()).to_lowercase();
    if text::split_ref(&head).len() > 6 {
        return said;
    }
    let verdict = inline_re!(
        r"\b(?:yes|no|not|never|always|usually|often|sometimes|normally|depends|mostly|generally|rarely|seldom)\b"
    );
    if !verdict.is_match(&head) {
        return said;
    }
    text::lstrip(&said[found.end()..]).to_string()
}

/// Python: bot/modules/answers.py:155 `_card_openings()`
/// The first few words of every card in the knowledge base, read once.
pub fn _card_openings() -> &'static HashSet<String> {
    static HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| {
        let mut heads = HashSet::new();
        for item in crate::knowledge::entry::entries() {
            if let crate::knowledge::entry::Reply::Text(said) = &item.reply {
                let flat = text::squash(said);
                if !flat.is_empty() {
                    heads.insert(text::slice(&flat, 0, Some(40)).to_lowercase());
                }
            }
        }
        heads
    });
    &HEADS
}

/// Python: bot/modules/answers.py:174 `_without_a_lead(text)`
/// The reply without a sentence a row declared in front of one of its own cards.
pub fn _without_a_lead(text_: &str) -> String {
    let said = text::squash(text_);
    let Some(found) = _BARE_VERDICT.match_(&said) else { return said };
    let rest = text::lstrip(&said[found.end()..]);
    if rest.is_empty() || !_card_openings().contains(&text::slice(rest, 0, Some(40)).to_lowercase()) {
        return said;
    }
    let head = _without_the_verdict(&said);
    if !head.is_empty() && head != said { head } else { said }
}

/// Python: bot/modules/answers.py:195 `_opening(text, words=8)`
/// The first few words of a reply: two answers that open the same way are the same answer.
pub fn _opening(text_: &str, words: i64) -> String {
    let said = _without_the_verdict(text_);
    let mut parts = inline_re!(r"(?<=[.!?])\s+").split(text::strip(&said));
    while parts.len() > 1 && LEAD_CAUSE_SENTENCE.match_(&parts[0]).is_some() {
        parts.remove(0);
    }
    let joined = parts.join(" ").to_lowercase();
    let w = text::split_ref(&joined);
    w[..w.len().min(words.max(0) as usize)].join(" ")
}

/// Python: bot/modules/answers.py:211 `_most_of_it_said(reply, memory, keep=3, least=40)`
/// True when a whole sentence of this answer is a sentence the bot has already given.
pub fn _most_of_it_said(reply: &str, memory: &Memory, keep: i64, least: i64) -> bool {
    let said = last_turns(memory, keep as usize).iter().map(|t| t.bot.as_str()).collect::<Vec<_>>().join(" ");
    if said.is_empty() {
        return false;
    }
    let said = _flat(&said);
    for sentence in inline_re!(r"(?<=[.;!?])\s+").split(reply) {
        let tail = inline_re!(r"[:;]\s+|\s+[-—]\s+").split(&sentence).pop().unwrap_or_default();
        for part in [&sentence, &tail] {
            let flat = _flat(part);
            if text::len(part) as i64 >= least && !flat.is_empty() && said.contains(&flat) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/answers.py:232 `_said_before(reply, memory, keep=3)`
/// True when the bot has already given this answer (or one that opens the same way).
pub fn _said_before(reply: &str, memory: &Memory, keep: i64) -> bool {
    let mine = _opening(reply, 8);
    if mine.is_empty() {
        return false;
    }
    last_turns(memory, keep as usize).iter().any(|turn| _opening(&turn.bot, 8) == mine)
}

/// Python: bot/modules/answers.py:441 `_in_short(reply)`
/// The first sentence of an answer, or the first two when the first is only a lead-in.
pub fn _in_short(reply: &str) -> String {
    let reply = _without_a_lead(reply);
    let parts = memory::quotes::sentences(&reply, g("SENTENCE"));
    if parts.is_empty() {
        return reply;
    }
    if text::len(&parts[0]) >= 60 || parts.len() == 1 {
        return text::strip(&parts[0]).to_string();
    }
    text::strip(&format!("{} {}", parts[0], parts[1])).to_string()
}

/// Python: bot/modules/answers.py:460 `_same_as_the_last_answer(text, memory)`
/// True when this answer is the one the person has just been given, in any words.
pub fn _same_as_the_last_answer(text_: &str, memory: &Memory) -> bool {
    let last = memory::last_bot(memory);
    if last.is_empty() {
        return false;
    }
    if _flat(text_) == _flat(&last) {
        return true;
    }
    let named_now: HashSet<String> = PICKED.findall_str(text_).into_iter().collect();
    let named_last: HashSet<String> = PICKED.findall_str(&last).into_iter().collect();
    !named_now.is_empty() && named_now == named_last
}

/// Python: bot/modules/answers.py:482 `_the_same_answer_in_other_words(text, memory)`
/// True when this answer says what is already on the screen, close enough to notice.
pub fn _the_same_answer_in_other_words(text_: &str, memory: &Memory) -> bool {
    let last = memory::last_bot(memory);
    if last.is_empty() {
        return false;
    }
    sequence_ratio(&_flat(text_), &_flat(&last)) >= 0.7
}

/// Python: bot/modules/answers.py:498 `_flat(text)`
/// A line reduced to its words, so a repeated question matches whatever its punctuation.
pub fn _flat(text_: &str) -> String {
    words_in(&text_.to_lowercase()).join(" ")
}

/// Python: bot/modules/answers.py:528 `_says_something(clause)`
pub fn _says_something(clause: &str) -> bool {
    A_SAYS_SOMETHING.is_match(clause) || inline_re!(r"\d").is_match(clause)
}

/// Python: bot/modules/answers.py:533 `_the_part_in_the_answer(answer, detail)`
/// The clause of `answer` that names the detail, said as its own sentence, or "".
pub fn _the_part_in_the_answer(answer: &str, detail: &str) -> String {
    let last = text::split_ref(detail).last().map(|w| w.to_lowercase()).unwrap_or_default();
    if text::len(&last) < 3 {
        return String::new();
    }
    if words_in(&answer.to_lowercase()).len() < 5 {
        return String::new();
    }
    for splitter in [*A_CLAUSE_BREAK, *A_CLAUSE_FINE] {
        for sentence in A_SENTENCE_BREAK.split(answer) {
            for clause in splitter.split(&sentence) {
                let clause = text::strip_chars(&clause, " ,;:-\u{2014}");
                if clause.is_empty() {
                    continue;
                }
                let body = words_in(&clause.to_lowercase());
                if !body.iter().any(|word| _names_it(word, &last)) {
                    continue;
                }
                if body.len() < 4 || body.len() > 18 {
                    continue;
                }
                if A_OPENS_A_CLAUSE.match_(clause).is_some() {
                    continue;
                }
                if A_SAYS_NOTHING.contains(&body[0]) {
                    continue;
                }
                if !_says_something(clause) {
                    continue;
                }
                let mut clause = up1(clause);
                if !clause.ends_with(['.', '!', '?']) {
                    clause.push('.');
                }
                return clause;
            }
        }
    }
    String::new()
}

/// Python: bot/modules/answers.py:579 `_names_it(word, last)`
/// Whether a word of the answer is the detail by name, or the stem of it.
pub fn _names_it(word: &str, last: &str) -> bool {
    if word == last {
        return true;
    }
    if text::len(word) < 4 || text::len(last) < 4 {
        return false;
    }
    word.starts_with(last) || last.starts_with(word)
}

/// Python: bot/modules/answers.py:612 `_a_part_named_in_the_line(text, memory, answer)`
/// The part a follow-up names in the plural ("the pubs", "the trams"), or "".
pub fn _a_part_named_in_the_line(text_: &str, memory: &Memory, answer: &str) -> String {
    let raw = words_in(&text_.to_lowercase());
    let wanted: HashSet<String> =
        memory::topics::keywords(text_).into_iter().filter(|w| text::len(w) > 2 && !ASK_ONLY.contains(w)).collect();
    let already = _said_before_now(text_, Some(memory));
    for index in (0..raw.len()).rev() {
        let word = &raw[index];
        if already.contains(word) || PRAISE_WORD.contains(word) || NOTHING_WORD.contains(word) {
            continue;
        }
        if text::len(word) < 4 || !word.ends_with('s') {
            continue;
        }
        let stem = &word[..word.len() - 1];
        if !wanted.contains(stem) {
            continue;
        }
        if !_named_in(answer, word) {
            continue;
        }
        let mut words = vec![word.clone()];
        let mut before = index as isize - 1;
        while before >= 0 && words.len() < 3 {
            let w = &raw[before as usize];
            if DETERMINERS.contains(w)
                || ASK_ONLY.contains(w)
                || STOP_BEFORE_A_PART.contains(w)
                || PRAISE_WORD.contains(w)
                || NOT_IN_A_NAME.contains(w)
                || A_QUALITY_WORD.contains(w)
            {
                break;
            }
            words.insert(0, w.clone());
            before -= 1;
        }
        if words.iter().any(|w| SUBJECT_WORD.contains(w)) {
            continue;
        }
        let name = words.join(" ");
        if before >= 0 && DETERMINERS.contains(&raw[before as usize]) {
            return format!("{} {name}", raw[before as usize]);
        }
        return name;
    }
    String::new()
}

/// Python: bot/modules/answers.py:656 `_short_form_of_the_last(answer, memory)`
/// The answer last turn, when this turn's answer is only its opening, else "".
pub fn _short_form_of_the_last(answer: &str, memory: &Memory) -> String {
    let last_bot = memory::last_bot(memory);
    let last = text::strip(&last_bot);
    let mine = _flat(answer);
    if last.is_empty() || mine.is_empty() || mine == _flat(last) {
        return String::new();
    }
    if !_flat(last).starts_with(&mine) {
        return String::new();
    }
    last.to_string()
}

/// Python: bot/modules/answers.py:667 `_answer_said(plan, memory)`
/// How many of the turns just before this one already gave the plan's sentence.
pub fn _answer_said(plan: &str, memory: &Memory) -> i64 {
    let mine = _flat(plan);
    if mine.is_empty() {
        return 0;
    }
    let mut count = 0;
    for turn in memory.turns.iter().rev() {
        if !_flat(&turn.bot).contains(&mine) {
            break;
        }
        if let Some(module) = &turn.module {
            if !module.is_empty() && module != "answers" {
                continue;
            }
        }
        count += 1;
    }
    count
}

/// Python: bot/modules/answers.py:701 `_carries(text)`
/// The words of a line that say what it is about.
pub fn _carries(text_: &str) -> HashSet<String> {
    words_in(&text_.to_lowercase()).into_iter().filter(|w| text::len(w) > 2 && !BARE.contains(w)).collect()
}

/// Python: bot/modules/answers.py:707 `_the_sentence_asked(reply, text)`
/// The sentence of an answer that answers a line asking for one part of it.
pub fn _the_sentence_asked(reply: &str, text_: &str) -> Option<String> {
    let sentences: Vec<String> = SENTENCE.split(reply).into_iter().filter(|p| !text::strip(p).is_empty()).collect();
    if sentences.len() < 2 {
        return None;
    }
    let wanted = _carries(text_);
    let (mut best, mut best_score, mut best_length): (Option<String>, usize, usize) = (None, 0, text::len(&sentences[0]));
    for sentence in &sentences {
        let score = _carries(sentence).intersection(&wanted).count();
        let length = text::len(sentence);
        if score > 0 && (score > best_score || (score == best_score && length < best_length)) {
            best = Some(sentence.clone());
            best_score = score;
            best_length = length;
        }
    }
    let mut best = best?;
    let stripped = text::strip(&best).to_string();
    if let Some(joined) = _JOINED_ON.match_(&stripped) {
        best = up1(&stripped[joined.end()..]);
    }
    Some(best)
}

/// Python: bot/modules/answers.py:741 `_the_sentence_naming(reply, text, memory=None)`
/// The later sentence of the answer that carries the part the line names, or None.
pub fn _the_sentence_naming(reply: &str, text_: &str, memory: Option<&Memory>) -> Option<String> {
    let detail = _the_detail_named(text_, memory, false);
    if detail.is_empty() {
        return None;
    }
    let sentences: Vec<String> =
        SENTENCE.split(reply).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    if sentences.len() < 2 {
        return None;
    }
    let word = text::split_ref(&detail).last().copied().unwrap_or("").to_string();
    for sentence in &sentences[1..] {
        if words_in(&sentence.to_lowercase()).iter().any(|other| similar(&word, other)) {
            return Some(sentence.clone());
        }
    }
    None
}

/// Python: bot/modules/answers.py:771 `_repeat_short(reply, text, memory=None)`
/// The short form of an answer given a second time: the part the line asks, or the opening.
pub fn _repeat_short(reply: &str, text_: &str, memory: Option<&Memory>) -> String {
    let reply = _without_a_lead(reply);
    let part = if memory::narrow::kind_of(text_).is_some_and(|k| !k.is_empty()) {
        _the_sentence_asked(&reply, text_)
    } else {
        None
    };
    if let Some(part) = part.filter(|p| !p.is_empty()) {
        return part;
    }
    if let Some(named) = _the_sentence_naming(&reply, text_, memory).filter(|p| !p.is_empty()) {
        return named;
    }
    _in_short(&reply)
}

/// Python: bot/modules/answers.py:820 `_first_step(reply)`
/// The first step of an answer, short enough to say on its own.
pub fn _first_step(reply: &str) -> String {
    let mut opening = text::strip(&SENTENCE.split(text::strip(reply))[0]).to_string();
    if let Some(joined) = _JOINED_ON.match_(&opening) {
        let rest = opening[joined.end()..].to_string();
        opening = up1(&rest);
    }
    if opening.is_empty() {
        return String::new();
    }
    if text::split_ref(&opening).len() <= 16 {
        return if opening.ends_with(['.', '!', '?']) { opening } else { format!("{opening}.") };
    }
    let first = STEP_END
        .split(&opening)
        .iter()
        .map(|one| text::strip_chars(one, " ,;:").to_string())
        .find(|one| !one.is_empty())
        .unwrap_or_default();
    let after_colon = first.rsplit(": ").next().unwrap_or("").to_string();
    for piece in [&first, &after_colon] {
        let n = text::split_ref(piece).len();
        if (4..=20).contains(&n) {
            return if piece.ends_with(['.', '!', '?']) { piece.clone() } else { format!("{piece}.") };
        }
    }
    String::new()
}

/// Python: bot/modules/answers.py:840 `_figure_missing(reply, text)`
/// The honest short line for a figure the answer does not hold, or None.
pub fn _figure_missing(reply: &str, text_: &str) -> Option<String> {
    let kind = memory::narrow::kind_of(text_)?;
    let no_figure = lookup(&NO_FIGURE, &kind)?;
    if _answers_the_figure(text_, reply) {
        return None;
    }
    if (kind == "time" || kind == "clock") && (SPOKEN.is_match(reply) || TIMEISH.is_match(reply)) {
        return None;
    }
    let step = _first_step(reply);
    if step.is_empty() {
        return Some(format!("{no_figure}{}", *NO_STEP));
    }
    Some(format!("{no_figure}{}", low1(&step)))
}

/// Python: bot/modules/answers.py:863 `_figures_changed(reply, said)`
/// True when both carry figures and the answer has one the earlier line did not.
pub fn _figures_changed(reply: &str, said: &str) -> bool {
    let mine: HashSet<String> = NUMBER_IN_TEXT.findall_str(reply).into_iter().collect();
    let theirs: HashSet<String> = NUMBER_IN_TEXT.findall_str(said).into_iter().collect();
    !mine.is_empty() && !theirs.is_empty() && mine.difference(&theirs).next().is_some()
}

/// Python: bot/modules/answers.py:870 `_same_answer(reply, said)`
/// True when `said` is this answer, or the short form of it, with or without a lead-in.
pub fn _same_answer(reply: &str, said: &str) -> bool {
    let mine = _flat(reply);
    let theirs = _flat(said);
    if mine.is_empty() || theirs.is_empty() {
        return false;
    }
    if theirs.contains(&_opening(&mine, 8)) {
        return !_figures_changed(reply, said);
    }
    for sentence in SENTENCE.split(reply) {
        let part = _flat(&sentence);
        if text::len(&part) > 20 && theirs.contains(&part) {
            return !_figures_changed(reply, said);
        }
    }
    for clause in said.split(':') {
        let cut = _flat(clause);
        if text::split_ref(&cut).len() >= 6 && mine.contains(&cut) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:905 `_asks_of_another_subject(text, understood)`
/// True when the line, read on its own, belongs to a different subject than the one it leans on.
pub fn _asks_of_another_subject(text_: &str, understood: Option<&Understood>) -> bool {
    let Some(understood) = understood else { return false };
    if text::strip(text_).is_empty() {
        return false;
    }
    let alone = understanding::understand(text_, Some(&Memory::new()));
    let own = alone.topic.clone().unwrap_or_default();
    let held = understood.leans_on.as_ref().and_then(|l| l.topic.clone()).unwrap_or_default();
    !own.is_empty() && !held.is_empty() && own != held
}

/// Python: bot/modules/answers.py:921 `_asked_again(text, reply, memory)`
/// How many times this same line was asked before and given this same answer.
pub fn _asked_again(text_: &str, reply: &str, memory: &Memory) -> i64 {
    let wanted = _flat(text_);
    let mine = _opening(&_flat(reply), 8);
    if wanted.is_empty() || mine.is_empty() {
        return 0;
    }
    let mut count = 0;
    for turn in memory.turns.iter().rev() {
        if _flat(&turn.user) != wanted {
            break;
        }
        if !_same_answer(reply, &turn.bot) {
            break;
        }
        count += 1;
    }
    count
}

/// Python: bot/modules/answers.py:946 `_ask_verb(text)`
/// The verb the ask leans its question on ("stop" in "how do i stop that"), or "".
pub fn _ask_verb(text_: &str) -> String {
    match ASK_VERB.search(text_) {
        Some(m) => m.str(1).to_lowercase(),
        None => String::new(),
    }
}

/// Python: bot/modules/answers.py:952 `_the_outcome_owed(item, text)`
/// The sentence this row owes a line that asks how its subject turns out, or None.
pub fn _the_outcome_owed(item: Option<&'static Entry>, text_: &str) -> Option<String> {
    let which = memory::narrow::the_outcome_asked(text_);
    let item = item?;
    if which.is_empty() {
        return None;
    }
    for (shape, sentence) in &item.outcomes {
        if *shape == which && !text::strip(sentence).is_empty() {
            return Some(sentence.clone());
        }
    }
    None
}

/// Python: bot/modules/answers.py:969 `_said_before_now(text, memory)`
/// The words the person has already used in this chat, before this line.
pub fn _said_before_now(text_: &str, memory: Option<&Memory>) -> HashSet<String> {
    let mut seen = HashSet::new();
    if let Some(memory) = memory {
        for turn in &memory.turns {
            seen.extend(words_in(&turn.user.to_lowercase()));
        }
    }
    seen
}

/// Python: bot/modules/answers.py:987 `_a_fact_about_the_person(memory)`
/// True when the line states something about the person rather than asking about a part.
pub fn _a_fact_about_the_person(memory: &Memory) -> bool {
    let Some(understood) = memory.understood.as_ref() else { return false };
    understood.kind == "statement" && understood.asks.is_empty() && understood.first_person
}

/// Python: bot/modules/answers.py:1003 `_the_detail_named(text, memory=None, with_quality=False)`
/// The part a follow-up names, or "" when it names nothing the answer has not already said.
pub fn _the_detail_named(text_: &str, memory: Option<&Memory>, with_quality: bool) -> String {
    let named = _the_part_named(text_, memory, with_quality);
    if !named.is_empty() && _a_restatement_of_the_answer(memory) && _all_of_it_in_the_answer(&named, memory) {
        return String::new();
    }
    named
}

/// Python: bot/modules/answers.py:1020 `_a_restatement_of_the_answer(memory)`
/// True when the line tells the bot something rather than asking about a part of the answer.
pub fn _a_restatement_of_the_answer(memory: Option<&Memory>) -> bool {
    let Some(understood) = memory.and_then(|m| m.understood.as_ref()) else { return false };
    understood.kind == "statement" && understood.asks.is_empty()
}

/// Python: bot/modules/answers.py:1033 `_all_of_it_in_the_answer(named, memory)`
/// True when the answer just given already says every word of the part named.
pub fn _all_of_it_in_the_answer(named: &str, memory: Option<&Memory>) -> bool {
    let empty = Memory::new();
    let said = words_in(&memory::last_bot(memory.unwrap_or(&empty)).to_lowercase());
    let words: Vec<String> =
        words_in(&named.to_lowercase()).into_iter().filter(|w| text::len(w) > 2 && !A_JOINING_WORD.contains(w)).collect();
    !words.is_empty() && words.iter().all(|w| said.contains(w) || said.contains(&memory::topics::stem(w)))
}

/// Python: bot/modules/answers.py:1051 `_the_clause_s_verb(raw, index)`
/// True when the word at `index` is the verb of a when-or-if clause, whatever the word is.
pub fn _the_clause_s_verb(raw: &[String], index: i64) -> bool {
    let at = |i: i64| raw[i as usize].as_str();
    if index >= 2 && CLAUSE_OPENER.contains(at(index - 2)) && CLAUSE_SUBJECT.contains(at(index - 1)) {
        return true;
    }
    index >= 3 && CLAUSE_OPENER.contains(at(index - 3)) && NUMBER_OR_OWNER.contains(at(index - 2))
}

re_static!(MONEY_SAID, SAID_OUT_LOUD, ABOUT_YOU, WHO_IS_THE_BOT, _A_LENGTH, _A_DURATION, _DEFINES_THE_UNIT, PROPERTY, SOMETHING_FOR,
    SOMETHING_FOR_GIVEN_TO, PLAN_RECALL, RECALL, WHEN_AM_I, ASKS_MEMORY);
set_static!(ONLY_THE_ASK, NEGATORS, A_SMALL_OPENER, MORE_ASK, LENGTH_WORDS, FIRST_PERSON, ASKING_VERBS,
    SOMETHING_FOR_PEOPLE, SOMETHING_FOR_FILLER, SOMETHING_FOR_TAIL, _OCCASION_WORDS, SOMETHING_FOR_STATE);
strs_static!(DETAIL_AGAIN, SAME_ADVICE, NOTHING_MORE);

static ANSWERING_A_QUOTED_ASK: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.understanding.kinds.ANSWERING_A_QUOTED_ASK"));
static KINDS_VERBISH: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.kinds.VERBISH"));
static KINDS_VERB_THIRD: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.kinds.VERB_THIRD"));
/// `same_again.AGAIN` (bot/memory/again.py), not this module's AGAIN.
static AGAIN_SAME: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.again.AGAIN"));
static MEM_KIND_ASK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.KIND_ASK"));
static MODIFIER_WORDS: LazyLock<i64> = LazyLock::new(|| g("MODIFIER_WORDS").as_int().unwrap_or(2));

/// Python `template.format(name=value, ...)` for the `{name}` fields PyBot's pools use (`{{` and
/// `}}` are literal braces). A field that is not given raises KeyError, as in Python.
pub fn py_format(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut chars = template.chars().peekable();
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
                for d in chars.by_ref() {
                    if d == '}' {
                        break;
                    }
                    name.push(d);
                }
                match fields.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => out.push_str(v),
                    None => panic!("KeyError: {name:?}"),
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// `pool[min(times, len(pool)) - 1]`
fn nth_of(pool: &[String], times: i64) -> &str {
    let n = (times.min(pool.len() as i64) - 1).rem_euclid(pool.len() as i64);
    &pool[n as usize]
}

/// `raw[i]` with Python's negative indexing.
fn at_py(raw: &[String], i: isize) -> &str {
    let n = raw.len() as isize;
    let j = if i < 0 { n + i } else { i };
    &raw[j as usize]
}

/// The memory as the plain dict a `memory: Value` contract takes.
pub fn memory_value(memory: &Memory) -> Value {
    Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
}

/// Python: bot/modules/answers.py:1061 `_the_part_named(text, memory=None, with_quality=False)`
/// The word a follow-up names as the part of the thing it asks about ("the biting"), or "".
pub fn _the_part_named(text_: &str, memory: Option<&Memory>, with_quality: bool) -> String {
    let empty = Memory::new();
    if memory::returns::a_bare_return(text_, memory.unwrap_or(&empty)) {
        return String::new();
    }
    let raw = words_in(&text_.to_lowercase());
    if !raw.is_empty() && raw.iter().all(|w| ONLY_THE_ASK.contains(w)) {
        return String::new();
    }
    let wanted: HashSet<String> = memory::topics::keywords(text_)
        .into_iter()
        .filter(|w| text::len(w) > 2 && !ASK_ONLY.contains(w) && !NOT_A_DETAIL.contains(w) && !A_CORRECTION_WORD.contains(w))
        .collect();
    if wanted.is_empty() {
        return String::new();
    }
    let verb = _ask_verb(text_);
    let already = _said_before_now(text_, memory);
    for index in (0..raw.len()).rev() {
        let word = raw[index].as_str();
        if A_JOINING_WORD.contains(word) {
            continue;
        }
        if !wanted.contains(word) && !wanted.contains(&memory::topics::stem(word)) {
            continue;
        }
        if PRAISE_WORD.contains(word)
            || NOTHING_WORD.contains(word)
            || FILLER_WORD.contains(word)
            || (A_QUALITY_WORD.contains(word) && !with_quality)
        {
            continue;
        }
        if !verb.is_empty() && (word == verb || word.starts_with(verb.as_str()) || verb.starts_with(word)) {
            continue;
        }
        // `raw[index - 1]`: at index 0 Python reads the last word
        if !DETERMINER_BEFORE_DETAIL.contains(at_py(&raw, index as isize - 1))
            && (KINDS_VERBISH.contains(word)
                || KINDS_VERB_THIRD.contains(word)
                || _the_clause_s_verb(&raw, index as i64)
                || (text::len(word) > 4 && (word.ends_with("ed") || word.ends_with("ing"))))
        {
            continue;
        }
        if already.contains(word) {
            continue;
        }
        if index + 1 < raw.len() && raw[index + 1] == "to" {
            continue;
        }
        let behind = if index > 0 { raw[index - 1].as_str() } else { "" };
        if A_PLACE_IN_IT.contains(word) && index + 2 < raw.len() && raw[index + 1] == "of" && BACK_POINTS.contains(&raw[index + 2]) {
            return String::new();
        }
        if matches!(behind, "the" | "my" | "his" | "her" | "their" | "your" | "a" | "this" | "that") {
            return format!("{behind} {word}");
        }
        let whole = _named_whole(&raw, index as i64);
        if !whole.is_empty() {
            return whole;
        }
        if !wanted.contains(word) && !_named_in(&memory::last_bot(memory.unwrap_or(&empty)), word) {
            return String::new();
        }
        return word.to_string();
    }
    String::new()
}

/// Python: bot/modules/answers.py:1159 `_only_a_quality_name(text, memory=None)`
/// Whether the part the line names is a quality word and nothing else ("a new"), or False.
pub fn _only_a_quality_name(text_: &str, memory: Option<&Memory>) -> bool {
    let named = _the_detail_named(text_, memory, true);
    if named.is_empty() {
        return false;
    }
    let words: Vec<&str> = text::split_ref(&named).into_iter().filter(|w| !DETERMINERS.contains(*w)).collect();
    !words.is_empty() && words.iter().all(|w| A_QUALITY_WORD.contains(*w))
}

/// Python: bot/modules/answers.py:1181 `_named_whole(raw, index)`
/// The determiner and the words between it and `raw[index]`, when they name one part, or "".
pub fn _named_whole(raw: &[String], index: i64) -> String {
    let index = index as usize;
    if index + 1 < raw.len() && BACK_POINTS.contains(&raw[index + 1]) {
        return String::new();
    }
    let head = raw[index].as_str();
    if KINDS_VERBISH.contains(head)
        || NEGATORS.contains(head)
        || (text::len(head) > 4 && (head.ends_with("ed") || head.ends_with("ing") || head.ends_with('s')))
    {
        return String::new();
    }
    let modifier = *MODIFIER_WORDS;
    let lo = (index as i64 - modifier - 1).max(0) as usize;
    if raw[lo..index].iter().any(|w| NEGATORS.contains(w)) {
        return String::new();
    }
    let stop = (index as i64 - 1 - modifier).max(-1);
    let mut back = index as i64 - 1;
    while back > stop {
        let word = raw[back as usize].as_str();
        let before = if back > 0 { raw[back as usize - 1].as_str() } else { "" };
        if DETERMINER_BEFORE_DETAIL.contains(before) {
            return raw[back as usize - 1..=index].join(" ");
        }
        if STOP_BEFORE_A_PART.contains(word)
            || BACK_POINTS.contains(word)
            || NOT_A_DETAIL.contains(word)
            || FILLER_WORD.contains(word)
            || NOTHING_WORD.contains(word)
            || PRAISE_WORD.contains(word)
            || NOT_IN_A_NAME.contains(word)
            || A_QUALITY_WORD.contains(word)
        {
            return String::new();
        }
        if KINDS_VERBISH.contains(word) || (text::len(word) > 4 && (word.ends_with("ed") || word.ends_with("ing"))) {
            return String::new();
        }
        back -= 1;
    }
    String::new()
}

/// Python: bot/modules/answers.py:1222 `_their_own_state(text)`
/// True when the line is about the person themselves and asks nothing of the thing.
pub fn _their_own_state(text_: &str) -> bool {
    if ASKS_ABOUT_HOW_IT_TURNED_OUT.is_match(text_) || !_ask_verb(text_).is_empty() {
        return false;
    }
    let raw = words_in(&text_.to_lowercase());
    !raw.is_empty() && FIRST_PERSON.contains(&raw[0])
}

/// Python: bot/modules/answers.py:1237 `_named_in(part, detail)`
/// True when `part` says the detail by name, so a lead-in may say the answer covers it.
pub fn _named_in(part: &str, detail: &str) -> bool {
    let last = text::split_ref(detail).last().copied().unwrap_or("").to_string();
    if text::len(&last) < 3 {
        return false;
    }
    let last4 = text::slice(&last, 0, Some(4));
    for word in words_in(&part.to_lowercase()) {
        if word == last {
            return true;
        }
        if text::len(&word) >= 4
            && text::len(&last) >= 4
            && (word.starts_with(&last4) || last.starts_with(&text::slice(&word, 0, Some(4))))
        {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:1259 `similar(word, other)`
/// Two words that are the same thing for this purpose: equal, or one carrying the other.
pub fn similar(word: &str, other: &str) -> bool {
    if word == other {
        return true;
    }
    if text::len(word) < 4 || text::len(other) < 4 {
        return false;
    }
    word.starts_with(other) || other.starts_with(word)
}

/// Python: bot/modules/answers.py:1272 `_entry_words(memory)`
/// The words the entry that has just answered is keyed on, all of them.
pub fn _entry_words(memory: &Memory) -> HashSet<String> {
    let mut kept: Vec<&Value> = Vec::new();
    if let Some(answer) = memory.get("entry_answer").filter(|v| v.is_dict()) {
        kept.push(answer);
    }
    if let Some(Value::List(others)) = memory.get("entry_answers") {
        kept.extend(others.iter().filter(|r| r.is_dict()));
    }
    let mut words = HashSet::new();
    for record in kept {
        if let Some(keys) = record.get("keys") {
            for key in keys.items() {
                words.extend(words_in(&py_str(key).to_lowercase()));
            }
        }
    }
    words
}

/// Python `str(value)` for the values memory holds (strings, numbers, None, booleans).
pub fn py_str(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::None => "None".into(),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::Int(i) => i.to_string(),
        Value::Float(f) => text::float_repr(*f),
        other => other.to_plain().to_string(),
    }
}

/// Python: bot/modules/answers.py:1292 `_what_they_typed(text, memory)`
/// The words the person typed, given a reading with an earlier turn in front of it.
pub fn _what_they_typed(text_: &str, memory: &Memory) -> String {
    if !memory.get("joined_read").is_some_and(Value::truthy) {
        return text_.to_string();
    }
    let topic = memory
        .understood
        .as_ref()
        .and_then(|u| u.leans_on.as_ref())
        .map(|l| text::strip(&l.text).to_string())
        .unwrap_or_default();
    let line = text::strip(text_);
    if !topic.is_empty() && line.to_lowercase().starts_with(&topic.to_lowercase()) {
        return text::strip(&text::slice(line, text::len(&topic) as isize, None)).to_string();
    }
    text_.to_string()
}

/// Python: bot/modules/answers.py:1312 `_covered(text, reply, memory=None, understood=None)`
/// True when the answer on the screen is about everything the line names.
pub fn _covered(text_: &str, reply: &str, memory: &Memory, understood: Option<&Understood>) -> bool {
    let said: HashSet<String> = words_in(&reply.to_lowercase()).into_iter().collect();
    let read = understood.or(memory.understood.as_ref());
    let typed = _what_they_typed(text_, memory);
    let detail = if _asks_of_another_subject(&typed, read) { String::new() } else { _the_detail_named(&typed, Some(memory), false) };
    let words = words_in(&text_.to_lowercase());
    if words.iter().any(|w| BACK_POINTS.contains(w)) {
        return true;
    }
    let verb = _ask_verb(text_);
    let entry = _entry_words(memory);
    let detail_words = text::split_ref(&detail);
    for word in memory::topics::keywords(text_) {
        if text::len(&word) < 3 || ASK_ONLY.contains(&word) || detail_words.contains(&word.as_str()) {
            continue;
        }
        if !verb.is_empty() && (word == verb || word.starts_with(verb.as_str()) || verb.starts_with(word.as_str())) {
            continue;
        }
        if ASKING_VERBS.contains(&word) {
            continue;
        }
        if said.iter().any(|other| similar(&word, other)) {
            continue;
        }
        if entry.iter().any(|other| similar(&word, other)) {
            continue;
        }
        return false;
    }
    true
}

/// Python: bot/modules/answers.py:1364 `_a_question_about_another_thing(text, reply, understood, memory, own=None, item=None)`
/// True when the line asks its own question about a thing the answer on the screen never names.
pub fn _a_question_about_another_thing(
    text_: &str,
    reply: &str,
    understood: Option<&Understood>,
    memory: &Memory,
    own: bool,
    item: Option<&'static Entry>,
) -> bool {
    if own {
        return false;
    }
    let Some(understood) = understood.filter(|u| u.kind == "question") else { return false };
    if text::strip(reply).is_empty() {
        return false;
    }
    let typed = _what_they_typed(text_, memory);
    if !_asks_of_another_subject(&typed, Some(understood)) {
        return false;
    }
    let thing = _the_detail_named(&typed, Some(memory), false);
    let parts = text::split_ref(&thing);
    if parts.len() != 2 {
        return false;
    }
    let word = parts[1];
    if words_in(&reply.to_lowercase()).iter().any(|other| similar(word, other)) {
        return false;
    }
    let mut held = _entry_words(memory);
    if let Some(item) = item {
        held.extend(_key_words(item));
    }
    if held.iter().any(|other| similar(word, other)) {
        return false;
    }
    let named: Vec<String> =
        memory::topics::keywords(&typed).into_iter().filter(|one| text::len(one) > 2 && !ASK_ONLY.contains(one)).collect();
    if named.iter().any(|one| held.iter().any(|other| similar(one, other))) {
        return false;
    }
    true
}

/// `string.punctuation`
const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// Python: bot/modules/answers.py:1444 `_small_letter(body)`
/// `body` with a sentence-opening pronoun, article or verb put in lower case after a lead-in.
pub fn _small_letter(body: &str) -> String {
    let (first, _, rest) = text::partition(body, " ");
    let bare = text::rstrip_chars(first, PUNCTUATION);
    let lowered = || {
        let head = text::slice(first, 0, Some(1)).to_lowercase();
        format!("{head}{}", text::slice(body, 1, None))
    };
    if A_SMALL_OPENER.contains(&bare.to_lowercase()) {
        return lowered();
    }
    if first_is_upper(bare) {
        let again = Re::new(&format!("(?<![A-Za-z]){}(?![A-Za-z])", crate::py::re::escape(&bare.to_lowercase())), 0);
        if again.is_match(rest) {
            return lowered();
        }
    }
    body.to_string()
}

/// Python: bot/modules/answers.py:1460 `_times_the_answer_was_said(reply, memory)`
/// How many turns in a row the chat has just had this answer, counted before this one.
pub fn _times_the_answer_was_said(reply: &str, memory: &Memory) -> i64 {
    let mut times = 0;
    for turn in last_turns(memory, 4).iter().rev() {
        if !_same_answer(reply, &turn.bot) {
            break;
        }
        times += 1;
    }
    times
}

/// Python: bot/modules/answers.py:1470 `_with_detail(detail, part, times, text='', memory=None, card='')`
/// `part` with the detail in front of it, said as what it is: the subject, or the same advice.
pub fn _with_detail(detail: &str, part: &str, times: i64, text_: &str, memory: &Memory, card: &str) -> String {
    let body = text::strip(part);
    if detail.is_empty() || body.is_empty() {
        return String::new();
    }
    let underneath = memory::again::the_same_thing_underneath(text_, body, times, memory);
    if !underneath.is_empty() {
        return underneath;
    }
    if text::split_ref(detail).last().is_some_and(|w| NOT_A_DETAIL.contains(*w)) {
        return py_format(nth_of(&AGAIN_SAME, times), &[("reply", body)]);
    }
    let named = memory::again::the_name_the_answer_says(body, detail, card);
    if named.is_empty() && _a_yes_no_ask(memory) {
        return py_format(nth_of(&AGAIN_SAME, times), &[("reply", body)]);
    }
    let pool: &[String] = if !named.is_empty() { &DETAIL_AGAIN } else { &SAME_ADVICE };
    let detail_said = if !named.is_empty() { named.as_str() } else { detail };
    py_format(nth_of(pool, times), &[("detail", detail_said), ("reply", &_small_letter(body))])
}

/// Python: bot/modules/answers.py:1506 `_a_yes_no_ask(memory)`
/// True when the line asks yes or no ("would a service help").
pub fn _a_yes_no_ask(memory: &Memory) -> bool {
    memory.understood.as_ref().is_some_and(|u| u.asks.iter().any(|a| a == "yesno"))
}

/// Python: bot/modules/answers.py:1532 `_asks_for_more(text)`
/// True when the line asks for the rest of the answer rather than about a part of it.
pub fn _asks_for_more(text_: &str) -> bool {
    memory::topics::keywords(text_).iter().any(|w| MORE_ASK.contains(w))
}

/// Python: bot/modules/answers.py:1537 `_nothing_more_said_again(reply, text, memory)`
/// The same answer for a line that names no part of it, said as nothing further, or "".
pub fn _nothing_more_said_again(reply: &str, text_: &str, memory: &Memory) -> String {
    if !_asks_for_more(text_) {
        return String::new();
    }
    let times = _times_the_answer_was_said(reply, memory);
    if times == 0 {
        return String::new();
    }
    let mut short = _in_short(reply);
    if short.is_empty() {
        short = reply.to_string();
    }
    let underneath = memory::again::the_same_thing_underneath(text_, &short, times, memory);
    if !underneath.is_empty() {
        return underneath;
    }
    py_format(nth_of(&NOTHING_MORE, times), &[("reply", &short)])
}

/// Python: bot/modules/answers.py:1555 `_the_detail_said_again(reply, text, memory)`
/// The same answer given again for a detail named inside it, or None.
pub fn _the_detail_said_again(reply: &str, text_: &str, memory: &Memory) -> Option<String> {
    let detail = _the_detail_named(text_, Some(memory), false);
    if detail.is_empty() {
        let picked = if memory::narrow::kind_of(text_).is_some_and(|k| !k.is_empty()) {
            _the_sentence_asked(reply, text_)
        } else {
            None
        };
        if let Some(picked) = picked {
            if !text::strip(&picked).is_empty() && memory::narrow::flat(&picked) != memory::narrow::flat(reply) {
                let times = _times_the_answer_was_said(reply, memory);
                if times > 0 {
                    let underneath = memory::again::the_same_thing_underneath(text_, &picked, times, memory);
                    if !underneath.is_empty() {
                        return Some(underneath);
                    }
                    return Some(py_format(nth_of(&AGAIN_SAME, times), &[("reply", &picked)]));
                }
            }
        }
        return Some(_nothing_more_said_again(reply, text_, memory));
    }
    let times = _times_the_answer_was_said(reply, memory);
    if times == 0 {
        return None;
    }
    let mut short = memory::narrow::narrow(reply, text_, "").unwrap_or_default();
    if text::strip(&short).is_empty() || memory::narrow::flat(&short) == memory::narrow::flat(reply) {
        short = _repeat_short(reply, text_, Some(memory));
        if short.is_empty() {
            short = reply.to_string();
        }
    }
    Some(_with_detail(&detail, &short, times, text_, memory, reply))
}

/// Python: bot/modules/answers.py:1591 `_detail_lead_in(text, part, memory)`
/// `part` with the detail the line named in front of it, when the part is already on screen.
pub fn _detail_lead_in(text_: &str, part: &str, memory: &Memory) -> String {
    let detail = _the_detail_named(text_, Some(memory), false);
    if detail.is_empty() || text::strip(part).is_empty() {
        return String::new();
    }
    let said = memory::narrow::flat(part);
    let mut times = 0;
    for turn in last_turns(memory, 4).iter().rev() {
        if !said.is_empty() && memory::narrow::flat(&turn.bot).contains(&said) {
            times += 1;
            continue;
        }
        break;
    }
    if times == 0 {
        return String::new();
    }
    _with_detail(&detail, part, times, text_, memory, &_the_answer_on_screen(memory))
}

/// Python: bot/modules/answers.py:1610 `_the_answer_on_screen(memory)`
/// The whole answer the person is looking at, which is the dictionary a lead-in may name from.
pub fn _the_answer_on_screen(memory: &Memory) -> String {
    for turn in last_turns(memory, 2).iter().rev() {
        if !text::strip(&turn.bot).is_empty() {
            return turn.bot.clone();
        }
    }
    String::new()
}

/// Python: bot/modules/answers.py:1619 `_the_part_said_again(part, text, understood, memory)`
/// The part of the last answer, with the detail the line named in front of it when needed.
pub fn _the_part_said_again(part: &str, text_: &str, understood: Option<&Understood>, memory: &Memory) -> String {
    let none = Understood::default();
    if !text::strip(part).is_empty() && memory::topics::a_detail_of_the_thing_we_hold(understood.unwrap_or(&none), memory) {
        let lead = _detail_lead_in(text_, part, memory);
        if !lead.is_empty() {
            return lead;
        }
    }
    part.to_string()
}

/// Python: bot/modules/answers.py:1633 `_already_said(reply, memory, keep=3)`
/// True when the bot has just given this answer, with only a lead-in in front of it.
pub fn _already_said(reply: &str, memory: &Memory, keep: i64) -> bool {
    let mine = text::squash(&reply.to_lowercase());
    if mine.is_empty() {
        return false;
    }
    for turn in last_turns(memory, keep as usize) {
        let bot = text::squash(&turn.bot.to_lowercase());
        if !bot.is_empty() && (bot == mine || bot.ends_with(&mine) || _the_same_sentences(&bot, &mine)) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:1654 `_the_same_sentences(said, mine)`
/// True when two replies are the same sentences with one of them moved to the front.
pub fn _the_same_sentences(said: &str, mine: &str) -> bool {
    if said.is_empty() || mine.is_empty() || said == mine {
        return false;
    }
    let splitter = inline_re!(r"(?<=[.!?])\s+");
    let mut parts: Vec<String> = splitter.split(said).into_iter().filter(|p| !p.is_empty()).map(|p| p.to_lowercase()).collect();
    let mut others: Vec<String> = splitter.split(mine).into_iter().filter(|p| !p.is_empty()).map(|p| p.to_lowercase()).collect();
    if parts.len() <= 1 || parts.len() != others.len() {
        return false;
    }
    parts.sort();
    others.sort();
    parts == others
}

/// Python: bot/modules/answers.py:1694 `_asks_who_the_bot_is(understood, text)`
/// True when the line asks who or what the bot is, not what the last answer was about.
pub fn _asks_who_the_bot_is(understood: Option<&Understood>, text_: &str) -> bool {
    if ANSWERING_A_QUOTED_ASK.is_match(text_) && !WHO_IS_THE_BOT.is_match(text_) {
        return false;
    }
    if understood.is_some_and(|u| u.about_bot) {
        return true;
    }
    WHO_IS_THE_BOT.is_match(text_)
}

/// Python: bot/modules/answers.py:1739 `_answers_a_length(item, reply)`
/// Whether this entry answers a length ask: its own key asks one and its answer gives one.
pub fn _answers_a_length(item: Option<&'static Entry>, reply: &str) -> bool {
    if reply.is_empty() || ["\u{a3}", "$", "\u{20ac}"].iter().any(|m| reply.contains(m)) {
        return false;
    }
    if _DEFINES_THE_UNIT.match_(reply).is_some() {
        return false;
    }
    if !_A_LENGTH.is_match(reply) {
        return false;
    }
    let Some(item) = item else { return false };
    for part in &item.key {
        for phrase in part.words() {
            if text::split_ref(&phrase.to_lowercase()).iter().any(|w| LENGTH_WORDS.contains(*w)) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/answers.py:1755 `_written_ask_with_an_answer(text, memory)`
/// The duration ask written out with the action the chat is on, when an entry answers it.
pub fn _written_ask_with_an_answer(text_: &str, memory: &mut Memory) -> Option<String> {
    let written = memory::topics::written_out(text_, memory).py()?;
    let landed = py_try(|| {
        let asked = understanding::understand(&written, Some(&*memory));
        let (found, found_reply) = _offer_with(
            &mut |_: &mut Memory, reject: Reject| reader_entry(knowledge::lookup::find(&asked, None, reject, false)),
            memory,
            &asked,
        );
        _answers_a_length(found, &found_reply.unwrap_or_default())
    });
    match landed {
        Some(true) => Some(written),
        _ => None,
    }
}

/// Python: bot/modules/answers.py:1779 `_the_article(subject, memory)`
/// The article a rewritten line puts in front of the thing: "the " or, for a name, nothing.
pub fn _the_article(subject: &str, memory: &Memory) -> String {
    let head = text::strip_chars(text::split_ref(subject).first().copied().unwrap_or(""), ".,!?;:'\"").to_lowercase();
    if head.is_empty() {
        return "the ".into();
    }
    let Some(user) = memory.get("user").filter(|u| u.is_dict()) else { return "the ".into() };
    let mut names: HashSet<String> = HashSet::new();
    for key in ["name", "city"] {
        if let Some(Value::Str(value)) = user.get(key) {
            if !text::strip(value).is_empty() {
                names.extend(text::split_ref(value).iter().map(|w| w.to_lowercase()));
            }
        }
    }
    for key in ["places", "people", "pets"] {
        if let Some(found) = user.get(key).filter(|v| v.is_list()) {
            for value in found.items() {
                if let Value::Str(value) = value {
                    if !text::strip(value).is_empty() {
                        names.extend(text::split_ref(value).iter().map(|w| w.to_lowercase()));
                    }
                }
            }
        }
    }
    if !names.contains(&head) { "the ".into() } else { String::new() }
}

/// Python: bot/modules/answers.py:1809 `_with_subject(text, memory, about=None)`
/// The follow-up written out with the thing it refers to, or the line itself.
pub fn _with_subject(text_: &str, memory: &mut Memory, about: Option<&str>) -> String {
    if let Some(written) = _written_ask_with_an_answer(text_, memory) {
        return written;
    }
    let mut words: Vec<String> = text::split(text_);
    if words.len() > 14 {
        return text_.to_string();
    }
    if ABOUT_YOU.is_match(text_) {
        return text_.to_string();
    }
    let subject = match about {
        Some(a) => a.to_string(),
        None => memory::topics::subject(memory),
    };
    if subject.is_empty() {
        return text_.to_string();
    }
    let clean = |w: &str| text::strip_chars(w, ".,!?;:").to_lowercase();
    let wanted: Vec<String> = text::split_ref(&subject).iter().map(|w| clean(w)).collect();
    let said: Vec<String> = words.iter().map(|w| clean(w)).collect();
    if wanted.iter().any(|w| said.contains(w)) {
        return text_.to_string();
    }
    for (i, word) in said.iter().enumerate() {
        if REFERRING.contains(word) {
            words[i] = format!("{}{subject}", _the_article(&subject, memory));
            return words.join(" ");
        }
    }
    text_.to_string()
}

/// Python: bot/modules/answers.py:1843 `_about_subject(text, memory, about=None)`
/// "What temperature should the oven be?" -> "... for the birthday cake?".
pub fn _about_subject(text_: &str, memory: &Memory, about: Option<&str>) -> String {
    let asks_which_one = MEM_KIND_ASK.is_match(text_);
    if text::split_ref(text_).len() > 14 || !(PROPERTY.is_match(text_) || asks_which_one) {
        return text_.to_string();
    }
    let subject = match about {
        Some(a) => a.to_string(),
        None => memory::topics::subject(memory),
    };
    if subject.is_empty() || !memory::topics::current(memory).is_some_and(Value::truthy) {
        return text_.to_string();
    }
    let low = text_.to_lowercase();
    if text::split_ref(&subject).iter().any(|w| low.contains(&text::strip_chars(w, ".,!?;:").to_lowercase())) {
        return text_.to_string();
    }
    format!("{} for {}{subject}?", text::rstrip_chars(text_, "?.! "), _the_article(&subject, memory))
}

/// Python: bot/modules/answers.py:1917 `_keyed_on_the_person(item, who)`
/// Whether the entry answers for this kind of person, not just for the word in the line.
pub fn _keyed_on_the_person(item: &Entry, who: &str) -> bool {
    if who.is_empty() {
        return false;
    }
    let keys: HashSet<String> = _key_words(item).into_iter().collect();
    let person = text::rstrip_chars(who, "s");
    if keys.contains(person) {
        return true;
    }
    const CHILD: [&str; 9] = ["baby", "toddler", "child", "kid", "son", "daughter", "newborn", "boy", "girl"];
    keys.iter().any(|k| CHILD.contains(&k.as_str())) && (CHILD.contains(&person) || person == "teenager" || person == "teen")
}

/// Python: bot/modules/answers.py:1934 `_something_for(text)`
/// The problem from "i need something for a headache" as a question, or "".
pub fn _something_for(text_: &str) -> String {
    let flat = _flat(text_);
    let Some(m) = SOMETHING_FOR.search(&flat) else { return String::new() };
    let mut rest: Vec<String> = inline_re!(r"\s+")
        .split(text::strip_chars(m.str(1), " ?.!"))
        .into_iter()
        .filter(|w| !w.is_empty())
        .collect();
    while rest.last().is_some_and(|w| SOMETHING_FOR_TAIL.contains(w)) {
        rest.pop();
    }
    while rest.first().is_some_and(|w| matches!(w.as_str(), "my" | "your" | "his" | "her" | "their" | "our")) {
        rest.remove(0);
    }
    if !(1..=6).contains(&rest.len()) {
        return String::new();
    }
    let ordinal = inline_re!(r"\d{1,3}(?:st|nd|rd|th)?");
    if rest.iter().all(|w| _OCCASION_WORDS.contains(w) || ordinal.fullmatch(w).is_some()) {
        return String::new();
    }
    if rest.iter().any(|w| SOMETHING_FOR_PEOPLE.contains(w)) {
        return String::new();
    }
    if rest.iter().any(|w| w.contains("pregnan") || SOMETHING_FOR_STATE.contains(w)) {
        return String::new();
    }
    if rest.iter().all(|w| SOMETHING_FOR_FILLER.contains(w)) {
        return String::new();
    }
    rest.join(" ")
}

/// Python: bot/modules/answers.py:1986 `_keeps_a_plan(memory)`
/// Whether the store holds a plan to say back.
pub fn _keeps_a_plan(memory: &Memory) -> bool {
    let Some(known) = memory.get("user") else { return false };
    known.get("plan").is_some_and(Value::truthy) || known.get("plans").is_some_and(Value::truthy)
}

/// What a reader hands `_offer`: the entry, and the reading it used when that is another reading
/// than the line's own (Python's `(item, held)` tuple; None stands for `held is reading`).
pub type Offered = Option<(&'static Entry, Option<Understood>)>;

/// A reader of `_offer`: `lambda reject: find(..., reject=reject)`. It is handed the memory
/// Python's lambda would have closed over.
pub type Reader<'a> = dyn FnMut(&mut Memory, Reject) -> Offered + 'a;

pub use crate::knowledge::lookup::Reject;

/// A reader that returns a bare entry (no reading of its own).
pub fn reader_entry(found: Option<&'static Entry>) -> Offered {
    found.map(|e| (e, None))
}

/// Python: bot/modules/answers.py:7021 `_offer(reader, memory, reading)` - the contract form. A
/// reader is a Python callable, which a `Value` cannot hold: callers use [`_offer_with`].
pub fn _offer(reader: &Value, memory: &Memory, reading: &Understood) -> Value {
    let _ = (reader, memory, reading);
    panic!("TypeError: _offer needs a reader function; call answers::_offer_with")
}

/// Python: bot/modules/answers.py:7021 `_offer(reader, memory, reading)`
/// The first entry the reader offers whose reply stands, and that reply.
pub fn _offer_with(reader: &mut Reader, memory: &mut Memory, reading: &Understood) -> (Option<&'static Entry>, Option<String>) {
    let text_here = reading.text.clone();
    let names_it = _names_what_it_wants_priced(&text_here);
    let price_asked = |held: &Understood| _asks_a_price(&held.text);
    let mut declined: HashSet<String> = HashSet::new();
    let refused = _refused(&text_here);
    let mut fallback: Option<(&'static Entry, Option<Understood>)> = None;
    let here = text::strip(&reading.plain).to_lowercase();
    let now = memory.understood.as_ref().map(|u| text::strip(&u.plain).to_lowercase()).unwrap_or_default();
    let words_wanted = if here.is_empty() || now.is_empty() || here == now { _the_ask_is_for_words(reading) } else { false };
    let mut words_fallback: Option<(&'static Entry, Option<Understood>)> = None;
    let mut words_declines = 0;
    loop {
        let got = {
            let declined_now = declined.clone();
            let reject = move |one: &Entry| declined_now.contains(&one.id);
            reader(memory, Some(&reject))
        };
        let Some((item, held_own)) = got else {
            let Some((item, held_own)) = fallback.or(words_fallback) else { return (None, None) };
            let held = held_own.as_ref().unwrap_or(reading);
            let reply = _payload_mut(item, memory, held);
            return if !text::strip(&reply).is_empty() { (Some(item), Some(reply)) } else { (None, None) };
        };
        let held_is_reading = held_own.is_none();
        let held = held_own.clone().unwrap_or_else(|| reading.clone());
        if price_asked(&held) {
            let reply = _payload_mut(item, memory, &held);
            if !text::strip(&reply).is_empty() && !MONEY_SAID.is_match(&reply) {
                if names_it && fallback.is_none() && !_said_before(&reply, memory, 3) {
                    fallback = Some((item, held_own.clone()));
                }
                declined.insert(item.id.clone());
                continue;
            }
        }
        if !(_wants_a_stand_in(reading) && _a_swap_row(item)) && _hands_back_the_refusal(item, &refused) {
            declined.insert(item.id.clone());
            continue;
        }
        let reply = _payload_mut(item, memory, &held);
        if !text::strip(&reply).is_empty()
            && !refused.is_empty()
            && !(_wants_a_stand_in(reading) && _a_swap_row(item))
            && _is_about(item, &refused)
            && !_says_it_leaves_it_out(&reply, &refused)
        {
            declined.insert(item.id.clone());
            continue;
        }
        if !text::strip(&reply).is_empty()
            && words_wanted
            && words_declines < 3
            && held_is_reading
            && !_said_before(&reply, memory, 3)
            && !SAID_OUT_LOUD.is_match(&reply)
            && !_puts_it_to_someone(&reply)
        {
            if words_fallback.is_none() {
                words_fallback = Some((item, held_own.clone()));
            }
            declined.insert(item.id.clone());
            words_declines += 1;
            continue;
        }
        if !text::strip(&reply).is_empty() {
            return (Some(item), Some(reply));
        }
        declined.insert(item.id.clone());
    }
}

/// Python: bot/modules/answers.py:6178 `_reply_of(item, reading, memory)`
/// What this entry would say, without keeping it, so a reply can be read before it is given.
pub fn _reply_of(item: &Entry, reading: &Understood, memory: &Memory) -> String {
    use crate::knowledge::entry::Reply;
    match &item.reply {
        Reply::Text(s) => s.clone(),
        Reply::Choices(c) => c.join(" "),
        Reply::Fn(f) => {
            // inside `try`: an exception is ""; an unported reply is still counted as one
            if !knowledge::registry::has_reply(&f.id) {
                panic!("UNPORTED {}", f.id);
            }
            let mut scratch = memory.clone();
            match knowledge::registry::try_reply(f, &mut scratch, reading) {
                Ok(v) => v.unwrap_or_default(),
                Err(()) => {
                    reraise_unported();
                    String::new()
                }
            }
        }
    }
}

/// After a caught panic: pass it on when it was a stub that is not ported yet.
fn reraise_unported() {
    if let Some(msg) = crate::runtime::take_last_panic() {
        if msg.starts_with("UNPORTED") {
            panic!("{msg}");
        }
    }
}

/// Python: bot/modules/answers.py:6319 `_payload(item, memory, reading)` - the contract form. The
/// entry's reply may write to memory; through `&Memory` that write is not kept (callers that hold
/// the memory mutably use [`_payload_mut`]).
pub fn _payload(item: &Entry, memory: &Memory, reading: &Understood) -> String {
    _payload_mut(item, &mut memory.clone(), reading)
}

/// Python: bot/modules/answers.py:6319 `_payload(item, memory, reading)`
/// The entry's reply, chosen the way the composer will choose it, or "" if it declines.
pub fn _payload_mut(item: &Entry, memory: &mut Memory, reading: &Understood) -> String {
    use crate::knowledge::entry::Reply;
    match &item.reply {
        Reply::Text(s) => s.clone(),
        // `pick(reply, memory)`; an exported None reply comes through as an empty pool
        Reply::Choices(c) if c.is_empty() => String::new(),
        Reply::Choices(c) => knowledge::d1_dup_g1::pick(c, memory, ""),
        Reply::Fn(f) => {
            // no `try` here: an exception goes up to the module, as in Python
            if !knowledge::registry::has_reply(&f.id) {
                panic!("UNPORTED {}", f.id);
            }
            match knowledge::registry::try_reply(f, memory, reading) {
                Ok(v) => v.unwrap_or_default(),
                Err(()) => {
                    reraise_unported();
                    panic!("entry reply {} raised", f.id);
                }
            }
        }
    }
}

re_static!(CHOOSING, CHOOSING_PRONOUN, HOLDING_A_NUMBER, MODAL, MONTH, TIME_WORD, QUALIFIER_OPEN, REFUSE_OPEN,
    REFUSE_CHOICE, REFUSE_WISH, REFUSAL_ACK, _ASKS_AFTER_A_TASTE, LACK_OPEN, AMOUNT);
set_static!(WHEN_WORDS, FINITE, WEAK_ACKS, CHOICE_NOUNS, SAID_POLITELY, POSSESSIVE, PERSON_WORDS);
strs_static!(MODALS, TRIP_TOPICS);
static PREPOSITION_OF: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| g("PREPOSITION_OF").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect());
static QUALIFIER_MAX_WORDS: LazyLock<usize> = LazyLock::new(|| g("QUALIFIER_MAX_WORDS").as_int().unwrap_or(9) as usize);
static OWN_THING_AT: LazyLock<usize> = LazyLock::new(|| g("OWN_THING_AT").as_int().unwrap_or(3) as usize);
static OWN_THING_SCAN: LazyLock<usize> = LazyLock::new(|| g("OWN_THING_SCAN").as_int().unwrap_or(12) as usize);

/// Python `repr(value)` for a string, list or tuple of strings (what `str(reply)` gives for an
/// entry whose reply is a pool).
pub fn py_repr(v: &Value) -> String {
    match v {
        Value::Str(s) => crate::understood::py_str_repr(s),
        Value::List(items) => format!("[{}]", items.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Tuple(items) if items.len() == 1 => format!("({},)", py_repr(&items[0])),
        Value::Tuple(items) => format!("({})", items.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        other => py_str(other),
    }
}

/// `find(reading)` with no reject and no context.
fn find_plain(reading: &Understood) -> Option<&'static Entry> {
    knowledge::lookup::find(reading, None, None, false)
}

/// Python: bot/modules/answers.py:2041 `_with_subject_tail(text, memory, understood=None)`
/// The follow-up with what the conversation is about put on the end, or "".
pub fn _with_subject_tail(text_: &str, memory: &Memory, understood: Option<&Understood>) -> String {
    _subject_tails(text_, memory, understood).into_iter().next().unwrap_or_default()
}

/// Python: bot/modules/answers.py:2049 `_subject_tails(text, memory, understood=None)`
/// The line with the subject on the end, longest subject first, then shorter runs of it.
pub fn _subject_tails(text_: &str, memory: &Memory, understood: Option<&Understood>) -> Vec<String> {
    let subject = memory::topics::subject(memory);
    let empty = Understood::default();
    let understood = understood.or(memory.understood.as_ref()).unwrap_or(&empty);
    if subject.is_empty() || text::split_ref(text_).len() > 10 || text::split_ref(&subject).len() > 4 {
        return vec![];
    }
    if understood.kind == "statement" || understood.asks.is_empty() {
        return vec![];
    }
    if understood.about_bot || ABOUT_YOU.is_match(text_) {
        return vec![];
    }
    if inline_re!(r"\s*(?:what|which|how|who|when|where|why)\b", RE_I).match_(&subject).is_some() {
        return vec![];
    }
    if RECALL.is_match(text_) || ASKS_MEMORY.is_match(text_) {
        return vec![];
    }
    let said: HashSet<String> = text::split_ref(text_).iter().map(|w| text::strip_chars(w, ".,!?;:").to_lowercase()).collect();
    let words = text::split(&subject);
    if words.iter().any(|w| said.contains(&w.to_lowercase())) {
        return vec![];
    }
    let stem = text::rstrip_chars(text_, "?.,! ");
    if words.len() == 1 {
        return vec![format!("{stem} {}", words[0])];
    }
    let mut tails = Vec::new();
    for start in 0..words.len().saturating_sub(1) {
        let tail = words[start..].join(" ");
        if text::split_ref(&tail).len() < 2 {
            break;
        }
        tails.push(format!("{stem} {tail}"));
    }
    tails
}

/// Python: bot/modules/answers.py:2121 `_bare_subject(subject)`
/// The subject without the words that say when or how many: "rome july" -> "rome".
pub fn _bare_subject(subject: &str) -> String {
    let words: Vec<&str> = text::split_ref(subject)
        .into_iter()
        .filter(|w| !WHEN_WORDS.contains(&w.to_lowercase()) && HOLDING_A_NUMBER.match_(w).is_none())
        .collect();
    if words.is_empty() { subject.to_string() } else { words.join(" ") }
}

/// Python: bot/modules/answers.py:2154 `_the_month(text)`
pub fn _the_month(text_: &str) -> String {
    MONTH.search(text_).map(|m| m.str(1).to_lowercase()).unwrap_or_default()
}

/// Python: bot/modules/answers.py:2165 `_a_trip(memory)`
/// Whether the conversation is about going somewhere, so a month means a season there.
pub fn _a_trip(memory: &Memory) -> bool {
    let mut lines: Vec<String> =
        vec![memory::topics::current(memory).and_then(|c| c.get("text")).map(|t| t.str().to_string()).unwrap_or_default()];
    match memory::last_user(memory, 5) {
        Value::List(items) | Value::Tuple(items) => lines.extend(items.iter().map(|i| i.str().to_string())),
        other => lines.push(other.str().to_string()),
    }
    for line in &lines {
        if line.is_empty() {
            continue;
        }
        if understanding::understand(line, Some(memory)).topics.iter().any(|t| TRIP_TOPICS.contains(t)) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:2193 `_a_clause(subject)`
/// True when the subject carries a verb, so a preposition cannot be hung on it.
pub fn _a_clause(subject: &str) -> bool {
    text::split_ref(subject).iter().any(|w| FINITE.contains(&text::strip_chars(w, PUNCTUATION).to_lowercase()))
}

/// Python: bot/modules/answers.py:2198 `_choosing(text, memory)`
/// The entry a bare choosing question is about, with the reading and the reply.
pub fn _choosing(text_: &str, memory: &mut Memory) -> Option<(Understood, &'static Entry, String)> {
    let m = CHOOSING.search(text::strip(text_))?;
    let verb = m.name_str("verb").to_lowercase();
    let subject = _bare_subject(&memory::topics::subject(memory));
    if subject.is_empty() {
        return None;
    }
    if TIME_WORD.match_(text::strip(&subject)).is_some() {
        return None;
    }
    let a_clause = _a_clause(&subject);
    let prep = lookup(&PREPOSITION_OF, &verb).expect("KeyError: the verb's preposition").to_string();
    let mut shapes = vec![format!("{prep} {{s}}")];
    if prep == "to" {
        shapes.push("to a {s}".to_string());
    }
    let recent_user = match memory::last_user(memory, 3) {
        Value::List(items) => items.iter().map(|i| i.str().to_string()).collect::<Vec<_>>().join(" "),
        Value::None => String::new(),
        other => py_str(&other),
    };
    let mut month = _the_month(text_);
    if month.is_empty() {
        month = _the_month(&memory::topics::topic_text(memory));
    }
    if month.is_empty() {
        month = _the_month(&recent_user);
    }
    if !month.is_empty() && _a_trip(memory) {
        let mut seasoned: Vec<String> = shapes.iter().map(|s| format!("{s} in {month}")).collect();
        seasoned.extend(shapes);
        shapes = seasoned;
    }
    let stem = text::strip(text::rstrip_chars(&CHOOSING_PRONOUN.sub("", text::strip(text_)), "?")).to_string();
    let mut declined: HashSet<String> = HashSet::new();
    let mut said: Option<(Understood, &'static Entry, String)> = None;
    for modal in MODALS.iter() {
        let asked = MODAL.sub(modal, &stem);
        for shape in &shapes {
            let fuller = format!("{asked} {}", py_format(shape, &[("s", &subject)]));
            let reading = understanding::understand(&fuller, Some(&*memory));
            let Some(item) = find_plain(&reading) else { continue };
            if declined.contains(&item.id) {
                continue;
            }
            if !(_is_about(item, &subject) || (!month.is_empty() && _a_trip(memory) && _is_about(item, &month))) {
                continue;
            }
            if a_clause && !knowledge::lookup::about_the_topic(item, &reading) {
                continue;
            }
            let reply = _payload_mut(item, memory, &reading);
            if text::strip(&reply).is_empty() {
                declined.insert(item.id.clone());
                continue;
            }
            if _said_before(&reply, memory, 3) {
                let part = memory::narrow::narrow(&reply, text_, "");
                if let Some(part) = part {
                    if !text::strip(&part).is_empty()
                        && memory::narrow::flat(&part) != memory::narrow::flat(&reply)
                        && !_said_before(&part, memory, 3)
                    {
                        return Some((reading, item, part));
                    }
                }
                if said.is_none() {
                    said = Some((reading, item, reply));
                }
                continue;
            }
            return Some((reading, item, reply));
        }
    }
    if let Some((reading, item, reply)) = said {
        if _asked_again(text_, &reply, memory) > 0 {
            if _asks_only_the_length(text_) {
                if let Some(honest) = _figure_missing(&reply, text_) {
                    return Some((reading, item, honest));
                }
            }
            return Some((reading, item, py_format(&AGAIN[0], &[("reply", &_in_short(&reply))])));
        }
    }
    None
}

/// Python: bot/modules/answers.py:2285 `_only_a_size_of(item, text)`
/// True when every word of the line is this entry's own, a size, or the ask itself.
pub fn _only_a_size_of(item: &Entry, text_: &str) -> bool {
    let words = _key_words(item);
    for word in memory::topics::keywords(text_) {
        if AMOUNT.is_match(&word) || words.contains(&word) {
            continue;
        }
        return false;
    }
    true
}

/// Python: bot/modules/answers.py:2310 `_choice_words(text, memory)`
/// The domain the chat is in and the word naming what it is choosing between.
pub fn _choice_words(text_: &str, memory: &Memory) -> (Option<String>, Vec<String>) {
    let line = text::squash(text_).to_lowercase();
    let earlier: Vec<String> =
        memory::topics::context(memory, 3).into_iter().filter(|one| text::squash(one).to_lowercase() != line).collect();
    let topic = earlier.last().cloned().unwrap_or_default();
    if topic.is_empty() {
        return (None, vec![]);
    }
    let mut found: Vec<String> = Vec::new();
    let item = find_plain(&understanding::understand(&topic, Some(memory)));
    if let Some(item) = item {
        if let Some(part) = item.key.first() {
            for option in part.words() {
                for word in text::split_ref(option).into_iter().take(1) {
                    let w = word.to_lowercase();
                    if !found.contains(&w) {
                        found.push(w);
                    }
                }
            }
        }
    }
    let domain = item.map(|i| i.domain.clone());
    (domain, found.into_iter().filter(|w| text::len(w) > 2).collect())
}

/// Python: bot/modules/answers.py:2343 `_qualifier_rewrites(text, memory)`
/// The qualifier line with one word of the choice added, one rewrite per candidate word.
pub fn _qualifier_rewrites(text_: &str, memory: &Memory) -> Vec<(String, String, Option<String>)> {
    let clean = text::squash(text_);
    if text::split_ref(&clean).len() > *QUALIFIER_MAX_WORDS || QUALIFIER_OPEN.match_(&clean.to_lowercase()).is_none() {
        return vec![];
    }
    if !memory::topics::current(memory).is_some_and(Value::truthy) {
        return vec![];
    }
    let (domain, words) = _choice_words(text_, memory);
    let said: HashSet<String> = text::split_ref(&clean).iter().map(|w| text::strip_chars(w, ".,!?;:").to_lowercase()).collect();
    let stem = text::rstrip_chars(&clean, "?.! ");
    words.into_iter().filter(|w| !said.contains(w)).map(|w| (format!("{stem} {w}"), w, domain.clone())).collect()
}

/// Python: bot/modules/answers.py:2433 `_a_lack_not_a_refusal(flat, end)`
/// True when the words after the negated word say what is missing, not what to leave out.
pub fn _a_lack_not_a_refusal(flat: &str, end: usize) -> bool {
    for word in text::split_ref(text::strip_chars(&flat[end..], " .,!?;:'\"")) {
        let word = text::strip_chars(word, ".,!?;:'\"").to_lowercase();
        if !word.is_empty() && !CHOICE_NOUNS.contains(&word) && !SAID_POLITELY.contains(&word) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:2442 `_refused(text)`
/// The word a line like "nothing funny" or "anything but a horror" rules out, if any.
pub fn _refused(text_: &str) -> String {
    let flat = text::squash(text_);
    let mut word: Option<String> = REFUSE_CHOICE.match_(&flat).map(|m| m.str(1).to_string());
    if word.is_none() {
        if let Some(m) = REFUSE_OPEN.match_(&flat) {
            if _a_lack_not_a_refusal(&flat, m.end()) {
                return String::new();
            }
            word = Some(m.str(1).to_string());
        } else if let Some(m) = REFUSE_WISH.match_(&flat) {
            if _a_lack_not_a_refusal(&flat, m.end()) && !_ASKS_AFTER_A_TASTE.is_match(&flat[m.end()..]) {
                return String::new();
            }
            word = Some(m.str(1).to_string());
        }
    }
    let Some(word) = word else { return String::new() };
    let word = text::strip_chars(&word, ".,!?;:").to_lowercase();
    if text::len(&word) > 2 { word } else { String::new() }
}

/// Python: bot/modules/answers.py:2476 `_says_it_leaves_it_out(reply, refused)`
/// Whether the answer says the refused thing is left out, not merely uses the same words.
pub fn _says_it_leaves_it_out(reply: &str, refused: &str) -> bool {
    let said = text::squash(&reply.to_lowercase());
    let word = text::strip(refused).to_lowercase();
    if said.is_empty() || word.is_empty() {
        return false;
    }
    let named = Re::new(&format!(r"\b{}\w*\b", crate::py::re::escape(&word)), 0);
    for found in REFUSAL_ACK.finditer(&said) {
        if !WEAK_ACKS.contains(found.as_str()) {
            return true;
        }
        let (start, end) = (text::char_index(&said, found.start()) as isize, text::char_index(&said, found.end()) as isize);
        if named.is_match(&text::slice(&said, end, Some(end + 40))) {
            return true;
        }
        if named.is_match(&text::slice(&said, (start - 30).max(0), Some(start))) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:2504 `_hands_back_the_refusal(item, refused)`
/// True when the entry is the thing the line ruled out and its answer does not skip it.
pub fn _hands_back_the_refusal(item: &Entry, refused: &str) -> bool {
    if refused.is_empty() || !_is_about(item, refused) {
        return false;
    }
    if _about_a_lack(item) {
        return false;
    }
    use crate::knowledge::entry::Reply;
    let said = match &item.reply {
        Reply::Fn(_) => return false,
        Reply::Text(s) => s.clone(),
        // `str(reply or "")`: a pool reads as its repr (the exported tuple or list, when the
        // entry came from the bundle)
        Reply::Choices(c) => match item.raw.attr("reply") {
            Some(v) if v.truthy() => py_repr(v),
            Some(_) => String::new(),
            None if c.is_empty() => String::new(),
            None => py_repr(&Value::Tuple(std::sync::Arc::new(c.iter().map(Value::from).collect()))),
        },
    };
    !_says_it_leaves_it_out(&said, refused)
}

/// Python: bot/modules/answers.py:2528 `_about_a_lack(item)`
/// True when the entry is keyed on a thing being missing, not on choosing one.
pub fn _about_a_lack(item: &Entry) -> bool {
    item.key.iter().any(|part| part.words().iter().any(|option| LACK_OPEN.match_(text::strip(option)).is_some()))
}

/// Python: bot/modules/answers.py:2538 `_key_words(item)`
/// Every word an entry says it is about.
pub fn _key_words(item: &Entry) -> HashSet<String> {
    let mut found = HashSet::new();
    for part in &item.key {
        for option in part.words() {
            for word in text::split_ref(&option.to_lowercase()) {
                let bare = text::rstrip_chars(word, "s");
                found.insert(if bare.is_empty() { word.to_string() } else { bare.to_string() });
            }
        }
    }
    found
}

/// Python: bot/modules/answers.py:2573 `_names_a_thing(understood)`
/// True when the line names something other than a person.
pub fn _names_a_thing(understood: &Understood) -> bool {
    understood.entities.iter().any(|ent| ent.kind != "person" && !ent.kind.is_empty())
}

re_static!(DETAIL_OF, ASK_OF, OBJECT_PRONOUN, SUM_TABLE, PROPERTY_WORD_THAT_NAMES_A_THING);
set_static!(DESCRIBING, DO_VERBS, REPORT_WORDS, ASK_WORDS, SAID_OF_THE_THING, BARE_GENERIC, ASKING, SPEECH, TIME_WORDS,
    PLACE_WORDS, FEELINGS, PROPERTY_WORDS, MONEY_ASK, VAGUE);
static MONTH_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.MONTH_NAMES"));
static SEASONS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.SEASONS"));

/// Whether another file's function returned None, whatever its contract type is today (`()`
/// where the trace only saw None reads as None).
pub trait PyNone {
    fn is_py_none(&self) -> bool;
}
impl PyNone for () {
    fn is_py_none(&self) -> bool {
        true
    }
}
impl<T> PyNone for Option<T> {
    fn is_py_none(&self) -> bool {
        self.is_none()
    }
}
impl PyNone for Value {
    fn is_py_none(&self) -> bool {
        self.is_none()
    }
}
impl PyNone for Understood {
    fn is_py_none(&self) -> bool {
        false
    }
}

/// A memory record's text: `str((record or {}).get("text") or "")`.
fn record_text(record: Option<&Value>) -> String {
    record.and_then(|r| r.get("text")).filter(|t| t.truthy()).map(py_str).unwrap_or_default()
}

/// Python: bot/modules/answers.py:2582 `_own_thing(understood)`
/// The thing the line itself opens with, when it opens with one.
pub fn _own_thing(understood: &Understood) -> String {
    let mut person = String::new();
    for ent in &understood.entities {
        if ent.at >= 0 && ent.at as usize <= *OWN_THING_AT {
            if ent.kind == "person" {
                if person.is_empty() {
                    person = ent.text.clone();
                }
                continue;
            }
            return ent.text.clone();
        }
    }
    if !person.is_empty() {
        return if _names_a_thing(understood) { String::new() } else { person };
    }
    let line = if !understood.clean.is_empty() { &understood.clean } else { &understood.text };
    let boilerplate = text::split_ref(line);
    for (at, word) in boilerplate.iter().take(*OWN_THING_SCAN).enumerate() {
        if !POSSESSIVE.contains(&text::strip_chars(word, ".,!?;:").to_lowercase()) {
            continue;
        }
        if at + 1 >= boilerplate.len() {
            break;
        }
        let span: Vec<String> = boilerplate[at + 1..(at + 4).min(boilerplate.len())]
            .iter()
            .map(|w| text::strip_chars(w, ".,!?;:").to_lowercase())
            .collect();
        if span.is_empty() || REFERRING.contains(&span[0]) {
            continue;
        }
        let named = _thing_named_in(understood, &span);
        if !named.is_empty() {
            return named;
        }
        let second = text::strip_chars(boilerplate[at + 1], ".,!?;:");
        if PERSON_WORDS.contains(&second.to_lowercase()) && _names_a_thing(understood) {
            continue;
        }
        if !second.is_empty() {
            return second.to_lowercase();
        }
    }
    String::new()
}

/// Python: bot/modules/answers.py:2637 `_thing_named_in(understood, span)`
/// The thing the understanding engine named among these words, if it named one.
pub fn _thing_named_in(understood: &Understood, span: &[String]) -> String {
    for ent in &understood.entities {
        let said: Vec<String> = text::split_ref(&ent.text).iter().map(|w| w.to_lowercase()).collect();
        if said.is_empty() || said.len() > span.len() {
            continue;
        }
        for i in 0..=(span.len() - said.len()) {
            if span[i..i + said.len()] != said[..] {
                continue;
            }
            if ent.kind == "person" && _names_a_thing(understood) {
                return String::new();
            }
            return said.last().unwrap().clone();
        }
    }
    String::new()
}

/// Python: bot/modules/answers.py:2654 `_new_thing(own, answer)`
/// True when the line names a thing the answer never mentions: a knee is not leftovers.
pub fn _new_thing(own: &str, answer: &str) -> bool {
    if own.is_empty() {
        return false;
    }
    let words: HashSet<String> = text::split_ref(answer).iter().map(|w| text::strip_chars(w, ".,!?;:").to_lowercase()).collect();
    let own = own.to_lowercase();
    !text::split_ref(&own).iter().any(|w| words.contains(text::rstrip_chars(w, "s")) || words.contains(*w))
}

/// Python: bot/modules/answers.py:2666 `_the_run_said(memory, extra='')`
/// What this run of answers has said, read as one answer.
pub fn _the_run_said(memory: &Memory, extra: &str) -> String {
    let mut said: Vec<String> = vec![extra.to_string()];
    let mut answered = 0;
    for turn in memory.turns.iter().rev() {
        if turn.module.as_deref() != Some("answers") {
            if answered > 0 {
                break;
            }
            continue;
        }
        said.push(turn.bot.clone());
        answered += 1;
        if answered >= 4 {
            break;
        }
    }
    said.into_iter().filter(|p| !p.is_empty()).collect::<Vec<_>>().join(" ")
}

/// Python: bot/modules/answers.py:2693 `_head_thing(subject)`
/// The thing a subject names, when it names one.
pub fn _head_thing(subject: &str) -> String {
    if subject.is_empty() {
        return String::new();
    }
    _own_thing(&understanding::understand(subject, None))
}

/// Python: bot/modules/answers.py:2708 `_the_same_kind_of_thing(item, memory, text='')`
/// True when the entry's domain answers the topic the chat is on.
pub fn _the_same_kind_of_thing(item: &Entry, memory: &Memory, text_: &str) -> bool {
    py_try(|| {
        let domain = &item.domain;
        if domain.is_empty() {
            return false;
        }
        let carried = if !text_.is_empty() { memory::topics::carried_topic(memory, text_) } else { None };
        let record = carried.filter(|r| r.truthy()).or_else(|| memory::topics::current(memory));
        let said = record_text(record);
        if said.is_empty() {
            return false;
        }
        let about = understanding::understand(&said, Some(&Memory::new())).topic.unwrap_or_default();
        !about.is_empty() && knowledge::by_topic().iter().any(|(d, topics)| d == domain && topics.contains(&about))
    })
    .unwrap_or(false)
}

/// Python: bot/modules/answers.py:2737 `_reaches_another_subject(text, item, memory)`
/// True when the entry the line reaches is about something the conversation is not.
pub fn _reaches_another_subject(text_: &str, item: &Entry, memory: &Memory) -> bool {
    py_try(|| {
        if !memory::topics::is_follow_up(text_, memory) {
            return false;
        }
        if understanding::understand(text_, Some(&Memory::new())).topic.is_some_and(|t| !t.is_empty()) {
            return false;
        }
        if memory.turns.is_empty() {
            return false;
        }
        let mut around: HashSet<String> = memory::topics::current(memory)
            .and_then(|t| t.get("words"))
            .map(|w| w.items().iter().map(|x| py_str(x).to_lowercase()).collect())
            .unwrap_or_default();
        around.extend(memory::topics::keywords(&memory::topics::subject(memory)));
        if around.is_empty() {
            return false;
        }
        let key_text = _key_words(item).into_iter().collect::<Vec<_>>().join(" ");
        let keys: HashSet<String> = memory::topics::keywords(&key_text).into_iter().filter(|w| !VAGUE.contains(w)).collect();
        let around: HashSet<String> = around.into_iter().filter(|w| !VAGUE.contains(w)).collect();
        if _the_same_kind_of_thing(item, memory, text_) {
            return false;
        }
        if _a_definition_of_the_entry(text_, item) {
            return false;
        }
        !keys.is_empty() && keys.is_disjoint(&around)
    })
    .unwrap_or(false)
}

/// What `_off_the_topic` hands back: the topic's key words, the domain of the entry that answers
/// it, whether the line says what the thing is.
pub type OffTopic = (HashSet<String>, String, bool);

/// Python: bot/modules/answers.py:2851 `_off_the_topic(understood, memory)`
pub fn _off_the_topic(understood: Option<&Understood>, memory: &Memory) -> OffTopic {
    let none = || (HashSet::new(), String::new(), false);
    let Some(understood) = understood else { return none() };
    let line = if !understood.clean.is_empty() { understood.clean.clone() } else { understood.text.clone() };
    let detail = !line.contains('?') && DETAIL_OF.is_match(&line);
    let asking = ASK_OF.is_match(&line) && OBJECT_PRONOUN.is_match(&line);
    if !(detail || asking) {
        return none();
    }
    let allowed = |w: &str| {
        REPORT_WORDS.contains(w) || DO_VERBS.contains(w) || (asking && ASK_WORDS.contains(w)) || (detail && SAID_OF_THE_THING.contains(w))
    };
    let mut says_what_it_is = false;
    for word in memory::topics::words(&line) {
        if crate::understanding::pyuni::is_digit(&word)
            || allowed(&word)
            || memory::topics::forms(&word).iter().any(|f| *f != word && allowed(f))
        {
            if detail && SAID_OF_THE_THING.contains(&word) {
                says_what_it_is = true;
            }
            continue;
        }
        return none();
    }
    let Some(leans_on) = understood.leans_on.as_ref() else { return none() };
    let said = leans_on.text.clone();
    if said.is_empty() {
        return none();
    }
    let mut domain = memory.get("last_answer").filter(|v| v.is_dict()).and_then(|l| l.get("domain")).map(py_str_or_empty).unwrap_or_default();
    if domain.is_empty() {
        let home = find_plain(&understanding::understand(&said, Some(memory)));
        domain = home.map(|h| h.domain.clone()).unwrap_or_default();
    }
    let words: HashSet<String> = memory::topics::keywords(&said)
        .into_iter()
        .filter(|w| !REPORT_WORDS.contains(w) && !DO_VERBS.contains(w) && !ASK_WORDS.contains(w))
        .collect();
    (words, domain, says_what_it_is)
}

/// `str(value or "")`
pub fn py_str_or_empty(v: &Value) -> String {
    if v.truthy() { py_str(v) } else { String::new() }
}

/// Python: bot/modules/answers.py:2903 `_from_another_domain(item, off_topic)`
/// True when the entry is about neither the topic's words nor the topic's own kind of thing.
pub fn _from_another_domain(item: &Entry, off_topic: &OffTopic) -> bool {
    let (words, domain) = (&off_topic.0, &off_topic.1);
    if !domain.is_empty() && item.domain == *domain {
        return false;
    }
    !_names_a_topic_word(item, words)
}

/// Python: bot/modules/answers.py:2911 `_names_a_topic_word(item, wanted)`
/// True when the entry is keyed on one of the words, in any of their forms.
pub fn _names_a_topic_word(item: &Entry, wanted: &HashSet<String>) -> bool {
    let keys: HashSet<String> = _key_words(item).into_iter().map(|k| k.to_lowercase()).collect();
    wanted.iter().any(|w| keys.contains(w) || keys.contains(&memory::topics::stem(w)))
}

/// Python: bot/modules/answers.py:2951 `_topic_words(line, memory)`
/// The words of the topic this line carries on from, without the words the line already says.
pub fn _topic_words(line: &str, memory: &Memory) -> Vec<String> {
    let said: HashSet<String> = memory::topics::content(line).into_iter().collect();
    let record = record_text(memory::topics::carried_topic(memory, line));
    memory::topics::content(&record)
        .into_iter()
        .filter(|w| !said.contains(w) && !BARE_GENERIC.contains(w) && !TIME_WORDS.contains(w))
        .collect()
}

/// Python: bot/modules/answers.py:2960 `_lent_words(reading, memory)`
/// The words the topic lent this line when it was read with it.
pub fn _lent_words(reading: &Understood, memory: &Memory) -> Vec<String> {
    let line = if !reading.clean.is_empty() { reading.clean.clone() } else { reading.text.clone() };
    let said: HashSet<String> = memory::topics::content(&line).into_iter().collect();
    let wider = memory::topics::followup_reading(reading, &memory_value(memory));
    let added = if !wider.clean.is_empty() { wider.clean.clone() } else { wider.text.clone() };
    let words: Vec<String> = memory::topics::content(&added).into_iter().filter(|w| !said.contains(w)).collect();
    if words.is_empty() {
        return _topic_words(&line, memory);
    }
    words.into_iter().filter(|w| !BARE_GENERIC.contains(w) && !TIME_WORDS.contains(w)).collect()
}

/// Python: bot/modules/answers.py:2978 `_same_word(word, said)`
/// True when two words are the same word, endings and all.
pub fn _same_word(word: &str, said: &str) -> bool {
    if word == said {
        return true;
    }
    if text::len(word) < 4 || text::len(said) < 4 {
        return false;
    }
    word.starts_with(said) || said.starts_with(word)
}

/// Python: bot/modules/answers.py:2988 `_writes_its_own_reply(item)`
/// True when the entry writes its answer from the reading instead of storing one.
pub fn _writes_its_own_reply(item: &Entry) -> bool {
    matches!(item.reply, crate::knowledge::entry::Reply::Fn(_))
}

/// Python: bot/modules/answers.py:3009 `_leans_on(reading, item, memory=None)`
/// True when an entry reached only with the last topic belongs to the line's own words too.
pub fn _leans_on(reading: &Understood, item: &Entry, memory: &Memory) -> bool {
    let line = if !reading.clean.is_empty() { reading.clean.clone() } else { reading.text.clone() };
    let line_words = memory::topics::content(&line);
    let own: Vec<String> = line_words
        .iter()
        .filter(|w| {
            !ASKING.contains(*w) && !BARE_GENERIC.contains(*w) && !ASKING_VERBS.contains(*w) && !MONTH_NAMES.contains(*w) && !SEASONS.contains(*w)
        })
        .cloned()
        .collect();
    let vague = own.is_empty() && line_words.iter().any(|w| BARE_GENERIC.contains(w));
    let keyed = _key_words(item);
    let mut says = keyed.clone();
    if let crate::knowledge::entry::Reply::Text(reply) = &item.reply {
        for w in text::split_ref(&reply.to_lowercase()) {
            says.insert(text::rstrip_chars(text::strip_chars(w, PUNCTUATION), "s").to_string());
        }
    }
    let topic = _topic_words(&line, memory);
    let carried = record_text(memory::topics::carried_topic(memory, &line));
    let topic_text = text::squash(&carried.to_lowercase());
    let subject_of_the_topic: Vec<String> = if !carried.is_empty() {
        memory::topics::key_subject(&carried, Some(memory)).iter().map(|w| w.to_lowercase()).collect()
    } else {
        vec![]
    };
    let about: Vec<String> = if !subject_of_the_topic.is_empty() { subject_of_the_topic.clone() } else { topic };
    let about_text = if !subject_of_the_topic.is_empty() { subject_of_the_topic.join(" ") } else { topic_text };
    let lent = _lent_words(reading, memory);
    let hits: HashSet<String> = lent.into_iter().filter(|w| says.iter().any(|other| _same_word(w, other))).collect();
    let named: Vec<String> = {
        let n: Vec<String> = own.iter().filter(|w| !PROPERTY_WORDS.contains(*w)).cloned().collect();
        if n.is_empty() { own.clone() } else { n }
    };
    let _names_it = named.iter().any(|w| keyed.iter().any(|said| _same_word(w, said)));
    let it_says_it_is_not = |phrase: &str| {
        let phrase = text::strip(phrase).to_lowercase();
        if phrase.is_empty() || FEELINGS.contains(&phrase) {
            return false;
        }
        about.contains(&phrase) || (phrase.contains(' ') && about_text.contains(&phrase))
    };
    if item.avoid.iter().any(|w| it_says_it_is_not(w)) {
        return false;
    }
    if own.is_empty() {
        return hits.len() >= if vague { 2 } else { 1 };
    }
    let ask_side: Vec<&String> = own.iter().filter(|w| !PROPERTY_WORDS.contains(*w)).collect();
    if ask_side.is_empty() {
        return !hits.is_empty();
    }
    if !own.iter().any(|w| says.iter().any(|said| _same_word(w, said))) && !_a_money_question(&own, &keyed) {
        return false;
    }
    if hits.is_empty() && !_writes_its_own_reply(item) && memory::topics::next_slot_reading(reading, &memory_value(memory)).is_py_none() {
        return false;
    }
    let line_low = line.to_lowercase();
    let line_set: HashSet<&str> = text::split_ref(&line_low).into_iter().collect();
    if line_set.iter().any(|w| SPEECH.contains(*w)) && !says.iter().any(|w| SPEECH.contains(w)) {
        return false;
    }
    true
}

/// Python: bot/modules/answers.py:3118 `_is_about(item, subject)`
/// True when the entry is about the thing, not about a neighbour: a stew is not a steak.
pub fn _is_about(item: &Entry, subject: &str) -> bool {
    let all = text::split_ref(subject);
    let words: Vec<&str> = {
        let w: Vec<&str> = all.iter().copied().filter(|w| !DESCRIBING.contains(*w)).collect();
        if w.is_empty() { all.clone() } else { w }
    };
    let head = words.last().copied().unwrap_or("");
    let bare = text::rstrip_chars(head, "s");
    let head = if bare.is_empty() { head } else { bare };
    let keys = _key_words(item);
    if head.is_empty() || !keys.contains(head) {
        return false;
    }
    if KINDS_VERBISH.contains(head) || (text::len(head) > 4 && (head.ends_with("ed") || head.ends_with("ing"))) {
        let named: Vec<&str> = words
            .iter()
            .copied()
            .filter(|w| !KINDS_VERBISH.contains(*w) && !(text::len(w) > 4 && (w.ends_with("ed") || w.ends_with("ing"))))
            .collect();
        return named.iter().any(|w| keys.contains(text::rstrip_chars(w, "s")) || keys.contains(*w));
    }
    true
}

/// Python: bot/modules/answers.py:3174 `_only_a_property(words)`
/// True when the line names no subject, only the property it asks about.
pub fn _only_a_property(words: &[String]) -> bool {
    !words.is_empty() && words.iter().all(|w| PROPERTY_WORDS.contains(w))
}

/// Python: bot/modules/answers.py:3197 `_a_property_word_names_a_thing(text)`
/// True when a property word in the line is the thing the line is about, not the ask.
pub fn _a_property_word_names_a_thing(text_: &str) -> bool {
    PROPERTY_WORD_THAT_NAMES_A_THING.is_match(text_)
}

/// Python: bot/modules/answers.py:3215 `_a_money_question(own, keyed)`
/// True when the line asks what a thing costs and the entry is keyed on what a thing costs.
pub fn _a_money_question(own: &[String], keyed: &HashSet<String>) -> bool {
    own.iter().any(|w| MONEY_ASK.contains(w)) && keyed.iter().any(|w| MONEY_ASK.contains(w))
}

/// Python: bot/modules/answers.py:3224 `_remember_answer(memory, item, reply, keys=())`
/// Keep what a follow-up can ask about again: the answer, what it is about, its turn.
pub fn _remember_answer(memory: &mut Memory, item: Option<&'static Entry>, reply: &str, keys: &[String]) {
    if text::strip(reply).is_empty() {
        return;
    }
    let mut all: Vec<String> = item.map(_key_words).unwrap_or_default().into_iter().chain(keys.iter().cloned()).collect();
    all.sort();
    all.dedup();
    let mut record = Value::dict();
    record.set_key("reply", reply);
    record.set_key("keys", all);
    record.set_key("id", Value::from(item.map(|i| i.id.clone())));
    record.set_key("domain", item.map(|i| i.domain.clone()).unwrap_or_default());
    record.set_key("turn", memory.turns.len() as i64);
    memory.set("last_answer", record);
}

/// Python: bot/modules/answers.py:3242 `_last_answer(memory)`
/// The answer just given, while the conversation is still about the same thing.
pub fn _last_answer(memory: &Memory) -> Option<Value> {
    let record = memory.get("last_answer").filter(|r| r.is_dict())?;
    if !record.get("reply").is_some_and(Value::truthy) {
        return None;
    }
    let turns = &memory.turns;
    let turn = match record.get("turn") {
        Some(Value::Int(t)) => *t,
        _ => return None,
    };
    if turn < 0 || turn as usize >= turns.len() {
        return None;
    }
    if turns.len() as i64 - 1 - turn > 4 {
        return None;
    }
    if turns[turn as usize].module.as_deref() != Some("answers") {
        return None;
    }
    let topic = memory::topics::current(memory).filter(|t| t.is_dict())?;
    let topic_turn = match topic.get("turn") {
        Some(Value::Int(t)) => *t,
        _ => return None,
    };
    if topic_turn > turn {
        let here: HashSet<String> = memory::topics::keywords(&record_text(Some(topic))).into_iter().collect();
        let keys: HashSet<String> = record.get("keys").map(|k| k.strs().into_iter().collect()).unwrap_or_default();
        if here.is_disjoint(&keys) {
            return None;
        }
    }
    Some(record.clone())
}

re_static!(YES_NO_ABOUT_A_THING, OPTION_TIMING, DIGIT_OR_SPAN, DEPENDENT_SENTENCE, WHO_TO_CALL, CONDITIONS_THE_LINE,
    OPENING_CONNECTIVE, SAYS_TO_GET_SOMEBODY, SOMEONE_TO_GET, STOCK_ACKNOWLEDGEMENT, CLAUSE_BREAK, CHOICE_ASK,
    CHOICE_CONTEXT, A_SPEC, CHOICE_CUT, CHOICE_FIGURE, CHOICE_LIST_HEAD, CHOICE_ITEM, CHOICE_LEAD_VERB,
    CHOICE_CONDITION, CHOICE_COST, CHOICE_PRICE_OPENS, CHOICE_PRICE_ONLY, _AN_ALTERNATIVE, _DISH_TIME, _A_WHOLE_TIME,
    _A_BARE_TIME, _A_LABELLED_TIME, _RATE, _TIME_TAIL, CORRECTION_FRAME, A_BARE_FIGURE, A_UNIT, SIZE_REPLACED, RATE);
set_static!(ASKING_FRESH, ABOUT_THE_ANSWER, A_TRADE, ACK_MODULES, WHO_ASK_WORDS, CHOICE_NOT_A_THING, _NOT_AN_OPTION,
    _HOW_LONG_ONLY);
strs_static!(CHOICE_TAIL);
static MOST_ACKS: LazyLock<usize> = LazyLock::new(|| g("MOST_ACKS").as_int().unwrap_or(2) as usize);
static MOST_WORDS_OF_AN_ACK: LazyLock<usize> = LazyLock::new(|| g("MOST_WORDS_OF_AN_ACK").as_int().unwrap_or(24) as usize);
static LONG_SENTENCE_WORDS: LazyLock<usize> = LazyLock::new(|| g("LONG_SENTENCE_WORDS").as_int().unwrap_or(24) as usize);
static CHOICE_SHORTEST: LazyLock<usize> = LazyLock::new(|| g("CHOICE_SHORTEST").as_int().unwrap_or(60) as usize);
static UNIT_WORDS: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| g("UNIT_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect());

/// `re.findall(r"[a-z']+", s)`
fn az_words(s: &str) -> Vec<String> {
    inline_re!(r"[a-z']+").findall_str(s)
}

/// `rx.split(text, 1)` for a pattern with no group.
fn split_once_re(rx: &Re, s: &str) -> Vec<String> {
    match rx.search(s) {
        Some(m) => vec![s[..m.start()].to_string(), s[m.end()..].to_string()],
        None => vec![s.to_string()],
    }
}

/// The `Memory` a `memory: Value` contract was handed (the plain dict back as a memory).
pub fn memory_from_value(memory: &Value) -> Memory {
    match memory.to_plain() {
        serde_json::Value::Object(m) => Memory::from_plain(&m),
        _ => Memory::new(),
    }
}

/// Python: bot/modules/answers.py:3274 `_asks_part(text)`
/// True when the line asks for one part of the answer and names no subject of its own.
pub fn _asks_part(text_: &str) -> bool {
    let kind = memory::narrow::kind_of(text_);
    if kind.as_deref() == Some("cause") && memory::topics::keywords(text_).is_empty() {
        return true;
    }
    if kind.is_some() {
        return _only_a_property(&memory::topics::keywords(text_));
    }
    memory::narrow::asks_a_part(text_)
}

/// Python: bot/modules/answers.py:3323 `_the_sentence_about(text, memory)`
/// The sentence of the last answer that answers a yes/no about a thing it named, or None.
pub fn _the_sentence_about(text_: &str, memory: &Memory) -> Option<String> {
    let earlier = _last_answer(memory)?;
    let reply = py_str_or_empty(earlier.get("reply").unwrap_or(&Value::None));
    if text::strip(&reply).is_empty() || text::split_ref(text_).len() > 7 {
        return None;
    }
    if DIGIT_OR_SPAN.is_match(text_) {
        return None;
    }
    let said_so_far = memory::narrow::words_of(text_);
    let flat_text = memory::narrow::flat(text_);
    let asked_fresh = az_words(&flat_text).iter().any(|w| ASKING_FRESH.contains(w));
    if !(YES_NO_ABOUT_A_THING.is_match(&flat_text) || (said_so_far.len() <= 3 && !asked_fresh)) {
        return None;
    }
    let mut said: HashSet<String> = memory::narrow::words_of(&reply).into_iter().collect();
    if let Some(keys) = earlier.get("keys") {
        said.extend(keys.strs());
    }
    let mut named: Vec<String> = Vec::new();
    let mut other = false;
    for word in memory::narrow::words_of(text_) {
        if ABOUT_THE_ANSWER.contains(&word) || text::len(&word) < 3 {
            continue;
        }
        if said.contains(&word) {
            named.push(word);
        } else {
            other = true;
        }
    }
    if named.is_empty() || other {
        return None;
    }
    let sentences = memory::quotes::sentences(&reply, g("SENTENCE"));
    let holding: Vec<&String> = sentences
        .iter()
        .filter(|one| {
            let own: HashSet<String> = memory::narrow::words_of(one).into_iter().collect();
            named.iter().all(|w| own.contains(w))
        })
        .collect();
    if holding.len() != 1 || sentences.len() < 2 {
        return None;
    }
    let part = holding[0].clone();
    if memory::narrow::flat(&part) == memory::narrow::flat(&reply) {
        return None;
    }
    if OPTION_TIMING.findall(&part).len() >= 2 {
        return None;
    }
    if DEPENDENT_SENTENCE.match_(&part).is_some() || text::len(&part) < 40 {
        return None;
    }
    _part_not_repeated(Some(&part), &reply, memory)
}

/// Python: bot/modules/answers.py:3426 `_a_stock_acknowledgement(turns, back)`
/// Whether the turn `back` from the end only handed part of the chat back, and nothing more.
pub fn _a_stock_acknowledgement(turns: &[Turn], back: usize) -> bool {
    let turn = &turns[turns.len() - 1 - back];
    let reply = text::strip(&turn.bot);
    if reply.is_empty() {
        return false;
    }
    if turn.module.as_deref().is_some_and(|m| ACK_MODULES.contains(m))
        && text::split_ref(reply).len() <= *MOST_WORDS_OF_AN_ACK
        && STOCK_ACKNOWLEDGEMENT.match_(reply).is_some()
    {
        return true;
    }
    if turns.len() > 1 + back {
        let before = &turns[turns.len() - 2 - back];
        let earlier = memory::narrow::flat(&before.bot);
        let said = memory::narrow::flat(reply);
        if !earlier.is_empty() && !said.is_empty() && text::split_ref(&said).len() >= 4 && earlier.contains(&said) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:3452 `_the_person_to_call(text, memory)`
/// The sentence of the last answer that says who to get, for a "who should I call", or None.
pub fn _the_person_to_call(text_: &str, memory: &Memory) -> Option<String> {
    if !WHO_TO_CALL.is_match(&memory::narrow::flat(text_)) {
        return None;
    }
    if memory::narrow::words_of(text_).iter().any(|w| !WHO_ASK_WORDS.contains(w) && text::len(w) > 2) {
        return None;
    }
    let turns = &memory.turns;
    let mut last = turns.last()?;
    if last.module.as_deref() != Some("answers") {
        let mut skipped = 0usize;
        while skipped < *MOST_ACKS && turns.len() as isize - 1 - skipped as isize >= 0 && _a_stock_acknowledgement(turns, skipped) {
            skipped += 1;
        }
        if skipped == 0 || (turns.len() as isize - 1 - skipped as isize) < 0 {
            return None;
        }
        last = &turns[turns.len() - 1 - skipped];
        if last.module.as_deref() != Some("answers") {
            return None;
        }
    }
    let reply = text::strip(&last.bot).to_string();
    if reply.is_empty() {
        return None;
    }
    let sentences: Vec<String> = SENTENCE.split(&reply).iter().map(|s| text::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    if sentences.len() < 2 {
        return None;
    }
    let names_a_trade = |one: &str| az_words(&one.to_lowercase()).iter().any(|w| A_TRADE.contains(w));
    let holding: Vec<&String> = sentences.iter().filter(|one| names_a_trade(one)).collect();
    if holding.is_empty() {
        return None;
    }
    let getting: Vec<&String> = holding.iter().copied().filter(|one| SOMEONE_TO_GET.is_match(one)).collect();
    let pool = if getting.is_empty() { holding } else { getting };
    let stating: Vec<&String> = pool.iter().copied().filter(|one| CONDITIONS_THE_LINE.match_(one).is_none()).collect();
    let pool = if stating.is_empty() { pool } else { stating };
    let up_front: Vec<&String> = pool.iter().copied().filter(|one| SAYS_TO_GET_SOMEBODY.match_(one).is_some()).collect();
    let mut ordered = up_front.clone();
    ordered.extend(pool.iter().copied().filter(|one| !up_front.contains(one)));
    let named: Vec<&String> = ordered.into_iter().filter(|one| names_a_trade(one)).collect();
    let mut part = named.first()?.to_string();
    let stripped = text::strip(&OPENING_CONNECTIVE.sub("", &part)).to_string();
    if stripped != part && !stripped.is_empty() {
        part = up1(&stripped);
    }
    if text::len(&part) < 25 || !first_is_upper(&part) || DEPENDENT_SENTENCE.match_(&part).is_some() {
        return None;
    }
    let part = _the_clause_that_says_who(&part);
    _part_not_repeated(Some(&part), &reply, memory)
}

/// Python: bot/modules/answers.py:3530 `_the_clause_that_says_who(part)`
/// The clause of a long sentence that names who to get, or the sentence as it stands.
pub fn _the_clause_that_says_who(part: &str) -> String {
    if text::split_ref(part).len() <= *LONG_SENTENCE_WORDS {
        return part.to_string();
    }
    let pieces: Vec<String> = CLAUSE_BREAK.split(part).iter().map(|s| text::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    for piece in pieces.iter().rev() {
        let named = az_words(&piece.to_lowercase()).iter().any(|w| A_TRADE.contains(w));
        if named && SOMEONE_TO_GET.is_match(piece) {
            let bare = text::strip(&OPENING_CONNECTIVE.sub("", piece)).to_string();
            let piece = if bare.is_empty() { piece.clone() } else { bare };
            return up1(&piece);
        }
    }
    part.to_string()
}

/// Python: bot/modules/answers.py:3600 `_the_clause_that_picks(sentence)`
/// The first clause of a long sentence, when the rest of it is a second thought.
pub fn _the_clause_that_picks(sentence: &str) -> String {
    if text::len(sentence) <= 150 {
        return sentence.to_string();
    }
    for found in CHOICE_CUT.finditer(sentence) {
        let start = text::char_index(sentence, found.start());
        if start >= *CHOICE_SHORTEST {
            return text::strip_chars(&sentence[..found.start()], " ,;").to_string();
        }
    }
    sentence.to_string()
}

/// Python: bot/modules/answers.py:3644 `_a_list_of_things(tail)`
/// True when a clause lists three or more things, each a phrase of its own.
pub fn _a_list_of_things(tail: &str) -> bool {
    let items: Vec<String> = inline_re!(r",\s*|\s+and\s+").split(tail).iter().map(|one| text::strip_chars(one, " .!?").to_string()).collect();
    let items: Vec<String> =
        items.into_iter().filter(|one| !one.is_empty()).map(|one| inline_re!(r"^(?:or|and)\s+").sub("", &one)).collect();
    items.len() >= 3 && items.iter().all(|one| (2..=11).contains(&text::split_ref(one).len()))
}

/// Python: bot/modules/answers.py:3659 `_the_thing_that_picks(sentence)`
/// The shortest span of a deciding sentence that names the thing to take, or None.
pub fn _the_thing_that_picks(sentence: &str) -> Option<String> {
    let mut span = text::strip(sentence).to_string();
    let (mut listed, mut sized) = (false, false);
    if span.contains(": ") {
        let (head, tail) = span.split_once(": ").unwrap();
        let (head, tail) = (text::strip(head).to_string(), text::strip(tail).to_string());
        if text::split_ref(&tail).len() >= 3 {
            if CHOICE_LIST_HEAD.is_match(&head) && !CHOICE_FIGURE.is_match(&head) && _a_list_of_things(&tail) {
                span = tail;
                listed = true;
            } else if CHOICE_FIGURE.findall(&head).len() > CHOICE_FIGURE.findall(&tail).len() {
                span = head;
                sized = true;
            } else if CHOICE_FIGURE.findall(&tail).len() > CHOICE_FIGURE.findall(&head).len() {
                span = tail;
                sized = true;
            } else {
                span = head;
            }
        }
    }
    let first = split_once_re(&CHOICE_ITEM, &span);
    let takes_the_figure = first.len() == 2 && CHOICE_FIGURE.is_match(&span) && !CHOICE_FIGURE.is_match(&first[0]);
    if first.len() == 2 {
        if text::split_ref(&first[0]).len() < 2 {
            return None;
        }
        if !takes_the_figure {
            span = first[0].clone();
        }
    }
    span = CHOICE_LEAD_VERB.sub("", &span);
    if let Some(conditional) = CHOICE_CONDITION.search(&span) {
        if text::char_index(&span, conditional.start()) >= 8 {
            span = span[..conditional.start()].to_string();
        }
    }
    let span = text::strip_chars(&_the_clause_that_picks(&span), " ,;:.!?").to_string();
    if text::split_ref(&span).len() < 2 {
        return None;
    }
    if _a_price_and_nothing_else(&span) {
        return None;
    }
    if listed {
        return Some(span);
    }
    if sized || CHOICE_FIGURE.is_match(&span) {
        return Some(span);
    }
    None
}

/// Python: bot/modules/answers.py:3729 `_a_price_and_nothing_else(span)`
/// True when the span is a price rather than a thing, so it answers "how much", not "which".
pub fn _a_price_and_nothing_else(span: &str) -> bool {
    let rest = CHOICE_PRICE_ONLY.sub(" ", span);
    if !inline_re!(r"[a-z]{2,}").findall_str(&rest.to_lowercase()).iter().any(|w| !CHOICE_NOT_A_THING.contains(w)) {
        return true;
    }
    let bare = text::rstrip_chars(span, " .!?");
    if CHOICE_PRICE_OPENS.match_(bare).is_some() {
        return true;
    }
    match CHOICE_COST.search(bare) {
        Some(ends) => text::strip_chars(&bare[ends.end()..], " .,!?").is_empty(),
        None => false,
    }
}

/// Python: bot/modules/answers.py:3750 `_the_composer_has_options(memory)`
/// True when the bot's last answer offered named options the composer can pick between.
pub fn _the_composer_has_options(memory: &Memory) -> bool {
    py_try(|| !compose::fallback::_offered(memory).is_empty()).unwrap_or(false)
}

/// Python: bot/modules/answers.py:3770 `_the_option_that_decides(said, reply)`
/// The option of theirs the answer opened with, or '' when it opened with something else.
pub fn _the_option_that_decides(said: &str, reply: &str) -> String {
    let stripped = text::strip(reply);
    let head = split_once_re(inline_re!(r"[.!?—:;]"), stripped).remove(0);
    let in_the_question: HashSet<String> = inline_re!(r"[a-z][a-z'’-]*").findall_str(&said.to_lowercase()).into_iter().collect();
    for word in inline_re!(r"[A-Za-z][A-Za-z'’-]*").findall_str(&head).into_iter().take(5) {
        let low = word.to_lowercase();
        if in_the_question.contains(&low) && !_NOT_AN_OPTION.contains(&low) {
            return word;
        }
    }
    String::new()
}

/// Python: bot/modules/answers.py:3781 `_the_answer_to_a_choice_behind(memory)`
/// The answer that decided between two things the person put up, or None.
pub fn _the_answer_to_a_choice_behind(memory: &Memory) -> Option<String> {
    for row in memory.turns.iter().rev() {
        let said = text::squash(&row.user.to_lowercase());
        let reply = text::strip(&row.bot);
        if said.is_empty() || reply.is_empty() || text::split_ref(&said).len() > 16 {
            continue;
        }
        if !_AN_ALTERNATIVE.is_match(&said) {
            continue;
        }
        let option = _the_option_that_decides(&said, reply);
        if option.is_empty() {
            continue;
        }
        return Some(format!("{}. That is the one I would go for.", up1(&option)));
    }
    None
}

/// Python: bot/modules/answers.py:3807 `_the_one_to_pick(text, memory)` - the contract form,
/// which reads the memory as the plain dict it is handed; the reply is [`_the_one_to_pick_for`]'s.
pub fn _the_one_to_pick(text_: &str, memory: &Value) {
    let _ = _the_one_to_pick_for(text_, &memory_from_value(memory));
}

/// Python: bot/modules/answers.py:3807 `_the_one_to_pick(text, memory)`
/// The sentence of the last answer that decides which one of the things to take.
pub fn _the_one_to_pick_for(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text_.to_lowercase());
    if CHOICE_ASK.match_(&clean).is_none() {
        return None;
    }
    let padded = format!(" {clean} ");
    if padded.contains(" or ") || padded.contains(" vs ") || clean.contains("better") {
        return None;
    }
    let Some(earlier) = _last_answer(memory) else { return _the_answer_to_a_choice_behind(memory) };
    let turns = &memory.turns;
    let before = match earlier.get("turn") {
        Some(Value::Int(t)) if *t >= 0 && (*t as usize) < turns.len() => turns[*t as usize].user.to_lowercase(),
        _ => String::new(),
    };
    if !CHOICE_CONTEXT.is_match(&before) {
        return None;
    }
    if _the_composer_has_options(memory) {
        return None;
    }
    let reply = text::strip(&py_str_or_empty(earlier.get("reply").unwrap_or(&Value::None))).to_string();
    if reply.is_empty() {
        return None;
    }
    let sentences: Vec<String> = SENTENCE.split(&reply).iter().map(|s| text::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    if sentences.is_empty() {
        return None;
    }
    let choose_from: Vec<&String> = sentences.iter().filter(|one| !text::rstrip(one).ends_with('?')).collect();
    let mut best = (*choose_from.first()?).clone();
    if !A_SPEC.is_match(&best) {
        if let Some(s) = choose_from[1..].iter().find(|s| A_SPEC.is_match(s)) {
            best = (*s).clone();
        }
    }
    let said = _flat(&best);
    let last = memory::last_bot(memory);
    if said.is_empty() || said == _flat(&last) {
        return None;
    }
    let pick = _the_thing_that_picks(&best)?;
    if text::split_ref(&pick).len() < 2 || _flat(&pick) == _flat(&last) {
        return None;
    }
    let tail = &CHOICE_TAIL[text::len(&said) % CHOICE_TAIL.len()];
    let trimmed = text::rstrip_chars(&pick, ". ");
    let pick = if trimmed.is_empty() { text::rstrip(&pick).to_string() } else { trimmed.to_string() };
    Some(format!("{}. {tail}", up1(&pick)))
}

/// Python: bot/modules/answers.py:3900 `_one_total(rest, whole_sentence=False)`
/// Whether what follows a figure in its sentence leaves that figure as the one total.
pub fn _one_total(rest: &str, whole_sentence: bool) -> bool {
    let rest = text::squash(text::strip_chars(rest, " .,;:!?-"));
    if crate::understanding::pyuni::is_digit(&text::slice(&rest, 0, Some(1))) || _RATE.is_match(&rest) {
        return false;
    }
    let first_clause = rest.split(';').next().unwrap_or("");
    if inline_re!(r"\b\d+(?:\.\d+)?(?![\w°])").is_match(first_clause) {
        return false;
    }
    if whole_sentence {
        return _TIME_TAIL.match_(&rest).is_some();
    }
    true
}

/// "about 35 minutes" -> "35 minutes": the whole-dish time as the reply says it.
fn bare_time(said: &str) -> String {
    text::squash(&said.to_lowercase()).replace("about ", "").replace("roughly ", "").replace("around ", "")
}

/// Python: bot/modules/answers.py:3924 `_the_dish_time(understood, memory)`
/// The whole-dish time in the answer just given, when the line asks for it.
pub fn _the_dish_time(understood: Option<&Understood>, memory: &Memory) -> Option<String> {
    let empty = Understood::default();
    let understood = understood.unwrap_or(&empty);
    let text_ = &understood.clean;
    if memory::narrow::kind_of(text_).as_deref() != Some("time") {
        return None;
    }
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if words.is_empty() || words.iter().any(|w| !_HOW_LONG_ONLY.contains(w)) {
        return None;
    }
    let earlier = _last_answer(memory)?;
    let reply = py_str_or_empty(earlier.get("reply").unwrap_or(&Value::None));
    let mut whole: Option<String> = None;
    let bracketed: HashSet<String> = _DISH_TIME.finditer(&reply).iter().map(|one| format!("{} {}", one.str(1), one.str(2).to_lowercase())).collect();
    if bracketed.len() == 1 {
        whole = bracketed.into_iter().next();
    }
    if whole.is_none() {
        let mut sentences: HashSet<String> = HashSet::new();
        for part in inline_re!(r"(?<=[.!?])\s+").split(text::strip(&reply)) {
            if let Some(bare) = _A_BARE_TIME.match_(&part) {
                if _one_total(&part[bare.end()..], true) {
                    sentences.insert(bare_time(bare.as_str()));
                }
            }
        }
        if sentences.len() == 1 {
            whole = sentences.into_iter().next();
        }
    }
    if whole.is_none() {
        if let Some(labelled) = _A_LABELLED_TIME.match_(text::strip(&reply)) {
            let said = labelled.str(1);
            if let Some(first) = _A_BARE_TIME.match_(said) {
                if _one_total(&said[first.end()..], false) {
                    whole = Some(bare_time(first.as_str()));
                }
            }
        }
    }
    let whole = whole?;
    let low_reply = reply.to_lowercase();
    for food in knowledge::cooking::ingredients(understood) {
        if !low_reply.contains(&food.to_lowercase()) {
            return None;
        }
    }
    Some(format!("About {whole} for that one, start to finish."))
}

/// Python: bot/modules/answers.py:4006 `_the_unit_last_said(memory, reply)`
/// The unit of the last amount the conversation said, or None.
pub fn _the_unit_last_said(memory: &Memory, reply: &str) -> Option<String> {
    for said_turn in memory.turns.iter().rev() {
        if let Some(said) = A_UNIT.search(&said_turn.user) {
            return Some(said.str(1).to_lowercase());
        }
    }
    for said in A_UNIT.finditer(reply) {
        let before = text::rstrip(&reply[..said.start()]).to_lowercase();
        if before.ends_with("per") {
            continue;
        }
        return Some(said.str(1).to_lowercase());
    }
    None
}

/// Python: bot/modules/answers.py:4026 `_the_figure_they_meant(text, memory, reply)`
/// The correction as the amount it corrects to ("and for 1.5 kilos"), or None.
pub fn _the_figure_they_meant(text_: &str, memory: &Memory, reply: &str) -> Option<String> {
    let body = CORRECTION_FRAME.sub("", text_);
    if text::strip(&body) == text::strip(text_) {
        return None;
    }
    let found = A_BARE_FIGURE.match_(&body)?;
    let figure = found.str(1).to_string();
    let mut unit = found.str(2).to_lowercase();
    if !["kg", "g", "lb", "l", "ml", "cm", "mm"].contains(&unit.as_str()) {
        unit = _the_unit_last_said(memory, reply)?;
    }
    let word = lookup(&UNIT_WORDS, &unit).map(str::to_string).unwrap_or(unit);
    Some(format!("and for {figure} {word}"))
}

/// Python: bot/modules/answers.py:4050 `_amount_asked(text, memory)`
/// The figure for a new amount the line names, or None.
pub fn _amount_asked(text_: &str, memory: &mut Memory) -> Option<String> {
    let earlier = _last_answer(memory)?;
    let reply = py_str_or_empty(earlier.get("reply").unwrap_or(&Value::None));
    let mut text_ = text_.to_string();
    if let Some(corrected) = _the_figure_they_meant(&text_, memory, &reply) {
        text_ = corrected;
    } else if memory::narrow::kind_of(&text_).as_deref() != Some("time") && !RATE.is_match(&reply) {
        return None;
    }
    let clean = |s: &str| -> HashSet<String> {
        AMOUNT.finditer(s).iter().map(|one| inline_re!(r"\s+").sub("", &one.as_str().to_lowercase())).collect()
    };
    let amounts: Vec<String> = AMOUNT.finditer(&text_).iter().map(|one| one.as_str().to_string()).collect();
    let wanted = clean(&text_);
    if wanted.is_empty() || wanted.is_subset(&clean(&reply)) {
        return None;
    }
    let subject = memory::topics::subject(memory);
    if subject.is_empty() {
        return None;
    }
    let replaced = SIZE_REPLACED.is_match(&text_) && memory::narrow::kind_of(&text_).as_deref() != Some("time") && RATE.is_match(&reply);
    let fuller = if replaced {
        format!("how long for {} {subject}?", amounts[0])
    } else {
        let mut fuller = _with_subject(&text_, memory, None);
        if text::strip(&fuller) == text::strip(&text_) {
            fuller = _about_subject(&text_, memory, None);
        }
        if text::strip(&fuller) == text::strip(&text_) {
            fuller = format!("{} for the {subject}?", text::rstrip_chars(&text_, "?.! "));
        }
        fuller
    };
    let reading = understanding::understand(&fuller, Some(&*memory));
    let item = find_plain(&reading)?;
    if !_is_about(item, &subject) {
        return None;
    }
    let own = if replaced { None } else { find_plain(&understanding::understand(&text_, Some(&*memory))) };
    if let Some(own) = own {
        if !std::ptr::eq(own, item) {
            return None;
        }
    }
    let fresh = _payload_mut(item, memory, &reading);
    let keys: Vec<String> = _key_words(item).into_iter().collect();
    if let Some(sized) = memory::topics::sized_reply(&fresh, &text_, memory, &keys).py() {
        return Some(sized);
    }
    let part = memory::narrow::narrow(&fresh, &fuller, "")
        .filter(|p| !p.is_empty())
        .or_else(|| memory::narrow::narrow(&fresh, &format!("how long for {}", amounts[0]), ""));
    if let Some(part) = part {
        if !text::strip(&part).is_empty() && memory::narrow::flat(&part) != memory::narrow::flat(&fresh) {
            return _part_not_repeated(Some(&part), &fresh, memory);
        }
    }
    None
}

/// Python: bot/modules/answers.py:4792 `_sentence_with(reply, part)`
/// The sentence of `reply` that `part` was cut from.
pub fn _sentence_with(reply: &str, part: &str) -> String {
    let wanted = _flat(part);
    for sentence in memory::quotes::sentences(reply, g("SENTENCE")) {
        if !wanted.is_empty() && _flat(&sentence).contains(&wanted) {
            return text::strip(&sentence).to_string();
        }
    }
    String::new()
}

/// Python: bot/modules/answers.py:4805 `_part_not_repeated(part, whole, memory)`
/// The part of an answer that answers this line, said another way if it was just said.
pub fn _part_not_repeated(part: Option<&str>, whole: &str, memory: &Memory) -> Option<String> {
    let part = part.filter(|p| !text::strip(p).is_empty())?;
    let last = _flat(&memory::last_bot(memory));
    if _flat(part) != last {
        return Some(text::strip(part).to_string());
    }
    let whole = if !text::strip(whole).is_empty() { whole } else { part };
    if _flat(whole) == last {
        return None;
    }
    let sentence = _sentence_with(whole, part);
    if !sentence.is_empty() && _flat(&sentence) != _flat(part) {
        return Some(sentence);
    }
    if _flat(whole) != _flat(part) {
        return Some(text::strip(whole).to_string());
    }
    None
}

re_static!(MACHINE_ASKING, WHEN_OF_THE_DAY, COOK_VERB, KEEPS_ASK, KEEPS_ANSWER, PROPORTION, MEASURE, CLAUSE_SEAM, JOINING_WORD,
    COUNTS_THE_CAUSES, CAUSE, ASKS_A_MEASURE, A_MEASURE, A_LEAD_TIME, ASKS_A_SPAN, WHEN_IS_A_CONDITION, A_WARNING,
    _HOURS_AND_MINUTES, FIGURE);
set_static!(NOT_NEW, WHERE_WORDS, GENERIC_ASK, NAMES_NO_CAUSE, GENERIC);
static _AMOUNT_BEFORE: LazyLock<String> = LazyLock::new(|| g("_AMOUNT_BEFORE").str().to_string());
static _PROPORTION_PHRASE: LazyLock<String> = LazyLock::new(|| g("_PROPORTION_PHRASE").str().to_string());
static KIND_SAID: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    g("KIND_SAID").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().expect("KIND_SAID holds regexes"))).collect()
});
static HOME_CALLING_SOMEONE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.home.CALLING_SOMEONE"));
static HOME_FOLLOWING_IT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.home.FOLLOWING_IT"));
static HOME_DECIDING_NOW: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.home.DECIDING_NOW"));

/// `_key_words` of every entry, by its position in `entries()`, read once (the scan in
/// `_part_asked` compares the keys of the whole base).
static ALL_KEY_WORDS: LazyLock<Vec<HashSet<String>>> =
    LazyLock::new(|| knowledge::entry::entries().iter().map(|e| _key_words_of(e)).collect());

fn _key_words_of(item: &Entry) -> HashSet<String> {
    let mut found = HashSet::new();
    for part in &item.key {
        for option in part.words() {
            for word in text::split_ref(&option.to_lowercase()) {
                let bare = text::rstrip_chars(word, "s");
                found.insert(if bare.is_empty() { word.to_string() } else { bare.to_string() });
            }
        }
    }
    found
}

/// The words of a record's "keys" list (`set(earlier.get("keys") or ())`).
fn record_keys(record: &Value) -> HashSet<String> {
    record.get("keys").map(|k| k.items().iter().map(py_str).collect()).unwrap_or_default()
}

/// `set(words_of(text))`
fn word_set(s: &str) -> HashSet<String> {
    memory::narrow::words_of(s).into_iter().collect()
}

fn minus(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    a.iter().filter(|w| !b.contains(*w)).cloned().collect()
}

/// `str(earlier.get("domain") or "")` and the like.
fn record_str(record: &Value, key: &str) -> String {
    record.get(key).map(py_str_or_empty).unwrap_or_default()
}

/// `getattr(own, "id", None) != earlier.get("id")`
fn other_id(item: &Entry, record: &Value) -> bool {
    record.get("id").and_then(|v| v.as_str()) != Some(item.id.as_str())
}

/// Python: bot/modules/answers.py:4133 `_the_amount_they_named_again(text, memory)`
/// The sentence of the last answer that gives the amount this line names, or None.
pub fn _the_amount_they_named_again(text_: &str, memory: &Memory) -> Option<String> {
    let earlier = _last_answer(memory);
    let reply = text::strip(&earlier.as_ref().and_then(|e| e.get("reply")).map(py_str_or_empty).unwrap_or_default()).to_string();
    if reply.is_empty() {
        return None;
    }
    let amounts: Vec<String> = AMOUNT.finditer(text_).iter().map(|m| m.as_str().to_string()).collect();
    if amounts.len() != 1 {
        return None;
    }
    if !SIZE_REPLACED.is_match(text_) {
        return None;
    }
    let escaped = crate::py::re::escape(&amounts[0]);
    if !Re::new(&format!(r"(?:^|[^\w]){escaped}(?:[^\w]|$)"), RE_I).is_match(&reply) {
        return None;
    }
    if memory::swap::thing(text_).py().is_some() {
        return None;
    }
    if !_own_thing(&understanding::understand(text_, Some(memory))).is_empty() {
        return None;
    }
    let has_amount = Re::new(&escaped, RE_I);
    let sentences: Vec<String> = SENTENCE.split(&reply).into_iter().filter(|s| !text::strip(s).is_empty()).collect();
    let carrying: Vec<&String> = sentences.iter().filter(|s| has_amount.is_match(s)).collect();
    if carrying.is_empty() {
        return None;
    }
    // `min(carrying, key=len)`: the first of the shortest
    let mut best = carrying.iter().min_by_key(|s| text::len(s)).map(|s| text::strip(s).to_string()).unwrap();
    for lead in [": ", ", so ", " so ", ", which means ", " \u{2014} ", ", then ", "; "] {
        let Some(cut) = best.rfind(lead) else { continue };
        let tail = text::strip(&best[cut + lead.len()..]).to_string();
        if has_amount.is_match(&tail) {
            best = tail;
            break;
        }
    }
    if best.is_empty() { None } else { Some(up1(&best)) }
}

/// Python: bot/modules/answers.py:4195 `_the_machine_follow_up(text, reading, memory)`
/// The decision about the machine the last line named, when this line names none of its own.
pub fn _the_machine_follow_up(text_: &str, reading: Option<&Understood>, memory: &Memory) -> Option<String> {
    let _ = reading;
    let line = _flat(text_);
    if !knowledge::home::machine_named(&line).is_empty() {
        return None;
    }
    if !MACHINE_ASKING.is_match(&line) {
        return None;
    }
    let asks = HOME_CALLING_SOMEONE.is_match(&line) || (HOME_FOLLOWING_IT.is_match(&line) && HOME_DECIDING_NOW.is_match(&line));
    if !asks {
        return None;
    }
    let earlier = knowledge::home::machine_before(memory);
    if earlier.is_empty() {
        return None;
    }
    if let Some(part) = memory::narrow::narrow(&memory::last_bot(memory), text_, "").filter(|p| !p.is_empty()) {
        return Some(part);
    }
    let reply = knowledge::home::the_machine_decision(memory, &earlier, &line);
    if text::strip(&reply).is_empty() { None } else { Some(reply) }
}

/// Python: bot/modules/answers.py:4247 `_part_asked(text, memory)`
/// The part of the last answer the line asks for, or None when it asks about something else.
pub fn _part_asked(text_: &str, memory: &mut Memory) -> Option<String> {
    if _a_property_word_names_a_thing(text_) {
        return None;
    }
    let kind = memory::narrow::kind_of(text_);
    if WHEN_OF_THE_DAY.is_match(text_) && kind.as_deref() == Some("date") {
        return None;
    }
    let earlier = _last_answer(memory);
    if !_asks_part(text_) {
        let Some(earlier) = earlier.as_ref() else { return None };
        if kind.is_none() {
            return None;
        }
        let mut said = word_set(&record_str(earlier, "reply"));
        said.extend(record_keys(earlier));
        let named = minus(&minus(&word_set(text_), &PROPERTY_WORDS), &NOT_NEW);
        if !named.is_empty() && named.is_disjoint(&said) {
            return None;
        }
    }
    if memory::topics::left_for_a_blank(&memory_value(memory), text_) {
        return None;
    }
    if RATE.is_match(text_) {
        return None;
    }
    let earlier = earlier?;
    let reply = record_str(&earlier, "reply");
    if COOK_VERB.is_match(text_) && !KEEPS_ASK.is_match(text_) && KEEPS_ANSWER.is_match(&reply) && !COOK_VERB.is_match(&reply) {
        return None;
    }
    if SUM_TABLE.findall(&reply).len() > 1 && kind.as_deref() == Some("time") {
        return None;
    }
    let earlier_keys = record_keys(&earlier);
    let own_named = minus(&minus(&word_set(text_), &PROPERTY_WORDS), &NOT_NEW);
    if !own_named.is_empty() && !earlier_keys.is_empty() {
        let reading = understanding::understand(text_, Some(&*memory));
        let same_item = knowledge::entry::entries().iter().zip(ALL_KEY_WORDS.iter()).find(|(_, keys)| **keys == earlier_keys);
        if let Some((item, keys)) = same_item {
            if !own_named.is_disjoint(keys) {
                let fresh = _payload_mut(item, memory, &reading);
                if text::strip(&fresh).is_empty()
                    || (memory::narrow::flat(&fresh) != memory::narrow::flat(&reply) && !own_named.is_disjoint(&word_set(&fresh)))
                {
                    return None;
                }
            }
        }
    }
    let reply_words = word_set(&reply);
    let named = minus(&minus(&minus(&minus(&word_set(text_), &PROPERTY_WORDS), &PLACE_WORDS), &reply_words), &earlier_keys);
    let reading = understanding::understand(text_, Some(&*memory));
    let own = find_plain(&reading);
    if let Some(own) = own.filter(|o| other_id(o, &earlier)) {
        let theirs = minus(&_key_words(own), &earlier_keys);
        let hit: HashSet<String> = minus(&word_set(text_), &PROPERTY_WORDS).intersection(&theirs).cloned().collect();
        if !hit.is_empty() && hit.is_disjoint(&reply_words) {
            let fresh = _payload_mut(own, memory, &reading);
            if !text::strip(&fresh).is_empty() && !hit.is_disjoint(&word_set(&fresh)) {
                return None;
            }
        }
        if !hit.is_empty() && kind.is_none() && own.domain != record_str(&earlier, "domain") {
            let fresh = _payload_mut(own, memory, &reading);
            if !text::strip(&fresh).is_empty()
                && memory::narrow::flat(&fresh) != memory::narrow::flat(&reply)
                && !hit.is_disjoint(&word_set(&fresh))
            {
                return None;
            }
        }
    }
    if !named.is_empty() {
        if let Some(own) = own {
            if !named.is_disjoint(&_key_words(own)) {
                if other_id(own, &earlier) {
                    return None;
                }
                let fresh = _payload_mut(own, memory, &reading);
                if !text::strip(&fresh).is_empty() && memory::narrow::flat(&fresh) != memory::narrow::flat(&reply) {
                    return None;
                }
            } else if other_id(own, &earlier) && _keyed_on_the_line(own, text_) {
                let fresh = _payload_mut(own, memory, &reading);
                if !text::strip(&fresh).is_empty() && memory::narrow::flat(&fresh) != memory::narrow::flat(&reply) {
                    return None;
                }
            }
            if other_id(own, &earlier) && AMOUNT.is_match(text_) && !AMOUNT.is_match(&reply) && _keyed_on_the_line(own, text_) {
                return None;
            }
            if _keyed_on_the_line(own, text_) {
                let mine = &own.domain;
                let theirs = record_str(&earlier, "domain");
                if !mine.is_empty() && !theirs.is_empty() && *mine != theirs {
                    return None;
                }
            }
        }
    }
    if let Some(own) = own {
        let shared: HashSet<String> =
            _key_words(own).intersection(&earlier_keys).filter(|w| word_set(text_).contains(*w)).cloned().collect();
        if !shared.is_empty() && other_id(own, &earlier) && _keyed_on_the_line(own, text_) {
            let mine = &own.domain;
            let theirs = record_str(&earlier, "domain");
            if !mine.is_empty() && !theirs.is_empty() && *mine != theirs {
                let fresh = _payload_mut(own, memory, &reading);
                if !text::strip(&fresh).is_empty() && memory::narrow::flat(&fresh) != memory::narrow::flat(&reply) {
                    return None;
                }
            }
        }
    }
    let same = match own {
        Some(o) => Some(o),
        None => find_plain(&reading),
    };
    if let Some(same) = same.filter(|s| !other_id(s, &earlier)) {
        let named = minus(&minus(&minus(&word_set(text_), &PROPERTY_WORDS), &PLACE_WORDS), &reply_words);
        if !named.is_disjoint(&_key_words(same)) || !minus(&_figures(text_), &_figures(&reply)).is_empty() {
            let fresh = _payload_mut(same, memory, &reading);
            if !text::strip(&fresh).is_empty() && memory::narrow::flat(&fresh) != memory::narrow::flat(&reply) {
                return None;
            }
        }
    }
    let other = find_plain(&understanding::understand(text_, Some(&*memory)));
    if let Some(other) = other.filter(|o| other_id(o, &earlier)) {
        let newly = minus(&minus(&minus(&word_set(text_), &PROPERTY_WORDS), &PLACE_WORDS), &reply_words);
        if !newly.is_disjoint(&_key_words(other)) {
            let fresh_reading = understanding::understand(text_, Some(&*memory));
            let fresh = _payload_mut(other, memory, &fresh_reading);
            if !text::strip(&fresh).is_empty() && memory::narrow::flat(&fresh) != memory::narrow::flat(&reply) {
                return None;
            }
        }
    }
    if kind.as_deref() == Some("time") && inline_re!(r"\bleav(?:e|ing)\b").is_match(text_) {
        for sentence in inline_re!(r"(?<=[.!?])\s+").split(text::strip(&reply)) {
            if inline_re!(r"\s*(?:if|unless)\b", RE_I).match_(&sentence).is_some() && inline_re!(r"\d|\ban?\b").is_match(&sentence) {
                return Some(text::strip(&sentence).to_string());
            }
        }
    }
    let keys_list: Vec<String> = earlier_keys.iter().cloned().collect();
    if let Some(sized) = memory::topics::sized_reply(&reply, text_, memory, &keys_list).py() {
        return Some(sized);
    }
    let own_price = find_plain(&understanding::understand(text_, Some(&*memory)));
    if let Some(own_price) = own_price.filter(|o| other_id(o, &earlier)) {
        if _only_a_figure(memory::narrow::narrow(&reply, text_, "").as_deref()) {
            return None;
        }
        let theirs = minus(&_key_words(own_price), &earlier_keys);
        let hit = !word_set(text_).is_disjoint(&theirs);
        let fresh_reading = understanding::understand(text_, Some(&*memory));
        let fresh = _payload_mut(own_price, memory, &fresh_reading);
        if hit && _a_figure_apart(memory::narrow::narrow(&reply, text_, "").as_deref(), &fresh) {
            return None;
        }
    }
    let about = earlier.get("keys").map(|k| k.items().iter().map(py_str).collect::<Vec<_>>().join(" ")).unwrap_or_default();
    let mut part = memory::narrow::the_part_asked(&reply, text_, &about).filter(|p| !text::strip(p).is_empty());
    if part.is_none() {
        part = _sentence_naming_the_part(&reply, text_);
    }
    if let Some(p) = part.clone().filter(|p| !text::strip(p).is_empty()) {
        let said = kind.as_deref().and_then(|k| KIND_SAID.iter().find(|(name, _)| name == k)).map(|(_, rx)| *rx);
        if let Some(said) = said {
            if !said.is_match(&p) {
                part = None;
            } else if kind.as_deref() == Some("time")
                && !inline_re!(r"\bbefore\b", RE_I).is_match(text_)
                && A_LEAD_TIME.is_match(&_sentence_with(&reply, &p))
            {
                part = None;
            }
        } else if kind.as_deref() == Some("time")
            && !inline_re!(r"\bbefore\b", RE_I).is_match(text_)
            && A_LEAD_TIME.is_match(&_sentence_with(&reply, &p))
        {
            part = None;
        }
    }
    if kind.as_deref() == Some("cause") {
        if let Some(p) = part.as_ref().filter(|p| !text::strip(p).is_empty()) {
            return Some(_reason_again(&_cause_part(p), &memory::last_bot(memory)));
        }
    }
    let whole = memory::topics::entry_answer(memory, text_).filter(|w| !w.is_empty()).unwrap_or(reply);
    _part_not_repeated(part.as_deref(), &whole, memory)
}

/// Python: bot/modules/answers.py:4541 `_amount_for_the_name(clause, named)`
/// True when the clause ties a quantity to that very thing, not to a neighbour.
pub fn _amount_for_the_name(clause: &str, named: &HashSet<String>) -> bool {
    let said = format!(" {} ", inline_re!(r"[^a-z0-9. ]+").sub(" ", &clause.to_lowercase()));
    let names: HashSet<String> = named.iter().map(|n| n.to_lowercase()).collect();
    for name in names {
        let thing = format!("{}s?", crate::py::re::escape(text::rstrip_chars(&name, "s")));
        if Re::new(&format!(r"\b(?:{})\s+(?:[a-z]+\s+){{0,2}}{thing}\b", *_AMOUNT_BEFORE), 0).is_match(&said) {
            return true;
        }
        if Re::new(&format!(r"\b(?:{})\s+(?:[a-z]+\s+){{0,3}}{thing}\b", *_PROPORTION_PHRASE), 0).is_match(&said) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:4596 `_sentence_naming_the_part(reply, text)`
/// The clause of the answer that gives a quantity for the thing a follow-up asks about.
pub fn _sentence_naming_the_part(reply: &str, text_: &str) -> Option<String> {
    let kind = memory::narrow::kind_of(text_)?;
    if kind == "cause" || kind == "order" {
        return None;
    }
    let named = minus(&minus(&minus(&word_set(text_), &PROPERTY_WORDS), &NOT_NEW), &WHERE_WORDS);
    if named.is_empty() {
        return None;
    }
    let sentences: Vec<String> =
        SENTENCE.split(text::strip(reply)).iter().map(|s| text::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    if sentences.is_empty() {
        return None;
    }
    let flat_reply = _flat(reply);
    for sentence in &sentences {
        for clause in CLAUSE_SEAM.split(sentence) {
            let mut clause = text::strip_chars(&clause, " .,").to_string();
            if clause.is_empty() || named.is_disjoint(&word_set(&clause)) {
                continue;
            }
            if !_amount_for_the_name(&clause, &named) {
                continue;
            }
            if text::split_ref(&clause).len() < 4 && !inline_re!(r"\b(?:of|to|per)\b").is_match(&clause) {
                continue;
            }
            if JOINING_WORD.match_(&clause).is_some() {
                clause = text::rstrip_chars(text::strip(sentence), ",;: ").to_string();
                if text::split_ref(&clause).len() > 40 {
                    continue;
                }
            } else if clause.chars().next().is_some_and(|c| crate::understanding::pyuni::ch_lower(c)) {
                clause = up1(&clause);
            }
            let part = if clause.ends_with(['.', '!', '?']) { clause } else { format!("{clause}.") };
            if _flat(&part) == flat_reply {
                return None;
            }
            return Some(part);
        }
    }
    None
}

/// Python: bot/modules/answers.py:4671 `_names_no_cause(head)`
/// True for the head of a colon sentence that labels a cause without naming one.
pub fn _names_no_cause(head: &str) -> bool {
    if COUNTS_THE_CAUSES.match_(head).is_some() {
        return true;
    }
    memory::topics::keywords(head).iter().all(|w| NAMES_NO_CAUSE.contains(w))
}

/// Python: bot/modules/answers.py:4679 `_cause_part(part)`
/// The side of a colon sentence that gives the reason, or the whole sentence.
pub fn _cause_part(part: &str) -> String {
    let part = text::strip(part);
    let Some((head, tail)) = part.split_once(':') else { return part.to_string() };
    // `candidate is head`: CPython shares one object for equal empty or one-character strings
    let interned = head == tail && head.chars().count() <= 1 && head.chars().all(|c| (c as u32) < 256);
    for (i, candidate) in [head, tail].into_iter().enumerate() {
        if (i == 0 || interned) && _names_no_cause(head) {
            continue;
        }
        if CAUSE.is_match(candidate) {
            let mut candidate = text::rstrip_chars(text::strip(candidate), ",;").to_string();
            if candidate.chars().next().is_some_and(|c| crate::understanding::pyuni::ch_lower(c)) {
                candidate = up1(&candidate);
            }
            return if candidate.ends_with(['.', '!', '?']) { candidate } else { format!("{candidate}.") };
        }
    }
    part.to_string()
}

/// Python: bot/modules/answers.py:4700 `_reason_again(reason, last)`
/// The same reason said another way when it was just given.
pub fn _reason_again(reason: &str, last: &str) -> String {
    let reason = text::strip(reason).to_string();
    let shape = inline_re!(r"^(.*?)\b(?:is|are|was|were)\s+(usually|commonly|often)\s+(.+)$", RE_I);
    if let Some(m) = shape.match_(&reason) {
        let short = format!("{} {}", text::capitalize(m.str(2)), m.str(3));
        if _flat(&short) != _flat(&reason) {
            if _flat(&reason) == _flat(last) {
                return short;
            }
            if _flat(&short) == _flat(last) {
                return reason;
            }
        }
    }
    let so = inline_re!(r",\s*so\b", RE_I);
    if _flat(&reason) == _flat(last) && so.is_match(&reason) {
        let head = split_once_re(so, &reason).remove(0);
        let mut short = text::rstrip_chars(text::strip(&head), ",;").to_string();
        if !short.ends_with(['.', '!', '?']) {
            short.push('.');
        }
        if _flat(&short) != _flat(&reason) {
            return short;
        }
    }
    reason
}

/// Python: bot/modules/answers.py:4762 `_asked_figure(text, reply, other_reply)`
/// The sentence of `reply` that carries the figure this line asks for, or "".
pub fn _asked_figure(text_: &str, reply: &str, other_reply: &str) -> String {
    if !ASKS_A_MEASURE.is_match(text_) {
        return String::new();
    }
    let condition = WHEN_IS_A_CONDITION.is_match(text_);
    if A_MEASURE.is_match(other_reply) {
        return String::new();
    }
    for sentence in SENTENCE.split(text::strip(reply)) {
        if !A_MEASURE.is_match(&sentence) {
            continue;
        }
        if condition && A_WARNING.is_match(&sentence) {
            continue;
        }
        return text::strip(&sentence).to_string();
    }
    String::new()
}

/// Python: bot/modules/answers.py:4787 `_the_figure(text, reply)`
/// The sentence of `reply` carrying the figure this line asks for, or "".
pub fn _the_figure(text_: &str, reply: &str) -> String {
    _asked_figure(text_, reply, "")
}

/// Python: bot/modules/answers.py:4856 `_names_a_property(text)`
/// True when the line names a property of its own, not just a length or a price.
pub fn _names_a_property(text_: &str) -> bool {
    memory::topics::keywords(text_).iter().any(|w| PROPERTY_WORDS.contains(w) && !GENERIC_ASK.contains(w))
}

/// Python: bot/modules/answers.py:5053 `_minutes_in(text)`
/// Every length of time a piece of text states, in minutes, as (low, high) pairs.
pub fn _minutes_in(text_: &str) -> Vec<(i64, i64)> {
    if text_.is_empty() {
        return vec![];
    }
    let stated = _HOURS_AND_MINUTES.sub_fn(text_, |m| {
        format!("{} minutes", py_int(m.str(1)).unwrap_or(0) * 60 + py_int(m.str(2)).unwrap_or(0))
    });
    let mut found = Vec::new();
    for m in inline_re!(r"\b(\d+)(?:\s*(?:to|-|–)\s*(\d+))?\s*(minutes?|mins?|hours?|hrs?)\b", RE_I).finditer(&stated) {
        let unit = if m.str(3).to_lowercase().starts_with('h') { 60 } else { 1 };
        let low = py_int(m.str(1)).unwrap_or(0) * unit;
        let high = py_int(m.get(2).unwrap_or(m.str(1))).unwrap_or(0) * unit;
        found.push((low, high));
    }
    found
}

/// Python: bot/modules/answers.py:5070 `_a_figure_apart(mine, theirs)`
/// True when two answers give lengths of time that cannot both be about the same thing.
pub fn _a_figure_apart(mine: Option<&str>, theirs: &str) -> bool {
    let mine_spans = _minutes_in(mine.unwrap_or(""));
    let their_spans = _minutes_in(theirs);
    if mine_spans.is_empty() || their_spans.is_empty() {
        return false;
    }
    mine_spans.iter().any(|(low, high)| their_spans.iter().any(|(low_other, high_other)| high_other < low || high < low_other))
}

/// Python: bot/modules/answers.py:5086 `_only_a_figure(part)`
/// True when the part is a sum of money and nothing else ("£800 to £1,200.").
pub fn _only_a_figure(part: Option<&str>) -> bool {
    let Some(part) = part.filter(|p| !text::strip(p).is_empty()) else { return false };
    let words = memory::narrow::words_of(part);
    inline_re!(r"[£$€]|\bpounds?\b|\bquid\b").is_match(part) && words.len() <= 6
}

/// Python: bot/modules/answers.py:5097 `_figures(text)`
/// The numbers a line states, comma or not: "£400,000" and "400000" are one figure.
pub fn _figures(text_: &str) -> HashSet<String> {
    FIGURE.findall_str(text_).into_iter().map(|one| one.replace(',', "")).collect()
}

/// Python: bot/modules/answers.py:5193 `_keyed_on_the_line(item, text)`
/// True when the entry is keyed on a word the line itself says: "wear" in "what do i wear".
pub fn _keyed_on_the_line(item: &Entry, text_: &str) -> bool {
    let keys = _key_words(item);
    for word in az_words(&text_.to_lowercase()) {
        if text::len(&word) < 3 || GENERIC.contains(&word) || NOT_NEW.contains(&word) || PROPERTY_WORDS.contains(&word) {
            continue;
        }
        let bare = text::rstrip_chars(&word, "s");
        if keys.contains(if bare.is_empty() { word.as_str() } else { bare }) {
            return true;
        }
    }
    false
}

re_static!(BARE_ASK, ADDING_TO_THE_SUBJECT, A_MEASURE_OF_THE_SUBJECT, SAY_IT, NAMING, ASKS_A_QUANTITY);
set_static!(WHEN_NOT_A_THING, _ASKING_ONLY, OPENS_WITH_AN_AUXILIARY, HANDOVER_FILLER, CHOICE_KINDS, THING_AFTER, AFTER_A_PLACE,
    NO_THING_NAMED, _AUXILIARY, DESCRIBES_ONE, PRONOUN_OBJECTS, EXPLAIN_LEADS, EXPLAIN_FILLER);
strs_static!(FIGURE_ASKS, _ADJECTIVE_ENDINGS);
static FEW_ENTRIES: LazyLock<usize> = LazyLock::new(|| g("FEW_ENTRIES").as_int().unwrap_or(12) as usize);
static DEFINITION_WORD_AT: LazyLock<usize> = LazyLock::new(|| g("DEFINITION_WORD_AT").as_int().unwrap_or(6) as usize);
static STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.STOPWORDS"));
static VERB_FORMS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.kinds.VERB_FORMS"));
static SIGNALS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    g("SIGNALS").pairs().iter().filter_map(|(k, v)| Some((k.str().to_string(), v.as_re()?))).collect()
});

/// Python truthiness of what another file's function returned, whatever its contract type is
/// today (`()` where the trace only saw a falsy value reads as false).
pub trait PyTruth {
    fn py_truth(&self) -> bool;
}
impl PyTruth for () {
    fn py_truth(&self) -> bool {
        false
    }
}
impl PyTruth for bool {
    fn py_truth(&self) -> bool {
        *self
    }
}
impl PyTruth for String {
    fn py_truth(&self) -> bool {
        !self.is_empty()
    }
}
impl<T: PyTruth> PyTruth for Option<T> {
    fn py_truth(&self) -> bool {
        self.as_ref().is_some_and(PyTruth::py_truth)
    }
}
impl PyTruth for Value {
    fn py_truth(&self) -> bool {
        self.truthy()
    }
}
impl PyTruth for Vec<String> {
    fn py_truth(&self) -> bool {
        !self.is_empty()
    }
}

/// The phrases of an entry's first key part, when that part is a tuple of words (`first[0]`
/// that is not a str).
fn first_key_things(item: &Entry) -> Vec<String> {
    match item.key.first() {
        Some(crate::knowledge::entry::KeyPart::Any(words)) => words.clone(),
        _ => vec![],
    }
}

/// `word.rstrip("s") or word`
fn bare_s(word: &str) -> &str {
    let bare = text::rstrip_chars(word, "s");
    if bare.is_empty() { word } else { bare }
}

/// Python: bot/modules/answers.py:4869 `_with_bare_subject(text, memory)`
/// A bare ask with the thing it is about on the end.
pub fn _with_bare_subject(text_: &str, memory: &Memory) -> String {
    if BARE_ASK.match_(text_).is_none() {
        return text_.to_string();
    }
    let subject = memory::topics::subject(memory);
    if subject.is_empty() || TIME_WORD.match_(text::strip(&subject)).is_some() {
        return text_.to_string();
    }
    let low = text_.to_lowercase();
    if text::split_ref(&subject).iter().any(|w| low.contains(&text::strip_chars(w, ".,!?;:").to_lowercase())) {
        return text_.to_string();
    }
    format!("{} {subject}", text::rstrip_chars(text_, "?.! "))
}

/// Python: bot/modules/answers.py:4883 `_the_things_to_try(understood, memory)`
/// The things a "that" in the line could mean, newest first.
pub fn _the_things_to_try(understood: Option<&Understood>, memory: &Memory) -> Vec<String> {
    let mut things: Vec<String> = Vec::new();
    if let Some(u) = understood {
        if let Some(leans) = &u.leans_on {
            for word in &leans.words {
                if text::len(text::strip(word)) > 2 {
                    things.push(text::strip(word).to_lowercase());
                }
            }
        }
        for value in [&u.reference.that, &u.reference.it] {
            if let Some(v) = value.as_ref().filter(|v| !text::strip(v).is_empty()) {
                things.push(text::strip(v).to_string());
                break;
            }
        }
        // ("this" is the third key Python tries; the reading's `ref` never carries one)
    }
    let subject = memory::topics::subject(memory);
    if !subject.is_empty() {
        things.push(subject);
    }
    let mut seen = HashSet::new();
    things.into_iter().filter(|t| !t.is_empty() && seen.insert(t.clone())).collect()
}

/// Python: bot/modules/answers.py:4916 `_the_answer_from_thing(text, thing, memory)`
/// The entry for `thing` and its reply to the line, or (None, None).
pub fn _the_answer_from_thing(text_: &str, thing: &str, memory: &mut Memory) -> (Option<&'static Entry>, Option<String>) {
    if thing.is_empty() {
        return (None, None);
    }
    let fuller = if thing == memory::topics::subject(memory) { _with_subject(text_, memory, None) } else { _with_the_thing(text_, thing) };
    if text::strip(&fuller) == text::strip(text_) {
        return (None, None);
    }
    let wider = understanding::understand(&fuller, Some(&*memory));
    let (found, said) = _offer_with(&mut |_: &mut Memory, reject: Reject| reader_entry(knowledge::lookup::find(&wider, None, reject, false)), memory, &wider);
    let Some(found) = found else { return (None, None) };
    let said_text = said.clone().unwrap_or_default();
    if !_is_about(found, thing) || _said_before(&said_text, memory, 3) {
        return (None, None);
    }
    (Some(found), said)
}

/// Python: bot/modules/answers.py:4932 `_with_the_thing(text, thing)`
/// The line with its "that"/"it" written out as the thing.
pub fn _with_the_thing(text_: &str, thing: &str) -> String {
    let mut words = text::split(text_);
    if thing.is_empty() {
        return text_.to_string();
    }
    let said: Vec<String> = words.iter().map(|w| text::strip_chars(w, ".,!?;:").to_lowercase()).collect();
    for (i, word) in said.iter().enumerate() {
        if REFERRING.contains(word) {
            words[i] = format!("the {thing}");
            return words.join(" ");
        }
    }
    text_.to_string()
}

/// Python: bot/modules/answers.py:4946 `_care_holds(text, memory)`
/// True when the composer is already holding this line inside a safety thread.
pub fn _care_holds(text_: &str, memory: &Memory) -> bool {
    py_try(|| compose::fallback::careful(text_, memory).py_truth()).unwrap_or(false)
}

/// Python: bot/modules/answers.py:4962 `_named_by(item, subject)`
/// True when the subject names the entry, not just brushes one of its words.
pub fn _named_by(item: &Entry, subject: &str) -> bool {
    let low = format!(" {} ", subject.to_lowercase());
    for part in &item.key {
        for option in part.words() {
            let option = text::strip(&option.to_lowercase()).to_string();
            if option.contains(' ') && low.contains(&option) {
                return true;
            }
        }
    }
    let mut subject_keys: HashSet<String> = HashSet::new();
    for part in item.key.iter().take(1) {
        for option in part.words() {
            for word in text::split_ref(&option.to_lowercase()) {
                subject_keys.insert(bare_s(word).to_string());
            }
        }
    }
    let parts = text::split_ref(&low);
    if parts.len() == 1 {
        return !subject_keys.is_empty();
    }
    let found = parts
        .iter()
        .filter(|w| {
            let w = text::rstrip_chars(text::strip_chars(w, ".,!?;:"), "s");
            text::len(w) > 2 && subject_keys.contains(w)
        })
        .count();
    found >= 2
}

/// Python: bot/modules/answers.py:5005 `_the_answer_was_about_the_subject(memory)`
/// True when the thing the chat is on is also what the last answer was about.
pub fn _the_answer_was_about_the_subject(memory: &Memory) -> bool {
    let subject = memory::topics::subject(memory).to_lowercase();
    if subject.is_empty() {
        return false;
    }
    let keys = memory.get("entry_answer").filter(|v| v.is_dict()).and_then(|l| l.get("keys")).filter(|k| k.truthy());
    let Some(keys) = keys else { return true };
    let words: HashSet<String> = text::split_ref(&subject).iter().map(|w| text::rstrip_chars(text::strip_chars(w, ".,!?;:"), "s").to_string()).collect();
    keys.items().iter().any(|k| words.contains(text::rstrip_chars(&py_str(k).to_lowercase(), "s")))
}

/// `_NAME_WORDS` and `_NAME_DOMAINS`, built once.
static NAME_WORDS: LazyLock<(HashSet<String>, std::collections::HashMap<String, HashSet<String>>)> = LazyLock::new(|| {
    let mut counted: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut domains: std::collections::HashMap<String, HashSet<String>> = std::collections::HashMap::new();
    for (one, words) in knowledge::entry::entries().iter().zip(ALL_KEY_WORDS.iter()) {
        for word in words {
            *counted.entry(word.clone()).or_default() += 1;
            if !one.domain.is_empty() {
                domains.entry(word.clone()).or_default().insert(one.domain.clone());
            }
        }
    }
    let names: HashSet<String> = counted.into_iter().filter(|(_, seen)| *seen <= *FEW_ENTRIES).map(|(w, _)| w).collect();
    let by_name = names.iter().map(|w| (w.clone(), domains.get(w).cloned().unwrap_or_default())).collect();
    (names, by_name)
});

/// Python: bot/modules/answers.py:5119 `_name_words()`
/// The words that name a thing: keyed on by a handful of entries, not by the whole base.
pub fn _name_words() -> &'static HashSet<String> {
    &NAME_WORDS.0
}

/// Python: bot/modules/answers.py:5149 `_filler(word)`
/// A word that says how a line is put, not what it is about.
pub fn _filler(word: &str) -> bool {
    if GENERIC.contains(word) || STOPWORDS.contains(word) || NOT_NEW.contains(word) {
        return true;
    }
    if PROPERTY_WORDS.contains(word) || WHEN_NOT_A_THING.contains(word) {
        return true;
    }
    for tail in ["re", "ve", "ll", "d", "m"] {
        if let Some(head) = word.strip_suffix(tail) {
            if STOPWORDS.contains(head) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/answers.py:5161 `_left_out(item, text)`
/// True when the line names a thing this entry never mentions.
pub fn _left_out(item: &Entry, text_: &str) -> bool {
    let keys = _key_words(item);
    let names = _name_words();
    for word in az_words(&text_.to_lowercase()) {
        let word = text::rstrip_chars(&word, "'").to_string();
        if text::len(&word) < 4 || _filler(&word) {
            continue;
        }
        let bare = bare_s(&word).to_string();
        if keys.contains(&bare) || keys.contains(&word) {
            continue;
        }
        if names.contains(&bare) || names.contains(&word) {
            let field = &item.domain;
            let in_field = [&word, &bare].iter().any(|form| NAME_WORDS.1.get(*form).is_some_and(|d| d.contains(field)));
            if !in_field {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/answers.py:5209 `_shares_a_word(item, subject)`
/// True when the entry is keyed on a word of the subject: "save money" is about saving.
pub fn _shares_a_word(item: &Entry, subject: &str) -> bool {
    let keys = _key_words(item);
    let low = subject.to_lowercase();
    text::split_ref(&low).iter().any(|w| {
        let w = text::strip_chars(w, ".,!?;:");
        text::len(w) > 2 && keys.contains(bare_s(w))
    })
}

/// Python: bot/modules/answers.py:5225 `_answers_the_figure(text, reply)`
/// False when the line asks for a figure and the entry's answer names none of that kind.
pub fn _answers_the_figure(text_: &str, reply: &str) -> bool {
    if text::strip(reply).is_empty() {
        return true;
    }
    let Some(kind) = memory::narrow::kind_of(text_) else { return true };
    if !FIGURE_ASKS.contains(&kind) {
        return true;
    }
    let mut wanted = vec![kind.clone()];
    if kind == "amount" || kind == "money" {
        wanted.push("money".into());
    }
    wanted.iter().any(|one| SIGNALS.iter().find(|(k, _)| k == one).is_some_and(|(_, rx)| rx.is_match(reply)))
}

/// Python: bot/modules/answers.py:5281 `_asks_how_to_say_it(text)`
/// Whether the line asks how to say something rather than what the thing is.
pub fn _asks_how_to_say_it(text_: &str) -> bool {
    SAY_IT.is_match(text_)
}

/// Python: bot/modules/answers.py:5286 `_in_the_subjects_field(item, subject)`
/// Whether the entry is in the same field as the thing the subject names, or has no field.
pub fn _in_the_subjects_field(item: &Entry, subject: &str) -> bool {
    let field = if subject.is_empty() { None } else { understanding::understand(subject, None).topic };
    match field.filter(|f| !f.is_empty()) {
        None => true,
        Some(f) => item.domain == f,
    }
}

/// Python: bot/modules/answers.py:5310 `_own_thing_words(text, memory)`
/// The words in the line that name a thing of its own, not the ask or the subject on screen.
pub fn _own_thing_words(text_: &str, memory: &Memory) -> Vec<String> {
    let understood = understanding::understand(text_, Some(memory));
    let subject = memory::topics::subject(memory).to_lowercase();
    let mut out = Vec::new();
    for word in &understood.keywords {
        let low = word.to_lowercase();
        if _ASKING_ONLY.contains(&low) || subject.contains(&low) || crate::understanding::pyuni::is_digit(&low) || text::len(&low) < 3 {
            continue;
        }
        out.push(low);
    }
    out
}

/// Python: bot/modules/answers.py:5332 `_about_the_talk(item, text, memory, reading)`
/// False when the entry is from a part of the knowledge base this talk never touched.
pub fn _about_the_talk(item: &Entry, text_: &str, memory: &Memory, reading: &Understood) -> bool {
    if memory.understood.as_ref().is_some_and(|u| u.change_of_subject) {
        return true;
    }
    let subject = memory::topics::subject(memory);
    let record = memory::topics::current(memory);
    match record.filter(|r| r.is_dict()) {
        None => return true,
        Some(r) => {
            if r.get("turn").and_then(Value::as_int) == Some(memory.turns.len() as i64) {
                return true;
            }
        }
    }
    let thing = _head_thing(&subject);
    if thing.is_empty() {
        return true;
    }
    if _is_about(item, &thing) {
        return true;
    }
    if memory.turns.is_empty() {
        return true;
    }
    if _names_the_entry(text_, item) {
        return true;
    }
    if _a_definition_of_the_entry(text_, item) {
        return true;
    }
    if _names_the_thing_apart_from_the_talk(item, text_, &subject, Some(reading)) {
        return true;
    }
    if _a_why_line_that_names(item, Some(reading)) {
        return true;
    }
    if _the_field_on_the_screen(item, text_, Some(reading), memory) {
        return true;
    }
    if _says_the_ask_whole(text_, item) {
        return true;
    }
    if _names_its_own_thing(text_, Some(memory)) && _agrees_with_its_own_field(item, reading) {
        return true;
    }
    knowledge::lookup::about_the_topic(item, reading)
}

/// Python: bot/modules/answers.py:5412 `_agrees_with_its_own_field(item, reading)`
/// True when the entry is in the field the line itself is read as, or in no field at all.
pub fn _agrees_with_its_own_field(item: &Entry, reading: &Understood) -> bool {
    if item.raw.attr("topic").map_or(true, Value::is_none) {
        return true;
    }
    knowledge::lookup::about_the_topic(item, reading)
}

/// Python: bot/modules/answers.py:5423 `_the_field_on_the_screen(item, text, reading, memory)`
/// True when the line says the entry's own subject and the entry is from the field on screen.
pub fn _the_field_on_the_screen(item: &Entry, text_: &str, reading: Option<&Understood>, memory: &Memory) -> bool {
    if !reading.is_some_and(|r| r.kind == "question") {
        return false;
    }
    let domain = memory.get("last_answer").filter(|v| v.is_dict()).map(|l| record_str(l, "domain")).unwrap_or_default();
    if domain.is_empty() || domain != item.domain {
        return false;
    }
    if _last_turn_answered_from_the_field(memory) {
        return false;
    }
    _says_a_subject_phrase(item, text_)
}

/// Python: bot/modules/answers.py:5455 `_last_turn_answered_from_the_field(memory)`
/// True when the turn before this one was the field's answer rather than an acknowledgement.
pub fn _last_turn_answered_from_the_field(memory: &Memory) -> bool {
    let Some(turn) = memory.turns.last() else { return false };
    let last = memory.get("last_answer");
    let reply = match last {
        Some(v) if v.is_dict() => v.get("reply"),
        other => other,
    };
    let Some(Value::Str(reply)) = reply else { return false };
    _same_answer(reply, &turn.bot)
}

/// Python: bot/modules/answers.py:5476 `_says_a_subject_phrase(item, text)`
/// True when the line says a subject of the entry in so many words, two words or more.
pub fn _says_a_subject_phrase(item: &Entry, text_: &str) -> bool {
    let Some(first) = item.key.first() else { return false };
    let said = format!(" {} ", inline_re!(r"[a-z0-9']+").findall_str(&text_.to_lowercase()).join(" "));
    for option in first.words() {
        let phrase = text::squash(&option.to_lowercase());
        if phrase.contains(' ') && said.contains(&format!(" {phrase} ")) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:5524 `_names_the_entry(text, item)`
/// True when the line names the thing this entry is about, with a determiner in front of it.
pub fn _names_the_entry(text_: &str, item: &Entry) -> bool {
    let mut heads: HashSet<String> = HashSet::new();
    for option in first_key_things(item) {
        let low = option.to_lowercase();
        if let Some(last) = text::split_ref(&low).last() {
            heads.insert(bare_s(last).to_string());
        }
    }
    if heads.is_empty() {
        return false;
    }
    let said = text::squash(&text_.to_lowercase());
    for found in NAMING.finditer(&said) {
        for word in inline_re!(r"[a-z-]+").findall_str(found.str(1)) {
            if heads.contains(text::rstrip_chars(&word, "s")) {
                return true;
            }
        }
    }
    let mut mine: Vec<&str> = text::split_ref(&said);
    for _ in 0..2 {
        if mine.first().is_some_and(|w| OPENS_WITH_AN_AUXILIARY.contains(*w)) {
            mine.remove(0);
        }
    }
    let opening = mine.iter().take(2).copied().collect::<Vec<_>>().join(" ");
    inline_re!(r"[a-z-]+").findall_str(&opening).iter().any(|w| heads.contains(text::rstrip_chars(w, "s")))
}

/// Python: bot/modules/answers.py:5564 `_a_definition_of_the_entry(text, item)`
/// True when the line asks what one of the entry's own words means ("what is inflation").
pub fn _a_definition_of_the_entry(text_: &str, item: &Entry) -> bool {
    let said = text::squash(&text_.to_lowercase());
    let words: Vec<String> = text::split_ref(&said).iter().map(|w| text::strip_chars(w, ".,!?;:").to_string()).collect();
    if words.is_empty() || words.len() > *DEFINITION_WORD_AT + 1 {
        return false;
    }
    let mut subject: HashSet<String> = HashSet::new();
    for option in first_key_things(item) {
        for word in text::split_ref(&option.to_lowercase()) {
            subject.insert(bare_s(word).to_string());
        }
    }
    if subject.is_empty() {
        return false;
    }
    for (at, word) in words.iter().enumerate() {
        if !subject.contains(bare_s(word)) {
            continue;
        }
        let before = &words[..at];
        if before.is_empty() || !before.iter().any(|one| EXPLAIN_LEADS.contains(one)) {
            continue;
        }
        if before.iter().all(|one| EXPLAIN_FILLER.contains(one)) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:5611 `_says_the_ask_whole(text, item)`
/// True when the line is nothing but one of the entry's own key phrases.
pub fn _says_the_ask_whole(text_: &str, item: &Entry) -> bool {
    let said: Vec<String> = words_in(&text_.to_lowercase()).into_iter().filter(|w| !HANDOVER_FILLER.contains(w)).collect();
    if said.len() < 2 {
        return false;
    }
    for part in &item.key {
        for option in part.words() {
            let phrase: Vec<String> = words_in(&option.to_lowercase()).into_iter().filter(|w| !HANDOVER_FILLER.contains(w)).collect();
            if phrase.len() >= 2 && phrase == said {
                return true;
            }
        }
    }
    false
}

/// Python: bot/modules/answers.py:5633 `_a_why_line_that_names(item, reading)`
/// True when a why-line names the subject of the entry it reached.
pub fn _a_why_line_that_names(item: &Entry, reading: Option<&Understood>) -> bool {
    let Some(reading) = reading.filter(|r| r.asks.iter().any(|a| a == "why")) else { return false };
    knowledge::lookup::names_its_own_subject(item, reading)
}

/// Python: bot/modules/answers.py:5640 `_names_the_thing_apart_from_the_talk(item, text, subject, reading)`
/// True when the line ends on this entry's own subject and says nothing of the talk's.
pub fn _names_the_thing_apart_from_the_talk(item: &Entry, text_: &str, subject: &str, reading: Option<&Understood>) -> bool {
    let Some(reading) = reading.filter(|r| r.kind == "question") else { return false };
    if !knowledge::lookup::names_its_own_subject(item, reading) {
        return false;
    }
    let said = memory::topics::words(text_);
    let Some(last) = said.last() else { return false };
    if !_is_about(item, last) {
        return false;
    }
    let all = words_in(&text_.to_lowercase());
    if all.iter().any(|w| PRONOUN_OBJECTS.contains(w)) {
        return false;
    }
    let thing = _head_thing(subject);
    if thing.is_empty() {
        return false;
    }
    let words: HashSet<String> = all.iter().map(|w| bare_s(w).to_string()).collect();
    !words.contains(bare_s(&thing))
}

/// Python: bot/modules/answers.py:5678 `_a_choice_with_one_thing_left_out(text)`
/// True when the line narrows a choice ("i dont want a scary film", "i hate horror books").
pub fn _a_choice_with_one_thing_left_out(text_: &str) -> bool {
    let flat_words: HashSet<String> = az_words(&text::squash(text_).to_lowercase()).into_iter().collect();
    if flat_words.is_disjoint(&CHOICE_KINDS) {
        return false;
    }
    !_refused(text_).is_empty()
}

/// Python: bot/modules/answers.py:5735 `_describes_the_thing(word)`
/// True when the word describes a thing rather than naming one.
pub fn _describes_the_thing(word: &str) -> bool {
    if NO_THING_NAMED.contains(word) {
        return true;
    }
    _ADJECTIVE_ENDINGS.iter().any(|ending| word.ends_with(ending.as_str()) && text::len(word) > text::len(ending) + 2)
}

/// Python: bot/modules/answers.py:5750 `_said_in_the_talk(memory)`
/// The words of the last few turns, the bot's own included.
pub fn _said_in_the_talk(memory: &Memory) -> HashSet<String> {
    let mut said = HashSet::new();
    for turn in &memory.turns {
        said.extend(words_in(&turn.user.to_lowercase()));
        said.extend(words_in(&turn.bot.to_lowercase()));
    }
    said
}

/// Python: bot/modules/answers.py:5761 `_already_in_the_talk(word, said)`
/// True when the word, its plural or a longer form of it has been said in the talk.
pub fn _already_in_the_talk(word: &str, said: &HashSet<String>) -> bool {
    if said.is_empty() {
        return false;
    }
    if said.contains(word) || said.contains(&format!("{word}s")) {
        return true;
    }
    if let Some(stem) = word.strip_suffix('s') {
        if said.contains(stem) {
            return true;
        }
    }
    said.iter().any(|one| one.starts_with(word))
}

/// Python: bot/modules/answers.py:5772 `_names_its_own_thing(text, memory=None)`
/// True when the line names a thing for itself rather than only what to do with the subject.
pub fn _names_its_own_thing(text_: &str, memory: Option<&Memory>) -> bool {
    let said = memory.map(_said_in_the_talk).unwrap_or_default();
    if ASKS_A_QUANTITY.is_match(text_) {
        return false;
    }
    let words = inline_re!(r"[a-z0-9']+").findall_str(&text_.to_lowercase());
    let count = inline_re!(r"\d+(?:s|st|nd|rd|th)?");
    for (index, word) in words.iter().enumerate() {
        if count.fullmatch(word).is_some() {
            continue;
        }
        if DESCRIBES_ONE.contains(word) || VERB_FORMS.contains(word) || _AUXILIARY.contains(word) {
            continue;
        }
        if let Some(stem) = word.strip_suffix('e') {
            if DESCRIBES_ONE.contains(&format!("{stem}ing")) {
                continue;
            }
        }
        if _describes_the_thing(word) {
            continue;
        }
        if words[(index + 1).min(words.len())..(index + 3).min(words.len())].iter().any(|w| w == "one" || w == "ones") {
            continue;
        }
        if _already_in_the_talk(word, &said) {
            continue;
        }
        let before = &words[index.saturating_sub(2)..index];
        if before.last().is_some_and(|w| THING_AFTER.contains(w)) {
            if before.len() == 2 && AFTER_A_PLACE.contains(&before[0]) {
                continue;
            }
            return true;
        }
        if before.len() == 2
            && THING_AFTER.contains(&before[0])
            && !THING_AFTER.contains(&before[1])
            && !_AUXILIARY.contains(&before[1])
            && !_describes_the_thing(&before[1])
        {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:5816 `_the_topic_rules_it_out(item, memory)`
/// True when the topic's own words are on the entry's avoid list.
pub fn _the_topic_rules_it_out(item: &'static Entry, memory: &Memory) -> bool {
    let topic = memory::topics::topic_text(memory);
    if topic.is_empty() {
        return false;
    }
    memory::topics::keeps_away(&topic, &topic, item)
}

re_static!(KEEPS, _NOT_THE_FRIDGE, A_FIGURE, A_PUT_RIGHT);
set_static!(STORING);

/// Python: bot/modules/answers.py:5829 `_in_context(text, memory, reject=None)` - the contract
/// form. Its memory is the plain dict and its reject a Python callable a `Value` cannot hold;
/// callers use [`_in_context_with`].
pub fn _in_context(text_: &str, memory: &Value, reject: &Value) {
    let _ = reject;
    let mut memory = memory_from_value(memory);
    let _ = _in_context_with(text_, &mut memory, None);
}

/// Python: bot/modules/answers.py:5829 `_in_context(text, memory, reject=None)`
/// The knowledge entry a follow-up is about, by naming the thing it refers to (the entry, and
/// the reading it phrases itself from when that is the written-out line's).
pub fn _in_context_with(text_: &str, memory: &mut Memory, reject: Reject) -> Offered {
    let subject = memory::topics::subject(memory);
    let thing = _head_thing(&subject);
    let refused = _refused(text_);
    let refusing = |one: &Entry| _hands_back_the_refusal(one, &refused) || reject.is_some_and(|r| r(one));
    let reject: Reject = if !refused.is_empty() { Some(&refusing) } else { reject };
    let understood = match memory.understood.clone() {
        Some(u) => u,
        None => understanding::understand(text_, Some(&*memory)),
    };
    if understood.intent == "ack" {
        return None;
    }
    if _asks_how_to_say_it(text_) {
        return None;
    }
    if !knowledge::lookup::leans_on_a_subject(&understood) {
        return None;
    }
    if _names_its_own_thing(text_, Some(memory)) {
        return None;
    }
    let mut fuller = _with_subject(text_, memory, None);
    if text::strip(&fuller) == text::strip(text_) && _own_thing_words(text_, memory).is_empty() {
        fuller = _about_subject(text_, memory, None);
    }
    if text::strip(&fuller) == text::strip(text_) {
        fuller = _with_bare_subject(text_, memory);
        for (rewrite, added, domain) in _qualifier_rewrites(text_, memory) {
            if let Some(item) = _from_rewrite(&rewrite, text_, memory, reject, &added, domain.as_deref()) {
                if _hands_back_the_refusal(item, &_refused(text_)) {
                    continue;
                }
                return Some((item, None));
            }
        }
    }
    if text::strip(&fuller) == text::strip(text_)
        && _the_answer_was_about_the_subject(memory)
        && (_own_thing_words(text_, memory).is_empty() || ADDING_TO_THE_SUBJECT.match_(text_).is_some())
    {
        let mut skip: HashSet<String> = HashSet::new();
        let own_reading = understanding::understand(text_, Some(&*memory));
        for fuller in _subject_tails(text_, memory, Some(&own_reading)) {
            let reading = understanding::understand(&fuller, Some(&*memory));
            let mut after_used = false;
            loop {
                let item = {
                    let skip_now = skip.clone();
                    let rej = move |one: &Entry| skip_now.contains(&one.id) || reject.is_some_and(|r| r(one));
                    knowledge::lookup::find(&reading, None, Some(&rej), false)
                };
                let Some(item) = item else { break };
                skip.insert(item.id.clone());
                if !_named_by(item, &subject) {
                    continue;
                }
                if !thing.is_empty() && !_is_about(item, &thing) {
                    continue;
                }
                let reply = _payload_mut(item, memory, &reading);
                if text::strip(&reply).is_empty() {
                    continue;
                }
                if _said_before(&reply, memory, 3) {
                    after_used = true;
                    continue;
                }
                if !_answers_the_figure(text_, &reply) {
                    return None;
                }
                if after_used && !_keyed_on_the_line(item, text_) {
                    return None;
                }
                if _left_out(item, text_) {
                    return None;
                }
                if _the_topic_rules_it_out(item, memory) {
                    return None;
                }
                return Some((item, None));
            }
        }
        return None;
    }
    if fuller.is_empty() || text::strip(&fuller) == text::strip(text_) {
        return None;
    }
    let reading_here = understanding::understand(&fuller, Some(&*memory));
    let want = if !thing.is_empty() { thing.clone() } else { subject.clone() };
    let fits = |one: &Entry| _is_about(one, &want) && knowledge::lookup::about_the_topic(one, &reading_here);
    let about = |one: &Entry| _is_about(one, &subject) && knowledge::lookup::about_the_topic(one, &reading_here);
    let strict = |one: &Entry| !fits(one) || reject.is_some_and(|r| r(one));
    let loose = |one: &Entry| !about(one) || reject.is_some_and(|r| r(one));
    let mut item = knowledge::lookup::find(&reading_here, None, Some(&loose), false);
    if item.is_none_or(|i| !_in_the_subjects_field(i, &subject)) {
        let better = knowledge::lookup::find(&reading_here, None, Some(&strict), false);
        if let Some(b) = better {
            if item.is_none_or(|_| _in_the_subjects_field(b, &subject)) {
                item = Some(b);
            }
        }
    }
    let item = item?;
    let own = if MEM_KIND_ASK.is_match(text_) { String::new() } else { _own_thing(&understanding::understand(text_, Some(&*memory))) };
    if !own.is_empty() && !_is_about(item, &own) {
        return None;
    }
    if _left_out(item, text_) {
        return None;
    }
    match &item.reply {
        crate::knowledge::entry::Reply::Choices(_) | crate::knowledge::entry::Reply::Fn(_) => {
            let mut held = reading_here.clone();
            let first = _payload_mut(item, memory, &reading_here);
            if _said_before(&first, memory, 3) {
                let here = understanding::understand(text_, Some(&*memory));
                let own_words = _payload_mut(item, memory, &here);
                if !text::strip(&own_words).is_empty() && !_said_before(&own_words, memory, 3) {
                    held = here;
                }
            }
            Some((item, Some(held)))
        }
        crate::knowledge::entry::Reply::Text(reply) => {
            if _said_before(reply, memory, 3) {
                return None;
            }
            if !_answers_the_figure(text_, reply) {
                return None;
            }
            Some((item, Some(reading_here)))
        }
    }
}

/// Python: bot/modules/answers.py:6005 `_from_rewrite(fuller, text, memory, reject, added='', domain=None)`
/// The entry a rewritten line reaches, when it is about the last subject and is not a repeat.
pub fn _from_rewrite(fuller: &str, text_: &str, memory: &Memory, reject: Reject, added: &str, domain: Option<&str>) -> Option<&'static Entry> {
    let item = knowledge::lookup::find(&understanding::understand(fuller, Some(memory)), None, reject, false)?;
    if let Some(domain) = domain {
        if item.domain != domain {
            return None;
        }
    }
    if !(_is_about(item, &memory::topics::subject(memory)) || (!added.is_empty() && _is_about(item, added))) {
        return None;
    }
    let own = _own_thing(&understanding::understand(text_, Some(memory)));
    if !own.is_empty() && !_is_about(item, &own) {
        return None;
    }
    match &item.reply {
        crate::knowledge::entry::Reply::Text(reply) => {
            if _said_before(reply, memory, 3) {
                return None;
            }
            Some(item)
        }
        _ => Some(item),
    }
}

/// Python: bot/modules/answers.py:6049 `_asks_storing(memory)`
/// True when the line before was a how-to-store (or how-to-keep-fresh) question.
pub fn _asks_storing(memory: &Memory) -> bool {
    let subject = memory::topics::subject(memory);
    text::split_ref(&subject).first().is_some_and(|w| STORING.contains(*w))
}

/// Python: bot/modules/answers.py:6060 `_keeping_thing(memory)`
/// The thing a keeping follow-up is about, without the storing word the how-to line asked with.
pub fn _keeping_thing(memory: &Memory) -> String {
    let subject = memory::topics::subject(memory);
    let mut words: Vec<&str> = text::split_ref(&subject);
    while words.first().is_some_and(|w| STORING.contains(*w) || STORING.contains(text::rstrip_chars(w, "s"))) {
        words.remove(0);
    }
    words.join(" ")
}

/// Python: bot/modules/answers.py:6068 `_the_plan_keeps(understood, memory)`
/// The keeping fact a dish the conversation planned says for itself: "it keeps 3 days".
pub fn _the_plan_keeps(understood: Option<&Understood>, memory: &Memory) -> Option<String> {
    let understood = understood?;
    let words: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if knowledge::dishes::ask_kind(understood) != Some("keep") || !["it", "that", "this", "them", "they"].iter().any(|w| words.contains(*w)) {
        return None;
    }
    if !_own_thing(understood).is_empty() {
        return None;
    }
    knowledge::dishes::follow_up(memory, understood)
}

/// Python: bot/modules/answers.py:6085 `_keeping_entry(text, memory, thing=None, storing=False)`
/// The entry that says how long the topic keeps, when the knowledge base has one.
pub fn _keeping_entry(text_: &str, memory: &mut Memory, thing: Option<&str>, storing: bool) -> Option<String> {
    let mut fuller = _with_subject(text_, memory, thing);
    if text::strip(&fuller) == text::strip(text_) {
        fuller = _about_subject(text_, memory, None);
    }
    if text::strip(&fuller) == text::strip(text_) {
        return None;
    }
    let reading = understanding::understand(&fuller, Some(&*memory));
    let item = find_plain(&reading)?;
    if !_is_about(item, &memory::topics::subject(memory)) {
        return None;
    }
    let reply = _reply_of(item, &reading, memory);
    if text::strip(&reply).is_empty() {
        return None;
    }
    let said_already = storing && _said_before(&reply, memory, 3);
    if !KEEPS.is_match(&reply) && !said_already {
        return None;
    }
    let subject = memory::topics::subject(memory);
    if !subject.is_empty() && knowledge::cooking::ingredients(&understanding::understand(&subject, Some(&*memory))).len() > 1 {
        return None;
    }
    if said_already {
        let figures: Vec<String> =
            SENTENCE.split(&reply).iter().filter(|part| FIGURE.is_match(part)).map(|part| text::strip(part).to_string()).collect();
        if !figures.is_empty() {
            return Some(figures.join(" "));
        }
    }
    Some(reply)
}

/// Python: bot/modules/answers.py:6140 `_in_this_conversation(reject, reading)`
/// The caller's veto, and one of its own: a fact from another part of the knowledge base.
pub fn _in_this_conversation<'a>(reject: Reject<'a>, reading: &'a Understood) -> impl Fn(&Entry) -> bool + 'a {
    move |item: &Entry| {
        if !knowledge::lookup::about_the_topic(item, reading) {
            return true;
        }
        reject.is_some_and(|r| r(item))
    }
}

/// Python: bot/modules/answers.py:6155 `_in_bare_context(text, memory, reject=None)`
/// The entry a bare ask reaches once the thing it is about is put on the end of it.
pub fn _in_bare_context(text_: &str, memory: &Memory, reject: Reject) -> Offered {
    if _names_its_own_thing(text_, Some(memory)) {
        return None;
    }
    let fuller = _with_bare_subject(text_, memory);
    if text::strip(&fuller) == text::strip(text_) {
        return None;
    }
    let reading = understanding::understand(&fuller, Some(memory));
    let veto = _in_this_conversation(reject, &reading);
    let item = knowledge::lookup::find(&reading, None, Some(&veto), false)?;
    if !_is_about(item, &memory::topics::subject(memory)) {
        return None;
    }
    if let crate::knowledge::entry::Reply::Text(reply) = &item.reply {
        if _said_before(reply, memory, 3) {
            return None;
        }
    }
    Some((item, Some(reading.clone())))
}

/// Python: bot/modules/answers.py:6198 `_figures_of(said)`
/// The numbers a sentence carries, as words a set can hold.
pub fn _figures_of(said: &str) -> HashSet<String> {
    NUMBER_IN_TEXT.findall_str(&memory::narrow::flat(said)).into_iter().collect()
}

/// Python: bot/modules/answers.py:6203 `_new_figure(fresh, before)`
/// Whether the answer has a figure the one before it did not.
pub fn _new_figure(fresh: &str, before: &str) -> bool {
    !minus(&_figures_of(fresh), &_figures_of(before)).is_empty()
}

/// Python: bot/modules/answers.py:6213 `_adds_a_figure(text)`
/// Whether the line brings a number for the thing under discussion, and asks nothing.
pub fn _adds_a_figure(text_: &str) -> bool {
    let said = memory::narrow::flat(text_);
    if said.is_empty() || said.contains('?') {
        return false;
    }
    A_FIGURE.is_match(&said)
}

/// Python: bot/modules/answers.py:6233 `_a_figure_put_right(text)`
/// True when the line moves a figure the chat already gave ("make it 150 instead").
pub fn _a_figure_put_right(text_: &str) -> bool {
    A_PUT_RIGHT.is_match(text_) && inline_re!(r"\d").is_match(text_)
}

/// Python: bot/modules/answers.py:6239 `_without_the_figures(line)`
/// The earlier line with its sums taken out, so only its words are handed on.
pub fn _without_the_figures(line: &str) -> String {
    text::squash(&inline_re!(r"[£$]?\b\d[\d,.]*\b").sub("", line))
}

/// Python: bot/modules/answers.py:6244 `_the_entry_asked_again(text, memory)`
/// The entry that answered last, asked this line with its own subject still in it.
pub fn _the_entry_asked_again(text_: &str, memory: &mut Memory) -> Option<String> {
    let record = _last_answer(memory)?;
    let entry_id = record.get("id").filter(|v| v.truthy()).map(py_str)?;
    let item = knowledge::entry::entries().iter().find(|c| c.id == entry_id)?;
    let turn = match record.get("turn") {
        Some(Value::Int(t)) => *t,
        _ => return None,
    };
    if turn < 0 || turn as usize >= memory.turns.len() {
        return None;
    }
    let line = text::strip(&memory.turns[turn as usize].user).to_string();
    if line.is_empty() {
        return None;
    }
    let earlier = if _a_figure_put_right(text_) { _without_the_figures(&line) } else { line };
    let reading = understanding::understand(&format!("{earlier} {text_}"), Some(&*memory));
    let fresh = _payload_mut(item, memory, &reading);
    if text::strip(&fresh).is_empty() {
        return None;
    }
    let said = record_str(&record, "reply");
    if memory::narrow::flat(&fresh) == memory::narrow::flat(&said) {
        return None;
    }
    if let Some(new_part) = _the_part_a_longer_variant_adds(&fresh, &said) {
        return Some(new_part);
    }
    Some(fresh)
}

/// Python: bot/modules/answers.py:6294 `_the_part_a_longer_variant_adds(fresh, said)`
/// What a longer variant of the answer on the screen adds to it, or None.
pub fn _the_part_a_longer_variant_adds(fresh: &str, said: &str) -> Option<String> {
    let said = memory::narrow::flat(said);
    let mine = memory::narrow::flat(fresh);
    if said.is_empty() || !mine.starts_with(&said) {
        return None;
    }
    let rest = &mine[said.len()..];
    if !rest.starts_with(' ') {
        return None;
    }
    let words = text::split_ref(&said).len();
    let added = text::split_ref(fresh).into_iter().skip(words).collect::<Vec<_>>().join(" ");
    let added = text::strip(&added).to_string();
    if text::split_ref(&added).len() < 3 {
        return None;
    }
    Some(added)
}

re_static!(STAND_IN_TARGET, PRICE_LIMIT, PRICE_NOT_OVER, PRICE_BARE, PRICE_OVER, CHEAPER, LIMIT_SHAPE, GIFT_WORD, HOW_TO);
set_static!(STAND_IN_WORDS, LIMIT_ONLY, PRICE_FILLER, PRICE_VERBS, LEAD_PREPOSITION, DETERMINER, NOT_A_THING, CLAUSE_START,
    ROOMS, NAMED_PLACES);
strs_static!(STAND_IN_PHRASES);
static PRICE_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    g("PRICE_WORDS").items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_int().unwrap_or(0))).collect()
});
static INTEREST_ALIASES: LazyLock<Vec<(String, String)>> =
    LazyLock::new(|| g("INTEREST_ALIASES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect());
static SAME_WHERE: LazyLock<String> = LazyLock::new(|| g("SAME_WHERE").str().to_string());
static SAME_PLAIN: LazyLock<String> = LazyLock::new(|| g("SAME_PLAIN").str().to_string());
static MOST_SAME: LazyLock<usize> = LazyLock::new(|| g("MOST_SAME").as_int().unwrap_or(220) as usize);
static SWAP_PREPOSITION: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.swap.PREPOSITION"));
static SWAP_STOP: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.swap.STOP"));
static INTEREST_GIFTS: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::get("bot.knowledge.shopping.INTEREST_GIFTS").pairs().iter().map(|(k, _)| k.str().to_string()).collect());

/// Python: bot/modules/answers.py:6329 `_the_sentence_owed(text, reply, memory)`
/// The sentence (the row's declared `lead`) the entry behind this reply owes the line, or None.
pub fn _the_sentence_owed(text_: &str, reply: &str, memory: &mut Memory) -> Option<Value> {
    let _ = text_;
    let understood = memory.understood.clone()?;
    let item = py_try(|| find_plain(&understood)).flatten()?;
    let lead = item.raw.attr("lead").cloned().unwrap_or(Value::None);
    if !lead.truthy() {
        return None;
    }
    let said = match &item.reply {
        crate::knowledge::entry::Reply::Text(s) => s.clone(),
        _ => py_try(|| _payload_mut(item, memory, &understood))?,
    };
    let plain = text::squash(&said);
    let here = text::squash(reply);
    if plain.is_empty() || here.is_empty() {
        return None;
    }
    if !here.starts_with(&text::slice(&plain, 0, Some(50))) {
        return None;
    }
    Some(lead)
}

/// Python: bot/modules/answers.py:6402 `_wants_a_stand_in(reading)`
/// True when the line asks what to use in place of the thing it says it has none of.
pub fn _wants_a_stand_in(reading: &Understood) -> bool {
    let clean = text::squash(&reading.clean).to_lowercase();
    if clean.is_empty() || STAND_IN_TARGET.is_match(&clean) {
        return false;
    }
    let mut asked = STAND_IN_PHRASES.iter().any(|p| clean.contains(p.as_str()));
    if !asked {
        // `reading.get("words")`: the reading always carries its words as a list
        asked = reading.words.iter().any(|w| STAND_IN_WORDS.contains(&w.to_lowercase()));
    }
    if !asked {
        return false;
    }
    (REFUSE_OPEN.match_(&clean).is_some() || REFUSE_WISH.match_(&clean).is_some()) && REFUSE_CHOICE.match_(&clean).is_none()
}

/// Python: bot/modules/answers.py:6428 `_a_swap_row(item)`
/// True for the table's stand-in rows, whose answer is the swapped ingredient by design.
pub fn _a_swap_row(item: &Entry) -> bool {
    item.id.starts_with("swap-")
}

/// Python: bot/modules/answers.py:6484 `_price_limit(text)`
/// (the amount in pounds or None, whether the line asked for something cheaper).
pub fn _price_limit(text_: &str) -> (Option<i64>, bool) {
    if text::strip(text_).is_empty() {
        return (None, false);
    }
    let flat = text::squash(&text_.to_lowercase());
    let cheaper = CHEAPER.is_match(&flat);
    if PRICE_OVER.is_match(&flat) && !PRICE_NOT_OVER.is_match(&flat) && !PRICE_LIMIT.is_match(&flat) {
        return (None, cheaper);
    }
    let found = PRICE_NOT_OVER.search(&flat).or_else(|| PRICE_LIMIT.search(&flat)).or_else(|| PRICE_BARE.search(&flat));
    if let Some(m) = found {
        return match m.str(1).parse::<f64>() {
            Ok(v) => (Some(v.trunc() as i64), cheaper),
            Err(_) => (py_int(m.str(1)), cheaper),
        };
    }
    for (phrase, amount) in PRICE_WORDS.iter() {
        if flat.contains(phrase.as_str()) {
            return (Some(*amount), cheaper);
        }
    }
    (None, cheaper)
}

/// Python: bot/modules/answers.py:6512 `_only_a_limit(text)`
/// True when the line is the limit and nothing else: "what about under 40 quid".
pub fn _only_a_limit(text_: &str) -> bool {
    let words: Vec<String> = words_in(&text::squash(&text_.to_lowercase()))
        .into_iter()
        .filter(|w| !crate::understanding::pyuni::is_digit(w))
        .collect();
    !words.is_empty() && words.iter().all(|w| LIMIT_ONLY.contains(w))
}

/// Python: bot/modules/answers.py:6519 `_a_gift_entry(reading)`
/// The present entry this line is about, if it is about one.
pub fn _a_gift_entry(reading: &Understood) -> Option<&'static Entry> {
    find_plain(reading).filter(|item| item.id.starts_with("gift-"))
}

/// Python: bot/modules/answers.py:6525 `_a_gift_just_answered(memory)`
/// The present entry one of the last few lines reached, while the chat is still on it.
pub fn _a_gift_just_answered(memory: &Memory) -> Option<&'static Entry> {
    for turn in last_turns(memory, 3).iter().rev() {
        if turn.user.is_empty() {
            continue;
        }
        if let Some(item) = _a_gift_entry(&understanding::understand(&turn.user, Some(memory))) {
            return Some(item);
        }
    }
    None
}

/// Python: bot/modules/answers.py:6540 `_asks_what_to_give(reading)`
/// True when the line is asking what to give somebody, with a person to give it to.
pub fn _asks_what_to_give(reading: &Understood) -> bool {
    crate::understanding::topics::gift_ask(reading)
}

/// Python: bot/modules/answers.py:6551 `_the_interests(text, memory, item)`
/// The hobbies this present is for, as the table names them: said in the chat, or in the key.
pub fn _the_interests(text_: &str, memory: &Memory, item: Option<&Entry>) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    let mut lines: Vec<String> = last_turns(memory, 3).iter().map(|t| t.user.clone()).collect();
    lines.push(text_.to_string());
    for line in &lines {
        for word in words_in(&line.to_lowercase()) {
            let name = lookup(&INTEREST_ALIASES, &word).map(str::to_string).unwrap_or(word);
            if INTEREST_GIFTS.contains(&name) && !said.contains(&name) {
                said.push(name);
            }
        }
    }
    if !said.is_empty() {
        return said;
    }
    if let Some(item) = item {
        for part in &item.key {
            for word in part.words() {
                let low = word.to_lowercase();
                let name = lookup(&INTEREST_ALIASES, &low).map(str::to_string).unwrap_or(low);
                if INTEREST_GIFTS.contains(&name) && !said.contains(&name) {
                    said.push(name);
                }
            }
        }
    }
    said
}

/// Python: bot/modules/answers.py:6580 `_the_entry_reply(item, memory, reading)`
/// What the entry says, without being said: the price it already names, or an empty string.
pub fn _the_entry_reply(item: &Entry, memory: &Memory, reading: &Understood) -> String {
    use crate::knowledge::entry::Reply;
    match &item.reply {
        Reply::Text(s) => s.clone(),
        // a pool is not a str, so it says nothing here
        Reply::Choices(_) => String::new(),
        Reply::Fn(f) => {
            if !knowledge::registry::has_reply(&f.id) {
                panic!("UNPORTED {}", f.id);
            }
            let mut scratch = memory.clone();
            match knowledge::registry::try_reply(f, &mut scratch, reading) {
                Ok(v) => v.unwrap_or_default(),
                Err(()) => {
                    reraise_unported();
                    String::new()
                }
            }
        }
    }
}

/// Python: bot/modules/answers.py:6590 `_the_present_with_a_price(text, reading, memory)`
/// A present at the price the line gives, or the limit on the present just suggested.
pub fn _the_present_with_a_price(text_: &str, reading: &Understood, memory: &Memory) -> Option<String> {
    let (money, cheaper) = _price_limit(text_);
    if money.is_none() && !cheaper {
        return None;
    }
    let mut item = _a_gift_entry(reading);
    if let Some(found) = item {
        if _the_entry_reply(found, memory, reading).contains('\u{a3}') {
            return None;
        }
    }
    let only_a_limit = _only_a_limit(text_);
    let interests = _the_interests(text_, memory, item);
    if item.is_none() {
        if only_a_limit {
            item = Some(_a_gift_just_answered(memory)?);
        } else if !GIFT_WORD.is_match(&text_.to_lowercase()) && !(!interests.is_empty() && _asks_what_to_give(reading)) {
            return None;
        }
    }
    if let Some(found) = item {
        if !only_a_limit && interests.is_empty() && found.id != "gift-generic" {
            return None;
        }
    }
    Some(match money {
        None => knowledge::shopping::present_less(&interests),
        Some(m) => knowledge::shopping::present_within(m as f64, &interests),
    })
}

/// Python: bot/modules/answers.py:6666 `_asks_a_price(text)`
/// Whether the line asks what something costs, rather than how much of a thing there is.
pub fn _asks_a_price(text_: &str) -> bool {
    let low = memory::narrow::flat(text_);
    let priced = inline_re!(
        r"\bhow much (?:does|do|is|are|will|would|should|can|could)\b.*\b(?:cost|pay|price|spend|budget)\b|\bwhat(?:'s| is) the price\b|\bhow much for\b"
    );
    if priced.is_match(&low) {
        return true;
    }
    let Some(found) = inline_re!(r"\bhow much\b(.*)$").search(&low) else { return false };
    for word in az_words(found.str(1)) {
        if PRICE_FILLER.contains(&word) {
            continue;
        }
        return PRICE_VERBS.contains(&word);
    }
    true
}

/// Python: bot/modules/answers.py:6687 `_names_what_it_wants_priced(text)`
/// Whether the price question says what it is for, rather than leaving it to the chat.
pub fn _names_what_it_wants_priced(text_: &str) -> bool {
    let low = memory::narrow::flat(text_);
    let Some(found) = inline_re!(r"\bhow much\b(.*)$").search(&low) else { return false };
    for word in az_words(found.str(1)) {
        if PRICE_FILLER.contains(&word) || PRICE_VERBS.contains(&word) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/modules/answers.py:6759 `_same_where_they_asked(text, said)`
/// The part of that answer the new thing has not been given yet, or "" for none left.
pub fn _same_where_they_asked(text_: &str, said: &str) -> String {
    let Some(thing) = memory::swap::thing(text_).py().filter(|t| !t.is_empty()) else { return String::new() };
    let thing_words = az_words(&thing.to_lowercase());
    if thing_words.iter().all(|w| NOT_A_THING.contains(w)) {
        return String::new();
    }
    if compose::fallback::reads_as_a_judgement(text_) {
        return String::new();
    }
    let clause = thing_words.first().is_some_and(|w| CLAUSE_START.contains(w));
    let mut words: Vec<String> = text::split(&thing);
    if words.len() > 1 && LEAD_PREPOSITION.contains(&words[0].to_lowercase()) {
        words.remove(0);
    }
    if !clause && !DETERMINER.contains(&words[0].to_lowercase()) && !(words.len() == 1 && words[0].ends_with("ing")) {
        words.insert(0, "the".into());
    }
    let thing = words.join(" ");
    let room = az_words(&thing.to_lowercase()).into_iter().find(|w| ROOMS.contains(text::rstrip_chars(w, "s"))).unwrap_or_default();
    let parts: Vec<String> = inline_re!(r"(?<=[.!?])\s+").split(said).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    let asked: HashSet<String> = az_words(&thing.to_lowercase()).iter().map(|w| bare_s(w).to_string()).collect();
    for part in &parts {
        for piece in _the_method_in(part) {
            if text::split_ref(&piece).len() < 5 || text::len(&piece) < 30 || text::len(&piece) > *MOST_SAME {
                continue;
            }
            if !_holds_where_they_asked(&piece, &asked) {
                continue;
            }
            let line = _mid_sentence(text::rstrip_chars(&piece, " ."));
            let mut line = _room_they_asked_about(&line, &asked, &room);
            if !line.ends_with(['.', '!', '?']) {
                line.push('.');
            }
            if clause {
                return py_format(&SAME_PLAIN, &[("said", &_small_letter(&line))]);
            }
            return py_format(&SAME_WHERE, &[("thing", &thing), ("said", &_small_letter(&line))]);
        }
    }
    String::new()
}

/// Python: bot/modules/answers.py:6816 `_mid_sentence(part)`
/// The line with its opening capital taken off, when it is only a capital because it began.
pub fn _mid_sentence(part: &str) -> String {
    let tokens = inline_re!(r"[A-Za-z']+").findall_str(part);
    if tokens.len() < 2 || !part.chars().next().is_some_and(crate::understanding::pyuni::ch_upper) {
        return part.to_string();
    }
    let first = tokens[0].to_lowercase();
    if tokens[1..].iter().any(|w| w.to_lowercase() == first) {
        return low1(part);
    }
    part.to_string()
}

/// Python: bot/modules/answers.py:6847 `_holds_where_they_asked(part, asked)`
/// Whether a sentence of the earlier answer is about the place asked about, or about none.
pub fn _holds_where_they_asked(part: &str, asked: &HashSet<String>) -> bool {
    let named: HashSet<String> =
        az_words(&part.to_lowercase()).iter().map(|w| bare_s(w).to_string()).filter(|w| NAMED_PLACES.contains(w)).collect();
    if named.is_empty() || !named.is_disjoint(asked) {
        return true;
    }
    named.iter().any(|w| ROOMS.contains(w)) && asked.iter().any(|w| ROOMS.contains(w))
}

/// Python: bot/modules/answers.py:6862 `_the_method_in(part)`
/// The sentence, and the part of it after a colon when it has one.
pub fn _the_method_in(part: &str) -> Vec<String> {
    let mut pieces = vec![part.to_string()];
    if let Some((_, after)) = part.split_once(": ") {
        pieces.push(after.to_string());
    }
    pieces
}

/// Python: bot/modules/answers.py:6876 `_room_they_asked_about(line, asked, room)`
/// The line with another room in it read as the one they asked about.
pub fn _room_they_asked_about(line: &str, asked: &HashSet<String>, room: &str) -> String {
    if room.is_empty() || asked.is_empty() {
        return line.to_string();
    }
    let found: HashSet<String> = az_words(&line.to_lowercase()).iter().map(|w| bare_s(w).to_string()).filter(|w| ROOMS.contains(w)).collect();
    let others: Vec<&String> = found.iter().filter(|w| !asked.contains(*w)).collect();
    if others.len() != 1 {
        return line.to_string();
    }
    let other = others[0];
    let name = bare_s(room).to_string();
    Re::new(&format!(r"\b{}(s?)\b", crate::py::re::escape(other)), RE_I).sub_fn(line, |m| format!("{name}{}", m.str(1)))
}

/// Python: bot/modules/answers.py:6898 `_a_place_of_the_same_kind(earlier, text)`
/// True when the place they now ask about is another of the kind the answer was about.
pub fn _a_place_of_the_same_kind(earlier: Option<&Value>, text_: &str) -> bool {
    let Some(earlier) = earlier.filter(|e| e.is_dict()) else { return false };
    let thing = memory::swap::thing(text_).py().unwrap_or_default();
    let asked: HashSet<String> = az_words(&thing.to_lowercase()).iter().map(|w| bare_s(w).to_string()).collect();
    let keys: HashSet<String> = earlier.get("keys").map(|k| k.items().iter().map(|w| py_str(w).to_lowercase()).collect()).unwrap_or_default();
    asked.iter().any(|w| ROOMS.contains(w)) && keys.iter().any(|w| ROOMS.contains(w))
}

/// Python: bot/modules/answers.py:6914 `_a_place_they_asked(text)`
/// True when the follow-up names a place with a place's own preposition.
pub fn _a_place_they_asked(text_: &str) -> bool {
    let thing = memory::swap::thing(text_).py().unwrap_or_default();
    text::split_ref(&thing).first().is_some_and(|w| SWAP_PREPOSITION.contains(&text::strip_chars(w, "?.!,;:").to_lowercase()))
}

/// Python: bot/modules/answers.py:6925 `_key_terms(item)`
/// Every phrase the entry is keyed on, however deeply its key is nested.
pub fn _key_terms(item: &Entry) -> Vec<String> {
    fn walk(node: &Value, out: &mut Vec<String>) {
        match node {
            Value::Str(s) => out.push(s.to_string()),
            Value::List(v) | Value::Tuple(v) | Value::Set(v) => v.iter().for_each(|c| walk(c, out)),
            Value::Dict(d) => d.iter().for_each(|(k, _)| walk(k, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    if let Some(key) = item.raw.attr("key").filter(|k| k.truthy()) {
        walk(key, &mut out);
    }
    out
}

/// Python: bot/modules/answers.py:6947 `_written_for(item, thing)`
/// True when one of the entry's own key words is the thing, or covers it.
pub fn _written_for(item: Option<&Entry>, thing: &str) -> bool {
    let Some(item) = item else { return false };
    let words: HashSet<String> = az_words(&thing.to_lowercase()).into_iter().collect();
    if words.is_empty() {
        return false;
    }
    _key_terms(item).iter().any(|term| {
        let own: HashSet<String> = az_words(&term.to_lowercase()).into_iter().collect();
        !own.is_empty() && (words.is_subset(&own) || own.is_subset(&words))
    })
}

/// Python: bot/modules/answers.py:6967 `_more_than_the_thing(item, rest)`
/// True when a key of the entry also names something in the rest of the question.
pub fn _more_than_the_thing(item: Option<&Entry>, rest: &str) -> bool {
    let Some(item) = item else { return false };
    let rest_words: HashSet<String> = az_words(&rest.to_lowercase()).into_iter().collect();
    if rest_words.is_empty() {
        return false;
    }
    _key_terms(item).iter().any(|term| {
        let own: HashSet<String> = az_words(&term.to_lowercase()).into_iter().collect();
        !own.is_empty() && own.is_subset(&rest_words) && !own.is_subset(&SWAP_STOP)
    })
}

/// Python: bot/modules/answers.py:6989 `_about_the_rest_of_the_question(item, question, new)`
/// True when the entry's own subject words are in the question besides the new thing.
pub fn _about_the_rest_of_the_question(item: &Entry, question: &str, new: &str) -> bool {
    let key = item.raw.attr("key").cloned().unwrap_or(Value::None);
    let subject = key.items().first().cloned().unwrap_or(Value::None);
    if !subject.truthy() {
        return false;
    }
    let new_words = az_words(&new.to_lowercase());
    if new_words.first().is_none_or(|w| !SWAP_PREPOSITION.contains(w)) {
        return false;
    }
    let asked: HashSet<String> = az_words(&question.to_lowercase()).into_iter().collect();
    let left: HashSet<&String> = asked.iter().filter(|w| !new_words.contains(*w)).collect();
    // `for term in subject`: a tuple's phrases, or a str's characters
    let terms: Vec<String> = match &subject {
        Value::Str(s) => s.chars().map(|c| c.to_string()).collect(),
        other => other.items().iter().map(py_str).collect(),
    };
    terms.iter().any(|term| {
        let words = az_words(&term.to_lowercase());
        !words.is_empty() && words.iter().all(|w| left.contains(w))
    })
}

re_static!(KEEPING_CLAUSE, KEEPS_A_FIGURE, DEGREES, REHEAT_ASK, KEEPING_WORD);
set_static!(COOKED_THING);
static REHEAT_REPLY: LazyLock<String> = LazyLock::new(|| g("REHEAT_REPLY").str().to_string());
static KEEPING_FROM_THE_ANSWER: LazyLock<String> = LazyLock::new(|| g("KEEPING_FROM_THE_ANSWER").str().to_string());
static AGAIN_HELP: LazyLock<String> = LazyLock::new(|| g("AGAIN_HELP").str().to_string());

/// `lookup.entry(id, key, reply)` for an answer built at run time. The reader hands entries out
/// as `&'static`, so each one built is kept (once per id, key and reply).
fn built_entry(id: &str, key: &[&str], reply: &str) -> &'static Entry {
    use std::sync::Mutex;
    static BUILT: LazyLock<Mutex<std::collections::HashMap<String, &'static Entry>>> = LazyLock::new(Default::default);
    let tag = format!("{id}\u{0}{}\u{0}{reply}", key.join("\u{1}"));
    let mut built = BUILT.lock().unwrap();
    if let Some(e) = built.get(&tag) {
        return e;
    }
    let parts = key.iter().map(|w| crate::knowledge::entry::KeyPart::Word(w.to_string())).collect();
    let made = knowledge::lookup::entry(id, parts, crate::knowledge::entry::Reply::Text(reply.to_string()), &[], 0, vec![], None);
    let leaked: &'static Entry = Box::leak(Box::new(made));
    built.insert(tag, leaked);
    leaked
}

/// `mem.last_module(memory)`
fn last_module(memory: &Memory) -> Option<String> {
    memory::last_module(memory).py()
}

/// Python: bot/modules/answers.py:7124 `_keeping_in_the_answer_before(memory)`
/// The sentence of the answer just given that says how long the thing in hand keeps, or "".
pub fn _keeping_in_the_answer_before(memory: &Memory) -> String {
    let last = memory::last_bot(memory);
    let said = text::strip(&last);
    if said.is_empty() {
        return String::new();
    }
    for sentence in SENTENCE.split(said) {
        let mut sentence = text::strip(&sentence).to_string();
        if !(knowledge::cooking::speaks_of_keeping(&sentence) && KEEPS_A_FIGURE.is_match(&sentence)) {
            continue;
        }
        if let Some(keep_at) = KEEPING_CLAUSE.search(&sentence) {
            if keep_at.start() > 0 {
                let lead = text::rstrip_chars(&sentence[..keep_at.start()], " ,;:-");
                if !lead.is_empty() && (lead.contains(',') || lead.to_lowercase().contains(" and")) {
                    let rest = sentence[keep_at.start()..].to_string();
                    sentence = up1(&rest);
                }
            }
        }
        if text::strip(&sentence) == said {
            continue;
        }
        return text::strip(&sentence).to_string();
    }
    String::new()
}

/// Python: bot/modules/answers.py:7167 `_keeping_reader(reading, memory)`
/// The reader for a line that asks how long something keeps rather than how long it cooks.
pub fn _keeping_reader(reading: &Understood, memory: &Memory) -> Box<Reader<'static>> {
    let reading = reading.clone();
    if !knowledge::cooking::keeping_ask(&reading) {
        return Box::new(move |_: &mut Memory, reject: Reject| reader_entry(knowledge::lookup::find(&reading, None, reject, false)));
    }
    let _ = memory;
    let own = _own_thing(&reading);
    // `keeps(of, reject)`: the entry for `of` whose answer speaks of keeping
    fn keeps(of: &Understood, reject: Reject, memory: &mut Memory) -> Option<&'static Entry> {
        let cell = std::cell::RefCell::new(memory);
        let not_about_keeping = |one: &Entry| {
            if reject.is_some_and(|r| r(one)) {
                return true;
            }
            let said = _payload_mut(one, &mut cell.borrow_mut(), of);
            !knowledge::cooking::speaks_of_keeping(&said)
        };
        knowledge::lookup::find(of, None, Some(&not_about_keeping), false)
    }
    fn in_the_fridge(of: &Understood) -> Understood {
        let mut cold = of.clone();
        let base = if !of.clean.is_empty() { of.clean.clone() } else { of.text.clone() };
        cold.clean = format!("{base} in the fridge");
        cold.words.extend(["in", "the", "fridge"].iter().map(|w| w.to_string()));
        cold
    }
    fn with_the_thing(of: &Understood, memory: &mut Memory) -> Option<Understood> {
        let said = if !of.text.is_empty() { of.text.clone() } else { of.clean.clone() };
        let fuller = _with_subject(&said, memory, None);
        if text::strip(&fuller).is_empty() || text::strip(&fuller) == text::strip(&said) {
            return None;
        }
        Some(understanding::understand(&fuller, Some(&*memory)))
    }
    Box::new(move |memory: &mut Memory, reject: Reject| {
        let mut found = keeps(&reading, reject, memory);
        if found.is_none() {
            if let Some(thing) = with_the_thing(&reading, memory) {
                found = keeps(&thing, reject, memory);
            }
        }
        if found.is_none() && own.is_empty() && last_module(memory).as_deref() == Some("answers") {
            let kept = _keeping_in_the_answer_before(memory);
            if !kept.is_empty() {
                return Some((built_entry(&KEEPING_FROM_THE_ANSWER, &["keep", "keeps", "last", "lasts"], &kept), None));
            }
        }
        if found.is_none() && last_module(memory).as_deref() == Some("answers") {
            let mut cold = keeps(&in_the_fridge(&reading), reject, memory);
            if let Some(c) = cold {
                if !own.is_empty() && !_is_about(c, &own) {
                    cold = None;
                }
            }
            found = cold;
        }
        reader_entry(found.or_else(|| knowledge::lookup::find(&reading, None, reject, false)))
    })
}

/// Python: bot/modules/answers.py:7251 `_cooked_thing(memory)`
/// True when the thing the conversation is on has been cooked already.
pub fn _cooked_thing(memory: &Memory) -> bool {
    text::split_ref(&memory::topics::subject(memory)).iter().any(|w| COOKED_THING.contains(*w))
}

/// Python: bot/modules/answers.py:7256 `_the_plan_for(text, memory, understood)`
/// The dish plan's answer to a part question the last answer did not state, or None.
pub fn _the_plan_for(text_: &str, memory: &Memory, understood: &Understood) -> Option<String> {
    let _ = text_;
    if !_the_plan_answers_this(memory) {
        return None;
    }
    knowledge::dishes::follow_up(memory, understood).filter(|a| !text::strip(a).is_empty())
}

/// Python: bot/modules/answers.py:7297 `_names_another_food(understood, subject)`
/// True when the line names a thing of its own that is not the food we are on.
pub fn _names_another_food(understood: &Understood, subject: &str) -> bool {
    if subject.is_empty() {
        return false;
    }
    let words: HashSet<&str> = text::split_ref(subject).into_iter().collect();
    for ent in &understood.entities {
        if ent.kind == "person" {
            continue;
        }
        let said = text::strip(&ent.text.to_lowercase()).to_string();
        if said.is_empty() {
            continue;
        }
        if PRONOUN_OBJECTS.contains(&said) || COOK_VERB.is_match(&said) {
            continue;
        }
        if text::split_ref(&said).iter().any(|w| words.contains(*w)) {
            return false;
        }
        return true;
    }
    false
}

/// Python: bot/modules/answers.py:7334 `_the_food_named(subject)`
/// The subject without the keeping words in it: "cooked rice keep" -> "cooked rice".
pub fn _the_food_named(subject: &str) -> String {
    text::split_ref(subject).into_iter().filter(|w| KEEPING_WORD.match_(w).is_none()).collect::<Vec<_>>().join(" ")
}

/// Python: bot/modules/answers.py:7340 `_a_cooking_ask_after_a_keeping_answer(understood, memory)`
/// The method entry's answer to a "how long do i cook it for" after a keeping answer.
pub fn _a_cooking_ask_after_a_keeping_answer(understood: &Understood, memory: &mut Memory) -> Option<String> {
    let line = if !understood.clean.is_empty() { understood.clean.clone() } else { understood.text.clone() };
    if line.is_empty() || !COOK_VERB.is_match(&line) || KEEPS_ASK.is_match(&line) {
        return None;
    }
    let in_hand = memory::topics::answer_in_hand(memory);
    let said = in_hand.get("bot").map(py_str_or_empty).unwrap_or_default();
    if said.is_empty() || !KEEPS_ANSWER.is_match(&said) {
        return None;
    }
    let subject = memory::topics::subject(memory).to_lowercase();
    if _names_another_food(understood, &subject) {
        return None;
    }
    if REHEAT_ASK.is_match(&line) {
        return Some(REHEAT_REPLY.clone());
    }
    let food = _the_food_named(&memory::topics::subject(memory));
    let food = if food.is_empty() { None } else { Some(food) };
    let mut fuller = _with_subject(&line, memory, food.as_deref());
    if text::strip(&fuller) == text::strip(&line) {
        fuller = _about_subject(&line, memory, food.as_deref());
    }
    let readings: Vec<String> = if text::strip(&fuller) != text::strip(&line) { vec![fuller, line.clone()] } else { vec![line.clone()] };
    let cooked = inline_re!(r"\b(cooked|leftover|left over|roast|roasted)\s+(?=[\w' ]+\bfor\b)");
    let readings: Vec<String> = readings.iter().map(|r| cooked.subn("", r, 1)).collect();
    let mut fallback: Option<String> = None;
    for written in readings {
        let reading = understanding::understand(&written, Some(&*memory));
        let (item, answer) = _offer_with(&mut |_: &mut Memory, reject: Reject| reader_entry(knowledge::lookup::find(&reading, None, reject, false)), memory, &reading);
        let Some(answer) = answer.filter(|a| item.is_some() && !text::strip(a).is_empty()) else { continue };
        if text::strip(&answer) == text::strip(&said) {
            continue;
        }
        if !_minutes_in(&answer).is_empty() {
            return Some(answer);
        }
        if fallback.is_none() {
            fallback = Some(answer);
        }
    }
    fallback
}

/// Python: bot/modules/answers.py:7405 `_the_plan_answers_this(memory)`
/// True when the last answer was about cooking the dish, so its plan is the answer here.
pub fn _the_plan_answers_this(memory: &Memory) -> bool {
    let said = memory::last_bot(memory);
    let low = said.to_lowercase();
    if low.contains("air fryer") || low.contains("airfryer") {
        return false;
    }
    DEGREES.is_match(&said) || memory::topics::has_time(&said)
}

/// Python: bot/modules/answers.py:7421 `_the_plan_said_again(understood, text, memory)`
/// The same property asked again about the plan we just gave, said as a repeat.
pub fn _the_plan_said_again(understood: &Understood, text_: &str, memory: &Memory) -> Option<String> {
    if !(memory::topics::asks_only(understood) && last_module(memory).as_deref() == Some("answers")) {
        return None;
    }
    if !_the_plan_answers_this(memory) {
        return None;
    }
    let plan = knowledge::dishes::follow_up(memory, understood).filter(|p| !text::strip(p).is_empty())?;
    let said = if memory::topics::pairing(text_) { 0 } else { _answer_said(&plan, memory) };
    if said >= 3 {
        return Some(py_format(&AGAIN_HELP, &[("reply", &plan)]));
    }
    if said > 0 {
        return Some(py_format(&AGAIN[(said - 1) as usize], &[("reply", &_repeat_short(&plan, text_, Some(memory)))]));
    }
    None
}

/// Python: bot/modules/answers.py:7454 `_filling_in_an_emergency(understood, memory)`
/// True when the line only adds detail to a safety reply the bot has just given.
pub fn _filling_in_an_emergency(understood: Option<&Understood>, memory: &Memory) -> bool {
    // `mem.recent(memory, 1)`: the last turn
    let Some(last) = memory.turns.last() else { return false };
    if last.module.as_deref() != Some("safety") {
        return false;
    }
    let Some(understood) = understood else { return true };
    !(understood.kind == "question" || understood.signals.question_mark)
}

/// Python: bot/modules/answers.py:7469 `_usable(item, reading, memory)`
/// False when the answer needs something the line says the kitchen has not got.
pub fn _usable(item: &Entry, reading: &Understood, memory: &Memory) -> bool {
    if item.fault {
        return true;
    }
    !knowledge::cooking::unusable(&_reply_of(item, reading, memory), reading)
}

/// Python: bot/modules/answers.py:7479 `_next_usable(item, reading, memory)`
/// The best entry that answers the same line without what the line has not got.
pub fn _next_usable(item: &Entry, reading: &Understood, memory: &Memory) -> Option<&'static Entry> {
    let kept: Vec<&'static Entry> =
        knowledge::entry::entries().iter().filter(|one| !std::ptr::eq(*one, item) && _usable(one, reading, memory)).collect();
    if kept.is_empty() {
        return None;
    }
    knowledge::lookup::find(reading, Some(&kept), None, false)
}

re_static!(CONTINUED, THEIR_EAT, COOK_WORD, SAID_WORD, EAT_VERB, MEAT_FREE, A_TASTE_OF_THEIRS, COMPLAINT, DETAIL_LINE, THEIR_OWN_ASK);
set_static!(TONING_WORDS, JOINING_WORDS, TONE_DESCRIPTORS, DIET_WORDS, NOT_THE_THING, WRITE_OBJECT, HEALTH_DOMAINS, LENGTH_SAID, _AN_ANSWER);
strs_static!(REWORD_ASKS, CHECK_ASKS, DETAIL_FRAMES);
static TONE_WORDS: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| g("TONE_WORDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());
static DEMANDS: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    g("DEMANDS").items().iter().map(|p| (p.items()[0].as_re().expect("DEMANDS holds regexes"), p.items()[1].str().to_string())).collect()
});
static HARD_TO_SAY: LazyLock<Vec<(&'static Re, String, String)>> = LazyLock::new(|| {
    g("HARD_TO_SAY")
        .items()
        .iter()
        .map(|p| {
            let it = p.items();
            (it[0].as_re().expect("HARD_TO_SAY holds regexes"), it[1].str().to_string(), it[2].str().to_string())
        })
        .collect()
});
static CIRCUMSTANCE: LazyLock<Vec<HashSet<String>>> =
    LazyLock::new(|| g("CIRCUMSTANCE").items().iter().map(|f| f.str_set()).collect());
static RELATIONS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.facts.RELATIONS"));
static MEANT_LOOK_BACK_FAR: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.meant.LOOK_BACK_FAR").as_int().unwrap_or(7) as usize);
static MEANT_NOT_TO_REPEAT: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.meant.NOT_TO_REPEAT"));
static MEANT_CONDOLENCE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.meant.CONDOLENCE"));
static MEANT_RECEIPT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.meant.RECEIPT"));
static REPEAT_HEDGE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.HEDGE"));

/// Python: bot/modules/answers.py:7485 `_without_names(reading)`
/// The same reading with a place word the line uses as a name taken out.
pub fn _without_names(reading: &Understood) -> Understood {
    if reading.person_names.is_empty() {
        return reading.clone();
    }
    let drop: HashSet<&str> = reading.person_names.iter().map(String::as_str).collect();
    let mut narrowed = reading.clone();
    narrowed.words = reading.words.iter().filter(|w| !drop.contains(w.to_lowercase().as_str())).cloned().collect();
    if !reading.clean.is_empty() {
        narrowed.clean = text::split_ref(&reading.clean)
            .into_iter()
            .filter(|w| !drop.contains(text::strip_chars(&w.to_lowercase(), ".,!?;:")))
            .collect::<Vec<_>>()
            .join(" ");
    }
    narrowed
}

/// Python: bot/modules/answers.py:7570 `_only_the_tone(rest)`
/// True when what follows the instruction is the tone again, not text to rewrite.
pub fn _only_the_tone(rest: &str) -> bool {
    let low = rest.to_lowercase();
    let words: Vec<String> = text::split_ref(&low)
        .iter()
        .map(|w| text::strip_chars(w, ".,!?;:'\"").to_string())
        .filter(|w| !w.is_empty() && !JOINING_WORDS.contains(w))
        .collect();
    if words.is_empty() {
        return true;
    }
    for word in &words {
        if TONE_DESCRIPTORS.contains(word) || CONTINUED.is_match(word) {
            continue;
        }
        if TONING_WORDS.iter().any(|stem| word.contains(stem.as_str())) {
            continue;
        }
        return false;
    }
    true
}

/// Python: bot/modules/answers.py:7604 `_reword_ask(text)`
/// The text the line wants rewritten, or None when the line is not a rewrite request.
pub fn _reword_ask(text_: &str) -> Option<String> {
    let flat = _flat(text_);
    let lowered = text::squash(&text_.to_lowercase());
    let asked = REWORD_ASKS.iter().any(|p| flat.contains(p.as_str()));
    let tone = TONE_WORDS.iter().any(|(_, words)| words.iter().any(|w| flat.contains(w.as_str())));
    if !(asked && (tone || lowered.contains(':'))) {
        return None;
    }
    let (head, sep, tail) = text::rpartition(&lowered, ":");
    if !sep.is_empty() && text::split_ref(tail).len() >= 2 && REWORD_ASKS.iter().any(|p| _flat(head).contains(p.as_str())) {
        return Some(text::strip_chars(tail, " .\"'").to_string());
    }
    let (mut at, mut found) = (text::len(&flat) as isize + 1, String::new());
    for phrase in REWORD_ASKS.iter() {
        let where_ = text::find(&flat, phrase);
        if where_ >= 0 && (where_ < at || (where_ == at && text::len(phrase) > text::len(&found))) {
            at = where_;
            found = phrase.clone();
        }
    }
    let mut rest = if !found.is_empty() { text::slice(&flat, at + text::len(&found) as isize, None) } else { flat.clone() };
    rest = inline_re!(r"\bso (it|that) (sounds?|reads?) (more |less )?.*$").sub("", &rest);
    rest = inline_re!(r"\bso (it|that) does ?n[o']?t sound .*$").sub("", &rest);
    rest = inline_re!(
        r"^\s*(a bit |abit |a little |alittle )?(sound|sounds|read|look)?\s*(less|more|softer|warmer|friendlier|nicer|politer|politelier|formal|professional|casual|shorter|briefer|blunt|harsh|rude|angry|aggressive|gentle|tactful|diplomatic|kind)\b\s*(angry|harsh|blunt|rude|aggressive|formal|polite)?"
    )
    .sub("", &rest);
    for _ in 0..3 {
        rest = inline_re!(r"^\s*(a bit|a little|please|thanks|thank you|for me|mate)\b[ ,]*").sub("", &rest);
    }
    let rest = text::strip_chars(&rest, " .,:;\"'").to_string();
    if text::split_ref(&rest).len() < 3 || _only_the_tone(&rest) {
        return Some(String::new());
    }
    Some(rest)
}

/// Python: bot/modules/answers.py:7647 `_tone_of(text)`
pub fn _tone_of(text_: &str) -> HashSet<String> {
    let flat = _flat(text_);
    let mut tones = HashSet::new();
    for (tone, words) in TONE_WORDS.iter() {
        if words.iter().any(|w| flat.contains(w.as_str())) {
            tones.insert(tone.clone());
        }
    }
    if flat.contains("less formal") {
        tones.insert("casual".to_string());
    }
    if CHECK_ASKS.iter().any(|p| flat.contains(p.as_str())) && tones.is_empty() {
        tones.insert("check".to_string());
    }
    tones
}

/// Python: bot/modules/answers.py:7660 `_softened(sentence, tones)`
/// The sentence with its demands asked instead, no shout in it, and the tone asked for.
pub fn _softened(sentence: &str, tones: &HashSet<String>) -> String {
    let mut out = text::strip(sentence).to_string();
    for (pattern, better) in DEMANDS.iter() {
        out = pattern.sub(better, &out);
    }
    out = out.replace('!', ".");
    out = inline_re!(r"\bnow\b(?=\s*(?:[.,;!?]|$|or\b|and\b))").sub("", &out);
    out = text::strip_chars(&inline_re!(r"\s+").sub(" ", &out), " ,;").to_string();
    out = inline_re!(r"\bi\b").sub("I", &out);
    for (bare, proper) in [
        ("cant", "can't"),
        ("dont", "don't"),
        ("wont", "won't"),
        ("doesnt", "doesn't"),
        ("didnt", "didn't"),
        ("couldnt", "couldn't"),
        ("wouldnt", "wouldn't"),
        ("isnt", "isn't"),
        ("im", "I'm"),
        ("ive", "I've"),
        ("ill", "I'll"),
    ] {
        out = Re::new(&format!(r"\b{bare}\b"), 0).sub(proper, &out);
    }
    if out.chars().next().is_some_and(crate::understanding::pyuni::ch_lower) {
        out = up1(&out);
    }
    if tones.contains("formal") {
        for (short, long) in [
            ("I'm", "I am"),
            ("I've", "I have"),
            ("I'd", "I would"),
            ("I'll", "I will"),
            ("don't", "do not"),
            ("doesn't", "does not"),
            ("can't", "cannot"),
            ("won't", "will not"),
            ("it's", "it is"),
            ("couldn't", "could not"),
            ("wouldn't", "would not"),
        ] {
            out = Re::new(&crate::py::re::escape(short), 0).sub(long, &out);
            out = out.replace(&up1(short), &up1(long));
        }
    }
    if tones.contains("casual") {
        for (long, short) in [("I would", "I'd"), ("I am", "I'm"), ("do not", "don't"), ("cannot", "can't"), ("will not", "won't")] {
            out = out.replace(long, short);
        }
    }
    for name in [
        "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday", "January", "February", "March", "April", "May",
        "June", "July", "August", "September", "October", "November", "December",
    ] {
        out = Re::new(&format!(r"\b{}\b", name.to_lowercase()), 0).sub(name, &out);
    }
    out = inline_re!(r"\b(Hi|Hello|Dear|Hey|Thanks|Thank you|Morning|Afternoon)\s+([a-z])")
        .sub_fn(&out, |m| format!("{} {}", m.str(1), m.str(2).to_uppercase()));
    if !out.is_empty() && !out.ends_with(['.', '?', '!']) {
        out.push('.');
    }
    out
}

/// Python: bot/modules/answers.py:7714 `_last_rewrite(memory)`
/// The text and tones of the rewrite before this turn, or None.
pub fn _last_rewrite(memory: &Memory) -> Option<(String, HashSet<String>)> {
    for turn in last_turns(memory, 4).iter().rev() {
        if let Some(asked) = _reword_ask(&turn.user).filter(|a| !a.is_empty()) {
            return Some((asked, _tone_of(&turn.user)));
        }
    }
    None
}

/// Python: bot/modules/answers.py:7747 `_hard_to_say(asked)`
/// A known hard sentence, said properly, or None.
pub fn _hard_to_say(asked: &str) -> Option<(String, String)> {
    let flat = text::squash(&asked.to_lowercase());
    HARD_TO_SAY.iter().find(|(pattern, _, _)| pattern.is_match(&flat)).map(|(_, s, t)| (s.clone(), t.clone()))
}

/// Python: bot/modules/answers.py:7756 `_rewritten(asked, tones, from_followup=False)`
/// The rewrite, phrased as the answer: the sentence, then what to leave in or out.
pub fn _rewritten(asked: &str, tones: &HashSet<String>, from_followup: bool) -> String {
    let mut body = _softened(asked, tones);
    let known = if from_followup { None } else { _hard_to_say(asked) };
    if let Some((sentence, tail)) = known {
        if tones.contains("warm") || tones.contains("short") {
            return format!("Softer, same point: \"{sentence}\" {tail}");
        }
    }
    if tones.len() == 1 && tones.contains("check") {
        return format!(
            "Reads all right: \"{body}\" Nothing needs fixing: it is short, it says what you want, and it leaves them an easy way to say yes. Add the date you need an answer by if there is one."
        );
    }
    let polite = tones.contains("warm") || tones.contains("formal");
    let mut asking = inline_re!(
        r"\b(want|need|give|send|pay|refund|back|fix|sort|complain|explain|ask|book|change|cancel|return|replace|stop|reply|respond|could|would|can)\b"
    )
    .is_match(&body.to_lowercase());
    if inline_re!(r"\bneed to (tell|say|let)\b").is_match(&body.to_lowercase()) {
        asking = false;
    }
    if polite && asking && !body.to_lowercase().contains("please") && !body.ends_with('?') {
        body = match body.strip_suffix('.') {
            Some(head) => format!("{head}, please."),
            None => format!("{body}?"),
        };
    }
    if tones.contains("short") {
        body = inline_re!(r"\b(just|really|actually|basically|obviously|literally)\b ?").sub("", &body);
        body = text::strip(&inline_re!(r"\s+").sub(" ", &body)).to_string();
    }
    let mut tail = "Ask for what you want and a date, and leave out the anger: a short, specific message gets answered more often than a hot one.";
    if !asking {
        tail = "Say it plainly, name the date, and stop there: the calmer and shorter it is, the better it lands.";
    }
    if tones.contains("formal") {
        tail = "Formal but human works best: the request, the date, and one line of reason, with no apology for asking.";
    } else if tones.contains("casual") {
        tail = "Keep it short and say it as you would out loud; the subject still has to be clear.";
    } else if tones.contains("short") {
        tail = "Cut everything that is not the request: the ask and the date are the whole message.";
    }
    format!("Softer, same point: \"{body}\" {tail}")
}

/// Python: bot/modules/answers.py:7845 `_asks_about_a_food(low)`
/// Whether a question about someone puts a thing to eat after the verb ("can he eat cake").
pub fn _asks_about_a_food(low: &str) -> bool {
    let Some(found) = EAT_VERB.search(low) else { return false };
    let words: Vec<String> = text::split_ref(&low[found.end()..])
        .iter()
        .map(|w| text::strip_chars(w, ".,!?;:'\"").to_string())
        .filter(|w| !w.is_empty() && !NOT_THE_THING.contains(w))
        .collect();
    match words.first() {
        None => false,
        Some(w) => !DIET_WORDS.contains(w),
    }
}

/// Python: bot/modules/answers.py:7857 `_someones_else_diet(text, memory, understood)`
/// True when the line is about what someone other than the person eats.
pub fn _someones_else_diet(text_: &str, memory: &Memory, understood: Option<&Understood>) -> bool {
    let low = text::squash(&text_.to_lowercase());
    if low.is_empty() || COOK_WORD.is_match(&low) {
        return false;
    }
    let Some(understood) = understood.filter(|_| THEIR_EAT.is_match(&low)) else { return false };
    if !_names_someone_else(&low) {
        return false;
    }
    if !memory.get("others").is_some_and(Value::truthy) {
        return false;
    }
    if understood.kind != "question" && _a_guest_restriction(text_, memory, understood) {
        return false;
    }
    if understood.kind == "question" {
        if SAID_WORD.is_match(&low) {
            return true;
        }
        if _asks_about_a_food(&low) {
            return false;
        }
        return !_about_them_alone(&low);
    }
    if MEAT_FREE.is_match(&low) {
        return false;
    }
    true
}

/// Python: bot/modules/answers.py:7895 `_a_guest_restriction(text, memory, understood)`
/// Whether the kitchen has a cooking answer for the diet this line names for someone else.
pub fn _a_guest_restriction(text_: &str, memory: &Memory, understood: &Understood) -> bool {
    let _ = text_;
    knowledge::dishes::has_a_diet_reply(memory, understood) || knowledge::dishes::has_a_veggie_reply(memory, understood)
}

/// Python: bot/modules/answers.py:7921 `_names_someone_else(low)`
/// Whether the line names or points at a person other than the speaker.
pub fn _names_someone_else(low: &str) -> bool {
    for relation in RELATIONS.iter() {
        if Re::new(&format!(r"\b{relation}\b"), 0).is_match(low) {
            return true;
        }
    }
    inline_re!(r"\b(?:he|she|they|him|her|them|his|their)\b").is_match(low)
}

/// Python: bot/modules/answers.py:7939 `_a_writing_job(understood)`
/// True when the line asks for the words by naming the thing to write itself.
pub fn _a_writing_job(understood: Option<&Understood>) -> bool {
    let Some(understood) = understood else { return false };
    let norm = &understood.norm;
    for (at, word) in norm.iter().enumerate() {
        if !matches!(word.as_str(), "thank" | "thanks" | "thankyou" | "thank you" | "thx" | "ta") {
            continue;
        }
        let rest: Vec<&String> = norm[at + 1..].iter().filter(|w| *w != "you").collect();
        if rest.first().is_some_and(|w| WRITE_OBJECT.contains(*w)) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:7963 `_asks_about_their_taste(text)`
/// Whether the line asks what somebody else likes or hates.
pub fn _asks_about_their_taste(text_: &str) -> bool {
    A_TASTE_OF_THEIRS.is_match(&text::squash(&text_.to_lowercase()))
}

/// Python: bot/modules/answers.py:7968 `_about_them_alone(low)`
/// "what can she eat" asks for the person; "my mum" is the guest, not the speaker.
pub fn _about_them_alone(low: &str) -> bool {
    let theirs = RELATIONS.join("|");
    if Re::new(&format!(r"\b(?:my|our)\s+(?:{theirs})\b"), 0).is_match(low) {
        return false;
    }
    inline_re!(r"\b(?:i|im|i'm|me|my|we|us|our)\b").is_match(low)
}

/// Python: bot/modules/answers.py:7982 `_health_entries()`
/// The answers about a person's own body, read once.
pub fn _health_entries() -> &'static [&'static Entry] {
    static HEALTH: LazyLock<Vec<&'static Entry>> =
        LazyLock::new(|| knowledge::entry::entries().iter().filter(|one| HEALTH_DOMAINS.contains(&one.domain)).collect());
    &HEALTH
}

/// Python: bot/modules/answers.py:7990 `_names_a_typed_word(item, text)`
/// True when the entry is about a word the person actually typed.
pub fn _names_a_typed_word(item: &Entry, text_: &str) -> bool {
    let keys = _key_words(item);
    text::split_ref(&_flat(text_)).iter().any(|w| keys.contains(&text::strip_chars(w, ".,!?;:").to_lowercase()))
}

/// Python: bot/modules/answers.py:8018 `_circumstances(text)`
/// The circumstance family of every circumstance word in a line, and nothing else.
pub fn _circumstances(text_: &str) -> HashSet<String> {
    let words: HashSet<String> = az_words(&text_.to_lowercase()).iter().map(|w| text::strip_chars(w, "'").to_string()).collect();
    let mut found = HashSet::new();
    for family in CIRCUMSTANCE.iter() {
        if !words.is_disjoint(family) {
            found.extend(family.iter().cloned());
        }
    }
    found
}

/// Python: bot/modules/answers.py:8041 `_detail_frame(part)`
pub fn _detail_frame(part: &str) -> String {
    py_format(&DETAIL_FRAMES[text::len(part) % DETAIL_FRAMES.len()], &[("part", part)])
}

/// Python: bot/modules/answers.py:8045 `_a_figure_the_answer_has(text, memory)`
/// The sentence of the last answer that carries a figure this line names, or None.
pub fn _a_figure_the_answer_has(text_: &str, memory: &Memory) -> Option<String> {
    py_try(|| memory::detail::the_fact_the_answer_has(text_, memory).py()).flatten()
}

/// Python: bot/modules/answers.py:8054 `_a_detail_answered(text, memory)`
/// The sentence of the last answer that speaks to this detail, or None.
pub fn _a_detail_answered(text_: &str, memory: &Memory) -> Option<String> {
    if !DETAIL_LINE.is_match(text_) {
        return None;
    }
    let reply = memory::last_bot(memory);
    if reply.is_empty() {
        return None;
    }
    let want = _circumstances(text_);
    let mut known_first: Option<String> = None;
    let said: HashSet<String> =
        memory::topics::keywords(text_).iter().filter(|w| text::len(w) > 2).map(|w| memory::topics::stem(w)).collect();
    let (mut best, mut best_score) = (String::new(), 0usize);
    let sentences: Vec<String> =
        inline_re!(r"(?<=[.!?])\s+").split(&reply).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    for (index, sentence) in sentences.iter().enumerate() {
        if text::split_ref(sentence).len() < 6 {
            continue;
        }
        if index == 0 && known_first.is_none() && memory::topics::already_said(memory, Some(sentence), 3) {
            known_first = Some(sentence.clone());
        }
        let shared = want.intersection(&_circumstances(sentence)).count();
        if shared == 0 {
            continue;
        }
        let there: HashSet<String> =
            memory::topics::keywords(sentence).iter().filter(|w| text::len(w) > 2).map(|w| memory::topics::stem(w)).collect();
        let score = 2 * shared + said.intersection(&there).count();
        if score > best_score {
            best = sentence.clone();
            best_score = score;
            let _known = memory::topics::already_said(memory, Some(sentence), 3);
        }
    }
    if !best.is_empty() && known_first.as_deref() == Some(best.as_str()) {
        return None;
    }
    if best.is_empty() {
        return None;
    }
    if text::split_ref(&best).len() * 2 >= text::split_ref(&reply).len() {
        return None;
    }
    let part = if first_is_upper(&best) { low1(&best) } else { best };
    Some(_detail_frame(&part))
}

/// Python: bot/modules/answers.py:8102 `_a_complaint_answered(text, understood, memory)`
/// The help a report of a sore or a trouble did not spell out, or None.
pub fn _a_complaint_answered(text_: &str, understood: &Understood, memory: &mut Memory) -> Option<String> {
    if !COMPLAINT.is_match(&_flat(text_)) {
        return None;
    }
    if compose::trouble::reads_as_trouble(&_flat(text_), Some(&understood.words), "").py().is_some() {
        return None;
    }
    if find_plain(understood).is_some() {
        return None;
    }
    let mut again = understood.clone();
    let base = if !understood.clean.is_empty() { understood.clean.clone() } else { _flat(text_) };
    again.clean = format!("what can i do about {base}");
    again.words = az_words(&again.clean.to_lowercase()).into_iter().filter(|w| !w.is_empty()).collect();
    let (item, reply) = _offer_with(
        &mut |_: &mut Memory, reject: Reject| reader_entry(knowledge::lookup::find(&again, Some(_health_entries()), reject, false)),
        memory,
        &again,
    );
    let item = item?;
    let reply = reply.filter(|r| !text::strip(r).is_empty())?;
    if !_names_a_typed_word(item, text_) {
        return None;
    }
    let keys: Vec<String> = _key_words(item).into_iter().collect();
    memory::topics::remember_answer(memory, &reply, &keys);
    _remember_answer(memory, Some(item), &reply, &[]);
    Some(reply)
}

/// Python: bot/modules/answers.py:8134 `_a_dish_was_offered(memory)`
/// Whether the meal being planned has already been answered with food.
pub fn _a_dish_was_offered(memory: &Memory) -> bool {
    if !memory.get("meal").is_some_and(Value::is_dict) {
        return false;
    }
    memory.turns.last().is_some_and(|t| t.module.as_deref() == Some("answers"))
}

/// Python: bot/modules/answers.py:8147 `observe(text, memory)` - a runtime hook; its signature is the runtime's.
/// Keep what the line says about the meal it is planning, for the next short line to narrow.
pub fn observe(text_: &str, memory: &mut Memory) {
    let understood = match memory.understood.clone() {
        Some(u) => u,
        None => understanding::understand(text_, Some(&*memory)),
    };
    let _ = py_try(|| memory::a_year_of_birth::observe(text_, memory));
    let Some(plan) = py_try(|| knowledge::cooking_more::remembered(&understood, memory)) else { return };
    if let Some(plan) = plan.filter(|p| !p.is_empty()) {
        memory.set("meal", knowledge::cooking_more::context_value(&plan));
    }
}

/// Python: bot/modules/answers.py:8178 `_asks_only_the_length(text)`
/// Is every word of this line the length ask or a word pointing back at the answer?
pub fn _asks_only_the_length(text_: &str) -> bool {
    let words: Vec<String> = text::split_ref(text_)
        .iter()
        .map(|w| inline_re!(r"[^a-z']").sub("", &w.to_lowercase()))
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() || !words.iter().any(|w| w == "long" || w == "many" || w == "much") {
        return false;
    }
    words.iter().all(|w| LENGTH_SAID.contains(w))
}

/// Python: bot/modules/answers.py:8187 `TAKES_AS_IT_COMES(text, memory=None)`
/// A request for less is answered with the answer they already have, cut down.
pub fn TAKES_AS_IT_COMES(text_: &str, memory: &mut Memory) -> bool {
    memory::repeat::asks_for_less(text_) && memory::repeat::shorter_for(text_, memory).py_truth()
}

/// Python: bot/modules/answers.py:8201 `claims_instead(text, memory=None)` (also bound as `CLAIMS_INSTEAD`)
/// The arbiter's `CLAIMS_INSTEAD`: a line asking how to answer somebody else's question.
pub fn claims_instead(text_: &str, memory: &mut Memory) -> Option<bool> {
    if !ANSWERING_A_QUOTED_ASK.is_match(text_) {
        return Some(false);
    }
    if WHO_IS_THE_BOT.is_match(text_) {
        return Some(false);
    }
    Some(py_try(|| respond(text_, memory).is_some()).unwrap_or(false))
}

/// Python: bot/modules/answers.py:8224 `_a_limit_on_the_plan(text, memory)`
/// True when the line is only a limit on the plan, which the composer answers itself.
pub fn _a_limit_on_the_plan(text_: &str, memory: &Memory) -> bool {
    py_try(|| memory::facts::a_limit_on_the_plan(text_).py_truth() && memory::ack::a_limit_reply(text_, memory).py_truth()).unwrap_or(false)
}

/// Python: bot/modules/answers.py:8250 `_a_bare_thanks(understood)`
/// True when the line is a thank you and nothing else.
pub fn _a_bare_thanks(understood: Option<&Understood>) -> bool {
    understood.is_some_and(|u| u.kind == "thanks") && !_a_writing_job(understood)
}

/// Python: bot/modules/answers.py:8265 `_a_judgement_of_the_figure_on_the_table(text, memory)`
/// True when the line is only a doubt about a figure, and the reading can answer it.
pub fn _a_judgement_of_the_figure_on_the_table(text_: &str, memory: &Memory) -> bool {
    py_try(|| compose::reading::judges_the_figure_on_the_table(text_, memory)).unwrap_or(false)
}

/// Python: bot/modules/answers.py:8282 `_the_bot_has_something_to_go_over(memory)`
/// True when one of the bot's last lines explains something the ask could be pointing at.
pub fn _the_bot_has_something_to_go_over(memory: &Memory) -> bool {
    for turn in last_turns(memory, *MEANT_LOOK_BACK_FAR).iter().rev() {
        let said = text::strip(&turn.bot);
        if said.is_empty() {
            continue;
        }
        let module = turn.module.clone().unwrap_or_default();
        if MEANT_NOT_TO_REPEAT.contains(&module) {
            break;
        }
        let asked = text::strip(&turn.user);
        if REPEAT_HEDGE.is_match(said) || MEANT_CONDOLENCE.match_(said).is_some() || MEANT_RECEIPT.is_match(said) {
            continue;
        }
        if _AN_ANSWER.contains(&module) || memory::meant::_thick_enough(said, asked) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/answers.py:8315 `_the_line_asks_what_this_one_meant(memory)`
/// True when this turn asks what the bot's own last answer meant.
pub fn _the_line_asks_what_this_one_meant(memory: &Memory) -> bool {
    if !memory.understood.as_ref().is_some_and(|u| u.asks_what_was_meant) {
        return false;
    }
    _the_bot_has_something_to_go_over(memory)
}

// ==== ported code ends; the stubs below are the functions not yet ported ====

/// Python: bot/modules/answers.py:8331 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:respond")
}

/// Python: bot/modules/answers.py:8580 `_the_question_with_the_place_on_it(text, memory, reply)`
pub fn _the_question_with_the_place_on_it(text: &str, memory: &Memory, reply: Option<&str>) -> String {
    crate::unported!("bot.modules.answers:_the_question_with_the_place_on_it")
}

/// Python: bot/modules/answers.py:8623 `_of_its_own(memory)` (never called on the trace corpus)
pub fn _of_its_own(memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_of_its_own")
}

/// Python: bot/modules/answers.py:8631 `_keep_the_answer(text, reply, memory)`
pub fn _keep_the_answer(text: &str, reply: Option<&str>, memory: &mut Memory) {
    crate::unported!("bot.modules.answers:_keep_the_answer")
}

/// Python: bot/modules/answers.py:8655 `_the_sentence_it_names(reply, text, memory)`
pub fn _the_sentence_it_names(reply: &str, text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_the_sentence_it_names")
}

/// Python: bot/modules/answers.py:8685 `_fitted_to_what_they_eat(reply, text, memory)`
pub fn _fitted_to_what_they_eat(reply: &str, text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_fitted_to_what_they_eat")
}

/// Python: bot/modules/answers.py:8726 `_the_dish_by_name(answer, memory)`
pub fn _the_dish_by_name(answer: Option<&str>, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_dish_by_name")
}

/// Python: bot/modules/answers.py:8762 `_the_same_answer_again(answer, memory)` (never called on the trace corpus)
pub fn _the_same_answer_again(answer: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_same_answer_again")
}

/// Python: bot/modules/answers.py:8774 `_the_figure_the_thing_we_are_on_holds(text, memory)` (never called on the trace corpus)
pub fn _the_figure_the_thing_we_are_on_holds(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_figure_the_thing_we_are_on_holds")
}

/// Python: bot/modules/answers.py:8799 `_a_length_ask_with_no_figure(text, memory, answer='')` (never called on the trace corpus)
pub fn _a_length_ask_with_no_figure(text: &str, memory: &Memory, answer: &str) -> Value {
    crate::unported!("bot.modules.answers:_a_length_ask_with_no_figure")
}

/// Python: bot/modules/answers.py:8828 `_a_city_card_asks_which_first(text, memory)` (never called on the trace corpus)
pub fn _a_city_card_asks_which_first(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_city_card_asks_which_first")
}

/// Python: bot/modules/answers.py:8839 `_the_answer_in_front(memory)` (never called on the trace corpus)
pub fn _the_answer_in_front(memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_the_answer_in_front")
}

/// Python: bot/modules/answers.py:8920 `_a_lead_time(text)`
pub fn _a_lead_time(text: &str) {
    crate::unported!("bot.modules.answers:_a_lead_time")
}

/// Python: bot/modules/answers.py:8950 `_the_thing_has_got_better(text, memory)`
pub fn _the_thing_has_got_better(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_thing_has_got_better")
}

/// Python: bot/modules/answers.py:8983 `_a_case_of_the_answer_in_hand(text, memory)`
pub fn _a_case_of_the_answer_in_hand(text: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_a_case_of_the_answer_in_hand")
}

/// Python: bot/modules/answers.py:9060 `_answer(text, memory)`
pub fn _answer(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_answer")
}

/// Python: bot/modules/answers.py:9644 `_a_measure_about_the_subject(measure, text, memory, alone='')`
pub fn _a_measure_about_the_subject(measure: &str, text: &str, memory: &mut Memory, alone: &str) -> String {
    crate::unported!("bot.modules.answers:_a_measure_about_the_subject")
}

/// Python: bot/modules/answers.py:9803 `_the_line_already_has_its_answer(alone, text)` (never called on the trace corpus)
pub fn _the_line_already_has_its_answer(alone: &Value, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_the_line_already_has_its_answer")
}

/// Python: bot/modules/answers.py:9833 `_names_a_repair(entry)` (never called on the trace corpus)
pub fn _names_a_repair(entry: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_names_a_repair")
}

/// Python: bot/modules/answers.py:9841 `_the_entry_names(entry, word)` (never called on the trace corpus)
pub fn _the_entry_names(entry: &'static Entry, word: &str) -> bool {
    crate::unported!("bot.modules.answers:_the_entry_names")
}

/// Python: bot/modules/answers.py:9849 `_the_plan_said(memory)` (never called on the trace corpus)
pub fn _the_plan_said(memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_the_plan_said")
}

/// Python: bot/modules/answers.py:9860 `_the_part_is_what_answers(part, memory)`
pub fn _the_part_is_what_answers(part: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_part_is_what_answers")
}

/// Python: bot/modules/answers.py:9891 `_the_part_of_the_thing_they_named(text, memory)`
pub fn _the_part_of_the_thing_they_named(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.answers:_the_part_of_the_thing_they_named")
}

/// Python: bot/modules/answers.py:9931 `_the_report_is_answered(text, entry)`
pub fn _the_report_is_answered(text: &str, entry: Option<&'static Entry>) -> bool {
    crate::unported!("bot.modules.answers:_the_report_is_answered")
}

/// Python: bot/modules/answers.py:9975 `_the_fact_is_what_is_answered(text, stated, entry, memory=None)`
pub fn _the_fact_is_what_is_answered(text: &str, stated: &str, entry: Option<&'static Entry>, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_fact_is_what_is_answered")
}

/// Python: bot/modules/answers.py:10032 `_ask_a_line_of_its_own(line, memory)` (never called on the trace corpus)
pub fn _ask_a_line_of_its_own(line: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_ask_a_line_of_its_own")
}

/// Python: bot/modules/answers.py:10046 `_the_entry_key_words(entry)` (never called on the trace corpus)
pub fn _the_entry_key_words(entry: &'static Entry) -> Value {
    crate::unported!("bot.modules.answers:_the_entry_key_words")
}

/// Python: bot/modules/answers.py:10056 `_the_practice_is_answered(practice, entry)` (never called on the trace corpus)
pub fn _the_practice_is_answered(practice: &Value, entry: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_the_practice_is_answered")
}

/// Python: bot/modules/answers.py:10099 `_the_ask_behind_a_moan(text, memory)`
pub fn _the_ask_behind_a_moan(text: &str, memory: &mut Memory) -> String {
    crate::unported!("bot.modules.answers:_the_ask_behind_a_moan")
}

/// Python: bot/modules/answers.py:10135 `_ask_a_rewritten_line(line, memory)`
pub fn _ask_a_rewritten_line(line: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_ask_a_rewritten_line")
}

/// Python: bot/modules/answers.py:10177 `_turns_on_this_line(text, memory)`
pub fn _turns_on_this_line(text: &str, memory: &Memory) -> i64 {
    crate::unported!("bot.modules.answers:_turns_on_this_line")
}

/// Python: bot/modules/answers.py:10192 `_without_lead_in(answer)`
pub fn _without_lead_in(answer: &str) -> String {
    crate::unported!("bot.modules.answers:_without_lead_in")
}

/// Python: bot/modules/answers.py:10212 `_the_line_again(text, memory)`
pub fn _the_line_again(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_line_again")
}

/// Python: bot/modules/answers.py:10281 `_stated_with_copula(text, value)` (never called on the trace corpus)
pub fn _stated_with_copula(text: &str, value: &Value) -> bool {
    crate::unported!("bot.modules.answers:_stated_with_copula")
}

/// Python: bot/modules/answers.py:10298 `_nothing_but_the_slot(said, value_words, about)` (never called on the trace corpus)
pub fn _nothing_but_the_slot(said: &str, value_words: &[String], about: &Value) -> bool {
    crate::unported!("bot.modules.answers:_nothing_but_the_slot")
}

/// Python: bot/modules/answers.py:10320 `_the_food_they_do_not_eat(text, understood)`
pub fn _the_food_they_do_not_eat(text: &str, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_the_food_they_do_not_eat")
}

/// Python: bot/modules/answers.py:10344 `_store_keeps_the_line(text, understood)`
pub fn _store_keeps_the_line(text: &str, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_store_keeps_the_line")
}

/// Python: bot/modules/answers.py:10418 `_says_line(text, words)`
pub fn _says_line(text: &str, words: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_says_line")
}

/// Python: bot/modules/answers.py:10436 `_a_pick_is_asked(text)`
pub fn _a_pick_is_asked(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_pick_is_asked")
}

/// Python: bot/modules/answers.py:10441 `_the_line_as_said(text)`
pub fn _the_line_as_said(text: &str) -> String {
    crate::unported!("bot.modules.answers:_the_line_as_said")
}

/// Python: bot/modules/answers.py:10477 `_only_the_office(memory)`
pub fn _only_the_office(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_only_the_office")
}

/// Python: bot/modules/answers.py:10492 `_a_food_named_of_its_own(understood, food)`
pub fn _a_food_named_of_its_own(understood: &Understood, food: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_food_named_of_its_own")
}

/// Python: bot/modules/answers.py:10512 `_keeping_about_the_food_there(memory, understood)`
pub fn _keeping_about_the_food_there(memory: &Memory, understood: &Understood) {
    crate::unported!("bot.modules.answers:_keeping_about_the_food_there")
}

/// Python: bot/modules/answers.py:10556 `_the_answer_on_the_table(understood, memory)`
pub fn _the_answer_on_the_table(understood: &Understood, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_answer_on_the_table")
}

/// Python: bot/modules/answers.py:10595 `_already_on_the_screen(reply, memory, turns=4)` (never called on the trace corpus)
pub fn _already_on_the_screen(reply: &str, memory: &Memory, turns: &[Turn]) -> bool {
    crate::unported!("bot.modules.answers:_already_on_the_screen")
}

/// Python: bot/modules/answers.py:10624 `_the_verse_said_again(text, memory)`
pub fn _the_verse_said_again(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_verse_said_again")
}

/// Python: bot/modules/answers.py:10664 `_the_sentence_around(fragment, text)` (never called on the trace corpus)
pub fn _the_sentence_around(fragment: &str, text: &str) -> String {
    crate::unported!("bot.modules.answers:_the_sentence_around")
}

/// Python: bot/modules/answers.py:10691 `_the_sentence_with_a_figure(text)` (never called on the trace corpus)
pub fn _the_sentence_with_a_figure(text: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_sentence_with_a_figure")
}

/// Python: bot/modules/answers.py:10741 `_puts_it_to_someone(reply)` (never called on the trace corpus)
pub fn _puts_it_to_someone(reply: &str) -> bool {
    crate::unported!("bot.modules.answers:_puts_it_to_someone")
}

/// Python: bot/modules/answers.py:10751 `_the_ask_is_for_words(reading)`
pub fn _the_ask_is_for_words(reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_the_ask_is_for_words")
}

/// Python: bot/modules/answers.py:10759 `_respond(text, memory)`
pub fn _respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_respond")
}

