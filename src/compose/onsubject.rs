//! Port of `bot/compose/onsubject.py`: a short follow-up with the subject left in the chat gets
//! the subject said back and who settles it, instead of the search hedge.
#![allow(non_snake_case, clippy::all)]

use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.onsubject.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.onsubject.", stringify!($name))));)*
    };
}
macro_rules! string_maps {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashMap<String, String>> =
            LazyLock::new(|| data::str_map(concat!("bot.compose.onsubject.", stringify!($name))));)*
    };
}

regexes!(
    A_ASK_OPENER, A_BACK_REFERENCE, A_EMPTY_ASK_OPENER, A_POINTER_ASK, A_HELP_ASK, A_RECALL_ASK, A_COMPARISON_ASK,
    A_COOKING_TIME_ASK, A_FOOD_PRONOUN, A_ROASTING_WORD, A_BOILING_WORD, A_OTHER_METHOD, A_NAMED_FOOD, A_COOKING_ASK,
    A_LOST_THING, A_MACHINE_CUE, A_A_QUESTION_ASK, A_JUDGEMENT_ASK, A_JUDGEMENT_OF_AMOUNT, A_WORDS_ASK, A_NAMING_ASK,
    A_EXPLANATION, A_ITS_OWN_SUBJECT, A_DVLA_WORD, A_THING_VERB_ASK, A_OPEN_ASK, A_WORD, A_THERE, A_NAMED_PLACE,
    A_REPEAT_LEAD, A_EXISTENTIAL_THERE,
);
word_sets!(A_PLACE_WORD, A_DETERMINER, A_PLACE_PREP, A_LIGHT_KEY, STOP, PLACES, A_PASSPORT_WORD, A_NOT_A_PLACE);
string_maps!(COOKING_TIME_SHAPES, A_SHOWN_AS, WHO, SHAPES, A_TURNS_ON);

fn int_named(name: &str) -> usize {
    data::get(&format!("bot.compose.onsubject.{name}")).as_int().expect("an int") as usize
}
static A_SUBJECT_WORD_LIMIT: LazyLock<usize> = LazyLock::new(|| int_named("A_SUBJECT_WORD_LIMIT"));
static A_EMPTY_ASK_WORDS: LazyLock<usize> = LazyLock::new(|| int_named("A_EMPTY_ASK_WORDS"));
static A_EMPTY_ASK_LENGTH: LazyLock<usize> = LazyLock::new(|| int_named("A_EMPTY_ASK_LENGTH"));
static A_KNOWS_IT: LazyLock<f64> =
    LazyLock::new(|| data::get("bot.compose.onsubject.A_KNOWS_IT").as_f64().expect("a float"));
static WHO_CAR_GARAGE: LazyLock<String> = LazyLock::new(|| data::string("bot.compose.onsubject.WHO_CAR_GARAGE"));
static WHO_PASSPORT: LazyLock<String> = LazyLock::new(|| data::string("bot.compose.onsubject.WHO_PASSPORT"));
static AMOUNT_KINDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.onsubject.AMOUNT_KINDS"));
static MACHINE_BEFORE_MACHINE_KINDS: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.compose.onsubject.MACHINE_BEFORE_MACHINE_KINDS"));
/// `KINDS`: (kind, keys) in order.
static KINDS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.onsubject.KINDS").items().iter().map(|row| (row.items()[0].str().to_string(), row.items()[1].strs())).collect()
});
static A_NEEDS_A_NEIGHBOUR: LazyLock<HashMap<String, HashSet<String>>> = LazyLock::new(|| {
    data::get("bot.compose.onsubject.A_NEEDS_A_NEIGHBOUR").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str_set())).collect()
});
/// `fallback.UNKNOWN`, the "I don't know" pool.
static UNKNOWN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.UNKNOWN"));

static WORD_RUN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static HOW_LONG: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow long\b", 0));
static HOW_SOON: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow soon\b", 0));
static HOW_MUCH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow much\b", 0));
static HOW_MANY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bhow many\b", 0));
static OPENS_HOW: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*how\b", 0));
static OPENS_WH: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*(?:what|which|who|when|where)\b", 0));
static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", 0));

/// `fallback._in_a_pool(text, (fallback.UNKNOWN,))`.
fn in_the_unknown_pool(text_: &str) -> bool {
    crate::compose::fallback::in_a_pool(text_, &[&UNKNOWN[..]])
}

/// `" ".join(str(x).lower().split())`.
fn flat(s: &str) -> String {
    text::squash(&text::lower(s))
}

/// Python: bot/compose/onsubject.py:347 `_words(line)`
pub fn _words(line: &str) -> Vec<String> {
    WORD_RUN.findall_str(&text::lower(line))
}

/// Python: bot/compose/onsubject.py:351 `_bare_follow_up(clean)`
/// The line when it is a short ask about something said earlier, else None.
pub fn _bare_follow_up(clean: &str) -> Option<String> {
    let text_ = flat(clean);
    let words = _words(&text_);
    if text_.is_empty() || words.len() > 8 {
        return None;
    }
    if !(A_ASK_OPENER.match_(&text_).is_some() || text::rstrip(&text_).ends_with('?')) {
        return None;
    }
    let subject: Vec<&String> = words.iter().filter(|w| !STOP.contains(*w) && text::len(w) > 2).collect();
    if subject.len() > *A_SUBJECT_WORD_LIMIT {
        return None;
    }
    if !A_BACK_REFERENCE.is_match(&text_) {
        // With nothing pointing back, only a short "how"/"when" ask with almost nothing in it is
        // a follow-up about what was said.
        if subject.len() > *A_EMPTY_ASK_WORDS || words.len() > *A_EMPTY_ASK_LENGTH || A_EMPTY_ASK_OPENER.match_(&text_).is_none() {
            return None;
        }
    }
    Some(text_)
}

/// Python: bot/compose/onsubject.py:372 `_stem(word)`
/// The part of a word that stays the same when it is inflected, so "sack" meets "sacked".
pub fn _stem(word: &str) -> String {
    let low = text::lower(word);
    let mut word = text::strip_chars(&low, "'").to_string();
    for suffix in ["ing", "ies", "ied", "es", "ed", "s"] {
        if text::len(&word) > suffix.len() + 2 && word.ends_with(suffix) {
            word.truncate(word.len() - suffix.len());
            break;
        }
    }
    if text::len(&word) > 3 && word.ends_with('e') {
        word.pop();
    }
    word
}

/// Python: bot/compose/onsubject.py:390 `_only_a_place(words, index)`
/// True when the word at index only ever appears as the place: "at work", "in a flat".
pub fn _only_a_place(words: &[String], index: usize) -> bool {
    let before = if index > 0 { words[index - 1].as_str() } else { "" };
    if A_PLACE_PREP.contains(before) {
        return true;
    }
    A_DETERMINER.contains(before) && index > 1 && A_PLACE_PREP.contains(&words[index - 2])
}

/// Python: bot/compose/onsubject.py:400 `_beside(words, index, allowed)`
/// True when the word either side of this one is in allowed: "washing machine", "a drum wash".
pub fn _beside(words: &[String], index: usize, allowed: &HashSet<String>) -> bool {
    if index > 0 && allowed.contains(&words[index - 1]) {
        return true;
    }
    index + 1 < words.len() && allowed.contains(&words[index + 1])
}

/// Python: bot/compose/onsubject.py:407 `_named(line, skip_place_words=False, light=True)`
/// [(the word from KINDS that the line says, the kind)] for one line, in KINDS order.
pub fn _named(line: &str, skip_place_words: bool, light: bool) -> Vec<(String, String)> {
    let words = _words(line);
    let mut said: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, word) in words.iter().enumerate() {
        said.entry(_stem(word)).or_default().push(index);
    }
    let mut found = Vec::new();
    for (kind, keys) in KINDS.iter() {
        for key in keys {
            let Some(places) = said.get(&_stem(key)).filter(|p| !p.is_empty()) else { continue };
            if light && A_LIGHT_KEY.contains(key) {
                continue;
            }
            if let Some(neighbour) = A_NEEDS_A_NEIGHBOUR.get(key).filter(|n| !n.is_empty()) {
                if !places.iter().any(|&index| _beside(&words, index, neighbour)) {
                    continue;
                }
            }
            if skip_place_words && A_PLACE_WORD.contains(key) && places.iter().all(|&index| _only_a_place(&words, index)) {
                // "at work", "in a flat": the place a thing happened, not the thing.
                continue;
            }
            found.push((key.clone(), kind.clone()));
            break;
        }
    }
    found
}

/// Python: bot/compose/onsubject.py:433 `_the_subject(memory, clean="")`
/// (the word the chat is on, the kind of thing it is, the line it came from) or blanks. The two
/// newest lines the person said that are not questions or explanations are read, newest first.
pub fn _the_subject(memory: &Memory, clean: &str) -> (String, String, String) {
    let now = flat(clean);
    let blank = || (String::new(), String::new(), String::new());
    let mut said_things: Vec<&str> = Vec::new();
    for turn in memory.turns.iter().rev() {
        if turn.user.is_empty() {
            continue;
        }
        let said = flat(&turn.user);
        if said.is_empty() || said == now || A_A_QUESTION_ASK.match_(&said).is_some() {
            continue;
        }
        if A_EXPLANATION.is_match(&said) {
            continue;
        }
        said_things.push(&turn.user);
        if said_things.len() == 2 {
            break;
        }
    }
    if said_things.is_empty() {
        return blank();
    }
    // The newest line names more than one kind: the ask itself says which one it is about.
    let newest = _named(said_things[0], true, true);
    let newest_kinds: HashSet<&String> = newest.iter().map(|(_, kind)| kind).collect();
    if newest_kinds.len() > 1 {
        for (word, kind) in _named(&now, true, true) {
            if newest_kinds.contains(&kind) {
                return (word, kind, now);
            }
        }
    }
    for line in said_things {
        if let Some((word, kind)) = _named(line, true, true).into_iter().next() {
            return (word, kind, line.to_string());
        }
    }
    blank()
}

/// Python: bot/compose/onsubject.py:492 `_already_the_who(word, kind)`
/// True when the reply's own place is this word, so saying it back would repeat it.
pub fn _already_the_who(word: &str, kind: &str) -> bool {
    let stem = _stem(word);
    _words(WHO.get(kind).map(String::as_str).unwrap_or("")).iter().any(|each| _stem(each) == stem)
}

/// Python: bot/compose/onsubject.py:503 `_beside_the_who(line, kind, subject)`
/// Another word of the same kind in the line, when the subject is already the reply's place.
pub fn _beside_the_who(line: &str, kind: &str, subject: &str) -> String {
    let keys: &[String] = KINDS.iter().rev().find(|(k, _)| k == kind).map(|(_, keys)| keys.as_slice()).unwrap_or(&[]);
    let mut by_stem: HashMap<String, &String> = HashMap::new();
    for key in keys {
        by_stem.entry(_stem(key)).or_insert(key);
    }
    for word in _words(line) {
        let Some(key) = by_stem.get(&_stem(&word)) else { continue };
        if key.as_str() == subject || A_LIGHT_KEY.contains(*key) || PLACES.contains(*key) {
            continue;
        }
        if _already_the_who(key, kind) {
            continue;
        }
        return key.to_string();
    }
    String::new()
}

/// Python: bot/compose/onsubject.py:523 `_shape(text)`
pub fn _shape(text_: &str) -> String {
    let shape = if A_WORDS_ASK.match_(text_).is_some() {
        "words"
    } else if HOW_LONG.is_match(text_) || HOW_SOON.is_match(text_) {
        "howlong"
    } else if HOW_MUCH.is_match(text_) || HOW_MANY.is_match(text_) {
        "howmuch"
    } else if OPENS_HOW.match_(text_).is_some() {
        "how"
    } else if OPENS_WH.match_(text_).is_some() {
        "what"
    } else {
        "yesno"
    };
    shape.to_string()
}

/// Python: bot/compose/onsubject.py:538 `_content(text)`
/// The words of a sentence that carry its meaning, for telling it from one already said.
pub fn _content(text_: &str) -> Vec<String> {
    A_WORD.findall_str(&text::lower(text_)).into_iter().filter(|w| text::len(w) > 3).collect()
}

/// Python: bot/compose/onsubject.py:543 `_already_said(sentence, memory)`
/// True when the bot has said most of this sentence already, in its last few replies.
pub fn _already_said(sentence: &str, memory: &Memory) -> bool {
    let mine = _content(sentence);
    if mine.is_empty() {
        return true;
    }
    let turns = &memory.turns;
    let seen: HashSet<String> = turns[turns.len().saturating_sub(3)..].iter().flat_map(|t| _content(&t.bot)).collect();
    if seen.is_empty() {
        return false;
    }
    mine.iter().filter(|w| seen.contains(*w)).count() as f64 >= *A_KNOWS_IT * mine.len() as f64
}

/// Python: bot/compose/onsubject.py:558 `_one_sentence(text, limit=250)`
/// The first sentence of an answer, cut at a clause if it runs long, for a short reply.
pub fn _one_sentence(text_: &str, limit: usize) -> String {
    let mut said = text::squash(text_);
    let stop = text::find(&said, ". ");
    if 40 < stop && stop < limit as isize {
        said = text::slice(&said, 0, Some(stop + 1));
    }
    if text::len(&said) > limit {
        let window = text::slice(&said, 0, Some(limit as isize - 3));
        let mut cut = text::rfind(&window, ", ");
        if cut < 100 {
            cut = text::rfind(&window, " ");
        }
        said = text::rstrip_chars(&text::slice(&said, 0, Some(cut.max(100))), " ,;:-").to_string() + ", and";
    }
    // An opening quote or bracket left without its close reads as broken.
    for opener in ["\"", "(", "["] {
        if text::count(&said, opener) % 2 == 1 {
            let at = text::rfind(&said, opener);
            if at > 40 {
                said = text::rstrip_chars(&text::slice(&said, 0, Some(at)), " ,;:-").to_string();
            }
        }
    }
    said
}

/// Python: bot/compose/onsubject.py:580 `_cap(clause)`
/// A clause that starts a sentence gets a capital; one used mid-sentence does not.
pub fn _cap(clause: &str) -> String {
    let mut chars = clause.chars();
    match chars.next() {
        Some(c) => text::upper(&c.to_string()) + chars.as_str(),
        None => String::new(),
    }
}

/// Python: bot/compose/onsubject.py:617 `_the_place(line)`
/// The place the subject's line names, when it names one: "we are in spain" -> "spain".
pub fn _the_place(line: &str) -> Option<String> {
    let low = text::lower(line);
    for found in A_NAMED_PLACE.finditer(&low) {
        let word = found.str(1);
        if !A_NOT_A_PLACE.contains(word) && !STOP.contains(word) && text::len(word) > 2 {
            return Some(word.to_string());
        }
    }
    None
}

/// Python: bot/compose/onsubject.py:626 `_the_answer_about_the_place(text, line, memory)`
/// What the bot knows about the ask with the place put back into it, or "". Only a real answer
/// is taken: the search hedge back means no entry claims that question.
pub fn _the_answer_about_the_place(text_: &str, line: &str, memory: &Memory) -> String {
    if !A_THERE.is_match(text_) {
        return String::new();
    }
    // "is there a fee" and "how long does it take there" are not a place put back.
    if A_EXISTENTIAL_THERE.is_match(text_) || _shape(text_) == "howlong" {
        return String::new();
    }
    let Some(place) = _the_place(line) else { return String::new() };
    let spliced = A_THERE.subn(&format!("in {place}"), text_, 1);
    // The answers lane is asked on a memory that holds only the turns, so nothing it notes while
    // it reads reaches the real one.
    let mut clean = Memory { turns: memory.turns.clone(), ..Memory::default() };
    let Some(answer) = crate::compose::fallback_chain::caught(|| crate::modules::answers::respond(&spliced, &mut clean)) else {
        return String::new();
    };
    let Some(answer) = answer.filter(|a| !text::strip(a).is_empty()) else { return String::new() };
    if in_the_unknown_pool(&answer) {
        return String::new();
    }
    let first = _one_sentence(&answer, 250);
    let rest = text::strip(&text::slice(&answer, text::len(&first) as isize, None)).to_string();
    let second = _one_sentence(&rest, 250);
    let said = text::strip(&format!("{first}{}", if second.is_empty() { String::new() } else { format!(" {second}") })).to_string();
    // "Same as before - ..." is the lane saying it already answered, not an answer about the place.
    if A_REPEAT_LEAD.match_(&said).is_some() {
        return String::new();
    }
    // The answer has to be about the place the ask was put back with.
    if !text::lower(&said).contains(&text::lower(&place)) {
        return String::new();
    }
    said
}

/// Python: bot/compose/onsubject.py:684 `_what_it_knows(line, memory)`
/// One sentence the knowledge base has about the thing the chat is on, or "". A sentence the bot
/// has already said is skipped; a lookup that fails knows nothing, and says so.
pub fn _what_it_knows(line: &str, memory: &Memory) -> String {
    let line = text::squash(line);
    if line.is_empty() {
        return String::new();
    }
    let answer = crate::compose::fallback_chain::caught(|| -> Option<String> {
        let reading = crate::understanding::understand(&line, None);
        let entry = crate::knowledge::lookup::find(&reading, None, None, false)?;
        let answer = match &entry.reply {
            crate::knowledge::entry::Reply::Text(s) => s.clone(),
            crate::knowledge::entry::Reply::Fn(f) => crate::knowledge::registry::reply(f, &mut Memory::new(), &reading),
            crate::knowledge::entry::Reply::Choices(_) => return None,
        };
        if text::strip(&answer).is_empty() || in_the_unknown_pool(&answer) {
            return None;
        }
        Some(answer)
    })
    .flatten();
    let Some(answer) = answer else { return String::new() };
    for sentence in SENTENCE_END.split(&text::squash(&answer)) {
        if text::len(&sentence) < 40 || _already_said(&sentence, memory) {
            continue;
        }
        return _one_sentence(&sentence, 250);
    }
    String::new()
}

/// Python: bot/compose/onsubject.py:721 `_the_line_before(chat, now)`
/// The newest line the person said that names a place, and is not the ask itself.
pub fn _the_line_before(chat: &Memory, now: &str) -> String {
    let turns: &[Turn] = &chat.turns;
    for turn in turns[turns.len().saturating_sub(3)..].iter().rev() {
        if turn.user.is_empty() || flat(&turn.user) == now {
            continue;
        }
        if _the_place(&turn.user).is_some() {
            return turn.user.clone();
        }
    }
    String::new()
}

/// Python: bot/compose/onsubject.py:734 `place_pool(clean, understood=None, memory=None)`
/// A there-ask about the place the chat named, where the line names no subject at all, or None.
///
/// Python asks `fallback.reply_for(clean, memory)` on the real memory, which the composer may
/// write to; the contract's `&Memory` cannot carry that, so this reads a copy. `place_pool_mut`
/// is the same reading on the memory itself.
pub fn place_pool(clean: &str, understood: &Understood, memory: &Memory) -> Option<String> {
    let mut chat = memory.clone();
    place_pool_mut(clean, understood, &mut chat)
}

/// `place_pool` with the composer's writes kept in `memory`, as in Python.
pub fn place_pool_mut(clean: &str, _understood: &Understood, memory: &mut Memory) -> Option<String> {
    let text_ = _bare_follow_up(clean)?;
    if !A_THERE.is_match(&text_) || A_EXISTENTIAL_THERE.is_match(&text_) {
        return None;
    }
    if _shape(&text_) == "howlong" {
        return None;
    }
    if A_POINTER_ASK.match_(&text_).is_some() || A_HELP_ASK.match_(&text_).is_some() || A_JUDGEMENT_ASK.is_match(&text_) {
        return None;
    }
    if A_RECALL_ASK.is_match(&text_) || A_COMPARISON_ASK.is_match(&text_) || A_COOKING_ASK.is_match(&text_) {
        return None;
    }
    if A_NAMING_ASK.is_match(&text_) || A_ITS_OWN_SUBJECT.is_match(&text_) {
        return None;
    }
    // A broken composer keeps its own turn.
    let reply = crate::compose::fallback_chain::caught(|| crate::compose::fallback_chain::reply_for(clean, memory, true))?;
    if !in_the_unknown_pool(&reply) {
        return None;
    }
    let line = _the_line_before(memory, &text_);
    if line.is_empty() {
        return None;
    }
    let answer = _the_answer_about_the_place(&text_, &line, memory);
    let place = _the_place(&line);
    match place {
        Some(place) if !answer.is_empty() => Some(format!("{}, then. {}", _cap(&place), answer)),
        _ => None,
    }
}

/// Python: bot/compose/onsubject.py:774 `_food_above(memory)`
/// True when a recent line is already about food, so the chat can answer this one.
pub fn _food_above(memory: &Memory) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(6)..].iter().any(|turn| {
        A_COOKING_ASK.is_match(&turn.user) || A_NAMED_FOOD.is_match(&turn.user) || A_ROASTING_WORD.is_match(&turn.user)
    })
}

/// Python: bot/compose/onsubject.py:786 `pool(clean, understood=None, memory=None)`
/// The honest line for a follow-up whose subject is only in the chat, or None.
pub fn pool(clean: &str, _understood: &Understood, memory: &Memory) -> Option<String> {
    let text_ = _bare_follow_up(clean)?;
    if A_POINTER_ASK.match_(&text_).is_some() || A_HELP_ASK.match_(&text_).is_some() || A_JUDGEMENT_ASK.is_match(&text_) {
        return None;
    }
    if A_NAMING_ASK.is_match(&text_) {
        return None;
    }
    if A_ITS_OWN_SUBJECT.is_match(&text_) {
        return None;
    }
    let first_subject = _the_subject(memory, &text_).0;
    if first_subject.is_empty()
        && A_COOKING_TIME_ASK.is_match(&text_)
        && A_FOOD_PRONOUN.is_match(&text_)
        && !A_NAMED_FOOD.is_match(&text_)
        && !_food_above(memory)
    {
        // "how long do i cook it for" with no food anywhere: the method says which rule of thumb.
        if A_ROASTING_WORD.is_match(&text_) {
            return COOKING_TIME_SHAPES.get("oven").cloned();
        }
        if A_BOILING_WORD.is_match(&text_) {
            return COOKING_TIME_SHAPES.get("boil").cloned();
        }
        if A_COOKING_ASK.is_match(&text_) && !A_OTHER_METHOD.is_match(&text_) {
            return COOKING_TIME_SHAPES.get("either").cloned();
        }
    }
    if A_RECALL_ASK.is_match(&text_) || A_COMPARISON_ASK.is_match(&text_) || A_COOKING_ASK.is_match(&text_) {
        return None;
    }
    let (mut subject, kind, line) = _the_subject(memory, &text_);
    if subject.is_empty() {
        return None;
    }
    // The ask names a kind of its own, and it is not the one the chat is on.
    if line != text_ {
        let mine: Vec<String> = _named(&text_, true, true).into_iter().map(|(_, k)| k).collect();
        if !mine.is_empty() && !mine.contains(&kind) {
            return None;
        }
    }
    let mut who = WHO.get(&kind).cloned().filter(|w| !w.is_empty())?;
    if AMOUNT_KINDS.contains(&kind) && A_JUDGEMENT_OF_AMOUNT.is_match(&text_) {
        return None;
    }
    if MACHINE_BEFORE_MACHINE_KINDS.contains(&kind) && A_MACHINE_CUE.is_match(&line) {
        return None;
    }
    if A_LOST_THING.is_match(&line) {
        // A thing that has gone missing wants finding, not the place that settles a rule.
        return None;
    }
    if _already_the_who(&subject, &kind) {
        let beside = _beside_the_who(&line, &kind, &subject);
        if !beside.is_empty() {
            subject = beside;
        }
    }
    if PLACES.contains(&subject) && A_THING_VERB_ASK.is_match(&text_) {
        return None;
    }
    if A_PASSPORT_WORD.contains(&subject) {
        who = WHO_PASSPORT.clone();
    }
    if kind == "car" && !(A_DVLA_WORD.is_match(&text_) || A_DVLA_WORD.is_match(&line)) {
        who = WHO_CAR_GARAGE.clone();
    }
    let mut said = if PLACES.contains(&subject) {
        subject.clone()
    } else {
        format!("the {}", A_SHOWN_AS.get(&subject).cloned().unwrap_or_else(|| subject.clone()))
    };
    let shape_kind = _shape(&text_);
    let shape = SHAPES.get(&shape_kind).cloned().unwrap_or_else(|| panic!("KeyError: {shape_kind:?}"));
    if shape.starts_with("{subject}") {
        said = _cap(&said);
    }
    let mut known = String::new();
    if A_OPEN_ASK.is_match(&text_) && _named(&text_, true, true).is_empty() {
        known = _what_it_knows(&line, memory);
    }
    let mut whole = String::new();
    if known.is_empty() {
        whole = _the_answer_about_the_place(&text_, &line, memory);
    }
    if !whole.is_empty() {
        return Some(format!("{said}, then. {whole}"));
    }
    if !known.is_empty() {
        let ends = known.ends_with(['.', '!', '?']);
        let join = if ends { " " } else { "; " };
        let place = if ends { _cap(&who) } else { who.clone() };
        if shape_kind == "howmuch" {
            return Some(format!("I haven't got a figure for {said}, and a guessed one is worse than none - {known}{join}{place}."));
        }
        return Some(format!("{said}, then. {known}{join}{place}."));
    }
    let mut on = String::new();
    if let Some(turns_on) = A_TURNS_ON.get(&kind).filter(|t| !t.is_empty()) {
        if shape_kind != "words" {
            on = format!("{}.", _cap(turns_on));
        }
    }
    let reply = crate::compose::fallback_chain::py_format(&shape, &[("subject", &said), ("on", &on), ("who", &who)]);
    Some(text::strip(&SPACES.sub(" ", &reply).replace(" .", ".")).to_string())
}

