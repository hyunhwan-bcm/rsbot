//! Port of `bot/modules/the_wait_about_a_body.py`: a wait about somebody's body, the news or the ruled-out thing, said over a placeholder reply (`bot/compose/a_wait_about_a_body.py`).
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

static GENERIC: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.the_wait_about_a_body.GENERIC"));
static GOT_IT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.modules.the_wait_about_a_body.GOT_IT"));
static NOTED: LazyLock<String> = LazyLock::new(|| data::string("bot.modules.the_wait_about_a_body.NOTED"));
static SAID_AGAIN: LazyLock<String> = LazyLock::new(|| data::string("bot.modules.the_wait_about_a_body.SAID_AGAIN"));
static NOT_A_WORD_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));

/// Python: bot/modules/the_wait_about_a_body.py `is_generic(reply)`
///
/// True when the reply is one of the placeholder families this module answers over.
pub fn is_generic(reply: &str) -> bool {
    let mut said = text::strip(reply).to_lowercase().replace('\u{2019}', "'");
    if said.starts_with(SAID_AGAIN.as_str()) {
        said = text::strip(&said[SAID_AGAIN.len()..]).to_string();
    }
    if said.is_empty() {
        return false;
    }
    said.starts_with(NOTED.as_str()) || GENERIC.iter().any(|opener| said.starts_with(opener.as_str()))
}

/// Python: bot/modules/the_wait_about_a_body.py `_words(line)`
///
/// The words of a line, with the person's own "my" heard as the bot's "your".
pub fn _words(line: &str) -> Vec<String> {
    let said = line.to_lowercase().replace('\u{2019}', "'");
    let said = NOT_A_WORD_CHAR.sub(" ", &said);
    text::split(&said).into_iter().map(|w| if w == "my" { "your".to_string() } else { w }).collect()
}

/// Python: bot/modules/the_wait_about_a_body.py `said_back(reply, text)`
///
/// True when the reply is the person's own line handed back and nothing else. Only the words are
/// compared, in order, after the one wrapper the bot puts in front of an echo.
pub fn said_back(reply: &str, text: &str) -> bool {
    let said = text::strip(reply).to_lowercase().replace('\u{2019}', "'");
    let Some(wrapper) = GOT_IT.iter().chain(std::iter::once(&*NOTED)).find(|wrapper| said.starts_with(wrapper.as_str())) else {
        return false;
    };
    let said = &said[wrapper.len()..];
    !_words(said).is_empty() && _words(said) == _words(text)
}

/// Python: bot/modules/the_wait_about_a_body.py `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Say the wait, the news or the ruled-out thing, or say nothing and leave the answer alone.
pub fn respond(text: &str, memory: &mut Memory) -> Option<String> {
    let said = s_or(Some(&crate::compose::fallback_chain::reply_for(text, memory, true).val()));
    if !(is_generic(&said) || said_back(&said, text)) {
        return None;
    }
    as_reply(crate::compose::a_wait_about_a_body::reply_for(text, memory))
}
