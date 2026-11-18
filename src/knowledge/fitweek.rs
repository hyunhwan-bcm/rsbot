//! Port of PyBot's `bot/knowledge/fitweek.py`: the shape of an exercise week (the cool-down,
//! coming back, home-gym kit, running load, fasted training, avoiding training injuries).
//!
//! The module's entries, `MACHINE_WORDS` and `ANSWERS` are data in the bundle; the only
//! runtime code is the training-injuries row's guard and the tense test it calls.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _ALREADY_HURT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.fitweek._ALREADY_HURT"));

/// True when the line says the injury has already happened, not how to avoid one.
pub fn _the_hurt_already_happened(understood: &Understood) -> bool {
    let line = sh::squash_lower(sh::clean_or_text(understood));
    _ALREADY_HURT.is_match(&line)
}

/// The training-injuries row stands down when the hurt has already happened (first aid, not
/// prevention).
pub fn lambda_282(u: &Understood) -> bool {
    !_the_hurt_already_happened(u)
}

fn g_lambda_282(_f: &FnRef, u: &Understood) -> bool {
    lambda_282(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[("bot.knowledge.fitweek:<lambda>:282", g_lambda_282)];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
