//! Port of PyBot's `bot/knowledge/body_energy.py`: low energy in the body, not the house's.

use crate::data::{self, FnRef};
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::understood::Understood;
use std::sync::LazyLock;

static _THE_HOUSE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.body_energy._THE_HOUSE"));

/// Whether the energy the line asks about is the person's own, not the house's (bills, a meter).
pub fn _the_energy_is_the_bodys(understood: &Understood) -> bool {
    let words = sh::words_line(understood);
    let text = sh::or(sh::clean_or_text(understood), &words);
    !_THE_HOUSE.is_match(text)
}

fn g_the_energy_is_the_bodys(_f: &FnRef, u: &Understood) -> bool {
    _the_energy_is_the_bodys(u)
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.body_energy:_the_energy_is_the_bodys:56", g_the_energy_is_the_bodys)];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
