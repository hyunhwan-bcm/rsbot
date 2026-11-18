//! Port of `bot/understanding/body.py`: which part of the speaker's own body the line is about.

#![allow(non_upper_case_globals)]

use crate::py::re::{Match, Re, I};
use crate::py::text;
use crate::understanding::kinds;
use crate::understanding::{pyuni, rx, rx_flags, t};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.understanding.body.";

static ALREADY_DEALT_WITH: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ALREADY_DEALT_WITH"));
static ANOTHER_PART: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ANOTHER_PART"));
static AN_ABSCESS: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "AN_ABSCESS"));
static ASKING: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "ASKING"));
static A_CHAIR_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_CHAIR_WORD"));
static A_GUT_ONLY: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_GUT_ONLY"));
static A_LOOSE_STOOL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_LOOSE_STOOL"));
static A_MOUTH_WORD: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_MOUTH_WORD"));
static A_POO_COLOUR: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_POO_COLOUR"));
static A_SAMPLE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_SAMPLE"));
static A_SPREADING_SIGN: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_SPREADING_SIGN"));
static A_THING_IS_TERRIBLE: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "A_THING_IS_TERRIBLE"));
static A_WORD_OF_ITS_OWN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "A_WORD_OF_ITS_OWN"));
static COMMON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "COMMON_WORDS"));
static FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "FILLER"));
static OFF_MAY_GO_ON: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "OFF_MAY_GO_ON"));
static PAIN_ADJECTIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PAIN_ADJECTIVE"));
static PAIN_BETWEEN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PAIN_BETWEEN"));
static PAIN_PHRASE: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| t::tuples(M, "PAIN_PHRASE"));
const PAIN_REACH: i64 = 3;
static PLAIN_PAIN: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLAIN_PAIN"));
static PLAIN_SKIP: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLAIN_SKIP"));
static PLAIN_SKIP_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLAIN_SKIP_WORDS"));
static PLAIN_UNWELL: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "PLAIN_UNWELL"));
static POSSESSIVE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "POSSESSIVE"));
static SPEAKER_ILL: LazyLock<&'static Re> = LazyLock::new(|| t::re(M, "SPEAKER_ILL"));
static WHEN_IT_HAPPENED: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "WHEN_IT_HAPPENED"));
static _A_HELPER_BEFORE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_HELPER_BEFORE"));
static _A_MAYBE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_MAYBE"));
static _A_NO: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_A_NO"));
static _A_VERB_NEEDING_ITS_SUBJECT: LazyLock<Vec<String>> = LazyLock::new(|| t::strs(M, "_A_VERB_NEEDING_ITS_SUBJECT"));  // t, 17 items
static _BEFORE_THE_COMPLAINT: LazyLock<HashSet<String>> = LazyLock::new(|| t::set(M, "_BEFORE_THE_COMPLAINT"));
/// `PARTS`: (name, pattern, nouns) for each part of the body, in Python's order.
static PARTS: LazyLock<Vec<(String, &'static Re, HashSet<String>)>> = LazyLock::new(|| {
    t::value(M, "PARTS").items().iter().map(|p| (p.items()[0].str().to_string(), rx_flags(p.items()[1].str(), I), p.items()[2].str_set())).collect()
});
/// `PART_NOUNS_BY_LENGTH`: the part nouns by length (built at import from PARTS).
static PART_NOUNS_BY_LENGTH: LazyLock<HashMap<usize, Vec<String>>> = LazyLock::new(|| {
    let mut m: HashMap<usize, BTreeSet<String>> = HashMap::new();
    for (_, _, nouns) in PARTS.iter() {
        for noun in nouns {
            m.entry(text::len(noun)).or_default().insert(noun.clone());
        }
    }
    m.into_iter().map(|(k, v)| (k, v.into_iter().collect())).collect()
});
static OTHER_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.kinds.", "OTHER_PEOPLE"));
static CONDITIONS: LazyLock<HashSet<String>> = LazyLock::new(|| t::set("bot.understanding.topics.", "CONDITIONS"));
static RE_IM: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi'?m\b", 0));
static RE_IVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi'?ve\b", 0));
static RE_WERE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwe'?re\b", 0));
static RE_WEVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwe'?ve\b", 0));

fn words_in(text_: &str) -> Vec<String> {
    rx("[a-z']+").findall_str(text_)
}

/// `_an_abscess_elsewhere(text)`: an abscess said of the gums or spreading is not toothache.
fn _an_abscess_elsewhere(text_: &str) -> bool {
    if !AN_ABSCESS.is_match(text_) {
        return false;
    }
    A_MOUTH_WORD.is_match(text_) || A_SPREADING_SIGN.is_match(text_) || ANOTHER_PART.is_match(text_) || ALREADY_DEALT_WITH.is_match(text_)
}

/// `_one_letter_apart(typed, spelled)`.
fn _one_letter_apart(typed: &str, spelled: &str) -> bool {
    let t_: Vec<char> = typed.chars().collect();
    let s: Vec<char> = spelled.chars().collect();
    if t_.len().abs_diff(s.len()) > 1 {
        return false;
    }
    if t_.len() == s.len() {
        return t_.iter().zip(s.iter()).filter(|(a, b)| a != b).count() == 1;
    }
    let (short, long) = if t_.len() <= s.len() { (&t_, &s) } else { (&s, &t_) };
    (0..long.len()).any(|at| long[..at].iter().chain(long[at + 1..].iter()).eq(short.iter()))
}

/// `_a_part_typed_with_one_letter_wrong(word)`: "stomache" -> "stomach".
fn _a_part_typed_with_one_letter_wrong(word: &str) -> Option<String> {
    let len = text::len(word);
    let first = word.chars().next();
    let mut fits: BTreeSet<String> = BTreeSet::new();
    for length in [len.wrapping_sub(1), len, len + 1] {
        for noun in PART_NOUNS_BY_LENGTH.get(&length).map(Vec::as_slice).unwrap_or(&[]) {
            if noun.chars().next() == first && _one_letter_apart(word, noun) {
                fits.insert(noun.clone());
                if fits.len() > 1 {
                    return None;
                }
            }
        }
    }
    fits.into_iter().next()
}

/// `_a_word_rather_than_a_typo(word)`.
fn _a_word_rather_than_a_typo(word: &str) -> bool {
    COMMON_WORDS.contains(word) || A_WORD_OF_ITS_OWN.contains(word) || CONDITIONS.contains(word)
}

/// `_spell_the_parts(text)`: body parts typed a letter off put right.
fn _spell_the_parts(text_: &str) -> String {
    let mut repaired = text_.to_string();
    let words: BTreeSet<String> = rx("[a-z]+").findall_str(&text_.to_lowercase()).into_iter().collect();
    for word in words {
        if text::len(&word) < 4 || _a_word_rather_than_a_typo(&word) {
            continue;
        }
        if let Some(part) = _a_part_typed_with_one_letter_wrong(&word) {
            repaired = rx(&format!(r"\b{word}\b")).sub(&part, &repaired);
        }
    }
    repaired
}

/// `_the_gut_is_settled(text)`.
fn _the_gut_is_settled(text_: &str) -> bool {
    let low = text_.to_lowercase();
    let words: HashSet<String> = words_in(&low).into_iter().collect();
    if !words.contains("stool") && !words.contains("stools") {
        return true;
    }
    A_GUT_ONLY.findall_str(&low).iter().any(|w| words.contains(w))
}

/// `read(text)`: the part of the speaker's body the line is about ("head", "stomach", ...).
pub fn read(text_: &str) -> Option<String> {
    if text::strip(text_).is_empty() {
        return None;
    }
    let text_ = _spell_the_parts(&text_.replace('|', " "));
    let text_ = text_.as_str();
    if A_LOOSE_STOOL.is_match(text_)
        && !A_SAMPLE.is_match(text_)
        && !A_POO_COLOUR.is_match(text_)
        && !A_CHAIR_WORD.is_match(text_)
        && !A_THING_IS_TERRIBLE.is_match(text_)
        && !_someone_else(text_)
        && _the_gut_is_settled(text_)
    {
        return Some("stomach".to_string());
    }
    let sample = A_SAMPLE.is_match(text_);
    for (name, pattern, nouns) in PARTS.iter() {
        if name == "teeth" && _an_abscess_elsewhere(text_) {
            continue;
        }
        if name == "stomach" && sample {
            continue;
        }
        for m in pattern.finditer(text_) {
            if _the_speakers(text_, m.end(), nouns, Some(m.start())) {
                return Some(name.clone());
            }
            if ASKING.is_match(text_) && !_someone_else(text_) {
                return Some(name.clone());
            }
            if _just_the_part(text_, &m) && !_someone_else(text_) {
                return Some(name.clone());
            }
        }
    }
    _said_about_itself(text_)
}

/// `_the_speakers(text, at, nouns, began=None)`: the part is the speaker's own.
fn _the_speakers(text_: &str, at: usize, nouns: &HashSet<String>, began: Option<usize>) -> bool {
    let words = words_in(&text_[..at]);
    let mut owner: Option<(String, usize)> = None;
    for (index, word) in words.iter().enumerate() {
        if OTHER_PEOPLE.contains(word) || word == "my" {
            owner = Some((word.clone(), index));
        }
    }
    if let Some((word, index)) = owner {
        if word != "my" {
            return false;
        }
        return words.get(index + 1).is_some_and(|w| nouns.contains(w));
    }
    let mut start = words.len().saturating_sub(6);
    while start > 0 && _A_VERB_NEEDING_ITS_SUBJECT.contains(&words[start]) {
        start -= 1;
    }
    let tail = words[start..].join(" ");
    let tail = RE_IM.sub("i am", &tail);
    let tail = RE_IVE.sub("i have", &tail);
    let tail = RE_WERE.sub("we are", &tail);
    let tail = RE_WEVE.sub("we have", &tail);
    if SPEAKER_ILL.is_match(&tail) || kinds::_speaker_before(text_, at) {
        return true;
    }
    let Some(began) = began else { return false };
    _the_speaker_is_the_subject(&words, words_in(&text_[..began]).len())
}

/// `_only_the_fillers_between(between)`.
fn _only_the_fillers_between(between: &[String]) -> bool {
    let mut between = between;
    if between.first().is_some_and(|w| _A_MAYBE.contains(w)) {
        if !(between.len() > 1 && _A_NO.contains(&between[1])) {
            return false;
        }
        between = &between[1..];
    }
    between.iter().all(|w| _BEFORE_THE_COMPLAINT.contains(w))
}

/// `_the_speaker_is_the_subject(words, began)`.
fn _the_speaker_is_the_subject(words: &[String], began: usize) -> bool {
    for index in (0..words.len()).rev() {
        if words[index] != "i" && words[index] != "we" {
            continue;
        }
        let between = if index + 1 <= began { &words[index + 1..began.min(words.len())] } else { &[][..] };
        if _only_the_fillers_between(between) {
            return true;
        }
    }
    false
}

/// `_an_unwell_word(words, at)`.
fn _an_unwell_word(words: &[String], at: usize) -> bool {
    let word = &words[at];
    if PLAIN_PAIN.contains(word) {
        return true;
    }
    if !PLAIN_UNWELL.contains(word) {
        return false;
    }
    if word != "off" {
        return true;
    }
    at + 1 >= words.len() || OFF_MAY_GO_ON.contains(&words[at + 1])
}

/// `_pain_after(words, index)`: a pain word follows the part ("knee hurts", "back is killing").
fn _pain_after(words: &[String], index: usize) -> bool {
    for step in 1..=PAIN_REACH as usize {
        let at = index + step;
        if at >= words.len() {
            return false;
        }
        if _an_unwell_word(words, at) {
            if words[at] == "off" && !(index > 0 && POSSESSIVE.contains(&words[index - 1])) {
                break;
            }
            return true;
        }
        if !PAIN_BETWEEN.contains(&words[at]) {
            break;
        }
    }
    _phrase_after(words, index)
}

/// `_phrase_after(words, index)`.
fn _phrase_after(words: &[String], index: usize) -> bool {
    'phrases: for phrase in PAIN_PHRASE.iter() {
        let mut at = index + 1;
        for word in phrase {
            if at < words.len() && PAIN_BETWEEN.contains(&words[at]) {
                at += 1;
            }
            if at >= words.len() || &words[at] != word {
                continue 'phrases;
            }
            at += 1;
        }
        return true;
    }
    false
}

/// `_said_about_itself(text)`: a part named with a pain word and no owner.
fn _said_about_itself(text_: &str) -> Option<String> {
    let found: Vec<Match> = rx("[a-z']+").finditer(text_);
    let words: Vec<String> = found.iter().map(|m| m.as_str().to_string()).collect();
    for (index, word) in words.iter().enumerate() {
        for (name, _pattern, nouns) in PARTS.iter() {
            if PLAIN_SKIP.contains(name) || PLAIN_SKIP_WORDS.contains(word) || _the_part_word(word, nouns).is_none() {
                continue;
            }
            let in_front = index > 0 && PAIN_ADJECTIVE.contains(&words[index - 1]);
            if !(in_front || _pain_after(&words, index)) {
                continue;
            }
            if _the_speakers(text_, found[index].end(), nouns, Some(found[index].start())) {
                return Some(name.clone());
            }
            if index > 0 && matches!(words[index - 1].as_str(), "my" | "our" | "me") {
                return Some(name.clone());
            }
            if index == 0 && _pain_after(&words, index) {
                return Some(name.clone());
            }
            if _someone_else(text_) {
                continue;
            }
            if in_front || _just_the_part_words(&words, index) {
                return Some(name.clone());
            }
        }
    }
    None
}

/// `_the_part_word(word, nouns)`.
fn _the_part_word(word: &str, nouns: &HashSet<String>) -> Option<String> {
    if nouns.contains(word) {
        return Some(word.to_string());
    }
    let bare = if let Some(b) = word.strip_suffix("'s") { b } else if let Some(b) = word.strip_suffix('s') { b } else { word };
    if nouns.contains(bare) { Some(bare.to_string()) } else { None }
}

/// `_just_the_part_words(words, index)`.
fn _just_the_part_words(words: &[String], index: usize) -> bool {
    words.iter().enumerate().filter(|(k, _)| *k != index).all(|(_, w)| FILLER.contains(w) || PLAIN_PAIN.contains(w) || PLAIN_UNWELL.contains(w) || PAIN_BETWEEN.contains(w))
}

/// `_just_the_part(text, match)`: the line is the part and filler ("headache since monday").
fn _just_the_part(text_: &str, m: &Match) -> bool {
    let rest = format!("{} {}", &text_[..m.start()], &text_[m.end()..]);
    let rest = text::strip(&rest);
    words_in(rest).iter().all(|w| FILLER.contains(w) || WHEN_IT_HAPPENED.contains(w) || _A_HELPER_BEFORE.contains(w) || pyuni::is_digit(w))
}

/// `_someone_else(text)`: the line is about somebody else.
fn _someone_else(text_: &str) -> bool {
    words_in(text_).iter().any(|w| OTHER_PEOPLE.contains(w) || OTHER_PEOPLE.contains(w.trim_end_matches('s')))
}
