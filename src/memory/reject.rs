//! Port of `bot/memory/reject.py`: they turned down the answer the bot just gave, and it owes them
//! another one.
//!
//! "whats a good film tonight" / "nah not that one": the whole line is the refusal, and the
//! answer offered things to choose between, so the reply is the next of the things the bot itself
//! offered - never a new fact. When the whole list is out, the reply asks the one thing that lets
//! the bot pick again.
#![allow(non_snake_case, clippy::all)]

use crate::data;
use crate::memory::choice;
use crate::memory::mem7::{self, inline_re, lists, res, sets, texts, values};
use crate::py::re;
use crate::py::text as pt;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

res!("reject"; A_HAVE_ALREADY, _I_HAVE, _WHOLE_LIST, _OFFER_NOT_A_LIST, _THE_LOT, _SOMEONE_ELSE_HAS_IT, STOPS_ON_AN_INITIAL, _NOT_A_NAME, _TITLE, TRIED_THE_LOT, _A_REASON, _OR_CHOICE, _FENCE, _HELD_THE_FENCE);
sets!("reject"; FILLER, NEGATION, OBJECT, JUDGED, DONE, POSSESSION, KNOWN, _ASKS_THE_BOT, _STEP_START, _CLAUSE_VERB, _NOT_A_THING_START);
lists!("reject"; ANOTHER, PICK_AGAIN, ALL_OF_THEM, ONE_LEFT, ONE_TAKEN, FENCED_OUT);
texts!("reject"; MEMORY_STORE);
values!("reject"; THE_MOST_A_LIST);
inline_re! {
    DO_NOT = r"\b(?:do not|did not|does not)\b", 0;
    NT = "n['\u{2019}]t\\b", 0;
    WORDS = r"[a-z']+", 0;
    HALF_BRACKET = r"\s*\([^)]*$", 0;
    A_FIGURE_FIRST = r"^(?:\d|[£$€]\s*\d)", 0;
    AN_ARTICLE = r"^(A|An|The) (?=[a-z])", 0;
    RUNS_ON = r"^(?:,|or\b|and\b|\*)", re::I;
    A_TIME_AFTER = r"\s+in\s+\d+\s*(?:minutes?|mins?|min)?\s*$", 0;
    A_NOTE = r"\s+(?:for|if|which|when|that|to|in|about)\s+", 0;
}
static NAME_GOES_ON: LazyLock<String> = LazyLock::new(|| data::re("bot.memory.reject.NAME_GOES_ON").pattern().to_string());

/// `recent(memory, n)`: the last n turns.
fn recent(memory: &Memory, n: usize) -> &[Turn] {
    mem7::tail(&memory.turns, n)
}

/// Python: bot/memory/reject.py:150 `_words(text)`
///
/// The line as words, chat-speak and courtesies stripped, or None when it is not a refusal.
pub fn _words(text: &str) -> Option<Vec<String>> {
    let low = pt::squash(&pt::lower(text).replace('\u{2019}', "'"));
    if low.is_empty() || low.contains('?') {
        return None;
    }
    let low = pt::strip_chars(&low, ".,;:!-\u{2026} ").to_string();
    let low = DO_NOT.sub("dont", &low);
    let low = NT.sub(" not", &low);
    let words: Vec<String> = pt::split_ref(&low).into_iter().map(|w| pt::strip_chars(w, ".,;:!-").to_string()).filter(|w| !w.is_empty()).collect();
    // "no thanks at all": the pair is how firmly the no was said, not the lot.
    let mut kept: Vec<String> = Vec::new();
    for word in words {
        if word == "all" && kept.last().is_some_and(|w| w == "at") {
            kept.pop();
            continue;
        }
        kept.push(word);
    }
    Some(kept.into_iter().filter(|w| !FILLER.contains(w)).collect())
}

/// Python: bot/memory/reject.py:186 `refusal(text, offer="")`
///
/// True when the whole line turns something down and says nothing else.
pub fn refusal(text: &str, offer: &str) -> bool {
    let Some(words) = _words(text) else {
        return false;
    };
    if words.is_empty() || words.len() > 7 || words.len() < 2 {
        return false;
    }
    if _SOMEONE_ELSE_HAS_IT.is_match(text) {
        return false;
    }
    let unknown: Vec<&String> = words.iter().filter(|w| !KNOWN.contains(*w)).collect();
    if !unknown.is_empty() {
        let named: HashSet<String> = WORDS.findall_str(&pt::lower(offer)).into_iter().collect();
        if named.is_empty() || unknown.len() > 3 || !unknown.iter().all(|w| named.contains(*w)) {
            return false;
        }
    }
    if _ASKS_THE_BOT.contains(&words[0]) {
        return false;
    }
    let judged: HashSet<&String> = words.iter().filter(|w| DONE.contains(*w)).collect();
    let joined = words.join(" ");
    if !judged.is_empty()
        && judged.iter().all(|w| POSSESSION.contains(*w))
        && !words.iter().any(|w| NEGATION.contains(w))
        && !A_HAVE_ALREADY.is_match(&joined)
        && _I_HAVE.match_(text).is_none()
    {
        return false;
    }
    if !words.iter().any(|w| NEGATION.contains(w) || DONE.contains(w)) {
        let points_at_the_whole = _WHOLE_LIST.is_match(&joined) && words.iter().all(|w| OBJECT.contains(w));
        if !points_at_the_whole {
            return false;
        }
    }
    words.iter().any(|w| OBJECT.contains(w) || JUDGED.contains(w))
}

/// Python: bot/memory/reject.py:233 `_the_whole_name(option, reply)`
///
/// The option, finished off when the reader cut it short on the full stop of an initial.
pub fn _the_whole_name(option: &str, reply: &str) -> String {
    let option = pt::strip(option);
    if !STOPS_ON_AN_INITIAL.is_match(option) {
        return option.to_string();
    }
    let core = pt::rstrip_chars(pt::split_ref(option).last().copied().unwrap_or(""), ".");
    let pattern = mem7::cached_re(&format!(r"\b{}{}", re::escape(core), *NAME_GOES_ON), 0);
    let Some(found) = pattern.search(reply) else {
        return option.to_string();
    };
    let rest = pt::rstrip_chars(pt::strip(found.str(1)), ".");
    format!("{}{} {}", option, if option.ends_with('.') { "" } else { "." }, rest)
}

/// Python: bot/memory/reject.py:249 `_tidy(option)`
///
/// One option as it can be said back: the name, without a note the reader cut in half; "" when
/// it is a figure, a step or a clause rather than a thing on offer.
pub fn _tidy(option: &str) -> String {
    let option = pt::strip(&HALF_BRACKET.sub("", option)).to_string();
    if A_FIGURE_FIRST.is_match(&option) {
        return String::new();
    }
    let option = AN_ARTICLE.sub_fn(&option, |m| pt::lower(m.str(1)));
    if !(2 < pt::len(&option) && mem7::word_count(&option) <= 10) {
        return String::new();
    }
    let low = pt::lower(&option);
    let words = pt::split_ref(&low);
    if _STEP_START.contains(words[0]) || words.iter().skip(1).take(2).any(|w| _CLAUSE_VERB.contains(*w)) {
        return String::new(); // a step or a clause, not one of the things on offer
    }
    option
}

/// Python: bot/memory/reject.py:304 `_by_title(titles, options)`
///
/// The reply's titles, in the order it gave them, each with its note when that survived.
pub fn _by_title(titles: &[String], options: &[String]) -> Vec<String> {
    if titles.len() < 2 {
        return options.to_vec();
    }
    let mut ordered: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for title in titles {
        let low = pt::lower(title);
        let mut said = options.iter().find(|o| pt::lower(o).starts_with(&low)).cloned().unwrap_or_default();
        if !said.is_empty() && pt::lower(&said) != low {
            // An option can run on past the title into the next thing.
            let rest = pt::strip(mem7::from_char(&said, pt::len(title))).to_string();
            if RUNS_ON.match_(&rest).is_some() {
                said = pt::strip(mem7::first_chars(&said, pt::len(title))).to_string();
            }
        }
        if said.is_empty() {
            said = options.iter().find(|o| pt::lower(o).contains(&low)).cloned().unwrap_or_default();
        }
        if said.is_empty() {
            said = title.clone();
        }
        if seen.contains(&pt::lower(&said)) {
            continue;
        }
        seen.insert(pt::lower(&said));
        ordered.push(said);
    }
    if ordered.len() >= 2 {
        ordered
    } else {
        options.to_vec()
    }
}

/// Python: bot/memory/reject.py:339 `_a_choice_of_things(reply)`
///
/// True when the reply put things up to choose between, not advice with a list in it.
pub fn _a_choice_of_things(reply: &str) -> bool {
    let reply = pt::squash(reply);
    let titles: HashSet<String> = _TITLE.findall_str(&reply).iter().map(|t| pt::lower(t)).collect();
    titles.len() >= 2 || _OR_CHOICE.is_match(&reply)
}

/// Python: bot/memory/reject.py:364 `_the_offer_reply(memory)`
///
/// The bot's own reply that put things up to choose between, or "".
pub fn _the_offer_reply(memory: &Memory) -> String {
    recent(memory, 4).iter().rev().map(|turn| turn.bot.clone()).find(|reply| !reply.is_empty() && _options(reply).len() >= 2).unwrap_or_default()
}

/// Python: bot/memory/reject.py:378 `_options(reply)`
///
/// The things one reply offered to choose between, or [] when it offered no choice.
pub fn _options(reply: &str) -> Vec<String> {
    let reply = pt::squash(reply);
    if reply.is_empty() || _OFFER_NOT_A_LIST.is_match(&reply) {
        return Vec::new();
    }
    let titles: Vec<String> = _TITLE.findall_str(&reply).iter().map(|t| _tidy(t)).filter(|t| !t.is_empty() && !_NOT_A_NAME.is_match(t)).collect();
    let options: Vec<String> = choice::options_in(&reply)
        .iter()
        .map(|o| _tidy(&_the_whole_name(o, &reply)))
        .filter(|o| !o.is_empty() && !_NOT_A_NAME.is_match(o))
        .collect();
    let options = _by_title(&titles, &options);
    // A one-word option that is not one of the reply's titles is a fragment.
    let known: HashSet<String> = titles.iter().map(|t| pt::lower(t)).collect();
    let options: Vec<String> = options.into_iter().filter(|o| mem7::word_count(o) > 1 || known.contains(&pt::lower(o))).collect();
    let short: Vec<String> = options.iter().filter(|o| mem7::word_count(o) <= 8).cloned().collect();
    let options = if short.is_empty() { options } else { short };
    if options.len() < 2 {
        return Vec::new();
    }
    // A choice says "or", or stars two titles, or is a colon list of names.
    if titles.len() < 2 && !_OR_CHOICE.is_match(&reply) {
        if !(reply.contains(':') && options.len() >= 3 && options.iter().all(|o| _a_noun_phrase(o))) {
            return Vec::new();
        }
    }
    if _a_list_of_causes(&reply, &options) {
        return Vec::new();
    }
    options
}

/// Python: bot/memory/reject.py:443 `_a_list_of_causes(reply, options)`
///
/// True when the "or" list the options came from is a reason rather than an offer.
pub fn _a_list_of_causes(reply: &str, options: &[String]) -> bool {
    let flat = pt::squash(reply);
    let low = pt::lower(&flat);
    for option in options {
        let at = pt::find(&low, &pt::lower(option));
        if at < 0 {
            continue;
        }
        let mut before = mem7::first_chars(&flat, at as usize).to_string();
        for stop in [". ", "! ", "? ", "; "] {
            if let Some(cut) = before.rfind(stop).filter(|cut| *cut > 0) {
                before = before[cut + stop.len()..].to_string();
            }
        }
        if _A_REASON.is_match(&before) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/reject.py:465 `_a_noun_phrase(option)`
///
/// True when an option reads as the name of a thing rather than a piece of a sentence.
pub fn _a_noun_phrase(option: &str) -> bool {
    let low = pt::lower(option);
    pt::split_ref(&low).first().is_some_and(|first| !_NOT_A_THING_START.contains(*first))
}

/// Python: bot/memory/reject.py:479 `_said_by_them(option, text)`
///
/// True when the person's own line says every word of `option` (or of its name, before a note).
pub fn _said_by_them(option: &str, text: &str) -> bool {
    let option = A_TIME_AFTER.subn("", option, 1);
    let said: HashSet<String> = WORDS.findall_str(&pt::lower(text)).into_iter().collect();
    let every_word = |name: &str| {
        let words: Vec<String> = WORDS.findall_str(&pt::lower(name)).into_iter().filter(|w| !matches!(w.as_str(), "a" | "an" | "the")).collect();
        !words.is_empty() && words.iter().all(|w| said.contains(w))
    };
    if every_word(&option) {
        return true;
    }
    // The bot's note on an option is not part of its name.
    let name = match A_NOTE.search(&option) {
        Some(m) => option[..m.start()].to_string(),
        None => option.clone(),
    };
    mem7::word_count(&name) > 1 && every_word(&name)
}

/// Python: bot/memory/reject.py:506 `_offered(memory)`
///
/// The offer the refusal is about, as the options of it the bot has not named again, or None.
pub fn _offered(memory: &Memory) -> Option<Vec<String>> {
    let turns = recent(memory, 4);
    let start = (0..turns.len()).rev().find(|&index| _options(&turns[index].bot).len() >= 2)?;
    // A reply after the offer that this module did not write means the chat has moved on.
    if turns[start + 1..].iter().any(|turn| turn.module.as_deref().unwrap_or("") != MEMORY_STORE.as_str()) {
        return None;
    }
    let mut left = _options(&turns[start].bot);
    for turn in &turns[start + 1..] {
        let named: Vec<String> = left.iter().filter(|o| _said_by_them(o, &turn.user)).cloned().collect();
        left = if named.is_empty() { left.into_iter().skip(1).collect() } else { left.into_iter().filter(|o| !named.contains(o)).collect() };
    }
    Some(left)
}

/// Python: bot/memory/reject.py:560 `_the_fence(memory)`
///
/// The thing the person ruled out in the request, when the offer that came back ignored it.
pub fn _the_fence(memory: &Memory) -> Option<String> {
    let turns = recent(memory, 3);
    let index = (0..turns.len()).rev().find(|&index| _options(&turns[index].bot).len() >= 2)?;
    let offer = &turns[index].bot;
    let options = _options(offer);
    let request = &turns[index].user;
    let ruled_out: Vec<String> = _FENCE.findall_str(request).into_iter().filter(|w| !KNOWN.contains(w)).collect();
    if ruled_out.is_empty() {
        return None;
    }
    // The lead-in is the offer's own words before the first thing it names.
    let started: Vec<usize> = options.iter().map(|o| pt::find(offer, o)).filter(|at| *at > 0).map(|at| at as usize).collect();
    let lead = match started.iter().min() {
        Some(first) => mem7::first_chars(offer, *first),
        None => mem7::first_chars(offer, 80),
    };
    if _HELD_THE_FENCE.is_match(lead) {
        return None;
    }
    let words: Vec<String> = WORDS.findall_str(&pt::lower(lead)).into_iter().filter(|w| pt::len(w) > 3).collect();
    for fence in &ruled_out {
        for word in &words {
            if mem7::first_chars(word, 4) == mem7::first_chars(fence, 4) {
                return Some(fence.clone());
            }
        }
    }
    None
}

/// Python: bot/memory/reject.py:597 `_the_whole_lot(options)`
///
/// "both" or "all three", for however many of them are still on the table.
pub fn _the_whole_lot(options: &[String]) -> String {
    match options.len() {
        2 => "both".into(),
        3 => "all three".into(),
        4 => "all four".into(),
        5 => "all five".into(),
        _ => "all of them".into(),
    }
}

/// Python: bot/memory/reject.py:607 `offers_a_choice(memory)`
///
/// True when the bot's last reply put two or more things up to choose between.
pub fn offers_a_choice(memory: &Memory) -> bool {
    if _offered(memory).map_or(0, |o| o.len()) < 2 {
        return false;
    }
    let reply = _the_offer_reply(memory);
    !reply.is_empty() && _a_choice_of_things(&reply)
}

/// Python: bot/memory/reject.py:624 `_the_names(options)`
///
/// The options as one list read aloud: "a, b and c".
pub fn _the_names(options: &[String]) -> String {
    let said: Vec<&String> = options.iter().filter(|o| !o.is_empty()).collect();
    if said.len() < 2 {
        return said.first().map_or(String::new(), |s| s.to_string());
    }
    let (last, rest) = said.split_last().expect("two at least");
    format!("{} and {}", rest.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "), last)
}

/// Python: bot/memory/reject.py:632 `_the_lot_and_the_verb(lines)`
///
/// (had the lot, wants the lot) for one line, read off the word that means all of it.
pub fn _the_lot_and_the_verb(lines: &[String]) -> (bool, bool) {
    let the_lot = _THE_LOT.is_match(&lines.join(" "));
    (the_lot && lines.iter().any(|w| DONE.contains(w)), the_lot)
}

/// `said[:1].upper() + said[1:]` of a pick formatted with these fields.
fn said_from(pool: &[String], memory: &Memory, text: &str, fields: &[(&str, &str)]) -> String {
    pt::upper_first(&mem7::fmt(&mem7::pick(pool, memory, text), fields))
}

/// Python: bot/memory/reject.py:644 `reply_for(text, memory)`
///
/// Another of the options the bot just offered, or a question that lets it pick again.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn reply_for(text: &str, memory: &Memory) -> Option<String> {
    let answer = recent(memory, 2).iter().rev().map(|turn| turn.bot.clone()).find(|bot| !bot.is_empty()).unwrap_or_default();
    if !refusal(text, &answer) {
        return None;
    }
    let offered = _offered(memory)?;
    // The whole of the advice tried is a report on it, not a shortlist turned down.
    if TRIED_THE_LOT.is_match(text) && !_a_choice_of_things(&_the_offer_reply(memory)) {
        return None;
    }
    // The offer is the thing they ruled out, so it has no answer in it at all.
    if let Some(fence) = _the_fence(memory) {
        return Some(said_from(&FENCED_OUT, memory, text, &[("kind", &fence)]));
    }
    let named_by_them: Vec<String> = offered.iter().filter(|o| _said_by_them(o, text)).cloned().collect();
    let left: Vec<String> = if named_by_them.is_empty() { offered.iter().skip(1).cloned().collect() } else { offered.iter().filter(|o| !named_by_them.contains(o)).cloned().collect() };
    let lines = _words(text).unwrap_or_default();
    let (mut had_them_all, mut wants_them_all) = _the_lot_and_the_verb(&lines);
    if offered.len() as i64 > THE_MOST_A_LIST.as_int().unwrap_or(5) {
        had_them_all = false;
        wants_them_all = false;
    }
    if offered.len() < 2 {
        if had_them_all || !(!left.is_empty() || wants_them_all) {
            return Some(mem7::pick(&PICK_AGAIN, memory, text));
        }
        let pool = if wants_them_all { &*ONE_TAKEN } else { &*ONE_LEFT };
        let option = left.first().or(offered.first()).expect("IndexError: nothing offered").clone();
        return Some(said_from(pool, memory, text, &[("option", &option)]));
    }
    let (which, names) = (_the_whole_lot(&offered), _the_names(&offered));
    if had_them_all {
        return Some(mem7::pick(&PICK_AGAIN, memory, text));
    }
    if wants_them_all {
        return Some(said_from(&ALL_OF_THEM, memory, text, &[("which", &which), ("names", &names)]));
    }
    if left.is_empty() || (!lines.is_empty() && _WHOLE_LIST.is_match(&lines.join(" "))) {
        return Some(mem7::pick(&PICK_AGAIN, memory, text));
    }
    Some(said_from(&ANOTHER, memory, text, &[("option", &left[0])]))
}
