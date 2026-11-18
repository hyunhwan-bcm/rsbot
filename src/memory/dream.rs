//! Port of `bot/memory/dream.py`: the reading of a line that is about a dream, and the scene the
//! dream was made of. Every function of the module is ported (the knowledge lookup stands down
//! for a dream through `told_as_a_dream`).

use crate::data;
use crate::py::re::Re;
use crate::state::Memory;
use std::sync::LazyLock;

fn re(name: &str) -> &'static Re {
    data::re(&format!("bot.memory.dream.{name}"))
}

pub static DREAM: LazyLock<&'static Re> = LazyLock::new(|| re("DREAM"));
pub static FIRST_PERSON: LazyLock<&'static Re> = LazyLock::new(|| re("FIRST_PERSON"));
pub static NAMED_A_NIGHTMARE: LazyLock<&'static Re> = LazyLock::new(|| re("NAMED_A_NIGHTMARE"));
pub static A_WISH: LazyLock<&'static Re> = LazyLock::new(|| re("A_WISH"));
pub static A_WISH_TO_BE_GONE: LazyLock<&'static Re> = LazyLock::new(|| re("A_WISH_TO_BE_GONE"));
pub static DREAM_VERB: LazyLock<&'static Re> = LazyLock::new(|| re("DREAM_VERB"));
pub static A_SAID_WISH: LazyLock<&'static Re> = LazyLock::new(|| re("A_SAID_WISH"));
pub static A_DREAM_AHEAD: LazyLock<&'static Re> = LazyLock::new(|| re("A_DREAM_AHEAD"));
pub static A_SCENE_OF_NOTHING: LazyLock<&'static Re> = LazyLock::new(|| re("A_SCENE_OF_NOTHING"));
pub static SETTLED_IT: LazyLock<&'static Re> = LazyLock::new(|| re("SETTLED_IT"));
pub static DREAM_NAMED: LazyLock<&'static Re> = LazyLock::new(|| re("DREAM_NAMED"));
pub static DREAM_NAMED_LOOSE: LazyLock<&'static Re> = LazyLock::new(|| re("DREAM_NAMED_LOOSE"));
pub static DREAM_IS_HAPPENING_NOW: LazyLock<&'static Re> = LazyLock::new(|| re("DREAM_IS_HAPPENING_NOW"));
pub static A_DEATH_BEFORE_THE_DREAM: LazyLock<&'static Re> = LazyLock::new(|| re("A_DEATH_BEFORE_THE_DREAM"));

/// `" ".join(str(line or "").lower().split())`.
fn squeezed_lower(line: &str) -> String {
    crate::py::text::split(&crate::py::text::lower(line)).join(" ")
}

/// `the_scene(line)`: what the dream was about, or None when the line is not a dream somebody
/// had.
pub fn the_scene(line: &str) -> Option<String> {
    let text = squeezed_lower(line);
    if text.is_empty() || crate::py::text::len(&text) > 400 {
        return None;
    }
    if NAMED_A_NIGHTMARE.is_match(&text) {
        return None;
    }
    let found = DREAM.search(&text)?;
    let before = &text[..found.start()];
    if !FIRST_PERSON.is_match(before) {
        return None;
    }
    let scene = text[found.end()..].trim_matches(|c| " ,.:;!-".contains(c)).to_string();
    if A_WISH.is_match(&text) || scene.contains("one day") || A_SCENE_OF_NOTHING.match_(&scene).is_some() {
        return None;
    }
    if A_WISH_TO_BE_GONE.is_match(&text) || A_SAID_WISH.is_match(&text) {
        return None;
    }
    if A_DREAM_AHEAD.is_match(&text) && !DREAM_VERB.is_match(&text) {
        return None;
    }
    if scene.is_empty() {
        None
    } else {
        Some(scene)
    }
}

/// `named(line)`: the line is a dream the speaker had.
pub fn named(line: &str) -> bool {
    the_scene(line).is_some()
}

/// `dreamed_before(line, pattern)`: `pattern` is found inside the dream and not before it.
pub fn dreamed_before(line: &str, pattern: &Re) -> bool {
    let text = squeezed_lower(line);
    let Some(scene) = the_scene(&text) else { return false };
    if !pattern.is_match(&scene) {
        return false;
    }
    // text[:text.find(scene)]; find is -1 when absent, and text[:-1] drops the last character.
    let head = match text.find(scene.as_str()) {
        Some(at) => &text[..at],
        None => {
            let mut end = text.len();
            if let Some((i, _)) = text.char_indices().last() {
                end = i;
            }
            &text[..end]
        }
    };
    !pattern.is_match(head)
}

/// `told_in_the_turn_before(memory)`: the turn just before this one was a dream.
pub fn told_in_the_turn_before(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    if last.module.as_deref().unwrap_or("") == "dreams" {
        return true;
    }
    the_scene(&last.user).is_some()
}

/// `told_as_a_dream(line)`: the line tells a dream of its own and nothing in it says the room
/// has it.
pub fn told_as_a_dream(line: &str) -> bool {
    named(line) && a_dream_is_not_happening(line)
}

/// `a_dream_is_not_happening(line)`.
pub fn a_dream_is_not_happening(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    let text = line;
    if !(DREAM_NAMED.is_match(text) || DREAM_NAMED_LOOSE.is_match(text)) {
        return false;
    }
    if A_WISH_TO_BE_GONE.is_match(text) || A_SAID_WISH.is_match(text) {
        return false;
    }
    if A_DREAM_AHEAD.is_match(text) && !DREAM_VERB.is_match(text) {
        return false;
    }
    !DREAM_IS_HAPPENING_NOW.is_match(text)
}

/// `a_death_came_first(line)`.
pub fn a_death_came_first(line: &str) -> bool {
    A_DEATH_BEFORE_THE_DREAM.is_match(&squeezed_lower(line))
}
