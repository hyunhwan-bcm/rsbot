//! Port of `bot/understanding/references.py`: what "it", "that", "they" point back at.

use crate::understanding::kinds::Read;
use crate::understanding::normalize::Line;
use crate::understood::{Entity, Ref, Understood};

/// `resolve(line, read, prev_line=None, prev_entities=None, prev_topic=None)`.
pub fn resolve(line: &Line, read: &Read, prev_line: Option<&Line>, prev_entities: &[Entity], prev_topic: Option<&str>) -> Ref {
    let _ = (line, read, prev_line, prev_entities, prev_topic);
    todo!("references::resolve")
}

/// `_target(prev_line, prev_entities, prev_topic)`: `(name, kind)`.
pub fn _target(prev_line: &Line, prev_entities: &[Entity], prev_topic: Option<&str>) -> (Option<String>, Option<String>) {
    let _ = (prev_line, prev_entities, prev_topic);
    todo!("references::_target")
}

/// `person_is_new(line, prev_entities)`.
pub fn person_is_new(line: &Line, prev_entities: &[Entity]) -> bool {
    let _ = (line, prev_entities);
    todo!("references::person_is_new")
}

/// `the_animal_in_hand(ref, words)`.
pub fn the_animal_in_hand(reference: &Ref, words: &[String]) -> String {
    let _ = (reference, words);
    todo!("references::the_animal_in_hand")
}

/// `the_animal_the_chat_is_on(words, last_read)`.
pub fn the_animal_the_chat_is_on(words: &[String], last_read: Option<&Understood>) -> String {
    let _ = (words, last_read);
    todo!("references::the_animal_the_chat_is_on")
}
