//! Port of PyBot's `bot/knowledge/joint_time.py`: how long a roast joint takes, oven or slow
//! cooker.

use crate::data::{self, FnRef};
use crate::knowledge::d1_dup_g7 as dup;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

static JOINT_BEEF: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.cooking.JOINT_BEEF"));
static _BOTH: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.joint_time._BOTH"));
static _PORK_IN_SHORT: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.joint_time._PORK_IN_SHORT"));
static _SLOW_COOKER: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.joint_time._SLOW_COOKER"));
static _SLOW_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.joint_time._SLOW_WORDS"));

/// The oven time by weight; both meats' times shortened to one line when the meat is not named.
pub fn _joint_by_weight(memory: &Memory, understood: &Understood) -> Option<String> {
    let said = dup::_roast_joint(memory, understood)?;
    if said == *_BOTH {
        return Some(format!("{} {}", *JOINT_BEEF, *_PORK_IN_SHORT));
    }
    Some(said)
}

/// Whether a gammon line is not an oven question (a gammon in the slow cooker still is ours).
pub fn _not_a_gammon_in_the_oven(understood: &Understood) -> bool {
    let words = understood.words.join(" ");
    if !words.contains("gammon") {
        return true;
    }
    _SLOW_WORDS.iter().any(|p| words.contains(p.as_str()))
}

/// One row for every joint time question: the slow cooker has its own hours, the oven its own.
pub fn _joint_time(memory: &Memory, understood: &Understood) -> Option<String> {
    let words = understood.words.join(" ");
    if _SLOW_WORDS.iter().any(|p| words.contains(p.as_str())) {
        return Some(_SLOW_COOKER.clone());
    }
    _joint_by_weight(memory, understood)
}

fn g_gammon(_f: &FnRef, u: &Understood) -> bool {
    _not_a_gammon_in_the_oven(u)
}
fn r_joint_time(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    _joint_time(m, u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.joint_time:_not_a_gammon_in_the_oven:75", g_gammon)];
pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.joint_time:_joint_time:82", r_joint_time)];
