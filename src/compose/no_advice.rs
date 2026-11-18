//! Port of `bot/compose/no_advice.py`: a line that declines advice, or asks only to be heard,
//! gets a listening line (or the bot's view, when that is what it asks for instead).

use crate::data;
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.no_advice.", stringify!($name))));)*
    };
}
regexes!(
    _NO_ADVICE, _WANT_TO_BE_HEARD, _JUST_LISTEN, _ASKS_A_VIEW, _NOT_THIS, _AN_ASK, _A_DECLINE,
    _AN_ASK_LATER, _NOT_AN_ASK, A_DEATH,
);
static HEARD: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.no_advice.HEARD"));
static A_VIEW: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.no_advice.A_VIEW"));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/no_advice.py:132 `_what_is_left_after_the_decline(line)`
pub fn _what_is_left_after_the_decline(line: &str) -> String {
    match _A_DECLINE.search(line) {
        Some(said) => line[said.end()..].to_string(),
        None => line.to_string(),
    }
}

/// Python: bot/compose/no_advice.py:140 `asks_something_of_its_own(line)`
///
/// True when the line asks something besides the decline, so the ask is the turn.
pub fn asks_something_of_its_own(line: &str) -> bool {
    let left = _NOT_AN_ASK.sub(" ", &_what_is_left_after_the_decline(line));
    _AN_ASK_LATER.is_match(&left)
}

/// Python: bot/compose/no_advice.py:160 `_plain(text)`
pub fn _plain(text: &str) -> String {
    text::squash(text)
}

/// Python: bot/compose/no_advice.py:164 `says_a_death(text)`
pub fn says_a_death(text: &str) -> bool {
    A_DEATH.is_match(&_plain(text))
}

/// Python: bot/compose/no_advice.py:169 `asks_for_a_view(text)`
pub fn asks_for_a_view(text: &str) -> bool {
    _ASKS_A_VIEW.is_match(&_plain(text))
}

/// Python: bot/compose/no_advice.py:174 `says_no_advice(text)`
///
/// True when the line declines advice, or asks only to be heard.
pub fn says_no_advice(text: &str) -> bool {
    let line = _plain(text);
    if line.is_empty() {
        return false;
    }
    let said = _NO_ADVICE.is_match(&line) || _WANT_TO_BE_HEARD.is_match(&line) || _JUST_LISTEN.is_match(&line);
    if !said {
        return false;
    }
    if asks_for_a_view(&line) {
        return true;
    }
    if _NOT_THIS.is_match(&line) {
        return false;
    }
    let rest = _JUST_LISTEN.sub(" ", &_WANT_TO_BE_HEARD.sub(" ", &line));
    if _AN_ASK.is_match(&rest) {
        return false;
    }
    if asks_something_of_its_own(&line) {
        return false;
    }
    true
}

/// Python: bot/compose/no_advice.py:204 `pool_name(text)`
///
/// Which reply pool the line wants: "a_view", "no_advice", or None.
pub fn pool_name(text: &str) -> Option<String> {
    if !says_no_advice(text) {
        return None;
    }
    Some(if asks_for_a_view(text) { "a_view" } else { "no_advice" }.to_string())
}

/// Python: bot/compose/no_advice.py:233 `lines_for(pool)`
pub fn lines_for(pool: &str) -> Vec<String> {
    if pool == "a_view" { A_VIEW.clone() } else { HEARD.clone() }
}

/// Python: bot/compose/no_advice.py:238 `reply_for(pool, text, memory=None)`
pub fn reply_for(pool: &str, text: &str, memory: &Memory) -> String {
    pick(&lines_for(pool), memory, text)
}
