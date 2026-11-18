//! lookup.py L72-L1700 and the readers the domains share: the small tests on a reading, the
//! guard and reply functions entries name, and the entry builders.

use super::*;
use crate::data::{FnRef, Value};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::state::Memory;
use crate::understanding::{rx, rx_flags};

/// `names_a_thing(phrase)`: the phrase has a word that is not grammar or a figure.
pub fn names_a_thing(phrase: &str) -> bool {
    rx("[^a-z0-9']+")
        .split(&text::lower(phrase))
        .iter()
        .filter(|w| !w.is_empty())
        .any(|w| !GRAMMAR_ONLY.contains(w) && !crate::understanding::pyuni::is_digit(w))
}

/// `_asking_with(typed, at)`: the word at `at` is a modal asking ("can i", "should we").
pub fn _asking_with(typed: &[String], at: usize) -> bool {
    if !ASKING_MODALS.contains(&typed[at]) {
        return false;
    }
    if at + 1 < typed.len() && ASKING_SUBJECTS.contains(&typed[at + 1]) {
        return true;
    }
    at > 0 && ASKING_SUBJECTS.contains(&typed[at - 1])
}

/// `saving_for_a_trip(understood)` (a guard): false when the line saves up for a trip.
pub fn saving_for_a_trip(understood: &Understood) -> bool {
    let line = first_text(understood, &["clean", "text"]);
    if _AFFORD_THE_TRIP.is_match(line) {
        return false;
    }
    !(_SAVING_VERB.is_match(line) && _SAVING_FOR_A_TRIP.is_match(line))
}

/// `the_line(understood)`: clean, norm_text and plain joined, lower-cased.
pub fn the_line(understood: &Understood) -> String {
    text::lower(&[understood.clean.as_str(), understood.norm_text.as_str(), understood.plain.as_str()].join(" "))
}

/// `said_text(understood)`: the reading's words (or norm) joined; "" when both are empty.
pub fn said_text(understood: &Understood) -> String {
    for taken in [&understood.words, &understood.norm] {
        if !taken.is_empty() {
            return taken.join(" ");
        }
    }
    String::new()
}

/// `guard_passes(item, reading)`: an entry without a guard passes; a guard that raises fails.
pub fn guard_passes(item: &Entry, reading: &Understood) -> bool {
    match &item.guard {
        None => true,
        Some(g) => crate::knowledge::registry::guard(g, reading),
    }
}

/// `asks_how_often(understood)` for a line already flattened (Python accepts a str).
pub fn asks_how_often_said(said: &str) -> bool {
    HOW_OFTEN_WORDS.iter().any(|w| said.contains(w.as_str()))
}

/// `asks_how_often(understood)`.
pub fn asks_how_often(understood: &Understood) -> bool {
    asks_how_often_said(&the_line(understood))
}

/// `a_sanding_job(understood)` for a line already flattened.
pub fn a_sanding_job_said(said: &str) -> bool {
    SANDING_JOB.is_match(said)
}

/// `a_sanding_job(understood)` (a guard).
pub fn a_sanding_job(understood: &Understood) -> bool {
    a_sanding_job_said(&the_line(understood))
}

/// `said(*words, leads=SAID_LEADS)`: each word after each lead ("i have a rash", ...).
pub fn said(words: &[&str], leads: Option<&[String]>) -> Vec<String> {
    let leads = leads.unwrap_or(&SAID_LEADS);
    let mut out = Vec::new();
    for word in words {
        for lead in leads {
            out.push(format!("{lead} {word}"));
        }
    }
    out
}

/// `the_appliance_swallows_the_name(understood, names)`: a name said only inside an appliance's
/// name ("dryer" in "tumble dryer").
pub fn the_appliance_swallows_the_name(understood: &Understood, names: &[String]) -> bool {
    // str(understood.get('norm') or ''): the list's repr.
    let norm = if understood.norm.is_empty() {
        String::new()
    } else {
        format!("[{}]", understood.norm.iter().map(|w| crate::understood::py_str_repr(w)).collect::<Vec<_>>().join(", "))
    };
    let parts = [understood.clean.clone(), understood.text.clone(), norm];
    let said = format!(" {} ", text::lower(&parts.join(" ")));
    let names: Vec<String> = names.iter().filter(|n| !text::strip(n).is_empty()).map(|n| text::lower(n)).collect();
    if names.is_empty() {
        return false;
    }
    for appliance in APPLIANCE_NAMES.iter() {
        let pattern = format!("(?<![a-z0-9]){}(?![a-z0-9])", crate::py::re::escape(appliance));
        if !rx(&pattern).is_match(&said) {
            continue;
        }
        if names.iter().any(|name| appliance.contains(name.as_str()) && text::len(name) < text::len(appliance)) {
            return true;
        }
    }
    false
}

/// `the_back_is_a_body_part(understood)` (a guard): "back" is not the back of a house.
pub fn the_back_is_a_body_part(understood: &Understood) -> bool {
    let said = text::lower(&understood.text);
    let above = understood.leans_on.as_ref().map(|l| text::lower(&l.text)).unwrap_or_default();
    let both = format!("{said} {above}");
    if !BACK_OF_THE_HOUSE.is_match(&both) {
        return true;
    }
    THE_BACK_ITSELF.is_match(&both)
}

/// `_avoid_words(parts)`: the avoid words flattened, a modal word put as its phrases.
pub fn _avoid_words(parts: &[Value]) -> Vec<String> {
    let mut flat = Vec::new();
    for part in parts {
        match part {
            Value::Str(s) => match MODAL_PHRASES.get(s.as_ref() as &str) {
                Some(phrases) => flat.extend(phrases.iter().cloned()),
                None => flat.push(s.to_string()),
            },
            Value::List(_) | Value::Tuple(_) | Value::Set(_) => flat.extend(_avoid_words(part.items())),
            _ => {}
        }
    }
    flat
}

/// `about_them(template, male=PEOPLE_MALE, female=PEOPLE_FEMALE)`'s `reply(memory, reading)`:
/// the template with the pronouns of the one the line is about.
pub fn about_them_reply(f: &FnRef, _memory: &mut Memory, reading: &Understood) -> Option<String> {
    let template = f.cell("template").map(|v| v.str().to_string()).unwrap_or_default();
    let male: Vec<String> = f.cell("male").map(Value::strs).unwrap_or_else(|| PEOPLE_MALE.clone());
    let female: Vec<String> = f.cell("female").map(Value::strs).unwrap_or_else(|| PEOPLE_FEMALE.clone());
    Some(about_them(&template, &male, &female, reading))
}

/// The body of `about_them`'s reply, for a template and the words for each sex.
pub fn about_them(template: &str, male: &[String], female: &[String], reading: &Understood) -> String {
    let mut they = "they";
    for word in &reading.words {
        if male.contains(word) {
            they = "he";
            break;
        }
        if female.contains(word) {
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
    let mut chars = out.chars();
    match chars.next() {
        Some(c) => text::upper(&c.to_string()) + chars.as_str(),
        None => out,
    }
}

/// `asks_where(understood)` (a guard): the line asks where.
pub fn asks_where(understood: &Understood) -> bool {
    if understood.asks.iter().any(|a| a == "where") {
        return true;
    }
    let line = format!(" {} ", text::lower(&understood.words.join(" ")));
    WHERE_PHRASES.iter().any(|p| line.contains(p.as_str()))
}

/// `asks_where_a_shop_answers(understood)`: where to buy a thing whose price is compared.
pub fn asks_where_a_shop_answers(understood: &Understood) -> bool {
    if !asks_where(understood) {
        return false;
    }
    let words: Vec<String> = understood.words.iter().map(|w| text::lower(w)).collect();
    if words.iter().any(|w| WHERE_PART_WORDS.contains(w)) {
        return false;
    }
    let line = format!(" {} ", words.join(" "));
    if PRICE_COMPARED.iter().any(|t| line.contains(t.as_str())) {
        return true;
    }
    let r = text::lower(text::strip(understood.reference.it.as_deref().unwrap_or("")));
    if r.is_empty() {
        return false;
    }
    PRICE_COMPARED.iter().any(|t| r == *t || r.ends_with(&format!(" {t}")))
}

/// `_bounded(name)`: the name as a whole word.
pub fn _bounded(name: &str) -> &'static Re {
    rx(&format!("(?<![a-z0-9]){}(?![a-z0-9])", crate::py::re::escape(name)))
}

/// `the_fault(template, when_unnamed, table=THE_FAULT)`'s `reply(memory, reading)`: the
/// template with the fault the line names, or the answer for an unnamed one.
pub fn the_fault_reply(f: &FnRef, _memory: &mut Memory, reading: &Understood) -> Option<String> {
    let template = f.cell("template").map(|v| v.str().to_string()).unwrap_or_default();
    let when_unnamed = f.cell("when_unnamed").map(|v| v.str().to_string()).unwrap_or_default();
    let line = text::lower(first_text(reading, &["text", "clean"]));
    let wanted: Vec<(&Re, String)> = match f.cell("wanted") {
        Some(w) => w.items().iter().map(|p| (p.items()[0].as_re().unwrap(), p.items()[1].str().to_string())).collect(),
        None => the_fault_wanted(&THE_FAULT),
    };
    for (pattern, noun) in wanted {
        if pattern.is_match(&line) {
            return Some(template.replace("{fault}", &noun));
        }
    }
    Some(when_unnamed)
}

/// `the_fault`'s table as patterns, the longest word first.
pub fn the_fault_wanted(table: &[(String, String)]) -> Vec<(&'static Re, String)> {
    let mut sorted: Vec<&(String, String)> = table.iter().collect();
    sorted.sort_by(|a, b| text::len(&b.0).cmp(&text::len(&a.0)));
    sorted.into_iter().map(|(word, noun)| (rx(&format!("\\b{}\\b", crate::py::re::escape(word))), noun.clone())).collect()
}

/// `a_thing_has_stopped(understood)`: a named thing has stopped working.
pub fn a_thing_has_stopped(understood: &Understood) -> bool {
    let words = if !understood.norm.is_empty() { understood.norm.join(" ") } else { understood.words.join(" ") };
    let base = if !words.is_empty() { words } else { first_text(understood, &["clean", "text"]).to_string() };
    let line = format!(" {} ", text::lower(&base)).replace(" | ", " ").replace('|', " ");
    if !STOPPED_WORKING.iter().any(|p| line.contains(&format!(" {p} "))) {
        return false;
    }
    let things: Vec<String> = understood.entities.iter().filter(|e| e.kind == "thing").map(|e| text::lower(&e.text)).collect();
    if things.is_empty() {
        return false;
    }
    let subject = text::lower(understood.subject.as_deref().unwrap_or(""));
    if subject.is_empty() {
        return true;
    }
    things.iter().any(|t| !t.is_empty() && (subject.contains(t.as_str()) || t.contains(subject.as_str())))
}

/// `the_thing_has_gone(understood)`: "gone" said of the thing, not of a state ("gone soft").
pub fn the_thing_has_gone(understood: &Understood) -> bool {
    let said: Vec<String> = understood.words.iter().map(|w| text::lower(w)).collect();
    let line = if said.is_empty() {
        format!(" {} ", text::lower(first_text(understood, &["clean", "text"])))
    } else {
        format!(" {} ", said.join(" "))
    };
    for found in HAS_GONE.finditer(&line) {
        let tail = found.str(1);
        let mut words: Vec<&str> = text::split_ref(tail).into_iter().map(|w| strip_chars(w, "'.,!?")).filter(|w| !w.is_empty()).collect();
        while !words.is_empty() && GONE_INTENSIFIER.contains(words[0]) {
            words.remove(0);
        }
        if words.is_empty() || !GONE_STATE.contains(words[0]) {
            return true;
        }
    }
    false
}

/// `food_has_gone_off(understood)`: the line says the food it names has gone off.
pub fn food_has_gone_off(understood: &Understood) -> bool {
    let line = text::lower(first_text(understood, &["plain", "clean", "text"]));
    let words = tokens(&line);
    if words.is_empty() || !words.iter().any(|w| FOOD_NAMED.contains(w)) {
        return false;
    }
    for phrase in OFF_STATE_ASKS.iter() {
        if line.contains(phrase.as_str()) {
            return true;
        }
    }
    for (pos, word) in words.iter().enumerate() {
        if !FOOD_NAMED.contains(word) {
            continue;
        }
        let mut at = pos + 1;
        while at < words.len() && BEFORE_OFF.contains(&words[at]) {
            at += 1;
        }
        if at >= words.len() {
            continue;
        }
        let said = &words[at];
        if said == "bad" {
            let between = &words[pos + 1..at + 1];
            if !between.iter().any(|w| _BAD_IS_A_STATE.contains(w)) {
                continue;
            }
        }
        if OFF_SAID_OF_FOOD.contains(said) {
            return true;
        }
        if SMELL_VERBS.contains(said) {
            let after = words.get(at + 1);
            if after.is_none() || !A_GOOD_SMELL.contains(after.unwrap()) {
                return true;
            }
        }
    }
    false
}

/// `text or clean`, else the words joined: what the plain guards below read.
fn text_or_words(understood: &Understood) -> String {
    let said = first_text(understood, &["text", "clean"]);
    if !said.is_empty() {
        return said.to_string();
    }
    understood.words.join(" ")
}

/// `a_price_not_a_loan(understood)` (a guard): the line is not borrowing money.
pub fn a_price_not_a_loan(understood: &Understood) -> bool {
    let line = text_or_words(understood);
    if line.is_empty() {
        return true;
    }
    !(_BORROWING_IN_THE_LINE.is_match(&line) || _OFF_A_PERSON.is_match(&line))
}

/// `not_the_heat_of_an_argument(understood)` (a guard).
pub fn not_the_heat_of_an_argument(understood: &Understood) -> bool {
    !_THE_HEAT_OF_AN_ARGUMENT.is_match(&text_or_words(understood))
}

/// `the_ask_is_the_order_of_steps(understood)`.
pub fn the_ask_is_the_order_of_steps(understood: &Understood) -> bool {
    let said = text_or_words(understood);
    !said.is_empty() && _ORDER_OF_STEPS.is_match(&said)
}

/// `not_the_ask_about_the_order(understood)`.
pub fn not_the_ask_about_the_order(understood: &Understood) -> bool {
    !the_ask_is_the_order_of_steps(understood)
}

/// `not_the_flat_you_live_in(understood)` (a guard).
pub fn not_the_flat_you_live_in(understood: &Understood) -> bool {
    !_FLAT_YOU_LIVE_IN.is_match(&text_or_words(understood))
}

/// `not_the_flat_of_something_made(understood)`.
pub fn not_the_flat_of_something_made(understood: &Understood) -> bool {
    let said = text_or_words(understood);
    if !rx_flags("\\bflat\\b", crate::py::re::I).is_match(&said) {
        return true;
    }
    if _A_BAKE_OR_A_PAN.is_match(&said) {
        return false;
    }
    if _CAME_OUT_FLAT.is_match(&said) && !_THE_GROUND_UNDERFOOT.is_match(&said) {
        return false;
    }
    true
}

/// The text before a byte offset, at most `n` characters of it (`text[max(0, at - n):at]`).
pub(crate) fn chars_before(text_: &str, at: usize, n: usize) -> &str {
    let head = &text_[..at];
    let count = head.chars().count();
    if count <= n {
        return head;
    }
    let skip = count - n;
    let start = head.char_indices().nth(skip).map(|(i, _)| i).unwrap_or(at);
    &head[start..]
}

/// `the_years_are_a_things(understood)`: every "N years old" in the line is a thing's age.
pub fn the_years_are_a_things(understood: &Understood) -> bool {
    let line = text::split(&text::lower(&text_or_words(understood))).join(" ");
    if line.is_empty() {
        return false;
    }
    let figures = _YEAR_OLD_FIGURE.finditer(&line);
    if figures.is_empty() {
        return false;
    }
    for found in figures {
        let window = chars_before(&line, found.start(), 60);
        if _PEOPLE_AGED.is_match(window) {
            return false;
        }
        if !(_A_THING_SAID_IS.is_match(window) || _A_THING_JUST_BEFORE.is_match(window) || _A_THING_POINTED_AT.is_match(window)) {
            return false;
        }
    }
    true
}

/// Every string in a key, however deep (the stack walk Python does over `item.key`), in the
/// order the stack pops them.
fn key_strings_popped(item: &Entry) -> Vec<String> {
    // stack = list(item.key); part = stack.pop(); a tuple's members are pushed in order.
    let mut out = Vec::new();
    for part in item.key.iter().rev() {
        for alternative in alternatives(part).iter().rev() {
            out.push(alternative.clone());
        }
    }
    out
}

/// `_the_age_is_a_childs(item)`: the row is about a child's age in years.
pub fn _the_age_is_a_childs(item: &Entry) -> bool {
    static CACHE: LazyLock<RwLock<HashMap<String, bool>>> = LazyLock::new(|| RwLock::new(HashMap::new()));
    if let Some(known) = CACHE.read().unwrap().get(&item.id) {
        return *known;
    }
    let mut words: HashSet<String> = HashSet::new();
    let mut phrases: Vec<String> = Vec::new();
    for part in key_strings_popped(item) {
        let plain = text::lower(&part).replace("'s", "s");
        words.extend(text::split(&plain));
        phrases.push(plain);
    }
    let a_person = words.iter().any(|w| _CHILD_WORDS.contains(w)) || phrases.iter().any(|p| _CHILD_PHRASES.contains(p));
    let answer = a_person && _key_says_years_old(item);
    CACHE.write().unwrap().insert(item.id.clone(), answer);
    answer
}

/// `_key_says_years_old(item)`.
pub fn _key_says_years_old(item: &Entry) -> bool {
    key_strings_popped(item).iter().any(|part| _A_YEARS_OLD.is_match(part))
}

/// `not_keeping_an_eye_on_it(understood)` (a guard).
pub fn not_keeping_an_eye_on_it(understood: &Understood) -> bool {
    !_KEEPING_AN_EYE.is_match(&text_or_words(understood))
}

/// `_the_device_is_the_one_falling_asleep(said, start)`: a device named last before the shape.
pub fn _the_device_is_the_one_falling_asleep(said: &str, start: usize) -> bool {
    let mut device: i64 = -1;
    let mut person: i64 = -1;
    for (index, word) in text::split_ref(&said[..start]).into_iter().enumerate() {
        let bare = text::lower(strip_chars(word, &_SLEEPY_MARKS));
        if A_DEVICE_THAT_SLEEPS.contains(&bare) {
            device = index as i64;
        } else if _A_PERSON_WHO_SLEEPS.contains(&bare) {
            person = index as i64;
        }
    }
    device >= 0 && device >= person
}

/// `the_body_is_short_of_sleep(understood)`.
pub fn the_body_is_short_of_sleep(understood: &Understood) -> bool {
    let said = _the_whole_line(understood);
    if NOT_TIRED_AFTER_ALL.is_match(&said) {
        return false;
    }
    for shape in SHORT_OF_SLEEP.finditer(&said) {
        if !_the_device_is_the_one_falling_asleep(&said, shape.start()) {
            return true;
        }
    }
    false
}

/// `not_a_sleepy_body(understood)` (a guard).
pub fn not_a_sleepy_body(understood: &Understood) -> bool {
    !the_body_is_short_of_sleep(understood)
}

/// `all_of(*guards)`'s `test(understood)`: every guard passes.
pub fn all_of_test(f: &FnRef, understood: &Understood) -> bool {
    let Some(guards) = f.cell("guards") else { return true };
    guards.items().iter().all(|g| match g.as_fn() {
        Some(g) => crate::knowledge::registry::guard(g, understood),
        // `if guard` skips a None; anything else that is not callable raises.
        None => !g.truthy(),
    })
}

/// `_the_whole_line(understood)`: text and clean, else the words, lower-cased.
pub fn _the_whole_line(understood: &Understood) -> String {
    let mut said = format!("{} {}", understood.text, understood.clean);
    if text::strip(&said).is_empty() {
        said = understood.words.join(" ");
    }
    text::lower(&said)
}

/// `entry(id, key, reply, ...)`: an Entry built in code (the knowledge base's own come from
/// the bundle). Its `index` is outside the knowledge base's.
#[allow(clippy::too_many_arguments)]
pub fn entry(id: &str, key: Vec<KeyPart>, reply: Reply, avoid: &[Value], bias: i64, ask: Vec<KeyPart>, guard: Option<FnRef>) -> Entry {
    Entry {
        index: usize::MAX,
        id: id.to_string(),
        key,
        reply,
        avoid: _avoid_words(avoid),
        bias,
        ask,
        slips: vec![],
        domain: String::new(),
        fault: false,
        report: false,
        guard,
        shape: vec![],
        explain: false,
        lead: crate::knowledge::entry::Lead::Text(String::new()),
        outcomes: vec![],
        raw: Value::None,
    }
}

/// `asks_the_kinds(understood)` (a guard): the line asks which kinds of a thing there are.
pub fn asks_the_kinds(understood: &Understood) -> bool {
    let said = first_text(understood, &["clean", "text"]);
    if !said.is_empty() && KINDS_ASKS_ME.is_match(said) {
        return false;
    }
    if !said.is_empty() && !KINDS_NAMED_A_USE.is_match(said) {
        let low = text::lower(said).replace(['[', ']', ',', '\''], " ");
        let words = format!(" {} ", text::split(&low).join(" "));
        let lowered = text::split(&text::lower(said)).join(" ");
        let says_it = understood.intent == "news"
            || (understood.kind != "question" && !KINDS_ASKS_WITH_A_VERB.is_match(&lowered) && KINDS_SAYS_HOW_IT_IS.is_match(&lowered));
        if !says_it && KINDS_WORDS.iter().any(|k| words.contains(k.as_str())) {
            return true;
        }
    }
    understood.wants.as_deref() == Some(KINDS_SHAPE.as_str())
        && !KINDS_NAMED_A_USE.is_match(said)
        && understood.asks.iter().all(|a| a == "what" || a == "yesno")
}

/// `kinds_entries(rows)`: the "what kinds of X are there" answers for one domain, from
/// (the line's subject words, the subject for the row's id, the answer) rows.
pub fn kinds_entries(rows: &[(Vec<String>, String, String)]) -> Vec<Entry> {
    let guard = FnRef { id: "bot.knowledge.lookup:asks_the_kinds:1648".to_string(), cells: vec![], defaults: vec![] };
    let mut out = Vec::new();
    for (subjects, name, answer) in rows {
        out.push(entry(
            &format!("kinds-of-{name}"),
            vec![KeyPart::Any(KINDS_WORDS.clone()), KeyPart::Any(subjects.clone())],
            Reply::Text(answer.clone()),
            &[],
            2,
            vec![],
            Some(guard.clone()),
        ));
        out.push(entry(
            &format!("kinds-of-{name}-told"),
            vec![KeyPart::Any(KINDS_FOLLOW_UP_WORDS.clone()), KeyPart::Any(subjects.clone())],
            Reply::Text(answer.clone()),
            &[],
            2,
            vec![],
            Some(guard.clone()),
        ));
    }
    out
}

/// `about_the_chat(own_line)`: the line asks about this chat itself.
pub fn about_the_chat(own_line: &str) -> bool {
    _ABOUT_THE_CHAT.is_match(&tokens(own_line).join(" "))
}

/// `not_a_count_for_the_box_or_the_dish(understood)` (a guard).
pub fn not_a_count_for_the_box_or_the_dish(understood: &Understood) -> bool {
    let line = format!(" {} ", text::lower(first_text(understood, &["clean", "text"])));
    !(GOES_IN.is_match(&line) || HOW_MANY_DAYS.is_match(&line) || KEEPS_OR_STORES.is_match(&line) || COUNTED_INTO_A_THING.is_match(&line))
}

/// `a_food_is_being_kept(understood)`: the line names a food and a keeping word.
pub fn a_food_is_being_kept(understood: &Understood) -> bool {
    let line = format!(" {} ", text::lower(first_text(understood, &["norm_text", "clean"])));
    if !KEEPING_WORDS.iter().any(|w| line.contains(w.as_str())) {
        return false;
    }
    let said: HashSet<String> = understood.words.iter().map(|w| text::lower(w)).collect();
    for food in KEPT_FOODS.iter() {
        if food.contains(' ') {
            if line.contains(food.as_str()) {
                return true;
            }
        } else if said.contains(food) || line.contains(&format!(" {food} ")) {
            return true;
        }
    }
    false
}

/// `only_a_container(understood, names)`: every name the line says is a container.
pub fn only_a_container(understood: &Understood, names: &[String]) -> bool {
    let line = format!(" {} ", understood.words.iter().map(|w| text::lower(w)).collect::<Vec<_>>().join(" "));
    let found: Vec<&String> = names.iter().filter(|one| line.contains(&format!(" {one} "))).collect();
    !found.is_empty() && found.iter().all(|one| CONTAINERS.contains(one))
}

fn g_saving_for_a_trip(_: &FnRef, u: &Understood) -> bool {
    saving_for_a_trip(u)
}
fn g_a_sanding_job(_: &FnRef, u: &Understood) -> bool {
    a_sanding_job(u)
}
fn g_the_back_is_a_body_part(_: &FnRef, u: &Understood) -> bool {
    the_back_is_a_body_part(u)
}
fn g_asks_where(_: &FnRef, u: &Understood) -> bool {
    asks_where(u)
}
fn g_a_price_not_a_loan(_: &FnRef, u: &Understood) -> bool {
    a_price_not_a_loan(u)
}
fn g_not_the_heat_of_an_argument(_: &FnRef, u: &Understood) -> bool {
    not_the_heat_of_an_argument(u)
}
fn g_not_the_flat_you_live_in(_: &FnRef, u: &Understood) -> bool {
    not_the_flat_you_live_in(u)
}
fn g_not_keeping_an_eye_on_it(_: &FnRef, u: &Understood) -> bool {
    not_keeping_an_eye_on_it(u)
}
fn g_not_a_sleepy_body(_: &FnRef, u: &Understood) -> bool {
    not_a_sleepy_body(u)
}
fn g_asks_the_kinds(_: &FnRef, u: &Understood) -> bool {
    asks_the_kinds(u)
}
fn g_not_a_count_for_the_box_or_the_dish(_: &FnRef, u: &Understood) -> bool {
    not_a_count_for_the_box_or_the_dish(u)
}

/// The guards lookup.py defines that entries name.
pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.lookup:saving_for_a_trip:232", g_saving_for_a_trip),
    ("bot.knowledge.lookup:a_sanding_job:367", g_a_sanding_job),
    ("bot.knowledge.lookup:the_back_is_a_body_part:594", g_the_back_is_a_body_part),
    ("bot.knowledge.lookup:asks_where:707", g_asks_where),
    ("bot.knowledge.lookup:a_price_not_a_loan:1108", g_a_price_not_a_loan),
    ("bot.knowledge.lookup:not_the_heat_of_an_argument:1152", g_not_the_heat_of_an_argument),
    ("bot.knowledge.lookup:not_the_flat_you_live_in:1201", g_not_the_flat_you_live_in),
    ("bot.knowledge.lookup:not_keeping_an_eye_on_it:1404", g_not_keeping_an_eye_on_it),
    ("bot.knowledge.lookup:not_a_sleepy_body:1496", g_not_a_sleepy_body),
    ("bot.knowledge.lookup:all_of.<locals>.test:1513", all_of_test),
    ("bot.knowledge.lookup:asks_the_kinds:1648", g_asks_the_kinds),
    ("bot.knowledge.lookup:not_a_count_for_the_box_or_the_dish:4234", g_not_a_count_for_the_box_or_the_dish),
];

/// The replies lookup.py defines (closures the domains build with `about_them` / `the_fault`).
pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.lookup:about_them.<locals>.reply:653", about_them_reply),
    ("bot.knowledge.lookup:the_fault.<locals>.reply:837", the_fault_reply),
];
