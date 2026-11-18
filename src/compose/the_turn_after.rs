//! Port of `bot/compose/the_turn_after.py`: "and tomorrow?" or "can you just check" straight
//! after an honest limit is the same question, and gets the same limit in other words.

use crate::compose::fallback_chain;
use crate::data::{self, Value};
use crate::memory::last_bot;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.the_turn_after.";
static BARE_DAY: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}BARE_DAY")));
static CHECK_REQUEST: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}CHECK_REQUEST")));
static CHECK_REQUEST_ALONE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}CHECK_REQUEST_ALONE")));
static SHORT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}SHORT_WORDS")));
/// `fallback_data.LIVE_KINDS`: (kind, phrases, options).
static LIVE_KINDS: LazyLock<Vec<(String, Vec<String>, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.fallback_data.LIVE_KINDS")
        .items()
        .iter()
        .map(|row| {
            let row: &[Value] = row.items();
            (row[0].str().to_string(), row[1].strs(), row[2].strs())
        })
        .collect()
});
static PLAIN_POOLS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
    ["CANNOT_ANYTHING", "CANNOT_SEE_ANYTHING_LINES", "CANNOT_YOUR_OWN"]
        .iter()
        .map(|name| data::strs(&format!("bot.compose.fallback.{name}")))
        .collect()
});
/// `re.findall(r"[a-z]+", ...)`, written inline.
static AZ: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));

/// Python: bot/compose/the_turn_after.py:66 `_the_limit_just_given(memory)`
///
/// The lines and phrases of the pool the reply just given came from, or (None, ()) - only this
/// file calls it.
pub fn _the_limit_just_given(memory: &Memory) -> (Option<Vec<String>>, Vec<String>) {
    // Memory that cannot be read claims nothing.
    let Some(said) = fallback_chain::caught(|| last_bot(memory)) else { return (None, Vec::new()) };
    let said = text::strip(&said).to_string();
    if said.is_empty() {
        return (None, Vec::new());
    }
    let opens = |line: &String| {
        let head = text::slice(line, 0, Some(40));
        !head.is_empty() && said.starts_with(&head)
    };
    for (_kind, phrases, options) in LIVE_KINDS.iter() {
        if options.iter().any(opens) {
            return (Some(options.clone()), phrases.clone());
        }
    }
    for pool in PLAIN_POOLS.iter() {
        if pool.iter().any(opens) {
            return (Some(pool.clone()), Vec::new());
        }
    }
    (None, Vec::new())
}

/// Python: bot/compose/the_turn_after.py:90 `_the_hours_pool()`
///
/// The composer's honest lines for a venue's opening hours, or ().
pub fn _the_hours_pool() -> Vec<String> {
    LIVE_KINDS.iter().find(|(kind, _, _)| kind == "hours").map(|(_, _, options)| options.clone()).unwrap_or_default()
}

/// Python: bot/compose/the_turn_after.py:98 `_words_of(phrases)`
///
/// The words a pool names its subject with, four letters or more, and the short ones too.
pub fn _words_of(phrases: &[String]) -> HashSet<String> {
    phrases
        .iter()
        .flat_map(|phrase| AZ.findall_str(&text::lower(phrase)))
        .filter(|word| text::len(word) >= 4 || SHORT_WORDS.contains(word))
        .collect()
}

/// Python: bot/compose/the_turn_after.py:107 `the_day_after(text, memory)`
///
/// "And tomorrow" straight after the honest hours line is the same question, another day.
pub fn the_day_after(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::lower(text::strip(text));
    if BARE_DAY.match_(&clean).is_none() {
        return None;
    }
    let hours = _the_hours_pool();
    if hours.is_empty() {
        return None;
    }
    let (just, _phrases) = _the_limit_just_given(memory);
    if just.as_ref() != Some(&hours) {
        return None;
    }
    fallback_chain::_choose(&hours, memory, &clean, None)
}

/// Python: bot/compose/the_turn_after.py:120 `asked_to_check(text, memory)`
///
/// "Can you just check" straight after the limit is the same limit, said in other words.
pub fn asked_to_check(text: &str, memory: &Memory) -> Option<String> {
    let clean = text::lower(text::strip(text));
    if !CHECK_REQUEST.is_match(&clean) {
        return None;
    }
    let (just, phrases) = _the_limit_just_given(memory);
    let just = just.filter(|j| !j.is_empty())?;
    if CHECK_REQUEST_ALONE.match_(&clean).is_none() {
        let covered = _words_of(&phrases);
        if covered.is_empty() {
            return None;
        }
        let said = AZ.findall_str(&clean);
        if !covered.iter().any(|word| said.contains(word)) {
            return None;
        }
    }
    fallback_chain::_choose(&just, memory, &clean, None)
}

/// Python: bot/compose/the_turn_after.py:137 `answer_for(text, memory)`
///
/// The turn after an honest limit, or None when this is not that turn.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    the_day_after(text, memory).filter(|s| !s.is_empty()).or_else(|| asked_to_check(text, memory))
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
        if let Some(got) = unless_unported(|| { let mut v: Vec<String> = _words_of(&["opening hours".to_string(), "is it open".to_string(), "the bus".to_string()]).into_iter().collect(); v.sort(); v }) {
            assert_eq!(got, vec!["bus".to_string(), "hours".to_string(), "open".to_string(), "opening".to_string()] as Vec<String>);
        }
    }
}
