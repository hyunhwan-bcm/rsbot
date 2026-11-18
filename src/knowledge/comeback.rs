//! Port of PyBot's `bot/knowledge/comeback.py`: coming back to exercise or a team sport.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::understood::Understood;
use std::sync::LazyLock;

static CLUB_SEARCH: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.comeback.CLUB_SEARCH"));
static _A_RETURN_WORD: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.comeback._A_RETURN_WORD"));
static A_CHILD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.comeback.A_CHILD"));
static MY_OWN_CHILDHOOD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.comeback.MY_OWN_CHILDHOOD"));
static _FIRST_PERSON_WITHOUT_THE_PRONOUN: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.comeback._FIRST_PERSON_WITHOUT_THE_PRONOUN"));
static FIRST_PERSON: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|im|i'm|ive|i've|we|my)\b", 0));

/// Whether the line is the person coming back to a team sport themselves (not a club search,
/// not a child's team).
pub fn _a_return_to_team_sport(reading: &Understood) -> bool {
    let said = text::squash(sh::clean_or_text(reading)).to_lowercase();
    if said.is_empty() {
        return false;
    }
    if sh::any_in(&said, &CLUB_SEARCH) {
        return false;
    }
    if A_CHILD.is_match(&said) && !MY_OWN_CHILDHOOD.is_match(&said) {
        return false;
    }
    if !sh::any_in(&said, &_A_RETURN_WORD) {
        return false;
    }
    if FIRST_PERSON.is_match(&said) {
        return true;
    }
    _FIRST_PERSON_WITHOUT_THE_PRONOUN.is_match(&said)
}

fn g_return(_f: &FnRef, u: &Understood) -> bool {
    _a_return_to_team_sport(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.comeback:_a_return_to_team_sport:144", g_return)];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
