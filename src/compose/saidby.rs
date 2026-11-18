//! Port of `bot/compose/saidby.py`: somebody else's wish or hard words, quoted to the bot ("my
//! wife said \"i want a divorce\""), answered with a question about the person holding them.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, format_named, inline_re};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.saidby"; QUOTED, SAID, SAID_TO_ME, A_WISH, A_DECLARATION, NOT_SPOKEN_TO_THEM, OWN_SAYING, ASKING_ABOUT_IT,
    WHO);
bundle_sets!("bot.compose.saidby"; OWN_SUBJECT, OWN_FILLER, A_PRONOUN_HEAD);
bundle_strs!("bot.compose.saidby"; THEY_SAID, THEY_SAID_AGAIN, THEY_DECLARED);
inline_re!(SAID_AGAIN, concat!(r"\b(?:keeps?\s+(?:saying|shouting|on)|kept\s+(?:saying|shouting|on)|always\s+says|",
    r"says\s+it\s+again|over\s+and\s+over)\b"));

/// Python: bot/compose/saidby.py:120 `the_quotation(text)`
///
/// The first sentence somebody said, or "" when the line quotes nobody.
pub fn the_quotation(text_: &str) -> String {
    let Some(found) = QUOTED.search(text_) else { return String::new() };
    let body = found.get(1).filter(|s| !s.is_empty()).or(found.get(2)).unwrap_or("");
    text::squash(body)
}

/// Python: bot/compose/saidby.py:128 `_the_one_who_said_it(text)`
///
/// The words for the person who said it: "my wife", "he", or "" when nobody is named.
pub fn _the_one_who_said_it(text_: &str) -> String {
    match SAID.search(text_) {
        Some(found) => text::squash(&found.name_str("who").to_lowercase()),
        None => String::new(),
    }
}

/// Python: bot/compose/saidby.py:139 `_before_the_quotation(text)`
pub fn _before_the_quotation(text_: &str) -> String {
    QUOTED.search(text_).map(|f| text_[..f.start()].to_string()).unwrap_or_default()
}

/// Python: bot/compose/saidby.py:145 `said_by_the_person(text)`
///
/// True when the quotation is the person's own message: the verb of saying's own subject
/// decides it.
pub fn said_by_the_person(text_: &str) -> bool {
    let before = _before_the_quotation(text_).to_lowercase();
    let words: Vec<&str> = text::split_ref(&before).into_iter().map(|w| text::strip_chars(w, ",.!?;:")).collect();
    for index in (0..words.len()).rev() {
        if OWN_SAYING.fullmatch(words[index]).is_none() {
            continue;
        }
        for word in words[..index].iter().rev() {
            if OWN_FILLER.contains(*word) {
                continue;
            }
            return OWN_SUBJECT.contains(*word);
        }
        return false;
    }
    false
}

/// Python: bot/compose/saidby.py:165 `reads_as_somebody_elses_wish(text)`
///
/// True when the line brings somebody else's wish or hard words, quoted, to the bot.
pub fn reads_as_somebody_elses_wish(text_: &str) -> bool {
    let said = text::squash(&text_.to_lowercase());
    if said.is_empty() || (!said.contains('"') && !said.contains('\u{201c}')) {
        return false;
    }
    let quoted = the_quotation(&said);
    if quoted.is_empty() || text::split_ref(&quoted).len() < 2 {
        return false;
    }
    if A_WISH.search(&quoted).is_none() && A_DECLARATION.search(&quoted).is_none() {
        return false;
    }
    if NOT_SPOKEN_TO_THEM.search(&said).is_some() || ASKING_ABOUT_IT.search(&said).is_some() {
        return false;
    }
    if said_by_the_person(&said) {
        return false;
    }
    SAID.search(&said).is_some() || SAID_TO_ME.search(&said).is_some()
}

/// Python: bot/compose/saidby.py:187 `_who_phrase(text)`
///
/// The person as the subject of the reply when the head is a pronoun, else "".
pub fn _who_phrase(text_: &str) -> String {
    let who = _the_one_who_said_it(text_);
    match text::split_ref(&who).first() {
        Some(head) if A_PRONOUN_HEAD.contains(*head) => text::upper_first(&who),
        _ => String::new(),
    }
}

/// Python: bot/compose/saidby.py:195 `_fits_the_head(line, who)`
///
/// Drop the lines that open on a person the line did not name that way.
pub fn _fits_the_head(line: &str, who: &str) -> bool {
    !line.contains("{Who}") || !who.is_empty()
}

/// Python: bot/compose/saidby.py:200 `_the_card_keeps_its_turn(text, memory)`
///
/// True when the knowledge base has a card for the line, which then keeps its turn: the answers
/// module is asked, with this memory (a card that raises answers nothing).
pub fn _the_card_keeps_its_turn(text_: &str, memory: &mut Memory) -> bool {
    if text_.is_empty() {
        return false;
    }
    com4::attempt(|| crate::modules::answers::respond(text_, memory).is_some_and(|r| !r.is_empty())).unwrap_or(false)
}

/// Python: bot/compose/saidby.py:222 `reply_for(text, memory=None)`
///
/// One reply that asks about the person holding somebody else's words, or None.
pub fn reply_for(text_: &str, memory: &mut Memory) -> Option<String> {
    if !reads_as_somebody_elses_wish(text_) {
        return None;
    }
    if _the_card_keeps_its_turn(text_, memory) {
        return None;
    }
    let said = text::squash(&text_.to_lowercase());
    let who = _who_phrase(&said);
    let quoted = the_quotation(&said);
    let pool: Vec<String> = if A_DECLARATION.search(&quoted).is_some() && A_WISH.search(&quoted).is_none() {
        THEY_DECLARED.clone()
    } else {
        let lines: &Vec<String> = if SAID_AGAIN.search(&said).is_some() { &THEY_SAID_AGAIN } else { &THEY_SAID };
        lines.iter().filter(|l| _fits_the_head(l, &who)).cloned().collect()
    };
    let options: Vec<String> = pool.iter().map(|l| format_named(l, &[("Who", &who)])).collect();
    Some(pick(&options, memory, &said))
}

#[allow(dead_code)]
fn _who() -> &'static crate::py::re::Re {
    &WHO
}
