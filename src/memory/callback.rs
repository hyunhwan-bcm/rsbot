//! Port of `bot/memory/callback.py`: a question about a person the chat already holds,
//! answered with what the chat holds.
//!
//!     my mum is in hospital / she had a fall / she is 82 / how is she doing
//!         -> "Your mum had a fall, you said. Any news since?"
//!
//! The reply is what the chat holds, said in one line: the person ("your mum"), what they said
//! about her ("had a fall"), her age if it was given, and the one thing a friend asks - whether
//! there is any word since. When the chat holds nobody, the module stands down.

use crate::data::{self, Value};
use crate::memory::narrow::{py_format, py_str_or_empty, re_cached};
use crate::py::re::{self as pyre, Re};
use crate::py::{random, text};
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! data_re {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.memory.callback.", stringify!($name)))); )*
    };
}

data_re!(THE_ASK, THE_NAMED_ASK, AN_AGE, IN_A_MEDICAL_EPISODE);

static THE_RELATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.callback.THE_RELATIONS"));
static MOST_WORDS_OF_NEWS: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.callback.MOST_WORDS_OF_NEWS").as_int().unwrap() as usize);
static EPISODE_TURNS: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.callback.EPISODE_TURNS").as_int().unwrap() as usize);
static HE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.HE"));
static SHE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.SHE"));

static MY_RELATION: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:my|our)\s+([a-z]{3,})\b", 0));

/// The verbs the news about a person reads on with ("had a fall", "is in hospital").
const NEWS_VERBS: [&str; 20] = [
    "is", "was", "had", "has", "have", "went", "got", "kept", "said", "wants", "wanted", "needs", "keeps", "lives",
    "does", "did", "came", "started", "turned", "seems",
];

/// `bot.memory.holds` as this file reads it: the turns before this one (`holds.turns`), what
/// the person said in them (`holds.lines`) and what the bot said (`holds.the_bot_lines`), each
/// tidied to one line, and whether there is any (`holds.there_is_one`).
mod earlier {
    use crate::py::text;
    use crate::state::Memory;

    pub fn there_is_one(memory: &Memory) -> bool {
        !memory.turns.is_empty()
    }
    pub fn lines(memory: &Memory) -> Vec<String> {
        memory.turns.iter().map(|t| text::squash(&t.user)).filter(|l| !l.is_empty()).collect()
    }
    pub fn the_bot_lines(memory: &Memory) -> Vec<String> {
        memory.turns.iter().map(|t| text::squash(&t.bot)).filter(|l| !l.is_empty()).collect()
    }
}

/// (relation, name, age) for a person the chat holds.
pub type Held = (String, String, Value);

/// `memory.others(memory)`: the store's list of other people, made an empty list when it is
/// missing (Python's `others` writes `memory["others"] = []`, and so does this).
///
/// CONTRACT: `crate::memory::others` returns `Vec<String>` in the generated contract, while
/// Python's list holds dicts, so the reading is done here on the memory itself.
fn the_others(memory: &mut Memory) -> Vec<Value> {
    match memory.get("others") {
        Some(Value::List(held)) => held.to_vec(),
        _ => {
            memory.set("others", Value::list());
            Vec::new()
        }
    }
}

/// Python: bot/memory/callback.py:49 `_the_person_held(memory)`
///
/// (relation, name, age) for the person the chat holds last, or None.
pub fn _the_person_held(memory: &mut Memory) -> Option<Held> {
    let held: Vec<Value> = the_others(memory).into_iter().filter(|one| one.is_dict()).collect();
    for one in held.iter().rev() {
        let relation = text::lower(text::strip(&py_str_or_empty(one.get("who"))));
        if relation.is_empty() {
            continue;
        }
        let name = text::strip(&py_str_or_empty(one.get("name"))).to_string();
        let age = one.get("age").cloned().unwrap_or(Value::None);
        return Some((relation, name, age));
    }
    // A person the chat only mentioned in passing is still the person a "how is she doing" asks
    // after: the relation the chat used is the fallback.
    let mut named: Vec<String> = Vec::new();
    for line in earlier::lines(memory) {
        let low = text::lower(&line);
        for found in MY_RELATION.finditer(&low) {
            let word = found.str(1).to_string();
            if THE_RELATIONS.contains(&word) && !named.contains(&word) {
                named.push(word);
            }
        }
    }
    if named.len() == 1 {
        return Some((named[0].clone(), String::new(), Value::None));
    }
    if let Some(last) = named.last() {
        // More than one person has been mentioned: only the last one the chat named is a reading.
        return Some((last.clone(), String::new(), Value::None));
    }
    None
}

/// Python: bot/memory/callback.py:79 `_the_news_about(memory, relation, name)`
///
/// What an earlier line said about them, with the person taken off the front.
pub fn _the_news_about(memory: &Memory, relation: &str, name: &str) -> String {
    let mut words: Vec<String> = vec![relation.to_string()];
    if !name.is_empty() {
        words.push(name.to_string());
    }
    words.extend(_the_pronouns(relation));
    for line in earlier::lines(memory).iter().rev() {
        let low = text::squash(&text::lower(line));
        if low.contains('?') || low.is_empty() {
            continue;
        }
        for word in &words {
            let pattern = format!(r"\b(?:my|our|the)?\s*{}\b", pyre::escape(word));
            let Some(found) = re_cached(&pattern, 0).search(&low) else { continue };
            let rest = text::strip_chars(&low[found.end()..], " ,.;:!-");
            if rest.is_empty() || text::split_ref(rest).len() > *MOST_WORDS_OF_NEWS {
                continue;
            }
            if AN_AGE.is_match(rest) {
                // "she is 82" is the age, and the reply says the age already: keep reading back.
                continue;
            }
            let first = text::split_ref(rest)[0];
            if NEWS_VERBS.contains(&first) {
                return text::rstrip_chars(rest, ".").to_string();
            }
        }
    }
    String::new()
}

/// Python: bot/memory/callback.py:109 `_the_pronouns(relation)`
pub fn _the_pronouns(relation: &str) -> Vec<String> {
    if HE.contains(relation) {
        return vec!["he".into()];
    }
    if SHE.contains(relation) {
        return vec!["she".into()];
    }
    vec!["they".into()]
}

/// Python: bot/memory/callback.py:117 `_the_ask(text, memory)`
///
/// (relation, name, age) for the person the line asks about, or None (Python's "" and None are
/// both "no ask here").
pub fn _the_ask(text_: &str, memory: &mut Memory) -> Option<Held> {
    let said = text::squash(&text::lower(text_));
    if let Some(found) = THE_ASK.match_(&said) {
        let held = _the_person_held(memory)?;
        // The pronoun is the chat's, not theirs: "how are they doing" is the one person held.
        let pointed = [found.name("who"), found.name("been"), found.name("better")]
            .into_iter()
            .flatten()
            .find(|p| !p.is_empty())
            .unwrap_or("");
        if pointed == "he" || pointed == "she" {
            if _the_pronouns(&held.0)[0] != pointed {
                return None;
            }
        }
        return Some(held);
    }
    if let Some(found) = THE_NAMED_ASK.match_(&said) {
        let word = [found.name("who"), found.name("news"), found.name("went")]
            .into_iter()
            .flatten()
            .find(|p| !p.is_empty())
            .unwrap_or("");
        let word = text::strip(word);
        if !THE_RELATIONS.contains(word) {
            return None;
        }
        let held = _the_person_held(memory)?;
        // "how is your mum" is about the mum the chat holds; a dad held instead is somebody else.
        if held.0 != word && held.0 != text::rstrip_chars(word, "s") {
            return None;
        }
        return Some(held);
    }
    None
}

/// Python: bot/memory/callback.py:167 `_a_medical_episode(memory)`
///
/// Whether a turn the chat holds is a stay, an operation or a move into care.
pub fn _a_medical_episode(memory: &Memory) -> bool {
    let lines = earlier::lines(memory);
    let from = lines.len().saturating_sub(*EPISODE_TURNS);
    lines[from..].iter().any(|line| IN_A_MEDICAL_EPISODE.is_match(line))
}

/// Python: bot/memory/callback.py:175 `reply_for(text, memory)`
///
/// What the chat holds about the person the line asks after, or "".
pub fn reply_for(text_: &str, memory: &mut Memory) -> String {
    if !earlier::there_is_one(memory) {
        return String::new();
    }
    if _a_medical_episode(memory) || IN_A_MEDICAL_EPISODE.is_match(text_) {
        return String::new();
    }
    let Some((relation, name, age)) = _the_ask(text_, memory) else { return String::new() };
    let who = if !name.is_empty() { text::capitalize(&name) } else { format!("your {relation}") };
    let mut news = _the_news_about(memory, &relation, &name);
    let age_said = py_str_or_empty(Some(&age));
    if age.truthy() && !news.is_empty() {
        let pattern = format!(r"(?:is|was)\s+{}(?:\s+years?\s+old)?", pyre::escape(&crate::memory::narrow::py_str(&age)));
        if re_cached(&pattern, 0).fullmatch(&news).is_some() {
            // "she is 82" is the age, which has a line of its own below.
            news = String::new();
        }
    }
    let since = ["Any word since then?", "Any news since?", "Has there been any change?", "Anything new since?"];
    let who_cap = text::capitalize(&who);
    if !news.is_empty() {
        let pool = [
            format!("You said {{who}} {news}. {{since}}"),
            format!("Last you said, {{who}} {news}. {{since}}"),
            format!("{{who_cap}} {news}, you said. {{since}}"),
        ];
        let lines: Vec<String> = pool
            .iter()
            .flat_map(|line| {
                since.iter().map(|ask| py_format(line, &[("who", &who), ("who_cap", &who_cap), ("since", ask)])).collect::<Vec<_>>()
            })
            .collect();
        return _choose(&lines, memory);
    }
    if age.truthy() {
        // The age is the one thing the chat holds: say it and ask something else, naming them.
        let pronoun = _the_pronouns(&relation)[0].clone();
        let pool = [
            format!("{{who_cap}} is {age_said} - what has {{pronoun}} been saying?"),
            format!("{{who_cap}} is {age_said}, and that is all I have. What is {{pronoun}} up to at the moment?"),
            format!("{{who_cap}} is {age_said} - has {{pronoun}} been in touch?"),
        ];
        let lines: Vec<String> =
            pool.iter().map(|line| py_format(line, &[("who_cap", &who_cap), ("pronoun", &pronoun)])).collect();
        return _choose(&lines, memory);
    }
    String::new()
}

/// Python: bot/memory/callback.py:217 `claims(text, memory)`
///
/// The lines this reading answers even when another module claims the turn as it comes.
pub fn claims(text_: &str, memory: &mut Memory) -> bool {
    !reply_for(text_, memory).is_empty()
}

/// Python: bot/memory/callback.py:228 `_choose(pool, memory)`
///
/// One line from the pool, not one this chat has already had, compared as it would read.
pub fn _choose(pool: &[String], memory: &Memory) -> String {
    let said = earlier::the_bot_lines(memory);
    let fresh: Vec<String> = pool.iter().filter(|line| !said.contains(line)).cloned().collect();
    if fresh.is_empty() { random::choice(pool) } else { random::choice(&fresh) }
}
