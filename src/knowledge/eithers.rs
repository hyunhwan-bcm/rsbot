//! Port of `bot/knowledge/eithers.py`: the either-or questions with one honest answer - here the
//! real or fake Christmas tree.

use crate::data;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;

/// The real-or-fake Christmas tree answer, for a line that is about the tree.
pub fn _a_christmas_tree(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = format!(" {} ", text::lower(&understood.words.join(" ")));
    let tree_words = data::get("bot.knowledge.eithers.TREE_WORDS").strs();
    if !tree_words.iter().any(|w| said.contains(&format!(" {w}"))) {
        return None;
    }
    Some("Fake if you will use it for six Christmasses or more, real if not: making a plastic tree costs about that many years of real ones in carbon. A real tree is the better evening either way - it smells of the day.".into())
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.eithers:_a_christmas_tree:38", |_, m, u| _a_christmas_tree(m, u))];
