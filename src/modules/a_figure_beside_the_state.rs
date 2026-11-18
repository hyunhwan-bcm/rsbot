//! Port of `bot/modules/a_figure_beside_the_state.py`: a body state said beside a figure of heat ("its 20 degrees and im freezing") gets the state's own pool.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.a_figure_beside_the_state.", stringify!($name))));
    )*};
}
rx!(FIGURE, MINUS_FIGURE, MINUS_WORD, FIGURE_WITH_LEAD);
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));
static THE_FIGURES_OWN_WORDS_IN_FRONT: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"^[\s,;:-]*(?:(?:its|it|it's|is|was|there|and|but|so|then|about|around|only|just|still|currently|be)\b[\s,;:-]*)+",
        0,
    )
});
static A_JOIN_AT_THE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s,;:-]+$", 0));
static THE_FIGURES_OWN_WORDS_BEHIND: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"[\s,;:-]+(?:(?:its|it|it's|is|was|there|about|around|only|just|still|currently|and|but|so|then)\b[\s,;:-]*)+$", 0)
});

/// Python: bot/modules/a_figure_beside_the_state.py `a_figure_of_heat(text)`
///
/// True when the line says a temperature: "20 degrees", "20c", "minus three degrees".
pub fn a_figure_of_heat(text: &str) -> bool {
    FIGURE.is_match(text) || MINUS_WORD.is_match(text) || MINUS_FIGURE.is_match(text)
}

/// Python: bot/modules/a_figure_beside_the_state.py `without_the_figure(text)`
///
/// The line with a bare figure of heat taken out, and the join left tidy. Only the figure and
/// the words that introduce it go, so the rest of the line reads as it did.
pub fn without_the_figure(text: &str) -> String {
    let line = FIGURE_WITH_LEAD.sub(" ", text);
    let line = MINUS_FIGURE.sub(" ", &line);
    let line = SPACES.sub(" ", &line);
    // What is left in front of the state when the figure was the whole of the other half: "its
    // -3 outside and im freezing" leaves "its and im freezing", and "its" is the figure's own
    // word rather than the state's.
    let line = THE_FIGURES_OWN_WORDS_IN_FRONT.sub("", text::strip(&line));
    let line = A_JOIN_AT_THE_END.sub("", text::strip(&line));
    // The figure's own introducing word can be left behind when the figure was the second half:
    // "im freezing, its -3 outside" is "im freezing," once the figure goes.
    let line = THE_FIGURES_OWN_WORDS_BEHIND.sub("", text::strip(&line));
    text::strip(&line).to_string()
}

/// Python: bot/modules/a_figure_beside_the_state.py `a_state_said_with_a_figure(text)`
///
/// The body state the line says beside a figure ("cold", "hot"), or None. The composer's reader
/// is the authority on what counts as a body state, so it is asked about the line with the figure
/// taken out. A line with no figure at all is not this module's.
pub fn a_state_said_with_a_figure(text: &str) -> Option<String> {
    if !a_figure_of_heat(text) {
        return None;
    }
    let rest = without_the_figure(text);
    if rest.is_empty() {
        return None;
    }
    // a reading that raises claims nothing
    or_on_raise(|| as_reply(crate::compose::fallback_chain::_body_state(&rest)), None)
}

/// `understood.get("clean") or text`.
fn clean_or<'a>(memory: &'a Memory, text: &'a str) -> &'a str {
    match memory.understood.as_ref() {
        Some(understood) if !understood.clean.is_empty() => understood.clean.as_str(),
        _ => text,
    }
}

/// Python: bot/modules/a_figure_beside_the_state.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The body state's own pool for a line that says a figure of heat beside it, or None.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let clean = clean_or(memory, text).to_string();
    let state = a_state_said_with_a_figure(&clean).filter(|state| !state.is_empty())?;
    let pool = data::get("bot.compose.fallback_data.BODY_POOLS").get(&state).filter(|pool| pool.truthy())?.strs();
    // a pool that raises claims nothing
    or_on_raise(|| as_reply(crate::compose::fallback_chain::_choose(&pool, memory, &clean, None)), None)
}

/// Python: bot/modules/a_figure_beside_the_state.py `takes_as_it_comes(text, memory)` (also bound
/// as `TAKES_AS_IT_COMES`).
///
/// Only on a line that says a figure of heat beside a body state, and nowhere else.
pub fn takes_as_it_comes(text: &str, memory: &Memory) -> Option<bool> {
    // a hook that raises claims nothing
    Some(or_on_raise(|| a_state_said_with_a_figure(clean_or(memory, text)).map_or(false, |state| !state.is_empty()), false))
}
