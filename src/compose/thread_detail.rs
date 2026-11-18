//! Port of `bot/compose/thread_detail.py`: a detail dropped into a plan's thread (a limit, an
//! attempt, a history) answered with the one question it raises.
#![allow(non_upper_case_globals)]

use crate::compose::aboutness::{is_a_hedge as _a_hedge, restates_the_line, unlike_the_line};
use crate::compose::com4::{self, bundle_res, bundle_vals};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::sync::LazyLock;

bundle_res!("bot.compose.thread_detail"; A_LIMIT, A_PERSON, A_MACHINE_VERB, AN_ATTEMPT, A_HISTORY, _LEAD, _QUESTION_OPEN, SERIOUS);
bundle_vals!("bot.compose.thread_detail"; _REPLIES, _TIDY, _SECOND_PERSON_WORDS);

static TIDY: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| {
    _TIDY
        .items()
        .iter()
        .map(|pair| (Re::new(pair.items()[0].str(), crate::py::re::I), pair.items()[1].str().to_string()))
        .collect()
});

/// Python: bot/compose/thread_detail.py:124 `_clean_line(text)`
pub fn _clean_line(text_: &str) -> String {
    text::squash(&com4::straight(&text_.to_lowercase()))
}

/// Python: bot/compose/thread_detail.py:137 `the_thread_is_serious(turns, line='')`
///
/// True when this thread is about somebody's health, safety or grief rather than a plan.
pub fn the_thread_is_serious(turns: &[Turn], line: &str) -> bool {
    let said = format!("{} {line}", turns.iter().map(|t| t.user.as_str()).collect::<Vec<_>>().join(" "));
    SERIOUS.search(&said).is_some()
}

/// Python: bot/compose/thread_detail.py:143 `fallback_heavy(turns)`
///
/// True when the thread is heavy news, read through the composer's own test for it.
pub fn fallback_heavy(turns: &[Turn]) -> bool {
    com4::attempt(|| {
        let last = turns.last().expect("IndexError: no turn");
        crate::compose::badnews::reads_as_heavy_news(&_clean_line(&last.user))
            || crate::compose::fallback::_the_thread_is_heavy(&Memory { turns: turns.to_vec(), ..Memory::new() })
    })
    .unwrap_or(false)
}

/// Python: bot/compose/thread_detail.py:152 `the_clause_said_back(text)`
///
/// The person's clause, as the bot would write it: leading join words off, punctuation off.
pub fn the_clause_said_back(text_: &str) -> String {
    let mut said = text::strip_chars(&text::squash(text_), " .,!?-").to_string();
    loop {
        let shorter = _LEAD.sub("", &said);
        if shorter == said {
            break;
        }
        said = shorter;
    }
    for (pattern, instead) in TIDY.iter() {
        said = pattern.sub(instead, &said);
    }
    if said.is_empty() {
        return String::new();
    }
    text::upper_first(&said)
}

/// Python: bot/compose/thread_detail.py:167 `the_kind_of_detail(clean)`
///
/// Which kind of detail the line drops into the thread, or None when it drops none.
pub fn the_kind_of_detail(clean: &str) -> Option<String> {
    if A_LIMIT.search(clean).is_some() && A_PERSON.search(clean).is_some() && A_MACHINE_VERB.search(clean).is_none() {
        return Some("limit".into());
    }
    if AN_ATTEMPT.search(clean).is_some() {
        return Some("attempt".into());
    }
    if A_HISTORY.search(clean).is_some() {
        return Some("history".into());
    }
    None
}

/// Python: bot/compose/thread_detail.py:178 `detail_pool(clean, memory)`
///
/// The one question the detail raises, or None when this turn is not that turn.
pub fn detail_pool(clean: &str, memory: &Memory) -> Option<Vec<String>> {
    let clean = _clean_line(clean);
    if text::split_ref(&clean).len() < 3 || clean.contains('?') || _QUESTION_OPEN.match_(&clean).is_some() {
        return None; // a question already has its own answers, and a bare word says nothing
    }
    let turns = &memory.turns;
    if turns.is_empty() {
        return None; // a detail needs a thread to be a detail of
    }
    if fallback_heavy(turns) || the_thread_is_serious(turns, "") {
        return None; // somebody ill or bereaved is owed the care lines
    }
    let kind = the_kind_of_detail(&clean)?;
    let clause = the_clause_said_back(&clean);
    if text::split_ref(&clause).len() < 3 {
        return None;
    }
    let said = _second_person(&clause);
    let lines = _REPLIES.get(&kind).unwrap_or_else(|| panic!("KeyError: {kind:?}")).strs();
    Some(lines.iter().map(|l| l.replacen("%s", &said, 1)).collect())
}

/// Python: bot/compose/thread_detail.py:210 `_second_person(clause)`
pub fn _second_person(clause: &str) -> String {
    let mut words = Vec::new();
    for (at, word) in text::split_ref(clause).into_iter().enumerate() {
        let core = word.trim_end_matches([',', '.', '?', '!']);
        let key = com4::straight(&core.to_lowercase());
        let Some(swap) = _SECOND_PERSON_WORDS.get(&key).map(|v| v.str().to_string()).filter(|s| !s.is_empty()) else {
            words.push(word.to_string());
            continue;
        };
        let swap = if at == 0 { text::upper_first(&swap) } else { swap };
        words.push(format!("{swap}{}", &word[core.len()..]));
    }
    words.join(" ")
}

/// Python: bot/compose/thread_detail.py:224 `says_nothing_about_the_line(reply, text)`
///
/// True when a reply is a hedge, the line given back, or a stock line about nothing.
pub fn says_nothing_about_the_line(reply: &str, text_: &str) -> bool {
    if text::strip(reply).is_empty() {
        return true;
    }
    if text::split_ref(reply).len() > 22 {
        return false; // a card, at card length
    }
    if _a_hedge(reply) || restates_the_line(reply, text_) {
        return true;
    }
    let pair = unlike_the_line(reply, text_);
    pair.items()[0].truthy() && !pair.items()[1].truthy()
}
