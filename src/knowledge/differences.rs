//! Port of `bot/knowledge/differences.py`: "what is the difference between X and Y". The pairs
//! and their entries are data (built at import time by `_pair` / `_slug`, exported in the
//! bundle); the runtime logic is the cream guard.

use crate::data::FnRef;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;

/// The line says cream somewhere: "single or double cream" is the same question as the pair.
pub fn _with_cream(understood: &Understood) -> bool {
    understood.clean.to_lowercase().contains("cream")
}

pub const GUARDS: &[(&str, GuardFn)] =
    &[("bot.knowledge.differences:_with_cream:274", |_: &FnRef, u: &Understood| _with_cream(u))];

pub const REPLIES: &[(&str, ReplyFn)] = &[];


// Stubs appended by harness/integrate_skeleton.py: the Python functions this file does not
// define yet, with signatures from the generated contract. Port them in place (move a
// function up into the file when you port it).
pub use self::skeleton_stubs::*;
#[allow(unused_imports, unused_variables, non_snake_case, dead_code, clippy::all)]
mod skeleton_stubs {
    use super::*;
    use crate::data::{self, Value};
    use crate::knowledge::entry::Entry;
    use crate::runtime::Candidate;
    use crate::state::{Memory, Turn};
    use crate::understood::Understood;

    /// Python: bot/knowledge/differences.py:280 `_slug(name)` (never called on the trace corpus)
    pub fn _slug(name: &str) -> String {
        crate::unported!("bot.knowledge.differences:_slug")
    }

    /// Python: bot/knowledge/differences.py:285 `_pair(row)` (never called on the trace corpus)
    pub fn _pair(row: &'static Entry) -> Value {
        crate::unported!("bot.knowledge.differences:_pair")
    }

}

