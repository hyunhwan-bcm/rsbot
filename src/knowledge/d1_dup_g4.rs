//! DUP ports (group 4 of workstream D1): private ports of helpers from modules other
//! workstreams own, each marked `DUP(<python module>::<name>)` so the integrator can replace it.
//! Also two small Python-compat helpers the group's modules share (`py_repr_str_list`,
//! `py_int`).

use crate::data::{self, Value};
use crate::knowledge::d1_shared as sh;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------------------------
// Python-compat helpers.

/// `repr(s)` for a Python str: single quotes unless the text holds a single quote and no double
/// quote, with backslash, the quote and control characters escaped as CPython does.
pub fn py_repr_str(s: &str) -> String {
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
            c if (0x80..0xa0).contains(&(c as u32)) => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// `str(list_of_str)`: Python's repr of a list of strings, e.g. `['a', "don't"]`.
pub fn py_repr_str_list<S: AsRef<str>>(items: &[S]) -> String {
    let parts: Vec<String> = items.iter().map(|s| py_repr_str(s.as_ref())).collect();
    format!("[{}]", parts.join(", "))
}

/// `int(s)` for a string of ASCII digits (as the bot's `\d` captures and `isdigit()` words are,
/// in practice); None where Python's `int()` would raise. Values too large for i64 saturate.
pub fn py_int(s: &str) -> Option<i64> {
    let t = text::strip(s);
    let (neg, digits) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = digits.chars().filter(|c| *c != '_').collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let v = digits.parse::<i64>().unwrap_or(i64::MAX);
    Some(if neg { -v } else { v })
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.pets

/// DUP(bot.knowledge.pets::_pet_in_the_talk): whether the talk is about a pet: the line and the
/// last three user turns, newest first, until one names a pet or a person.
pub fn pets_pet_in_the_talk(memory: &Memory, understood: &Understood) -> bool {
    static PET_NAMED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.pets.PET_NAMED"));
    static PERSON_NAMED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.pets.PERSON_NAMED"));
    let mut said: Vec<String> = vec![understood.clean.clone()];
    let start = memory.turns.len().saturating_sub(3);
    said.extend(memory.turns[start..].iter().map(|t| t.user.to_lowercase()));
    for line in said.iter().rev() {
        if PET_NAMED.is_match(line) {
            return true;
        }
        if PERSON_NAMED.is_match(line) {
            return false;
        }
    }
    false
}

/// DUP(bot.knowledge.pets::_pet_words): the animal the line is about, as ("a cat", "cat",
/// "cats"); a dog when nothing says otherwise.
pub fn pets_pet_words(understood: &Understood) -> (&'static str, &'static str, &'static str) {
    static CAT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(cat|cats|kitten|kittens)\b", 0));
    static RABBIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(rabbit|rabbits|bunny|bunnies)\b", 0));
    let mut text: &str = &understood.clean;
    let pointed_at: &str = &understood.animal_in_hand;
    if !pointed_at.is_empty() {
        if pointed_at.starts_with("cat") || pointed_at.starts_with("kitten") {
            return ("a cat", "cat", "cats");
        }
        if pointed_at.starts_with("rabbit") || pointed_at.starts_with("bunny") {
            return ("a rabbit", "rabbit", "rabbits");
        }
        if pointed_at.starts_with("dog") || pointed_at.starts_with("pupp") {
            return ("a dog", "dog", "dogs");
        }
        text = pointed_at;
    }
    if CAT.is_match(text) {
        return ("a cat", "cat", "cats");
    }
    if RABBIT.is_match(text) {
        return ("a rabbit", "rabbit", "rabbits");
    }
    ("a dog", "dog", "dogs")
}

/// DUP(bot.knowledge.pets::_pet_salt_water): ring the vet about a pet that drank salt water;
/// None when no pet is in the talk.
pub fn pets_pet_salt_water(memory: &Memory, understood: &Understood) -> Option<String> {
    if !pets_pet_in_the_talk(memory, understood) {
        return None;
    }
    let (one, _kind, _they) = pets_pet_words(understood);
    Some(format!(
        "Ring the vet now and say how much salt water and when: salty water pulls water out of \
         the body, and {one} that has had a lot will be sick, wobbly or drowsy within a few \
         hours. Take the fresh water away so there is none to drink, offer small sips of plain \
         water once the vet has said so, and do not wait for symptoms before you ring - this \
         one is much easier treated early."
    ))
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.health

/// DUP(bot.knowledge.health::tablets_for_a_pregnancy): a reply's "paracetamol or ibuprofen"
/// made paracetamol alone, with the pregnancy note, when the line says the person is pregnant.
pub fn health_tablets_for_a_pregnancy(reply: &str, understood: &Understood) -> String {
    static PREGNANT_AT_ALL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.health._PREGNANT_AT_ALL"));
    static PREGNANT_DENIED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.health._PREGNANT_DENIED"));
    static TABLETS_SAFE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
        data::get("bot.knowledge.health.TABLETS_SAFE")
            .items()
            .iter()
            .map(|p| (p.items()[0].str().to_string(), p.items()[1].str().to_string()))
            .collect()
    });
    static NOTE: LazyLock<String> = LazyLock::new(|| data::string("bot.knowledge.health.PREGNANT_TABLETS_NOTE"));
    let text = sh::squash_lower(sh::clean_or_text(understood));
    if !PREGNANT_AT_ALL.is_match(&text) || PREGNANT_DENIED.is_match(&text) {
        return reply.to_string();
    }
    let mut said = reply.to_string();
    let mut swapped = false;
    for (from, to) in TABLETS_SAFE.iter() {
        if said.contains(from.as_str()) {
            said = said.replace(from.as_str(), to);
            swapped = true;
        }
    }
    let lowered = said.to_lowercase();
    if ["ibuprofen", "nurofen"].iter().any(|w| lowered.contains(w)) {
        swapped = true;
    }
    if swapped {
        said.push_str(&NOTE);
    }
    said
}

// ---------------------------------------------------------------------------------------------
// bot.compose.repair / bot.knowledge.habits

/// DUP(bot.compose.repair::line_of): the line as the reading holds it (clean, else text, else
/// the words).
pub fn repair_line_of(reading: &Understood) -> String {
    if !reading.clean.is_empty() {
        return reading.clean.clone();
    }
    if !reading.text.is_empty() {
        return reading.text.clone();
    }
    reading.words.join(" ")
}

/// DUP(bot.knowledge.habits::_words_of): the line and whatever the reading carried with it
/// (`leans_on`, `ref`, `topic`), as one lower-case string.
pub fn habits_words_of(understood: &Understood) -> String {
    let mut said: Vec<String> = vec![repair_line_of(understood)];
    // "leans_on": a dict (an earlier reading) in multi-turn Python; None here. LEANS_ON
    // "ref": a dict, its str/int values in insertion order (a bool is an int: "True"/"False").
    let r = &understood.reference;
    for v in [&r.it, &r.that, &r.they, &r.there, &r.he, &r.she, &r.kind] {
        if let Some(s) = v {
            said.push(s.clone());
        }
    }
    if let Some(b) = r.followup {
        said.push(if b { "True".into() } else { "False".into() });
    }
    // "topic": a str, which is neither a dict nor a list: nothing is added.
    let joined = said.join(" ").to_lowercase().replace('\u{2019}', "'");
    text::squash(&joined)
}

/// DUP(bot.compose.repair::deed_of): the deed the person reports doing to somebody close, or
/// None.
pub fn repair_deed_of(clean: &str) -> Option<String> {
    static PET_OR_PROFESSIONAL: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PET_OR_PROFESSIONAL"));
    static PHYSICAL_HARM: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.PHYSICAL_HARM"));
    static GRIEF: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._GRIEF"));
    static A_BABY: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._A_BABY"));
    static REGRET_FIRST: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair._REGRET_FIRST"));
    static NOT_A_DEED: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.NOT_A_DEED"));
    static NOBODY_TO_REPAIR: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.repair.NOBODY_TO_REPAIR"));
    fn table(name: &str) -> Vec<(String, &'static Re)> {
        data::get(name)
            .items()
            .iter()
            .map(|p| (p.items()[0].str().to_string(), p.items()[1].as_re().expect("deed pattern")))
            .collect()
    }
    static LAPSES: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| table("bot.compose.repair.LAPSES"));
    static DEEDS: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| table("bot.compose.repair.DEEDS"));
    let text = text::squash(&clean.to_lowercase().replace('\u{2019}', "'"));
    if text.is_empty() {
        return None;
    }
    if PET_OR_PROFESSIONAL.is_match(&text) || PHYSICAL_HARM.is_match(&text) {
        return None;
    }
    if GRIEF.is_match(&text) || A_BABY.is_match(&text) || REGRET_FIRST.is_match(&text) {
        return None;
    }
    for (deed, pattern) in LAPSES.iter() {
        if pattern.is_match(&text) {
            return Some(deed.clone());
        }
    }
    if NOT_A_DEED.is_match(&text) || NOBODY_TO_REPAIR.is_match(&text) {
        return None;
    }
    for (deed, pattern) in DEEDS.iter() {
        if pattern.is_match(&text) {
            return Some(deed.clone());
        }
    }
    None
}

/// DUP(bot.compose.repair::owns_the_deed): the line reports something the person did to
/// somebody close (read from the reading's clean line, else its text).
pub fn repair_owns_the_deed(reading: &Understood) -> bool {
    repair_deed_of(sh::clean_or_text(reading)).is_some()
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.lookup

/// DUP(bot.knowledge.lookup::names_a_thing): the phrase has a word that is not grammar or a
/// number.
pub fn lookup_names_a_thing(phrase: &str) -> bool {
    static GRAMMAR_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.knowledge.lookup.GRAMMAR_ONLY"));
    static SPLIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9']+", 0));
    SPLIT
        .split(&phrase.to_lowercase())
        .iter()
        .filter(|w| !w.is_empty())
        .any(|w| !GRAMMAR_ONLY.contains(w.as_str()) && !text::is_digit(w))
}

/// DUP(bot.knowledge.lookup::a_food_is_being_kept): the line asks how a named food keeps.
pub fn lookup_a_food_is_being_kept(understood: &Understood) -> bool {
    static KEEPING_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEEPING_WORDS"));
    static KEPT_FOODS: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.lookup.KEPT_FOODS"));
    let text = sh::padded(&sh::or(&understood.norm_text, &understood.clean).to_lowercase());
    if !sh::any_in(&text, &KEEPING_WORDS) {
        return false;
    }
    let said: HashSet<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    for food in KEPT_FOODS.iter() {
        if food.contains(' ') {
            if text.contains(food.as_str()) {
                return true;
            }
        } else if said.contains(food) || text.contains(&format!(" {food} ")) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// bot.knowledge.wording

/// DUP(bot.knowledge.wording::the_line_said): the line squashed and lower-cased with a space
/// at each end, or "" for an empty line.
pub fn wording_the_line_said(understood: &Understood) -> String {
    let said = text::squash(sh::clean_or_text(understood)).to_lowercase();
    if said.is_empty() { String::new() } else { format!(" {said} ") }
}

/// DUP(bot.knowledge.wording::_names_one_of): a phrase (substring) or a word (token) of the
/// list is in the line.
pub fn wording_names_one_of<S: AsRef<str>>(said: &str, phrases: &[S]) -> bool {
    static TOKEN: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));
    let tokens: HashSet<String> = TOKEN.findall_str(said).into_iter().collect();
    phrases.iter().any(|p| {
        let p = p.as_ref();
        if p.contains(' ') { said.contains(p) } else { tokens.contains(p) }
    })
}

/// DUP(bot.knowledge.wording::screens_are_a_childs): the screens in the line are a child's.
pub fn wording_screens_are_a_childs(understood: &Understood) -> bool {
    static ONLY_SAID_ABOUT_A_CHILD: LazyLock<Vec<String>> =
        LazyLock::new(|| data::strs("bot.knowledge.wording._ONLY_SAID_ABOUT_A_CHILD"));
    static A_CHILD_NAMED: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.wording._A_CHILD_NAMED"));
    let said = wording_the_line_said(understood);
    if text::strip(&said).is_empty() {
        return true;
    }
    if sh::any_in(&said, &ONLY_SAID_ABOUT_A_CHILD) {
        return true;
    }
    wording_names_one_of(&said, &A_CHILD_NAMED)
}

/// DUP(bot.knowledge.wording::screens_are_somebody_elses): the screen time in the line is
/// another adult's at home, not the person's own or a child's.
pub fn wording_screens_are_somebody_elses(understood: &Understood) -> bool {
    static MY_OWN_SCREENS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.wording._MY_OWN_SCREENS"));
    static THEIR_SCREENS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.knowledge.wording._THEIR_SCREENS"));
    static SOMEBODY_AT_HOME: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.wording._SOMEBODY_AT_HOME"));
    let said = wording_the_line_said(understood);
    if text::strip(&said).is_empty() {
        return false;
    }
    if MY_OWN_SCREENS.is_match(&said) {
        return false;
    }
    if wording_screens_are_a_childs(understood) {
        return false;
    }
    if THEIR_SCREENS.is_match(&said) {
        return true;
    }
    SOMEBODY_AT_HOME.iter().any(|p| said.contains(&format!("{p} ")))
}

// ---------------------------------------------------------------------------------------------
// bot.memory.thread

/// DUP(bot.memory.thread::user_lines): the user's earlier lines, oldest first. Python pairs
/// each with `understand(text, {})`; the one caller here (science `_night_waking`) reads only
/// the text, so the readings are not made.
pub fn thread_user_lines(memory: &Memory) -> Vec<String> {
    memory.turns.iter().filter(|t| !text::strip(&t.user).is_empty()).map(|t| t.user.clone()).collect()
}

/// `memory.get(key)` for a key a module keeps in `extra`, as Python truthiness.
pub fn memory_truthy(memory: &Memory, key: &str) -> bool {
    memory.extra.get(key).is_some_and(Value::truthy)
}
