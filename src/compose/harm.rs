//! Port of `bot/compose/harm.py`: a line that says they want to hurt somebody ("i want to punch
//! my brother"), answered about the anger rather than the plan, and with the safe thing first
//! when it is their own child.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_str, bundle_strs, format_named, inline_re};
use crate::compose::pick;
use crate::py::re::Match;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.harm"; _HURT, _PERSON, _THE_PERSON, _IDIOM, _WISH, _FEELINGS, A_SMALL_CHILD, A_CHILD, _IN_LAW,
    OUTBURST, OUTBURST_WALL, OUTBURST_VERB);
bundle_sets!("bot.compose.harm"; _BARE_LEAD, _JOINERS, _BODY);
bundle_strs!("bot.compose.harm"; POOL, BABY_LINES, CHILD_LINES);
bundle_str!("bot.compose.harm"; NAMED);
inline_re!(THEIR_OWN_LIFE, r"\bkill(?:ing)?\s+(?:my|your|him|her)sel(?:f|ves)?\b|\bkill(?:ing)?\s+me\b");
inline_re!(A_DENIAL, concat!(r"\b(?:dont|don'?t|do not|never|no longer|wouldnt|wouldn'?t|couldnt|couldn'?t|",
    r"cant|can'?t|wont|won'?t)\b(?:\s+\w+){0,4}\s*$"));
inline_re!(SOMEBODY_ELSE_DOING_IT, concat!(r"\b(?:he|she|they|we|you|his|her|their|its)\b(?:\s+\w+){0,3}\s+",
    r"(?:wants?|wanted|is|was|keeps|kept|might|may|will|would)\s*$"));
inline_re!(THE_SPEAKER, r"\b(?:i|i'm|im|i'd|id|ive)\b");
inline_re!(A_WORD, r"[a-z']+");

/// Python: bot/compose/harm.py:171 `_person_said(clean)`
///
/// "your boss" - the person they are angry with, said in the second person. The one after the
/// verb first, then the whole line.
pub fn _person_said(clean: &str) -> String {
    let t = text::squash(&clean.to_lowercase());
    let mut spans: Vec<&str> = Vec::new();
    if let Some(hurt) = _HURT.search(&t) {
        spans.push(&t[hurt.start()..]);
    }
    spans.push(&t);
    for span in spans {
        let said = _said_back(span);
        if !said.is_empty() {
            return said;
        }
    }
    String::new()
}

/// Python: bot/compose/harm.py:191 `_said_back(text)`
///
/// The person in this piece of the line, said in the second person, or "".
pub fn _said_back(text_: &str) -> String {
    let Some(found) = _THE_PERSON.search(text_) else { return String::new() };
    if !_stands_on_its_own(text_, &found) {
        // "i want to kill my son's teacher" names the teacher, not the son.
        return String::new();
    }
    let mut said = text::strip(found.str(1)).to_string();
    let (head, _, rest) = text::partition(&said, " ");
    if _BODY.contains(rest) {
        // "i want to smash his face in": a face cannot have worn them down.
        return String::new();
    }
    if head == "the" || head == "that" {
        // "the man who hurt my daughter" is somebody, but not somebody they named.
        return String::new();
    }
    let swap = match head {
        "my" | "our" => Some("your"),
        "his" | "her" | "its" => Some("their"),
        _ => None,
    };
    if let Some(swap) = swap {
        said = text::strip(&format!("{swap} {rest}")).to_string();
    } else if said == "him" || said == "her" {
        said = "them".to_string();
    }
    if matches!(said.as_str(), "their" | "them" | "your") {
        // A determiner with nothing after it is no person at all.
        return String::new();
    }
    said
}

/// Python: bot/compose/harm.py:225 `reads_as_hurting_someone(clean)`
///
/// True when the line says they want to hurt a person. A question is not this, and neither is
/// the same verb said of somebody else.
pub fn reads_as_hurting_someone(clean: &str) -> bool {
    let t = text::squash(&clean.to_lowercase().replace('\u{2019}', "'"));
    if t.is_empty() || t.contains('?') {
        return false;
    }
    if _IDIOM.search(&t).is_some() {
        return false;
    }
    if THEIR_OWN_LIFE.search(&t).is_some() {
        return false; // their own life is the safety module's line, and it answers it first
    }
    let Some(hurt) = _HURT.search(&t) else { return false };
    if _FEELINGS.search(&t).is_some() {
        return false; // "i want to hurt my son's feelings" is not a body
    }
    if _PERSON.search(&t[hurt.start()..]).is_none() {
        // Nothing to aim at is still the same anger: "i want to punch something".
        let tail = text::slice(&t[hurt.start()..], 0, Some(40));
        if !(OUTBURST.search(&tail).is_some() || (OUTBURST_WALL.search(&tail).is_some() && OUTBURST_VERB.search(&tail).is_some())) {
            return false;
        }
    }
    let opening = text::strip_chars(&t[..hurt.start()], " ,;:-\u{2013}\u{2014}");
    if A_DENIAL.search(opening).is_some() {
        return false; // "i dont want to hurt him any more" is a wish for it to stop
    }
    // Somebody else doing it is a story, not their wish: "she wants to kill her boss".
    if SOMEBODY_ELSE_DOING_IT.search(opening).is_some() {
        return false;
    }
    if THE_SPEAKER.search(opening).is_some() {
        // A wish has its frame: "i want to", "i could", "im going to".
        return _WISH.search(opening).is_some();
    }
    // The bare form is an instruction ("punch your brother").
    opening.is_empty() || text::split_ref(opening).iter().all(|w| _BARE_LEAD.contains(*w))
}

/// Python: bot/compose/harm.py:271 `reads_as_our_reply(reply)`
///
/// Whether this reply is one of the lines from here (the named line by its two fixed halves).
pub fn reads_as_our_reply(reply: &str) -> bool {
    let t = text::squash(reply);
    if t.is_empty() {
        return false;
    }
    if POOL.iter().chain(BABY_LINES.iter()).chain(CHILD_LINES.iter()).any(|l| *l == t) {
        return true;
    }
    let (head, _, tail) = text::partition(&NAMED, "{person}");
    !head.is_empty()
        && !tail.is_empty()
        && t.starts_with(head)
        && t.ends_with(tail)
        && text::len(&t) > text::len(head) + text::len(tail)
}

/// Python: bot/compose/harm.py:288 `_child_at_the_hurt(flat)`
///
/// "small" or "child" when the doing is aimed at their own child, else "".
pub fn _child_at_the_hurt(flat: &str) -> String {
    let Some(hurt) = _HURT.search(flat) else { return String::new() };
    let tail = &flat[hurt.start()..];
    let Some(first) = _PERSON.search(tail) else { return String::new() };
    let phrase = first.as_str();
    if _IN_LAW.match_(&tail[first.end()..]).is_some() {
        return String::new(); // "my son in law" is a person to name back, not their child
    }
    if !_stands_on_its_own(tail, &first) {
        return String::new();
    }
    if A_SMALL_CHILD.search(phrase).is_some() {
        return "small".to_string();
    }
    if A_CHILD.search(phrase).is_some() { "child".to_string() } else { String::new() }
}

/// Python: bot/compose/harm.py:313 `_stands_on_its_own(text, found)`
///
/// Whether the person phrase is the whole person, not the start of a longer noun phrase.
pub fn _stands_on_its_own(text_: &str, found: &Match) -> bool {
    let rest = text::lstrip_chars(&text_[found.end()..], " ,.;:!?-");
    if rest.is_empty() {
        return true;
    }
    match A_WORD.match_(rest) {
        Some(word) => _JOINERS.contains(word.as_str()),
        None => false,
    }
}

/// Python: bot/compose/harm.py:322 `reply_for(clean, memory=None)`
///
/// The reply to a line that says they want to hurt somebody, or None.
pub fn reply_for(clean: &str, memory: &Memory) -> Option<String> {
    if !reads_as_hurting_someone(clean) {
        return None;
    }
    let flat = text::squash(&clean.to_lowercase());
    let child = _child_at_the_hurt(&flat);
    if child == "small" {
        // The safe thing first.
        return Some(pick(&BABY_LINES, memory, clean));
    }
    if !child.is_empty() {
        return Some(pick(&CHILD_LINES, memory, clean));
    }
    let person = _person_said(clean);
    let mut options = POOL.clone();
    if !person.is_empty() && !matches!(person.as_str(), "him" | "her" | "them") {
        options.push(format_named(&NAMED, &[("person", &person)]));
    }
    Some(pick(&options, memory, clean))
}
