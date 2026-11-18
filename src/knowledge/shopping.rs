//! Port of `bot/knowledge/shopping.py`: buying things - supermarkets, presents, coats, watches,
//! haggling and whether a purchase is worth it. The entries and every table are data from the
//! bundle; this is the logic their guards and replies run, and the present helpers
//! `modules/answers.py` calls (`present_within`, `present_less`, `the_card_after_a_present`,
//! `a_present_was_answered`, `the_verdict_on_a_present`).
//!
//! Build-time only (not ported): `_openings`, which builds PRESENT_OPENINGS (in the bundle).

use crate::data::{self, FnRef, Value};
use crate::knowledge::d2_shared::{self, any_in, asks_where, asks_where_a_shop_answers, pick_in};
use crate::knowledge::d2dup_g7::{
    about_them, cached_re, gift_ask, last_bot, py_field, py_format, recent, the_thread_has_a_new_baby, _age,
    _with_an_age,
};
use crate::knowledge::d2dup_g7 as people;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{self as pyre, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn name(n: &str) -> String {
    format!("bot.knowledge.shopping.{n}")
}
fn re(n: &str) -> &'static Re {
    data::re(&name(n))
}
fn strs(n: &str) -> Vec<String> {
    data::strs(&name(n))
}
fn string(n: &str) -> String {
    data::string(&name(n))
}
fn set(n: &str) -> HashSet<String> {
    data::set(&name(n))
}
/// A table of (names, reply) rows.
fn rows(n: &str) -> Vec<(Vec<String>, String)> {
    data::get(&name(n)).items().iter().map(|r| (r.items()[0].strs(), r.items()[1].str().to_string())).collect()
}

/// One shop of SUPERMARKETS, in the order the price logic sorts by: rank first, then the name.
type Shop = (i64, String, String, String, String);

static SUPERMARKETS: LazyLock<Vec<(String, Vec<String>, i64, String, String, String)>> = LazyLock::new(|| {
    data::get(&name("SUPERMARKETS"))
        .items()
        .iter()
        .map(|r| {
            let f = r.items();
            (
                f[0].str().to_string(),
                f[1].strs(),
                f[2].as_int().unwrap(),
                f[3].str().to_string(),
                f[4].str().to_string(),
                f[5].str().to_string(),
            )
        })
        .collect()
});
static _CHEAPEST_TAIL: LazyLock<String> = LazyLock::new(|| string("_CHEAPEST_TAIL"));
static _MIDDLE_TAIL: LazyLock<String> = LazyLock::new(|| string("_MIDDLE_TAIL"));
static _DEAR_TAIL: LazyLock<String> = LazyLock::new(|| string("_DEAR_TAIL"));
static _QUALITY_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_QUALITY_WORDS"));
static _RANGE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_RANGE_WORDS"));
static _SERVICE_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SERVICE_WORDS"));
static _SHOP_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_SHOP_ASK"));
static PERSON_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("PERSON_WORDS"));
static TASTE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("TASTE_WORDS"));
static PLURAL_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("PLURAL_WORDS"));
static _FOLD: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get(&name("_FOLD")).items().iter().map(|r| (r.items()[0].str().to_string(), r.items()[1].str().to_string())).collect()
});
static _GIFT_ANYONE: LazyLock<String> = LazyLock::new(|| string("_GIFT_ANYONE"));
static CHEAP_CHILD: LazyLock<String> = LazyLock::new(|| string("CHEAP_CHILD"));
static CHEAP_TEEN: LazyLock<String> = LazyLock::new(|| string("CHEAP_TEEN"));
static CHEAP_ADULT: LazyLock<String> = LazyLock::new(|| string("CHEAP_ADULT"));
static CHILD_WORDS: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    strs("CHILD_WORDS")
        .iter()
        .map(|w| cached_re(&format!("(?<![a-z]){}(?![a-z])", pyre::escape(w)), 0))
        .collect()
});
static INTEREST_GIFTS: LazyLock<Vec<(String, Vec<(String, f64)>)>> = LazyLock::new(|| {
    data::get(&name("INTEREST_GIFTS"))
        .pairs()
        .iter()
        .map(|(k, v)| {
            let picks = v.items().iter().map(|p| (p.items()[0].str().to_string(), p.items()[1].as_f64().unwrap())).collect();
            (k.str().to_string(), picks)
        })
        .collect()
});
static A_CARD_IS_NOT_A_PRESENT: LazyLock<&'static Re> = LazyLock::new(|| re("A_CARD_IS_NOT_A_PRESENT"));
static NOT_THAT_CARD: LazyLock<&'static Re> = LazyLock::new(|| re("NOT_THAT_CARD"));
static CARD_AFTER_A_PRESENT: LazyLock<Vec<String>> = LazyLock::new(|| strs("CARD_AFTER_A_PRESENT"));
static GIFT_FOR_A_CONDITION: LazyLock<Vec<(Vec<String>, String, Vec<String>)>> = LazyLock::new(|| {
    data::get(&name("GIFT_FOR_A_CONDITION"))
        .items()
        .iter()
        .map(|r| (r.items()[0].strs(), r.items()[1].str().to_string(), r.items()[2].strs()))
        .collect()
});
static _PRESENT_PEOPLE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PRESENT_PEOPLE"));
static A_VERDICT_THING: LazyLock<&'static Re> = LazyLock::new(|| re("A_VERDICT_THING"));
static A_VERDICT_ON_A_PRESENT: LazyLock<&'static Re> = LazyLock::new(|| re("A_VERDICT_ON_A_PRESENT"));
static A_VERDICT_ABOUT_MONEY: LazyLock<&'static Re> = LazyLock::new(|| re("A_VERDICT_ABOUT_MONEY"));
static MONEY_VERDICT_YES: LazyLock<Vec<String>> = LazyLock::new(|| strs("MONEY_VERDICT_YES"));
static VERDICT_YES: LazyLock<Vec<String>> = LazyLock::new(|| strs("VERDICT_YES"));
static NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| set("NOT_THE_THING"));
static PRICE_SAID: LazyLock<&'static Re> = LazyLock::new(|| re("PRICE_SAID"));
static A_CHEAP_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("A_CHEAP_ASK"));
static PLURAL_ONLY: LazyLock<&'static Re> = LazyLock::new(|| re("PLURAL_ONLY"));
static CHILD_GIFT: LazyLock<String> = LazyLock::new(|| string("CHILD_GIFT"));
static TODDLER_GIFT: LazyLock<String> = LazyLock::new(|| string("TODDLER_GIFT"));
static TEEN_GIFT: LazyLock<String> = LazyLock::new(|| string("TEEN_GIFT"));
static GROWN_GIFT: LazyLock<String> = LazyLock::new(|| string("GROWN_GIFT"));
static PRESENT_OPENINGS: LazyLock<Vec<String>> = LazyLock::new(|| strs("PRESENT_OPENINGS"));
static _METER_FAULT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_METER_FAULT"));
static _METER_READ: LazyLock<Vec<String>> = LazyLock::new(|| strs("_METER_READ"));
static _DECISION_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_DECISION_WORDS"));
static _TRADER_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_TRADER_WORDS"));
static CHOICE_VALUE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| rows("CHOICE_VALUE"));
static APPLIANCE_VALUE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| rows("APPLIANCE_VALUE"));
static EVERYDAY_WHERE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| rows("EVERYDAY_WHERE"));
static APPLIANCE_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("APPLIANCE_ASK"));
static _ACTION_NAME: LazyLock<&'static Re> = LazyLock::new(|| re("_ACTION_NAME"));
static _DRYER_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_DRYER_WORDS"));
static _INSIDE: LazyLock<&'static Re> = LazyLock::new(|| re("_INSIDE"));
static _SHOP_PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| set("_SHOP_PLACES"));
static _MINE: LazyLock<&'static Re> = LazyLock::new(|| re("_MINE"));
static TURNING: LazyLock<&'static Re> = LazyLock::new(|| re("TURNING"));
static CHILD_PEOPLE: LazyLock<Vec<String>> = LazyLock::new(|| strs("CHILD_PEOPLE"));
static _SAID_AS_WORDS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    data::get(&name("_SAID_AS_WORDS")).pairs().iter().map(|(k, v)| (k.str().to_string(), v.as_int().unwrap())).collect()
});
static _AN_AGE_SAID: LazyLock<&'static Re> = LazyLock::new(|| re("_AN_AGE_SAID"));
static _A_CHILDS_AGE: LazyLock<&'static Re> = LazyLock::new(|| re("_A_CHILDS_AGE"));
static _WATCH_SMART: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WATCH_SMART"));
static _WATCH_SMART_SAY: LazyLock<String> = LazyLock::new(|| string("_WATCH_SMART_SAY"));
static _WATCH_VERBS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WATCH_VERBS"));
static _WATCH_VERB_SHAPE: LazyLock<&'static Re> = LazyLock::new(|| re("_WATCH_VERB_SHAPE"));
static _WATCH_PLAIN: LazyLock<String> = LazyLock::new(|| string("_WATCH_PLAIN"));
static PRESENT_OVER: LazyLock<String> = LazyLock::new(|| string("PRESENT_OVER"));
static PRESENT_BANDS: LazyLock<Vec<(f64, String)>> = LazyLock::new(|| {
    data::get(&name("PRESENT_BANDS"))
        .items()
        .iter()
        .map(|r| (r.items()[0].as_f64().unwrap(), r.items()[1].str().to_string()))
        .collect()
});
static BUY_INTO: LazyLock<String> = LazyLock::new(|| string("BUY_INTO"));
static _HEIGHT: LazyLock<&'static Re> = LazyLock::new(|| re("_HEIGHT"));
static _COAT_KINDS: LazyLock<Vec<(Vec<&'static Re>, String)>> = LazyLock::new(|| {
    rows("_COAT_KINDS")
        .into_iter()
        .map(|(words, reply)| (words.iter().map(|w| cached_re(&format!("\\b{}", pyre::escape(w)), 0)).collect(), reply))
        .collect()
});
static _COAT_CHECK: LazyLock<String> = LazyLock::new(|| string("_COAT_CHECK"));
static _COAT_CHECK_WORDS: LazyLock<Vec<&'static Re>> = LazyLock::new(|| {
    strs("_COAT_CHECK_WORDS").iter().map(|w| cached_re(&format!("\\b{}", pyre::escape(w)), 0)).collect()
});
static _NAMED_BY_PRONOUN: LazyLock<HashSet<String>> = LazyLock::new(|| set("_NAMED_BY_PRONOUN"));
static _KEEP_RECEIPT_RE: LazyLock<&'static Re> = LazyLock::new(|| re("_KEEP_RECEIPT_RE"));

static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z']+", 0));
static LOWER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z]+", 0));
static DECISION_TOKENS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z0-9']+", 0));

/// `str(x)` for an age the way the verdict prints it (an int, or a float's repr).
fn age_text(held: f64) -> String {
    if held == held.trunc() { format!("{}", held as i64) } else { text::float_repr(held) }
}

/// `" ".join(str(part) for part in (clean, text, " ".join(words))).lower()`.
fn said_all(u: &Understood) -> String {
    format!("{} {} {}", u.clean, u.text, u.words.join(" ")).to_lowercase()
}

/// How shops compare on quality, which is a different answer from how they compare on price.
pub fn _supermarket_quality(mut named: Vec<Shop>) -> String {
    named.sort();
    if named.len() >= 2 {
        let (cheap, dear) = (&named[0], &named[named.len() - 1]);
        if cheap.0 < dear.0 {
            return format!(
                "On quality they are close on the staples: most own-brand basics are made in the same factories as the \
named ones, so the store cupboard is the same food under a different label. It shows in the fresh food and the range \
- {} {}, where {} {}. For a full shop the cheaper one is fine; for a specific ingredient, or for a weekend meal, the \
dearer one earns it.",
                cheap.1, cheap.4, dear.1, dear.4
            );
        }
        return format!(
            "On quality they are closer than on price: the basics are the same food under a different label, so what \
you pay for is the fresh food and the range - {} {}, and {} {}.",
            cheap.1,
            cheap.4,
            dear.1,
            _without_the_same_opening(&cheap.4, &dear.4)
        );
    }
    let (rank, name, sentence, why, _contrast) = &named[0];
    if *rank >= 4 {
        return format!(
            "{name} is {sentence}: {why}. The extra money goes on the parts of the shop the staples do not cover, not on \
the basics, which are the same items under a different label, so a full shop there costs more for much the same basket."
        );
    }
    if *rank <= 1 {
        return format!(
            "{name} is {sentence}: {why}. None of that makes the staples worse: most own-brand basics come from the same \
factories as the named ones, so the saving is on the same food, and a basket built on ingredients rather than ready-made \
versions of them misses the weakest part."
        );
    }
    if *rank == 3 {
        return format!(
            "{name} is {sentence}: {why}. Much the same as the rest on quality: the basics are the same items, and what \
the extra buys is convenience."
        );
    }
    format!(
        "{name} is {sentence}: {why}. Much the same as the rest on quality: the basics are the same items at a \
price-matched price, and what it adds is range and fresh food."
    )
}

/// The second shop's clause with the words the first has already used taken off the front.
pub fn _without_the_same_opening(first: &str, second: &str) -> String {
    for lead in ["costs more, and ", "costs more and ", "costs more, ", "costs more "] {
        if first.starts_with("costs more") && second.starts_with(lead) {
            return text::strip(&second[lead.len()..]).to_string();
        }
    }
    second.to_string()
}

/// True when the line is about something a shop sells besides food.
pub fn _a_service_not_a_basket(line: &str) -> bool {
    any_in(line, &_SERVICE_WORDS)
}

/// The words of the line the person has just sent, whatever reading carried the entry here.
pub fn _the_turn_just_typed(memory: &Memory) -> String {
    recent(memory, 1).last().map(|t| t.user.clone()).unwrap_or_default()
}

/// True when the line just says where they shop, with no question about it.
pub fn _a_statement_about_where_i_shop(typed: &str) -> bool {
    if !["i shop at", "i shop in", "i get my shopping"].iter().any(|p| typed.contains(p)) {
        return false;
    }
    !any_in(typed, &_SHOP_ASK)
}

/// Where the shops named in the line sit on price, or None when it is about something else.
pub fn _supermarket_cheapest(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let typed = _the_turn_just_typed(memory).to_lowercase();
    if any_in(&typed, &_RANGE_WORDS) {
        return None;
    }
    let line = _said_line(Some(understood)).to_lowercase().replace('\u{2019}', "'");
    if any_in(&line, &_RANGE_WORDS) {
        return None;
    }
    if _a_service_not_a_basket(&line) {
        return None;
    }
    if _a_statement_about_where_i_shop(&line) {
        return None;
    }
    let mut named: Vec<Shop> = Vec::new();
    for (name, spellings, rank, sentence, why, contrast) in SUPERMARKETS.iter() {
        if spellings.iter().any(|s| line.contains(s.as_str())) {
            named.push((*rank, name.clone(), sentence.clone(), why.clone(), contrast.clone()));
        }
    }
    if !named.is_empty() && any_in(&line, &_QUALITY_WORDS) {
        return Some(_supermarket_quality(named.clone()));
    }
    if named.is_empty() {
        return Some(
            "Aldi and Lidl are the cheapest of the big names for own-brand staples, around a fifth less than Tesco or \
Sainsbury's. Asda is usually the cheapest of the big four, Morrisons, Tesco and Sainsbury's come next and price-match \
Aldi on the basics, and Waitrose and M&S are the dearest. A convenience shop - the Co-op, a corner shop - costs more \
again, so it is for a top-up, not the weekly shop."
                .to_string(),
        );
    }
    named.sort();
    if named.len() >= 2 {
        let (cheap, dear) = (&named[0], &named[named.len() - 1]);
        if cheap.0 == dear.0 {
            return Some(format!(
                "{} and {} are level on price, so pick on what you cook: {} {}, and {} {}. Go to whichever is nearer.",
                cheap.1, dear.1, cheap.1, cheap.4, dear.1, dear.4
            ));
        }
        let tail = if cheap.0 >= 2 {
            " Check the price per 100g on the shelf label rather than the pack size, because the price match only \
covers the basics."
        } else {
            ""
        };
        return Some(format!("{} is the cheaper of the two: {}. {} {}.{tail}", cheap.1, cheap.3, dear.1, dear.4));
    }
    let (rank, name, sentence, why, _contrast) = &named[0];
    let tail: &str = if *rank <= 1 {
        &_CHEAPEST_TAIL
    } else if *rank == 2 {
        &_MIDDLE_TAIL
    } else {
        &_DEAR_TAIL
    };
    Some(format!("{name} is {sentence}: {why}.{tail}"))
}

/// A person or taste word with a plural, possessive or contraction folded back to the listed word.
pub fn _fold(word: &str) -> String {
    if PERSON_WORDS.contains(word) || TASTE_WORDS.contains(word) {
        return word.to_string();
    }
    for (cut, join) in _FOLD.iter() {
        if let Some(stem) = word.strip_suffix(cut.as_str()) {
            let folded = format!("{stem}{join}");
            if PERSON_WORDS.contains(&folded) || TASTE_WORDS.contains(&folded) {
                return folded;
            }
        }
    }
    word.to_string()
}

/// The line's words, folded.
pub fn _the_words(line: &str) -> HashSet<String> {
    LETTERS.findall_str(&line.to_lowercase()).iter().map(|w| _fold(w)).collect()
}

/// The line as typed (text, else clean); "" without a reading.
pub fn _said_line(understood: Option<&Understood>) -> String {
    match understood {
        Some(u) => d2_shared::or(&u.text, &u.clean).to_string(),
        None => String::new(),
    }
}

/// True when the chat names a person and a taste of theirs, in this line or the last few.
pub fn _knows_a_taste(memory: &Memory, understood: Option<&Understood>) -> bool {
    let mut lines: Vec<String> = recent(memory, 8).iter().map(|t| t.user.clone()).collect();
    let said = _said_line(understood);
    if !said.is_empty() {
        lines.push(said);
    }
    lines.iter().any(|line| {
        let words = _the_words(line);
        words.iter().any(|w| PERSON_WORDS.contains(w)) && words.iter().any(|w| TASTE_WORDS.contains(w))
    })
}

/// A cheap present for the age the line or the chat gives, or for a child named in the line.
pub fn _gift_cheap(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let mut age = _an_age_in_the_line(understood);
    if age.is_none() {
        age = people::the_age_of_the_person_they_mean(memory, Some(understood)).map(|a| a as f64);
    }
    if let Some(age) = age {
        if age < 13.0 {
            return Some(CHEAP_CHILD.clone());
        }
        if age < 20.0 {
            return Some(CHEAP_TEEN.clone());
        }
        return Some(CHEAP_ADULT.clone());
    }
    let clean = &understood.clean;
    if CHILD_WORDS.iter().any(|r| r.is_match(clean)) {
        return Some(CHEAP_CHILD.clone());
    }
    Some(CHEAP_ADULT.clone())
}

fn interest_gifts(interest: &str) -> &'static [(String, f64)] {
    INTEREST_GIFTS.iter().find(|(k, _)| k == interest).map(|(_, v)| v.as_slice()).unwrap_or(&[])
}

/// The interests the chat has named, newest line first.
pub fn _the_tastes(memory: &Memory, understood: Option<&Understood>) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    let mut lines: Vec<String> = recent(memory, 4).iter().map(|t| t.user.clone()).collect();
    let here = _said_line(understood);
    if !here.is_empty() {
        lines.push(here);
    }
    for turn in lines.iter().rev() {
        for word in LETTERS.findall_str(&turn.to_lowercase()) {
            let name = _fold(&word);
            for candidate in [name.clone(), format!("{name}ing")] {
                if INTEREST_GIFTS.iter().any(|(k, _)| *k == candidate) && !said.contains(&candidate) {
                    said.push(candidate);
                }
            }
        }
        if !said.is_empty() {
            return said;
        }
    }
    said
}

/// Up to three presents for someone with these interests, cheapest first, or None.
pub fn present_known(interests: &[String], memory: Option<&Memory>, understood: Option<&Understood>) -> Option<String> {
    let wanted = _interest_phrase(interests);
    let mut picks: Vec<(String, f64)> = Vec::new();
    for interest in interests {
        for (item, price) in interest_gifts(&interest.to_lowercase()) {
            if !picks.iter().any(|(n, _)| n == item) {
                picks.push((item.clone(), *price));
            }
        }
    }
    if picks.is_empty() {
        return None;
    }
    picks.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let named: Vec<String> = picks.iter().take(3).map(|(item, price)| format!("{item} ({})", _money(*price))).collect();
    let listed = if named.len() == 1 {
        named[0].clone()
    } else {
        format!("{} or {}", named[..named.len() - 1].join(", "), named[named.len() - 1])
    };
    let sentence = format!(
        "For someone into {wanted}: {listed}. Buy the one {{they}} will actually use rather than three {{they}} might, \
and put a real message in the card."
    );
    if memory.is_none() && understood.is_none() {
        return Some(sentence);
    }
    let blank = Understood::default();
    Some(about_them(&sentence, understood.unwrap_or(&blank)))
}

/// The card line for a "what do i write in the card" after a present, or None.
pub fn the_card_after_a_present(
    understood: Option<&Understood>,
    memory: Option<&Memory>,
    line: Option<&str>,
) -> Option<String> {
    let said = match line {
        Some(l) if !text::strip(l).is_empty() => l.to_string(),
        _ => _said_line(understood),
    };
    if !A_CARD_IS_NOT_A_PRESENT.is_match(&said) || NOT_THAT_CARD.is_match(&said) {
        return None;
    }
    let blank = Memory::default();
    Some(pick_in(&CARD_AFTER_A_PRESENT, memory.unwrap_or(&blank), &said))
}

/// The present rows stand down for a line that does not ask what to give someone.
pub fn not_a_present(understood: &Understood) -> bool {
    !gift_ask(understood)
}

/// The condition a present has to respect, said in the text: (sentence, phrases to drop).
pub fn _the_condition_in(text: &str) -> Option<(String, Vec<String>)> {
    let text = format!(" {} ", text.to_lowercase());
    GIFT_FOR_A_CONDITION
        .iter()
        .find(|(phrases, _, _)| any_in(&text, phrases))
        .map(|(_, sentence, drop)| (sentence.clone(), drop.clone()))
}

/// The condition from this line, or from the two lines before when this one names nobody.
pub fn _the_condition_gift(understood: Option<&Understood>, memory: Option<&Memory>) -> (String, Vec<String>) {
    let line = _said_line(understood);
    let mut found = _the_condition_in(&line);
    if found.is_none() {
        let low = format!(" {} ", line.to_lowercase());
        if !_PRESENT_PEOPLE.iter().any(|w| low.contains(&format!(" {w} ")) || low.contains(&format!(" {w}s "))) {
            if let Some(memory) = memory {
                let turns = &memory.turns;
                let earlier: Vec<&str> = turns[turns.len().saturating_sub(2)..].iter().map(|t| t.user.as_str()).collect();
                found = _the_condition_in(&earlier.join(" "));
            }
        }
    }
    found.unwrap_or_default()
}

/// `_present_answer(text)`'s reply: the text with the condition the chat named.
fn _present_answer_reply(f: &FnRef, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let text = f.cell("text").map(|v| v.str().to_string()).unwrap_or_default();
    Some(_with_the_condition(&text, Some(understood), Some(memory)))
}

/// What a "is X a good present" line asks about, or "".
pub fn _the_thing_a_verdict_is_about(line: &str) -> String {
    static LEAD: LazyLock<Re> = LazyLock::new(|| Re::new("^(?:(?:that|it|this)\\s+)+", pyre::I));
    static THREE: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z]{3}", pyre::I));
    let Some(found) = A_VERDICT_THING.search(line) else { return String::new() };
    let thing = text::squash(found.get(1).expect("group 1 is None"));
    let thing = LEAD.subn("", &thing, 1);
    let low = thing.to_lowercase();
    if [
        "a", "an", "the", "my", "some", "a gift", "a present", "that", "it", "this", "them", "they",
    ]
    .contains(&low.as_str())
    {
        return String::new();
    }
    if thing.is_empty() || !THREE.is_match(&thing) {
        return String::new();
    }
    thing
}

/// The one age the store holds, when it holds exactly one.
pub fn _the_only_age_in_the_store(memory: &Memory) -> Option<i64> {
    let mut ages: Vec<Option<i64>> = Vec::new();
    if let Some(others) = memory.extra.get("others") {
        for entry in others.items() {
            if matches!(entry, Value::Dict(_)) && _with_an_age(entry) {
                ages.push(_age(entry));
            }
        }
    }
    if let Some(held) = memory.extra.get("their_age") {
        if matches!(held, Value::Dict(_)) && _with_an_age(held) {
            ages.push(_age(held));
        }
    }
    if ages.len() == 1 { ages[0] } else { None }
}

/// True when the thing is one of the picks in the pool for that age.
pub fn _a_pick_for_that_age(pool: Option<&str>, thing: &str) -> bool {
    static ARTICLE: LazyLock<Re> = LazyLock::new(|| Re::new("^(?:a|an|the|my|some)\\s+", pyre::I));
    let Some(pool) = pool.filter(|p| !p.is_empty()) else { return false };
    if thing.is_empty() {
        return false;
    }
    let thing = ARTICLE.subn("", text::strip(thing), 1);
    let words: Vec<String> = LOWER_WORDS
        .findall_str(&thing.to_lowercase())
        .into_iter()
        .filter(|w| w.chars().count() > 2 && !NOT_THE_THING.contains(w))
        .collect();
    if words.is_empty() {
        return false;
    }
    let low = pool.to_lowercase();
    words.iter().any(|w| cached_re(&format!("\\b{}s?\\b", pyre::escape(w)), 0).is_match(&low))
}

/// "is a scooter a good present for a 6 year old": yes, when the thing is a pick for that age.
pub fn the_verdict_on_a_present(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = _said_line(Some(understood));
    if !A_VERDICT_ON_A_PRESENT.is_match(&line) {
        return None;
    }
    let thing = _the_thing_a_verdict_is_about(&line);
    let truthy = |a: Option<f64>| a.filter(|v| *v != 0.0);
    let held = truthy(_an_age_in_the_line(understood))
        .or_else(|| truthy(people::the_age_of_the_person_they_mean(memory, Some(understood)).map(|a| a as f64)))
        .or_else(|| truthy(_the_only_age_in_the_store(memory).map(|a| a as f64)))?;
    let pool = _gift_for_an_age(Some(held));
    if thing.is_empty() {
        return None;
    }
    if !_a_pick_for_that_age(pool.as_deref(), &thing) {
        return None;
    }
    let age = if held < 1.0 {
        "a baby".to_string()
    } else if held == 1.0 {
        "one".to_string()
    } else {
        age_text(held)
    };
    let pool_of_words: &[String] =
        if A_VERDICT_ABOUT_MONEY.is_match(&line) { &MONEY_VERDICT_YES } else { &VERDICT_YES };
    Some(py_format(&pick_in(pool_of_words, memory, &line), &[("age", &age), ("thing", &thing)]))
}

/// `_gift_row(fallback)`'s reply: a present for whoever the chat is about.
pub fn _gift_row_reply(fallback: &str, memory: &mut Memory, understood: &Understood) -> Option<String> {
    if let Some(verdict) = the_verdict_on_a_present(memory, understood).filter(|v| !v.is_empty()) {
        return Some(verdict);
    }
    let said = _said_line(Some(understood));
    if A_VERDICT_ON_A_PRESENT.is_match(&said) {
        return None;
    }
    if !gift_ask(understood) {
        if A_CARD_IS_NOT_A_PRESENT.is_match(&said) {
            return None;
        }
        return Some(fallback.to_string());
    }
    if PRICE_SAID.is_match(&said) {
        return None;
    }
    if A_CHEAP_ASK.is_match(&said) {
        let cheap = _gift_cheap(memory, understood);
        return match cheap {
            Some(c) if !c.is_empty() => Some(_with_the_condition(&c, Some(understood), Some(memory))),
            _ => None,
        };
    }
    if !_knows_a_taste(memory, Some(understood)) {
        if let Some(baby) = _the_newborn_present(memory, understood) {
            return Some(_with_the_condition(&baby, Some(understood), Some(memory)));
        }
        let line_age = _an_age_in_the_line(understood).filter(|a| *a != 0.0);
        let age = match line_age {
            Some(a) => Some(a),
            None => people::the_age_of_the_person_they_mean(memory, Some(understood)).map(|a| a as f64),
        };
        if let Some(pool) = _gift_for_an_age(age) {
            return Some(_with_the_condition(&pool, Some(understood), Some(memory)));
        }
        if _only_a_them_says_who(understood) {
            return None;
        }
        return Some(_with_the_condition(fallback, Some(understood), Some(memory)));
    }
    if PRICE_SAID.is_match(&said) {
        return None;
    }
    let tastes = _the_tastes(memory, Some(understood));
    let picks = present_known(&tastes, Some(memory), Some(understood)).filter(|p| !p.is_empty());
    let chosen = picks.unwrap_or_else(|| fallback.to_string());
    Some(_with_the_condition(&chosen, Some(understood), Some(memory)))
}

fn _gift_row_entry(f: &FnRef, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let fallback = f.cell("fallback").map(|v| v.str().to_string()).unwrap_or_default();
    _gift_row_reply(&fallback, memory, understood)
}

/// True when only a plural pronoun says who the present is for.
pub fn _only_a_them_says_who(understood: &Understood) -> bool {
    let said = _said_line(Some(understood));
    if !PLURAL_ONLY.is_match(&said) {
        return false;
    }
    !_the_words(&said).iter().any(|w| PERSON_WORDS.contains(w) && !PLURAL_WORDS.contains(w))
}

/// The baby present when the chat is about a new baby.
pub fn _the_newborn_present(memory: &Memory, understood: &Understood) -> Option<String> {
    if !the_thread_has_a_new_baby(memory, Some(understood)) {
        return None;
    }
    _gift_for_an_age(Some(0.5))
}

/// The answer with the condition the chat named (no alcohol, no sugar...) added on.
pub fn _with_the_condition(text: &str, understood: Option<&Understood>, memory: Option<&Memory>) -> String {
    if text.is_empty() {
        return String::new();
    }
    let (note, drop) = _the_condition_gift(understood, memory);
    if note.is_empty() || text.to_lowercase().contains(&note.to_lowercase()) {
        return text.to_string();
    }
    let mut answer = text.to_string();
    for phrase in &drop {
        answer = answer.replace(phrase.as_str(), "");
    }
    format!("{} {}", answer.trim_end_matches(text::is_space), note)
}

/// A present for anyone: by their tastes, their age, a new baby, or the general advice.
pub fn _gift_anyone(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !gift_ask(understood) {
        return None;
    }
    if _knows_a_taste(memory, Some(understood)) {
        let tastes = _the_tastes(memory, Some(understood));
        return present_known(&tastes, Some(memory), Some(understood)).filter(|p| !p.is_empty());
    }
    let held = people::the_age_of_the_person_they_mean(memory, Some(understood));
    let pool = _gift_for_an_age(held.map(|a| a as f64)).or_else(|| _the_newborn_present(memory, understood));
    if let Some(pool) = pool {
        return Some(_with_the_condition(&pool, Some(understood), Some(memory)));
    }
    if people::a_child_the_chat_is_about(memory, Some(understood)) {
        return Some(_with_the_condition(&CHILD_GIFT, Some(understood), Some(memory)));
    }
    Some(_with_the_condition(&_GIFT_ANYONE, Some(understood), Some(memory)))
}

/// What matters when buying a laptop.
pub fn _laptop_general(_memory: &mut Memory, _understood: &Understood) -> Option<String> {
    Some(
        "Get 16GB of RAM and a 512GB SSD if the budget stretches, 8GB and 256GB if it doesn't, and a 13 or 14 inch \
screen for carrying about. Don't pay for a graphics card unless you game or edit video; a good screen and keyboard \
matter far more day to day."
            .to_string(),
    )
}

/// True when the bot's last reply was a present answer.
pub fn a_present_was_answered(memory: &Memory) -> bool {
    let said = text::squash(&last_bot(memory));
    if said.is_empty() {
        return false;
    }
    PRESENT_OPENINGS.iter().any(|o| said.contains(o.as_str()))
}

/// A smart meter: a fault, how to read it, or "" for anything else.
pub fn _smart_meter_how(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = d2_shared::or(&understood.text, &understood.clean).to_lowercase();
    if any_in(&said, &_METER_FAULT) || said.contains("not working") {
        return Some(
            "Ring your supplier and say the meter or its display has stopped - it is their equipment, so fixing it \
is free, and it is usually a firmware update or a new display they send out. Photograph the reading on the meter \
itself before you ring, and send them a reading weekly until it is sorted, so you are not billed on an estimate in the \
meantime."
                .to_string(),
        );
    }
    if understood.words.iter().any(|w| _METER_READ.contains(w)) {
        return Some(
            "Press the button marked A (some meters say B) until the screen shows a number with kWh after it - that \
is your reading, and the one the supplier wants. On a smart meter the in-home display shows it too, and with a \
single-rate tariff it is the only number that matters; a two-rate one shows day and night separately, so send both."
                .to_string(),
        );
    }
    Some(String::new())
}

/// True when the line is only the thing's name, the decision words and the ask.
pub fn _bare_decision(said: &str, name: &str, ask: &[String]) -> bool {
    let mut rest = said.to_string();
    for phrase in std::iter::once(name).chain(ask.iter().map(String::as_str)) {
        let phrase = text::strip(phrase);
        if !phrase.is_empty() {
            rest = cached_re(&format!("\\b{}\\b", pyre::escape(phrase)), 0).sub(" ", &rest);
        }
    }
    DECISION_TOKENS.findall_str(&rest).iter().all(|w| _DECISION_WORDS.contains(w))
}

/// The reply of the row whose longest name the line says, and that name.
pub fn _longest_match_named(said: &str, table: &[(Vec<String>, String)]) -> (Option<String>, String) {
    let (mut best, mut longest, mut found): (Option<&String>, isize, &str) = (None, -1, "");
    for (names, reply) in table {
        for name in names {
            let name = text::strip(name);
            let n = name.chars().count() as isize;
            if n > longest && cached_re(&format!("\\b{}\\b", pyre::escape(name)), 0).is_match(said) {
                best = Some(reply);
                longest = n;
                found = name;
            }
        }
    }
    (best.cloned(), found.to_string())
}

/// The reply of the row whose longest name the line says.
pub fn _longest_match(said: &str, table: &[(Vec<String>, String)]) -> Option<String> {
    _longest_match_named(said, table).0
}

/// Haggling: whether it is rude, getting a trader's quote down, or how to ask.
pub fn _haggle(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    static AWKWARD: LazyLock<Re> = LazyLock::new(|| Re::new("\\brude\\b|\\bawkward\\b|\\bembarrass", 0));
    let said = said_all(understood);
    if AWKWARD.is_match(&said) {
        return Some(
            "No - asking once is normal in a market, a car boot sale, a small independent shop or on anything big like \
a sofa, and in a chain with price tags it is still fine to ask. One question, then take the answer and either buy or \
leave: asking twice or arguing after a no is the part that gets awkward."
                .to_string(),
        );
    }
    if any_in(&said, &_TRADER_WORDS) {
        return Some(
            "Get three written quotes first, then ask each trader the same question: what would bring that down? \
Supplying the materials, being flexible on dates so the job fills a gap in their diary, and paying a deposit up front \
are the three that work. Compare the quotes line by line rather than on the total, and treat one miles under the \
others as a warning: the extras come later."
                .to_string(),
        );
    }
    Some(
        "Ask once, cheerfully, once you have picked the thing - \"is there anything you can do on the price?\" - then \
stop talking and be ready to walk away: silence and a customer who will leave are what move a price. Take the display \
model or last season's colour, offer to pay in full today, check the online price so they can match it, and never say \
your maximum. At a market or car boot sale, offer about two thirds and settle in the middle."
            .to_string(),
    )
}

/// Whether a choice (a gym, a heat pump...) is worth it, by the longest name the line says.
pub fn _choice_value(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said_all(understood);
    let (row, name) = _longest_match_named(&said, &CHOICE_VALUE);
    if row.is_some() && _ACTION_NAME.is_match(&name) && !_bare_decision(&said, &name, &APPLIANCE_ASK) {
        return None;
    }
    if row.is_some() && any_in(&said, &_DRYER_WORDS) {
        for (names, answer) in CHOICE_VALUE.iter() {
            if names.iter().any(|n| n.contains("dryer") || n.contains("drier")) {
                return Some(answer.clone());
            }
        }
    }
    row
}

/// Whether an appliance is worth buying, by the longest name the line says.
pub fn _appliance_value(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    _longest_match(&said_all(understood), &APPLIANCE_VALUE)
}

/// Where to buy an everyday thing, unless the line is about finding one at home.
pub fn _everyday_where(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said_all(understood);
    if said.is_empty() {
        return None;
    }
    if let Some(inside) = _INSIDE.search(&said) {
        match inside.get(1) {
            Some(place) if _SHOP_PLACES.contains(place) => {}
            _ => return None,
        }
    }
    if text::split_ref(&said).contains(&"find") && _MINE.is_match(&said) {
        return None;
    }
    _longest_match(&said, &EVERYDAY_WHERE)
}

/// The present card for an age, or None without one.
pub fn _gift_for_an_age(age: Option<f64>) -> Option<String> {
    let age = age?;
    if age < 1.0 {
        return Some(
            "Under a year, buy the boring things the parents will use: clothes in the size for three to six months \
ahead, a pack of muslins, a board book, or a soft ball. Skip anything that needs batteries or a cot to hang it from, \
and skip newborn sizes, because everyone buys those and the baby grows out of them in a fortnight."
                .to_string(),
        );
    }
    if age >= 20.0 {
        return Some(GROWN_GIFT.clone());
    }
    if age >= 13.0 {
        return Some(TEEN_GIFT.clone());
    }
    Some(if age <= 2.0 { TODDLER_GIFT.clone() } else { CHILD_GIFT.clone() })
}

/// The age the line gives as a number in months or years, or a "turning 7" that the reading
/// also read as a number.
pub fn _an_age_in_the_line(understood: &Understood) -> Option<f64> {
    for number in &understood.numbers {
        match number.unit.as_deref() {
            Some("months") => return Some(number.value.as_ref().expect("None / 12.0").as_f64() / 12.0),
            Some("years") => return number.value.as_ref().map(|v| v.as_f64()),
            _ => {}
        }
    }
    let line = text::squash(&understood.clean.to_lowercase());
    if let Some(turning) = TURNING.search(&line) {
        let age: i64 = text::strip(turning.name("age").expect("no age group")).parse().expect("int()");
        if (0..=120).contains(&age)
            && understood.numbers.iter().any(|n| n.value.as_ref().is_some_and(|v| v.as_f64() == age as f64))
        {
            return Some(age as f64);
        }
    }
    None
}

/// True when the line says the present is for a child or a teenager.
pub fn _a_child_is_the_one_they_buy_for(understood: &Understood) -> bool {
    let words = _the_words(&_said_line(Some(understood)));
    if CHILD_PEOPLE.iter().any(|p| words.contains(p)) {
        return true;
    }
    ["teenager", "teenagers", "teen", "teens", "teenage", "schoolkid", "schoolkids", "schoolboy", "schoolgirl"]
        .iter()
        .any(|w| words.contains(*w))
}

/// `_the_person_row(adult, child)`'s reply: the child's row or the grown-up's.
fn _the_person_row_reply(f: &FnRef, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let key = if _a_child_is_the_one_they_buy_for(understood) { "young" } else { "grown_up" };
    let chosen = f.cell(key).and_then(Value::as_fn).expect("closure cell");
    let fallback = chosen.cell("fallback").map(|v| v.str().to_string()).unwrap_or_default();
    _gift_row_reply(&fallback, memory, understood)
}

/// An age said as digits or as a word ("three").
pub fn _an_age_given(said: &str) -> Option<i64> {
    if text::is_digit(said) {
        return Some(said.parse().expect("int()"));
    }
    _SAID_AS_WORDS.iter().find(|(k, _)| k == said).map(|(_, v)| *v)
}

/// The child's age the line says ("a 6 year old", "18 months", "her 7th"), in years.
pub fn _an_age_said_in_the_line(line: &str) -> Option<f64> {
    let low = line.to_lowercase();
    let found = _AN_AGE_SAID.search(&low)?;
    if let Some(months) = found.name("months").filter(|m| !m.is_empty()) {
        let months = _an_age_given(months);
        return match months {
            Some(m) if m != 0 => Some(m as f64 / 12.0),
            _ => None,
        };
    }
    for group in ["years", "near", "ordinal"] {
        if let Some(said) = found.name(group).filter(|m| !m.is_empty()) {
            return _an_age_given(said).map(|a| a as f64);
        }
    }
    None
}

/// A row shape: the line names a child's age.
pub fn _names_a_childs_age(line: &str) -> bool {
    _A_CHILDS_AGE.is_match(&line.to_lowercase())
}

/// Guard: the age row answers a line about a child of an age, and a child named without one.
pub fn _this_row_answers_the_line(line: &str) -> bool {
    let text = line.to_lowercase();
    if text.is_empty() {
        return false;
    }
    let named = _a_child_is_named(&text);
    if !_A_CHILDS_AGE.is_match(&text) {
        return !named;
    }
    let age = _an_age_said_in_the_line(&text);
    if named && age.is_some_and(|a| a > 19.0) {
        return false;
    }
    true
}

/// True when the text names a child ("my son", "the kids").
pub fn _a_child_is_named(text: &str) -> bool {
    let words = LETTERS.findall_str(&text.to_lowercase());
    for person in CHILD_PEOPLE.iter() {
        if person.contains(' ') {
            if text.contains(person.as_str()) {
                return true;
            }
        } else if words.contains(person) {
            return true;
        }
    }
    false
}

/// The present card for the age the line gives, or by the kind of child it names.
pub fn _gift_by_age(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    static TODDLER: LazyLock<Re> =
        LazyLock::new(|| Re::new("\\b(?:toddler|toddlers|preschool|preschooler|baby|babies)\\b", 0));
    static TEEN: LazyLock<Re> = LazyLock::new(|| Re::new("\\b(?:teen|teens|teenager|teenagers|teenage)\\b", 0));
    let low = d2_shared::or(&understood.clean, &understood.text).to_lowercase();
    let age = _an_age_in_the_line(understood).filter(|a| *a != 0.0).or_else(|| _an_age_said_in_the_line(&low));
    if let Some(chosen) = _gift_for_an_age(age) {
        return Some(chosen);
    }
    if TODDLER.is_match(&low) {
        return Some(TODDLER_GIFT.clone());
    }
    if TEEN.is_match(&low) {
        return Some(TEEN_GIFT.clone());
    }
    Some(CHILD_GIFT.clone())
}

/// The age row: the card line after a present, else a present for the age.
pub fn _the_age_row(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if let Some(card) = the_card_after_a_present(Some(understood), Some(memory), None).filter(|c| !c.is_empty()) {
        return Some(card);
    }
    _gift_by_age(memory, understood)
}

/// A watch to buy: a smart watch, or a plain one; None for a watch being done something to.
pub fn _watch_help(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let forms: Vec<String> = ["clean", "norm_text", "norm"].iter().map(|k| py_field(understood, k).to_lowercase()).collect();
    let said = forms.join(" ");
    if any_in(&said, &_WATCH_SMART) {
        return Some(_WATCH_SMART_SAY.clone());
    }
    if any_in(&said, &_WATCH_VERBS) {
        return None;
    }
    if forms.iter().any(|f| _WATCH_VERB_SHAPE.is_match(f)) {
        return None;
    }
    Some(_WATCH_PLAIN.clone())
}

/// A whole-pound price, "£12".
pub fn _money(amount: f64) -> String {
    format!("\u{a3}{}", text::round_half_even(amount))
}

/// "gardening", "gardening and cooking", "gardening, cooking and fishing".
pub fn _interest_phrase<S: AsRef<str>>(interests: &[S]) -> String {
    let named: Vec<&str> = interests.iter().map(|w| w.as_ref()).filter(|w| !w.is_empty()).collect();
    match named.len() {
        0 => String::new(),
        1 => named[0].to_string(),
        n => format!("{} and {}", named[..n - 1].join(", "), named[n - 1]),
    }
}

/// A present within a budget, for someone with these interests.
pub fn present_within<S: AsRef<str>>(money: f64, interests: &[S]) -> String {
    let wanted = _interest_phrase(interests);
    let mut fits: Vec<(String, f64)> = Vec::new();
    for interest in interests {
        for (item, price) in interest_gifts(&interest.as_ref().to_lowercase()) {
            if *price <= money && !fits.iter().any(|(i, p)| i == item && p == price) {
                fits.push((item.clone(), *price));
            }
        }
    }
    fits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    if !fits.is_empty() {
        let listed: Vec<String> = fits.iter().take(2).map(|(item, price)| format!("{item} ({})", _money(*price))).collect();
        let listed = listed.join(", ");
        if money >= 40.0 && fits[0].1 < money / 2.0 {
            return format!(
                "{} is plenty for someone into {wanted}: {listed}, and the rest towards something they have been \
putting off.",
                _money(money)
            );
        }
        return format!(
            "{} or less, for someone into {wanted}: {listed}. Buy the one thing they will actually use rather than \
three they might, and put a real message in the card.",
            _money(money)
        );
    }
    if money == 0.0 {
        return py_format(&PRESENT_OVER, &[("money", "that")]);
    }
    for (band, advice) in PRESENT_BANDS.iter() {
        if money <= *band {
            return _with_buy_into(&py_format(advice, &[("money", &_money(money))]), !interests.is_empty());
        }
    }
    _with_buy_into(&py_format(&PRESENT_OVER, &[("money", &_money(money))]), !interests.is_empty())
}

/// The budget advice, with "buy into what they do" when no interest is known.
pub fn _with_buy_into(advice: &str, has_interests: bool) -> String {
    if has_interests { advice.to_string() } else { format!("{advice}{}", *BUY_INTO) }
}

/// Something cheaper than the last idea, for someone with these interests.
pub fn present_less<S: AsRef<str>>(interests: &[S]) -> String {
    let wanted = _interest_phrase(interests);
    let mut fits: Vec<(String, f64)> = Vec::new();
    for interest in interests {
        for (item, price) in interest_gifts(&interest.as_ref().to_lowercase()) {
            if !fits.iter().any(|(n, _)| n == item) {
                fits.push((item.clone(), *price));
            }
        }
    }
    if !fits.is_empty() {
        fits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let (item, price) = &fits[0];
        return format!(
            "Cheaper than that: {item} ({}) is the small version of the same idea for someone into {wanted}, and a \
good one beats something bigger and cheaper.",
            _money(*price)
        );
    }
    "Cheaper than that: go for the small version of the same idea - a good bar of chocolate, a decent paperback, \
nice soap, or a packet of good coffee - and let the card do the rest."
        .to_string()
}

/// A coat to buy: by the height given, the kind named, or the weather; with the check list.
pub fn _coat_to_buy(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let said = said_all(understood);
    if let Some(tall) = _HEIGHT.search(&said) {
        let (feet, inches) = (tall.str(1), tall.get(2).expect("int(None)"));
        let total = feet.parse::<i64>().expect("int()") * 12 + inches.parse::<i64>().expect("int()");
        let (low, high) = ((total as f64 * 0.42) as i64, (total as f64 * 0.50) as i64);
        return Some(format!(
            "At {feet} foot {inches}, look for a coat about {low} to {high} inches from the shoulder - mid-thigh on \
you - so it keeps your legs warm without swamping you. Check the sleeve ends at your wrist bone with your arms down, \
and try it on over the thickest jumper you own."
        ));
    }
    for (words, reply) in _COAT_KINDS.iter() {
        if words.iter().any(|r| r.is_match(&said)) {
            return Some(reply.clone());
        }
    }
    let generic = "Pick the coat by the weather you are actually out in: a waterproof shell for rain, down or padded \
for cold and dry, wool for town. \u{a3}80 to \u{a3}150 buys a padded coat that lasts a decade, \u{a3}150 to \u{a3}250 \
a waterproof shell that does not leak at the shoulders.";
    if _mentions_coat_check(&said) {
        return Some(if _coat_said_already(memory) {
            _COAT_CHECK.clone()
        } else {
            format!("{generic} {}", *_COAT_CHECK)
        });
    }
    Some(generic.to_string())
}

/// True when the bot's last reply was the general coat answer.
pub fn _coat_said_already(memory: &Memory) -> bool {
    let said = last_bot(memory).to_lowercase();
    said.contains("buys a padded coat")
        || said.contains("pick the coat by the weather")
        || said.contains("waterproof shell with taped seams")
}

/// Guard: no pronoun in the line says who it is for.
pub fn _nobody_named_by_pronoun(understood: &Understood) -> bool {
    let words = if !understood.words.is_empty() { &understood.words } else { &understood.norm };
    !words.iter().any(|w| _NAMED_BY_PRONOUN.contains(&w.to_lowercase()))
}

/// True when the line asks what to check on a coat.
pub fn _mentions_coat_check(said: &str) -> bool {
    _COAT_CHECK_WORDS.iter().any(|r| r.is_match(said))
}

/// Guard: "how long should i keep the receipt" is not a length of anything.
pub fn _not_a_length(understood: &Understood) -> bool {
    let said = format!("{} {}", understood.text, understood.clean).to_lowercase();
    !_KEEP_RECEIPT_RE.is_match(&said)
}

/// Guard: a bag a food is kept in is not the carrier bag charge.
pub fn _the_bag_is_not_packaging(understood: &Understood) -> bool {
    !d2_shared::a_food_is_being_kept(understood)
}

/// Guard: a line about visiting a place (Iceland) is not about the shop.
pub fn _the_place_not_the_shop(understood: &Understood) -> bool {
    let said = format!(" {} ", d2_shared::or(&understood.clean, &understood.text).to_lowercase());
    const GOING: &[&str] = &[
        " pack", " packing", " trip", " travel", " travelling", " traveling", " flight", " fly", " holiday",
        " holidays", " visit", " visiting", " hotel", " weather", " things to do", " airport", " tourist",
        " sightseeing", " northern lights", " what to wear", " what to take", " what to see", " in october",
        " in november", " in december", " in january", " in february", " in march",
    ];
    const PLACE: &[&str] = &[
        " with kids", " with children", " with a family", " with the family", " with a baby", " how many days",
        " how long", " best time to", " when to go", " worth visiting", " things to do", " it worth going",
        " reykjavik", " blue lagoon", " golden circle", " whale watching", " northern lights", " hire car",
        " car hire", " road trip", " visa", " what to pack", " what to wear", " is it safe", " safe to",
    ];
    const PRICE: &[&str] = &[
        " cheap", " cheaper", " price", " prices", " quality", " fresh", " value", " compare", " comparison",
        " expensive", " basket", " groceries", " supermarket", " shop",
    ];
    if any_in(&said, PLACE) {
        return false;
    }
    if any_in(&said, GOING) && !any_in(&said, PRICE) {
        return false;
    }
    true
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.shopping:<lambda>:2623", |_: &FnRef, u: &Understood| {
        !asks_where(u) || asks_where_a_shop_answers(u)
    }),
    ("bot.knowledge.shopping:<lambda>:3742", |_: &FnRef, u: &Understood| !asks_where_a_shop_answers(u)),
    ("bot.knowledge.shopping:_nobody_named_by_pronoun:2041", |_: &FnRef, u: &Understood| _nobody_named_by_pronoun(u)),
    ("bot.knowledge.shopping:_not_a_length:2062", |_: &FnRef, u: &Understood| _not_a_length(u)),
    ("bot.knowledge.shopping:_the_bag_is_not_packaging:2071", |_: &FnRef, u: &Understood| _the_bag_is_not_packaging(u)),
    ("bot.knowledge.shopping:_the_place_not_the_shop:2080", |_: &FnRef, u: &Understood| _the_place_not_the_shop(u)),
    ("bot.knowledge.shopping:_this_row_answers_the_line:1666", |_: &FnRef, u: &Understood| {
        _this_row_answers_the_line(d2_shared::or(&u.clean, &u.text))
    }),
    ("bot.knowledge.shopping:not_a_present:544", |_: &FnRef, u: &Understood| not_a_present(u)),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.shopping:_appliance_value:1393", |_: &FnRef, m: &mut Memory, u: &Understood| _appliance_value(m, u)),
    ("bot.knowledge.shopping:_choice_value:1372", |_: &FnRef, m: &mut Memory, u: &Understood| _choice_value(m, u)),
    ("bot.knowledge.shopping:_coat_to_buy:1990", |_: &FnRef, m: &mut Memory, u: &Understood| _coat_to_buy(m, u)),
    ("bot.knowledge.shopping:_everyday_where:1459", |_: &FnRef, m: &mut Memory, u: &Understood| _everyday_where(m, u)),
    ("bot.knowledge.shopping:_gift_anyone:803", |_: &FnRef, m: &mut Memory, u: &Understood| _gift_anyone(m, u)),
    ("bot.knowledge.shopping:_gift_cheap:348", |_: &FnRef, m: &mut Memory, u: &Understood| _gift_cheap(m, u)),
    ("bot.knowledge.shopping:_gift_row.<locals>.reply:708", _gift_row_entry),
    ("bot.knowledge.shopping:_haggle:1340", |_: &FnRef, m: &mut Memory, u: &Understood| _haggle(m, u)),
    ("bot.knowledge.shopping:_laptop_general:837", |_: &FnRef, m: &mut Memory, u: &Understood| _laptop_general(m, u)),
    ("bot.knowledge.shopping:_present_answer.<locals>.reply:604", _present_answer_reply),
    ("bot.knowledge.shopping:_smart_meter_how:1257", |_: &FnRef, m: &mut Memory, u: &Understood| _smart_meter_how(m, u)),
    ("bot.knowledge.shopping:_supermarket_cheapest:224", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _supermarket_cheapest(m, u)
    }),
    ("bot.knowledge.shopping:_the_age_row:1734", |_: &FnRef, m: &mut Memory, u: &Understood| _the_age_row(m, u)),
    ("bot.knowledge.shopping:_the_person_row.<locals>.reply:1571", _the_person_row_reply),
    ("bot.knowledge.shopping:_watch_help:1776", |_: &FnRef, m: &mut Memory, u: &Understood| _watch_help(m, u)),
];
