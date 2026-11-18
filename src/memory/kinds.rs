//! Port of `bot/memory/kinds.py`: the kind of detail a line asks for (the time, the make, the
//! colour, the price ...), and the earlier line of theirs that gives that kind. It reads and
//! never decides a reply; `bot/memory/lineback.py` says the line back.

use super::mem8_py::{rx, strset, val};
use crate::data::Value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.kinds.";

val!(ASKS, GIVES);
rx!(BACK_REF, LEAD, A_RECALL_SAID, A_QUESTION, AN_END, A_START, ASKS_WHAT_TO_DO);
strset!(FRAME, UNNAMED_KINDS);

static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z][a-z'\-]*", 0));

/// Python: bot/memory/kinds.py:170 `_the_other_end(ask, line)`
///
/// Whether the ask and the line are about different ends of the same plan.
pub fn _the_other_end(ask: &str, line: &str) -> bool {
    let asking_end = AN_END.is_match(ask);
    let asking_start = A_START.is_match(ask);
    if asking_end == asking_start {
        return false;
    }
    if asking_end { A_START.is_match(line) } else { AN_END.is_match(line) }
}

/// Python: bot/memory/kinds.py:183 `_flat(text)`
pub fn _flat(text: &str) -> String {
    text::squash(&text.to_lowercase())
}

/// Python: bot/memory/kinds.py:187 `_words(line, length=2)`
///
/// The words of a line worth matching on, stemmed the way people vary them. (`length` is
/// unused in the Python.)
pub fn _words(line: &str, _length: i64) -> HashSet<String> {
    let mut out = HashSet::new();
    for word in A_WORD.findall_str(&_flat(line)) {
        let word = text::strip_chars(&word, "'-").to_string();
        if text::len(&word) < 3 || FRAME.contains(&word) {
            continue;
        }
        if text::len(&word) > 4 && word.ends_with('s') {
            out.insert(word[..word.len() - 1].to_string());
        }
        out.insert(word);
    }
    out
}

/// Python: bot/memory/kinds.py:200 `_mentions(line, wanted)`
///
/// Whether the line names one of the words, in the shape a line would hold it.
pub fn _mentions(line: &str, wanted: &HashSet<String>) -> bool {
    let have = _words(line, 2);
    let flat = _flat(line);
    for word in wanted {
        if have.contains(word) || flat.contains(word.as_str()) {
            return true;
        }
        if text::len(word) > 4 && have.contains(&text::slice(word, 0, Some(-1))) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/kinds.py:212 `_their_lines(memory)`
///
/// Every line of theirs in the chat, oldest first, without their own questions.
pub fn _their_lines(memory: &Memory) -> Vec<String> {
    let mut out = Vec::new();
    for turn in &memory.turns {
        let line = text::squash(&turn.user);
        if line.is_empty() || A_QUESTION.match_(&line).is_some() {
            continue;
        }
        out.push(line);
    }
    out
}

/// Python: bot/memory/kinds.py:237 `the_kind_asked(text)`
///
/// The kinds of detail the line asks for, in the order the table reads them.
pub fn the_kind_asked(text: &str) -> Vec<String> {
    let low = _flat(text);
    ASKS.items()
        .iter()
        .filter(|pair| pair.items()[1].as_re().expect("ASKS pattern").is_match(&low))
        .map(|pair| pair.items()[0].str().to_string())
        .collect()
}

/// Python: bot/memory/kinds.py:243 `the_line_that_gives(text, lines)`
///
/// The earlier line of theirs that holds the kind of detail the line asks for, as
/// `{"line": ..., "kind": ...}`, or None.
pub fn the_line_that_gives(text: &str, lines: &[String]) -> Option<Value> {
    let kinds = the_kind_asked(text);
    if kinds.is_empty() {
        return None;
    }
    let lines: Vec<&String> = lines.iter().filter(|l| !l.is_empty() && A_QUESTION.match_(l).is_none()).collect();
    if lines.is_empty() {
        return None;
    }
    let low = LEAD.sub("", &_flat(text));
    // A statement is not an ask.
    if A_QUESTION.match_(&low).is_none() {
        return None;
    }
    // A line that asks what to do wants advice, not a detail they gave.
    if ASKS_WHAT_TO_DO.is_match(&low) {
        return None;
    }
    // What the line names, besides its own frame: "what make is the boiler" names the boiler.
    let named: HashSet<String> =
        A_WORD.findall_str(&low).into_iter().filter(|w| !FRAME.contains(w) && text::len(w) > 2).collect();
    let asking_again = A_RECALL_SAID.is_match(&low);
    if named.is_empty() && !asking_again && !kinds.iter().any(|k| UNNAMED_KINDS.contains(k)) {
        return None;
    }
    let found = |line: &str, kind: &str| {
        let mut d = Value::dict();
        d.set_key("line", line);
        d.set_key("kind", kind);
        Some(d)
    };
    for kind in &kinds {
        let Some(gives) = GIVES.get(kind).and_then(Value::as_re) else { continue };
        if named.is_empty() && !asking_again && !UNNAMED_KINDS.contains(kind) {
            continue;
        }
        for index in (0..lines.len()).rev() {
            let line = lines[index];
            if !gives.is_match(line) {
                continue;
            }
            if _the_other_end(&low, line) {
                continue;
            }
            // The line names nothing of its own: any earlier line that gives the kind is the one.
            if named.is_empty() || _mentions(line, &named) {
                return found(line, kind);
            }
            // A detail line is about the thing the line before it named.
            if index > 0 && BACK_REF.match_(line).is_some() && _mentions(lines[index - 1], &named) {
                return found(line, kind);
            }
        }
    }
    None
}
