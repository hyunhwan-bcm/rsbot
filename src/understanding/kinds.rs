//! Port of `bot/understanding/kinds.py`: what kind of line it is (a question, a greeting, news,
//! a feeling), what it asks for, and who a state belongs to.

use crate::understanding::normalize::Line;
use crate::understood::{CareHome, Suggestion, Summons, TakingUp, Understood};
use crate::data;
use crate::py::re::Re;
use crate::py::text;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.understanding.kinds.";

fn set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn re(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}

/// What `classify(line)` returns (Python's dict, same keys). `_read_line` changes `intent`,
/// `asks`, `replies_to_bot`, `about_bot` and `about_bot_kind` afterwards.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Read {
    pub kind: String,
    pub intent: String,
    pub asks: Vec<String>,
    pub tag: bool,
    pub emotion: Option<String>,
    pub state_owner: Option<String>,
    pub shortage: Option<String>,
    pub polarity: i64,
    pub short: bool,
    pub first_person: bool,
    pub asks_opinion: bool,
    pub asks_recommendation: bool,
    pub asks_steps: bool,
    pub help_ask: bool,
    pub asks_a_verdict: bool,
    pub about_bot: bool,
    pub about_bot_kind: Option<String>,
    pub counts_the_bots_own_day: bool,
    pub replies_to_bot: bool,
    pub agrees_to_try: bool,
    pub wants_a_quiz: bool,
    pub agreement: Option<String>,
    pub summons: Option<Summons>,
    pub asks_back: bool,
    pub asks_what_was_meant: bool,
    pub excludes: Vec<String>,
    pub ruled_out: Vec<String>,
    pub constraint: bool,
    pub suggestion: Option<Suggestion>,
}

/// The keys of a reading that `over_time`, `taking_up` and `is_a_routine` read with
/// `read.get(...)`: they are handed either `classify`'s dict or the finished reading.
pub trait ReadView {
    fn kind(&self) -> Option<&str>;
    fn intent(&self) -> Option<&str>;
    fn asks(&self) -> &[String];
    /// `read.get("followup")` (classify's dict has no such key: false).
    fn followup(&self) -> bool;
    /// `read.get("text")` (None for classify's dict).
    fn text(&self) -> Option<&str>;
    /// `read.get("clean")` (None for classify's dict).
    fn clean(&self) -> Option<&str>;
}

impl ReadView for Read {
    fn kind(&self) -> Option<&str> {
        Some(&self.kind)
    }
    fn intent(&self) -> Option<&str> {
        Some(&self.intent)
    }
    fn asks(&self) -> &[String] {
        &self.asks
    }
    fn followup(&self) -> bool {
        false
    }
    fn text(&self) -> Option<&str> {
        None
    }
    fn clean(&self) -> Option<&str> {
        None
    }
}

impl ReadView for Understood {
    fn kind(&self) -> Option<&str> {
        Some(&self.kind)
    }
    fn intent(&self) -> Option<&str> {
        Some(&self.intent)
    }
    fn asks(&self) -> &[String] {
        &self.asks
    }
    fn followup(&self) -> bool {
        self.followup
    }
    fn text(&self) -> Option<&str> {
        Some(&self.text)
    }
    fn clean(&self) -> Option<&str> {
        Some(&self.clean)
    }
}

/// `classify(line)`.
pub fn classify(line: &Line) -> Read {
    let _ = line;
    todo!("kinds::classify")
}

/// `after_any_link(norm)`: the words after "any"/"anything" ... (empty when none).
pub fn after_any_link(norm: &[String]) -> Vec<String> {
    let _ = norm;
    todo!("kinds::after_any_link")
}

/// `over_time(line, read, previous=None, carrying_on=False)`: `(over_time, over_time_what)`;
/// None only when `carrying_on` and nothing carries on.
pub fn over_time(line: &Line, read: &dyn ReadView, previous: Option<&Understood>, carrying_on: bool) -> Option<(Option<bool>, Option<String>)> {
    let _ = (line, read, previous, carrying_on);
    todo!("kinds::over_time")
}

/// `asks_how_i_am(norm)`.
pub fn asks_how_i_am(norm: &[String]) -> bool {
    let _ = norm;
    todo!("kinds::asks_how_i_am")
}

/// `taking_up(line, read=None)`.
pub fn taking_up(line: &Line, read: Option<&dyn ReadView>) -> Option<TakingUp> {
    let _ = (line, read);
    todo!("kinds::taking_up")
}

/// `shift_worked(text)`.
pub fn shift_worked(text: &str) -> Option<String> {
    let _ = text;
    todo!("kinds::shift_worked")
}

/// `a_care_home(text)`.
pub fn a_care_home(text: &str) -> Option<CareHome> {
    let _ = text;
    todo!("kinds::a_care_home")
}

/// `a_home_carried_forward(text, earlier_lines)`.
pub fn a_home_carried_forward(text: &str, earlier_lines: &[String]) -> Option<CareHome> {
    let _ = (text, earlier_lines);
    todo!("kinds::a_home_carried_forward")
}

/// `the_setup(line)`.
pub fn the_setup(line: &Line) -> Option<String> {
    let _ = line;
    todo!("kinds::the_setup")
}

/// `is_a_routine(line, read=None)`.
pub fn is_a_routine(line: &Line, read: Option<&dyn ReadView>) -> bool {
    let _ = (line, read);
    todo!("kinds::is_a_routine")
}

/// `noun_phrase_request(line)`.
pub fn noun_phrase_request(line: &Line) -> bool {
    let _ = line;
    todo!("kinds::noun_phrase_request")
}

/// `opening_line(norm)`.
pub fn opening_line(norm: &[String]) -> bool {
    let _ = norm;
    todo!("kinds::opening_line")
}

/// `talk_frame(norm)`.
pub fn talk_frame(norm: &[String]) -> bool {
    let _ = norm;
    todo!("kinds::talk_frame")
}

/// `the_line_after_a_courtesy(norm)`: the words after a thank-you (empty when none).
pub fn the_line_after_a_courtesy(norm: &[String]) -> Vec<String> {
    let _ = norm;
    todo!("kinds::the_line_after_a_courtesy")
}

/// `_a_fall_that_is_not_one(rest)`.
pub fn _a_fall_that_is_not_one(rest: &[String]) -> bool {
    let _ = rest;
    todo!("kinds::_a_fall_that_is_not_one")
}

/// `_a_step_done(rest)`.
pub fn _a_step_done(rest: &[String]) -> bool {
    let _ = rest;
    todo!("kinds::_a_step_done")
}

/// `_the_step_went_wrong(rest)`.
pub fn _the_step_went_wrong(rest: &[String]) -> bool {
    let _ = rest;
    todo!("kinds::_the_step_went_wrong")
}


/// `_asks_back(norm)`.
pub fn _asks_back(norm: &[String]) -> bool {
    let _ = norm;
    todo!("kinds::_asks_back")
}




/// `a_broken_heart_shape(text)`.
pub fn a_broken_heart_shape(text: &str) -> bool {
    let _ = text;
    todo!("kinds::a_broken_heart_shape")
}

static FEELING_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| set("FEELING_FILLER"));
static MOOD_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| set("MOOD_NOUNS"));
static SPEAKER_TAIL: LazyLock<&'static Re> = LazyLock::new(|| re("SPEAKER_TAIL"));
static VERDICT_NOUNS: LazyLock<HashSet<String>> = LazyLock::new(|| set("VERDICT_NOUNS"));
static VERDICT_ADJECTIVES: LazyLock<HashSet<String>> = LazyLock::new(|| set("VERDICT_ADJECTIVES"));
static VERDICT_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| set("VERDICT_FILLER"));
static COPULAS: LazyLock<HashSet<String>> = LazyLock::new(|| set("COPULAS"));
static PRONOUN_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| set("PRONOUN_SUBJECT"));
static ASKING_SUBJECT: LazyLock<HashSet<String>> = LazyLock::new(|| set("ASKING_SUBJECT"));
static NOT_A_THING: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_A_THING"));
static SMALL_TALK_LEFTOVER: LazyLock<HashSet<String>> = LazyLock::new(|| set("SMALL_TALK_LEFTOVER"));
static SMALL_TALK_WORDS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get(&format!("{M}SMALL_TALK_WORDS")).items().iter().map(|p| p.strs()).collect());
/// `EMOTION_WORDS`: feeling name -> its cue words, in Python's dict order.
pub(crate) static EMOTION_WORDS: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&format!("{M}EMOTION_WORDS")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static RE_IM: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi'?m\b", 0));
static RE_IVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi'?ve\b", 0));

/// The words of a text with Python's `word.strip('.,!?;:-|"\'').lower()`.
fn stripped_words(text: &str) -> Vec<String> {
    text::split_ref(text).into_iter().map(|w| text::strip_chars(w, ".,!?;:-|\"'").to_lowercase()).collect()
}

/// `_speaker_before(text, at)`: the words just before `at` (a byte offset) make the speaker the
/// one in the state ("i am so", "i feel", "my mood is").
pub fn _speaker_before(text: &str, at: usize) -> bool {
    let mut words = stripped_words(&text[..at.min(text.len())]);
    while words.last().is_some_and(|w| FEELING_FILLER.contains(w)) {
        words.pop();
    }
    let tail = words[words.len().saturating_sub(5)..].join(" ");
    let tail = RE_IM.sub("i am", &tail);
    let tail = RE_IVE.sub("i have", &tail);
    if SPEAKER_TAIL.is_match(&tail) {
        return true;
    }
    let recent = &words[words.len().saturating_sub(4)..];
    if let Some(my) = recent.iter().position(|w| w == "my") {
        for word in &recent[my + 1..] {
            if MOOD_NOUNS.contains(word) {
                return true;
            }
        }
    }
    false
}

/// `a_bad_verdict(norm, index)`: the word at `index` is a verdict on a thing ("the film was
/// rubbish"), not a feeling.
pub fn a_bad_verdict(norm: &[String], index: usize) -> bool {
    if index >= norm.len() || (!VERDICT_NOUNS.contains(&norm[index]) && !VERDICT_ADJECTIVES.contains(&norm[index])) {
        return false;
    }
    let mut back = index as isize - 1;
    while back >= 0 && VERDICT_FILLER.contains(&norm[back as usize]) {
        back -= 1;
    }
    if back < 0 || !COPULAS.contains(&norm[back as usize]) {
        return false;
    }
    let subject = back - 1;
    if subject < 0 {
        return false;
    }
    let s = &norm[subject as usize];
    !(PRONOUN_SUBJECT.contains(s) || ASKING_SUBJECT.contains(s) || COPULAS.contains(s) || NOT_A_THING.contains(s))
}

/// `small_talk_only(norm)`: the line is a small-talk phrase and nothing else ("how are you").
pub fn small_talk_only(norm: &[String]) -> bool {
    if norm.is_empty() || norm.len() > 10 {
        return false;
    }
    for phrase in SMALL_TALK_WORDS.iter() {
        let size = phrase.len();
        if size > norm.len() {
            continue;
        }
        for at in 0..=(norm.len() - size) {
            if norm[at..at + size] != phrase[..] {
                continue;
            }
            let rest_left = norm[..at].iter().chain(norm[at + size..].iter());
            if rest_left.into_iter().all(|w| SMALL_TALK_LEFTOVER.contains(w)) {
                return true;
            }
        }
    }
    false
}

/// `_skeleton(word)`: the consonants of a word ("anxous" and "anxious" share one).
pub fn _skeleton(word: &str) -> String {
    word.to_lowercase().chars().filter(|c| c.is_alphabetic() && !"aeiouy".contains(*c)).collect()
}

/// `_sound_cues()`: (skeleton -> feeling -> cues, every one-word cue).
fn _sound_cues() -> &'static (HashMap<String, BTreeMap<String, Vec<String>>>, HashSet<String>) {
    static CUES: LazyLock<(HashMap<String, BTreeMap<String, Vec<String>>>, HashSet<String>)> = LazyLock::new(|| {
        let mut index: HashMap<String, BTreeMap<String, Vec<String>>> = HashMap::new();
        let mut known = HashSet::new();
        for (name, cues) in EMOTION_WORDS.iter() {
            for cue in cues {
                if cue.contains(' ') {
                    continue;
                }
                known.insert(cue.clone());
                if text::len(cue) < 4 {
                    continue;
                }
                index.entry(_skeleton(cue)).or_default().entry(name.clone()).or_default().push(cue.clone());
            }
        }
        (index, known)
    });
    &CUES
}

/// `feeling_by_sound(word)`: the one feeling a misspelt word sounds like ("anxous" -> anxious).
pub fn feeling_by_sound(word: &str) -> Option<String> {
    static COMMON_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.lexicon.COMMON_NAMES"));
    let low = text::strip_chars(text::strip(word), ".,!?;:'\"()").to_lowercase();
    if text::len(&low) < 4 || !text::is_alpha(&low) {
        return None;
    }
    let (index, known) = _sound_cues();
    if known.contains(&low) || COMMON_NAMES.contains(&low) {
        return None;
    }
    let first: String = low.chars().take(1).collect();
    let found: BTreeSet<&String> = match index.get(&_skeleton(&low)) {
        Some(matches) => matches.iter().filter(|(_, cues)| cues.iter().any(|c| c.chars().take(1).collect::<String>() == first)).map(|(n, _)| n).collect(),
        None => BTreeSet::new(),
    };
    if found.len() == 1 { found.into_iter().next().cloned() } else { None }
}
