//! Port of `bot/knowledge/pensions.py`: the UK state pension (when, how much, the NI record).

use crate::data::FnRef;
use crate::knowledge::d2_shared::a_thing_has_stopped;
use crate::knowledge::d2dup_g10::clean_or_text;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

static ASKS_RECORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\brecord\b|\bcheck\b.*\b(?:national insurance|\bni\b)", 0));
static COUNTED: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b\d+\s*years?\b|\bhow many years\b|\byears? of (?:national insurance|ni)\b|\bqualifying\b", 0));
static ASKS_WHEN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwhen\b|how old|what age|\bdate\b|\bretir(?:e|ing|ement)\b", 0));
static ASKS_AMOUNT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow much\b|\bamount\b|\bper week\b|\ba week\b", 0));
static RETIRING: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bretir(?:e|ing|ement)\b|\bstop(?:ping)?\s+work", 0));

const RECORD: &str = "Your record is on gov.uk under \"check your National Insurance record\": it lists each year and whether it counts, and gaps can often be filled by paying voluntary contributions.";
const WHEN: &str = "The state pension starts at 66 now, and at 67 for anyone reaching that age from 2028, with 68 to come later, so check your own date on gov.uk. It is not automatic: a letter should come about two months before, and if it does not, claim it on gov.uk or by phone.";
const YEARS: &str = "The full new state pension is about £230 a week, £11,500 a year, in 2025-26, on 35 qualifying years of National Insurance; each year short pays about a thirtieth less, and you need at least 10 years to get anything at all.";

/// When it starts, how much it is, or where the record is, led by what the line asks.
pub fn _state_pension(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = clean_or_text(understood).to_lowercase();
    let asks_record = ASKS_RECORD.is_match(&text);
    let counted = COUNTED.is_match(&text);
    let asks_when = ASKS_WHEN.is_match(&text);
    let asks_amount = ASKS_AMOUNT.is_match(&text);
    let mut reply = if asks_record && !asks_when {
        format!("{RECORD} {YEARS} {WHEN}")
    } else if (asks_amount || counted) && !asks_when {
        format!("{YEARS} {WHEN}")
    } else {
        format!("{WHEN} {YEARS}")
    };
    if RETIRING.is_match(&text) {
        reply.push_str(" Stopping work and claiming it are two different things, so retiring early is a question about your own money rather than about the pension.");
    }
    Some(reply)
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.pensions:<lambda>:118", |_: &FnRef, u: &Understood| !a_thing_has_stopped(u))];

pub const REPLIES: &[(&str, ReplyFn)] =
    &[("bot.knowledge.pensions:_state_pension:20", |_: &FnRef, m: &mut Memory, u: &Understood| _state_pension(m, u))];
