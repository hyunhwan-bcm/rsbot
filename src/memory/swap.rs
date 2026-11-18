//! Port of `bot/memory/swap.py`: a follow-up that only names a new thing ("what about a
//! dessert", "no i mean brown rice") read as the earlier question with that thing in it. This
//! module offers a reading; whether the rewritten line reaches an answer about the new thing is
//! the caller's check.

use super::recent;
use super::topics::person;
use crate::data;
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.swap.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_of(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static LEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("LEAD"));
static REPAIR: LazyLock<&'static Re> = LazyLock::new(|| rx("REPAIR"));
static TAKE_BACK: LazyLock<&'static Re> = LazyLock::new(|| rx("TAKE_BACK"));
static A_COURTESY: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("A_COURTESY"));
static A_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_CLAUSE"));
static THING: LazyLock<&'static Re> = LazyLock::new(|| rx("THING"));
static ASKS_A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("ASKS_A_THING"));
static TAIL_OFF: LazyLock<&'static Re> = LazyLock::new(|| rx("TAIL_OFF"));
static HEDGE_OFF: LazyLock<&'static Re> = LazyLock::new(|| rx("HEDGE_OFF"));
static DEICTIC: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("DEICTIC"));
static STOP: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("STOP"));
static OPENERS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("OPENERS"));
static ASK: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("ASK"));
static PERSON_LEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("PERSON_LEAD"));
static PERSON_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("PERSON_FILLER"));
static OWNER: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("OWNER"));
static OWNER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("OWNER_WORDS"));
static DETERMINER: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("DETERMINER"));
static PREPOSITION: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("PREPOSITION"));
static PREPOSITION_TAIL: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("PREPOSITION_TAIL"));

static POSSESSIVE_S: LazyLock<Re> = LazyLock::new(|| Re::new(r"'s\b", re::U));
static SAID_TWICE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\w+)\s+\1\b", re::U));

const ENDS: &str = " ?!.";
const ENDS_AND_COMMA: &str = " ?!.,";
const PUNCT: &str = "?.!,;:";

/// `re.split(r"[ ']", s)` with the empty pieces dropped.
fn bits(s: &str) -> Vec<&str> {
    s.split([' ', '\'']).filter(|word| !word.is_empty()).collect()
}

/// `word.strip("?.!,;:").lower()`.
fn bare_low(word: &str) -> String {
    text::strip_chars(word, PUNCT).to_lowercase()
}

/// Python: bot/memory/swap.py:57 `_after_a_take_back(said)`
/// What a line says after its "no wait", or None when the line is not a take-back and a thing.
/// The rest has to be short and name no happening of its own.
pub fn _after_a_take_back(said: &str) -> Option<String> {
    let lead = TAKE_BACK.match_(said)?;
    let cut = TAIL_OFF.sub("", text::strip_chars(&said[lead.end()..], ENDS_AND_COMMA));
    let rest = text::strip_chars(&cut, ENDS_AND_COMMA);
    if rest.is_empty() || text::split_ref(rest).len() > 3 || A_CLAUSE.is_match(rest) {
        return None;
    }
    Some(rest.to_string())
}

/// Python: bot/memory/swap.py:125 `_said(text)`
pub fn _said(text_: &str) -> String {
    text::squash(&text_.to_lowercase())
}

/// Python: bot/memory/swap.py:129 `_stem(word)`
pub fn _stem(word: &str) -> String {
    let word = text::strip_chars(word, PUNCT);
    for end in ["ies", "es", "s"] {
        if text::len(word) > 4 && word.ends_with(end) {
            return word[..word.len() - end.len()].to_string();
        }
    }
    word.to_string()
}

/// Python: bot/memory/swap.py:137 `thing(text)`
/// The new thing a follow-up names, or None: "what about a dessert" -> "dessert".
pub fn thing(text_: &str) -> Option<String> {
    let flat = _said(text_);
    let said = text::strip_chars(&flat, ENDS);
    if said.is_empty() {
        return None;
    }
    let mut repaired = false;
    let rest: String = if let Some(lead) = LEAD.match_(said) {
        said[lead.end()..].to_string()
    } else if let Some(lead) = REPAIR.match_(said) {
        repaired = true;
        said[lead.end()..].to_string()
    } else {
        _after_a_take_back(said)?
    };
    // A repair puts another *thing* where the last one was, and a thing is not a clause: "no i
    // mean should i quit" is the question they meant, said plainer, not a new subject.
    if repaired && A_CLAUSE.is_match(&rest) {
        return None;
    }
    // The words after the thing only say it is a follow-up.
    let cut = TAIL_OFF.sub("", text::strip_chars(&rest, ENDS_AND_COMMA));
    let mut rest = text::strip_chars(&cut, ENDS_AND_COMMA).to_string();
    // And the hesitation comes off before the name is read: "a labrador probably".
    for _ in 0..3 {
        let hedged = text::strip_chars(&HEDGE_OFF.sub("", &rest), ENDS_AND_COMMA).to_string();
        if hedged == rest {
            break;
        }
        rest = hedged;
    }
    if rest.is_empty() || text::split_ref(&rest).len() > 4 {
        return None;
    }
    let found = THING.match_(&rest)?;
    let mut name = text::strip(found.str(1)).to_string();
    let mut words: Vec<String> = bits(&name).into_iter().map(str::to_string).collect();
    if words.len() > 1 && OWNER_WORDS.contains(&words[0]) {
        // "what about my hip" names the hip: the possessive says whose it is, not which one.
        words.remove(0);
        name = words.join(" ");
    }
    if words.is_empty() || words.len() > 3 {
        return None;
    }
    if DEICTIC.contains(&name) || DEICTIC.contains(&words.join(" ")) {
        return None;
    }
    // "what about it" and "and the same again" point back; "what about a dessert" names a thing.
    if words.iter().any(|word| DEICTIC.contains(word)) {
        return None;
    }
    // An ask is not a thing.
    if ASKS_A_THING.is_match(&name) {
        return None;
    }
    if words.iter().all(|word| STOP.contains(word)) {
        return None;
    }
    // "no thanks at all", "no ta", "no worries": what is left after the no answers the question
    // instead of naming a new thing to ask it about.
    if words.iter().all(|word| A_COURTESY.contains(word) || STOP.contains(word)) {
        return None;
    }
    Some(name)
}

/// Python: bot/memory/swap.py:229 `someone(text)`
/// The person a follow-up only names, or None: "what about my dad" -> "dad". A line that says
/// anything else about them ("what about my dad's birthday") asks its own thing.
pub fn someone(text_: &str) -> Option<String> {
    let flat = _said(text_);
    let said = text::strip_chars(&flat, ENDS);
    if said.is_empty() || text::split_ref(said).len() > 6 {
        return None;
    }
    let lead = PERSON_LEAD.match_(said).or_else(|| REPAIR.match_(said))?;
    let rest = text::strip_chars(&said[lead.end()..], ENDS_AND_COMMA);
    let who = person(rest).filter(|who| !who.is_empty())?;
    let theirs = Re::new(&format!(r"\b(?:my|our)\s+{}\b", re::escape(&who)), re::U);
    let left = POSSESSIVE_S.sub("", &theirs.sub(" ", rest));
    let spare = bits(&left).into_iter().any(|word| !PERSON_FILLER.contains(word) && !STOP.contains(word));
    if spare {
        return None; // their birthday, their new job: a thing of theirs, not the person
    }
    Some(who)
}

/// Python: bot/memory/swap.py:256 `_earlier(memory, text, n=4, wide=False)`
/// The person's own questions before this line, newest first, with the answer each got.
/// *wide* takes statements as well as questions: a repair can take back a statement too.
pub fn _earlier(memory: &Memory, text_: &str, n: usize, wide: bool) -> Vec<(String, String)> {
    let here = _said(text_);
    let mut found = Vec::new();
    for turn in recent(Some(memory), n as i64 + 2).iter().rev() {
        let question = _said(&turn.user);
        if !question.is_empty() && (question == here || question == text::rstrip_chars(&here, ENDS)) {
            continue; // this line, already written down
        }
        let answer = _said(&turn.bot);
        let words = text::split_ref(&question);
        if words.len() < 3 {
            continue;
        }
        let first = text::strip_chars(words[0], "?,!");
        if !wide && !question.contains('?') && !OPENERS.contains(first) && !ASK.contains(first) {
            continue; // "whats the capital of australia" with no question mark is still a question
        }
        found.push((question.clone(), answer));
        if found.len() >= n {
            break;
        }
    }
    found
}

/// Python: bot/memory/swap.py:290 `_about(question)`
/// The thing an earlier line is asked about, or None: the chicken in "how long does a chicken
/// take to roast". Read off the understanding of the earlier line rather than guessed from its
/// words; a line with no subject of its own has nothing to offer.
pub fn _about(question: &str) -> Option<String> {
    // `except Exception`: a line that cannot be read offers no subject.
    let read = std::panic::catch_unwind(|| crate::understanding::understand(question, Some(&Memory::new()))).ok()?;
    let flat = _said(read.subject.as_deref().unwrap_or(""));
    let subject = text::strip_chars(&flat, ENDS_AND_COMMA);
    if subject.is_empty() || text::split_ref(subject).len() > 3 {
        return None;
    }
    let mut words = text::split_ref(subject);
    while !words.is_empty() && OWNER.contains(words[0]) {
        words.remove(0);
    }
    if words.is_empty() || words.iter().all(|word| STOP.contains(*word)) {
        return None;
    }
    Some(words.join(" "))
}

/// Python: bot/memory/swap.py:324 `_reads_as_a_question(line)`
/// True when the earlier line asks something of its own: a question mark, or a question word
/// at the front.
pub fn _reads_as_a_question(line: &str) -> bool {
    let words = text::split_ref(line);
    if words.is_empty() {
        return false;
    }
    if line.contains('?') {
        return true;
    }
    let first = text::strip_chars(words[0], "?,!");
    OPENERS.contains(first) || ASK.contains(first)
}

/// Python: bot/memory/swap.py:341 `_word(question, answer, new_words, taken_up)`
/// The earlier question's content word to swap: the one the answer took up (or, on the second
/// pass, its last content word), or None.
pub fn _word(question: &str, answer: &str, new_words: &HashSet<String>, taken_up: bool) -> Option<String> {
    for word in text::split_ref(question).into_iter().rev() {
        let word = text::strip_chars(word, PUNCT);
        if STOP.contains(word) || text::len(word) < 3 || !text::is_alpha(word) {
            continue;
        }
        if new_words.contains(word) || DEICTIC.contains(word) || ASK.contains(word) {
            continue;
        }
        if taken_up {
            let stem = _stem(word);
            if stem.is_empty() || !answer.contains(&stem) {
                continue;
            }
        }
        return Some(word.to_string());
    }
    None
}

/// Python: bot/memory/swap.py:356 `_replace(question, word, new)`
/// The earlier question with *word* taken out and the new thing in its place.
pub fn _replace(question: &str, word: &str, new: &str) -> Option<String> {
    let named = Re::new(&format!(r"\b(?:(a|an)\s+)?{}\b", re::escape(word)), re::I | re::U);
    let found = named.search(question)?;
    // "boil an egg" with "duck egg" put in is "boil a duck egg": the article belongs to the new
    // thing, not to the one it replaced.
    let lead = match found.get(1) {
        Some(article) if !article.is_empty() => {
            if new.chars().next().is_none_or(|c| "aeiou".contains(c)) { "an " } else { "a " }
        }
        _ => "",
    };
    let rewritten = format!("{}{lead}{new}{}", &question[..found.start()], &question[found.end()..]);
    // A new thing that ends in a word the question already has next to it would say it twice.
    let rewritten = SAID_TWICE.sub(r"\1", &rewritten);
    if rewritten == question {
        return None;
    }
    // A correction renames the thing, so the old thing goes: when the new thing's own head noun
    // is still in the line, the swap took out something else.
    let new_words = text::split_ref(new);
    let head = new_words.last().expect("IndexError: list index out of range").to_lowercase();
    if new_words.len() > 1 && text::count(&rewritten.to_lowercase(), &head) > 1 {
        return None;
    }
    Some(rewritten)
}

/// `may_be_it(word)` in `_spans`.
fn may_be_it(word: &str) -> bool {
    !word.is_empty() && text::is_alpha(word) && !STOP.contains(word) && !DEICTIC.contains(word) && !ASK.contains(word)
}

/// Python: bot/memory/swap.py:386 `_spans(question, subject)`
/// The stretches of the earlier question a new thing could take the place of: the subject and
/// the words around it, longest first, because a longer stretch leaves less of the old thing
/// behind.
pub fn _spans(question: &str, subject: &str) -> Vec<String> {
    let words: Vec<&str> = text::split_ref(question).into_iter().map(|w| text::strip_chars(w, PUNCT)).collect();
    let Some(at) = words.iter().position(|word| *word == subject).filter(|_| !subject.contains(' ')) else {
        return vec![subject.to_string()];
    };
    let mut spans: Vec<String> = Vec::new();
    for back in [2, 1] {
        if at >= back && words[at - back..at].iter().all(|w| may_be_it(w)) {
            spans.push(words[at - back..=at].join(" "));
        }
    }
    if at + 1 < words.len() && may_be_it(words[at + 1]) {
        spans.push(words[at..at + 2].join(" "));
    }
    spans.push(subject.to_string());
    for back in [2, 1] {
        if at >= back && words[at - back..at].iter().all(|w| may_be_it(w)) {
            spans.push(words[at - back..at].join(" "));
        }
    }
    let mut found = HashSet::new();
    spans.into_iter().filter(|span| !span.is_empty() && found.insert(span.clone())).collect()
}

/// Python: bot/memory/swap.py:420 `is_repair(text)`
/// True when the line takes back the thing it named: "i meant a turkey", "no wait, duck".
pub fn is_repair(text_: &str) -> bool {
    let flat = _said(text_);
    let said = text::strip_chars(&flat, ENDS);
    REPAIR.match_(said).is_some() || _after_a_take_back(said).is_some()
}

/// Python: bot/memory/swap.py:426 `candidates(text, memory, most=3)`
/// The lines this one could be read as, best first, each with the word it replaced. The thing
/// the earlier line is asked about is tried first, then the word the answer took up, then the
/// question's last content word.
pub fn candidates(text_: &str, memory: &Memory, most: usize) -> Vec<(String, String)> {
    let Some(new) = thing(text_).filter(|new| !new.is_empty()) else { return Vec::new() };
    let repaired = is_repair(text_);
    let new_words: HashSet<String> = new.split([' ', '\'']).map(str::to_string).collect();
    let place = !_the_preposition_of(&new).is_empty();
    let mut found: Vec<(String, String)> = Vec::new();
    for (question, answer) in _earlier(memory, text_, 4, repaired) {
        let subject = _about(&question);
        let mut words: Vec<Option<String>> = Vec::new();
        if place {
            // A thing that arrives with its own preposition goes where the earlier question's
            // own place was, whole: the place the same preposition was said with is tried first.
            if let Some(had) = _the_place_after_the_same_preposition(&question, &new) {
                words.push(Some(had));
            }
        }
        if let Some(subject) = subject.filter(|s| !s.is_empty()) {
            words.extend(_spans(&question, &subject).into_iter().map(Some));
        }
        words.push(_word(&question, &answer, &new_words, true));
        words.push(_word(&question, &answer, &new_words, false));
        for word in words.into_iter().flatten() {
            if word.is_empty() {
                continue;
            }
            let line = if place { placed(&question, &word, &new) } else { _replace(&question, &word, &new) };
            let Some(line) = line.filter(|line| !line.is_empty()) else { continue };
            if _a_second_place(&question, &line, &new) {
                continue;
            }
            if found.iter().any(|(had, _)| *had == line) {
                continue;
            }
            found.push((line, word));
            if found.len() >= most {
                return found;
            }
        }
    }
    found
}

/// Python: bot/memory/swap.py:475 `rewritten(text, memory)`
/// The line read as the earlier question with the earlier thing swapped, or None. Returns the
/// rewritten question and the word it replaced.
pub fn rewritten(text_: &str, memory: &Memory) -> Option<(String, String)> {
    let new = thing(text_).filter(|new| !new.is_empty())?;
    let new_words: HashSet<String> = new.split([' ', '\'']).map(str::to_string).collect();
    // A statement counts here as well as a question: "my knee hurts" ... "what about my hip".
    let earlier = _earlier(memory, text_, 4, true);
    let (asked, told): (Vec<_>, Vec<_>) = earlier.iter().partition(|(line, _)| _reads_as_a_question(line));
    // The thing being talked about is the one the answer took up, and it is tried first; a
    // second pass takes the question's own last content word.
    for taken_up in [true, false] {
        for (question, answer) in &asked {
            if answer.is_empty() {
                continue;
            }
            let Some(swap) = _word(question, answer, &new_words, taken_up).filter(|w| !w.is_empty()) else { continue };
            if let Some(candidate) = placed(question, &swap, &new).filter(|c| !c.is_empty() && c != question) {
                return Some((candidate, swap));
            }
        }
    }
    // A question always gets first refusal: a statement is the weaker reading of the two. It is
    // only a list being worked down when the new thing is another kind of the thing a follow-up
    // in between named.
    let in_between = earlier.first().and_then(|(line, _)| thing(line)).filter(|t| !t.is_empty());
    let new_parts = text::split_ref(&new);
    let head = if new_parts.len() > 1 { bare_low(new_parts[new_parts.len() - 1]) } else { String::new() };
    if let Some(in_between) = in_between {
        if !text::split_ref(&in_between).contains(&head.as_str()) {
            return None;
        }
    }
    for (question, answer) in &told {
        if answer.is_empty() {
            continue;
        }
        let Some(subject) = _about(question).filter(|s| !s.is_empty()) else { continue };
        for swap in _spans(question, &subject) {
            if let Some(candidate) = placed(question, &swap, &new).filter(|c| !c.is_empty() && c != question) {
                return Some((candidate, swap));
            }
        }
    }
    None
}

/// Python: bot/memory/swap.py:539 `_named_somebody(memory, text, n=4)`
/// The person's own lines before this one that name somebody, newest first, with the answer.
/// A statement counts here where the thing swap asks for a question.
pub fn _named_somebody(memory: &Memory, text_: &str, n: usize) -> Vec<(String, String)> {
    let here = _said(text_);
    let mut found = Vec::new();
    for turn in recent(Some(memory), n as i64 + 2).iter().rev() {
        let line = _said(&turn.user);
        if line.is_empty() || line == here || line == text::rstrip_chars(&here, ENDS) {
            continue;
        }
        if text::split_ref(&line).len() < 3 {
            continue;
        }
        found.push((line, _said(&turn.bot)));
        if found.len() >= n {
            break;
        }
    }
    found
}

/// Python: bot/memory/swap.py:564 `rewritten_person(text, memory)`
/// The line read as the earlier question with the earlier person swapped, or None. Returns the
/// rewritten question and the person it replaced.
pub fn rewritten_person(text_: &str, memory: &Memory) -> Option<(String, String)> {
    let who = someone(text_)?;
    for (question, _answer) in _named_somebody(memory, text_, 4) {
        let Some(old) = person(&question).filter(|old| !old.is_empty() && *old != who) else { continue };
        let named = Re::new(&format!(r"\b(my|our)\s+{}\b", re::escape(&old)), re::I | re::U);
        if !named.is_match(&question) {
            continue;
        }
        let candidate = named.sub_with(&question, 1, |m| format!("{} {who}", m.str(1)));
        if candidate != question {
            return Some((candidate, old));
        }
    }
    None
}

/// Python: bot/memory/swap.py:593 `names_person(old, answer, text, last='')`
/// True when an answer to the rewritten question is about the new person, not the old one:
/// the old person's name is not in it, and it is not the answer just given handed over again.
pub fn names_person(old: &str, answer: &str, text_: &str, last: &str) -> bool {
    let who = someone(text_);
    let said = _said(answer);
    if who.is_none() || said.is_empty() {
        return false;
    }
    if !old.is_empty() && Re::new(&format!(r"\b{}\b", re::escape(old)), re::U).is_match(&said) {
        return false;
    }
    last.is_empty() || said != _said(last)
}

/// Python: bot/memory/swap.py:625 `_the_preposition_of(new)`
/// The preposition a new thing arrives with ("from birmingham" -> "from"), or "".
pub fn _the_preposition_of(new: &str) -> String {
    let Some(first) = text::split_ref(new).first().copied() else { return String::new() };
    let head = bare_low(first);
    if PREPOSITION.contains(&head) { head } else { String::new() }
}

/// Python: bot/memory/swap.py:634 `_the_place_after_the_same_preposition(question, new)`
/// The earlier question's place, said with the preposition the new thing brings, or None.
pub fn _the_place_after_the_same_preposition(question: &str, new: &str) -> Option<String> {
    let head = _the_preposition_of(new);
    if head.is_empty() {
        return None;
    }
    let said = text::split_ref(question);
    for (at, word) in said.iter().enumerate() {
        if bare_low(word) != head {
            continue;
        }
        let mut after = at + 1;
        while after < said.len() && DETERMINER.contains(&bare_low(said[after])) {
            after += 1;
        }
        let mut span: Vec<&str> = Vec::new();
        for follow in &said[after.min(said.len())..] {
            let bare = text::strip_chars(follow, PUNCT);
            let low = bare.to_lowercase();
            if !text::is_alpha(bare) || STOP.contains(&low) || PREPOSITION.contains(&low) {
                break;
            }
            span.push(bare);
            if span.len() == 2 {
                break;
            }
        }
        if !span.is_empty() {
            return Some(span.join(" "));
        }
    }
    None
}

/// Python: bot/memory/swap.py:664 `_a_second_place(question, line, new)`
/// True when a reading hangs a second place on the earlier question with the same preposition
/// ("from ... from ...").
pub fn _a_second_place(question: &str, line: &str, new: &str) -> bool {
    let head = _the_preposition_of(new);
    if head.is_empty() {
        return false;
    }
    let count = |text_: &str| text::split_ref(text_).iter().filter(|word| bare_low(word) == head).count();
    count(line) > count(question)
}

/// Python: bot/memory/swap.py:682 `placed(question, old, new)`
/// The earlier question with the old thing taken out and the new one in its place, or None. A
/// new thing that carries its own preposition goes where the old place was whole: the
/// preposition in front of the old thing, any "my"/"the" in between, and the thing itself come
/// out together.
pub fn placed(question: &str, old: &str, new: &str) -> Option<String> {
    let words = text::split_ref(question);
    let new_words = text::split_ref(new);
    let at = words.iter().position(|word| text::strip_chars(word, PUNCT) == old)?;
    if new_words.is_empty() {
        return None;
    }
    let low = |i: usize| bare_low(words[i]);
    let joined = |start: usize| {
        words[..start].iter().chain(new_words.iter()).chain(words[at + 1..].iter()).copied().collect::<Vec<_>>().join(" ")
    };
    // What decides is whether the new thing arrives with a place's preposition of its own.
    let head = bare_low(new_words[0]);
    if PREPOSITION.contains(&head) || PREPOSITION_TAIL.contains(&head) {
        let mut start = at;
        while start > 0 && DETERMINER.contains(&low(start - 1)) {
            start -= 1;
        }
        if start == 0 || !PREPOSITION.contains(&low(start - 1)) {
            // The earlier question has no place around the thing, so the new place goes where
            // the thing was: "how do i clean under the sink".
            return Some(joined(start));
        }
        start -= 1;
        if start > 0 && PREPOSITION_TAIL.contains(&low(start - 1)) {
            start -= 1; // "next to my shed", "on top of the fridge"
        }
        return Some(joined(start));
    }
    let rewritten = question.replacen(old, new, 1);
    // A new thing whose own head noun is already beside the old one would say it twice.
    let head = bare_low(new_words[new_words.len() - 1]);
    if new_words.len() > 1 && text::count(&rewritten.to_lowercase(), &head) > 1 {
        return None;
    }
    Some(rewritten)
}

/// Python: bot/memory/swap.py:726 `_said_again(answer, memory)`
/// True when this answer is the one just given, word for word.
pub fn _said_again(answer: &str, memory: Option<&Memory>) -> bool {
    let Some(turn) = memory.and_then(|memory| memory.turns.last()) else { return false };
    if text::strip(&turn.bot).is_empty() {
        return false;
    }
    text::squash(&answer.to_lowercase()) == text::squash(&turn.bot.to_lowercase())
}

/// Python: bot/memory/swap.py:745 `_brings_its_own_place(new)`
/// Whether the new thing is a place said with its own preposition ("under the sink").
pub fn _brings_its_own_place(new: &str) -> bool {
    text::split_ref(new).first().is_some_and(|word| PREPOSITION.contains(&bare_low(word)))
}

/// Python: bot/memory/swap.py:751 `names(old, answer, text, memory=None, question=None)`
/// True when an answer to the rewritten question is about the new thing, not the old one. An
/// answer that names neither is taken, except for a place that brings its own preposition:
/// there the answer also has to carry one of the question's own subject words.
pub fn names(old: &str, answer: &str, text_: &str, memory: Option<&Memory>, question: &str) -> bool {
    let said = _said(answer);
    let Some(new) = thing(text_).filter(|new| !new.is_empty() && !said.is_empty()) else { return false };
    if _said_again(answer, memory) {
        return false; // the same paragraph again is not an answer about the new thing
    }
    if bits(&new).into_iter().filter(|word| !STOP.contains(*word)).any(|word| said.contains(&_stem(word))) {
        return true;
    }
    if _brings_its_own_place(&new) {
        let asked = _said(question);
        return bits(&asked).into_iter().filter(|word| !STOP.contains(*word)).any(|word| said.contains(&_stem(word)));
    }
    !said.contains(&_stem(old))
}
