//! Port of `bot/modules/words.py`: the words the last answer quoted, handed back when the person asks what to say.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.words.", stringify!($name))));
    )*};
}
rx!(ASKS_WORDS, QUOTED, EXAMPLED, ASK_OBJECT, ASK_FILLER);
static LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.words.LINES"));
static A_SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"[?!.]$", 0));
static NOT_A_LETTER_OR_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9]", 0));

/// Python: bot/modules/words.py `_quoted(said)`
///
/// The first quoted sentence of three words or more in the bot's last reply, with its marks. A
/// phrase the answer only named as a sort of message is skipped.
pub fn _quoted(said: &str) -> String {
    for m in QUOTED.finditer(said) {
        let words = text::strip(m.groups().into_iter().flatten().find(|group| !group.is_empty()).unwrap_or(""));
        if text::split(words).len() < 3 {
            continue;
        }
        // `text[max(0, match.start() - 18):match.start()]`, in characters
        let upto = &said[..m.start()];
        let before = &upto[text::byte_at(upto, upto.chars().count().saturating_sub(18))..];
        if EXAMPLED.is_match(before) && !A_SENTENCE_END.is_match(words) && !words.contains(',') {
            continue; // a sort of message, not one to copy
        }
        return text::strip(m.as_str()).to_string();
    }
    String::new()
}

/// Python: bot/modules/words.py `_names_its_own(line)`
///
/// True when the line asks for the words *and* says what they are for. "what should i say to
/// him" is the ask and nothing else; "how do i say sorry" carries its own subject.
pub fn _names_its_own(line: &str) -> bool {
    let Some(m) = ASKS_WORDS.search(line) else { return true };
    let mut rest = text::strip(&line[m.end()..]).to_string();
    for _step in 0..2 {
        rest = text::strip_chars(&rest, " ,.-").to_string();
        if let Some(end) = ASK_OBJECT.match_(&rest).map(|moved| moved.end()) {
            rest = text::strip(&rest[end..]).to_string();
        }
        for _ in 0..3 {
            let Some(end) = ASK_FILLER.match_(&rest).map(|filler| filler.end()) else { break };
            rest = text::strip(&rest[end..]).to_string();
        }
    }
    !NOT_A_LETTER_OR_DIGIT.sub("", &rest.to_lowercase()).is_empty()
}

/// Python: bot/modules/words.py `_the_words(text, memory)`
///
/// The quoted words this line is asking for, or "" when this line is not that ask.
pub fn _the_words(text: &str, memory: &Memory) -> String {
    let line = text::strip(text);
    if line.is_empty() || text::split(line).len() > 14 || !ASKS_WORDS.is_match(line) {
        return String::new();
    }
    if _names_its_own(line) {
        return String::new(); // "how do i say sorry" names its own question and has its own answer
    }
    _quoted(&last_bot(memory))
}

/// Python: bot/modules/words.py `TAKES_AS_IT_COMES(text, memory)`
///
/// This line is asking for the words the last answer already gave, so it is this module's.
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    !_the_words(text, memory).is_empty()
}

/// Python: bot/modules/words.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let line = text::strip(text);
    let words = _the_words(text, memory);
    if words.is_empty() {
        return None;
    }
    // The quote keeps the punctuation the answer gave it; a sentence that opens inside a longer
    // one ("Hi - the boiler has stopped working, please let me know when an engineer can come")
    // is handed back as the answer wrote it, in the same marks.
    Some(fmt_named(&pick(&LINES, memory, line), &[("words", words.as_str())]))
}
