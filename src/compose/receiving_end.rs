//! Port of `bot/compose/receiving_end.py`: "my boss keeps moving the goalposts", "the council
//! says thats just how it is" - a role that keeps doing it, or a brush-off, gets the step that
//! moves it: put it in writing, and the next rung.

use crate::compose::fallback::{self, _grief_thread, _in_a_loss_thread, _the_line_before, in_a_safety_thread};
use crate::compose::fallback_chain::{self, py_format};
use crate::data;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashMap;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.receiving_end.", stringify!($name))));)*
    };
}
regexes!(A_COMPLAINT_WORD, KEEPS_DOING_IT, A_PERSON_WHO_KEEPS, A_WARM_MEMORY, A_BRUSH_OFF, A_BODY_LINE);
static THE_BRUSH_OFF_REPLIES: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.compose.receiving_end.THE_BRUSH_OFF_REPLIES"));
static PERSON_WHO_KEEPS_REPLIES: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.compose.receiving_end.PERSON_WHO_KEEPS_REPLIES"));
static ROLE_SAID_BACK: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| data::str_map("bot.compose.receiving_end.ROLE_SAID_BACK"));
static THEIR: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.compose.receiving_end.THEIR"));

/// Python: bot/compose/receiving_end.py:117 `_the_role_said_back(found)`
///
/// The role as the person wrote it, facing them the right way round (a str; only this file
/// calls it).
pub fn _the_role_said_back(found: &str) -> String {
    let role = text::lower(&text::squash(found));
    if let Some(said) = ROLE_SAID_BACK.get(&role) {
        return said.clone();
    }
    if role.starts_with("the ") || role.starts_with("a ") {
        return role;
    }
    let (first, _, rest) = text::partition(&role, " ");
    if !rest.is_empty() {
        if let Some(their) = THEIR.get(first) {
            return format!("{their} {rest}");
        }
    }
    if matches!(first, "your" | "their" | "the" | "a") {
        return role;
    }
    format!("the {role}")
}

/// Python: bot/compose/receiving_end.py:146 `_a_body_line_above(memory)`
///
/// True when one of the last two things the person said was about a body or a threat.
pub fn _a_body_line_above(memory: &Memory) -> bool {
    let turns = &memory.turns[memory.turns.len().saturating_sub(2)..];
    turns.iter().any(|turn| A_BODY_LINE.is_match(&text::squash(&text::lower(&turn.user))))
}

/// Python: bot/compose/receiving_end.py:154 `_a_person_who_keeps(clean, repaired, memory)`
///
/// The question back to "my boss keeps moving the goalposts", or None.
pub fn _a_person_who_keeps(clean: &str, repaired: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(clean);
    if said.is_empty() {
        return None;
    }
    let mut found: Option<(String, &Vec<String>)> = None;
    for line in [said.clone(), text::squash(repaired)] {
        let Some(m) = A_PERSON_WHO_KEEPS.search(&line) else { continue };
        // `line[match.end():match.end() + 70]`, in characters
        let after = text::slice(&line[m.end()..], 0, Some(70));
        let brush = A_BRUSH_OFF.is_match(&after);
        if (brush || (KEEPS_DOING_IT.is_match(&after) && A_COMPLAINT_WORD.is_match(&line))) && !A_WARM_MEMORY.is_match(&line) {
            let role = _the_role_said_back(&line[m.start()..m.end()]);
            let pool = if brush { &*THE_BRUSH_OFF_REPLIES } else { &*PERSON_WHO_KEEPS_REPLIES };
            found = Some((role, pool));
            break;
        }
    }
    let (role, pool) = found.filter(|(role, _)| !role.is_empty())?;
    if in_a_safety_thread(memory) || _grief_thread(memory) || _in_a_loss_thread(memory) || _a_body_line_above(memory) {
        return None;
    }
    let says = _the_line_before(memory);
    if !says.is_empty() && fallback::in_a_pool(&says, &[&PERSON_WHO_KEEPS_REPLIES[..], &THE_BRUSH_OFF_REPLIES[..]]) {
        return None;
    }
    let mut chars = role.chars();
    let upper = chars.next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default() + chars.as_str();
    let lines: Vec<String> = pool.iter().map(|line| py_format(line, &[("role", &role), ("Role", &upper)])).collect();
    fallback_chain::_choose(&lines, memory, clean, None)
}

/// Python: bot/compose/receiving_end.py:184 `reply_for(clean, repaired, memory)`
pub fn reply_for(clean: &str, repaired: &str, memory: &Memory) -> Option<String> {
    _a_person_who_keeps(clean, repaired, memory)
}
