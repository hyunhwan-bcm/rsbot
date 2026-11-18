//! Port of `bot/memory/healing.py`: a healing ask answered from the card the person was just
//! given.
//!
//! "my back hurts" gets a paragraph about what helps, and the next line is "how long will it
//! take to heal". The span that paragraph states is the answer; where it states none, the plain
//! fact that what the bot has is the care rather than the timeline. Only a line that asks how
//! long and names recovering, with a care card in front of it, is this module's.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! hres {
    ($($n:ident),* $(,)?) => { $(
        static $n: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.memory.healing.", stringify!($n))));
    )* };
}
hres!(ASKS, CARE, HEDGE, SENTENCE, WORD, _TIMELINE, _ONLY_A_SPAN, _REPEAT_LEAD);

static NO_TIMELINE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.healing.NO_TIMELINE"));
static NO_TIME_THEN_STEP: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.healing.NO_TIME_THEN_STEP"));
static _ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.healing._ASK_WORDS"));

/// Python: bot/memory/healing.py:123 `asks(text)`
///
/// Whether the line asks how long something takes to mend.
pub fn asks(text: &str) -> bool {
    !pt::strip(text).is_empty() && ASKS.is_match(text)
}

/// Python: bot/memory/healing.py:131 `the_timeline(answer)`
///
/// The sentence of an answer that states how long a thing takes to mend, or "". The sentence
/// is cut at the end of the length itself, because what follows it in a card is the next thing
/// to do rather than the answer.
pub fn the_timeline(answer: &str) -> String {
    if pt::strip(answer).is_empty() {
        return String::new();
    }
    for sentence in SENTENCE.split(answer) {
        let Some(m) = _TIMELINE.search(&sentence) else {
            continue;
        };
        let end = m.name_span("span").map_or(m.end(), |span| span.1);
        let clause = pt::rstrip_chars(&sentence[..end], " ,;:-");
        if clause.is_empty() {
            continue;
        }
        return pt::upper_first(clause) + ".";
    }
    String::new()
}

/// Python: bot/memory/healing.py:153 `_the_same_length(reply, timeline)`
///
/// Whether a reply that is only a length says the length the timeline says.
pub fn _the_same_length(reply: &str, timeline: &str) -> bool {
    let Some(said) = _ONLY_A_SPAN.match_(reply) else {
        return false;
    };
    if timeline.is_empty() {
        return false;
    }
    pt::lower(timeline).contains(&pt::lower(pt::strip_chars(said.as_str(), " .!")))
}

/// Python: bot/memory/healing.py:176 `_stem(word)`
///
/// A word with its ending taken off, so "sprained" and "sprain" count as the same thing.
pub fn _stem(word: &str) -> String {
    for ending in ["ing", "ed", "es", "s"] {
        if word.ends_with(ending) && pt::len(word) - ending.len() >= 4 {
            return word[..word.len() - ending.len()].to_string();
        }
    }
    word.to_string()
}

/// Python: bot/memory/healing.py:188 `_carries(text)`
///
/// The words of a line that say what it is about, without the words of the ask.
pub fn _carries(text: &str) -> HashSet<String> {
    WORD.findall_str(&pt::lower(text))
        .iter()
        .filter(|word| pt::len(word) > 2 && !_ASK_WORDS.contains(*word))
        .map(|word| _stem(word))
        .collect()
}

/// Python: bot/memory/healing.py:194 `_said_before(memory, keep=2)`
///
/// The user's own lines before this turn, newest first.
pub fn _said_before(memory: &Memory, keep: i64) -> Vec<String> {
    let mut lines = Vec::new();
    for turn in memory.turns.iter().rev() {
        if !turn.user.is_empty() {
            lines.push(turn.user.clone());
            if lines.len() as i64 >= keep {
                break;
            }
        }
    }
    lines
}

/// Python: bot/memory/healing.py:205 `derived(memory)`
///
/// Whether the reply on the screen was read off the answer above it rather than out of an
/// entry. The mark is the turn the line was spoken on, so it cannot outlive that turn.
pub fn derived(memory: &Memory) -> bool {
    // `isinstance(at, int)`: a bool is an int in Python.
    let at = match memory.get("healing_at") {
        Some(Value::Int(at)) => *at,
        Some(Value::Bool(at)) => *at as i64,
        _ => return false,
    };
    at == memory.turns.len() as i64 - 1
}

/// Python: bot/memory/healing.py:222 `_no_timeline(memory, step='')`
///
/// The honest line, a different one each time the same ask comes back.
pub fn _no_timeline(memory: &Memory, step: &str) -> String {
    let turns = memory.turns.len();
    let step = pt::strip(step);
    if !step.is_empty() {
        let step = pt::upper_first(step);
        return NO_TIME_THEN_STEP[turns % NO_TIME_THEN_STEP.len()].replace("{step}", &step);
    }
    NO_TIMELINE[turns % NO_TIMELINE.len()].clone()
}

/// Python: bot/memory/healing.py:232 `_a_hedge(reply)`
///
/// Whether the reply already says the bot has no figure for the ask.
pub fn _a_hedge(reply: &str) -> bool {
    HEDGE.is_match(reply)
}

/// Python: bot/memory/healing.py:237 `_the_card(memory)`
///
/// The answer on the screen: the bot's last line, or the card behind a healing line.
pub fn _the_card(memory: &Memory) -> String {
    let said: Vec<&str> = memory.turns.iter().map(|turn| turn.bot.as_str()).filter(|bot| !bot.is_empty()).collect();
    if derived(memory) && said.len() > 1 {
        return said[said.len() - 2].to_string();
    }
    said.last().map_or(String::new(), |bot| bot.to_string())
}

/// `str(step_of(card) or "").strip()`, or "" when the function raises: a step is a nicety, and
/// the line stands without it. `step_of` is a function by its Python id; `answers._first_step`
/// is the only one PyBot passes. A panic that marks an unported function is passed on, so the
/// checkers still count it.
fn the_step(step_of: &Value, card: &str) -> String {
    let Some(function) = step_of.as_fn() else {
        return String::new(); // `callable(step_of)` is false for None and for any plain value
    };
    let run: fn(&str) -> String = match function.name() {
        "bot.modules.answers:_first_step" => crate::modules::answers::_first_step,
        other => panic!("healing.vetted: no port of the step_of function {other}"),
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(card))) {
        Ok(step) => pt::strip(&step).to_string(),
        Err(panic) => {
            let message = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|text| text.to_string()))
                .unwrap_or_default();
            if message.contains("UNPORTED") {
                std::panic::resume_unwind(panic);
            }
            String::new()
        }
    }
}

/// Python: bot/memory/healing.py:252 `vetted(text, memory, reply, step_of=None)`
///
/// The reply to a healing ask, or the reply it had. A healing ask is answered from the card
/// the person was just given: the timeline it states, or the plain statement that it states
/// none.
pub fn vetted(text: &str, memory: &mut Memory, reply: Option<&str>, step_of: &Value) -> Option<String> {
    let unchanged = || reply.map(str::to_string);
    if !asks(text) {
        return unchanged();
    }
    let card = _the_card(memory);
    if pt::strip(&card).is_empty() || !CARE.is_match(&card) {
        return unchanged(); // the last thing said was not care, so this is not a body being looked after
    }
    let timeline = the_timeline(&card);
    let said = reply.unwrap_or("");
    if pt::strip(said).is_empty() && timeline.is_empty() {
        // This module found no answer of its own and the card states no span either: the turn is
        // the composer's hedge, which is honest already.
        return unchanged();
    }
    let step = the_step(step_of, &card);
    let vetted_reply = _the_healing_line(said, &timeline, &card, memory, &step);
    if vetted_reply != said {
        // The turn this line is spoken on, so a later "and what should i do" reads the card and
        // not this: there are no steps in a timeline.
        memory.set("healing_at", memory.turns.len() as i64);
    }
    Some(vetted_reply)
}

/// Python: bot/memory/healing.py:294 `_the_same_as_the_card(said, card)`
///
/// Whether the reply is the card the person already has, said again.
pub fn _the_same_as_the_card(said: &str, card: &str) -> bool {
    let said = pt::lower(pt::strip(said));
    let card = pt::lower(pt::strip(card));
    if pt::len(&said) < 20 || pt::len(&card) < 40 {
        return false;
    }
    // The whole of one is the whole of the other, possibly behind a repeat frame ("Same as
    // before: Most shoulder pain...") or cut to the part the line asks for.
    let body = _REPEAT_LEAD.sub("", &said);
    card.contains(&body) || body.contains(&card)
}

/// Python: bot/memory/healing.py:314 `_the_healing_line(said, timeline, card, memory, step='')`
pub fn _the_healing_line(said: &str, timeline: &str, card: &str, memory: &Memory, step: &str) -> String {
    let the_timeline_or_none = || if timeline.is_empty() { _no_timeline(memory, step) } else { timeline.to_string() };
    if pt::strip(said).is_empty() {
        return the_timeline_or_none();
    }
    if _the_same_as_the_card(said, card) {
        // The card again, with no timeline in it: the honest line is the answer.
        return the_timeline_or_none();
    }
    if _a_hedge(said) {
        // Already honest, so only a card that states the span improves on it.
        return if timeline.is_empty() { said.to_string() } else { timeline.to_string() };
    }
    if _ONLY_A_SPAN.match_(said).is_some() {
        return if _the_same_length(said, timeline) { said.to_string() } else { the_timeline_or_none() };
    }
    if pt::split_ref(said).len() > 10 {
        // A paragraph, and a short answer that is a whole sentence: what matters is whether it
        // is about the thing the ask leans on. The subject is what the last lines of theirs
        // named, both of them.
        let before = _said_before(memory, 2);
        let subject = if before.is_empty() { HashSet::new() } else { _carries(&before.join(" ")) };
        let carried = _carries(said);
        if !subject.is_disjoint(&carried) {
            return said.to_string(); // a paragraph about the same body
        }
        if !timeline.is_empty() && !_carries(timeline).is_disjoint(&carried) {
            return said.to_string();
        }
        return the_timeline_or_none();
    }
    said.to_string()
}
