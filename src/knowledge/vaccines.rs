//! Port of PyBot's `bot/knowledge/vaccines.py`: boosters and which jab is which.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

fn s(name: &str) -> String {
    data::string(&format!("bot.knowledge.vaccines.{name}"))
}
static VACCINE_BOOSTER_ANSWER: LazyLock<String> = LazyLock::new(|| s("VACCINE_BOOSTER_ANSWER"));
static TETANUS_JAB_ANSWER: LazyLock<String> = LazyLock::new(|| s("TETANUS_JAB_ANSWER"));
static JAB_SAFETY_ANSWER: LazyLock<String> = LazyLock::new(|| s("JAB_SAFETY_ANSWER"));
static PREGNANCY_JAB_ANSWER: LazyLock<String> = LazyLock::new(|| s("PREGNANCY_JAB_ANSWER"));
static _AN_APPOINTMENT_DAY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.vaccines._AN_APPOINTMENT_DAY"));
static _A_JAB_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.vaccines._A_JAB_WORD"));
static _ASKING_IF_IT_IS_SAFE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.vaccines._ASKING_IF_IT_IS_SAFE"));
/// `_JAB_BY_DISEASE`: (words, answer).
static _JAB_BY_DISEASE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.knowledge.vaccines._JAB_BY_DISEASE")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});

/// Whether the line is not telling of a jab already booked (a question always counts).
pub fn _it_is_not_an_appointment_already(understood: &Understood) -> bool {
    if understood.kind == "question" {
        return true;
    }
    !_AN_APPOINTMENT_DAY.is_match(&sh::health_the_line(understood))
}

/// The booster row answers: no appointment already made, not flu on its own, and the booster
/// is a jab.
pub fn _the_vaccine_row_is_the_answer(understood: &Understood) -> bool {
    _it_is_not_an_appointment_already(understood) && _not_the_flu_on_its_own(understood) && _the_booster_is_the_jab(understood)
}

/// Whether the "booster" is a jab: named in the line, or (on a follow-up) in the earlier line.
pub fn _the_booster_is_the_jab(understood: &Understood) -> bool {
    if _A_JAB_WORD.is_match(&sh::health_the_line(understood)) {
        return true;
    }
    let Some(lean) = &understood.leans_on else { return true };
    if crate::py::text::strip(&lean.text).is_empty() {
        return true;
    }
    let said = [lean.text.as_str(), lean.subject.as_deref().unwrap_or(""), &lean.words.join(" ")].join(" ").to_lowercase();
    _A_JAB_WORD.is_match(&said)
}

/// False for a line about flu on its own (the flu jab has its own row), unless it is about
/// pregnancy or a booster.
pub fn _not_the_flu_on_its_own(understood: &Understood) -> bool {
    let line = sh::health_the_line(understood);
    if line.contains("flu") && !["pregnan", "expecting", "booster", "jabbed", "booster jab"].iter().any(|w| line.contains(w)) {
        return false;
    }
    true
}

/// The jab card for the line: pregnancy, safety, a named disease, tetanus, or boosters in general.
pub fn _vaccine_booster_answer(_memory: &Memory, understood: &Understood) -> String {
    let line = sh::health_the_line(understood);
    if ["pregnan", "expecting", "due in", "weeks gone"].iter().any(|w| line.contains(w)) {
        return PREGNANCY_JAB_ANSWER.clone();
    }
    if _ASKING_IF_IT_IS_SAFE.is_match(&line) {
        return JAB_SAFETY_ANSWER.clone();
    }
    for (words, answer) in _JAB_BY_DISEASE.iter() {
        if sh::any_in(&line, words) {
            return answer.clone();
        }
    }
    if line.contains("tetanus") || line.contains("lockjaw") {
        return TETANUS_JAB_ANSWER.clone();
    }
    VACCINE_BOOSTER_ANSWER.clone()
}

fn g_row(_f: &FnRef, u: &Understood) -> bool {
    _the_vaccine_row_is_the_answer(u)
}
fn r_answer(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    Some(_vaccine_booster_answer(m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.vaccines:_the_vaccine_row_is_the_answer:104", g_row)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.vaccines:_vaccine_booster_answer:164", r_answer)];
