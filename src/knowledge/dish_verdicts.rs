//! Port of `bot/knowledge/dish_verdicts.py`: whether an everyday dish is good for you - one row
//! that reads the dish off the line, fenced off a child's, a pet's, a pregnancy's or a pack
//! date's question.

use crate::data;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

static _NAMES: LazyLock<Vec<String>> = LazyLock::new(|| data::get("bot.knowledge.dish_verdicts._NAMES").strs());
static _FENCE: LazyLock<Vec<String>> = LazyLock::new(|| data::get("bot.knowledge.dish_verdicts._FENCE").strs());
static NOT_PLAIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

/// True when the line is about a child, a pet, a pregnancy, a diet or a pack date - with the dish
/// names taken out first, since one of them holds a fence word ("hot dog").
pub fn _fenced(said: &str) -> bool {
    let mut rest = format!(" {said} ");
    for name in _NAMES.iter() {
        rest = rest.replace(&format!(" {name} "), " ").replace(&format!(" {name}s "), " ");
    }
    _FENCE.iter().any(|w| rest.contains(&format!(" {w} ")) || rest.contains(&format!(" {w}s ")))
}

/// The verdict for the dish the line names, or None when it names none of them.
pub fn _the_dish(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", NOT_PLAIN.sub(" ", &text::lower(&understood.clean)));
    let said = SPACES.sub(" ", &said);
    if _fenced(&said) {
        return None;
    }
    let by_name = data::get("bot.knowledge.dish_verdicts._by_name");
    let verdicts = data::get("bot.knowledge.dish_verdicts.VERDICTS");
    for name in _NAMES.iter() {
        if said.contains(&format!(" {name} ")) || said.contains(&format!(" {name}s ")) {
            let dish = by_name.get(name).map(|d| d.str()).unwrap_or_else(|| panic!("KeyError: {name:?}"));
            let row = verdicts.get(dish).unwrap_or_else(|| panic!("KeyError: {dish:?}"));
            return Some(row.items()[1].str().to_string());
        }
    }
    None
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.dish_verdicts:_the_dish:384", |_, m, u| _the_dish(m, u))];
