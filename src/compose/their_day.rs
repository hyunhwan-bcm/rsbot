//! Port of `bot/compose/their_day.py`: a question about something of the person's own ("how was
//! the wedding", "when is my renewal") handed back to them, since only they know.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_strs, format_named, inline_re};
use crate::compose::pick;
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.their_day"; _HOW, _THING, _TAIL, _PRONOUN, _STATE, _STATE_BARE, _NAMES, _HEARD_BACK, _SAID, _ABOUT_THE_BOT,
    _A_LIFE_CHOICE, _OWN_TASTE, _ASKS_FOR_ADVICE, _ASKS, _MINE_AND_THING, _A_COMPOSITION, _AN_IDENTIFIER, _A_WHEN, _NOT_A_DIARY,
    _A_DEPOSIT);
bundle_strs!("bot.compose.their_day"; LINES, CARE_LINES, OWN_FIGURE, OWN_DATE, OWN_POLICY, OWN_FIGURE_LINES, OWN_POLICY_LINES,
    OWN_DATE_LINES, OWN_LIFE_LINES, OWN_TASTE_LINES);
inline_re!(HOWS, r"\bhow's\b|\bhows\b");
inline_re!(HOWD, r"\bhow'd\b|\bhowd\b");
inline_re!(OPENS_HOWS, r"^hows\b");
inline_re!(OPENS_HOWD, r"^howd\b");

/// Python: bot/compose/their_day.py:202 `pool_for(text, care=False)`
///
/// The lines for a question about something of their own, or None. `care` is the pool after a
/// death or a diagnosis.
pub fn pool_for(text_: &str, care: bool) -> Option<Vec<String>> {
    let ask = reads(text_, None).or_else(|| {
        // A typo or a laugh on the end is the same question; the understanding engine has the
        // spelling of it.
        let plain = crate::understanding::understand(text_, None).plain;
        reads(&plain, None)
    })?;
    let said_back = text::upper_first(&ask);
    let lines: &Vec<String> = if care { &CARE_LINES } else { &LINES };
    Some(lines.iter().map(|l| format_named(l, &[("ask", &ask), ("Ask", &said_back)])).collect())
}

/// Python: bot/compose/their_day.py:222 `reads(clean, repaired=None)`
///
/// The ask it is theirs to answer, in their own words, or None. `repaired` is only read, never
/// echoed.
pub fn reads(clean: &str, repaired: Option<&str>) -> Option<String> {
    let t = text::squash(&com4::straight(&clean.to_lowercase()));
    if t.is_empty() || text::split_ref(&t).len() > 14 {
        return None;
    }
    let plain = text::squash(&com4::straight(&repaired.unwrap_or("").to_lowercase()));
    let base = if plain.is_empty() { &t } else { &plain };
    let matched = text::squash(&HOWD.sub("how did", &HOWS.sub("how is", base)));
    // The bot's own day is chat's to answer.
    if _ABOUT_THE_BOT.match_(&t).is_some() || _ABOUT_THE_BOT.match_(&plain).is_some() || _ABOUT_THE_BOT.match_(&matched).is_some() {
        return None;
    }
    // Asked back in the words they typed, with the apostrophe they left out put in.
    let ask = OPENS_HOWS.sub("how's", &OPENS_HOWD.sub("how'd", &t));
    let ask = text::rstrip_chars(text::strip(&ask), "?.! ").to_string();
    if let Some(how) = _HOW.match_(&matched) {
        let rest = how.name_str("rest");
        if let Some(thing) = _THING.match_(rest) {
            if _TAIL.match_(&rest[thing.end()..]).is_some() {
                return Some(ask);
            }
        }
    }
    for pattern in [&*_STATE, &*_STATE_BARE, &*_SAID, &*_HEARD_BACK, &*_PRONOUN, &*_NAMES] {
        if pattern.match_(&matched).is_some() {
            return Some(ask);
        }
    }
    None
}

/// Python: bot/compose/their_day.py:380 `own_fact(text, memory=None)`
///
/// The bucket a question about a fact of theirs belongs in ("figure", "date", "policy", "taste"
/// or "life"), or None.
pub fn own_fact(text_: &str, memory: &Memory) -> Option<(String, String)> {
    let _ = memory;
    let line = text::squash(&com4::straight(&text_.to_lowercase()));
    let words = text::split_ref(&line);
    if line.is_empty() || words.len() > 14 {
        return None;
    }
    let head = words[..words.len().min(3)].join(" ");
    if _ASKS_FOR_ADVICE.match_(&line).is_some() || _ASKS.match_(&head).is_none() {
        return None;
    }
    let found = |kind: &str| Some((kind.to_string(), String::new()));
    if _A_LIFE_CHOICE.search(&line).is_some() {
        return found("life");
    }
    if _OWN_TASTE.search(&line).is_some() {
        return found("taste");
    }
    // The possessive has to be on the thing itself.
    let mine = _MINE_AND_THING.search(&line)?;
    let thing = mine.name_str("thing");
    if _NOT_A_DIARY.search(&line).is_some() || _A_DEPOSIT.search(&line).is_some() || _AN_IDENTIFIER.search(&line).is_some() {
        return None;
    }
    let order: [(&str, &Vec<String>); 3] = if _A_WHEN.search(&line).is_some() {
        [("date", &OWN_DATE), ("policy", &OWN_POLICY), ("figure", &OWN_FIGURE)]
    } else {
        [("policy", &OWN_POLICY), ("figure", &OWN_FIGURE), ("date", &OWN_DATE)]
    };
    for (kind, words) in order {
        if !words.iter().any(|w| Re::new(&format!(r"(?<![a-z]){}(?![a-z])", pyre::escape(w)), 0).search(thing).is_some()) {
            continue;
        }
        // "does paye come out of my wages" is a question about the rule, not their number.
        if kind == "figure" && _A_COMPOSITION.search(&line).is_some() {
            return None;
        }
        return found(kind);
    }
    None
}

/// Python: bot/compose/their_day.py:423 `own_fact_for(text, memory=None)`
///
/// The reply for a question about a fact of theirs, or None.
pub fn own_fact_for(text_: &str, memory: &Memory) -> Option<String> {
    let (kind, _) = own_fact(text_, memory)?;
    let pool: &Vec<String> = match kind.as_str() {
        "figure" => &OWN_FIGURE_LINES,
        "date" => &OWN_DATE_LINES,
        "policy" => &OWN_POLICY_LINES,
        "taste" => &OWN_TASTE_LINES,
        "life" => &OWN_LIFE_LINES,
        other => panic!("KeyError: {other:?}"),
    };
    Some(pick(pool, memory, &text_.to_lowercase()))
}
