//! Port of `bot/memory/carry.py`: a condition added to a choice ("but i work long hours" after
//! a question about a cat), a verdict asked about the chat ("is that fair"), a quality asked of
//! the dish in hand ("something cold") and a limit on it ("and no dairy") are answered about the
//! thing in hand, by asking the knowledge base the question the person would have made of it.

use super::pyv;
use super::topics;
use crate::data::{self, Value};
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.carry.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static CONDITION: LazyLock<&'static Re> = LazyLock::new(|| rx("CONDITION"));
static MINE: LazyLock<&'static Re> = LazyLock::new(|| rx("MINE"));
static A_PLAN: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PLAN"));
static A_JOB: LazyLock<&'static Re> = LazyLock::new(|| rx("A_JOB"));
static DECIDING: LazyLock<&'static Re> = LazyLock::new(|| rx("DECIDING"));
static A_STATE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_STATE"));
static A_DATE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_DATE"));
static A_FIGURE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_FIGURE"));
static ASKS: LazyLock<&'static Re> = LazyLock::new(|| rx("ASKS"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_THING"));
static A_NAMED_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("A_NAMED_THING"));
static THE_MOST_THINGS_A_VERDICT_TRIES: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.carry.THE_MOST_THINGS_A_VERDICT_TRIES").as_int().unwrap() as usize);
static VERDICT: LazyLock<&'static Re> = LazyLock::new(|| rx("VERDICT"));
static VERDICT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("VERDICT_FILLER"));
static ASKS_A_VERDICT: LazyLock<&'static Re> = LazyLock::new(|| rx("ASKS_A_VERDICT"));
static NOT_A_THING_AS_A_MEASURE: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_THING_AS_A_MEASURE"));
static QUALITY_LEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("QUALITY_LEAD"));
static QUALITY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("QUALITY_WORDS"));
static A_PUDDING: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PUDDING"));
static A_MEAL: LazyLock<&'static Re> = LazyLock::new(|| rx("A_MEAL"));
static NOT_A_PUDDING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("NOT_A_PUDDING"));
static FREE_FROM_FOODS: LazyLock<HashMap<String, Vec<String>>> =
    LazyLock::new(|| data::get("bot.memory.carry.FREE_FROM_FOODS").str_list_map());
static FREE_FROM: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.carry.FREE_FROM"));
static CARRY_STOP: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("CARRY_STOP"));
static COOKS_AROUND_IT: LazyLock<&'static Re> = LazyLock::new(|| rx("COOKS_AROUND_IT"));

static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", re::U));
static SPACES: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s+", re::U));

/// `re.search(r"\b" + re.escape(word) + r"\b", said)`.
fn has_word(said: &str, word: &str) -> bool {
    Re::new(&format!(r"\b{}\b", re::escape(word)), re::U).is_match(said)
}

/// `topics.forms(word) or {word}`.
fn forms_of(word: &str) -> HashSet<String> {
    let forms = topics::forms(word);
    if forms.is_empty() { HashSet::from([word.to_string()]) } else { forms }
}

/// `answers.respond(line, memory)` behind `except Exception`.
fn answers_respond(line: &str, memory: &mut Memory) -> Option<Option<String>> {
    pyv::attempt(|| crate::modules::answers::respond(line, memory))
}

/// `answers.respond(line, _a_fresh_memory(memory))`, with the facts the reader wrote put back.
fn answered_fresh(line: &str, memory: &mut Memory) -> Option<Option<String>> {
    let mut fresh = _a_fresh_memory(memory);
    let reply = answers_respond(line, &mut fresh);
    pyv::shared_back(memory, &fresh);
    reply
}

/// Python: bot/memory/carry.py:96 `_flat(text)`
pub fn _flat(text_: &str) -> String {
    text::squash(text_)
}

/// Python: bot/memory/carry.py:100 `_the_thing_in_hand(text, memory)`
/// The subject of the chat before this line: the newest topic that is not this line itself,
/// as (its words, the thing it named).
pub fn _the_thing_in_hand(text_: &str, memory: &Memory) -> Option<(String, String)> {
    let flat = _flat(text_).to_lowercase();
    let said = text::rstrip_chars(&flat, ".");
    let mut seen: Vec<&Value> = Vec::new();
    if let Some(record) = memory.get("topic").filter(|r| r.is_dict()) {
        seen.push(record);
    }
    if let Some(Value::List(held)) = memory.get("topics") {
        seen.extend(held.iter().rev().filter(|other| other.is_dict()));
    }
    for other in seen {
        let words = _flat(&pyv::str_or_empty(pyv::get(other, "text")));
        if words.is_empty() || text::rstrip_chars(&words.to_lowercase(), ".") == said {
            continue;
        }
        if let Some(thing) = super::repair::_the_thing_the_last_line_named(&words) {
            return Some((words, thing));
        }
    }
    None
}

/// Python: bot/memory/carry.py:120 `_a_fresh_memory(memory)`
/// The facts the person stated, without the chat: the ask is read as a line of its own.
pub fn _a_fresh_memory(memory: &Memory) -> Memory {
    pyv::a_fresh_memory(memory)
}

/// Python: bot/memory/carry.py:136 `_the_answer_without_the_condition(subject_text, memory)`
/// What the knowledge base says to this line as it was said, on its own.
pub fn _the_answer_without_the_condition(subject_text: &str, memory: &mut Memory) -> String {
    answered_fresh(subject_text, memory).flatten().unwrap_or_default()
}

/// Python: bot/memory/carry.py:146 `_named_in(reply, thing, subject)`
/// Does the answer talk about the thing in hand? A plural counts as its own word.
pub fn _named_in(reply: &str, thing: &str, subject: &str) -> bool {
    let said = _flat(reply).to_lowercase();
    if said.is_empty() {
        return false;
    }
    for word in topics::keywords(subject) {
        let mut forms = forms_of(&word);
        if word == thing || topics::alike(&word, thing) {
            forms.extend(topics::forms(thing));
            forms.insert(thing.to_string());
        }
        if forms.iter().any(|form| has_word(&said, form)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/carry.py:165 `_already_on_the_screen(reply, memory)`
/// Is this the answer the person has already read in this chat (its opening, in any bot turn)?
pub fn _already_on_the_screen(reply: &str, memory: &Memory) -> bool {
    if memory.turns.is_empty() {
        return false;
    }
    let said = _flat(reply).to_lowercase();
    let opening = text::split_ref(&said).into_iter().take(10).collect::<Vec<_>>().join(" ");
    if opening.is_empty() {
        return false;
    }
    memory.turns.iter().any(|turn| _flat(&turn.bot).to_lowercase().contains(&opening))
}

/// Python: bot/memory/carry.py:206 `_a_verdict_on_the_other_thing(reply, condition, thing, subject)`
/// True when the answer weighs the thing in hand against a thing the condition named.
pub fn _a_verdict_on_the_other_thing(reply: &str, condition: &str, _thing: &str, subject: &str) -> bool {
    let said = _flat(reply).to_lowercase();
    if said.is_empty() {
        return false;
    }
    let held: Vec<String> = topics::keywords(subject).iter().map(|w| w.to_lowercase()).collect();
    for word in A_NAMED_THING.findall_str(&_flat(condition).to_lowercase()) {
        if NOT_A_THING.contains(&word) || held.iter().any(|one| topics::alike(&word, one)) {
            continue;
        }
        if has_word(&said, &word) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/carry.py:228 `_a_decision_in_the_chat(memory)`
/// True when what the chat is about is something the person is weighing up.
pub fn _a_decision_in_the_chat(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let mut said: Vec<String> =
        turns[turns.len().saturating_sub(6)..].iter().filter(|turn| !turn.user.is_empty()).map(|turn| turn.user.clone()).collect();
    if let Some(topic) = memory.get("topic").filter(|t| t.is_dict()) {
        let text_ = pyv::get(topic, "text");
        if text_.truthy() {
            said.push(pyv::str_of(text_));
        }
    }
    said.iter().any(|one| DECIDING.is_match(one))
}

/// Python: bot/memory/carry.py:269 `_a_verdict_asked(clean)`
/// True when the whole line is a verdict asked about the chat, not a thing of its own.
pub fn _a_verdict_asked(clean: &str) -> bool {
    let lowered = clean.to_lowercase();
    let low = text::strip_chars(&lowered, " .!");
    if low.is_empty() || !VERDICT.is_match(low) {
        return false;
    }
    if !clean.contains('?') && ASKS_A_VERDICT.match_(low).is_none() {
        return false;
    }
    LETTERS.findall_str(low).iter().all(|word| VERDICT_FILLER.contains(word) || VERDICT.is_match(word))
}

/// Python: bot/memory/carry.py:283 `the_verdict_in_the_chat(text, memory)`
/// The verdict on the thing in hand, asked with what the person has since said, or None.
pub fn the_verdict_in_the_chat(text_: &str, memory: &mut Memory) -> Option<String> {
    let line = _flat(text_);
    if text::split_ref(&line).len() > 8 || !_a_verdict_asked(&line) {
        return None;
    }
    for (subject_text, thing, since) in _the_things_the_verdict_is_about(memory, &line) {
        if let Some(reply) = _the_verdict_answer(memory, &line, &subject_text, &thing, &since) {
            return Some(reply);
        }
    }
    None
}

/// Python: bot/memory/carry.py:302 `_the_verdict_answer(memory, line, subject_text, thing, since)`
/// The answer to a verdict about one thing the chat named, or None.
pub fn _the_verdict_answer(memory: &mut Memory, line: &str, subject_text: &str, thing: &str, since: &[String]) -> Option<String> {
    if subject_text.is_empty() || thing.is_empty() {
        return None;
    }
    // The facts they added come first and the thing in hand last.
    let mut parts: Vec<&str> = since.iter().map(String::as_str).filter(|one| !one.is_empty()).collect();
    parts.push(subject_text);
    parts.push(line);
    let asked = parts.join(", ");
    let reply = answered_fresh(&asked, memory)??;
    if text::strip(&reply).is_empty() {
        return None;
    }
    if !_named_in(&reply, thing, subject_text) {
        return None;
    }
    if _already_on_the_screen(&reply, memory) {
        // Read earlier in the chat (not just now): it goes back framed as the repeat it is.
        if !_the_answer_just_given(&reply, memory) {
            if let Some(framed) = pyv::attempt(|| super::again::already_had(&reply, memory)).flatten() {
                if !framed.is_empty() {
                    return Some(framed);
                }
            }
        }
        return None;
    }
    Some(text::strip(&reply).to_string())
}

/// Python: bot/memory/carry.py:339 `_the_answer_just_given(reply, memory)`
/// True when this reply is the last answer word for word, or the opening of it.
pub fn _the_answer_just_given(reply: &str, memory: &Memory) -> bool {
    let Some(turn) = memory.turns.last() else { return false };
    let mine = _flat(reply).to_lowercase();
    let last = _flat(&turn.bot).to_lowercase();
    if mine.is_empty() || last.is_empty() {
        return false;
    }
    mine == last || last.starts_with(&mine) || mine.starts_with(&text::split_ref(&last).into_iter().take(10).collect::<Vec<_>>().join(" "))
}

/// Python: bot/memory/carry.py:351 `_the_things_the_verdict_is_about(memory, line)`
/// The things the chat named that the verdict could be about, newest first: (the line that
/// named it, the thing, the lines said since).
pub fn _the_things_the_verdict_is_about(memory: &Memory, line: &str) -> Vec<(String, String, Vec<String>)> {
    let flat = _flat(line).to_lowercase();
    let said = text::rstrip_chars(&flat, ".");
    let mut order: Vec<String> = Vec::new();
    for other in _the_lines_of_the_chat(memory) {
        let words = _flat(&other);
        let low = words.to_lowercase();
        if words.is_empty() || text::rstrip_chars(&low, ".") == said {
            continue;
        }
        if order.last().is_some_and(|last| text::rstrip_chars(&last.to_lowercase(), ".") == text::rstrip_chars(&low, ".")) {
            continue;
        }
        order.push(words);
    }
    let mut found = Vec::new();
    for index in (0..order.len()).rev() {
        if let Some(thing) = _the_thing_they_named(&order[index]) {
            found.push((order[index].clone(), thing, order[index + 1..].to_vec()));
            if found.len() >= *THE_MOST_THINGS_A_VERDICT_TRIES {
                break;
            }
        }
    }
    found
}

/// Python: bot/memory/carry.py:383 `_the_thing_they_named(line)`
/// The thing the line names with a word in front of it: "a dog", "my dog", "the kitchen".
pub fn _the_thing_they_named(line: &str) -> Option<String> {
    A_NAMED_THING
        .findall_str(&_flat(line).to_lowercase())
        .into_iter()
        .filter(|word| !NOT_A_THING.contains(word) && !NOT_A_THING_AS_A_MEASURE.contains(word))
        .last()
}

/// Python: bot/memory/carry.py:404 `_the_lines_of_the_chat(memory)`
/// Every line the person said, oldest first.
pub fn _the_lines_of_the_chat(memory: &Memory) -> Vec<String> {
    memory.turns.iter().map(|turn| _flat(&turn.user)).filter(|one| !one.is_empty()).collect()
}

/// A byte offset into `low` (the lowered line) as the same character's offset in `line`.
fn same_place(line: &str, low: &str, at: usize) -> usize {
    text::byte_at(line, text::char_index(low, at))
}

/// Python: bot/memory/carry.py:418 `the_subject_in_the_condition(text, memory)`
/// The answer to the choice with the person's own condition in it, or None.
pub fn the_subject_in_the_condition(text_: &str, memory: &mut Memory) -> Option<String> {
    let line = _flat(text_);
    if line.is_empty() || ASKS.is_match(&line) {
        return None;
    }
    let low = line.to_lowercase();
    let end = match CONDITION.match_(&low) {
        Some(found) => found.end(),
        None => {
            let mine = MINE.match_(&low)?;
            if A_FIGURE.is_match(&low) {
                return None;
            }
            if A_STATE.match_(&line).is_some() || A_DATE.is_match(&line) {
                return None; // how they are, or a date, is the store's own answer
            }
            if A_PLAN.match_(text::strip(&low[mine.end()..])).is_some() {
                return None; // what they are doing next, not a condition on the choice
            }
            mine.end()
        }
    };
    if !_a_decision_in_the_chat(memory) {
        return None;
    }
    let (subject_text, thing) = _the_thing_in_hand(text_, memory)?;
    // The words of the condition first and the thing in hand last.
    let condition = text::strip_chars(&line[same_place(&line, &low, end)..], " ,.?!").to_string();
    if condition.is_empty() {
        return None;
    }
    if A_JOB.is_match(&condition) {
        if let Some(job_thing) = super::repair::_the_thing_the_last_line_named(&condition) {
            if job_thing != _flat(&thing) {
                return None; // a job on another thing is a new subject
            }
        }
    }
    let ask = format!("{condition}, {subject_text}");
    let reply = answered_fresh(&ask, memory)??;
    if text::strip(&reply).is_empty() {
        return None;
    }
    let held = topics::keywords(&condition);
    let own = _the_answer_without_the_condition(&line, memory);
    if !own.is_empty() && !held.is_empty() && held[held.len() - 1..].iter().any(|word| has_word(&own.to_lowercase(), word)) {
        // The limit has an answer of its own in the knowledge base, about the limit.
        return None;
    }
    if !_named_in(&reply, &thing, &subject_text) {
        return None;
    }
    if _a_verdict_on_the_other_thing(&reply, &condition, &thing, &subject_text) {
        return None;
    }
    if _already_on_the_screen(&reply, memory) {
        return None;
    }
    Some(text::strip(&reply).to_string())
}

/// Python: bot/memory/carry.py:520 `_the_quality(line)`
/// The quality the line asks for, as "something cold", or None.
pub fn _the_quality(line: &str) -> Option<String> {
    let spaced = SPACES.sub(" ", line).to_lowercase();
    let mut said = text::strip_chars(&spaced, " ,.!?").to_string();
    if said.is_empty() {
        return None;
    }
    if let Some(lead) = QUALITY_LEAD.match_(&said) {
        said = text::strip_chars(&said[lead.end()..], " ,.!?").to_string();
    }
    if !text::starts_with_any(&said, &["something ", "anything ", "summat "]) {
        return None;
    }
    let words = text::split_ref(&said);
    if words.len() < 2 || words.len() > 4 {
        return None;
    }
    if words[1..].iter().any(|word| !QUALITY_WORDS.contains(*word)) {
        return None;
    }
    Some(format!("something {}", words[1..].join(" ")))
}

/// Python: bot/memory/carry.py:546 `_the_course(memory, quality='')`
/// "pudding" when the chat is about the sweet, "dinner" when it is about the meal, else None.
pub fn _the_course(memory: &Memory, quality: &str) -> Option<String> {
    let turns = &memory.turns;
    let mut lines: Vec<String> = turns[turns.len().saturating_sub(4)..].iter().map(|turn| turn.user.clone()).collect();
    if let Some(record) = memory.get("topic").filter(|r| r.is_dict()) {
        lines.push(pyv::str_or_empty(pyv::get(record, "text")));
    }
    if lines.is_empty() || !lines.iter().any(|line| !text::strip(line).is_empty()) {
        return None;
    }
    let said = lines.join(" ");
    let words: HashSet<&str> = text::split_ref(quality).into_iter().collect();
    if A_PUDDING.is_match(&said) && !words.iter().any(|w| NOT_A_PUDDING.contains(*w)) {
        return Some("pudding".into());
    }
    if A_MEAL.is_match(&said) {
        return Some("dinner".into());
    }
    None
}

/// Python: bot/memory/carry.py:569 `_is_a_hedge(reply)`
pub fn _is_a_hedge(reply: &str) -> bool {
    pyv::attempt(|| crate::compose::fallback::is_a_hedge(reply)).unwrap_or(false)
}

/// Python: bot/memory/carry.py:577 `the_quality_in_the_chat(text, memory)`
/// The dish that answers the quality the person asked for, or None.
pub fn the_quality_in_the_chat(text_: &str, memory: &mut Memory) -> Option<String> {
    let line = _flat(text_);
    if line.is_empty() {
        return None;
    }
    let quality = _the_quality(&line)?;
    let course = _the_course(memory, &quality)?;
    // The knowledge base's own reading of the line, on a copy, is the better answer when it has one.
    let mut probe = memory.clone();
    let direct = answers_respond(&line, &mut probe).flatten();
    if direct.as_ref().is_some_and(|d| !d.is_empty() && !_is_a_hedge(d)) {
        return None;
    }
    let ask = format!("{quality} for {course}");
    let reply = answered_fresh(&ask, memory)??;
    if text::strip(&reply).is_empty() || _is_a_hedge(&reply) || _already_on_the_screen(&reply, memory) {
        return None;
    }
    Some(text::strip(&reply).to_string())
}

/// Python: bot/memory/carry.py:628 `_the_last_ask(lines, said)`
/// The last line of the person's own that asked for something, near enough to still be on.
pub fn _the_last_ask(lines: &[String], said: &str) -> Option<String> {
    for line in lines.iter().rev() {
        let words = text::strip(&_flat(line)).to_string();
        if words.is_empty() || text::rstrip_chars(&words.to_lowercase(), ".") == said {
            continue;
        }
        if text::split_ref(&words).len() > 14 {
            return None;
        }
        return Some(words);
    }
    None
}

/// Python: bot/memory/carry.py:673 `_speaks_of(reply, things)`
/// True when the answer carries the thing that was ruled out, by its name or as a food.
pub fn _speaks_of(reply: &str, things: &[String]) -> bool {
    let said = _flat(reply).to_lowercase();
    for thing in things {
        let mut forms: HashSet<String> = topics::forms(thing);
        forms.insert(thing.clone());
        for food in FREE_FROM_FOODS.get(&thing.to_lowercase()).into_iter().flatten() {
            forms.insert(food.clone());
            forms.extend(topics::forms(food));
        }
        if forms.iter().any(|form| !form.is_empty() && has_word(&said, form)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/carry.py:696 `_a_free_from(things)`
pub fn _a_free_from(things: &[String]) -> bool {
    things.iter().all(|word| FREE_FROM.iter().any(|known| topics::alike(word, known)))
}

/// Python: bot/memory/carry.py:708 `_the_subject_named(reply, base)`
/// Does the answer name the thing the ask was about?
pub fn _the_subject_named(reply: &str, base: &str) -> bool {
    let said = _flat(reply).to_lowercase();
    for word in topics::keywords(base) {
        if CARRY_STOP.contains(&word) {
            continue;
        }
        let mut forms = topics::forms(&word);
        forms.insert(word.clone());
        if forms.iter().any(|form| !form.is_empty() && has_word(&said, form)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/carry.py:720 `the_limit_in_the_chat(text, memory)`
/// The dish or list the person still wants, with what they ruled out, or None.
pub fn the_limit_in_the_chat(text_: &str, memory: &mut Memory) -> Option<String> {
    let said = text::strip(&_flat(text_)).to_string();
    let read = memory.understood.as_ref().filter(|read| read.constraint)?;
    let things: Vec<String> = read.excludes.iter().map(|t| text::strip(t).to_string()).filter(|t| !t.is_empty()).collect();
    if things.is_empty() || text::split_ref(&said).len() > 5 || !_a_free_from(&things) {
        return None; // a limit and nothing else is the carry's
    }
    let turns = &memory.turns;
    let lines: Vec<String> = turns[turns.len().saturating_sub(3)..].iter().map(|turn| turn.user.clone()).collect();
    let base = _the_last_ask(&lines, &said.to_lowercase())?;
    // The ask they made with the limit in it, and the same ask as a cook would put it.
    let mut asks = vec![format!("{base}, no {}", things.join(" or "))];
    let subject: Vec<String> = topics::keywords(&base).into_iter().filter(|w| !CARRY_STOP.contains(w)).collect();
    if !subject.is_empty() {
        asks.push(format!(
            "what can i cook with {}, no {}",
            subject.iter().take(2).cloned().collect::<Vec<_>>().join(" "),
            things.join(" or ")
        ));
    }
    for ask in asks {
        let Some(Some(reply)) = answered_fresh(&ask, memory) else { continue };
        if text::strip(&reply).is_empty() {
            continue;
        }
        if _is_a_hedge(&reply) || _already_on_the_screen(&reply, memory) {
            continue;
        }
        if _speaks_of(&reply, &things) {
            // Unless what it carries is the thing being cooked round.
            if !_cooks_around_it(&reply) {
                continue;
            }
        } else if !_the_subject_named(&reply, &base) {
            continue;
        }
        return Some(text::strip(&reply).to_string());
    }
    None
}

/// Python: bot/memory/carry.py:781 `_cooks_around_it(reply)`
/// True when the answer names the thing ruled out in order to say what goes in its place.
pub fn _cooks_around_it(reply: &str) -> bool {
    COOKS_AROUND_IT.is_match(&_flat(reply).to_lowercase())
}
