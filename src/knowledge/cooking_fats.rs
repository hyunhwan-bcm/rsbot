//! Port of `bot/knowledge/cooking_fats.py`: butter or oil in the pan - the choice between two
//! fats, which the dish table would otherwise answer with a dish for each.

use crate::data::{self, Value};
use crate::knowledge::d2_shared::pick_in;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

const P: &str = "bot.knowledge.cooking_fats.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{P}{name}"))
}

static _ALT_FAT: LazyLock<String> = LazyLock::new(|| g("_ALT_FAT").str().to_string());
static _IN_THE_PAN: LazyLock<Vec<String>> = LazyLock::new(|| g("_IN_THE_PAN").strs());
static _NOT_A_PAN: LazyLock<Vec<String>> = LazyLock::new(|| g("_NOT_A_PAN").strs());
static THE_CARD: LazyLock<String> = LazyLock::new(|| g("THE_CARD").str().to_string());
static THE_DECIDER: LazyLock<Vec<String>> = LazyLock::new(|| g("THE_DECIDER").strs());

/// True when the line is a choice between two fats for a pan, not a request for a dish.
///
/// Python's first test walks the characters of `" " + _ALT_FAT + " "` (a string, not a tuple of
/// phrases), so it holds whenever one of those characters is in the padded line - always, since
/// the line is padded with a space. Kept as written.
pub fn _asks_between(line: &str) -> bool {
    let low = format!(" {} ", _fold(line));
    let padded = format!(" {} ", *_ALT_FAT);
    if !padded.chars().any(|c| low.contains(c)) {
        return false;
    }
    if !_IN_THE_PAN.iter().any(|w| low.contains(&format!(" {w} "))) {
        return false;
    }
    !_NOT_A_PAN.iter().any(|w| low.contains(&format!(" {w} ")))
}

/// The text lower-cased with its runs of space squashed.
pub fn _fold(text_: &str) -> String {
    text::squash(&text::lower(text_))
}

/// What the bot said last, so the same answer is not given word for word twice.
pub fn _last_reply(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

/// `s[:40]`.
fn head(s: &str) -> String {
    text::slice(s, 0, Some(40))
}

/// The card, or - when it is already on the screen - the sentence that decides the pan.
pub fn _fat_for_the_pan(memory: &Memory, _reading: &Understood) -> Option<String> {
    let said = _last_reply(memory);
    if said.starts_with(&head(&THE_DECIDER[0])) || said.starts_with(&head(&THE_DECIDER[1])) {
        return None; // the short answer is on the screen too; a third card would be the same
    }
    if said.starts_with(&head(&THE_CARD)) {
        return Some(pick_in(&THE_DECIDER, memory, ""));
    }
    Some(THE_CARD.clone())
}

pub const GUARDS: &[(&str, GuardFn)] = &[];

pub const REPLIES: &[(&str, ReplyFn)] = &[("bot.knowledge.cooking_fats:_fat_for_the_pan:119", |_, m, u| _fat_for_the_pan(m, u))];
