//! Port of `bot/compose/urgent.py`: someone the person looks after in trouble right now (not
//! breathing, a fit, swallowed something) - the careful line, ambulance first, or today's.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, inline_re};
use crate::py::random::Random;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;

bundle_res!("bot.compose.urgent"; WORD, PRO_ANSWERING, BODY_IN_THE_ROOM, BLUE_LOOSE, BLUE_BODY, GREY_LOOSE, GREY_BODY, STUCK,
    SECOND_CLAUSE, SETTLING_PHRASES, OBJECTISH, CONFUSED_SIGN, CONFUSED_AFTER, NOSE_BREATH, NOSE_RED_BESIDE, FIT_OF, FIT_ROUTINE,
    NORMAL_ASK, PAST_MARK, NOW_WORDS, ALL_BETTER, GAIT_STEM, LAUGH_IDIOM);
bundle_sets!("bot.compose.urgent"; FAMILY, INGEST, MEDICINE, AMBIGUOUS_CUES, NOT_A_BODY, MANNER, SETTLING, CALENDAR, PLACE,
    OBJECT_WORD, SYMPTOM, ORDINARY_STATE, DURATION, ONLINE, WAKE_OBJECT, SMALL_PERSON, A_FEED, SOFT_STATES, GAIT_BEFORE,
    WAKE_OCCASION, PETS);
bundle_strs!("bot.compose.urgent"; NOW, TODAY, CHEMICAL, SMALL_OBJECT, WAKE_CUES, DEATH, VERBY, TO_PAIN, FIT_STILL_RINGS,
    PRONOUN_SAFE, NEARLY, ASKING, NOW_REPLIES, POISON_NOW_REPLIES, POISON_TODAY_REPLIES, TODAY_REPLIES, FIGURE_OF_SPEECH);
inline_re!(NOT_A_LETTER, r"[^a-z ]");

/// Python: bot/compose/urgent.py:244 `_not_a_body(rest, before='')`
///
/// True when what follows the state word says the state is not about a body.
pub fn _not_a_body(rest: &str, before: &str) -> bool {
    let words = text::split_ref(rest);
    let Some(&first) = words.first() else { return false };
    let second = words.get(1).copied().unwrap_or("");
    let third = words.get(2).copied().unwrap_or("");
    let to_pain = |w: &str| TO_PAIN.iter().any(|p| p == w);
    if NOT_A_BODY.contains(first) {
        if first == "to" && (to_pain(second) || (matches!(second, "the" | "any" | "some" | "his" | "her") && to_pain(third))) {
            return false;
        }
        if first == "to" && _fed_and_unresponsive(before, rest) {
            return false;
        }
        return true;
    }
    if SETTLING.contains(text::strip_chars(first, ".,!?;:")) {
        // A degree word straight after the state word says how a thing is going, not a body.
        let joined = words[1..].join(" ");
        let tail = text::strip_chars(&joined, " .,!?;:");
        if tail.is_empty() || CALENDAR.contains(tail) {
            return true;
        }
        if SECOND_CLAUSE.match_(tail).is_some() {
            return true;
        }
        return SETTLING_PHRASES.match_(tail).is_some();
    }
    if first == "together" && matches!(second, "a" | "an" | "the" | "my" | "our" | "his" | "her" | "their" | "some" | "these" | "those" | "this") {
        // "my son is fitting together a lego set" is a child assembling.
        return true;
    }
    if first == "when" || first == "whenever" {
        // An ordinary state read by what follows, not a body tonight.
        return words.iter().take(6).any(|w| ORDINARY_STATE.contains(*w));
    }
    if first == "in" {
        if second == "with" || second == "at" {
            return true; // settling in
        }
        if MANNER.contains(second) && !PLACE.contains(second) {
            return true;
        }
        if matches!(second, "the" | "a" | "an" | "his" | "her" | "their" | "my" | "our") {
            return !PLACE.contains(third);
        }
        return false;
    }
    if first == "with" {
        if matches!(second, "a" | "an" | "the") {
            // "burning up with a fever" is a body; "burning up with the old letters" is not.
            return !SYMPTOM.contains(third);
        }
        return OBJECT_WORD.contains(second);
    }
    if first == "on" {
        if matches!(second, "the" | "a" | "an" | "my" | "his" | "her" | "their" | "our" | "this" | "that") {
            return ONLINE.contains(third);
        }
        return ONLINE.contains(second);
    }
    if first == "for" {
        return DURATION.contains(second) || (matches!(second, "a" | "an" | "one" | "two" | "half") && DURATION.contains(third));
    }
    false
}

/// Python: bot/compose/urgent.py:397 `_fed_and_unresponsive(before, rest)`
///
/// Whether this is a small person who will not respond to a feed.
pub fn _fed_and_unresponsive(before: &str, rest: &str) -> bool {
    let b = NOT_A_LETTER.sub(" ", &before.to_lowercase());
    let r = NOT_A_LETTER.sub(" ", &rest.to_lowercase());
    let who = text::split_ref(&b);
    let what = text::split_ref(&r);
    com4::tail(&who, 3).iter().any(|w| SMALL_PERSON.contains(*w)) && what.iter().take(4).any(|w| A_FEED.contains(*w))
}

/// Python: bot/compose/urgent.py:451 `reads_as_a_story(clean)`
///
/// Whether the line puts the event firmly in the past, so no ambulance is needed now.
pub fn reads_as_a_story(clean: &str) -> bool {
    PAST_MARK.search(clean).is_some() && (NOW_WORDS.search(clean).is_none() || ALL_BETTER.search(clean).is_some())
}

/// Python: bot/compose/urgent.py:522 `_named_a_gait(clean, start)`
///
/// True when the word before the state word makes `limp` a leg rather than a body. `start` is
/// a byte offset.
pub fn _named_a_gait(clean: &str, start: usize) -> bool {
    text::split_ref(&clean[..start]).last().is_some_and(|w| GAIT_BEFORE.contains(*w))
}

/// Python: bot/compose/urgent.py:535 `_walking(rest)`
pub fn _walking(rest: &str) -> bool {
    GAIT_STEM.match_(text::lstrip(rest)).is_some()
}

/// Python: bot/compose/urgent.py:546 `_signs_a_body(clean, phrase, rest, start=0)`
///
/// Whether an ambiguous state word is about a body in this line.
pub fn _signs_a_body(clean: &str, phrase: &str, rest: &str, start: usize) -> bool {
    if phrase == "limp" {
        return !_named_a_gait(clean, start) && !_walking(rest);
    }
    if WAKE_CUES.iter().any(|c| c == phrase) {
        let mut words = text::split_ref(rest);
        if words.first() == Some(&"up") {
            words.remove(0);
        }
        let Some(&first) = words.first() else { return true };
        if WAKE_OCCASION.contains(first) {
            // "wont wake up for work" is a heavy sleeper.
            return false;
        }
        return words.iter().take(5).any(|w| WAKE_OBJECT.contains(*w));
    }
    if phrase == "burning up" {
        let lead = text::lstrip(rest);
        return !FIGURE_OF_SPEECH.iter().any(|f| lead.starts_with(f.as_str()));
    }
    if phrase == "confused" {
        return CONFUSED_SIGN.search(clean).is_some() || CONFUSED_AFTER.match_(rest).is_some();
    }
    true
}

/// `clean.find(phrase, start + 1)`: the next place after the one at `start` (one character on).
fn find_after(clean: &str, phrase: &str, start: usize) -> Option<usize> {
    let step = clean[start..].chars().next().map_or(1, char::len_utf8);
    clean[start + step..].find(phrase).map(|i| i + start + step)
}

/// Python: bot/compose/urgent.py:567 `_first(clean, phrases)`
///
/// The first cue the line really carries. An ambiguous cue is dropped when what follows it says
/// it is not about a body.
pub fn _first<S: AsRef<str>>(clean: &str, phrases: &[S]) -> Option<String> {
    for phrase in phrases {
        let phrase = phrase.as_ref();
        let mut start = clean.find(phrase);
        while let Some(at) = start {
            let rest = &clean[at + phrase.len()..];
            if !AMBIGUOUS_CUES.contains(phrase) {
                return Some(phrase.to_string());
            }
            if !_signs_a_body(clean, phrase, rest, at) {
                start = find_after(clean, phrase, at);
                continue;
            }
            if VERBY.iter().any(|v| v == phrase) && OBJECTISH.match_(rest).is_some() {
                start = find_after(clean, phrase, at);
                continue;
            }
            if !_not_a_body(rest, &clean[..at]) {
                return Some(phrase.to_string());
            }
            start = find_after(clean, phrase, at);
        }
    }
    None
}

/// Python: bot/compose/urgent.py:609 `_pet_is_the_subject(clean)`
pub fn _pet_is_the_subject(clean: &str) -> bool {
    for word in WORD.findall_str(clean) {
        if FAMILY.contains(&word) {
            return false;
        }
        if PETS.contains(&word) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/urgent.py:619 `reply_for(text, memory=None)`
///
/// A careful line when someone they look after is in trouble, else None.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text_.to_lowercase());
    // A dream of the thing is not the thing.
    if crate::memory::dream::a_dream_is_not_happening(&clean) {
        return None;
    }
    if NOSE_BREATH.search(&clean).is_some() && NOSE_RED_BESIDE.search(&clean).is_none() {
        // A blocked nose is not an airway: the whole line belongs to the knowledge rows.
        return None;
    }
    let words: HashSet<String> = WORD.findall_str(&clean).into_iter().collect();
    let family = words.iter().any(|w| FAMILY.contains(w));
    let he_she = words.contains("he") || words.contains("she");
    // A grey body written with only a pronoun.
    let pronoun_grey = ["he", "she", "hes", "shes"].iter().any(|w| words.contains(*w)) && GREY_BODY.search(&clean).is_some();
    if !family && !pronoun_grey && !(he_she && _first(&clean, &PRONOUN_SAFE).is_some()) {
        return None;
    }
    if PRO_ANSWERING.search(&clean).is_some() && BODY_IN_THE_ROOM.search(&clean).is_none() {
        return None;
    }
    if _pet_is_the_subject(&clean) {
        return None;
    }
    if (FIT_OF.search(&clean).is_some() || FIT_ROUTINE.search(&clean).is_some()) && !FIT_STILL_RINGS.iter().any(|m| clean.contains(m.as_str())) {
        // A fit of coughing, or the fits a child has every few months.
        return None;
    }
    if LAUGH_IDIOM.search(&clean).is_some() {
        return None;
    }
    if DEATH.iter().any(|m| clean.contains(m.as_str())) {
        // A death in the line is the loss module's.
        return None;
    }
    let padded = format!(" {clean} ");
    let nearly = NEARLY.iter().any(|h| padded.contains(h.as_str()));
    let swallowed = words.iter().any(|w| INGEST.contains(w));
    // A paintball is a soft pellet, not a bottle of paint.
    let chem_text = clean.replace("paint ball", " ").replace("paintball", " ");
    let chemical = swallowed && CHEMICAL.iter().any(|p| chem_text.contains(p.as_str()));
    let poisoned = swallowed
        && !chemical
        && (words.iter().any(|w| MEDICINE.contains(w)) || SMALL_OBJECT.iter().any(|p| clean.contains(p.as_str())));
    let blue = BLUE_LOOSE.search(&clean).is_some() && BLUE_BODY.search(&clean).is_some();
    let grey = GREY_LOOSE.search(&clean).is_some() && GREY_BODY.search(&clean).is_some();
    let mut now: Option<String> = if nearly {
        None
    } else {
        _first(&clean, &NOW)
            .or_else(|| blue.then(|| "is blue".to_string()))
            .or_else(|| grey.then(|| "is grey".to_string()))
            .or_else(|| STUCK.search(&clean).map(|_| "airway".to_string()))
            .or_else(|| chemical.then(|| "chemical".to_string()))
    };
    if now.is_some() && reads_as_a_story(&clean) {
        now = None;
    }
    if now.as_ref().is_some_and(|n| SOFT_STATES.contains(n)) && NORMAL_ASK.search(&clean).is_some() {
        now = None;
    }
    let mut today: Option<String> = None;
    if !clean.contains('?') && !ASKING.iter().any(|a| padded.contains(a.as_str())) {
        today = _first(&clean, &TODAY).or_else(|| poisoned.then(|| "swallowed".to_string()));
        if today.is_some() && PAST_MARK.search(&clean).is_some() {
            today = None;
        }
    }
    for (state, options) in [(now, &*NOW_REPLIES), (today, &*TODAY_REPLIES)] {
        let Some(state) = state else { continue };
        let options: &Vec<String> = match state.as_str() {
            "chemical" => &POISON_NOW_REPLIES,
            "swallowed" => &POISON_TODAY_REPLIES,
            _ => options,
        };
        let turns = &memory.turns;
        let recent: HashSet<&str> = turns[turns.len().saturating_sub(5)..].iter().map(|t| t.bot.as_str()).collect();
        let mut fresh: Vec<&String> = options.iter().filter(|o| !recent.contains(o.as_str())).collect();
        if fresh.is_empty() {
            fresh = options.iter().collect();
        }
        let seed = turns.len() as i128 * 7919 + com4::ord_sum(&clean);
        return Some(Random::new(seed).choice(&fresh).to_string());
    }
    None
}
