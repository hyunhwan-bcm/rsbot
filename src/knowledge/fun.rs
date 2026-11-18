//! Port of `bot/knowledge/fun.py`: something to hear (a fact, a riddle), a fact about a subject,
//! a game, a name for a thing, a toast, a quiz question. Pools are chosen with `compose.pick`,
//! seeded with the line.

use crate::data::FnRef;
use crate::knowledge::d2_shared::pick_in;
use crate::knowledge::d2dup_g10::{dying_in_the_chat, globals, re_cached, reads_as_loss};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self, Re};
use crate::state::Memory;
use crate::understood::Understood;
use std::sync::LazyLock;

globals!("bot.knowledge.fun",
    set { _ASK_WORDS },
    strs { RIDDLES, FUN_FACTS, _COUNTED, _FACT_ASKS, _FACT_NOT, THING_NAMED, GROUP_GAMES, BOAT_NAMES, BAND_NAMES,
           PODCAST_NAMES, SHOP_NAMES, TECH_NAMES, PLAIN_TOASTS },
    value { FACTS_ABOUT, TOAST_OCCASIONS });

static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z']+", 0));

fn clean_or_text_lower(understood: &Understood) -> String {
    let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
    line.to_lowercase()
}

/// True when the line asks to be told something and names nothing itself.
pub fn _the_line_is_only_the_ask(understood: &Understood) -> bool {
    LETTERS.findall_str(&clean_or_text_lower(understood)).iter().all(|w| _ASK_WORDS.contains(w))
}

/// A fact or a riddle for a line that asked for something to hear, and nothing else; nothing
/// for a chat where somebody has just died.
pub fn _something_to_hear(memory: &mut Memory, reading: &Understood) -> Option<String> {
    if dying_in_the_chat(memory) {
        return None;
    }
    let turns = &memory.turns;
    for earlier in &turns[turns.len().saturating_sub(2)..] {
        if !earlier.user.is_empty() && reads_as_loss(&earlier.user) {
            return None;
        }
    }
    let text = clean_or_text_lower(reading);
    if text.contains("entertain") || text.contains("amuse") {
        return Some(pick_in(&RIDDLES, memory, &text));
    }
    Some(pick_in(&FUN_FACTS, memory, &text))
}

/// True when `word` is said in `clean` as a word, not inside a longer one.
pub fn _said_here(clean: &str, word: &str) -> bool {
    re_cached(&format!("(?<![a-z]){}(?![a-z])", re::escape(word)), 0).is_match(clean)
}

/// False when the line counts something out: that number has its own row.
pub fn _the_line_does_not_state_the_fact(reading: &Understood) -> bool {
    let clean = format!(" {} ", clean_or_text_lower(reading));
    !_COUNTED.iter().any(|w| _said_here(&clean, w))
}

/// A fact about the subject the line names, or None when it names none this table holds.
pub fn _a_fact_about(memory: &mut Memory, reading: &Understood) -> Option<String> {
    let clean = format!(" {} ", clean_or_text_lower(reading));
    if !_FACT_ASKS.iter().any(|a| _said_here(&clean, a)) {
        return None;
    }
    if _FACT_NOT.iter().any(|w| _said_here(&clean, w)) {
        return None;
    }
    for (_subject, row) in FACTS_ABOUT.pairs() {
        let (words, facts) = (row.items()[0].strs(), row.items()[1].strs());
        if words.iter().any(|w| _said_here(&clean, w)) {
            return Some(pick_in(&facts, memory, &clean));
        }
    }
    None
}

/// The thing of the ones above that the line names, longest phrase first.
pub fn _the_thing_being_named(understood: &Understood) -> String {
    let text = format!(" {} ", understood.clean).replace("'s", "");
    let mut best = String::new();
    for one in THING_NAMED.iter() {
        if one.chars().count() > best.chars().count() && text.contains(&format!(" {one} ")) {
            best = one.clone();
        }
    }
    best
}

/// The game the line names, or one of the pool when it names none.
pub fn _the_game_the_line_names(memory: &mut Memory, reading: &Understood) -> Option<String> {
    let text = format!(" {} ", reading.clean);
    let games: [(&[&str], usize); 4] = [
        (&["two truths", "2 truths", "truths and a lie", "truth or lie"], 0),
        (&["would you rather"], 1),
        (&["twenty questions", "20 questions"], 2),
        (&["alphabet"], 3),
    ];
    for (marks, at) in games {
        if marks.iter().any(|m| text.contains(m)) {
            return Some(GROUP_GAMES[at].clone());
        }
    }
    // the alphabet game needs a journey to play
    let journey = [" car", " journey", " travel", " flight", " drive", " road", " trip", " train", " plane", " airport"];
    let pool: &[String] = if journey.iter().any(|w| text.contains(w)) { &GROUP_GAMES } else { &GROUP_GAMES[..3] };
    Some(pick_in(pool, memory, &text))
}

/// Three names and the rule they follow, for the thing the line names.
pub fn _a_name_for_the_thing(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    let thing = _the_thing_being_named(reading);
    let names: &[String] = match thing.as_str() {
        "boat" | "boats" | "yacht" | "kayak" | "canoe" => &BOAT_NAMES,
        "band" | "bands" | "quiz team" | "team" => &BAND_NAMES,
        "podcast" => &PODCAST_NAMES,
        "pub" | "cafe" | "café" | "shop" | "business" | "company" | "startup" => &SHOP_NAMES,
        _ => &TECH_NAMES,
    };
    Some(format!(
        "Short, sayable and easy to spell beats clever: {}, {} or {}. Say it out loud in a sentence - 'this is {}' - before you commit, and check nobody selling the same thing already has it.",
        names[0],
        names[1],
        names[2],
        names[0].to_lowercase()
    ))
}

/// `_pool.<locals>.choose`: one of the captured options, seeded with the line.
fn pool_choose(f: &FnRef, memory: &mut Memory, reading: &Understood) -> Option<String> {
    let text = if reading.text.is_empty() { &reading.clean } else { &reading.text };
    let options = f.cell("options").map(|v| v.strs()).unwrap_or_default();
    Some(pick_in(&options, memory, text))
}

/// The toast for the occasion the line names, or a plain one when it names none.
pub fn _toast(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let joined = understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" ");
    for row in TOAST_OCCASIONS.items() {
        let (occasion_words, pool) = (row.items()[1].strs(), row.items()[2].strs());
        if occasion_words.iter().any(|w| joined.contains(w.as_str())) {
            return Some(pick_in(&pool, memory, &joined));
        }
    }
    Some(pick_in(&PLAIN_TOASTS, memory, &joined))
}

/// The row answers only when the line asks for a question to answer (`wants_a_quiz`).
pub fn _the_person_is_the_one_quizzed(understood: &Understood) -> bool {
    understood.wants_a_quiz
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.fun:_the_line_is_only_the_ask:71", |_: &FnRef, u: &Understood| _the_line_is_only_the_ask(u)),
    ("bot.knowledge.fun:_the_line_does_not_state_the_fact:445", |_: &FnRef, u: &Understood| {
        _the_line_does_not_state_the_fact(u)
    }),
    ("bot.knowledge.fun:_the_person_is_the_one_quizzed:717", |_: &FnRef, u: &Understood| _the_person_is_the_one_quizzed(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.fun:_something_to_hear:82", |_: &FnRef, m: &mut Memory, u: &Understood| _something_to_hear(m, u)),
    ("bot.knowledge.fun:_a_fact_about:452", |_: &FnRef, m: &mut Memory, u: &Understood| _a_fact_about(m, u)),
    ("bot.knowledge.fun:_the_game_the_line_names:569", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _the_game_the_line_names(m, u)
    }),
    ("bot.knowledge.fun:_a_name_for_the_thing:594", |_: &FnRef, m: &mut Memory, u: &Understood| _a_name_for_the_thing(m, u)),
    ("bot.knowledge.fun:_pool.<locals>.choose:621", pool_choose),
    ("bot.knowledge.fun:_toast:691", |_: &FnRef, m: &mut Memory, u: &Understood| _toast(m, u)),
];
