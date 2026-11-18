//! Port of `bot/compose/childwords.py`: "how do i tell my son", in a thread where somebody has
//! died, is an ask for the words to use with a child.

use crate::compose::fallback_chain::caught;
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.childwords.", stringify!($name))));)*
    };
}
regexes!(
    A_CHILD_DIED, A_DEATH_IN_THE_LINE, A_SUICIDE_IN_THE_LINE, TELL_ASK, A_CHILD_IN_THE_ASK,
    _A_PERSON_IN_THE_TAIL, A_DEATH_IN_THE_TAIL,
);
static TAIL_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.childwords.TAIL_FILLER"));
static TAIL_ABOUT_THE_ONE_WHO_DIED: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.compose.childwords.TAIL_ABOUT_THE_ONE_WHO_DIED"));
/// `re.findall(r"[a-z']+", tail)`, written inline in `_the_tail_is_about_the_death`.
static AZ_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// `turns[-look_back:]` with Python's slice rules (0 is the whole list).
fn last_turns(turns: &[Turn], look_back: i64) -> &[Turn] {
    let n = turns.len() as i64;
    let start = if look_back == 0 { 0 } else if look_back > 0 { (n - look_back).max(0) } else { (-look_back).min(n) };
    &turns[start as usize..]
}

/// Python: bot/compose/childwords.py:57 `_the_tail_after_the_child(text)`
///
/// The words after the last child named in the ask, or None when no child is named.
pub fn _the_tail_after_the_child(text: &str) -> Option<String> {
    let last = A_CHILD_IN_THE_ASK.finditer(text).into_iter().last()?;
    Some(text[last.end()..].to_string())
}

/// Python: bot/compose/childwords.py:67 `_the_tail_is_about_the_death(tail)`
///
/// True when what follows the child leaves the ask about the death, and not about something else.
pub fn _the_tail_is_about_the_death(tail: &str) -> bool {
    let words = AZ_WORDS.findall_str(tail);
    if words.is_empty() {
        return true; // "how do i tell my son"
    }
    if A_DEATH_IN_THE_TAIL.is_match(tail) {
        return !_A_PERSON_IN_THE_TAIL.is_match(tail);
    }
    // A word that is neither filler nor about the one who died is a subject of its own.
    words.iter().all(|word| TAIL_FILLER.contains(word) || TAIL_ABOUT_THE_ONE_WHO_DIED.contains(word))
}

/// Python: bot/compose/childwords.py:105 `a_death_in_the_thread(memory, look_back=3)`
///
/// True when the person said a death in one of their last few lines. `look_back` is an int
/// (callers pass 3 for the default).
pub fn a_death_in_the_thread(memory: &Memory, look_back: &Value) -> bool {
    let look_back = look_back.as_int().unwrap_or(3);
    last_turns(&memory.turns, look_back).iter().any(|turn| {
        let said = text::squash(&text::lower(&turn.user));
        !said.is_empty() && A_DEATH_IN_THE_LINE.is_match(&said)
    })
}

/// Python: bot/compose/childwords.py:124 `asks_the_words_for_a_child(clean, memory)`
///
/// A how-do-I-tell ask about a child, in a thread where somebody has died.
pub fn asks_the_words_for_a_child(clean: &str, memory: &Memory) -> bool {
    let text = text::squash(&text::lower(clean));
    if text.is_empty() {
        return false;
    }
    if !(TELL_ASK.is_match(&text) && A_CHILD_IN_THE_ASK.is_match(&text)) {
        return false;
    }
    let thread = caught(|| {
        if !a_death_in_the_thread(memory, &Value::Int(3)) {
            return None;
        }
        Some(_the_last_few_lines(memory, &Value::Int(4)))
    });
    let Some(Some(thread)) = thread else { return false };
    if A_SUICIDE_IN_THE_LINE.is_match(&thread) || A_SUICIDE_IN_THE_LINE.is_match(&text) {
        return false;
    }
    if A_CHILD_DIED.is_match(&text) {
        return false;
    }
    match _the_tail_after_the_child(&text) {
        Some(tail) => _the_tail_is_about_the_death(&tail),
        None => false,
    }
}

/// Python: bot/compose/childwords.py:154 `_the_last_few_lines(memory, look_back=4)`
///
/// The recent lines of the conversation, the person's and the bot's, as one lower-case string.
pub fn _the_last_few_lines(memory: &Memory, look_back: &Value) -> String {
    let look_back = look_back.as_int().unwrap_or(4);
    let mut lines: Vec<&str> = Vec::new();
    for turn in last_turns(&memory.turns, look_back) {
        lines.push(&turn.user);
        lines.push(&turn.bot);
    }
    text::lower(&lines.join(" "))
}
