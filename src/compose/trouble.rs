//! Port of `bot/compose/trouble.py`: something has gone wrong and they are telling you about it.
//!
//! The shape of the trouble ('thing' for a broken thing, 'sore' for their own body, 'strain' for
//! a part of their life gone flat, 'vague' for a setback with nothing behind it yet) picks the
//! question a friend would ask next. Tables and patterns come from the bundle by Python name.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback_chain::caught;
use crate::data::{self, Value};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.trouble.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.trouble.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.trouble.", stringify!($name))));)*
    };
}

regexes!(
    THING_STATE_ONLY, THING, CANT_GET, BODY, FIGURATIVE, NUISANCE, ACUTE, FRACTURE, BREAKS, KEEPS_AT_IT,
    ASKS_TO_CHOOSE, FLAT, DEATHWISH, PLEA, MINE, OWNS, DURATION, A_DECLINE, STILL_FAILING, DETAIL, TRIGGER,
    _NOT_A_STRAIN_DETAIL, _A_SENTENCE, _A_LENGTH_OF_TIME, ALREADY_TRIED, IT, MINE_AGAIN, WHAT_IT_DOES,
    WANTING_OR_PRAISE,
);
word_sets!(
    OTHERS, GROWING, BODY_PART, BONE, NOT_A_THING, KEEPS_FINE, KEEPS_NOT_A_THING, KEEPS_NOT_THEIR_THING,
    KEEPS_THEIRS, KEEPS_POSSESSIVE, KEEPS_OF_A_HOUSE, NOT_THE_THING, FLAT_SUBJECT, JOINING, NOT_OWNED,
    DEAD_AS_VERY_AFTER, FAULT_FILLERS, _A_HEDGE, ANIMAL_MODIFIERS, NOT_A_MODIFIER,
);
pools!(
    OWNER_END, BROKEN, BROKEN_TIME, DECLINE, DECLINE_TIME, SORE, SORE_TIME, STRAIN, STRAIN_TIME, SORE_NEXT,
    BROKEN_NEXT, STRAIN_NEXT,
);
static ODD_MARK: LazyLock<String> = LazyLock::new(|| data::string("bot.compose.trouble.ODD_MARK"));

static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static AN_ACHE_SAID_OF_A_PART: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:dead|killing|kills|hurts?|banging|thumping|pounding|splitting|had\s+it|done\s+for)\b", 0)
});

/// `re.findall(r"[a-z']+", text)`.
fn az_words(text_: &str) -> Vec<String> {
    A_WORD.findall_str(text_)
}

/// `set(words) & names` is not empty.
fn any_in<S: AsRef<str>>(words: &[S], names: &HashSet<String>) -> bool {
    words.iter().any(|w| names.contains(w.as_ref()))
}

/// Python: bot/compose/trouble.py:459 `someone_elses(words)`
/// True when the line names someone else's body as the thing that hurts: a word that is in
/// OTHERS, or one of them with a possessive on the end ("mum's", "mums", "dogs"). Nothing else is
/// stripped, so "ribs" is not "rib" and "legs" is not "leg".
pub fn someone_elses<S: AsRef<str>>(words: &[S]) -> bool {
    for word in words {
        let word = word.as_ref();
        if OTHERS.contains(word) {
            return true;
        }
        for ending in OWNER_END.iter() {
            if word.ends_with(ending.as_str()) && text::len(word) > text::len(ending) + 2 {
                if OTHERS.contains(&word[..word.len() - ending.len()]) {
                    return true;
                }
            }
        }
    }
    false
}

/// Python: bot/compose/trouble.py:491 `_the_fault_hangs_on(text)`
/// The word each fault in the line hangs on: "flap" in "the cat flap is broken" (a frozenset in
/// Python; the words in the order found, each once).
pub fn _the_fault_hangs_on(text_: &str) -> Vec<String> {
    let mut heads: Vec<String> = Vec::new();
    let matches = THING.finditer(text_).into_iter().chain(CANT_GET.finditer(text_)).chain(FLAT.finditer(text_));
    for found in matches {
        let before = az_words(&text_[..found.start()]);
        for word in before.iter().rev() {
            if FAULT_FILLERS.contains(word) {
                continue;
            }
            if !heads.contains(word) {
                heads.push(word.clone());
            }
            break;
        }
    }
    heads
}

/// Python: bot/compose/trouble.py:572 `_only_the_decline(text)`
/// True when the line says a thing of theirs is failing and nothing more about it. The decline
/// phrase is taken out of the line first, and any fault left in what remains means the line is
/// not only the decline.
pub fn _only_the_decline(text_: &str) -> bool {
    let Some(found) = A_DECLINE.search(text_) else { return false };
    let rest = format!("{} {}", &text_[..found.start()], &text_[found.end()..]);
    !(THING.is_match(&rest) || CANT_GET.is_match(&rest) || BODY.is_match(&rest) || FLAT.is_match(&rest))
}

/// Python: bot/compose/trouble.py:665 `answers_a_strain_question(clean, memory=None)`
/// Whether this line is the answer to the strain question this module has just asked.
pub fn answers_a_strain_question(clean: &str, memory: &Memory) -> bool {
    if _shape_on_the_table(memory) != Some("strain") {
        return false;
    }
    _answers_a_strain_question(&_flat(clean), memory)
}

/// Python: bot/compose/trouble.py:697 `_answers_a_strain_question(text, memory=None)`
/// Whether a short line is the answer to "what's gone wrong, do you think?". A cause has no
/// vocabulary of its own, so the line is judged by what it is not - a question, a fault to
/// repair, somebody else's news, or long enough to be a new subject.
pub fn _answers_a_strain_question(text_: &str, memory: &Memory) -> bool {
    let words = text::split_ref(text_);
    if words.len() < 2 || words.len() > 9 || text_.contains('?') {
        return false;
    }
    if _NOT_A_STRAIN_DETAIL.is_match(text_) || _A_SENTENCE.is_match(text_) {
        return false;
    }
    if words.iter().all(|word| _A_HEDGE.contains(text::strip_chars(word, ",."))) {
        return false;
    }
    // A card that answers the line itself is worth more than another question about the strain.
    if _a_card_answers(text_) || _a_length_of_time(text_, memory) {
        return false;
    }
    reads_as_trouble(text_, None, "").is_none()
}

/// The lines of chat's length pools (`STANDING_REPLIES`, `STANDING_FEELING_REPLIES`), each a
/// dict of buckets of lines.
static LENGTH_LINES: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut lines = Vec::new();
    for name in ["bot.modules.chat.STANDING_REPLIES", "bot.modules.chat.STANDING_FEELING_REPLIES"] {
        let Some(buckets) = data::try_get(name).filter(|b| b.is_dict()) else { continue };
        for (_, bucket) in buckets.pairs() {
            lines.extend(bucket.items().iter().map(|line| line.str().to_string()));
        }
    }
    lines
});

/// Python: bot/compose/trouble.py:737 `_a_length_of_time(text, memory=None)`
/// Whether the line says how long it has been going on, and chat answers it as a length. Chat is
/// asked for its reply on a copy of the memory, and this stands aside only when the reply it
/// would give is one of the length pools.
pub fn _a_length_of_time(text_: &str, memory: &Memory) -> bool {
    if !_A_LENGTH_OF_TIME.is_match(text_) {
        return false;
    }
    let mut copy = memory.clone();
    // A chat module that cannot answer is simply no length reply.
    let Some(said) = caught(|| crate::modules::chat::respond(text_, &mut copy)) else { return false };
    let Some(said) = said.filter(|s| !s.is_empty()) else { return false };
    let said = text::lower(&said);
    LENGTH_LINES.iter().any(|line| !line.is_empty() && said.contains(&text::slice(&text::lower(line), 0, Some(60))))
}

/// Python: bot/compose/trouble.py:768 `_a_card_answers(text)`
/// Whether the knowledge base has an answer for this line on its own words.
pub fn _a_card_answers(text_: &str) -> bool {
    let understood = Understood { clean: text_.to_string(), words: text::split(text_), ..Default::default() };
    caught(|| crate::knowledge::lookup::find(&understood, None, None, false).is_some()).unwrap_or(false)
}

/// Python: bot/compose/trouble.py:781 `_flat(line)`
pub fn _flat(line: &str) -> String {
    text::squash(&text::lower(line))
}

/// Python: bot/compose/trouble.py:785 `_said_already(lines, memory)`
/// The lines the bot has not said yet in this conversation. A pool the conversation has used up
/// is still never empty: the whole pool is said once, and after that everything but the reply
/// just given.
pub fn _said_already(lines: &[String], memory: &Memory) -> Vec<String> {
    let said: Vec<String> = memory.turns.iter().map(|turn| _flat(&turn.bot)).filter(|line| !line.is_empty()).collect();
    if said.is_empty() {
        return lines.to_vec();
    }
    let fresh: Vec<String> = lines.iter().filter(|line| !said.contains(&_flat(line))).cloned().collect();
    if !fresh.is_empty() {
        return fresh;
    }
    let last = &said[said.len() - 1];
    let kept: Vec<String> = lines.iter().filter(|line| &_flat(line) != last).cloned().collect();
    if kept.is_empty() { lines.to_vec() } else { kept }
}

/// Python: bot/compose/trouble.py:820 `pool(shape, clean, memory=None)`
/// The replies for a shape, without the ones that would ask what the line just told us, and
/// without the questions this conversation has already asked. (`shape` is the reading's string.)
pub fn pool(shape: &Value, clean: &str, memory: &Memory) -> Vec<String> {
    let shape = shape.as_str().unwrap_or("");
    let text_ = format!(" {} ", text::lower(clean));
    if shape == "vague" {
        // A setback with nothing behind it yet: what happened is the only question, and the
        // wording carries the person's own thing (bot/memory/setback.py).
        return _said_already(&crate::memory::setback::pool(clean).strs(), memory);
    }
    let stated = DURATION.is_match(&text_);
    let mut lines: Vec<String> = if shape == "sore" {
        if stated { SORE_TIME.clone() } else { SORE.clone() }
    } else if shape == "strain" {
        if stated { STRAIN_TIME.clone() } else { STRAIN.clone() }
    } else if _only_the_decline(&text_) {
        let mut lines = if stated { DECLINE_TIME.clone() } else { DECLINE.clone() };
        if !STILL_FAILING.is_match(&text_) {
            // The line says the thing has stopped, not that it is failing: "what makes you think
            // it is on the way out?" asks about a doubt the person has already settled.
            lines.retain(|line| !line.contains("on the way out"));
        }
        lines
    } else if stated {
        BROKEN_TIME.clone()
    } else {
        BROKEN.clone()
    };
    if ALREADY_TRIED.is_match(&text_) {
        // "i tried turning it really hard and now its worse" says what they tried, so the one
        // question the pool cannot ask is "what have you tried so far".
        lines.retain(|line| !line.contains("tried so far"));
    }
    _said_already(&lines, memory)
}

/// Python: bot/compose/trouble.py:851 `_shape_on_the_table(memory)`
/// 'sore', 'thing' or 'strain' when our last reply was one of this module's questions, else None.
pub fn _shape_on_the_table(memory: &Memory) -> Option<&'static str> {
    let last = _flat(&memory.turns.last()?.bot);
    if last.is_empty() {
        return None;
    }
    let shapes: [(&'static str, Vec<&String>); 6] = [
        ("sore", SORE.iter().chain(SORE_TIME.iter()).collect()),
        ("thing", BROKEN.iter().chain(BROKEN_TIME.iter()).collect()),
        ("strain", STRAIN.iter().chain(STRAIN_TIME.iter()).collect()),
        ("sore", SORE_NEXT.iter().collect()),
        ("thing", BROKEN_NEXT.iter().collect()),
        ("strain", STRAIN_NEXT.iter().collect()),
    ];
    for (shape, lines) in shapes {
        if lines.iter().any(|line| last.contains(&_flat(line))) {
            return Some(shape);
        }
    }
    None
}

/// Python: bot/compose/trouble.py:873 `_points_at_it(text, memory)`
/// Whether a detail is about what we were already talking about: it points back at it ("its
/// worse in the morning") or names again a thing of theirs the last line named.
pub fn _points_at_it(text_: &str, memory: &Memory) -> bool {
    if IT.is_match(text_) {
        return true;
    }
    let said = memory.turns.last().map(|turn| _flat(&turn.user)).unwrap_or_default();
    if said.is_empty() {
        return true;
    }
    for found in MINE_AGAIN.finditer(text_) {
        let noun = found.str(1);
        if Re::new(&format!(r"\b{}\b", pyre::escape(noun)), 0).is_match(&said) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/trouble.py:893 `_shape_answered(memory)`
/// The shape of a fault we have just given a whole answer to, or None. Only the previous turn
/// counts, and only when our answer did not ask anything.
pub fn _shape_answered(memory: &Memory) -> Option<String> {
    let last_turn = memory.turns.last()?;
    let last = _flat(&last_turn.bot);
    if last.is_empty() || text::rstrip(&last).ends_with('?') {
        return None;
    }
    let said = _flat(&last_turn.user);
    if said.is_empty() {
        return None;
    }
    _shape(&said, None)
}

/// Python: bot/compose/trouble.py:921 `carried_on(clean, words=None, memory=None)`
/// The replies for a detail added to the complaint we are already on, or None. It needs both
/// halves: one of our own questions (or a whole answer to a fault) as the last reply, and a line
/// that is a detail about it rather than a new subject.
pub fn carried_on(clean: &str, words: &[String], memory: &Memory) -> Option<Vec<String>> {
    carried_on_with(clean, Some(words), memory)
}

/// `carried_on` with `words` None (read off the line) or given.
pub fn carried_on_with(clean: &str, words: Option<&[String]>, memory: &Memory) -> Option<Vec<String>> {
    let mut shape = _shape_on_the_table(memory).map(str::to_string);
    let mut fuzzy = false;
    if shape.is_none() {
        // A whole answer to a fault is a looser thread than one of this module's questions, so
        // the line has to say it is about that same thing, not just share a word with it.
        shape = _shape_answered(memory);
        fuzzy = shape.is_some();
    }
    let shape = shape?;
    let text_ = _flat(clean);
    if text_.is_empty() || text_.contains('?') {
        return None;
    }
    let words: Vec<String> = match words {
        Some(words) => words.to_vec(),
        None => az_words(&text_),
    };
    // Once this module has asked the strain question, a short line that is not a question, a
    // fault to repair or one of their people's news is that answer.
    let answers_the_strain = shape == "strain" && !fuzzy && _answers_a_strain_question(&text_, memory);
    if someone_elses(&words) && !_a_fault_of_a_thing(&text_) && !answers_the_strain {
        return None;
    }
    let padded = format!(" {text_} ");
    let says_what = answers_the_strain || DETAIL.is_match(&padded) || TRIGGER.is_match(&padded);
    if !(says_what || DURATION.is_match(&padded)) {
        return None;
    }
    if fuzzy && !_points_at_it(&padded, memory) {
        return None; // "the weather is better today" after a bad back is not that back
    }
    let nexts: &Vec<String> = match shape.as_str() {
        "sore" => &SORE_NEXT,
        "strain" => &STRAIN_NEXT,
        _ => &BROKEN_NEXT,
    };
    let mut lines = _said_already(nexts, memory);
    for line in pool(&Value::from(shape.as_str()), clean, memory) {
        // The first question of the thread is not a second question: once the person has said
        // what went wrong, "what's gone wrong, do you think?" in another wording is the same ask.
        if STRAIN.contains(&line) || lines.contains(&line) {
            continue;
        }
        lines.push(line);
    }
    // "What's it doing when it goes wrong?" is the first question of a complaint, not the second.
    lines.retain(|line| !WHAT_IT_DOES.is_match(line));
    if lines.is_empty() { None } else { Some(lines) }
}

/// Python: bot/compose/trouble.py:975 `reads_as_trouble(clean, words=None, repaired='')`
/// 'thing' for a broken thing, 'sore' for their own body, 'strain' or 'vague' for those shapes,
/// None for anything else. `repaired` is the same line with its wrong words put right, and it is
/// only consulted when the line as typed is not trouble.
pub fn reads_as_trouble(clean: &str, words: Option<&[String]>, repaired: &str) -> Option<String> {
    let text_ = _without_a_dead_that_only_means_very(&_without_wanting_or_praise(&_flat(clean)));
    if text_.is_empty() || text_.contains('?') {
        return None; // a question is not a fault to talk them through, whatever it says next
    }
    if crate::compose::badnews::reads_as_someones_death(&text_) {
        // "someone in my class died" is not a thing that is broken. A phone that died stays here,
        // because the subject of that death is not a person.
        return None;
    }
    let mut shape = _shape(&text_, words);
    if shape.is_none() && !repaired.is_empty() && repaired != text_ {
        shape = _shape(&_without_a_dead_that_only_means_very(&_without_wanting_or_praise(repaired)), None);
    }
    shape
}

/// Python: bot/compose/trouble.py:1017 `_without_wanting_or_praise(text)`
/// The line with a "to die for" / "I'd die for" taken out of it; the words around it stay.
pub fn _without_wanting_or_praise(text_: &str) -> String {
    WANTING_OR_PRAISE.sub(" ", text_)
}

/// Python: bot/compose/trouble.py:1027 `_without_a_dead_that_only_means_very(text)`
/// The line with a "dead" that describes the next word taken out (see DEAD_AS_VERY_AFTER).
pub fn _without_a_dead_that_only_means_very(text_: &str) -> String {
    let words = text::split_ref(text_);
    if !words.contains(&"dead") {
        return text_.to_string();
    }
    let mut kept = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let following = words.get(index + 1).copied().unwrap_or("");
        if *word == "dead" && DEAD_AS_VERY_AFTER.contains(text::strip_chars(following, &ODD_MARK)) {
            continue;
        }
        kept.push(*word);
    }
    kept.join(" ")
}

/// Python: bot/compose/trouble.py:1065 `_someone_is_the_subject(text)`
/// True when a person or an animal in the line is what it is about. A name is the subject
/// unless it is the front half of a thing's name: "the cat flap is broken" is a flap.
pub fn _someone_is_the_subject(text_: &str) -> bool {
    let words = az_words(text_);
    for (index, word) in words.iter().enumerate() {
        if !someone_elses(std::slice::from_ref(word)) {
            continue;
        }
        let following = words.get(index + 1).map(String::as_str).unwrap_or("");
        if ANIMAL_MODIFIERS.contains(word) && !following.is_empty() && !NOT_A_MODIFIER.contains(following) {
            continue; // "cat flap", "fish tank": the animal is part of the thing's name
        }
        return true;
    }
    false
}

/// Python: bot/compose/trouble.py:1083 `_the_noun_before(text, at)`
/// The noun just before a match, whoever owns it: "the leaves" is leaves, "my slabs" is slabs.
/// (`at` is a byte offset into `text`, as a match gives it.)
pub fn _the_noun_before(text_: &str, at: usize) -> String {
    let mut words = az_words(&format!(" {}", &text_[..at.min(text_.len())]));
    while words.last().is_some_and(|w| JOINING.contains(w)) {
        words.pop();
    }
    words.pop().unwrap_or_default()
}

/// Python: bot/compose/trouble.py:1096 `_a_fault_of_a_thing(text)`
/// True when the fault in the line hangs on a thing, not on a person or an animal.
pub fn _a_fault_of_a_thing(text_: &str) -> bool {
    let padded = format!(" {text_} ");
    THING.is_match(&padded) && OWNS.is_match(&padded) && !_someone_is_the_subject(text_)
}

/// Python: bot/compose/trouble.py:1103 `_names_a_body_part(text)`
/// True when the line names a part of a body, which is what a bodily pain needs.
pub fn _names_a_body_part(text_: &str) -> bool {
    any_in(&az_words(text_), &BODY_PART)
}

/// Python: bot/compose/trouble.py:1108 `_figurative_passes(text)`
/// True when a line whose only ache word can also mean "a nuisance" is about the body: a named
/// body part settles it, and without one the nuisance reading needs a thing of theirs.
pub fn _figurative_passes(text_: &str) -> bool {
    if _names_a_body_part(text_) {
        return true;
    }
    !NUISANCE.is_match(text_)
}

/// Python: bot/compose/trouble.py:1119 `_shape(clean, words=None)`
/// The shape of the trouble in this one text, or None.
pub fn _shape(clean: &str, words: Option<&[String]>) -> Option<String> {
    let text_ = _flat(clean);
    if text_.is_empty() || text_.contains('?') {
        return None;
    }
    let words: Vec<String> = match words {
        Some(words) if !words.is_empty() => words.to_vec(),
        _ => az_words(&text_),
    };
    let has_body_part = any_in(&words, &BODY_PART);
    let mine = format!(" {text_} ");
    // A state of a thing - "has gone all black", "is burnt on", "has seized up" - read before the
    // body and the acute fence, because the burn and the seizure are words those two also own.
    let a_state = THING_STATE_ONLY.search(&mine);
    let growing_state = a_state.as_ref().is_some_and(|state| GROWING.contains(&_the_noun_before(&mine, state.start())));
    if someone_elses(&words) && !_a_fault_of_a_thing(&text_) {
        // Someone else in the line stands this reading down when they are what the fault hangs on.
        return None;
    }
    if PLEA.is_match(&text_) || DEATHWISH.is_match(&text_) {
        return None;
    }
    // The acute stand-down holds everywhere but a state of a thing with no body part in the line.
    if ACUTE.is_match(&text_) && !(a_state.is_some() && !growing_state && !has_body_part) {
        return None;
    }
    // A fracture word is an injury only when a body is there for it to belong to.
    if FRACTURE.is_match(&text_) && words.iter().any(|w| BODY_PART.contains(w) || BONE.contains(w)) {
        return None;
    }
    // A broken heart, a broken marriage, a broken trust are not things that are doing something.
    // The word a break hangs on decides it.
    let hung_on = _the_fault_hangs_on(&text_);
    if BREAKS.is_match(&text_) && (any_in(&words, &NOT_A_THING) || any_in(&hung_on, &NOT_A_THING)) {
        return None;
    }
    let own = MINE.match_(&text_).is_some();
    // A body part with dead/killing/hurting, or with the end-of-a-thing words, counts as the
    // body's trouble too.
    let mut body_word = BODY.is_match(&mine);
    if a_state.is_some() && !growing_state && !has_body_part {
        // "the pan is burnt" is a thing's damage; only a body part in the line keeps the word for
        // the body ("my lips are burnt").
        body_word = false;
    }
    let ache = body_word || (has_body_part && AN_ACHE_SAID_OF_A_PART.is_match(&mine));
    if own && ache {
        if FIGURATIVE.is_match(&mine) && !_figurative_passes(&text_) {
            return None;
        }
        return Some("sore".to_string());
    }
    // A part of their life going flat is not a fault to repair and not a shrug.
    if FLAT.is_match(&mine) && any_in(&_the_fault_hangs_on(&mine), &FLAT_SUBJECT) {
        return Some("strain".to_string());
    }
    // A setback said vaguely ("my cake was a disaster") is a thing of theirs too, and the
    // question is what happened rather than what it is doing.
    if crate::compose::fallback_chain::truthy(crate::memory::setback::reading(&text_)) && !has_body_part {
        return Some("vague".to_string());
    }
    if let Some(thing) = THING.search(&mine) {
        if OWNS.is_match(&mine) {
            // A fault said of a part of their life rather than of a thing they own is the
            // "strain" shape, which asks what has gone wrong with it instead.
            let hangs_on = _the_thing_it_hangs_on(&mine, thing.start());
            if NOT_A_THING.contains(&hangs_on) {
                return Some("strain".to_string());
            }
            let state_here = a_state.as_ref().is_some_and(|state| state.start() == thing.start());
            // A body part said in the shape of a state - "my knee is stiff" - is the body's complaint.
            if BODY_PART.contains(&hangs_on) && state_here {
                return Some("sore".to_string());
            }
            // The state branch is what matched, and the thing that went into the state is
            // growing: its colour is the season.
            if growing_state && state_here {
                return None;
            }
            // "my knee is on its last legs" is a body said with the words for a washing machine.
            if BODY_PART.contains(&hangs_on) && A_DECLINE.is_match(&mine[thing.start()..]) {
                return None;
            }
            return Some("thing".to_string());
        }
    }
    if CANT_GET.is_match(&mine) && OWNS.is_match(&mine) && !has_body_part {
        return Some("thing".to_string());
    }
    if _keeps_doing_it(&mine) && !has_body_part {
        return Some("thing".to_string());
    }
    None
}

/// Python: bot/compose/trouble.py:1221 `_keeps_doing_it(text)`
/// True when something of theirs keeps doing the wrong thing: "my razor keeps pulling". The
/// thing the fault hangs on has to be a thing of theirs, and the verb not a way it should work.
pub fn _keeps_doing_it(text_: &str) -> bool {
    if ASKS_TO_CHOOSE.is_match(text_) {
        return false;
    }
    for found in KEEPS_AT_IT.finditer(text_) {
        let doing = found.str(1);
        if KEEPS_FINE.contains(doing) {
            continue;
        }
        let before = az_words(&text_[..found.start()]);
        if !any_in(&before[before.len().saturating_sub(3)..], &KEEPS_THEIRS) {
            continue;
        }
        let head = before
            .iter()
            .rev()
            .find(|word| !FAULT_FILLERS.contains(*word) && !NOT_THE_THING.contains(*word))
            .cloned()
            .unwrap_or_default();
        if head.is_empty() || OTHERS.contains(&head) || NOT_A_THING.contains(&head) || BODY_PART.contains(&head) {
            continue;
        }
        if NOT_THE_THING.contains(&head) || KEEPS_NOT_A_THING.contains(&head) {
            continue;
        }
        if KEEPS_NOT_THEIR_THING.contains(&head) {
            continue;
        }
        if !OWNS.is_match(text_) {
            continue;
        }
        // The words in front of the verb have to carry a possessive, or name something a
        // household is made of, or the line keeps the answer it had.
        let phrase: Vec<&String> =
            before.iter().filter(|w| !FAULT_FILLERS.contains(*w) && !NOT_THE_THING.contains(*w)).collect();
        if !phrase.iter().any(|w| KEEPS_POSSESSIVE.contains(*w)) && !phrase.iter().any(|w| KEEPS_OF_A_HOUSE.contains(*w)) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/compose/trouble.py:1265 `_the_thing_it_hangs_on(text, at)`
/// The word a fault hangs on: the noun just before it, past the words that only join them, with
/// a "my" or an "our" in front of it and nothing but the thing's own words in between. (`at` is
/// a byte offset into `text`.)
pub fn _the_thing_it_hangs_on(text_: &str, at: usize) -> String {
    let mut words = az_words(&text_[..at.min(text_.len())]);
    while words.last().is_some_and(|w| JOINING.contains(w)) {
        words.pop();
    }
    let Some(noun) = words.pop() else { return String::new() };
    let mut back = words;
    for _ in 0..3 {
        let Some(last) = back.last() else { break };
        if last == "my" || last == "our" {
            return noun;
        }
        if NOT_OWNED.contains(last) {
            return String::new();
        }
        back.pop();
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_as_very_is_taken_out() {
        assert_eq!(_without_a_dead_that_only_means_very("im dead tired"), "im tired");
        assert_eq!(_without_a_dead_that_only_means_very("the boiler is dead"), "the boiler is dead");
    }
}
