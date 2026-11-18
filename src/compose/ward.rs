//! Port of `bot/compose/ward.py`: a practical question about somebody's stay in hospital is the
//! ward's to answer, and the reply says so without guessing a number, a ward or a time.

use crate::compose::fallback_chain::py_format;
use crate::data;
use crate::py::re::{Re, I};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.ward.", stringify!($name))));)*
    };
}
regexes!(PERSON, SELF, PLACE, STAFF, DETAIL, ASK, SOMETHING_WRONG, A_PET, HOME_AGAIN, A_STAY_IN_THE_LINE, THE_BOT);
static THE_WARD_KNOWS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.ward.THE_WARD_KNOWS"));
static WHOSE: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.ward.WHOSE"));
/// The two `re.search` patterns written inline in `_whom`.
static SHE: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z])she(?![a-z])", I));
static HE: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z])he(?![a-z])", I));

/// Python: bot/compose/ward.py:117 `_whom(said)`
///
/// The pronoun for the person, from the words of the line: her, him, them or you.
pub fn _whom(said: &str) -> String {
    if SHE.is_match(said) {
        return "her".into();
    }
    if HE.is_match(said) {
        return "him".into();
    }
    // Their own stay, named with a place of care and no third person in the line.
    if !PERSON.is_match(said) {
        return "you".into();
    }
    "them".into()
}

/// Python: bot/compose/ward.py:129 `_the_place_before(memory)`
///
/// The words of the last three turns, lowercased, or "" on the first turn.
pub fn _the_place_before(memory: &Memory) -> String {
    let turns = &memory.turns[memory.turns.len().saturating_sub(3)..];
    turns.iter().map(|turn| text::lower(&turn.user)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/ward.py:144 `answer_for(text, memory=None)`
///
/// The ward line for a question about somebody's care, or None to leave the reply alone.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(&text::lower(text));
    if text::split_ref(&said).len() < 3 || THE_BOT.is_match(&said) {
        return None;
    }
    // The place may be one turn back ("how long will she be in for" after "my mum went into
    // hospital").
    let room = format!("{said} {}", _the_place_before(memory));
    if A_PET.is_match(&room) || SOMETHING_WRONG.is_match(&room) || HOME_AGAIN.is_match(&room) {
        return None;
    }
    if !(PLACE.is_match(&room) || STAFF.is_match(&room) || DETAIL.is_match(&room)) {
        return None;
    }
    // Nothing of the care in this line: the line has to be about the stay.
    if !(PLACE.is_match(&said) || STAFF.is_match(&said) || DETAIL.is_match(&said)) && !A_STAY_IN_THE_LINE.is_match(&said) {
        return None;
    }
    // Who it is about: somebody else, or the speaker with a place of care in the line.
    if !(PERSON.is_match(&said) || (SELF.is_match(&said) && PLACE.is_match(&room))) {
        return None;
    }
    if !ASK.is_match(&said) {
        return None;
    }
    let whom = _whom(&said);
    let seed = memory.turns.len() * 31 + text::len(&said);
    let line = &THE_WARD_KNOWS[seed % THE_WARD_KNOWS.len()];
    Some(py_format(line, &[("whose", &WHOSE[&whom]), ("whom", &whom)]))
}
