//! Port of `bot/compose/limit.py`: a line that only rules something out ("no horror") after an
//! answer, met with the answer above said again with that part taken out.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_strs, bundle_vals, format_named, inline_re};
use crate::data::Value;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{BTreeSet, HashSet};

bundle_res!("bot.compose.limit"; PART, CONTINUES, COUNTED, WORD, FRAME, BACK_REFERENCE, RESTATED, A_STATE_IS_NOT_A_LIMIT,
    NOTHING_THAT_LIMITS, NO_LAUGH_OR_SCARES, A_LIMIT_SHAPE);
bundle_strs!("bot.compose.limit"; KEPT, NOTHING_LEFT, MISSED_LIMIT);
bundle_vals!("bot.compose.limit"; LEAST_KEPT, LEAST_WORDS);
inline_re!(A_JOINER, r"^(?:and|or|then|but|plus|so|because|also|instead)\s+", crate::py::re::I);

/// Python: bot/compose/limit.py:81 `restates_the_answer(bot_line)`
///
/// True when this bot line is one of ours: the answer above, said again with a limit out.
pub fn restates_the_answer(bot_line: &str) -> bool {
    RESTATED.match_(text::strip(bot_line)).is_some()
}

/// Python: bot/compose/limit.py:98 `forms_of(thing)`
///
/// The word forms of a ruled-out thing that count as the same thing in the bot's own line,
/// sorted.
pub fn forms_of(thing: &str) -> Vec<String> {
    let words: Vec<String> = WORD.findall_str(&thing.to_lowercase()).into_iter().filter(|w| !w.is_empty()).collect();
    let mut out: BTreeSet<String> = BTreeSet::new();
    for word in &words {
        let n = text::len(word);
        out.insert(word.clone());
        if word.ends_with("ing") && n > 5 {
            // "no walking up hills" is the walking the answer above calls "walk".
            let stem = text::slice(word, 0, Some(-3));
            out.insert(stem.clone());
            out.insert(format!("{stem}e"));
            let chars: Vec<char> = stem.chars().collect();
            if chars.len() > 2 && chars[chars.len() - 1] == chars[chars.len() - 2] {
                out.insert(text::slice(&stem, 0, Some(-1)));
            }
        }
        if word.ends_with("ies") && n > 4 {
            out.insert(format!("{}y", text::slice(word, 0, Some(-3))));
        } else if word.ends_with("es") && n > 3 {
            out.insert(text::slice(word, 0, Some(-2)));
            out.insert(text::slice(word, 0, Some(-1)));
        } else if word.ends_with('s') && n > 3 {
            out.insert(text::slice(word, 0, Some(-1)));
        } else {
            out.insert(format!("{word}s"));
            out.insert(format!("{word}es"));
        }
    }
    out.into_iter().collect()
}

/// Python: bot/compose/limit.py:128 `_carries(part, forms)`
///
/// True when this part of the bot's own answer is about the thing ruled out.
pub fn _carries(part: &str, forms: &[String]) -> bool {
    let said: HashSet<String> = WORD.findall_str(&part.to_lowercase()).into_iter().collect();
    forms.iter().any(|f| said.contains(f))
}

/// Python: bot/compose/limit.py:134 `_last_answer(memory)`
///
/// The bot's own last line, or "" when there is not one. A safety line is never edited.
pub fn _last_answer(memory: &Memory) -> String {
    for last in memory.turns.iter().rev() {
        if matches!(last.module.as_deref(), Some("safety" | "loss" | "badnews")) {
            return String::new();
        }
        let said = text::strip(&last.bot);
        if !said.is_empty() {
            return said.to_string();
        }
    }
    String::new()
}

/// Python: bot/compose/limit.py:150 `_pick(pool, memory)`
///
/// One line from a pool, never the line the bot has just said.
pub fn _pick(pool: &[String], memory: &Memory) -> String {
    let last = _last_answer(memory);
    for line in pool {
        let head = text::strip(line.split('{').next().unwrap_or(""));
        if !head.is_empty() && !last.is_empty() && last.contains(head) {
            continue;
        }
        return line.clone();
    }
    pool[0].clone()
}

/// Python: bot/compose/limit.py:161 `_take_out(said, forms)`
///
/// The answer above with the parts about the thing taken out, or "" when not enough is left.
pub fn _take_out(said: &str, forms: &[String]) -> String {
    let (kept, _dropped) = _the_parts_left(said, forms);
    if kept.is_empty() {
        return String::new();
    }
    let left = _joined(&kept);
    let least_kept = LEAST_KEPT.as_int().expect("an int") as usize;
    let least_words = LEAST_WORDS.as_int().expect("an int") as usize;
    if text::len(&left) < least_kept || text::split_ref(&left).len() < least_words {
        return String::new();
    }
    left
}

/// `[str(t).strip() for t in understood.get("excludes") or [] if str(t).strip()]`.
fn excluded(understood: &Understood) -> Vec<String> {
    understood.excludes.iter().map(|t| text::strip(t).to_string()).filter(|t| !t.is_empty()).collect()
}

/// Python: bot/compose/limit.py:172 `steps_left(understood, memory)`
///
/// The parts of the answer above that the limit leaves, in the order they were given.
pub fn steps_left(understood: &Understood, memory: &Memory) -> Vec<String> {
    if !understood.constraint {
        return Vec::new();
    }
    let things = excluded(understood);
    let said = _last_answer(memory);
    if things.is_empty() || said.is_empty() {
        return Vec::new();
    }
    let (kept, _dropped) = _the_parts_left(&said, &forms_of(&things[0]));
    // A part that ends with a colon only introduces the parts after it.
    kept.iter()
        .filter(|p| !text::strip(p).is_empty() && !text::strip(p).ends_with(':'))
        .map(|p| text::rstrip_chars(text::strip(p), ".,;:").to_string())
        .collect()
}

/// Python: bot/compose/limit.py:191 `_the_parts_left(said, forms)`
///
/// (the parts of the answer that survive the limit, how many were taken out).
pub fn _the_parts_left(said: &str, forms: &[String]) -> (Vec<String>, usize) {
    let parts: Vec<String> = PART.split(said).into_iter().filter(|p| !text::strip(p).is_empty()).collect();
    let mut kept: Vec<String> = Vec::new();
    let mut dropped = 0usize;
    let mut after_a_colon = false;
    for part in &parts {
        if kept.is_empty() && FRAME.match_(text::strip(part)).is_some() && !_carries(part, forms) {
            dropped += 1; // the frame that said the answer was unchanged, now that it is not
            continue;
        }
        if _carries(part, forms) || (dropped > 0 && CONTINUES.match_(part).is_some()) || after_a_colon {
            // What is left of a part that introduced the one taken out goes too.
            if kept.last().is_some_and(|k| text::rstrip(k).ends_with(':')) {
                kept.pop();
            }
            // The part after a part taken out with a colon goes with it.
            after_a_colon = text::rstrip(part).ends_with(':');
            dropped += 1;
            continue;
        }
        after_a_colon = false;
        // A count of the choices is wrong once one of them is gone.
        if dropped > 0 && COUNTED.search(part).is_some() {
            dropped += 1;
            continue;
        }
        kept.push(text::strip(part).to_string());
    }
    if dropped == 0 || kept.is_empty() {
        return (Vec::new(), dropped);
    }
    // The first thing left points back at what has just been taken out.
    if BACK_REFERENCE.match_(&kept[0]).is_some() {
        return (Vec::new(), dropped);
    }
    (kept, dropped)
}

fn first_char(s: &str) -> String {
    s.chars().take(1).collect()
}

/// Python: bot/compose/limit.py:226 `_joined(kept)`
///
/// The kept parts as one line, with the punctuation the missing parts have left behind fixed.
pub fn _joined<S: AsRef<str>>(kept: &[S]) -> String {
    let mut out: Vec<String> = Vec::new();
    for part in kept {
        // A part that led with "and" or "or" has to stand on its own now.
        let mut part = A_JOINER.sub("", part.as_ref());
        if part.is_empty() {
            continue;
        }
        let head = first_char(&part);
        let starts_a_sentence = text::is_upper(&head) || "*\"'(".contains(head.as_str());
        if let Some(prev) = out.last_mut() {
            let last = prev.chars().last().unwrap_or(' ');
            if starts_a_sentence && ",;:".contains(last) {
                prev.pop();
                prev.push('.');
            }
            let last = prev.chars().last().unwrap_or(' ');
            if ".!?".contains(last) && text::is_lower(&head) {
                part = text::upper_first(&part);
            }
        }
        out.push(part);
    }
    let mut left = text::rstrip_chars(&out.join(" "), " ,;:").to_string();
    if !left.is_empty() && !".!?".contains(left.chars().last().unwrap()) {
        left.push('.');
    }
    if text::is_lower(&first_char(&left)) {
        left = text::upper_first(&left);
    }
    left
}

/// Python: bot/compose/limit.py:250 `_a_fact_put_right(memory)`
///
/// True when this line corrects one of their own facts rather than limiting the answer: the
/// store writes `memory["correction"]` with the value that replaced the old one.
pub fn _a_fact_put_right(memory: &Memory) -> bool {
    match memory.get("correction") {
        Some(fixed) if fixed.is_dict() => fixed.get("value").is_some_and(Value::truthy),
        _ => false,
    }
}

/// Python: bot/compose/limit.py:291 `a_limit_on_the_answer(text)`
///
/// True when the line is a thing taken out of the answer above, not a report of their day.
pub fn a_limit_on_the_answer(text_: &str) -> bool {
    let said = format!(" {} ", text::squash(&text_.to_lowercase()));
    NOTHING_THAT_LIMITS.search(&said).is_some() || NO_LAUGH_OR_SCARES.search(&said).is_some()
}

/// Python: bot/compose/limit.py:316 `a_limit_the_reading_missed(clean, memory)`
///
/// The reply for a limit the reading did not file, or None to leave the line alone.
pub fn a_limit_the_reading_missed(clean: &str, memory: &Memory) -> Option<String> {
    let said = format!(" {} ", text::squash(&clean.to_lowercase()));
    if text::strip(&said).is_empty() {
        return None;
    }
    if A_LIMIT_SHAPE.match_(text::strip(&said)).is_none() || !a_limit_on_the_answer(&said) {
        return None;
    }
    let above = _last_answer(memory);
    if above.is_empty() || text::rstrip(&above).ends_with('?') {
        // Nothing was on the table, so there is nothing the person has narrowed.
        return None;
    }
    Some(_pick(&MISSED_LIMIT, memory))
}

/// Python: bot/compose/limit.py:330 `reply_for(understood, memory)`
///
/// The reply for a line that only rules something out, or None to leave the line alone.
pub fn reply_for(understood: &Understood, memory: &Memory) -> Option<String> {
    if !understood.constraint {
        return None;
    }
    if _a_fact_put_right(memory) {
        // A correction is not a limit, and the store's own note is the reply to it.
        return None;
    }
    let things = excluded(understood);
    if things.is_empty() {
        return None;
    }
    let said = _last_answer(memory);
    // Nothing has been answered yet, so a limit on it limits nothing.
    if said.is_empty() {
        return None;
    }
    let thing = &things[0];
    if A_STATE_IS_NOT_A_LIMIT.match_(thing).is_some() {
        // The person's own state, not a part of that answer they want taken out.
        return None;
    }
    let forms = forms_of(thing);
    let left = _take_out(&said, &forms);
    if left.is_empty() {
        if !_carries(&said, &forms) {
            return None;
        }
        return Some(format_named(&_pick(&NOTHING_LEFT, memory), &[("thing", thing)]));
    }
    Some(format_named(&_pick(&KEPT, memory), &[("thing", thing), ("kept", &left)]))
}
