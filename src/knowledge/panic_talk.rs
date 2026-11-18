//! Port of PyBot's `bot/knowledge/panic_talk.py`: panic attacks, the person's own or someone's.

use crate::data::{self, FnRef};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::understood::Understood;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.panic_talk.{name}"))
}
static _MINE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_MINE"));
static _BEHIND_THEM: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BEHIND_THEM"));
static _THEM_OR_THEIR_PAST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = strs("_THEM");
    v.extend(strs("_THEIR_OWN_PAST"));
    v
});
static _ADVICE_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ADVICE_ASK"));
static _HABIT_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HABIT_VERBS"));
static _HABIT_MARKERS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HABIT_MARKERS"));
static _HABIT_ONCE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_HABIT_ONCE"));

fn has_word<S: AsRef<str>>(low: &str, phrases: &[S]) -> bool {
    phrases.iter().any(|p| low.contains(&format!(" {} ", p.as_ref())))
}

/// The habit reading's complement: a line about one attack, not panics as a habit.
pub fn _a_flat_habit(understood: &Understood) -> bool {
    !_a_habit_is_not_a_minute(understood)
}

/// Whether the attack in the line is the speaker's own (or long behind someone), not a
/// relative's current one.
pub fn _the_attack_is_the_speakers_own(understood: &Understood) -> bool {
    let low = format!(" {} ", _text(understood));
    if has_word(&low, &_MINE) {
        return true;
    }
    if has_word(&low, &["i had", "ive had", "i have had", "i get", "i used to", "i was having"]) {
        return true;
    }
    if has_word(&low, &_BEHIND_THEM) {
        return true;
    }
    !has_word(&low, &_THEM_OR_THEIR_PAST)
}

/// Whether the line asks what to do (advice), not just tells.
pub fn _the_line_asks_what_to_do(understood: &Understood) -> bool {
    let low = format!(" {} ", _text(understood));
    let stripped = crate::py::text::strip(&low);
    _ADVICE_ASK.iter().any(|p| low.contains(&format!(" {p} ")) || stripped.starts_with(p.as_str()))
}

/// Whether the line is not about panics as a habit (or, if it is, asks what to do about them).
pub fn _a_habit_is_not_a_minute(understood: &Understood) -> bool {
    let low = format!(" {} ", _text(understood));
    let mut habit = has_word(&low, &_HABIT_VERBS);
    if !habit {
        let once = _HABIT_ONCE.iter().any(|p| low.contains(p.as_str()));
        habit = !once && has_word(&low, &_HABIT_MARKERS);
    }
    !habit || _the_line_asks_what_to_do(understood)
}

/// The line as lower-case words (the clean line when there are none).
pub fn _text(understood: &Understood) -> String {
    if !understood.words.is_empty() {
        return understood.words.join(" ").to_lowercase();
    }
    understood.clean.to_lowercase()
}

fn g_flat(_f: &FnRef, u: &Understood) -> bool {
    _a_flat_habit(u)
}
fn g_minute(_f: &FnRef, u: &Understood) -> bool {
    _a_habit_is_not_a_minute(u)
}
fn g_own(_f: &FnRef, u: &Understood) -> bool {
    _the_attack_is_the_speakers_own(u)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.panic_talk:_a_flat_habit:151", g_flat),
    ("bot.knowledge.panic_talk:_a_habit_is_not_a_minute:183", g_minute),
    ("bot.knowledge.panic_talk:_the_attack_is_the_speakers_own:156", g_own),
];
pub const REPLIES: &[(&str, ReplyFn)] = &[];
