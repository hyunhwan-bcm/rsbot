//! Port of `bot/modules/attribute_said_again.py`: a fact of theirs asked for again ("what size
//! am i again", "how much did i say the rent was", "when did i say we were going") is answered
//! from their own earlier line, in the words they used.
#![allow(non_snake_case, clippy::all)]

use super::chat::{dyn_re, said};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! res {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.attribute_said_again.", stringify!($n))));)* };
}
macro_rules! vals {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Value> = LazyLock::new(|| data::get(concat!("bot.modules.attribute_said_again.", stringify!($n))));)* };
}

res!(
    ASK, OF_MINE, A_FIGURE, NOT_A_COST, PAY_ASK, COST_ASK, WHATS_MY, WHEN_ASK, WHERE_ASK,
    TIME_SPAN, PLACE, SIZE_ASK, SIZE_ADVICE, A_SIZE, MONEY_ASK, A_PER_YEAR, A_MONEY, AGE_ASK,
    OFTEN_ASK, AN_AGE_CLAUSE, SOMEBODY, A_COUNT_OF_PEOPLE, A_HABIT_SAID, PRONOUN_LINE, SAID_ASK,
    THEIR_QUESTION, OFF_WITH_ASK, OFF_WITH_SAID,
);
vals!(
    QUESTION_WORDS, CONTRACTIONS, MONTHS, WEEKDAYS, TIME_WORDS, SALARY_WORDS, MONEY_WORDS,
    SLOT_STOP, VALUE_KINDS, HOW_KINDS, PAY_ASK_WORDS,
);

static WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
static WORD_CURLY: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z'\u{2019}]+", 0));

fn words(s: &str) -> Vec<String> {
    WORD.findall_str(s)
}

fn stems(s: &str) -> HashSet<String> {
    words(&s.to_lowercase()).iter().map(|w| _stem(w)).collect()
}

/// Python: bot/modules/attribute_said_again.py:128 `_their_attribute(low)`
pub fn _their_attribute(low: &str) -> String {
    for pattern in [&*ASK, &*OF_MINE] {
        if let Some(m) = pattern.search(low) {
            return text::squash(&m.name_str("attr").to_lowercase());
        }
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:137 `_stem(word)`
pub fn _stem(word: &str) -> String {
    let word = text::strip_chars(&word.to_lowercase(), "'").to_string();
    for suffix in ["ing", "ed", "es", "s"] {
        if text::len(&word) > suffix.len() + 3 && word.ends_with(suffix) {
            return word[..word.len() - suffix.len()].to_string();
        }
    }
    word
}

/// Python: bot/modules/attribute_said_again.py:146 `_names(tokens, attribute)`
pub fn _names(tokens: &[String], attribute: &str) -> bool {
    let stems: HashSet<String> = tokens.iter().map(|t| _stem(t)).collect();
    text::split_ref(attribute).iter().all(|w| stems.contains(&_stem(w)))
}

/// Python: bot/modules/attribute_said_again.py:152 `_the_phrase_of_theirs(attribute, memory, needs_a_figure=False)`
pub fn _the_phrase_of_theirs(attribute: &str, memory: &Memory, needs_a_figure: bool) -> String {
    for turn in memory.turns.iter().rev() {
        let line = &turn.user;
        if line.is_empty() {
            continue;
        }
        if !_names(&words(&line.to_lowercase()), attribute) {
            continue;
        }
        if needs_a_figure && !(A_FIGURE.search(line).is_some() && NOT_A_COST.search(line).is_none()) {
            continue;
        }
        let phrase = _phrase(line);
        if phrase.is_empty() || !_names(&words(&phrase.to_lowercase()), attribute) {
            continue;
        }
        return phrase;
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:182 `_a_cost_line(line)`
pub fn _a_cost_line(line: &str) -> bool {
    !line.is_empty() && A_FIGURE.search(line).is_some() && NOT_A_COST.search(line).is_none()
}

/// Python: bot/modules/attribute_said_again.py:188 `_the_newest_cost(memory)`
pub fn _the_newest_cost(memory: &Memory) -> String {
    for turn in memory.turns.iter().rev() {
        if _a_cost_line(&turn.user) {
            return _phrase(&turn.user);
        }
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:200 `_the_cost_just_before(memory)`
pub fn _the_cost_just_before(memory: &Memory) -> String {
    let Some(last) = memory.turns.last() else { return String::new() };
    if _a_cost_line(&last.user) {
        _phrase(&last.user)
    } else {
        String::new()
    }
}

/// Python: bot/modules/attribute_said_again.py:213 `_phrase(line)`
pub fn _phrase(line: &str) -> String {
    super::chat::attempt(|| crate::memory::store_replies::_told_phrase(line)).unwrap_or_default()
}

static VALUE_FRAMES: LazyLock<Vec<Re>> = LazyLock::new(|| {
    [
        r"^you(?:'re| are| were) (.+)$",
        r"^you(?:'ve| have) (.+)$",
        r"^your .+? is (.+)$",
        r"^your (.+)$",
        r"^it(?:'s|s) (.+)$",
        r"^you (?:pay|paid) (.+)$",
        r"^(?:it )?costs? (.+)$",
    ]
    .iter()
    .map(|p| Re::new(p, 0))
    .collect()
});

/// Python: bot/modules/attribute_said_again.py:226 `_the_value(phrase)`
pub fn _the_value(phrase: &str) -> String {
    for re in VALUE_FRAMES.iter() {
        if let Some(m) = re.match_(phrase) {
            return text::rstrip_chars(text::strip(m.str(1)), ".").to_string();
        }
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:368 `_the_slot_ask(low)`: (slot, subject words).
pub fn _the_slot_ask(low: &str) -> (String, HashSet<String>) {
    for (slot, pattern) in [("when", &*WHEN_ASK), ("where", &*WHERE_ASK)] {
        if let Some(m) = pattern.search(low) {
            let rest = (1..=m.group_count()).filter_map(|i| m.get(i)).find(|g| !g.is_empty()).unwrap_or("");
            return (slot.into(), _the_subject(rest));
        }
    }
    for (slot, pattern) in [("age", &*AGE_ASK), ("often", &*OFTEN_ASK), ("off_with", &*OFF_WITH_ASK)] {
        if pattern.search(low).is_some() {
            return (slot.into(), HashSet::new());
        }
    }
    if !_their_attribute(low).is_empty() {
        return (String::new(), HashSet::new());
    }
    if let Some(m) = SIZE_ASK.search(low) {
        if SIZE_ADVICE.search(low).is_none() {
            return ("size".into(), _the_subject(m.name_str("rest")));
        }
    }
    if let Some(m) = MONEY_ASK.search(low) {
        let what = (1..=m.group_count()).filter_map(|i| m.get(i)).find(|g| !g.is_empty()).unwrap_or("").to_lowercase();
        return (format!("money:{what}"), HashSet::new());
    }
    (String::new(), HashSet::new())
}

/// Python: bot/modules/attribute_said_again.py:394 `_the_subject(rest)`
pub fn _the_subject(rest: &str) -> HashSet<String> {
    words(&rest.to_lowercase()).iter().filter(|w| !SLOT_STOP.contains_str(w)).map(|w| _stem(w)).collect()
}

/// Python: bot/modules/attribute_said_again.py:400 `_a_time(line)`
pub fn _a_time(line: &str) -> String {
    let low = line.to_lowercase();
    let found = WORD_CURLY.findall_str(&low);
    if let Some(named) = found.iter().find(|w| MONTHS.contains_str(w) || WEEKDAYS.contains_str(w)) {
        return text::capitalize(named);
    }
    if let Some(span) = TIME_SPAN.search(&low) {
        return text::squash(span.as_str());
    }
    for word in &found {
        if TIME_WORDS.contains_str(word) {
            return word.clone();
        }
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:420 `_a_place(line)`
pub fn _a_place(line: &str) -> String {
    let Some(m) = PLACE.search(line) else { return String::new() };
    let place = text::squash(m.name_str("place"));
    if place.is_empty() || text::split_ref(&place).iter().map(|w| _stem(w)).all(|s| SLOT_STOP.contains_str(&s)) {
        return String::new();
    }
    text::squash(m.as_str())
}

/// Python: bot/modules/attribute_said_again.py:431 `_a_size(line, subject)`
pub fn _a_size(line: &str, subject: &HashSet<String>) -> String {
    if let Some(m) = A_SIZE.finditer(line).into_iter().next() {
        let measure = text::squash(m.name_str("measure"));
        let after = text::strip(m.name("after").unwrap_or("")).to_lowercase();
        if !after.is_empty() && !subject.is_empty() && subject.contains(&_stem(&after)) {
            return format!("{measure} {after}");
        }
        return measure;
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:451 `_the_size_recall(subject, lines)`
pub fn _the_size_recall(subject: &HashSet<String>, lines: &[String]) -> String {
    for index in (0..lines.len()).rev() {
        let line = &lines[index];
        let value = _a_size(line, subject);
        if value.is_empty() {
            continue;
        }
        let said_ = stems(line);
        if !subject.is_empty() {
            if !said_.is_disjoint(subject) {
                return value;
            }
            let older = if index > 0 { lines[index - 1].as_str() } else { "" };
            if PRONOUN_LINE.match_(line).is_some() && !subject.is_disjoint(&stems(older)) {
                return value;
            }
            continue;
        }
        if lines.len() - index <= 2 {
            return value;
        }
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:475 `_the_money_recall(what, lines)`
pub fn _the_money_recall(what: &str, lines: &[String]) -> String {
    for line in lines.iter().rev() {
        let said_ = stems(line);
        if SALARY_WORDS.contains_str(what) {
            if let Some(m) = A_PER_YEAR.search(line) {
                return text::squash(m.as_str());
            }
            continue;
        }
        if !said_.contains(&_stem(what)) {
            continue;
        }
        if let Some(m) = A_MONEY.search(line) {
            return text::squash(m.as_str());
        }
    }
    String::new()
}

/// What a line has to hold to answer an ask: a pattern, the time reader, or the money search.
pub enum ValueKind {
    Pattern(&'static Re),
    Time,
    Money,
}

/// Python: bot/modules/attribute_said_again.py:543 `_the_value_kind(attribute, how)`
pub fn _the_value_kind(attribute: Option<&str>, how: Option<&str>) -> Option<ValueKind> {
    let attribute = text::squash(&attribute.unwrap_or("").to_lowercase());
    if !attribute.is_empty() {
        if let Some(kind) = VALUE_KINDS.get(&attribute) {
            return kind.as_re().map(ValueKind::Pattern);
        }
        if ["time", "day", "date", "month", "year", "week"].contains(&attribute.as_str()) {
            return Some(ValueKind::Time);
        }
        if MONEY_WORDS.contains_str(&attribute) || attribute == "price" || attribute == "cost" {
            return Some(ValueKind::Money);
        }
        return None;
    }
    HOW_KINDS.get(&text::squash(&how.unwrap_or("").to_lowercase())).and_then(|k| k.as_re()).map(ValueKind::Pattern)
}

fn holds(kind: &ValueKind, line: &str) -> bool {
    match kind {
        ValueKind::Pattern(re) => re.search(line).is_some(),
        ValueKind::Time => !_a_time(line).is_empty(),
        ValueKind::Money => A_MONEY.search(line).is_some(),
    }
}

fn kind_of_ask(low: &str) -> Option<ValueKind> {
    let m = SAID_ASK.match_(low)?;
    _the_value_kind(m.name("attr"), m.name("how"))
}

/// Python: bot/modules/attribute_said_again.py:564 `_the_line_that_says_it(text, memory)`
pub fn _the_line_that_says_it(text_: &str, memory: &Memory) -> String {
    let low = text::squash(&text_.to_lowercase());
    let Some(kind) = kind_of_ask(&low) else { return String::new() };
    for turn in memory.turns.iter().rev() {
        let line = text::squash(&turn.user);
        if line.is_empty() || !holds(&kind, &line) {
            continue;
        }
        if THEIR_QUESTION.match_(&line).is_some() || SAID_ASK.match_(&line.to_lowercase()).is_some() {
            continue;
        }
        if !_their_attribute(&line.to_lowercase()).is_empty() {
            continue;
        }
        return line;
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:592 `_the_values_the_kind_takes(text, line)`
pub fn _the_values_the_kind_takes(text_: &str, line: &str) -> Vec<String> {
    let low = text::squash(&text_.to_lowercase());
    let Some(kind) = kind_of_ask(&low) else { return Vec::new() };
    let pattern = match kind {
        ValueKind::Time => {
            let value = text::squash(&_a_time(line));
            return if value.is_empty() { Vec::new() } else { vec![value] };
        }
        ValueKind::Money => {
            // Python: `str(kind(line) or "")` where `kind` is `lambda line: A_MONEY.search(line)`,
            // so a found figure is said as the match object's repr.
            let value = match A_MONEY.search(line) {
                Some(m) => text::squash(&format!(
                    "<re.Match object; span=({}, {}), match={}>",
                    text::char_index(line, m.start()),
                    text::char_index(line, m.end()),
                    crate::understood::py_str_repr(m.as_str())
                )),
                None => String::new(),
            };
            return if value.is_empty() { Vec::new() } else { vec![value] };
        }
        ValueKind::Pattern(re) => re,
    };
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    for hit in pattern.finditer(line) {
        let value = text::squash(hit.as_str());
        let key = _stem(&value);
        if !value.is_empty() && !seen.contains(&key) {
            seen.insert(key);
            found.push(value);
        }
    }
    found
}

/// Python: bot/modules/attribute_said_again.py:619 `_the_value_said(text, memory)`
pub fn _the_value_said(text_: &str, memory: &mut Memory) -> Option<String> {
    let line = _the_line_that_says_it(text_, memory);
    if line.is_empty() {
        return None;
    }
    if crate::modules::memory_store::respond(text_, memory).is_some_and(|r| !r.is_empty()) {
        return None;
    }
    let values = _the_values_the_kind_takes(text_, &line);
    let said_ = if values.len() == 1 { values[0].clone() } else { line.clone() };
    let reply = if values.len() != 1 { format!("You said: \"{said_}\"") } else { format!("You said {said_}.") };
    let last = _the_last_reply(memory);
    if last.to_lowercase().contains(text::rstrip_chars(&reply.to_lowercase(), ".")) {
        return Some(format!("Still the same: {said_}"));
    }
    Some(reply)
}

/// Python: bot/modules/attribute_said_again.py:644 `_the_slot_recall(text, memory)`
pub fn _the_slot_recall(text_: &str, memory: &Memory) -> Option<String> {
    let low = text::squash(&text_.to_lowercase());
    if low.is_empty() {
        return None;
    }
    let (slot, subject) = _the_slot_ask(&low);
    if slot.is_empty() {
        return None;
    }
    let turns = &memory.turns;
    if turns.is_empty() {
        return None;
    }
    if slot == "age" || slot == "often" || slot == "off_with" {
        let value = match slot.as_str() {
            "age" => _the_age_they_gave(turns, &low),
            "often" => _the_habit_they_gave(turns),
            _ => _the_off_with_they_gave(turns),
        };
        return if value.is_empty() { None } else { Some(_the_slot_reply(&value, memory)) };
    }
    let lines: Vec<String> = turns.iter().map(|t| t.user.clone()).collect();
    if let Some(what) = slot.strip_prefix("money:") {
        let value = _the_money_recall(what, &lines);
        return if value.is_empty() { None } else { Some(_the_slot_reply(&value, memory)) };
    }
    if slot == "size" {
        let value = _the_size_recall(&subject, &lines);
        return if value.is_empty() { None } else { Some(_the_slot_reply(&value, memory)) };
    }
    for turn in turns.iter().rev() {
        let line = &turn.user;
        if line.is_empty() {
            continue;
        }
        let value = if slot == "when" { _a_time(line) } else { _a_place(line) };
        if value.is_empty() {
            continue;
        }
        if !subject.is_empty() && stems(line).is_disjoint(&subject) {
            continue;
        }
        return Some(_the_slot_reply(&value, memory));
    }
    None
}

static WHOSE_AGE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(his|her|their|my|our|its)\b[^.?!]{0,12}?\b(?:age|old)\b", crate::py::re::I));
static SAID_OF: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(he|she|they)\b", crate::py::re::I));
static FIRST_PERSON: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:i|im|i'm|am)\b", crate::py::re::I));

/// Python: bot/modules/attribute_said_again.py:700 `_the_age_they_gave(turns, low)`
pub fn _the_age_they_gave(turns: &[Turn], low: &str) -> String {
    let mut whose = "their".to_string();
    if let Some(named) = WHOSE_AGE.search(low) {
        whose = named.str(1).to_lowercase();
    } else if let Some(said_of) = SAID_OF.search(low) {
        whose = match said_of.str(1).to_lowercase().as_str() {
            "he" => "his",
            "she" => "her",
            _ => "their",
        }
        .into();
    }
    if whose == "my" {
        whose = "your".into();
    }
    for turn in turns.iter().rev() {
        let line = &turn.user;
        if line.is_empty() || A_COUNT_OF_PEOPLE.search(line).is_some() || SOMEBODY.search(line).is_none() {
            continue;
        }
        let Some(found) = AN_AGE_CLAUSE.search(line) else { continue };
        let number = found.name_str("n").to_string();
        let n: i64 = number.parse().unwrap_or(0);
        if !(0 < n && n < 121) {
            continue;
        }
        let named_person = dyn_re(
            &format!(r"\b(?:my|our)\s+(?P<who>[a-z]+)\s+(?P<was>is|was)\s+{}\b", crate::py::re::escape(&number)),
            crate::py::re::I,
        );
        if let Some(np) = named_person.search(line) {
            return format!("your {} {} {}", np.name_str("who").to_lowercase(), np.name_str("was").to_lowercase(), number);
        }
        if (whose == "their" || whose == "its") && FIRST_PERSON.search(line).is_some() {
            return format!("your age was {number}");
        }
        if whose == "your" {
            return format!("your age was {number}");
        }
        return format!("{whose} age was {number}");
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:743 `_the_off_with_they_gave(turns)`
pub fn _the_off_with_they_gave(turns: &[Turn]) -> String {
    for turn in turns.iter().rev() {
        let line = &turn.user;
        if line.is_empty() {
            continue;
        }
        if let Some(found) = OFF_WITH_SAID.search(line) {
            let thing = text::squash(&found.name_str("what").to_lowercase());
            let article = if text::strip(found.name("thing").unwrap_or("")).to_lowercase() == "an" { "an " } else { "a " };
            return format!("you were off with {article}{thing}");
        }
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:757 `_the_habit_they_gave(turns)`
pub fn _the_habit_they_gave(turns: &[Turn]) -> String {
    for turn in turns.iter().rev() {
        let line = &turn.user;
        if line.is_empty() {
            continue;
        }
        if let Some(found) = A_HABIT_SAID.search(line) {
            return text::squash(&found.as_str().to_lowercase());
        }
    }
    String::new()
}

/// Python: bot/modules/attribute_said_again.py:769 `_the_slot_reply(value, memory)`
pub fn _the_slot_reply(value: &str, memory: &Memory) -> String {
    let reply = format!("You said {value}.");
    if _the_last_reply(memory).to_lowercase().contains(&reply.to_lowercase()) {
        return format!("Still {value}.");
    }
    reply
}

/// Python: bot/modules/attribute_said_again.py:777 `_the_last_reply(memory)`
pub fn _the_last_reply(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.bot.clone()).unwrap_or_default()
}

static ITS_AT_START: LazyLock<Re> = LazyLock::new(|| Re::new(r"^its\b", 0));

/// Python: bot/modules/attribute_said_again.py:784 `_spoken(phrase)`
pub fn _spoken(phrase: &str) -> String {
    let mut phrase = ITS_AT_START.sub("it's", phrase);
    for pair in CONTRACTIONS.items() {
        let (full, short) = (pair.items()[0].str(), pair.items()[1].str());
        phrase = dyn_re(&format!(r"\b{full}\b"), 0).sub(short, &phrase);
    }
    phrase
}

/// Python: bot/modules/attribute_said_again.py:806 `_the_ask_names_something_new(low, memory)`
pub fn _the_ask_names_something_new(low: &str, memory: &Memory) -> bool {
    let mut said_: HashSet<String> = HashSet::new();
    for turn in &memory.turns {
        said_.extend(stems(&turn.user));
    }
    let mut asked = stems(low);
    for word in PAY_ASK_WORDS.items() {
        asked.remove(&_stem(word.str()));
    }
    asked.iter().any(|w| !said_.contains(w))
}

/// Python: bot/modules/attribute_said_again.py:823 `_the_recall(text, memory)`
pub fn _the_recall(text_: &str, memory: &mut Memory) -> Option<String> {
    let low = text::squash(&text_.to_lowercase());
    if low.is_empty() {
        return None;
    }
    if let Some(said_value) = _the_value_said(&low, memory) {
        return Some(said_value);
    }
    if let Some(slot) = _the_slot_recall(&low, memory) {
        return Some(slot);
    }
    let mut attribute = _their_attribute(&low);
    let mut wants_a_figure = false;
    if attribute.is_empty() {
        if PAY_ASK.search(&low).is_some() || COST_ASK.match_(&low).is_some() {
            wants_a_figure = true;
        } else {
            let candidate = WHATS_MY.search(&low).map(|m| text::squash(m.name_str("attr"))).unwrap_or_default();
            let parts = text::split_ref(&candidate);
            if candidate.is_empty() || parts.len() > 3 || QUESTION_WORDS.contains_str(parts[0]) {
                return None;
            }
            attribute = candidate.clone();
            wants_a_figure = true;
        }
    }
    let phrase = if !attribute.is_empty() {
        _the_phrase_of_theirs(&attribute, memory, wants_a_figure)
    } else if COST_ASK.match_(&low).is_some() {
        _the_cost_just_before(memory)
    } else {
        _the_newest_cost(memory)
    };
    if phrase.is_empty() {
        return None;
    }
    if PAY_ASK.search(&low).is_some()
        && _the_ask_names_something_new(&low, memory)
        && said(crate::memory::repeat::the_figure_they_gave(text_, memory)).is_none()
    {
        return None;
    }
    let said_ = _spoken(&phrase);
    let value = _the_value(&said_);
    let last = _the_last_reply(memory).to_lowercase();
    if !value.is_empty() && last.contains(&value.to_lowercase()) && last.contains("you told me") {
        return Some(format!("Still {value}."));
    }
    Some(format!("You told me {said_}."))
}

/// Python: bot/modules/attribute_said_again.py:878 `takes_as_it_comes(text, memory)`
pub fn takes_as_it_comes(text_: &str, memory: &mut Memory) -> bool {
    let low = text::squash(&text_.to_lowercase());
    let (slot, _) = _the_slot_ask(&low);
    if slot == "age" || slot == "often" || slot == "off_with" {
        return false;
    }
    _the_recall(text_, memory).is_some()
}

/// Python: `TAKES_AS_IT_COMES = takes_as_it_comes` - the arbiter's hook.
pub fn TAKES_AS_IT_COMES(text_: &str, memory: &mut Memory) -> bool {
    takes_as_it_comes(text_, memory)
}

/// Python: bot/modules/attribute_said_again.py:904 `respond(text, memory)` - a runtime hook.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    _the_recall(text_, memory)
}
