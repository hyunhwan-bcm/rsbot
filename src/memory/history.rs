//! Port of `bot/memory/history.py`: questions about the conversation as a whole ("what have we
//! talked about", "what was the first thing I said"), and the one check for "is there an
//! earlier turn?" that every reply leaning on the chat goes through. Nothing here replies.

use super::pyv;
use super::topics;
use crate::data;
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.history.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}

static TALKED_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| rx("TALKED_ABOUT"));
static FIRST_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("FIRST_THING"));
static FIRST_SAID_BY_BOT: LazyLock<&'static Re> = LazyLock::new(|| rx("FIRST_SAID_BY_BOT"));
static RECALL_ABOUT: LazyLock<&'static Re> = LazyLock::new(|| rx("RECALL_ABOUT"));
static SAID_ABOUT_HEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("SAID_ABOUT_HEAD"));
static CHAT_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| rx("CHAT_QUESTION"));
static REMIND_ME_RECAP: LazyLock<&'static Re> = LazyLock::new(|| rx("REMIND_ME_RECAP"));
static WHERE_WE_WERE: LazyLock<&'static Re> = LazyLock::new(|| rx("WHERE_WE_WERE"));
static STORE_SUMMARY: LazyLock<&'static Re> = LazyLock::new(|| rx("STORE_SUMMARY"));
static RECALL_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| rx("RECALL_SHAPE"));
static MY_SAID_ABOUT_HEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("MY_SAID_ABOUT_HEAD"));
static NOTHING_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("NOTHING_SAID"));
static GREETING_WITH_AN_ADDRESS: LazyLock<&'static Re> = LazyLock::new(|| rx("GREETING_WITH_AN_ADDRESS"));
static CLAIMS_A_PAST: LazyLock<&'static Re> = LazyLock::new(|| rx("CLAIMS_A_PAST"));
static GREETS_A_RETURN: LazyLock<&'static Re> = LazyLock::new(|| rx("GREETS_A_RETURN"));
static ANSWERS_AN_EARLIER_TURN: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.memory.history.ANSWERS_AN_EARLIER_TURN"));
static NO_CONTENT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.history.NO_CONTENT_WORDS"));
static A_THEY_ASKED: LazyLock<&'static Re> = LazyLock::new(|| rx("A_THEY_ASKED"));
/// How far back a follow-up may lean.
pub static LOOK_BACK: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.history.LOOK_BACK").as_int().unwrap() as usize);
static THEIR_NEWS: LazyLock<&'static Re> = LazyLock::new(|| rx("THEIR_NEWS"));
static A_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| rx("A_QUESTION"));
static A_WANT: LazyLock<&'static Re> = LazyLock::new(|| rx("A_WANT"));
static A_BARE_YES: LazyLock<&'static Re> = LazyLock::new(|| rx("A_BARE_YES"));
static PUNT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.PUNT"));

static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", re::U));
static WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", re::U));

const POINTERS: [&str; 6] = ["it", "that", "this", "them", "those", "these"];

/// Python: bot/memory/history.py:123 `_flat(text)`
/// "What have we talked about?" as plain words, punctuation stripped from the ends.
pub fn _flat(text_: &str) -> String {
    let low = text_.to_lowercase().replace('\u{2019}', "'");
    text::strip_chars(&text::squash(&low), " ?.!,").to_string()
}

/// Python: bot/memory/history.py:128 `the_thing_named(text)`
/// The thing a "what did you say about X" names, or "" when there is no such thing.
pub fn the_thing_named(text_: &str) -> String {
    let flat = _flat(text_);
    let low = text::strip_chars(&flat, " ?.!,");
    let Some(found) = SAID_ABOUT_HEAD.match_(low) else { return String::new() };
    let thing = text::strip_chars(found.name_str("thing"), " ?.!,");
    // "what did you say about it" names nothing: the ask is for the last answer again.
    if thing.is_empty() || POINTERS.contains(&thing) {
        return String::new();
    }
    thing.to_string()
}

/// Python: bot/memory/history.py:156 `the_thing_they_named(text)`
/// The thing a "what did i say about X" names, or "": the person's own side of the same ask.
pub fn the_thing_they_named(text_: &str) -> String {
    let flat = _flat(text_);
    let low = text::strip_chars(&flat, " ?.!,");
    let Some(found) = MY_SAID_ABOUT_HEAD.match_(low) else { return String::new() };
    let thing = SPACES.sub(" ", text::strip_chars(found.name_str("thing"), " ?.!,"));
    if thing.is_empty() || POINTERS.contains(&thing.as_str()) {
        return String::new();
    }
    thing
}

/// Python: bot/memory/history.py:174 `asked(text, memory=None)`
/// Which conversation question this is: "said_about", "first_bot", "first", "topics" or None.
/// (`memory` is not read.)
pub fn asked(text_: &str, _memory: Option<&Memory>) -> Option<String> {
    let low = _flat(text_);
    if WHERE_WE_WERE.is_match(&low) || STORE_SUMMARY.is_match(&low) {
        return None;
    }
    if REMIND_ME_RECAP.is_match(&low) {
        return None;
    }
    if RECALL_ABOUT.is_match(&low) {
        // The store's own recall reader answers a line that names the thing it wants back.
        return None;
    }
    let kind = if !the_thing_named(&low).is_empty() {
        "said_about"
    } else if FIRST_SAID_BY_BOT.is_match(&low) {
        "first_bot"
    } else if FIRST_THING.is_match(&low) {
        "first"
    } else if TALKED_ABOUT.is_match(&low) {
        "topics"
    } else {
        return None;
    };
    Some(kind.to_string())
}

/// Python: bot/memory/history.py:217 `_nothing_said(text)`
/// A greeting, a thank you or a request to be entertained: not a subject.
pub fn _nothing_said(text_: &str) -> bool {
    let low = _flat(text_);
    NOTHING_SAID.fullmatch(&low).is_some() || GREETING_WITH_AN_ADDRESS.fullmatch(&low).is_some()
}

/// Python: bot/memory/history.py:223 `about_the_chat(text)`
/// True for anything that asks about the conversation itself, answered or not.
pub fn about_the_chat(text_: &str) -> bool {
    let low = _flat(text_);
    asked(&low, None).is_some() || RECALL_ABOUT.is_match(&low) || CHAT_QUESTION.is_match(&low)
}

/// Python: bot/memory/history.py:233 `not_a_subject(text)`
/// Nothing we can say we have been on: a chat question, an ask about what they told you, a
/// greeting, or a joke request.
pub fn not_a_subject(text_: &str) -> bool {
    about_the_chat(text_) || _nothing_said(text_) || RECALL_SHAPE.is_match(&_flat(text_))
}

/// Python: bot/memory/history.py:312 `brings_nothing_of_its_own(text)`
/// True when the line has no subject of its own, only a point back at the conversation.
pub fn brings_nothing_of_its_own(text_: &str) -> bool {
    WORDS.findall_str(&text_.to_lowercase()).iter().all(|word| NO_CONTENT_WORDS.contains(word))
}

/// Python: bot/memory/history.py:320 `an_earlier_turn(memory, text='')`
/// Whether the conversation has a turn before this one. A turn carrying `text` is this line
/// and not an earlier one.
pub fn an_earlier_turn(memory: &Memory, text_: &str) -> bool {
    !_earlier_turns(Some(memory), text_).is_empty()
}

/// Python: bot/memory/history.py:332 `a_past_that_is_not_there(reply, memory, said=None)`
/// True when a reply leans on a conversation that has not happened yet: it points back at one
/// ("welcome back", "as I said"), or it answers one ("good to hear") when `said`, the person's
/// line, brings nothing of its own. `memory` None is anything that is not a memory dict (PyBot
/// passes a reading here at times): it has no turns.
pub fn a_past_that_is_not_there(reply: &str, memory: Option<&Memory>, said: Option<&str>) -> bool {
    if !_earlier_turns(memory, "").is_empty() {
        return false;
    }
    let low = text::squash(&reply.to_lowercase());
    if low.is_empty() {
        return false;
    }
    if CLAIMS_A_PAST.is_match(&low) || GREETS_A_RETURN.match_(&low).is_some() {
        return true;
    }
    let Some(said) = said else { return false };
    if !brings_nothing_of_its_own(said) {
        return false;
    }
    ANSWERS_AN_EARLIER_TURN.iter().any(|frame| low.contains(frame.as_str()))
}

/// Python: bot/memory/history.py:356 `first_bot_line(memory)`
/// The first thing the bot said in this conversation, or ''.
pub fn first_bot_line(memory: &Memory) -> String {
    memory.turns.iter().find(|turn| !text::strip(&turn.bot).is_empty()).map(|turn| text::squash(&turn.bot)).unwrap_or_default()
}

/// Python: bot/memory/history.py:366 `_earlier_turns(memory, text='')`
/// Every turn of theirs before this line, oldest first: a turn carrying this line's words is
/// this line, not an earlier one.
pub fn _earlier_turns<'a>(memory: Option<&'a Memory>, text_: &str) -> Vec<&'a Turn> {
    let mine = text::squash(text_).to_lowercase();
    let Some(memory) = memory else { return Vec::new() };
    memory
        .turns
        .iter()
        .filter(|row| {
            let line = text::squash(&row.user);
            !line.is_empty() && line.to_lowercase() != mine
        })
        .collect()
}

/// Python: bot/memory/history.py:392 `_a_line_they_asked(row)`
/// True when the turn is a question of theirs, not something they told the chat.
pub fn _a_line_they_asked(row: &Turn) -> bool {
    A_THEY_ASKED.is_match(&text::squash(&row.user))
}

/// Python: bot/memory/history.py:397 `_the_forms(words)`
/// The words and the verb forms of each: "walking" asks about a line that says "walk".
pub fn _the_forms(words: &[String]) -> HashSet<String> {
    words.iter().flat_map(|word| topics::forms(word)).collect()
}

/// Python: bot/memory/history.py:410 `the_earlier_turn(memory, text='', about='')`
/// The turn of theirs this line is about, or None when the chat holds no earlier one: the turn
/// that names the thing the ask names, else the newest one. Lines they told the chat come
/// before lines they asked.
pub fn the_earlier_turn<'a>(memory: &'a Memory, text_: &str, about: &str) -> Option<&'a Turn> {
    let mut earlier = _earlier_turns(Some(memory), text_);
    if earlier.is_empty() {
        return None;
    }
    let told: Vec<&Turn> = earlier.iter().copied().filter(|row| !_a_line_they_asked(row)).collect();
    if !told.is_empty() {
        earlier = told;
    }
    let mut thing = about.to_string();
    if thing.is_empty() {
        thing = the_thing_they_named(text_);
    }
    if thing.is_empty() {
        thing = the_thing_named(text_);
    }
    let wanted = if thing.is_empty() { HashSet::new() } else { _the_forms(&topics::content(&thing)) };
    if wanted.is_empty() {
        return earlier.last().copied();
    }
    let scored: Vec<(usize, &Turn)> =
        earlier.iter().map(|row| (wanted.intersection(&_the_forms(&topics::content(&row.user))).count(), *row)).collect();
    let best = scored.iter().map(|(hit, _)| *hit).max().unwrap_or(0);
    if best == 0 {
        return earlier.last().copied();
    }
    scored.into_iter().filter(|(hit, _)| *hit == best).map(|(_, row)| row).last()
}

/// Python: bot/memory/history.py:438 `first_line(memory)`
/// The first thing the user said in this conversation, or ''.
pub fn first_line(memory: &Memory) -> String {
    memory.turns.iter().find(|turn| !text::strip(&turn.user).is_empty()).map(|turn| text::squash(&turn.user)).unwrap_or_default()
}

/// Python: bot/memory/history.py:448 `topic_lines(memory, limit=3)`
/// The subjects the conversation has been on, oldest first, as the user put them, with the last
/// line joined on unless it was a question about the chat itself.
pub fn topic_lines(memory: &Memory, limit: usize) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    if let Some(records @ crate::data::Value::List(_)) = memory.get("topics") {
        for record in records.items() {
            if !record.is_dict() {
                continue;
            }
            let said = text::squash(&pyv::str_of(record.get("text").unwrap_or(&crate::data::Value::from(""))));
            if said.is_empty() || seen.contains(&said.to_lowercase()) || not_a_subject(&said) {
                continue;
            }
            seen.insert(said.to_lowercase());
            out.push(said);
        }
    }
    // The store only records a line that moves the topic on; the last line was still said.
    let last = memory
        .turns
        .iter()
        .rev()
        .filter(|turn| !text::strip(&turn.user).is_empty())
        .find(|turn| turn.module.as_deref() != Some("memory_store"))
        .map(|turn| text::squash(&turn.user))
        .unwrap_or_default();
    if !last.is_empty() && !seen.contains(&last.to_lowercase()) && !not_a_subject(&last) {
        out.push(last);
    }
    let start = if limit == 0 { 0 } else { out.len().saturating_sub(limit) };
    out.split_off(start)
}

/// Python: bot/memory/history.py:507 `turns_before(memory)`
/// Every turn the chat has had before this one, oldest first. Never raises.
pub fn turns_before(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// Python: bot/memory/history.py:517 `the_turn_before(memory)`
/// The newest turn before this one, or None when the chat has none.
pub fn the_turn_before(memory: &Memory) -> Option<&Turn> {
    turns_before(memory).last()
}

/// Python: bot/memory/history.py:527 `nothing_said_before(memory)`
/// Whether this line is the first thing they have said.
pub fn nothing_said_before(memory: &Memory) -> bool {
    turns_before(memory).is_empty()
}

/// Python: bot/memory/history.py:536 `the_lines_before(memory, look_back=LOOK_BACK)`
/// What they said in the last `look_back` turns, oldest first, empties dropped. (Python's
/// `turns[-look_back:]`: 0 is every turn.)
pub fn the_lines_before(memory: &Memory, look_back: usize) -> Vec<String> {
    let turns = turns_before(memory);
    let start = if look_back == 0 { 0 } else { turns.len().saturating_sub(look_back) };
    turns[start..].iter().map(|turn| text::strip(&turn.user).to_string()).filter(|line| !line.is_empty()).collect()
}

/// Python: bot/memory/history.py:542 `the_line_before(memory, look_back=LOOK_BACK)`
/// What they said in the newest earlier turn, or ''.
pub fn the_line_before(memory: &Memory, look_back: usize) -> String {
    the_lines_before(memory, look_back).pop().unwrap_or_default()
}

/// Python: bot/memory/history.py:548 `is_their_news(line, words=3)`
/// Whether a line is news of theirs, rather than a question, a want or a bare "yes".
pub fn is_their_news(line: &str, words: usize) -> bool {
    let said = text::squash(line);
    if text::split_ref(&said).len() < words {
        return false;
    }
    let low = said.to_lowercase();
    if said.ends_with('?') || A_QUESTION.match_(&low).is_some() || A_BARE_YES.match_(&low).is_some() {
        return false;
    }
    if A_WANT.is_match(&low) {
        return false;
    }
    THEIR_NEWS.is_match(&low)
}

/// Python: bot/memory/history.py:567 `the_news_turn_before(memory, words=3)`
/// What they said in the newest earlier turn that was their news, or ''.
pub fn the_news_turn_before(memory: &Memory, words: usize) -> String {
    the_lines_before(memory, *LOOK_BACK).into_iter().rev().find(|line| is_their_news(line, words)).unwrap_or_default()
}

/// Python: bot/memory/history.py:579 `the_earlier_line(memory, pattern, look_back=LOOK_BACK)`
/// The newest earlier line of theirs that matches `pattern`, or ''. (A pattern Python gets as
/// a string is compiled with re.I: `Re::new(pattern, re::I)`.)
pub fn the_earlier_line(memory: &Memory, pattern: &Re, look_back: usize) -> String {
    the_lines_before(memory, look_back).into_iter().rev().find(|line| pattern.is_match(line)).unwrap_or_default()
}

/// Python: bot/memory/history.py:594 `the_earlier_lines_that(memory, pattern, look_back=LOOK_BACK)`
/// Every earlier line of theirs that matches, newest last.
pub fn the_earlier_lines_that(memory: &Memory, pattern: &Re, look_back: usize) -> Vec<String> {
    the_lines_before(memory, look_back).into_iter().filter(|line| pattern.is_match(line)).collect()
}

/// Python: bot/memory/history.py:601 `only_with_an_earlier_turn(reader)`
/// A store reader that answers only when the chat has a turn before this one (Python's
/// decorator, applied at the call: `only_with_an_earlier_turn(memory, |m| reader(m, ..))`).
pub fn only_with_an_earlier_turn<T>(memory: &Memory, reader: impl FnOnce(&Memory) -> Option<T>) -> Option<T> {
    if nothing_said_before(memory) {
        return None;
    }
    reader(memory)
}

/// Python: bot/memory/history.py:631 `the_last_answer(memory)`
/// The bot's last line as it was written, notes and hedges included, or "".
pub fn the_last_answer(memory: &Memory) -> String {
    turns_before(memory).iter().rev().find(|turn| !text::strip(&turn.bot).is_empty()).map(|turn| text::squash(&turn.bot)).unwrap_or_default()
}

/// Python: bot/memory/history.py:644 `_a_note_not_an_answer(said)`
/// The composer's own turn is not an answer to come back to.
pub fn _a_note_not_an_answer(said: &str) -> bool {
    crate::compose::verdict::takes_a_fact_back(said) || crate::compose::fallback::is_said_it_already(said)
}

/// Python: bot/memory/history.py:650 `_a_hedge(said)`
pub fn _a_hedge(said: &str) -> bool {
    let flat = said.to_lowercase();
    crate::compose::fallback::_is_a_hedge(said) || PUNT.iter().any(|punt| flat.contains(punt.as_str()))
}

/// Python: bot/memory/history.py:657 `the_answer_on_screen(memory, within=3, modules=("answers", "safety"))`
/// The knowledge answer the person is looking at, or "": it steps back past the composer's
/// notes, and stops at the first real answer only when it came from one of `modules` and is
/// not a hedge.
pub fn the_answer_on_screen(memory: &Memory, within: i64, modules: &[String]) -> String {
    let mut seen = 0;
    for turn in turns_before(memory).iter().rev() {
        let said = text::strip(&turn.bot);
        if said.is_empty() {
            continue;
        }
        seen += 1;
        if seen > within {
            break;
        }
        if _a_note_not_an_answer(said) {
            continue;
        }
        if !modules.iter().any(|module| Some(module.as_str()) == turn.module.as_deref().or(Some(""))) {
            return String::new();
        }
        if _a_hedge(said) {
            return String::new();
        }
        return text::squash(said);
    }
    String::new()
}

fn answers_and_safety() -> Vec<String> {
    vec!["answers".to_string(), "safety".to_string()]
}

/// Python: bot/memory/history.py:683 `the_turn_the_answer_came_from(memory, within=3)`
/// The turn `the_answer_on_screen` read, or None.
pub fn the_turn_the_answer_came_from(memory: &Memory, within: i64) -> Option<&Turn> {
    let answer = the_answer_on_screen(memory, within, &answers_and_safety());
    if answer.is_empty() {
        return None;
    }
    turns_before(memory).iter().rev().find(|turn| text::squash(&turn.bot) == answer)
}

/// Python: bot/memory/history.py:694 `the_answer_is_a_card(memory, within=3)`
/// Whether the answer on the screen is a safety card rather than a knowledge entry.
pub fn the_answer_is_a_card(memory: &Memory, within: i64) -> bool {
    the_turn_the_answer_came_from(memory, within).is_some_and(|turn| turn.module.as_deref() == Some("safety"))
}
