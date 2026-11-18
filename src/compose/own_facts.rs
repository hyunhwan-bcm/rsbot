//! Port of `bot/compose/own_facts.py`: a question about a fact of the person's own life, where
//! the bot has not been told it; questions about earlier chats and about the bot learning; and
//! a line of theirs said back when they ask what they said.
//!
//! Every table and compiled regex comes from the data bundle under its Python name.
#![allow(non_upper_case_globals, clippy::all)]

use crate::compose::fallback::{format_named, memory_value};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understanding::rx;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.own_facts.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.own_facts.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.own_facts.", stringify!($name))));)*
    };
}

regexes!(HOW_MANY, HOW_MANY_GOT, HOW_OLD, WHAT_MY, WHEN_MY, WHEN_AUX_MY, WHERE_MY, HOW_TO, ABOUT_BOT);
regexes!(A_HAND_ON_THEM, THE_OTHER_CHATS, OTHER_THAN_THIS, A_RECORD_OF_THEM, SAID_IN_A_PAST_CHAT, LEARNING_ASK, TAUGHT_BY_THIS, SAID_BEFORE);
regexes!(WHEN_I, PAY_DAY, NOT_A_NOUN_PHRASE, SAID_ASK, FREQUENCY_ASK, PROPERTY_ASK, A_RATE, HOW_OFTEN_MINE);
word_sets!(STOP_WORDS, OWN_WORDS, CHAT_WORDS, THEIR_OWN_WORD, ROTA_WORDS, ROTA_NOUNS, PAY_WORDS, GENERIC_WORDS);
pools!(HAVEN_T_SAID, AS_THE_QUESTION, NO_OTHER_CHATS, NO_TRAINING, ON_THE_ROTA, ON_THE_PAYSLIP, SAID_POOL, NOT_A_RECORD, NOT_A_RECORD_BUT);
static _THE_PLACE: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.compose.own_facts._THE_PLACE").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});

/// `from . import pick`: one of the options, never the reply just given.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, &memory_value(memory), seed_text)
}

/// `" ".join(text.lower().replace("’", "'").split()).strip(" ?.!,")`.
fn cleaned(text_: &str) -> String {
    text::squash(&text::lower(text_).replace('\u{2019}', "'")).trim_matches([' ', '?', '.', '!', ',']).to_string()
}

/// `re.findall(r"[a-z']+", s)`.
fn az_words(s: &str) -> Vec<String> {
    rx("[a-z']+").findall_str(s)
}

/// Python: bot/compose/own_facts.py:180 `_names_their_chats(clean)`
/// True when the line speaks of the person's own chats, not of the word by itself.
pub fn _names_their_chats(clean: &str) -> bool {
    let tokens = az_words(clean);
    for (index, word) in tokens.iter().enumerate() {
        if !CHAT_WORDS.contains(word) {
            continue;
        }
        if index > 0 && THEIR_OWN_WORD.contains(&tokens[index - 1]) {
            return true;
        }
        if index > 1 && THEIR_OWN_WORD.contains(&tokens[index - 2]) {
            return true;
        }
        if index > 1 && CHAT_WORDS.contains(&tokens[index - 2]) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/own_facts.py:196 `about_earlier_chats(text, memory)`
/// A reply for a question about the chats before this one, or None. (The skeleton's `()` return
/// could not carry the reply.)
pub fn about_earlier_chats(text_: &str, memory: &Memory) -> Option<String> {
    if text_.is_empty() {
        return None;
    }
    let clean = cleaned(text_);
    if clean.is_empty() {
        return None;
    }
    let names = _names_their_chats(&clean);
    if !names && !SAID_IN_A_PAST_CHAT.is_match(&clean) {
        return None;
    }
    if names {
        if !(THE_OTHER_CHATS.is_match(&clean) || A_RECORD_OF_THEM.is_match(&clean)) {
            return None;
        }
        if !(OTHER_THAN_THIS.is_match(&clean) || A_RECORD_OF_THEM.is_match(&clean) || A_HAND_ON_THEM.is_match(&clean)) {
            return None;
        }
    }
    if !rx(r"\b(?:you|your|u|show me)\b").is_match(&clean) && !A_RECORD_OF_THEM.is_match(&clean) {
        return None;
    }
    Some(pick(&NO_OTHER_CHATS, memory, text_))
}

/// Python: bot/compose/own_facts.py:222 `learns_from_the_chat(text, memory)`
/// A reply for "do you learn from this chat", "will you get better", or None. (The skeleton's
/// `()` return could not carry the reply.)
pub fn learns_from_the_chat(text_: &str, memory: &Memory) -> Option<String> {
    if text_.is_empty() {
        return None;
    }
    let clean = cleaned(text_);
    if clean.is_empty() || !LEARNING_ASK.is_match(&clean) {
        return None;
    }
    if !rx(r"\b(?:you|u)\b").is_match(&clean) {
        return None;
    }
    if !TAUGHT_BY_THIS.is_match(&clean) && !(_names_their_chats(&clean) && rx(r"\b(?:these|this|our|the)\b").is_match(&clean)) {
        return None;
    }
    if rx(r"\b(?:help|teach|show) me\b").is_match(&clean) || rx(r"\bhow (?:do|can|could) i\b").is_match(&clean) {
        return None;
    }
    Some(pick(&NO_TRAINING, memory, text_))
}

/// Python: bot/compose/own_facts.py:269 `_the_place(text)`
/// Which place settles a question about their own timetable ("rota", "payslip"), or None. (The
/// skeleton's `()` return could not carry it.)
pub fn _the_place(text_: &str) -> Option<String> {
    if PAY_DAY.match_(text_).is_some() {
        return Some("payslip".into());
    }
    for pattern in [&*WHEN_I, &*WHEN_MY, &*WHEN_AUX_MY] {
        let Some(found) = pattern.match_(text_) else { continue };
        let said = found.str(1);
        let tail: HashSet<String> = rx("[a-z]+").findall_str(said).into_iter().collect();
        if tail.iter().any(|w| PAY_WORDS.contains(w)) || (tail.contains("pay") && (tail.contains("in") || tail.contains("into"))) {
            return Some("payslip".into());
        }
        if tail.iter().any(|w| ROTA_WORDS.contains(w)) {
            if !tail.iter().any(|w| ROTA_NOUNS.contains(w)) {
                if rx(r"\bon[-\s]?call\b").is_match(said) {
                    return Some("rota".into());
                }
                let said = rx(r"\bwork(?:s|ing)?\s+(?:out|on)\b").sub("", said);
                if rx("[a-z]+").findall_str(&said).iter().any(|w| ROTA_WORDS.contains(w)) {
                    return Some("rota".into());
                }
                return None;
            }
            return Some("rota".into());
        }
        return None;
    }
    None
}

/// Python: bot/compose/own_facts.py:299 `_has_a_shape(text)`
/// True when the line is one of the question shapes below, whatever its tail is.
pub fn _has_a_shape(text_: &str) -> bool {
    if HOW_MANY.match_(text_).is_some() || HOW_MANY_GOT.match_(text_).is_some() || HOW_OLD.match_(text_).is_some() {
        return true;
    }
    [&*WHAT_MY, &*WHEN_MY, &*WHERE_MY].iter().any(|p| p.match_(text_).is_some())
}

/// Python: bot/compose/own_facts.py:306 `_as_clause(text)`
/// The question as the thing they have not said: "how many brothers you have".
pub fn _as_clause(text_: &str) -> Option<String> {
    if let Some(found) = HOW_MANY.match_(text_).or_else(|| HOW_MANY_GOT.match_(text_)) {
        return Some(format!("how {} {} you have", found.str(1), found.str(2)));
    }
    if HOW_OLD.match_(text_).is_some() {
        return Some("how old you are".into());
    }
    for (kind, pattern) in [("what", &*WHAT_MY), ("when", &*WHEN_MY), ("where", &*WHERE_MY)] {
        if let Some(found) = pattern.match_(text_) {
            let tail = text::strip(found.str(1));
            if rx(r"\bcalled$").is_match(tail) {
                return Some(format!("what your {tail}"));
            }
            return Some(match kind {
                "when" => format!("when your {tail} is"),
                "where" => format!("where your {tail} is"),
                _ => format!("what your {tail} is"),
            });
        }
    }
    None
}

/// Python: bot/compose/own_facts.py:327 `_as_question(clause)`
/// The same clause turned back into the question a person would ask.
pub fn _as_question(clause: &str) -> String {
    if clause.starts_with("how many") || clause.starts_with("how much") {
        return clause.replace(" you have", " do you have");
    }
    if clause.starts_with("how old you are") {
        return "how old are you".into();
    }
    for (head, ask) in [("what your ", "what is "), ("when your ", "when is "), ("where your ", "where is ")] {
        if let Some(tail) = clause.strip_prefix(head) {
            let tail = tail.strip_suffix(" is").unwrap_or(tail);
            return format!("{ask}your {tail}");
        }
    }
    clause.to_string()
}

/// Python: bot/compose/own_facts.py:349 `_the_thing(text)`
/// The word the question turns on, when it is a thing of their own life.
pub fn _the_thing(text_: &str) -> Option<String> {
    let words = rx("[a-z]+").findall_str(text_);
    if let Some(w) = words.iter().find(|w| OWN_WORDS.contains(*w)) {
        return Some(w.clone());
    }
    for word in &words {
        if word.ends_with('s') && OWN_WORDS.contains(&word[..word.len() - 1]) {
            return Some(word[..word.len() - 1].to_string());
        }
    }
    None
}

/// Python: bot/compose/own_facts.py:361 `_second_person(text)`
/// The person's own question with themselves in the second person, every word in its place.
pub fn _second_person(text_: &str) -> String {
    let out = rx(r"\b(whats|wheres|whens|hows)\b").sub_fn(text_, |m| {
        match m.str(1) {
            "whats" => "what's",
            "wheres" => "where's",
            "whens" => "when's",
            _ => "how's",
        }
        .to_string()
    });
    let out = rx(r"\bmy\b").sub("your", &out);
    let out = rx(r"\bdo i have\b").sub("do you have", &out);
    let out = rx(r"\bhave i got\b").sub("have you got", &out);
    rx(r"\bam i\b").sub("are you", &out)
}

/// Python: bot/compose/own_facts.py:373 `_capital(clause)`
pub fn _capital(clause: &str) -> String {
    format!("{}{}", text::upper(&text::slice(clause, 0, Some(1))), text::slice(clause, 1, None))
}

/// Python: bot/compose/own_facts.py:377 `answer_for(text, memory)`
/// A reply for a fact question about the person that nobody ever told the bot, or None. (The
/// skeleton's `()` return could not carry the reply.)
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    if text_.is_empty() {
        return None;
    }
    let clean = cleaned(text_);
    if clean.is_empty() || text::split_ref(&clean).len() > 10 {
        return None;
    }
    if ABOUT_BOT.is_match(&clean) || HOW_TO.is_match(&clean) {
        return None;
    }
    if let Some(place) = _the_place(&clean) {
        let pool = _THE_PLACE.iter().find(|(k, _)| *k == place).map(|(_, v)| v.clone()).unwrap_or_else(|| panic!("KeyError: {place:?}"));
        return Some(pick(&pool, memory, text_));
    }
    if SAID_BEFORE.is_match(&clean) {
        return None;
    }
    if _the_thing(&clean).is_none() && HOW_OLD.match_(&clean).is_none() {
        return None;
    }
    if !_has_a_shape(&clean) {
        return None;
    }
    let mut clause = _as_clause(&clean);
    if let Some(c) = &clause {
        let after = match c.find("your") {
            Some(i) => &c[i + 4..],
            None => c.as_str(),
        };
        if NOT_A_NOUN_PHRASE.is_match(after) {
            clause = None;
        }
    }
    let question = _capital(&_second_person(&clean));
    let Some(clause) = clause else {
        let line = pick(&AS_THE_QUESTION, memory, text_);
        return Some(format_named(&line, &[("Q", &question), ("q", &clean)]));
    };
    let pool: &[String] = if text::split_ref(&clause).len() > 4 { &HAVEN_T_SAID } else { &AS_THE_QUESTION };
    let line = pick(pool, memory, text_);
    Some(format_named(&line, &[("q", &clause), ("Q", &question)]))
}

/// Python: bot/compose/own_facts.py:428 `_their_own_lines(memory)`
/// The lines the person said, newest first, from the store's own record of the turns.
pub fn _their_own_lines(memory: &Memory) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut add = |said: &str, lines: &mut Vec<String>| {
        let said = text::strip(said);
        if !said.is_empty() && !seen.contains(&text::lower(said)) {
            seen.push(text::lower(said));
            lines.push(said.to_string());
        }
    };
    for turn in memory.turns.iter().rev() {
        add(&turn.user, &mut lines);
    }
    if let Some(topics) = memory.get("topics") {
        for topic in topics.items().iter().rev() {
            let said = topic.get("text").and_then(Value::as_str).unwrap_or("");
            add(said, &mut lines);
        }
    }
    lines
}

/// Python: bot/compose/own_facts.py:441 `_stem(word)`
/// Enough of a stem to see "walking" and "walk" as one subject, and no more.
pub fn _stem(word: &str) -> String {
    let word = text::lower(word);
    for tail in ["ing", "ed", "es", "s"] {
        if text::len(&word) > 4 && word.ends_with(tail) {
            return text::slice(&word, 0, Some(-(text::len(tail) as isize)));
        }
    }
    word
}

/// Python: bot/compose/own_facts.py:450 `said_back(text, memory)`
/// The line of theirs the ask is about, said back, or None when the line is not this ask. (The
/// skeleton's `()` return could not carry the reply.)
pub fn said_back(text_: &str, memory: &Memory) -> Option<String> {
    if text_.is_empty() {
        return None;
    }
    let clean = cleaned(text_);
    let Some(found) = SAID_ASK.search(&clean) else { return _the_one_rate_on_record(&clean, memory, text_) };
    let own_stem = _stemmed(&clean);
    let lines: Vec<String> = _their_own_lines(memory).into_iter().filter(|l| _stemmed(l) != own_stem).collect();
    if lines.is_empty() {
        return Some(pick(&NOT_A_RECORD, memory, text_));
    }
    if FREQUENCY_ASK.is_match(&clean) {
        for line in lines.iter().take(8) {
            if A_RATE.is_match(line) {
                return Some(format_named(&pick(&SAID_POOL, memory, text_), &[("said", line)]));
            }
        }
    }
    let asked: Vec<String> = az_words(&clean[found.end()..])
        .into_iter()
        .filter(|w| !STOP_WORDS.contains(w) && !GENERIC_WORDS.contains(&_stem(w)))
        .map(|w| _stem(&w))
        .collect();
    let (mut best, mut best_score) = (lines[0].clone(), 0);
    for line in lines.iter().take(8) {
        let words: HashSet<String> = az_words(line).iter().map(|w| _stem(w)).collect();
        let score = asked.iter().filter(|w| words.contains(*w)).count();
        if score > best_score {
            best = line.clone();
            best_score = score;
        }
    }
    if !asked.is_empty() && best_score == 0 {
        return Some(format_named(&pick(&NOT_A_RECORD_BUT, memory, text_), &[("said", &lines[0])]));
    }
    if PROPERTY_ASK.is_match(&clean) && !rx(r"\d").is_match(&best) {
        return Some(format_named(&pick(&NOT_A_RECORD_BUT, memory, text_), &[("said", &best)]));
    }
    Some(format_named(&pick(&SAID_POOL, memory, text_), &[("said", &best)]))
}

/// Python: bot/compose/own_facts.py:495 `_stemmed(line)`
pub fn _stemmed(line: &str) -> String {
    az_words(&text::lower(line)).iter().map(|w| _stem(w)).collect::<Vec<_>>().join(" ")
}

/// Python: bot/compose/own_facts.py:499 `_the_one_rate_on_record(clean, memory, text)`
/// The one rate the store holds, said back, for "how often do i go"; or None. (The skeleton's
/// `()` return could not carry the reply.)
pub fn _the_one_rate_on_record(clean: &str, memory: &Memory, text_: &str) -> Option<String> {
    let found = HOW_OFTEN_MINE.match_(clean)?;
    let rates: Vec<&Value> = memory.get("rates").map(|r| r.items().iter().filter(|rate| rate.get("said").is_some_and(Value::truthy)).collect()).unwrap_or_default();
    if rates.len() != 1 {
        return None;
    }
    let rate = rates[0];
    let verb = _stem(found.str(1));
    let what = match rate.get("what").filter(|w| w.truthy()) {
        Some(what) => {
            let flat = rx("[^a-z]").sub(" ", &text::lower(what.as_str().unwrap_or("")));
            let first = text::split_ref(&flat).first().copied().unwrap_or_else(|| panic!("IndexError: list index out of range"));
            _stem(first)
        }
        None => String::new(),
    };
    if !GENERIC_WORDS.contains(&verb) || verb == what {
        return None;
    }
    let said = rate.get("said").and_then(Value::as_str).unwrap_or("").to_string();
    Some(format_named(&pick(&SAID_POOL, memory, text_), &[("said", &said)]))
}
