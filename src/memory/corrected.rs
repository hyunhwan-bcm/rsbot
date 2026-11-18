//! Port of `bot/memory/corrected.py`: the correction readers moved out of
//! `bot/modules/memory_store.py`: a value put right ("sorry, I meant Florence, not Rome") moves
//! the fact, the plan, the clock, the count or the name it corrects.
//!
//! The memory this file writes is PyBot's memory dict: `memory["user"]`, `memory["others"]`,
//! `memory["facts_log"]`, `memory["topic"]`/`memory["topics"]` and `memory["correction"]`. Python
//! mutates the dicts it finds there in place; the port writes them back where they live. A topic
//! record held both as `memory["topic"]` and as an item of `memory["topics"]` is one object in
//! Python, and one `Arc` here: a write to it goes to every place that holds it.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::facts as extract;
use crate::memory::topics;
use crate::py::re::{Match, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

const I: u32 = crate::py::re::I;

fn g(module: &str, name: &str) -> &'static Value {
    data::get(&format!("bot.memory.{module}.{name}"))
}
fn cre(name: &str) -> &'static Re {
    g("corrected", name).as_re().unwrap_or_else(|| panic!("bot.memory.corrected.{name} is not a regex"))
}

macro_rules! cres {
    ($($n:ident),* $(,)?) => { $(
        static $n: LazyLock<&'static Re> = LazyLock::new(|| cre(stringify!($n)));
    )* };
}
cres!(
    OTHER_BARE_NUMBER, MADE_OF_A_VALUE, A_UNIT_SAID, NAMING_VERB, A_DAY_SAID_LONG, NUMBER_VALUE,
    NUMBER_DIGITS, BARE_HOUR, COUNT_SAID, ANY_NUMBER, BARE_NEW_COUNT, A_JOURNEY_GOING, DATE_DAY,
    A_DAY_BEFORE_A_MONTH, WEEKDAY_IN_PLAN, RELATIVE_MONTH, AMOUNT, NUMBER_WORD, MONTH_WORD,
    DATE_PHRASE, AT_TIME, DAY_WORD, CURRENCY, ASK_START,
);

fn cset(name: &str) -> HashSet<String> {
    g("corrected", name).str_set()
}
static NOT_A_FACT_OF_THEIRS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("NOT_A_FACT_OF_THEIRS"));
static CORRECTABLE: LazyLock<HashSet<String>> = LazyLock::new(|| cset("CORRECTABLE"));
static MONTHS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("MONTHS"));
static NO_SUBJECT_OF_ITS_OWN: LazyLock<HashSet<String>> = LazyLock::new(|| cset("NO_SUBJECT_OF_ITS_OWN"));
static STILL_THE_SAME_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| cset("STILL_THE_SAME_PERSON"));
static MONTH_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| cset("MONTH_WORDS"));

// The store's own tables (`bot/memory/store_lines.py`).
static AMOUNT_WORD: LazyLock<String> = LazyLock::new(|| g("store_lines", "AMOUNT_WORD").str().to_string());
static AT_TIME_VALUE: LazyLock<String> = LazyLock::new(|| g("store_lines", "AT_TIME_VALUE").str().to_string());
static NOT_A_UNIT: LazyLock<String> = LazyLock::new(|| g("store_lines", "NOT_A_UNIT").str().to_string());
static CORRECTION_BACK: LazyLock<i64> = LazyLock::new(|| g("store_lines", "CORRECTION_BACK").as_int().unwrap_or(4));
static SPELLING_APART: LazyLock<i64> = LazyLock::new(|| g("store_lines", "SPELLING_APART").as_int().unwrap_or(3));
static UNIT_FAMILY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("store_lines", "UNIT_FAMILY").str_map());
static FACT_FAMILY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("store_lines", "FACT_FAMILY").str_map());
static PLURAL_PETS: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("store_lines", "PLURAL_PETS").str_map());
static OTHER_KIND_SAY: LazyLock<HashMap<String, String>> = LazyLock::new(|| g("store_lines", "OTHER_KIND_SAY").str_map());
static OTHER_SAID: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("store_lines", "OTHER_SAID").items().iter().map(|t| (t.items()[0].str().to_string(), t.items()[1].str().to_string())).collect()
});
static COUNT_WORDS: LazyLock<Vec<(i64, String)>> = LazyLock::new(|| {
    g("store_lines", "COUNT_WORDS").pairs().iter().filter_map(|(k, v)| Some((k.as_int()?, v.str().to_string()))).collect()
});
/// `sorted(facts.MONTHS, key=len, reverse=True)` in the order the export's process made it (read
/// off `A_DAY_BEFORE_A_MONTH`). Python's set order breaks the ties of one length per process.
static MONTHS_LONGEST_FIRST: LazyLock<Vec<String>> = LazyLock::new(|| {
    let p = A_DAY_BEFORE_A_MONTH.pattern();
    let start = p.rfind("(?:").map(|i| i + 3).unwrap_or(0);
    let end = p[start..].find(')').map(|i| start + i).unwrap_or(p.len());
    p[start..end].split('|').map(str::to_string).collect()
});

// ---------------------------------------------------------------------------------------------
// Python idioms and the memory dict

/// `str(value or "")`.
fn s(v: &Value) -> String {
    if !v.truthy() {
        return String::new();
    }
    extract::py_str(v)
}

/// `str(value)` (None is "None").
fn sv(v: &Value) -> String {
    match v {
        Value::None => "None".into(),
        other => s(other),
    }
}

/// `" ".join(str(x or "").lower().split())`.
fn flat_low(text: &str) -> String {
    pt::squash(&pt::lower(text))
}

fn lower(text: &str) -> String {
    pt::lower(text)
}

fn dict(pairs: Vec<(&str, Value)>) -> Value {
    Value::Dict(Arc::new(pairs.into_iter().map(|(k, v)| (Value::from(k), v)).collect()))
}

/// `entry.get(key)`, or None.
fn vget<'a>(v: &'a Value, key: &str) -> &'a Value {
    static NONE: Value = Value::None;
    v.get(key).unwrap_or(&NONE)
}

/// Python's `re.escape`.
fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        if "()[]{}?*+-|^$\\.&~# \t\n\r\x0b\x0c".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// `s[:1].upper() + s[1:]`.
fn up1(text: &str) -> String {
    pt::upper_first(text)
}

/// `template.format(name=value, ...)` for named fields, in one pass.
fn fmt(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) => {
                let name = &after[..close];
                match fields.iter().find(|(k, _)| *k == name) {
                    Some((_, v)) => out.push_str(v),
                    None => panic!("KeyError: '{name}'"),
                }
                rest = &after[close + 1..];
            }
            None => {
                out.push_str(&rest[open..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// `bot.memory.user(memory)`: the user's facts (a dict), created on first use.
fn user(memory: &mut Memory) -> &mut Value {
    if !memory.get("user").is_some_and(Value::is_dict) {
        memory.set("user", Value::dict());
    }
    memory.get_mut("user").unwrap()
}

/// `bot.memory.log(memory)`: every fact the user stated, created on first use.
fn log(memory: &mut Memory) -> &mut Vec<Value> {
    if !matches!(memory.get("facts_log"), Some(Value::List(_))) {
        memory.set("facts_log", Value::list());
    }
    memory.get_mut("facts_log").unwrap().items_mut().unwrap()
}

/// `bot.memory.others(memory)`: facts about other people, created on first use.
fn others(memory: &mut Memory) -> &mut Vec<Value> {
    if !matches!(memory.get("others"), Some(Value::List(_))) {
        memory.set("others", Value::list());
    }
    memory.get_mut("others").unwrap().items_mut().unwrap()
}

/// `memory.get(key)` items when it is a list (`memory.get(key) or []`).
fn mem_items(memory: &Memory, key: &str) -> Vec<Value> {
    memory.get(key).map(|v| v.items().to_vec()).unwrap_or_default()
}

/// `(memory.get("user") or {})`, read without creating it.
fn user_read(memory: &Memory) -> Value {
    match memory.get("user") {
        Some(v) if v.truthy() => v.clone(),
        _ => Value::dict(),
    }
}

/// `last_user(memory) or ""`.
fn last_user(memory: &Memory) -> String {
    s(&crate::memory::last_user(memory, 1))
}

fn remember(memory: &mut Memory, key: &str, value: Value, text: &str) {
    crate::memory::remember(memory, key, &value, text);
}

fn remember_other(memory: &mut Memory, entry: &Value, text: &str) {
    crate::memory::remember_other(memory, entry, text);
}

fn note(memory: &mut Memory, text: &str) {
    let _ = topics::note(memory, text, "");
}

/// `a is b` for two dict values (one `Arc`).
fn same_object(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Dict(x), Value::Dict(y)) => Arc::ptr_eq(x, y),
        _ => false,
    }
}

/// Every place memory holds `old` (by identity) now holds `new`.
fn replace_object(memory: &mut Memory, old: &Value, new: &Value) {
    if memory.get("topic").is_some_and(|t| same_object(t, old)) {
        memory.set("topic", new.clone());
    }
    if let Some(list) = memory.get_mut("topics").and_then(Value::items_mut) {
        for item in list.iter_mut() {
            if same_object(item, old) {
                *item = new.clone();
            }
        }
    }
}

/// The user's plans as strings with text (`[p for p in known.get("plans") or [] if
/// isinstance(p, str) and p.strip()]`), with `plan` added when it is not among them.
fn plans_and_latest(known: &Value, add_latest_if_missing: bool) -> Vec<String> {
    let mut plans: Vec<String> = vget(known, "plans").items().iter().filter_map(|p| p.as_str()).filter(|p| !pt::strip(p).is_empty()).map(str::to_string).collect();
    if let Some(latest) = known.get("plan").and_then(Value::as_str) {
        if !pt::strip(latest).is_empty() && (!add_latest_if_missing || !plans.iter().any(|p| p == latest)) {
            plans.push(latest.to_string());
        }
    }
    plans
}

/// `known["plans"] = [fixed if test(p) else p for p in plans]` when the user's plans are a list.
fn replace_plan(memory: &mut Memory, fixed: &str, test: impl Fn(&Value) -> bool) {
    let known = user(memory);
    if let Some(Value::List(plans)) = known.get("plans") {
        let swapped: Vec<Value> = plans.iter().map(|p| if test(p) { Value::from(fixed) } else { p.clone() }).collect();
        known.set_key("plans", Value::from(swapped));
    }
}

// ---------------------------------------------------------------------------------------------
// someone else's facts

/// Python: bot/memory/corrected.py:17 `_number_belongs_to_that_age(memory)`: whether a bare number
/// can only be the age of the person named before it.
pub fn _number_belongs_to_that_age(memory: &Memory) -> bool {
    let last = pt::strip(&last_user(memory)).to_string();
    if last.is_empty() {
        return true;
    }
    let stated: Vec<Value> =
        mem_items(memory, "facts_log").into_iter().filter(|e| e.is_dict() && vget(e, "key").as_str() == Some("other")).collect();
    if let Some(last_stated) = stated.last() {
        if pt::strip(&s(vget(last_stated, "text"))) == last {
            return true;
        }
    }
    if let Some(fixed) = memory.get("correction").filter(|f| f.truthy() && f.is_dict()) {
        if pt::strip(&s(vget(fixed, "said"))) == last {
            let said = s(vget(fixed, "line"));
            if !said.is_empty() && vget(&extract::extract(&said, false), "others").truthy() {
                return true;
            }
        }
    }
    _asks(&last)
}

/// Python: bot/memory/corrected.py:43 `_a_name_for_a_person(value)`: whether a corrected value can be
/// somebody's name at all.
pub fn _a_name_for_a_person(value: &Value) -> bool {
    let word = pt::squash(&s(value));
    let low = lower(&word);
    if word.is_empty() || extract::MONTHS.contains(&low) || extract::DAYS.contains(&low) {
        return false;
    }
    if extract::_not_a_name(&word, false) {
        return false;
    }
    if extract::NAMES_THAT_END_LIKE_A_VERB.contains(&low) {
        return true;
    }
    !extract::NOT_NAME_ENDING.iter().any(|e| low.ends_with(e.as_str()))
}

/// Python: bot/memory/corrected.py:64 `_a_new_name_was_said(target, text)`: whether the correction
/// gives somebody a name at all, in one of the three shapes.
pub fn _a_new_name_was_said(target: &Value, text: &str) -> bool {
    let word = pt::squash(&s(target));
    if word.is_empty() {
        return false;
    }
    if crate::memory::named::carries_a_name(text) || _denies_and_names(text) {
        return true;
    }
    let parts = pt::split(&word);
    if parts.len() == 2 && _a_word_of_a_name(&parts[0]) && extract::a_name_could_be(&parts[1]) {
        return true;
    }
    extract::a_name_could_be(&word)
}

/// Python: bot/memory/corrected.py:90 `_a_word_of_a_name(word)`: whether a word can lead a name: a
/// title ("mr", "dr") or a name itself.
pub fn _a_word_of_a_name(word: &str) -> bool {
    let low = lower(pt::strip(pt::strip_chars(word, ".")));
    if extract::TITLE_WORDS.contains(&low) {
        return true;
    }
    extract::a_name_could_be(&low)
}

/// Python: bot/memory/corrected.py:101 `_denies_and_names(text)`: whether the line denies the old
/// name and gives the new one: "no he isn't, it's Dave".
pub fn _denies_and_names(text: &str) -> bool {
    if extract::a_name_is_taken_back(text) {
        return false;
    }
    match extract::DENIAL_OF_A_VALUE.match_(pt::strip(text)) {
        Some(d) => d.name("says").is_some_and(|v| !v.is_empty()) && d.name("value").is_some_and(|v| !v.is_empty()),
        None => false,
    }
}

/// Python: bot/memory/corrected.py:114 `_correct_other(memory, target, text="")`: the correction of
/// a fact kept against someone else, as {"line", "note"}, or None.
pub fn _correct_other(memory: &mut Memory, target: &Value, text: &str) -> Value {
    let known: Vec<usize> = others(memory)
        .iter()
        .enumerate()
        .filter(|(_, e)| e.is_dict() && vget(e, "who").truthy())
        .map(|(i, _)| i)
        .collect();
    if known.is_empty() {
        return Value::None;
    }
    let target_s = sv(target);
    let number = OTHER_BARE_NUMBER.match_(pt::strip(&s(target))).map(|m| m.as_str().to_string());
    if number.is_some() && !_number_belongs_to_that_age(memory) {
        return Value::None;
    }
    if let Some(number) = number {
        for &i in known.iter().rev() {
            let entry = others(memory)[i].clone();
            if vget(&entry, "age").as_int().is_none() {
                continue;
            }
            let who = pt::strip(&s(vget(&entry, "who"))).to_string();
            let age = number.parse::<i64>().unwrap_or(-1);
            if who.is_empty() || !(0..=120).contains(&age) {
                return Value::None;
            }
            remember_other(memory, &dict(vec![("who", who.as_str().into()), ("age", age.into())]), "");
            return dict(vec![
                ("line", format!("my {who} is {age}").into()),
                ("note", format!("{age} \u{2014} got it, your {who} is {age} from now on.").into()),
            ]);
        }
        return Value::None;
    }
    for &i in known.iter().rev() {
        let entry = others(memory)[i].clone();
        let who = pt::strip(&s(vget(&entry, "who"))).to_string();
        if who.is_empty() {
            continue;
        }
        let kept: Vec<&String> = OTHER_SAID.iter().filter(|(k, _)| vget(&entry, k).truthy()).map(|(k, _)| k).collect();
        for (kind, saying) in OTHER_SAID.iter() {
            if !kept.is_empty() && !kept.contains(&kind) {
                continue;
            }
            if kind == "name" && (!_a_name_for_a_person(target) || !_a_new_name_was_said(target, text)) {
                continue;
            }
            let line = format!("my {who} {saying} {target_s}");
            let found = extract::extract(&line, false);
            let mine: Vec<Value> = vget(&found, "others")
                .items()
                .iter()
                .filter(|item| item.is_dict() && lower(&s(vget(item, "who"))) == lower(&who))
                .cloned()
                .collect();
            if !mine.iter().any(|item| vget(item, kind).truthy()) {
                continue;
            }
            others(memory)[i].del_key(kind);
            for item in &mine {
                remember_other(memory, item, &line);
            }
            let shown = up1(&target_s);
            let shape = OTHER_KIND_SAY.get(kind.as_str()).cloned().unwrap_or_else(|| "{v} it is, then.".into());
            return dict(vec![("line", line.as_str().into()), ("note", fmt(&shape, &[("v", &shown), ("who", &who)]).into())]);
        }
    }
    Value::None
}

/// Python: bot/memory/corrected.py:186 `_make_corrects(target, memory)`: whether "make it ..." puts a
/// value where a value was.
pub fn _make_corrects(target: &Value, memory: &Memory) -> bool {
    if MADE_OF_A_VALUE.match_(pt::strip(&s(target))).is_some() {
        return true;
    }
    extract::extract(&last_user(memory), false).truthy()
}

/// Python: bot/memory/corrected.py:201 `_the_unit(text)`: the family of the unit a value is said
/// in, or "" when it names no unit.
pub fn _the_unit(text: &str) -> String {
    match A_UNIT_SAID.search(&lower(text)) {
        Some(m) => UNIT_FAMILY.get(m.str(1)).cloned().unwrap_or_default(),
        None => String::new(),
    }
}

/// Python: bot/memory/corrected.py:208 `_stated_families(before)`: the families of unit a line is
/// said in: the unit it names, and the facts it states.
pub fn _stated_families(before: &str) -> HashSet<String> {
    let mut families = HashSet::new();
    let unit = _the_unit(before);
    if !unit.is_empty() {
        families.insert(unit);
    }
    let found = std::panic::catch_unwind(|| extract::extract(before, false)).unwrap_or_else(|_| Value::dict());
    for (key, _) in found.pairs() {
        if let Some(family) = FACT_FAMILY.get(key.str()) {
            if !family.is_empty() {
                families.insert(family.clone());
            }
        }
    }
    families
}

/// Python: bot/memory/corrected.py:224 `_measured_the_same(value, before)`: true when the line before
/// is measured in the unit the new value is said in.
pub fn _measured_the_same(value: &Value, before: &str) -> bool {
    let family = _the_unit(&s(value));
    if family.is_empty() {
        return true;
    }
    let (day, month) = date_said(&lower(&s(value)));
    if day.is_some() || month.is_some() {
        return true;
    }
    _stated_families(before).contains(&family)
}

/// Python: bot/memory/corrected.py:247 `_without_the_naming_verb(value)`: "no its called max": the
/// name is the word after the verb (a str).
pub fn _without_the_naming_verb(value: &Value) -> Value {
    let said = pt::strip(&s(value)).to_string();
    let fixed = NAMING_VERB.sub("", &said);
    if fixed == said {
        return Value::from(said);
    }
    Value::from(up1(&fixed))
}

/// Python: bot/memory/corrected.py:264 `_the_plan_on_the_table(memory)`: the plan as the chat holds
/// it right now, or None.
pub fn _the_plan_on_the_table(memory: &Memory) -> Option<String> {
    let known = user_read(memory);
    if let Some(value) = known.get("plan").and_then(Value::as_str) {
        if !pt::strip(value).is_empty() {
            return Some(value.to_string());
        }
    }
    for value in vget(&known, "plans").items() {
        if let Some(v) = value.as_str() {
            if !pt::strip(v).is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Python: bot/memory/corrected.py:283 `_the_plan_before(memory, plan_after)`: the plan a correction
/// came from: the newest one in the log that is not the new one.
pub fn _the_plan_before(memory: &Memory, plan_after: &Value) -> Option<String> {
    let after = flat_low(&s(plan_after));
    for entry in mem_items(memory, "facts_log").iter().filter(|e| e.is_dict()).rev() {
        if !matches!(vget(entry, "key").as_str(), Some("plan") | Some("plans")) {
            continue;
        }
        let Some(value) = vget(entry, "value").as_str() else {
            continue;
        };
        if pt::strip(value).is_empty() {
            continue;
        }
        if flat_low(value) != after {
            return Some(value.to_string());
        }
    }
    None
}

/// Python: bot/memory/corrected.py:301 `_what_took_its_place(before, after, value)`: the words of
/// `after` that stand where `value` stood in `before`, or None.
pub fn _what_took_its_place(before: &str, after: &str, value: &str) -> Option<String> {
    let before: Vec<char> = flat_low(before).chars().collect();
    let after: Vec<char> = flat_low(after).chars().collect();
    let value: Vec<char> = flat_low(value).chars().collect();
    if value.is_empty() || before.is_empty() || after.is_empty() {
        return None;
    }
    let start = before.windows(value.len()).position(|w| w == value.as_slice())?;
    let end = start + value.len();
    let mut head = 0;
    while head < before.len().min(after.len()) && before[head] == after[head] {
        head += 1;
    }
    let mut tail = 0;
    while tail < before.len().min(after.len()) - head && before[before.len() - 1 - tail] == after[after.len() - 1 - tail] {
        tail += 1;
    }
    let (change_start, change_end) = (head, before.len() - tail);
    if change_end <= start || change_start >= end {
        return None;
    }
    let slice = |v: &[char], a: usize, b: usize| -> String {
        let b = b.min(v.len());
        if a >= b { String::new() } else { v[a..b].iter().collect() }
    };
    let kept_head = slice(&before, start, change_start.min(end));
    let kept_tail = if change_end < end { slice(&before, change_end.max(start), end) } else { String::new() };
    let new_middle = slice(&after, change_start, change_start.max(after.len() - tail));
    let fixed = pt::strip(&format!("{kept_head}{new_middle}{kept_tail}")).to_string();
    if fixed.is_empty() {
        None
    } else {
        Some(fixed)
    }
}

/// Python: bot/memory/corrected.py:338 `_a_day_written_the_way_the_store_writes_it(fact)`: the day
/// without the little words it was said with, when the fact is a day.
pub fn _a_day_written_the_way_the_store_writes_it(fact: &str) -> String {
    let text = pt::squash(fact);
    match A_DAY_SAID_LONG.match_(&text) {
        Some(found) => format!("{} {}", found.str(1), found.str(2)),
        None => text,
    }
}

/// Python: bot/memory/corrected.py:346 `_the_person_the_plan_is_about(memory, correction, before=None)`:
/// a fact kept against a person moves with the plan the person put right; how many moved.
pub fn _the_person_the_plan_is_about(memory: &mut Memory, correction: &Value, before: Option<&str>) -> i64 {
    let plan_after = if correction.is_dict() { vget(correction, "value").clone() } else { Value::None };
    let before = match before.filter(|b| !b.is_empty()) {
        Some(b) => b.to_string(),
        None => match _the_plan_before(memory, &plan_after) {
            Some(b) if !b.is_empty() => b,
            _ => return 0,
        },
    };
    let sentence = pt::strip(&s(vget(correction, "sentence"))).to_string();
    let after_s = s(&plan_after);
    let mut moved = 0;
    let n = others(memory).len();
    for i in 0..n {
        let entry = others(memory)[i].clone();
        if !entry.is_dict() {
            continue;
        }
        let who_v = [vget(&entry, "who"), vget(&entry, "pronoun"), vget(&entry, "name")].into_iter().find(|v| v.truthy()).cloned().unwrap_or(Value::None);
        let who = pt::strip(&s(&who_v)).to_string();
        let mentions = |line: &str| Re::new(&format!(r"\b{}\w*\b", esc(&lower(&who))), 0).is_match(line);
        if who.is_empty() || !mentions(&before) {
            continue;
        }
        if plan_after.truthy() && !mentions(&lower(&after_s)) {
            continue;
        }
        for (key, value) in entry.pairs().to_vec() {
            let key = key.str().to_string();
            if NOT_A_FACT_OF_THEIRS.contains(&key) {
                continue;
            }
            let said = value.as_str().unwrap_or("").to_string();
            let Some(taken) = _what_took_its_place(&before, &after_s, &said) else {
                continue;
            };
            if taken == flat_low(&said) {
                continue;
            }
            let taken = _a_day_written_the_way_the_store_writes_it(&taken);
            others(memory)[i].set_key(key.as_str(), taken.as_str());
            let current = others(memory)[i].clone();
            let (k, v) = if vget(&current, "who").truthy() {
                ("who", vget(&current, "who").clone())
            } else {
                ("pronoun", vget(&current, "pronoun").clone())
            };
            let kept = dict(vec![(k, v), (key.as_str(), taken.as_str().into())]);
            let turn = memory.turns.len() as i64;
            let text = if sentence.is_empty() { pt::squash(&after_s) } else { sentence.clone() };
            log(memory).push(dict(vec![("key", "other".into()), ("value", kept), ("turn", turn.into()), ("text", text.into())]));
            moved += 1;
        }
    }
    moved
}

/// Python: bot/memory/corrected.py:397 `_a_correction_is_not_a_plan(memory, text)`: a line that put
/// a value right is not also a plan of its own.
pub fn _a_correction_is_not_a_plan(memory: &mut Memory, text: &str) {
    let said = flat_low(text);
    if said.is_empty() {
        return;
    }
    let body = pt::squash(&extract::CORRECTION_LEAD_RE.sub("", &said));
    let known = user(memory);
    let Some(Value::List(plans)) = known.get("plans") else {
        return;
    };
    let kept: Vec<Value> = plans.iter().filter(|one| {
        let f = flat_low(&sv(one));
        f != said && f != body
    }).cloned().collect();
    if kept.len() != plans.len() {
        known.set_key("plans", Value::from(kept));
    }
}

/// Python: bot/memory/corrected.py:421 `_a_value_already_the_plans(memory, target, text="")`:
/// whether the value a line corrects to is already what the person's own plan says.
pub fn _a_value_already_the_plans(memory: &mut Memory, target: &Value, text: &str) -> bool {
    let said = lower(pt::strip(&s(target)));
    if said.is_empty() {
        return false;
    }
    let line = flat_low(text);
    let known = user(memory).clone();
    let plans = plans_and_latest(&known, false);
    let named = Re::new(&format!(r"\b{}\b", esc(&said)), 0);
    for one in plans {
        let held = flat_low(&one);
        if !line.is_empty() && (line.contains(&held) || held.contains(&line)) {
            continue;
        }
        if named.is_match(&held) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// the correction itself

/// Python: bot/memory/corrected.py:450 `_apply_correction(memory, correction, text="")`: "Sorry, I
/// meant Florence, not Rome" replaces Rome in the topic it corrects.
///
/// Python also writes "to" and "from" back into `correction`; no caller reads them afterwards,
/// so the contract's `&Value` is enough (`_apply_correction_mut` writes them for one that does).
pub fn _apply_correction(memory: &mut Memory, correction: &Value, text: &str) {
    let mut correction = correction.clone();
    _apply_correction_mut(memory, &mut correction, text);
}

static MEAN_NOTE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*\(mean [^)]*\)\s*$", I));
static NOT_ALNUM: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9]+", 0));

/// `_apply_correction(memory, correction, text)` with the correction dict written back.
pub fn _apply_correction_mut(memory: &mut Memory, correction: &mut Value, text: &str) {
    let is_dict = correction.is_dict();
    let mut target = if is_dict { vget(correction, "to").clone() } else { Value::None };
    if target.truthy() {
        target = _without_the_naming_verb(&target);
        correction.set_key("to", target.clone());
    }
    let mut wrong = if is_dict { vget(correction, "from").clone() } else { Value::None };
    if !target.truthy() {
        return;
    }
    let target_s = sv(&target);
    if is_dict && vget(correction, "applied").truthy() {
        return;
    }
    if _a_value_already_the_plans(memory, &target, text) {
        return;
    }
    if !wrong.truthy() && !_measured_the_same(&target, &last_user(memory)) {
        return;
    }
    if is_dict && vget(correction, "make").truthy() && !_make_corrects(&target, memory) {
        return;
    }
    let dash = |v: &Value| {
        let t = pt::strip(&s(v)).to_string();
        if t.is_empty() { "-".to_string() } else { t }
    };
    let both_numbers = extract::a_clock_time(&s(&target)).is_empty()
        && NUMBER_VALUE.match_(&dash(&target)).is_some()
        && NUMBER_VALUE.match_(&dash(&wrong)).is_some();
    let plan_on_the_table = _the_plan_on_the_table(memory);
    let wrong_or_empty = if wrong.truthy() { wrong.clone() } else { Value::from("") };
    let mut fixed = _correct_plan(memory, &target, &wrong, text);
    if !fixed.truthy() && !(both_numbers && _stated_plan(memory).is_some()) {
        let readers: [&dyn Fn(&mut Memory) -> Value; 8] = [
            &|m| _correct_date(m, &target, text, &wrong_or_empty),
            &|m| _correct_clock(m, &target, text, &wrong_or_empty),
            &|m| _correct_number(m, &target, text),
            &|m| _correct_count(m, &target, text),
            &|m| _correct_fact(m, &target, &wrong, text),
            &|m| _correct_a_persons_name(m, &target, text, &wrong_or_empty),
            &|m| _correct_when(m, &target, text),
            &|m| _correct_the_count_on_the_table(m, &target, text),
        ];
        for reader in readers {
            fixed = reader(memory);
            if fixed.truthy() {
                break;
            }
        }
    }
    if fixed.truthy() && vget(&fixed, "from").truthy() && !wrong.truthy() {
        wrong = vget(&fixed, "from").clone();
        correction.set_key("from", wrong.clone());
    }
    let said = flat_low(text);
    let mut other = Value::None;
    if !fixed.truthy() {
        other = _correct_other(memory, &target, text);
    }
    if other.truthy() {
        other.set_key("said", said.as_str());
        other.set_key("applied", true);
        memory.set("correction", other.clone());
    }
    if fixed.truthy() {
        if vget(&fixed, "key").as_str() == Some("plan") {
            _the_person_the_plan_is_about(memory, &fixed, plan_on_the_table.as_deref());
            _a_correction_is_not_a_plan(memory, text);
        }
        fixed.set_key("said", said.as_str());
        fixed.set_key("applied", true);
        memory.set("correction", fixed.clone());
    }
    if !fixed.truthy() && !wrong.truthy() {
        return;
    }
    let Some(record) = topics::current(memory).cloned() else {
        return;
    };
    let mut text_now = s(vget(&record, "text"));
    let fixed_from = s(vget(&fixed, "from"));
    let moved_off_the_topic = fixed.truthy()
        && vget(&fixed, "key").as_str() == Some("plan")
        && !fixed_from.is_empty()
        && !Re::new(&format!(r"\b{}\b", esc(&fixed_from)), I).is_match(&text_now);
    if moved_off_the_topic {
        let line = pt::squash(if said.is_empty() { &text_now } else { &said });
        if !line.is_empty() {
            let fresh = dict(vec![
                ("turn", (memory.turns.len() as i64).into()),
                ("text", line.as_str().into()),
                ("words", Value::from(topics::keywords(&line))),
            ]);
            memory.set("topic", fresh.clone());
            if !matches!(memory.get("topics"), Some(Value::List(_))) {
                memory.set("topics", Value::list());
            }
            let list = memory.get_mut("topics").unwrap();
            let last_text = list.items().last().map(|t| vget(t, "text").clone());
            if last_text.as_ref().map_or(true, |t| t.as_str() != Some(line.as_str())) {
                list.push(fresh);
            }
        }
        return;
    }
    let wrong_s = s(&wrong);
    if other.truthy() {
        let line = s(vget(&other, "line"));
        if !line.is_empty() {
            text_now = line;
        }
    } else if fixed.truthy() && vget(&fixed, "line").truthy() {
        text_now = s(vget(&fixed, "line"));
    } else if wrong.truthy() && Re::new(&format!(r"\b{}\b", esc(&wrong_s)), I).is_match(&text_now) {
        text_now = format!("{text_now} (mean {target_s})");
    } else if fixed.truthy() && vget(&fixed, "key").as_str() == Some("plan") && {
        let plan = s(vget(user(memory), "plan"));
        _the_topic_with_the_new_clock(&text_now, &plan).is_some()
    } {
        let plan = s(vget(user(memory), "plan"));
        text_now = _the_topic_with_the_new_clock(&text_now, &plan).unwrap_or_default();
    } else if !lower(&text_now).contains(&lower(&target_s)) {
        text_now = format!("{text_now} (mean {target_s})");
    }
    let mut new_record = match &record {
        Value::Dict(d) => Value::Dict(Arc::new(d.as_ref().clone())),
        other => other.clone(),
    };
    new_record.set_key("text", text_now.as_str());
    new_record.set_key("words", Value::from(topics::keywords(&text_now)));
    replace_object(memory, &record, &new_record);
    let record = new_record;
    let before = MEAN_NOTE.sub("", &text_now);
    let swapped = if wrong.truthy() { _swap_in(&Value::from(before.as_str()), &target, &wrong) } else { Value::None };
    let stated = if swapped.truthy() { s(&swapped) } else { before.clone() };
    let reading = if stated.is_empty() { text_now.clone() } else { stated.clone() };
    for entry in vget(&extract::extract(&reading, false), "others").items() {
        let mut entry = entry.clone();
        if !entry.pairs().iter().any(|(k, _)| !matches!(k.str(), "who" | "pronoun")) {
            continue;
        }
        let fresh = vget(&entry, "birthday").clone();
        let theirs = others(memory).iter().find(|one| *vget(one, "who") == *vget(&entry, "who")).cloned();
        let kept = flat_low(&s(vget(theirs.as_ref().unwrap_or(&Value::None), "birthday")));
        if fresh.truthy() && wrong.truthy() && !kept.is_empty() {
            let hated: Vec<String> = NOT_ALNUM
                .split(&lower(&sv(&wrong)))
                .into_iter()
                .filter(|p| !p.is_empty() && !["the", "of", "on", "in"].contains(&p.as_str()))
                .collect();
            if !hated.is_empty() && !hated.iter().any(|p| kept.contains(p.as_str())) {
                continue;
            }
        }
        if fresh.truthy() {
            let held_birthday = lower(&s(vget(theirs.as_ref().unwrap_or(&Value::None), "birthday")));
            let month = MONTHS_LONGEST_FIRST.iter().find(|name| held_birthday.contains(name.as_str())).cloned();
            let fresh_low = lower(&sv(&fresh));
            if let Some(month) = month {
                if !MONTHS_LONGEST_FIRST.iter().any(|name| fresh_low.contains(name.as_str())) {
                    entry.set_key("birthday", format!("{} {month}", sv(&fresh)));
                }
            }
        }
        remember_other(memory, &entry, &reading);
    }
    let places_held = user(memory).get("places").cloned();
    if let Some(Value::List(places)) = places_held.filter(|_| wrong.truthy() && extract::place_value(&target_s).truthy()) {
        let known = user(memory);
        {
            if !places.iter().any(|p| lower(&sv(p)) == lower(&target_s)) {
                let mut grown = places.as_ref().clone();
                grown.push(Value::from(target_s.as_str()));
                known.set_key("places", Value::from(grown));
            }
        }
    }
    let record_text = s(vget(&record, "text"));
    let prefix: String = record_text.chars().take(10).collect();
    if let Some(list) = memory.get_mut("topics").and_then(Value::items_mut) {
        if let Some(last) = list.last_mut() {
            if !same_object(last, &record) && s(vget(last, "text")).starts_with(&prefix) {
                *last = record.clone();
            }
        }
    }
    note(memory, &text_now);
}

/// Python: bot/memory/corrected.py:629 `_stated_plan(memory)`: the plan the line before this one
/// stated, or None.
pub fn _stated_plan(memory: &Memory) -> Option<String> {
    let said = flat_low(&last_user(memory));
    let entries = match memory.get("facts_log") {
        Some(Value::List(e)) => e.clone(),
        _ => return None,
    };
    if said.is_empty() {
        return None;
    }
    for entry in entries.iter().rev() {
        if !entry.is_dict() {
            continue;
        }
        if flat_low(&s(vget(entry, "text"))) != said {
            return None;
        }
        if matches!(vget(entry, "key").as_str(), Some("plan") | Some("plans")) {
            return vget(entry, "value").as_str().filter(|v| !pt::strip(v).is_empty()).map(str::to_string);
        }
        return None;
    }
    None
}

static AMOUNT_WORDS_RE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z0-9']+", 0));

/// Python: bot/memory/corrected.py:650 `_an_amount_on_the_measure(plan, target)`: the plan with the
/// amount they named now in it, or None.
pub fn _an_amount_on_the_measure(plan: &str, target: &Value) -> Value {
    let words = AMOUNT_WORDS_RE.findall_str(&lower(&sv(target)));
    if words.len() < 2 {
        return Value::None;
    }
    let measure = words.last().unwrap().clone();
    let stem = if measure.ends_with('s') && pt::len(&measure) > 3 { measure[..measure.len() - 1].to_string() } else { measure };
    if pt::len(&stem) < 3 {
        return Value::None;
    }
    let pattern = Re::new(&format!(r"(?<![\w])({}\s+)({}(?:s|es)?)(?![\w])", *AMOUNT_WORD, esc(&stem)), I);
    let Some(found) = pattern.search(plan) else {
        return Value::None;
    };
    let before = &plan[..found.start()];
    let mut said = pt::strip(&s(target)).to_string();
    if !pt::strip(before).is_empty() {
        said = format!("{}{}", lower(&pt::slice(&said, 0, Some(1))), pt::slice(&said, 1, None));
    }
    dict(vec![
        ("plan", format!("{before}{said}{}", &plan[found.end()..]).into()),
        ("value", pt::strip(&sv(target)).into()),
        ("in_the_sentence", said.as_str().into()),
        ("was", found.as_str().into()),
    ])
}

/// Python: bot/memory/corrected.py:684 `_correct_plan(memory, target, wrong, text="")`: a number
/// corrected inside a plan: "lose 5 kilos" / "i meant 8 kilos not 5".
pub fn _correct_plan(memory: &mut Memory, target: &Value, wrong: &Value, text: &str) -> Value {
    let _ = text;
    let plan = _stated_plan(memory);
    if let Some(plan) = plan.as_ref().filter(|_| !wrong.truthy()) {
        let amount = _an_amount_on_the_measure(plan, target);
        if amount.truthy() {
            let fixed = s(vget(&amount, "plan"));
            let shown = s(vget(&amount, "value"));
            user(memory).set_key("plan", fixed.as_str());
            replace_plan(memory, &fixed, |p| p.as_str() == Some(plan.as_str()));
            note(memory, &fixed);
            let mut origin = crate::memory::plan_line(memory, plan);
            if !origin.is_empty() {
                let was = s(vget(&amount, "was"));
                origin = Re::new(&format!(r"(?<![\w]){}(?![\w])", esc(&was)), I).subn(&s(vget(&amount, "in_the_sentence")), &origin, 1);
            }
            let sentence = crate::memory::plan_sentence(&fixed, &origin);
            return dict(vec![
                ("key", "plan".into()),
                ("value", shown.into()),
                ("sentence", if sentence.is_empty() { fixed.as_str().into() } else { sentence.into() }),
                ("line", if origin.is_empty() { Value::None } else { origin.into() }),
                ("quiet", true.into()),
            ]);
        }
    }
    let target_s = s(target);
    let right = NUMBER_VALUE.match_(pt::strip(&target_s));
    let (Some(plan), Some(right)) = (plan, right) else {
        return Value::None;
    };
    if !wrong.truthy() {
        return Value::None;
    }
    if !extract::a_clock_time(&target_s).is_empty() || !extract::a_clock_time(&s(wrong)).is_empty() {
        return Value::None;
    }
    let wrong_s = pt::strip(&sv(wrong)).to_string();
    if NUMBER_VALUE.match_(&wrong_s).is_none() {
        return Value::None;
    }
    let number = right.str(1).to_string();
    let unit = lower(right.str(2));
    let mut pattern = format!(r"(?<![\d.]){}", esc(&wrong_s));
    if !unit.is_empty() {
        pattern.push_str(&format!(r"(\s+{}\b|\s+(?!{}\b)[a-z]+\b)?", esc(&unit), *NOT_A_UNIT));
    }
    let found_value = if unit.is_empty() { number.clone() } else { format!("{number} {unit}") };
    let re = Re::new(&pattern, I);
    if re.search(&plan).is_none() {
        return Value::None;
    }
    let fixed = re.sub_with(&plan, 1, |_| found_value.clone());
    let mut shown = found_value.clone();
    if unit.is_empty() {
        if let Some(there) = Re::new(&format!(r"(?<![\d.]){}(\s+[a-z]+)?", esc(&number)), I).search(&fixed) {
            if let Some(g1) = there.get(1).filter(|g| !g.is_empty()) {
                shown = format!("{number}{g1}");
            }
        }
    }
    user(memory).set_key("plan", fixed.as_str());
    replace_plan(memory, &fixed, |p| p.as_str() == Some(plan.as_str()));
    note(memory, &fixed);
    dict(vec![("key", "plan".into()), ("value", shown.into()), ("wrong", wrong_s.into())])
}

/// Python: bot/memory/corrected.py:767 `_number_in(value)`: the digits of a value, so "£500" and
/// "500" are the same number.
pub fn _number_in(value: &Value) -> String {
    match NUMBER_DIGITS.search(&s(value)) {
        Some(m) => pt::strip(pt::lstrip_chars(m.as_str(), "£$€")).to_string(),
        None => String::new(),
    }
}

/// Python: bot/memory/corrected.py:772 `_correct_a_persons_name(memory, target, text="", wrong="")`:
/// a name given for somebody else is put right when the right name is said.
pub fn _correct_a_persons_name(memory: &mut Memory, target: &Value, text: &str, wrong: &Value) -> Value {
    let new = pt::squash(&s(target));
    if new.is_empty() || extract::NAME_WORD.match_(&new).is_none() || extract::NOT_A_NAME.contains(&lower(&new)) {
        return Value::None;
    }
    let taken_back = pt::squash(&lower(pt::strip(&s(wrong))));
    if !taken_back.is_empty() && taken_back != lower(&new) {
        let moved = _a_name_the_store_keeps(memory, &taken_back, &new);
        if !moved.is_none() {
            return moved;
        }
    }
    let said = last_user(memory);
    let named: Vec<Value> = vget(&extract::extract(&said, false), "others")
        .items()
        .iter()
        .filter(|e| e.is_dict() && vget(e, "who").truthy() && vget(e, "name").truthy())
        .cloned()
        .collect();
    if named.len() != 1 {
        return Value::None;
    }
    let who = sv(vget(&named[0], "who"));
    let old = sv(vget(&named[0], "name"));
    let was: Vec<String> = vget(&named[0], "was").items().iter().map(sv).collect();
    if lower(&old) == lower(&new) {
        return Value::None;
    }
    let taken_back = lower(pt::strip(&s(wrong)));
    let new_v = Value::from(new.as_str());
    if !_a_spelling_of(&new, &old)
        && taken_back != lower(&old)
        && !was.iter().any(|w| lower(w) == taken_back)
        && !(_a_new_name_was_said(&new_v, text) && _a_name_for_a_person(&new_v))
    {
        return Value::None;
    }
    let n = others(memory).len();
    for i in 0..n {
        let entry = others(memory)[i].clone();
        if entry.is_dict() && vget(&entry, "who").as_str() == Some(who.as_str()) && vget(&entry, "name").truthy() {
            let mut previous: Vec<String> = vget(&entry, "was").items().iter().map(sv).collect();
            previous.push(old.clone());
            let previous: Vec<Value> = previous.into_iter().filter(|o| lower(o) != lower(&new)).map(Value::from).collect();
            let slot = &mut others(memory)[i];
            slot.set_key("name", new.as_str());
            slot.set_key("was", Value::from(previous));
            return dict(vec![
                ("key", "person_name".into()),
                ("value", new.as_str().into()),
                ("wrong", up1(&old).into()),
                ("from", old.as_str().into()),
            ]);
        }
    }
    Value::None
}

/// Python: bot/memory/corrected.py:830 `_a_spelling_of(new, old)`: whether a word reads as another
/// go at the same name: the same first letter, and close.
pub fn _a_spelling_of(new: &str, old: &str) -> bool {
    let one = lower(pt::strip(new));
    let other = lower(pt::strip(old));
    if one.is_empty() || other.is_empty() || one.chars().next() != other.chars().next() {
        return false;
    }
    if extract::MONTHS.contains(&one) || extract::DAYS.contains(&one) {
        return false;
    }
    if one.starts_with(other.as_str()) || other.starts_with(one.as_str()) {
        return true;
    }
    _letters_apart(&one, &other) <= *SPELLING_APART
}

/// Python: bot/memory/corrected.py:846 `_letters_apart(one, other)`: how many single letters have to
/// be added, dropped or changed to turn one into other.
pub fn _letters_apart(one: &str, other: &str) -> i64 {
    let (mut one, mut other): (Vec<char>, Vec<char>) = (one.chars().collect(), other.chars().collect());
    if one.len() < other.len() {
        std::mem::swap(&mut one, &mut other);
    }
    let mut before: Vec<i64> = (0..=other.len() as i64).collect();
    for (i, letter) in one.iter().enumerate() {
        let mut after = vec![i as i64 + 1];
        for (j, other_letter) in other.iter().enumerate() {
            let j = j + 1;
            let v = (before[j] + 1).min(after[j - 1] + 1).min(before[j - 1] + (letter != other_letter) as i64);
            after.push(v);
        }
        before = after;
    }
    *before.last().unwrap()
}

/// `_clock_key(extract.the_clock_in(plan))`: Python hands the match object itself to
/// `_clock_key`, which reads `str(match)` ("<re.Match object; span=(..), match='10am'>"), so the
/// key is the match's repr with everything but letters and figures taken out - never equal to a
/// clock's own key - and "" when there is no clock.
fn clock_key_of_found(plan: &str, found: Option<&Match>) -> String {
    match found {
        None => String::new(),
        Some(m) => {
            let (a, b) = (pt::char_index(plan, m.start()), pt::char_index(plan, m.end()));
            _clock_key(&format!("<re.Match object; span=({a}, {b}), match='{}'>", m.as_str()))
        }
    }
}

/// Python: bot/memory/corrected.py:859 `_correct_clock(memory, target, text="", wrong="")`: a clock
/// inside a plan is put right: "the meeting is at 10am" then "make it 2pm".
pub fn _correct_clock(memory: &mut Memory, target: &Value, text: &str, wrong: &Value) -> Value {
    let mut new = extract::a_clock_time(&s(target));
    let bare = if new.is_empty() { _a_bare_hour(target) } else { String::new() };
    if new.is_empty() && bare.is_empty() {
        return Value::None;
    }
    let known = user(memory).clone();
    let mut plans = plans_and_latest(&known, true);
    if wrong.truthy() {
        let wanted = _clock_key(&s(wrong));
        let named: Vec<String> = plans.iter().filter(|p| clock_key_of_found(p, extract::the_clock_in(p).as_ref()) == wanted).cloned().collect();
        if !named.is_empty() {
            plans = named;
        }
    }
    for plan in plans.iter().rev() {
        let Some(found) = extract::the_clock_in(plan) else {
            continue;
        };
        let old = pt::squash(found.as_str());
        if new.is_empty() {
            if _names_a_half_of_the_day(&old) {
                continue;
            }
            new = bare.clone();
        }
        if _clock_key(&old) == _clock_key(&new) {
            return Value::None;
        }
        let fixed = pt::strip(&format!("{}{}{}", &plan[..found.start()], new, &plan[found.end()..])).to_string();
        remember(memory, "plan", Value::from(fixed.as_str()), text);
        replace_plan(memory, &fixed, |p| sv(p) == *plan);
        return dict(vec![("key", "plan".into()), ("value", fixed.into()), ("from", old.into())]);
    }
    Value::None
}

const CLOCK_SMALL_WORDS: [&str; 11] = ["my", "your", "our", "the", "a", "an", "im", "i", "you", "is", "are"];

/// Python: bot/memory/corrected.py:901 `_the_topic_with_the_new_clock(text, plan_now)`: the topic's
/// own line with the plan's new clock in it, or None when the two are not the same line.
pub fn _the_topic_with_the_new_clock(text: &str, plan_now: &str) -> Option<String> {
    let said = flat_low(text);
    let plan = flat_low(plan_now);
    if said.is_empty() || plan.is_empty() {
        return None;
    }
    let was = extract::the_clock_in(&said)?;
    let now = extract::the_clock_in(&plan)?;
    if _clock_key(was.as_str()) == _clock_key(now.as_str()) {
        return None;
    }
    let words = |line: &str, found: &Match| -> Vec<String> {
        let rest = pt::squash(&format!("{}{}", &line[..found.start()], &line[found.end()..]));
        pt::split(&rest).into_iter().filter(|w| !CLOCK_SMALL_WORDS.contains(&w.as_str()) && w != "its" && w != "at").collect()
    };
    if words(&said, &was) != words(&plan, &now) {
        return None;
    }
    // `text[:was.start()] + now.group(0) + text[was.end():]`: the positions are the flattened
    // line's (in characters), applied to the line as it is.
    let a = pt::char_index(&said, was.start()) as isize;
    let b = pt::char_index(&said, was.end()) as isize;
    let fixed = format!("{}{}{}", pt::slice(text, 0, Some(a)), now.as_str(), pt::slice(text, b, None));
    Some(pt::strip(&fixed).to_string())
}

/// Python: bot/memory/corrected.py:933 `_a_bare_hour(value)`: the hour a bare figure could be, or "".
pub fn _a_bare_hour(value: &Value) -> String {
    match BARE_HOUR.match_(pt::strip(&s(value))) {
        Some(m) => {
            let h = m.str(1).parse::<i64>().unwrap_or(0);
            if (1..=24).contains(&h) { m.str(1).to_string() } else { String::new() }
        }
        None => String::new(),
    }
}

static HALF_OF_THE_DAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"am|pm|o'?clock|noon|midday|midnight", I));
static NOT_A_CLOCK_CHAR: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^0-9a-z]", 0));

/// Python: bot/memory/corrected.py:940 `_names_a_half_of_the_day(clock)`.
pub fn _names_a_half_of_the_day(clock: &str) -> bool {
    HALF_OF_THE_DAY.is_match(clock)
}

/// Python: bot/memory/corrected.py:944 `_clock_key(clock)`: a clock with everything but its figures
/// and letters taken out, so 10am is 10am.
pub fn _clock_key(clock: &str) -> String {
    NOT_A_CLOCK_CHAR.sub("", &lower(clock))
}

static ALL_NUMBERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d+(?:[.,]\d+)?", 0));
static HAS_A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));

/// Python: bot/memory/corrected.py:948 `_correct_number(memory, target, text="")`: a number moves
/// whatever the line before it was about.
pub fn _correct_number(memory: &mut Memory, target: &Value, text: &str) -> Value {
    let new = pt::squash(&s(target));
    let digits = _number_in(&Value::from(new.as_str()));
    if digits.is_empty() {
        return Value::None;
    }
    if _names_a_date(&Value::from(new.as_str())) {
        return Value::None;
    }
    let said = pt::squash(&last_user(memory));
    if said.is_empty() || _a_birthday_on_the_last_line(memory) {
        return Value::None;
    }
    let olds: Vec<String> = ALL_NUMBERS.findall_str(&said).into_iter().filter(|n| *n != digits).collect();
    if olds.is_empty() {
        return Value::None;
    }
    for old in &olds {
        for key in ["budget", "age", "height", "weight"] {
            let value = user(memory).get(key).cloned().unwrap_or(Value::None);
            if !value.is_none() && _number_in(&value) == *old {
                let same = match &value {
                    Value::Str(v) if HAS_A_DIGIT.is_match(v) => Value::from(ALL_NUMBERS.subn(&digits, v, 1)),
                    Value::Int(_) | Value::Bool(_) => Value::Int(digits.parse::<i64>().unwrap_or_else(|_| panic!("ValueError: invalid literal for int(): {digits}"))),
                    _ => Value::from(digits.as_str()),
                };
                remember(memory, key, same.clone(), text);
                let kept = user(memory).get(key).cloned().unwrap_or(Value::None);
                let shown = if kept.is_none() || kept.as_str() == Some("") { same } else { kept };
                return dict(vec![("key", key.into()), ("value", shown), ("from", old.as_str().into())]);
            }
        }
        for key in ["plan", "job"] {
            let Some(value) = user(memory).get(key).and_then(Value::as_str).map(str::to_string) else {
                continue;
            };
            if !(value.contains(old.as_str()) && !value.contains(digits.as_str())) {
                continue;
            }
            let fixed = Re::new(&format!(r"\b{old}\b"), 0).subn(&digits, &value, 1);
            if fixed == value {
                continue;
            }
            let mut told = text.to_string();
            if key == "plan" {
                let origin = crate::memory::plan_line(memory, &value);
                let swapped = if origin.is_empty() {
                    String::new()
                } else {
                    s(&_swap_in(&Value::from(origin.as_str()), &Value::from(digits.as_str()), &Value::from(old.as_str())))
                };
                if !swapped.is_empty() {
                    told = swapped;
                }
            }
            remember(memory, key, Value::from(fixed.as_str()), &told);
            if key == "plan" {
                replace_plan(memory, &fixed, |p| lower(&sv(p)) == lower(&value));
                return dict(vec![("key", "plan".into()), ("value", fixed.into()), ("sentence", told.into()), ("from", old.as_str().into())]);
            }
            let lowered = format!("{}{}", lower(&pt::slice(&fixed, 0, Some(1))), pt::slice(&fixed, 1, None));
            return dict(vec![("key", "line".into()), ("value", lowered.into()), ("from", old.as_str().into())]);
        }
    }
    Value::None
}

/// Python: bot/memory/corrected.py:1031 `_count_said(text)`: the number a line says, as a number.
pub fn _count_said(text: &str) -> i64 {
    let said = flat_low(text);
    let Some(m) = COUNT_SAID.search(&said) else {
        return 0;
    };
    let word = lower(m.str(1));
    if pt::is_digit(&word) {
        return word.parse::<i64>().unwrap_or(0);
    }
    extract::PET_COUNT_WORDS.get(&word).copied().unwrap_or(0)
}

/// Python: bot/memory/corrected.py:1041 `_a_count(count, animal)`: a count of an animal said back as
/// a person says it: "three cats", "two fish".
pub fn _a_count(count: i64, animal: &Value) -> String {
    let animal = lower(pt::strip(&{
        let a = s(animal);
        if a.is_empty() { "pet".to_string() } else { a }
    }));
    let many = if animal.ends_with("fish") || animal.ends_with("sheep") || animal.ends_with("deer") || count == 1 {
        animal
    } else {
        format!("{animal}s")
    };
    format!("{count} {many}")
}

/// `_animal_family(pet.get("animal"))`.
fn family(animal: &Value) -> String {
    crate::memory::relations::_animal_family(&s(animal))
}

/// The dict pets of the user, with their places in `user["pets"]`.
fn user_pets(memory: &mut Memory) -> Vec<(usize, Value)> {
    vget(user(memory), "pets").items().iter().enumerate().filter(|(_, p)| p.is_dict()).map(|(i, p)| (i, p.clone())).collect()
}

/// `pet[key] = value` on the user's pet at `index`.
fn set_pet(memory: &mut Memory, index: usize, key: &str, value: Value) {
    if let Some(list) = user(memory).get_mut("pets").and_then(Value::items_mut) {
        list[index].set_key(key, value);
    }
}

/// Python: bot/memory/corrected.py:1050 `_correct_count(memory, target, text="")`: a count of an
/// animal already on the store is put right by saying the new count.
pub fn _correct_count(memory: &mut Memory, target: &Value, text: &str) -> Value {
    let _ = text;
    let new = _count_said(&s(target));
    if !(1..=30).contains(&new) {
        return Value::None;
    }
    let held = user_pets(memory);
    if held.is_empty() {
        return Value::None;
    }
    let said = flat_low(&last_user(memory));
    let animals_of = |line: &str| -> Vec<String> { vget(&extract::extract(line, false), "pets").items().iter().map(|p| s(vget(p, "animal"))).collect() };
    let mut named = animals_of(&s(target));
    if named.is_empty() {
        named = animals_of(&said);
    }
    let named: Vec<String> = named.into_iter().filter(|a| !a.is_empty()).collect();
    if named.is_empty() {
        return Value::None;
    }
    let told: Vec<Value> = vget(&extract::extract(&said, false), "pets").items().iter().filter(|p| vget(p, "count").truthy()).cloned().collect();
    let old = told.first().map(|t| vget(t, "count").as_int().unwrap_or(0)).unwrap_or(0);
    if old == 0 || old == new {
        return Value::None;
    }
    for (index, pet) in held.iter().rev() {
        if family(vget(pet, "animal")) != crate::memory::relations::_animal_family(&named[0]) {
            continue;
        }
        set_pet(memory, *index, "count", Value::Int(new));
        let animal = vget(pet, "animal").clone();
        return dict(vec![
            ("key", "pets_count".into()),
            ("value", _a_count(new, &animal).into()),
            ("from", _a_count(old, &animal).into()),
        ]);
    }
    Value::None
}

/// Python: bot/memory/corrected.py:1104 `_correct_the_count_on_the_table(memory, target, text="")`:
/// put a corrected count on what the line before it counted.
pub fn _correct_the_count_on_the_table(memory: &mut Memory, target: &Value, text: &str) -> Value {
    let _ = target;
    let typed = pt::squash(text);
    let Some(line) = BARE_NEW_COUNT.match_(&typed) else {
        return Value::None;
    };
    let new = _count_said(line.name_str("count"));
    if !(0 < new && new <= 30) {
        return Value::None;
    }
    let said = pt::squash(&last_user(memory));
    if said.is_empty() || ANY_NUMBER.findall(&said).len() > 1 {
        return Value::None;
    }
    let counted = vget(&extract::extract(&said, false), "kids").clone();
    let held = vget(user(memory), "kids").clone();
    let (Value::Int(counted), Value::Int(old)) = (counted, held) else {
        return Value::None;
    };
    if counted == new {
        return Value::None;
    }
    remember(memory, "kids", Value::Int(new), text);
    let kid = Value::from("kid");
    dict(vec![("key", "kids".into()), ("value", _a_count(new, &kid).into()), ("from", _a_count(old, &kid).into())])
}

/// Python: bot/memory/corrected.py:1145 `_a_fact_of_somebody_else(memory, plan)`: whether a plan is a
/// date kept for another person ("mums birthday on 2 april"), not a plan.
pub fn _a_fact_of_somebody_else(memory: &Memory, plan: &str) -> bool {
    let said = flat_low(plan);
    if said.is_empty() {
        return false;
    }
    for entry in mem_items(memory, "others") {
        let who = lower(pt::strip(&s(vget(&entry, "who"))));
        if who.is_empty() {
            continue;
        }
        let word = pt::split(&who)[0].clone();
        if Re::new(&format!(r"\b{}(?:'?s)?\b", esc(&word)), 0).is_match(&said) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/corrected.py:1165 `_the_plans_a_journey_moves(memory, plans, text)`: the plans a
/// line about a journey can move, or None when the line names no journey.
pub fn _the_plans_a_journey_moves(memory: &Memory, plans: &[String], text: &str) -> Option<Vec<String>> {
    if !A_JOURNEY_GOING.is_match(text) {
        return None;
    }
    Some(plans.iter().filter(|p| !_a_fact_of_somebody_else(memory, p)).cloned().collect())
}

static LETTER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]+", 0));

/// Python: bot/memory/corrected.py:1176 `_correct_when(memory, target, text="")`: a month or a day
/// lives inside the plan the line before it stated.
pub fn _correct_when(memory: &mut Memory, target: &Value, text: &str) -> Value {
    let target_s = sv(target);
    let low = lower(&target_s);
    if !extract::MONTHS.contains(&low) && !extract::DAYS.contains(&low) {
        return Value::None;
    }
    let known = user(memory).clone();
    let mut plans = plans_and_latest(&known, true);
    let in_months = extract::MONTHS.contains(&low);
    if let Some(journeys) = _the_plans_a_journey_moves(memory, &plans, text) {
        if journeys.is_empty() {
            return Value::None;
        }
        plans = journeys;
    }
    for plan in plans.iter().rev() {
        for word in LETTER_WORDS.findall_str(&lower(plan)) {
            let in_group = if in_months { extract::MONTHS.contains(&word) } else { extract::DAYS.contains(&word) };
            if in_group && word != low {
                let fixed = Re::new(&format!(r"\b{word}\b"), I).subn(&target_s, plan, 1);
                let told = s(&_plan_told_line(memory, &Value::from(plan.as_str()), &Value::from(fixed.as_str()), text));
                remember(memory, "plan", Value::from(fixed.as_str()), &told);
                replace_plan(memory, &fixed, |p| lower(&sv(p)) == lower(plan));
                let mut read_back = crate::memory::plan_sentence(&fixed, &told);
                if lower(&read_back).starts_with("you're planning ") {
                    read_back = String::new();
                }
                return dict(vec![
                    ("key", "plan".into()),
                    ("value", pt::rstrip_chars(&fixed, " .").into()),
                    ("from", up1(&word).into()),
                    ("sentence", read_back.into()),
                ]);
            }
        }
    }
    Value::None
}

static MONTH_NAME_RES: LazyLock<Vec<(String, Re)>> =
    LazyLock::new(|| MONTHS_LONGEST_FIRST.iter().map(|n| (n.clone(), Re::new(&format!(r"\b{n}\b"), 0))).collect());

/// `_date_said(low)` as (day, month).
fn date_said(low: &str) -> (Option<String>, Option<String>) {
    let day = DATE_DAY.search(low);
    let mut month: Option<String> = None;
    for (name, re) in MONTH_NAME_RES.iter() {
        if re.is_match(low) {
            month = Some(name.clone());
            break;
        }
    }
    if day.is_none() && month.is_some() {
        if let Some(bare) = A_DAY_BEFORE_A_MONTH.search(low) {
            return (Some(bare.str(1).to_string()), month);
        }
    }
    (day.map(|d| d.as_str().to_string()), month)
}

/// Python: bot/memory/corrected.py:1229 `_date_said(low)`: the day and the month a value states,
/// either of them None (a pair).
pub fn _date_said(low: &str) -> Value {
    let (day, month) = date_said(low);
    Value::Tuple(Arc::new(vec![Value::from(day), Value::from(month)]))
}

static ONE_OR_TWO_DIGITS: LazyLock<Re> = LazyLock::new(|| Re::new(r"(\d{1,2})", 0));
static AN_ORDINAL_DAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d{1,2}(?:st|nd|rd|th)$", I));

/// Python: bot/memory/corrected.py:1245 `_a_day_in_the_kind(old_day, day)`: a day written the way the
/// plan already writes its days.
pub fn _a_day_in_the_kind(old_day: &str, day: &str) -> String {
    let Some(number) = ONE_OR_TWO_DIGITS.match_(day) else {
        return s(&_date_word_case(&Value::from(day)));
    };
    if AN_ORDINAL_DAY.match_(old_day).is_some() {
        let count = number.str(1).parse::<i64>().unwrap_or(0);
        let suffix = if (11..=13).contains(&(count % 100)) {
            "th"
        } else {
            match count % 10 {
                1 => "st",
                2 => "nd",
                3 => "rd",
                _ => "th",
            }
        };
        return format!("{}{suffix}", number.str(1));
    }
    number.str(1).to_string()
}

/// Python: bot/memory/corrected.py:1263 `_date_word_case(value)`: "july" is written "July", and a day
/// said as "13th" is left as it is said (a str).
pub fn _date_word_case(value: &Value) -> Value {
    let text = s(value);
    if text.chars().next().is_some_and(char::is_alphabetic) {
        Value::from(up1(&text))
    } else {
        Value::from(text)
    }
}

fn date_word_case(text: &str) -> String {
    s(&_date_word_case(&Value::from(text)))
}

/// Python: bot/memory/corrected.py:1268 `_date_in_plan(plan)`: the day and the month the plan already
/// says, either of them None.
pub fn _date_in_plan(plan: &str) -> (Option<String>, Option<String>) {
    date_said(&lower(plan))
}

/// Python: bot/memory/corrected.py:1276 `_the_weekday_in(plan)`: the weekday a plan says its day as,
/// or None.
pub fn _the_weekday_in(plan: &str) -> Option<Match<'_>> {
    WEEKDAY_IN_PLAN.search(plan)
}

/// Python: bot/memory/corrected.py:1280 `_names_a_date(value)`: true when a value states a date.
pub fn _names_a_date(value: &Value) -> bool {
    let (day, month) = date_said(&lower(&s(value)));
    day.is_some() || month.is_some()
}

/// Python: bot/memory/corrected.py:1288 `_a_relative_month(plan, day, month)`: the plan with a
/// relative month replaced by the month they named, what moved, and what was said.
pub fn _a_relative_month(plan: &str, day: Option<&str>, month: Option<&str>) -> Option<(String, String, String)> {
    let month = month.filter(|m| !m.is_empty())?;
    let found = RELATIVE_MONTH.search(plan)?;
    let mut said = date_word_case(month);
    match day.filter(|d| !d.is_empty()) {
        Some(day) => said = format!("on the {day} of {said}"),
        None => said = format!("in {said}"),
    }
    let fixed = pt::strip(&format!("{}{}{}", &plan[..found.start()], said, &plan[found.end()..])).to_string();
    let fixed = pt::squash(&fixed.replace(" ,", ","));
    Some((fixed, lower(found.as_str()), said))
}

/// Python: bot/memory/corrected.py:1310 `_a_relative_line(memory, plan, was, said)`: the line the plan
/// was stated in, with the relative date said as the date they mean.
pub fn _a_relative_line(memory: &mut Memory, plan: &str, was: &str, said: &str) -> Option<String> {
    let told = crate::memory::plan_line(memory, plan);
    if told.is_empty() || was.is_empty() || said.is_empty() {
        return None;
    }
    Some(Re::new(&format!(r"\b{}\b", esc(was)), I).subn(said, &told, 1))
}

/// Python: bot/memory/corrected.py:1321 `_plan_saying(memory, day, month)`: the plan in the store that
/// says the date being taken back, or None.
pub fn _plan_saying(memory: &Memory, day: Option<&str>, month: Option<&str>) -> Option<String> {
    if day.is_none() && month.is_none() {
        return None;
    }
    let plans: Vec<String> = vget(&user_read(memory), "plans").items().iter().filter_map(|p| p.as_str().map(str::to_string)).collect();
    for plan in plans.iter().rev() {
        let (found_day, found_month) = _date_in_plan(plan);
        if let Some(m) = month {
            if found_month.as_deref() != Some(m) {
                continue;
            }
        }
        if let (Some(d), Some(fd)) = (day, found_day.as_deref()) {
            if fd != d {
                continue;
            }
        }
        return Some(plan.clone());
    }
    None
}

/// Python: bot/memory/corrected.py:1340 `_correct_date(memory, target, text="", wrong="")`: a date
/// said again moves the date in the plan it belongs to, part by part.
pub fn _correct_date(memory: &mut Memory, target: &Value, text: &str, wrong: &Value) -> Value {
    let low = lower(&s(target));
    let (day, month) = date_said(&low);
    if day.is_none() && month.is_none() {
        return Value::None;
    }
    if _a_birthday_on_the_last_line(memory) {
        return Value::None;
    }
    let mut plan: Value = match _stated_plan(memory) {
        Some(p) => Value::from(p),
        None => vget(user(memory), "plan").clone(),
    };
    let (gone_day, gone_month) = date_said(&lower(&s(wrong)));
    if let Some(saying) = _plan_saying(memory, gone_day.as_deref(), gone_month.as_deref()) {
        plan = Value::from(saying);
    }
    let Some(plan) = plan.as_str().filter(|p| !pt::strip(p).is_empty()).map(str::to_string) else {
        return Value::None;
    };
    let (old_day, old_month) = _date_in_plan(&plan);
    let mut fixed = plan.clone();
    let mut moved: Vec<String> = Vec::new();
    let mut told: Option<String> = None;
    if old_day.is_none() && old_month.is_none() {
        let weekday = _the_weekday_in(&plan);
        let (was, said);
        if let (Some(d), Some(m), Some(w)) = (&day, &month, &weekday) {
            said = format!("the {} of {}", date_word_case(d), date_word_case(m));
            fixed = pt::squash(&format!("{}{}{}", &plan[..w.start()], said, &plan[w.end()..]));
            was = lower(w.as_str());
            moved.push(was.clone());
        } else {
            let Some((f, w, sd)) = _a_relative_month(&plan, day.as_deref(), month.as_deref()) else {
                return Value::None;
            };
            fixed = f;
            was = w;
            said = sd;
            moved.push(was.clone());
        }
        told = _a_relative_line(memory, &plan, &was, &said);
    }
    if let (Some(d), None, Some(om)) = (&day, &old_day, &old_month) {
        let with_a_day = Re::new(&format!(r"\b(in|on|for|by|until|till|from|around|before|after)\s+(?:the\s+)?{}\b", esc(om)), I);
        if let Some(found) = with_a_day.search(&fixed) {
            let join = if lower(found.str(1)) == "in" { "on".to_string() } else { lower(found.str(1)) };
            fixed = format!("{}{join} the {} of {}{}", &fixed[..found.start()], date_word_case(d), date_word_case(om), &fixed[found.end()..]);
            moved.push(om.clone());
        }
    }
    if let (Some(d), Some(od)) = (&day, &old_day) {
        if lower(d) != lower(od) {
            fixed = Re::new(&format!(r"\b{od}\b"), I).subn(&_a_day_in_the_kind(od, d), &fixed, 1);
            moved.push(od.clone());
        }
    }
    if let (Some(m), Some(om)) = (&month, &old_month) {
        if m != om {
            fixed = Re::new(&format!(r"\b{om}\b"), I).subn(&date_word_case(m), &fixed, 1);
            moved.push(om.clone());
        }
    }
    if fixed == plan {
        return Value::None;
    }
    let told = match told {
        Some(t) => t,
        None => s(&_plan_told_line(memory, &Value::from(plan.as_str()), &Value::from(fixed.as_str()), text)),
    };
    remember(memory, "plan", Value::from(fixed.as_str()), &told);
    replace_plan(memory, &fixed, |p| lower(&sv(p)) == lower(&plan));
    let mut read_back = crate::memory::plan_sentence(&fixed, &told);
    if lower(&read_back).starts_with("you're planning ") {
        read_back = String::new();
    }
    dict(vec![
        ("key", "plan".into()),
        ("value", pt::rstrip_chars(&fixed, " .").into()),
        ("from", moved.join(" ").into()),
        ("sentence", read_back.into()),
    ])
}

/// Python: bot/memory/corrected.py:1451 `_a_name_the_store_keeps(memory, taken_back, new)`: the
/// correction that moves the name of the one person the store keeps under it.
pub fn _a_name_the_store_keeps(memory: &mut Memory, taken_back: &str, new: &str) -> Value {
    let n = others(memory).len();
    for i in 0..n {
        let entry = others(memory)[i].clone();
        if !entry.is_dict() || !vget(&entry, "name").truthy() {
            continue;
        }
        if lower(pt::strip(&sv(vget(&entry, "name")))) != taken_back {
            continue;
        }
        if !_a_name_for_a_person(&Value::from(new)) {
            return Value::None;
        }
        let before = s(vget(&entry, "name"));
        let slot = &mut others(memory)[i];
        slot.set_key("name", new);
        if !before.is_empty() && lower(&before) != lower(new) {
            let mut was: Vec<Value> = vget(&entry, "was").items().iter().map(sv).filter(|o| lower(o) != lower(new)).map(Value::from).collect();
            was.push(Value::from(before.as_str()));
            slot.set_key("was", Value::from(was));
        }
        slot.del_key("turns");
        let kept = slot.clone();
        let who = sv(vget(&kept, "who"));
        let turn = memory.turns.len() as i64;
        log(memory).push(dict(vec![
            ("key", "other".into()),
            ("value", kept),
            ("turn", turn.into()),
            ("text", format!("my {who} is called {new}").into()),
        ]));
        return dict(vec![
            ("key", "person_name".into()),
            ("value", new.into()),
            ("wrong", up1(taken_back).into()),
            ("from", taken_back.into()),
        ]);
    }
    Value::None
}

/// Python: bot/memory/corrected.py:1478 `_stated_last(memory)`: the key of the fact the correction
/// fixes, or None.
pub fn _stated_last(memory: &Memory) -> Option<String> {
    let entries = match memory.get("facts_log") {
        Some(Value::List(e)) => e.clone(),
        _ => return None,
    };
    let mut by_line: Vec<(String, Vec<Value>)> = Vec::new();
    for entry in entries.iter().filter(|e| e.is_dict()) {
        let said = flat_low(&s(vget(entry, "text")));
        let key = vget(entry, "key").clone();
        match by_line.iter_mut().find(|(k, _)| *k == said) {
            Some(slot) => slot.1.push(key),
            None => by_line.push((said, vec![key])),
        }
    }
    let lines = crate::memory::last_user(memory, *CORRECTION_BACK);
    for line in lines.items().iter().rev() {
        let said = flat_low(&s(line));
        if said.is_empty() {
            continue;
        }
        if let Some((_, keys)) = by_line.iter().find(|(k, _)| *k == said).filter(|(_, keys)| !keys.is_empty()) {
            for key in keys.iter().rev() {
                if let Some(k) = key.as_str() {
                    if CORRECTABLE.contains(k) {
                        return Some(k.to_string());
                    }
                }
            }
            if !_asks(&said) {
                return None;
            }
            continue;
        }
        if !_asks(&said) {
            return None;
        }
    }
    None
}

/// Python: bot/memory/corrected.py:1547 `_plan_told_line(memory, old, new, text="")`: the line to keep
/// with a plan whose date was changed (a str).
pub fn _plan_told_line(memory: &mut Memory, old: &Value, new: &Value, text: &str) -> Value {
    let told = crate::memory::plan_line(memory, &s(old));
    let was = s(&_date_word(&s(old)));
    let now = s(&_date_word(&s(new)));
    if told.is_empty() || was.is_empty() || now.is_empty() || lower(&was) == lower(&now) {
        return Value::from(text);
    }
    Value::from(Re::new(&format!(r"\b{}\b", esc(&was)), I).subn(&now, &told, 1))
}

/// Python: bot/memory/corrected.py:1562 `_date_word(phrase)`: the month or day a phrase names, as it
/// was written there, or "" (a str).
pub fn _date_word(phrase: &str) -> Value {
    match MONTH_WORD.search(phrase).or_else(|| DAY_WORD.search(phrase)) {
        Some(m) => Value::from(m.as_str()),
        None => Value::from(""),
    }
}

/// Python: bot/memory/corrected.py:1567 `_kind_of_value(target, told)`: what part of the fact's
/// sentence a corrected value is a value of.
pub fn _kind_of_value(target: &Value, told: &Value) -> Option<String> {
    let target = sv(target);
    let told = sv(told);
    let low = lower(&target);
    if MONTHS.contains(&low) {
        return Some("month".into());
    }
    if extract::DAYS.contains(&low) {
        return Some("day".into());
    }
    if AMOUNT.fullmatch(&target).is_some() && !AT_TIME.is_match(&told) {
        return Some("amount".into());
    }
    if AT_TIME.is_match(&told) && (AMOUNT.fullmatch(&target).is_some() || NUMBER_WORD.fullmatch(&target).is_some()) {
        return Some("time".into());
    }
    if DATE_PHRASE.is_match(&target) {
        return Some("date".into());
    }
    None
}

static MONEY_WORD_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"^\s*(?:pounds?|dollars?|euros?|quid|bucks?)\b", I));
static HALF_A_DAY: LazyLock<Re> = LazyLock::new(|| Re::new(r"(am|pm|o'?clock)\b", I));
static AT_TIME_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"^at\s+{}", *AT_TIME_VALUE), I));

/// Python: bot/memory/corrected.py:1590 `_swap_in(told, target, wrong=None)`: the fact's own sentence
/// with the corrected value in place of the one it corrects (a str), or None.
pub fn _swap_in(told: &Value, target: &Value, wrong: &Value) -> Value {
    let Some(told) = told.as_str().filter(|t| !pt::strip(t).is_empty()) else {
        return Value::None;
    };
    if !target.truthy() {
        return Value::None;
    }
    let mut target = pt::strip(&sv(target)).to_string();
    let kind = _kind_of_value(&Value::from(target.as_str()), &Value::from(told));
    fn last_of<'t>(re: &'t Re, told: &'t str) -> Option<Match<'t>> {
        re.finditer(told).into_iter().last()
    }
    match kind.as_deref() {
        Some("month") | Some("day") => {
            let wanted: &Re = if kind.as_deref() == Some("month") { *MONTH_WORD } else { *DAY_WORD };
            let Some(last) = last_of(wanted, told) else {
                return Value::None;
            };
            Value::from(format!("{}{}{}", &told[..last.start()], target, &told[last.end()..]))
        }
        Some("amount") => {
            let Some(last) = last_of(*AMOUNT, told) else {
                return Value::None;
            };
            let unit = CURRENCY.match_(&told[last.start()..]);
            let mut tail = told[last.end()..].to_string();
            if let Some(unit) = unit.as_ref().filter(|_| CURRENCY.match_(&target).is_none()) {
                let u = unit.as_str();
                target = format!("{}{}{}", pt::strip(u), if u.ends_with(' ') { " " } else { "" }, target);
            } else if CURRENCY.match_(&target).is_some() {
                tail = MONEY_WORD_FIRST.subn("", &tail, 1);
            }
            Value::from(format!("{}{}{}", &told[..last.start()], target, tail))
        }
        Some("date") => {
            let Some(last) = last_of(*DATE_PHRASE, told) else {
                return Value::None;
            };
            let tl = lower(&target);
            let said = if tl.starts_with("the ") || tl.starts_with("a ") || tl.starts_with("an ") { tl } else { target.clone() };
            Value::from(format!("{}{}{}", &told[..last.start()], said, &told[last.end()..]))
        }
        Some("time") => {
            let Some(last) = last_of(*AT_TIME, told) else {
                return Value::None;
            };
            let said = last.as_str();
            if let Some(half) = HALF_A_DAY.search(said) {
                if !HALF_OF_THE_DAY.is_match(&target) {
                    let sep = if target.chars().last().is_some_and(|c| c.is_ascii_digit() || pt::is_digit(&c.to_string())) { "" } else { " " };
                    target = format!("{target}{sep}{}", half.str(1));
                }
            }
            let rest = AT_TIME_FIRST.subn(&format!("at {target}"), said, 1);
            Value::from(format!("{}{}{}", &told[..last.start()], rest, &told[last.end()..]))
        }
        _ => {
            if wrong.truthy() {
                let named = Re::new(&format!(r"\b{}\b", esc(pt::strip(&sv(wrong)))), I);
                if !named.is_match(told) {
                    return Value::None;
                }
                return Value::from(named.subn(&target, told, 1));
            }
            if AMOUNT.fullmatch(&target).is_none() {
                return Value::None;
            }
            let Some(last) = last_of(*AMOUNT, told) else {
                return Value::None;
            };
            Value::from(format!("{}{}{}", &told[..last.start()], target, &told[last.end()..]))
        }
    }
}

/// Python: bot/memory/corrected.py:1665 `_plan_the_line_named(plans, told)`: the plan in the list that
/// the corrected sentence was kept as, or None.
pub fn _plan_the_line_named(plans: &[String], told: &str) -> Option<String> {
    let low = flat_low(told);
    if low.is_empty() {
        return None;
    }
    for plan in plans {
        let said = flat_low(plan);
        if !said.is_empty() && (said == low || low.contains(&said) || said.contains(&low)) {
            return Some(plan.clone());
        }
    }
    None
}

/// Python: bot/memory/corrected.py:1682 `_carries(value, word)`: whether a kept value's own wording
/// carries `word` as a word.
pub fn _carries(value: &Value, word: &str) -> bool {
    let said = flat_low(&s(value));
    let word = flat_low(word);
    !said.is_empty() && !word.is_empty() && Re::new(&format!(r"\b{}\b", esc(&word)), 0).is_match(&said)
}

/// Python: bot/memory/corrected.py:1688 `_fact_naming(memory, wrong)`: (key, sentence) of the fact
/// whose own wording carries the value being corrected, or None.
pub fn _fact_naming(memory: &Memory, wrong: &Value) -> Value {
    let value = flat_low(&s(wrong));
    let entries = match memory.get("facts_log") {
        Some(Value::List(e)) => e.clone(),
        _ => return Value::None,
    };
    if value.is_empty() {
        return Value::None;
    }
    let pattern = Re::new(&format!(r"\b{}\b", esc(&value)), I);
    for entry in entries.iter().rev() {
        if !entry.is_dict() || !vget(entry, "key").as_str().is_some_and(|k| CORRECTABLE.contains(k)) {
            continue;
        }
        let said = s(vget(entry, "text"));
        if pattern.is_match(&said) {
            return Value::Tuple(Arc::new(vec![vget(entry, "key").clone(), Value::from(said)]));
        }
    }
    Value::None
}

/// Python: bot/memory/corrected.py:1710 `_a_birthday_on_the_last_line(memory)`: whether the line
/// before this one stated a birthday, whichever entry it was logged as.
pub fn _a_birthday_on_the_last_line(memory: &Memory) -> bool {
    let said = flat_low(&last_user(memory));
    if said.is_empty() {
        return false;
    }
    for entry in mem_items(memory, "facts_log").iter().rev() {
        if !entry.is_dict() {
            continue;
        }
        if flat_low(&s(vget(entry, "text"))) != said {
            continue;
        }
        if vget(entry, "key").as_str() == Some("birthday") {
            return true;
        }
    }
    false
}

/// Python: bot/memory/corrected.py:1729 `_correct_birthday(memory, target, text="")`: the birthday a
/// person gives again is the one the store keeps.
pub fn _correct_birthday(memory: &mut Memory, target: &Value, text: &str) -> Value {
    let held = pt::squash(&s(vget(user(memory), "birthday")));
    let (day, month) = date_said(&flat_low(&s(target)));
    if day.is_none() && month.is_none() {
        return Value::None;
    }
    let (old_day, old_month) = date_said(&lower(&held));
    let day = day.or(old_day);
    let month = month.or(old_month);
    let value = match (&day, &month) {
        (Some(d), Some(m)) => format!("{d} of {m}"),
        (None, Some(m)) => m.clone(),
        (Some(d), None) => d.clone(),
        (None, None) => return Value::None,
    };
    if lower(&value) == lower(&held) {
        return Value::None;
    }
    let mut told = text.to_string();
    for entry in mem_items(memory, "facts_log").iter().rev() {
        if entry.is_dict() && vget(entry, "key").as_str() == Some("birthday") {
            let t = s(vget(entry, "text"));
            told = if t.is_empty() { text.to_string() } else { t };
            break;
        }
    }
    remember(memory, "birthday", Value::from(value.as_str()), &told);
    let sentence = _birthday_words(&Value::from(value.as_str()));
    dict(vec![("key", "birthday".into()), ("value", value.into()), ("sentence", sentence)])
}

/// Python: bot/memory/corrected.py:1763 `_a_birthday_for_somebody(memory)`: the person the line
/// before this one gave a birthday to, as the store keeps them, or None.
pub fn _a_birthday_for_somebody(memory: &mut Memory) -> Value {
    let said = flat_low(&last_user(memory));
    if said.is_empty() || (!said.contains("birthday") && !said.contains("born")) {
        return Value::None;
    }
    for entry in others(memory).iter().rev() {
        if !entry.is_dict() || !vget(entry, "birthday").truthy() {
            continue;
        }
        let who = lower(pt::strip(&s(vget(entry, "who"))));
        if who.is_empty() || !Re::new(&format!(r"\b{}s?\b", esc(&who)), 0).is_match(&said) {
            continue;
        }
        return entry.clone();
    }
    Value::None
}

static LOW_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

/// Python: bot/memory/corrected.py:1795 `_the_line_is_about_that_birthday(text, held, target)`:
/// whether a line naming a date is about the birthday the line before it gave.
pub fn _the_line_is_about_that_birthday(text: &str, held: &Value, target: &Value) -> bool {
    let value: HashSet<String> = LOW_WORDS.findall_str(&lower(&s(target))).into_iter().collect();
    let left: Vec<String> = LOW_WORDS
        .findall_str(&lower(text))
        .into_iter()
        .filter(|w| !NO_SUBJECT_OF_ITS_OWN.contains(w) && !value.contains(w))
        .collect();
    if left.is_empty() {
        return true;
    }
    let who = lower(pt::strip(&s(vget(held, "who"))));
    let theirs: HashSet<String> = if who.is_empty() { HashSet::new() } else { [who.clone(), format!("{who}s")].into_iter().collect() };
    left.iter().all(|w| STILL_THE_SAME_PERSON.contains(w) || theirs.contains(w))
}

static BARE_DAY_NUMBER: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(\d{1,2})\b", 0));

/// Python: bot/memory/corrected.py:1814 `_correct_their_birthday(memory, target, wrong, text="")`: a
/// date said again after somebody else's birthday is that person's birthday, moved.
pub fn _correct_their_birthday(memory: &mut Memory, target: &Value, wrong: &Value, text: &str) -> Value {
    let held = _a_birthday_for_somebody(memory);
    if !held.truthy() {
        return Value::None;
    }
    let low = flat_low(&s(target));
    let (day, month) = date_said(&low);
    if day.is_none() && month.is_none() {
        return Value::None;
    }
    let known = flat_low(&s(vget(&held, "birthday")));
    let (mut old_day, old_month) = date_said(&known);
    if old_day.is_none() {
        old_day = BARE_DAY_NUMBER.search(&known).map(|m| m.str(1).to_string());
    }
    let (gone_day, gone_month) = date_said(&flat_low(&s(wrong)));
    if (gone_day.is_some() || gone_month.is_some())
        && !((gone_day.is_some() && gone_day == old_day) || (gone_month.is_some() && gone_month == old_month))
    {
        return Value::None;
    }
    if !_the_line_is_about_that_birthday(text, &held, target) {
        return Value::None;
    }
    let day = day.or(old_day);
    let month = month.or(old_month);
    let value = match (&day, &month) {
        (Some(d), Some(m)) => format!("{d}{}{m}", if known.contains(" of ") { " of " } else { " " }),
        (None, Some(m)) => m.clone(),
        (Some(d), None) => d.clone(),
        (None, None) => return Value::None,
    };
    if lower(&value) == lower(&known) {
        return Value::None;
    }
    let who = pt::strip(&s(vget(&held, "who"))).to_string();
    let mut entry = dict(vec![("who", who.as_str().into()), ("birthday", value.as_str().into())]);
    if vget(&held, "name").truthy() {
        entry.set_key("name", vget(&held, "name").clone());
    }
    let last = last_user(memory);
    remember_other(memory, &entry, if last.is_empty() { text } else { &last });
    let line = format!("my {who}s birthday is {}{value}", if day.is_some() { "on " } else { "in " });
    let link = if day.is_some() { "on " } else { "" };
    let said_back = format!("your {who}s birthday is {link}{}", s(&_birthday_words(&Value::from(value.as_str()))));
    dict(vec![
        ("key", "birthday".into()),
        ("value", value.into()),
        ("from", known.into()),
        ("line", line.into()),
        ("note", format!("Got it \u{2014} {said_back}.").into()),
    ])
}

/// Python: bot/memory/corrected.py:1868 `_correct_fact(memory, target, wrong, text="")`: replace the
/// fact the previous line stated with the value the user corrected it to.
pub fn _correct_fact(memory: &mut Memory, target: &Value, wrong: &Value, text: &str) -> Value {
    let theirs = _correct_their_birthday(memory, target, wrong, text);
    if theirs.truthy() {
        return theirs;
    }
    let target_s = sv(target);
    let counted = extract::pet_counts(&target_s);
    let pets = user_pets(memory);
    if counted.truthy() && !pets.is_empty() {
        let mut moved = String::new();
        let mut kept: Vec<Value> = pets.iter().map(|(_, p)| p.clone()).collect();
        for said in counted.items() {
            for pet in kept.iter_mut() {
                if family(vget(pet, "animal")) != family(vget(said, "animal")) {
                    continue;
                }
                if *vget(pet, "count") == *vget(said, "count") {
                    continue;
                }
                pet.set_key("count", vget(said, "count").clone());
                let animal = if vget(pet, "animal").truthy() { vget(pet, "animal").clone() } else { Value::from("pet") };
                moved = _many(vget(said, "count").as_int().unwrap_or(0), &animal);
            }
        }
        if !moved.is_empty() {
            user(memory).set_key("pets", Value::from(kept));
            return dict(vec![("key", "pets".into()), ("value", moved.into())]);
        }
    }
    let named = _fact_naming(memory, wrong);
    let key = if named.truthy() { named.items().first().map(|k| s(k)) } else { _stated_last(memory) };
    let Some(mut key) = key.filter(|k| !k.is_empty()) else {
        return Value::None;
    };
    if (key == "city" || key == "places") && !extract::place_value(&target_s).truthy() {
        return Value::None;
    }
    if key != "birthday" && _a_birthday_on_the_last_line(memory) {
        key = "birthday".into();
    }
    if key == "birthday" {
        return _correct_birthday(memory, target, text);
    }
    if key == "pet" || key == "pets" {
        let pets = vget(user(memory), "pets").clone();
        let Value::List(list) = &pets else {
            return Value::None;
        };
        for (index, pet) in list.iter().enumerate().rev() {
            if !pet.is_dict() || !vget(pet, "name").truthy() {
                continue;
            }
            if wrong.truthy() && lower(&sv(vget(pet, "name"))) != lower(&sv(wrong)) {
                continue;
            }
            let name = crate::memory::relations::_tidy_name(target);
            set_pet(memory, index, "name", name.clone());
            let animal = if vget(pet, "animal").truthy() { vget(pet, "animal").clone() } else { Value::from("pet") };
            return dict(vec![("key", "pet".into()), ("value", name), ("animal", animal)]);
        }
        return Value::None;
    }
    let held = vget(user(memory), &key).clone();
    let mut told: Option<String> = None;
    if named.truthy() && !_carries(&held, &s(wrong)) {
        told = named.items().get(1).map(sv);
    }
    if key == "budget" || key == "plan" {
        let mut replaced = wrong.clone();
        if !replaced.truthy() && key == "plan" {
            let places: Vec<String> = vget(user(memory), "places").items().iter().filter_map(|p| p.as_str().map(str::to_string)).collect();
            replaced = places.last().map(|p| Value::from(p.as_str())).unwrap_or(Value::None);
        }
        if key == "plan" && replaced.truthy() && !extract::place_value(&target_s).truthy() {
            return Value::None;
        }
        let old = match &told {
            Some(t) => Value::from(t.as_str()),
            None => held.clone(),
        };
        let swapped = _swap_in(&old, target, &replaced);
        if swapped.truthy() {
            let swapped_s = sv(&swapped);
            let mut said = text.to_string();
            if key == "plan" {
                let origin = crate::memory::plan_line(memory, &s(&old));
                let in_origin = _swap_in(&Value::from(origin.as_str()), target, &replaced);
                said = if in_origin.truthy() {
                    sv(&in_origin)
                } else {
                    s(&_plan_told_line(memory, &old, &swapped, text))
                };
            }
            remember(memory, &key, swapped.clone(), &said);
            if key == "plan" {
                let plans: Vec<String> = vget(user(memory), "plans").items().iter().filter_map(|p| p.as_str().map(str::to_string)).collect();
                let holder = _plan_the_line_named(&plans, &s(&old));
                let new_plans: Option<Vec<String>> = match holder {
                    Some(h) => Some(plans.iter().map(|p| if *p == h { swapped_s.clone() } else { p.clone() }).collect()),
                    None if !plans.contains(&swapped_s) => {
                        let mut all = plans.clone();
                        all.push(swapped_s.clone());
                        let from = all.len().saturating_sub(6);
                        Some(all[from..].to_vec())
                    }
                    None => None,
                };
                if let Some(new_plans) = new_plans {
                    user(memory).set_key("plans", Value::from(new_plans));
                }
                let places: Vec<String> = vget(user(memory), "places").items().iter().filter_map(|p| p.as_str().map(str::to_string)).collect();
                if replaced.truthy()
                    && _kind_of_value(target, &old).is_none()
                    && places.iter().any(|p| lower(p) == lower(&sv(&replaced)))
                    && !places.iter().any(|p| lower(p) == lower(&target_s))
                {
                    let mut grown = places.clone();
                    grown.push(target_s.clone());
                    user(memory).set_key("places", Value::from(grown));
                }
            }
            let mut note_line = swapped_s.clone();
            if key == "plan" {
                let sentence = crate::memory::plan_sentence(&swapped_s, &said);
                note_line = if sentence.is_empty() { swapped_s.clone() } else { sentence };
                if lower(&note_line).starts_with("you're planning ") {
                    note_line = swapped_s.clone();
                }
            }
            return dict(vec![("key", key.as_str().into()), ("value", target.clone()), ("sentence", note_line.into())]);
        }
    }
    if held.is_str() && _kind_of_value(target, &held).is_some() {
        return Value::None;
    }
    let found = extract::reread(target, &key);
    if !found.truthy() {
        return Value::None;
    }
    if key == "city" || key == "places" {
        let place = if vget(&found, "city").truthy() {
            vget(&found, "city").clone()
        } else {
            vget(&found, "places").items().first().cloned().unwrap_or(Value::None)
        };
        if !place.truthy() {
            return Value::None;
        }
        let places: Vec<Value> = vget(user(memory), "places").items().iter().filter(|p| p.is_str()).cloned().collect();
        user(memory).set_key("places", Value::from(places));
        remember(memory, "places", Value::from(vec![place.clone()]), text);
        remember(memory, "city", place.clone(), text);
        return dict(vec![("key", "city".into()), ("value", place)]);
    }
    let mut value = vget(&found, &key).clone();
    if let Value::List(items) = &value {
        value = items.first().cloned().unwrap_or(Value::None);
    }
    if !value.truthy() && !matches!(value, Value::Int(_) | Value::Float(_) | Value::Bool(_)) {
        return Value::None;
    }
    if key == "budget" {
        value = _keep_unit(&value, vget(user(memory), "budget"));
    }
    if key == "allergies" || key == "people" || key == "conditions" {
        _drop_wrong(user(memory), &key, wrong, target);
    }
    remember(memory, &key, value.clone(), text);
    dict(vec![("key", key.as_str().into()), ("value", value), ("from", wrong.clone())])
}

static A_LETTER: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z]", 0));
static A_UNIT_AT_THE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"([a-z]+)\s*$", I));

/// Python: bot/memory/corrected.py:2032 `_keep_unit(value, was)`: a bare number takes the unit the
/// fact it replaces was said in (a str).
pub fn _keep_unit(value: &Value, was: &Value) -> Value {
    let value = pt::strip(&s(value)).to_string();
    if value.is_empty() || A_LETTER.is_match(&value) {
        return Value::from(value);
    }
    match A_UNIT_AT_THE_END.search(pt::strip(&s(was))) {
        Some(unit) => Value::from(format!("{value} {}", lower(unit.str(1)))),
        None => Value::from(value),
    }
}

/// Python: bot/memory/corrected.py:2041 `_drop_wrong(known, key, wrong, target)`: a correction said
/// which value was wrong ("tree nuts, not peanuts"): that one goes (from `known`, the user dict).
pub fn _drop_wrong(known: &mut Value, key: &str, wrong: &Value, target: &Value) {
    let kept: Vec<String> = vget(known, key).items().iter().filter_map(|v| v.as_str().map(str::to_string)).collect();
    if kept.is_empty() {
        return;
    }
    let doomed: HashSet<String> = [wrong, target].into_iter().filter(|w| w.truthy()).map(|w| lower(&sv(w))).collect();
    let mut left: Vec<String> = kept.iter().filter(|v| !doomed.contains(&lower(v))).cloned().collect();
    if left == kept && kept.len() == 1 {
        left.clear();
    }
    known.set_key(key, Value::from(left));
}

static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", 0));
static AN_ORDINAL_FIRST: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d{1,2}(?:st|nd|rd|th)\b", 0));
static AN_ORDINAL_AND_A_MONTH: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d{1,2}(?:st|nd|rd|th)\s+[A-Z][a-z]+$", 0));

/// Python: bot/memory/corrected.py:2062 `_birthday_words(date)`: the date the way it reads in a
/// sentence: "3rd of may" -> "the 3rd of May" (a str).
pub fn _birthday_words(date: &Value) -> Value {
    let said = pt::squash(&sv(date));
    let parts: Vec<String> = pt::split(&said).iter().map(|w| if MONTH_WORDS.contains(w) { pt::capitalize(w) } else { w.clone() }).collect();
    let said = parts.join(" ");
    if !A_DIGIT.is_match(&said) {
        return Value::from(format!("in {said}"));
    }
    if AN_ORDINAL_FIRST.match_(&said).is_none() {
        return Value::from(said);
    }
    if AN_ORDINAL_AND_A_MONTH.match_(&said).is_some() {
        let words = pt::split(&said);
        return Value::from(format!("the {} of {}", words[0], words[1]));
    }
    Value::from(format!("the {said}"))
}

/// Python: bot/memory/corrected.py:2082 `_many(count, animal)`: "two cats": how many animals, said the
/// way a person says it.
pub fn _many(count: i64, animal: &Value) -> String {
    let a = sv(animal);
    let noun = PLURAL_PETS.get(&lower(&a)).cloned().unwrap_or_else(|| format!("{a}s"));
    let number = COUNT_WORDS.iter().find(|(k, _)| *k == count).map(|(_, v)| v.clone()).unwrap_or_else(|| count.to_string());
    format!("{number} {noun}")
}

/// Python: bot/memory/corrected.py:2098 `_asks(line)`: true when the user's line is a question rather
/// than something they told us.
pub fn _asks(line: &str) -> bool {
    ASK_START.match_(line).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Value {
        Value::from(s)
    }

    /// Expected values from PyBot (`bot.memory.corrected`), called directly.
    #[test]
    fn the_readers_say_what_pybot_says() {
        assert_eq!(_letters_apart("anna", "hannah"), 2);
        assert!(_a_spelling_of("anna", "ana"));
        assert!(!_a_spelling_of("june", "jane"));
        assert_eq!(_birthday_words(&v("3rd of may")).str(), "the 3rd of May");
        assert_eq!(_birthday_words(&v("3rd may")).str(), "the 3rd of May");
        assert_eq!(_birthday_words(&v("august")).str(), "in August");
        assert_eq!(_birthday_words(&v("4 july 1980")).str(), "4 July 1980");
        assert_eq!(_many(2, &v("cat")), "two cats");
        assert_eq!(_many(12, &v("puppy")), "12 puppies");
        assert_eq!(_a_count(1, &v("cats")), "1 cats");
        assert_eq!(_a_count(3, &v("fish")), "3 fish");
        assert_eq!(date_said("the 4th of june"), (Some("4th".into()), Some("june".into())));
        assert_eq!(date_said("4 june"), (Some("4".into()), Some("june".into())));
        assert_eq!(date_said("sept"), (None, Some("sept".into())));
        assert_eq!(_a_day_in_the_kind("3rd", "the 4th"), "The 4th");
        assert_eq!(_a_day_in_the_kind("3", "the 22nd"), "The 22nd");
        assert_eq!(_a_day_in_the_kind("11th", "12"), "12th");
        assert_eq!(_swap_in(&v("A trip to Lisbon in May"), &v("August"), &Value::None).str(), "A trip to Lisbon in August");
        assert_eq!(_swap_in(&v("400 pounds"), &v("500"), &Value::None).str(), "500 pounds");
        assert_eq!(_swap_in(&v("£400"), &v("500"), &Value::None).str(), "£500");
        assert_eq!(_swap_in(&v("my flight is at 6am"), &v("8"), &Value::None).str(), "my flight is at 8am");
        assert_eq!(_swap_in(&v("Lisbon in May"), &v("Porto"), &v("lisbon")).str(), "Porto in May");
        assert_eq!(
            _swap_in(&v("Portugal on the 12th of june"), &v("the 13th of june"), &Value::None).str(),
            "Portugal on the 13th of june"
        );
        assert_eq!(_what_took_its_place("sons birthday on 3 may", "sons birthday on 4 may", "3 may").as_deref(), Some("4 may"));
        assert_eq!(_the_topic_with_the_new_clock("im flying at 6am", "im flying at 7am").as_deref(), Some("im flying at 7am"));
        assert_eq!(
            _a_relative_month("Rome next month", None, Some("july")),
            Some(("Rome in July".into(), "next month".into(), "in July".into()))
        );
        assert_eq!(
            _a_relative_month("Rome next month", Some("12th"), Some("july")),
            Some(("Rome on the 12th of July".into(), "next month".into(), "on the 12th of July".into()))
        );
        assert_eq!(_kind_of_value(&v("500"), &v("my budget is 400")).as_deref(), Some("amount"));
        assert_eq!(_kind_of_value(&v("8"), &v("leave at 4")).as_deref(), Some("time"));
        assert_eq!(_keep_unit(&v("500"), &v("300 pounds")).str(), "500 pounds");
        assert!(!_asks("were redoing our bathroom"));
        assert!(_asks("so what do i do"));
        assert_eq!(_without_the_naming_verb(&v("called max")).str(), "Max");
        assert_eq!(_without_the_naming_verb(&v("hes max")).str(), "Max");
        assert_eq!(_clock_key("10 a.m."), "10am");
        let amount = _an_amount_on_the_measure("Lose a stone", &v("half a stone"));
        assert_eq!(amount.get("plan").map(Value::str), Some("Lose half a stone"));
        assert_eq!(amount.get("was").map(Value::str), Some("a stone"));
        assert_eq!(_count_said("a couple of dogs"), 2);
        assert_eq!(_a_bare_hour(&v("12")), "12");
    }
}
