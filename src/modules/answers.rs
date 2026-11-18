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
pub fn _keyed_on_the_person(item: &'static Entry, who: &str) -> bool {
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
pub fn _reply_of(item: &'static Entry, reading: &Understood, memory: &Memory) -> String {
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
pub fn _payload(item: &'static Entry, memory: &Memory, reading: &Understood) -> String {
    _payload_mut(item, &mut memory.clone(), reading)
}

/// Python: bot/modules/answers.py:6319 `_payload(item, memory, reading)`
/// The entry's reply, chosen the way the composer will choose it, or "" if it declines.
pub fn _payload_mut(item: &'static Entry, memory: &mut Memory, reading: &Understood) -> String {
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

// ==== ported code ends; the stubs below are the functions not yet ported ====

/// Python: bot/modules/answers.py:2041 `_with_subject_tail(text, memory, understood=None)` (never called on the trace corpus)
pub fn _with_subject_tail(text: &str, memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_with_subject_tail")
}

/// Python: bot/modules/answers.py:2049 `_subject_tails(text, memory, understood=None)`
pub fn _subject_tails(text: &str, memory: &Memory, understood: &Understood) -> Vec<String> {
    crate::unported!("bot.modules.answers:_subject_tails")
}

/// Python: bot/modules/answers.py:2121 `_bare_subject(subject)`
pub fn _bare_subject(subject: &str) -> String {
    crate::unported!("bot.modules.answers:_bare_subject")
}

/// Python: bot/modules/answers.py:2154 `_the_month(text)`
pub fn _the_month(text: &str) -> String {
    crate::unported!("bot.modules.answers:_the_month")
}

/// Python: bot/modules/answers.py:2165 `_a_trip(memory)` (never called on the trace corpus)
pub fn _a_trip(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_trip")
}

/// Python: bot/modules/answers.py:2193 `_a_clause(subject)`
pub fn _a_clause(subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_clause")
}

/// Python: bot/modules/answers.py:2198 `_choosing(text, memory)`
pub fn _choosing(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_choosing")
}

/// Python: bot/modules/answers.py:2285 `_only_a_size_of(item, text)`
pub fn _only_a_size_of(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_only_a_size_of")
}

/// Python: bot/modules/answers.py:2310 `_choice_words(text, memory)` (never called on the trace corpus)
pub fn _choice_words(text: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_choice_words")
}

/// Python: bot/modules/answers.py:2343 `_qualifier_rewrites(text, memory)`
pub fn _qualifier_rewrites(text: &str, memory: &Value) -> Vec<String> {
    crate::unported!("bot.modules.answers:_qualifier_rewrites")
}

/// Python: bot/modules/answers.py:2433 `_a_lack_not_a_refusal(flat, end)` (never called on the trace corpus)
pub fn _a_lack_not_a_refusal(flat: &Value, end: i64) -> bool {
    crate::unported!("bot.modules.answers:_a_lack_not_a_refusal")
}

/// Python: bot/modules/answers.py:2442 `_refused(text)`
pub fn _refused(text: &str) -> String {
    crate::unported!("bot.modules.answers:_refused")
}

/// Python: bot/modules/answers.py:2476 `_says_it_leaves_it_out(reply, refused)` (never called on the trace corpus)
pub fn _says_it_leaves_it_out(reply: &str, refused: &str) -> bool {
    crate::unported!("bot.modules.answers:_says_it_leaves_it_out")
}

/// Python: bot/modules/answers.py:2504 `_hands_back_the_refusal(item, refused)`
pub fn _hands_back_the_refusal(item: &'static Entry, refused: &str) -> bool {
    crate::unported!("bot.modules.answers:_hands_back_the_refusal")
}

/// Python: bot/modules/answers.py:2528 `_about_a_lack(item)` (never called on the trace corpus)
pub fn _about_a_lack(item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_about_a_lack")
}

/// Python: bot/modules/answers.py:2538 `_key_words(item)`
pub fn _key_words(item: &'static Entry) -> Vec<String> {
    crate::unported!("bot.modules.answers:_key_words")
}

/// Python: bot/modules/answers.py:2573 `_names_a_thing(understood)`
pub fn _names_a_thing(understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_names_a_thing")
}

/// Python: bot/modules/answers.py:2582 `_own_thing(understood)`
pub fn _own_thing(understood: &Understood) -> String {
    crate::unported!("bot.modules.answers:_own_thing")
}

/// Python: bot/modules/answers.py:2637 `_thing_named_in(understood, span)`
pub fn _thing_named_in(understood: &Understood, span: &[String]) -> String {
    crate::unported!("bot.modules.answers:_thing_named_in")
}

/// Python: bot/modules/answers.py:2654 `_new_thing(own, answer)`
pub fn _new_thing(own: &str, answer: &str) -> bool {
    crate::unported!("bot.modules.answers:_new_thing")
}

/// Python: bot/modules/answers.py:2666 `_the_run_said(memory, extra='')`
pub fn _the_run_said(memory: &Memory, extra: &str) -> String {
    crate::unported!("bot.modules.answers:_the_run_said")
}

/// Python: bot/modules/answers.py:2693 `_head_thing(subject)`
pub fn _head_thing(subject: &str) -> String {
    crate::unported!("bot.modules.answers:_head_thing")
}

/// Python: bot/modules/answers.py:2708 `_the_same_kind_of_thing(item, memory, text='')`
pub fn _the_same_kind_of_thing(item: &'static Entry, memory: &Memory, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_the_same_kind_of_thing")
}

/// Python: bot/modules/answers.py:2737 `_reaches_another_subject(text, item, memory)`
pub fn _reaches_another_subject(text: &str, item: &'static Entry, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_reaches_another_subject")
}

/// Python: bot/modules/answers.py:2851 `_off_the_topic(understood, memory)`
pub fn _off_the_topic(understood: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_off_the_topic")
}

/// Python: bot/modules/answers.py:2903 `_from_another_domain(item, off_topic)` (never called on the trace corpus)
pub fn _from_another_domain(item: &'static Entry, off_topic: &Value) -> bool {
    crate::unported!("bot.modules.answers:_from_another_domain")
}

/// Python: bot/modules/answers.py:2911 `_names_a_topic_word(item, wanted)` (never called on the trace corpus)
pub fn _names_a_topic_word(item: &'static Entry, wanted: &Value) -> bool {
    crate::unported!("bot.modules.answers:_names_a_topic_word")
}

/// Python: bot/modules/answers.py:2951 `_topic_words(line, memory)`
pub fn _topic_words(line: &str, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.answers:_topic_words")
}

/// Python: bot/modules/answers.py:2960 `_lent_words(reading, memory)`
pub fn _lent_words(reading: &Understood, memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.answers:_lent_words")
}

/// Python: bot/modules/answers.py:2978 `_same_word(word, said)`
pub fn _same_word(word: &str, said: &str) -> bool {
    crate::unported!("bot.modules.answers:_same_word")
}

/// Python: bot/modules/answers.py:2988 `_writes_its_own_reply(item)`
pub fn _writes_its_own_reply(item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_writes_its_own_reply")
}

/// Python: bot/modules/answers.py:3009 `_leans_on(reading, item, memory=None)`
pub fn _leans_on(reading: &Understood, item: &'static Entry, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_leans_on")
}

/// Python: bot/modules/answers.py:3118 `_is_about(item, subject)`
pub fn _is_about(item: &'static Entry, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_is_about")
}

/// Python: bot/modules/answers.py:3174 `_only_a_property(words)`
pub fn _only_a_property(words: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_only_a_property")
}

/// Python: bot/modules/answers.py:3197 `_a_property_word_names_a_thing(text)`
pub fn _a_property_word_names_a_thing(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_property_word_names_a_thing")
}

/// Python: bot/modules/answers.py:3215 `_a_money_question(own, keyed)`
pub fn _a_money_question(own: &[String], keyed: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_a_money_question")
}

/// Python: bot/modules/answers.py:3224 `_remember_answer(memory, item, reply, keys=())`
pub fn _remember_answer(memory: &mut Memory, item: Option<&'static Entry>, reply: &str, keys: &[String]) {
    crate::unported!("bot.modules.answers:_remember_answer")
}

/// Python: bot/modules/answers.py:3242 `_last_answer(memory)`
pub fn _last_answer(memory: &Memory) {
    crate::unported!("bot.modules.answers:_last_answer")
}

/// Python: bot/modules/answers.py:3274 `_asks_part(text)`
pub fn _asks_part(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_part")
}

/// Python: bot/modules/answers.py:3323 `_the_sentence_about(text, memory)`
pub fn _the_sentence_about(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_sentence_about")
}

/// Python: bot/modules/answers.py:3426 `_a_stock_acknowledgement(turns, back)` (never called on the trace corpus)
pub fn _a_stock_acknowledgement(turns: &[Turn], back: &Value) -> bool {
    crate::unported!("bot.modules.answers:_a_stock_acknowledgement")
}

/// Python: bot/modules/answers.py:3452 `_the_person_to_call(text, memory)`
pub fn _the_person_to_call(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_person_to_call")
}

/// Python: bot/modules/answers.py:3530 `_the_clause_that_says_who(part)` (never called on the trace corpus)
pub fn _the_clause_that_says_who(part: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_clause_that_says_who")
}

/// Python: bot/modules/answers.py:3600 `_the_clause_that_picks(sentence)` (never called on the trace corpus)
pub fn _the_clause_that_picks(sentence: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_clause_that_picks")
}

/// Python: bot/modules/answers.py:3644 `_a_list_of_things(tail)` (never called on the trace corpus)
pub fn _a_list_of_things(tail: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_list_of_things")
}

/// Python: bot/modules/answers.py:3659 `_the_thing_that_picks(sentence)` (never called on the trace corpus)
pub fn _the_thing_that_picks(sentence: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_thing_that_picks")
}

/// Python: bot/modules/answers.py:3729 `_a_price_and_nothing_else(span)` (never called on the trace corpus)
pub fn _a_price_and_nothing_else(span: &Value) -> bool {
    crate::unported!("bot.modules.answers:_a_price_and_nothing_else")
}

/// Python: bot/modules/answers.py:3750 `_the_composer_has_options(memory)` (never called on the trace corpus)
pub fn _the_composer_has_options(memory: &Memory) -> Option<bool> {
    crate::unported!("bot.modules.answers:_the_composer_has_options")
}

/// Python: bot/modules/answers.py:3770 `_the_option_that_decides(said, reply)` (never called on the trace corpus)
pub fn _the_option_that_decides(said: &str, reply: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_option_that_decides")
}

/// Python: bot/modules/answers.py:3781 `_the_answer_to_a_choice_behind(memory)` (never called on the trace corpus)
pub fn _the_answer_to_a_choice_behind(memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_answer_to_a_choice_behind")
}

/// Python: bot/modules/answers.py:3807 `_the_one_to_pick(text, memory)`
pub fn _the_one_to_pick(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_the_one_to_pick")
}

/// Python: bot/modules/answers.py:3900 `_one_total(rest, whole_sentence=False)` (never called on the trace corpus)
pub fn _one_total(rest: &str, whole_sentence: &Value) -> bool {
    crate::unported!("bot.modules.answers:_one_total")
}

/// Python: bot/modules/answers.py:3924 `_the_dish_time(understood, memory)`
pub fn _the_dish_time(understood: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_dish_time")
}

/// Python: bot/modules/answers.py:4006 `_the_unit_last_said(memory, reply)` (never called on the trace corpus)
pub fn _the_unit_last_said(memory: &Memory, reply: &str) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_unit_last_said")
}

/// Python: bot/modules/answers.py:4026 `_the_figure_they_meant(text, memory, reply)`
pub fn _the_figure_they_meant(text: &str, memory: &Memory, reply: &str) {
    crate::unported!("bot.modules.answers:_the_figure_they_meant")
}

/// Python: bot/modules/answers.py:4050 `_amount_asked(text, memory)`
pub fn _amount_asked(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_amount_asked")
}

/// Python: bot/modules/answers.py:4133 `_the_amount_they_named_again(text, memory)`
pub fn _the_amount_they_named_again(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_amount_they_named_again")
}

/// Python: bot/modules/answers.py:4195 `_the_machine_follow_up(text, reading, memory)`
pub fn _the_machine_follow_up(text: &str, reading: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_machine_follow_up")
}

/// Python: bot/modules/answers.py:4247 `_part_asked(text, memory)`
pub fn _part_asked(text: &str, memory: &Value) {
    crate::unported!("bot.modules.answers:_part_asked")
}

/// Python: bot/modules/answers.py:4541 `_amount_for_the_name(clause, named)`
pub fn _amount_for_the_name(clause: &str, named: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_amount_for_the_name")
}

/// Python: bot/modules/answers.py:4596 `_sentence_naming_the_part(reply, text)`
pub fn _sentence_naming_the_part(reply: &str, text: &str) -> Option<String> {
    crate::unported!("bot.modules.answers:_sentence_naming_the_part")
}

/// Python: bot/modules/answers.py:4671 `_names_no_cause(head)`
pub fn _names_no_cause(head: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_no_cause")
}

/// Python: bot/modules/answers.py:4679 `_cause_part(part)`
pub fn _cause_part(part: &str) -> String {
    crate::unported!("bot.modules.answers:_cause_part")
}

/// Python: bot/modules/answers.py:4700 `_reason_again(reason, last)`
pub fn _reason_again(reason: &str, last: &str) -> String {
    crate::unported!("bot.modules.answers:_reason_again")
}

/// Python: bot/modules/answers.py:4762 `_asked_figure(text, reply, other_reply)`
pub fn _asked_figure(text: &str, reply: &str, other_reply: Option<&str>) -> String {
    crate::unported!("bot.modules.answers:_asked_figure")
}

/// Python: bot/modules/answers.py:4787 `_the_figure(text, reply)`
pub fn _the_figure(text: &str, reply: &str) -> String {
    crate::unported!("bot.modules.answers:_the_figure")
}

/// Python: bot/modules/answers.py:4792 `_sentence_with(reply, part)` (never called on the trace corpus)
pub fn _sentence_with(reply: &str, part: &str) -> String {
    crate::unported!("bot.modules.answers:_sentence_with")
}

/// Python: bot/modules/answers.py:4805 `_part_not_repeated(part, whole, memory)`
pub fn _part_not_repeated(part: Option<&str>, whole: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_part_not_repeated")
}

/// Python: bot/modules/answers.py:4856 `_names_a_property(text)`
pub fn _names_a_property(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_a_property")
}

/// Python: bot/modules/answers.py:4869 `_with_bare_subject(text, memory)`
pub fn _with_bare_subject(text: &str, memory: &Value) -> String {
    crate::unported!("bot.modules.answers:_with_bare_subject")
}

/// Python: bot/modules/answers.py:4883 `_the_things_to_try(understood, memory)` (never called on the trace corpus)
pub fn _the_things_to_try(understood: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_things_to_try")
}

/// Python: bot/modules/answers.py:4916 `_the_answer_from_thing(text, thing, memory)` (never called on the trace corpus)
pub fn _the_answer_from_thing(text: &str, thing: &str, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_answer_from_thing")
}

/// Python: bot/modules/answers.py:4932 `_with_the_thing(text, thing)` (never called on the trace corpus)
pub fn _with_the_thing(text: &str, thing: &str) -> Value {
    crate::unported!("bot.modules.answers:_with_the_thing")
}

/// Python: bot/modules/answers.py:4946 `_care_holds(text, memory)`
pub fn _care_holds(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_care_holds")
}

/// Python: bot/modules/answers.py:4962 `_named_by(item, subject)`
pub fn _named_by(item: &'static Entry, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_named_by")
}

/// Python: bot/modules/answers.py:5005 `_the_answer_was_about_the_subject(memory)`
pub fn _the_answer_was_about_the_subject(memory: &Value) -> bool {
    crate::unported!("bot.modules.answers:_the_answer_was_about_the_subject")
}

/// Python: bot/modules/answers.py:5053 `_minutes_in(text)`
pub fn _minutes_in(text: Option<&str>) -> Value {
    crate::unported!("bot.modules.answers:_minutes_in")
}

/// Python: bot/modules/answers.py:5070 `_a_figure_apart(mine, theirs)`
pub fn _a_figure_apart(mine: &Memory, theirs: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_figure_apart")
}

/// Python: bot/modules/answers.py:5086 `_only_a_figure(part)`
pub fn _only_a_figure(part: &str) -> bool {
    crate::unported!("bot.modules.answers:_only_a_figure")
}

/// Python: bot/modules/answers.py:5097 `_figures(text)`
pub fn _figures(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.answers:_figures")
}

/// Python: bot/modules/answers.py:5119 `_name_words()`
pub fn _name_words() -> Vec<String> {
    crate::unported!("bot.modules.answers:_name_words")
}

/// Python: bot/modules/answers.py:5149 `_filler(word)`
pub fn _filler(word: &str) -> bool {
    crate::unported!("bot.modules.answers:_filler")
}

/// Python: bot/modules/answers.py:5161 `_left_out(item, text)`
pub fn _left_out(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_left_out")
}

/// Python: bot/modules/answers.py:5193 `_keyed_on_the_line(item, text)`
pub fn _keyed_on_the_line(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_keyed_on_the_line")
}

/// Python: bot/modules/answers.py:5209 `_shares_a_word(item, subject)` (never called on the trace corpus)
pub fn _shares_a_word(item: &'static Entry, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_shares_a_word")
}

/// Python: bot/modules/answers.py:5225 `_answers_the_figure(text, reply)`
pub fn _answers_the_figure(text: &str, reply: &str) -> bool {
    crate::unported!("bot.modules.answers:_answers_the_figure")
}

/// Python: bot/modules/answers.py:5281 `_asks_how_to_say_it(text)`
pub fn _asks_how_to_say_it(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_how_to_say_it")
}

/// Python: bot/modules/answers.py:5286 `_in_the_subjects_field(item, subject)`
pub fn _in_the_subjects_field(item: &'static Entry, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_in_the_subjects_field")
}

/// Python: bot/modules/answers.py:5310 `_own_thing_words(text, memory)`
pub fn _own_thing_words(text: &str, memory: &Value) -> Vec<String> {
    crate::unported!("bot.modules.answers:_own_thing_words")
}

/// Python: bot/modules/answers.py:5332 `_about_the_talk(item, text, memory, reading)`
pub fn _about_the_talk(item: &'static Entry, text: &str, memory: &Memory, reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_about_the_talk")
}

/// Python: bot/modules/answers.py:5412 `_agrees_with_its_own_field(item, reading)`
pub fn _agrees_with_its_own_field(item: &'static Entry, reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_agrees_with_its_own_field")
}

/// Python: bot/modules/answers.py:5423 `_the_field_on_the_screen(item, text, reading, memory)`
pub fn _the_field_on_the_screen(item: &'static Entry, text: &str, reading: &Understood, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_field_on_the_screen")
}

/// Python: bot/modules/answers.py:5455 `_last_turn_answered_from_the_field(memory)`
pub fn _last_turn_answered_from_the_field(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_last_turn_answered_from_the_field")
}

/// Python: bot/modules/answers.py:5476 `_says_a_subject_phrase(item, text)`
pub fn _says_a_subject_phrase(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_says_a_subject_phrase")
}

/// Python: bot/modules/answers.py:5524 `_names_the_entry(text, item)`
pub fn _names_the_entry(text: &str, item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_names_the_entry")
}

/// Python: bot/modules/answers.py:5564 `_a_definition_of_the_entry(text, item)`
pub fn _a_definition_of_the_entry(text: &str, item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_a_definition_of_the_entry")
}

/// Python: bot/modules/answers.py:5611 `_says_the_ask_whole(text, item)`
pub fn _says_the_ask_whole(text: &str, item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_says_the_ask_whole")
}

/// Python: bot/modules/answers.py:5633 `_a_why_line_that_names(item, reading)`
pub fn _a_why_line_that_names(item: &'static Entry, reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_a_why_line_that_names")
}

/// Python: bot/modules/answers.py:5640 `_names_the_thing_apart_from_the_talk(item, text, subject, reading)`
pub fn _names_the_thing_apart_from_the_talk(item: &'static Entry, text: &str, subject: &str, reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_names_the_thing_apart_from_the_talk")
}

/// Python: bot/modules/answers.py:5678 `_a_choice_with_one_thing_left_out(text)`
pub fn _a_choice_with_one_thing_left_out(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_choice_with_one_thing_left_out")
}

/// Python: bot/modules/answers.py:5735 `_describes_the_thing(word)`
pub fn _describes_the_thing(word: &str) -> bool {
    crate::unported!("bot.modules.answers:_describes_the_thing")
}

/// Python: bot/modules/answers.py:5750 `_said_in_the_talk(memory)`
pub fn _said_in_the_talk(memory: &Memory) -> Vec<String> {
    crate::unported!("bot.modules.answers:_said_in_the_talk")
}

/// Python: bot/modules/answers.py:5761 `_already_in_the_talk(word, said)`
pub fn _already_in_the_talk(word: &str, said: &[String]) -> bool {
    crate::unported!("bot.modules.answers:_already_in_the_talk")
}

/// Python: bot/modules/answers.py:5772 `_names_its_own_thing(text, memory=None)`
pub fn _names_its_own_thing(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_names_its_own_thing")
}

/// Python: bot/modules/answers.py:5816 `_the_topic_rules_it_out(item, memory)` (never called on the trace corpus)
pub fn _the_topic_rules_it_out(item: &'static Entry, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_topic_rules_it_out")
}

/// Python: bot/modules/answers.py:5829 `_in_context(text, memory, reject=None)`
pub fn _in_context(text: &str, memory: &Value, reject: &Value) {
    crate::unported!("bot.modules.answers:_in_context")
}

/// Python: bot/modules/answers.py:6005 `_from_rewrite(fuller, text, memory, reject, added='', domain=None)` (never called on the trace corpus)
pub fn _from_rewrite(fuller: &Value, text: &str, memory: &Memory, reject: &Value, added: &Value, domain: &Value) -> Value {
    crate::unported!("bot.modules.answers:_from_rewrite")
}

/// Python: bot/modules/answers.py:6049 `_asks_storing(memory)`
pub fn _asks_storing(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_asks_storing")
}

/// Python: bot/modules/answers.py:6060 `_keeping_thing(memory)` (never called on the trace corpus)
pub fn _keeping_thing(memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_keeping_thing")
}

/// Python: bot/modules/answers.py:6068 `_the_plan_keeps(understood, memory)` (never called on the trace corpus)
pub fn _the_plan_keeps(understood: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_the_plan_keeps")
}

/// Python: bot/modules/answers.py:6085 `_keeping_entry(text, memory, thing=None, storing=False)` (never called on the trace corpus)
pub fn _keeping_entry(text: &str, memory: &Memory, thing: &str, storing: &Value) -> Value {
    crate::unported!("bot.modules.answers:_keeping_entry")
}

/// Python: bot/modules/answers.py:6140 `_in_this_conversation(reject, reading)`
pub fn _in_this_conversation(reject: &Value, reading: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_in_this_conversation")
}

/// Python: bot/modules/answers.py:6155 `_in_bare_context(text, memory, reject=None)`
pub fn _in_bare_context(text: &str, memory: &Value, reject: &Value) {
    crate::unported!("bot.modules.answers:_in_bare_context")
}

/// Python: bot/modules/answers.py:6198 `_figures_of(said)`
pub fn _figures_of(said: &str) -> Vec<String> {
    crate::unported!("bot.modules.answers:_figures_of")
}

/// Python: bot/modules/answers.py:6203 `_new_figure(fresh, before)`
pub fn _new_figure(fresh: &str, before: &str) -> bool {
    crate::unported!("bot.modules.answers:_new_figure")
}

/// Python: bot/modules/answers.py:6213 `_adds_a_figure(text)`
pub fn _adds_a_figure(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_adds_a_figure")
}

/// Python: bot/modules/answers.py:6233 `_a_figure_put_right(text)`
pub fn _a_figure_put_right(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_figure_put_right")
}

/// Python: bot/modules/answers.py:6239 `_without_the_figures(line)` (never called on the trace corpus)
pub fn _without_the_figures(line: &str) -> String {
    crate::unported!("bot.modules.answers:_without_the_figures")
}

/// Python: bot/modules/answers.py:6244 `_the_entry_asked_again(text, memory)`
pub fn _the_entry_asked_again(text: &str, memory: &Memory) -> Option<String> {
    crate::unported!("bot.modules.answers:_the_entry_asked_again")
}

/// Python: bot/modules/answers.py:6294 `_the_part_a_longer_variant_adds(fresh, said)`
pub fn _the_part_a_longer_variant_adds(fresh: &str, said: &str) {
    crate::unported!("bot.modules.answers:_the_part_a_longer_variant_adds")
}

/// Python: bot/modules/answers.py:6329 `_the_sentence_owed(text, reply, memory)`
pub fn _the_sentence_owed(text: &str, reply: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_sentence_owed")
}

/// Python: bot/modules/answers.py:6402 `_wants_a_stand_in(reading)`
pub fn _wants_a_stand_in(reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_wants_a_stand_in")
}

/// Python: bot/modules/answers.py:6428 `_a_swap_row(item)` (never called on the trace corpus)
pub fn _a_swap_row(item: &'static Entry) -> bool {
    crate::unported!("bot.modules.answers:_a_swap_row")
}

/// Python: bot/modules/answers.py:6484 `_price_limit(text)`
pub fn _price_limit(text: &str) -> Value {
    crate::unported!("bot.modules.answers:_price_limit")
}

/// Python: bot/modules/answers.py:6512 `_only_a_limit(text)`
pub fn _only_a_limit(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_only_a_limit")
}

/// Python: bot/modules/answers.py:6519 `_a_gift_entry(reading)`
pub fn _a_gift_entry(reading: &Understood) {
    crate::unported!("bot.modules.answers:_a_gift_entry")
}

/// Python: bot/modules/answers.py:6525 `_a_gift_just_answered(memory)` (never called on the trace corpus)
pub fn _a_gift_just_answered(memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_a_gift_just_answered")
}

/// Python: bot/modules/answers.py:6540 `_asks_what_to_give(reading)` (never called on the trace corpus)
pub fn _asks_what_to_give(reading: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_asks_what_to_give")
}

/// Python: bot/modules/answers.py:6551 `_the_interests(text, memory, item)`
pub fn _the_interests(text: &str, memory: &Memory, item: &'static Entry) -> Vec<String> {
    crate::unported!("bot.modules.answers:_the_interests")
}

/// Python: bot/modules/answers.py:6580 `_the_entry_reply(item, memory, reading)` (never called on the trace corpus)
pub fn _the_entry_reply(item: &'static Entry, memory: &Memory, reading: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_the_entry_reply")
}

/// Python: bot/modules/answers.py:6590 `_the_present_with_a_price(text, reading, memory)`
pub fn _the_present_with_a_price(text: &str, reading: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_present_with_a_price")
}

/// Python: bot/modules/answers.py:6666 `_asks_a_price(text)`
pub fn _asks_a_price(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_a_price")
}

/// Python: bot/modules/answers.py:6687 `_names_what_it_wants_priced(text)`
pub fn _names_what_it_wants_priced(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_what_it_wants_priced")
}

/// Python: bot/modules/answers.py:6759 `_same_where_they_asked(text, said)` (never called on the trace corpus)
pub fn _same_where_they_asked(text: &str, said: &str) -> String {
    crate::unported!("bot.modules.answers:_same_where_they_asked")
}

/// Python: bot/modules/answers.py:6816 `_mid_sentence(part)` (never called on the trace corpus)
pub fn _mid_sentence(part: &str) -> Value {
    crate::unported!("bot.modules.answers:_mid_sentence")
}

/// Python: bot/modules/answers.py:6847 `_holds_where_they_asked(part, asked)` (never called on the trace corpus)
pub fn _holds_where_they_asked(part: &str, asked: &str) -> bool {
    crate::unported!("bot.modules.answers:_holds_where_they_asked")
}

/// Python: bot/modules/answers.py:6862 `_the_method_in(part)` (never called on the trace corpus)
pub fn _the_method_in(part: &str) -> Value {
    crate::unported!("bot.modules.answers:_the_method_in")
}

/// Python: bot/modules/answers.py:6876 `_room_they_asked_about(line, asked, room)` (never called on the trace corpus)
pub fn _room_they_asked_about(line: &str, asked: &str, room: &Value) -> Value {
    crate::unported!("bot.modules.answers:_room_they_asked_about")
}

/// Python: bot/modules/answers.py:6898 `_a_place_of_the_same_kind(earlier, text)` (never called on the trace corpus)
pub fn _a_place_of_the_same_kind(earlier: &Value, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_place_of_the_same_kind")
}

/// Python: bot/modules/answers.py:6914 `_a_place_they_asked(text)`
pub fn _a_place_they_asked(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_a_place_they_asked")
}

/// Python: bot/modules/answers.py:6925 `_key_terms(item)` (never called on the trace corpus)
pub fn _key_terms(item: &'static Entry) -> Value {
    crate::unported!("bot.modules.answers:_key_terms")
}

/// Python: bot/modules/answers.py:6947 `_written_for(item, thing)` (never called on the trace corpus)
pub fn _written_for(item: &'static Entry, thing: &str) -> bool {
    crate::unported!("bot.modules.answers:_written_for")
}

/// Python: bot/modules/answers.py:6967 `_more_than_the_thing(item, rest)` (never called on the trace corpus)
pub fn _more_than_the_thing(item: &'static Entry, rest: &str) -> bool {
    crate::unported!("bot.modules.answers:_more_than_the_thing")
}

/// Python: bot/modules/answers.py:6989 `_about_the_rest_of_the_question(item, question, new)` (never called on the trace corpus)
pub fn _about_the_rest_of_the_question(item: &'static Entry, question: &str, new: &Value) -> bool {
    crate::unported!("bot.modules.answers:_about_the_rest_of_the_question")
}

/// Python: bot/modules/answers.py:7124 `_keeping_in_the_answer_before(memory)` (never called on the trace corpus)
pub fn _keeping_in_the_answer_before(memory: &Memory) -> String {
    crate::unported!("bot.modules.answers:_keeping_in_the_answer_before")
}

/// Python: bot/modules/answers.py:7167 `_keeping_reader(reading, memory)`
pub fn _keeping_reader(reading: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_keeping_reader")
}

/// Python: bot/modules/answers.py:7251 `_cooked_thing(memory)`
pub fn _cooked_thing(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_cooked_thing")
}

/// Python: bot/modules/answers.py:7256 `_the_plan_for(text, memory, understood)` (never called on the trace corpus)
pub fn _the_plan_for(text: &str, memory: &Memory, understood: &Understood) -> Value {
    crate::unported!("bot.modules.answers:_the_plan_for")
}

/// Python: bot/modules/answers.py:7297 `_names_another_food(understood, subject)`
pub fn _names_another_food(understood: &Understood, subject: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_another_food")
}

/// Python: bot/modules/answers.py:7334 `_the_food_named(subject)`
pub fn _the_food_named(subject: &str) -> String {
    crate::unported!("bot.modules.answers:_the_food_named")
}

/// Python: bot/modules/answers.py:7340 `_a_cooking_ask_after_a_keeping_answer(understood, memory)`
pub fn _a_cooking_ask_after_a_keeping_answer(understood: &Understood, memory: &Memory) {
    crate::unported!("bot.modules.answers:_a_cooking_ask_after_a_keeping_answer")
}

/// Python: bot/modules/answers.py:7405 `_the_plan_answers_this(memory)`
pub fn _the_plan_answers_this(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_plan_answers_this")
}

/// Python: bot/modules/answers.py:7421 `_the_plan_said_again(understood, text, memory)`
pub fn _the_plan_said_again(understood: &Understood, text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_the_plan_said_again")
}

/// Python: bot/modules/answers.py:7454 `_filling_in_an_emergency(understood, memory)`
pub fn _filling_in_an_emergency(understood: &Understood, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_filling_in_an_emergency")
}

/// Python: bot/modules/answers.py:7469 `_usable(item, reading, memory)`
pub fn _usable(item: &'static Entry, reading: &Understood, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_usable")
}

/// Python: bot/modules/answers.py:7479 `_next_usable(item, reading, memory)` (never called on the trace corpus)
pub fn _next_usable(item: &'static Entry, reading: &Understood, memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_next_usable")
}

/// Python: bot/modules/answers.py:7485 `_without_names(reading)`
pub fn _without_names(reading: &Understood) -> Understood {
    crate::unported!("bot.modules.answers:_without_names")
}

/// Python: bot/modules/answers.py:7570 `_only_the_tone(rest)` (never called on the trace corpus)
pub fn _only_the_tone(rest: &str) -> bool {
    crate::unported!("bot.modules.answers:_only_the_tone")
}

/// Python: bot/modules/answers.py:7604 `_reword_ask(text)`
pub fn _reword_ask(text: &str) {
    crate::unported!("bot.modules.answers:_reword_ask")
}

/// Python: bot/modules/answers.py:7647 `_tone_of(text)` (never called on the trace corpus)
pub fn _tone_of(text: &str) -> Value {
    crate::unported!("bot.modules.answers:_tone_of")
}

/// Python: bot/modules/answers.py:7660 `_softened(sentence, tones)` (never called on the trace corpus)
pub fn _softened(sentence: &str, tones: &Value) -> Value {
    crate::unported!("bot.modules.answers:_softened")
}

/// Python: bot/modules/answers.py:7714 `_last_rewrite(memory)` (never called on the trace corpus)
pub fn _last_rewrite(memory: &Memory) -> Value {
    crate::unported!("bot.modules.answers:_last_rewrite")
}

/// Python: bot/modules/answers.py:7747 `_hard_to_say(asked)` (never called on the trace corpus)
pub fn _hard_to_say(asked: &str) -> Value {
    crate::unported!("bot.modules.answers:_hard_to_say")
}

/// Python: bot/modules/answers.py:7756 `_rewritten(asked, tones, from_followup=False)` (never called on the trace corpus)
pub fn _rewritten(asked: &str, tones: &Value, from_followup: &Value) -> String {
    crate::unported!("bot.modules.answers:_rewritten")
}

/// Python: bot/modules/answers.py:7845 `_asks_about_a_food(low)` (never called on the trace corpus)
pub fn _asks_about_a_food(low: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_about_a_food")
}

/// Python: bot/modules/answers.py:7857 `_someones_else_diet(text, memory, understood)`
pub fn _someones_else_diet(text: &str, memory: &Memory, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_someones_else_diet")
}

/// Python: bot/modules/answers.py:7895 `_a_guest_restriction(text, memory, understood)` (never called on the trace corpus)
pub fn _a_guest_restriction(text: &str, memory: &Memory, understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_a_guest_restriction")
}

/// Python: bot/modules/answers.py:7921 `_names_someone_else(low)`
pub fn _names_someone_else(low: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_someone_else")
}

/// Python: bot/modules/answers.py:7939 `_a_writing_job(understood)`
pub fn _a_writing_job(understood: &Understood) -> bool {
    crate::unported!("bot.modules.answers:_a_writing_job")
}

/// Python: bot/modules/answers.py:7963 `_asks_about_their_taste(text)`
pub fn _asks_about_their_taste(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_about_their_taste")
}

/// Python: bot/modules/answers.py:7968 `_about_them_alone(low)` (never called on the trace corpus)
pub fn _about_them_alone(low: &str) -> bool {
    crate::unported!("bot.modules.answers:_about_them_alone")
}

/// Python: bot/modules/answers.py:7982 `_health_entries()`
pub fn _health_entries() -> Value {
    crate::unported!("bot.modules.answers:_health_entries")
}

/// Python: bot/modules/answers.py:7990 `_names_a_typed_word(item, text)` (never called on the trace corpus)
pub fn _names_a_typed_word(item: &'static Entry, text: &str) -> bool {
    crate::unported!("bot.modules.answers:_names_a_typed_word")
}

/// Python: bot/modules/answers.py:8018 `_circumstances(text)`
pub fn _circumstances(text: &str) -> Vec<String> {
    crate::unported!("bot.modules.answers:_circumstances")
}

/// Python: bot/modules/answers.py:8041 `_detail_frame(part)` (never called on the trace corpus)
pub fn _detail_frame(part: &str) -> String {
    crate::unported!("bot.modules.answers:_detail_frame")
}

/// Python: bot/modules/answers.py:8045 `_a_figure_the_answer_has(text, memory)`
pub fn _a_figure_the_answer_has(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_a_figure_the_answer_has")
}

/// Python: bot/modules/answers.py:8054 `_a_detail_answered(text, memory)`
pub fn _a_detail_answered(text: &str, memory: &Memory) {
    crate::unported!("bot.modules.answers:_a_detail_answered")
}

/// Python: bot/modules/answers.py:8102 `_a_complaint_answered(text, understood, memory)`
pub fn _a_complaint_answered(text: &str, understood: &Understood, memory: &mut Memory) {
    crate::unported!("bot.modules.answers:_a_complaint_answered")
}

/// Python: bot/modules/answers.py:8134 `_a_dish_was_offered(memory)`
pub fn _a_dish_was_offered(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_dish_was_offered")
}

/// Python: bot/modules/answers.py:8147 `observe(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn observe(text: &str, memory: &mut Memory) {
    crate::unported!("bot.modules.answers:observe")
}

/// Python: bot/modules/answers.py:8178 `_asks_only_the_length(text)`
pub fn _asks_only_the_length(text: &str) -> bool {
    crate::unported!("bot.modules.answers:_asks_only_the_length")
}

/// Python: bot/modules/answers.py:8187 `TAKES_AS_IT_COMES(text, memory=None)`
pub fn TAKES_AS_IT_COMES(text: &str, memory: &mut Memory) -> bool {
    crate::unported!("bot.modules.answers:TAKES_AS_IT_COMES")
}

/// Python: bot/modules/answers.py:8201 `claims_instead(text, memory=None)` (never called on the trace corpus)
pub fn claims_instead(text: &str, memory: &mut Memory) -> Option<bool> {
    crate::unported!("bot.modules.answers:claims_instead")
}

/// Python: bot/modules/answers.py:8224 `_a_limit_on_the_plan(text, memory)`
pub fn _a_limit_on_the_plan(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_limit_on_the_plan")
}

/// Python: bot/modules/answers.py:8250 `_a_bare_thanks(understood)`
pub fn _a_bare_thanks(understood: Option<&Understood>) -> bool {
    crate::unported!("bot.modules.answers:_a_bare_thanks")
}

/// Python: bot/modules/answers.py:8265 `_a_judgement_of_the_figure_on_the_table(text, memory)`
pub fn _a_judgement_of_the_figure_on_the_table(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_a_judgement_of_the_figure_on_the_table")
}

/// Python: bot/modules/answers.py:8282 `_the_bot_has_something_to_go_over(memory)` (never called on the trace corpus)
pub fn _the_bot_has_something_to_go_over(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_bot_has_something_to_go_over")
}

/// Python: bot/modules/answers.py:8315 `_the_line_asks_what_this_one_meant(memory)`
pub fn _the_line_asks_what_this_one_meant(memory: &Memory) -> bool {
    crate::unported!("bot.modules.answers:_the_line_asks_what_this_one_meant")
}

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

