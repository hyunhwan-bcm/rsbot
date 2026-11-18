//! Port of `bot/compose/news_ask.py`.
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

/// Python: bot/compose/news_ask.py:57 `reads_as_an_ask_for_the_news(clean)`
pub fn reads_as_an_ask_for_the_news(clean: &str) -> bool {
    crate::unported!("bot.compose.news_ask:reads_as_an_ask_for_the_news")
}

