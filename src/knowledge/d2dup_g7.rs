//! Helpers the shopping, clothing and work ports need from code other workstreams own, ported
//! privately until the owners' ports are merged (each marked `// DUP(<python module>::<name>)`),
//! plus a few Python-behaviour shims those three modules share.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

// ---------------------------------------------------------------------------------------------
// Python shims

/// `str(understood.get(key) or "")` for a reading key: a list field that is not empty prints as
/// Python's list repr, a key the reading does not carry is "".
pub fn py_field(u: &Understood, key: &str) -> String {
    fn list(v: &[String]) -> String {
        if v.is_empty() { String::new() } else { crate::knowledge::d2_shared::py_repr_list(v) }
    }
    fn opt(v: &Option<String>) -> String {
        v.clone().unwrap_or_default()
    }
    match key {
        "text" => u.text.clone(),
        "clean" => u.clean.clone(),
        "plain" => u.plain.clone(),
        "norm_text" => u.norm_text.clone(),
        "words" => list(&u.words),
        "norm" => list(&u.norm),
        "keywords" => list(&u.keywords),
        "topic" => opt(&u.topic),
        "subject" => opt(&u.subject),
        "kind" => u.kind.clone(),
        "intent" => u.intent.clone(),
        _ => String::new(),
    }
}

/// A regex built from a pattern made at run time (`re.search(r"\b" + re.escape(w) + r"\b", s)`),
/// compiled once per distinct pattern, as Python's own `re` cache does.
pub fn cached_re(pattern: &str, flags: u32) -> &'static Re {
    static CACHE: LazyLock<Mutex<HashMap<(String, u32), &'static Re>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
    let mut cache = CACHE.lock().unwrap();
    if let Some(r) = cache.get(&(pattern.to_string(), flags)) {
        return r;
    }
    let r: &'static Re = Box::leak(Box::new(Re::new(pattern, flags)));
    cache.insert((pattern.to_string(), flags), r);
    r
}

/// `template.format(name=value, ...)` for templates that use plain `{name}` fields (and `{{`/`}}`).
pub fn py_format(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;
    while let Some(i) = rest.find(['{', '}']) {
        out.push_str(&rest[..i]);
        let c = rest.as_bytes()[i];
        let after = &rest[i + 1..];
        if c == b'{' && after.starts_with('{') {
            out.push('{');
            rest = &after[1..];
        } else if c == b'}' && after.starts_with('}') {
            out.push('}');
            rest = &after[1..];
        } else if c == b'{' {
            let end = after.find('}').unwrap_or_else(|| panic!("format: unclosed field in {template:?}"));
            let name = &after[..end];
            match fields.iter().find(|(k, _)| *k == name) {
                Some((_, v)) => out.push_str(v),
                None => panic!("format: KeyError {name:?}"),
            }
            rest = &after[end + 1..];
        } else {
            panic!("format: single '}}' in {template:?}");
        }
    }
    out.push_str(rest);
    out
}

/// `template % (args...)` for templates made of `%s` / `%d` and `%%`.
pub fn py_percent(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len() + 32);
    let mut args = args.iter();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('%') => out.push('%'),
            Some('s') | Some('d') => out.push_str(args.next().expect("not enough arguments for format string")),
            other => panic!("unsupported format %{other:?}"),
        }
    }
    out
}

/// A str value as Python's `str(x or "")` would give it for the plain values the store keeps.
pub fn value_str(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::None) => String::new(),
        Some(Value::Str(s)) => s.to_string(),
        Some(Value::Int(i)) if *i != 0 => i.to_string(),
        Some(Value::Bool(true)) => "True".to_string(),
        Some(Value::Float(f)) if *f != 0.0 => text::float_repr(*f),
        Some(x) if !x.truthy() => String::new(),
        Some(x) => format!("{x:?}"),
    }
}

/// Python `a == b` on stored values (dicts compare without regard to order; a list never
/// equals a tuple).
pub fn py_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::None, Value::None) => true,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(_) | Value::Int(_) | Value::Float(_), Value::Bool(_) | Value::Int(_) | Value::Float(_)) => {
            a.as_f64().or(a.as_int().map(|i| i as f64)) == b.as_f64().or(b.as_int().map(|i| i as f64))
        }
        (Value::List(x), Value::List(y)) | (Value::Tuple(x), Value::Tuple(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| py_eq(p, q))
        }
        (Value::Set(x), Value::Set(y)) => {
            x.len() == y.len() && x.iter().all(|p| y.iter().any(|q| py_eq(p, q)))
        }
        (Value::Dict(x), Value::Dict(y)) => {
            x.len() == y.len()
                && x.iter().all(|(k, v)| y.iter().any(|(k2, v2)| py_eq(k, k2) && py_eq(v, v2)))
        }
        _ => false,
    }
}

/// `isinstance(x, int)` (bools count, as in Python) and its value.
pub fn py_int(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Int(i)) => Some(*i),
        Some(Value::Bool(b)) => Some(*b as i64),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------
// bot/memory/__init__.py

// DUP(bot.memory::recent)
/// The last n turns, oldest first.
pub fn recent(memory: &Memory, n: usize) -> &[Turn] {
    let t = &memory.turns;
    &t[t.len().saturating_sub(n)..]
}

// DUP(bot.memory::last_bot)
/// The bot's last reply.
pub fn last_bot(memory: &Memory) -> String {
    recent(memory, 1).last().map(|t| t.bot.clone()).unwrap_or_default()
}

// DUP(bot.memory::others)
/// Facts about other people the user mentioned (`memory["others"]`, made a list when missing).
pub fn others(memory: &mut Memory) -> Vec<Value> {
    match memory.extra.get("others") {
        Some(Value::List(v)) => v.to_vec(),
        _ => {
            memory.extra.insert("others".to_string(), Value::List(Arc::new(Vec::new())));
            Vec::new()
        }
    }
}

// ---------------------------------------------------------------------------------------------
// bot/memory/thread.py

// DUP(bot.memory.thread::user_lines)
/// The user's earlier lines, oldest first (the texts; the reading Python pairs them with is not
/// read by these callers).
pub fn user_lines(memory: &Memory) -> Vec<String> {
    memory.turns.iter().filter(|t| !text::strip(&t.user).is_empty()).map(|t| t.user.clone()).collect()
}

// ---------------------------------------------------------------------------------------------
// bot/memory/topics.py

static TOPICS_WORD: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.WORD"));
static WHO_SAID: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.WHO_SAID"));

// was DUP(bot.memory.topics::words): now the real port
pub use crate::memory::topics::words;

// was DUP(bot.memory.topics::person): now the real port
pub use crate::memory::topics::person;

// ---------------------------------------------------------------------------------------------
// bot/memory/people.py

static HIM: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.people.HIM"));
static HER: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.people.HER"));
static HIM_SAID: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.people.HIM_SAID"));
static HER_SAID: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.people.HER_SAID"));
static THEM_SAID: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.people.THEM_SAID"));
static CHILD_OF_THE_FAMILY: LazyLock<HashSet<String>> =
    LazyLock::new(|| data::set("bot.memory.people.CHILD_OF_THE_FAMILY"));

fn who(record: &Value) -> String {
    value_str(record.get("who")).to_lowercase()
}

// DUP(bot.memory.people::_the_age_of_the_group)
pub fn _the_age_of_the_group(memory: &Memory) -> Option<i64> {
    let theirs = memory.extra.get("their_age")?;
    if !matches!(theirs, Value::Dict(_)) {
        return None;
    }
    let age = py_int(theirs.get("age"))?;
    if 0 < age && age < 120 { Some(age) } else { None }
}

// DUP(bot.memory.people::_with_an_age)
/// True for a person the store knows the age of.
pub fn _with_an_age(record: &Value) -> bool {
    if !matches!(record, Value::Dict(_)) {
        return false;
    }
    if py_int(record.get("age")).is_some_and(|a| a > 0) {
        return true;
    }
    record.get("ages").map(|a| a.items().iter().any(|x| py_int(Some(x)).is_some_and(|a| a > 0))).unwrap_or(false)
}

// DUP(bot.memory.people::_age)
/// The age the store holds for a person: the one number, or the first of a list.
pub fn _age(record: &Value) -> Option<i64> {
    if let Some(a) = py_int(record.get("age")) {
        return Some(a);
    }
    record.get("ages").and_then(|a| a.items().iter().find_map(|x| py_int(Some(x))))
}

fn reading_line(u: Option<&Understood>) -> String {
    match u {
        Some(u) => crate::knowledge::d2_shared::or(&u.clean, &u.text).to_string(),
        None => String::new(),
    }
}

// DUP(bot.memory.people::a_person_with_an_age)
/// The person the line is about, from the store, or None.
pub fn a_person_with_an_age(memory: &mut Memory, understood: Option<&Understood>) -> Option<Value> {
    let line = reading_line(understood);
    let people: Vec<Value> =
        others(memory).into_iter().filter(|one| matches!(one, Value::Dict(_)) && one.get("who").is_some_and(|w| w.truthy())).collect();
    if people.is_empty() {
        return _the_group_record(memory, &line);
    }
    let named = person(&line);
    if let Some(named) = &named {
        for one in &people {
            if &who(one) == named && _with_an_age(one) {
                return Some(one.clone());
            }
        }
        return None;
    }
    let said_him = HIM_SAID.is_match(&line);
    let said_her = HER_SAID.is_match(&line);
    if said_him == said_her && THEM_SAID.is_match(&line) {
        return _the_ones_a_them_points_at(&people);
    }
    if said_him == said_her {
        return _the_group_record(memory, &line);
    }
    let wanted: &HashSet<String> = if said_him { &HIM } else { &HER };
    let found: Vec<&Value> = people.iter().filter(|one| wanted.contains(&who(one)) && _with_an_age(one)).collect();
    if found.len() != 1 {
        return None;
    }
    Some(found[0].clone())
}

// DUP(bot.memory.people::_the_group_record)
pub fn _the_group_record(memory: &Memory, line: &str) -> Option<Value> {
    if !THEM_SAID.is_match(line) {
        return None;
    }
    let age = _the_age_of_the_group(memory)?;
    Some(Value::Dict(Arc::new(vec![
        (Value::Str(Arc::from("who")), Value::Str(Arc::from("them"))),
        (Value::Str(Arc::from("age")), Value::Int(age)),
    ])))
}

// DUP(bot.memory.people::_the_ones_a_them_points_at)
pub fn _the_ones_a_them_points_at(people: &[Value]) -> Option<Value> {
    let with_an_age: Vec<&Value> = people.iter().filter(|one| _with_an_age(one)).collect();
    if with_an_age.is_empty() {
        return None;
    }
    let ages: HashSet<Option<i64>> = with_an_age.iter().map(|one| _age(one)).collect();
    if ages.len() != 1 {
        return None;
    }
    Some(with_an_age[0].clone())
}

// DUP(bot.memory.people::the_people)
/// Every person the store keeps for this chat: `memory["others"]` plus the one in hand.
pub fn the_people(memory: &mut Memory) -> Vec<Value> {
    let mut found: Vec<Value> =
        others(memory).into_iter().filter(|one| matches!(one, Value::Dict(_)) && one.get("who").is_some_and(|w| w.truthy())).collect();
    if let Some(one) = memory.extra.get("person") {
        if matches!(one, Value::Dict(_))
            && one.get("who").is_some_and(|w| w.truthy())
            && !found.iter().any(|f| py_eq(f, one))
        {
            found.push(one.clone());
        }
    }
    found
}

// DUP(bot.memory.people::a_child_the_chat_is_about)
/// True when the person this chat is about is a child of the family, with or without an age.
pub fn a_child_the_chat_is_about(memory: &mut Memory, understood: Option<&Understood>) -> bool {
    let line = reading_line(understood);
    let people = the_people(memory);
    if let Some(named) = person(&line) {
        if !CHILD_OF_THE_FAMILY.contains(&named) {
            return false;
        }
        for one in &people {
            if who(one) != named {
                continue;
            }
            if _age(one).is_some_and(|a| a >= 18) {
                return false;
            }
        }
        return true;
    }
    let said_him = HIM_SAID.is_match(&line);
    let said_her = HER_SAID.is_match(&line);
    let wanted: Option<&HashSet<String>> = if said_him && !said_her {
        Some(&HIM)
    } else if said_her && !said_him {
        Some(&HER)
    } else {
        None
    };
    let mut ages: HashMap<String, i64> = HashMap::new();
    for one in &people {
        if let Some(age) = _age(one) {
            ages.insert(who(one), age);
        }
    }
    if let Some(wanted) = wanted {
        let of_that_kind: HashSet<String> = people.iter().map(who).filter(|w| wanted.contains(w)).collect();
        if of_that_kind.len() > 1 {
            return false;
        }
    }
    for one in &people {
        let w = who(one);
        if !CHILD_OF_THE_FAMILY.contains(&w) {
            continue;
        }
        if let Some(wanted) = wanted {
            if !wanted.contains(&w) {
                continue;
            }
        }
        if ages.get(&w).is_some_and(|a| *a >= 18) {
            continue;
        }
        return true;
    }
    false
}

// DUP(bot.memory.people::the_age_of_the_person_they_mean)
/// How old the person this line is about is, from the store, or None.
pub fn the_age_of_the_person_they_mean(memory: &mut Memory, understood: Option<&Understood>) -> Option<i64> {
    if let Some(found) = a_person_with_an_age(memory, understood) {
        return _age(&found);
    }
    let line = reading_line(understood);
    if !THEM_SAID.is_match(&line) || HIM_SAID.is_match(&line) || HER_SAID.is_match(&line) {
        return None;
    }
    let held = memory.extra.get("their_age")?;
    if matches!(held, Value::Dict(_)) && _with_an_age(held) {
        return _age(held);
    }
    None
}

// ---------------------------------------------------------------------------------------------
// bot/memory/newborn.py

static BABY_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.newborn.BABY_WORDS"));
static A_BIRTH: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.newborn.A_BIRTH"));
static JUST_BORN: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.newborn.JUST_BORN"));
static A_YEAR: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.newborn.A_YEAR"));
static A_BIRTH_WEIGHT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.newborn.A_BIRTH_WEIGHT"));
static NOT_THE_BABY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.newborn.NOT_THE_BABY"));
static LETTER_WORDS: LazyLock<Re> = LazyLock::new(|| Re::new("[a-z']+", 0));

// DUP(bot.memory.newborn::_the_lines)
/// The last few turns' topic records as words, newest first.
pub fn _the_lines(memory: &Memory) -> Vec<String> {
    const HOW_MANY_TURNS: usize = 3;
    let mut lines: Vec<&Value> = Vec::new();
    if let Some(current @ Value::Dict(_)) = memory.extra.get("topic") {
        lines.push(current);
    }
    if let Some(topics) = memory.extra.get("topics") {
        let items = topics.items();
        for record in items[items.len().saturating_sub(HOW_MANY_TURNS)..].iter().rev() {
            if matches!(record, Value::Dict(_)) {
                lines.push(record);
            }
        }
    }
    let mut said = Vec::new();
    for record in lines {
        let words = record.get("words");
        let text = value_str(record.get("text"));
        match words {
            Some(Value::List(ws)) if !ws.is_empty() => {
                let mut parts = vec![text];
                parts.extend(ws.iter().map(|w| w.str().to_string()));
                said.push(parts.join(" "));
            }
            _ => said.push(text),
        }
    }
    said
}

// DUP(bot.memory.newborn::the_thread_has_a_new_baby)
/// True when the last few turns are about a baby arriving.
pub fn the_thread_has_a_new_baby(memory: &Memory, understood: Option<&Understood>) -> bool {
    let line = reading_line(understood);
    if !line.is_empty() && NOT_THE_BABY.is_match(&line) {
        return false;
    }
    let said = _the_lines(memory);
    if said.is_empty() {
        return false;
    }
    let together = said.join(" ");
    if NOT_THE_BABY.is_match(&together) {
        return false;
    }
    let words: HashSet<String> = LETTER_WORDS.findall_str(&together.to_lowercase()).into_iter().collect();
    if JUST_BORN.is_match(&together) && !A_YEAR.is_match(&together) {
        return true;
    }
    if words.iter().any(|w| BABY_WORDS.contains(w)) && (A_BIRTH.is_match(&together) || A_BIRTH_WEIGHT.is_match(&together)) {
        return true;
    }
    A_BIRTH_WEIGHT.is_match(&together) && A_BIRTH.is_match(&together)
}

// ---------------------------------------------------------------------------------------------
// bot/understanding/topics.py

fn topics_set(name: &str) -> HashSet<String> {
    data::set(&format!("bot.understanding.topics.{name}"))
}
static GIFT_ALWAYS: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIFT_ALWAYS"));
static GIFT_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIFT_WORDS"));
static GIFT_ASKING: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIFT_ASKING"));
static GIVE_VERB: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIVE_VERB"));
static GIFT_OCCASION: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIFT_OCCASION"));
static GIFT_WANTING: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIFT_WANTING"));
static GIVE_TO: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIVE_TO"));
static GIVE_THINGS: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIVE_THINGS"));
static GIVE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIVE_THING"));
static GIVE_DETERMINERS: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIVE_DETERMINERS"));
static GIVE_RELATIVE: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIVE_RELATIVE"));
static GIFTS_OR_OCCASIONS: LazyLock<HashSet<String>> = LazyLock::new(|| topics_set("GIFTS_OR_OCCASIONS"));

fn meets(words: &HashSet<&str>, set: &HashSet<String>) -> bool {
    words.iter().any(|w| set.contains(*w))
}

// was DUP(bot.understanding.topics::gift_ask): now the real port
pub use crate::understanding::topics::gift_ask;

// DUP(bot.understanding.topics::_gift_ask)
pub fn _gift_ask(words: &HashSet<&str>, norm: &[String]) -> bool {
    if meets(words, &GIFT_ALWAYS) {
        return true;
    }
    if meets(words, &GIFT_WORDS)
        && (meets(words, &GIFT_ASKING) || meets(words, &GIVE_VERB) || meets(words, &GIFT_OCCASION) || words.contains("for"))
    {
        return true;
    }
    if meets(words, &GIFT_WANTING) && _gift_still_to_get(words, norm) {
        return true;
    }
    if !(meets(words, &GIFT_ASKING) || meets(words, &GIFT_OCCASION)) {
        return false;
    }
    if meets(words, &GIFT_OCCASION) && !norm.is_empty() {
        for position in 0..norm.len() - 1 {
            if GIVE_TO.contains(&norm[position])
                && norm[position + 1..(position + 3).min(norm.len())].iter().any(|w| GIVE_THINGS.contains(w))
            {
                return true;
            }
        }
    }
    for (position, word) in norm.iter().enumerate() {
        if !GIVE_VERB.contains(word) {
            continue;
        }
        let rest = &norm[position + 1..];
        if rest.is_empty() {
            continue;
        }
        let head = &rest[0];
        if GIVE_THING.contains(head) || GIFT_WORDS.contains(head) || GIFT_OCCASION.contains(head) {
            return true;
        }
        let mut tail: Option<&[String]> = None;
        for (index, one) in rest.iter().take(2).enumerate() {
            if GIVE_TO.contains(one) {
                tail = Some(&rest[index + 1..]);
                break;
            }
            if GIVE_DETERMINERS.contains(one) && rest.get(index + 1).is_some_and(|n| GIVE_TO.contains(n)) {
                tail = Some(&rest[index + 2..]);
                break;
            }
        }
        let Some(tail) = tail else { continue };
        if tail.is_empty() || GIVE_RELATIVE.contains(&tail[0]) {
            return true;
        }
        if tail.iter().take(3).any(|one| GIFTS_OR_OCCASIONS.contains(one)) {
            return true;
        }
        if tail[0] == "for"
            && tail.iter().skip(1).take(3).any(|one| one.chars().any(|ch| text::is_digit(&ch.to_string())))
        {
            return true;
        }
    }
    false
}

// DUP(bot.understanding.topics::_gift_still_to_get)
pub fn _gift_still_to_get(words: &HashSet<&str>, norm: &[String]) -> bool {
    if meets(words, &GIVE_TO) && words.contains("for") && (words.contains("nothing") || words.contains("anything")) {
        return true;
    }
    for (position, word) in norm.iter().enumerate() {
        if !GIVE_VERB.contains(word) {
            continue;
        }
        let rest = &norm[position + 1..];
        let mut tail: Option<&[String]> = None;
        for (index, one) in rest.iter().take(2).enumerate() {
            if GIVE_TO.contains(one) {
                tail = Some(&rest[index + 1..]);
                break;
            }
            if GIVE_DETERMINERS.contains(one) && rest.get(index + 1).is_some_and(|n| GIVE_TO.contains(n)) {
                tail = Some(&rest[index + 2..]);
                break;
            }
        }
        let Some(tail) = tail else { continue };
        if tail.is_empty() {
            continue;
        }
        if GIVE_THING.contains(&tail[0]) || GIFT_WORDS.contains(&tail[0]) {
            return true;
        }
        if tail.iter().any(|w| w == "for") && (words.contains("nothing") || words.contains("anything")) {
            return true;
        }
        if tail.iter().take(3).any(|one| GIFT_OCCASION.contains(one)) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/lookup.py

static PEOPLE_MALE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.PEOPLE_MALE"));
static PEOPLE_FEMALE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.PEOPLE_FEMALE"));
static CONTAINERS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.CONTAINERS"));

// DUP(bot.knowledge.lookup::about_them)
/// `about_them(template)(memory, reading)` with the default people lists: the template with
/// he, she or they filled in according to the line, first letter capitalised.
pub fn about_them(template: &str, reading: &Understood) -> String {
    let mut they = "they";
    for word in &reading.words {
        if PEOPLE_MALE.iter().any(|w| w == word) {
            they = "he";
            break;
        }
        if PEOPLE_FEMALE.iter().any(|w| w == word) {
            they = "she";
            break;
        }
    }
    let plural = they == "they";
    let he = they == "he";
    let fills = [
        ("{they}", they),
        ("{them}", if plural { "them" } else if he { "him" } else { "her" }),
        ("{their}", if plural { "their" } else if he { "his" } else { "her" }),
        ("{self}", if plural { "themselves" } else if he { "himself" } else { "herself" }),
        ("{has}", if plural { "have" } else { "has" }),
        ("{is}", if plural { "are" } else { "is" }),
        ("{likes}", if plural { "like" } else { "likes" }),
        ("{loves}", if plural { "love" } else { "loves" }),
    ];
    let mut out = template.to_string();
    for (token, word) in fills {
        out = out.replace(token, word);
    }
    let first: String = out.chars().take(1).collect::<String>().to_uppercase();
    let rest: String = out.chars().skip(1).collect();
    first + &rest
}

// DUP(bot.knowledge.lookup::only_a_container)
/// True when every word of `names` the line says is a container a thing is kept in.
pub fn only_a_container<S: AsRef<str>>(understood: &Understood, names: &[S]) -> bool {
    let text = format!(" {} ", understood.words.iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join(" "));
    let found: Vec<&str> =
        names.iter().map(|n| n.as_ref()).filter(|one| text.contains(&format!(" {one} "))).collect();
    !found.is_empty() && found.iter().all(|one| CONTAINERS.contains(*one))
}

// ---------------------------------------------------------------------------------------------
// bot/knowledge/wording.py

struct Sign {
    phrases: Vec<String>,
    label: String,
}

static SIGNS: LazyLock<Vec<Sign>> = LazyLock::new(|| {
    data::get("bot.knowledge.wording.SIGNS")
        .items()
        .iter()
        .map(|row| Sign { phrases: row.items()[0].strs(), label: row.items()[1].str().to_string() })
        .collect()
});
static SIGN_PLURAL: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.wording._SIGN_PLURAL"));

// was DUP(bot.knowledge.wording::sign_said): now the real port
pub use crate::knowledge::wording::sign_said;

// was DUP(bot.knowledge.wording::nerves_opening): now the real port
pub use crate::knowledge::wording::nerves_opening;

// ---------------------------------------------------------------------------------------------
// bot/knowledge/home.py

// DUP(bot.knowledge.home::_dryer_heat)
/// A dryer that will not heat, said as cold air or as clothes that take three goes.
pub fn _dryer_heat(_memory: &mut Memory, reading: &Understood) -> Option<String> {
    const SLOW: [&str; 20] = [
        "ages", "forever", "slow", "slower", "slowly", "long", "longer", "hours", "again", "twice", "ages.", "slowly.",
        "goes", "go", "cycles", "cycle", "damp", "damp.", "wet", "wet.",
    ];
    if reading.words.iter().any(|w| SLOW.contains(&w.as_str())) {
        return Some(
            "Slow drying is nearly always the air: empty the fluff filter, rinse the condenser \
if it has one, and check the vent hose is not kinked. A drum stuffed full will not \
dry either, so try half a load. If the air still comes out cold, it is the fuse on \
the element - an engineer's job, cheap part."
                .to_string(),
        );
    }
    Some(
        "Nearly always the air or the setting, not a part: check the dial first, since most \
machines finish on a cool-down phase that heats nothing, then empty the fluff filter \
and check the vent hose is not kinked. If the drum turns and the air still comes out \
cold, it is the thermal fuse on the element - an engineer's job, cheap part."
            .to_string(),
    )
}
