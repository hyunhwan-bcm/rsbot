//! Port of `bot/memory/filler.py`: the chat-speak people hang off the end of a line, and the
//! line without it.

use super::mem8_py::{strs, strset};
use crate::py::text;

const M: &str = "bot.memory.filler.";

strset!(TAIL_OFF, IN_THE_WAY);
strs!(TAIL_PHRASES, ASK_TAIL);

/// The loop both end readers share: drop one of `phrases` from the end of `text` while one is
/// there and something is left in front of it.
fn without_tail_phrases(mut text_: String, phrases: &[String]) -> String {
    let mut changed = true;
    while changed && !text_.is_empty() {
        changed = false;
        let stripped = text::rstrip_chars(&text_, " .,;:!?").to_string();
        let low = stripped.to_lowercase();
        for phrase in phrases {
            if low.ends_with(&format!(" {phrase}")) || low == *phrase {
                let keep = text::len(&stripped) as isize - text::len(phrase) as isize;
                let head = text::slice(&stripped, 0, Some(keep));
                let cut = text::rstrip_chars(&head, " ,;:-");
                if !cut.is_empty() {
                    text_ = text::squash(cut);
                    changed = true;
                }
                break;
            }
        }
    }
    text_
}

/// Python: bot/memory/filler.py:31 `without_filler(said)`
///
/// `said` without the chat-speak hanging off the end: "im veggie tho" -> "im veggie".
pub fn without_filler(said: &str) -> String {
    let mut words = text::split_ref(said);
    while words.len() > 1 && TAIL_OFF.contains(text::strip_chars(&words[words.len() - 1].to_lowercase(), ".,;:!?")) {
        words.pop();
    }
    // The phrases go last and all at once, so the words of a shrug that is a phrase are never
    // read as words of the line.
    without_tail_phrases(words.join(" "), &TAIL_PHRASES)
}

/// Python: bot/memory/filler.py:65 `without_the_end_shrug(said)`
///
/// `said` without the shrug hung on the end of an ask: "how old is my son by the way".
pub fn without_the_end_shrug(said: &str) -> String {
    // sorted(ASK_TAIL, key=len, reverse=True): longest first, ties in their written order.
    let mut phrases: Vec<String> = ASK_TAIL.clone();
    phrases.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    without_tail_phrases(text::squash(said), &phrases)
}

/// Python: bot/memory/filler.py:83 `without_the_shrug(said)`
///
/// `said` without the fillers in the middle of it: "what can i actually do" -> "what can i do".
/// One word is always left, so a line that was nothing but a shrug comes back as it was.
pub fn without_the_shrug(said: &str) -> String {
    let words = text::split_ref(said);
    let kept: Vec<&str> = words
        .iter()
        .copied()
        .filter(|word| !IN_THE_WAY.contains(text::strip_chars(&word.to_lowercase(), ".,;:!?")))
        .collect();
    if kept.is_empty() { words.join(" ") } else { kept.join(" ") }
}
