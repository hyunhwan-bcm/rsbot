//! Port of `bot/memory/paragraph.py`: the repeat floor under the answers module's replies. A
//! paragraph the chat has already had goes out framed as the repeat it is (`again.instead`), and
//! a framed reply said again word for word goes back shorter behind the next lead.

use super::again;
use super::mem8_py::strs;
use crate::data::{FnRef, Value};
use crate::py::text;
use crate::state::Memory;
use std::sync::Arc;

const M: &str = "bot.memory.paragraph.";

strs!(A_LEAD_ALREADY);
static SHORT_AGAIN: std::sync::LazyLock<Vec<String>> =
    std::sync::LazyLock::new(|| crate::data::strs("bot.memory.again.SHORT_AGAIN"));
static AGAIN_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| crate::data::string("bot.memory.again.AGAIN_HELP"));

/// Python: bot/memory/paragraph.py:43 `a_paragraph_the_chat_has_had(reply, text, memory)`
///
/// `again.instead(reply)`, or the reply itself when it is new to the chat.
pub fn a_paragraph_the_chat_has_had(reply: Option<&str>, text: &str, memory: &Memory) -> Option<String> {
    let reply = reply?;
    if text::strip(reply).is_empty() {
        return Some(reply.to_string());
    }
    // `except Exception: return reply` - a turn is never lost to the repeat floor.
    let framed = crate::runtime::guarded(|| {
        if text::starts_with_any(&reply.to_lowercase(), &A_LEAD_ALREADY) {
            // A reader above has already framed it, so a second lead is never stacked on the
            // first - but the framed reply itself can come back word for word.
            return Err(the_same_paragraph_again(reply, memory).unwrap_or_else(|| reply.to_string()));
        }
        Ok(again::instead(reply, text, memory)
            .filter(|f| !f.is_empty())
            .or_else(|| again::already_had(reply, memory)))
    });
    let framed = match framed {
        None => return Some(reply.to_string()),
        Some(Err(done)) => return Some(done),
        Some(Ok(framed)) => framed,
    };
    let Some(framed) = framed.filter(|f| !f.is_empty()) else { return Some(reply.to_string()) };
    // The frame the reader would put on it is the frame the turn above already wears: say it
    // behind the next lead instead.
    Some(the_same_paragraph_again(&framed, memory).unwrap_or(framed))
}

/// Python: bot/memory/paragraph.py:62 `the_same_paragraph_again(reply, memory)`
///
/// A reader above framed it, and the chat has had that framed reply word for word: the short of
/// the paragraph behind the next lead in `again.SHORT_AGAIN`, then `again.AGAIN_HELP`.
pub fn the_same_paragraph_again(reply: &str, memory: &Memory) -> Option<String> {
    let body = again::unframed(reply).filter(|b| !b.is_empty())?;
    let mine = again::_flat(reply);
    let said = memory.turns.iter().filter(|turn| again::_flat(&turn.bot) == mine).count();
    if said == 0 {
        return None;
    }
    let mut short = again::_short(&body, None);
    if short.is_empty() || again::_flat(&short) == again::_flat(&body) {
        short = body;
    }
    if said >= SHORT_AGAIN.len() {
        return Some(AGAIN_HELP.replace("{reply}", &short));
    }
    Some(SHORT_AGAIN[said - 1].replace("{reply}", &short))
}

/// The id of the function `then_framed` returns (Python's closure).
pub const FRAMED_ID: &str = "bot.memory.paragraph:then_framed.<locals>.a_reply_framed_as_the_repeat_it_is:103";

/// Python: bot/memory/paragraph.py:96 `then_framed(respond)`
///
/// Wrap a `respond(text, memory)` in the floor. Python returns a closure; here the value is a
/// function reference holding `respond` in its `respond` cell, run by [`run_framed`].
pub fn then_framed(respond: &Value) -> Value {
    Value::Fn(Arc::new(FnRef {
        id: FRAMED_ID.to_string(),
        cells: vec![("respond".to_string(), respond.clone())],
        defaults: Vec::new(),
    }))
}

/// Run what [`then_framed`] returned: the wrapped module's `respond`, then the floor. The
/// wrapped function is a module's `respond` ("bot.modules.<name>:respond:<line>"), found in the
/// module registry by name.
pub fn run_framed(framed: &FnRef, text: &str, memory: &mut Memory) -> Option<String> {
    let respond = framed.cell("respond").and_then(Value::as_fn).expect("then_framed holds a function");
    let module = respond
        .name()
        .strip_prefix("bot.modules.")
        .and_then(|rest| rest.strip_suffix(":respond"))
        .unwrap_or_else(|| panic!("then_framed: {} is not a module's respond", respond.id));
    let def = crate::modules::registry()
        .iter()
        .find(|m| m.name == module)
        .and_then(|m| m.respond)
        .unwrap_or_else(|| panic!("then_framed: no respond for {module}"));
    let reply = def(text, memory);
    a_paragraph_the_chat_has_had(reply.as_deref(), text, memory)
}
