//! Port of `bot/memory/opening.py`: the question about a fact, on the first line of a chat,
//! where the plain receipt would be the whole reply ("My daughter is six next month" -> "Six
//! next month. What would she like to do?").

use super::mem8_py::{pick, rx, strs};
use crate::py::re::{Match, Re, I};
use crate::py::text;
use crate::state::Memory;
use std::sync::LazyLock;

const M: &str = "bot.memory.opening.";

rx!(
    A_DAY_FOR_SOMEONE, A_TIME, A_STANDING_STATE, A_TASTE, A_TASTE_OF_A_PERSON, A_CRUSH, A_PERSON_AS_OBJECT,
    A_START, A_PAST, MONEY, A_MEASURE, A_MONTHS, NOT_AN_OCCASION, A_CORRECTION
);
strs!(SHE, HE, THEY, A_DAY_TAILS, A_LIKED_TAILS, A_DOING_TAILS, A_STATE_TAILS);

static A_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b\d{1,3}\b", 0));
static A_JOIN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:and|but|n)\b", I));
static REPORTED: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        concat!(
            r"^\s*(?:and |so |but |well )?(?:they|he|she|people|someone|everyone)\s+",
            r"(?:said|says|say|reckons?|thinks?|told me|tell me)\s+"
        ),
        0,
    )
});
static STARTS: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*(?:and |so |but |well )?(he|she|they|theyre|they're)\b", 0));

/// Python: bot/memory/opening.py:139 `_the_day_is_the_point(clean, match)`
///
/// Whether the age is what the line is about rather than background. (`found` is the match of
/// A_DAY_FOR_SOMEONE in `clean`.)
pub fn _the_day_is_the_point(clean: &str, found: &Match) -> bool {
    if A_DAY_FOR_SOMEONE.finditer(clean).len() > 1 {
        return false;
    }
    if A_NUMBER.finditer(clean).len() > 1 {
        return false;
    }
    !A_JOIN.is_match(&clean[found.end()..])
}

/// Python: bot/memory/opening.py:150 `_names_my(lower, word)`
///
/// Whether the line names that relation of theirs, however it is typed.
pub fn _names_my(lower: &str, word: &str) -> bool {
    Re::new(&format!(r"\bmy (?:\w+ )?{word}(?:s|'s|s')?\b"), 0).is_match(lower)
}

/// Python: bot/memory/opening.py:156 `_who(clean)`
///
/// ("you", "are") for a fact about the speaker, and the third person for one about somebody named.
pub fn _who(clean: &str) -> (String, String) {
    let lower = clean.to_lowercase();
    // A report of what somebody else said is not a fact about the reporter.
    let lower = REPORTED.sub("", &lower);
    if let Some(starts) = STARTS.match_(&lower) {
        let who = starts.str(1);
        let who = if who.starts_with("they") { "they" } else { who };
        return (who.to_string(), if who == "they" { "are" } else { "is" }.to_string());
    }
    for (words, who, be) in [(&*THEY, "they", "are"), (&*SHE, "she", "is"), (&*HE, "he", "is")] {
        if words.iter().any(|word| _names_my(&lower, word)) {
            return (who.to_string(), be.to_string());
        }
    }
    ("you".to_string(), "are".to_string())
}

/// Python: bot/memory/opening.py:180 `_said_before(memory)`
///
/// Whether anything has been answered yet: a turn the bot has answered.
pub fn _said_before(memory: &Memory) -> bool {
    memory.turns.iter().any(|turn| !text::strip(&turn.bot).is_empty())
}

/// Python: bot/memory/opening.py:189 `the_opening(thing, clean, memory)`
///
/// The question about the fact, where the plain receipt would be the whole reply, or None.
pub fn the_opening(thing: &str, clean: &str, memory: &Memory) -> Option<String> {
    if thing.is_empty() || clean.is_empty() {
        return None;
    }
    if _said_before(memory) {
        return None;
    }
    // A day already had, money, a measure and a baby in months are none of them the occasion a
    // friend asks about.
    if A_PAST.is_match(clean) || A_MEASURE.is_match(clean) || A_MONTHS.is_match(clean) {
        return None;
    }
    if A_CORRECTION.is_match(clean) {
        return None;
    }
    if MONEY.is_match(clean) || MONEY.is_match(thing) || NOT_AN_OCCASION.is_match(thing) {
        return None;
    }
    let (subj, be) = _who(clean);
    let has = if subj == "you" || subj == "they" { "have" } else { "has" };
    let them = match subj.as_str() {
        "she" => "her",
        "he" => "him",
        "they" => "them",
        _ => "you",
    };
    let mut taste = A_TASTE.match_(clean);
    if let Some(t) = &taste {
        let after = &clean[t.end()..];
        if A_TASTE_OF_A_PERSON.is_match(thing)
            || A_CRUSH.is_match(clean)
            || A_PERSON_AS_OBJECT.is_match(after)
            || after.contains(" but ")
        {
            // None of these is one thing loved: they are left to the store.
            taste = None;
        }
    }
    let day = if taste.is_some() { None } else { A_DAY_FOR_SOMEONE.search(clean) };
    let pool: Vec<String> = match &day {
        Some(found) if _the_day_is_the_point(clean, found) && !NOT_AN_OCCASION.is_match(thing) => {
            if subj == "you" {
                // An age of the speaker's own has nothing planned for it: ask about marking it.
                A_DAY_TAILS
                    .iter()
                    .filter(|t| !t.contains("asked for"))
                    .map(|tail| {
                        if tail.contains("would {subj} like to do") {
                            "{Thing} \u{2014} anything you're marking it with?".to_string()
                        } else {
                            tail.clone()
                        }
                    })
                    .collect()
            } else {
                A_DAY_TAILS.clone()
            }
        }
        Some(_) => return None,
        None if taste.is_some() => A_LIKED_TAILS.clone(),
        None if A_TIME.is_match(clean) || A_START.is_match(clean) => A_DOING_TAILS.clone(),
        None if A_STANDING_STATE.is_match(clean) => A_STATE_TAILS.clone(),
        None => return None,
    };
    let said = pick(&pool, memory, clean)
        .replace("{Thing}", &text::upper_first(thing))
        .replace("{subj}", &subj)
        .replace("{be}", &be)
        .replace("{has}", has)
        .replace("{them}", them);
    Some(text::squash(&said))
}
