//! Port of `bot/compose/taste.py`: a line that only states a like or a dislike ("i hate the
//! rain", "i love my coffee strong") answered as a taste, with the thing in the person's words.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{bundle_res, bundle_sets, bundle_vals, format_named, inline_re};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.taste"; _A_PERSON, _A_SOMEONE, _A_PERSON_DESCRIBED, _ABOUT_WORK, _ABOUT_ME, _A_SYMPTOM, _A_ASK, _A_DEED,
    _A_SECOND_CLAUSE, _A_STANCE, _A_STANCE_NO_SUBJECT, _A_DREAD, _A_WAIT, _TWO_STANCES, _THE_BOTS, _A_MANNER_PHRASE, _THEIR_OWN,
    _AN_INFINITIVE, _NOT_A_THING, _TAIL_FILLER);
bundle_sets!("bot.compose.taste"; _DISLIKE_VERBS, _LIKE_VERBS, _MANNER_WORDS, _A_LINKING_WORD, _AN_ANIMAL, _A_CLAUSE_WORD,
    _NOT_A_DOING, _DOUBLING, _A_PRONOUN, _DEMONSTRATIVE, _PROPER);
bundle_vals!("bot.compose.taste"; _BY_KIND, _IRREGULAR_DOING);
inline_re!(A_FILLER_OPENING, r"^(?:hi|hey|hello|yo|so|well|oh|god|honestly|anyway)[,!\s]+");
inline_re!(OPENS_ON_TO, r"^to\s+[a-z]", crate::py::re::I);
inline_re!(A_TO, r"\bto\b");
inline_re!(MY_OR_OUR, r"^(my|our)\b", crate::py::re::I);

/// (kind, the thing, the manner) a taste line reads as.
pub type Read = (String, String, String);

/// Python: bot/compose/taste.py:274 `_split_manner(thing)`
///
/// (the thing, the way they have it): "my tea weak" is ("my tea", "weak").
pub fn _split_manner(thing: &str) -> (String, String) {
    let words = text::split_ref(thing);
    if words.len() == 2
        && _MANNER_WORDS.contains(&words[1].to_lowercase())
        && !matches!(words[0].to_lowercase().as_str(), "my" | "our" | "the" | "a" | "an" | "your" | "his" | "her" | "their" | "it" | "that" | "this")
    {
        // "i like coffee strong": the noun comes first, so the last word is the manner.
        return (words[0].to_string(), words[1].to_lowercase());
    }
    if words.len() < 3 || !matches!(words[0].to_lowercase().as_str(), "my" | "our") {
        return (thing.to_string(), String::new());
    }
    let said = words[1..].join(" ");
    if let Some(phrase) = _A_MANNER_PHRASE.search(&said) {
        let lower = said.to_lowercase();
        let at = text::find(&lower, phrase.as_str());
        if at < 0 {
            panic!("ValueError: substring not found");
        }
        let head = text::strip(&text::slice(&said, 0, Some(at))).to_string();
        if !head.is_empty() && !text::split_ref(&head).iter().any(|w| _A_LINKING_WORD.contains(&w.to_lowercase())) {
            return (format!("{} {head}", words[0]), phrase.as_str().to_lowercase());
        }
        return (thing.to_string(), String::new());
    }
    let last = text::strip_chars(&words[words.len() - 1].to_lowercase(), ".,!;:").to_string();
    if !_MANNER_WORDS.contains(&last) {
        return (thing.to_string(), String::new());
    }
    if words[1..words.len() - 1].iter().any(|w| _A_LINKING_WORD.contains(text::strip_chars(&w.to_lowercase(), ".,!;:"))) {
        return (thing.to_string(), String::new());
    }
    (words[..words.len() - 1].join(" "), last)
}

/// Python: bot/compose/taste.py:305 `_a_manner_kind(kind)`
///
/// The pool for a thing and a manner said together; the kind's own pool otherwise.
pub fn _a_manner_kind(kind: &str) -> String {
    match kind {
        "like" | "like_own" => "like_manner",
        "dislike" => "dislike_manner",
        other => other,
    }
    .to_string()
}

/// Python: bot/compose/taste.py:346 `_a_doing(verb)`
///
/// The doing said as a name: "cook" is "cooking", "swim" is "swimming".
pub fn _a_doing(verb: &str) -> String {
    let verb = verb.to_lowercase();
    if let Some(irregular) = _IRREGULAR_DOING.get(&verb) {
        return irregular.str().to_string();
    }
    if let Some(stem) = verb.strip_suffix("ie") {
        return format!("{stem}ying");
    }
    if verb.ends_with('e') && !text::ends_with_any(&verb, &["ee", "ye", "oe"]) {
        return format!("{}ing", &verb[..verb.len() - 1]);
    }
    if _DOUBLING.contains(&verb) {
        let last = verb.chars().last().unwrap();
        return format!("{verb}{last}ing");
    }
    format!("{verb}ing")
}

/// Python: bot/compose/taste.py:360 `_the_doing(thing)`
///
/// The doing a stance about "to <verb>" is about, capitalised, or "" when it is not one.
pub fn _the_doing(thing: &str) -> String {
    let Some(m) = _AN_INFINITIVE.match_(thing) else { return String::new() };
    let verb = m.name_str("verb").to_lowercase();
    let rest = text::strip(m.name("rest").unwrap_or("")).to_string();
    if _NOT_A_DOING.contains(&verb) || verb.is_empty() {
        return String::new();
    }
    if !rest.is_empty() && (text::split_ref(&rest).len() > 2 || text::split_ref(&rest).iter().any(|w| _A_CLAUSE_WORD.contains(*w))) {
        return String::new();
    }
    let doing = _a_doing(&verb);
    text::capitalize(&if rest.is_empty() { doing } else { format!("{doing} {rest}") })
}

/// Python: bot/compose/taste.py:375 `_is_an_animal(thing)`
///
/// True when the thing of theirs is the animal they live with.
pub fn _is_an_animal(thing: &str) -> bool {
    if _THEIR_OWN.match_(thing).is_none() {
        return false;
    }
    let lower = thing.to_lowercase();
    let words: Vec<&str> = text::split_ref(&lower).into_iter().filter(|w| *w != "my" && *w != "our").collect();
    let Some(last) = words.last() else { return false };
    _AN_ANIMAL.contains(*last) || last.strip_suffix('s').is_some_and(|s| _AN_ANIMAL.contains(s))
}

/// Python: bot/compose/taste.py:388 `_the_stance(thing, liked)`
///
/// (kind, thing) for the words after the verb: a doing, an animal of theirs, or the thing. None
/// when the words after the verb are a clause and not a thing at all.
pub fn _the_stance(thing: &str, liked: bool) -> Option<(String, String)> {
    let doing = _the_doing(thing);
    if !doing.is_empty() {
        return Some((if liked { "like_doing" } else { "dislike_doing" }.to_string(), doing));
    }
    if OPENS_ON_TO.match_(thing).is_some() {
        return None;
    }
    if _is_an_animal(thing) {
        return Some((if liked { "like_pet" } else { "dislike_pet" }.to_string(), thing.to_string()));
    }
    if liked && _THEIR_OWN.match_(thing).is_some() {
        return Some(("like_own".to_string(), thing.to_string()));
    }
    Some((if liked { "like" } else { "dislike" }.to_string(), thing.to_string()))
}

/// Python: bot/compose/taste.py:407 `_clean(text)`
///
/// The line as words: lowercased, filler in front taken off, no trailing punctuation.
pub fn _clean(text_: &str) -> String {
    let said = text::squash(&text_.to_lowercase());
    let said = A_FILLER_OPENING.sub("", &said);
    text::strip_chars(&said, " .!,").to_string()
}

/// Python: bot/compose/taste.py:446 `_a_pointed_at_thing(thing)`
///
/// True for a pointing word that has the thing it points at after it.
pub fn _a_pointed_at_thing(thing: &str) -> bool {
    let lower = thing.to_lowercase();
    let words = text::split_ref(&lower);
    words.len() > 1 && _DEMONSTRATIVE.contains(words[0]) && !matches!(words[1], "one" | "ones" | "sort" | "kind" | "type")
}

/// `" ".join(raw.split()).strip(" .!,;:")` with the tail filler taken off.
fn trimmed(raw: &str) -> String {
    let thing = text::strip_chars(&text::squash(raw), " .!,;:").to_string();
    text::strip_chars(&_TAIL_FILLER.sub("", &thing), " .!,;:").to_string()
}

/// Python: bot/compose/taste.py:453 `_the_thing(raw)`
///
/// The words after the verb as they were said, or "" when they cannot carry a reply.
pub fn _the_thing(raw: &str) -> String {
    let thing = trimmed(raw);
    // The first word decides: a thing that starts with a pronoun names nothing.
    if thing.is_empty() || text::split_ref(&thing).len() > 5 {
        return String::new();
    }
    if _A_PRONOUN.contains(&text::split_ref(&thing)[0].to_lowercase()) && !_a_pointed_at_thing(&thing) {
        return String::new();
    }
    if _A_DEED.match_(&thing).is_some() || _A_SECOND_CLAUSE.search(&thing).is_some() || _NOT_A_THING.match_(&thing).is_some() {
        return String::new();
    }
    // The manner is said about the thing, not part of its name.
    _split_manner(&thing).0
}

/// Python: bot/compose/taste.py:473 `_the_manner(raw)`
///
/// The way they have it, when the object names a thing and a manner: "my tea weak" -> "weak".
pub fn _the_manner(raw: &str) -> String {
    let thing = trimmed(raw);
    if thing.is_empty() || text::split_ref(&thing).len() > 5 {
        return String::new();
    }
    _split_manner(&thing).1
}

/// Python: bot/compose/taste.py:482 `reads_as_a_taste(text)`
///
/// (kind, thing) for a line that only states a like or a dislike, else None.
pub fn reads_as_a_taste(text_: &str) -> Option<(String, String)> {
    _read(text_).map(|(kind, thing, _)| (kind, thing))
}

/// Python: bot/compose/taste.py:493 `_read(text)`
///
/// (kind, thing, manner) for a line that only states a like or a dislike, else None.
pub fn _read(text_: &str) -> Option<Read> {
    let said = _clean(text_);
    if said.is_empty() || said.contains('?') {
        return None;
    }
    // The love lines about a person have their own pool.
    if crate::compose::fallback_chain::_loves_someone(&said) {
        return None;
    }
    let has = |rx: &Re| rx.search(&said).is_some();
    if has(&_A_PERSON) || has(&_A_SOMEONE) || has(&_ABOUT_ME) || has(&_A_PERSON_DESCRIBED) || has(&_A_SYMPTOM) || has(&_ABOUT_WORK) || has(&_TWO_STANCES) {
        return None;
    }
    let readers: [(&Re, Option<&str>); 4] = [(&_A_STANCE, None), (&_A_STANCE_NO_SUBJECT, None), (&_A_DREAD, Some("dread")), (&_A_WAIT, Some("wait"))];
    for (index, (pattern, kind)) in readers.iter().enumerate() {
        let Some(m) = pattern.match_(&said) else { continue };
        let raw_thing = m.name_str("thing");
        if _the_thing(raw_thing).is_empty() {
            // Nothing nameable after the verb.
            return None;
        }
        if _A_ASK.search(raw_thing).is_some() {
            // The line asks something as well as stating the taste.
            return None;
        }
        let raw = text::strip_chars(&text::squash(raw_thing), " .!,;:").to_string();
        if _THE_BOTS.match_(&raw).is_some() {
            // A compliment paid to the bot.
            if let Some(kind) = kind {
                return Some((if *kind == "wait" { "the_bot_liked" } else { "the_bot_disliked" }.to_string(), String::new(), String::new()));
            }
            let verb = text::squash(&m.name_str("verb").to_lowercase());
            return Some((if _LIKE_VERBS.contains(&verb) { "the_bot_liked" } else { "the_bot_disliked" }.to_string(), String::new(), String::new()));
        }
        let manner = _the_manner(&raw);
        if let Some(kind) = kind {
            return Some((kind.to_string(), _the_thing(&raw), String::new()));
        }
        if index == 1 {
            // `_A_STANCE_NO_SUBJECT`: every opener in this pattern is a dislike.
            let (k, t) = _the_stance(&_the_thing(&raw), false)?;
            return Some((if manner.is_empty() { k } else { _a_manner_kind(&k) }, t, manner));
        }
        let verb = text::squash(&m.name_str("verb").to_lowercase());
        let subject = text::squash(&m.name_str("subj").to_lowercase());
        if (subject == "im" || subject == "i'm") && (verb == "like" || verb == "liking") {
            // "im like this every morning" is a state, not a taste.
            return None;
        }
        let thing = _the_thing(&raw);
        let read = if _DISLIKE_VERBS.contains(&verb) {
            _the_stance(&thing, false)
        } else if _LIKE_VERBS.contains(&verb) {
            if verb == "prefer" && A_TO.search(&thing).is_some() {
                // "i prefer tea to coffee" compares two things.
                return None;
            }
            _the_stance(&thing, true)
        } else {
            return None;
        };
        let (k, t) = read?;
        return Some((if manner.is_empty() { k } else { _a_manner_kind(&k) }, t, manner));
    }
    None
}

/// Python: bot/compose/taste.py:577 `_as_a_name(thing)`
///
/// The first word of the thing with its capital back, when it is a name rather than a word.
pub fn _as_a_name(thing: &str) -> String {
    let (first, rest) = match thing.split_once(' ') {
        Some((a, b)) => (a, Some(b)),
        None => (thing, None),
    };
    let head = text::strip_chars(&first.to_lowercase(), ",.").to_string();
    if !_PROPER.contains(&head) {
        return thing.to_string();
    }
    let named = text::upper_first(first);
    match rest {
        Some(r) => format!("{named} {r}"),
        None => named,
    }
}

/// Python: bot/compose/taste.py:586 `_in_a_sentence(thing)`
///
/// The person's own words for the thing, lowercased mid-line: "the rain", not "The rain".
pub fn _in_a_sentence(thing: &str) -> String {
    let head = text::strip_chars(&thing.split(' ').next().unwrap_or("").to_lowercase(), ",.").to_string();
    if thing.is_empty() || (_as_a_name(thing) == thing && _PROPER.contains(&head)) {
        return thing.to_string();
    }
    let mut c = thing.chars();
    let f = c.next().unwrap();
    f.to_lowercase().chain(c).collect()
}

/// Python: bot/compose/taste.py:593 `_as_yours(thing)`
///
/// The person's own thing as the bot would name it: "my flat" is "your flat" to a friend.
pub fn _as_yours(thing: &str) -> String {
    MY_OR_OUR.sub("your", thing)
}

/// Python: bot/compose/taste.py:598 `_at_the_front(thing)`
pub fn _at_the_front(thing: &str) -> String {
    text::upper_first(thing)
}

/// Python: bot/compose/taste.py:603 `reply_for(text, memory=None)`
///
/// The reply for a stated like or dislike, or None when the line is not one.
pub fn reply_for(text_: &str, memory: &Memory) -> Option<String> {
    let (kind, thing, manner) = _read(text_)?;
    let thing = _as_a_name(&_as_yours(&thing));
    // A template that opens on the thing starts a sentence with it.
    let lines = _BY_KIND.get(&kind).unwrap_or_else(|| panic!("KeyError: {kind:?}")).strs();
    let options: Vec<String> = lines
        .iter()
        .map(|line| {
            let t = if line.starts_with("{thing}") { _at_the_front(&thing) } else { _in_a_sentence(&thing) };
            let m = if line.starts_with("{manner}") { _at_the_front(&manner) } else { manner.clone() };
            format_named(line, &[("thing", &t), ("manner", &m)])
        })
        .collect();
    let clean = _clean(text_);
    crate::compose::fallback_chain::_choose(&options, memory, &clean, Some(&clean))
}
