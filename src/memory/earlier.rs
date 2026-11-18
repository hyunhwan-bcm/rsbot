//! Port of `bot/memory/earlier.py`: one check for what the conversation already holds - is there
//! a line of theirs that carried it?
//!
//! `carried()` answers "which of their lines has a value of this kind", and `put_right()` is the
//! gate every store reply goes through: a reply that denies a thing the chat says gets the line
//! handed back instead of the denial. The kinds are data (`KINDS`).
#![allow(non_snake_case, clippy::all)]

use crate::data;
use crate::memory::mem7::{self, inline_re, lists, res, sets};
use crate::py::re::{self, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

res!("earlier"; CLAIM, A_RATE, RECALL, HAND_BACKS, QUESTION, THEIR_OWN_WORDS, NOT_A_RECALL_ASK);
sets!("earlier"; VALUE_WORDS, PRONOUNS, STOP);
lists!("earlier"; HAND_BACK, A_TIMED_KIND, NOT_A_PLAIN_ASK);
inline_re! {
    WORDS = r"[a-z']+", 0;
    NOT_A_VALUE_CHAR = r"[^a-z0-9. ]", 0;
    HOW_MUCH = r"\bhow much\b", re::I;
    QUOTED = "\"([^\"]{2,300})\"", 0;
}
static KINDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.memory.earlier.KINDS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_re().expect("a kind pattern"))).collect()
});
static ASKS: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.memory.earlier.ASKS").items().iter().map(|p| (p.items()[0].as_re().expect("an ask pattern"), p.items()[1].str().to_string())).collect()
});

/// A line of theirs: (turn number, counted from one; the line).
pub type Said = (i64, String);

/// Python: bot/memory/earlier.py:130 `_a_word(word)`
///
/// A word that can be the subject of a claim.
pub fn _a_word(word: &str) -> bool {
    !STOP.contains(word) && !PRONOUNS.contains(word) && !VALUE_WORDS.contains(word) && pt::len(word) > 2 && !word.contains('\'')
}

/// Python: bot/memory/earlier.py:145 `_turns(memory)`
pub fn _turns(memory: &Memory) -> &[crate::state::Turn] {
    &memory.turns
}

/// Python: bot/memory/earlier.py:152 `said_lines(memory)`
///
/// What the person said, oldest first, as (turn number, line); a question of theirs is skipped.
pub fn said_lines(memory: &Memory) -> Vec<Said> {
    let mut said = Vec::new();
    for (number, turn) in _turns(memory).iter().enumerate() {
        let line = pt::strip(&turn.user);
        if line.is_empty() || QUESTION.match_(line).is_some() || line.ends_with('?') {
            continue;
        }
        said.push((number as i64 + 1, line.to_string()));
    }
    said
}

/// `" ".join(text.lower().replace("’", "'").split())`.
fn the_ask(text: &str) -> String {
    pt::squash(&pt::lower(text).replace('\u{2019}', "'"))
}

/// Python: bot/memory/earlier.py:172 `kind_of(text)`
///
/// The kind of value this line asks for, or None.
pub fn kind_of(text: &str) -> Option<String> {
    let low = the_ask(text);
    if low.is_empty() {
        return None;
    }
    ASKS.iter().find(|(pattern, _)| pattern.is_match(&low)).map(|(_, kind)| kind.clone())
}

fn kind_pattern(kind: &str) -> Option<&'static Re> {
    KINDS.iter().find(|(k, _)| k == kind).map(|(_, p)| *p)
}

/// Python: bot/memory/earlier.py:196 `carries(kind, line)`
///
/// Whether this line holds a value of that kind.
pub fn carries(kind: &str, line: &str) -> bool {
    let Some(pattern) = kind_pattern(kind) else {
        return false;
    };
    if A_TIMED_KIND.iter().any(|k| k == kind) {
        return pattern.is_match(&A_RATE.sub(" ", line));
    }
    pattern.is_match(line)
}

/// The `[a-z']+` words of a squashed, lower-case text.
fn words_of(text: &str) -> Vec<String> {
    WORDS.findall_str(&pt::squash(&pt::lower(text)))
}

/// Python: bot/memory/earlier.py:207 `_about(text)`
///
/// The words a claim names as its subject, none of them pronouns.
pub fn _about(text: &str) -> Vec<String> {
    words_of(text).into_iter().filter(|w| _a_word(w)).collect()
}

/// Python: bot/memory/earlier.py:213 `_said_words(text)`
///
/// The words of a line that could name the thing it is about, in order, each once.
pub fn _said_words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in words_of(text) {
        if pt::len(&word) > 1 && !STOP.contains(&word) && !PRONOUNS.contains(&word) && !VALUE_WORDS.contains(&word) && !word.contains('\'') && !out.contains(&word) {
            out.push(word);
        }
    }
    out
}

/// Python: bot/memory/earlier.py:224 `_mentions(line, words)`
///
/// Whether the line mentions any of these words, in any of its forms.
pub fn _mentions<S: AsRef<str>>(line: &str, words: &[S]) -> bool {
    let low = pt::squash(&pt::lower(line));
    words.iter().any(|word| mem7::cached_re(&format!(r"\b{}(?:s|es|'s|ing|ed)?\b", re::escape(word.as_ref())), 0).is_match(&low))
}

/// Python: bot/memory/earlier.py:238 `kinds_for(kind, ask)`
///
/// The kinds a value asked for could be in, the likeliest first.
pub fn kinds_for(kind: &str, ask: &str) -> Vec<String> {
    if kind == "money" && HOW_MUCH.is_match(ask) {
        return vec!["money".into(), "size".into()];
    }
    vec![kind.to_string()]
}

/// Python: bot/memory/earlier.py:251 `carried(memory, kind, about=(), window=None, prefer=(), after=0)`
///
/// The latest line of theirs that carried a value of this kind, or None.
pub fn carried<A: AsRef<str>, P: AsRef<str>>(memory: &Memory, kind: &str, about: &[A], window: Option<usize>, prefer: &[P], after: i64) -> Option<Said> {
    let mut lines = said_lines(memory);
    if let Some(window) = window.filter(|w| *w > 0) {
        lines = mem7::tail(&lines, window).to_vec();
    }
    if after != 0 {
        lines.retain(|(number, _)| *number > after);
    }
    let hits: Vec<Said> = lines.into_iter().filter(|(_, line)| carries(kind, line)).collect();
    if hits.is_empty() {
        return None;
    }
    if !about.is_empty() {
        if let Some(hit) = hits.iter().rev().find(|(_, line)| _mentions(line, about)) {
            return Some(hit.clone());
        }
    }
    if !prefer.is_empty() {
        let (mut best, mut best_hit) = (0usize, None);
        for hit in &hits {
            let score = prefer.iter().filter(|word| _mentions(&hit.1, &[word.as_ref()])).count();
            if score >= best {
                best = score;
                best_hit = Some(hit.clone());
            }
        }
        if best > 0 {
            return best_hit;
        }
    }
    if !about.is_empty() {
        // The thing named is not in any line of this kind.
        return None;
    }
    hits.last().cloned()
}

/// Python: bot/memory/earlier.py:287 `_pointing_back(memory, about)`
///
/// Whether a line of the chat names what the ask names, so the ask is pointing at it.
pub fn _pointing_back<S: AsRef<str>>(memory: &Memory, about: &[S]) -> bool {
    !about.is_empty() && said_lines(memory).iter().any(|(_, line)| _mentions(line, about))
}

/// Python: bot/memory/earlier.py:299 `carried_alone(memory, kind)`
///
/// The one line of the chat that carries a value of this kind, when there is exactly one.
pub fn carried_alone(memory: &Memory, kind: &str) -> Option<String> {
    let hits: Vec<String> = said_lines(memory).into_iter().map(|(_, line)| line).filter(|line| carries(kind, line)).collect();
    if hits.len() == 1 {
        hits.into_iter().next()
    } else {
        None
    }
}

/// Python: bot/memory/earlier.py:311 `hand_back(line, memory=None, text="")`
///
/// Their own line, quoted, in the plainest words there are for it.
pub fn hand_back(line: &str, memory: Option<&Memory>, text: &str) -> Option<String> {
    let low = pt::squash(line);
    if low.is_empty() {
        return None;
    }
    let empty = Memory::new();
    let picked = mem7::pick(&HAND_BACK, memory.unwrap_or(&empty), &format!("{low} {text}"));
    Some(mem7::fmt(&picked, &[("line", &low)]))
}

/// Python: bot/memory/earlier.py:319 `a_claim(reply)`
///
/// The part of a reply that claims the person never said something, or None.
pub fn a_claim(reply: &str) -> Option<String> {
    CLAIM.search(reply).map(|found| reply[found.start()..].to_string())
}

/// Python: bot/memory/earlier.py:336 `_values(kind, text)`
///
/// The values of a kind one line carries, as plain words, for telling two lines apart.
pub fn _values(kind: &str, text: &str) -> HashSet<String> {
    let Some(pattern) = kind_pattern(kind) else {
        return HashSet::new();
    };
    let low = pt::squash(&pt::lower(text));
    pattern.finditer(&low).iter().map(|m| pt::squash(&NOT_A_VALUE_CHAR.sub(" ", m.as_str()))).filter(|p| !p.is_empty()).collect()
}

/// Python: bot/memory/earlier.py:350 `_the_line_the_ask_names(text, memory)`
///
/// (the kind asked for, the words the ask names, the line it means), any of them empty.
pub fn _the_line_the_ask_names(text: &str, memory: &Memory) -> (Option<String>, Vec<String>, Option<Said>) {
    let low = the_ask(text);
    if !(RECALL.is_match(&low) || their_own_words(&low)) {
        return (None, Vec::new(), None);
    }
    let kind = kind_of(&low);
    let named = _said_words(&low);
    let Some(kind) = kind.filter(|_| !named.is_empty()) else {
        return (None, named, None);
    };
    let found = carried::<&str, String>(memory, &kind, &[], None, &named, 0);
    (Some(kind), named, found)
}

/// Python: bot/memory/earlier.py:367 `gives_the_named_value(text, memory, reply)`
///
/// Whether the reply gives the value of the line the ask names, not another line's value.
pub fn gives_the_named_value(text: &str, memory: &Memory, reply: &str) -> bool {
    let (kind, _named, found) = _the_line_the_ask_names(text, memory);
    let Some((_, line)) = found else {
        return false;
    };
    let kind = kind.unwrap_or_default();
    !_values(&kind, reply).is_disjoint(&_values(&kind, &line))
}

/// Python: bot/memory/earlier.py:375 `another_line_was_asked_for(text, memory, reply)`
///
/// True when a recall ask that names something was answered from a different line.
pub fn another_line_was_asked_for(text: &str, memory: &Memory, reply: &str) -> bool {
    let (_kind, named, found) = _the_line_the_ask_names(text, memory);
    if found.is_none() {
        return false;
    }
    if _mentions(reply, &named) {
        return false;
    }
    !gives_the_named_value(text, memory, reply)
}

/// Python: bot/memory/earlier.py:396 `a_misquoted_line(reply, memory, ask="")`
///
/// The line to hand back when the reply quotes a line that does not carry what was asked.
pub fn a_misquoted_line(reply: &str, memory: &Memory, ask: &str) -> Option<String> {
    if HAND_BACKS.match_(reply).is_none() {
        return None;
    }
    let quoted = QUOTED.search(reply);
    let kind = kind_of(ask);
    let (Some(quoted), Some(kind)) = (quoted, kind) else {
        return None;
    };
    if !_values(&kind, quoted.str(1)).is_empty() {
        return None;
    }
    let (_, line) = carried::<&str, &str>(memory, &kind, &[], None, &[], 0)?;
    if pt::squash(&pt::lower(&line)) == pt::squash(&pt::lower(quoted.str(1))) {
        return None;
    }
    hand_back(&line, Some(memory), &a_claim(reply).unwrap_or_default())
}

/// The line an ask means: by its subject, by the words it names, or the newest of the kind.
fn the_line_for(memory: &Memory, kind: &str, about: &[String], named: &[String]) -> Option<Said> {
    if !about.is_empty() {
        return carried::<String, &str>(memory, kind, about, None, &[], 0);
    }
    carried::<&str, String>(memory, kind, &[], None, named, 0).or_else(|| {
        if named.is_empty() {
            carried::<&str, &str>(memory, kind, &[], Some(3), &[], 0)
        } else {
            None
        }
    })
}

/// Python: bot/memory/earlier.py:422 `answer_for(text, memory)`
///
/// A recall ask nothing else answered: their own line, handed back, when it carries the value.
pub fn answer_for(text: &str, memory: &Memory) -> Option<String> {
    let low = the_ask(text);
    if low.is_empty() || !RECALL.is_match(&low) {
        return None;
    }
    let kind = kind_of(&low)?;
    let about = _about(&low);
    let named = _said_words(&low);
    let mut found = the_line_for(memory, &kind, &about, &named);
    if found.is_none() && (named.is_empty() || _pointing_back(memory, &about)) {
        // "the only line of that kind" answers an ask that points at the chat.
        found = carried_alone(memory, &kind).map(|alone| (said_lines(memory).len() as i64, alone));
    }
    let (_, line) = found?;
    hand_back(&line, Some(memory), &low)
}

/// Python: bot/memory/earlier.py:477 `their_own_words(text)`
///
/// Whether the ask is about something the person themselves said or does.
pub fn their_own_words(text: &str) -> bool {
    THEIR_OWN_WORDS.is_match(&pt::squash(&pt::lower(text)))
}

/// Python: bot/memory/earlier.py:498 `from_the_line(text, memory)`
///
/// The answer a line of theirs holds for this question, when nothing else had one.
pub fn from_the_line(text: &str, memory: &Memory) -> Option<String> {
    let low = the_ask(text);
    if low.is_empty() || QUESTION.match_(&low).is_none() {
        return None;
    }
    let kind = kind_of(&low)?;
    if NOT_A_PLAIN_ASK.iter().any(|k| *k == kind) {
        return None;
    }
    if NOT_A_RECALL_ASK.is_match(&low) {
        // A question that wants something worked out is not answered by the last line of that kind.
        return None;
    }
    let about = _about(&low);
    let named = _said_words(&low);
    let mut found = None;
    for kind in kinds_for(&kind, &low) {
        found = the_line_for(memory, &kind, &about, &named);
        if found.is_some() {
            break;
        }
        if named.is_empty() || _pointing_back(memory, &about) {
            found = carried_alone(memory, &kind).map(|alone| (said_lines(memory).len() as i64, alone));
            if found.is_some() {
                break;
            }
        }
    }
    let (_, line) = found?;
    hand_back(&line, Some(memory), &low)
}

/// Python: bot/memory/earlier.py:537 `put_right(reply, memory, ask="")`
///
/// The reply with a false claim of not knowing replaced by their own line, or None.
pub fn put_right(reply: &str, memory: &Memory, ask: &str) -> Option<String> {
    let claim = a_claim(reply)?;
    let asked = kind_of(&claim).or_else(|| kind_of(ask));
    let kinds: Vec<String> = match &asked {
        Some(kind) => vec![kind.clone()],
        None => ["age", "name", "when", "size", "money", "often", "duration"].iter().map(|k| k.to_string()).collect(),
    };
    let named = _about(&claim);
    // Either line can say what to look for.
    let subject = if named.is_empty() { _about(ask) } else { named.clone() };
    let named_here = !named.is_empty() && _mentions(ask, &named);
    // A pronoun points at the line just said.
    let window = if subject.is_empty() { Some(3) } else { None };
    for kind in &kinds {
        if let Some((_, line)) = carried::<String, &str>(memory, kind, &subject, window, &[], 0) {
            return hand_back(&line, Some(memory), &claim);
        }
        if !subject.is_empty() && named.is_empty() {
            // The claim named nobody and the ask's own words named no line: the line just said.
            if let Some((_, line)) = carried::<&str, &str>(memory, kind, &[], Some(3), &[], 0) {
                return hand_back(&line, Some(memory), &claim);
            }
        } else if !named.is_empty() {
            // A line with a pronoun in it, after the thing was introduced, may be about it.
            let introduced = said_lines(memory).into_iter().filter(|(_, line)| _mentions(line, &named)).map(|(n, _)| n).max().unwrap_or(0);
            let after = if named_here { introduced } else { 0 };
            if let Some((_, line)) = carried::<&str, &str>(memory, kind, &["she", "he", "they"], None, &[], after) {
                return hand_back(&line, Some(memory), &claim);
            }
        }
    }
    if !named.is_empty() && asked.is_none() {
        // No kind was named: a line that mentions the thing is what the person has.
        for (_, line) in said_lines(memory).iter().rev() {
            if _mentions(line, &named) {
                return hand_back(line, Some(memory), &claim);
            }
        }
    }
    None
}
