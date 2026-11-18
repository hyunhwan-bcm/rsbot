//! Port of `bot/compose/invitation.py`: an invitation to the bot ("fancy a pint?", "can you come
//! to the party") gets the honest no, and the help it can give instead.

use crate::data::{self, Value};
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.invitation.", stringify!($name))));)*
    };
}
regexes!(INVITED, INVITED_AT_THE_START, NOT_THEIRS, NOT_WANTED, NOT_AN_INVITE, PRESENCE, NOT_PRESENCE, ASKING_IF_IM_HERE);
static TOGETHER_AT: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.invitation.TOGETHER_AT")
        .items()
        .iter()
        .map(|row| {
            let row: &'static [Value] = row.items();
            (row[0].str().to_string(), row[1].as_re().unwrap())
        })
        .collect()
});
static TOGETHER_REPLIES: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.compose.invitation.TOGETHER_REPLIES"));
static PRESENCE_REPLIES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.invitation.PRESENCE_REPLIES"));

/// `from . import pick as compose_pick`: its contract takes the memory as a `Value`.
fn compose_pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/invitation.py:170 `_the_invitation(clean)`
///
/// Which pool answers an invitation to the bot, or None when the line is not one.
pub fn _the_invitation(clean: &str) -> Option<Vec<String>> {
    let text = text::squash(&text::lower(clean));
    if text.is_empty() || ASKING_IF_IM_HERE.match_(&text).is_some() {
        return None;
    }
    let present = PRESENCE.is_match(&text) && !NOT_PRESENCE.is_match(&text);
    if !present && !INVITED.is_match(&text) {
        return None;
    }
    if INVITED_AT_THE_START.match_(&text).is_none() && NOT_THEIRS.match_(&text).is_some() {
        return None;
    }
    if !present && (NOT_WANTED.is_match(&text) || NOT_AN_INVITE.is_match(&text)) {
        return None;
    }
    if present && NOT_AN_INVITE.is_match(&text) {
        return None;
    }
    for (kind, pattern) in TOGETHER_AT.iter() {
        if pattern.is_match(&text) {
            return Some(TOGETHER_REPLIES.get(kind).expect("KeyError").strs());
        }
    }
    if present {
        return Some(PRESENCE_REPLIES.clone());
    }
    None
}

/// Python: bot/compose/invitation.py:196 `reply_for(text, memory)`
///
/// The line an invitation to the bot gets, or None when the line is not one.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text::lower(text));
    let pool = _the_invitation(&clean).filter(|p| !p.is_empty())?;
    Some(compose_pick(&pool, memory, &clean))
}
