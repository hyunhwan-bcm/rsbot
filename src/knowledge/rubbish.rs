//! Port of `bot/knowledge/rubbish.py`: which bin, the tip, bin day, recycling.

use crate::data::FnRef;
use crate::knowledge::d2dup_g10::{cook_ask, globals};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.rubbish",
    value { ITEM_ROUTES },
    string { WHICH_BIN },
    re { PUT_INTO, PLACEMENT, A_MACHINE },
    strs { ONLY_A_MACHINE, CONSUMABLE });

/// `(spellings, said)` per item route, in order.
static ROUTES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    ITEM_ROUTES.items().iter().map(|r| (r.items()[1].strs(), r.items()[2].str().to_string())).collect()
});

/// The route for the item this line names, or None when it names none of them.
pub fn _the_item_named(line: &str) -> Option<String> {
    let words = format!(" {} ", line.to_lowercase().replace('\u{2019}', "'"));
    ROUTES.iter().find(|(spellings, _)| spellings.iter().any(|s| words.contains(&format!(" {s} ")))).map(|(_, said)| said.clone())
}

/// The route for the item in the line, or the depends-on-the-item question when there is none.
pub fn _a_bin_for_the_item(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = if understood.text.is_empty() { &understood.clean } else { &understood.text };
    if let Some(said) = _the_item_named(line).filter(|s| !s.is_empty()) {
        return Some(said);
    }
    Some(WHICH_BIN.clone())
}

/// False when the line is asking what to cook, so no item row answers.
pub fn _not_a_cook_ask(understood: &Understood) -> bool {
    !cook_ask(understood)
}

/// The line as typed, then the reading's own text - what a word match here looks at.
pub fn _said(understood: &Understood) -> &str {
    [&understood.plain, &understood.clean, &understood.norm_text].into_iter().find(|s| !s.is_empty()).map(|s| s.as_str()).unwrap_or("")
}

/// False when the item named holds what the line is putting into it.
pub fn _not_a_container(understood: &Understood) -> bool {
    !PUT_INTO.is_match(&_said(understood).to_lowercase())
}

/// True only when the line asks where to put something that goes in a machine.
pub fn _a_consumable_is_placed(understood: &Understood) -> bool {
    let said = _said(understood).to_lowercase();
    if !PLACEMENT.is_match(&said) {
        return false;
    }
    if ONLY_A_MACHINE.iter().any(|w| said.contains(w.as_str())) {
        return true;
    }
    A_MACHINE.is_match(&said) && CONSUMABLE.iter().any(|w| said.contains(w.as_str()))
}

/// False when this line is another row's question: a cook ask, a container or a placement.
pub fn _disposal_guard(understood: &Understood) -> bool {
    _not_a_cook_ask(understood) && _not_a_container(understood) && !_a_consumable_is_placed(understood)
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.rubbish:_disposal_guard:796", |_: &FnRef, u: &Understood| _disposal_guard(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[(
    "bot.knowledge.rubbish:_a_bin_for_the_item:637",
    |_: &FnRef, m: &mut Memory, u: &Understood| _a_bin_for_the_item(m, u),
)];
