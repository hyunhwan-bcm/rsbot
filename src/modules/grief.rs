//! Port of `bot/modules/grief.py`: a short remark inside a recent loss, answered naming the
//! person who died (the funeral, the paperwork, the nights, the guilt), and the loss written
//! down as it is said so the turns after it are read in it.
#![allow(non_snake_case, clippy::all)]

use super::chat::{attempt, dyn_re, py, truthy};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! res {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Re> = LazyLock::new(|| data::re(concat!("bot.modules.grief.", stringify!($n))));)* };
}
macro_rules! vals {
    ($($n:ident),* $(,)?) => { $(static $n: LazyLock<&'static Value> = LazyLock::new(|| data::get(concat!("bot.modules.grief.", stringify!($n))));)* };
}

res!(
    SUDDEN, EXPECTED, SLEEP, GUILT, HARD, CRYING, TALK_ABOUT, MISSING, TALK_TO_SOMEONE,
    CANT_TALK, SIGNS, AGE, AGE_NUMBER, BELONGINGS, EULOGY_WRITE, EULOGY, ARRANGEMENTS, REGISTER,
    COST, PROBATE, PAPERWORK, ARRANGEMENTS_ELSE, FAMILY, TELL, TELLING, SAY_TO, WORK,
    WORK_PRESSURE, ADMIN, OTHER_PERSON, EMERGENCY, THANKS_LINE, DOUBT_IT, DOUBT_ME, OPEN_ASK,
    PET_LOSS,
);
vals!(
    REMEMBER, SHORT, SHORT_LIFE, ANIMAL_SPAN, WORK_PRESSURE_REPLIES, WORK_REPLIES, PROBATE_REPLIES,
    ADMIN_REPLIES, PRACTICAL, SUDDEN_REPLIES, EXPECTED_REPLIES, SLEEP_REPLIES, GUILT_REPLIES,
    HARD_REPLIES, TALK_ABOUT_REPLIES, CRYING_REPLIES, MISSING_REPLIES, CANT_TALK_REPLIES,
    SIGNS_REPLIES, AGE_REPLIES, PET_OLD_REPLIES, PET_YOUNG_REPLIES, YOUNG_AGE_REPLIES,
    BELONGINGS_REPLIES, REGISTER_REPLIES, COST_REPLIES, ARRANGEMENTS_REPLIES, PAPERWORK_REPLIES,
    FAMILY_REPLIES, TELLING_REPLIES, SAY_TO_REPLIES, TELL_REPLIES, COMPANY_REPLIES,
    DOUBT_IT_REPLIES, DOUBT_ME_REPLIES, OPEN_ASK_REPLIES, LOSS_SPEAKERS, PERSON_ONLY,
    ANIMAL_PAPERWORK_REPLIES, ASK_OPENERS,
);

static PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.loss.PEOPLE"));
static PET_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.loss.PET_WORDS"));

/// Python: bot/modules/grief.py:34 `defers_to(reply, line)`
pub fn defers_to(reply: &str, line: &str) -> bool {
    let _ = reply;
    if !(crate::compose::loss::a_pet_died(line) && truthy(crate::compose::loss::asks_about_the_remains(line, &Value::None))) {
        return false;
    }
    !crate::compose::loss::names_a_feeling(line) || crate::compose::loss::feeling_about_the_body(line)
}

/// Python: `DEFERS_TO = defers_to` - the arbiter's hook.
pub fn DEFERS_TO(reply: &str, line: &str) -> bool {
    defers_to(reply, line)
}

/// Python: bot/modules/grief.py:521 `_words(text)`
pub fn _words(text_: &str) -> Vec<String> {
    let low = text_.to_lowercase().replace('\u{2019}', "'");
    text::split_ref(&low)
        .into_iter()
        .map(|w| text::strip_chars(w, ".,!?;:\"'()").to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Python: bot/modules/grief.py:527 `_who(words)`: (who, is a pet), or (None, False).
pub fn _who(words: &[String]) -> (Option<String>, bool) {
    let animal = crate::compose::loss::the_animal(&words.join(" "));
    if !animal.is_empty() {
        return (Some(format!("your {animal}")), true);
    }
    for word in words {
        let singular = match word.strip_suffix('s') {
            Some(stem) if PET_WORDS.contains(stem) => stem.to_string(),
            _ => word.clone(),
        };
        if PEOPLE.contains(&singular) || PET_WORDS.contains(&singular) {
            let pet = PET_WORDS.contains(&singular);
            return (Some(format!("your {singular}")), pet);
        }
    }
    (None, false)
}

/// Python: bot/modules/grief.py:542 `_record(text, memory)`
pub fn _record(text_: &str, memory: &mut Memory) {
    if text::strip(text_).is_empty() {
        return;
    }
    let clean = _words(text_).join(" ");
    if clean.is_empty() || !crate::compose::loss::reads_as_loss(&clean, None, false) {
        return;
    }
    if crate::compose::loss::whose(&clean) != "own" {
        return;
    }
    let (who, pet) = _who(&_words(text_));
    let Some(who) = who.filter(|w| !w.is_empty()) else { return };
    let mut record = Value::dict();
    record.set_key("who", who);
    record.set_key("pet", pet);
    record.set_key("turn", memory.turns.len());
    record.set_key("ahead", crate::compose::loss::is_dying_now(&clean));
    memory.set("loss", record);
}

/// Python: bot/modules/grief.py:571 `_still_here(memory)`
pub fn _still_here(memory: &Memory) -> bool {
    match memory.turns.last() {
        Some(t) => LOSS_SPEAKERS.contains_str(t.module.as_deref().unwrap_or("")),
        None => false,
    }
}

/// Python: bot/modules/grief.py:580 `_next_to_the_loss(record, memory)`
pub fn _next_to_the_loss(record: &Value, memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    if record.is_dict() && record.get("replied").cloned().unwrap_or(Value::None) == Value::Int(memory.turns.len() as i64 - 1) {
        return true;
    }
    if LOSS_SPEAKERS.contains_str(last.module.as_deref().unwrap_or("")) {
        return true;
    }
    let previous = text::squash(&last.user);
    if previous.is_empty() {
        return false;
    }
    crate::compose::loss::reads_as_loss(&previous, None, false)
        || FAMILY.search(&previous).is_some()
        || GUILT.search(&previous).is_some()
        || CRYING.search(&previous).is_some()
        || SLEEP.search(&previous).is_some()
        || HARD.search(&previous).is_some()
}

/// Python: bot/modules/grief.py:607 `_remembered(memory)`: the loss record, when it is recent.
pub fn _remembered(memory: &Memory) -> Option<Value> {
    let record = memory.get("loss")?;
    if !record.is_dict() || !record.get("who").is_some_and(|w| w.truthy()) {
        return None;
    }
    let turn = record.get("turn").and_then(|t| t.as_int()).unwrap_or(0);
    if turn < memory.turns.len() as i64 - REMEMBER.as_int().unwrap_or(3) {
        return None;
    }
    Some(record.clone())
}

/// Python: bot/modules/grief.py:617 `_age(clean)`
pub fn _age(clean: &str) -> Option<i64> {
    let m = AGE_NUMBER.search(clean)?;
    let digits = m.get(1).filter(|s| !s.is_empty()).map(str::to_string).or_else(|| m.get(2).filter(|s| !s.is_empty()).map(str::to_string)).unwrap_or_else(|| format!("{}0", m.str(3)));
    Some(digits.parse().unwrap_or_else(|_| panic!("ValueError: invalid literal for int(): {digits:?}")))
}

/// Python: bot/modules/grief.py:625 `_animal(who)`
pub fn _animal(who: &str) -> Option<String> {
    let mut words: Vec<&str> = ANIMAL_SPAN.pairs().iter().filter_map(|(k, _)| k.as_str()).collect();
    words.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    for word in words {
        if dyn_re(&format!(r"\b{word}\b"), crate::py::re::I).search(who).is_some() {
            return Some(word.to_string());
        }
    }
    None
}

static PET_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:your )?pets?\b", crate::py::re::I));

/// Python: bot/modules/grief.py:635 `_any_animal(who)`
pub fn _any_animal(who: &str) -> Option<String> {
    if let Some(animal) = _animal(who) {
        return Some(animal);
    }
    if PET_WORD.search(who).is_some() {
        Some("pet".into())
    } else {
        None
    }
}

/// `ANIMAL_SPAN[animal]`.
fn span_of(animal: &str) -> f64 {
    ANIMAL_SPAN.get(animal).and_then(|v| v.as_f64()).unwrap_or_else(|| panic!("KeyError: {animal:?}"))
}

/// Python: bot/modules/grief.py:643 `_young(clean, who="")`
pub fn _young(clean: &str, who: &str) -> bool {
    let Some(number) = _age(clean) else { return false };
    if let Some(animal) = _animal(who) {
        return (number as f64) < span_of(&animal) * 0.8;
    }
    number < SHORT_LIFE.as_int().unwrap_or(70)
}

/// Python: bot/modules/grief.py:657 `_age_pool(clean, who)`
pub fn _age_pool(clean: &str, who: &str) -> Vec<String> {
    let number = _age(clean);
    let animal = _animal(who);
    let mut pool = if _young(clean, who) { &*YOUNG_AGE_REPLIES } else { &*AGE_REPLIES };
    if animal.is_some() {
        pool = if _young(clean, who) { &*PET_YOUNG_REPLIES } else { &*PET_OLD_REPLIES };
    }
    let big = upper1(who);
    pool.strs()
        .iter()
        .map(|reply| {
            reply
                .replace("{who}", who)
                .replace("{Who}", &big)
                .replace("{animal}", animal.as_deref().unwrap_or(""))
                .replace("{Age}", &_word(number))
        })
        .collect()
}

fn upper1(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Python: bot/modules/grief.py:672 `_word(number)`
pub fn _word(number: Option<i64>) -> String {
    const WORDS: [&str; 20] = [
        "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Eleven", "Twelve", "Thirteen",
        "Fourteen", "Fifteen", "Sixteen", "Seventeen", "Eighteen", "Nineteen", "Twenty",
    ];
    match number {
        None => "That".into(),
        Some(n) if (1..=20).contains(&n) => WORDS[(n - 1) as usize].into(),
        Some(n) => n.to_string(),
    }
}

/// Python: bot/modules/grief.py:683 `_fill(pool, who)`
pub fn _fill(pool: &[String], who: &str) -> Vec<String> {
    let big = upper1(who);
    pool.iter().map(|reply| reply.replace("{who}", who).replace("{Who}", &big)).collect()
}

/// `record["replied"] = len(memory.get("turns") or [])`, on the record kept in memory.
fn note_replied(memory: &mut Memory) {
    let turns = memory.turns.len();
    if let Some(record) = memory.get_mut("loss") {
        record.set_key("replied", turns);
    }
}

fn practical(clean: &str) -> bool {
    PRACTICAL.items().iter().any(|p| p.as_re().is_some_and(|re| re.search(clean).is_some()))
}

fn person_only(re: &Re) -> bool {
    PERSON_ONLY.items().iter().any(|p| p.as_re().is_some_and(|q| q.pattern() == re.pattern() && q.flags() == re.flags()))
}

/// Python: bot/modules/grief.py:714 `respond(text, memory)` - a runtime hook.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    let words = _words(text_);
    if words.is_empty() || words.len() as i64 > SHORT.as_int().unwrap_or(16) {
        return None;
    }
    let clean = words.join(" ");
    if OTHER_PERSON.search(&clean).is_some() || EMERGENCY.search(&clean).is_some() {
        return None;
    }
    let read = memory.understood.clone().unwrap_or_default();
    if crate::knowledge::lookup::a_colour_is_worn(&read) {
        return None;
    }
    if data::re("bot.compose.loss.FUNERAL").search(&clean).is_some() && crate::compose::loss::a_part_of_the_day_asked(&clean) {
        return None;
    }
    let record = _remembered(memory)?;
    if crate::compose::loss::dying_in_the_chat(memory, 6)
        && (data::re("bot.compose.loss.SAY_GOODBYE").search(&clean).is_some()
            || TELLING.search(&clean).is_some()
            || SAY_TO.search(&clean).is_some())
    {
        return None;
    }
    let pool = if SLEEP.search(&clean).is_some() { None } else { _composer_pool(text_, memory) };
    if let Some(pool) = &pool {
        let practical_ask = (_asks(&clean) || ADMIN.search(&clean).is_some()) && practical(&clean);
        let loss_pool = Value::from(pool.clone()).items() == data::get("bot.compose.loss.LOSS_REPLIES").items();
        if !(practical_ask
            && (loss_pool
                || WORK.search(&clean).is_some()
                || WORK_PRESSURE.search(&clean).is_some()
                || TELLING.search(&clean).is_some()
                || TELL.search(&clean).is_some()
                || ADMIN.search(&clean).is_some()))
        {
            return None;
        }
    }
    if crate::compose::loss::reads_as_loss(&clean, None, false) && !practical(&clean) {
        return None;
    }
    let who = py_text(record.get("who").unwrap_or_else(|| panic!("KeyError: 'who'")));
    if CANT_TALK.search(&clean).is_some() && TALK_TO_SOMEONE.search(&clean).is_none() {
        note_replied(memory);
        return Some(_pick(&_fill(&CANT_TALK_REPLIES.strs(), &who), memory, &clean));
    }
    if OPEN_ASK.match_(&clean).is_some() && !practical(&clean) && !_answered_by_the_knowledge_base(text_, memory) {
        note_replied(memory);
        return Some(_pick(&_fill(&OPEN_ASK_REPLIES.strs(), &who), memory, &clean));
    }
    if _next_to_the_loss(&record, memory) {
        for (doubt, pool) in [(&*DOUBT_IT, &*DOUBT_IT_REPLIES), (&*DOUBT_ME, &*DOUBT_ME_REPLIES)] {
            if doubt.match_(&clean).is_some() {
                note_replied(memory);
                return Some(_pick(&pool.strs(), memory, &clean));
            }
        }
    }
    let table: [(&Re, Option<&Value>); 23] = [
        (&FAMILY, Some(&FAMILY_REPLIES)),
        (&WORK_PRESSURE, Some(&WORK_PRESSURE_REPLIES)),
        (&WORK, Some(&WORK_REPLIES)),
        (&TELL, Some(&TELL_REPLIES)),
        (&TELLING, Some(&TELLING_REPLIES)),
        (&SAY_TO, Some(&SAY_TO_REPLIES)),
        (&REGISTER, Some(&REGISTER_REPLIES)),
        (&COST, Some(&COST_REPLIES)),
        (&PROBATE, Some(&PROBATE_REPLIES)),
        (&PAPERWORK, Some(&PAPERWORK_REPLIES)),
        (&ADMIN, Some(&ADMIN_REPLIES)),
        (&ARRANGEMENTS, Some(&ARRANGEMENTS_REPLIES)),
        (&BELONGINGS, Some(&BELONGINGS_REPLIES)),
        (&EXPECTED, Some(&EXPECTED_REPLIES)),
        (&SUDDEN, Some(&SUDDEN_REPLIES)),
        (&SLEEP, Some(&SLEEP_REPLIES)),
        (&HARD, Some(&HARD_REPLIES)),
        (&TALK_ABOUT, Some(&TALK_ABOUT_REPLIES)),
        (&GUILT, Some(&GUILT_REPLIES)),
        (&CRYING, Some(&CRYING_REPLIES)),
        (&AGE, None),
        (&SIGNS, Some(&SIGNS_REPLIES)),
        (&MISSING, Some(&MISSING_REPLIES)),
    ];
    for (index, (pattern, pool)) in table.iter().enumerate() {
        if person_only(pattern) {
            if let Some(animal) = _any_animal(&who) {
                if _the_answer_says_the_animal(text_, memory, Some(&animal)) {
                    continue;
                }
                if pattern.search(&clean).is_some() {
                    note_replied(memory);
                    return Some(_pick(&_fill(&ANIMAL_PAPERWORK_REPLIES.strs(), &who), memory, ""));
                }
            }
        }
        let arrangements = index == 11;
        if arrangements && (ARRANGEMENTS_ELSE.search(&clean).is_some() || crate::compose::loss::a_part_of_the_day_asked(&clean)) {
            continue;
        }
        if arrangements && EULOGY.search(&clean).is_some() && EULOGY_WRITE.search(&clean).is_some() {
            continue;
        }
        if pattern.search(&clean).is_some() {
            note_replied(memory);
            let lines = match pool {
                Some(pool) => pool.strs(),
                None => _age_pool(&clean, &who),
            };
            return Some(_pick(&_fill(&lines, &who), memory, ""));
        }
    }
    if THANKS_LINE.match_(&clean).is_some() && _still_here(memory) {
        note_replied(memory);
        return Some(_pick(&_fill(&COMPANY_REPLIES.strs(), &who), memory, ""));
    }
    None
}

/// `str(value)` for a str kept in memory.
fn py_text(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::None => "None".into(),
        other => panic!("str() of {other:?} is not ported"),
    }
}

/// Python: bot/modules/grief.py:840 `_asks(clean)`
pub fn _asks(clean: &str) -> bool {
    clean.contains('?') || ASK_OPENERS.contains_str(text::split_ref(clean).first().copied().unwrap_or(""))
}

/// Python: bot/modules/grief.py:845 `_composer_pool(text, memory)`
pub fn _composer_pool(text_: &str, memory: &Memory) -> Option<Vec<String>> {
    let pool = attempt(|| py(crate::compose::loss::reply_for(text_, memory, "", &[], None)))?;
    if pool.is_none() {
        return None;
    }
    Some(pool.strs())
}

/// Python: bot/modules/grief.py:853 `observe(text, memory)` - a runtime hook.
pub fn observe(text_: &str, memory: &mut Memory) {
    _record(text_, memory);
}

/// The line asked with the chat's topic in front of it (`topic + " " + text`).
fn with_the_topic(text_: &str, memory: &Memory) -> String {
    let topic = match memory.get("topic") {
        Some(t) if t.is_dict() => t.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        _ => String::new(),
    };
    if topic.is_empty() {
        text_.to_string()
    } else {
        text::strip(&format!("{topic} {text_}")).to_string()
    }
}

/// Python: bot/modules/grief.py:857 `_the_answer_says_the_animal(text, memory, animal)`
pub fn _the_answer_says_the_animal(text_: &str, memory: &Memory, animal: Option<&str>) -> bool {
    let got = attempt(|| {
        let asked = with_the_topic(text_, memory);
        let mut copy = memory.clone();
        crate::modules::answers::respond(&asked, &mut copy)
    });
    let Some(got) = got else { return false };
    let low = got.unwrap_or_default().to_lowercase();
    if !(low.contains("vet") || animal.is_some_and(|a| !a.is_empty() && low.contains(a))) {
        return false;
    }
    PET_LOSS.search(&low).is_some()
}

/// Python: bot/modules/grief.py:893 `_answered_by_the_knowledge_base(text, memory)`
pub fn _answered_by_the_knowledge_base(text_: &str, memory: &Memory) -> bool {
    attempt(|| {
        let asked = with_the_topic(text_, memory);
        let mut copy = memory.clone();
        crate::modules::answers::respond(&asked, &mut copy).is_some_and(|r| !r.is_empty())
    })
    .unwrap_or(false)
}

/// Python: bot/modules/grief.py:916 `_pick(options, memory, seed_text="")`
pub fn _pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    match attempt(|| crate::compose::pick(options, memory, seed_text)) {
        Some(line) => line,
        None => options.first().cloned().unwrap_or_else(|| panic!("IndexError: list index out of range")),
    }
}
