//! Port of `bot/compose/clause.py`: the question in the clause after the comma ("camping, might
//! rain, what to pack" asks for a list), lifted over a shape the whole line made weak.

#![allow(non_upper_case_globals)]

use crate::py::re::Re;
use crate::py::text;
use crate::understanding::t;
use crate::understood::Understood;
use std::sync::LazyLock;

const M: &str = "bot.compose.clause.";

static NOTHING_TO_A_THING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "NOTHING_TO_A_THING"));
static WHAT_TO_LOOK: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "WHAT_TO_LOOK"));
static _OPENS_A_QUESTION: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "_OPENS_A_QUESTION"));  // t, 3 items
static _SHAPES_FROM_A_CLAUSE: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "_SHAPES_FROM_A_CLAUSE"));  // t, 3 items
static WHAT_TO_PICK: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.compose.fit.", "WHAT_TO_PICK"));
static WHAT_I_NEED: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.compose.fit.", "WHAT_I_NEED"));
static WHAT_YOU_TAKE: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.compose.fit.", "WHAT_YOU_TAKE"));
static WHAT_TO_STEPS: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.compose.fit.", "WHAT_TO_STEPS"));
static WHAT_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| t::re("bot.compose.fit.", "WHAT_TO_DO"));

/// `_said(read)`: the line (clean, else text) lower-cased.
fn _said(read: &Understood) -> String {
    let s = if read.clean.is_empty() { &read.text } else { &read.clean };
    text::squash(&s.to_lowercase())
}

/// `_focus(read, said)`: the asking clause, when it is a part of the line.
fn _focus(read: &Understood, said: &str) -> Option<String> {
    let focus = text::squash(&read.focus.join(" ").to_lowercase().replace('\u{2019}', "'"));
    if focus.is_empty() || focus == said {
        return None;
    }
    if text::len(&focus) as f64 > 0.85 * (text::len(said).max(1) as f64) {
        return None;
    }
    Some(focus)
}

/// `the_clause_ask(read)`: the shape the asking clause names on its own.
pub fn the_clause_ask(read: &Understood) -> Option<String> {
    let said = _said(read);
    let focus = _focus(read, &said)?;
    if !_OPENS_A_QUESTION.iter().any(|p| focus.starts_with(p.as_str())) {
        return None;
    }
    let focus = format!(" {focus} ");
    if WHAT_TO_PICK.is_match(&focus) {
        return Some("pick".to_string());
    }
    if WHAT_I_NEED.is_match(&focus) || WHAT_YOU_TAKE.is_match(&focus) {
        return Some("options".to_string());
    }
    if WHAT_TO_STEPS.is_match(&focus) || WHAT_TO_DO.is_match(&focus) || WHAT_TO_LOOK.is_match(&focus) {
        return Some("steps".to_string());
    }
    None
}

/// `shape(read, want)`: the shape, with the clause's own ask lifted over a weak one.
pub fn shape(read: &Understood, want: Option<String>) -> Option<String> {
    if let Some(w) = &want {
        if _SHAPES_FROM_A_CLAUSE.contains(w) || matches!(w.as_str(), "time" | "amount" | "place" | "person" | "meaning" | "reason" | "number" | "clock" | "price") {
            return want;
        }
    }
    let said = _said(read);
    if NOTHING_TO_A_THING.is_match(&said) && !read.asks.iter().any(|a| a == "list") {
        return Some("pick".to_string());
    }
    if WHAT_TO_LOOK.is_match(&format!(" {said} ")) {
        return Some("steps".to_string());
    }
    the_clause_ask(read).or(want)
}
