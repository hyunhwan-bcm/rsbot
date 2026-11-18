//! Port of `bot/modules/my_work.py`: a line that says where they work is answered with the job
//! the chat already holds (`bot/memory/workplace.py` reads the line).
#![allow(non_snake_case, clippy::all)]

use super::chat::{opt_str, truthy};
use crate::state::Memory;

/// Python: bot/modules/my_work.py:13 `TAKES_AS_IT_COMES(text, memory)`
pub fn TAKES_AS_IT_COMES(text: &str, memory: &Memory) -> bool {
    truthy(crate::memory::workplace::the_place_the_person_works(text, memory))
}

/// Python: bot/modules/my_work.py:23 `respond(text, memory)` - a runtime hook.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    opt_str(crate::memory::workplace::the_place_the_person_works(text, memory))
}
