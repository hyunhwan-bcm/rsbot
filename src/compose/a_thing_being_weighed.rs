//! Port of `bot/compose/a_thing_being_weighed.py`: a decision the person has just named ("im
//! thinking about a new boiler"), helped with in place of a bare receipt.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, bundle_strs, bundle_vals, format_named, inline_re};
use crate::compose::pick;
use crate::py::text;
use crate::state::Memory;

bundle_strs!("bot.compose.a_thing_being_weighed"; RECEIPTS, _OPENERS, _A_NOT_AN, _CUTS_HEAD, _CUTS_NAME, _POSSESSIVE);
bundle_res!("bot.compose.a_thing_being_weighed"; _WEIGH, _DECIDING, _LOOKING_FOR, _ITS_FOR, NOT_A_DECISION, ASKS,
    _ALREADY_HAVE);
bundle_sets!("bot.compose.a_thing_being_weighed"; _VOWELS);
bundle_vals!("bot.compose.a_thing_being_weighed"; _KIND_OF, _POOLS);
inline_re!(THAN, r"\bthan\b");

/// `s[0].isdigit()` for a non-empty str.
fn opens_on_a_digit(s: &str) -> bool {
    s.chars().next().is_some_and(|c| text::is_digit(&c.to_string()))
}

/// Python: bot/compose/a_thing_being_weighed.py:122 `_plain(text)`
pub fn _plain(text: &str) -> String {
    com4::ws_one(&com4::straight(&text.to_lowercase()))
}

/// Python: bot/compose/a_thing_being_weighed.py:126 `_bare(phrase)`
pub fn _bare(phrase: &str) -> String {
    let plain = _plain(phrase);
    let mut said: &str = text::strip_chars(&plain, " .,!?-");
    // sorted(_OPENERS, key=len, reverse=True): longest first, ties in their own order
    let mut openers: Vec<&String> = _OPENERS.iter().collect();
    openers.sort_by_key(|o| std::cmp::Reverse(text::len(o)));
    for opener in openers {
        if said.starts_with(&format!("{opener} ")) {
            said = &said[opener.len() + 1..];
            break;
        }
    }
    text::strip_chars(said, " .,!-").to_string()
}

/// Python: bot/compose/a_thing_being_weighed.py:140 `_as_said(name, head)`
///
/// The thing in the person's own words, with an article only if they gave none.
pub fn _as_said(name: &str, head: &str) -> String {
    let words = text::split_ref(name);
    let mut first: String = words.first().map(|w| w.to_string()).unwrap_or_default();
    if first == "an" || first == "a" || text::is_digit(&first) || _POSSESSIVE.contains(&first) {
        return name.to_string();
    }
    let mut name = name.to_string();
    if matches!(first.as_str(), "the" | "some" | "any") {
        name = words[1..].join(" ");
        first = text::split_ref(&name).first().map(|w| w.to_string()).unwrap_or_default();
        if name.is_empty() {
            return String::new();
        }
    }
    if text::is_digit(&first) || _POSSESSIVE.contains(&first) || _is_plural(head) {
        return name;
    }
    _with_article(&name)
}

/// Python: bot/compose/a_thing_being_weighed.py:155 `_with_article(phrase)`
///
/// "electric blanket" -> "an electric blanket"; "two laptops" stays as it is.
pub fn _with_article(phrase: &str) -> String {
    let said = _bare(phrase);
    let words = text::split_ref(&said);
    if said.is_empty() || opens_on_a_digit(&said) || _is_plural(words[words.len() - 1]) {
        return said;
    }
    if _A_NOT_AN.iter().any(|w| w == words[0]) {
        return format!("a {said}");
    }
    if _is_plural(words[0]) && words.len() == 1 {
        return said;
    }
    let first = said.chars().next().unwrap().to_string();
    format!("{}{said}", if _VOWELS.contains(&first) { "an " } else { "a " })
}

/// Python: bot/compose/a_thing_being_weighed.py:191 `_is_plural(word)`
pub fn _is_plural(word: &str) -> bool {
    word.ends_with('s') && !word.ends_with("ss")
}

/// Python: bot/compose/a_thing_being_weighed.py:195 `_cut_at(said, cuts)`
pub fn _cut_at(said: &str, cuts: &[String]) -> String {
    let low = format!(" {said} ");
    let mut at = said.len();
    for word in cuts {
        if let Some(found) = low.find(&format!(" {word} ")) {
            if found < at {
                at = found;
            }
        }
    }
    if at >= said.len() { said.to_string() } else { text::strip(&said[..at]).to_string() }
}

/// Python: bot/compose/a_thing_being_weighed.py:205 `_head_and_name(thing)`
///
/// "a second hand bike for commuting" -> ("bike", "a second hand bike", head part).
pub fn _head_and_name(thing: &str) -> (String, String, String) {
    let plain = _plain(thing);
    let raw = text::strip_chars(&plain, " .,!?-");
    let said = _bare(raw);
    if said.is_empty() {
        return (String::new(), String::new(), String::new());
    }
    let head_part = _cut_at(&said, &_CUTS_HEAD);
    let head = text::split_ref(&head_part).last().map(|w| text::strip_chars(w, ".,!?-").to_lowercase()).unwrap_or_default();
    // The name keeps the article the person used - "a few jobs" is not "few jobs" - and only the
    // head drops it, because that is the word the kind is read from.
    (head, _cut_at(raw, &_CUTS_NAME), head_part)
}

/// Python: bot/compose/a_thing_being_weighed.py:218 `_read(said)`
///
/// (kind, the thing as the person said it), or None for Python's `(None, None)`.
pub fn _read(said: &str) -> Option<(String, String)> {
    if said.is_empty() || ASKS.match_(said).is_some() || NOT_A_DECISION.search(said).is_some() {
        return None;
    }
    let mut thing: Option<String> = None;
    if let Some(m) = _WEIGH.search(said).or_else(|| _DECIDING.search(said)) {
        thing = m.name("thing").map(str::to_string);
    } else if let Some(m) = _LOOKING_FOR.search(said) {
        thing = m.name("thing").map(str::to_string);
    } else if let Some(m) = _ITS_FOR.search(said) {
        thing = m.name("thing").filter(|s| !s.is_empty()).or(m.name("bare")).map(str::to_string);
    }
    let thing = thing.filter(|t| !t.is_empty())?;
    // "its a bigger job than i thought" is a remark about the work, not a decision about a job.
    if THAN.search(&_plain(&thing)).is_some() {
        return None;
    }
    let (head, name, head_part) = _head_and_name(&thing);
    if head.is_empty() || name.is_empty() || text::split_ref(&name).len() > 6 {
        return None;
    }
    // "im thinking of getting a bike": a thing the person is doing is not a thing they are
    // deciding between, and "a getting a bike" is not a phrase.
    let last_of_head = text::split_ref(&head).last().copied().unwrap_or("");
    if last_of_head.ends_with("ing") || text::split_ref(&name)[0].ends_with("ing") {
        return None;
    }
    let kind = _KIND_OF.get(&head).and_then(|k| k.as_str()).map(str::to_string);
    // Only the part before the first preposition counts: "selling my car" is a car being let go.
    if matches!(kind.as_deref(), Some("job" | "bought")) && _ALREADY_HAVE.search(&head_part).is_some() {
        return None;
    }
    // The kind cannot be read off it, so whoever reads it properly keeps the turn.
    let kind = kind.filter(|k| !k.is_empty())?;
    Some((kind, name))
}

/// Python: bot/compose/a_thing_being_weighed.py:295 `pool(line)`
///
/// Every phrasing for the kind of decision the line names, or None.
pub fn pool(line: &str) -> Option<Vec<String>> {
    let (kind, thing) = _read(&_plain(line))?;
    // The thing goes back into the reply as the person said it, with the article that suits it.
    let (head, name, _) = _head_and_name(&thing);
    let said = _as_said(&name, &head);
    // "im looking at cars" is not "a car": a plural gets "them".
    let it = if _is_plural(&head) { "them" } else { "it" };
    let templates = _POOLS.get(&kind).expect("_POOLS has every kind").strs();
    Some(templates.iter().map(|t| format_named(t, &[("thing", &said), ("it", it)])).collect())
}

/// Python: bot/compose/a_thing_being_weighed.py:308 `reads_as_a_receipt(reply)`
///
/// True when the reply is one of the three receipts, with or without a question after it.
pub fn reads_as_a_receipt(reply: &str) -> bool {
    let said = _plain(reply);
    !said.is_empty() && RECEIPTS.iter().any(|r| said.starts_with(r.as_str()))
}

/// Python: bot/compose/a_thing_being_weighed.py:314 `answer_for(text, memory=None, reply='')`
///
/// A decision the person has just named, helped with, or None. Only a receipt is replaced.
pub fn answer_for(text: &str, memory: &Memory, reply: &str) -> Option<String> {
    if !reads_as_a_receipt(reply) {
        return None;
    }
    let options = pool(text).filter(|o| !o.is_empty())?;
    Some(pick(&options, memory, text))
}
