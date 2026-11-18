//! Port of `bot/compose/clause.py`: the question in the clause after the comma.

use crate::understood::Understood;

/// `shape(read, want)`.
pub fn shape(read: &Understood, want: Option<String>) -> Option<String> {
    let _ = (read, want);
    todo!("clause::shape")
}
