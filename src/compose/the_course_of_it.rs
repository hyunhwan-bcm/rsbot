//! Port of `bot/compose/the_course_of_it.py`: "does it get easier", "will it last forever" -
//! one honest answer about the course of a thing: no date, the shape, and one question.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_str, bundle_strs, format_named, inline_re};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

bundle_sets!("bot.compose.the_course_of_it"; HEAVY, BODY, MED, RECOVERY, THING, IT_IS_HOW_IT_FEELS, DISTRESS, ORDINARY, CARE, DOES_IT);
bundle_res!("bot.compose.the_course_of_it"; COURSE, ASKS, LASTS, TIME_IN_THE_CARD, EASING, ACK_LEAD, FIRST_BIT, WISH_TO_DIE, AN_AGE);
bundle_str!("bot.compose.the_course_of_it"; CARD_IN_THE_CHAT, NO_SUBJECT);
bundle_strs!("bot.compose.the_course_of_it"; NOTHING_NAMED, CARE_PROSE, PROSE, CARD_PROSE, CARD_PROSE_UNTIMED, EASES, KEEP_UP,
    KEEP_UP_UNTIMED, STILL_STANDS, STILL_STANDS_UNTIMED);
inline_re!(A_WORD, r"[a-z']+");
inline_re!(A_LETTER_WORD, r"[A-Za-z']+");
inline_re!(A_MEASURE_ASK, r"\bhow\s+long\b|\bhow\s+much\b|\bwhat\s+time\b");

fn meets(a: &HashSet<String>, b: &HashSet<String>) -> bool {
    a.iter().any(|w| b.contains(w))
}

/// Python: bot/compose/the_course_of_it.py:284 `_words(text)`
pub fn _words(text_: &str) -> HashSet<String> {
    A_WORD.findall_str(&text_.to_lowercase()).into_iter().collect()
}

/// Python: bot/compose/the_course_of_it.py:288 `_same_as_the_last_answer(written, memory)`
///
/// True when the card is the answer the person was just given, word for word at the start.
pub fn _same_as_the_last_answer(written: &str, memory: &Memory) -> bool {
    let mine = text::slice(written, 0, Some(60));
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        let said = text::strip(&turn.bot);
        if said.is_empty() {
            continue;
        }
        if text::slice(said, 0, Some(60)) == mine {
            return true;
        }
        seen += 1;
        if seen >= 3 {
            break;
        }
    }
    false
}

/// Python: bot/compose/the_course_of_it.py:317 `_an_answer_worth_quoting(said)`
///
/// True when a short reply still has an account of the thing in it, not just a nod.
pub fn _an_answer_worth_quoting(said: &str) -> bool {
    let content = ACK_LEAD.sub("", said);
    A_LETTER_WORD.findall_str(text::strip(&content)).len() >= 5
}

/// Python: bot/compose/the_course_of_it.py:323 `_the_card_in_the_chat(memory)`
///
/// What the last answer was about, as the knowledge base puts it, and where it came from
/// ("card" or "said"); ("", "") for nothing.
pub fn _the_card_in_the_chat(memory: &Memory) -> (String, String) {
    let topic = memory.get("topic").and_then(|t| t.get("text")).map(|t| t.str().to_string()).unwrap_or_default();
    let mut lines = vec![topic];
    lines.extend(memory.turns.iter().rev().map(|t| t.user.clone()));
    // The last reading is handed back to the reader so an animal or a thing in hand stays.
    let before = Memory { understood: Some(memory.understood.clone().unwrap_or_default()), ..Memory::new() };
    for line in &lines {
        let line = text::strip(line);
        if line.is_empty() {
            continue;
        }
        let found = com4::attempt(|| {
            let reading = crate::understanding::understand(line, Some(&before));
            crate::knowledge::lookup::find(&reading, None, None, false)
        });
        let Some(Some(entry)) = found else { continue };
        // Only a reply that is already words can be quoted.
        if let crate::knowledge::entry::Reply::Text(reply) = &entry.reply {
            let written = text::strip(reply);
            if !written.is_empty() {
                let source = if _same_as_the_last_answer(written, memory) { "said" } else { "card" };
                return (written.to_string(), source.to_string());
            }
        }
    }
    // The answer the person already has is the thing the "it" is about.
    for turn in memory.turns.iter().rev() {
        let said = text::strip(&turn.bot);
        if said.ends_with('?') || !_an_answer_worth_quoting(said) {
            continue;
        }
        // A hedge, a stall or "tell me which bit" is not a card.
        if crate::compose::fallback::_is_a_hedge(said) || crate::compose::fallback::_is_an_ask_which_bit(said) {
            continue;
        }
        // A reply that asks something back, or opens with a frame before a colon, is not about
        // anything yet.
        let opening = text::slice(said, 0, Some(60)).to_lowercase();
        if opening.contains('?') || opening.contains(':') || opening.contains("tell me") || opening.contains("say what") {
            continue;
        }
        return (said.to_string(), "said".to_string());
    }
    (String::new(), String::new())
}

/// `FIRST_BIT.split(s, maxsplit=1)` (the pattern has no groups).
fn split_once_at(s: &str) -> (String, Option<String>) {
    match FIRST_BIT.search(s) {
        Some(m) => (s[..m.start()].to_string(), Some(s[m.end()..].to_string())),
        None => (s.to_string(), None),
    }
}

/// Python: bot/compose/the_course_of_it.py:393 `_the_first_bit(card, limit=150, skip_openers=False)`
///
/// The card's first sentence or clause, short enough to quote inside a sentence of our own.
pub fn _the_first_bit(card: &str, limit: i64, skip_openers: bool) -> String {
    let said = text::strip(card);
    if said.is_empty() {
        return String::new();
    }
    let (head, rest) = split_once_at(said);
    let mut first = text::strip(&head).to_string();
    if skip_openers && text::len(&first) < 30 {
        if let Some(rest) = rest.filter(|r| !text::strip(r).is_empty()) {
            first = text::strip(&split_once_at(text::strip(&rest)).0).to_string();
        }
    }
    let mut first = text::rstrip_chars(&first, " .;:").to_string();
    let limit = limit as usize;
    if text::len(&first) > limit {
        let cut = text::slice(&first, 0, Some(limit as isize));
        let comma = text::rfind(&cut, ", ");
        first = if comma > 40 { text::slice(&cut, 0, Some(comma)) } else { cut };
        first = text::rstrip_chars(&first, " ,;:-").to_string();
    }
    first
}

/// Python: bot/compose/the_course_of_it.py:418 `_leans_on(memory)`
pub fn _leans_on(memory: &Memory) -> String {
    memory.understood.as_ref().and_then(|u| u.leans_on.as_ref()).map(|l| l.words.join(" ")).unwrap_or_default()
}

/// Python: bot/compose/the_course_of_it.py:424 `_recent(memory, line='')`
///
/// What was said in the last few turns, so the "it" can be read from the thread itself.
pub fn _recent(memory: &Memory, line: &str) -> String {
    let mine = text::strip(line).to_lowercase();
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(4)..]
        .iter()
        .map(|t| text::strip(&t.user).to_lowercase())
        .filter(|t| !t.is_empty() && *t != mine)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python: bot/compose/the_course_of_it.py:454 `reads(text, memory=None)`
///
/// The course asked about, or None when this is not the shape (or one of the two markers,
/// CARD_IN_THE_CHAT + ":" + source and NO_SUBJECT).
pub fn reads(text_: &str, memory: &Memory) -> Option<String> {
    let line = text::strip(text_);
    if line.is_empty() || text::split_ref(line).len() > 14 {
        return None;
    }
    let words = _words(line);
    if meets(&words, &MED) || meets(&words, &BODY) || meets(&words, &THING) || meets(&words, &RECOVERY) {
        return None;
    }
    let found = COURSE.search(line);
    let lasts = LASTS.search(line);
    if found.is_none() && lasts.is_none() {
        return None;
    }
    if ASKS.search(line).is_none() && !line.ends_with('?') {
        return None;
    }
    let recent = _recent(memory, line);
    let lean: HashSet<String> = _words(&_leans_on(memory)).union(&_words(&recent)).cloned().collect();
    let both: HashSet<String> = words.union(&lean).cloned().collect();
    // A thread told about self-harm or wanting to die is the safety lane's.
    if meets(&both, &DISTRESS) || WISH_TO_DIE.search(&format!("{line} {recent}")).is_some() {
        return None;
    }
    let like_this = found.as_ref().is_some_and(|m| m.as_str().to_lowercase().contains("like this"));
    if !meets(&words, &HEAVY) {
        let (card, source) = _the_card_in_the_chat(memory);
        if !card.is_empty() {
            return Some(format!("{}:{source}", *CARD_IN_THE_CHAT));
        }
        // Nothing in the talk says what "it" is, and no card is on the table: one short question.
        let asks_about_a_feeling = meets(&words, &IT_IS_HOW_IT_FEELS);
        let about_a_hard_time = meets(&lean, &HEAVY) || ["died", "death", "funeral", "loss"].iter().any(|w| lean.contains(*w));
        let nothing_named = text::split_ref(&recent).len() < 3;
        if nothing_named && !asks_about_a_feeling && !about_a_hard_time && !like_this && A_MEASURE_ASK.search(&line.to_lowercase()).is_none() {
            return Some(NO_SUBJECT.clone());
        }
    }
    let heavy = like_this || meets(&words, &HEAVY) || words.contains("easier") || meets(&lean, &HEAVY);
    if !heavy {
        return None;
    }
    // An ordinary life-stage thing in the thread keeps its own answer; a loss outranks it.
    let ordinary = meets(&both, &ORDINARY) || AN_AGE.search(&format!("{line} {recent}")).is_some();
    if ordinary && !meets(&both, &CARE) {
        return None;
    }
    // A recovery in the thread is a body healing.
    if meets(&lean, &RECOVERY) && !meets(&lean, &CARE) {
        return None;
    }
    if let Some(u) = memory.understood.as_ref() {
        if u.body.as_deref().is_some_and(|b| !b.is_empty()) || u.about_bot {
            return None;
        }
    }
    let asked = match (&found, &lasts) {
        (Some(m), _) => m.as_str(),
        (None, Some(l)) => l.as_str(),
        _ => unreachable!(),
    };
    Some(text::strip(asked).to_lowercase())
}

/// `sum(ord(c) for c in said) + k * len(said)`.
fn spread(said: &str, k: i128) -> i128 {
    com4::ord_sum(said) + k * text::len(said) as i128
}

/// Python: bot/compose/the_course_of_it.py:526 `reply_for(text, memory=None)`
///
/// One honest answer about the course of it: no date, the shape, and one question.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    let asked = reads(text_, memory).filter(|a| !a.is_empty())?;
    // Vary with the line itself, so the same line always gets the same answer.
    if asked == *NO_SUBJECT {
        let said = text::strip(text_);
        let at = (spread(said, 5) % NOTHING_NAMED.len() as i128) as usize;
        return Some(NOTHING_NAMED[at].clone());
    }
    if asked.starts_with(CARD_IN_THE_CHAT.as_str()) {
        let (whole, source) = _the_card_in_the_chat(memory);
        let card = _the_first_bit(&whole, 150, source == "said");
        if card.is_empty() {
            return None;
        }
        let said = format!("{text_}|{card}");
        let at = (spread(&said, 7) % 3) as usize;
        let bit = |pool: &Vec<String>| format_named(&pool[at], &[("bit", &card)]);
        // No deadline is put on a card that carries none.
        let timed = TIME_IN_THE_CARD.search(&whole).is_some();
        if EASING.search(text_).is_some() {
            return Some(bit(&EASES));
        }
        if source == "card" {
            return Some(bit(if timed { &CARD_PROSE } else { &CARD_PROSE_UNTIMED }));
        }
        let first = A_WORD.findall_str(&card.to_lowercase());
        if first.first().is_some_and(|w| DOES_IT.contains(w)) {
            return Some(bit(if timed { &KEEP_UP } else { &KEEP_UP_UNTIMED }));
        }
        return Some(bit(if timed { &STILL_STANDS } else { &STILL_STANDS_UNTIMED }));
    }
    let said = format!("{text_}|{}", _leans_on(memory));
    let at = (spread(&said, 7) % PROSE.len() as i128) as usize;
    // A loss named in the line itself counts.
    let lean = _words(&_leans_on(memory));
    if _words(text_).iter().chain(lean.iter()).any(|w| CARE.contains(w)) {
        return Some(CARE_PROSE[at % CARE_PROSE.len()].clone());
    }
    Some(PROSE[at].clone())
}
