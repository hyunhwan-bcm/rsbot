//! Port of `bot/understanding/normalize.py`: the line cleaned, respelled and cut into clauses.

use crate::understood::{Kept, Signals};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// What `normalise(text)` returns (Python's dict, same keys).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Line {
    pub text: String,
    pub clean: String,
    pub words: Vec<String>,
    pub plain: String,
    pub norm: Vec<String>,
    pub norm_text: String,
    pub fixes: IndexMap<String, String>,
    pub intensifier: Vec<String>,
    pub intensifier_left: Vec<String>,
    pub clauses: Vec<Vec<String>>,
    /// None when there is no clause at all.
    pub focus: Option<Vec<String>>,
    pub asking: Vec<String>,
    pub signals: Signals,
    pub dropped_subject: bool,
    pub change_of_subject: bool,
    pub kept: Option<Kept>,
}

/// `normalise(text)`.
pub fn normalise(text: &str) -> Line {
    let _ = text;
    todo!("normalize::normalise")
}

/// `without_the_figure_of_speech(clean)`.
pub fn without_the_figure_of_speech(clean: &str) -> String {
    let _ = clean;
    todo!("normalize::without_the_figure_of_speech")
}

/// `_the_part_that_hurts(clean)`.
pub fn _the_part_that_hurts(clean: &str) -> String {
    let _ = clean;
    todo!("normalize::_the_part_that_hurts")
}

/// `_the_sugar_that_is_blood(clean)`.
pub fn _the_sugar_that_is_blood(clean: &str) -> String {
    let _ = clean;
    todo!("normalize::_the_sugar_that_is_blood")
}

/// `background_positions(norm)`: positions of background words (a set in Python).
pub fn background_positions(norm: &[String]) -> BTreeSet<usize> {
    let _ = norm;
    todo!("normalize::background_positions")
}

/// `background_clause(norm)`.
pub fn background_clause(norm: &[String]) -> BTreeSet<usize> {
    let _ = norm;
    todo!("normalize::background_clause")
}

/// `reason_positions(norm)`.
pub fn reason_positions(norm: &[String]) -> BTreeSet<usize> {
    let _ = norm;
    todo!("normalize::reason_positions")
}

/// `opens_without_subject(words, index)`.
pub fn opens_without_subject(words: &[String], index: usize) -> bool {
    let _ = (words, index);
    todo!("normalize::opens_without_subject")
}

/// `strip_leaders(clause)`: the clause without its leading filler words.
pub fn strip_leaders(clause: &[String]) -> &[String] {
    let _ = clause;
    todo!("normalize::strip_leaders")
}
