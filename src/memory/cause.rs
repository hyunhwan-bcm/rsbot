//! Port of `bot/memory/cause.py`: a bare thing, offered as the reason for the state they just
//! reported. "im knackered" / "why" / "the baby": the two words are the answer to the bot's
//! own question, so the line is joined to the state ("im knackered, the baby") and answered as
//! a line of its own.

use super::pyv::{self, Truthy};
use super::topics;
use crate::data::{self, Value};
use crate::py::random;
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understanding::understand;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.cause.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_of(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}

static STATE: LazyLock<&'static Re> = LazyLock::new(|| rx("STATE"));
static STATE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("STATE_WORDS"));
static STATE_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("STATE_PHRASES"));
static A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("A_THING"));
static A_GUESS_IN_FRONT: LazyLock<&'static Re> = LazyLock::new(|| rx("A_GUESS_IN_FRONT"));
static NOT_A_REASON: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("NOT_A_REASON"));
static NOT_A_REASON_PHRASES: LazyLock<Vec<String>> = LazyLock::new(|| strs("NOT_A_REASON_PHRASES"));
static A_SWAP_TO_YOURS: LazyLock<Vec<(&'static Re, String)>> = LazyLock::new(|| {
    data::get("bot.memory.cause.A_SWAP_TO_YOURS")
        .items()
        .iter()
        .map(|pair| (pair.items()[0].as_re().unwrap(), pair.items()[1].str().to_string()))
        .collect()
});
static NO_IDEA: LazyLock<&'static Re> = LazyLock::new(|| rx("NO_IDEA"));
static A_BODY_OR_A_PERSON: LazyLock<&'static Re> = LazyLock::new(|| rx("A_BODY_OR_A_PERSON"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("NOT_A_THING"));
static A_NOD: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("A_NOD"));
static NO_REASON: LazyLock<&'static Re> = LazyLock::new(|| rx("NO_REASON"));
static A_VERB: LazyLock<&'static Re> = LazyLock::new(|| rx("A_VERB"));
static A_QUESTION_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("A_QUESTION_WORD"));
static REPEATED_ON_PURPOSE: LazyLock<&'static Re> = LazyLock::new(|| rx("REPEATED_ON_PURPOSE"));
static NOTHING_SAID: LazyLock<&'static Re> = LazyLock::new(|| rx("NOTHING_SAID"));
static A_HESITATION: LazyLock<&'static Re> = LazyLock::new(|| rx("A_HESITATION"));
static A_REASON_TOOK: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_REASON_TOOK"));
static A_REASON_NOBODY_HAS: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_REASON_NOBODY_HAS"));
static A_STOCK_LOW_POOLS: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_STOCK_LOW_POOLS"));
static CHAT_REPLIES: LazyLock<&'static Value> = LazyLock::new(|| data::get("bot.modules.chat.REPLIES"));

static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", re::U));

/// The little words a thing is said with, which are not the thing.
const DETERMINERS: [&str; 10] = ["the", "my", "our", "his", "her", "their", "a", "an", "this", "that"];
const SMALL_WORDS: [&str; 16] = [
    "the", "my", "our", "his", "her", "their", "a", "an", "this", "that", "just", "only", "mostly", "mainly", "probably",
    "definitely",
];

/// `re.search(r"\b" + re.escape(word) + r"\b", line)`.
fn has_word(line: &str, word: &str) -> bool {
    Re::new(&format!(r"\b{}\b", re::escape(word)), re::U).is_match(line)
}

/// Python: bot/memory/cause.py:228 `_flat(text)`
pub fn _flat(text_: &str) -> String {
    text::squash(&text_.to_lowercase().replace('\u{2019}', "'"))
}

/// Python: bot/memory/cause.py:232 `_turns(memory)`
pub fn _turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// The last `n` turns (`_turns(memory)[-n:]`).
fn last_turns(memory: &Memory, n: usize) -> &[Turn] {
    let turns = _turns(memory);
    &turns[turns.len().saturating_sub(n)..]
}

/// Python: bot/memory/cause.py:236 `_the_state_they_reported(memory, back=4)`
/// The state line, said as it was said, and the word for the state, from the last few turns
/// (newest first).
pub fn _the_state_they_reported(memory: &Memory, back: usize) -> Option<(String, String)> {
    for turn in last_turns(memory, back).iter().rev() {
        let line = _flat(&turn.user);
        if line.is_empty() {
            continue;
        }
        for phrase in STATE_PHRASES.iter() {
            if has_word(&line, phrase) {
                return Some((line, phrase.clone()));
            }
        }
        if let Some(found) = STATE.search(&line) {
            let word = text::strip(found.str(1));
            if STATE_WORDS.contains(word) {
                return Some((line.clone(), word.to_string()));
            }
        }
    }
    None
}

/// Python: bot/memory/cause.py:251 `_the_bot_just_spoke(memory)`
/// The bot's line on the turn before, or "" when it said nothing.
pub fn _the_bot_just_spoke(memory: &Memory) -> String {
    last_turns(memory, 2).iter().rev().map(|turn| text::strip(&turn.bot)).find(|said| !said.is_empty()).unwrap_or("").to_string()
}

/// Python: bot/memory/cause.py:260 `_already_has_it(said, thing)`
/// The bot's own line already names the thing: it has been understood.
pub fn _already_has_it(said: &str, thing: &str) -> bool {
    if said.is_empty() {
        return false;
    }
    let line = format!(" {} ", _flat(said));
    let flat = _flat(thing);
    let words: Vec<&str> = text::split_ref(&flat).into_iter().filter(|w| !SMALL_WORDS.contains(w)).collect();
    !words.is_empty() && words.iter().all(|w| has_word(&line, w))
}

/// Python: bot/memory/cause.py:286 `_only_a_nod(text)`
/// True when the line is nothing but a nod: the store's own ack reading, or every word of it a
/// nod word ("good stuff", "thats grand").
pub fn _only_a_nod(text_: &str) -> bool {
    if topics::is_ack(text_) {
        return true;
    }
    let flat = _flat(text_).to_lowercase();
    let words: Vec<&str> = text::split_ref(&flat).into_iter().map(|w| text::strip_chars(w, ".,!?'")).collect();
    !words.is_empty() && words.iter().filter(|w| !w.is_empty()).all(|w| A_NOD.contains(*w))
}

/// Python: bot/memory/cause.py:295 `_a_thing(text)`
/// The thing itself, said the way they said it, or None when the line is more than a thing.
pub fn _a_thing(text_: &str) -> Option<String> {
    let flat = _flat(text_);
    let mut low = text::strip_chars(&flat, " .!,").to_string();
    if low.is_empty() || low.ends_with('?') {
        return None;
    }
    // A hedge before the thing is not the thing: "dunno, the baby".
    if A_HESITATION.match_(&low).is_some() {
        low = text::strip_chars(&A_HESITATION.sub("", &low), " .,:;-").to_string();
        if low.is_empty() {
            return None;
        }
    }
    if A_QUESTION_WORD.match_(&low).is_some() {
        return None;
    }
    // A guess at the reason is still the reason: "maybe its work".
    let mut body = text::strip(&A_GUESS_IN_FRONT.sub("", &low)).to_string();
    if body.is_empty() {
        body = low.clone();
    }
    let again = text::strip(&A_GUESS_IN_FRONT.sub("", &body)).to_string();
    if !again.is_empty() {
        body = again;
    }
    let words = text::split_ref(&body);
    if words.len() > 4 || words.is_empty() {
        return None;
    }
    if NOT_A_THING.contains(&body) || NOT_A_THING.contains(&words.join(" ")) {
        return None;
    }
    // One word on its own cannot be a statement, so "work" is the reason.
    if words.len() > 1 && A_VERB.is_match(&body) {
        return None;
    }
    if A_THING.match_(&body).is_none() {
        return None;
    }
    if _only_a_nod(&body) {
        return None;
    }
    // A thing that is only a determiner ("the") is not a thing.
    const NOT_IT: [&str; 20] = [
        "the", "my", "our", "his", "her", "their", "this", "that", "a", "an", "it", "just", "only", "mostly", "mainly",
        "probably", "definitely", "honestly", "maybe", "i",
    ];
    if words.iter().all(|w| NOT_IT.contains(w) || *w == "think" || *w == "suppose") {
        return None;
    }
    Some(body)
}

/// Python: bot/memory/cause.py:345 `expand(text, memory)`
/// The full statement this bare thing is the reason for ("im knackered, the baby"), or None.
pub fn expand(text_: &str, memory: &Memory) -> Option<String> {
    let thing = _a_thing(text_)?;
    let spoke = _the_bot_just_spoke(memory);
    if spoke.is_empty() || _already_has_it(&spoke, &thing) {
        return None;
    }
    let (said, word) = _the_state_they_reported(memory, 4)?;
    if said.is_empty() || word.is_empty() {
        return None;
    }
    // A long line already carries the reason, so only the state word is kept from it.
    let state = if text::split_ref(&said).len() <= 8 { said } else { format!("im {word}") };
    Some(format!("{state}, {thing}"))
}

/// Python: bot/memory/cause.py:370 `_the_bot_would_only_shrug(line, memory)`
/// True when the composer's reply to these words on their own is a line that says nothing.
/// The composer is asked on a copy of the memory, with the generator put back afterwards.
pub fn _the_bot_would_only_shrug(line: &str, memory: &Memory) -> bool {
    let mut probe = memory.clone();
    let state = random::getstate();
    // Asked as the words on their own, not as this turn: no verdict pool.
    let said = pyv::attempt(|| crate::compose::fallback_chain::reply_for(line, &mut probe, false));
    random::setstate(state);
    match said {
        Some(said) => NOTHING_SAID.is_match(&said),
        None => false,
    }
}

/// `module.respond(line, memory)` for the module of that name.
fn respond_of(name: &str, line: &str, memory: &mut Memory) -> Option<String> {
    match name {
        "answers" => crate::modules::answers::respond(line, memory),
        "chat" => crate::modules::chat::respond(line, memory),
        other => panic!("ModuleNotFoundError: bot.modules.{other}"),
    }
}

/// `memory` with the reading of `line` in it, as a module's respond sees it.
fn read_into(line: &str, memory: &Memory) -> Memory {
    let mut reading_memory = memory.clone();
    reading_memory.understood = Some(understand(line, Some(&reading_memory)));
    reading_memory
}

/// Python: bot/memory/cause.py:398 `_asked(line, memory, names=("answers", "chat"))`
/// What the bot would say to this line, from the knowledge base first, then the chat.
pub fn _asked(line: &str, memory: &Memory, names: &[&str]) -> Option<String> {
    let mut reading_memory = read_into(line, memory);
    for name in names {
        let Some(said) = pyv::attempt(|| respond_of(name, line, &mut reading_memory)) else { continue };
        if let Some(said) = said {
            if !text::strip(&said).is_empty() && !NOTHING_SAID.is_match(&said) {
                return Some(text::strip(&said).to_string());
            }
        }
    }
    None
}

/// Python: bot/memory/cause.py:452 `_a_stock_low_line(said)`
/// True when the reply is one of the general lines a low mood is answered with.
pub fn _a_stock_low_line(said: &str) -> bool {
    let line = _flat(said);
    A_STOCK_LOW_POOLS.iter().any(|name| {
        CHAT_REPLIES.get(name).is_some_and(|pool| pool.items().iter().any(|one| one.as_str().is_some_and(|one| _flat(one) == line)))
    })
}

/// Python: bot/memory/cause.py:466 `_says_the_reason(said, thing)`
/// True when the reply names the reason at all.
pub fn _says_the_reason(said: &str, thing: &str) -> bool {
    let flat = _flat(thing);
    let words: Vec<&str> =
        text::split_ref(&flat).into_iter().filter(|w| !SMALL_WORDS[..14].contains(w)).collect();
    if words.is_empty() {
        return true;
    }
    let line = format!(" {} ", _flat(said));
    words.iter().any(|word| {
        let stem = if text::len(word) > 5 { text::slice(word, 0, Some(5)) } else { word.to_string() };
        Re::new(&format!(r"\b{}", re::escape(&stem)), re::U).is_match(&line)
    })
}

/// Python: bot/memory/cause.py:481 `_names_nothing(reason)`
/// True when the reason is a word for every reason at once ("everything", "life", "it all").
pub fn _names_nothing(reason: &str) -> bool {
    let flat = _flat(reason);
    let line = format!(" {flat} ");
    if NOT_A_REASON_PHRASES.iter().any(|phrase| line.contains(&format!(" {phrase} "))) {
        return true;
    }
    let said: Vec<&str> = text::split_ref(&flat).into_iter().filter(|w| !DETERMINERS.contains(w)).collect();
    !said.is_empty() && said.iter().all(|w| NOT_A_REASON.contains(*w))
}

/// Python: bot/memory/cause.py:491 `_the_persons_own_reason(reason)`
/// The reason said in the bot's mouth: theirs, so "my sleep" and "her job" are "your".
pub fn _the_persons_own_reason(reason: &str) -> String {
    let mut said = _flat(reason);
    for (pattern, replacement) in A_SWAP_TO_YOURS.iter() {
        said = pattern.sub(replacement, &said);
    }
    said
}

/// Python: bot/memory/cause.py:499 `_asks_for_the_rest(text)`
/// True when the line asks for another option or the rest of the answer, naming nothing.
pub fn _asks_for_the_rest(text_: &str) -> bool {
    pyv::attempt(|| super::another::names_nothing(text_)).unwrap_or(false)
}

/// The kept repeats (`memory.get("repeated")`) when they are a list.
fn repeated(memory: &Memory) -> Option<&Value> {
    memory.get("repeated").filter(|kept| matches!(kept, Value::List(_)))
}

/// Python: bot/memory/cause.py:508 `_a_reason_for_how_they_are(thing, memory)`
/// A warm line about the reason they named, from the reason itself, or None.
pub fn _a_reason_for_how_they_are(thing: &str, memory: &Memory) -> Option<String> {
    let flat = _flat(thing);
    let words: Vec<&str> = text::split_ref(&flat).into_iter().filter(|w| !SMALL_WORDS[..14].contains(w)).collect();
    if words.is_empty() || words.len() > 3 {
        return None;
    }
    // "everything" and "it all" would give a sentence that says nothing about anything.
    if _names_nothing(thing) {
        return None;
    }
    // A request for the rest of an answer is not a reason for how they feel.
    if _asks_for_the_rest(thing) {
        return None;
    }
    let body = pyv::pick(&A_REASON_TOOK, memory, thing);
    let reason = _the_persons_own_reason(thing);
    // Capitalised only where it opens the line.
    let said = body.replace("{reason}", &if body.starts_with("{reason}") { text::upper_first(&reason) } else { reason });
    let said = text::upper_first(&said);
    if repeated(memory).is_some_and(|kept| kept.contains_str(&said)) {
        return None;
    }
    if let Some(line) = last_turns(memory, 2).iter().rev().map(|turn| text::strip(&turn.bot)).find(|line| !line.is_empty()) {
        if _flat(line) == _flat(&said) {
            return None;
        }
    }
    Some(said)
}

/// Python: bot/memory/cause.py:543 `_without_what_was_just_said(said, last)`
/// The sentences of an answer the person has not just read, in order.
pub fn _without_what_was_just_said(said: &str, last: &str) -> String {
    let seen = _flat(last);
    let kept: Vec<String> = SENTENCE_END
        .split(text::strip(said))
        .into_iter()
        .filter(|sentence| !sentence.is_empty() && !seen.contains(&_flat(sentence)))
        .map(|sentence| text::strip(&sentence).to_string())
        .collect();
    kept.join(" ")
}

/// The bot's newest line in the last three turns, or "".
fn the_last_line(memory: &Memory) -> String {
    let mut last = String::new();
    for turn in last_turns(memory, 3).iter().rev() {
        last = text::strip(&turn.bot).to_string();
        if !last.is_empty() {
            break;
        }
    }
    last
}

/// Python: bot/memory/cause.py:553 `_a_new_answer(said, memory)`
/// The same line, unless the bot has just said it or is framing it as a repeat; an answer is
/// kept to the sentences the person has not just read.
pub fn _a_new_answer(said: Option<String>, memory: &Memory) -> Option<String> {
    let mut said = said.filter(|said| !said.is_empty())?;
    let last = the_last_line(memory);
    if !last.is_empty() && _flat(&said) == _flat(&last) {
        return None;
    }
    if !last.is_empty() {
        said = _without_what_was_just_said(&said, &last);
        if said.is_empty() {
            return None;
        }
    }
    // `said in (memory.get("repeated") or [])`: a str is searched for the words, a container
    // for the item.
    let kept = memory.get("repeated").filter(|kept| Value::truthy(kept));
    if kept.is_some_and(|kept| match kept {
        Value::Str(_) | Value::List(_) | Value::Tuple(_) | Value::Set(_) | Value::Dict(_) => kept.contains_str(&said),
        other => panic!("TypeError: argument of type {} is not iterable", pyv::repr(other)),
    }) {
        return None;
    }
    if REPEATED_ON_PURPOSE.match_(&said).is_some() {
        // "Same answer, and it covers this: ..." is not an answer to the reason they named.
        return None;
    }
    Some(said)
}

/// Python: bot/memory/cause.py:582 `_the_feeling_without_a_reason(memory)`
/// The state they reported, answered again with the missing reason said out loud; or, when
/// that answer was just given, a line about the reason they cannot give.
pub fn _the_feeling_without_a_reason(memory: &Memory) -> Option<String> {
    let (said, word) = _the_state_they_reported(memory, 4)?;
    if said.is_empty() || word.is_empty() {
        return None;
    }
    let state = if text::split_ref(&said).len() <= 8 { said } else { format!("im {word}") };
    let answer = _a_new_answer(_asked(&format!("{state}, and i don't know why"), memory, &["answers", "chat"]), memory);
    if answer.is_some() {
        return answer;
    }
    _the_reason_nobody_has(memory)
}

/// Python: bot/memory/cause.py:609 `_the_reason_nobody_has(memory)`
/// A line about there being no reason, not the one just said.
pub fn _the_reason_nobody_has(memory: &Memory) -> Option<String> {
    let mut said = pyv::pick(&A_REASON_NOBODY_HAS, memory, "no reason");
    if repeated(memory).is_some_and(|kept| kept.contains_str(&said)) {
        return None;
    }
    let last = the_last_line(memory);
    if !last.is_empty() && _flat(&said) == _flat(&last) {
        // Never the same sentence twice in a row: the next line in the pool instead.
        let pool = &*A_REASON_NOBODY_HAS;
        let here = pool.iter().position(|one| *one == said).unwrap_or(0);
        said = pool[(here + 1) % pool.len()].clone();
    }
    Some(said)
}

/// Python: bot/memory/cause.py:631 `_the_knowledge_base_answers_it(thing, memory)`
/// True when the knowledge base has an answer to the bare thing on its own.
pub fn _the_knowledge_base_answers_it(thing: &str, memory: &Memory) -> bool {
    pyv::attempt(|| {
        let mut reading_memory = read_into(thing, memory);
        crate::modules::answers::respond(thing, &mut reading_memory)
    })
    .flatten()
    .is_some_and(|said| !text::strip(&said).is_empty())
}

/// Python: bot/memory/cause.py:654 `_asks_about_the_bot(text, memory)`
/// True when the line is the question about the bot's last line, and the meant reader has
/// something to say back.
pub fn _asks_about_the_bot(text_: &str, memory: &Memory) -> bool {
    let Some(reading) = pyv::attempt(|| understand(text_, Some(memory))) else { return false };
    if !(reading.topic.as_deref() == Some("about-the-bot") && reading.kind == "question" && reading.entities.is_empty()) {
        return false;
    }
    let mut probe = memory.clone();
    pyv::attempt(|| super::meant::answer_for(text_, &mut probe).truthy()).unwrap_or(false)
}

/// Python: bot/memory/cause.py:679 `answer_for(text, memory)`
/// The reply to the reason they named, or None when there is nothing to answer it with.
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    if _asks_about_the_bot(text_, memory) {
        return None;
    }
    if NO_REASON.match_(&_flat(text_)).is_some() {
        return _the_feeling_without_a_reason(memory);
    }
    let reading = expand(text_, memory)?;
    let thing = _a_thing(text_);
    if let Some(thing) = &thing {
        // A reason the bot can answer on its own is a line of its own.
        if _the_knowledge_base_answers_it(thing, memory) {
            return None;
        }
        // The two words on their own already have an answer: the gap is only where the bot
        // would otherwise shrug.
        if !_the_bot_would_only_shrug(thing, memory) {
            return None;
        }
    }
    let said = _asked(&reading, memory, &["answers", "chat"]);
    // A shrug is still read as the reason it is, but there is no reason in it to write about.
    let thing = thing.unwrap_or_default();
    let plain = !thing.is_empty() && !NO_IDEA.is_match(&thing);
    let Some(said) = said else {
        // Nothing in the bot knows anything about the reason: answered as the reason itself.
        return if plain { _a_reason_for_how_they_are(&thing, memory) } else { None };
    };
    if plain && !A_BODY_OR_A_PERSON.is_match(&thing) && _a_stock_low_line(&said) && !_says_the_reason(&said, &thing) {
        // A line from a pool about a mood that would fit any low day says nothing about the
        // reason they have just named.
        if let Some(warmer) = _a_reason_for_how_they_are(&thing, memory) {
            return Some(warmer);
        }
    }
    _a_new_answer(Some(said), memory)
}
