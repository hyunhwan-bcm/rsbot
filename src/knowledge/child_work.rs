//! Port of `bot/knowledge/child_work.py`: a child with a part-time job (UK rules).

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::globals;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::state::Memory;
use crate::understood::Understood;

globals!("bot.knowledge.child_work",
    re { CHILD_WORK_GET, CHILD_WORK_PUB, CHILD_WORK_OLDER, CHILD_WORK_ABROAD, CHILD_WORK_TOO_YOUNG,
         CHILD_WORK_PERMIT_ASK, CHILD_WORK_JOBS_ASK, CHILD_WORK_PAY_ASK, CHILD_WORK_HOLIDAY,
         CHILD_WORK_TIME_ASK, CHILD_WORK_HOURS_ASK, CHILD_WORK_NOT_AN_AGE },
    string { CHILD_WORK_PUB_REPLY, CHILD_WORK_OLDER_REPLY, CHILD_WORK_ABROAD_REPLY,
             CHILD_WORK_TOO_YOUNG_REPLY, CHILD_WORK_PERMIT, CHILD_WORK_JOBS, CHILD_WORK_PAY,
             CHILD_WORK_HOLIDAY_REPLY, CHILD_WORK_TIME, CHILD_WORK_HOURS, CHILD_WORK_AGE });

fn said(understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    line.to_lowercase()
}

/// The hours, the jobs, the pay or the permit, whichever the line asks for.
pub fn _child_work(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said(understood);
    if CHILD_WORK_GET.is_match(&said) {
        return None; // how to hunt for one is the job-hunting row's answer
    }
    let rows: [(&LazyRe, &LazyStr); 10] = [
        (&CHILD_WORK_PUB, &CHILD_WORK_PUB_REPLY),
        (&CHILD_WORK_OLDER, &CHILD_WORK_OLDER_REPLY),
        (&CHILD_WORK_ABROAD, &CHILD_WORK_ABROAD_REPLY),
        (&CHILD_WORK_TOO_YOUNG, &CHILD_WORK_TOO_YOUNG_REPLY),
        (&CHILD_WORK_PERMIT_ASK, &CHILD_WORK_PERMIT),
        (&CHILD_WORK_JOBS_ASK, &CHILD_WORK_JOBS),
        (&CHILD_WORK_PAY_ASK, &CHILD_WORK_PAY),
        (&CHILD_WORK_HOLIDAY, &CHILD_WORK_HOLIDAY_REPLY),
        (&CHILD_WORK_TIME_ASK, &CHILD_WORK_TIME),
        (&CHILD_WORK_HOURS_ASK, &CHILD_WORK_HOURS),
    ];
    for (ask, reply) in rows {
        if ask.is_match(&said) {
            return Some(reply.to_string());
        }
    }
    Some(CHILD_WORK_AGE.clone())
}

type LazyRe = std::sync::LazyLock<&'static crate::py::re::Re>;
type LazyStr = std::sync::LazyLock<String>;

/// False when the 13 to 16 in the line is hours, weight or money rather than an age.
pub fn _a_number_is_an_age(understood: &Understood) -> bool {
    !CHILD_WORK_NOT_AN_AGE.is_match(&said(understood))
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.child_work:_a_number_is_an_age:209", |_: &FnRef, u: &Understood| _a_number_is_an_age(u))];

pub const REPLIES: &[(&str, ReplyFn)] =
    &[("bot.knowledge.child_work:_child_work:174", |_: &FnRef, m: &mut Memory, u: &Understood| _child_work(m, u))];
