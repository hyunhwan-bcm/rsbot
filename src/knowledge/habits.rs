//! Port of `bot/knowledge/habits.py`: habits (breaking one, the small ones like nail biting) and
//! forgetting things (whose memory it is).

use crate::data::FnRef;
use crate::knowledge::d2_shared::{a_thing_has_stopped, line_of};
use crate::knowledge::d2dup_g10::{globals, leans_on_field, leans_on_words};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.habits",
    re { _PET_ANY, _MY_AGE, _FORGETTING, _SOMEONE_ELSE_FORGETTING, _SOMEBODY_ELSE, _A_RELATIVE,
         _SOMETHING_FORGOTTEN, _ANOTHER_PERSONS_MEMORY },
    string { MOUTH_TIP });

static TO: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bto\b", 0));

/// The `ref` values of a reading in Python's dict order, as `str` of each str/int value (the
/// `followup` flag is a bool, which Python counts as an int: "True"/"False").
fn ref_values(understood: &Understood, with_flag: bool) -> Vec<String> {
    let r = &understood.reference;
    let mut out: Vec<String> =
        [&r.it, &r.that, &r.they, &r.there, &r.he, &r.she, &r.kind].iter().filter_map(|v| (*v).clone()).collect();
    if with_flag {
        if let Some(flag) = r.followup {
            out.push(if flag { "True" } else { "False" }.to_string());
        }
    }
    out
}

/// Whether the subject this line points back to is an animal (the carried subject only).
pub fn _about_a_pet(understood: &Understood) -> bool {
    let mut said = ref_values(understood, false);
    for key in ["subject", "text"] {
        if let Some(v) = leans_on_field(understood, key) {
            said.push(v.to_string());
        }
    }
    !said.is_empty() && _PET_ANY.is_match(&said.join(" ").to_lowercase())
}

// NEEDS(bot.knowledge.lookup::score, lookup::_forms, bot.knowledge::entries)
/// Whether a row about the animal itself can answer this line instead of the mouth one: the
/// line (with the words it leans on) scored against every "pets" entry. Python catches any
/// failure and says False; until the lookup engine is merged this is that False.
pub fn _pet_row_answers_before_it(_memory: &Memory, understood: &Understood) -> bool {
    if understood.words.is_empty() {
        return false;
    }
    false
}

/// The mouth tip, unless the thing being chewed is an animal's habit and not a person's.
pub fn _mouth_tip(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _about_a_pet(understood) && _pet_row_answers_before_it(memory, understood) {
        return None;
    }
    Some(MOUTH_TIP.clone())
}

/// Whether the speaker is asking about their own memory coming with age. A `guard=` test.
pub fn _my_own_memory_and_my_age(understood: &Understood) -> bool {
    let Some(said) = line_of(understood) else { return false };
    let text = text::squash(&said.to_lowercase().replace('\u{2019}', "'"));
    _MY_AGE.is_match(&text) && _FORGETTING.is_match(&text)
}

/// The line and whatever the reading carried with it, as one lower-case string. Python's
/// `" ".join` fails on a line with no clean and no text (line_of is None): a panic here.
/// TYPE(leans_on): the dict's str/int values and its "words" (see `leans_on_field`); `topic` is a
/// string, which Python's dict/list tests skip.
pub fn _words_of(understood: &Understood) -> String {
    let line = line_of(understood).expect("TypeError: sequence item 0: expected str instance, NoneType found");
    let mut said = vec![line.to_string()];
    if let Some(words) = leans_on_words(understood) {
        said.extend(words);
    }
    said.extend(ref_values(understood, true));
    text::squash(&said.join(" ").to_lowercase().replace('\u{2019}', "'"))
}

/// Whether another adult, not the speaker, is the one forgetting. A `guard=` test.
pub fn _someone_else_is_forgetting(understood: &Understood) -> bool {
    let text = _words_of(understood);
    if text.is_empty() {
        return false;
    }
    _SOMEONE_ELSE_FORGETTING.is_match(&text) || _MY_AGE.is_match(&text)
}

/// True when the writer is the one with the errand, so the reminder card may answer.
pub fn _the_writer_is_the_one_forgetting(understood: &Understood) -> bool {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    let said = text::squash(line).to_lowercase();
    if said.is_empty() {
        return true;
    }
    if _SOMEBODY_ELSE.is_match(&said) {
        return false;
    }
    if let Some(relative) = _A_RELATIVE.search(&said) {
        if _SOMETHING_FORGOTTEN.is_match(&said[relative.end()..]) {
            return false;
        }
    }
    true
}

/// False when the forgetting is somebody else's, so `parent-memory` answers it instead.
pub fn _the_errand_is_the_speakers_own(understood: &Understood) -> bool {
    let said = understood.clean.to_lowercase();
    if !_ANOTHER_PERSONS_MEMORY.is_match(&said) {
        return true;
    }
    TO.is_match(&said)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.habits:_the_writer_is_the_one_forgetting:206", |_: &FnRef, u: &Understood| {
        _the_writer_is_the_one_forgetting(u)
    }),
    ("bot.knowledge.habits:<lambda>:653", |_: &FnRef, u: &Understood| !a_thing_has_stopped(u)),
    ("bot.knowledge.habits:<lambda>:1017", |_: &FnRef, u: &Understood| {
        _the_errand_is_the_speakers_own(u) && !_someone_else_is_forgetting(u) && !_my_own_memory_and_my_age(u)
    }),
];

pub const REPLIES: &[(&str, ReplyFn)] =
    &[("bot.knowledge.habits:_mouth_tip:97", |_: &FnRef, m: &mut Memory, u: &Understood| _mouth_tip(m, u))];
