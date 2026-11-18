//! Port of `bot/compose/fit.py`: the shape of answer a line wants (the parts `wants` reaches).

use crate::understood::Understood;

/// `wants(understood, before=None)`.
pub fn wants(understood: &Understood, before: Option<&str>) -> Option<String> {
    let _ = (understood, before);
    todo!("fit::wants")
}
