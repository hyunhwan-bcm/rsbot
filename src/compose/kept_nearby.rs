//! Port of `bot/compose/kept_nearby.py`: a short follow-up that names no kind of its own ("and
//! the kids, do they get one each") takes the kind of place its answer is kept from the person's
//! own last line, in place of the plain hedge.

use crate::compose::fallback_data::the_place_the_answer_is_kept;
use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.kept_nearby.";
static PLAIN_HEADS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}PLAIN_HEADS")));
static LEANING: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}LEANING")));
static SHORT: LazyLock<usize> = LazyLock::new(|| data::get(&format!("{M}SHORT")).as_int().unwrap() as usize);
static ASKING: LazyLock<HashSet<String>> = LazyLock::new(|| data::set(&format!("{M}ASKING")));
static A_METHOD: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}A_METHOD")));
static BAGGAGE: LazyLock<&'static Re> = LazyLock::new(|| data::re(&format!("{M}BAGGAGE")));
static BAGGAGE_POOL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs(&format!("{M}BAGGAGE_POOL")));
/// `re.split(r"[^a-z']+", line)`, written inline in `answer_for`.
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z']+", 0));

/// Python: bot/compose/kept_nearby.py:87 `_turns(memory)`: the turns (only this file calls it).
pub fn _turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// Python: bot/compose/kept_nearby.py:91 `_clean(text)`
pub fn _clean(text: &str) -> String {
    text::squash(&text::lower(text))
}

/// Python: bot/compose/kept_nearby.py:95 `_without_plurals(line)`
///
/// The line with the s taken off every word longer than three letters ("orders" -> "order"),
/// but not off "ss", "us" or "is".
pub fn _without_plurals(line: &str) -> String {
    text::split_ref(line)
        .iter()
        .map(|word| {
            if text::len(word) > 3 && word.ends_with('s') && !(word.ends_with("ss") || word.ends_with("us") || word.ends_with("is")) {
                word[..word.len() - 1].to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Python: bot/compose/kept_nearby.py:113 `is_the_plain_hedge(said)`
///
/// True when the reply is the plain pointer and not one of the hedges that already say more.
pub fn is_the_plain_hedge(said: &str) -> bool {
    let head = text::slice(said, 0, Some(40));
    PLAIN_HEADS.iter().any(|h| *h == head)
}

/// Python: bot/compose/kept_nearby.py:118 `_inheritable(pool)`
///
/// Whether this kind's own line can stand on a kind borrowed from the turn above (only this file
/// calls it, with the pool's lines).
pub fn _inheritable(pool: &[String]) -> bool {
    !pool.iter().any(|line| text::lower(line).contains("council"))
}

/// Python: bot/compose/kept_nearby.py:130 `_the_kind(line, borrowable=False)`
///
/// The kind a line names itself, as the table or this file knows it, or None.
pub fn _the_kind(line: &str, borrowable: bool) -> Option<Vec<String>> {
    for said in [line.to_string(), _without_plurals(line)] {
        if let Some(pool) = the_place_the_answer_is_kept(&said).filter(|p| !p.is_empty()) {
            if borrowable && !_inheritable(&pool) {
                continue;
            }
            return Some(pool);
        }
        if BAGGAGE.is_match(&said) {
            return Some(BAGGAGE_POOL.clone());
        }
    }
    None
}

/// Python: bot/compose/kept_nearby.py:142 `answer_for(text, memory=None)`
///
/// The place this follow-up's answer is kept, or None when the thread names no kind.
pub fn answer_for(text: &str, memory: &Memory) -> Option<Vec<String>> {
    let line = _clean(text);
    if line.is_empty() {
        return None;
    }
    if the_place_the_answer_is_kept(&line).is_some_and(|p| !p.is_empty()) {
        return None;
    }
    if BAGGAGE.is_match(&line) {
        return Some(BAGGAGE_POOL.clone());
    }
    if !LEANING.is_match(&line) {
        return None;
    }
    if A_METHOD.is_match(&line) {
        return None;
    }
    let said: Vec<String> = NOT_A_WORD.split(&line).into_iter().filter(|w| !w.is_empty()).collect();
    if said.len() > *SHORT {
        return None;
    }
    if said.iter().any(|word| !ASKING.contains(word)) {
        return None;
    }
    let turns = _turns(memory);
    let Some(last) = turns.last() else { return None };
    let mut near = vec![last.user.as_str()];
    if turns.len() > 1 && LEANING.is_match(&_clean(&last.user)) {
        near.push(&turns[turns.len() - 2].user);
    }
    for said in near {
        let said = _clean(said);
        if said.is_empty() || said == line {
            continue;
        }
        if let Some(pool) = _the_kind(&said, true) {
            return Some(pool);
        }
    }
    None
}
