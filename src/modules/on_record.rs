//! Port of `bot/modules/on_record.py`: the store's own "you haven't told me" counted as an answer to a question about the person's own record.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::fit;

static ON_RECORD: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.on_record.ON_RECORD"));
static NOT_ON_RECORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.on_record.NOT_ON_RECORD"));
static LONGEST: LazyLock<usize> = LazyLock::new(|| data::get("bot.modules.on_record.LONGEST").as_int().unwrap() as usize);

/// Python: bot/modules/on_record.py:47 `gives(reply, want, line=None)`
///
/// `fit.gives`, with the store's own "i hold nothing about that" counted as the answer.
///
/// Python rebinds `fit.gives` to this function when the module is imported, so every reader of
/// `fit.gives` by attribute (the arbiter, `bot/compose/lead.py`) gets this wider reading: in Rust
/// those callers call this function, and `compose::fit::gives` is the reading from before
/// (`_gives_before_this_module`). (Contract fix: `line` is optional, as in Python and in `fit`.)
pub fn gives(reply: &str, want: &str, line: Option<&str>) -> Option<bool> {
    let got = fit::gives(reply, want, line);
    if got == Some(true) || !ON_RECORD.contains(want) {
        return got;
    }
    let said = text::squash(reply);
    if said.contains('?') || text::len(&said) > *LONGEST {
        return got;
    }
    if !NOT_ON_RECORD.is_match(&fit::first_sentence(&said)) {
        return got;
    }
    Some(true)
}
