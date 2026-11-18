//! Port of `bot/compose/safetyask.py`: "is it safe to leave it on" about a thing of theirs that
//! plugs in gets the plain test (a burning smell, sparks, heat), unless a card answers it better
//! or the bot already ruled it off.

use crate::compose::fallback_chain::{caught, py_format};
use crate::data;
use crate::memory::{last_bot, last_user};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.safetyask.";
macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.safetyask.", stringify!($name))));)*
    };
}
regexes!(ASK_IF_A_THING_IS_SAFE, A_SAFETY_WORD, A_FAULT_DESCRIBED, A_DANGER_SAID);
static A_THING_THAT_PLUGS_IN: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}A_THING_THAT_PLUGS_IN")));
static A_SAFETY_TEST_LINES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}A_SAFETY_TEST_LINES")));
static A_RULING_ALREADY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}A_RULING_ALREADY")));
static A_SAFETY_ALREADY_RULED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}A_SAFETY_ALREADY_RULED")));
/// `re.findall(r"[a-z']{3,20}", ...)`, written inline in `_the_thing_on_the_table`.
static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']{3,20}", 0));
const SHRUGS: [&str; 12] = [
    "i don't know enough", "i do not know enough", "beyond what i know", "rather not guess",
    "not sure about that one", "i'm not sure about that", "search should sort", "quick search",
    "search online", "better sources", "can't call that one", "cant call that one",
];

/// `str(last_user(memory) or "")`
fn the_line_before(memory: &Memory) -> String {
    last_user(memory, 1).as_str().unwrap_or("").to_string()
}

/// Python: bot/compose/safetyask.py:84 `reply_for(clean, memory)`
///
/// The lines for a safety question about a thing of theirs, or None.
pub fn reply_for(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let text = text::squash(&text::lower(clean));
    if text.is_empty() || !ASK_IF_A_THING_IS_SAFE.is_match(&text) {
        return None;
    }
    let thing = _the_thing_on_the_table(&text, memory);
    if thing.is_empty() {
        return None;
    }
    if !(_a_safety_word(&text) || _the_thing_is_described_faulty(memory, &thing)) {
        return None;
    }
    if let Some(card) = _the_answers_have_this(&text, memory) {
        if text::lower(&card).contains(&thing) || !_a_shrug(&card) {
            return None;
        }
    }
    if the_bot_already_ruled(memory) {
        return Some(A_SAFETY_ALREADY_RULED.clone());
    }
    let named = format!("the {thing}");
    Some(A_SAFETY_TEST_LINES.iter().map(|line| py_format(line, &[("thing", &named)])).collect())
}

/// Python: bot/compose/safetyask.py:105 `_a_safety_word(clean)`
pub fn _a_safety_word(clean: &str) -> bool {
    A_SAFETY_WORD.is_match(&text::lower(clean))
}

/// Python: bot/compose/safetyask.py:110 `_the_thing_is_described_faulty(memory, thing)`
///
/// True when the line before describes the thing as faulty in a way that could bite.
pub fn _the_thing_is_described_faulty(memory: &Memory, thing: &str) -> bool {
    let said = text::squash(&text::lower(&the_line_before(memory)));
    if said.is_empty() || thing.is_empty() || !said.contains(thing) {
        return false;
    }
    A_FAULT_DESCRIBED.is_match(&said)
}

/// Python: bot/compose/safetyask.py:121 `_a_shrug(reply)`
///
/// True when an answer to this line says the bot cannot answer it.
pub fn _a_shrug(reply: &str) -> bool {
    let said = text::lower(reply);
    SHRUGS.iter().any(|phrase| said.contains(phrase))
}

/// Python: bot/compose/safetyask.py:133 `_the_thing_on_the_table(clean, memory)`
///
/// The appliance a follow-up safety question is about (the last one named, in the line or else
/// in the line before), or "".
pub fn _the_thing_on_the_table(clean: &str, memory: &Memory) -> String {
    let in_the_line = |said: &str| A_WORD.findall_str(&text::lower(said)).into_iter().filter(|w| A_THING_THAT_PLUGS_IN.contains(w)).last();
    if let Some(found) = in_the_line(clean) {
        return found;
    }
    in_the_line(&the_line_before(memory)).unwrap_or_default()
}

/// Python: bot/compose/safetyask.py:152 `the_bot_already_ruled(memory)`
///
/// Whether the bot's own last reply told the person to stop using the thing, and why not to.
pub fn the_bot_already_ruled(memory: &Memory) -> bool {
    let said = text::lower(&last_bot(memory));
    if !A_RULING_ALREADY.iter().any(|phrase| said.contains(phrase.as_str())) {
        return false;
    }
    A_DANGER_SAID.is_match(&said)
}

/// Python: bot/compose/safetyask.py:164 `_the_answers_have_this(clean, memory)`
///
/// The knowledge module's own answer to this line, asked with a copy of the conversation, or None.
pub fn _the_answers_have_this(clean: &str, memory: &Memory) -> Option<String> {
    let mut copy = memory.clone();
    let said = caught(|| crate::modules::answers::respond(clean, &mut copy)).flatten()?;
    if text::strip(&said).is_empty() {
        return None;
    }
    Some(said)
}

// ---- COM5-CASES (generated by harness/com5_cases.py; expected values from PyBot) ----
#[cfg(test)]
mod pybot_cases {
    #![allow(unused_imports, dead_code)]
    use super::*;
    use crate::data::Value;
    use crate::state::{Memory, Turn};

    fn mem(turns: &[(&str, &str, &str)]) -> Memory {
        let mut m = Memory::new();
        m.turns = turns
            .iter()
            .map(|(u, b, md)| Turn { user: u.to_string(), bot: b.to_string(), module: (!md.is_empty()).then(|| md.to_string()) })
            .collect();
        m
    }

    /// The call's result, or None when it reached a function another workstream has not ported yet.
    fn unless_unported<T>(f: impl FnOnce() -> T) -> Option<T> {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
            Ok(v) => Some(v),
            Err(e) => {
                let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                if msg.starts_with("UNPORTED") {
                    eprintln!("skipped: {msg}");
                    return None;
                }
                std::panic::resume_unwind(e)
            }
        }
    }

    /// True when the call raises (panics), as the Python does.
    fn raises<T>(f: impl FnOnce() -> T) -> bool {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
    }

    #[test]
    fn case_0() {
        if let Some(got) = unless_unported(|| _a_safety_word("is it safe")) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_1() {
        if let Some(got) = unless_unported(|| _a_safety_word("is the heater safe to leave on")) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_2() {
        if let Some(got) = unless_unported(|| _a_safety_word("is the plug socket cracked dangerous")) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_3() {
        if let Some(got) = unless_unported(|| _a_shrug("A quick search online will get you there.")) {
            assert_eq!(got, true);
        }
    }

    #[test]
    fn case_4() {
        if let Some(got) = unless_unported(|| _a_shrug("The boiler needs a Gas Safe engineer.")) {
            assert_eq!(got, false);
        }
    }
}
