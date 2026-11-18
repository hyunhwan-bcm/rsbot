//! Port of `bot/memory/spent.py`: what they told the bot they spent ("i spent 300 on food",
//! "40 on a takeaway") and had ("i've got 200 till payday"), read from the lines of the chat,
//! and the asks about it ("how much have i spent", "how much have i got left", "can i afford a
//! takeaway"). Nothing here writes to the store.

use crate::data;
use crate::py::re::{self, Match, Re};
use crate::py::text;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.memory.spent.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static SPENT: LazyLock<&'static Re> = LazyLock::new(|| rx("SPENT"));
static COST: LazyLock<&'static Re> = LazyLock::new(|| rx("COST"));
static BARE_ON: LazyLock<&'static Re> = LazyLock::new(|| rx("BARE_ON"));
static _WHEN_TAIL: LazyLock<&'static Re> = LazyLock::new(|| rx("_WHEN_TAIL"));
static _TAIL_OFF: LazyLock<&'static Re> = LazyLock::new(|| rx("_TAIL_OFF"));
static _TRAILING: LazyLock<&'static Re> = LazyLock::new(|| rx("_TRAILING"));
static ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("ASK"));
static ON_A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("ON_A_THING"));
static _ASK_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("_ASK_FILLER"));
static CATEGORIES: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    data::get("bot.memory.spent.CATEGORIES").items().iter().map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].strs())).collect()
});
static _TWO_SPENDS: LazyLock<&'static Re> = LazyLock::new(|| rx("_TWO_SPENDS"));
static _SAID_AS_A_COST: LazyLock<&'static Re> = LazyLock::new(|| rx("_SAID_AS_A_COST"));
static _PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("_PEOPLE"));
static _TWO_FIGURES: LazyLock<&'static Re> = LazyLock::new(|| rx("_TWO_FIGURES"));
static _PAY_FOR: LazyLock<&'static Re> = LazyLock::new(|| rx("_PAY_FOR"));
static _PAY_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_PAY_THING"));
static _PLAIN: LazyLock<&'static Re> = LazyLock::new(|| rx("_PLAIN"));
static _PLAIN_BACK: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.spent._PLAIN_BACK"));
static PRESENT_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("PRESENT_ASK"));
static A_POSSESSIVE_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("A_POSSESSIVE_ASK"));
static HELD: LazyLock<&'static Re> = LazyLock::new(|| rx("HELD"));
static BUDGET_HELD: LazyLock<&'static Re> = LazyLock::new(|| rx("BUDGET_HELD"));
static SAVINGS_HELD: LazyLock<&'static Re> = LazyLock::new(|| rx("SAVINGS_HELD"));
static ON_A_BUDGET: LazyLock<&'static Re> = LazyLock::new(|| rx("ON_A_BUDGET"));
static THERE_LEFT: LazyLock<&'static Re> = LazyLock::new(|| rx("THERE_LEFT"));
static _A_COUNTED_NOUN: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_COUNTED_NOUN"));
static _A_CURRENCY_PLURAL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.spent._A_CURRENCY_PLURAL"));
static _A_SCOPE_OF_A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_SCOPE_OF_A_THING"));
static _A_MONEY_PLURAL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.spent._A_MONEY_PLURAL"));
static _A_PLURAL: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_PLURAL"));
static _HELD_SCOPE: LazyLock<&'static Re> = LazyLock::new(|| rx("_HELD_SCOPE"));
static LEFT: LazyLock<&'static Re> = LazyLock::new(|| rx("LEFT"));
static AFFORD: LazyLock<&'static Re> = LazyLock::new(|| rx("AFFORD"));
static A_PRICE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PRICE"));
static _ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("_ASK_WORDS"));
static A_WHAT_IF: LazyLock<&'static Re> = LazyLock::new(|| rx("A_WHAT_IF"));
static A_LENGTH_NOT_MONEY: LazyLock<&'static Re> = LazyLock::new(|| rx("A_LENGTH_NOT_MONEY"));

static CLAUSE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;.]", re::U));
static A_PLACE_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:at|in|on|for)\s+", re::I | re::U));
static SENTENCE_STOP: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,.!?;]", re::U));
static A_POINTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:it|that|one|anything)\b", re::U));
static A_DETERMINER: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:a|an|the|some|my|our|any)\s+", re::I | re::U));
static NAMED_WITH_A_DETERMINER: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?:a|an|the|some|my|our|any)\s", re::I | re::U));

/// One spend a line reports: the figure as typed, the thing it went on, and the line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spend {
    pub amount: String,
    pub thing: String,
    pub line: String,
}

/// The money they last said they had, and the line that said it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    pub amount: String,
    pub line: String,
}

/// `float(x or 0)` of a figure as typed (Python raises on anything else).
fn money(amount: &str) -> f64 {
    if amount.is_empty() {
        return 0.0;
    }
    text::strip(amount).parse().unwrap_or_else(|_| panic!("ValueError: could not convert string to float: {amount:?}"))
}

/// Python: bot/memory/spent.py:71 `_amount(number)`
pub fn _amount(number: &str) -> String {
    text::strip(number).to_string()
}

/// Python: bot/memory/spent.py:76 `_the_thing(rest)`
/// The thing the money went on, in the words the person used, or "".
pub fn _the_thing(rest: &str) -> String {
    let rest = text::strip(rest);
    if rest.is_empty() {
        return String::new();
    }
    let rest = CLAUSE_END.split(rest).swap_remove(0);
    let low = rest.to_lowercase();
    let rest = if low.starts_with("at ") || low.starts_with("in ") { A_PLACE_WORD.sub("", &rest) } else { rest };
    let rest = _WHEN_TAIL.sub("", &rest);
    let rest = _TAIL_OFF.sub("", &rest);
    let rest = text::strip(&_TRAILING.sub("", &rest)).to_string();
    if text::len(&rest) > 1 { rest } else { String::new() }
}

/// Python: bot/memory/spent.py:121 `_a_bill(text)`
/// The money a line says a thing of theirs costs, or None.
pub fn _a_bill(text_: &str) -> Option<Spend> {
    let shapes: [(&Re, usize, usize, usize, bool); 3] =
        [(*_SAID_AS_A_COST, 2, 1, 3, true), (*_PAY_FOR, 1, 3, 2, false), (*_PAY_THING, 1, 3, 2, false)];
    for (pattern, amount_group, thing_group, currency_group, said_as_a_cost) in shapes {
        let Some(found) = pattern.search(text_) else { continue };
        let thing = text::strip_chars(found.str(thing_group), " .,").to_string();
        if _family(&thing).is_empty() {
            continue;
        }
        if found.get(currency_group).is_none_or(str::is_empty) && _an_age_not_a_bill(&found, &thing, said_as_a_cost) {
            continue;
        }
        return Some(Spend { amount: _amount(found.str(amount_group)), thing, line: String::new() });
    }
    None
}

/// Python: bot/memory/spent.py:138 `_an_age_not_a_bill(match, thing, pattern)`
/// Whether a cost said as a fact is really somebody's age.
pub fn _an_age_not_a_bill(found: &Match, thing: &str, said_as_a_cost: bool) -> bool {
    if !said_as_a_cost {
        return false;
    }
    if text::split_ref(&thing.to_lowercase()).iter().any(|w| _PEOPLE.contains(*w)) {
        return true;
    }
    _TWO_FIGURES.match_(&found.text[found.end()..]).is_some()
}

/// Python: bot/memory/spent.py:148 `_one(text)`
/// The spend one clause reports, or None.
pub fn _one(text_: &str) -> Option<Spend> {
    let found = SPENT.search(text_).or_else(|| COST.search(text_)).or_else(|| BARE_ON.match_(text_));
    match found {
        Some(found) => Some(Spend { amount: _amount(found.str(1)), thing: _the_thing(found.str(2)), line: String::new() }),
        None => _a_bill(text_),
    }
}

/// Python: bot/memory/spent.py:168 `_plain(text)`
/// The line with the pronoun contractions spelled the way the readers expect.
pub fn _plain(text_: &str) -> String {
    _PLAIN.sub_fn(text_, |m| _PLAIN_BACK.get(&m.str(1).to_lowercase()).cloned().expect("KeyError"))
}

/// Python: bot/memory/spent.py:173 `readings(line)`
/// Every spend a line reports, in the order they were said.
pub fn readings(line: &str) -> Vec<Spend> {
    let text_ = _plain(&text::squash(line));
    if text_.is_empty() || text_.contains('?') {
        return Vec::new();
    }
    _TWO_SPENDS.split(&text_).iter().filter_map(|clause| _one(clause)).collect()
}

/// Python: bot/memory/spent.py:186 `reading(line)`
/// The first spend a line reports, or None.
pub fn reading(line: &str) -> Option<Spend> {
    readings(line).into_iter().next()
}

/// Python: bot/memory/spent.py:195 `the_lines(memory, text=None)`
/// Every line the person has said this chat, oldest first, with `text` last if given ("" is None).
pub fn the_lines(memory: &Memory, text_: &str) -> Vec<String> {
    let mut lines: Vec<String> = memory.turns.iter().map(|t| t.user.clone()).filter(|said| !text::strip(said).is_empty()).collect();
    if !text_.is_empty() && lines.last().is_none_or(|last| last != text_) {
        lines.push(text_.to_string());
    }
    lines
}

/// Python: bot/memory/spent.py:207 `the_spends(memory, text=None)`
/// What they have said they spent, oldest first.
pub fn the_spends(memory: &Memory, text_: &str) -> Vec<Spend> {
    let mut found = Vec::new();
    for line in the_lines(memory, text_) {
        for mut spend in readings(&line) {
            spend.line = line.clone();
            found.push(spend);
        }
    }
    found
}

/// Python: bot/memory/spent.py:228 `_named(thing, wanted)`
/// Whether a spend went on the very thing the present-tense ask names.
pub fn _named(thing: &str, wanted: &str) -> bool {
    let thing = text::strip(&thing.to_lowercase()).to_string();
    let wanted = text::strip(&wanted.to_lowercase()).to_string();
    if thing.is_empty() || wanted.is_empty() {
        return false;
    }
    if thing.contains(&wanted) || wanted.contains(&thing) {
        return true;
    }
    let bare = text::rstrip_chars(&wanted, "s");
    !bare.is_empty() && thing.contains(bare)
}

/// Python: bot/memory/spent.py:243 `_about(thing, wanted)`
/// Whether the thing a spend went on is the thing the ask names, by name or by family.
pub fn _about(thing: &str, wanted: &str) -> bool {
    let thing = thing.to_lowercase();
    let wanted = text::strip(&wanted.to_lowercase()).to_string();
    if wanted.is_empty() {
        return true;
    }
    if thing.is_empty() {
        return false;
    }
    let bare = text::rstrip_chars(&wanted, "s");
    if thing.contains(&wanted) || wanted.contains(&thing) || (!bare.is_empty() && thing.contains(bare)) {
        return true;
    }
    !_family(&thing).is_disjoint(&_family(&wanted))
}

/// Python: bot/memory/spent.py:259 `_family(thing)`
/// The words one spend and one ask can both be named by: a broad name and its members.
pub fn _family(thing: &str) -> HashSet<String> {
    let low = thing.to_lowercase();
    let mut words = HashSet::new();
    for (name, members) in CATEGORIES.iter() {
        if low.contains(name.as_str()) || members.iter().any(|m| low.contains(m.as_str())) {
            words.insert(name.clone());
            words.extend(members.iter().cloned());
        }
    }
    words
}

/// A sum as the person's own figures read: whole numbers bare, pence kept.
fn as_figure(total: f64) -> String {
    if (total - text::round_half_even(total) as f64).abs() < 0.005 {
        text::round_half_even(total).to_string()
    } else {
        format!("{total:.2}")
    }
}

/// Python: bot/memory/spent.py:270 `the_total(spends)`
/// The spends added up, as a figure that reads like the ones they typed.
pub fn the_total(spends: &[Spend]) -> String {
    let total: f64 = spends.iter().filter_map(|s| if s.amount.is_empty() { Some(0.0) } else { text::strip(&s.amount).parse::<f64>().ok() }).sum();
    if total == 0.0 {
        return String::new();
    }
    as_figure(total)
}

/// Python: bot/memory/spent.py:284 `_listed(spends)`
/// "300 on food and 40 on the train".
pub fn _listed(spends: &[Spend]) -> String {
    let parts: Vec<String> =
        spends.iter().map(|s| if s.thing.is_empty() { s.amount.clone() } else { format!("{} on {}", s.amount, s.thing) }).collect();
    if parts.len() == 1 {
        return parts[0].clone();
    }
    format!("{} and {}", parts[..parts.len() - 1].join(", "), parts[parts.len() - 1])
}

/// The words of an ask's thing without the filler.
fn without_filler(said: &str) -> String {
    text::split_ref(said).into_iter().filter(|w| !_ASK_FILLER.contains(&w.to_lowercase())).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/spent.py:295 `answer(text, memory)`
/// The reply for "how much have i spent" and its many wordings, or None.
pub fn answer(text_: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(text_);
    if said.is_empty() {
        return None;
    }
    let past = ASK.is_match(&said);
    let present = PRESENT_ASK.is_match(&said) && !past;
    if !past && !present {
        return None;
    }
    let mut wanted = ON_A_THING.search(&said).map(|on| without_filler(on.str(1))).unwrap_or_default();
    let mut spends;
    if present {
        if wanted.is_empty() {
            if let Some(mine) = A_POSSESSIVE_ASK.search(&said) {
                wanted = without_filler(mine.str(1));
            }
        }
        if wanted.is_empty() {
            return None;
        }
        spends = the_spends(memory, &said).into_iter().filter(|s| _named(&s.thing, &wanted)).collect::<Vec<_>>();
        if spends.is_empty() {
            return None;
        }
    } else {
        spends = the_spends(memory, &said);
    }
    if !wanted.is_empty() {
        spends.retain(|s| _about(&s.thing, &wanted));
        if spends.is_empty() {
            return Some(format!("You haven't told me what you spent on {wanted}. Tell me and I'll keep a tally."));
        }
        if spends.len() == 1 {
            return Some(format!("You told me {}.", _listed(&spends)));
        }
        return Some(format!("You've told me {} - {} on {wanted} in all.", _listed(&spends), the_total(&spends)));
    }
    if spends.is_empty() {
        return Some("You haven't told me what you've spent yet. Tell me the sums as they go out and I'll keep a tally.".into());
    }
    if spends.len() == 1 {
        return Some(format!("You've told me {}. Tell me the rest and I'll keep a tally.", _listed(&spends)));
    }
    Some(format!("You've told me {} - {} in all.", _listed(&spends), the_total(&spends)))
}

/// Python: bot/memory/spent.py:400 `_a_count_of_a_thing(text)`
/// True when the figure in the line counts something, so it is not money of theirs.
pub fn _a_count_of_a_thing(text_: &str) -> bool {
    if _A_COUNTED_NOUN.findall_str(text_).iter().any(|noun| !_A_CURRENCY_PLURAL.contains(&noun.to_lowercase())) {
        return true;
    }
    if _A_SCOPE_OF_A_THING.is_match(text_) {
        return true;
    }
    let before = if text_.contains("there") { text_.split_once("there").unwrap().0 } else { "" };
    _A_PLURAL.findall_str(before).iter().any(|word| {
        let word = word.to_lowercase();
        !(_A_MONEY_PLURAL.contains(&word) || word.ends_with("ss"))
    })
}

/// Python: bot/memory/spent.py:445 `_figure(value)`
/// A sum of money the way the person's own figures read: 20, 14, 37.50.
pub fn _figure(value: f64) -> String {
    as_figure(value)
}

/// Python: bot/memory/spent.py:453 `_with_scale(number, word)`
/// A figure with the size word written after it: "6 thousand" -> "6000", "2k" -> "2000".
pub fn _with_scale(number: &str, word: &str) -> String {
    let word = text::strip(word).to_lowercase();
    if !(word.starts_with('k') || word.starts_with("thousand") || word.starts_with("grand")) {
        return number.to_string();
    }
    let Ok(value) = text::strip(number).parse::<f64>().map(|v| v * 1000.0) else { return number.to_string() };
    if (value - text::round_half_even(value) as f64).abs() < 0.005 { text::round_half_even(value).to_string() } else { number.to_string() }
}

/// Python: bot/memory/spent.py:470 `_held_in(line)`
/// The money one line says they have, as a figure, or None.
pub fn _held_in(line: &str) -> Option<String> {
    let text_ = _plain(&text::squash(line));
    if text_.is_empty() || text_.contains('?') {
        return None;
    }
    if let Some(found) = BUDGET_HELD.search(&text_).or_else(|| ON_A_BUDGET.search(&text_)).or_else(|| SAVINGS_HELD.search(&text_)) {
        return Some(_with_scale(found.str(1), found.str(2)));
    }
    if let Some(found) = THERE_LEFT.search(&text_) {
        if !_a_count_of_a_thing(&text_) {
            return Some(_with_scale(found.str(1), found.str(2)));
        }
    }
    let found = HELD.search(&text_)?;
    let currency = text::strip(found.str(2));
    let tail = found.str(3);
    let marked = !currency.is_empty() || found.as_str().contains('£');
    if !marked && _HELD_SCOPE.match_(tail).is_none() {
        return None;
    }
    if !marked && _a_count_of_a_thing(&text_) {
        return None;
    }
    Some(_with_scale(found.str(1), found.str(2)))
}

/// Python: bot/memory/spent.py:493 `the_held(memory, text=None)`
/// The money they last said they had this chat, or None.
pub fn the_held(memory: &Memory, text_: &str) -> Option<Held> {
    let mut found = None;
    for line in the_lines(memory, text_) {
        if let Some(amount) = _held_in(&line) {
            found = Some(Held { amount, line });
        }
    }
    found
}

/// Python: bot/memory/spent.py:503 `_a_thing_to_buy(said)`
/// The thing an affordability ask names, in the words they used: "a takeaway", "new shoes".
pub fn _a_thing_to_buy(said: &str) -> String {
    let Some(found) = AFFORD.search(said) else { return String::new() };
    let thing = (1..=found.group_count()).find_map(|i| found.get(i).filter(|g| !g.is_empty())).unwrap_or("");
    let thing = SENTENCE_STOP.split(&text::squash(thing)).swap_remove(0);
    let thing = text::strip(&_TAIL_OFF.sub("", &_TRAILING.sub("", &thing))).to_string();
    let thing = text::strip(&A_POINTER.sub("", &thing)).to_string();
    let n = text::len(&thing);
    if 1 < n && n < 40 { thing } else { String::new() }
}

/// Python: bot/memory/spent.py:523 `_the_price(thing)`
/// The price a thing is named with, or None: "a 200 coat" -> 200.0.
pub fn _the_price(thing: &str) -> Option<f64> {
    let found = A_PRICE.search(thing)?;
    for group in ["sigil", "word", "ask"] {
        if let Some(value) = found.name(group).filter(|v| !v.is_empty()) {
            return Some(value.replace(',', "").parse().expect("ValueError"));
        }
    }
    let item = found.name_str("item");
    if item.ends_with('s') && !item.ends_with("ss") {
        return None; // "10 tickets" is a count, not a price
    }
    let bare = found.name("bare").filter(|b| !b.is_empty()).unwrap_or("0");
    let value: f64 = bare.replace(',', "").parse().expect("ValueError");
    if value >= 10.0 { Some(value) } else { None }
}

/// Python: bot/memory/spent.py:545 `_bare_thing(thing)`
/// The thing without the little word in front of it: "a holiday" -> "holiday".
pub fn _bare_thing(thing: &str) -> String {
    A_DETERMINER.sub("", text::strip(thing))
}

/// Python: bot/memory/spent.py:555 `_only_the_ask(said)`
/// True when the line is the sum asked and nothing else.
pub fn _only_the_ask(said: &str) -> bool {
    text::split_ref(said)
        .iter()
        .map(|w| text::strip_chars(w, ".,!?;:'\"").to_lowercase())
        .all(|w| w.is_empty() || _ASK_WORDS.contains(&w))
}

/// Python: bot/memory/spent.py:561 `_nothing_before(memory, text)`
/// True when this line is the first thing said in the chat.
pub fn _nothing_before(memory: &Memory, text_: &str) -> bool {
    the_lines(memory, text_).len() <= 1
}

/// The amounts spent, added up (`sum(float(spend.get("amount") or 0) for spend in spends)`).
fn spent_out(spends: &[Spend]) -> f64 {
    spends.iter().map(|s| money(&s.amount)).sum()
}

/// Python: bot/memory/spent.py:580 `_the_what_if(match, memory)`
/// The sum with a spend in it that has not happened, or None.
pub fn _the_what_if(found: &Match, memory: &Memory) -> Option<String> {
    let currency = text::strip(found.name_str("currency")).to_lowercase();
    if currency == "k" || currency == "p" {
        return None;
    }
    if A_LENGTH_NOT_MONEY.match_(found.name_str("tail")).is_some() {
        return None;
    }
    let amount = money(&found.name("amount").unwrap_or("0").replace(',', ""));
    let held = the_held(memory, "")?;
    if amount <= 0.0 {
        return None;
    }
    let had: f64 = text::strip(&held.amount).parse().ok()?;
    let spends = the_spends(memory, "");
    let left = had - if spends.is_empty() { 0.0 } else { spent_out(&spends) };
    if amount > left {
        return Some(format!("Spend {} more and you are {} over - {} is what is left now.", _figure(amount), _figure(amount - left), _figure(left)));
    }
    if spends.is_empty() {
        return Some(format!("Spend {} of the {} you told me you have and {} is left.", _figure(amount), _figure(had), _figure(left - amount)));
    }
    Some(format!("Spend {} more and {} is left.", _figure(amount), _figure(left - amount)))
}

/// Python: bot/memory/spent.py:605 `what_is_left(text, memory)`
/// The reply for "how much have i got left" and "can i afford a takeaway", or None.
pub fn what_is_left(text_: &str, memory: &Memory) -> Option<String> {
    let said = text::squash(text_);
    if said.is_empty() {
        return None;
    }
    if let Some(what_if) = A_WHAT_IF.search(&said) {
        return _the_what_if(&what_if, memory);
    }
    if !(LEFT.is_match(&said) || AFFORD.is_match(&said)) {
        return None;
    }
    if !readings(&said).is_empty() {
        return None; // a line with a spend in it is a report, not the ask
    }
    let Some(held) = the_held(memory, "") else {
        if _nothing_before(memory, &said) && _only_the_ask(&said) {
            if AFFORD.is_match(&said) && !LEFT.is_match(&said) {
                return Some("Afford what, and with what? Tell me the thing and what you've got to spend, and I'll do the sum.".into());
            }
            return Some("Left of what? Tell me what you've got and what has gone out of it, and I'll do the sum.".into());
        }
        return None;
    };
    let had: f64 = text::strip(&held.amount).parse().ok()?;
    let spends = the_spends(memory, "");
    let out = if spends.is_empty() { 0.0 } else { spent_out(&spends) };
    let left = had - out;
    if AFFORD.is_match(&said) && !LEFT.is_match(&said) {
        let thing = _a_thing_to_buy(&said);
        let price = _the_price(&thing);
        let name = _bare_thing(&thing);
        let named = if NAMED_WITH_A_DETERMINER.match_(&thing).is_some() {
            thing.clone()
        } else if !name.is_empty() {
            format!("the {name}")
        } else {
            String::new()
        };
        let cost = format!("{} and I'll say whether it fits.", if named.is_empty() { "Tell me the price".to_string() } else { format!("Tell me the price of {named}") });
        if !spends.is_empty() && left <= 0.0 {
            let after = if thing.is_empty() { "Tell me if more comes in and I'll work it out again.".to_string() } else { format!("{} needs more coming in first.", text::capitalize(&thing)) };
            return Some(format!("You had {} and {} has gone, so you are {} over. {after}", _figure(had), _listed(&spends), _figure(-left)));
        }
        if let Some(price) = price {
            if !spends.is_empty() {
                if price > left {
                    return Some(format!("No - {thing} is more than the {} you have left. It wants {} more.", _figure(left), _figure(price - left)));
                }
                return Some(format!("Yes - {thing} comes out of the {} you have left, with {} spare.", _figure(left), _figure(left - price)));
            }
            if price > had {
                return Some(format!("No - {thing} is more than the {} you told me you have. It wants {} more.", _figure(had), _figure(price - had)));
            }
            return Some(format!("Yes - {thing} comes out of the {} you told me you have, with {} spare.", _figure(had), _figure(had - price)));
        }
        if spends.is_empty() {
            return Some(format!(
                "You told me you have {} and nothing has gone out of it yet, so anything up to {} is covered. {cost}",
                _figure(had),
                _figure(had)
            ));
        }
        return Some(format!("You had {} and {} has gone, so {} is left. {cost}", _figure(had), _listed(&spends), _figure(left)));
    }
    if spends.is_empty() {
        return Some(format!("You told me you have {}, and you've not said that any of it has gone yet.", _figure(had)));
    }
    if left <= 0.0 {
        return Some(format!("You had {} and {} has gone, so you are {} over.", _figure(had), _listed(&spends), _figure(-left)));
    }
    Some(format!("You had {} and {} has gone, so {} is left.", _figure(had), _listed(&spends), _figure(left)))
}
