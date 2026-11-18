//! Port of `bot/understanding/kinds.py`: what kind of line it is (a question, a greeting, news,
//! a feeling), what it asks for, and who a state belongs to.

use crate::understanding::normalize::Line;
use crate::understood::{CareHome, Suggestion, Summons, TakingUp, Understood};
use serde::{Deserialize, Serialize};

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

/// `_speaker_before(text, at)`: `at` is a byte offset into `text` (Python's index, as Rust
/// regex matches give it).
pub fn _speaker_before(text: &str, at: usize) -> bool {
    let _ = (text, at);
    todo!("kinds::_speaker_before")
}

/// `_asks_back(norm)`.
pub fn _asks_back(norm: &[String]) -> bool {
    let _ = norm;
    todo!("kinds::_asks_back")
}

/// `a_bad_verdict(norm, index)`.
pub fn a_bad_verdict(norm: &[String], index: usize) -> bool {
    let _ = (norm, index);
    todo!("kinds::a_bad_verdict")
}

/// `small_talk_only(norm)`.
pub fn small_talk_only(norm: &[String]) -> bool {
    let _ = norm;
    todo!("kinds::small_talk_only")
}

/// `feeling_by_sound(word)`.
pub fn feeling_by_sound(word: &str) -> Option<String> {
    let _ = word;
    todo!("kinds::feeling_by_sound")
}

/// `a_broken_heart_shape(text)`.
pub fn a_broken_heart_shape(text: &str) -> bool {
    let _ = text;
    todo!("kinds::a_broken_heart_shape")
}
