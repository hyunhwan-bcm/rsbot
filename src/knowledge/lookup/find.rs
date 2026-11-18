//! lookup.py's matcher: `find` and `_the_best_entry`, the line read as the entry wants it
//! (handoffs, intensifiers, a child said by place, a colour worn), `_best` (every reached entry
//! scored, ranked and turned down), and the readings `find` falls back on.

use super::*;
use crate::understood::{LeansOn, Num};
use std::borrow::Cow;

/// `reject(item)`: an entry the caller turns down.
pub type Reject<'a> = Option<&'a dyn Fn(&Entry) -> bool>;

fn rejected(reject: Reject, item: &Entry) -> bool {
    reject.is_some_and(|r| r(item))
}

fn lower_all(words: &[String]) -> Vec<String> {
    words.iter().map(|w| text::lower(w)).collect()
}

/// `dict(understood)` with a new clean line and words.
fn with_line(u: &Understood, clean: String, words: Vec<String>) -> Understood {
    let mut again = u.clone();
    again.clean = clean;
    again.words = words;
    again
}

/// A `leans_on` dict read as a reading of its own (`_the_best_entry(leans, ...)`).
pub fn leans_reading(leans: &LeansOn) -> Understood {
    let mut u = Understood { text: leans.text.clone(), words: leans.words.clone(), subject: leans.subject.clone(), topic: leans.topic.clone(), ..Default::default() };
    if let Some(t) = &leans.subject_type {
        u.extra.insert("subject_type".into(), serde_json::Value::String(t.clone()));
    }
    u
}

/// `str(number.get('value'))`.
fn num_str(v: &Option<Num>) -> String {
    match v {
        None => "None".into(),
        Some(Num::Int(i)) => i.to_string(),
        Some(Num::Float(f)) => text::float_repr(*f),
    }
}

/// `_y_word_stems(word)`: "crumbly" -> "crumbl", "crumble".
pub fn _y_word_stems(word: &str) -> Vec<String> {
    let w: Vec<char> = word.chars().collect();
    if w.len() < 5 || !word.ends_with('y') || "aeiouy".contains(w[w.len() - 2]) {
        return vec![];
    }
    if NOT_THE_ADJECTIVE.contains(word) {
        return vec![];
    }
    let stem: String = w[..w.len() - 1].iter().collect();
    [stem.clone(), format!("{stem}e")].into_iter().filter(|f| text::len(f) >= *MIN_FORM).collect()
}

/// `_a_word_itself(word)`.
pub fn _a_word_itself(word: &str) -> bool {
    _COMMON.contains(word) || COMMON_WORDS.contains(word) || LEX_CHAT.contains_key(word) || LEX_EXPAND.contains_key(word)
}

/// `_read_the_adjective(words, forms, vocabulary)`: a "-y" word read as its noun.
pub fn _read_the_adjective(words: &[String], forms: &mut [Forms], vocabulary: &HashMap<String, usize>) -> bool {
    let mut gave = false;
    for (index, word) in words.iter().enumerate() {
        if _a_word_itself(word) {
            continue;
        }
        for stem in _y_word_stems(word) {
            if vocabulary.contains_key(&stem) && !forms[index].contains(&stem) {
                let mut more = (*forms[index]).clone();
                more.insert(stem);
                forms[index] = Arc::new(more);
                gave = true;
            }
        }
    }
    gave
}

/// `_read_a_thing_had(words, forms)`: "has" said of a person reads as "have".
pub fn _read_a_thing_had(words: &[String], forms: &mut [Forms]) {
    for (index, word) in words.iter().enumerate() {
        if text::lower(word) != "has" {
            continue;
        }
        let after = words.get(index + 1).map(|w| text::lower(w)).unwrap_or_default();
        if _NOT_A_THING_HAD.contains(&after) {
            continue;
        }
        if after == "got" && index + 2 < words.len() && text::lower(&words[index + 2]) == "to" {
            continue;
        }
        let before = &words[index.saturating_sub(4)..index];
        if !before.iter().any(|w| _A_PERSON.contains(&text::lower(w))) {
            continue;
        }
        let mut more = (*forms[index]).clone();
        more.extend(_forms("have").iter().cloned());
        forms[index] = Arc::new(more);
    }
}

/// `_read_typos(words, forms, vocabulary, known)`: a misspelt key word read as the word.
pub fn _read_typos(words: &[String], forms: &mut [Forms], vocabulary: &HashMap<String, usize>, known: &HashSet<String>) {
    for (index, word) in words.iter().enumerate() {
        if meets(&forms[index], known) || _engine_reads(word) {
            continue;
        }
        if word.contains('\'') {
            continue;
        }
        let typo = _typo_forms(word, vocabulary);
        let mut more = (*forms[index]).clone();
        more.extend(typo);
        forms[index] = Arc::new(more);
    }
}

/// `_without_possessive(word)`.
pub fn _without_possessive(word: &str) -> &str {
    for tail in ["'s", "\u{2019}s"] {
        if let Some(w) = word.strip_suffix(tail) {
            return w;
        }
    }
    word
}

/// `_names_a_fault(occasion)`.
pub fn _names_a_fault(occasion: &[String]) -> bool {
    occasion.iter().any(|w| FEELING_FAULT_WORDS.contains(&text::lower(w)))
}

fn feeling_at(low: &[String], pos: usize) -> bool {
    FEELING_ABOUT_A_THING.contains(&low[pos]) || (low[pos] == "fed" && low.get(pos + 1).is_some_and(|w| w == "up"))
}

/// `_speaker_is_feeling(words)`: the speaker says how they feel ("i'm anxious about ...").
pub fn _speaker_is_feeling(words: &[String]) -> bool {
    let low = lower_all(words);
    for pos in 0..low.len() {
        if !feeling_at(&low, pos) {
            continue;
        }
        if low[pos.saturating_sub(6)..pos].iter().any(|b| FEELING_SPEAKER.contains(b)) {
            return true;
        }
    }
    false
}

/// `_names_a_feared_thing(occasion)`.
pub fn _names_a_feared_thing(occasion: &[String]) -> bool {
    occasion.iter().any(|w| FEARED_THINGS.contains(&text::lower(w)))
}

/// `_feeling_occasion(words)`: what a feeling is about ("worried about my exam" -> ["my",
/// "exam"]), or None.
pub fn _feeling_occasion(words: &[String]) -> Option<Vec<String>> {
    let low = lower_all(words);
    for pos in 0..low.len() {
        if !feeling_at(&low, pos) {
            continue;
        }
        let mut at = pos + 1;
        if at < low.len() && (low[at] == "up" || low[at] == "off") {
            at += 1;
        }
        if at >= low.len() || !FEELING_ABOUT.contains(&low[at]) {
            continue;
        }
        let mut got = Vec::new();
        for word_after in low[(at + 1).min(low.len())..(at + 6).min(low.len())].iter() {
            if FEELING_ENDS.contains(word_after) || !crate::understanding::pyuni::is_alpha(word_after) {
                break;
            }
            got.push(word_after.clone());
        }
        if !got.iter().any(|w| !FEELING_NAMES_NOTHING.contains(w)) {
            continue;
        }
        return Some(got);
    }
    None
}

/// `_the_kept_rows()`: the entries about where a thing is kept.
pub fn _the_kept_rows() -> &'static [&'static Entry] {
    static ROWS: LazyLock<Vec<&'static Entry>> = LazyLock::new(|| all_entry_refs().iter().copied().filter(|item| _key_names(&_KEPT_SOMEWHERE, item)).collect());
    &ROWS
}

/// The text a reply says (`str(reply)`), or None when a function reply raised.
fn reply_said(item: &Entry, reading: &Understood) -> Option<String> {
    match &item.reply {
        Reply::Text(s) => Some(s.clone()),
        Reply::Fn(f) => {
            let mut memory = crate::state::Memory::default();
            crate::knowledge::registry::try_reply(f, &mut memory, reading).ok().map(|r| r.unwrap_or_default())
        }
        Reply::Choices(lines) => {
            let inner = lines.iter().map(|l| crate::understood::py_str_repr(l)).collect::<Vec<_>>().join(", ");
            let tuple = matches!(item.raw.attr("reply"), Some(crate::data::Value::Tuple(_)));
            Some(if tuple {
                if lines.len() == 1 {
                    format!("({inner},)")
                } else {
                    format!("({inner})")
                }
            } else {
                format!("[{inner}]")
            })
        }
    }
}

/// `_a_price_judged_is_not_answered_here(understood, found)`: the line judges a price, the
/// entry gives none, and the knowledge base has the price of the thing.
pub fn _a_price_judged_is_not_answered_here(understood: &Understood, found: &Entry) -> bool {
    let line = first_text(understood, &["clean", "text"]);
    if !crate::knowledge::prices::a_price_is_judged(line) {
        return false;
    }
    let Some(reply) = reply_said(found, understood) else { return false };
    if _HOLDS_MONEY.is_match(&reply) {
        return false;
    }
    _a_range_for_the_thing(line, Some(understood)).is_some()
}

/// `_the_thing_the_chat_is_on(understood)`: what "it" points at, or the subject.
pub fn _the_thing_the_chat_is_on(understood: Option<&Understood>) -> String {
    let Some(u) = understood else { return String::new() };
    let r = &u.reference;
    for thing in [&r.it, &r.that, &r.they].into_iter().flatten() {
        if !thing.is_empty() && !_POINTS_AT_IT.contains(&text::lower(thing)) {
            return thing.clone();
        }
    }
    if let Some(subject) = &u.subject {
        if !subject.is_empty() && !_POINTS_AT_IT.contains(&text::lower(subject)) {
            return subject.clone();
        }
    }
    String::new()
}

/// `_a_range_for_the_thing(text, understood=None)`: the price the knowledge base gives for the
/// thing the line judges, or None.
pub fn _a_range_for_the_thing(line: &str, understood: Option<&Understood>) -> Option<String> {
    let stripped = text::strip(line).trim_end_matches(['?', '.', '!']);
    let found = _THE_THING_JUDGED.search(stripped);
    let mut said = match &found {
        Some(m) => text::strip(m.get(2).expect("AttributeError: 'NoneType' object has no attribute 'strip'")).to_string(),
        None => String::new(),
    };
    if _POINTS_AT_IT.contains(&text::lower(&said)) || said.is_empty() {
        said = _the_thing_the_chat_is_on(understood);
    }
    if said.is_empty() {
        return None;
    }
    let thing = match &found {
        Some(m) => {
            let lead = m.get(1).filter(|g| !g.is_empty()).unwrap_or("a ");
            let mut words = text::split(lead);
            words.extend(text::split(&said));
            words.join(" ")
        }
        None => format!("a {said}"),
    };
    for question in [format!("how much is {thing}"), format!("how much does {thing} cost")] {
        let asked = Understood { clean: question.clone(), words: text::split(&question), ..Default::default() };
        let Some(entry) = find(&asked, None, None, false) else { continue };
        let Some(reply) = reply_said(entry, &asked) else { continue };
        if !reply.is_empty() && _HOLDS_MONEY.is_match(&reply) {
            return Some(reply);
        }
    }
    None
}

/// `find(understood, entries=None, reject=None, use_context=False)`: the entry the line asks
/// for, or None. `entries` None means every entry; `reject` turns an entry down.
pub fn find(understood: &Understood, entries: Option<&[&'static Entry]>, reject: Reject, use_context: bool) -> Option<&'static Entry> {
    let line = first_text(understood, &["clean", "text"]);
    if crate::memory::dream::told_as_a_dream(line) {
        return None;
    }
    if entries.is_none() && _WHERE_I_PUT.is_match(&tokens(line).join(" ")) {
        return None;
    }
    if entries.is_none() && _WHERE_TO_KEEP.is_match(&tokens(line).join(" ")) {
        if let Some(kept) = _the_best_entry(understood, Some(_the_kept_rows()), reject, use_context) {
            return Some(kept);
        }
    }
    let found = _the_best_entry(understood, entries, reject, use_context);
    if let Some(f) = found {
        if _a_price_judged_is_not_answered_here(understood, f) {
            return None;
        }
        return Some(f);
    }
    for again in _what_is_wrong_with_them(understood) {
        if let Some(f) = _the_best_entry(&again, entries, reject, use_context) {
            return Some(f);
        }
    }
    for again in _a_thing_that_has_not_gone(understood) {
        if let Some(f) = _the_best_entry(&again, entries, reject, use_context) {
            return Some(f);
        }
    }
    for again in _without_the_place(understood) {
        if let Some(f) = _the_best_entry(&again, entries, reject, use_context) {
            return Some(f);
        }
    }
    None
}

/// `_without_the_place(understood)`: the line without "on my arm" when that leaves enough.
pub fn _without_the_place(understood: &Understood) -> Vec<Understood> {
    let clean = first_text(understood, &["clean", "text"]);
    let Some(said) = _A_PLACE_ON_THEM.search(clean) else { return vec![] };
    if !BODY_PARTS.contains(&text::lower(said.str(1))) {
        return vec![];
    }
    let joined = format!("{}{}", &clean[..said.start()], &clean[said.end()..]);
    let left = strip_chars(&joined, " ,.;:!?").to_string();
    if text::split(&left).len() < 2 {
        return vec![];
    }
    let words = tokens(&left);
    vec![with_line(understood, left, words)]
}

/// `_a_thing_that_has_not_gone(understood)`: "<thing> hasn't gone" asked as the thing that
/// will not go away.
pub fn _a_thing_that_has_not_gone(understood: &Understood) -> Vec<Understood> {
    let clean = first_text(understood, &["clean", "text"]);
    let Some(said) = _NOT_GONE.search(clean) else { return vec![] };
    let thing = _SUBJECT_LEAD.sub("", strip_chars(&clean[..said.start()], " ,.;:!?"));
    let thing = text::strip(&_POSSESSIVE.sub("", &thing)).to_string();
    if thing.is_empty() || text::split(&thing).len() > 5 {
        return vec![];
    }
    [format!("a {thing} that wont go away"), format!("how do i treat {thing}"), format!("{thing} still there")]
        .into_iter()
        .map(|said_so| {
            let words = tokens(&said_so);
            with_line(understood, said_so, words)
        })
        .collect()
}

/// `_what_is_wrong_with_them(understood)`: a carer's line asked as how to deal with the illness.
pub fn _what_is_wrong_with_them(understood: &Understood) -> Vec<Understood> {
    let Some(cared) = _past_the_carer(first_text(understood, &["clean", "text"])) else { return vec![] };
    ["how do i deal with ", "how do i treat "]
        .iter()
        .map(|opener| {
            let said = format!("{opener}{cared}");
            let words = tokens(&said);
            with_line(understood, said, words)
        })
        .collect()
}

/// `_without_the_ruled_out(understood)`: the line without the words it rules out.
pub fn _without_the_ruled_out(understood: &Understood) -> Cow<'_, Understood> {
    let gone: Vec<String> = understood.ruled_out.iter().filter(|w| !text::strip(w).is_empty()).map(|w| text::lower(text::strip(w))).collect();
    if gone.is_empty() {
        return Cow::Borrowed(understood);
    }
    let words = if understood.words.is_empty() { tokens(&understood.clean) } else { understood.words.clone() };
    let mut wanted = gone;
    let mut left = Vec::new();
    for word in words {
        let key = text::lower(&word);
        if !key.is_empty() {
            if let Some(at) = wanted.iter().position(|w| *w == key) {
                wanted.remove(at);
                continue;
            }
        }
        left.push(word);
    }
    if !left.iter().any(|w| !_OF_THE_RULE_OUT.contains(&text::lower(w))) {
        return Cow::Borrowed(understood);
    }
    let clean = left.join(" ");
    Cow::Owned(with_line(understood, clean, left))
}

/// `_the_degrees_are_a_temperature(understood)`.
pub fn _the_degrees_are_a_temperature(understood: &Understood) -> bool {
    if understood.numbers.is_empty() {
        return false;
    }
    let mut said: Vec<String> = Vec::new();
    for number in &understood.numbers {
        said.extend(text::split(&text::lower(&number.text)));
    }
    said.extend(understood.norm.iter().map(|w| text::lower(w)));
    for word in &said {
        if word.contains("degree") {
            return true;
        }
        if ["°c", "°f", "º", "°", "°c"].iter().any(|sign| word.contains(sign)) {
            return true;
        }
    }
    false
}

/// `_answers_a_qualification(item)`: the row is about degrees as a qualification.
pub fn _answers_a_qualification(item: &Entry) -> bool {
    let key = _key_words(item);
    if !key.iter().any(|w| _THE_DEGREE_WORDS.contains(w)) {
        return false;
    }
    !key.iter().any(|w| _THE_HEAT_OF_THE_DAY.contains(w) && !_THE_DEGREE_WORDS.contains(w))
}

/// `_the_person_says_they_are_cold(typed)`.
pub fn _the_person_says_they_are_cold(typed: &[String]) -> bool {
    for (at, word) in typed.iter().enumerate() {
        if !_HOW_A_PERSON_IS_COLD.contains(word) && !_COLD_WORDS.contains(word) {
            continue;
        }
        for before in &typed[at.saturating_sub(4)..at] {
            if _SAYS_OF_THEMSELVES.contains(before) || ["feel", "feeling", "feels", "keep", "keeps", "kept", "being", "got", "get"].contains(&before.as_str()) {
                return true;
            }
        }
    }
    false
}

/// `_the_place_the_line_says_is_cold(typed)`: "the house is freezing" -> "house".
pub fn _the_place_the_line_says_is_cold(typed: &[String]) -> Option<String> {
    for (at, word) in typed.iter().enumerate() {
        if !_PLACES_PEOPLE_LIVE_IN.contains(word) {
            continue;
        }
        for next_word in &typed[(at + 1).min(typed.len())..(at + 4).min(typed.len())] {
            if _COLD_WORDS.contains(next_word) {
                return Some(word.clone());
            }
            if !_BETWEEN_A_PLACE_AND_ITS_COLD.contains(next_word) {
                break;
            }
        }
    }
    None
}

/// `_answers_the_cold_of_a_person(item, place=None)`.
pub fn _answers_the_cold_of_a_person(item: &Entry, place: Option<&str>) -> bool {
    if !FEELING_DOMAINS.contains(&item.domain) {
        return false;
    }
    let key = _key_words(item);
    if !key.iter().any(|w| _COLD_WORDS.contains(w)) {
        return false;
    }
    place.is_none_or(|p| !key.contains(p))
}

/// `_answers_the_cold_of_a_place(item, place)`.
pub fn _answers_the_cold_of_a_place(item: &Entry, place: &str) -> bool {
    if _answers_the_cold_of_a_person(item, Some(place)) {
        return false;
    }
    let key = _key_words(item);
    key.contains(place) && key.iter().any(|w| _COLD_WORDS.contains(w))
}

/// `_the_card_for_the_place_first(place, reached)`.
pub fn _the_card_for_the_place_first(place: &str, reached: Vec<Row>) -> Vec<Row> {
    if reached.is_empty() {
        return reached;
    }
    for at in 1..reached.len() {
        if _answers_the_cold_of_a_place(reached[at].item, place) {
            let mut out = vec![reached[at].clone()];
            out.extend(reached[..at].iter().cloned());
            out.extend(reached[at + 1..].iter().cloned());
            return out;
        }
    }
    reached
}

/// `_the_ask_reading(understood, ask)`: the reading of the ask alone.
pub fn _the_ask_reading(understood: &Understood, ask: &[String]) -> Understood {
    let mut asked = understood.clone();
    let said: HashSet<String> = ask.iter().map(|w| text::lower(w)).collect();
    asked.clean = ask.join(" ");
    asked.words = ask.to_vec();
    asked.norm = ask.to_vec();
    asked.numbers = understood
        .numbers
        .iter()
        .filter(|n| {
            let written = text::split(&text::lower(&n.text));
            written.iter().any(|w| said.contains(w)) || said.contains(&num_str(&n.value))
        })
        .cloned()
        .collect();
    asked.entities = understood.entities.iter().filter(|e| text::split(&text::lower(&e.text)).iter().any(|w| said.contains(w))).cloned().collect();
    if let Some(things) = understood.extra.get("things").and_then(|v| v.as_array()) {
        let kept: Vec<serde_json::Value> = things
            .iter()
            .filter(|thing| {
                let t = thing.get("text").unwrap_or(thing);
                t.as_str().is_some_and(|t| text::split(&text::lower(t)).iter().any(|w| said.contains(w)))
            })
            .cloned()
            .collect();
        asked.extra.insert("things".into(), serde_json::Value::Array(kept));
    }
    asked.subject = understood.subject.clone().filter(|s| said.contains(&text::lower(s)));
    asked
}

/// `_the_ask_on_its_own(understood)`: the wh-ask that ends a line of two clauses.
pub fn _the_ask_on_its_own(understood: &Understood) -> Option<Vec<String>> {
    if understood.clauses.len() < 2 {
        return None;
    }
    let ask: Vec<String> = understood.focus.iter().filter(|w| !text::strip(w).is_empty()).cloned().collect();
    if ask.len() < 2 {
        return None;
    }
    let whole = if !understood.norm.is_empty() { &understood.norm } else { &understood.words };
    let line = lower_all(whole);
    let tail = lower_all(&ask);
    if tail.len() >= line.len() || line[line.len() - tail.len()..] != tail[..] {
        return None;
    }
    if !_THE_WH_WORDS.contains(&tail[0]) {
        return None;
    }
    Some(ask)
}

/// Drop words matching `counts` from the end of `words` (each count used once).
fn without_counted(words: Vec<String>, counts: &mut HashMap<String, i64>, also_stem: bool) -> Vec<String> {
    let mut kept = Vec::new();
    for word in words.into_iter().rev() {
        let key = text::lower(&word);
        let stem = _without_possessive(&key).to_string();
        let hit = if counts.get(&key).copied().unwrap_or(0) != 0 {
            key
        } else if also_stem && counts.get(&stem).copied().unwrap_or(0) != 0 {
            stem
        } else {
            String::new()
        };
        if !hit.is_empty() {
            *counts.get_mut(&hit).unwrap() -= 1;
            continue;
        }
        kept.push(word);
    }
    kept.reverse();
    kept
}

/// `_the_best_entry(understood, entries=None, reject=None, use_context=False)`.
pub fn _the_best_entry(understood: &Understood, entries: Option<&[&'static Entry]>, reject: Reject, use_context: bool) -> Option<&'static Entry> {
    let understood = _without_the_ruled_out(understood);
    let understood: &Understood = &understood;
    let clean = understood.clean.clone();
    let mut words: Vec<String> = understood.words.clone();
    if ['\u{2019}', '\u{2018}', '\u{2bc}'].iter().any(|c| clean.contains(*c)) {
        words = tokens(&clean);
    }
    if let Some(past) = _past_a_handoff(&clean) {
        words = tokens(&past);
    }
    // left = understood.get('intensifier_left'): a full reading always has it.
    if !understood.intensifier_left.is_empty() {
        let mut going: HashMap<String, i64> = HashMap::new();
        for word in &understood.intensifier_left {
            *going.entry(text::lower(word)).or_default() += 1;
        }
        words = without_counted(words, &mut going, true);
    }
    if !understood.background.is_empty() && understood.topic_own.as_deref().unwrap_or("").is_empty() {
        let mut drop: HashMap<String, i64> = HashMap::new();
        for word in &understood.background {
            *drop.entry(text::lower(word)).or_default() += 1;
        }
        words = without_counted(words, &mut drop, false);
    }
    if understood.kind == "empty" {
        return None;
    }
    let every_entry = entries.unwrap_or(all_entry_refs());
    if let Some(occasion) = _feeling_occasion(&words) {
        if understood.asks.is_empty()
            && understood.topic.as_deref() != Some("pets")
            && !_names_a_fault(&occasion)
            && !_speaker_is_feeling(&words)
            && !_names_a_feared_thing(&occasion)
        {
            let vocab = _vocabulary(every_entry);
            let here = _best(understood, &words, &clean, &FormLine::of(&words), &lower_all(&words), every_entry, &vocab.by_domain, &vocab.every, reject, false);
            if here.is_none_or(|h| !FEELING_DOMAINS.contains(&h.domain)) {
                let mut gone: HashMap<String, i64> = HashMap::new();
                for word in &occasion {
                    *gone.entry(word.clone()).or_default() += 1;
                }
                words = without_counted(words, &mut gone, false);
            }
        }
    }
    let original = words.clone();
    let mut stripped = false;
    if words.iter().any(|w| text::lower(w) == "dead") {
        words = _without_a_dead_that_only_means_very(&words);
        stripped = words.len() != original.len();
    }
    if words.iter().any(|w| text::lower(w) == "cooking") {
        words = _without_a_cooking_that_is_a_name(&words);
        stripped = stripped || words.len() != original.len();
    }
    words = _with_the_child_by_place(words, &clean);
    if a_colour_is_worn(understood) {
        let mut kept: Vec<String> = words
            .iter()
            .enumerate()
            .filter(|(index, w)| !(COLOUR_ASKED_WORDS.contains(&text::lower(w)) && !_a_dress_code(&words, *index)))
            .map(|(_, w)| w.clone())
            .collect();
        kept.push("wear".into());
        words = kept;
    }
    let mut forms = FormLine::of(&words);
    let entries = every_entry;
    let vocab = _vocabulary(entries);
    let (by_domain, every, known) = (&vocab.by_domain, &vocab.every, &vocab.known);
    let typed = lower_all(&words);
    let own_line = if understood.text.is_empty() { clean.as_str() } else { understood.text.as_str() };
    if about_the_chat(own_line) {
        return None;
    }
    _read_typos(&typed, &mut forms, every, known);
    _read_a_thing_had(&lower_all(&words), &mut forms);
    let wrapped = _wrapped_subject(&clean, &understood.norm_text);
    let whole = _best(understood, &words, &clean, &forms, &typed, entries, by_domain, every, reject, false);
    let mut whole = _the_answer_the_colour_dragged_in(whole, understood, &words, &clean, entries, by_domain, every, reject);
    if a_colour_is_worn(understood) {
        whole = _the_card_for_the_occasion_asked(whole, understood, &words, &clean, &forms, &typed, entries, by_domain, every, reject);
    }
    let mut item = None;
    if let Some(w) = &wrapped {
        item = _the_ask_it_wraps(understood, w, whole, entries, by_domain, every, reject);
    }
    if item.is_none() {
        item = whole;
    }
    let filler = typed.iter().any(|w| _FILLER.contains(w));
    let kept: Vec<String> = if filler { words.iter().filter(|w| !_FILLER.contains(&text::lower(w))).cloned().collect() } else { vec![] };
    let bare = if filler {
        let mut b = understood.clone();
        b.clean = kept.join(" ");
        Some(b)
    } else {
        None
    };
    if item.is_none() {
        if let Some(b) = &bare {
            item = _best(b, &kept, &b.clean, &FormLine::of(&kept), &lower_all(&kept), entries, by_domain, every, reject, false);
        }
    }
    if let Some(ask) = _the_ask_on_its_own(understood) {
        let asked_for: HashSet<String> = ask.iter().map(|w| text::lower(w)).filter(|w| !_ASK_FRAME.contains(w)).collect();
        if item.is_none_or(|i| !_key_words(i).iter().any(|w| asked_for.contains(w))) {
            let asked = _the_ask_reading(understood, &ask);
            let found = _best(&asked, &ask, &asked.clean, &FormLine::of(&ask), &lower_all(&ask), entries, by_domain, every, reject, false);
            if found.is_some() {
                item = found;
            }
        }
    }
    if item.is_none() && stripped {
        item = _best(understood, &original, &clean, &FormLine::of(&original), &lower_all(&original), entries, by_domain, every, reject, false);
    }
    if item.is_none() {
        let mut described = FormLine::new(forms.forms.clone());
        if _read_the_adjective(&typed, &mut described, every) {
            item = _best(understood, &words, &clean, &described, &typed, entries, by_domain, every, reject, false);
        }
    }
    if item.is_none() && use_context && understood.leans_on.is_some() && leans_on_a_subject(understood) {
        item = _following_on(understood, entries, &forms, &typed, by_domain, every, known, reject);
        if item.is_none() && bare.is_some() {
            item = _following_on(understood, entries, &FormLine::of(&kept), &lower_all(&kept), by_domain, every, known, reject);
        }
        if let (Some(i), Some(w)) = (item, &wrapped) {
            if !about(i, Some(w)) {
                item = None;
            }
        }
    }
    if let Some(i) = item {
        if _the_animal_rules_out_that_card(i, understood) {
            if let Some(pointed) = _with_the_animal_in_hand(&words, understood) {
                let better = _best(understood, &pointed, &clean, &FormLine::of(&pointed), &lower_all(&pointed), entries, by_domain, every, reject, false);
                if better.is_some() {
                    item = better;
                }
            }
        }
    }
    if item.is_none() {
        item = _defines(understood, &forms, &typed, entries, reject);
    }
    if item.is_none() {
        let allowed = _topic_domains(understood, Some(entries));
        let named = _the_name_card(&lower_all(&words), entries, reject, allowed.as_ref());
        item = _the_name_alone(named, entries, reject, allowed.as_ref(), Some(understood), &words, use_context);
    }
    item
}

/// `_past_a_handoff(clean)`: the line after "my friend says ..." / "the doctor said ...".
pub fn _past_a_handoff(clean: &str) -> Option<String> {
    let line = _THROAT.sub("", &text::split(&text::lower(clean)).join(" "));
    let mut leads: Vec<&String> = HANDOFF_LEADS.iter().collect();
    leads.sort_by(|a, b| text::len(b).cmp(&text::len(a)));
    for lead in leads {
        let Some(rest) = line.strip_prefix(lead.as_str()) else { continue };
        let mut after: Option<&str> = None;
        if let Some(p) = _PUNCTUATION.match_(rest) {
            after = Some(&rest[p.end()..]);
        } else if let Some(b) = _BRIDGE.match_(rest) {
            after = Some(&rest[b.end()..]);
        }
        let Some(mut after) = after else { continue };
        if let Some(b) = _BRIDGE.match_(after) {
            after = &after[b.end()..];
        }
        let out = strip_chars(after, " ,.:;\t\u{2013}\u{2014}");
        return if out.is_empty() { None } else { Some(out.to_string()) };
    }
    None
}

/// `_past_the_carer(clean)`: the illness after "i care for my mum who has ...".
pub fn _past_the_carer(clean: &str) -> Option<String> {
    let line = text::split(&text::lower(clean)).join(" ");
    let found = CARER_FRAME.match_(&line)?;
    let left = strip_chars(&line[found.end()..], " ,.:;\t").to_string();
    let words: Vec<&str> = text::split_ref(&left).into_iter().map(|w| strip_chars(w, &_MARKS)).filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return None;
    }
    if AN_ILLNESS.fullmatch(words[words.len() - 1]).is_some() || (words.len() <= 3 && words.iter().any(|w| AN_ILLNESS.fullmatch(w).is_some())) {
        return Some(left);
    }
    None
}

/// `_without_a_dead_that_only_means_very(words)`: "dead easy" is very easy.
pub fn _without_a_dead_that_only_means_very(words: &[String]) -> Vec<String> {
    let mut kept = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let following = words.get(index + 1).map(|w| w.as_str()).unwrap_or("");
        if text::lower(word) == "dead" && DEAD_AS_VERY_AFTER.contains(strip_chars(&text::lower(following), &_MARKS)) {
            continue;
        }
        kept.push(word.clone());
    }
    kept
}

/// `_without_a_cooking_that_is_a_name(words)`: "cooking apples" names the apple.
pub fn _without_a_cooking_that_is_a_name(words: &[String]) -> Vec<String> {
    let mut kept = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let following = words.get(index + 1).map(|w| w.as_str()).unwrap_or("");
        if text::lower(word) == "cooking" && COOKING_AS_A_NAME.contains(strip_chars(&text::lower(following), &_MARKS)) {
            continue;
        }
        kept.push(word.clone());
    }
    kept
}

/// `_with_the_animal_in_hand(words, understood)`: "it" read as the animal the chat is about;
/// None when nothing changed (Python returns the same list).
pub fn _with_the_animal_in_hand(words: &[String], understood: &Understood) -> Option<Vec<String>> {
    let animal = &understood.animal_in_hand;
    if animal.is_empty() {
        return None;
    }
    let kept: Vec<String> = words
        .iter()
        .map(|piece| {
            let low = text::lower(piece);
            let low = strip_chars(&low, &_MARKS);
            let low = low.split('\'').next().unwrap_or("");
            if _THE_PRONOUNS.contains(low) {
                animal.clone()
            } else {
                piece.clone()
            }
        })
        .collect();
    if kept != words {
        Some(kept)
    } else {
        None
    }
}

/// `_the_animal_rules_out_that_card(item, understood)`.
pub fn _the_animal_rules_out_that_card(item: &Entry, understood: &Understood) -> bool {
    let animal = &understood.animal_in_hand;
    if animal.is_empty() {
        return false;
    }
    let stood_down_for: HashSet<String> = item.avoid.iter().map(|w| text::lower(w)).collect();
    stood_down_for.contains(animal) || stood_down_for.contains(&format!("{animal}s"))
}

/// `_with_the_child_by_place(words, clean)`: "my eldest" / "the little one" read as "child".
pub fn _with_the_child_by_place(words: Vec<String>, clean: &str) -> Vec<String> {
    let line = clean;
    if !CHILD_BY_PLACE.is_match(line) || TWO_OF_US.match_(line).is_some() {
        return words;
    }
    let small = !A_TEEN.is_match(line);
    let bare = |w: &str| strip_chars(&text::lower(w), &_MARKS).to_string();
    let said: HashSet<String> = words.iter().map(|w| bare(w)).collect();
    if said.iter().any(|w| NOT_A_CHILD_SAID.contains(w)) {
        return words;
    }
    if A_CHILD_GETTING_OLDER.is_match(line) && !A_CHILD_WORD_BY_ITSELF.is_match(line) {
        return words;
    }
    let mut kept = Vec::new();
    let mut skipping = 0;
    for (at, word) in words.iter().enumerate() {
        let low = bare(word);
        if skipping != 0 && (low == "one" || low.is_empty()) {
            skipping = 0;
            continue;
        }
        skipping = 0;
        let child_word = CHILD_PLACE_WORDS.contains(&low) || (small && EVERYDAY_CHILD.contains(&low));
        if CHILD_SIZE_WORDS.contains(&low) {
            let after = words[at + 1..].iter().map(|w| bare(w)).find(|w| !w.is_empty()).unwrap_or_default();
            if CHILD_PLACE_WORDS.contains(&after) || CHILD_AFTER_A_SIZE.contains(&after) || (small && EVERYDAY_CHILD.contains(&after)) {
                continue;
            }
        }
        if child_word || (small && CHILD_AFTER_A_SIZE.contains(&low) && at > 0 && CHILD_SIZE_WORDS.contains(&bare(&words[at - 1]))) {
            kept.push("child".to_string());
            skipping = if low == "little" || low == "middle" { 1 } else { 0 };
            continue;
        }
        kept.push(word.clone());
    }
    kept
}

/// `_the_thing_asked_about(clean)`: the thing "what is a ..." asks to have defined.
pub fn _the_thing_asked_about(clean: &str) -> Vec<String> {
    if text::strip(clean).is_empty() {
        return vec![];
    }
    let low = text::lower(text::strip(clean));
    let Some(found) = _DEFINITION_ASK.match_(&low) else { return vec![] };
    let thing = found.groups().into_iter().flatten().find(|p| !p.is_empty()).unwrap_or("").to_string();
    let first = text::split(&thing).into_iter().next().unwrap_or_default();
    let opening = strip_chars(&first, "'\"-");
    if _DEFINITION_POINTERS.contains(opening) {
        return vec![];
    }
    let raw = text::lower(&thing).replace('\u{2019}', "'");
    let padded = format!(" {raw} ");
    if _words_of(&raw).iter().any(|w| _DEFINITION_TROUBLE.contains(w)) || ["going on", "up with", "up to"].iter().any(|p| padded.contains(p)) {
        return vec![];
    }
    let mut words: Vec<String> = Vec::new();
    for word in text::split(&thing.replace('\u{2019}', "'")) {
        let word = strip_chars(&word, "'\"-").to_string();
        if word.is_empty() || _DEFINITION_EMPTY.contains(&word) || _DEFINITION_TAIL.contains(&word) {
            continue;
        }
        if text::len(&word) < 2 && !crate::understanding::pyuni::is_alpha(&word) {
            continue;
        }
        words.push(word);
    }
    while words.last().is_some_and(|w| _DEFINITION_TAIL.contains(w)) {
        words.pop();
    }
    words.truncate(3);
    words
}

/// `_words_of(text)`.
pub fn _words_of(line: &str) -> Vec<String> {
    text::split_ref(line).into_iter().map(|w| strip_chars(w, "'\"-?!.,;:")).filter(|w| !w.is_empty()).map(str::to_string).collect()
}

/// `_asks_what_it_is_for(clean)`.
pub fn _asks_what_it_is_for(clean: &str) -> bool {
    crate::understanding::rx("\\bfor\\b\\s*\\??\\s*$|\\bused for\\b|\\bdo(?:es)? (?:it|that|this) do\\b").is_match(&text::lower(text::strip(clean)))
}

/// `_a_number(word)`.
pub fn _a_number(word: &str) -> bool {
    word.chars().any(|c| crate::understanding::pyuni::ch_digit(c))
}

/// `_the_thing_in(sentence, thing)`: where the thing is said in the sentence, and its words.
pub fn _the_thing_in(sentence: &str, thing: &[String]) -> Option<(usize, Vec<String>)> {
    let words = lower_all(&tokens(sentence));
    let wanted: Vec<Forms> = thing.iter().map(|w| _forms(w)).collect();
    if words.len() + 1 > wanted.len() {
        for start in 0..words.len() + 1 - wanted.len() {
            if words[start..start + wanted.len()].iter().zip(&wanted).all(|(w, want)| meets(&_forms(w), want)) {
                return Some((start, words));
            }
        }
    }
    None
}

/// `_the_reply_defines(reply, thing, purpose=False)`: the reply's first sentence defines the
/// thing ("A ... is ...").
pub fn _the_reply_defines(reply: &Reply, thing: &[String], purpose: bool) -> bool {
    let Reply::Text(reply) = reply else { return false };
    if text::strip(reply).is_empty() {
        return false;
    }
    let body = text::strip(reply);
    let opening = match crate::understanding::rx("(?<=[.!?])\\s").search(body) {
        Some(m) => &body[..m.start()],
        None => body,
    };
    let Some((start, words)) = _the_thing_in(opening, thing) else { return false };
    if start > *_SUBJECT_AT_MOST {
        return false;
    }
    if purpose {
        return true;
    }
    if start > 0 && !_DEFINITION_ARTICLES.contains(&words[0]) {
        return false;
    }
    let before = &words[..start];
    if before.iter().any(|w| _BE_VERBS.contains(w)) {
        return false;
    }
    if let Some(last) = before.last() {
        if _BEFORE_THE_THING.contains(last) || _a_number(last) {
            return false;
        }
    }
    let after = words.get(start + thing.len());
    after.is_some_and(|a| _DEFINING_VERBS.contains(a))
}

/// `_the_key_names_the_thing(item, thing)`.
pub fn _the_key_names_the_thing(item: &Entry, thing: &[String]) -> bool {
    let wanted: Vec<Forms> = thing.iter().map(|w| _forms(w)).collect();
    for part in &item.key {
        for alternative in alternatives(part) {
            let words = text::split(&text::lower(alternative));
            if words.len() != wanted.len() {
                continue;
            }
            if words.iter().zip(&wanted).all(|(w, want)| meets(&_forms(w), want)) {
                return true;
            }
        }
    }
    false
}

/// `_said_as_a_report(understood, words, entries, reject=None)`: the words read as "i have a
/// ...".
pub fn _said_as_a_report(understood: &Understood, words: &[String], entries: Option<&[&'static Entry]>, reject: Reject) -> Option<&'static Entry> {
    let mut second = understood.clone();
    second.clean = format!("i have a {}", words.join(" "));
    second.words = ["i", "have", "a"].iter().map(|w| w.to_string()).chain(words.iter().cloned()).collect();
    second.background.clear();
    second.intensifier.clear();
    second.intensifier_left.clear();
    second.leans_on = None;
    second.topic_own = None;
    second.extra.shift_remove("past");
    second.extra.shift_remove("handoff");
    _the_best_entry(&second, entries, reject, false)
}

/// `_the_name_card(typed, entries, reject=None, allowed=None)`: the row whose first key phrase
/// is exactly the line's words.
pub fn _the_name_card(typed: &[String], entries: &[&'static Entry], reject: Reject, allowed: Option<&HashSet<String>>) -> Option<&'static Entry> {
    let said: HashSet<&String> = typed.iter().collect();
    let mut found: Vec<&'static Entry> = Vec::new();
    for &item in entries {
        if !NAME_ALONE_DOMAINS.contains(&item.domain) {
            continue;
        }
        if rejected(reject, item) {
            continue;
        }
        if allowed.is_some_and(|a| !a.contains(&item.domain)) {
            continue;
        }
        let Some(part) = item.key.first() else { continue };
        let Some(first) = alternatives(part).first() else { continue };
        let phrase: HashSet<String> = text::split(&text::lower(first)).into_iter().collect();
        if phrase.len() >= 2 && phrase.len() == said.len() && phrase.iter().all(|w| said.contains(w)) {
            found.push(item);
        }
    }
    if found.is_empty() {
        return None;
    }
    if found.len() == 1 {
        return Some(found[0]);
    }
    let names_itself = |item: &Entry| -> i32 {
        let name = item.key.first().and_then(|p| alternatives(p).first()).cloned().unwrap_or_default();
        let reply = match &item.reply {
            Reply::Text(s) => s.clone(),
            _ => panic!("AttributeError: the reply of {} has no attribute 'lower'", item.id),
        };
        let reply: String = text::lower(&reply).chars().take(140).collect();
        if reply.contains(&text::lower(&name)) {
            0
        } else {
            1
        }
    };
    let mut best = found[0];
    let mut best_key = names_itself(best);
    for &item in &found[1..] {
        let k = names_itself(item);
        if k < best_key {
            best = item;
            best_key = k;
        }
    }
    Some(best)
}

/// `_the_name_alone(named, entries, reject, allowed=None, understood=None, words=None,
/// use_context=False)`: a line that is only the row's name, in a domain the line's topic
/// declares.
pub fn _the_name_alone(
    named: Option<&'static Entry>,
    _entries: &[&'static Entry],
    _reject: Reject,
    allowed: Option<&HashSet<String>>,
    understood: Option<&Understood>,
    words: &[String],
    _use_context: bool,
) -> Option<&'static Entry> {
    let typed = lower_all(words);
    if !(2..=4).contains(&typed.len()) {
        return None;
    }
    if typed.iter().any(|w| NOT_A_NAME_ALONE.contains(w)) {
        return None;
    }
    if typed.iter().any(|w| A_STATE_IN_THE_PHRASE.contains(w)) {
        return None;
    }
    if A_STATE_AS_A_NAME_END.contains(&typed[typed.len() - 1]) {
        return None;
    }
    let named = named?;
    let allowed = allowed.filter(|a| !a.is_empty())?;
    let declared;
    let allowed = match understood {
        Some(u) => {
            declared = _declared_for(allowed, u);
            &declared
        }
        None => allowed,
    };
    if !allowed.contains(&named.domain) {
        return None;
    }
    let said: HashSet<&String> = typed.iter().collect();
    if said.len() != typed.len() {
        return None;
    }
    Some(named)
}

/// `by_topic()` as a lookup.
fn judged(name: &str) -> Option<&'static Vec<String>> {
    static BY: LazyLock<HashMap<&'static str, &'static Vec<String>>> =
        LazyLock::new(|| crate::knowledge::by_topic().iter().map(|(k, v)| (k.as_str(), v)).collect());
    BY.get(name).copied()
}

/// The reading's topics (`topics` then `topic`), non-empty strings.
fn reading_topics(understood: &Understood, with_leans: bool) -> Vec<String> {
    let mut topics: Vec<String> = understood.topics.clone();
    if let Some(t) = &understood.topic {
        topics.push(t.clone());
    }
    if with_leans {
        if let Some(t) = understood.leans_on.as_ref().and_then(|l| l.topic.clone()) {
            topics.push(t);
        }
    }
    topics.retain(|t| !t.is_empty());
    topics
}

/// `_declared_for(allowed, understood)`: the allowed domains that declare the line's topic.
pub fn _declared_for(allowed: &HashSet<String>, understood: &Understood) -> HashSet<String> {
    let topics = reading_topics(understood, false);
    if topics.is_empty() {
        return HashSet::new();
    }
    allowed.iter().filter(|name| topics.iter().any(|t| judged(name).is_some_and(|j| j.contains(t)))).cloned().collect()
}

/// `_defines(understood, forms, typed, entries, reject=None)`: the row whose reply defines the
/// thing "what is a ..." asks about. (The key test runs before the guard: both are pure tests,
/// so the order does not change which row is found.)
pub fn _defines(understood: &Understood, forms: &FormLine, _typed: &[String], entries: &[&'static Entry], reject: Reject) -> Option<&'static Entry> {
    let said = first_text(understood, &["clean", "text"]);
    let thing = _the_thing_asked_about(said);
    if thing.is_empty() {
        return None;
    }
    let purpose = _asks_what_it_is_for(said);
    let mut best: Option<&'static Entry> = None;
    let mut best_rank: Option<i64> = None;
    for &item in entries {
        if rejected(reject, item) {
            continue;
        }
        if !_the_key_names_the_thing(item, &thing) {
            continue;
        }
        if !guard_passes(item, understood) {
            continue;
        }
        if !_the_reply_defines(&item.reply, &thing, purpose) {
            continue;
        }
        if item.avoid.iter().any(|w| _part_score(Part::Str(w), forms, None) != 0) {
            continue;
        }
        let rank = -(_key_forms(item).len() as i64);
        if best_rank.is_none_or(|b| rank > b) {
            best = Some(item);
            best_rank = Some(rank);
        }
    }
    best
}

/// `_named_things(understood)`: the places, names and titles the line names.
pub fn _named_things(understood: &Understood) -> Vec<String> {
    understood
        .entities
        .iter()
        .filter(|e| matches!(e.kind.as_str(), "place" | "name" | "title"))
        .map(|e| text::split(&text::lower(&e.text)).join(" "))
        .filter(|n| text::len(n) > 2)
        .collect()
}

/// `_plans_it(item, forms)`: the line and the key both say planning.
pub fn _plans_it(item: &Entry, forms: &FormLine) -> bool {
    if !PLAN_WORDS.iter().any(|w| _part_score(Part::Str(w), forms, None) != 0) {
        return false;
    }
    let key = _key_words(item);
    PLAN_WORDS.iter().any(|w| key.contains(w))
}

/// `_names_one(item, named)`: the key names one of the named things.
pub fn _names_one(item: &Entry, named: &[String]) -> bool {
    for part in &item.key {
        for alternative in alternatives(part) {
            let line = text::split(&text::lower(alternative)).join(" ");
            for name in named {
                if _bounded(name).is_match(&line) {
                    return true;
                }
            }
        }
    }
    false
}

/// `_only_asking_about_it(understood, leans)`: the line adds nothing to what it leans on.
pub fn _only_asking_about_it(understood: &Understood, leans: &LeansOn) -> bool {
    let mut earlier: HashSet<String> = tokens(&leans.text).into_iter().collect();
    earlier.extend(leans.words.iter().map(|w| text::lower(w)));
    let now = tokens(first_text(understood, &["clean", "text"]));
    if now.is_empty() {
        return false;
    }
    for word in now {
        if earlier.contains(&word) || GRAMMAR_ONLY.contains(&word) || POINTS_BACK.contains(&word) {
            continue;
        }
        if FOLLOW_ON_WORDS.contains(&word) || crate::understanding::pyuni::is_digit(&word) {
            continue;
        }
        return false;
    }
    true
}

/// `names_nothing_of_its_own(understood)`.
pub fn names_nothing_of_its_own(understood: &Understood) -> bool {
    match &understood.leans_on {
        Some(leans) => _only_asking_about_it(understood, leans),
        None => false,
    }
}

/// `leans_on_a_subject(understood)`: the line can take its subject from the line before.
pub fn leans_on_a_subject(understood: &Understood) -> bool {
    if understood.kind == "question" {
        return true;
    }
    if _says_how_the_speaker_handles_it(understood) {
        return true;
    }
    tokens(first_text(understood, &["clean", "text"])).iter().any(|w| POINTS_BACK.contains(w))
}

/// `_the_names_in_the_ask(understood, every)`: the rare words of a two-clause line's ask.
pub fn _the_names_in_the_ask(understood: &Understood, every: &HashMap<String, usize>) -> Vec<String> {
    let Some(focus) = _two_clauses(understood) else { return vec![] };
    let mut names: Vec<String> = Vec::new();
    let pick = |cap: usize, names: &mut Vec<String>| {
        for word in &focus {
            let low = text::lower(word);
            if text::len(&low) < 3 || _NOT_A_SUBJECT.contains(&low) || ASK_FRAME_WORDS.contains(&low) {
                continue;
            }
            if every.get(&low).copied().unwrap_or(0) > cap || names.contains(&low) {
                continue;
            }
            names.push(low);
        }
    };
    pick(*ASK_NAME_CAP, &mut names);
    let plain = !understood.asks.iter().any(|a| a == "yesno");
    if names.is_empty() && plain {
        pick(*ASK_PLAIN_CAP, &mut names);
    }
    names
}

/// `_the_ask_in_order(understood)`.
pub fn _the_ask_in_order(understood: &Understood) -> Vec<String> {
    _two_clauses(understood).unwrap_or_default()
}

/// `_two_clauses(understood)`: the ask's words when the line is a long statement and then an
/// ask about something in it.
pub fn _two_clauses(understood: &Understood) -> Option<Vec<String>> {
    if understood.asks.is_empty() {
        return None;
    }
    let focus = &understood.focus;
    let clauses: Vec<&Vec<String>> = understood.clauses.iter().filter(|c| !(c.len() == 1 && c[0] == "text")).collect();
    if focus.is_empty() || clauses.len() < 2 {
        return None;
    }
    let last: Vec<Forms> = clauses[clauses.len() - 1].iter().map(|w| _forms(w)).collect();
    let earlier: Vec<Forms> = clauses[..clauses.len() - 1].iter().flat_map(|c| c.iter().map(|w| _forms(w))).collect();
    let longest = clauses[..clauses.len() - 1].iter().map(|c| c.len()).max().unwrap_or(0);
    if last.is_empty() || earlier.is_empty() || longest < 4 {
        return None;
    }
    let has = |set: &[Forms], form: &Forms| set.iter().any(|f| **f == **form);
    for word in focus {
        let form = _forms(word);
        if has(&last, &form) || !has(&earlier, &form) {
            continue;
        }
        return None;
    }
    Some(focus.clone())
}

/// One ranked row of `_scanned_rows`: (asked, ranked, plans, named, len(key), item, run).
#[derive(Clone, Debug)]
pub struct Row {
    pub asked: i64,
    pub ranked: i64,
    pub plans: bool,
    pub named: bool,
    pub keys: usize,
    pub item: &'static Entry,
    pub run: usize,
}

/// `_row_order(row)`.
pub fn _row_order(row: &Row) -> (i64, usize, i64, bool, bool, usize) {
    (row.asked, row.run, row.ranked, row.plans, row.named, row.keys)
}

/// `rows.sort(key=_row_order, reverse=True)` (stable).
fn sort_rows(rows: &mut [Row]) {
    rows.sort_by(|a, b| _row_order(b).cmp(&_row_order(a)));
}

/// `max(rows, key=_row_order)`: the first of the best.
fn best_row(rows: &[Row]) -> &Row {
    let mut best = &rows[0];
    for row in &rows[1..] {
        if _row_order(row) > _row_order(best) {
            best = row;
        }
    }
    best
}

/// `_line_rows(forms, typed, entries, by_domain, every, named=(), frame=True,
/// stands_down=None, framed=False, ask_names=(), ask_said=())`: the scanned rows, without the
/// ones that stand down.
#[allow(clippy::too_many_arguments)]
pub fn _line_rows(
    forms: &FormLine,
    typed: &[String],
    entries: &[&'static Entry],
    by_domain: &HashMap<String, HashMap<String, usize>>,
    every: &HashMap<String, usize>,
    named: &[String],
    frame: bool,
    stands_down: Option<&dyn Fn(&Entry) -> bool>,
    framed: bool,
    ask_names: &[String],
    ask_said: &[String],
) -> Vec<Row> {
    let rows = _scanned_rows(forms, typed, entries, by_domain, every, named, frame, framed, ask_names, ask_said);
    match stands_down {
        None => rows,
        Some(s) => rows.into_iter().filter(|r| !s(r.item)).collect(),
    }
}

/// `_scanned_rows(...)`: every reached entry the line scores, ranked.
#[allow(clippy::too_many_arguments)]
pub fn _scanned_rows(
    forms: &FormLine,
    typed: &[String],
    entries: &[&'static Entry],
    by_domain: &HashMap<String, HashMap<String, usize>>,
    every: &HashMap<String, usize>,
    named: &[String],
    frame: bool,
    framed: bool,
    ask_names: &[String],
    ask_said: &[String],
) -> Vec<Row> {
    let ask_forms: Vec<Forms> = ask_names.iter().map(|w| _forms(w)).collect();
    let ask_line = FormLine::of(ask_said);
    let seen = new_seen();
    let asked_seen = new_seen();
    let mut rows: Vec<Row> = Vec::new();
    for item in _reaching(entries, forms) {
        let value = score(item, forms, Some(typed), Some(&seen), frame, framed);
        if value == 0 {
            continue;
        }
        let rarity = by_domain.get(&item.domain).filter(|d| !d.is_empty()).unwrap_or(every);
        let ranked = _rank(item, forms, rarity, Some(&seen));
        let asked = if _only_the_want(item, forms, Some(&seen)) { 0 } else { 1 };
        let run = _the_longest_run(item, &ask_line, &ask_forms, &asked_seen);
        rows.push(Row { asked, ranked, plans: _plans_it(item, forms), named: _names_one(item, named), keys: item.key.len(), item, run });
    }
    let longest = rows.iter().map(|r| r.run).max().unwrap_or(0);
    if longest != 0 {
        for row in &mut rows {
            row.run = if row.run == longest { 1 } else { 0 };
        }
    }
    rows
}

/// `_the_longest_run(item, ask_line, name_forms, asked_seen)`: the longest key phrase the ask
/// says that holds one of its names.
pub fn _the_longest_run(item: &Entry, ask_line: &FormLine, name_forms: &[Forms], asked_seen: &SeenRef) -> usize {
    let mut wanted = HashSet::new();
    for f in name_forms {
        wanted.extend(f.iter().cloned());
    }
    let mut best = 0;
    for part in &item.key {
        for hit in _part_matches(part.into(), ask_line, Some(asked_seen)).iter() {
            if hit.weight == 0 || hit.span.len() <= best {
                continue;
            }
            if !hit.words.iter().any(|w| meets(&_forms(w), &wanted)) {
                continue;
            }
            best = hit.span.len();
        }
    }
    best
}

/// `_plainer(plain, framed, margin=PLAIN_MARGIN)`: the unframed reading's best row is plainly
/// better.
pub fn _plainer(plain: &[Row], framed: &[Row], margin: i64) -> bool {
    let better = plain.iter().map(|r| (r.asked, r.ranked)).max().unwrap();
    let other = framed.iter().map(|r| (r.asked, r.ranked)).max().unwrap();
    better.0 > other.0 || better.1 >= other.1 + margin
}

/// `_the_subjects_own_row(forms, item, rows)`.
pub fn _the_subjects_own_row(forms: &[Forms], item: &Entry, rows: &[Row]) -> bool {
    if item.domain.is_empty() || !_a_clause_about_the_thing(forms) {
        return false;
    }
    if !rows.iter().any(|r| r.item.domain == item.domain) {
        return false;
    }
    _offers_a_substitute(best_row(rows).item)
}

/// `_offers_a_substitute(item)`.
pub fn _offers_a_substitute(item: &Entry) -> bool {
    item.key.iter().any(|part| alternatives(part).iter().any(|a| text::split(&text::lower(a)).iter().any(|w| SUBSTITUTE_WORDS.contains(w))))
}

/// `_asks_if_it_is_still_good(typed)`.
pub fn _asks_if_it_is_still_good(typed: &[String]) -> bool {
    typed.iter().any(|w| _STILL_GOOD_ASHERS.contains(&text::lower(w)))
}

/// `_a_keeping_row(item)`: the reply says how long a thing keeps.
pub fn _a_keeping_row(item: &Entry) -> bool {
    let Reply::Text(reply) = &item.reply else { return _a_keeping_key(item) };
    if reply.is_empty() {
        return false;
    }
    let reply = format!(" {} ", text::split(&text::lower(reply)).join(" "));
    _KEEPING_ANSWER.iter().any(|p| reply.contains(p.as_str())) || _A_KEEP_AND_A_LENGTH.is_match(&reply)
}

/// `_a_keeping_key(item)`.
pub fn _a_keeping_key(item: &Entry) -> bool {
    _key_words(item).iter().any(|w| _KEEPING_WORDS.contains(w))
}

/// `_a_dress_code(words, index)`: "black tie".
pub fn _a_dress_code(words: &[String], index: usize) -> bool {
    index + 1 < words.len() && (words[index] == "black" || words[index] == "white") && A_DRESS_CODE_AFTER.contains(&text::lower(&words[index + 1]))
}

/// `string.punctuation`.
const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// `_the_colour_is_the_whole_ask(words, index)`.
pub fn _the_colour_is_the_whole_ask(words: &[String], index: usize) -> bool {
    if index + 1 >= words.len() {
        return true;
    }
    let mut after = index + 1;
    if text::len(&words[after]) == 1 && after + 1 < words.len() {
        after += 1;
    }
    let low = text::lower(&words[after]);
    let word = strip_chars(&low, PUNCTUATION);
    if word.is_empty() {
        return true;
    }
    AFTER_A_COLOUR_STANDS.contains(word) || WORN_AFTER_A_COLOUR.contains(word) || COLOUR_WORDS.contains(word)
}

/// `a_colour_is_worn(understood)` for a line of text (Python accepts a str).
pub fn a_colour_is_worn_text(said: &str) -> bool {
    let u = Understood { text: said.to_string(), ..Default::default() };
    a_colour_is_worn(&u)
}

/// `a_colour_is_worn(understood)`: the line asks what colour to wear.
pub fn a_colour_is_worn(understood: &Understood) -> bool {
    let mut said = first_text(understood, &["text", "clean"]).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    let typing = tokens(&said);
    let typed: HashSet<&String> = typing.iter().collect();
    if !typed.iter().any(|w| COLOUR_ASKED_WORDS.contains(*w)) {
        return false;
    }
    if !typing.iter().enumerate().any(|(index, w)| COLOUR_ASKED_WORDS.contains(w) && !_a_dress_code(&typing, index) && _the_colour_is_the_whole_ask(&typing, index)) {
        return false;
    }
    if !(typed.iter().any(|w| WEARING.contains(*w)) || OCCASION.is_match(&typing.join(" "))) {
        return false;
    }
    if understood.kind == "question" || said.contains('?') {
        return true;
    }
    let first = tokens(&said).into_iter().find(|w| w != "so").unwrap_or_default();
    ASKING_FIRST.contains(&first) || A_COLOUR_ASKED.is_match(&said)
}

/// `_the_words_without_a_colour(clean, words)`.
pub fn _the_words_without_a_colour(clean: &str, words: &[String]) -> Option<Vec<String>> {
    let said: HashSet<String> = A_COLOUR_USED.finditer(clean).iter().map(|m| text::lower(m.name_str("colour"))).filter(|c| COLOUR_WORDS.contains(c)).collect();
    if said.is_empty() {
        return None;
    }
    let kept: Vec<String> = words.iter().filter(|w| !said.contains(&text::lower(w))).cloned().collect();
    if kept.is_empty() || kept.len() == words.len() {
        return None;
    }
    Some(kept)
}

/// `_the_key_words(entry)`.
pub fn _the_key_words(entry: &Entry) -> HashSet<String> {
    _key_words(entry)
}

/// `_the_occasion_family(phrase)`.
pub fn _the_occasion_family(phrase: &str) -> String {
    let word = text::lower(text::strip(phrase));
    for (family, members) in THE_SAME_OCCASION.iter() {
        if members.contains(&word) || word.starts_with(family.as_str()) {
            return family.clone();
        }
    }
    String::new()
}

/// `_the_occasion_words(typed)`.
pub fn _the_occasion_words(typed: &[String]) -> HashSet<String> {
    let said = typed.join(" ");
    let mut found = HashSet::new();
    for m in OCCASION.finditer(&said) {
        let family = _the_occasion_family(m.as_str());
        if !family.is_empty() {
            if let Some((_, members)) = THE_SAME_OCCASION.iter().find(|(f, _)| *f == family) {
                found.extend(members.iter().cloned());
            }
        }
    }
    found
}

/// `_the_card_for_the_occasion_asked(...)`: the clothing row for the occasion the line names.
#[allow(clippy::too_many_arguments)]
pub fn _the_card_for_the_occasion_asked(
    found: Option<&'static Entry>,
    understood: &Understood,
    words: &[String],
    clean: &str,
    forms: &FormLine,
    typed: &[String],
    entries: &[&'static Entry],
    by_domain: &HashMap<String, HashMap<String, usize>>,
    every: &HashMap<String, usize>,
    reject: Reject,
) -> Option<&'static Entry> {
    let asked = _the_occasion_words(typed);
    if asked.is_empty() {
        return found;
    }
    if let Some(f) = found {
        if _the_key_words(f).iter().any(|w| asked.contains(w)) {
            return found;
        }
    }
    let for_this: Vec<&'static Entry> = entries.iter().copied().filter(|e| e.domain == "clothing" && _the_key_words(e).iter().any(|w| asked.contains(w))).collect();
    if for_this.is_empty() {
        return found;
    }
    _best(understood, words, clean, forms, typed, &for_this, by_domain, every, reject, false).or(found)
}

/// `_the_answer_the_colour_dragged_in(...)`: the row found again without a colour that is not
/// what the line asks about.
#[allow(clippy::too_many_arguments)]
pub fn _the_answer_the_colour_dragged_in(
    found: Option<&'static Entry>,
    understood: &Understood,
    words: &[String],
    clean: &str,
    entries: &[&'static Entry],
    by_domain: &HashMap<String, HashMap<String, usize>>,
    every: &HashMap<String, usize>,
    reject: Reject,
) -> Option<&'static Entry> {
    let Some(kept) = _the_words_without_a_colour(clean, words) else { return found };
    let f = found?;
    let others: HashSet<String> = kept.iter().map(|w| text::lower(w)).filter(|w| !_BARE_IN_A_KEY.contains(w)).collect();
    if _the_key_words(f).iter().any(|w| others.contains(w)) {
        return found;
    }
    _best(understood, &kept, &kept.join(" "), &FormLine::of(&kept), &lower_all(&kept), entries, by_domain, every, reject, false).or(found)
}

/// `_best(understood, words, clean, forms, typed, entries, by_domain, every, reject=None,
/// framed=False)`: the best entry for the line as read.
#[allow(clippy::too_many_arguments)]
pub fn _best(
    understood: &Understood,
    words: &[String],
    clean: &str,
    forms: &FormLine,
    typed: &[String],
    entries: &[&'static Entry],
    by_domain: &HashMap<String, HashMap<String, usize>>,
    every: &HashMap<String, usize>,
    reject: Reject,
    framed: bool,
) -> Option<&'static Entry> {
    let line_parts = _body_parts(words);
    let keeping_question = understood.kept.is_some() && _asks_if_it_is_still_good(typed);
    let ask_names = _the_names_in_the_ask(understood, every);
    let ask_said = _the_ask_in_order(understood);
    let degrees_are_heat = _the_degrees_are_a_temperature(understood);
    let aged_a_thing = the_years_are_a_things(understood);
    let mut cold_place = None;
    if !_speaker_is_feeling(typed) && !_the_person_says_they_are_cold(typed) {
        cold_place = _the_place_the_line_says_is_cold(typed);
    }
    let rank = |reading: &Understood, filter_keeping: bool| -> (Vec<Row>, Option<Vec<Row>>) {
        let stands_down = |item: &Entry| -> bool {
            if !guard_passes(item, reading) {
                return true;
            }
            if degrees_are_heat && _answers_a_qualification(item) {
                return true;
            }
            if aged_a_thing && _the_age_is_a_childs(item) {
                return true;
            }
            if filter_keeping && !_a_keeping_row(item) {
                return true;
            }
            _another_part(item, &line_parts)
        };
        let named = _named_things(reading);
        let mut reached = _line_rows(forms, typed, entries, by_domain, every, &named, true, Some(&stands_down), framed, &ask_names, &ask_said);
        let mut plain_rows = None;
        if !_frame_masked(forms).1.is_empty() {
            let mut plain = _line_rows(forms, typed, entries, by_domain, every, &named, false, Some(&stands_down), framed, &ask_names, &ask_said);
            if !plain.is_empty() {
                sort_rows(&mut plain);
                if reached.is_empty() || _plainer(&plain, &reached, *PLAIN_MARGIN) || _offers_a_substitute(plain[0].item) || _the_subjects_own_row(forms, plain[0].item, &reached) {
                    reached = plain.clone();
                }
            }
            plain_rows = Some(plain);
        }
        (reached, plain_rows)
    };
    let (mut ranked_items, mut plain) = rank(understood, keeping_question);
    if ranked_items.is_empty() && keeping_question {
        let mut without_kept = understood.clone();
        without_kept.kept = None;
        (ranked_items, plain) = rank(&without_kept, false);
    }
    if ranked_items.is_empty() {
        return _other_reading(understood, words, clean, entries, reject);
    }
    ranked_items.retain(|r| r.ranked > 1);
    if ranked_items.is_empty() {
        return _other_reading(understood, words, clean, entries, reject);
    }
    sort_rows(&mut ranked_items);
    if let Some(place) = &cold_place {
        ranked_items = _the_card_for_the_place_first(place, ranked_items);
    }
    let carried = FormLine::new(_the_subject_the_line_leans_on(understood));
    let ask_thing = understood.ask_subject.as_deref();
    let scene = understood.setting.as_deref();
    let scene_only = match (ask_thing, scene) {
        (Some(a), Some(s)) => a != s && !_says_what_is_missing(understood) && _is_a_person(understood, a) == _is_a_person(understood, s),
        _ => false,
    };
    let mut turned_down: HashSet<&str> = HashSet::new();
    for whole_line_first in [true, false] {
        for row in &ranked_items {
            let item = row.item;
            if rejected(reject, item) {
                turned_down.insert(item.id.as_str());
                continue;
            }
            if _the_subject_it_avoids(item, forms, &carried, None) {
                continue;
            }
            if !scene_only {
                return Some(item);
            }
            let says_ask = about(item, ask_thing);
            let says_scene = about(item, scene);
            if says_scene && !says_ask {
                continue;
            }
            if says_scene && whole_line_first {
                continue;
            }
            return Some(item);
        }
        turned_down.clear();
    }
    if let Some(plain) = &plain {
        for row in plain {
            let item = row.item;
            if turned_down.contains(item.id.as_str()) {
                continue;
            }
            if rejected(reject, item) {
                continue;
            }
            if _the_subject_it_avoids(item, forms, &carried, None) {
                continue;
            }
            return Some(item);
        }
    }
    _other_reading(understood, words, clean, entries, reject)
}

/// `_household_reading(words)`: "we" read as "i"; None when nothing changes.
pub fn _household_reading(words: &[String]) -> Option<Vec<String>> {
    let shifted: Vec<String> = words.iter().map(|w| HOUSEHOLD_WORDS.get(&text::lower(w)).cloned().unwrap_or_else(|| w.clone())).collect();
    if shifted != words {
        Some(shifted)
    } else {
        None
    }
}

/// `_other_reading(understood, words, clean, entries, reject)`: the line found again as the
/// engine's plain line, its tidied words, or said of the speaker.
pub fn _other_reading(understood: &Understood, words: &[String], clean: &str, entries: &[&'static Entry], reject: Reject) -> Option<&'static Entry> {
    let plain = &understood.plain;
    if !plain.is_empty() && plain != clean {
        let mut again = understood.clone();
        again.plain = String::new();
        again.clean = plain.clone();
        again.words = _words_of(plain).iter().map(|w| text::lower(w)).collect();
        if let Some(found) = find(&again, Some(entries), reject, false) {
            return Some(found);
        }
    }
    let tidied = &understood.norm;
    let typed = lower_all(words);
    if !tidied.is_empty() && *tidied != typed {
        let mut again = understood.clone();
        again.plain = String::new();
        again.clean = tidied.join(" ");
        again.words = tidied.clone();
        again.norm = vec![];
        return find(&again, Some(entries), reject, false);
    }
    if let Some(shifted) = _household_reading(words) {
        let mut again = understood.clone();
        again.plain = String::new();
        again.norm = vec![];
        again.clean = shifted.join(" ");
        again.words = shifted;
        return find(&again, Some(entries), reject, false);
    }
    None
}

/// `_says_a_key_word(item, forms, rarity=None)`: the line says one of the key's specific words.
pub fn _says_a_key_word(item: &Entry, forms: &FormLine, rarity: &HashMap<String, usize>) -> bool {
    for part in &item.key {
        for hit in _part_matches(part.into(), forms, None).iter() {
            if hit.words.len() != 1 {
                continue;
            }
            if hit.words[0].chars().any(|c| crate::understanding::pyuni::ch_digit(c)) || rarity.get(&hit.words[0]).copied().unwrap_or(0) <= *SPECIFIC {
                return true;
            }
        }
    }
    false
}

/// `_touches(item, forms, rarity=None)`: the line says a phrase or a specific word of the key.
pub fn _touches(item: &Entry, forms: &FormLine, rarity: &HashMap<String, usize>) -> bool {
    for part in &item.key {
        for hit in _part_matches(part.into(), forms, None).iter() {
            let words = &hit.words;
            if !words.is_empty() && words.iter().all(|w| A_SAYS_NOTHING.contains(w)) {
                continue;
            }
            if words.len() > 1 || words[0].chars().any(|c| crate::understanding::pyuni::ch_digit(c)) || rarity.get(&words[0]).copied().unwrap_or(0) <= *SPECIFIC {
                return true;
            }
        }
    }
    false
}

/// `_parts_across(item, carried_forms, forms)`: the key is said partly before and partly now.
pub fn _parts_across(item: &Entry, carried_forms: &FormLine, forms: &FormLine) -> bool {
    let mut own = HashSet::new();
    let mut above = HashSet::new();
    for (index, part) in item.key.iter().enumerate() {
        if !_part_matches(part.into(), forms, None).is_empty() {
            own.insert(index);
        }
        if !_part_matches(part.into(), carried_forms, None).is_empty() {
            above.insert(index);
        }
    }
    own.difference(&above).next().is_some() && above.difference(&own).next().is_some()
}

/// `key_words(item)`: the key's words without a plural "s".
pub fn key_words(item: &Entry) -> HashSet<String> {
    let mut found = HashSet::new();
    for part in &item.key {
        for option in alternatives(part) {
            for word in text::split(&text::lower(option)) {
                let bare = word.trim_end_matches('s');
                found.insert(if bare.is_empty() { word.clone() } else { bare.to_string() });
            }
        }
    }
    found
}

/// `about(item, subject)`: the subject's head word is in the key.
pub fn about(item: &Entry, subject: Option<&str>) -> bool {
    let words = text::split(subject.unwrap_or(""));
    let head = words.last().cloned().unwrap_or_default();
    let bare = head.trim_end_matches('s');
    let head = if bare.is_empty() { head.clone() } else { bare.to_string() };
    !head.is_empty() && key_words(item).contains(&head)
}

/// `a_weight(understood)`: the line gives a weight.
pub fn a_weight(understood: &Understood) -> bool {
    understood.numbers.iter().any(|n| WEIGHT_UNITS.contains(&text::lower(n.unit.as_deref().unwrap_or(""))))
}

/// `_refers_to_the_person(understood)`.
pub fn _refers_to_the_person(understood: &Understood) -> bool {
    understood.norm.iter().any(|w| PERSON_PRONOUNS.contains(w))
}

/// `_says_how_the_speaker_handles_it(understood)`: "i always just nod".
pub fn _says_how_the_speaker_handles_it(understood: &Understood) -> bool {
    let line = tokens(first_text(understood, &["clean", "text"]));
    if line.is_empty() || !SPEAKER_ONLY.contains(&line[0]) {
        return false;
    }
    if understood.keywords.is_empty() {
        return false;
    }
    understood.keywords.iter().all(|w| HANDLING_WORDS.contains(w))
}

/// `_following_on(understood, entries, forms, typed, by_domain, every, known=(), reject=None)`:
/// the entry the line and the words it leans on find together.
#[allow(clippy::too_many_arguments)]
pub fn _following_on(
    understood: &Understood,
    entries: &[&'static Entry],
    forms: &FormLine,
    typed: &[String],
    by_domain: &HashMap<String, HashMap<String, usize>>,
    every: &HashMap<String, usize>,
    known: &HashSet<String>,
    reject: Reject,
) -> Option<&'static Entry> {
    let leans = understood.leans_on.as_ref()?;
    let carried: Vec<String> = leans.words.iter().take(*CONTEXT_WORDS).cloned().collect();
    if carried.is_empty() {
        return None;
    }
    let mut carried_forms = FormLine::of(&carried);
    _read_typos(&carried, &mut carried_forms, every, known);
    let together = FormLine::new(carried_forms.iter().chain(forms.iter()).cloned().collect());
    let together_typed: Vec<String> = carried.iter().chain(typed.iter()).cloned().collect();
    let allowed = _topic_domains(understood, Some(entries));
    let mut best: Option<&'static Entry> = None;
    let mut best_score = 0;
    let mut best_domain = String::new();
    let seen = new_seen();
    let named_subject = leans.subject.clone().unwrap_or_default();
    let rarity_of = |domain: &str| by_domain.get(domain).filter(|d| !d.is_empty()).unwrap_or(every);
    for item in _reaching(entries, &together) {
        let domain = &item.domain;
        if allowed.as_ref().is_some_and(|a| !a.contains(domain)) {
            continue;
        }
        if score(item, &together, Some(&together_typed), Some(&seen), true, false) == 0 {
            continue;
        }
        if rejected(reject, item) {
            continue;
        }
        if !guard_passes(item, understood) {
            continue;
        }
        let mut ranked = _rank(item, &together, rarity_of(domain), Some(&seen));
        if ranked != 0 && !named_subject.is_empty() && about(item, Some(&named_subject)) {
            ranked += 100;
        }
        if ranked > best_score {
            best = Some(item);
            best_score = ranked;
            best_domain = domain.clone();
        }
    }
    if _only_asking_about_it(understood, leans) {
        let back = _the_best_entry(&leans_reading(leans), Some(entries), reject, false);
        if let Some(b) = back {
            if !best.is_some_and(|bst| !std::ptr::eq(bst, b) && _parts_across(bst, &carried_forms, forms)) {
                best = Some(b);
                best_domain = b.domain.clone();
            }
        }
    }
    let best = best?;
    if !_touches(best, forms, rarity_of(&best_domain)) && !_parts_across(best, &carried_forms, forms) && !_only_asking_about_it(understood, leans) {
        return None;
    }
    let said = _says_a_key_word(best, forms, rarity_of(&best_domain));
    if !_reaches(best, &carried_forms) {
        return None;
    }
    let subject = leans.subject.as_deref().unwrap_or("");
    let named = !subject.is_empty();
    if named && !about(best, Some(subject)) {
        return None;
    }
    if named
        && leans.subject_type.as_deref() == Some("person")
        && understood.kind != "question"
        && understood.kind != "request"
        && _refers_to_the_person(understood)
        && !said
    {
        let head = _forms(text::split(subject).last().map(|s| s.as_str()).unwrap_or(""));
        let other: Vec<Forms> = carried_forms.iter().filter(|f| !meets(f, &head)).cloned().collect();
        let mut words = HashSet::new();
        for word in key_words(best) {
            words.extend(_forms(&word).iter().cloned());
        }
        if !other.is_empty() && !_reaches(best, &FormLine::new(other.clone())) && !other.iter().any(|f| meets(f, &words)) {
            return None;
        }
    }
    if !named && !_touches(best, &carried_forms, rarity_of(&best_domain)) {
        return None;
    }
    if !_in_topic(best, understood) {
        return None;
    }
    Some(best)
}

/// `asks_what_to_wear(understood)`.
pub fn asks_what_to_wear(understood: &Understood) -> bool {
    let mut said = first_text(understood, &["text", "clean"]).to_string();
    if said.is_empty() {
        said = understood.words.join(" ");
    }
    _WEAR_ASK.is_match(&tokens(&said).join(" ")) || a_colour_is_worn(understood)
}

/// The domain of every entry in the knowledge base.
fn every_domain() -> &'static HashSet<String> {
    static D: LazyLock<HashSet<String>> = LazyLock::new(|| all_entry_refs().iter().map(|e| e.domain.clone()).collect());
    &D
}

/// `_topic_domains(understood, entries=None)`: the domains that answer the line's topics, or
/// None when the line has no topic (any domain may answer).
pub fn _topic_domains(understood: &Understood, entries: Option<&[&'static Entry]>) -> Option<HashSet<String>> {
    let topics = reading_topics(understood, true);
    if topics.is_empty() {
        return None;
    }
    let mut allowed: HashSet<String> = crate::knowledge::by_topic()
        .iter()
        .filter(|(_, answered)| answered.is_empty() || topics.iter().any(|t| answered.contains(t)))
        .map(|(name, _)| name.clone())
        .collect();
    if let Some(entries) = entries {
        let all = all_entry_refs();
        if std::ptr::eq(entries.as_ptr(), all.as_ptr()) && entries.len() == all.len() {
            allowed.retain(|d| every_domain().contains(d));
        } else {
            let domains: HashSet<&String> = entries.iter().map(|e| &e.domain).collect();
            allowed.retain(|d| domains.contains(d));
        }
    }
    if !allowed.is_empty() && asks_what_to_wear(understood) {
        allowed.insert("clothing".to_string());
    }
    if allowed.is_empty() {
        None
    } else {
        Some(allowed)
    }
}

/// `_in_topic(item, understood)`.
pub fn _in_topic(item: &Entry, understood: &Understood) -> bool {
    match _topic_domains(understood, None) {
        None => true,
        Some(allowed) => allowed.contains(&item.domain),
    }
}

/// `about_the_topic(item, understood)`.
pub fn about_the_topic(item: &Entry, understood: &Understood) -> bool {
    _in_topic(item, understood)
}

/// `_is_a_person(understood, word)`.
pub fn _is_a_person(understood: &Understood, word: &str) -> bool {
    let word = text::lower(word);
    for ent in &understood.entities {
        if text::lower(&ent.text) == word {
            return ent.kind == "person";
        }
    }
    false
}

/// `_says_what_is_missing(understood)`: a clause other than the ask says what is missing.
pub fn _says_what_is_missing(understood: &Understood) -> bool {
    for clause in &understood.clauses {
        if *clause == understood.focus {
            continue;
        }
        if MISSING_WORDS.is_match(&clause.join(" ")) {
            return true;
        }
    }
    false
}

/// `_reaches(item, forms)`: the line says some part of the key.
pub fn _reaches(item: &Entry, forms: &FormLine) -> bool {
    item.key.iter().any(|part| !_part_matches(part.into(), forms, None).is_empty())
}

/// `names_its_own_subject(item, understood)`: the line says the key's first part.
pub fn names_its_own_subject(item: &Entry, understood: &Understood) -> bool {
    let forms = FormLine::of(&tokens(first_text(understood, &["clean", "text"])));
    !forms.is_empty() && !item.key.is_empty() && !_part_matches((&item.key[0]).into(), &forms, None).is_empty()
}

/// `_wrapped_subject(clean, norm_text=None)`: the subject inside "what should i do about ...".
pub fn _wrapped_subject(clean: &str, norm_text: &str) -> Option<String> {
    let plain = WORD.findall_str(clean).into_iter().filter(|w| !_FILLER.contains(w)).collect::<Vec<_>>().join(" ");
    for line in [clean, norm_text, plain.as_str()] {
        let line = text::split(&line.replace(" | ", " ")).join(" ");
        if line.is_empty() {
            continue;
        }
        for pattern in _WRAPPED.iter() {
            let Some(found) = pattern.match_(&line) else { continue };
            let subject = strip_chars(found.name_str("subject"), " .,!?;:").to_string();
            if _NOT_A_THING.contains(&text::lower(&subject)) {
                return None;
            }
            if !WORD.findall_str(&subject).iter().any(|w| !_NOT_A_SUBJECT.contains(w)) {
                return None;
            }
            return Some(subject);
        }
    }
    None
}

/// `_the_ask_it_wraps(understood, subject, whole, entries, by_domain, every, reject)`.
#[allow(clippy::too_many_arguments)]
pub fn _the_ask_it_wraps(
    understood: &Understood,
    subject: &str,
    whole: Option<&'static Entry>,
    entries: &[&'static Entry],
    by_domain: &HashMap<String, HashMap<String, usize>>,
    every: &HashMap<String, usize>,
    reject: Reject,
) -> Option<&'static Entry> {
    if subject.is_empty() {
        return None;
    }
    let words: Vec<String> = tokens(subject).into_iter().filter(|w| !_SUBJECT_POINTS.contains(w)).collect();
    if words.len() == 1 {
        if let Some(alone) = _the_subject_alone(&words.join(" "), entries, by_domain, every, reject) {
            if whole.is_some_and(|w| ASK_FRAME_IDS.contains(&w.id)) {
                return None;
            }
            return Some(alone);
        }
    }
    if whole.is_some() {
        return None;
    }
    let mut again = understood.clone();
    again.plain = String::new();
    again.norm = vec![];
    again.norm_text = String::new();
    again.clean = subject.to_string();
    again.words = _words_of(subject).iter().map(|w| text::lower(w)).collect();
    if let Some(found) = find(&again, Some(entries), reject, false) {
        return Some(found);
    }
    _the_subject_alone(subject, entries, by_domain, every, reject)
}

/// `names_one_thing(words, every)`: every word is rare enough to name one thing.
pub fn names_one_thing(words: &[String], every: &HashMap<String, usize>) -> bool {
    for word in words {
        let counts: Vec<usize> = _forms(word).iter().map(|f| every.get(f).copied().unwrap_or(0)).collect();
        if counts.is_empty() || *counts.iter().max().unwrap() > *COMMON_WORD {
            return false;
        }
    }
    true
}

/// `_the_subject_alone(subject, entries, by_domain, every, reject=None)`: the subject asked
/// about on its own.
pub fn _the_subject_alone(
    subject: &str,
    entries: &[&'static Entry],
    by_domain: &HashMap<String, HashMap<String, usize>>,
    every: &HashMap<String, usize>,
    reject: Reject,
) -> Option<&'static Entry> {
    let words = tokens(subject);
    if words.is_empty() {
        return None;
    }
    let forms = FormLine::of(&words);
    let typed = lower_all(&words);
    let reading = Understood { clean: subject.to_string(), kind: "question".into(), ..Default::default() };
    let framed = names_one_thing(&words.iter().filter(|w| !_SUBJECT_POINTS.contains(*w)).cloned().collect::<Vec<_>>(), every);
    _best(&reading, &words, subject, &forms, &typed, entries, by_domain, every, reject, framed)
}

/// `says_it_has_gone_off(clean)`.
pub fn says_it_has_gone_off(clean: &str) -> bool {
    let low = format!(" {} ", text::lower(clean));
    GONE_OFF_SIGNS.iter().any(|sign| low.contains(&format!(" {sign} ")) || low.contains(&format!(" {sign}")))
}

/// `said_in_this_line(understood, memory=None)`: what this line itself says, without the words
/// it carried from before.
pub fn said_in_this_line(understood: &Understood, memory: Option<&crate::state::Memory>) -> String {
    if let Some(m) = memory {
        if let Some(typed) = &m.asked_now {
            if !text::strip(typed).is_empty() {
                return text::strip(typed).to_string();
            }
        }
    }
    if let Some(typed) = understood.extra.get("said_now").and_then(|v| v.as_str()) {
        if !text::strip(typed).is_empty() {
            return text::strip(typed).to_string();
        }
    }
    if understood.setup.as_deref().is_some_and(|s| !s.is_empty()) && !understood.focus.is_empty() {
        return text::strip(&understood.focus.join(" ")).to_string();
    }
    let mut line = first_text(understood, &["clean", "text"]).to_string();
    let carried = understood.leans_on.as_ref().map(|l| l.text.clone()).unwrap_or_default();
    if !carried.is_empty() {
        let low = text::lower(&line);
        let carried = text::lower(&carried);
        if low.contains(&carried) {
            line = low.replace(&carried, " ");
        }
    }
    text::strip(&line).to_string()
}
