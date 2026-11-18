//! Port of `bot/compose/rule.py`: a question about the law, a council's rule or a court letter
//! gets the one place whose wording counts.

use crate::data::{self, Value};
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.rule.", stringify!($name))));)*
    };
}
regexes!(
    A_COURT_THING, A_GAME_COURT, A_LOCAL_RULE, LAW_ASKED, NOT_A_RULE_ASK, A_FOLLOW_UP,
    PERMISSION_ASKED, WHO_OWNS_IT, A_PENALTY,
);
static UNKNOWN_THE_LAW: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.rule.UNKNOWN_THE_LAW"));
static UNKNOWN_A_LOCAL_RULE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.rule.UNKNOWN_A_LOCAL_RULE"));
static UNKNOWN_A_COURT_THING: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.rule.UNKNOWN_A_COURT_THING"));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// `turns[-look_back:]` with Python's slice rules (0 is the whole list).
fn last_turns(turns: &[Turn], look_back: i64) -> &[Turn] {
    let n = turns.len() as i64;
    let start = if look_back == 0 { 0 } else if look_back > 0 { (n - look_back).max(0) } else { (-look_back).min(n) };
    &turns[start as usize..]
}

/// Python: bot/compose/rule.py:108 `_said(text)`
pub fn _said(text: &str) -> String {
    text::squash(&text::lower(text))
}

/// Python: bot/compose/rule.py:113 `_a_court_of_law(said)`
///
/// True when a court word in the line means a court of law rather than a place you play.
pub fn _a_court_of_law(said: &str) -> bool {
    if !A_COURT_THING.is_match(said) {
        return false;
    }
    !(A_GAME_COURT.is_match(said) && !LAW_ASKED.is_match(said))
}

/// Python: bot/compose/rule.py:125 `_the_thread_said(memory, pattern, look_back=2)`
///
/// True when one of the last few things the person said matched `pattern` (a compiled pattern
/// value; `look_back` an int, None for the default 2).
pub fn _the_thread_said(memory: &Memory, pattern: &Value, look_back: &Value) -> bool {
    let pattern = pattern.as_re().expect("pattern is a compiled pattern");
    last_turns(&memory.turns, look_back.as_int().unwrap_or(2)).iter().any(|turn| pattern.is_match(&_said(&turn.user)))
}

/// Python: bot/compose/rule.py:131 `_a_court_of_law_in_the_thread(memory, look_back=2)`
pub fn _a_court_of_law_in_the_thread(memory: &Memory, look_back: i64) -> bool {
    last_turns(&memory.turns, look_back).iter().any(|turn| _a_court_of_law(&_said(&turn.user)))
}

/// Python: bot/compose/rule.py:168 `_said_a_local_rule_recently(memory, look_back=2)`
pub fn _said_a_local_rule_recently(memory: &Memory, look_back: i64) -> bool {
    last_turns(&memory.turns, look_back).iter().any(|turn| A_LOCAL_RULE.is_match(&_said(&turn.user)))
}

/// Python: bot/compose/rule.py:174 `_a_rule_ask_in_the_thread(memory, look_back=2)`
///
/// True when one of the last few lines was itself a rule ask (`look_back` None is 2).
pub fn _a_rule_ask_in_the_thread(memory: &Memory, look_back: &Value) -> bool {
    for turn in last_turns(&memory.turns, look_back.as_int().unwrap_or(2)) {
        let earlier = _said(&turn.user);
        if earlier.is_empty() || NOT_A_RULE_ASK.is_match(&earlier) {
            continue;
        }
        if LAW_ASKED.is_match(&earlier) || WHO_OWNS_IT.is_match(&earlier) {
            return true;
        }
        if A_LOCAL_RULE.is_match(&earlier) && PERMISSION_ASKED.is_match(&earlier) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/rule.py:191 `pool_for(text, memory=None)`
///
/// The pool that names who owns this rule, or None when the line is not about one.
pub fn pool_for(text: &str, memory: &Memory) -> Option<Vec<String>> {
    let said = _said(text);
    if said.is_empty() || NOT_A_RULE_ASK.is_match(&said) {
        return None;
    }
    if _a_court_of_law(&said) || _a_court_of_law_in_the_thread(memory, 2) {
        return Some(UNKNOWN_A_COURT_THING.clone());
    }
    if LAW_ASKED.is_match(&said) {
        if A_LOCAL_RULE.is_match(&said) {
            return Some(UNKNOWN_A_LOCAL_RULE.clone());
        }
        return Some(UNKNOWN_THE_LAW.clone());
    }
    if A_FOLLOW_UP.is_match(&said) && _said_a_local_rule_recently(memory, 2) {
        if A_PENALTY.is_match(&said) || _a_rule_ask_in_the_thread(memory, &Value::Int(2)) {
            return Some(UNKNOWN_A_LOCAL_RULE.clone());
        }
    }
    None
}

/// Python: bot/compose/rule.py:207 `answer_for(text, memory=None)`
///
/// One line naming the rule's owner, or None when the line is not a rule ask.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let said = _said(text);
    let pool = pool_for(&said, memory).filter(|p| !p.is_empty())?;
    Some(pick(&pool, memory, &said))
}
