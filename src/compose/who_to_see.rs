//! Port of `bot/compose/who_to_see.py`: a bare "do i need to see someone" after an answer,
//! answered with the answer's own referral (or the sign it was fenced behind).
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_strs, bundle_vals, inline_re};
use crate::compose::{pick, verdict};
use crate::data;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

bundle_res!("bot.compose.who_to_see"; ASK, A_PERSON, A_PLACE, A_SIGN_MEANS, ANYBODY, AN_OBJECT_OF, A_SUBSET_OF_TIME, SAYS_TO_DO);
bundle_strs!("bot.compose.who_to_see"; SOMEONE_LINES, PLACE_LINES, ON_THE_PHONE_LINES, CONDITION_LINES);
bundle_vals!("bot.compose.who_to_see"; A_LONG_CALL);
inline_re!(SENTENCE_BREAK, r"(?<=[.!?]) ");
inline_re!(A_BOOKING, r"\s*(appointments?|visits?|checks?)\b");

static NAMES_A_CLINICIAN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.verdict.NAMES_A_CLINICIAN"));
static FENCED_IN_SENTENCE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.verdict.FENCED_IN_SENTENCE"));

/// Python: bot/compose/who_to_see.py:91 `_clean(text)`
pub fn _clean(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

/// Python: bot/compose/who_to_see.py:95 `_the_answer_in_hand(memory)`
///
/// The reply the bot gave last, or "".
pub fn _the_answer_in_hand(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|t| !text::strip(&t.bot).is_empty()).map(|t| t.bot.clone()).unwrap_or_default()
}

/// Python: bot/compose/who_to_see.py:137 `_the_condition(said)`
///
/// The answer's own sentence making a referral depend on a sign, cut to the sign.
pub fn _the_condition(said: &str) -> String {
    for sentence in SENTENCE_BREAK.split(said) {
        let Some(person) = A_PERSON.search(&sentence).or_else(|| ANYBODY.search(&sentence)) else { continue };
        if NAMES_A_CLINICIAN.search(&sentence).is_none() && ANYBODY.search(&sentence).is_none() {
            continue;
        }
        if A_SIGN_MEANS.search(&sentence).is_none() && !_a_sign_not_a_step(&sentence, Some(&person)) {
            continue;
        }
        let mut clause = text::strip(&sentence).to_string();
        // The sign after "but" is what the doubt is about.
        if let Some(cut) = clause.rfind(", but ").filter(|c| *c > 0) {
            // `_a_sign_not_a_step(clause, person)`: Python reads the sentence's match against the
            // stripped clause, offsets and all, and so does this.
            if _a_sign_not_a_step_at(&clause, person.start()) {
                clause = text::upper_first(&clause[cut + ", but ".len()..]);
            }
        }
        return clause;
    }
    String::new()
}

/// Python: bot/compose/who_to_see.py:160 `_the_question(said, clause='')`
///
/// The answer's own question, if it asked one the person can answer.
pub fn _the_question(said: &str, clause: &str) -> String {
    for sentence in SENTENCE_BREAK.split(said) {
        let said_now = text::strip(&sentence);
        if !said_now.is_empty() && !clause.contains(said_now) && (said_now.ends_with('?') || said_now.contains("? ")) {
            if let Some(at) = said_now.find('?') {
                return said_now[..at + 1].to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/compose/who_to_see.py:171 `_the_condition_reply(said, memory)`
pub fn _the_condition_reply(said: &str, memory: &Memory) -> String {
    let clause = _the_condition(said);
    if clause.is_empty() {
        return String::new();
    }
    let written = format!("{}.", text::rstrip_chars(&clause, " ."));
    let mut reply = pick(&CONDITION_LINES, memory, &clause).replace("{clause}", &written);
    let question = _the_question(said, &clause);
    if !question.is_empty() && !reply.contains(&question) {
        reply = format!("{} {question}", text::rstrip(&reply));
    }
    reply
}

/// `_a_sign_not_a_step` with the person's position given as a byte offset into `said`.
fn _a_sign_not_a_step_at(said: &str, start: usize) -> bool {
    let before = &said[..start.min(said.len())];
    A_SIGN_MEANS.search(before).is_some() || A_SUBSET_OF_TIME.search(before).is_some()
}

/// Python: bot/compose/who_to_see.py:196 `_a_sign_not_a_step(said, person)`
///
/// True when what comes before the named person makes the referral depend on a sign.
pub fn _a_sign_not_a_step(said: &str, person: Option<&Match>) -> bool {
    match person {
        Some(p) => _a_sign_not_a_step_at(said, p.start()),
        None => false,
    }
}

/// Python: bot/compose/who_to_see.py:204 `_a_sentence_that_names_someone(said)`
///
/// The answer's own sentence naming somebody to contact, if it is advice and not a condition.
pub fn _a_sentence_that_names_someone(said: &str) -> String {
    for sentence in SENTENCE_BREAK.split(said) {
        if NAMES_A_CLINICIAN.search(&sentence).is_none() || FENCED_IN_SENTENCE.search(&sentence).is_some() {
            continue;
        }
        if _a_sign_not_a_step(&sentence, A_PERSON.search(&sentence).as_ref()) {
            continue;
        }
        if SAYS_TO_DO.search(&sentence).is_some() {
            return sentence;
        }
    }
    String::new()
}

/// What `_the_call` finds: the instruction (first letter lowered) and the span of the person
/// named in it, or ("", None).
pub type Call = (String, Option<(usize, usize)>);

/// Python: bot/compose/who_to_see.py:218 `_the_call(said)`
///
/// The instruction in the answer that names somebody to contact, and who it names.
pub fn _the_call(said: &str) -> Call {
    let mut sentence = verdict::_the_referral_sentence(said).str().to_string();
    if !sentence.is_empty() {
        let person_in_it = A_PERSON.search(&sentence).map(|m| m.start());
        if person_in_it.is_none_or(|start| _a_sign_not_a_step_at(&sentence, start)) {
            // The referral names nobody, or puts it behind a sign: the instruction is elsewhere.
            sentence = String::new();
        }
    }
    if sentence.is_empty() || verdict::_referral(said).is_none() {
        // No unconditional referral: a sentence that plainly names somebody and says to go is
        // still advice.
        sentence = _a_sentence_that_names_someone(said);
        if sentence.is_empty() {
            return (String::new(), None);
        }
    }
    let t = text::strip(&sentence).to_string();
    let mut call = t.clone();
    let head = text::strip(t.split(';').next().unwrap_or("")).to_string();
    if A_PERSON.search(&head).is_some() {
        call = head;
    }
    // A sentence that still runs long is cut after the person is named, never before.
    let long = A_LONG_CALL.as_int().expect("an int") as usize;
    if text::split_ref(&call).len() > long {
        let end = A_PERSON.search(&call).map(|p| p.end());
        let tail = match end {
            Some(e) => &call[e..],
            None => "",
        };
        let breaks: Vec<usize> = [",", ":", ";"].iter().filter_map(|m| tail.find(m)).collect();
        if let Some(least) = breaks.iter().min() {
            let cut = end.unwrap_or(0) + least;
            call = text::strip(&call[..cut]).to_string();
        }
    }
    let call = text::rstrip_chars(&call, " .;:,").to_string();
    let Some(person) = A_PERSON.search(&call).map(|p| (p.start(), p.end())) else { return (String::new(), None) };
    if call.is_empty() {
        return (String::new(), None);
    }
    let mut c = call.chars();
    let first = c.next().unwrap();
    let lowered: String = first.to_lowercase().chain(c).collect();
    (lowered, Some(person))
}

/// Python: bot/compose/who_to_see.py:256 `reply_for(text, memory)`
///
/// The reply to a bare "do i need to see someone", or None.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    ASK.match_(&_clean(text_))?;
    let said = _the_answer_in_hand(memory);
    if said.is_empty() {
        return None;
    }
    let (call, person) = _the_call(&said);
    let Some((start, end)) = person.filter(|_| !call.is_empty()) else {
        // The answer fenced its referral, so the condition answers a bare doubt about it.
        let reply = _the_condition_reply(&said, memory);
        return if reply.is_empty() { None } else { Some(reply) };
    };
    let named = text::squash(&call[start..end]);
    let on_the_phone = matches!(text::strip_chars(&named, "the ").to_lowercase().as_str(), "999" | "911" | "111" | "112");
    let a_place = on_the_phone || A_PLACE.match_(&named).is_some();
    let said_again = if a_place || AN_OBJECT_OF.search(&call[..start]).is_none() {
        // A place or a number is nobody for "them" to stand in for; a mention that is not the
        // object of an instruction keeps the answer's own sentence.
        call.clone()
    } else {
        // The mention inside the instruction becomes a pronoun; a mention that is the thing to
        // book takes "an appointment" instead.
        let after = &call[end..];
        match A_BOOKING.match_(after) {
            Some(place) => {
                let noun = place.str(1);
                let pronoun = if noun.starts_with("appointment") {
                    "an appointment".to_string()
                } else {
                    format!("a {}", noun.trim_end_matches('s'))
                };
                text::strip(&format!("{} {pronoun}{}", text::rstrip(&call[..start]), text::rstrip(&after[place.end()..]))).to_string()
            }
            None => text::strip(&format!("{} them {}", text::rstrip(&call[..start]), text::lstrip(after))).to_string(),
        }
    };
    // A number is not somebody to name in front of the instruction.
    let pool: &Vec<String> = if on_the_phone {
        &ON_THE_PHONE_LINES
    } else if a_place {
        &PLACE_LINES
    } else {
        &SOMEONE_LINES
    };
    let line = pick(pool, memory, &call).replace("{person}", &named);
    let said_again = format!("{}.", text::upper_first(&said_again));
    let mut c = said_again.chars();
    let lower = c.next().map(|f| f.to_lowercase().chain(c).collect::<String>()).unwrap_or_default();
    Some(line.replace("{call}", &said_again).replace("{call_lower}", &lower))
}

/// Python: bot/compose/who_to_see.py:302 `replies_to_a_who_to_see(text, memory)`
///
/// True when this file has the line, for the module that declares the claim.
pub fn replies_to_a_who_to_see(text_: &str, memory: &Memory) -> bool {
    crate::compose::com4::attempt(|| reply_for(text_, memory).is_some_and(|r| !r.is_empty())).unwrap_or(false)
}
