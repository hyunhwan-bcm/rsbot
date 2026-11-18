//! Port of `bot/modules/recap.py`: a recap ask the store's own gate does not read ("what were we just saying", "where were we up to").
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::memory::store_replies as sr;
use crate::memory::thread;

static WANTED_AGAIN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.modules.recap.WANTED_AGAIN"));

/// Python: bot/modules/recap.py `_recap_shape(text, memory)`
///
/// The line, folded, when it is a recap main's own gate does not read, else "".
pub fn _recap_shape(text: &str, memory: &Memory) -> String {
    // a line that cannot be read keeps the ordinary answer
    let Ok(low) = py_try(|| ret_str(crate::memory::relations::_folded(text, memory))) else { return String::new() };
    if low.is_empty() || !WANTED_AGAIN.is_match(&low) {
        return String::new();
    }
    // No handing back to the store on the lines its own gate says it reads: its reminder-wrapper
    // pattern ("remind me what", "remind me where") matches the line and its recap reader still
    // does not answer it, so deferring here was the shrug it was meant to remove. The two
    // candidates say the same thing where both are made, and the store's band is the front one.
    low
}

/// Python: bot/modules/recap.py `ASKS_ABOUT_THE_CHAT(text, memory)`
///
/// This line asks about the conversation, whatever topic its words happen to name.
pub fn ASKS_ABOUT_THE_CHAT(text: &str, memory: &Memory) -> bool {
    !_recap_shape(text, memory).is_empty()
}

/// Python: bot/modules/recap.py:78 `_not_a_recap(text, memory)`
///
/// True when a line is about the conversation rather than about the world it names: the store's
/// own reading, widened by the shapes this module adds. (Always Some: the contract's Option is
/// from the trace.)
pub fn _not_a_recap(text: &str, memory: &Memory) -> Option<bool> {
    // an unreadable line is simply not the filter's business
    Some(or_on_raise(|| ret_true(sr::_about_this_chat(text, &mut memory.clone())) || !_recap_shape(text, memory).is_empty(), false))
}

/// `_not_a_recap`, as the function value `bot/memory/thread.py`'s readers take for `skip`.
fn not_a_recap_fn() -> Value {
    Value::Fn(Arc::new(data::FnRef { id: "bot.modules.recap:_not_a_recap:78".to_string(), cells: Vec::new(), defaults: Vec::new() }))
}

/// Python: bot/modules/recap.py `_the_recap(memory, low)`
///
/// The thing we were on and the ask, said back in the person's own words.
pub fn _the_recap(memory: &Memory, low: &str) -> String {
    let skip = not_a_recap_fn();
    let said = ret_str(thread::last_said(memory, &skip));
    let mut thing = ret_str(thread::about(memory, &skip));
    if !thing.is_empty() && !ret_true(sr::_a_thing(&thing)) {
        thing = String::new();
    }
    let not_before = thread::topic_index(memory, &skip).val();
    let ask = ret_str(thread::last_ask(memory, &skip, &not_before));
    let options: Vec<String> = if !thing.is_empty() && !ask.is_empty() {
        vec![
            format!("We were on {thing} \u{2014} you asked {ask}. Want to carry on?"),
            format!("Still on {thing}: you asked {ask}."),
            format!("The last thing we got into was {thing}, and you asked {ask}."),
        ]
    } else if !ask.is_empty() && !said.is_empty() {
        vec![
            format!("You said {said}. You asked {ask}. Want to carry on?"),
            format!("You said {said} - and you asked {ask}."),
            format!("{}. Your question was {ask}.", cap1(&said)),
        ]
    } else if !ask.is_empty() {
        vec![format!("You asked {ask}. Want to carry on?"), format!("You were asking {ask}.")]
    } else if !said.is_empty() {
        vec![
            format!("You said {said}. Want to carry on?"),
            format!("You last said {said}."),
            format!("You'd told me {said}."),
            format!("Last thing you said was {said}."),
        ]
    } else if !thing.is_empty() {
        vec![format!("We were on {thing}."), format!("Still {thing}.")]
    } else {
        return String::new();
    };
    _fresh(&options, memory)
}

/// Python: bot/modules/recap.py `_fresh(options, memory)`
///
/// An option that is not the line the bot has just said. Varying the words is the module's job,
/// so the last line said is dropped here before the pick.
pub fn _fresh(options: &[String], memory: &Memory) -> String {
    let last = text::strip(&last_bot(memory)).to_string();
    let fresh: Vec<String> = options.iter().filter(|option| text::strip(option) != last).cloned().collect();
    let fresh = if fresh.is_empty() { options.to_vec() } else { fresh };
    ret_str(sr::_vary(&fresh, memory))
}

/// Python: bot/modules/recap.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The recap itself, on the shapes the store's own gate does not read.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let low = _recap_shape(text, memory);
    if low.is_empty() {
        return None;
    }
    // a reader that cannot answer leaves the turn as it was
    let said = py_try(|| {
        if ret_true(sr::_about_the_bot_here(memory)) {
            // "what were you saying" about the bot itself is the bot's line, not the chat's.
            return None;
        }
        Some(_the_recap(memory, &low))
    })
    .ok()??;
    nz(said)
}
