//! Port of `bot/compose/waited_for.py`: "how long will it be", "will it be ready by then" - a
//! wait whose date belongs to somebody else (a shop, a repairer, a booking, a reply, a sale) gets
//! where that date is held and how to pin it down.

use crate::data;
use crate::memory::holds as earlier;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.waited_for.", stringify!($name))));)*
    };
}
regexes!(A_WAIT_ASK, ORDER_WORDS, REPAIR_WORDS, BOOKING_WORDS, REPLY_WORDS, OFFER_WORDS, NOT_OURS);
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.waited_for.", stringify!($name))));)*
    };
}
pools!(
    NOTHING_HELD, WAITING_FOR_AN_ORDER, WAITING_FOR_A_REPAIR, WAITING_FOR_A_REPLY,
    WAITING_FOR_A_BOOKING, WAITING_FOR_AN_OFFER, WAITING_A_DECISION,
);
/// The `re.search` pattern written inline in `_the_kind`.
static A_DECISION: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(?:should\s+(?:i|we)\s+wait|is\s+it\s+worth\s+waiting|wait\s+(?:for|till|until)\s+the\s+(?:sales?|payday|christmas|black\s+friday))\b",
        0,
    )
});

/// Python: bot/compose/waited_for.py:183 `asks_for_a_date(clean, repaired=None)`
///
/// True when the line asks how long a wait is, or whether a date holds.
pub fn asks_for_a_date(clean: &str, repaired: &str) -> bool {
    [clean, repaired].iter().any(|line| !line.is_empty() && A_WAIT_ASK.is_match(line))
}

/// Python: bot/compose/waited_for.py:191 `_the_thread(clean, memory)`
///
/// What the person has said lately, this line included, as one lowercase string.
pub fn _the_thread(clean: &str, memory: &Memory) -> String {
    let lines = earlier::lines(memory);
    let mut said = vec![clean.to_string()];
    said.extend(lines[lines.len().saturating_sub(4)..].iter().cloned());
    text::lower(&said.join(" "))
}

/// Python: bot/compose/waited_for.py:202 `_the_kind(clean, thread)`
///
/// "decision", "repair", "order", "booking", "reply", "offer", or None for what is being waited on.
pub fn _the_kind(clean: &str, thread: &str) -> Option<String> {
    if A_DECISION.is_match(clean) {
        return Some("decision".into());
    }
    let kinds: [(&str, &Re); 5] = [
        ("repair", &REPAIR_WORDS),
        ("order", &ORDER_WORDS),
        ("booking", &BOOKING_WORDS),
        ("reply", &REPLY_WORDS),
        ("offer", &OFFER_WORDS),
    ];
    kinds.iter().find(|(_, words)| words.is_match(thread)).map(|(kind, _)| kind.to_string())
}

/// Python: bot/compose/waited_for.py:215 `for_the_ask(clean, repaired, memory)`
///
/// The pool of lines for this wait, or None when the line asks no date or the thread is a body,
/// a death or a pet.
pub fn for_the_ask(clean: &str, repaired: &str, memory: &Memory) -> Option<Vec<String>> {
    if !asks_for_a_date(clean, repaired) {
        return None;
    }
    let thread = _the_thread(clean, memory);
    if NOT_OURS.is_match(&thread) {
        return None;
    }
    let pool = match _the_kind(clean, &thread).as_deref() {
        Some("decision") => &*WAITING_A_DECISION,
        Some("repair") => &*WAITING_FOR_A_REPAIR,
        Some("order") => &*WAITING_FOR_AN_ORDER,
        Some("booking") => &*WAITING_FOR_A_BOOKING,
        Some("reply") => &*WAITING_FOR_A_REPLY,
        Some("offer") => &*WAITING_FOR_AN_OFFER,
        _ => &*NOTHING_HELD,
    };
    Some(pool.clone())
}
