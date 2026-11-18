//! STUB for `bot/memory/facts.py` (owned by another workstream): only the two functions the
//! understanding engine calls (`kinds._says_what_they_are`, `kinds._a_name`), so this branch
//! compiles. The integrator replaces this file with the real port.

/// `about_themselves(said)`: the facts a line says about the person, or None.
pub fn about_themselves(said: &str) -> Option<Vec<String>> {
    let _ = said;
    None
}

/// `reads_as_a_name(word)`: the name in its spelled form, or None.
pub fn reads_as_a_name(word: &str) -> Option<String> {
    let _ = word;
    None
}
