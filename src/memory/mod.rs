//! Port of PyBot's `bot/memory` package: the parts the two observers call. Each Python file
//! `bot/memory/<name>.py` is `<name>.rs` here.

pub mod a_year_of_birth;
pub mod aboutthem;
pub mod ack;
pub mod again;
pub mod already;
pub mod amounts;
pub mod another;
pub mod asked_about;
pub mod asleep;
pub mod back_again;
pub mod baking;
pub mod callback;
pub mod carried_on;
pub mod carry;
pub mod cause;
pub mod change;
pub mod choice;
pub mod company;
pub mod condition;
pub mod corrected;
pub mod counts;
pub mod deadline;
pub mod decided;
pub mod detail;
pub mod doing_it;
pub mod doubt;
pub mod earlier;
pub mod enough;
pub mod favourite;
pub mod filler;
pub mod goals;
pub mod groupages;
pub mod guest;
pub mod healing;
pub mod history;
pub mod holds;
pub mod hope;
pub mod hours;
pub mod household;
pub mod kidcount;
pub mod kinds;
pub mod limit;
pub mod limits;
pub mod lineback;
pub mod matters;
pub mod meant;
pub mod measure;
pub mod more;
pub mod named;
pub mod narrow;
pub mod newborn;
pub mod not_a_trip;
pub mod not_this;
pub mod obstacle;
pub mod onset;
pub mod opening;
pub mod outfit;
pub mod outgoings;
pub mod paragraph;
pub mod passing_the_time;
pub mod pending;
pub mod people;
pub mod pile;
pub mod places;
pub mod plan;
pub mod plans;
pub mod portions;
pub mod prefer;
pub mod price;
pub mod purse;
pub mod quotes;
pub mod rates;
pub mod reject;
pub mod relations;
pub mod remedy;
pub mod reminded;
pub mod rename;
pub mod repair;
pub mod repeat;
pub mod resize;
pub mod returned;
pub mod returns;
pub mod saving;
pub mod setback;
pub mod size;
pub mod spent;
pub mod spoken;
pub mod stands;
pub mod store_lines;
pub mod store_replies;
pub mod stuck;
pub mod surname;
pub mod swap;
pub mod taking;
pub mod taste;
pub mod the_verdict;
pub mod the_years;
pub mod their_feeling;
pub mod thread;
pub mod times;
pub mod told;
pub mod too_late;
pub mod tried;
pub mod trips;
pub mod warmth;
pub mod whathurts;
pub mod when;
pub mod whereabout;
pub mod whose_body;
pub mod will_it_work;
pub mod workplace;
pub mod years_in;
pub mod dream;
pub mod facts;
pub mod recall;
pub mod topics;


// ---------------------------------------------------------------------------------------------
// bot/memory/__init__.py: the store itself.

use crate::data::{self, Value};
use crate::py::re::{self, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Python's reading of the dicts and lists the memory holds (`Value`s), for the `bot/memory`
/// ports: `d.get(key)`, `str(x)`, `for x in y`, `int(x)` with Python's results and errors.
pub mod pyv {
    use crate::data::Value;
    use crate::py::text;

    pub static NONE: Value = Value::None;

    /// `d.get(key)`: None when the key is missing. On anything but a dict Python raises
    /// AttributeError, which nothing in the store catches.
    pub fn get<'a>(d: &'a Value, key: &str) -> &'a Value {
        match d {
            Value::Dict(_) => d.get(key).unwrap_or(&NONE),
            other => panic!("AttributeError: {} has no attribute 'get'", repr(other)),
        }
    }

    /// `d.get(key)` for a key of any type.
    pub fn get_key<'a>(d: &'a Value, key: &Value) -> &'a Value {
        d.pairs().iter().find(|(k, _)| k == key).map(|(_, v)| v).unwrap_or(&NONE)
    }

    /// `repr(s)` of a str.
    pub fn repr_str(s: &str) -> String {
        let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
        let mut out = String::with_capacity(s.len() + 2);
        out.push(quote);
        for c in s.chars() {
            match c {
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c == quote => {
                    out.push('\\');
                    out.push(c);
                }
                c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
                c if (0x80..0xa0).contains(&(c as u32)) || c as u32 == 0xad => out.push_str(&format!("\\x{:02x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push(quote);
        out
    }

    /// `repr(v)`.
    pub fn repr(v: &Value) -> String {
        let join = |xs: &[Value]| xs.iter().map(repr).collect::<Vec<_>>().join(", ");
        match v {
            Value::None => "None".into(),
            Value::Bool(true) => "True".into(),
            Value::Bool(false) => "False".into(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => text::float_repr(*f),
            Value::Str(s) => repr_str(s),
            Value::List(xs) => format!("[{}]", join(xs)),
            Value::Tuple(xs) if xs.len() == 1 => format!("({},)", repr(&xs[0])),
            Value::Tuple(xs) => format!("({})", join(xs)),
            Value::Set(xs) if xs.is_empty() => "set()".into(),
            Value::Set(xs) => format!("{{{}}}", join(xs)),
            Value::Dict(d) => {
                format!("{{{}}}", d.iter().map(|(k, v)| format!("{}: {}", repr(k), repr(v))).collect::<Vec<_>>().join(", "))
            }
            Value::Re(r) => format!("re.compile({})", repr_str(r.pattern())),
            Value::Fn(f) => format!("<function {}>", f.name()),
            Value::Obj(c, _) => format!("<{c} object>"),
            Value::Module(m) => format!("<module '{m}'>"),
            Value::Opaque(o) => format!("<{o}>"),
        }
    }

    /// `str(v)`.
    pub fn str_of(v: &Value) -> String {
        match v {
            Value::Str(s) => s.to_string(),
            other => repr(other),
        }
    }

    /// `str(v or "")`.
    pub fn str_or_empty(v: &Value) -> String {
        if v.truthy() { str_of(v) } else { String::new() }
    }

    /// `for x in v`: the items of a list, tuple or set, the keys of a dict, the characters of a
    /// str. Anything else is not iterable (TypeError).
    pub fn iter(v: &Value) -> Vec<Value> {
        match v {
            Value::List(xs) | Value::Tuple(xs) | Value::Set(xs) => xs.to_vec(),
            Value::Dict(d) => d.iter().map(|(k, _)| k.clone()).collect(),
            Value::Str(s) => s.chars().map(|c| Value::from(c.to_string())).collect(),
            other => panic!("TypeError: {} is not iterable", repr(other)),
        }
    }

    /// The whole memory as the dict Python passes around, for a contract that takes it as a
    /// `Value` (`compose.pick`).
    pub fn memory_value(memory: &crate::state::Memory) -> Value {
        Value::from_plain(&serde_json::Value::Object(memory.to_plain()))
    }

    /// `compose.pick(options, memory, seed_text=text)`.
    pub fn pick(options: &[String], memory: &crate::state::Memory, seed_text: &str) -> String {
        crate::compose::pick(options, &memory_value(memory), seed_text)
    }

    /// What another file's function gave back, read for its truth whatever its contract type
    /// is today (a stub that returns `()` is Python's None).
    pub trait Truthy {
        fn truthy(&self) -> bool;
    }
    impl Truthy for () {
        fn truthy(&self) -> bool {
            false
        }
    }
    impl Truthy for bool {
        fn truthy(&self) -> bool {
            *self
        }
    }
    impl Truthy for String {
        fn truthy(&self) -> bool {
            !self.is_empty()
        }
    }
    impl Truthy for Value {
        fn truthy(&self) -> bool {
            Value::truthy(self)
        }
    }
    impl<T: Truthy> Truthy for Option<T> {
        fn truthy(&self) -> bool {
            self.as_ref().is_some_and(Truthy::truthy)
        }
    }

    /// `try: f() except Exception: None`: a panic (Python's exception) is caught.
    pub fn attempt<T>(f: impl FnOnce() -> T) -> Option<T> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).ok()
    }

    /// `isinstance(v, int)` (a bool is an int in Python).
    pub fn is_int(v: &Value) -> bool {
        matches!(v, Value::Int(_) | Value::Bool(_))
    }

    /// The number a value is compared as (`x > 1`); a str or a container against a number is a
    /// TypeError in Python.
    pub fn num(v: &Value) -> f64 {
        v.as_f64().unwrap_or_else(|| panic!("TypeError: {} is not a number", repr(v)))
    }

    /// `int(v)`.
    pub fn int_of(v: &Value) -> i64 {
        match v {
            Value::Int(i) => *i,
            Value::Bool(b) => *b as i64,
            Value::Float(f) => f.trunc() as i64,
            Value::Str(s) => text::strip(s)
                .replace('_', "")
                .parse()
                .unwrap_or_else(|_| panic!("ValueError: invalid literal for int(): {}", repr_str(s))),
            other => panic!("TypeError: int() argument {}", repr(other)),
        }
    }
}

const M: &str = "bot.memory.";
fn words_of(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}

static LIST_FACTS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.LIST_FACTS"));
static PETS_KEPT: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.PETS_KEPT").as_int().unwrap() as usize);
static ANOTHER_ONE: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("ANOTHER_ONE"));
static COUNT_HEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("COUNT_HEAD"));
static QUESTION_OPEN: LazyLock<&'static Re> = LazyLock::new(|| rx("QUESTION_OPEN"));
static WHEN_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("WHEN_WORDS"));
static PLAN_OPENS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("PLAN_OPENS"));
static JOINED_SUBJECT: LazyLock<&'static Re> = LazyLock::new(|| rx("JOINED_SUBJECT"));
static PLAN_VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("PLAN_VERBS"));
static APPOINTMENT_PLAN: LazyLock<&'static Re> = LazyLock::new(|| rx("APPOINTMENT_PLAN"));
static APPOINTMENT_DOING: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("APPOINTMENT_DOING"));
static APPOINTMENT_A_DETERMINER: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("APPOINTMENT_A_DETERMINER"));
static APPOINTMENT_A_PERSON_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("APPOINTMENT_A_PERSON_VERB"));
static APPOINTMENT_A_QUANTITY: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("APPOINTMENT_A_QUANTITY"));
static APPOINTMENT_A_PLURAL: LazyLock<&'static Re> = LazyLock::new(|| rx("APPOINTMENT_A_PLURAL"));
static APPOINTMENT_NOT_A_PLURAL: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.APPOINTMENT_NOT_A_PLURAL"));
static _WEEKDAY_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("_WEEKDAY_NAMES"));
static _DATE_LEADS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("_DATE_LEADS"));
static _A_DAY_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_DAY_NUMBER"));
static MONTHS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("MONTHS"));
static WHOSE_IT_IS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("WHOSE_IT_IS"));
static A_THING_DOES_BARE: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("A_THING_DOES_BARE"));
static A_THING_DOES_S: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("A_THING_DOES_S"));
static JOURNEY_LINE: LazyLock<&'static Re> = LazyLock::new(|| rx("JOURNEY_LINE"));
static SAYS_TO_A_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("SAYS_TO_A_PERSON"));
static DONE_TO_A_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("DONE_TO_A_PERSON"));
static SUBJECT_HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("SUBJECT_HEADS"));
static FIRST_PERSON_HEADS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("FIRST_PERSON_HEADS"));
static _BOT_ME: LazyLock<String> = LazyLock::new(|| data::string("bot.memory._BOT_ME"));
static _BOT_MY: LazyLock<String> = LazyLock::new(|| data::string("bot.memory._BOT_MY"));
static APOSTROPHE: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.APOSTROPHE"));
static APOSTROPHE_WORD: LazyLock<&'static Re> = LazyLock::new(|| rx("APOSTROPHE_WORD"));
static QUOTED: LazyLock<&'static Re> = LazyLock::new(|| rx("QUOTED"));
static NOT_OWNED: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("NOT_OWNED"));
static WITH_SOMEBODY: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("WITH_SOMEBODY"));
static WHEN_OR_WHERE: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("WHEN_OR_WHERE"));
static ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("ASK_WORDS"));
static ASK_TAIL: LazyLock<&'static Re> = LazyLock::new(|| rx("ASK_TAIL"));
static WERE_DOING: LazyLock<&'static Re> = LazyLock::new(|| rx("WERE_DOING"));
static DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("DETERMINERS"));
static COUNTRIES: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("COUNTRIES"));
static NOT_PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| words_of("NOT_PLACES"));
static ANIMAL_KINDS: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.facts.ANIMAL_KINDS"));
static RELATIONS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.facts.RELATIONS"));
static TOPIC_PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.topics.PLACES"));
static TOPIC_NOT_PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.topics.NOT_PLACES"));

static LOWER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", re::U));
static PLAN_LEAD_IN: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"^(?:and\s+)?(?:also|oh|plus|by the way|btw|anyway|anyways|so|well|right|ok|okay|hey|and)\b[\s,]*",
        re::I | re::U,
    )
});
static LEADING_TO: LazyLock<Re> = LazyLock::new(|| Re::new(r"^to\s+", re::I | re::U));
static PLANS_TO: LazyLock<Re> = LazyLock::new(|| Re::new(r"^plans?\s+to\s+", re::I | re::U));
static IM_IVE_ILL: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:im|ive|ill)\b", re::I | re::U));
static WERE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bwere\b", re::I | re::U));
static WEVE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bweve\b", re::I | re::U));
static MID_YOU: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[\s,])You\b", re::U));
static WORD_THEN_YOU: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b([a-z']+) +\b(you|your)\b", re::I | re::U));
static NEXT_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"[\s,]*([a-z']+)", re::U));
static RELATION_THEN_THING: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b((?:[Mm]y|[Oo]ur)) ([a-z]+s) ([A-Za-z0-9']+)", re::U));
static BARE_RELATION: LazyLock<Re> = LazyLock::new(|| Re::new(r"([A-Za-z]+s)\s+([A-Za-z0-9']+)", re::U));
static ME: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bme\b", re::I | re::U));
static THEN_A_COMMA: LazyLock<Re> = LazyLock::new(|| Re::new(r"\s*,", re::U));
static FIRST_PERSON: LazyLock<Re> = LazyLock::new(|| {
    Re::new(
        r"\b(i'm|i am|i've|i have|i was not|i wasn't|i wasnt|i was|i were|we wasnt|we was|im|ive|my|mine|myself|me|we|our|ourselves|us|i)\b",
        re::I | re::U,
    )
});
/// `swaps` in `_flip_to_the_second_person`.
static SWAPS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("i'm", "you're"),
        ("i am", "you are"),
        ("i've", "you've"),
        ("i have", "you have"),
        ("i'll", "you'll"),
        ("we'll", "you'll"),
        ("im", "you're"),
        ("ive", "you've"),
        ("i was not", "you were not"),
        ("i wasn't", "you weren't"),
        ("i wasnt", "you weren't"),
        ("i was", "you were"),
        ("i were", "you were"),
        ("we was", "you were"),
        ("we wasnt", "you weren't"),
        ("my", "your"),
        ("mine", "yours"),
        ("myself", "yourself"),
        ("me", "you"),
        ("we", "you"),
        ("our", "your"),
        ("ourselves", "yourselves"),
        ("us", "you"),
        ("i", "you"),
    ])
});

/// `s[:1].lower() + s[1:]`.
fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// `s[:1].isupper()`.
fn opens_upper(s: &str) -> bool {
    s.chars().next().is_some_and(|c| text::is_upper(c.encode_utf8(&mut [0; 4])))
}

/// `s.split()[0]`: IndexError when there is no word.
fn first_word(s: &str) -> &str {
    text::split_ref(s).first().copied().expect("IndexError: list index out of range")
}

/// `{"key": key, "value": value, "turn": len(memory.get("turns") or []), "text": text}`.
fn log_entry(memory: &Memory, key: &str, value: Value, text_: &str) -> Value {
    let mut entry = Value::dict();
    entry.set_key("key", key);
    entry.set_key("value", value);
    entry.set_key("turn", memory.turns.len());
    entry.set_key("text", text_);
    entry
}

fn log_push(memory: &mut Memory, key: &str, value: Value, text_: &str) {
    let entry = log_entry(memory, key, value, text_);
    log(memory).push(entry);
}

/// A record put in the store is the same object in its log entry in Python (`pets.append(pet)`
/// beside `{"value": pet}`), so a later write to the record shows in the log as well. Values
/// here are copied on write, so the write is carried over by hand: `was` is the record before
/// the write (still sharing its storage with the log's copy), `now` the record after it.
///
/// PyBot's runtime deep-copies each key of the memory for every module, which parts the two;
/// only entries logged on this turn can still be the same object.
pub fn write_through(memory: &mut Memory, was: &Value, now: &Value) {
    let Value::Dict(was) = was else { return };
    let turn = Value::from(memory.turns.len());
    let Some(Value::List(entries)) = memory.get("facts_log") else { return };
    let shared: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            entry.get("turn") == Some(&turn)
                && matches!(entry.get("value"), Some(Value::Dict(held)) if std::sync::Arc::ptr_eq(held, was))
        })
        .map(|(i, _)| i)
        .collect();
    for at in shared {
        log(memory).items_mut().unwrap()[at].set_key("value", now.clone());
    }
}

/// `items[-n:]` for n > 0.
fn last_n(items: &[Value], n: usize) -> Value {
    Value::from(items[items.len().saturating_sub(n)..].to_vec())
}

/// Python: bot/memory/__init__.py:89 `user(memory)`
/// The user's facts (a dict), created on first use. Python hands back the dict itself, so
/// callers write through it.
pub fn user(memory: &mut Memory) -> &mut Value {
    if !memory.get("user").is_some_and(Value::is_dict) {
        memory.set("user", Value::dict());
    }
    memory.get_mut("user").unwrap()
}

/// Python: bot/memory/__init__.py:97 `log(memory)`
/// Every fact the user stated, in the order they said it (the list itself).
pub fn log(memory: &mut Memory) -> &mut Value {
    if !matches!(memory.get("facts_log"), Some(Value::List(_))) {
        memory.set("facts_log", Value::list());
    }
    memory.get_mut("facts_log").unwrap()
}

/// Python: bot/memory/__init__.py:105 `others(memory)`
/// Facts about other people the user mentioned: `[{"who": "son", "allergies": ["nuts"]}]`
/// (the list itself, created on first use).
pub fn others(memory: &mut Memory) -> &mut Value {
    if !matches!(memory.get("others"), Some(Value::List(_))) {
        memory.set("others", Value::list());
    }
    memory.get_mut("others").unwrap()
}

/// Python: bot/memory/__init__.py:129 `_said_another(text, who)`
/// Whether the line put one of the ANOTHER_ONE words in front of this person's word (a window
/// of two words). `who` is `str(who or "")`.
pub fn _said_another(text_: &str, who: &str) -> bool {
    let word = text::strip(who).to_lowercase();
    let words = LOWER_WORDS.findall_str(&text_.to_lowercase());
    if word.is_empty() || words.is_empty() {
        return false;
    }
    for (index, one) in words.iter().enumerate() {
        if *one != word && text::rstrip_chars(one, "s") != text::rstrip_chars(&word, "s") {
            continue;
        }
        if words[index.saturating_sub(2)..index].iter().any(|earlier| ANOTHER_ONE.contains(earlier)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/__init__.py:147 `_another_of_them(entry, known, text)`
/// Whether this fact is about a second person of the same kind, not the one already held.
pub fn _another_of_them(entry: &Value, known: &Value, text_: &str) -> bool {
    if pyv::get(entry, "name").truthy() || pyv::get(known, "name").truthy() {
        return false;
    }
    let (age, held) = (pyv::get(entry, "age"), pyv::get(known, "age"));
    if !pyv::is_int(age) || !pyv::is_int(held) || age == held {
        return false;
    }
    let who = if pyv::get(known, "who").truthy() { pyv::get(known, "who") } else { pyv::get(entry, "who") };
    _said_another(text_, &pyv::str_or_empty(who))
}

/// Python: bot/memory/__init__.py:164 `remember_other(memory, entry, text='')`
/// Keep a fact about someone else the user mentioned, merged into that person.
pub fn remember_other(memory: &mut Memory, entry: &Value, text_: &str) {
    let Value::Dict(pairs) = entry else { return };
    if !pairs.iter().any(|(_, v)| v.truthy()) {
        return;
    }
    let who = match pyv::get(entry, "who") {
        w if w.truthy() => w.clone(),
        _ => Value::from(""),
    };
    // A name on the entry is what tells two people under the same word apart.
    let named = text::strip(&pyv::str_or_empty(pyv::get(entry, "name"))).to_lowercase();
    if !who.truthy() && !named.is_empty() {
        // A line that names the person and nothing else is about the one person the chat
        // already knows by that name; two of them, or none, is nobody to file it under.
        let same: Vec<usize> = others(memory)
            .items()
            .iter()
            .enumerate()
            .filter(|(_, one)| text::strip(&pyv::str_or_empty(pyv::get(one, "name"))).to_lowercase() == named)
            .map(|(i, _)| i)
            .collect();
        if same.len() != 1 {
            return;
        }
        fold_in_place(memory, same[0], entry);
        return;
    }
    for index in 0..others(memory).len() {
        let known = others(memory).items()[index].clone();
        if *pyv::get(&known, "who") != who {
            continue;
        }
        let (new_name, old_name) = (pyv::get(entry, "name"), pyv::get(&known, "name"));
        if new_name.truthy() && old_name.truthy() && pyv::str_of(new_name).to_lowercase() != pyv::str_of(old_name).to_lowercase()
        {
            // Two people with the same word for them are two people.
            continue;
        }
        if _another_of_them(entry, &known, text_) {
            // A second one of the same kind keeps its own age beside the first.
            continue;
        }
        fold_in_place(memory, index, entry);
        return;
    }
    let mut kept = Value::dict();
    for (key, value) in pairs.iter() {
        kept.set_key(key.clone(), if value.is_list() { Value::from(value.items().to_vec()) } else { value.clone() });
    }
    others(memory).push(kept.clone());
    log_push(memory, "other", kept, text_);
}

/// `_fold_into` on the person at `index` of the store's list.
fn fold_in_place(memory: &mut Memory, index: usize, entry: &Value) {
    let was = others(memory).items()[index].clone();
    let now = _fold_into(&mut others(memory).items_mut().unwrap()[index], entry);
    write_through(memory, &was, &now);
}

/// Python: bot/memory/__init__.py:203 `_fold_into(memory, known, entry, text='')`
/// Put one fact the line stated about a person onto the person the store already holds.
/// (Python's `memory` and `text` arguments are not read; `known` is the record in the store.)
pub fn _fold_into(known: &mut Value, entry: &Value) -> Value {
    for (key, value) in entry.pairs() {
        let name = key.as_str().unwrap_or("");
        if name == "who" || name == "pronoun" {
            continue;
        }
        if name == "ages" {
            // A group of ages said again stands in place of the old one.
            known.set_key("ages", Value::from(pyv::iter(value)));
            known.del_key("age");
            known.del_key("to_come");
            known.del_key("turns");
            continue;
        }
        if name == "age" && pyv::get(known, "ages").truthy() {
            // The other way round: one age said for the same word replaces the list.
            known.del_key("ages");
            known.set_key("age", value.clone());
            known.del_key("months");
            known.del_key("to_come");
            known.del_key("turns");
            continue;
        }
        if name == "age" && pyv::is_int(value) && pyv::is_int(pyv::get(known, "age")) {
            // One of the two numbers can be a birthday still to come and the other the age now.
            let ahead = pyv::get(entry, "turns").clone();
            let knew = pyv::get(known, "turns").clone();
            if ahead.truthy() && !knew.truthy() {
                known.set_key("to_come", value.clone());
                known.set_key("turns", ahead);
                continue;
            }
            if knew.truthy() && !ahead.truthy() {
                if !pyv::is_int(pyv::get(known, "to_come")) && pyv::get(known, "age") != value {
                    let was = pyv::get(known, "age").clone();
                    known.set_key("to_come", was);
                }
                known.set_key("age", value.clone());
                if pyv::is_int(pyv::get(known, "to_come")) && pyv::num(pyv::get(known, "to_come")) <= pyv::num(value) {
                    known.del_key("to_come");
                    known.del_key("turns");
                }
                continue;
            }
            if ahead.truthy() && knew.truthy() && value != pyv::get(known, "age") {
                known.set_key("to_come", value.clone());
                known.set_key("turns", ahead);
                continue;
            }
        }
        if !value.is_list() {
            known.set_key(key.clone(), value.clone()); // a diet or a name is one thing, not a list of things
            continue;
        }
        if pyv::get_key(known, key).is_none() && !known.pairs().iter().any(|(k, _)| k == key) {
            known.set_key(key.clone(), Value::list());
        }
        let mut have = pyv::get_key(known, key).clone();
        if matches!(have, Value::List(_)) {
            for item in value.items() {
                if !have.items().contains(item) {
                    have.push(item.clone());
                }
            }
            known.set_key(key.clone(), have);
        }
    }
    known.clone()
}

/// Python: bot/memory/__init__.py:271 `reads_as_a_plan(value, text='')`
/// Whether a plan kept from a line reads well back in the summary of the chat. `value` is
/// `str(value)`.
pub fn reads_as_a_plan(value: &str, text_: &str) -> bool {
    let value = text::squash(value);
    if text::split_ref(&value).len() < 2 {
        return false;
    }
    if COUNT_HEAD.match_(&value).is_some() {
        return false;
    }
    let said = text::squash(text_);
    if !said.is_empty() && (said.ends_with('?') || QUESTION_OPEN.match_(&said).is_some()) {
        return false;
    }
    true
}

/// Python: bot/memory/__init__.py:299 `_the_thing_the_plan_is(plan)`
/// The plan without the words that say when it happens.
pub fn _the_thing_the_plan_is(plan: &str) -> String {
    let words: Vec<String> =
        LOWER_WORDS.findall_str(&plan.to_lowercase()).into_iter().filter(|word| !WHEN_WORDS.contains(word)).collect();
    if words.len() > 1 { words.join(" ") } else { String::new() }
}

/// `(x or 0)` read as a number.
fn count_or(v: &Value, default: f64) -> f64 {
    if v.truthy() { pyv::num(v) } else { default }
}

/// Python: bot/memory/__init__.py:307 `remember(memory, key, value, text='')`
/// Keep a fact about the user, logging when they said it.
///
/// A record put in the store is the same object in its log entry, so a later write to the
/// record shows in the log too: see `write_through`.
pub fn remember(memory: &mut Memory, key: &str, value: &Value, text_: &str) {
    // value in (None, "", [], {})
    if match value {
        Value::None => true,
        Value::Str(s) => s.is_empty(),
        Value::List(l) => l.is_empty(),
        Value::Dict(d) => d.is_empty(),
        _ => false,
    } {
        return;
    }
    user(memory);
    let mut value = value.clone();
    if let Value::List(items) = &value {
        let cleaned: Vec<Value> = items
            .iter()
            .map(|one| match one {
                Value::Str(s) => Value::from(filler::without_filler(s)),
                other => other.clone(),
            })
            .collect();
        value = Value::from(cleaned);
    }
    if key == "plan" || key == "plans" {
        // The chat-speak people hang off the end of a line is not part of the plan.
        if !matches!(value, Value::List(_)) {
            value = Value::from(filler::without_filler(&pyv::str_or_empty(&value)));
        }
    }
    if key == "plans" {
        // One line can carry more than one thing coming up, in the order said.
        let ones: Vec<Value> = match &value {
            Value::List(items) => items.to_vec(),
            one => vec![one.clone()],
        };
        for one in ones {
            if !reads_as_a_plan(&pyv::str_of(&one), text_) {
                continue;
            }
            let known = user(memory);
            let mut plans: Vec<Value> = match known.get("plans") {
                Some(Value::List(plans)) => plans.to_vec(),
                _ => Vec::new(),
            };
            if plans.contains(&one) {
                continue;
            }
            // The same thing coming up, said again with another date, is the plan they mean now.
            let thing = _the_thing_the_plan_is(&pyv::str_or_empty(&one));
            if !thing.is_empty() {
                plans.retain(|kept| _the_thing_the_plan_is(&pyv::str_or_empty(kept)) != thing);
            }
            plans.push(one);
            known.set_key("plans", last_n(&plans, 6));
        }
        log_push(memory, key, value, text_);
        return;
    }
    if key == "pets" {
        let mut pets: Vec<Value> = match user(memory).get("pets") {
            Some(Value::List(pets)) => pets.to_vec(),
            _ => Vec::new(),
        };
        // An animal they want, dislike or had once is not kept; one on its way is, marked "coming".
        let value: Vec<Value> = pyv::iter(&value)
            .into_iter()
            .filter(|pet| !(pet.is_dict() && pyv::get(pet, "not_theirs").truthy()))
            .collect();
        if value.is_empty() {
            return;
        }
        // A line that names more than one animal is a list of animals, not one animal named twice.
        let one_animal = value.len() == 1;
        for pet in &value {
            if !pet.is_dict() {
                continue;
            }
            let get = |key: &str| pyv::get(pet, key);
            let (animal, name, age) = (get("animal"), get("name"), get("age"));
            let another = get("another").truthy();
            // A name they already have an animal under is that animal, not a rename of its
            // brother: the name being new is what makes a line a rename.
            let already_named =
                pets.iter().filter(|p| pyv::get(p, "animal") == animal && pyv::get(p, "name") == name).count() > 0;
            if one_animal && name.truthy() && !another && !already_named && count_or(get("count"), 0.0) <= 1.0 {
                let rename: Vec<usize> = pets
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| {
                        pyv::get(p, "animal") == animal
                            && pyv::get(p, "name").truthy()
                            && pyv::get(p, "name") != name
                            && !(count_or(pyv::get(p, "count"), 0.0) > 1.0)
                    })
                    .map(|(i, _)| i)
                    .collect();
                if rename.len() == 1 {
                    log_push(memory, "pet", pet.clone(), text_);
                    let was = pets[rename[0]].clone();
                    let renamed = &mut pets[rename[0]];
                    renamed.set_key("name", name.clone());
                    if age.truthy() && age != pyv::get(renamed, "age") {
                        renamed.set_key("age", age.clone());
                        renamed.del_key("unit");
                        if get("unit").truthy() {
                            renamed.set_key("unit", get("unit").clone());
                        }
                    }
                    write_through(memory, &was, &pets[rename[0]]);
                    continue;
                }
            }
            // "my other cat" is only the same animal as one already kept when it gives the same
            // name; the count is the exception ("i have two cats" then "my other cat is max").
            let counted = pets.iter().position(|p| {
                pyv::get(p, "animal") == animal && count_or(pyv::get(p, "count"), 0.0) > 1.0 && !pyv::get(p, "name").truthy()
            });
            let mut same = pets.iter().enumerate().position(|(i, p)| {
                pyv::get(p, "animal") == animal
                    && (pyv::get(p, "name") == name
                        || (!another && (!pyv::get(p, "name").truthy() || !name.truthy()))
                        || (counted.is_some() && counted == Some(i)))
            });
            if same.is_none() && !another && count_or(get("count"), 0.0) <= 1.0 {
                // An animal still on its way takes the breed the next line gives it.
                let kind_of = |p: &Value| {
                    ANIMAL_KINDS.get(&pyv::str_or_empty(pyv::get(p, "animal")).to_lowercase()).cloned().unwrap_or_default()
                };
                let kind = kind_of(pet);
                if get("breed").truthy() && !name.truthy() {
                    same = pets.iter().position(|p| pyv::get(p, "coming").truthy() && !kind.is_empty() && kind_of(p) == kind);
                }
            }
            if let Some(same) = same {
                let was = pets[same].clone();
                // A name fills in the animal we already knew about.
                if name.truthy() && !pyv::get(&pets[same], "name").truthy() {
                    pets[same].set_key("name", name.clone());
                    log_push(memory, "pet", pet.clone(), text_);
                }
                let (breed, count) = (get("breed"), get("count"));
                if breed.truthy() && !pyv::get(&pets[same], "breed").truthy() {
                    pets[same].set_key("breed", breed.clone());
                    log_push(memory, "pet", pet.clone(), text_);
                } else if count.truthy() && count != pyv::get(&pets[same], "count") {
                    // "i have 2 cats" and then "sorry, 3 cats" is the same fact put right.
                    pets[same].set_key("count", count.clone());
                    log_push(memory, "pet", pet.clone(), text_);
                }
                if age.truthy() && age != pyv::get(&pets[same], "age") {
                    pets[same].set_key("age", age.clone());
                    pets[same].del_key("unit");
                    if get("unit").truthy() {
                        pets[same].set_key("unit", get("unit").clone());
                    }
                    log_push(memory, "pet", pet.clone(), text_);
                }
                // A later breed takes the place of the old one, as a later age does.
                if breed.truthy() && breed != pyv::get(&pets[same], "breed") {
                    pets[same].set_key("breed", breed.clone());
                    log_push(memory, "pet", pet.clone(), text_);
                }
                write_through(memory, &was, &pets[same]);
                continue;
            }
            pets.push(pet.clone());
            log_push(memory, "pet", pet.clone(), text_);
        }
        // Everything this line named is kept, and so are the newest of the rest.
        user(memory).set_key("pets", last_n(&pets, (*PETS_KEPT).max(value.len())));
        return;
    }
    if key == "tastes" {
        // A taste is a fact with a thing and a way: the same thing said again is the same entry.
        let mut tastes: Vec<Value> = match user(memory).get("tastes") {
            Some(Value::List(tastes)) => tastes.to_vec(),
            _ => Vec::new(),
        };
        let said: Vec<Value> = if value.is_list() { value.items().to_vec() } else { vec![value.clone()] };
        let flat = |t: &Value| text::squash(&pyv::str_or_empty(pyv::get(t, "thing")).to_lowercase());
        for taste in &said {
            if !taste.is_dict() || !pyv::get(taste, "thing").truthy() {
                continue;
            }
            let thing = flat(taste);
            if let Some(same) = tastes.iter().position(|t| flat(t) == thing) {
                if pyv::get(&tastes[same], "how") != pyv::get(taste, "how") {
                    let was = tastes[same].clone();
                    tastes[same].set_key("how", pyv::get(taste, "how").clone());
                    log_push(memory, "taste", taste.clone(), text_);
                    write_through(memory, &was, &tastes[same]);
                }
                continue;
            }
            tastes.push(taste.clone());
            log_push(memory, "taste", taste.clone(), text_);
        }
        user(memory).set_key("tastes", last_n(&tastes, 6));
        return;
    }
    if key == "owns" {
        // A thing they own is kept by the name of the thing, so a second one adds to the first.
        let mut held = match user(memory).get("owns") {
            Some(held @ Value::Dict(_)) => held.clone(),
            _ => Value::dict(),
        };
        if let Value::Dict(pairs) = &value {
            for (thing, name) in pairs.iter() {
                if !thing.truthy() || !name.truthy() {
                    continue;
                }
                if pyv::get_key(&held, thing) == name {
                    continue;
                }
                held.set_key(thing.clone(), name.clone());
                let mut one = Value::dict();
                one.set_key(thing.clone(), name.clone());
                log_push(memory, "owns", one, text_);
            }
        }
        user(memory).set_key("owns", held);
        return;
    }
    if LIST_FACTS.iter().any(|fact| fact == key) {
        let mut seen: Vec<Value> = match user(memory).get(key) {
            Some(Value::List(seen)) => seen.to_vec(),
            _ => Vec::new(),
        };
        let said: Vec<Value> = if value.is_list() { value.items().to_vec() } else { vec![value.clone()] };
        for item in said {
            if item.truthy() && !seen.contains(&item) {
                seen.push(item.clone());
                log_push(memory, key, item, text_);
            }
        }
        user(memory).set_key(key, last_n(&seen, 8));
        return;
    }
    let known = user(memory);
    if known.get(key).is_some_and(|held| *held == value) {
        return;
    }
    known.set_key(key, value.clone());
    log_push(memory, key, value, text_);
}

/// Python: bot/memory/__init__.py:533 `forget(memory, key, values, why=None)`
/// Take values back out of a fact: "im not coeliac" after "im coeliac". Returns the values
/// that actually went. `values` is a list, tuple or set, or one value (a pet may be handed back
/// as its record); `why` "" is Python's None.
pub fn forget(memory: &mut Memory, key: &str, values: &Value, why: &str) -> Vec<Value> {
    user(memory);
    let wanted: Vec<Value> = match values {
        Value::List(items) | Value::Tuple(items) | Value::Set(items) => items.to_vec(),
        one => vec![one.clone()],
    };
    let flat = |value: &Value| text::strip(&pyv::str_of(value)).to_lowercase();
    let doomed: HashSet<String> = wanted.iter().map(flat).filter(|word| !word.is_empty()).collect();
    let had = user(memory).get(key).cloned();
    if key == "pets" {
        // A pet is kept as a small record, so it goes back by the animal or the name.
        let Some(Value::List(had)) = &had else { return Vec::new() };
        if doomed.is_empty() {
            return Vec::new();
        }
        let everything = doomed.contains("*");
        // A record may also be handed back one at a time.
        let records: Vec<&Value> = wanted.iter().filter(|value| value.is_dict()).collect();
        let (mut kept, mut gone) = (Vec::new(), Vec::new());
        for pet in had.iter() {
            if !pet.is_dict() {
                continue;
            }
            let worded = doomed.contains(&text::strip(&pyv::str_or_empty(pyv::get(pet, "animal"))).to_lowercase())
                || doomed.contains(&text::strip(&pyv::str_or_empty(pyv::get(pet, "name"))).to_lowercase());
            let named = records.iter().any(|record| *record == pet);
            if !(everything || worded || named) {
                kept.push(pet.clone());
                continue;
            }
            if named && !everything && !worded && count_or(pyv::get(pet, "count"), 1.0) > 1.0 {
                // "i have two cats" then one of them dies: one animal goes, not the species.
                let was = pet.clone();
                let mut pet = pet.clone();
                let count = pyv::int_of(pyv::get(&pet, "count")) - 1;
                pet.set_key("count", count);
                if count <= 1 {
                    pet.del_key("count");
                }
                write_through(memory, &was, &pet);
                kept.push(pet);
                continue;
            }
            gone.push(pet.clone());
        }
        let known = user(memory);
        known.set_key(key, Value::from(kept.clone()));
        if kept.is_empty() {
            known.del_key(key);
        }
        // A pet that is gone is not a pet never mentioned: kept with the reason in `pets_gone`.
        if !gone.is_empty() {
            let before = pyv::get(known, "pets_gone");
            let mut past: Vec<Value> =
                if before.truthy() { pyv::iter(before).into_iter().filter(Value::is_dict).collect() } else { Vec::new() };
            for pet in &gone {
                if !past.contains(pet) {
                    let mut record = pet.clone();
                    record.set_key("gone", if why.is_empty() { "gone" } else { why });
                    past.push(record);
                }
            }
            known.set_key("pets_gone", Value::from(past));
        }
        return gone;
    }
    if LIST_FACTS.iter().any(|fact| fact == key) {
        let Some(Value::List(had)) = &had else { return Vec::new() };
        if doomed.is_empty() {
            return Vec::new();
        }
        let left: Vec<Value> = had.iter().filter(|value| !doomed.contains(&flat(value))).cloned().collect();
        user(memory).set_key(key, Value::from(left));
        return had.iter().filter(|value| doomed.contains(&flat(value))).cloned().collect();
    }
    let Some(had) = had.filter(|had| !had.is_none()) else { return Vec::new() };
    if doomed.is_empty() {
        return Vec::new();
    }
    if doomed.contains(&flat(&had)) {
        user(memory).del_key(key);
        return vec![had];
    }
    Vec::new()
}

/// Python: bot/memory/__init__.py:600 `there_is_a_chat(memory)`
/// Whether the person said anything before this line.
pub fn there_is_a_chat(memory: &Memory) -> bool {
    memory.turns.iter().any(|turn| !text::strip(&turn.user).is_empty())
}

/// Python: bot/memory/__init__.py:614 `recent(memory, n=6)`
/// The last n turns, oldest first. A reader with no memory to hand (`recent(None, 8)`) has no
/// turns to look at. The slice is Python's `turns[-n:]`, so n = 0 is every turn.
pub fn recent(memory: Option<&Memory>, n: i64) -> &[Turn] {
    let Some(memory) = memory else { return &[] };
    let turns = &memory.turns;
    let start = if n > 0 { turns.len().saturating_sub(n as usize) } else { (n.unsigned_abs() as usize).min(turns.len()) };
    &turns[start..]
}

/// Python: bot/memory/__init__.py:633 `last_user(memory, n=1)`
/// The last n things the user said (before this turn), oldest first: a list when n > 1, else
/// the one line or "".
pub fn last_user(memory: &Memory, n: i64) -> Value {
    let lines: Vec<Value> = recent(Some(memory), n).iter().map(|turn| Value::from(turn.user.as_str())).collect();
    if n > 1 { Value::from(lines) } else { lines.last().cloned().unwrap_or_else(|| Value::from("")) }
}

/// Python: bot/memory/__init__.py:639 `last_bot(memory)`
/// The bot's last reply.
pub fn last_bot(memory: &Memory) -> String {
    recent(Some(memory), 1).last().map(|turn| turn.bot.clone()).unwrap_or_default()
}

/// Python: bot/memory/__init__.py:645 `last_module(memory)`
pub fn last_module(memory: &Memory) -> Option<String> {
    recent(Some(memory), 1).last().and_then(|turn| turn.module.clone())
}

/// Python: bot/memory/__init__.py:650 `name(memory)`
/// The user's name, or ''.
pub fn name(memory: &mut Memory) -> String {
    user(memory).get("name").and_then(Value::as_str).unwrap_or("").to_string()
}

/// Python: bot/memory/__init__.py:724 `_takes_a_thing(word)`
/// True when the word after the verb marks what follows as a thing, not a compound's half.
pub fn _takes_a_thing(word: &str) -> bool {
    if APPOINTMENT_A_QUANTITY.contains(word) {
        return true;
    }
    APPOINTMENT_A_PLURAL.match_(word).is_some() && !text::ends_with_any(word, &APPOINTMENT_NOT_A_PLURAL)
}

/// Python: bot/memory/__init__.py:732 `_a_doing(thing)`
/// True when the plan's thing is something they are doing, not something they have on.
pub fn _a_doing(thing: &str) -> bool {
    let low = thing.to_lowercase();
    let words: Vec<&str> = text::split_ref(&low).into_iter().map(|word| text::strip_chars(word, ",.;:")).collect();
    if words.is_empty() {
        return false;
    }
    // A gerund is a doing read as a thing: "viewing the flat at 3" is the viewing.
    if words[0].ends_with("ing") && text::len(words[0]) > 4 {
        return false;
    }
    if APPOINTMENT_A_PERSON_VERB.contains(words[0]) {
        return true;
    }
    // A determiner cannot come after the head of a noun phrase.
    if words[1..].iter().any(|word| APPOINTMENT_A_DETERMINER.contains(*word)) {
        return true;
    }
    // "bake bread" is a doing; "book club" is a club.
    words.len() > 1 && APPOINTMENT_DOING.contains(words[0]) && _takes_a_thing(words[1])
}

/// Python: bot/memory/__init__.py:768 `named_days(said)`
/// "on tuesday in june" -> "on Tuesday in June"; "we may go to york" is left alone.
pub fn named_days(said: &str) -> String {
    let words = text::split_ref(said);
    let bare_of = |word: &str| text::strip_chars(&word.to_lowercase(), ",.;:!?").to_string();
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    for (index, word) in words.iter().enumerate() {
        let bare = bare_of(word);
        let before = if index > 0 { bare_of(words[index - 1]) } else { String::new() };
        let after = if index + 1 < words.len() { bare_of(words[index + 1]) } else { String::new() };
        if _WEEKDAY_NAMES.contains(&bare)
            || (MONTHS.contains(&bare)
                && (_DATE_LEADS.contains(&before)
                    || _A_DAY_NUMBER.match_(&before).is_some()
                    || _A_DAY_NUMBER.match_(&after).is_some()))
        {
            out.push(word.replacen(&bare, &text::upper_first(&bare), 1));
        } else {
            out.push(word.to_string());
        }
    }
    out.join(" ")
}

/// Python: bot/memory/__init__.py:784 `_a_name_at_the_front(text)`
/// Whether a kept plan opens with a capital that is not the article of a thing.
pub fn _a_name_at_the_front(text_: &str) -> bool {
    let words = text::split_ref(text_);
    if words.is_empty() || !opens_upper(words[0]) {
        return false;
    }
    !matches!(text::strip_chars(&words[0].to_lowercase(), ",.!?"), "a" | "an" | "the")
}

/// Python: bot/memory/__init__.py:816 `_a_plural(word)`
/// True for the subject of a sentence said in the plural: "clocks", "school holidays".
pub fn _a_plural(word: &str) -> bool {
    let word = word.to_lowercase();
    word.ends_with('s') && !text::ends_with_any(&word, &["ss", "us", "is"])
}

/// Python: bot/memory/__init__.py:822 `_has_its_own_verb(thing)`
/// True when the thing kept for a day already says what it does: "clocks go back".
pub fn _has_its_own_verb(thing: &str) -> bool {
    let low = thing.to_lowercase();
    let words: Vec<&str> = text::split_ref(&low).into_iter().map(|word| text::strip_chars(word, ",.;:!?")).collect();
    if words.len() < 2 {
        return false;
    }
    for index in 1..words.len() {
        let word = words[index];
        if A_THING_DOES_S.contains(word) {
            return true;
        }
        // The word in front of the verb is the head of the subject.
        if _a_plural(words[index - 1]) && A_THING_DOES_BARE.contains(word) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/__init__.py:837 `appointment_sentence(text)`
/// "dentist at 3 on tuesday" -> "Your dentist is at 3 on tuesday", or "" when it is not one.
pub fn appointment_sentence(text_: &str) -> String {
    let flat = text::squash(text_);
    let Some(found) = APPOINTMENT_PLAN.match_(text::strip_chars(&flat, " .!?,")) else { return String::new() };
    // "sons parents evening" is "son's parents evening".
    let thing = _the_possessive_a_relation_means(text::strip(found.name_str("thing")));
    let when = text::strip(found.name_str("when"));
    if thing.is_empty()
        || matches!(
            first_word(&thing).to_lowercase().as_str(),
            "i" | "im" | "i'm" | "we" | "were" | "we're" | "my" | "our" | "to" | "it" | "its" | "he" | "she" | "they" | "you"
        )
    {
        return String::new();
    }
    // The day is a name: "on tuesday" is said back as "on Tuesday".
    let when = named_days(when);
    let head = first_word(&thing).to_lowercase();
    if _a_doing(&thing) {
        // Something they are doing keeps its own words and takes the doing frame.
        return text::upper_first(&format!("You're planning to {} {when}", lower_first(&thing)));
    }
    if head == "a" || head == "an" {
        // The plan kept only the object: what the store has is a thing of theirs on a day.
        return text::upper_first(&format!("You've got {} {when}", lower_first(&thing)));
    }
    let is = if _has_its_own_verb(&thing) { "" } else { " is" };
    if WHOSE_IT_IS.contains(&head) {
        // The thing already says whose it is.
        return text::upper_first(&format!("{thing}{is} {when}"));
    }
    // A thing that brings its own verb keeps the day after it, with no "is" in between.
    text::upper_first(&format!("Your {thing}{is} {when}"))
}

/// Python: bot/memory/__init__.py:884 `_a_journey_line(said_line)`
/// True when the line the plan came from puts the person on their way somewhere.
pub fn _a_journey_line(said_line: &str) -> bool {
    JOURNEY_LINE.is_match(said_line)
}

/// Python: bot/memory/__init__.py:894 `plan_sentence(plan, said_line=None)`
/// A plan said back, with the day a name: "in june" -> "in June". `said_line` "" is None.
pub fn plan_sentence(plan: &str, said_line: &str) -> String {
    named_days(&_the_plan_said_back(plan, said_line))
}

/// Python: bot/memory/__init__.py:899 `_the_plan_said_back(plan, said_line=None)`
/// A plan said back to the person: their own sentence when they said one ("You start your new
/// job on monday"), the phrase after "you're planning" when it is only the thing itself, and
/// the sentence the phrase came from (`said_line`) when the phrase alone would read broken.
pub fn _the_plan_said_back(plan: &str, said_line: &str) -> String {
    let cleaned = filler::without_filler(&text::squash(plan));
    let mut text_ = text::strip_chars(&cleaned, " .!?,").to_string();
    // The words that only lead in to the thing are not part of the plan.
    text_ = text::strip_chars(&PLAN_LEAD_IN.sub("", &text_), " .!?,").to_string();
    // The sentence below adds its own "to".
    text_ = LEADING_TO.sub("", &text_);
    if text_.is_empty() {
        return String::new();
    }
    // The line it came from, when it is a sentence of their own.
    let toned = facts::no_tone(&filler::without_filler(&text::squash(said_line)));
    let said_sentence = text::strip_chars(&toned, " .!?,").to_string();
    let theirs = !said_sentence.is_empty()
        && (PLAN_OPENS.contains(&first_word(&said_sentence).to_lowercase()) || JOINED_SUBJECT.match_(&said_sentence).is_some());
    // A journey is not an appointment, and the person is the subject; a plan kept with a
    // capital that is not an article is a place or a sentence of their own, not a thing.
    let appointment =
        if _a_journey_line(said_line) || _a_name_at_the_front(&text_) { String::new() } else { appointment_sentence(&text_) };
    if !appointment.is_empty() {
        if theirs && said_sentence.to_lowercase().contains(&text_.to_lowercase()) {
            let said = _mid(&you(&said_sentence));
            return text::upper_first(text::rstrip_chars(&said, " ."));
        }
        return appointment;
    }
    // "to run 3 times a week" is already the phrase "planning to" wants.
    text_ = LEADING_TO.sub("", &text_);
    // "plan to visit my mum" names the plan in the same words "planning to visit" does.
    text_ = PLANS_TO.sub("planning to ", &text_);
    let first = text::strip_chars(&first_word(&text_).to_lowercase(), ",.;:").to_string();
    if PLAN_VERBS.contains(&first) {
        // A plan that already says what they are doing with it keeps its own verb.
        let phrase = _mid(&you(&text_));
        return text::rstrip_chars(&format!("You're {}", lower_first(&phrase)), " .").to_string();
    }
    if PLAN_OPENS.contains(&first) || JOINED_SUBJECT.match_(&text_).is_some() {
        let said = IM_IVE_ILL.sub_fn(&text_, |m| format!("i'{}", text::slice(m.as_str(), 1, None)));
        let said = WERE.sub("we're", &said);
        let said = WEVE.sub("we've", &said);
        let out = _mid(&you(&said));
        return text::upper_first(text::rstrip_chars(&out, " ."));
    }
    if !said_sentence.is_empty() && theirs {
        // The line it came from says what they are doing with the thing, which the bare phrase
        // cannot.
        return plan_sentence(&said_sentence, "");
    }
    let phrase = lower_first(&you(&text_));
    if first.ends_with("ing") && text::len(&first) > 4 {
        // A plan that keeps its verb is said back as the person's own doing.
        return text::rstrip_chars(&format!("You're {phrase}"), " .").to_string();
    }
    // A place they are off to, not an activity; a place word used as a thing is not a place.
    if DETERMINERS.contains(&first)
        || COUNTRIES.contains(&first)
        || NOT_PLACES.contains(&first)
        || (_places().contains(&first) && !_not_places().contains(&first))
    {
        return text::rstrip_chars(&format!("You're planning {phrase}"), " .").to_string();
    }
    // A bare verb phrase reads as a plan only with "to"; a plan kept with its own "to" keeps one.
    if phrase.to_lowercase().starts_with("to ") {
        return text::rstrip_chars(&format!("You're planning {phrase}"), " .").to_string();
    }
    text::rstrip_chars(&format!("You're planning to {phrase}"), " .").to_string()
}

/// Python: bot/memory/__init__.py:987 `_mid(text)`
/// A "You" that ends up in the middle of a sentence goes back to lower case.
pub fn _mid(text_: &str) -> String {
    MID_YOU.sub("you", text_)
}

/// Python: bot/memory/__init__.py:1036 `_to_the_bot(text)`
/// Mark the "you" that is the bot in a line about the person's own doing.
pub fn _to_the_bot(text_: &str) -> String {
    let words = LOWER_WORDS.findall_str(&text_.to_lowercase());
    let head = words.iter().find(|word| SUBJECT_HEADS.contains(*word)).map(String::as_str).unwrap_or("");
    if !FIRST_PERSON_HEADS.contains(head) {
        return text_.to_string();
    }
    WORD_THEN_YOU.sub_fn(text_, |m| {
        let word = m.str(2);
        let before = m.str(1).to_lowercase();
        if !SAYS_TO_A_PERSON.contains(&before) && !DONE_TO_A_PERSON.contains(&before) {
            return m.as_str().to_string();
        }
        // "thank you" is the courtesy, and a thank you card or note is a thing: only "thank you
        // for" and "thank you again" name who it goes to.
        if before == "thank" || before == "thanks" {
            let after = NEXT_WORD.match_(&m.text[m.end()..]);
            if !after.is_some_and(|a| matches!(a.str(1).to_lowercase().as_str(), "for" | "again" | "so" | "too")) {
                return m.as_str().to_string();
            }
        }
        let marker: &str = if word.to_lowercase() == "your" { &_BOT_MY } else { &_BOT_ME };
        format!("{} {}", m.str(1), if opens_upper(word) { marker.to_uppercase() } else { marker.to_string() })
    })
}

/// Python: bot/memory/__init__.py:1059 `_from_the_bot(text)`
/// The marked words go back to the bot's own person.
pub fn _from_the_bot(text_: &str) -> String {
    text_.replace(_BOT_ME.as_str(), "me").replace(_BOT_MY.as_str(), "my")
}

/// Python: bot/memory/__init__.py:1063 `_places()`
pub fn _places() -> &'static HashSet<String> {
    &TOPIC_PLACES
}

/// Python: bot/memory/__init__.py:1071 `_not_places()`
pub fn _not_places() -> &'static HashSet<String> {
    &TOPIC_NOT_PLACES
}

/// Python: bot/memory/__init__.py:1109 `_outside_quotes(text, repair)`
/// `repair` over the parts of the line that are the person's own words, quotes left alone.
pub fn _outside_quotes(text_: &str, repair: impl Fn(&str) -> String) -> String {
    if text_.is_empty() || !QUOTED.is_match(text_) {
        return repair(text_);
    }
    let mut out = String::new();
    let mut last = 0;
    for found in QUOTED.finditer(text_) {
        out.push_str(&repair(&text_[last..found.start()]));
        out.push_str(found.as_str());
        last = found.end();
    }
    out.push_str(&repair(&text_[last..]));
    out
}

/// Python: bot/memory/__init__.py:1122 `without_the_apostrophe(text)`
/// The person's words with the apostrophes they left out put back: "shes 72" -> "she's 72".
pub fn without_the_apostrophe(text_: &str) -> String {
    if text_.is_empty() {
        return String::new();
    }
    _outside_quotes(text_, |part| {
        APOSTROPHE_WORD.sub_fn(part, |m| {
            let word = m.as_str();
            let other = APOSTROPHE.get(&word.to_lowercase()).map(String::as_str).unwrap_or(word);
            if opens_upper(word) { text::upper_first(other) } else { other.to_string() }
        })
    })
}

/// Python: bot/memory/__init__.py:1165 `_the_relation_possessive(text)`
/// The "s" of a relation with something after it becomes the possessive it means: "my mums
/// birthday" -> "my mum's birthday". Only a relation the store knows, and only when the word
/// after it is a thing it could own.
pub fn _the_relation_possessive(text_: &str) -> String {
    RELATION_THEN_THING.sub_fn(text_, |m| {
        let (whole, relation, thing) = (m.as_str(), m.str(2), m.str(3));
        let one = &relation[..relation.len() - 1];
        if !RELATIONS.contains(&one.to_lowercase()) || NOT_OWNED.contains(&thing.to_lowercase()) {
            return whole.to_string();
        }
        if WHEN_OR_WHERE.contains(&thing.to_lowercase()) && _with_somebody(text_, m.start()) {
            return whole.to_string();
        }
        format!("{} {one}'s {thing}", m.str(1))
    })
}

/// Python: bot/memory/__init__.py:1183 `_the_possessive_a_relation_means(phrase)`
/// The possessive a relation means when its determiner is not in the phrase: "sons parents
/// evening" -> "son's parents evening".
pub fn _the_possessive_a_relation_means(phrase: &str) -> String {
    let stripped = text::strip(phrase);
    let Some(m) = BARE_RELATION.match_(stripped) else { return phrase.to_string() };
    let (relation, owned) = (m.str(1), m.str(2));
    let one = &relation[..relation.len() - 1];
    if !RELATIONS.contains(&one.to_lowercase()) || NOT_OWNED.contains(&owned.to_lowercase()) {
        return phrase.to_string();
    }
    if WHEN_OR_WHERE.contains(&owned.to_lowercase()) {
        return phrase.to_string();
    }
    format!("{one}'s {}", text::lstrip(&stripped[relation.len()..]))
}

/// Python: bot/memory/__init__.py:1207 `_with_somebody(text, start)`
/// True when the relation is somebody the line says they are with or going to see. `start` is
/// a byte offset here (a character index in Python).
pub fn _with_somebody(text_: &str, start: usize) -> bool {
    text::split_ref(&text_[..start]).last().is_some_and(|word| WITH_SOMEBODY.contains(&word.to_lowercase()))
}

/// Python: bot/memory/__init__.py:1218 `you(text)`
/// A fact in the user's own words, reworded to say it back to them: "my daughter's birthday"
/// -> "your daughter's birthday". A quotation in the line is left as it was said.
pub fn you(text_: &str) -> String {
    _outside_quotes(text_, |part| _flip_to_the_second_person(&_the_relation_possessive(part)))
}

/// Python: bot/memory/__init__.py:1227 `_flip_to_the_second_person(text)`
/// `you`'s reading, over one stretch of the line: the first person becomes the second.
pub fn _flip_to_the_second_person(text_: &str) -> String {
    // "me" written for "my" is "your", not "you": only "me" in front of a thing it could own.
    let mut text_ = _the_relation_possessive(text_);
    text_ = _to_the_bot(&text_);
    let words = LOWER_WORDS.findall_str(&text_.to_lowercase());
    let mine = crate::understanding::normalize::possessive_me(&words);
    let mut owns: HashSet<usize> = words
        .iter()
        .enumerate()
        .filter(|(_, word)| *word == "me")
        .map(|(index, _)| index)
        .enumerate()
        .filter(|(_, index)| *index < mine.len() && mine[*index] == "my")
        .map(|(place, _)| place)
        .collect();
    // A comma after it is a list of people, not a thing owned: "me, mum and dad went to spain".
    for (place, found) in ME.finditer(&text_).iter().enumerate() {
        if THEN_A_COMMA.match_(&text_[found.end()..]).is_some() {
            owns.remove(&place);
        }
    }
    if !owns.is_empty() {
        let mut seen = 0usize;
        text_ = ME.sub_fn(&text_, |m| {
            let place = seen;
            seen += 1;
            let word = m.as_str();
            if owns.contains(&place) {
                return if opens_upper(word) { "Your".to_string() } else { "your".to_string() };
            }
            word.to_string()
        });
    }
    text_ = FIRST_PERSON.sub_fn(&text_, |m| {
        let word = m.as_str();
        let low = word.to_lowercase();
        let other = SWAPS.get(low.as_str()).copied().unwrap_or(word);
        if opens_upper(word) { text::upper_first(other) } else { other.to_string() }
    });
    without_the_apostrophe(&_from_the_bot(&text_))
}

/// Python: bot/memory/__init__.py:1288 `_statement(text)`
/// The statement half of a line, or "" when the line is a question.
pub fn _statement(text_: &str) -> String {
    let flat = text::squash(text_);
    let mut line = text::strip_chars(&flat, " .!?,");
    let words = text::split_ref(line);
    if !words.is_empty()
        && ASK_WORDS.contains(text::strip_chars(&words[0].to_lowercase(), ",.;:"))
        && WERE_DOING.match_(line).is_none()
    {
        return String::new();
    }
    if let Some(cut) = ASK_TAIL.search(line) {
        line = &line[..cut.start()];
    }
    text::strip_chars(line, " .,!?").to_string()
}

/// Python: bot/memory/__init__.py:1301 `plan_line(memory, plan)`
/// The line the person said that a plan came from, if it names that plan; "" is Python's None.
/// Only the statement part of that line is a plan said back.
pub fn plan_line(memory: &mut Memory, plan: &str) -> String {
    let wanted = text::squash(&plan.to_lowercase());
    if wanted.is_empty() {
        return String::new();
    }
    for entry in log(memory).items().iter().rev() {
        let key = pyv::get(entry, "key").as_str().unwrap_or("");
        if key != "plan" && key != "plans" {
            continue;
        }
        if text::squash(&pyv::str_or_empty(pyv::get(entry, "value")).to_lowercase()) != wanted {
            continue;
        }
        let said_line = _statement(&pyv::str_or_empty(pyv::get(entry, "text")));
        if !said_line.is_empty() {
            return said_line;
        }
    }
    String::new()
}

/// Python: bot/memory/__init__.py:1321 `said(memory, key)`
/// The first value the user gave for a fact key, as (value, turn) or (None, None).
pub fn said(memory: &mut Memory, key: &str) -> (Value, Value) {
    for entry in log(memory).items() {
        if pyv::get(entry, "key").as_str() == Some(key) {
            return (pyv::get(entry, "value").clone(), pyv::get(entry, "turn").clone());
        }
    }
    (Value::None, Value::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value as J;

    /// A call that reaches a function another workstream has not ported yet is not a failure
    /// of this file: None.
    fn ported<T>(f: impl FnOnce() -> T + std::panic::UnwindSafe) -> Option<T> {
        match std::panic::catch_unwind(f) {
            Ok(v) => Some(v),
            Err(e) => {
                let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                assert!(msg.starts_with("UNPORTED"), "panicked: {msg}");
                None
            }
        }
    }

    /// CPython's values for the store's functions, recorded by `harness/mem6_init_cases.py`.
    #[test]
    fn matches_python() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/harness/mem6_init_cases.json");
        let cases: Vec<J> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let (mut checked, mut skipped) = (0, 0);
        for case in &cases {
            let (name, args, want) = (case[0].as_str().unwrap(), case[1].as_array().unwrap(), &case[2]);
            let a = |i: usize| args[i].as_str().unwrap().to_string();
            let got: Option<J> = if let Some(scenario) = name.strip_prefix("scen:") {
                ported(|| {
                    let mut memory = Memory::new();
                    memory.turns.push(Turn { user: "hi".into(), bot: "yo".into(), module: Some("chat".into()) });
                    let mut rets = Vec::new();
                    for step in args {
                        let (func, a) = (step[0].as_str().unwrap(), step[1].as_array().unwrap());
                        let s = |i: usize| a.get(i).and_then(J::as_str).unwrap_or("");
                        let v = |i: usize| a.get(i).map(Value::from_plain).unwrap_or(Value::None);
                        let ret = match func {
                            "remember" => {
                                remember(&mut memory, s(0), &v(1), s(2));
                                J::Null
                            }
                            "remember_other" => {
                                remember_other(&mut memory, &v(0), s(1));
                                J::Null
                            }
                            "forget" => Value::from(forget(&mut memory, s(0), &v(1), s(2))).to_plain(),
                            "plan_line" => match plan_line(&mut memory, s(0)) {
                                line if line.is_empty() => J::Null,
                                line => J::String(line),
                            },
                            "said" => {
                                let (value, turn) = said(&mut memory, s(0));
                                J::Array(vec![value.to_plain(), turn.to_plain()])
                            }
                            "name" => J::String(name_of(&mut memory)),
                            "last_user" => last_user(&memory, a[0].as_i64().unwrap()).to_plain(),
                            "last_bot" => J::String(last_bot(&memory)),
                            "last_module" => serde_json::json!(last_module(&memory)),
                            other => panic!("no such step {other}"),
                        };
                        rets.push(ret);
                    }
                    serde_json::json!({"mem": memory.to_plain(), "rets": rets})
                })
                .inspect(|_| eprintln!("scenario {scenario} checked"))
            } else {
                ported(|| match name {
                    "you" => J::String(you(&a(0))),
                    "plan_sentence" => J::String(plan_sentence(&a(0), &a(1))),
                    "_statement" => J::String(_statement(&a(0))),
                    "reads_as_a_plan" => J::Bool(reads_as_a_plan(&a(0), &a(1))),
                    "_said_another" => J::Bool(_said_another(&a(0), &a(1))),
                    "named_days" => J::String(named_days(&a(0))),
                    other => panic!("no such function {other}"),
                })
            };
            match got {
                Some(got) => {
                    assert_eq!(&got, want, "{name} {args:?}");
                    checked += 1;
                }
                None => skipped += 1,
            }
        }
        eprintln!("bot.memory: {checked} cases match CPython, {skipped} wait on an unported function");
        assert!(checked > 0);
    }

    fn name_of(memory: &mut Memory) -> String {
        name(memory)
    }

    #[test]
    fn recent_is_pythons_slice() {
        let mut memory = Memory::new();
        for i in 0..4 {
            memory.turns.push(Turn { user: i.to_string(), ..Turn::default() });
        }
        assert!(recent(None, 8).is_empty());
        assert_eq!(recent(Some(&memory), 2).len(), 2);
        assert_eq!(recent(Some(&memory), 2)[0].user, "2");
        assert_eq!(recent(Some(&memory), 9).len(), 4);
        assert_eq!(recent(Some(&memory), 0).len(), 4); // turns[-0:]
        assert_eq!(recent(Some(&memory), -1).len(), 3); // turns[1:]
    }
}
