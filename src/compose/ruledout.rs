//! Port of `bot/compose/ruledout.py`: a thing the person rules out ("not tablets", "nothing too
//! loud"), answered against the answer they just had - it was never in it, and what still stands.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback_chain::{_choose, caught, py_format};
use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understanding::kinds::names_a_claim;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.ruledout.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.ruledout.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.ruledout.", stringify!($name))));)*
    };
}

regexes!(
    RULED_OUT_TAIL, RULED_OUT_PREP, RULED_OUT_OPENS_WITH, RULED_OUT_SOFTENING, RULED_OUT_KEEPS, RULED_OUT_NAMED,
    RULED_OUT_KEEP_WHOLE, RULED_OUT_SYMPATHY, MANNERS, RULED_OUT_THEN_ELSE, CLAIM_OF_FAILURE, CLAIM_CONDITION,
    STEP_ELSEWHERE, PET_NAMED, WANTED_ONE, TITLES_IN_ANSWER,
);
word_sets!(
    RULED_OUT_WORDS, RULED_OUT_QUALITY, RULED_OUT_NOT_A_THING, RULED_OUT_CLASS, RULED_OUT_PARTICLES, CLAIM_SMALL,
);
pools!(
    RULED_OUT_REPLIES_PLAIN, RULED_OUT_REPLIES_THE, RULED_OUT_REPLIES_A_ROW, RULED_OUT_NO_ANSWER, CLAIM_LEADS,
    CLAIM_LEADS_IF, KINDS_A_LIST_CAN_SWAP,
);

/// `PUNT`, the shrugs, as `fallback` holds them.
static PUNT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.PUNT"));
static RULED_OUT_OPENING: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.compose.ruledout.RULED_OUT_OPENING").as_int().expect("an int") as usize);
static RULED_OUT_WHOLE: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.compose.ruledout.RULED_OUT_WHOLE").as_int().expect("an int") as usize);
static RULED_OUT_INSTANCES: LazyLock<Vec<(String, HashSet<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.ruledout.RULED_OUT_INSTANCES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
});
static A_ROW_NAMES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.compose.ruledout.A_ROW_NAMES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});

/// (pattern, keeps its determiner) rows.
fn pattern_rows(name: &str) -> Vec<(&'static Re, bool)> {
    data::get(name)
        .items()
        .iter()
        .map(|row| (row.items()[0].as_re().expect("a pattern"), row.items()[1].truthy()))
        .collect()
}
static RULED_OUT: LazyLock<Vec<(&'static Re, bool)>> = LazyLock::new(|| pattern_rows("bot.compose.ruledout.RULED_OUT"));
static RULED_OUT_NOT_THEN_ELSE: LazyLock<Vec<(&'static Re, bool)>> =
    LazyLock::new(|| pattern_rows("bot.compose.ruledout.RULED_OUT_NOT_THEN_ELSE"));
static RULED_OUT_NO_THEN_ELSE: LazyLock<Vec<(&'static Re, bool)>> =
    LazyLock::new(|| pattern_rows("bot.compose.ruledout.RULED_OUT_NO_THEN_ELSE"));

static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static SENTENCE_GAP: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s", 0));

/// `set(re.findall(r"[a-z']+", text))`.
fn az_set(text_: &str) -> HashSet<String> {
    A_WORD.findall_str(text_).into_iter().collect()
}

/// `{word, word + "s"}` plus the forms without "es" and "s".
fn forms_of(word: &str) -> HashSet<String> {
    let mut forms: HashSet<String> = [word.to_string(), format!("{word}s")].into_iter().collect();
    if let Some(stem) = word.strip_suffix("es") {
        forms.insert(stem.to_string());
    }
    if let Some(stem) = word.strip_suffix('s') {
        forms.insert(stem.to_string());
    }
    forms
}

/// `word[:7]`.
fn stem7(word: &str) -> String {
    text::slice(word, 0, Some(7))
}

/// `s[:1].upper() + s[1:]`.
fn first_upper(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => text::upper(&c.to_string()) + chars.as_str(),
        None => String::new(),
    }
}

/// `" ".join(str(text).lower().replace("’", "'").split())`.
fn cleaned(text_: &str) -> String {
    text::squash(&text::lower(text_).replace('\u{2019}', "'"))
}

/// Python: bot/compose/ruledout.py:219 `_named_picks(answer)`
/// True when the answer offers two or more named things rather than describing them.
pub fn _named_picks(answer: &str) -> bool {
    RULED_OUT_NAMED.findall(answer).len() > 1
}

/// Python: bot/compose/ruledout.py:224 `_class_of(word)`
/// The classes a ruled-out word belongs to, read from the word itself (a frozenset in Python).
pub fn _class_of(word: &str) -> Vec<String> {
    let word = text::strip_chars(word, "'");
    if word.is_empty() {
        return Vec::new();
    }
    let forms = forms_of(word);
    let mut classes: Vec<String> = RULED_OUT_CLASS.iter().filter(|key| forms.contains(*key)).cloned().collect();
    // A class said with a different ending, where the stem is long enough to be the word.
    if classes.is_empty() {
        let stem = stem7(word);
        if text::len(&stem) >= 6 {
            classes = RULED_OUT_CLASS.iter().filter(|key| key.starts_with(stem.as_str())).cloned().collect();
        }
    }
    // A member stands for its class too, but only when the word is not a class of its own.
    if classes.is_empty() {
        for (key, members) in RULED_OUT_INSTANCES.iter() {
            if forms.iter().any(|form| members.contains(form)) && !classes.contains(key) {
                classes.push(key.clone());
            }
        }
    }
    classes.sort();
    classes
}

/// Python: bot/compose/ruledout.py:256 `_a_class_word(word)`
/// True when the word is one of the class names itself, in any spelling.
pub fn _a_class_word(word: &str) -> bool {
    if forms_of(word).iter().any(|form| RULED_OUT_CLASS.contains(form)) {
        return true;
    }
    let stem = stem7(word);
    text::len(&stem) >= 6 && RULED_OUT_CLASS.iter().any(|key| key.starts_with(stem.as_str()))
}

/// Python: bot/compose/ruledout.py:269 `_an_answer_of_that_class(answer, thing)`
/// True when the answer names the ruled-out class or one of its members.
pub fn _an_answer_of_that_class(answer: &str, thing: &str) -> bool {
    let said = az_set(&text::lower(answer));
    for word in text::split(&RULED_OUT_KEEPS.sub("", thing)) {
        let classes = _class_of(&word);
        for key in &classes {
            let mut forms: HashSet<String> = [key.clone(), format!("{key}s")].into_iter().collect();
            if let Some(stem) = key.strip_suffix('s') {
                forms.insert(stem.to_string());
            }
            if forms.iter().any(|form| said.contains(form)) {
                return true;
            }
            if RULED_OUT_INSTANCES.iter().find(|(k, _)| k == key).is_some_and(|(_, members)| members.iter().any(|m| said.contains(m))) {
                return true;
            }
            // A class said in the answer with a different ending is the same class.
            let stem = stem7(key);
            if text::len(&stem) >= 6 && said.iter().any(|said_word| said_word.starts_with(stem.as_str())) {
                return true;
            }
        }
        // A *member* ruled out of an answer that offers named picks: the answer never says what
        // its titles are, so claiming none of them was one would be knowledge the bot lacks.
        if !classes.is_empty() && !_a_class_word(&word) && _named_picks(answer) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/ruledout.py:328 `_without_the_manners(clean)`
/// The line with the politeness it ends on taken off.
pub fn _without_the_manners(clean: &str) -> String {
    text::strip(&MANNERS.sub("", clean)).to_string()
}

/// Python: bot/compose/ruledout.py:361 `_ruled_out_thing(clean)`
/// The thing a line rules out, in the person's own words, and whether it kept its "a"/"the".
pub fn _ruled_out_thing(clean: &str) -> (String, bool) {
    let none = || (String::new(), false);
    let mut clean = _without_the_manners(clean);
    let asked_for_the_other = RULED_OUT_THEN_ELSE.is_match(&clean);
    if asked_for_the_other {
        clean = text::strip_chars(&RULED_OUT_THEN_ELSE.sub("", &clean), " ,;:-").to_string();
    }
    if clean.is_empty() || text::split_ref(&clean).len() > 7 {
        return none();
    }
    // "nothing too sad" softens the ask rather than leaving something out.
    if RULED_OUT_SOFTENING.match_(&clean).is_some() {
        return none();
    }
    let mut patterns: Vec<(&Re, bool)> = RULED_OUT.clone();
    if asked_for_the_other {
        patterns.extend(RULED_OUT_NOT_THEN_ELSE.iter().cloned());
        patterns.extend(RULED_OUT_NO_THEN_ELSE.iter().cloned());
    }
    let mut found_thing = None;
    for (pattern, with_determiner) in patterns {
        if let Some(found) = pattern.match_(&clean) {
            found_thing = Some((found.name_str("thing").to_string(), with_determiner));
            break;
        }
    }
    let Some((thing, kept)) = found_thing else { return none() };
    let thing = text::split_ref(&thing).into_iter().filter(|w| !RULED_OUT_WORDS.contains(*w)).collect::<Vec<_>>().join(" ");
    let thing = text::strip_chars(&RULED_OUT_TAIL.sub("", &thing), " .,!?'\"").to_string();
    if thing.is_empty() || RULED_OUT_PREP.is_match(&thing) || RULED_OUT_OPENS_WITH.is_match(&thing) {
        return none();
    }
    let words = text::split_ref(&thing);
    // A quality anywhere in the thing is an opinion about something, not a thing left out.
    if words.iter().any(|w| RULED_OUT_QUALITY.contains(*w)) {
        return none();
    }
    if words.iter().any(|w| RULED_OUT_NOT_A_THING.contains(*w)) {
        return none();
    }
    // A claim anywhere in the tail is a sentence about how things are, not something to hand back.
    if clean.starts_with("nothing") && words.iter().any(|w| names_a_claim(w)) {
        return none();
    }
    let kept = kept && RULED_OUT_KEEPS.match_(&thing).is_some();
    if words.len() > 4 || !words.iter().all(|w| text::is_alpha(w)) {
        return none();
    }
    (thing.clone(), kept)
}

/// Python: bot/compose/ruledout.py:416 `_a_row_thing(thing)`
/// The row in a ruled-out thing, or "" - trouble between people is how, not what.
pub fn _a_row_thing(thing: &str) -> String {
    let bare = RULED_OUT_KEEPS.sub("", &text::squash(thing));
    if let Some((_, row)) = A_ROW_NAMES.iter().find(|(k, _)| *k == bare) {
        return row.clone();
    }
    // `sorted(A_ROW_NAMES, key=len, reverse=True)`: longest first, a stable sort on dict order.
    let mut phrases: Vec<&(String, String)> = A_ROW_NAMES.iter().collect();
    phrases.sort_by(|a, b| text::len(&b.0).cmp(&text::len(&a.0)));
    for (phrase, row) in phrases {
        if bare == *phrase || bare.starts_with(&format!("{phrase} ")) {
            return row.clone();
        }
    }
    String::new()
}

/// Python: bot/compose/ruledout.py:427 `_names_it(answer, thing)`
/// True when an answer already names the thing, in any form it might be written in.
pub fn _names_it(answer: &str, thing: &str) -> bool {
    let said = az_set(&text::lower(answer));
    let mut words = text::split(&RULED_OUT_KEEPS.sub("", thing));
    let carrying: Vec<String> = words.iter().filter(|w| !RULED_OUT_PARTICLES.contains(*w)).cloned().collect();
    if !carrying.is_empty() {
        words = carrying;
    }
    for word in words {
        let mut forms: HashSet<String> =
            [word.clone(), format!("{word}s"), format!("{word}es"), format!("{word}ing"), format!("{word}ed")].into_iter().collect();
        if let Some(stem) = word.strip_suffix('s') {
            forms.insert(stem.to_string());
        }
        if let Some(stem) = word.strip_suffix("es") {
            forms.insert(stem.to_string());
        }
        if forms.iter().any(|form| said.contains(form)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/ruledout.py:451 `_the_whole_answer(memory)`
/// The bot's last line when it was an answer, not a nod, a question back or a shrug.
pub fn _the_whole_answer(memory: &Memory) -> String {
    let Some(turn) = memory.turns.last() else { return String::new() };
    let last = text::squash(&turn.bot);
    if text::len(&last) < 60 || last.ends_with('?') {
        return String::new();
    }
    let low = text::lower(&last);
    // A shrug is not an answer to stand behind.
    if PUNT.iter().any(|punt| low.contains(punt.as_str())) {
        return String::new();
    }
    // Nor is a line that answered a feeling.
    if RULED_OUT_NO_ANSWER.iter().any(|opener| low.starts_with(opener.as_str())) {
        return String::new();
    }
    // An emergency or a sympathy answer is not a paragraph to hand back a stretch of.
    if RULED_OUT_KEEP_WHOLE.is_match(&low) || RULED_OUT_SYMPATHY.is_match(&low) {
        return String::new();
    }
    last
}

/// Python: bot/compose/ruledout.py:476 `_the_standing_part(answer)`
/// The stretch of an answer worth handing back: a short answer whole, else its first step, or ""
/// when there is nothing short enough to be a reminder.
pub fn _the_standing_part(answer: &str) -> String {
    if text::len(answer) <= *RULED_OUT_WHOLE {
        return text::squash(answer);
    }
    let parts = SENTENCE_GAP.split(text::strip(answer));
    let mut first = parts[0].clone();
    let (head, _, tail) = text::partition(&first, ":");
    if !tail.is_empty() && text::split_ref(head).len() <= 8 {
        first = tail.to_string();
    }
    let first = text::squash(&first);
    if text::len(&first) <= *RULED_OUT_OPENING {
        return first;
    }
    let cut = text::slice(&first, 0, Some(*RULED_OUT_OPENING as isize));
    let mut stretches: Vec<String> = Vec::new();
    for mark in ["; ", ", ", " - ", " and "] {
        if cut.contains(mark) {
            let at = text::rfind(&cut, mark);
            stretches.push(text::strip_chars(&text::slice(&cut, 0, Some(at)), " ,;:-").to_string());
        }
    }
    stretches.retain(|stretch| text::split_ref(stretch).len() >= 4);
    // `max(stretches, key=len)`: the first of the longest.
    let mut best: Option<&String> = None;
    for stretch in &stretches {
        if best.is_none_or(|b| text::len(stretch) > text::len(b)) {
            best = Some(stretch);
        }
    }
    best.cloned().unwrap_or_default()
}

/// Python: bot/compose/ruledout.py:542 `_a_claim_of_failure(clean, memory)`
/// The next part of the answer, for a line that says the answer's steps did not take.
pub fn _a_claim_of_failure(clean: &str, memory: &Memory) -> String {
    if CLAIM_OF_FAILURE.match_(clean).is_none() {
        return String::new();
    }
    const LEADING: [&str; 21] = [
        "nothing", "nowt", "none", "neither", "of", "ok", "okay", "so", "well", "yeah", "yep", "yes", "and", "but",
        "oh", "right", "alright", "no", "nope", "nah", "still",
    ];
    let mut words = A_WORD.findall_str(clean);
    while words.first().is_some_and(|w| LEADING.contains(&w.as_str()) || w == "now") {
        words.remove(0);
    }
    if words.is_empty() || !words.iter().any(|w| names_a_claim(w)) {
        return String::new();
    }
    if words.iter().any(|w| !CLAIM_SMALL.contains(w) && !names_a_claim(w)) {
        return String::new();
    }
    let answer = _the_whole_answer(memory);
    if answer.is_empty() {
        return String::new();
    }
    // An answer that ends on a question is mid-conversation: there is no step to hand back.
    if text::rstrip(&answer).ends_with('?') {
        return String::new();
    }
    // The answer holds the step that comes after the one that did not work, read as `repeat`
    // reads "ive tried the plunger already"; a reading that fails leaves the line to the entries.
    let Some(step) = caught(|| crate::memory::repeat::later_part(clean, &answer)) else { return String::new() };
    let step = text::squash(&step);
    if text::split_ref(&step).len() < 4 || step.is_empty() {
        return String::new();
    }
    // What comes back has to be the answer going further: a condition, or somebody to go to.
    let conditional = CLAIM_CONDITION.match_(&step).is_some();
    if !conditional && !STEP_ELSEWHERE.is_match(&step) {
        return String::new();
    }
    let step = first_upper(&step);
    let pool = if conditional { &*CLAIM_LEADS_IF } else { &*CLAIM_LEADS };
    let lines: Vec<String> = pool.iter().map(|line| py_format(line, &[("step", &step)])).collect();
    _choose(&lines, memory, clean, None).unwrap_or_default()
}

/// Python: bot/compose/ruledout.py:607 `_a_pet_turned_down(clean, memory)`
/// True when the line turns down an animal the line above asked about getting one of.
pub fn _a_pet_turned_down(clean: &str, memory: &Memory) -> bool {
    if !PET_NAMED.is_match(clean) {
        return false;
    }
    let turns = &memory.turns;
    for turn in turns[turns.len().saturating_sub(2)..].iter().rev() {
        let before = text::lower(&text::squash(&turn.user));
        if !before.is_empty() {
            return PET_NAMED.is_match(&before) && WANTED_ONE.is_match(&before);
        }
    }
    false
}

/// Python: bot/compose/ruledout.py:636 `_a_list_that_can_be_swapped(answer, thing)`
/// True when the answer is a list of picks and the thing ruled out is the kind in it.
pub fn _a_list_that_can_be_swapped(answer: &str, thing: &str) -> bool {
    let bare = RULED_OUT_KEEPS.sub("", &text::squash(thing));
    if !text::split_ref(&bare).iter().any(|w| KINDS_A_LIST_CAN_SWAP.iter().any(|k| k == w)) {
        return false;
    }
    TITLES_IN_ANSWER.findall(answer).len() >= 2
}

/// Python: bot/compose/ruledout.py:644 `_rules_out_what_i_never_said(text, memory)`
/// The thing ruled out, whether it kept its "a"/"the", and the part that still stands.
pub fn _rules_out_what_i_never_said(text_: &str, memory: &Memory) -> (String, bool, String) {
    let none = || (String::new(), false, String::new());
    let clean = cleaned(text_);
    let (thing, kept) = _ruled_out_thing(&clean);
    if thing.is_empty() {
        return none();
    }
    if _a_pet_turned_down(&clean, memory) {
        return none();
    }
    let answer = _the_whole_answer(memory);
    if answer.is_empty() || _names_it(&answer, &thing) {
        return none();
    }
    if _a_list_that_can_be_swapped(&answer, &thing) {
        return none();
    }
    if _an_answer_of_that_class(&answer, &thing) {
        return none();
    }
    let opening = _the_standing_part(&answer);
    if opening.is_empty() || _names_it(&opening, &thing) {
        return none();
    }
    let thing = if kept { thing } else { RULED_OUT_KEEPS.sub("", &thing) };
    (thing, kept, opening)
}

/// Python: bot/compose/ruledout.py:665 `_ruled_out_reply(memory, clean)`
/// One line saying the thing just ruled out was never in the answer, and what still stands.
pub fn _ruled_out_reply(memory: &Memory, clean: &str) -> String {
    let (thing, kept, opening) = _rules_out_what_i_never_said(clean, memory);
    if thing.is_empty() {
        return String::new();
    }
    let opening = first_upper(&opening);
    // A row between two people is how the advice is to be done, not a thing missing from it.
    let row = _a_row_thing(&thing);
    if !row.is_empty() {
        let lines: Vec<String> =
            RULED_OUT_REPLIES_A_ROW.iter().map(|line| py_format(line, &[("thing", &row), ("opening", &opening)])).collect();
        return _choose(&lines, memory, clean, None).unwrap_or_default();
    }
    let mut pool: &[String] = if kept { &RULED_OUT_REPLIES_THE } else { &RULED_OUT_REPLIES_PLAIN };
    // "There's no tablets" reads wrong, so a plural thing gets the two lines that fit either.
    if !kept && text::split_ref(&thing).last().is_some_and(|w| w.ends_with('s')) {
        pool = &RULED_OUT_REPLIES_PLAIN[1..];
    }
    let lines: Vec<String> = pool.iter().map(|line| py_format(line, &[("thing", &thing), ("opening", &opening)])).collect();
    _choose(&lines, memory, clean, None).unwrap_or_default()
}

/// Python: bot/compose/ruledout.py:685 `reply_for(text, memory)`
/// The rule-out answered against the answer above, or "" when this is not that shape; a reading
/// that fails answers the line as it did before.
pub fn reply_for(text_: &str, memory: &Memory) -> String {
    caught(|| {
        let clean = cleaned(text_);
        // A claim that nothing worked is a report about the answer, not a thing left out of it.
        let claim = _a_claim_of_failure(&clean, memory);
        if !claim.is_empty() {
            return claim;
        }
        _ruled_out_reply(memory, &clean)
    })
    .unwrap_or_default()
}

/// Python: bot/compose/ruledout.py:698 `takes_as_it_comes(text, memory)`
/// Does this line steer the answer it followed? A hook is asked before the reply is made; a hook
/// that raises claims nothing.
pub fn takes_as_it_comes(text_: &str, memory: &Memory) -> Option<bool> {
    let claims = caught(|| {
        let clean = cleaned(text_);
        if !_a_claim_of_failure(&clean, memory).is_empty() {
            return true;
        }
        !_rules_out_what_i_never_said(text_, memory).0.is_empty()
    });
    Some(claims.unwrap_or(false))
}

