//! Port of `bot/memory/baking.py`: a line that names a kind of cake is a question about that
//! cake, not about its vegetable.
//!
//! Only an ask takes this route; written out as "how do i make a carrot cake" the line reaches
//! the cake entries, which answer it.

use super::mem8_py::{int, rx};
use crate::py::text;
use crate::state::Memory;

const M: &str = "bot.memory.baking.";

rx!(NAMED_BAKES, BAKE, ASK, TAIL, NAME_WORD, LACKS);
int!(LONGEST);

/// Python: bot/memory/baking.py:45 `_the_name(noun)`
///
/// The name of the bake in the line without the ask around it, or "" when it is not one.
pub fn _the_name(noun: &str) -> String {
    let mut words = text::split_ref(noun);
    while let Some(last) = words.last() {
        if TAIL.match_(&format!(" {last}")).is_none() {
            break;
        }
        words.pop();
    }
    if words.is_empty() {
        return String::new();
    }
    let mut named = words.clone();
    while !named.is_empty() && ["a", "an", "some", "another"].contains(&named[0].to_lowercase().as_str()) {
        named.remove(0);
    }
    if named.is_empty() || named.len() as i64 > *LONGEST {
        return String::new();
    }
    for word in &named {
        if NAME_WORD.match_(text::strip_chars(word, ".,!?;")).is_none() {
            return String::new();
        }
    }
    text::strip_chars(&words.join(" "), ".,!?;:").to_lowercase()
}

/// Python: bot/memory/baking.py:87 `written_out(text, memory=None)`
///
/// The line written out as a how-to about the bake it names, or None to leave it alone.
pub fn written_out(text: &str, _memory: &Memory) -> Option<String> {
    let line = text::squash(text);
    if line.is_empty() || text::split_ref(&line).len() > 12 {
        return None;
    }
    let asked = ASK.match_(&line)?;
    let rest = &line[asked.end()..];
    if !(BAKE.is_match(rest) || NAMED_BAKES.is_match(rest)) {
        return None;
    }
    let noun = _the_name(rest);
    if noun.is_empty() {
        return None;
    }
    if LACKS.is_match(&noun) {
        return None;
    }
    if !(BAKE.is_match(&noun) || NAMED_BAKES.is_match(&noun)) {
        return None;
    }
    Some(format!("how do i make {noun}"))
}
