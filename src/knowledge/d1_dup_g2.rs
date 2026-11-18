//! DUP ports (group 2 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.

use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn set_of(name: &str) -> HashSet<String> {
    data::get(name).str_set()
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup

static WEIGHT_UNITS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.knowledge.lookup.WEIGHT_UNITS"));

/// DUP(bot.knowledge.lookup::a_weight): true when the line gives a weight in kilos, grams or
/// pounds (a kitchen line, not a question about the animal or the place the noun also names).
pub fn a_weight(understood: &Understood) -> bool {
    understood
        .numbers
        .iter()
        .any(|number| WEIGHT_UNITS.contains(number.unit.as_deref().unwrap_or("").to_lowercase().as_str()))
}

// ---------------------------------------------------------------------------------------------
// bot.memory.topics / bot.memory.named: what the chat is on, read from the memory store's
// records (`memory["topic"]`, `memory["topics"]`, `memory["user"]`), which live in
// `Memory::extra` under their Python keys.

/// `memory.get(key)` from the store's own keys.
fn mem<'a>(memory: &'a Memory, key: &str) -> Option<&'a Value> {
    memory.extra.get(key)
}

/// `str(record.get(key) or "")` for a dict record.
fn rec_str<'a>(record: &'a Value, key: &str) -> &'a str {
    match record.get(key) {
        Some(Value::Str(s)) => s,
        _ => "",
    }
}

/// DUP(bot.memory.topics::context): the last n topic texts, oldest first. None where Python
/// would raise (a record that is not a dict), which the caller catches.
pub fn topics_context(memory: &Memory, n: usize) -> Option<Vec<String>> {
    let Some(Value::List(records)) = mem(memory, "topics") else {
        return Some(Vec::new());
    };
    let start = records.len().saturating_sub(n);
    let mut out = Vec::new();
    for r in &records[start..] {
        if !matches!(r, Value::Dict(_)) {
            return None; // r.get on a non-dict raises
        }
        // only truthy texts are kept; the store only ever writes a str there
        if let Some(Value::Str(s)) = r.get("text") {
            if !s.is_empty() {
                out.push(s.to_string());
            }
        }
    }
    Some(out)
}

/// DUP(bot.memory.topics::current): the current topic record, or None.
fn topics_current(memory: &Memory) -> Option<&Value> {
    let record = mem(memory, "topic")?;
    if matches!(record, Value::Dict(_)) && record.get("text").map(Value::truthy).unwrap_or(false) {
        return Some(record);
    }
    None
}

/// DUP(bot.memory.topics::topic_text).
fn topics_topic_text(memory: &Memory) -> String {
    match topics_current(memory) {
        Some(r) => rec_str(r, "text").to_string(),
        None => String::new(),
    }
}

static ROOM_LIMIT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics._ROOM_LIMIT"));
static HOME_SPACES: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.topics._HOME_SPACES"));

/// DUP(bot.memory.topics::_room_limit): the home space a line only says the size of, or "".
fn topics_room_limit(text_: &str) -> String {
    let words = text::squash(&text_.to_lowercase());
    let Some(found) = ROOM_LIMIT.match_(&words) else {
        return String::new();
    };
    let head = text::strip_chars(found.name_str("head"), " ,.;:!?-");
    if head.is_empty() {
        return String::new();
    }
    match text::split_ref(head).last() {
        Some(last) if HOME_SPACES.contains(*last) => head.to_string(),
        _ => String::new(),
    }
}

fn topics(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.topics.{name}"))
}
fn topics_re(name: &str) -> &'static Re {
    topics(name).as_re().unwrap_or_else(|| panic!("bot.memory.topics.{name} is not a regex"))
}

static WORD: LazyLock<&'static Re> = LazyLock::new(|| topics_re("WORD"));
static STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| topics("STOPWORDS").str_set());
static ASK_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| topics("ASK_ONLY").str_set());
static TIME_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| topics("_TIME_WORDS").str_set());
static A_FIGURE: LazyLock<HashSet<String>> = LazyLock::new(|| topics("_A_FIGURE").str_set());
static DATE_OCCASIONS: LazyLock<HashSet<String>> = LazyLock::new(|| topics("_DATE_OCCASIONS").str_set());
static DATE_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| topics("_DATE_FILLER").str_set());
static MAX_SUBJECT: LazyLock<usize> = LazyLock::new(|| topics("_MAX_SUBJECT").as_int().unwrap_or(4) as usize);
/// `sorted(SWITCHERS, key=len, reverse=True)` (stable: equal lengths keep their order).
static SWITCHERS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = topics("SWITCHERS").strs();
    v.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    v
});
/// `sorted(lexicon.HANDOFF_LEADS, key=len, reverse=True)`.
static HANDOFF_LEADS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let mut v = data::strs("bot.understanding.lexicon.HANDOFF_LEADS");
    v.sort_by(|a, b| b.chars().count().cmp(&a.chars().count()));
    v
});
/// `_the_people()`: facts.RELATIONS in the forms keywords keeps.
static THE_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| {
    data::strs("bot.memory.facts.RELATIONS").iter().flat_map(|w| [w.clone(), topics_stem(w)]).collect()
});
static CORRECTION_LEAD: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_CORRECTION_LEAD"));
static PHRASAL_RE: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_PHRASAL_RE"));
static RELATION: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_RELATION"));
static FRAME_ASK: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_FRAME_ASK"));
static WITHOUT_CLAUSE: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_WITHOUT_CLAUSE"));
static CONSIDERING: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_CONSIDERING"));
static BUYING_A_STATE: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_BUYING_A_STATE"));
static WISH_RE: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_WISH_RE"));
static LEAD: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_LEAD"));
static LEAD_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_LEAD_QUESTION"));
static HANDOFF_THROAT: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_HANDOFF_THROAT"));
static MAKING_A_THING: LazyLock<&'static Re> = LazyLock::new(|| topics_re("_MAKING_A_THING"));
static AND_OR_BUT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:and|but)\b|[,;]", 0));
static A_TIME_SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:for|tonight|today|tomorrow|yesterday|this|next)\b", 0));
static COMMA_OR_SEMI: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;]", 0));

/// DUP(bot.memory.topics::words): the words of a line, apostrophe forms folded.
pub fn topics_words(text_: &str) -> Vec<String> {
    let lowered = text_.to_lowercase().replace('\u{2019}', "'");
    WORD.findall_str(&lowered)
        .into_iter()
        .map(|w| {
            let w = w.strip_suffix("'s").map(str::to_string).unwrap_or(w);
            w.replace('\'', "")
        })
        .filter(|w| !w.is_empty())
        .collect()
}

/// DUP(bot.memory.topics::stem).
pub fn topics_stem(word: &str) -> String {
    let n = word.chars().count();
    let chars: Vec<char> = word.chars().collect();
    if n > 4 && word.ends_with("ies") {
        return format!("{}y", chars[..n - 3].iter().collect::<String>());
    }
    if n > 3 && word.ends_with("es") && "sxzh".contains(chars[n - 3]) {
        return chars[..n - 2].iter().collect();
    }
    if n > 3 && word.ends_with('s') && !word.ends_with("ss") {
        return chars[..n - 1].iter().collect();
    }
    word.to_string()
}

/// DUP(bot.memory.topics::keywords): the content words of a line, stemmed, first seen first.
pub fn topics_keywords(text_: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in topics_words(text_) {
        if STOPWORDS.contains(&word) || word.chars().count() < 3 || text::is_digit(&word) {
            continue;
        }
        let key = topics_stem(&word);
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}

/// DUP(bot.memory.topics::_strip_considering).
fn topics_strip_considering(source: &str) -> String {
    let mut source = source.to_string();
    loop {
        let stripped = CONSIDERING.sub("", &source);
        if stripped == source {
            return source;
        }
        source = stripped;
    }
}

/// DUP(bot.memory.topics::_only_a_handoff).
fn topics_only_a_handoff(head: &str) -> bool {
    let t = text::squash(&head.to_lowercase());
    let t = text::strip_chars(&t, " ,.;:!?-");
    let t = HANDOFF_THROAT.sub("", t);
    let t = text::strip_chars(&t, " ,.;:!?-");
    HANDOFF_LEADS
        .iter()
        .any(|lead| t == lead || (t.starts_with(lead.as_str()) && topics_keywords(&t[lead.len()..]).is_empty()))
}

/// DUP(bot.memory.topics::_a_word_of_a_date).
fn topics_a_word_of_a_date(word: &str) -> bool {
    let forms = [word.to_string(), topics_stem(word)];
    forms.iter().any(|f| DATE_OCCASIONS.contains(f))
        || forms.iter().any(|f| TIME_WORDS.contains(f))
        || DATE_FILLER.contains(word)
        || forms.iter().any(|f| A_FIGURE.contains(f))
        || forms.iter().any(|f| THE_PEOPLE.contains(f))
}

/// DUP(bot.memory.topics::_only_a_date).
fn topics_only_a_date(clause: &str) -> bool {
    let kept = topics_keywords(clause);
    if kept.is_empty() || !kept.iter().any(|w| DATE_OCCASIONS.contains(&topics_stem(w))) {
        return false;
    }
    kept.iter().all(|w| topics_a_word_of_a_date(w))
}

/// DUP(bot.memory.topics::_the_thing_after_a_date).
fn topics_the_thing_after_a_date(text_: &str) -> String {
    let Some(m) = AND_OR_BUT.search(text_) else {
        return String::new();
    };
    let (head, tail) = (&text_[..m.start()], &text_[m.end()..]);
    if !topics_only_a_date(head) || topics_keywords(tail).is_empty() {
        return String::new();
    }
    if MAKING_A_THING.is_match(tail) { tail.to_string() } else { String::new() }
}

/// DUP(bot.memory.topics::gist): the short noun phrase a line is really about ("birthday
/// cake", "kitchen tap", "turkey").
pub fn topics_gist(text_: &str) -> String {
    let mut source = text::squash(&text_.to_lowercase());
    source = CORRECTION_LEAD.sub("", &source);
    for switcher in SWITCHERS.iter() {
        if source.starts_with(switcher.as_str()) {
            source = text::lstrip_chars(&source[switcher.len()..], " ,.!-").to_string();
            break;
        }
    }
    let dated = topics_the_thing_after_a_date(&source);
    if !dated.is_empty() {
        source = dated;
    }
    source = text::squash(&PHRASAL_RE.sub(" ", &source));
    source = RELATION.split(&source).into_iter().next().unwrap_or_default();
    let whole = source.clone();
    if FRAME_ASK.is_match(&source) {
        let trimmed = WITHOUT_CLAUSE.sub("", &source);
        let trimmed = text::strip_chars(&trimmed, " ,.;:!?");
        if !trimmed.is_empty() && !topics_keywords(trimmed).is_empty() {
            source = trimmed.to_string();
        }
    }
    source = topics_strip_considering(&source);
    let buying = BUYING_A_STATE.match_(&source).is_some();
    let mut wished = false;
    for _ in 0..(if buying { 0 } else { 3 }) {
        let stripped = WISH_RE.sub("", &source);
        if stripped == source || text::strip_chars(&stripped, " ,.;:!?").is_empty() {
            break;
        }
        source = stripped;
        wished = true;
    }
    let before_split = source.clone();
    let segments = A_TIME_SPLIT.split(&source);
    source = segments.first().cloned().unwrap_or_default();
    if !wished && !buying {
        source = LEAD.sub("", text::strip_chars(&source, " ,.;:!?"));
        loop {
            let trimmed = LEAD_QUESTION.sub("", &source);
            if trimmed == source || text::strip(&trimmed).is_empty() {
                break;
            }
            source = trimmed;
        }
    }
    source = topics_strip_considering(&source);
    let (head, sep, after_handoff) = text::partition(&source, ",");
    source = if !sep.is_empty() && !text::strip(after_handoff).is_empty() && topics_only_a_handoff(head) {
        after_handoff.to_string()
    } else {
        COMMA_OR_SEMI.split(&source).into_iter().next().unwrap_or_default()
    };
    let mut kept: Vec<String> = topics_keywords(&source).into_iter().filter(|w| !ASK_ONLY.contains(w)).collect();
    while kept.last().map(|w| TIME_WORDS.contains(w)).unwrap_or(false) {
        kept.pop();
    }
    if !kept.is_empty() && kept.iter().all(|w| A_FIGURE.contains(w)) {
        for part in segments.iter().skip(1) {
            let later: Vec<String> = topics_keywords(part)
                .into_iter()
                .filter(|w| !ASK_ONLY.contains(w) && !TIME_WORDS.contains(w))
                .collect();
            if !later.is_empty() && !later.iter().all(|w| A_FIGURE.contains(w)) {
                kept = later;
                break;
            }
        }
    }
    if kept.is_empty() && wished {
        for source in [&before_split, &whole] {
            kept = topics_keywords(source).into_iter().filter(|w| !ASK_ONLY.contains(w) && !TIME_WORDS.contains(w)).collect();
            if !kept.is_empty() {
                break;
            }
        }
    }
    if kept.is_empty() {
        return String::new();
    }
    kept.truncate(*MAX_SUBJECT);
    kept.join(" ")
}

/// DUP(bot.memory.topics::_names_nothing).
fn topics_names_nothing(record: &Value) -> bool {
    if !text::squash(rec_str(record, "subject")).is_empty() {
        return false;
    }
    topics_gist(rec_str(record, "text")).is_empty()
}

/// DUP(bot.memory.topics::_about_record): the last topic that names something.
fn topics_about_record(memory: &Memory) -> Option<&Value> {
    let Some(Value::List(records)) = mem(memory, "topics") else {
        return topics_current(memory);
    };
    for record in records.iter().rev() {
        if !matches!(record, Value::Dict(_)) || !topics_room_limit(rec_str(record, "text")).is_empty() {
            continue;
        }
        if topics_names_nothing(record) {
            continue;
        }
        return Some(record);
    }
    topics_current(memory)
}

/// DUP(bot.memory.topics::_thing_of): the record a detail line points back at.
fn topics_thing_of<'a>(memory: &'a Memory, record: Option<&Value>) -> Option<&'a Value> {
    let record = record?;
    let back = rec_str(record, "of");
    if back.is_empty() {
        return None;
    }
    if let Some(Value::List(records)) = mem(memory, "topics") {
        for other in records.iter().rev() {
            if matches!(other, Value::Dict(_)) && rec_str(other, "text") == back {
                return Some(other);
            }
        }
    }
    None
}

/// DUP(bot.memory.topics::subject): what the conversation is about now, as a noun phrase.
pub fn topics_subject(memory: &Memory) -> String {
    let known = match mem(memory, "user") {
        Some(v @ Value::Dict(_)) => Some(v),
        _ => None,
    };
    let mut record = topics_about_record(memory);
    if let Some(thing) = topics_thing_of(memory, record) {
        record = Some(thing);
    }
    let subject = record.map(|r| text::squash(rec_str(r, "subject"))).unwrap_or_default();
    let now = if !subject.is_empty() {
        subject
    } else {
        let t = record.map(|r| rec_str(r, "text")).unwrap_or("");
        let t = if t.is_empty() { topics_topic_text(memory) } else { t.to_string() };
        topics_gist(&t)
    };
    let planned = topics_gist(known.map(|k| rec_str(k, "plan")).unwrap_or(""));
    if !planned.is_empty() && !now.is_empty() && now.contains(&planned) {
        return planned;
    }
    if now.is_empty() { planned } else { now }
}

static NOT_THE_THING: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.NOT_THE_THING"));
static MEASURE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.MEASURE_WORDS"));
static A_FAULT: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.A_FAULT"));
static A_GETTING: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.A_GETTING"));
static PLACE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set_of("bot.memory.named.PLACE_WORDS"));

/// DUP(bot.memory.named::_the_noun_in): the content words of a phrase, in order.
fn named_the_noun_in(said: &str) -> Vec<String> {
    if said.is_empty() {
        return Vec::new();
    }
    said.split(|c: char| !c.is_ascii_lowercase())
        .filter(|w| {
            w.chars().count() > 2
                && !NOT_THE_THING.contains(*w)
                && !MEASURE_WORDS.contains(*w)
                && !A_FAULT.contains(*w)
                && !["the", "and", "for", "with", "some", "any", "new"].contains(w)
        })
        .map(str::to_string)
        .collect()
}

/// DUP(bot.memory.named::_the_record_before): the record before the newest one.
fn named_the_record_before(memory: &Memory) -> Option<&Value> {
    let now = mem(memory, "topic").map(|n| rec_str(n, "text")).unwrap_or("");
    let Some(Value::List(records)) = mem(memory, "topics") else {
        return None;
    };
    records
        .iter()
        .filter(|r| matches!(r, Value::Dict(_)) && r.get("text").map(Value::truthy).unwrap_or(false))
        .rev()
        .find(|r| rec_str(r, "text") != now)
}

/// DUP(bot.memory.named::subject_word): the thing the chat is on, in one word, or "".
pub fn subject_word(memory: &Memory) -> String {
    let said = text::squash(&topics_subject(memory)).to_lowercase();
    let mut found = named_the_noun_in(&said);
    if !found.is_empty() && found.iter().all(|w| A_GETTING.contains(w.as_str())) {
        let own = mem(memory, "topic").map(|n| rec_str(n, "text")).unwrap_or("");
        found = named_the_noun_in(&text::squash(own).to_lowercase());
    }
    if let Some(last) = found.last() {
        if PLACE_WORDS.contains(last.as_str()) {
            if let Some(before) = named_the_record_before(memory) {
                let again = named_the_noun_in(&text::squash(rec_str(before, "text")).to_lowercase());
                if !again.is_empty() {
                    found = again;
                }
            }
        }
    }
    found.last().cloned().unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.clothing

static A_CHILD_REFUSING_A_GARMENT: LazyLock<&'static Re> =
    LazyLock::new(|| data::re("bot.knowledge.clothing.A_CHILD_REFUSING_A_GARMENT"));

/// DUP(bot.knowledge.clothing::_the_line): clean, norm_text and text joined, lower-cased.
fn clothing_the_line(understood: &Understood) -> String {
    format!("{} {} {}", understood.clean, understood.norm_text, understood.text).to_lowercase()
}

/// DUP(bot.knowledge.clothing::not_a_fight_about_clothes): false when the line is a child
/// fighting the clothes ("my son fights getting dressed"), so a buying or sibling row stands down.
pub fn not_a_fight_about_clothes(understood: &Understood) -> bool {
    !A_CHILD_REFUSING_A_GARMENT.is_match(&clothing_the_line(understood))
}

// ---------------------------------------------------------------------------------------------
// bot.compose.repair

fn repair_re(name: &str) -> &'static Re {
    data::re(&format!("bot.compose.repair.{name}"))
}
fn repair_table(name: &str) -> Vec<(String, &'static Re)> {
    data::get(&format!("bot.compose.repair.{name}"))
        .items()
        .iter()
        .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().expect("a regex")))
        .collect()
}

static PET_OR_PROFESSIONAL: LazyLock<&'static Re> = LazyLock::new(|| repair_re("PET_OR_PROFESSIONAL"));
static PHYSICAL_HARM: LazyLock<&'static Re> = LazyLock::new(|| repair_re("PHYSICAL_HARM"));
static GRIEF: LazyLock<&'static Re> = LazyLock::new(|| repair_re("_GRIEF"));
static A_BABY: LazyLock<&'static Re> = LazyLock::new(|| repair_re("_A_BABY"));
static REGRET_FIRST: LazyLock<&'static Re> = LazyLock::new(|| repair_re("_REGRET_FIRST"));
static NOT_A_DEED: LazyLock<&'static Re> = LazyLock::new(|| repair_re("NOT_A_DEED"));
static NOBODY_TO_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| repair_re("NOBODY_TO_REPAIR"));
static AMENDS: LazyLock<&'static Re> = LazyLock::new(|| repair_re("AMENDS"));
static ASKS_FOR_THE_WORDS: LazyLock<&'static Re> = LazyLock::new(|| repair_re("ASKS_FOR_THE_WORDS"));
static ASKS_WHAT_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| repair_re("ASKS_WHAT_TO_DO"));
static LAPSES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| repair_table("LAPSES"));
static DEEDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| repair_table("DEEDS"));

/// The line a repair test reads: lower-cased, curly apostrophes made plain, whitespace squashed.
fn repair_text(clean: &str) -> String {
    text::squash(&clean.to_lowercase().replace('\u{2019}', "'"))
}

/// DUP(bot.compose.repair::deed_of): the deed the person reports doing to somebody, or None.
/// Takes the line (a reading's `clean or text`).
pub fn deed_of(clean: &str) -> Option<String> {
    let text_ = repair_text(clean);
    if text_.is_empty() {
        return None;
    }
    if PET_OR_PROFESSIONAL.is_match(&text_) || PHYSICAL_HARM.is_match(&text_) {
        return None;
    }
    if GRIEF.is_match(&text_) || A_BABY.is_match(&text_) || REGRET_FIRST.is_match(&text_) {
        return None;
    }
    if let Some((deed, _)) = LAPSES.iter().find(|(_, p)| p.is_match(&text_)) {
        return Some(deed.clone());
    }
    if NOT_A_DEED.is_match(&text_) || NOBODY_TO_REPAIR.is_match(&text_) {
        return None;
    }
    DEEDS.iter().find(|(_, p)| p.is_match(&text_)).map(|(deed, _)| deed.clone())
}

/// DUP(bot.compose.repair::owns_the_deed) for a reading: true when the line reports something
/// the person did to somebody close.
pub fn owns_the_deed(reading: &Understood) -> bool {
    deed_of(crate::knowledge::d1_shared::clean_or_text(reading)).is_some()
}

/// DUP(bot.compose.repair::owns_the_deed) for a line of text.
pub fn owns_the_deed_line(line: &str) -> bool {
    deed_of(line).is_some()
}

/// DUP(bot.compose.repair::amends_to): the line asks how to make amends to somebody.
pub fn amends_to(clean: &str) -> bool {
    let text_ = repair_text(clean);
    if text_.is_empty() || !AMENDS.is_match(&text_) {
        return false;
    }
    !(PET_OR_PROFESSIONAL.is_match(&text_) || PHYSICAL_HARM.is_match(&text_))
}

/// DUP(bot.compose.repair::a_repair_is_open): the turns just said are one repair, with the asks
/// in the middle of it.
pub fn a_repair_is_open(memory: &Memory) -> bool {
    let told: Vec<String> = memory.turns.iter().map(|t| t.user.to_lowercase()).collect();
    for said in told[told.len().saturating_sub(3)..].iter().rev() {
        let said = text::squash(said);
        if said.is_empty() {
            continue;
        }
        if owns_the_deed_line(&said) || amends_to(&said) {
            return true;
        }
        if text::split_ref(&said).len() > 9 || !(ASKS_FOR_THE_WORDS.is_match(&said) || ASKS_WHAT_TO_DO.is_match(&said)) {
            return false;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup (more)

static HOW_OFTEN_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.HOW_OFTEN_WORDS"));

/// DUP(bot.knowledge.lookup::asks_how_often): the line asks the interval rather than the method.
pub fn asks_how_often(understood: &Understood) -> bool {
    let said = crate::knowledge::d1_shared::the_line(understood);
    HOW_OFTEN_WORDS.iter().any(|w| said.contains(w.as_str()))
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.cooking

fn cooking(name: &str) -> &'static Value {
    data::get(&format!("bot.knowledge.cooking.{name}"))
}
static INGREDIENT_WORDS: LazyLock<std::collections::HashMap<String, String>> =
    LazyLock::new(|| cooking("_INGREDIENT_WORDS").str_map());
static REFUSAL_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| cooking("_REFUSAL_CUES").str_set());
static REFUSAL_JOINERS: LazyLock<HashSet<String>> = LazyLock::new(|| cooking("_REFUSAL_JOINERS").str_set());
static REFUSAL_FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| cooking("_REFUSAL_FILLER").str_set());
static BARE_NO_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| cooking("_BARE_NO_CUES").str_set());
static SHORTAGE_AFTER: LazyLock<HashSet<String>> = LazyLock::new(|| cooking("SHORTAGE_AFTER").str_set());
static FAMILIES: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    cooking("FAMILIES").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect()
});
static DAIRY: LazyLock<Vec<String>> = LazyLock::new(|| cooking("DAIRY").strs());
static TIRED_OF: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:tired|sick|bored)\s+of\b|\bfed\s+up\s+(?:of|with)\b", 0));

/// DUP(bot.knowledge.cooking::_fold): a word with its plural undone (eggs/egg, tomatoes/tomato).
pub fn cooking_fold(word: &str) -> String {
    let lowered = word.to_lowercase().replace('\u{2019}', "'");
    let w: Vec<char> = text::strip_chars(&lowered, ".,!?;:'\"").chars().collect();
    let n = w.len();
    let s: String = w.iter().collect();
    if n > 4 && s.ends_with("ies") {
        return format!("{}y", w[..n - 3].iter().collect::<String>());
    }
    if n > 4 && s.ends_with("oes") {
        return w[..n - 2].iter().collect();
    }
    if n > 3 && s.ends_with("es") && "sxzh".contains(w[n - 3]) {
        return w[..n - 2].iter().collect();
    }
    if n > 2 && s.ends_with('s') && !s.ends_with("ss") {
        return w[..n - 1].iter().collect();
    }
    s
}

/// DUP(bot.knowledge.cooking::_refused_foods): the foods the line says somebody will not eat.
pub fn cooking_refused_foods(text_: &str, families: bool) -> HashSet<String> {
    let said = cooking_fold(text_);
    let said = TIRED_OF.sub(" no ", &said);
    let mut refused = HashSet::new();
    let words: Vec<&str> =
        said.split(|c: char| !(c.is_ascii_lowercase() || c == '\'')).filter(|w| !w.is_empty()).collect();
    for (start, word) in words.iter().enumerate() {
        if !REFUSAL_CUES.contains(*word) {
            continue;
        }
        let mut index = start + 1;
        while index < words.len() {
            let here = words[index];
            let name = INGREDIENT_WORDS
                .get(here)
                .filter(|n| !n.is_empty())
                .or_else(|| INGREDIENT_WORDS.get(&cooking_fold(here)).filter(|n| !n.is_empty()));
            if let Some(name) = name {
                let after = &words[(index + 1).min(words.len())..(index + 3).min(words.len())];
                if BARE_NO_CUES.contains(*word) && after.iter().any(|p| SHORTAGE_AFTER.contains(*p)) {
                    break;
                }
                refused.insert(name.clone());
                index += 1;
                continue;
            }
            if let Some((_, members)) = FAMILIES.iter().find(|(k, _)| k == here) {
                if families {
                    let take = if here == "lactose" { &*DAIRY } else { members };
                    refused.extend(take.iter().cloned());
                }
                index += 1;
                continue;
            }
            if REFUSAL_FILLER.contains(here) || REFUSAL_JOINERS.contains(here) {
                index += 1;
                continue;
            }
            break;
        }
    }
    refused
}
