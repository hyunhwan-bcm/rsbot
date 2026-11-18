//! Port of `bot/understanding/senses.py`: which sense of a word the line uses.

use crate::understanding::normalize::Line;
use indexmap::IndexMap;

/// `senses(line)`.
pub fn senses(line: &Line) -> IndexMap<String, String> {
    let _ = line;
    todo!("senses::senses")
}
