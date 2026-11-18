//! Port of `bot/modules/where_it_answers.py`: where the answer to a question is kept, named where the reply would only be a hedge or a card about something else (`bot/compose/routes.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use crate::compose::{fallback, fallback_chain, findout, routes};

macro_rules! rx {
    ($($name:ident),*) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.where_it_answers.", stringify!($name))));
    )*};
}
rx!(SEARCH_ONLY, A_YES_OR_NO_ASK, LIVE_ONLY, INCLUDED_ASK, A_FIGURE_REPLY, MONEY_WORDS);
static QUESTION_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.where_it_answers.QUESTION_FILLER"));
static OLD_ONE_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.modules.where_it_answers.OLD_ONE_VERBS"));
static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));

/// `route is routes.<NAME>`: the pool `routes.options` handed back is that module-level pool.
/// (Python compares identity; the pools are told apart here by their lines.)
fn is_route(route: &[String], name: &str) -> bool {
    data::strs(&format!("bot.compose.routes.{name}")) == route
}

/// Python: bot/modules/where_it_answers.py `_content(text, skip=())`
///
/// The words of a line that carry its subject: no filler, no figure, and no verb of the ask.
pub fn _content(text: &str, skip: Option<&HashSet<String>>) -> HashSet<String> {
    LETTERS
        .findall_str(&text.to_lowercase())
        .into_iter()
        .filter(|w| text::len(w) > 2 && !QUESTION_FILLER.contains(w) && !skip.map_or(false, |skip| skip.contains(w)))
        .collect()
}

/// Python: bot/modules/where_it_answers.py `_would_be_said(text, memory)`
///
/// The reply the person would get if this module said nothing, which is greeting's; the fallback
/// only when greeting passes.
pub fn _would_be_said(text: &str, memory: &mut Memory) -> String {
    // a broken greeting must not cost the person an answer
    let said = py_try(|| super::greeting::respond(text, memory)).ok().flatten().filter(|said| !said.is_empty());
    match said {
        Some(said) => said,
        None => s_or(Some(&fallback_chain::reply_for(text, memory, true).val())),
    }
}

/// Python: bot/modules/where_it_answers.py `_answers_the_ask(route, clean, said)`
///
/// Whether the reply the person would get is an answer to the question in `clean`. Three ways a
/// reply is not: a price instead of a yes, today's listings instead of what a building has, and
/// a card about another subject taking the line on one word of it.
pub fn _answers_the_ask(route: &[String], clean: &str, said: &str) -> bool {
    if said.is_empty() {
        return true;
    }
    if is_route(route, "A_PLACE_HAS") && A_YES_OR_NO_ASK.is_match(clean) {
        if MONEY_WORDS.is_match(said) || LIVE_ONLY.is_match(said) {
            return false;
        }
        return !_content(said, None).is_disjoint(&_content(clean, None));
    }
    if is_route(route, "THE_OLD_ONE_POOL") {
        return !_content(said, None).is_disjoint(&_content(clean, Some(&OLD_ONE_VERBS)));
    }
    if is_route(route, "INCLUDED") && INCLUDED_ASK.is_match(clean) && A_FIGURE_REPLY.is_match(said) {
        return false;
    }
    true
}

/// Python: bot/modules/where_it_answers.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Name the place, or say nothing and let the composer answer as it did.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let line = match memory.understood.as_ref() {
        Some(understood) if !understood.clean.is_empty() => understood.clean.clone(),
        _ => text.to_string(),
    };
    let clean = fallback::_matchable(&fallback::_plain(&line));
    // How busy somewhere is now is not a rule the council or the operator writes, and the council
    // line is the wrong subject for it: "is the tip busy today" was answered about the rules for a
    // tip instead of about the queue. The crowd reading in `findout` owns that ask and answers it
    // under this module, so this one stands down while it does.
    if ret_true(findout::reads_as_busy(&clean)) {
        return None;
    }
    let route = routes::options(&clean).val();
    if !route.truthy() {
        return None;
    }
    let route = py_strs(&route);
    let said = _would_be_said(text, memory);
    let chosen = |memory: &mut Memory| as_reply(fallback_chain::_choose(&route, memory, &fallback::_matchable(text), None));
    if ret_true(fallback::_is_a_hedge(&said)) || SEARCH_ONLY.is_match(&said) {
        return chosen(memory);
    }
    if is_route(&route, "INCLUDED") && INCLUDED_ASK.is_match(&clean) && A_FIGURE_REPLY.is_match(&said) {
        // what a figure depends on is not what the price includes
        return chosen(memory);
    }
    if !_answers_the_ask(&route, &clean, &said) {
        return chosen(memory);
    }
    None // the composer has an answer of its own; it keeps the turn
}
