//! Port of `bot/knowledge/friends.py`: friends and the people close to you (a hard talk, a
//! friend in a bad way, a treat for someone).

use crate::data::FnRef;
use crate::knowledge::d2_shared::owns_the_deed_reading;
use crate::knowledge::d2dup_g10::{clean_or_text, globals};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;

globals!("bot.knowledge.friends",
    re { _COOKING_FOR_THEM, _GESTURE, _LOW_MOOD, _SAID_IN_A_LANGUAGE },
    strs { _THE_PERSON, _THE_TALK });

/// False when the line is about cooking for the people it names, not meeting them.
pub fn _not_cooking_for_them(understood: &Understood) -> bool {
    !_COOKING_FOR_THEM.is_match(clean_or_text(understood))
}

/// False when the line asks for a treat for someone and says nothing about how they are.
pub fn _not_a_plain_gesture(understood: &Understood) -> bool {
    let said = clean_or_text(understood);
    if !_GESTURE.is_match(said) {
        return true;
    }
    _LOW_MOOD.is_match(said)
}

/// `_names_a_person_or_the_talk`, and not a line where the hurt is the speaker's own to mend.
pub fn _names_a_person_and_is_not_a_deed(understood: &Understood) -> bool {
    _names_a_person_or_the_talk(understood) && !owns_the_deed_reading(understood)
}

/// A hard conversation is about someone, or about the talk; a body is neither.
pub fn _names_a_person_or_the_talk(understood: &Understood) -> bool {
    let words = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    let whole = format!(" {} ", crate::py::text::squash(&words));
    _THE_PERSON.iter().chain(_THE_TALK.iter()).any(|part| whole.contains(part.as_str()))
}

/// True when the line asks for the words in another language, which is study.py's question.
pub fn said_in_a_language(understood: &Understood) -> bool {
    _SAID_IN_A_LANGUAGE.is_match(&clean_or_text(understood).to_lowercase())
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.friends:_not_cooking_for_them:20", |_: &FnRef, u: &Understood| _not_cooking_for_them(u)),
    ("bot.knowledge.friends:_not_a_plain_gesture:47", |_: &FnRef, u: &Understood| _not_a_plain_gesture(u)),
    ("bot.knowledge.friends:_names_a_person_and_is_not_a_deed:82", |_: &FnRef, u: &Understood| {
        _names_a_person_and_is_not_a_deed(u)
    }),
    ("bot.knowledge.friends:<lambda>:202", |_: &FnRef, u: &Understood| !owns_the_deed_reading(u)),
    ("bot.knowledge.friends:<lambda>:367", |_: &FnRef, u: &Understood| !owns_the_deed_reading(u)),
    ("bot.knowledge.friends:<lambda>:407", |_: &FnRef, u: &Understood| !owns_the_deed_reading(u)),
    ("bot.knowledge.friends:<lambda>:866", |_: &FnRef, u: &Understood| !said_in_a_language(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[];
