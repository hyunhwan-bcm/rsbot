//! Port of `bot/knowledge/music.py`: playing an instrument (strings, practice, the neighbours).

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;

globals!("bot.knowledge.music", set { _MUSIC_AROUND_STAFF, _STAFF_TAILS }, strs { _MUSIC_PHRASES, _STAFF_MEANS_PEOPLE });

/// False when the sore finger sounds infected rather than new to the strings. (`norm` is a list
/// in a reading, so Python's `isinstance(norm, str)` test leaves only the clean line.)
pub fn _not_an_infection(understood: &Understood) -> bool {
    let text = &understood.clean;
    for word in [
        "red", "swollen", "swelling", "pus", "infection", "infected", "whitlow", "abscess", "throbbing", "hot",
        "fever", "septic", "nail fold",
    ] {
        if text.contains(word) {
            return false;
        }
    }
    true
}

/// The line is about playing music.
pub fn _a_musical_line(understood: &Understood) -> bool {
    let text = format!(" {}", understood.clean);
    let split = crate::py::text::split_ref(&text);
    let padded = format!(" {text} ");
    for word in [
        "music", "practice", "practise", "practising", "practicing", "playing", "play", "guitar", "piano", "violin",
        "drums", "singing", "sing", "band", "instrument", "chords", "rehearsal", "rehearsing", "trumpet", "flute",
        "saxophone", "cello", "ukulele", "bass", "clarinet",
    ] {
        if split.contains(&word) || padded.contains(&format!(" {word} ")) {
            return true;
        }
    }
    false
}

/// "Staff" is the five lines of music, not the people at work.
pub fn _staff_is_the_five_lines(understood: &Understood) -> bool {
    let clean = if understood.clean.is_empty() { &understood.text } else { &understood.clean }.to_lowercase();
    let stripped = clean.replace('\'', "");
    let words: Vec<&str> = crate::py::text::split_ref(&stripped);
    let staff = |w: &&str| w.starts_with("staff") && _STAFF_TAILS.contains(&w.chars().skip(5).collect::<String>());
    if !words.iter().any(|w| staff(w)) {
        return true;
    }
    if _MUSIC_PHRASES.iter().any(|p| clean.contains(p.as_str())) {
        return true;
    }
    if _STAFF_MEANS_PEOPLE.iter().any(|p| clean.contains(p.as_str())) {
        return false;
    }
    words.iter().any(|w| _MUSIC_AROUND_STAFF.contains(*w))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.music:_a_musical_line:100", |_: &FnRef, u: &Understood| _a_musical_line(u)),
    ("bot.knowledge.music:_not_an_infection:89", |_: &FnRef, u: &Understood| _not_an_infection(u)),
    ("bot.knowledge.music:_staff_is_the_five_lines:141", |_: &FnRef, u: &Understood| _staff_is_the_five_lines(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[];
