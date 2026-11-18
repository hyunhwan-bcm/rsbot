//! Port of `bot/compose/unseen.py`: a line about not being seen by the people around them
//! ("nobody cares", "i feel invisible", "my friends never invite me").

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.unseen.", stringify!($name))));)*
    };
}
regexes!(OTHER, NOT_ME, PROFESSIONAL, RISK, DENIED, BARE_VERB, THE_PERSON, IGNORED_BY, ME, NAMED_OVER, NAMED_TOLD, NAMED);
static PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.unseen.PEOPLE"));

/// Python: bot/compose/unseen.py:162 `reads(clean)`
///
/// True when the line is about not being seen by the people around them.
pub fn reads(clean: &str) -> bool {
    let text = text::squash(&text::lower(clean));
    if text.is_empty() || RISK.is_match(&text) {
        return false;
    }
    let named = NAMED.is_match(&text) && !NAMED_OVER.is_match(&text) && !NAMED_TOLD.is_match(&text);
    if !named && !ME.is_match(&text) {
        return false;
    }
    if DENIED.is_match(&text) {
        // A denial is not the complaint.
        return false;
    }
    if named {
        // The people leaving them out are named: OTHER and the bare-verb tail do not apply.
        return !PROFESSIONAL.is_match(&text);
    }
    if NOT_ME.is_match(&text) || OTHER.is_match(&text) || PROFESSIONAL.is_match(&text) {
        return false;
    }
    if let Some(bare) = BARE_VERB.search(&text) {
        if THE_PERSON.match_(bare.name_str("tail")).is_none() {
            // "nobody cares about my plants" is about the plants.
            return false;
        }
    }
    if let Some(by) = IGNORED_BY.search(&text) {
        if !PEOPLE.contains(by.str(1)) {
            // "i felt ignored by the satnav" is an ordinary line about a machine.
            return false;
        }
    }
    true
}
