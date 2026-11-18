//! Port of `bot/compose/how_it_went.py`: "how was the wedding" asked about a thing of the
//! person's own, and their verdict on how it went ("better than i expected").
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, format_named, inline_re};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;

bundle_sets!("bot.compose.how_it_went"; THEIRS, NOT_AN_EVENT, HEAVY, MORE_THAN_ONE, SINGULAR_ENDS_IN_S, STANDS_ALONE, LOW,
    NOT_SO_BAD);
bundle_res!("bot.compose.how_it_went"; HOW, FELT, THE_BOT, COMPARED, NOT_AS, AS_EXPECTED, EXPECTED_BARE, WHICH_WAS_THE_AIM,
    NOTHING_WENT_WRONG, COMPARE_WORD, LEAD_STRIP);
bundle_strs!("bot.compose.how_it_went"; PAST, STILL_PLURAL, STILL_SINGLE, CAREFUL, BETTER, WORSE, RELIEF, NO_SURPRISES,
    WANTED_IT);
inline_re!(HOW_IS, concat!(r"^(?:(?:so|and|ok|okay|well|oh|but|right)[,\s]+)?", r"how(?:'s|s| is| are)\b"));

/// `" ".join(text.lower().split()).rstrip(" .!,;:")`.
fn the_line(text_: &str) -> String {
    text::rstrip_chars(&text::squash(&text_.to_lowercase()), " .!,;:").to_string()
}

/// Python: bot/compose/how_it_went.py:133 `a_kind_of_theirs(head)`
///
/// Is the bare "the X" a kind of thing that sits in a person's own life? A plural of a word in
/// THEIRS counts too.
pub fn a_kind_of_theirs(head: &str) -> bool {
    if THEIRS.contains(head) {
        return true;
    }
    ["s", "es"].iter().any(|suffix| head.strip_suffix(suffix).is_some_and(|stem| THEIRS.contains(stem)))
}

/// Python: bot/compose/how_it_went.py:149 `more_than_one(head)`
///
/// Is the thing the person asks about more than one of something?
pub fn more_than_one(head: &str) -> bool {
    MORE_THAN_ONE.contains(head) || (head.ends_with('s') && !SINGULAR_ENDS_IN_S.contains(head))
}

/// Python: bot/compose/how_it_went.py:156 `_head(thing)`
///
/// The last word is what the thing is: "school play" is a play, "first day" is a day.
pub fn _head(thing: &str) -> String {
    text::split_ref(thing).last().map(|w| text::strip_chars(w, "'").to_string()).unwrap_or_default()
}

/// Python: bot/compose/how_it_went.py:161 `answer_for(text, memory=None)`
pub fn answer_for(text_: &str, memory: &Memory) -> Option<String> {
    let line = the_line(text_);
    if line.is_empty() || THE_BOT.search(&line).is_some() {
        return None;
    }
    let Some(m) = HOW.match_(&line) else {
        if FELT.match_(&line).is_some() {
            return Some(pick(&PAST, memory, &line));
        }
        return None;
    };
    let thing = text::strip(m.name("thing").unwrap_or("")).to_string();
    let whose = m.name("whose").unwrap_or("").to_lowercase();
    let head = _head(&thing);
    if head.is_empty() || NOT_AN_EVENT.contains(&head) {
        return None;
    }
    // "my exam", "their wedding": theirs whatever the noun is. A bare "the ..." has to be a kind
    // of thing that belongs to somebody's own life, or "how was the war" would be their war.
    if !matches!(whose.as_str(), "my" | "our" | "his" | "her" | "their") && !a_kind_of_theirs(&head) {
        return None;
    }
    let pool: &Vec<String> = if HEAVY.contains(&head) {
        &CAREFUL
    } else if HOW_IS.match_(&line).is_some() {
        if more_than_one(&head) { &STILL_PLURAL } else { &STILL_SINGLE }
    } else {
        &PAST
    };
    Some(pick(pool, memory, &line))
}

/// Python: bot/compose/how_it_went.py:282 `_opens_on_the_comparison(line)`
///
/// True when nothing but the comparison is in front of it.
pub fn _opens_on_the_comparison(line: &str) -> bool {
    let rest = LEAD_STRIP.sub("", line);
    let first = text::split_ref(&rest).first().copied().unwrap_or("");
    STANDS_ALONE.contains(first) || COMPARE_WORD.match_(first).is_some()
}

/// Python: bot/compose/how_it_went.py:293 `the_answer_they_gave(text, memory=None)`
///
/// The reply to a person's verdict on how a thing of theirs went, or None.
pub fn the_answer_they_gave(text_: &str, memory: &Memory) -> Option<String> {
    let line = the_line(text_);
    if line.is_empty() || line.contains('?') || THE_BOT.search(&line).is_some() {
        return None;
    }
    if text::split_ref(&line).len() > 9 {
        return None;
    }
    if let Some(aim) = WHICH_WAS_THE_AIM.match_(&line) {
        let mut what = text::strip(aim.name("what").unwrap_or("")).to_string();
        for lead in ["it was ", "it is ", "that was ", "this was "] {
            if let Some(rest) = what.strip_prefix(lead) {
                what = rest.to_string();
            }
        }
        if what.is_empty() || NOTHING_WENT_WRONG.search(&what).is_some() || text::split_ref(&what).len() > 3 {
            return None;
        }
        let pool: Vec<String> = WANTED_IT.iter().map(|l| format_named(l, &[("0", &what)])).collect();
        return Some(pick(&pool, memory, &line));
    }
    if !_opens_on_the_comparison(&line) {
        return None;
    }
    let pool: &Vec<String> = if let Some(m) = COMPARED.search(&line) {
        if LOW.contains(&m.name("word").unwrap_or("").to_lowercase()) { &WORSE } else { &BETTER }
    } else if let Some(not_as) = NOT_AS.search(&line) {
        if NOT_SO_BAD.contains(&not_as.name("word").unwrap_or("").to_lowercase()) { &RELIEF } else { &WORSE }
    } else if AS_EXPECTED.search(&line).is_some() || EXPECTED_BARE.search(&line).is_some() {
        &NO_SURPRISES
    } else {
        return None;
    };
    Some(pick(pool, memory, &line))
}
