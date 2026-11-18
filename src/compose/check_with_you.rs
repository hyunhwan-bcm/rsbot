//! Port of `bot/compose/check_with_you.py`: a check-with-you question - "u ok with that", "is that
//! ok with you" - gets an answer: agreement plus whose call it is.
//!
//! The shapes are tight on purpose: the same words in other orders belong to other readers ("is
//! that ok to drive on", "are you safe to talk to", "u alright to drive").
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::state::Memory;
use std::sync::LazyLock;

/// `A_CHECK`: the five shapes, in order.
static A_CHECK: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    data::get("bot.compose.check_with_you.A_CHECK").items().iter().map(|v| v.as_re().expect("A_CHECK: a pattern")).collect()
});
static _BARE_FOR_YOU: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.check_with_you._BARE_FOR_YOU"));
static _NOT_A_CHECK: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.check_with_you._NOT_A_CHECK"));
static AGREED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.check_with_you.AGREED"));
static NOT_A_CHECK_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9'? ]", 0));

/// Python: bot/compose/check_with_you.py:91 `reads_as_a_check(text)`
/// True when the line is a check of a plan against the bot.
pub fn reads_as_a_check(text: &str) -> bool {
    let said = format!(" {} ", NOT_A_CHECK_CHAR.sub(" ", &crate::py::text::lower(text)));
    if _NOT_A_CHECK.is_match(&said) {
        return false;
    }
    for shape in A_CHECK.iter() {
        // `shape is _BARE_FOR_YOU`: the anchored shape is also read on the stripped line.
        let bare = std::ptr::eq(*shape, *_BARE_FOR_YOU) || shape.pattern() == _BARE_FOR_YOU.pattern();
        if shape.is_match(&said) || (bare && shape.is_match(crate::py::text::strip(&said))) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/check_with_you.py:105 `answer_for(text, memory=None, choose=None)`
/// The agreement line, or None when this is not a check-with-you question. `choose` None is
/// `random.choice` on the global generator (the only chooser any caller passes).
pub fn answer_for(text: &str, _memory: &Memory, choose: &Value) -> Option<String> {
    if !reads_as_a_check(text) {
        return None;
    }
    match choose {
        Value::None => Some(crate::py::random::choice(&AGREED)),
        Value::Fn(f) if f.name().starts_with("random:choice") || f.name().starts_with("random:Random.choice") => {
            Some(crate::py::random::choice(&AGREED))
        }
        other => panic!("TypeError: no Rust chooser for {other:?}"),
    }
}
