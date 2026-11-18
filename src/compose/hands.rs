//! Port of `bot/compose/hands.py`: an ask for a job the bot has no hands for (a timer, a diary
//! entry, sending, playing, reading a file, looking something up) gets the honest no and the
//! half it can do.

use crate::data::{self, Value};
use crate::memory::narrow::memory_value;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.hands.", stringify!($name))));)*
    };
}
regexes!(
    ASKED, FOR_ME, FLAT, POINTS_HERE, COMPOSE, TO_ME, LOOKUP, A_HOW_TO_TAIL, NOT_FOR_HELP, OWN_PLAN,
    NOT_A_JOB, MAIL,
);
static JOBS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.compose.hands.JOBS")
        .items()
        .iter()
        .map(|row| {
            let row: &'static [Value] = row.items();
            (row[0].str().to_string(), row[1].as_re().unwrap())
        })
        .collect()
});
static JOBS_UNAVAILABLE: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.compose.hands.JOBS_UNAVAILABLE"));
static ABOUT_SELF: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.compose.fallback.ABOUT_SELF"));
/// `re.sub(r"[^a-z0-9 ]+", " ", ...)`, written inline in `_bare`.
static NOT_A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9 ]+", 0));

/// `from . import pick`: its contract takes the memory as a `Value`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// Python: bot/compose/hands.py:24 `_bare(text)`
///
/// The line as these readings see it: lowercase, no punctuation, no apostrophes.
pub fn _bare(text: &str) -> String {
    let said = text::lower(text).replace('\u{2019}', "'").replace('\'', "");
    text::squash(&NOT_A_WORD.sub(" ", &said))
}

/// Python: bot/compose/hands.py:179 `reads(text)`
///
/// The job this line asks the bot to do ("timer", "diary", "note", "send", "play", "read" or
/// "lookup"), or None.
pub fn reads(text: &str) -> Option<String> {
    let line = _bare(text);
    if line.is_empty() || text::split_ref(&line).len() > 18 {
        return None;
    }
    if TO_ME.is_match(&line) || COMPOSE.is_match(&line) || NOT_FOR_HELP.is_match(&line) {
        return None;
    }
    let asked = ASKED.match_(&line).is_some() || FOR_ME.is_match(&line);
    if asked && LOOKUP.is_match(&line) && !A_HOW_TO_TAIL.is_match(&line) {
        return Some("lookup".into());
    }
    if NOT_A_JOB.match_(&line).is_some() {
        return None;
    }
    let flat = FLAT.match_(&line).is_some();
    if !asked && !flat {
        return None;
    }
    for (kind, shape) in JOBS.iter() {
        if !shape.is_match(&line) {
            continue;
        }
        if flat && !asked && (kind == "send" || kind == "note" || OWN_PLAN.is_match(&line)) {
            if !POINTS_HERE.is_match(&line) || OWN_PLAN.is_match(&line) {
                return None;
            }
        }
        return Some(kind.clone());
    }
    None
}

/// Python: bot/compose/hands.py:209 `writes_the_words(text)`
///
/// True for the asks whose half the bot can do is the writing: a message, a note, a page.
pub fn writes_the_words(text: &str) -> bool {
    matches!(reads(text).as_deref(), Some("send" | "note" | "read" | "play"))
}

/// Python: bot/compose/hands.py:222 `reply_for(text, memory)`
///
/// The reply for an ask the bot has no hands for, or None when the line is not one.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let kind = reads(text).filter(|k| !k.is_empty())?;
    if kind == "lookup" {
        let pool = ABOUT_SELF.get("online").expect("KeyError: 'online'").strs();
        return Some(pick(&pool, memory, &_bare(text)));
    }
    let mut pool = JOBS_UNAVAILABLE.get(&kind).expect("KeyError").strs();
    if kind == "read" {
        pool = if MAIL.is_match(&_bare(text)) { vec![pool[2].clone()] } else { pool[..2].to_vec() };
    }
    Some(pick(&pool, memory, &_bare(text)))
}
