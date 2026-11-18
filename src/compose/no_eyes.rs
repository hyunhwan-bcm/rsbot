//! Port of `bot/compose/no_eyes.py`: the honest line for a thing the bot cannot see, hear,
//! smell or open (a photo, a link, a recording, a noise in the room, their own body), in place
//! of a reply that reads as though it had.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_strs, bundle_vals, inline_re};
use crate::data;
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

bundle_res!("bot.compose.no_eyes"; A_HAND_DOES_IT, AN_EYE_READS_IT, AN_EAR_HEARS_IT, NO_HAND_VERB, READS_HANDED_TEXT,
    ASKS_THE_BOT_TO_ACT, HOLDS_THE_TEXT, ABOUT_WHAT_IT_KEEPS, A_URL, HANDED_OVER, A_RECORDING_WORD, A_LISTEN_OFFER,
    A_A_LENGTH, A_AN_EMAIL_ADDRESS, A_A_PHONE_NUMBER, A_A_CONTACT_WORD, A_A_HANDOVER, A_ASKING_FOR_IT, WRITES_A_MESSAGE,
    MESSAGE_WORD, TO_A_PERSON, A_FORWARD, THEIR_OWN_MESSAGE, A_NAMED_ADDRESSEE, BACK_AS_A_BODY, GAME_TO_PLAY, A_LOOK_ASK,
    A_LOOK_ASK_IT, A_MARK_ON_THEM, GOING_BACK, MY_BACK, A_THING_IN_THE_ROOM, A_THING_NAMED, A_NOISE_NAMED,
    A_SOUNDS_AGREEING, A_WITNESSED, A_NOISE_ABOVE, A_NOISE_IN_THE_PIPES, A_NOISE_OUTSIDE, A_NOISE_VERB,
    A_MACHINE_OR_A_NEIGHBOUR, A_NAME_ASK, THAT_OPENS_A_SENTENCE, A_BEEP_WORD, A_LIGHT_WORD, A_LIGHT_OUT_THERE,
    LIGHTS_ARE_OUT, A_POWER_CUT, A_THE_STREET, A_SMELL_WORD, A_BEEP_OR_AN_ALARM, A_DESCRIPTION);
bundle_strs!("bot.compose.no_eyes"; CANNOT_SMELL, CANNOT_HEAR_A_BEEP, CANNOT_SEE_YOU, THE_PLACEHOLDERS, _ROOM_ASKS,
    _DESCRIBED_GOES_QUIET_LINES, DESCRIBED_CYCLE, DESCRIBED_ROOM);
bundle_vals!("bot.compose.no_eyes"; SEEING_FAMILIES, HANDED_FAMILIES, CANNOT_SEE, DESCRIBED, A_MORE_BODY);
inline_re!(AN_ADDRESS_OR_NUMBER, r"\S+@\S+\.\w+|\baddress\b|\bnumber\b");
inline_re!(A_HOW_DO_I, r"\bhow\s+(?:do|can|should)\s+i\b");
inline_re!(SEE_ATTACHED, r"\bsee\s+attached\b|\battached\s+(?:is|are)\b");
inline_re!(FILL, r"\bfill(?:ing)?\b");
inline_re!(THIS_OR_THAT, r"\b(?:this|that)\b");
inline_re!(A_POINTER, r"\b(?:this|that|these|those)\b");
inline_re!(IT, r"\bit\b");
inline_re!(DID_YOU_SENSE, r"^\s*did\s+you\s+(?:hear|smell)\b");
inline_re!(A_WORD, r"[a-z']+");

static BODY_PARTS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.fallback.BODY_PARTS"));
static HEALTH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.fallback.HEALTH_WORDS"));
static SYMPTOM_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.fallback.SYMPTOM_WORDS"));

/// `r"\b(?:" + "|".join(re.escape(t) for t in things) + r")\b"`.
fn any_of(things: &[String]) -> Re {
    Re::new(&format!(r"\b(?:{})\b", things.iter().map(|t| pyre::escape(t)).collect::<Vec<_>>().join("|")), 0)
}

/// Each seeing family by name, with the pattern for its things, in the table's order.
static FAMILIES: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    SEEING_FAMILIES.pairs().iter().map(|(k, v)| (k.str().to_string(), any_of(&v.strs()))).collect()
});
/// The pattern for a family's things when it is handed over (HANDED_FAMILIES first).
static HANDED: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| {
    ["voice", "photo", "link", "file", "inbox", "form"]
        .iter()
        .map(|family| {
            let things = match HANDED_FAMILIES.get(family).filter(|v| v.truthy()) {
                Some(v) => v.strs(),
                None => SEEING_FAMILIES.get(family).expect("a seeing family").strs(),
            };
            (family.to_string(), any_of(&things))
        })
        .collect()
});

/// `CANNOT_SEE[key]`.
fn cannot_see(key: &str) -> Vec<String> {
    CANNOT_SEE.get(key).unwrap_or_else(|| panic!("KeyError: {key:?}")).strs()
}

fn n_words(s: &str) -> usize {
    text::split_ref(s).len()
}

/// Python: bot/compose/no_eyes.py:337 `a_recording_they_offer(clean)`
///
/// The honest line for a recording the person is offering the bot, or None.
pub fn a_recording_they_offer(clean: &str) -> Option<Vec<String>> {
    let line = text::strip(clean);
    if n_words(line) > 25 {
        return None;
    }
    let length = A_A_LENGTH.search(line).is_some();
    if A_LISTEN_OFFER.search(line).is_none() && !length {
        return None;
    }
    if A_RECORDING_WORD.search(line).is_some() || length {
        return Some(cannot_see("voice"));
    }
    None
}

/// Python: bot/compose/no_eyes.py:368 `a_contact_they_handed_over(clean)`
///
/// The honest line for an address or a number handed over, or None.
pub fn a_contact_they_handed_over(clean: &str) -> Option<Vec<String>> {
    let line = text::strip(clean);
    if n_words(line) > 25 {
        return None;
    }
    let has = |rx: &Re| rx.search(line).is_some();
    if !(has(&A_AN_EMAIL_ADDRESS) || has(&A_A_PHONE_NUMBER) || has(&A_A_CONTACT_WORD)) {
        return None;
    }
    if !has(&A_A_HANDOVER) {
        return None;
    }
    if has(&A_ASKING_FOR_IT) && !(has(&A_AN_EMAIL_ADDRESS) || has(&A_A_PHONE_NUMBER)) {
        return None;
    }
    Some(cannot_see("contact"))
}

/// Python: bot/compose/no_eyes.py:385 `handed_you_something(clean)`
///
/// The honest line for a thing the person believes they have just handed over, or None.
pub fn handed_you_something(clean: &str) -> Option<Vec<String>> {
    if HOLDS_THE_TEXT.search(clean).is_some() || n_words(clean) > 25 {
        return None;
    }
    if AN_ADDRESS_OR_NUMBER.search(clean).is_some() {
        // The person handing over something the bot can keep, not a thing to look at.
        return None;
    }
    if A_URL.search(clean).is_some() && A_HOW_DO_I.search(clean).is_none() {
        // A link in the line is a page the bot cannot open.
        return Some(cannot_see("link"));
    }
    if HANDED_OVER.search(clean).is_none() {
        return None;
    }
    for (family, pattern) in HANDED.iter() {
        if pattern.search(clean).is_some() {
            // The family they name is the thing on their own side.
            return Some(cannot_see(family));
        }
    }
    if ASKS_THE_BOT_TO_ACT.search(clean).is_some() {
        // An ask with no thing named is somebody else's reader.
        return None;
    }
    if SEE_ATTACHED.search(clean).is_some() {
        return Some(cannot_see("file"));
    }
    None
}

/// Python: bot/compose/no_eyes.py:458 `names_the_addressee(clean)`
pub fn names_the_addressee(clean: &str) -> bool {
    A_NAMED_ADDRESSEE.search(clean).is_some()
}

/// Python: bot/compose/no_eyes.py:463 `writes_the_message(clean)`
///
/// True when the ask is for the words of a message to somebody: write it, not send it.
pub fn writes_the_message(clean: &str) -> bool {
    if MESSAGE_WORD.search(clean).is_none() || WRITES_A_MESSAGE.search(clean).is_none() {
        return false;
    }
    if A_FORWARD.search(clean).is_some() || THEIR_OWN_MESSAGE.search(clean).is_some() {
        return false;
    }
    TO_A_PERSON.search(clean).is_some()
}

/// Python: bot/compose/no_eyes.py:479 `_a_body_or_a_doctor(clean)`
///
/// True when the line names a body, a symptom or a doctor - a rash is not a file.
pub fn _a_body_or_a_doctor(clean: &str) -> bool {
    let mut plain: HashSet<String> = A_WORD.findall_str(clean).into_iter().collect();
    if plain.contains("back") && BACK_AS_A_BODY.search(clean).is_none() {
        plain.remove("back");
    }
    plain.iter().any(|w| BODY_PARTS.contains(w) || SYMPTOM_WORDS.contains(w) || HEALTH_WORDS.contains(w))
}

/// Python: bot/compose/no_eyes.py:493 `_a_voice_is_named(clean)`
///
/// A recording, a memo, a podcast: something with words in it to be written down.
pub fn _a_voice_is_named(clean: &str) -> bool {
    FAMILIES.iter().find(|(name, _)| name == "voice").expect("a voice family").1.search(clean).is_some()
}

/// Python: bot/compose/no_eyes.py:516 `_a_game_to_play(clean)`
pub fn _a_game_to_play(clean: &str) -> bool {
    GAME_TO_PLAY.search(clean).is_some()
}

/// Python: bot/compose/no_eyes.py:521 `cannot_see_your_thing(clean)`
///
/// The honest line for a thing of theirs the bot cannot open, or None.
pub fn cannot_see_your_thing(clean: &str) -> Option<Vec<String>> {
    if ASKS_THE_BOT_TO_ACT.search(clean).is_none() || NO_HAND_VERB.search(clean).is_none() {
        return None;
    }
    if HOLDS_THE_TEXT.search(clean).is_some() || ABOUT_WHAT_IT_KEEPS.search(clean).is_some() {
        return None;
    }
    if n_words(clean) > 25 || _a_body_or_a_doctor(clean) || writes_the_message(clean) {
        return None;
    }
    // The longest name wins (`max` of (length, family name)), not the first family.
    let best = FAMILIES
        .iter()
        .filter_map(|(name, pattern)| pattern.search(clean).map(|hit| (text::len(hit.as_str()), name.clone())))
        .max();
    if let Some((_, name)) = best {
        return Some(cannot_see(&name));
    }
    if FILL.search(clean).is_some() && THIS_OR_THAT.search(clean).is_some() {
        return Some(cannot_see("form"));
    }
    if A_HAND_DOES_IT.search(clean).is_some() {
        return Some(cannot_see("hands"));
    }
    let pointed = A_POINTER.search(clean).is_some();
    if pointed && AN_EAR_HEARS_IT.search(clean).is_some() && !_a_game_to_play(clean) {
        if _a_voice_is_named(clean) {
            return Some(cannot_see("voice"));
        }
        return Some(cannot_see("sound"));
    }
    if pointed && AN_EYE_READS_IT.search(clean).is_some() {
        return Some(cannot_see("text"));
    }
    if IT.search(clean).is_some() && READS_HANDED_TEXT.search(clean).is_some() {
        return Some(cannot_see("text"));
    }
    None
}

/// Python: bot/compose/no_eyes.py:631 `_asks_me_to_look_at_them(clean, repaired=None, memory=None)`
///
/// A line asking the bot to look at their body, or a mark on it.
pub fn _asks_me_to_look_at_them(clean: &str, repaired: Option<&str>, memory: &Memory) -> bool {
    let second = repaired.filter(|r| !r.is_empty()).unwrap_or(clean);
    for line in [clean, second] {
        if A_LOOK_ASK.search(line).is_none() {
            continue;
        }
        if A_MARK_ON_THEM.search(line).is_some() {
            return true;
        }
        let body: HashSet<String> = A_WORD
            .findall_str(&line.to_lowercase())
            .into_iter()
            .filter(|w| BODY_PARTS.contains(w) || A_MORE_BODY.contains_str(w) || SYMPTOM_WORDS.contains(w))
            .collect();
        if !body.is_empty() {
            // A returning "back" is not a body.
            if body.iter().any(|w| w != "back") || GOING_BACK.search(line).is_none() || MY_BACK.search(line).is_some() {
                return true;
            }
        }
        if A_LOOK_ASK_IT.search(line).is_some() && crate::compose::fallback::_thread_is_about_the_body(memory, 4) {
            // The body is in the turn before, so the bare "it" is a body.
            return true;
        }
    }
    false
}

/// Python: bot/compose/no_eyes.py:660 `look_at_a_body_lines(clean, repaired=None, memory=None)`
///
/// The pool to say for a line asking the bot to look at the person's body, or None.
pub fn look_at_a_body_lines(clean: &str, repaired: Option<&str>, memory: &Memory) -> Option<Vec<String>> {
    if _asks_me_to_look_at_them(clean, repaired, memory) {
        return Some(CANNOT_SEE_YOU.clone());
    }
    None
}

/// Python: bot/compose/no_eyes.py:794 `a_thing_in_the_room(clean)`
///
/// The pool for a noise, a smell or a light in the room with them, or None.
pub fn a_thing_in_the_room(clean: &str) -> Option<Vec<String>> {
    let line = text::strip(clean);
    let has = |rx: &Re| rx.search(line).is_some();
    let n = n_words(line);
    if has(&A_POWER_CUT) && !has(&A_LIGHT_OUT_THERE) && n <= 20 {
        return Some(cannot_see("power_cut"));
    }
    if has(&LIGHTS_ARE_OUT) && has(&A_THE_STREET) && !has(&A_LIGHT_OUT_THERE) && n <= 20 {
        return Some(cannot_see("power_cut"));
    }
    if has(&LIGHTS_ARE_OUT) && !has(&A_LIGHT_OUT_THERE) && n <= 20 && !has(&A_NAME_ASK) {
        return Some(cannot_see("lights_out"));
    }
    let a_place_at_it = (has(&A_NOISE_ABOVE) || has(&A_NOISE_IN_THE_PIPES) || has(&A_NOISE_OUTSIDE))
        && has(&A_NOISE_VERB)
        && !has(&THAT_OPENS_A_SENTENCE);
    if !has(&A_THING_IN_THE_ROOM) && !has(&A_LIGHT_WORD) && !has(&A_THING_NAMED) && !has(&A_NOISE_NAMED) && !a_place_at_it {
        return None;
    }
    if has(&A_SOUNDS_AGREEING) || n > 20 || has(&A_NAME_ASK) {
        return None;
    }
    if DID_YOU_SENSE.match_(line).is_some() && has(&THAT_OPENS_A_SENTENCE) {
        return None;
    }
    if has(&A_SMELL_WORD) {
        return Some(CANNOT_SMELL.clone());
    }
    if has(&A_BEEP_WORD) {
        return Some(CANNOT_HEAR_A_BEEP.clone());
    }
    if has(&A_LIGHT_WORD) && !has(&A_LIGHT_OUT_THERE) {
        return Some(cannot_see("light"));
    }
    // A machine's noise or a neighbour's has a reader of its own.
    if has(&A_MACHINE_OR_A_NEIGHBOUR) {
        return None;
    }
    // The place the noise comes from is the part that is known.
    if has(&A_NOISE_ABOVE) {
        return Some(cannot_see("noise_roof"));
    }
    if has(&A_NOISE_IN_THE_PIPES) {
        return Some(cannot_see("noise_pipes"));
    }
    if has(&A_NOISE_OUTSIDE) {
        return Some(cannot_see("noise_outside"));
    }
    // Only a noise they were there for.
    if !has(&A_THING_IN_THE_ROOM) && !has(&A_WITNESSED) {
        return None;
    }
    Some(cannot_see("sound"))
}

/// Python: bot/compose/no_eyes.py:863 `_is_a_placeholder(reply)`
pub fn _is_a_placeholder(reply: &str) -> bool {
    let said = text::squash(&reply.to_lowercase().replace('\u{2014}', "-").replace('\u{2013}', "-"));
    let head = text::slice(&said, 0, Some(60));
    THE_PLACEHOLDERS.iter().any(|stem| said.starts_with(stem.as_str()) || head.contains(stem.as_str()))
}

/// Python: bot/compose/no_eyes.py:868 `a_light_in_the_room_over_a_placeholder(clean, reply)`
///
/// The light pool for a light in the room that was answered with a placeholder, else None.
pub fn a_light_in_the_room_over_a_placeholder(clean: &str, reply: &str) -> Option<Vec<String>> {
    if !_is_a_placeholder(reply) {
        return None;
    }
    // `pool is CANNOT_SEE["light"]`: the pools are told apart by their lines.
    let pool = a_thing_in_the_room(clean)?;
    if pool != cannot_see("light") && pool != cannot_see("lights_out") {
        return None;
    }
    Some(pool)
}

/// Python: bot/compose/no_eyes.py:878 `a_thing_in_the_room_over_a_placeholder(clean, reply)`
///
/// The room pool for a noise, a smell or a light answered with a placeholder, else None.
pub fn a_thing_in_the_room_over_a_placeholder(clean: &str, reply: &str) -> Option<Vec<String>> {
    if !_is_a_placeholder(reply) {
        return None;
    }
    a_thing_in_the_room(clean)
}

/// Python: bot/compose/no_eyes.py:1086 `what_they_described(clean, memory)`
///
/// The other half of a room question: what the thing they named usually is, or None.
pub fn what_they_described(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let before = memory.turns.iter().rev().find(|t| !t.bot.is_empty()).map(|t| t.bot.to_lowercase()).unwrap_or_default();
    if before.is_empty() || !_ROOM_ASKS.iter().any(|ask| before.contains(ask.as_str())) {
        return None;
    }
    let line = text::strip(clean).to_lowercase();
    if line.is_empty() || n_words(&line) > 20 {
        return None;
    }
    for row in DESCRIBED.items() {
        let (words, pool) = (row.items()[0].strs(), row.items()[1].strs());
        if words.iter().any(|w| line.contains(w.as_str())) {
            if pool == *_DESCRIBED_GOES_QUIET_LINES && A_BEEP_OR_AN_ALARM.search(&before).is_none() {
                return Some(DESCRIBED_CYCLE.clone());
            }
            return Some(pool);
        }
    }
    if A_DESCRIPTION.search(&line).is_some() {
        // A noise described but not placed: the fixed-or-moving question narrows it.
        return Some(DESCRIBED_ROOM.clone());
    }
    None
}

