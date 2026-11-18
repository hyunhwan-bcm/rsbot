//! Port of `bot/memory/taste.py`: a taste they have told the bot about ("i love dogs", "not
//! keen on fish", "not sure if i like curry").
//!
//! A taste is a fact about the person, so it is said back like any other, but it is not a pet.
//! `read()` is the taste itself: the thing, and whether they like it, do not, or are not sure.
//!
//! Positions are byte offsets into the text (Python's are code points); where Python counts
//! characters back from a position (`said_about`), the port counts characters too.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::py::re::{self, Match, Re};
use crate::py::text as pt;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! tres {
    ($($n:ident),* $(,)?) => { $(
        static $n: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.memory.taste.", stringify!($n))));
    )* };
}
macro_rules! tsets {
    ($($n:ident),* $(,)?) => { $(
        static $n: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.memory.taste.", stringify!($n))));
    )* };
}

tsets!(TAIL, THING_STOP, NOT_A_THING);
tres!(
    UNSURE, NO, NOT_KEEN, HATE, MY_THING, YES, LIKING_BEFORE, OWNED_BEFORE, ASKS_ABOUT_A_TASTE,
    _QUANTITY, _SHORTFALL_TAIL, _SHORTFALL_VERB, _CLAUSE_SPLIT, _ASK_FORM, _ASKS_FOR_A_THING,
    _HATED_CHORE, _OTHER_OWNER, _OTHER_CLAUSE, _A_CLAUSE_AFTER, _ASKS_SOMETHING, _OTHER_TASTE,
);

/// `re.match(_TASTE_VERB, clause, re.I)`.
static _TASTE_VERB: LazyLock<Re> = LazyLock::new(|| Re::new(&data::string("bot.memory.taste._TASTE_VERB"), re::I));
/// `_OTHER_SAYS`: the verb a taste arrives in, and the one the reader understands
/// (`re.fullmatch(pattern, verb, re.I)`).
static _OTHER_SAYS: LazyLock<Vec<(Re, String)>> = LazyLock::new(|| {
    data::get("bot.memory.taste._OTHER_SAYS")
        .items()
        .iter()
        .map(|pair| (Re::new(pair.items()[0].str(), re::I), pair.items()[1].str().to_string()))
        .collect()
});
static TO_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*to\b", 0));
static MY_OUR_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*(?:my|our)\b", re::I));
static A_PERSON_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:i|we|he|she|they)\b", re::I));
static A_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static EATS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:eat|eats|ate|drink|drinks|drank)\b", re::I));

const OWNERS: [&str; 6] = ["my", "our", "his", "her", "their", "your"];

/// `{"thing": thing, "how": how}`.
fn a_taste(thing: &str, how: &str) -> Value {
    let mut taste = Value::dict();
    taste.set_key("thing", thing);
    taste.set_key("how", how);
    taste
}

/// Python: bot/memory/taste.py:117 `_kept(thing)`
///
/// The words of the thing that are kept, with the index each had in `thing`.
pub fn _kept(thing: &str) -> Vec<(usize, String)> {
    let mut kept: Vec<(usize, String)> = Vec::new();
    for (index, raw) in pt::split_ref(thing).into_iter().enumerate() {
        let word = pt::strip_chars(raw, ".,;:!?");
        if word.is_empty() {
            continue;
        }
        let low = pt::lower(word);
        if !kept.is_empty() && (THING_STOP.contains(&low) || OWNERS.contains(&low.as_str())) {
            // A thing has one owner in it at most: "fish but my kids love it" ends at "fish".
            break;
        }
        if kept.is_empty() && TAIL.contains(&low) {
            continue;
        }
        kept.push((index, word.to_string()));
    }
    while kept.last().is_some_and(|(_, word)| TAIL.contains(&pt::lower(word))) {
        kept.pop();
    }
    if kept.is_empty() || NOT_A_THING.contains(&pt::lower(&kept[0].1)) {
        return Vec::new();
    }
    kept
}

/// Python: bot/memory/taste.py:139 `_clean(thing)`
///
/// The thing with its tail off ("curry tho" -> "curry"), or None when it names nothing.
pub fn _clean(thing: &str) -> Option<String> {
    let kept = _kept(thing);
    if kept.is_empty() {
        return None;
    }
    let mut words: Vec<String> = kept.into_iter().map(|(_, word)| word).collect();
    // "we love our dog" is a taste about the animal they have: the store reads it from the bot's
    // side, so the possessive becomes "your" ("you love your dog").
    if matches!(pt::lower(&words[0]).as_str(), "my" | "our") {
        words[0] = "your".to_string();
    }
    Some(words.join(" "))
}

/// Python: bot/memory/taste.py:152 `_thing_end(text, match)`
///
/// Where the thing ends in the clause (a byte offset), so the words after it can be looked at.
pub fn _thing_end(text: &str, r#match: &Match) -> usize {
    let (start, end) = r#match.span(1).expect("the thing is group 1 of every taste pattern");
    let took = &text[start..end];
    let kept = _kept(took);
    let Some((last_index, _)) = kept.last() else {
        return start;
    };
    let last = pt::split_ref(took)[*last_index];
    // `took.rfind(last)` cannot miss: `last` is one of the words of `took`.
    start + took.rfind(last).unwrap_or(0) + last.len()
}

/// Python: bot/memory/taste.py:176 `_read_clause(text)`
///
/// `(the taste in the clause, where in the clause it ends)`, or `(None, None)`. The clause the
/// end is an offset into is the text with its whitespace squashed, as Python's is.
pub fn _read_clause(text: &str) -> (Option<Value>, Option<usize>) {
    let text = pt::squash(text);
    let text = text.as_str();
    if text.is_empty() || text.ends_with('?') {
        return (None, None); // "do i like fish?" is a question about them, not a taste they have
    }
    let patterns: [(&Re, &str); 5] =
        [(*UNSURE, "unsure"), (*NO, "no"), (*NOT_KEEN, "no"), (*MY_THING, "no"), (*YES, "yes")];
    for (pattern, how) in patterns {
        if let Some(m) = pattern.search(text) {
            let raw = m.str(1);
            if TO_FIRST.match_(raw).is_some() {
                // "i dont want to lose the house", "i want to stop drinking": the verb is about
                // something they mean to do, not about a thing they like or dislike.
                return (None, None);
            }
            let thing = _clean(raw);
            // `_kept` drops a leading quantity ("much water" -> "water"), so the check is made on
            // the words as they came as well as on the cleaned thing.
            let start = m.span(1).map_or(0, |span| span.0);
            let shortfall = _SHORTFALL_VERB.search(&text[..start]).is_some();
            if let Some(thing) = thing.filter(|thing| !thing.is_empty()) {
                if _QUANTITY.match_(&thing).is_none()
                    && _QUANTITY.match_(pt::strip(raw)).is_none()
                    && !(shortfall && _SHORTFALL_TAIL.search(raw).is_some())
                {
                    return (Some(a_taste(&thing, how)), Some(_thing_end(text, &m)));
                }
            }
        }
    }
    if let Some(m) = HATE.search(text) {
        // "i hate my job" is about their week, not a taste: the possessive on the raw thing is
        // how that is told apart from "i hate cats".
        let raw = m.str(1);
        if let Some(thing) = _clean(raw).filter(|thing| !thing.is_empty()) {
            if MY_OUR_FIRST.match_(raw).is_none() {
                return (Some(a_taste(&thing, "no")), Some(_thing_end(text, &m)));
            }
        }
    }
    (None, None)
}

/// Python: bot/memory/taste.py:215 `read(line)`
///
/// The taste in the line as `{"thing", "how"}`, or None. `how` is "unsure", "no" or "yes".
pub fn read(line: &str) -> Value {
    _read_clause(line).0.unwrap_or(Value::None)
}

/// "hate cats" carries the person from the clause before it: whether the clause opens with a
/// taste verb and no person.
fn bare_verb(clause: &str) -> bool {
    _TASTE_VERB.match_(clause).is_some() && A_PERSON_FIRST.match_(clause).is_none()
}

/// Python: bot/memory/taste.py:276 `read_all(line)`
///
/// Every taste in the line, in the order they said it (a list of `{"thing", "how"}`).
pub fn read_all(line: &str) -> Value {
    let text = pt::squash(line);
    let mut found: Vec<Value> = Vec::new();
    for clause in _CLAUSE_SPLIT.split(&text) {
        let mut clause = pt::strip(&clause).to_string();
        if clause.is_empty() {
            continue;
        }
        if bare_verb(&clause) {
            clause = format!("i {clause}");
        }
        let taste = read(&clause);
        if taste.truthy() && !found.contains(&taste) {
            found.push(taste);
        }
    }
    Value::from(found)
}

/// `set(re.findall(r"[a-z']+", left.lower())) - TAIL` is not empty.
fn more_than_a_tail(left: &str) -> bool {
    A_WORD.findall_str(&pt::lower(left)).iter().any(|word| !TAIL.contains(word))
}

/// Python: bot/memory/taste.py:298 `only_a_taste(text)`
///
/// Whether the line says nothing but tastes of theirs, one per clause.
pub fn only_a_taste(text: &str) -> bool {
    let text = pt::squash(text);
    if text.is_empty() || text.contains('?') {
        return false;
    }
    // A how-to that names a chore they hate is a how-to, not a taste to file away.
    if _ASK_FORM.is_match(&text) && _HATED_CHORE.is_match(&text) {
        return false;
    }
    // A request for a thing is answered, not noted, whatever tastes are attached to it.
    if _ASKS_FOR_A_THING.is_match(&text) {
        return false;
    }
    let mut seen = false;
    for clause in _CLAUSE_SPLIT.split(&text) {
        let clause = pt::strip(&clause);
        if clause.is_empty() {
            continue;
        }
        let mut said = clause.to_string();
        if bare_verb(&said) {
            said = format!("i {said}");
        }
        let (taste, end) = _read_clause(&said);
        if taste.is_none() {
            return false;
        }
        // `said` is already squashed (it is a piece of a squashed text), so the end is an
        // offset into it.
        if more_than_a_tail(&said[end.unwrap_or(0)..]) {
            return false;
        }
        seen = true;
    }
    seen
}

/// The byte offset `count` characters before `position` (Python's `max(0, position - count)`).
fn chars_back(text: &str, position: usize, count: usize) -> usize {
    text[..position].char_indices().rev().take(count).last().map_or(position, |(at, _)| at)
}

/// Python: bot/memory/taste.py:342 `said_about(text, position)`
///
/// Whether the animal at `position` is what a taste in the line is about, not their own pet.
/// `position` is a byte offset into `text`.
pub fn said_about(text: &str, position: i64) -> bool {
    let position = (position.max(0) as usize).min(text.len());
    let before = &text[chars_back(text, position, 30)..position];
    if !LIKING_BEFORE.is_match(before) {
        return false;
    }
    !OWNED_BEFORE.is_match(&text[chars_back(text, position, 12)..position])
}

/// Python: bot/memory/taste.py:371 `asks_about_a_taste(line)`
///
/// True when the line is a question about a taste of theirs ("do i like fish?").
pub fn asks_about_a_taste(line: &str) -> bool {
    ASKS_ABOUT_A_TASTE.is_match(line)
}

/// Python: bot/memory/taste.py:442 `read_other(line)`
///
/// Tastes somebody else has, as `[{"thing", "how", ("eaten",) "who"/"pronoun"}]`.
///
/// CONTRACT: the skeleton typed this `Vec<String>`; PyBot returns a list of dicts.
pub fn read_other(line: &str) -> Vec<Value> {
    let text = pt::squash(line);
    if text.is_empty() || text.ends_with('?') {
        return Vec::new();
    }
    let mut found: Vec<Value> = Vec::new();
    let mut person: Option<(&str, String)> = None;
    for clause in _OTHER_CLAUSE.split(&text) {
        let mut clause = pt::strip(&clause).to_string();
        if clause.is_empty() {
            continue;
        }
        let owned = _OTHER_OWNER.match_(&clause).map(|m| {
            let person = match m.name("who").filter(|who| !who.is_empty()) {
                Some(who) => ("who", pt::lower(who)),
                None => ("pronoun", pt::lower(m.name_str("pronoun"))),
            };
            (person, m.name_str("rest").to_string())
        });
        if let Some((named, rest)) = owned {
            person = Some(named);
            clause = rest;
        }
        let Some((key, name)) = &person else {
            // Nobody has been named yet, so a taste with no subject of its own ("cant stand
            // museums") belongs to nobody this reader can name.
            continue;
        };
        let Some(said) = _OTHER_TASTE.match_(&clause) else {
            continue;
        };
        if let Some(mut tasted) = _their_taste_in(&clause, said.as_str(), said.end()) {
            tasted.set_key(*key, name.as_str());
            found.push(tasted);
        }
    }
    found
}

/// Python: bot/memory/taste.py:479 `_their_taste_in(rest, said)`
///
/// The taste one clause gives, read as the person's own ("i love cooking"), or None. `said` is
/// the `_OTHER_TASTE` match at the start of `rest`, given as its text and its end.
pub fn _their_taste_in(rest: &str, said: &str, said_end: usize) -> Option<Value> {
    let verb = pt::strip(said);
    let thing = pt::strip(&rest[said_end..]);
    let mut as_own = "love";
    for (pattern, rewritten) in _OTHER_SAYS.iter() {
        if pattern.fullmatch(verb).is_some() {
            as_own = rewritten;
            break;
        }
    }
    if thing.is_empty() {
        return None;
    }
    let clause = format!("i {as_own} {thing}");
    let (taste, end) = _read_clause(&clause);
    let taste = taste?;
    // `_read_clause` squashes the clause before it reads it, and the end is an offset into the
    // squashed text, while Python slices the clause as it was built. The two differ only when
    // `thing` holds a run of whitespace, which a clause of a squashed line never does.
    let left = &clause[end.unwrap_or(0).min(clause.len())..];
    if more_than_a_tail(left) {
        // What is left over is a second clause of its own ("he hates walking so keep it short")
        // or the thing carried on. Only a clause that starts where the taste ends can follow
        // it, and never one that asks the bot something.
        if _A_CLAUSE_AFTER.match_(left).is_none() || _ASKS_SOMETHING.is_match(left) {
            return None;
        }
    }
    let mut tasted = a_taste(taste.get("thing").map_or("", Value::str), taste.get("how").map_or("", Value::str));
    if EATS.is_match(verb) {
        // "she doesnt eat meat" is how she eats, which the diet reader already has.
        tasted.set_key("eaten", true);
    }
    Some(tasted)
}
