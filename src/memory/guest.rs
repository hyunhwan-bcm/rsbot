//! Port of `bot/memory/guest.py`: what a guest will not eat, and the answer it changes.
//!
//! "and she hates mushrooms" is one more constraint on the dinner already being planned: the
//! store keeps it against the person, and this revises the suggestion the bot has just given.
//! A food is told apart from the other things a person might hate by the bot's own vocabularies
//! (the cooking topic words and the pantry table).
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, inline_re, res, sets};
use crate::py::re::{self, Re};
use crate::py::text as pt;
use std::collections::HashSet;
use std::sync::LazyLock;

res!("guest"; WORD, STOP, DISLIKE, NOT_MY_THING, NO_FOR_ME, NO_FOOD_ALONE, _WHEN, NO_X_ALONE);
sets!("guest"; GENERIC, PRONOUN, EXTRA, DRINK_PAIR);
inline_re! {
    ARTICLE_FIRST = r"^(?:the|any|some|more|all|a|an)\s+", 0;
    SOFT_LAST = r"\s+(?:too|either|as well|much|really|actually|at all|though|tho|anyway)$", 0;
    SENSE_OF = r"^(?:the\s+)?(?:taste|texture|smell|look|sight)\s+of\s+(?:the\s+)?", re::I;
    A_PLACE_WORD = r"\b(?:on|with|of|in|at)\b", re::I;
    NO_FIRST = r"^(?:no|not)\s+", re::I;
    PIECES = r",|\band\b|\bor\b", 0;
    OR_AND = r"\b(?:or|and)\b", 0;
    AN_ARTICLE = r"^(?:a|an|the)\s+", re::I;
    OPTIONS = r",\s*|\s+or\s+", 0;
    OR_AND_FIRST = r"^(?:or|and)\s+", 0;
    SENTENCE_END = r"(?<=[.!?])\s+", 0;
}

/// Python: bot/memory/guest.py:101 `_fold(word)`
///
/// A word with its plural undone, so mushrooms/mushroom are one food.
pub fn _fold(word: &str) -> String {
    let word = pt::strip_chars(&pt::lower(word), ".,!?;:'\"").to_string();
    let n = pt::len(&word);
    if n > 4 && word.ends_with("ies") {
        return format!("{}y", &word[..word.len() - 3]);
    }
    if n > 4 && word.ends_with("oes") {
        return word[..word.len() - 2].to_string();
    }
    if n > 3 && word.ends_with("es") && "sxzh".contains(word.chars().rev().nth(2).unwrap_or(' ')) {
        return word[..word.len() - 2].to_string();
    }
    if n > 3 && word.ends_with('s') && !word.ends_with("ss") {
        return word[..word.len() - 1].to_string();
    }
    word
}

static KNOWN: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut words: Vec<String> = EXTRA.iter().cloned().collect();
    // The cooking words of the understanding lane: a vocabulary that cannot load only narrows
    // this reader, as in PyBot's try/except.
    if let Some(strong) = data::try_get("bot.understanding.topics.STRONG") {
        for topic in ["cooking", "food-and-drink"] {
            if let Some(said) = strong.get(topic) {
                words.extend(pt::split(said.str()));
            }
        }
    }
    if let Some(pantry) = data::try_get("bot.knowledge.cooking.PANTRY") {
        for (name, entry) in pantry.pairs() {
            words.push(mem7::py_str(name));
            if let Some(aliases) = entry.items().first() {
                words.extend(aliases.items().iter().map(mem7::py_str));
            }
        }
    }
    words.into_iter().filter(|w| !w.is_empty()).map(|w| _fold(&w)).collect()
});

/// Python: bot/memory/guest.py:115 `_known()`
///
/// Every food word the rest of the bot already knows, built once.
pub fn _known() -> &'static HashSet<String> {
    &KNOWN
}

/// Python: bot/memory/guest.py:148 `_clean(item)`
pub fn _clean(item: &str) -> String {
    let item = pt::squash(&pt::lower(pt::strip_chars(item, " .,!?;:'\"")));
    let item = pt::strip(&ARTICLE_FIRST.sub("", &item)).to_string();
    let item = pt::strip(&SOFT_LAST.sub("", &item)).to_string();
    pt::strip_chars(&_WHEN.sub("", &item), " ,").to_string()
}

/// Python: bot/memory/guest.py:166 `_one_half(piece)`
///
/// One half of a pair that was turned down: a food, or a drink named as one.
pub fn _one_half(piece: &str) -> bool {
    if _is_food(piece) {
        return true;
    }
    let words = WORD.findall_str(piece);
    !words.is_empty() && words.iter().all(|w| DRINK_PAIR.contains(&_fold(w)))
}

/// Python: bot/memory/guest.py:174 `_is_food(item)`
pub fn _is_food(item: &str) -> bool {
    if item.is_empty() || GENERIC.contains(item) {
        return false;
    }
    let words = WORD.findall_str(item);
    if words.is_empty() || PRONOUN.contains(&words[0]) {
        return false;
    }
    words.iter().any(|w| KNOWN.contains(&_fold(w)))
}

/// Python: bot/memory/guest.py:184 `plain(item, a_pair=False)`
///
/// The food itself for a dislike of the user's own, or None when the phrase only contains one.
pub fn plain(item: &str, a_pair: bool) -> Option<String> {
    let item = _clean(item);
    let item = pt::strip(&SENSE_OF.sub("", &item)).to_string();
    if !(_is_food(&item) || (a_pair && _one_half(&item))) || A_PLACE_WORD.is_match(&item) {
        return None;
    }
    if !WORD.findall_str(&item).iter().all(|w| KNOWN.contains(&_fold(w))) {
        return None;
    }
    Some(item)
}

/// Python: bot/memory/guest.py:207 `found(text)`
///
/// [(food, where)] in the line: [("mushrooms", 15)] for "and she hates mushrooms". The position
/// is a byte offset into `text` (Python's is a code point), which is what `facts.subject`, the
/// caller's next step, reads.
pub fn found(text: &str) -> Vec<(String, i64)> {
    let mut out: Vec<(String, i64)> = Vec::new();
    let patterns: [(&Re, bool); 4] = [(*DISLIKE, false), (*NOT_MY_THING, false), (*NO_FOR_ME, false), (*NO_FOOD_ALONE, true)];
    for (pattern, alone) in patterns {
        for m in pattern.finditer(text) {
            let chunk = STOP.split(m.name_str("food")).into_iter().next().unwrap_or_default();
            let pieces: Vec<String> = PIECES
                .split(&chunk)
                .iter()
                .map(|piece| _clean(&NO_FIRST.subn("", pt::strip(piece), 1)))
                .filter(|piece| !piece.is_empty())
                .collect();
            // Two halves that are both things to eat or drink are the pair form.
            let a_pair = pieces.len() >= 2 && pieces.iter().all(|p| _one_half(p));
            if alone && !a_pair {
                continue;
            }
            let at = m.name_span("food").map_or(0, |s| s.0) as i64;
            for item in pieces {
                if (_is_food(&item) || a_pair) && !out.iter().any(|(food, _)| *food == item) {
                    out.push((item, at));
                }
            }
        }
    }
    out
}

/// Python: bot/memory/guest.py:240 `turned_down_alone(text)`
///
/// The foods a bare "no mushrooms" names, or [] when the line says anything more.
pub fn turned_down_alone(text: &str) -> Vec<String> {
    let squashed = pt::squash(text);
    let Some(m) = NO_X_ALONE.match_(&squashed) else {
        return Vec::new();
    };
    let item = _clean(m.name_str("food"));
    if item.is_empty() || OR_AND.is_match(&item) {
        // A pair is `found`'s to read.
        return Vec::new();
    }
    if _is_food(&item) {
        vec![item]
    } else {
        Vec::new()
    }
}

/// Python: bot/memory/guest.py:252 `_contains(text, food)`
pub fn _contains(text: &str, food: &str) -> bool {
    let root = _fold(food);
    WORD.findall_str(text).iter().any(|w| _fold(w) == root)
}

/// Python: bot/memory/guest.py:257 `_without_article(item)`
pub fn _without_article(item: &str) -> String {
    AN_ARTICLE.sub("", pt::strip(item))
}

/// Python: bot/memory/guest.py:261 `_split_list(rest)`
///
/// The options in "a curry, a lasagne, or a traybake", or None for half a sentence.
pub fn _split_list(rest: &str) -> Option<Vec<String>> {
    let items: Vec<String> = OPTIONS
        .split(rest)
        .iter()
        .filter(|part| !pt::strip_chars(part, " .").is_empty())
        .map(|part| OR_AND_FIRST.sub("", pt::strip_chars(part, " .")))
        .collect();
    // An option is a short thing on a menu, not half a sentence.
    if items.iter().any(|item| WORD.findall_str(item).len() > 5) {
        return None;
    }
    Some(items)
}

/// Python: bot/memory/guest.py:275 `_kept_phrase(items)`
pub fn _kept_phrase(items: &[String]) -> String {
    match items.len() {
        1 => format!("{} still works", items[0]),
        2 => format!("{} or {} both still work", items[0], items[1]),
        _ => {
            let (last, rest) = items.split_last().expect("IndexError: no options");
            format!("{} or {} all still work", rest.join(", "), last)
        }
    }
}

/// Python: bot/memory/guest.py:283 `revise(reply, foods)`
///
/// The last answer with the disliked food taken out, or None when it does not mention it.
pub fn revise(reply: &str, foods: &Value) -> Option<String> {
    let foods: Vec<String> = foods.items().iter().map(mem7::py_str).collect();
    if foods.is_empty() {
        return None;
    }
    for sentence in SENTENCE_END.split(pt::strip(reply)) {
        let hit: Vec<&String> = foods.iter().filter(|food| _contains(&sentence, food)).collect();
        if hit.is_empty() {
            continue;
        }
        let (_head, sep, rest) = pt::partition(&sentence, ": ");
        if sep.is_empty() {
            continue;
        }
        let Some(items) = _split_list(rest).filter(|items| !items.is_empty()) else {
            continue;
        };
        let removed: Vec<String> = items.iter().filter(|item| hit.iter().any(|food| _contains(item, food))).cloned().collect();
        let kept: Vec<String> = items.iter().filter(|item| !removed.contains(item)).cloned().collect();
        if removed.is_empty() || kept.is_empty() {
            continue;
        }
        let mut gone = format!("the {}", removed.iter().map(|item| _without_article(item)).collect::<Vec<_>>().join(" and the "));
        gone.push_str(if removed.len() == 1 { " is out" } else { " are out" });
        return Some(format!("Then {gone} \u{2014} {}.", _kept_phrase(&kept)));
    }
    None
}

/// Python: bot/memory/guest.py:313 `left_out(foods)`
///
/// A short reply to a food the user themselves will not eat, with no suggestion to edit.
pub fn left_out(foods: &Value) -> Option<String> {
    let mut items: Vec<String> = Vec::new();
    for food in mem7::iterate(foods) {
        let food = pt::strip(&mem7::py_str(&food)).to_string();
        if !food.is_empty() && !items.contains(&food) {
            items.push(food);
        }
    }
    let said = if items.len() == 2 { items.join(" or ") } else { items.join(", ") };
    if said.is_empty() {
        return None;
    }
    Some(format!("Fair enough \u{2014} no {said} in anything I suggest."))
}
