//! Port of `bot/memory/outfit.py`: what to wear, said about the occasion already on the table.
//!
//! "my sister is getting married in june" / "im not sure what to wear" / "and what about shoes":
//! written out with the occasion in the words an entry is keyed on ("what should i wear to my
//! sister's wedding") the lines reach the outfit answer. `written_out` only fires when the line
//! is about what to wear and names no occasion of its own.
#![allow(non_snake_case, clippy::all)]

use crate::memory::mem7::{self, inline_re, res, values};
use crate::py::re::{self, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::BTreeSet;
use std::sync::LazyLock;

res!("outfit"; RELATION, WHAT_TO_WEAR, PART, PART_ASK, PUT_ON_A_THING, PAST_OCCASION, _A_DEATH, CLOTHING_WORDS, PERMISSION_ASK, NOT_THIS, A_COLOUR, GOING_SOMEWHERE);
values!("outfit"; RECENT);
inline_re! {
    SENTENCE_END = r"(?<=[.!?])\s+", 0;
}

/// `OCCASIONS`: (the pattern, its key, the occasion as a thing), in order.
static OCCASIONS: LazyLock<Vec<(&'static Re, String, String)>> = LazyLock::new(|| {
    crate::data::get("bot.memory.outfit.OCCASIONS")
        .items()
        .iter()
        .map(|one| {
            let parts = one.items();
            (parts[0].as_re().expect("an occasion pattern"), parts[1].str().to_string(), parts[2].str().to_string())
        })
        .collect()
});

/// Python: bot/memory/outfit.py:107 `_recent(memory)`
///
/// The last few things the person said, newest first, each squashed and lower-case.
pub fn _recent(memory: &Memory) -> Vec<String> {
    let recent = RECENT.as_int().unwrap_or(5) as usize;
    mem7::tail(&memory.turns, recent)
        .iter()
        .rev()
        .map(|turn| pt::squash(&pt::lower(&turn.user)))
        .filter(|text| !text.is_empty())
        .collect()
}

/// Python: bot/memory/outfit.py:123 `_an(phrase)`
///
/// The article a word takes: "an interview", "a wedding".
pub fn _an(phrase: &str) -> String {
    // `phrase[:1] in "aeiou"`: an empty phrase is in every string.
    let first = mem7::first_chars(phrase, 1);
    format!("{}{}", if "aeiou".contains(first) { "an " } else { "a " }, phrase)
}

/// Python: bot/memory/outfit.py:142 `the_occasion(memory)`
///
/// The occasion the chat is on, as "a wedding" or "my sister's wedding", or None.
///
/// CONTRACT: the skeleton typed memory `&Value` and gave no return type; it is the chat's
/// memory, and PyBot returns the occasion or None.
pub fn the_occasion(memory: &Memory) -> Option<String> {
    for text in _recent(memory) {
        for (pattern, _key, phrase) in OCCASIONS.iter() {
            if !pattern.is_match(&text) {
                continue;
            }
            if PAST_OCCASION.is_match(&text) && !_A_DEATH.is_match(&text) {
                continue;
            }
            if let Some(who) = RELATION.search(&text) {
                // Only the person's own relation in the same line: her wedding.
                let last = pt::split_ref(phrase).last().copied().unwrap_or("");
                return Some(format!("my {}'s {}", who.str(1), last));
            }
            return Some(phrase.clone());
        }
    }
    None
}

/// Python: bot/memory/outfit.py:161 `_the_last_answer(memory)`
///
/// What the bot said last, lower-case: the answer a part of the outfit is asked about.
pub fn _the_last_answer(memory: &Memory) -> String {
    memory.turns.last().map_or(String::new(), |turn| pt::squash(&pt::lower(&turn.bot)))
}

/// Python: bot/memory/outfit.py:176 `_the_answer_already_gave_it(memory)`
///
/// True when the bot's last reply was already the outfit answer for this occasion.
pub fn _the_answer_already_gave_it(memory: &Memory) -> bool {
    let last = _the_last_answer(memory);
    if last.is_empty() {
        return false;
    }
    let words: BTreeSet<String> = CLOTHING_WORDS.findall_str(&last).iter().map(|word| pt::lower(word)).collect();
    words.len() >= 2
}

/// `\b` + re.escape(word) + `\b`.
fn whole_word(word: &str) -> Re {
    Re::new(&format!(r"\b{}\b", re::escape(word)), 0)
}

/// " ".join(text.lower().split()).strip(" .!,")
fn the_line(text: &str) -> String {
    pt::strip_chars(&pt::squash(&pt::lower(text)), " .!,").to_string()
}

/// Python: bot/memory/outfit.py:220 `the_sentence_answered(text, memory)`
///
/// The sentence of the bot's last answer that answers a follow-up about one thing, or None.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the sentence or None.
pub fn the_sentence_answered(text: &str, memory: &Memory) -> Option<String> {
    let line = the_line(text);
    if line.is_empty() || mem7::word_count(&line) > 12 {
        return None;
    }
    the_occasion(memory)?;
    if !(PART.is_match(&line) || A_COLOUR.is_match(&line)) {
        return None;
    }
    if !(PART_ASK.is_match(&line) || PERMISSION_ASK.is_match(&line)) {
        return None;
    }
    let last = _the_last_answer(memory);
    if last.is_empty() || mem7::word_count(&last) < 6 {
        return None; // a punt or a one-word line is not a card with an answer in it
    }
    let sentences: Vec<String> = SENTENCE_END.split(&last).into_iter().filter(|one| !pt::strip(one).is_empty()).collect();
    if sentences.len() < 2 {
        // One sentence is the whole answer, so handing it back is the bot saying it again.
        return None;
    }
    // The colour is asked about more closely than the thing.
    let colours: BTreeSet<String> = A_COLOUR.findall_str(&line).into_iter().collect();
    let parts: BTreeSet<String> = PART.findall_str(&line).into_iter().collect();
    for word in colours.into_iter().chain(parts) {
        let pattern = whole_word(&word);
        if !pattern.is_match(&last) {
            continue;
        }
        for sentence in &sentences {
            if !pattern.is_match(sentence) {
                continue;
            }
            let mut sentence = pt::strip(sentence).to_string();
            if sentence.is_empty() {
                continue;
            }
            if sentence.chars().next().is_some_and(char::is_alphabetic) {
                sentence = pt::upper_first(&sentence);
            }
            // A sentence that warns against the thing answers on its own; one that says the
            // thing is fine is said as the yes it is.
            let lead = if NOT_THIS.is_match(&sentence) {
                "Careful - "
            } else if NOT_THIS.is_match(&line) {
                "Then "
            } else {
                "Yes - "
            };
            return Some(format!("{lead}{}", mem7::low1(&sentence)));
        }
    }
    None
}

/// Python: bot/memory/outfit.py:281 `_names_an_occasion(text)`
pub fn _names_an_occasion(text: &str) -> bool {
    OCCASIONS.iter().any(|(pattern, _, _)| pattern.is_match(text))
}

/// Python: bot/memory/outfit.py:302 `_names_its_own_outing(text)`
///
/// Whether the ask goes somewhere of its own, so the chat's occasion is not what it is about.
pub fn _names_its_own_outing(text: &str) -> bool {
    GOING_SOMEWHERE.is_match(text) && !_names_an_occasion(text)
}

/// Python: bot/memory/outfit.py:310 `written_out(text, memory)`
///
/// The outfit question the line is asking, or None when the line is not one.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the question or None.
pub fn written_out(text: &str, memory: &Memory) -> Option<String> {
    let line = the_line(text);
    if line.is_empty() || mem7::word_count(&line) > 12 {
        return None;
    }
    if let Some(worn) = PUT_ON_A_THING.search(&line) {
        if !CLOTHING_WORDS.is_match(worn.str(1)) {
            return None;
        }
    }
    let shaped = WHAT_TO_WEAR.is_match(&line) || (PART.is_match(&line) && PART_ASK.is_match(&line));
    if !shaped {
        return None;
    }
    let occasion = the_occasion(memory)?;
    if _the_answer_already_gave_it(memory) {
        return None;
    }
    // A line that says the occasion itself is asking its own question.
    if _names_an_occasion(&line) || _names_its_own_outing(&line) {
        return None;
    }
    if WHAT_TO_WEAR.is_match(&line) {
        return Some(format!("what should i wear to {occasion}"));
    }
    let part = PART.search(&line)?;
    if PART_ASK.is_match(&line) {
        // A part the answer never mentioned is a question the entry does not answer.
        if !_the_last_answer(memory).contains(part.as_str()) {
            return None;
        }
        return Some(format!("what {} should i wear to {occasion}", part.as_str()));
    }
    None
}
