//! Port of `bot/compose/habit.py`: a recurring thing somebody does ("my brother keeps borrowing
//! my car and never asks") answered with the words to raise it, and the second turn of the same
//! complaint.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, bundle_vals, inline_re};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

bundle_res!("bot.compose.habit"; A_PERSON, SOMEONE, RECURS, DONE_TO_ME, ASK_FOR, NOT_ASKED, UNSEEN, NOT_HELPED, MY_THING,
    MESS, CRITICISED, CRITICISED_MY, BAILS, KEEPS_THE_ARRANGEMENT, ALWAYS_DOES_IT, THANKED, COURTESY, COURTESY_DENIED,
    NOT_SINCERE, A_SHARED_OCCASION, ARRIVES_LATE, GOING_ON, EMAIL_HABIT, SAYS_ABOUT_ME, CREDIT);
bundle_sets!("bot.compose.habit"; PEOPLE_SET, WORK_PEOPLE, NOT_THIS, NOT_A_GRUMBLE, CHILD_PEOPLE, GOOD_WORDS);
bundle_strs!("bot.compose.habit"; STRAIN, WORK_WORDS, ASKED, TAKEN, MESSY, LEFT_OUT, TAKEN_FOR_GRANTED, CRITICISED_LINES,
    WORK_LINES, CREDIT_LINES, GOOD_HABIT_APPRECIATED, GOOD_HABIT_INCLUDED, WORDS_NOT_MEANT, RAISE, BAILING, LATE_LINES,
    SUBJECT_AGAIN, AFTER_A_SAYING, THE_SAYING, ALWAYS_COUNT, SELF_DOUBT, ABOUT_ME, SAID_IT_POOL, EVERY_TIME_POOL,
    DOUBT_POOL, HOLD_THE_LINE);
bundle_vals!("bot.compose.habit"; MY_POOLS, THREAD_TURNS);
inline_re!(ABOUT_THE_BOT, r"\b(?:you|your|the bot|this bot)\b");
inline_re!(ASKS_FOR_THE_WORDS, r"\b(?:what|how)\b[^?]{0,30}\b(?:say|tell|word|phrase|put it|bring it up)\b");
inline_re!(A_PROMISE, concat!(r"\b(?:says? (?:she|he|they)'?ll|said (?:she|he|they)'?d|promised|",
    r"says? (?:she|he|they) (?:will|would)|doesn'?t|didn'?t|won'?t)\b"));
inline_re!(A_PRONOUN, r"\b(?:he|she|they|him|her|them)\b");
inline_re!(LEAVES_IT, concat!(r"\b(?:leaves?|left|leaving|drops?|dropped|", r"dumping|dumps|put|puts|putting)\b"));
inline_re!(A_WORD, r"[a-z']+");
inline_re!(ASKS_HOW_TO_SAY_IT, concat!(r"\b(?:what|how)\b[^?]{0,24}\b(?:say|tell|word|", r"phrase|put it|raise it)\b"));

/// The first 30 characters of every line in this file's pools.
static MY_OPENINGS: LazyLock<Vec<String>> = LazyLock::new(|| {
    MY_POOLS.items().iter().flat_map(|pool| pool.strs()).map(|reply| text::slice(&reply, 0, Some(30))).collect()
});

fn any_phrase(clean: &str, phrases: &[String]) -> bool {
    phrases.iter().any(|p| clean.contains(p.as_str()))
}

/// Python: bot/compose/habit.py:511 `_in_my_pools(line)`
///
/// True when a bot line is one of this file's own, so the thread is still open.
pub fn _in_my_pools(line: &str) -> bool {
    let said = text::squash(line);
    !said.is_empty() && MY_OPENINGS.iter().any(|opening| said.starts_with(opening.as_str()))
}

/// Python: bot/compose/habit.py:519 `_thread_open(memory)`
///
/// True when the bot's last line in this conversation came out of one of these pools.
pub fn _thread_open(memory: &Memory) -> bool {
    let turns = &memory.turns;
    if turns.is_empty() {
        return false;
    }
    let n = THREAD_TURNS.as_int().expect("THREAD_TURNS is an int");
    for turn in crate::compose::com4::tail(turns, n).iter().rev() {
        if turn.bot.is_empty() {
            continue;
        }
        return _in_my_pools(&turn.bot);
    }
    false
}

/// Python: bot/compose/habit.py:534 `holds_the_thread(memory)`
///
/// True when the bot's last line was one of this file's, so a short reply is still this line.
pub fn holds_the_thread(memory: &Memory) -> bool {
    _thread_open(memory)
}

/// Python: bot/compose/habit.py:545 `_follow_up(clean, memory)`
///
/// The second turn of the same complaint, or None when it is not one.
pub fn _follow_up(clean: &str, memory: &Memory) -> Option<&'static Vec<String>> {
    if !_thread_open(memory) {
        return None;
    }
    let n = text::split_ref(clean).len();
    if ABOUT_THE_BOT.search(clean).is_some() {
        return None;
    }
    if any_phrase(clean, &SELF_DOUBT) {
        return Some(&DOUBT_POOL);
    }
    if any_phrase(clean, &ALWAYS_COUNT) && n <= 12 {
        return Some(&EVERY_TIME_POOL);
    }
    // "what do i say to him" is a request for the sentence; the composer's drafter offers it.
    if ASKS_FOR_THE_WORDS.search(clean).is_some() {
        return None;
    }
    let tried = any_phrase(clean, &THE_SAYING);
    let outcome = any_phrase(clean, &AFTER_A_SAYING);
    if (tried || outcome) && n <= 20 {
        return Some(&SAID_IT_POOL);
    }
    // A promise that was not kept is the same talk as a line already said.
    if A_PROMISE.search(clean).is_some() && n <= 20 {
        return Some(&SAID_IT_POOL);
    }
    // The line still names the person and says nothing else: hold the line.
    if A_PRONOUN.search(clean).is_some() && n <= 12 && !clean.contains('?') {
        return Some(&HOLD_THE_LINE);
    }
    None
}

/// Python: bot/compose/habit.py:590 `_the_one(word)`
///
/// The person with a plural taken off, so "my kids" is read as a kid and a boss as a boss.
pub fn _the_one(word: &str) -> String {
    let known = |w: &str| CHILD_PEOPLE.contains(w) || WORK_PEOPLE.contains(w) || PEOPLE_SET.contains(w);
    if known(word) {
        return word.to_string();
    }
    for cut in [text::slice(word, 0, Some(-1)), text::slice(word, 0, Some(-2))] {
        if known(&cut) {
            return cut;
        }
    }
    word.to_string()
}

/// Python: bot/compose/habit.py:604 `_person_named(clean)`
///
/// The person the line is about, as they were named, or "".
pub fn _person_named(clean: &str) -> String {
    A_PERSON.search(clean).map(|m| m.as_str().to_string()).unwrap_or_default()
}

fn last_person(person: &str) -> String {
    text::split_ref(person).last().map(|w| _the_one(w)).unwrap_or_default()
}

/// Python: bot/compose/habit.py:615 `_shape(clean, person)`
///
/// Which pool answers this line, or None when it is not this file's turn. Work first, then
/// being criticised, then the three ways of not being asked, then the thing in the line itself,
/// and the general raising-it pool last.
pub fn _shape(clean: &str, person: &str) -> Option<&'static Vec<String>> {
    let last = last_person(person);
    let has = |rx: &crate::py::re::Re| rx.search(clean).is_some();
    if has(&NOT_SINCERE) {
        // the words are there and what is behind them is not
        return Some(&WORDS_NOT_MEANT);
    }
    let good_habit = || -> &'static Vec<String> {
        if has(&THANKED) { &GOOD_HABIT_APPRECIATED } else { &GOOD_HABIT_INCLUDED }
    };
    if has(&ALWAYS_DOES_IT) && !has(&BAILS) && (has(&A_SHARED_OCCASION) || !has(&ASK_FOR)) {
        // good news, read first so a boss who always thanks you is not sent to the work pool
        return Some(good_habit());
    }
    if has(&COURTESY)
        && !has(&COURTESY_DENIED)
        && !(has(&BAILS) || _strained(clean) || has(&CRITICISED) || has(&NOT_ASKED) || has(&UNSEEN))
    {
        // A courtesy with nothing wrong in the line is good news.
        return Some(good_habit());
    }
    if WORK_PEOPLE.contains(&last) && (any_phrase(clean, &WORK_WORDS) || has(&DONE_TO_ME)) {
        return Some(&WORK_LINES);
    }
    if has(&SAYS_ABOUT_ME) {
        // the complaint is aimed at the person, not held by them
        return Some(&ABOUT_ME);
    }
    if has(&CRITICISED) && (has(&DONE_TO_ME) || has(&CRITICISED_MY) || has(&SAYS_ABOUT_ME) || _strained(clean)) {
        return Some(&CRITICISED_LINES);
    }
    if has(&CREDIT) {
        return Some(&CREDIT_LINES);
    }
    if has(&NOT_ASKED) {
        return Some(&LEFT_OUT);
    }
    if has(&UNSEEN) {
        return Some(&TAKEN_FOR_GRANTED);
    }
    if has(&ASK_FOR) {
        return Some(&ASKED);
    }
    if has(&MESS) && has(&LEAVES_IT) {
        return Some(&MESSY);
    }
    if has(&MY_THING) {
        return Some(&TAKEN);
    }
    if has(&NOT_HELPED) {
        return Some(&RAISE);
    }
    if has(&BAILS) && !has(&KEEPS_THE_ARRANGEMENT) {
        return Some(&BAILING);
    }
    if has(&ARRIVES_LATE) && !has(&KEEPS_THE_ARRANGEMENT) {
        return Some(&LATE_LINES);
    }
    if has(&GOING_ON) && !CHILD_PEOPLE.contains(&last) && !has(&KEEPS_THE_ARRANGEMENT) {
        return Some(&SUBJECT_AGAIN);
    }
    // A mailbox habit is a work complaint: a neighbour does not reply-all to anything.
    if has(&EMAIL_HABIT) && WORK_PEOPLE.contains(&last) {
        return Some(&RAISE);
    }
    if has(&DONE_TO_ME) {
        return Some(&CRITICISED_LINES);
    }
    if _strained(clean) {
        return Some(&RAISE);
    }
    None
}

/// Python: bot/compose/habit.py:683 `_said_at_work(clean, person)`
///
/// True when a line about the boss or a colleague names a thing that changes at work.
pub fn _said_at_work(clean: &str, person: &str) -> bool {
    WORK_PEOPLE.contains(&last_person(person)) && any_phrase(clean, &WORK_WORDS)
}

/// Python: bot/compose/habit.py:689 `_strained(clean)`
pub fn _strained(clean: &str) -> bool {
    any_phrase(clean, &STRAIN)
}

/// Python: bot/compose/habit.py:693 `answer_for(text, memory)`
///
/// The words to raise it with somebody, or None when this is not that shape.
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text_.to_lowercase());
    if clean.is_empty() || text::split_ref(&clean).len() > 30 {
        return None;
    }
    let words: HashSet<String> = A_WORD.findall_str(&clean).into_iter().collect();
    if words.iter().any(|w| NOT_THIS.contains(w)) {
        return None;
    }
    if let Some(follow) = _follow_up(&clean, memory).filter(|p| !p.is_empty()) {
        return Some(pick(follow, memory, &clean));
    }
    // Something that recurs is what this file answers - with one exception, a courtesy nobody
    // means.
    let has = |rx: &crate::py::re::Re| rx.search(&clean).is_some();
    if !has(&RECURS) && !has(&NOT_SINCERE) {
        return None;
    }
    let person = match _person_named(&clean) {
        p if !p.is_empty() => p,
        _ => SOMEONE.search(&clean).map(|m| m.as_str().to_string()).unwrap_or_default(),
    };
    if person.is_empty() {
        return None;
    }
    if words.iter().any(|w| NOT_A_GRUMBLE.contains(w) || GOOD_WORDS.contains(w)) {
        return None;
    }
    // A question is somebody else's turn unless it asks for the words.
    if clean.ends_with('?') && !has(&ASKS_HOW_TO_SAY_IT) {
        return None;
    }
    if !(has(&DONE_TO_ME)
        || has(&MY_THING)
        || has(&MESS)
        || has(&NOT_ASKED)
        || has(&UNSEEN)
        || has(&NOT_HELPED)
        || has(&CRITICISED_MY)
        || has(&SAYS_ABOUT_ME)
        || has(&CREDIT)
        || _strained(&clean)
        || has(&NOT_SINCERE)
        || has(&BAILS)
        || has(&ARRIVES_LATE)
        || has(&GOING_ON)
        || has(&EMAIL_HABIT)
        || has(&COURTESY)
        || _said_at_work(&clean, &person)
        || has(&ALWAYS_DOES_IT))
    {
        return None;
    }
    let pool = _shape(&clean, &person).filter(|p| !p.is_empty())?;
    Some(pick(pool, memory, &clean))
}
