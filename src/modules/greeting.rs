//! Port of `bot/modules/greeting.py`.
//!
//! GENERATED SKELETON: every function is a stub until its workstream ports it. The
//! signatures are the contract other files call against; keep them.
#![allow(unused_variables, unused_imports, non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::py::re::Re;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::sync::LazyLock;

/// Python: bot/modules/greeting.py:84 `_a_line_pointing_back_at_nothing(clean, memory)`
pub fn _a_line_pointing_back_at_nothing(clean: &str, memory: &Memory) {
    crate::unported!("bot.modules.greeting:_a_line_pointing_back_at_nothing")
}

/// Python: bot/modules/greeting.py:99 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    crate::unported!("bot.modules.greeting:respond")
}

/// Python: bot/modules/greeting.py:911 `first_line_points_back_at_nothing(reply, memory)`
pub fn first_line_points_back_at_nothing(reply: &str, memory: &Memory) -> String {
    crate::unported!("bot.modules.greeting:first_line_points_back_at_nothing")
}

/// Python: bot/modules/greeting.py:937 `sentences_of(answer)`
pub fn sentences_of(answer: &str) -> Vec<String> {
    crate::unported!("bot.modules.greeting:sentences_of")
}

/// Python: bot/modules/greeting.py:978 `_a_step_of_this_answer(text, answer)` (never called on the trace corpus)
pub fn _a_step_of_this_answer(text: &str, answer: &str) -> bool {
    crate::unported!("bot.modules.greeting:_a_step_of_this_answer")
}

/// Python: bot/modules/greeting.py:1048 `_a_plan_for_later(text)`
pub fn _a_plan_for_later(text: &str) -> bool {
    crate::unported!("bot.modules.greeting:_a_plan_for_later")
}

/// Python: bot/modules/greeting.py:1066 `_answers_the_line(text, answer, turns=())`
pub fn _answers_the_line(text: &str, answer: &str, turns: &Value) -> bool {
    crate::unported!("bot.modules.greeting:_answers_the_line")
}

/// Python: bot/modules/greeting.py:1098 `_said_back(text, said)`
pub fn _said_back(text: &str, said: &str) -> bool {
    crate::unported!("bot.modules.greeting:_said_back")
}

/// Python: bot/modules/greeting.py:1121 `the_step_they_have_not_had(text, memory, said)`
pub fn the_step_they_have_not_had(text: &str, memory: &Memory, said: &str) -> String {
    crate::unported!("bot.modules.greeting:the_step_they_have_not_had")
}

/// Python: bot/modules/greeting.py:1229 `_the_answer_again(answer, turns)`
pub fn _the_answer_again(answer: &str, turns: &Value) -> bool {
    crate::unported!("bot.modules.greeting:_the_answer_again")
}

/// Python: bot/modules/greeting.py:1246 `takes_as_it_comes(text, memory)`
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> bool {
    crate::unported!("bot.modules.greeting:takes_as_it_comes")
}

