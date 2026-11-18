//! Port of `bot/memory/quotes.py`: sentences cut with a quotation or a name ("No. 9") kept in
//! one piece.

use super::mem8_py::rx;
use crate::data::Value;
use crate::py::text;

const M: &str = "bot.memory.quotes.";

rx!(NAME_END, OPENS_ON_A_FIGURE);

/// Python: bot/memory/quotes.py:27 `left_open(said)`
///
/// True when `said` opens a quotation it does not close.
pub fn left_open(said: &str) -> bool {
    if text::count(said, "\"") % 2 == 1 {
        return true;
    }
    text::count(said, "\u{201c}") > text::count(said, "\u{201d}")
}

/// Python: bot/memory/quotes.py:39 `closed(said, rest, whole='')`
///
/// `said` carried on through `rest` until the quotation in it closes; `whole` (or `said`) when
/// it never closes.
pub fn closed(said: &str, rest: &[String], whole: &str) -> String {
    let mut said = text::strip(said).to_string();
    if !left_open(&said) {
        return said;
    }
    for part in rest {
        said = text::strip(&format!("{} {}", said, text::strip(part))).to_string();
        if !left_open(&said) {
            return said;
        }
    }
    text::strip(if whole.is_empty() { &said } else { whole }).to_string()
}

/// Python: bot/memory/quotes.py:66 `a_name(text, start, end)`
///
/// True when the full stop ending `text[start:end]` is part of a name rather than a sentence.
/// (`start` and `end` are byte offsets here.)
pub fn a_name(text: &str, start: usize, end: usize) -> bool {
    let before = text::strip(&text[start..end.max(start)]);
    if !before.ends_with('.') {
        return false;
    }
    if !NAME_END.is_match(before) {
        return false;
    }
    let after = text::lstrip(&text[end..]);
    OPENS_ON_A_FIGURE.match_(after).is_some()
}

/// Python: bot/memory/quotes.py:77 `sentences(text, splitter)`
///
/// `splitter`'s sentences, with a quotation or a name kept in one piece. `splitter` is a
/// compiled pattern; anything else is split by its own `.split(text)`, as in Python.
pub fn sentences(text: &str, splitter: &Value) -> Vec<String> {
    let Some(splitter) = splitter.as_re() else {
        // `[part.strip() for part in splitter.split(text) if str(part).strip()]` on a str.
        let Value::Str(s) = splitter else { panic!("AttributeError: {splitter:?} has no split") };
        if text.is_empty() {
            panic!("ValueError: empty separator");
        }
        return text::split_on(s, text).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    };
    let mut parts: Vec<String> = Vec::new();
    let mut start = 0;
    for end in splitter.finditer(text) {
        if a_name(text, start, end.start()) {
            continue;
        }
        parts.push(text::strip(&text[start..end.start().max(start)]).to_string());
        start = end.end();
    }
    parts.push(text::strip(&text[start..]).to_string());
    let mut out: Vec<String> = Vec::new();
    for part in parts.into_iter().filter(|p| !p.is_empty()) {
        match out.last_mut() {
            Some(last) if left_open(last) => *last = text::strip(&format!("{last} {part}")).to_string(),
            _ => out.push(part),
        }
    }
    out
}
