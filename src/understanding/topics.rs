//! Port of `bot/understanding/topics.py`: what the line is about (topics, entities, numbers).

use crate::understanding::normalize::Line;
use crate::understood::{Entity, Number};

/// `detect_entities(line, text=None)`.
pub fn detect_entities(line: &Line, text: Option<&str>) -> Vec<Entity> {
    let _ = (line, text);
    todo!("topics::detect_entities")
}

/// `detect_numbers(line)`.
pub fn detect_numbers(line: &Line) -> Vec<Number> {
    let _ = line;
    todo!("topics::detect_numbers")
}

/// `detect_topic(line, inherited=None, entities=None, asks=None)`: `(topic, score)`.
pub fn detect_topic(line: &Line, inherited: Option<&str>, entities: Option<&[Entity]>, asks: Option<bool>) -> (Option<String>, i64) {
    let _ = (line, inherited, entities, asks);
    todo!("topics::detect_topic")
}

/// `rank_topics(line, inherited=None, entities=None, asks=None)`: `[(topic, score)]`.
pub fn rank_topics(line: &Line, inherited: Option<&str>, entities: Option<&[Entity]>, asks: Option<bool>) -> Vec<(String, i64)> {
    let _ = (line, inherited, entities, asks);
    todo!("topics::rank_topics")
}

/// `subject_topic(line, entities=None)`.
pub fn subject_topic(line: &Line, entities: Option<&[Entity]>) -> Option<String> {
    let _ = (line, entities);
    todo!("topics::subject_topic")
}

/// `the_words_that_said(line, topic, entities=None)`: `[(word, weight)]`.
pub fn the_words_that_said(line: &Line, topic: Option<&str>, entities: Option<&[Entity]>) -> Vec<(String, i64)> {
    let _ = (line, topic, entities);
    todo!("topics::the_words_that_said")
}

/// `the_warmth_said(norm)`.
pub fn the_warmth_said(norm: &[String]) -> Option<String> {
    let _ = norm;
    todo!("topics::the_warmth_said")
}

/// `own_output_word(norm)`: the position of the word, or None.
pub fn own_output_word(norm: &[String]) -> Option<usize> {
    let _ = norm;
    todo!("topics::own_output_word")
}

/// `person_names(line)`.
pub fn person_names(line: &Line) -> Vec<String> {
    let _ = line;
    todo!("topics::person_names")
}

/// `purse_till_at(norm, text='')`: the position, or None.
pub fn purse_till_at(norm: &[String], text: &str) -> Option<usize> {
    let _ = (norm, text);
    todo!("topics::purse_till_at")
}

/// `tight_on_money(norm, kind=None, text='')`.
pub fn tight_on_money(norm: &[String], kind: Option<&str>, text: &str) -> bool {
    let _ = (norm, kind, text);
    todo!("topics::tight_on_money")
}

/// `unclear_subject(norm, inherited=None, previous=None)`.
pub fn unclear_subject(norm: &[String], inherited: Option<&str>, previous: Option<&Line>) -> Option<String> {
    let _ = (norm, inherited, previous);
    todo!("topics::unclear_subject")
}

/// `unsure_words(norm, topic)`.
pub fn unsure_words(norm: &[String], topic: Option<&str>) -> Vec<String> {
    let _ = (norm, topic);
    todo!("topics::unsure_words")
}
