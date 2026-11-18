//! Port of `bot/compose/onscreen.py`: a follow-up about the answer the bot has just given is
//! answered out of that answer, and whether a reply talks about the conversation itself.

use crate::compose::fallback;
use crate::compose::fallback_chain::{_choose_opt, caught};
use crate::data::{self, Value};
use crate::memory::history;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understanding::kinds::a_stop_ask;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.compose.onscreen.";

fn re_(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn strs_(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}
fn set_(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static ON_SCREEN_LINES: LazyLock<Vec<String>> = LazyLock::new(|| strs_("ON_SCREEN_LINES"));
static STANDS_BY_LINES: LazyLock<Vec<String>> = LazyLock::new(|| strs_("STANDS_BY_LINES"));
static ON_SCREEN_ASK: LazyLock<&'static Re> = LazyLock::new(|| re_("ON_SCREEN_ASK"));
static A_THING_NAMED_IN_FRONT: LazyLock<&'static Re> = LazyLock::new(|| re_("A_THING_NAMED_IN_FRONT"));
static A_STEP_IN_IT: LazyLock<&'static Re> = LazyLock::new(|| re_("A_STEP_IN_IT"));
static NOT_TO_HAND_BACK: LazyLock<&'static Re> = LazyLock::new(|| re_("NOT_TO_HAND_BACK"));
static ASKS_WHAT_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| re_("ASKS_WHAT_TO_DO"));
static POINTS_BACK: LazyLock<&'static Re> = LazyLock::new(|| re_("POINTS_BACK"));
static NAMES_NOTHING_OF_ITS_OWN: LazyLock<HashSet<String>> = LazyLock::new(|| set_("NAMES_NOTHING_OF_ITS_OWN"));
static ASKS_RATHER_THAN_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| set_("ASKS_RATHER_THAN_NAMES"));
static THE_THINGS_IT_COULD_BE_ABOUT: LazyLock<HashSet<String>> =
    LazyLock::new(|| set_("THE_THINGS_IT_COULD_BE_ABOUT"));
static WHO_TO_TALK_TO: LazyLock<&'static Re> = LazyLock::new(|| re_("WHO_TO_TALK_TO"));
static A_HELP_IN_IT: LazyLock<&'static Re> = LazyLock::new(|| re_("A_HELP_IN_IT"));
static A_BUY_ASK: LazyLock<&'static Re> = LazyLock::new(|| re_("A_BUY_ASK"));
static A_PURCHASE_IN_IT: LazyLock<&'static Re> = LazyLock::new(|| re_("A_PURCHASE_IN_IT"));
static A_LEAD_IN: LazyLock<&'static Re> = LazyLock::new(|| re_("A_LEAD_IN"));
static QUOTED: LazyLock<&'static Re> = LazyLock::new(|| re_("QUOTED"));
static A_DECISION_ASK: LazyLock<&'static Re> = LazyLock::new(|| re_("A_DECISION_ASK"));
static LONG_SENTENCE: LazyLock<usize> =
    LazyLock::new(|| data::get(&format!("{M}LONG_SENTENCE")).as_int().unwrap() as usize);
static A_WORD_THAT_ONLY_JOINS: LazyLock<HashSet<String>> = LazyLock::new(|| set_("A_WORD_THAT_ONLY_JOINS"));
static _DOUBLED: LazyLock<String> = LazyLock::new(|| data::string(&format!("{M}_DOUBLED")));
static EFFORT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set_("EFFORT_WORDS"));
static CLAUSE_AFTER_A_COLON: LazyLock<&'static Re> = LazyLock::new(|| re_("CLAUSE_AFTER_A_COLON"));
static A_LONG_SENTENCE_SPLIT: LazyLock<&'static Re> = LazyLock::new(|| re_("A_LONG_SENTENCE_SPLIT"));
static A_GO_ON_ASK: LazyLock<&'static Re> = LazyLock::new(|| re_("A_GO_ON_ASK"));
static A_LIMIT_IN_IT: LazyLock<&'static Re> = LazyLock::new(|| re_("A_LIMIT_IN_IT"));
static ASKS_WHAT_DECIDES: LazyLock<&'static Re> = LazyLock::new(|| re_("ASKS_WHAT_DECIDES"));
static _ABOUT_THE_CONVERSATION: LazyLock<&'static Re> = LazyLock::new(|| re_("_ABOUT_THE_CONVERSATION"));
/// `fallback.PUNT` (defined in fallback_data, read through fallback).
static PUNT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback_data.PUNT"));

/// `re.split(r"(?<=[.!?])\s+", ...)`, written inline in `the_answer_on_screen`.
static SENTENCE_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", 0));
/// `re.sub(r"[\"“][^\"”]*[\"”]", " ", ...)`, written inline in `talks_about_the_conversation`.
static QUOTED_PART: LazyLock<Re> = LazyLock::new(|| Re::new("[\"\u{201c}][^\"\u{201d}]*[\"\u{201d}]", 0));

/// Python: bot/compose/onscreen.py:215 `_stem(word)`
pub fn _stem(word: &str) -> String {
    let mut word: Vec<char> = word.chars().collect();
    for ending in ["ing", "ed", "es", "s"] {
        let n = ending.chars().count();
        if word.len() as isize - n as isize >= 4 && word.iter().collect::<String>().ends_with(ending) {
            word.truncate(word.len() - n);
            break;
        }
    }
    if word.len() >= 4 && word[word.len() - 1] == word[word.len() - 2] && _DOUBLED.contains(word[word.len() - 1]) {
        word.pop();
    }
    word.into_iter().collect()
}

/// Python: bot/compose/onscreen.py:231 `_the_same_word(word, said)`
pub fn _the_same_word(word: &str, said: &[String]) -> bool {
    let has = |w: &str| said.iter().any(|s| s == w);
    if has(word) || has(text::rstrip_chars(word, "s")) || has(&format!("{word}s")) {
        return true;
    }
    if _stems(said).contains(&_stem(word)) {
        return true;
    }
    text::len(word) >= 5 && said.iter().any(|other| text::len(other) >= 5 && text::slice(word, 0, Some(5)) == text::slice(other, 0, Some(5)))
}

/// Python: bot/compose/onscreen.py:241 `_stems(words)` (a set in Python; membership is all it is
/// used for)
pub fn _stems(words: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in words {
        let s = _stem(word);
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// Python: bot/compose/onscreen.py:252 `_names_something_new(mine, said, in_the_chat)`
pub fn _names_something_new(mine: &[String], said: &[String], in_the_chat: &[String]) -> bool {
    let last = mine.last().map(String::as_str).unwrap_or("");
    for word in mine {
        if _the_same_word(word, said) {
            continue;
        }
        if ASKS_RATHER_THAN_NAMES.contains(word) || EFFORT_WORDS.contains(word) {
            continue;
        }
        if THE_THINGS_IT_COULD_BE_ABOUT.contains(word) && _the_same_word(word, in_the_chat) {
            continue;
        }
        if word != last && A_STEP_IN_IT.search(word).is_some() {
            continue;
        }
        return true;
    }
    false
}

/// `set(words)` kept as a list without repeats (only membership is read).
fn unique(words: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    words.into_iter().filter(|w| seen.insert(w.clone())).collect()
}

/// Python: bot/compose/onscreen.py:277 `_the_words_in_the_chat(memory, answer)`
pub fn _the_words_in_the_chat(memory: &Memory, answer: &str) -> Vec<String> {
    let mut words = fallback::_flat_words(answer);
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(2)..] {
        words.extend(fallback::_flat_words(&turn.user));
    }
    unique(words)
}

/// Python: bot/compose/onscreen.py:286 `_the_answer_to_hand_back(memory)`
pub fn _the_answer_to_hand_back(memory: &Memory) -> String {
    history::the_answer_on_screen(memory, 3, &["answers".to_string(), "safety".to_string()])
}

/// Python: bot/compose/onscreen.py:308 `_the_answer_is_a_card(memory)`
pub fn _the_answer_is_a_card(memory: &Memory) -> bool {
    history::the_answer_is_a_card(memory, &Value::Int(3))
}

/// Python: bot/compose/onscreen.py:328 `_the_parts(sentence)`
pub fn _the_parts(sentence: &str) -> Vec<String> {
    if text::split(sentence).len() <= *LONG_SENTENCE {
        return vec![sentence.to_string()];
    }
    let parts: Vec<String> = A_LONG_SENTENCE_SPLIT
        .split(sentence)
        .iter()
        .map(|one| text::strip(one).to_string())
        .filter(|one| !one.is_empty())
        .collect();
    if parts.len() < 2 {
        return vec![sentence.to_string()];
    }
    let mut out = vec![parts[0].clone()];
    out.extend(parts[1..].iter().map(|one| text::upper_first(one)));
    out
}

/// Python: bot/compose/onscreen.py:338 `_the_sentence_to_hand_back(sentence, mine, said)`
pub fn _the_sentence_to_hand_back(sentence: &str, mine: &[String], said: &[String]) -> String {
    let mut sentence = sentence.to_string();
    if let Some(tail) = CLAUSE_AFTER_A_COLON.search(&sentence) {
        let clause = text::strip(tail.str(1)).to_string();
        let clause_words = unique(fallback::_flat_words(&clause));
        let in_the_clause = mine.iter().any(|word| _the_same_word(word, &clause_words));
        if text::split(&clause).len() >= 6
            && (in_the_clause || mine.iter().filter(|word| _the_same_word(word, said)).count() == 0)
        {
            sentence = text::upper_first(&clause);
        }
    }
    sentence
}

/// Python: bot/compose/onscreen.py:384 `the_answer_on_screen(text, memory)`
pub fn the_answer_on_screen(text: &str, memory: &Memory) -> String {
    // `except Exception: return ""` - a reading that fails leaves the composer's reply alone.
    caught(|| the_answer_on_screen_inner(text, memory)).unwrap_or_default()
}

fn the_answer_on_screen_inner(text_: &str, memory: &Memory) -> String {
    let clean = text::squash(&text::lower(text_));
    if clean.is_empty() || ON_SCREEN_ASK.match_(&clean).is_none() {
        return String::new();
    }
    if A_THING_NAMED_IN_FRONT.match_(&clean).is_some() {
        return String::new();
    }
    if text::split(&clean).len() > 8 {
        return String::new();
    }
    if fallback::_hedged_recently(memory, 3) {
        return String::new();
    }
    let answer = text::squash(&_the_answer_to_hand_back(memory));
    if answer.is_empty() {
        return String::new();
    }
    let sentences: Vec<String> = SENTENCE_SPLIT
        .split(&answer)
        .iter()
        .map(|one| text::strip(one).to_string())
        .filter(|one| !one.is_empty())
        .collect();
    let mine: Vec<String> =
        fallback::content_words(&clean).into_iter().filter(|w| !A_WORD_THAT_ONLY_JOINS.contains(w)).collect();
    let said = unique(fallback::_flat_words(&answer));
    let asks_who_to_talk_to = WHO_TO_TALK_TO.search(&clean).is_some() && _the_answer_is_a_card(memory);
    let names_nothing = |mine: &[String]| mine.iter().all(|w| NAMES_NOTHING_OF_ITS_OWN.contains(w));
    if !asks_who_to_talk_to
        && !names_nothing(&mine)
        && _names_something_new(&mine, &said, &_the_words_in_the_chat(memory, &answer))
    {
        return String::new();
    }
    let (mut best, mut best_score, mut first_step, mut opening) = (String::new(), 0usize, String::new(), String::new());
    let mut steps: Vec<String> = Vec::new();
    for sentence in &sentences {
        if text::split(sentence).len() < 5 || sentence.ends_with('?') {
            continue;
        }
        if NOT_TO_HAND_BACK.match_(sentence).is_some() {
            continue;
        }
        if QUOTED.search(sentence).is_some() {
            continue;
        }
        let low = text::lower(sentence);
        if PUNT.iter().any(|punt| low.contains(punt.as_str())) {
            continue;
        }
        let step = A_STEP_IN_IT.search(sentence).is_some();
        if step {
            steps.push(sentence.clone());
            if first_step.is_empty() {
                first_step = sentence.clone();
            }
        }
        if opening.is_empty() {
            opening = sentence.clone();
        }
        for piece in _the_parts(sentence) {
            if text::split(&piece).len() > *LONG_SENTENCE {
                continue;
            }
            let words = unique(fallback::_flat_words(&piece));
            let score = mine.iter().filter(|word| _the_same_word(word, &words)).count();
            if score > 0 && score > best_score {
                best = piece;
                best_score = score;
            }
        }
    }
    if best.is_empty()
        && !first_step.is_empty()
        && ASKS_WHAT_TO_DO.search(&clean).is_some()
        && POINTS_BACK.search(&clean).is_some()
    {
        best = first_step.clone();
    }
    let mut says_it_again = false;
    if asks_who_to_talk_to && best.is_empty() {
        for sentence in &sentences {
            if A_HELP_IN_IT.search(sentence).is_some() && NOT_TO_HAND_BACK.match_(sentence).is_none() {
                best = sentence.clone();
                break;
            }
        }
    }
    if best.is_empty() && !opening.is_empty() && asks_who_to_talk_to {
        best = opening.clone();
        says_it_again = true;
    }
    if best.is_empty() && !mine.is_empty() && names_nothing(&mine) {
        best = opening.clone();
        says_it_again = true;
    }
    if best.is_empty() && A_GO_ON_ASK.search(&clean).is_some() {
        best = steps
            .iter()
            .find(|one| A_LIMIT_IN_IT.search(one).is_some() || ASKS_WHAT_DECIDES.search(one).is_some())
            .cloned()
            .unwrap_or_default();
    }
    if best.is_empty() {
        return String::new();
    }
    if A_DECISION_ASK.search(&clean).is_some() && A_STEP_IN_IT.search(&best).is_none() {
        return String::new();
    }
    if A_BUY_ASK.search(&clean).is_some() && A_PURCHASE_IN_IT.search(&best).is_none() {
        return String::new();
    }
    let already = fallback::_flat_words(&best).join(" ");
    let turns = &memory.turns;
    let mut source: isize = -1;
    for (place, turn) in turns.iter().enumerate() {
        if turn.bot == answer {
            source = place as isize;
        }
    }
    for turn in &turns[(source + 1) as usize..] {
        if !already.is_empty() && fallback::_flat_words(&turn.bot).join(" ").contains(already.as_str()) {
            return String::new();
        }
    }
    if !says_it_again {
        best = _the_sentence_to_hand_back(&best, &mine, &said);
    }
    let first_said = fallback::_flat_words(&answer).join(" ");
    if !says_it_again && first_said.starts_with(&text::slice(&fallback::_flat_words(&best).join(" "), 0, Some(60))) {
        return String::new();
    }
    let given = A_LEAD_IN.sub("", &best);
    if !given.is_empty() && given != best {
        best = text::upper_first(&given);
    }
    let frames: Vec<String> = if a_stop_ask(&clean) {
        STANDS_BY_LINES.clone()
    } else if says_it_again {
        ON_SCREEN_LINES[..1].to_vec()
    } else {
        ON_SCREEN_LINES.clone()
    };
    // `_choose` gives None when the conversation has used up the pool: `None.replace` raises and
    // the except gives "".
    let Some(chosen) = _choose_opt(&frames, memory, &clean, None) else {
        return String::new();
    };
    let reply = chosen.replace("{said}", &best);
    let flat = fallback::_flat_words(&reply).join(" ");
    let recent: Vec<String> =
        turns[turns.len().saturating_sub(3)..].iter().map(|t| fallback::_flat_words(&t.bot).join(" ")).collect();
    if recent.contains(&flat) {
        return String::new();
    }
    reply
}

/// Python: bot/compose/onscreen.py:551 `earlier_turn(memory)`
pub fn earlier_turn(memory: &Memory) -> bool {
    !memory.turns.is_empty()
}

/// Python: bot/compose/onscreen.py:561 `talks_about_the_conversation(reply)`
pub fn talks_about_the_conversation(reply: &str) -> bool {
    let unquoted = QUOTED_PART.sub(" ", reply);
    _ABOUT_THE_CONVERSATION.search(&unquoted).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    // Expected values from `harness/pycall.py bot.compose.onscreen ...`.
    #[test]
    fn stems_and_words() {
        assert_eq!(_stem("running"), "run");
        assert_eq!(_stem("stopped"), "stop");
        assert_eq!(_stem("less"), "less");
        assert_eq!(_stem("stuff"), "stuff");
        assert_eq!(_stem("cats"), "cats");
        assert_eq!(_stem("pricking"), "prick");
        assert!(_the_same_word("pricked", &s(&["pricking"])));
        assert!(!_the_same_word("frame", &s(&["chain", "slipping"])));
        assert!(_the_same_word("leaves", &s(&["leave"])));
        assert!(_the_same_word("machines", &s(&["machinery"])));
    }

    #[test]
    fn parts_and_talk() {
        assert_eq!(_the_parts("a short one: with a colon"), s(&["a short one: with a colon"]));
        assert!(talks_about_the_conversation("Hello again. Where were we?"));
        assert!(!talks_about_the_conversation("come back with \"sorry - where were we?\""));
        assert!(!talks_about_the_conversation("Not the last time I looked"));
        let long = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty twenty-one twenty-two twenty-three twenty-four twenty-five twenty-six twenty-seven: twenty-eight twenty-nine thirty thirty-one thirty-two thirty-three thirty-four thirty-five thirty-six thirty-seven thirty-eight; thirty-nine forty forty-one forty-two forty-three forty-four forty-five forty-six forty-seven forty-eight forty-nine fifty fifty-one fifty-two fifty-three";
        let parts = _the_parts(long);
        assert_eq!(parts.len(), 3);
        assert!(parts[1].starts_with("Twenty-eight") && parts[2].starts_with("Thirty-nine"));
    }
}
