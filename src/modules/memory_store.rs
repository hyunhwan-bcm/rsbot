//! Port of `bot/modules/memory_store.py`: the memory store at work. It observes every line and
//! keeps the shared keys in `bot/memory/`; it only proposes a reply when the user asks about the
//! conversation itself, so it passes on every ordinary question.
//!
//! Module-level data (regexes, word sets, pools, including every name the Python module imports
//! from `store_lines`, `relations`, `memory_tables` and `store_replies`) is read from the exported
//! bundle under `bot.modules.memory_store.<NAME>`, never retyped.
//!
//! Calls into files other workstreams own go through the small wrappers in the `ext` section
//! below, so a contract that changes is adapted in one place.
#![allow(unused_variables, unused_imports, non_snake_case, non_upper_case_globals, dead_code, unused_macros, unused_mut, clippy::all)]

use super::mod4::*;
use crate::data::{self, Value};
use crate::knowledge::entry::Entry;
use crate::memory::store_replies as sr;
use crate::memory::{facts as extract, relations};
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::runtime::Candidate;
use crate::state::{Memory, Turn};
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

const M: &str = "bot.modules.memory_store.";

fn g(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}

/// A compiled module-level regex, by Python name.
macro_rules! rx {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<&'static Re> = LazyLock::new(|| g(stringify!($name)).as_re().expect(stringify!($name)));
    )*};
}
/// A module-level collection of strings, for membership tests.
macro_rules! strset {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<HashSet<String>> = LazyLock::new(|| g(stringify!($name)).str_set());
    )*};
}
/// A module-level tuple/list of strings, in order.
macro_rules! strs {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<Vec<String>> = LazyLock::new(|| g(stringify!($name)).strs());
    )*};
}
/// Any module-level value, as exported.
macro_rules! val {
    ($($name:ident),* $(,)?) => {$(
        static $name: LazyLock<&'static Value> = LazyLock::new(|| g(stringify!($name)));
    )*};
}
/// A regex written inline in a Python function.
macro_rules! inline_re {
    ($name:ident, $pat:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pat, $flags));
    };
}

// ---------------------------------------------------------------------------------------------
// ext: the calls into files other workstreams own, each through one wrapper. (The Python
// behaviour helpers and `pick` are the shared ones in `modules::mod4`.)

/// `bot.memory.remember(memory, key, value, text)`.
fn remember(memory: &mut Memory, key: &str, value: &Value, text_: &str) {
    crate::memory::remember(memory, key, value, text_)
}

/// `bot.memory.remember_other(memory, entry, text)`.
fn remember_other(memory: &mut Memory, entry: &Value, text_: &str) {
    crate::memory::remember_other(memory, entry, text_)
}

/// `bot.memory.forget(memory, key, values, why)`: the values that actually went, as a list.
///
/// CONTRACT NOTE: the skeleton contract is `forget(.., values: &[String], why: &str) ->
/// Vec<String>`, which cannot hold the pet records Python passes and returns; this wrapper is the
/// one place to adapt when the owner of `bot/memory/__init__.py` lands the real signature.
fn forget(memory: &mut Memory, key: &str, values: &Value, why: Option<&str>) -> Value {
    crate::memory::forget(memory, key, values, why.unwrap_or("")).val()
}

/// `bot.memory.stem(word)`.
fn stem(word: &str) -> String {
    crate::memory::topics::stem(word)
}

/// `_person_from_pronoun(text, memory, named_here=())`.
fn _person_from_pronoun(text_: &str, memory: &mut Memory, named_here: &[String]) -> String {
    ret_str(relations::_person_from_pronoun(text_, memory, named_here))
}

/// `_folded(text, memory)`.
fn _folded(text_: &str, memory: &Memory) -> String {
    ret_str(relations::_folded(text_, memory))
}

/// `_animal_family(animal)` on whatever a record holds (`str(animal or "")` is the reader's own).
fn _animal_family(animal: &Value) -> String {
    ret_str(relations::_animal_family(&s_or(Some(animal))))
}

/// `_bare(text)`.
fn _bare(folded: &str) -> String {
    ret_str(sr::_bare(folded))
}

// ---------------------------------------------------------------------------------------------
// The store's own claims on a line.

rx!(NOT_A_FACT, PLAN_MINE, ACTIVITY_ASK);

/// Python: bot/modules/memory_store.py:396 `_a_kept_fact_read(text, memory)`
///
/// True when the line asks back a fact they gave, in whatever words, and the store has it. The
/// key is read by `recall.ask`, which knows the shapes and not the store, so the fact has to be
/// one the store really holds: with nothing kept the line is left to the entries.
pub fn _a_kept_fact_read(text_: &str, memory: &mut Memory) -> bool {
    if !user(memory).truthy() {
        return false;
    }
    let said = match py_try(|| {
        if !ret_true(crate::memory::recall::ask(text_, false)) {
            return None;
        }
        // `respond(text, copy.deepcopy(memory))`
        let mut copy = memory.clone();
        Some(respond(text_, &mut copy))
    }) {
        Ok(Some(said)) => said,
        // a hook that raises must not lose the turn
        _ => return false,
    };
    match said {
        Some(said) if !text::strip(&said).is_empty() => NOT_A_FACT.search(&said).is_none(),
        _ => false,
    }
}

/// Python: bot/modules/memory_store.py:422 `_their_own_line_ask(text, memory)`
///
/// True on the one ask the store must win even against a topic entry: their own line back. The
/// store answers only from a line they really said, so this is said for exactly the asks whose
/// answer is that line.
pub fn _their_own_line_ask(text_: &str, memory: &mut Memory) -> bool {
    py_try(|| {
        let low = _bare(text_);
        if PLAN_MINE.is_match(&low) {
            let known = user(memory);
            return has(known, "plans") || has(known, "plan");
        }
        if !ACTIVITY_ASK.is_match(&low) {
            return false;
        }
        ret_true(sr::_activity_line(memory))
    })
    .unwrap_or(false)
}

strset!(NOT_FROM_THE_REPAIR);

/// Python: bot/modules/memory_store.py:468 `_with_repaired(found, text, memory, kind)`
///
/// The facts the understanding engine's own repairs make plain. The repaired line is read as
/// well, and a fact is taken from it only where the typed line gave nothing for that key - so
/// this can add what a typo hid, and cannot overwrite what they actually said.
pub fn _with_repaired(mut found: Value, text_: &str, memory: &Memory, kind: Option<&str>) -> Value {
    let Some(understood) = memory.understood.as_ref() else { return found };
    let plain = understood.plain.as_str();
    if text::strip(plain).is_empty() {
        return found;
    }
    if plain == text::squash(text_) {
        return found; // nothing was repaired, so there is nothing new to read
    }
    let more = extract::extract(plain, kind == Some("question"));
    for (key, value) in more.pairs() {
        let key = key.str();
        if !NOT_FROM_THE_REPAIR.contains(key) && !has(&found, key) && value.truthy() {
            found.set_key(key, value.clone());
        }
    }
    // A list of facts about other people is not one fact. The typed line can name the person and
    // one thing about them while the repair is what makes the other plain: "my mums 70 n she
    // loves gardning" reads the hobby as typed, and the age only once it is repaired. Taking the
    // repaired list only when the typed one was empty threw the age away because the hobby was
    // there, so the person is merged key by key instead.
    if has(&found, "others") && has(&more, "others") {
        let mut held: Vec<Value> = vget(&found, "others").items().to_vec();
        for entry in vget(&more, "others").items() {
            if !entry.is_dict() || !has(entry, "who") {
                continue;
            }
            let who = vget(entry, "who");
            match held.iter().position(|one| one.is_dict() && vget(one, "who") == who) {
                None => held.push(entry.clone()),
                Some(at) => {
                    for (key, value) in entry.pairs() {
                        if key.str() != "who" && !has(&held[at], key.str()) {
                            held[at].set_key(key.clone(), value.clone());
                        }
                    }
                }
            }
        }
        found.set_key("others", list_of(held));
    }
    found
}

rx!(NOT_A_NAME_BEFORE, THE_PLACE_IS_NAMED);

/// Python: bot/modules/memory_store.py:523 `_the_places_are_written_as_names(reply, memory)`
///
/// A place the store holds is written the way the store holds it. Only a place the store is
/// holding is touched, and a determiner in front of the word says it is a common noun, not a
/// place: "You're planning a bath and an early night" is not the city of Bath.
pub fn _the_places_are_written_as_names(reply: &str, memory: &mut Memory) -> String {
    let known = user(memory).clone();
    if text::strip(reply).is_empty() {
        return reply.to_string();
    }
    let mut names: Vec<String> = vget(&known, "places")
        .items()
        .iter()
        .filter_map(|n| n.as_str())
        .filter(|n| !text::strip(n).is_empty())
        .map(str::to_string)
        .collect();
    if let Some(city) = vget(&known, "city").as_str() {
        if !text::strip(city).is_empty() && !names.iter().any(|n| n == city) {
            names.push(city.to_string());
        }
    }
    if names.is_empty() {
        return reply.to_string();
    }
    let mut out = reply.to_string();
    for name in &names {
        let pattern = Re::new(&format!(r"(?<![\w']){}(?![\w'])", pyre::escape(name)), pyre::I);
        let whole = out.clone();
        out = pattern.sub_fn(&whole, |m| {
            // `match.string[max(0, match.start() - 28):match.start()]`, in characters
            let upto = &whole[..m.start()];
            let chars = upto.chars().count();
            let before = &upto[text::byte_at(upto, chars.saturating_sub(28))..];
            if NOT_A_NAME_BEFORE.is_match(before) {
                return m.as_str().to_string();
            }
            if !THE_PLACE_IS_NAMED.is_match(before) {
                return m.as_str().to_string();
            }
            name.clone()
        });
    }
    out
}

/// Python: bot/modules/memory_store.py:558 `note_the_hours(text, memory)`
///
/// Keep the kind of week the line names, for the follow-up on the next turn. The reading is
/// bot/memory/ack.py's (`_hours_family`), so the six shapes are written down in one place.
pub fn note_the_hours(text_: &str, memory: &mut Memory) {
    if text::strip(text_).is_empty() {
        return;
    }
    // The week has to be theirs. "my son wakes at five" is his morning, and "i work nights, my
    // back is killing me" is a line about the back: the understanding engine reads that one as
    // health and the son's morning as family, while the lines this is for read work, or read no
    // topic at all ("i start at six so im up at five").
    if let Some(understood) = memory.understood.as_ref() {
        if let Some(topic) = understood.topic.as_deref() {
            if !topic.is_empty() && topic != "work" {
                return;
            }
        }
    }
    // a reading that cannot run must not cost the store the turn
    let Ok(read) = py_try(|| crate::memory::ack::_hours_family(&text_.to_lowercase()).val()) else { return };
    let family = read.items().first().cloned().unwrap_or(Value::None);
    if family.truthy() {
        crate::memory::hours::note(memory, &family);
    }
}

/// Python: bot/modules/memory_store.py:587 `_a_detail_does_not_replace_the_job(found, memory)`
///
/// A line that only says when or where they work keeps the job they already gave. A shift is a
/// detail of the work, and a detail does not take the place of the fact; a place is a detail
/// beside the role, and the summary says both.
pub fn _a_detail_does_not_replace_the_job(mut found: Value, memory: &mut Memory) -> Value {
    if !has_key(&found, "job") {
        return found;
    }
    let held = vget(user(memory), "job").clone();
    let held_frame = sget(user(memory), "job_frame");
    let said = vget(&found, "job").clone();
    if !held.truthy() || !said.truthy() || py_str(&held) == py_str(&said) {
        return found;
    }
    let said_frame = sget(&found, "job_frame");
    let said_weight = extract::work_weight(&said, &Value::from(said_frame.as_str()));
    if said_weight < extract::work_weight(&held, &Value::from(held_frame.as_str())) {
        let mut kept = found.clone();
        kept.del_key("job");
        // The frame belongs to the job it was said with: keeping "work" beside the nurse they
        // already told us would say "you work nurse".
        kept.del_key("job_frame");
        // It is still something they told us, so it is kept as what it is - when or where they
        // work, beside the job - rather than thrown away: memory then says both ("you're a
        // nurse; you work nights", "you're a teacher; you work in a school") instead of one of
        // the two going missing.
        if said_weight == 2 {
            kept.set_key("workplace", text::strip(&format!("{} {}", said_frame, py_str(&said))).to_string());
        } else {
            kept.set_key("shift", said);
        }
        return kept;
    }
    if extract::is_a_place_of_work(&Value::from(held_frame.as_str())) && vget(&found, "job_frame").is_none() {
        // The role is said second: "i work in a school" then "im a teacher". The school stays
        // beside it as where they work, rather than being overwritten by the role.
        if !has(user(memory), "workplace") {
            found.set_key("workplace", text::strip(&format!("{} {}", held_frame, py_str(&held))).to_string());
        }
    }
    if !has_key(&found, "job_frame") {
        // A role said on its own is "you're a nurse", never "you work nurse": the frame belongs to
        // the line it was said with ("i work nights"), and left in the store it carries over.
        user(memory).del_key("job_frame");
    }
    found
}

strset!(ASK_OPENERS);

/// Python: bot/modules/memory_store.py:641 `_without_the_shrug_at_the_end(text)`
///
/// The line without the chat-speak hung on the end of it, when that leaves words behind.
pub fn _without_the_shrug_at_the_end(text_: &str) -> String {
    let said = text::squash(text_);
    let lowered = said.to_lowercase();
    let words = text::split(text::strip_chars(&lowered, " .,!;:"));
    if !said.contains('?') && !words.first().map_or(false, |w| ASK_OPENERS.contains(w)) {
        // The shrug is dropped from an ask, and from an ask only. "i meant 43 actually" is a
        // correction, and the word on the end of it is part of how it reads: with the shrug gone
        // the correction went to the greeting and the line "43 - got it, 43 it is" was lost.
        return said;
    }
    let clean = crate::memory::filler::without_the_end_shrug(&said);
    if !clean.is_empty() && clean != said {
        clean
    } else {
        said
    }
}

/// The (animal, name) pairs as the set value `memory["_pets_before"]` holds.
fn pets_set(pairs: &[(String, String)]) -> Value {
    Value::Set(Arc::new(
        pairs
            .iter()
            .map(|(a, n)| Value::Tuple(Arc::new(vec![Value::from(a.as_str()), Value::from(n.as_str())])))
            .collect(),
    ))
}

/// Python: bot/modules/memory_store.py:661 `observe(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// Remember what the line says, and note a second animal of a kind as the turn's news. The
/// wording is not read: the store's own record covers every phrasing.
pub fn observe(text_: &str, memory: &mut Memory) {
    let before = _the_pets_on_record(memory);
    // The readers below file pets, and the line that names an animal files one even when it is
    // putting a name right ("no the cat is called max" in a house of two cats filed a third).
    // The set the turn before held is carried so that entry can be told from a pet of its own.
    memory.set("_pets_before", pets_set(&before));
    let hurt_before = crate::memory::whathurts::before(memory).val();
    let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| _observe(text_, memory)));
    memory.remove("_pets_before");
    if let Err(raised) = ran {
        std::panic::resume_unwind(raised);
    }
    crate::memory::whathurts::after(memory, &ret_str(hurt_before));
    _note_a_pet_that_is_new(text_, memory, &before);
    // the store never crashes a turn
    let _ = py_try(|| {
        sr::_the_limit_they_share(text_, memory);
    });
}

/// Python: bot/modules/memory_store.py:690 `after(text, reply, memory)` - a runtime hook; its signature is the runtime's.
///
/// Keep a correction the chat was given, whichever module gave the reply to it. The correction
/// is the chat's, not the reply's, so it happens here, once, after the reply has been chosen.
pub fn after(text_: &str, reply: &str, memory: &mut Memory) {
    // the store never crashes a turn
    let _ = py_try(|| {
        crate::memory::repair::keep_the_correction(text_, memory);
    });
}

/// `AGES_IN_WORDS`: the age a figure is, in words ({7: "seven"}).
static AGES_IN_WORDS: LazyLock<HashMap<i64, String>> = LazyLock::new(|| {
    g("AGES_IN_WORDS").pairs().iter().filter_map(|(k, v)| Some((int_of(k)?, v.str().to_string()))).collect()
});

/// Python: bot/modules/memory_store.py:711 `_the_ages_of_the_children(text, memory)`
///
/// The reply for the ages of the children the store counts, given one at a time, or "". A bare
/// list of figures is read only when the line before counted the children, because "7 and 4" on
/// its own could be anything.
pub fn _the_ages_of_the_children(text_: &str, memory: &mut Memory) -> String {
    let held = match int_of(vget(user(memory), "kids")) {
        Some(n) if n >= 2 => n,
        _ => return String::new(),
    };
    // Ages already on the group are a correction, and `matters.a_corrected_age` reads those: a
    // second full list here would overwrite the ages one at a time.
    if others(memory).items().iter().any(|entry| entry.is_dict() && vget(entry, "ages").truthy() && vget(entry, "ages").len() > 1) {
        return String::new();
    }
    let ages = extract::ages_said_one_by_one_n(text_, held);
    if ages.is_empty() {
        return String::new();
    }
    if !(extract::the_ones_are_given_one_at_a_time(text_) || _the_count_was_the_line_before(memory)) {
        return String::new();
    }
    let entry = dict_of(vec![("who", Value::from("kids")), ("ages", list_of(ages.iter().map(|a| Value::Int(*a)).collect()))]);
    remember_other(memory, &entry, text_);
    // Said in the order the person gave them: "ones 7 and ones 4" is seven and four, and the store
    // keeps that order everywhere else.
    let words: Vec<String> = ages.iter().map(|age| AGES_IN_WORDS.get(age).cloned().unwrap_or_else(|| age.to_string())).collect();
    let said = if words.len() > 2 {
        format!("{} and {}", words[..words.len() - 1].join(", "), words[words.len() - 1])
    } else {
        words.join(" and ")
    };
    pick(&[format!("{} — got it.", cap1(&said)), format!("Got it — {}.", said), format!("{}.", cap1(&said))], memory, "")
}

inline_re!(KIDS_COUNTED, r"\b(?:kids?|children|child)\b", pyre::I);

/// Python: bot/modules/memory_store.py:745 `_the_count_was_the_line_before(memory)`
///
/// Whether the line before this one counted the children, so a bare list of ages is theirs.
pub fn _the_count_was_the_line_before(memory: &Memory) -> bool {
    let said = sget(mget(memory, "topic"), "text");
    KIDS_COUNTED.is_match(&said)
}

/// Python: bot/modules/memory_store.py:751 `_the_pets_on_record(memory)`
///
/// The (animal, name) pairs the store holds, for comparing one turn with the next (a set: each
/// pair once).
pub fn _the_pets_on_record(memory: &mut Memory) -> Vec<(String, String)> {
    let mut held: Vec<(String, String)> = Vec::new();
    for pet in vget(user(memory), "pets").items() {
        if !pet.is_dict() {
            continue;
        }
        let pair = (text::strip(&sget(pet, "animal")).to_lowercase(), text::strip(&sget(pet, "name")).to_lowercase());
        if !held.contains(&pair) {
            held.push(pair);
        }
    }
    held
}

/// Python: bot/modules/memory_store.py:764 `_note_a_pet_that_is_new(text, memory, before)`
///
/// A line that leaves two of an animal on record is answered as a line about the two. Nothing is
/// read out of the words: the store is compared with itself. A first pet is left alone and so is
/// a rename, which leaves one animal behind.
pub fn _note_a_pet_that_is_new(text_: &str, memory: &mut Memory, before: &[(String, String)]) -> Option<Value> {
    let held = mget(memory, "correction");
    if held.is_dict() && vget(held, "said").as_str() == Some(fold(text_).as_str()) {
        return None; // the line has already been read as a correction, and that is its answer
    }
    let mut added: Vec<(String, String)> = _the_pets_on_record(memory).into_iter().filter(|pair| !before.contains(pair)).collect();
    added.sort();
    let pets: Vec<Value> = vget(user(memory), "pets").items().iter().filter(|pet| pet.is_dict()).cloned().collect();
    for (animal, name) in added {
        if animal.is_empty() || name.is_empty() {
            continue;
        }
        let same = pets.iter().filter(|pet| text::strip(&sget(pet, "animal")).to_lowercase() == animal && has(pet, "name")).count();
        if same < 2 {
            continue;
        }
        let correction = dict_of(vec![
            ("key", Value::from("pets_more")),
            ("value", Value::from(name)),
            ("animal", Value::from(animal)),
            ("said", Value::from(fold(text_))),
            ("text", Value::from(text_)),
        ]);
        memory.set("correction", correction.clone());
        return Some(correction);
    }
    None
}

rx!(REMIND_QUESTION, CORRECTING);
inline_re!(A_MONEY_SIGN, "[\u{a3}$\u{20ac}]", 0);

/// Python: bot/modules/memory_store.py:805 `_observe(text, memory)`
pub fn _observe(text_: &str, memory: &mut Memory) {
    let mut text_: String = text_.to_string();
    if ret_true(crate::memory::not_this::the_place_ruled_out(&text_, memory)) {
        // A place they ruled out is not a place they are going: "im going to spain" / "we arent
        // going to portugal" filed Portugal as a second place and answered with its card. The
        // line states nothing to keep, so nothing below it is read.
        return;
    }
    // The kind of week their hours make, kept for the turn after it: "does that get easier",
    // "i never know when to stop" are about the week just described, and the reply is written in
    // bot/memory/hours.py. It is noted here rather than where the reply is written because the
    // hours line is often answered by a knowledge entry ("my rota changes every week") that never
    // reaches bot/memory/ack.py, and the follow-up is the same follow-up either way.
    note_the_hours(&text_, memory);
    // How many of a thing they have, kept as the number: "i have three brothers" was filed as one
    // brother, so "how many brothers do i have" said the store did not know. Noted here, beside the
    // hours, because the line that says it is often answered by another module.
    crate::memory::counts::observe(&text_, memory);
    // A family name is a second name and not the one they are called: "my last name is okafor" was
    // filed as nothing, and "whats my surname" a turn later said the chat had never said one. It is
    // kept here, beside the count, because a line that gives one is often answered by another
    // module, and the file has to be there whichever module takes the turn.
    crate::memory::surname::observe(&text_, memory);
    // The paragraph a plan announcement implies is kept for the next line rather than given now:
    // "im getting married in june" wants a warm word and a question, which the composer has, while
    // "what do i do first" wants the plan's own first step. It is kept here, in observe, because
    // each module works on its own copy of memory and only the chosen module's copy is kept: a
    // plan recorded by whichever module happened to answer is lost whenever the composer has the
    // turn (sturdy-poppy, #1848).
    let planned = crate::memory::plans::answer(&text_, memory).val();
    if planned.truthy() {
        memory.set("plan_answer", dict_of(vec![("text", planned), ("user", Value::from(text_.as_str()))]));
    }
    let kind: Option<String> = memory.understood.as_ref().map(|u| u.kind.clone());
    let question = kind.as_deref() == Some("question");
    // A remembering frame can carry the telling in the same line: "can you remember my sisters
    // name is amy" asks about the store and gives it a name. The frame states nothing - what
    // follows it is a line the store already knows how to read - so it is read as if it had been
    // said on its own. Left whole, the line filed the sister and not her name, and the reply to it
    // told a person who had just given the name that the store had nothing on her.
    if let Some(said) = ret_some(sr::_behind_a_remembering_frame(&text_)) {
        text_ = said;
    }
    if ret_true(crate::memory::recall::ask(&text_, false)) || _reads_the_store(&text_, memory) {
        return; // a question about what is already known states nothing new to keep
    }
    if REMIND_QUESTION.is_match(&text_) {
        return; // so does asking to be reminded, whatever the ask names
    }
    if _asks_about_someone_else(&text_) {
        // A yes/no question about someone else asks what is on record, so the frame itself
        // states nothing: "is he allergic to peanuts" must not file peanuts. A line that asks
        // and then tells ("does my mum eat meat? she is vegetarian") is two lines in one, and
        // the telling is kept: the frame is cut off and the rest is read as the statement it
        // is, or nothing is filed at all when the frame is the whole line.
        let rest = ret_str(sr::_after_the_yes_no_frame(&text_));
        if rest.is_empty() {
            return;
        }
        text_ = rest;
    }
    if ret_true(sr::_statement(&text_)) {
        // "do i live in york?" / "am i a baker?" / "am i right?" ask about the person; they state
        // nothing, so nothing is filed from them. Without this the question's own hypothetical
        // becomes the fact ("i live in leeds" then "do i live in york?" left York in the store)
        // and "am i right?" was filed as a name.
        return;
    }
    let text_: &str = &text_;
    // A chat that weighs two things and picks one keeps the choice: "im thinking of getting a
    // cat" then "actually a dog" is a decision, and "what did i decide to get" is a question
    // about it. Read here because the line is a statement the store's own readers pass over.
    crate::memory::decided::observe(text_, memory);
    // "im travelling with my mum": who is going with them is a fact about the chat, and
    // "who is coming with me" three turns later is answered from it (bot/memory/company.py).
    crate::memory::company::observe(text_, memory);
    memory.remove("pet_age"); // an age taken last turn is not news for this one
    crate::memory::times::keep(text_, memory);
    // "i go to the gym twice a week" is a rate of their own doing, filed so a late "how often
    // was that again" is answered from it rather than asked back (bot/memory/rates.py).
    crate::memory::rates::keep(text_, memory);
    let mut found = extract::extract(text_, question);
    if !found.truthy() {
        // "im an accountant" then "actually an engineer rather than an accountant": the person
        // says the correction on its own, and the subject is the turn before's.
        found = extract::a_correction_without_a_subject(text_, &last_user(memory));
    }
    if !has(&found, "name") {
        // "can you remember my name" then "sam": the answer to the bot's own question, which no
        // pattern of the line's own words can read.
        if let Some(answered) = _their_name_answered(text_, memory) {
            found.set_key("name", answered);
        }
    }
    found = _with_repaired(found, text_, memory, kind.as_deref());
    if !has(&found, "kids") {
        // "actually there are three of them": the number said back about the people in hand, with
        // no word for the people in the line at all. `kid_count` reads the count in front of the
        // word ("my three kids") and nothing here, because which people it counts is the store's
        // question: the children, when they are what the turn before was about.
        let counted = extract::count_of_the_people(text_, memory).val();
        if counted.truthy() {
            found.set_key("kids", counted);
        }
    }
    if !has(&found, "budget") {
        // Chat-speak hides the amount: "ive got 60 pounds to spend" is a budget once the folded
        // line reads "i have got 60 pounds to spend", so the folded line is read for the fact.
        let folded = _folded(text_, memory);
        if !folded.is_empty() && folded != fold(text_) {
            let budget = vget(&extract::extract(&folded, question), "budget").clone();
            if budget.truthy() {
                found.set_key("budget", budget);
            }
        }
    }
    if has(&found, "budget") && !A_MONEY_SIGN.is_match(vget(&found, "budget").str()) {
        // "ive got £60 to spend": the folded line is plain words, so the sign is read off the
        // line they typed and put back on the amount.
        let budget = py_str(vget(&found, "budget"));
        let sign = _budget_sign(text_, &budget);
        if !sign.is_empty() {
            found.set_key("budget", format!("{}{}", sign, text::lstrip(&budget)));
        }
    }
    let mut correction = found.del_key("correction").unwrap_or(Value::None);
    if correction.is_none() {
        // "its Preeya not Priya": a correction with no marker in front of it is only one when the
        // store already holds the value it takes back, which is the store's question, not the
        // reader's.
        correction = _a_correction_without_a_marker(text_, memory);
    }
    let dropped = found.del_key("drop").unwrap_or(Value::None);
    let retracted = found.del_key("retract").unwrap_or(Value::None);
    if !has(&found, "allergies") {
        // "im allergic to peanuts" then "and milk too": the line names one more thing and no
        // fact of its own, so it is the allergy on record with the milk in it. A food word on its
        // own is not enough - nothing is added to an allergy the person never gave.
        let more = extract::one_more(text_).val();
        let held = vget(user_ro(memory), "allergies").clone();
        if more.truthy() && held.truthy() {
            let fresh: Vec<Value> = more.items().iter().filter(|item| !held.items().contains(item)).cloned().collect();
            found.set_key("allergies", list_of(fresh));
        }
    }
    // Someone named in this same line comes first: "my mum is allergic to nuts and she doesnt
    // eat meat" is two facts about the mum, not one about the mum and one about nobody.
    let named_here: Vec<String> = vget(&found, "people").items().iter().filter_map(|who| who.as_str().map(str::to_string)).collect();
    _remember_dislikes(text_, memory, &named_here);
    let second = _a_second_persons_limit(text_, memory);
    if second.truthy() {
        // Filed with the rest of this line's people, so it is logged and merged like any other: the
        // reader is the only one that knows the limit, because the limit is the line before's.
        found.setdefault("others", Value::list()).push(second);
    }
    let their_facts: Vec<Value> = match found.del_key("others") {
        Some(v) if v.truthy() => v.items().to_vec(),
        _ => Vec::new(),
    };
    for mut entry in their_facts {
        let pronoun = entry.del_key("pronoun").unwrap_or(Value::None);
        let pronoun_s = pronoun.as_str().unwrap_or("").to_string();
        if !has(&entry, "who") {
            if pronoun.truthy() && pronoun_s != "she" && pronoun_s != "he" {
                // "they" is nobody in particular, so the age is not the last person's: "my son is
                // 7" then "theyre 70 next month" is not the son's seventieth birthday, and folding
                // it onto him took the age the plain line gave away. Kept for the "how old are
                // they" ask instead, where the answer is about the same "they".
                let age = sr::_a_number(vget(&entry, "age")).val();
                if age.truthy() {
                    let turns = vget(&entry, "turns");
                    let turns = if turns.truthy() { turns.clone() } else { Value::from("") };
                    memory.set("their_age", dict_of(vec![("age", age), ("turns", turns)]));
                }
                continue;
            }
            let mut who = _person_from_pronoun(text_, memory, &named_here);
            if pronoun.truthy() {
                // "she's 78 and doesn't walk far" in a chat about her mum: the age goes on the
                // mum, never on the dad the chat was on before, because the pronoun has to fit.
                let fits = if pronoun_s == "she" { "SHE" } else { "HE" };
                if !_pronoun_fits(&who, fits) {
                    who = String::new();
                }
            }
            if who.is_empty() {
                if (pronoun_s == "she" || pronoun_s == "he") && is_int(vget(&entry, "months")) {
                    // An age in months said with only a pronoun is still the chat's fact: "rough
                    // night, the baby was up 3 times" then "shes 4 months" names nobody the store
                    // can file it on, and dropping it made "how old did i say she was" say she had
                    // never said. Kept against the pronoun, the same way "they" is, so the ask
                    // about her can answer it.
                    entry.set_key("pronoun", pronoun.clone());
                    remember_other(memory, &entry, text_);
                }
                continue; // a pronoun and nobody to put it on is not a fact about nobody
            }
            entry.set_key("who", who);
        }
        let who = vget(&entry, "who").clone();
        if pronoun.truthy() && has_key(&entry, "age") && CORRECTING.is_match(text_) {
            // "my son is 7 kilos" stores no age; "actually hes 8" then corrects nothing, so
            // it must not become the first age he has. A correction replaces a fact.
            let known = others(memory).items().iter().find(|one| vget(one, "who") == &who).cloned();
            if !known.map_or(false, |known| known.truthy() && has_key(&known, "age")) {
                continue;
            }
        }
        let was = others(memory).items().iter().find(|one| vget(one, "who") == &who).cloned();
        remember_other(memory, &entry, text_);
        _note_a_corrected_fact(memory, text_, &entry, was.as_ref());
        _note_a_fact_moved(memory, text_, &entry, was.as_ref());
    }
    found = _a_detail_does_not_replace_the_job(found, memory);
    let held_name = sget(user_ro(memory), "name");
    for (key, value) in found.pairs() {
        remember(memory, key.str(), value, text_);
    }
    if has(&found, "name") && !held_name.is_empty() && py_str(vget(&found, "name")).to_lowercase() != held_name.to_lowercase() {
        // A name that arrives when another is already on record is the name put right, whether
        // or not the line carries a correcting word: "im tom" then "i go by tomas" was answered
        // as a first introduction - "Hi Tomas - good to meet you" - one line after meeting Tom,
        // and the old name went without a word. Said as the correction it is, the person hears
        // which name the bot goes with.
        memory.set(
            "name_fixed",
            dict_of(vec![
                ("name", Value::from(py_str(vget(&found, "name")))),
                ("from", Value::from(held_name.as_str())),
                ("said", Value::from(fold(text_))),
            ]),
        );
    }
    if has(&found, "job") && !has(&found, "job_frame") {
        // A job said plainly ("i'm a chef now") is said back the same way: the frame belongs to
        // the job it was said with, so "i work in a warehouse" then "i'm a chef" is not "in a
        // chef" - the old frame goes with the old job.
        if let Some(known) = memory.get_mut("user") {
            if known.truthy() {
                known.del_key("job_frame");
            }
        }
    }
    // A name put right moves the name on the one pet it is about, and the "last pet" reader below
    // would otherwise drop that name on whichever pet came last: "i have two cats called milo and
    // tom" then "no hes called max" renamed Tom and then Milo, leaving one cat called Max.
    _fix_a_pets_name(text_, memory);
    if !has_key(&found, "pets") {
        _name_the_last_pet(text_, memory);
        _age_the_named_pet(text_, memory);
        _age_they_answered(text_, memory);
        _age_said_of_the_only_animal(text_, memory);
        _fix_a_pets_age(text_, memory);
    }
    _fix_a_persons_age(text_, memory);
    _fix_the_weeks_of_a_pregnancy(text_, memory);
    let fixed = _fix_a_persons_name(text_, memory);
    if fixed.truthy() {
        // The correction is the turn's news: the summary has the new name, and the reply says
        // both halves so the person knows the old one is gone.
        let mut kept = fixed.clone();
        kept.set_key("said", fold(text_));
        memory.set("name_fixed", kept);
    }
    if !has_key(&found, "owns") {
        _fix_the_make_of_a_thing(text_, memory);
    }
    _the_name_they_gave_to_the_person_they_mean(text_, memory);
    _a_trip_booked_named(text_, memory);
    if correction.truthy() {
        crate::memory::corrected::_apply_correction(memory, &correction, text_);
        _fix_a_pets_name(text_, memory);
        return;
    }
    if dropped.truthy() && _drop_value(memory, &dropped, text_).truthy() {
        return;
    }
    if retracted.truthy() && _retract_last(memory, text_).truthy() {
        return;
    }
    _forget_denied(text_, memory);
    _the_back_on_the_plate(text_, memory);
    if ret_true(sr::_about_this_chat(text_, memory)) {
        return;
    }
    if ret_true(crate::memory::told::a_callback_the_chat_has(text_, memory)) {
        // "remember when i said my knee hurt" names a thing the conversation is already on, so
        // the line adds no subject: filing it made "remember" a word of the topic, and the turn
        // after it ("its better now") was read against remembering and answered with a revision
        // tip. A callback naming something the chat does not hold ("remember when we talked
        // about lisbon") does say what the conversation is on, so that one is filed as usual.
        return;
    }
    // A line that put a value right is the turn's news, not a new subject: the conversation is
    // still on the thing the value belongs to. Noted as a topic, "my dog is called rex" then
    // "his names actually rocky" left the chat on a subject whose only word was "name", and
    // "how old is he" after it was answered with a tip on remembering names instead of with the
    // dog. The record the correction fixed stays the current one, so the next line about the
    // dog is read against the dog.
    if !_this_line_put_something_right(memory, text_) {
        note(memory, text_, kind.as_deref());
    }
}

/// `bot.memory.note(memory, text, kind)`.
fn note(memory: &mut Memory, text_: &str, kind: Option<&str>) {
    crate::memory::topics::note(memory, text_, kind.unwrap_or(""));
}

/// Python: bot/modules/memory_store.py:1048 `_a_trip_booked_named(text, memory)`
///
/// A trip that is already booked is a trip the chat is on, and the store keeps it: the place is
/// filed, and the plan in the shape the other trips are kept in ("Cornwall in August"), unless
/// the store is already on that plan.
pub fn _a_trip_booked_named(text_: &str, memory: &mut Memory) {
    let booked = crate::memory::trips::the_trip_they_booked(text_).val();
    let place = s_or(booked.items().first());
    let when = s_or(booked.items().get(1));
    if place.is_empty() {
        return;
    }
    let known = user(memory).clone();
    let place = cap1(&place);
    let plan = if !when.is_empty() { text::squash(&format!("{place} in {when}")) } else { place.clone() };
    let places: Vec<String> = vget(&known, "places").items().iter().map(py_str).filter(|one| !text::strip(one).is_empty()).collect();
    if !places.iter().any(|one| one.to_lowercase() == place.to_lowercase()) {
        remember(memory, "places", &Value::from(place.as_str()), text_);
    }
    if text::strip(&sget(&known, "plan")).to_lowercase() != plan.to_lowercase() {
        // The plan is the last thing the line states, the way a trip said as a plan keeps it, so a
        // correction after the booking moves the trip rather than the place on its own.
        remember(memory, "plan", &Value::from(plan.as_str()), text_);
        remember(memory, "plans", &Value::from(plan.as_str()), text_);
    }
}

/// Python: bot/modules/memory_store.py:1075 `_this_line_put_something_right(memory, text)`
///
/// Whether this very line is the correction one of the readers has just applied.
pub fn _this_line_put_something_right(memory: &Memory, text_: &str) -> bool {
    let said = fold(text_);
    if said.is_empty() {
        return false;
    }
    for key in ["correction", "name_fixed"] {
        let kept = mget(memory, key);
        if kept.is_dict() && said == text::squash(&sget(kept, "said")) {
            return true;
        }
    }
    false
}

inline_re!(LOWER_WORDS, r"[a-z']+", 0);

/// Python: bot/modules/memory_store.py:1091 `_on_record_before(memory, records)`
///
/// The animals an earlier line put on record, out of the ones this line has just taken back. A
/// take-back only says itself when there was something to take back.
pub fn _on_record_before(memory: &Memory, records: &Value) -> Vec<Value> {
    let said = memory.turns.iter().map(|turn| turn.user.to_lowercase()).collect::<Vec<_>>().join(" ");
    let words: HashSet<String> = LOWER_WORDS.findall_str(&said).iter().map(|word| stem(word)).collect();
    let mut kept = Vec::new();
    for record in records.items() {
        if !record.is_dict() {
            continue;
        }
        for word in [vget(record, "animal"), vget(record, "name")] {
            if word.truthy() && words.contains(&stem(&py_str(word).to_lowercase())) {
                kept.push(record.clone());
                break;
            }
        }
    }
    kept
}

/// Python: bot/modules/memory_store.py:1112 `_forget_denied(text, memory)`
///
/// A line that says they do not have a fact takes the old one out of the store. A store that
/// keeps it goes on warning about, or cooking around, the thing the person has just said they
/// are fine with.
pub fn _forget_denied(text_: &str, memory: &mut Memory) -> Option<Value> {
    let held = vget(user_ro(memory), "pets").clone();
    let held = if held.truthy() { held } else { Value::list() };
    let mut denied = extract::denials(text_, &held);
    // A denial about somebody else comes back under its own key and comes off that person's
    // record, not the user's: "my son is allergic to nuts" then "he isnt allergic to nuts" left
    // the nuts on his list and answered "Your son is allergic to nuts." after the line that took
    // them back.
    let theirs = if denied.is_dict() { denied.del_key("theirs").unwrap_or(Value::None) } else { Value::None };
    let denied_pets: Vec<Value> = if has(&denied, "pets") { vget(&denied, "pets").items().to_vec() } else { Vec::new() };
    // A pet that died is one they no longer have, but it is not a take-back to say back to them:
    // the loss is answered warmly, and "Right - no cat, then." is the wrong reply to a death.
    // The store still hears it, or it hands the animal back later ("what pets do i have" after
    // "my cat died" must not answer "You've got a cat called Milly.").
    let died: Vec<Value> = _losses_taking(text_, &held, memory)
        .into_iter()
        .filter(|pet| {
            // `_kind` here is store_replies' second definition (the kind of thing a word names):
            // it tests `thing in words` on frozensets, so a record in the list is a TypeError.
            let kinds: Vec<Value> = denied_pets
                .iter()
                .map(|item| match item.as_str() {
                    Some(thing) => sr::_kind(thing).val(),
                    None => panic!("TypeError: unhashable type: {:?}", py_repr(item)),
                })
                .collect();
            !kinds.contains(&Value::from(sget(pet, "animal")))
        })
        .collect();
    // "we gave one away" out of "i have two cats" names no animal, only a number: one cat goes,
    // and the store keeps the count straight instead of answering "You've got two cats."
    let given: Vec<Value> = extract::given_away_one_records(text_, &held)
        .into_iter()
        .filter(|pet| !denied_pets.iter().any(|item| item.is_dict() && item == pet))
        .collect();
    if !denied.truthy() && !theirs.truthy() && died.is_empty() && given.is_empty() {
        return None;
    }
    let mut gone = Value::dict();
    for (key, value) in denied.pairs().to_vec() {
        let key = key.str();
        let mut value = value;
        if key == "diet" {
            value = _the_diet_they_are_back_on(memory, &value);
        }
        let mut dropped = forget(memory, key, &value, Some("gone"));
        if key == "pets" {
            // The store forgets either way; only a pet they had before is worth saying back.
            dropped = list_of(_on_record_before(memory, &dropped));
        }
        if dropped.truthy() {
            gone.set_key(key, dropped);
            if key == "pets" && py_list(&value).iter().any(|item| py_str(item) == "*") {
                // "no pets" took them all: the line says that, not the last animal left.
                gone.set_key("pets_all", true);
            }
        }
    }
    if !died.is_empty() && forget(memory, "pets", &list_of(died), Some("died")).truthy() {
        gone.set_key("pets_died", true);
    }
    if !given.is_empty() && forget(memory, "pets", &list_of(given), Some("given away")).truthy() {
        // One of a set is gone, not the whole kind: "Right - no cat, then." would say they have
        // none left, so the store hears it and the turn is answered like any other.
        gone.set_key("pets_given", true);
    }
    if theirs.truthy() {
        for entry in theirs.items() {
            if let Some(taken) = _off_their_list(memory, entry, text_) {
                if taken.truthy() {
                    for (k, v) in taken.pairs() {
                        gone.set_key(k.clone(), v.clone());
                    }
                    break;
                }
            }
        }
    }
    if !gone.truthy() {
        return None;
    }
    gone.set_key("said", fold(text_));
    memory.set("denial", gone.clone());
    Some(gone)
}

/// Python: bot/modules/memory_store.py:1170 `_the_back_on_the_plate(text, memory)`
///
/// A line that puts back what somebody else does not eat takes it off their record. Only what is
/// already on that person's record is lifted, so a line that describes somebody for the first
/// time ("my son eats fish") is left to the modules that read it on its own.
pub fn _the_back_on_the_plate(text_: &str, memory: &mut Memory) -> bool {
    for said in extract::their_back_records(text_) {
        let mut who = text::strip(&sget(&said, "who")).to_string();
        if who.is_empty() {
            who = _person_from_pronoun(text_, memory, &[]);
        }
        if who.is_empty() {
            continue;
        }
        let count = others(memory).len();
        for at in 0..count {
            let mut record = others(memory).items()[at].clone();
            if sget(&record, "who").to_lowercase() != who.to_lowercase() {
                continue;
            }
            let kind = vget(&said, "kind").clone();
            let kind_s = kind.as_str().unwrap_or("");
            let mut food = text::strip(&sget(&said, "food")).to_string();
            if ["them", "it", "that"].contains(&food.to_lowercase().as_str()) {
                // "actually she loves them" names the food only by pointing at it, and there is
                // one thing on her record to point at or the line cannot be read at all.
                food = _the_one_they_are_back_on(&record, kind_s);
            }
            if food.is_empty() {
                continue;
            }
            if kind_s == "diet" {
                let held = sget(&record, "diet").to_lowercase();
                if held != format!("no {}", food.to_lowercase()) && held != format!("no {}s", food.to_lowercase()) {
                    continue;
                }
                // The restriction is gone and the food they do eat takes its place: the record
                // says "eats fish" so "what does my son eat" answers with it, not with a blank.
                record.set_key("diet", format!("eats {food}"));
            } else {
                let gone = py_strs(vget(&record, "dislikes"));
                if !gone.iter().any(|item| item.to_lowercase() == food.to_lowercase()) {
                    continue;
                }
                let left: Vec<String> = gone.iter().filter(|item| item.to_lowercase() != food.to_lowercase()).cloned().collect();
                record.set_key("dislikes", str_list(&left));
                let mut likes = py_strs(vget(&record, "likes"));
                if !likes.iter().any(|item| item.to_lowercase() == food.to_lowercase()) {
                    likes.push(food.clone());
                }
                record.set_key("likes", str_list(&likes));
            }
            others(memory).items_mut().unwrap()[at] = record;
            memory.set(
                "reversal",
                dict_of(vec![("who", Value::from(who.as_str())), ("food", Value::from(food.as_str())), ("kind", kind), ("said", Value::from(fold(text_)))]),
            );
            return true;
        }
    }
    false
}

/// Python: bot/modules/memory_store.py:1217 `_the_one_they_are_back_on(record, kind)`
///
/// The one thing on a person's record a "them" can point at, or "" when there are two.
pub fn _the_one_they_are_back_on(record: &Value, kind: &str) -> String {
    if kind == "diet" {
        let held = sget(record, "diet");
        let held = text::strip(&held);
        return if held.to_lowercase().starts_with("no ") { text::strip(&sl(held, 3, None)).to_string() } else { String::new() };
    }
    let gone = py_strs(vget(record, "dislikes"));
    if gone.len() == 1 {
        gone[0].clone()
    } else {
        String::new()
    }
}

/// Python: bot/modules/memory_store.py:1233 `_off_their_list(memory, entry, text)`
///
/// The take-back a line makes about somebody else, taken off the record kept for them. The note
/// is said even when they never had it on record.
pub fn _off_their_list(memory: &mut Memory, entry: &Value, text_: &str) -> Option<Value> {
    let items: Vec<String> = vget(entry, "allergies").items().iter().map(py_str).filter(|item| !item.is_empty()).collect();
    if items.is_empty() {
        return None;
    }
    let mut who = sget(entry, "who");
    if who.is_empty() {
        who = _person_from_pronoun(text_, memory, &[]);
    }
    if who.is_empty() {
        return None;
    }
    let lowered: HashSet<String> = items.iter().map(|one| one.to_lowercase()).collect();
    let count = others(memory).len();
    for at in 0..count {
        let record = others(memory).items()[at].clone();
        if sget(&record, "who") != who {
            continue;
        }
        let held = py_strs(vget(&record, "allergies"));
        let left: Vec<String> = held.iter().filter(|item| !lowered.contains(&item.to_lowercase())).cloned().collect();
        if left.len() == held.len() {
            continue;
        }
        others(memory).items_mut().unwrap()[at].set_key("allergies", str_list(&left));
    }
    Some(dict_of(vec![("allergies", str_list(&items)), ("who", Value::from(who))]))
}

/// Python: bot/modules/memory_store.py:1264 `_the_diet_they_are_back_on(memory, value)`
///
/// The diet as the store holds it, when the take-back names only the food.
pub fn _the_diet_they_are_back_on(memory: &Memory, value: &Value) -> Value {
    let held = vget(user_ro(memory), "diet");
    let said = fold(&s_or(Some(value)));
    let Some(held) = held.as_str() else { return value.clone() };
    if said.is_empty() {
        return value.clone();
    }
    if fold(held).contains(&said) {
        return Value::from(held);
    }
    value.clone()
}

/// Python: bot/modules/memory_store.py:1285 `_the_last_line(memory)`
///
/// The line just before this one, so a bare "he died" can point back at what it named.
pub fn _the_last_line(memory: &Memory) -> String {
    memory.turns.last().map(|t| t.user.clone()).unwrap_or_default()
}

/// Python: bot/modules/memory_store.py:1353 `_losses_taking(text, held, memory=None)`
///
/// The animals a line about a death takes off the store, one record at a time.
pub fn _losses_taking(text_: &str, held: &Value, memory: &Memory) -> Vec<Value> {
    let said = text_.to_lowercase();
    let everything = data::re("bot.memory.facts.LOSS_ALL").is_match(&said);
    let mut taken: Vec<Value> = Vec::new();
    let before = _the_last_line(memory);
    for kind in extract::losses(text_, held, &before).val().items() {
        let same: Vec<&Value> = held.items().iter().filter(|pet| pet.is_dict() && _animal_family(vget(pet, "animal")) == _animal_family(kind)).collect();
        if same.is_empty() {
            continue;
        }
        if everything {
            for pet in &same {
                if !taken.contains(pet) {
                    taken.push((*pet).clone());
                }
            }
            continue;
        }
        let named: Vec<&&Value> = same
            .iter()
            .filter(|pet| {
                has(pet, "name") && Re::new(&format!(r"\b{}\b", pyre::escape(&py_str(vget(pet, "name")).to_lowercase())), 0).is_match(&said)
            })
            .collect();
        // "rex died in january" is about Rex and not about Sam: the animal they named goes.
        taken.push(match named.last() {
            Some(pet) => (**pet).clone(),
            None => (*same[same.len() - 1]).clone(),
        });
    }
    taken
}

/// Python: bot/modules/memory_store.py:1381 `_last_stated(memory)`
///
/// The facts the line before this one put in the store.
pub fn _last_stated(memory: &mut Memory) -> Vec<Value> {
    let said = fold(&last_user(memory));
    if said.is_empty() {
        return Vec::new();
    }
    log(memory).items().iter().filter(|entry| entry.is_dict() && fold(&sget(entry, "text")) == said).cloned().collect()
}


/// Python: bot/modules/memory_store.py:1396 `_retract_last(memory, text='')`
///
/// A line that takes back the last thing they said without naming it: "take that back", "scratch
/// that". A retraction names no value, so the only thing it can be about is the line before it:
/// every fact that line put in the store goes. A taste or a fact about someone else is left
/// alone.
pub fn _retract_last(memory: &mut Memory, text_: &str) -> Value {
    let mut gone: Vec<String> = Vec::new();
    let mut shown: Vec<String> = Vec::new();
    for entry in _last_stated(memory) {
        let key = vget(&entry, "key").clone();
        let value = vget(&entry, "value").clone();
        let key_s = key.as_str().unwrap_or("");
        if key_s == "pet" {
            // The animal is taken off the record outright, not filed under `pets_gone`: that list is
            // for a pet that died or went to a new home, and answering "you don't have your dog
            // Biscuit any more" tells them about a loss they never mentioned.
            let known = user(memory);
            let had = vget(known, "pets").items().to_vec();
            let pets: Vec<Value> = had.iter().filter(|pet| *pet != &value).cloned().collect();
            if pets.len() != had.len() {
                if !pets.is_empty() {
                    known.set_key("pets", list_of(pets));
                } else {
                    known.del_key("pets");
                }
                gone.push("pets".to_string());
                let animal = [vget(&value, "animal"), vget(&value, "name")].into_iter().find(|v| v.truthy()).map(py_str).unwrap_or_else(|| "pet".to_string());
                shown.push(animal);
            }
            continue;
        }
        if key_s == "plans" {
            user(memory);
            if forget(memory, "plans", &list_of(vec![value.clone()]), None).truthy() {
                gone.push("plans".to_string());
                shown.push(py_str(&value));
            }
            let known = user(memory);
            if vget(known, "plan") == &value {
                known.del_key("plan");
            }
            continue;
        }
        if g("NOT_RETRACTED").items().contains(&key) {
            continue;
        }
        let dropped = forget(memory, key_s, &value, None);
        if dropped.truthy() {
            gone.push(py_str(&key));
            shown.extend(dropped.items().iter().filter(|item| !item.is_dict()).map(py_str));
        }
    }
    if gone.is_empty() {
        return Value::None;
    }
    if gone.iter().any(|key| key == "plans" || key == "places") {
        // The trip went with the plan, so the where ask names what they took back instead of an
        // empty store ("You haven't mentioned a place yet" denies the trip they just discussed).
        // As the store held it, not lower-cased: the place is a name, and "Sri Lanka is off" is
        // the trip said back where "Sri lanka" is the line read aloud.
        let first = shown.first().cloned().unwrap_or_else(|| panic!("IndexError: list index out of range"));
        user(memory).set_key("plan_dropped", first);
    }
    let retraction = dict_of(vec![("said", Value::from(fold(text_))), ("keys", str_list(&gone)), ("shown", str_list(&shown))]);
    memory.set("retraction", retraction.clone());
    retraction
}

// ---------------------------------------------------------------------------------------------
// What the store reads for certain.

rx!(NAMED_FACT_ASK, TOPIC_ASK, ASK_WITHOUT_MY, HOW_MANY_OF_MINE, OTHER_FACT, CANT_HAVE, CAN_HAVE, CAN_THEY_HAVE, STATES_A_LIMIT, A_HAVE_QUESTION);

/// A module-level value of `bot.memory.facts` (the Python module reads them as `extract.NAME`).
fn fx(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.facts.{name}"))
}

/// Python: bot/modules/memory_store.py:1456 `_reads_the_store(text, memory=None)`
///
/// True for a line that only asks for something already kept, so it states nothing new. An ask
/// about a person's allergy or a food they cannot have is the same kind of line; said with nobody
/// the store knows, it claims nothing.
pub fn _reads_the_store(text_: &str, memory: &mut Memory) -> bool {
    let low = _bare(&_folded(text_, memory));
    if NAMED_FACT_ASK.is_match(&low) || TOPIC_ASK.is_match(&low) || ASK_WITHOUT_MY.is_match(&low) || HOW_MANY_OF_MINE.is_match(&low) {
        return true;
    }
    if !(OTHER_FACT.is_match(&low)
        || CANT_HAVE.is_match(&low)
        || CAN_HAVE.is_match(&low)
        || CAN_THEY_HAVE.is_match(&low)
        // "what can she not eat" and "remind me what hes allergic to" name a limit and a
        // person, and the person is a pronoun with nobody behind it, so the asks that need a
        // relation never took the line. EAT_ASK is deliberately not in this list: "and how
        // long do i need for the chicken" has an eating word in it and is the answers lane's,
        // and read as an ask that names nothing it stopped the store filing the chicken, so
        // the next turn of a roast conversation answered about hens.
        || ret_true(sr::_a_bare_limit_ask(&low, memory)))
    {
        return false;
    }
    if !ret_true(sr::_asks_about_someone(&low, text_, memory)) {
        return false;
    }
    let whole = fold(text_);
    if STATES_A_LIMIT.is_match(&whole) && !A_HAVE_QUESTION.is_match(&whole) {
        // The line says the limit and asks about it in one breath: "my son is 7 and has a nut
        // allergy, what is he allergic to". The fact is the line's own and keeping it is another
        // reading's job, so this one stands down. Claimed here, the store told a person who had
        // just said their son has a nut allergy that it had not been told about an allergy, and
        // the answer it took the turn from was a real one.
        return false;
    }
    // The reader only says whose fact is missing when the line does name a person it can read
    // (a relation, a name the store knows, or a "he" that points at one), so that line is the
    // store's own too: "is he allergic to anything" after three people were named was answered
    // with what to check on a label, as if "he" were the person typing.
    ret_true(sr::_other_person_reply(memory, &low))
}

rx!(WHERE_PLAN, TIME_ASKS, TIME_THING, TIME_IT, DAY_THING, DAY_IT);

/// Python: bot/modules/memory_store.py:1548 `_the_plan_they_took_back(text, memory)`
///
/// A where ask about a trip they have just taken back is the store's line, not a repeat. Only a
/// where ask, and only while the thing they took back is the newest thing in the store.
pub fn _the_plan_they_took_back(text_: &str, memory: &mut Memory) -> bool {
    match vget(user(memory), "plan_dropped").as_str() {
        Some(gone) if !text::strip(gone).is_empty() => {}
        _ => return false,
    }
    WHERE_PLAN.is_match(&fold(text_))
}

/// Python: bot/modules/memory_store.py:1568 `_the_clock_in_their_line(text, memory)`
///
/// True when the ask wants a clock or a day the person gave, and the store can place it. Every
/// such ask it cannot place is left alone.
pub fn _the_clock_in_their_line(text_: &str, memory: &mut Memory) -> bool {
    // a claim that cannot be read is not a claim
    py_try(|| {
        let low = text_.to_lowercase();
        if !(TIME_ASKS.is_match(&low) || TIME_THING.is_match(&low) || TIME_IT.is_match(&low) || DAY_THING.is_match(&low) || DAY_IT.is_match(&low)) {
            return false;
        }
        ret_true(sr::_plans_reply(memory, &low))
    })
    .unwrap_or(false)
}

rx!(ASK_ABOUT_THEM, EAT_ASK);

/// Python: bot/modules/memory_store.py:1600 `_an_ask_about_someone_else(text, memory)`
///
/// True when the line asks about a fact the store keeps for someone else. Only a line that names
/// or points at a person whose fact is on record is claimed.
pub fn _an_ask_about_someone_else(text_: &str, memory: &mut Memory) -> bool {
    if !ASK_ABOUT_THEM.is_match(text::strip(text_)) {
        return false;
    }
    let mut keys: Vec<&str> = Vec::new();
    if OTHER_FACT.is_match(text_) {
        keys.push("allergies");
    }
    if EAT_ASK.is_match(text_) {
        keys.push("diet");
    }
    if keys.is_empty() {
        return false;
    }
    // a line that cannot be read claims nothing
    let Ok(who) = py_try(|| {
        let low = _bare(&_folded(text_, memory));
        ret_str(sr::_someone_else_named(&low, memory))
    }) else {
        return false;
    };
    if who.is_empty() {
        return false;
    }
    for known in others(memory).items() {
        if vget(known, "who").as_str() == Some(who.as_str()) {
            return keys.iter().any(|key| has(known, key));
        }
    }
    false
}

/// Python: bot/modules/memory_store.py:1632 `_the_take_back_about_someone(text, memory)`
///
/// A line that takes a fact back off somebody else's list is the store's, not the topic's.
pub fn _the_take_back_about_someone(text_: &str, memory: &Memory) -> bool {
    let gone = mget(memory, "denial");
    if !gone.is_dict() || !has(gone, "who") {
        return false;
    }
    vget(gone, "said").as_str() == Some(fold(text_).as_str())
}

/// Python: bot/modules/memory_store.py:1646 `_the_part_that_hurts_asked(text, memory)`
///
/// The ask for the part that hurts, and the side it was said with. The claim is made for every
/// line the hurting-part reader can answer, and for no other.
pub fn _the_part_that_hurts_asked(text_: &str, memory: &Memory) -> bool {
    let low = fold(text_);
    !low.is_empty() && ret_true(crate::memory::whathurts::the_part_asked(memory, &low))
}

rx!(WHOLE_ASK, POINTS_BACK_AT_IT, WORK_ON_IT);

/// Python: bot/modules/memory_store.py:1661 `_the_store_readings(text, memory=None)`
///
/// The lines this store read for certain: a read of what it keeps, the ask whose answer is a
/// line they said themselves, a `when` ask it can place, and a question about what the chat
/// already holds. All are answered from what the person said, so a topic the knowledge base
/// declares must not take the turn.
///
/// This is a reading, not the hook: the exported name is `TAKES_AS_IT_COMES` at the end of the
/// file, and it asks this one first.
pub fn _the_store_readings(text_: &str, memory: &mut Memory) -> bool {
    if _reads_the_store(text_, memory)
        || _their_own_line_ask(text_, memory)
        || _the_part_that_hurts_asked(text_, memory)
        || _their_fact_ask(text_, memory)
        || _a_kept_fact_read(text_, memory)
        || ret_true(crate::memory::decided::asks(text_))
        || ret_true(crate::memory::told::ABOUT_THE_CHAT(text_, Some(&*memory)))
        || ret_true(crate::memory::told::A_CALLBACK(text_, Some(&*memory)))
    {
        return true;
    }
    if WHOLE_ASK.match_(&text::strip(text_).to_lowercase()).is_some() && ret_true(sr::_a_whole_answer_to_hand_back(memory, text_)) {
        // "in full", "the whole thing", "all of it": the line names no thing of its own, so the
        // specialist rule read the chat's topic on it and handed the turn to an entry that folded
        // "the full" as a thing ("The same goes for the full: If you forget, keep a filled bottle
        // ...") instead of the answer the person asked for again. `_repeat_ask` already reads this
        // ask - it is the same `WHOLE_ASK` and the same "is there a whole answer behind it" test
        // that keeps "hi" / "in full" with the small talk - but nothing asked it from here, so the
        // store never claimed the line.
        return true;
    }
    if ret_true(sr::_a_state_that_was(text_, memory, "")) {
        // A pregnancy or a feeding that is over is this store's line and nobody else's: the
        // knowledge base reads the word "pregnant" and serves the trying-for-a-baby entry, and the
        // news reader congratulates a state that has passed. The store has nothing to file for it,
        // so it only says the tense back.
        return true;
    }
    if _a_feeling_about_their_own_news(text_, memory) {
        // A bare feeling is about the thing they said a moment ago, which is a read of this store
        // and nothing the knowledge base or the small-talk topics know about.
        return true;
    }
    if _the_plan_they_took_back(text_, memory) {
        // A where ask whose trip was taken back the turn before, answered by the store that knows
        // it went rather than by the paragraph that was just taken off the record.
        return true;
    }
    if _the_take_back_about_someone(text_, memory) {
        // "he isnt allergic to nuts" one line after "my son is allergic to nuts": the topic is
        // nuts and the line is about the chat, so the entry that knows nuts must not take it.
        return true;
    }
    if ret_true(sr::_can_close(text_, memory)) {
        // A line that asks to leave the subject is not about the subject, so the entry that knows
        // it must not take the turn ("can we talk about something else" after a hospital line was
        // answered with a care assessment). The store knows what the subject was, so it answers.
        return true;
    }
    if ret_true(sr::_the_call_from_the_advice(text_, memory)) {
        // A decision the answer above already settled: the line names no subject of its own, so
        // the specialist rule would read the chat's topic on it and hand the turn to that entry.
        return true;
    }
    if _asks_whether_we_followed(text_, memory) {
        // "did you get that" names nothing of the world, so the specialist rule read the chat's
        // topic on it and the knowledge entry that topic belongs to took the turn - the person
        // asking whether the bot had followed got a lookup. The store knows what it kept.
        return true;
    }
    if ret_true(crate::memory::rates::is_a_rate_ask(text_, memory)) {
        // The ask for a rate this store holds: "how often do i go to the gym" after "i go twice a
        // week". The exercise entry answers from the guideline a person should keep, which is the
        // wrong answer to someone who told this store their own.
        return true;
    }
    if ret_true(crate::memory::times::is_a_time_ask(text_, memory)) {
        // The ask for the hour their own day runs on ("what time do i get up" after "i get up at
        // seven"). The sleep entry answers from the routine a person should keep - "work it back
        // from the morning" - which is the wrong answer to someone who told this store their own
        // hour: the line names one of the times on file and nothing of its own, so the store
        // answers it.
        return true;
    }
    if _an_ask_about_someone_else(text_, memory) {
        // The fact is in this store, filed under the person the line named: the knowledge base
        // has nothing about their sister's allergy.
        return true;
    }
    if _the_group_size_ask(text_, memory) {
        // How many the person's own group is, when this store holds the number: "how many of us"
        // after "we are four of us" was answered with the dish card the cooking topic had in hand
        // ("Quick: tuna pasta in 15, macaroni cheese in 25..."), one turn after the person had
        // said how many they were. The number is kept here, so the topic must not take the line.
        return true;
    }
    if ret_true(crate::memory::repeat::how_long_its_been(text_)) && ret_true(crate::memory::repeat::the_figure_they_gave_text(text_, memory)) {
        // The length they gave about their own thing, asked back with the thing named: "how long
        // has my back been like this now" after "my lower back has been sore for about 3 weeks".
        // Claimed here because the carry into the card that answers a back took the turn and said
        // what keeps a back quiet, while the three weeks were in the chat.
        return true;
    }
    if _the_figure_the_answer_names(text_, memory).is_some() {
        // A figure ask that names no thing ("how many weeks" after the training answer) is read
        // off the answer the person just read, and `respond` says that figure. Claimed here
        // because a card the bare unit reaches answers it with a length of the wrong kind, and a
        // hook that says yes on a line `respond` answers differently would take the turn off the
        // module that should have had it.
        return true;
    }
    if ret_true(sr::_the_budget_on_the_answer(text_, memory)) {
        // How much time there is is not a new question: it is the room the answer above has, and
        // read as a question the knowledge base took it for one - "and i only get 20 minutes"
        // after the lunch answer reached the eggs, the fastest dinner there is.
        return true;
    }
    if ret_true(sr::_the_back_this_turn(text_, memory)) {
        // The line that puts back what somebody else does not eat is the store's own: only the
        // store knows what it lifts, and the small talk answered it with "Got it - he eats fish."
        // while the diet it lifts stayed on his record and came back on the next ask.
        return true;
    }
    if ret_true(sr::_the_same_fact_about_someone(text_, memory).map(|(who, fact)| (who, fact))) {
        // "and my wife is too" after "im allergic to nuts" carries no subject of its own, so the
        // specialist rule read the chat's topic on it and handed the turn to the entry about that
        // diet ("same for my wife" after "im vegan" came back as the vegan amounts). The fact the
        // "too" points at is this store's, and the note it makes is kept with the reply, so the
        // store says so in front of it.
        return true;
    }
    // a module that cannot claim declares nothing
    py_try(|| {
        let line = _bare(&_folded(text_, memory));
        if (POINTS_BACK_AT_IT.is_match(&line) || WORK_ON_IT.is_match(&line)) && !ret_true(sr::_the_prep_day(memory, &line)) {
            // An ask that puts a verb on the thing ("to prep for it", "to book it") is answered by
            // the plan only when the store can name the day the work for it goes on. The plan said
            // back on its own is their own line again, and that echo was landing on the asks the
            // knowledge base has a real answer for ("when is the best time to book it" after a
            // Rome trip came back "You're going to rome in may."), so those are not claimed here.
            // "when is that again" is not this shape: the pointing word is the whole ask, not the
            // object of a verb, and the date they gave is the answer it wants.
            return false;
        }
        if _the_clock_in_their_line(text_, memory) {
            // "what time is my appointment" is the hour they gave, so it is not a question about
            // how to get an appointment. Read before the "when" gate because the line says
            // "what time" and that is the same ask.
            return true;
        }
        if !text_.to_lowercase().contains("when") {
            return _a_count_from_the_date(text_, memory);
        }
        ret_true(crate::memory::when::when_answer(text_, memory))
    })
    .unwrap_or(false)
}

rx!(PERSON_ADVICE_ASK, CAN_THEY_HAVE_A_FOOD, THEM, NO_RECORD);
static MY_RELATION: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"\bmy\s+(?:{})\b(?!'s|s\b)(?!s\b)", g("_RELATION").str()), 0));

/// `_named_person(low)` with no memory to hand (`memory=None`).
///
/// CONTRACT NOTE: the skeleton contract takes `&mut Memory` and cannot say "no memory"; an empty
/// one is handed over until the owner of `bot/memory/relations.py` lands `Option<&mut Memory>`.
fn _named_person_alone(low: &str) -> String {
    ret_str(relations::_named_person(low, None))
}

/// Python: bot/modules/memory_store.py:1865 `_their_fact_ask(text, memory=None)`
///
/// True for a line that only asks back a fact kept about someone else.
pub fn _their_fact_ask(text_: &str, memory: &mut Memory) -> bool {
    let low = _bare(&_folded(text_, memory));
    if PERSON_ADVICE_ASK.match_(&low).is_some() && !(CAN_THEY_HAVE.is_match(&low) && ret_true(sr::_a_limit_kept_for_them(&low, memory))) {
        // "can he have cheese" opens like an advice ask and is not one: the person is asking
        // whether one food is all right for somebody with a limit the store already holds, and
        // the limit is the answer. A line with no limit kept for them is advice and is left alone.
        return false;
    }
    let undeclared = ret_true(sr::_the_person_still_in_hand(&low, memory));
    if !(undeclared || OTHER_FACT.is_match(&low) || EAT_ASK.is_match(&low) || CANT_HAVE.is_match(&low) || CAN_HAVE.is_match(&low) || CAN_THEY_HAVE.is_match(&low)) {
        return false;
    }
    // A line that tells the fact is not asking for it: "my son doesnt eat meat" and "my sister
    // is vegan" are the cooking entry's lines, and the store answering them with the fact said
    // back ("Your son doesn't eat meat.") is an echo where the dinner should be. Only an ask is
    // the store's, with or without the question mark.
    if !(undeclared || ret_true(crate::memory::corrected::_asks(&low)) || text_.contains('?') || _asks_about_someone_else(text_)) {
        // "does my mum eat meat she is vegetarian" is an ask and a telling in one breath, and most
        // people type it with no question mark: the auxiliary at the front and the person after it
        // are what make it an ask (`_asks_about_someone_else` reads exactly that shape).
        return false;
    }
    // Someone else is in the line: a pronoun, or a relation of theirs ("my mum"). A possessive in
    // front of another noun ("my mums car") is the car, so the word has to be the person.
    if !(undeclared || THEM.is_match(&low) || !_named_person_alone(&low).is_empty() || MY_RELATION.is_match(&low)) {
        return false;
    }
    if CAN_THEY_HAVE_A_FOOD.is_match(&low) && !ret_true(sr::_a_limit_kept_for_them(&low, memory)) {
        // "can she have cheese" is answered from a limit the store holds for that person. With
        // nothing kept but their name, the store has no fact to answer with, and saying so about
        // a person the chat only named once is not worth the turn; with only an allergy kept, the
        // foods that contain the thing are the cooking entries' reading, not the store's.
        return false;
    }
    let Some(said) = ret_some(sr::_other_person_reply(memory, &low)) else { return false };
    // A miss is not a denial: when the store holds nothing about that person, the honest "you
    // haven't told me" is worse than the answer the knowledge base has. "is my son allergic to
    // nuts? he had a rash" was claimed only to be told nothing was on record, while the entry
    // about a nut-free kitchen had a real answer for it.
    !NO_RECORD.is_match(&said)
}

// ---------------------------------------------------------------------------------------------
// Pets: a name, an age, a name put right.

/// `user(memory).get("pets")` as its items, when it is a list.
fn pets_of_store(memory: &mut Memory) -> Option<Vec<Value>> {
    match vget(user(memory), "pets") {
        Value::List(items) => Some(items.to_vec()),
        _ => None,
    }
}

/// Write one pet record back where the store keeps it (Python mutates the dict in place).
fn put_pet(memory: &mut Memory, at: usize, pet: Value) {
    if let Some(items) = user(memory).get_mut("pets").and_then(Value::items_mut) {
        if at < items.len() {
            items[at] = pet;
        }
    }
}

/// Write one person's record back where the store keeps it.
fn put_other(memory: &mut Memory, at: usize, known: Value) {
    if let Some(items) = others(memory).items_mut() {
        if at < items.len() {
            items[at] = known;
        }
    }
}

/// `len(memory.get("turns") or [])` as a value.
fn turn_no(memory: &Memory) -> Value {
    Value::Int(memory.turns.len() as i64)
}

/// `log(memory).append({"key": key, "value": value, "turn": len(turns), "text": text})`.
fn log_fact(memory: &mut Memory, key: &str, value: Value, text_: &str) {
    let turn = turn_no(memory);
    log(memory).push(dict_of(vec![("key", Value::from(key)), ("value", value), ("turn", turn), ("text", Value::from(text_))]));
}

rx!(CORRECTS_A_NAME, CORRECTS_A_NAME_POSSESSIVE);

/// Python: bot/modules/memory_store.py:1912 `_name_the_last_pet(text, memory)`
///
/// "we've also got a cat" then "and her name is Biscuit" is one cat with a name. A name given a
/// second time is a correction when the line opens like one; only the opener makes it a rename.
pub fn _name_the_last_pet(text_: &str, memory: &mut Memory) {
    let pets = pets_of_store(memory);
    let named = pets.as_ref().and_then(|p| p.last()).map_or(false, |last| last.is_dict() && last.truthy());
    // The lower-case shape is only read when there is a pet to put the name on, which is here.
    let name = match ret_some(extract::pet_name_only(text_, named)).or_else(|| ret_some(sr::_name_they_answered(text_, memory))) {
        Some(name) => name,
        None => return,
    };
    let pets = match pets {
        Some(pets) if !pets.is_empty() => pets,
        _ => {
            // The name can come before the animal is in the store: "im getting a dog" / "what should
            // i name it" / "milo" keeps no dog yet, and the name is the one thing that turn gave.
            // The animal is the one the list or the question was about, so the pet is filed whole.
            let animal = sr::_the_animal_of_the_pick(memory).val();
            if !animal.truthy() || animal.as_str() == Some("pet") {
                return;
            }
            let pet = dict_of(vec![("animal", animal), ("name", Value::from(name))]);
            remember(memory, "pets", &list_of(vec![pet]), text_);
            return;
        }
    };
    let at = pets.len() - 1;
    let mut last = pets[at].clone();
    if last.is_dict() && !has(&last, "name") {
        last.set_key("name", name.as_str());
        let mut kept = dict_of(vec![("animal", last.get("animal").cloned().unwrap_or_else(|| Value::from("pet"))), ("name", Value::from(name.as_str()))]);
        if let Some(count) = int_of(vget(&last, "count")) {
            if count > 1 {
                kept.set_key("count", vget(&last, "count").clone());
            }
        }
        put_pet(memory, at, last);
        remember(memory, "pets", &list_of(vec![kept]), text_);
        return;
    }
    // A correction that names the animal it is about ("no the cat is called tom") is that
    // animal's rename and nobody else's: `_fix_a_pets_name` has already moved the name on the cat,
    // and putting it on the pet said last as well renamed the dog in a house with one of each.
    if last.is_dict()
        && sget(&last, "name").to_lowercase() != name.to_lowercase()
        && (CORRECTS_A_NAME.match_(text_).is_some() || CORRECTS_A_NAME_POSSESSIVE.match_(text_).is_some())
        && !_names_a_pet_here(text_, memory)
    {
        // The one on record is the same animal under the name they have just taken back, so the
        // entry is renamed rather than a second one added: "we have a dog called max / sorry his
        // name is milo" is one dog, not two.
        let before = sget(&last, "name");
        last.set_key("name", name.as_str());
        log_fact(memory, "pet", dict_of(vec![("animal", vget(&last, "animal").clone()), ("name", Value::from(name.as_str()))]), text_);
        // The name on record is kept beside the new one, so the next line about the pet still
        // finds it: "my dog is called rex" then "his names actually rocky" then "how old is
        // he" answered with a memory trick about names, because the dog under Rocky was a
        // different dog from the one under Rex.
        if !before.is_empty() && before.to_lowercase() != name.to_lowercase() {
            let mut was: Vec<String> = py_strs(vget(&last, "was")).into_iter().filter(|old| old.to_lowercase() != name.to_lowercase()).collect();
            was.push(before.clone());
            last.set_key("was", str_list(&was));
        }
        let animal = if has(&last, "animal") { vget(&last, "animal").clone() } else { Value::from("pet") };
        put_pet(memory, at, last);
        // The person is told which name the bot goes with rather than having their own line
        // handed back, and it is the pet's own note: the two-name shape ("Rocky, not Rex — got
        // it.") reads to the next module as a conversation about names rather than about the
        // dog, and "how old is he" then got a tip on remembering names.
        if !before.is_empty() {
            memory.set(
                "correction",
                dict_of(vec![("key", Value::from("pet")), ("value", Value::from(name.as_str())), ("animal", animal), ("said", Value::from(fold(text_)))]),
            );
        }
    }
}

/// `re.search(r"\b" + re.escape(word) + r"s?\b", low)`.
fn says_with_s(word: &str, low: &str) -> bool {
    Re::new(&format!(r"\b{}s?\b", pyre::escape(word)), 0).is_match(low)
}

/// `re.search(r"\b" + re.escape(word) + r"\b", low)`.
fn says_word(word: &str, low: &str) -> bool {
    Re::new(&format!(r"\b{}\b", pyre::escape(word)), 0).is_match(low)
}

/// Python: bot/modules/memory_store.py:1977 `_the_animals_named(text, memory)`
///
/// The animals of the store's pets the line names: "no the cat is called tom" names the cat (a
/// set; here in sorted order, which is the only order the caller reads it in).
pub fn _the_animals_named(text_: &str, memory: &mut Memory) -> Vec<String> {
    let low = fold(text_);
    let mut here: Vec<String> = Vec::new();
    for pet in vget(user(memory), "pets").items() {
        if !pet.is_dict() || !has(pet, "animal") {
            continue;
        }
        let animal = py_str(vget(pet, "animal")).to_lowercase();
        if says_with_s(&animal, &low) && !here.contains(&animal) {
            here.push(animal);
        }
    }
    here.sort();
    here
}

/// Python: bot/modules/memory_store.py:1990 `_the_name_it_had(animal, said, memory)`
///
/// The name the store had for this animal before the line that named it again.
pub fn _the_name_it_had(animal: &str, said: &str, memory: &mut Memory) -> String {
    for entry in log(memory).items().iter().rev() {
        let value = vget(entry, "value");
        if !value.is_dict() {
            continue;
        }
        if sget(value, "animal").to_lowercase() != animal {
            continue;
        }
        let name = sget(value, "name");
        if !name.is_empty() && name.to_lowercase() != said.to_lowercase() {
            return name;
        }
    }
    String::new()
}

/// Python: bot/modules/memory_store.py:2004 `_names_a_pet_here(text, memory)`
///
/// Whether the line names an animal the store keeps a pet of ("the cat", "the dogs").
pub fn _names_a_pet_here(text_: &str, memory: &mut Memory) -> bool {
    let low = fold(text_);
    vget(user(memory), "pets")
        .items()
        .iter()
        .any(|pet| pet.is_dict() && has(pet, "animal") && says_with_s(&py_str(vget(pet, "animal")).to_lowercase(), &low))
}

/// Python: bot/modules/memory_store.py:2035 `_the_figure_said(match)`
///
/// The age a reader's own regex caught, as a number, whether it was digits or a word.
pub fn _the_figure_said(m: &Match) -> Option<i64> {
    let said = text::strip(m.name_str("age")).to_lowercase();
    if said.is_empty() {
        return None;
    }
    let figure = fx("FIGURE_WORDS").get(&said).cloned().unwrap_or_else(|| Value::from(said.as_str()));
    match sr::_a_number(&figure).val() {
        Value::Int(n) => Some(n),
        Value::Bool(b) => Some(b as i64),
        _ => None,
    }
}

/// The figure in a chained comparison (`0 < age <= 30`): comparing None with an int raises.
fn figure(age: Option<i64>) -> i64 {
    age.unwrap_or_else(|| panic!("TypeError: '<' not supported between instances of 'int' and 'NoneType'"))
}

rx!(PET_NAME_AGE, PET_LEAD_IN);

/// Python: bot/modules/memory_store.py:2057 `_without_a_lead_in(text)`
///
/// The line with the lead-ins people open a short sentence with taken off.
pub fn _without_a_lead_in(text_: &str) -> String {
    let mut said = text::strip(text_).to_string();
    for _ in 0..4 {
        let shorter = PET_LEAD_IN.subn("", &said, 1);
        if shorter == said {
            break;
        }
        said = text::strip(&shorter).to_string();
    }
    said
}

/// Python: bot/modules/memory_store.py:2068 `_put_a_pets_age(pet, age, unit, text)`
///
/// Store an age on one animal, or None when it already says exactly that.
pub fn _put_a_pets_age(pet: &mut Value, age: i64, unit: &str, text_: &str) -> Option<Value> {
    if vget(pet, "age") == &Value::Int(age) && sget(pet, "unit") == unit {
        return None;
    }
    pet.set_key("age", age);
    pet.del_key("unit");
    if !unit.is_empty() {
        pet.set_key("unit", unit);
    }
    Some(dict_of(vec![("key", Value::from("pet")), ("value", pet.clone()), ("text", Value::from(text_))]))
}

/// `_put_a_pets_age` on the pet at `at` in the store's list, written back in place.
fn put_a_pets_age_at(memory: &mut Memory, at: usize, age: i64, unit: &str, text_: &str) -> Option<Value> {
    let mut pet = pets_of_store(memory)?.get(at)?.clone();
    let noted = _put_a_pets_age(&mut pet, age, unit, text_);
    put_pet(memory, at, pet);
    noted
}

/// Python: bot/modules/memory_store.py:2097 `_age_the_named_pet(text, memory)`
///
/// "my dog is called fred" then "fred is 5" is an age for the dog, not a fact about nobody. The
/// line names no animal at all, only the name the store has for one.
pub fn _age_the_named_pet(text_: &str, memory: &mut Memory) -> Option<Value> {
    let low = _without_a_lead_in(text_).to_lowercase();
    let m = PET_NAME_AGE.match_(&low)?;
    let age = figure(_the_figure_said(&m));
    if !(0 < age && age <= 30) {
        return None;
    }
    let unit = if m.name_str("unit").to_lowercase().starts_with("month") { "months" } else { "" };
    let said = if !text::strip(&low).is_empty() { text::strip_chars(&text::split(text::strip(&low))[0], ".,!?").to_string() } else { String::new() };
    let pets = pets_of_store(memory)?;
    for (at, pet) in pets.iter().enumerate().rev() {
        if !pet.is_dict() || sget(pet, "name").to_lowercase() != said {
            continue;
        }
        return put_a_pets_age_at(memory, at, age, unit, text_);
    }
    None
}

rx!(_PET_WORD, AGE_FIX);
inline_re!(MONTHS_WORD, r"\bmonths?\b", 0);
static PET_AGE_FIX_WORDS: LazyLock<usize> = LazyLock::new(|| g("PET_AGE_FIX_WORDS").as_int().unwrap() as usize);
static AGE_FIX_WORDS: LazyLock<usize> = LazyLock::new(|| g("AGE_FIX_WORDS").as_int().unwrap() as usize);
static NAME_FIX_WORDS: LazyLock<usize> = LazyLock::new(|| g("NAME_FIX_WORDS").as_int().unwrap() as usize);
static NAME_GIVEN_WORDS: LazyLock<usize> = LazyLock::new(|| g("NAME_GIVEN_WORDS").as_int().unwrap() as usize);

/// Python: bot/modules/memory_store.py:2124 `_fix_a_pets_age(text, memory)`
///
/// "my dog is 5" then "sorry he is 6": the correction goes on the animal it corrects. Only a
/// short line, only with a correction word in it, and only when there is an age stored to
/// replace, because a correction replaces a fact rather than being the first one said.
pub fn _fix_a_pets_age(text_: &str, memory: &mut Memory) -> Option<Value> {
    let said = fold(text_);
    let low = _without_a_lead_in(&said);
    // The correction word is one of the lead-ins, so it has to be read off the line as it was
    // said, before the lead-in comes off.
    if low.is_empty() || text::split(&low).len() > *PET_AGE_FIX_WORDS || !CORRECTING.is_match(&said) {
        return None;
    }
    if _PET_WORD.is_match(&low) {
        return None; // "my dog is 6" names the animal and is a fact of its own
    }
    let m = AGE_FIX.search(&low)?;
    let age = figure(_the_figure_said(&m));
    if !(0 < age && age <= 30) {
        return None;
    }
    let who = _person_from_pronoun(&low, memory, &[]);
    if !who.is_empty() && others(memory).items().iter().any(|known| sget(known, "who") == who && has_key(known, "age")) {
        // "my dog is 5" / "my son is 7" / "actually hes 8": the pronoun points at a person, and
        // a person's corrected age is read by `_fix_a_persons_age`. It is one figure, not both.
        return None;
    }
    let unit = if MONTHS_WORD.is_match(&low) { "months" } else { "" };
    let aged: Vec<usize> = vget(user(memory), "pets")
        .items()
        .iter()
        .enumerate()
        .filter(|(_, pet)| pet.is_dict() && !vget(pet, "age").is_none())
        .map(|(at, _)| at)
        .collect();
    if aged.len() != 1 {
        // Two animals with ages and no animal named: the pronoun is the only thing to go on, and
        // "they are 6" is not a correction of either of them.
        return None;
    }
    put_a_pets_age_at(memory, aged[0], age, unit, text_)
}

rx!(PET_AGE_ANSWER);
inline_re!(A_PRONOUN_WITH_ITS_VERB, r"\b(?:he|she|it|they)(?:'s|s|\s+is|\s+are)?\b", 0);

/// `int(pet.get("count") or 1)`.
fn pet_count(pet: &Value) -> i64 {
    let count = vget(pet, "count");
    if !count.truthy() {
        return 1;
    }
    match count {
        Value::Int(n) => *n,
        Value::Bool(b) => *b as i64,
        Value::Float(f) => f.trunc() as i64,
        Value::Str(s) => py_int(s).unwrap_or_else(|| panic!("ValueError: invalid literal for int() with base 10: {s:?}")),
        other => panic!("TypeError: int() argument must be a string or a number, not {}", py_repr(other)),
    }
}

/// The unit an age answer names: "months", "weeks" or "" (years).
fn age_unit(m: &Match) -> &'static str {
    let unit = m.name_str("unit").to_lowercase();
    if unit.starts_with("month") {
        "months"
    } else if unit.starts_with("week") {
        "weeks"
    } else {
        ""
    }
}

/// Python: bot/modules/memory_store.py:2173 `_age_said_of_the_only_animal(text, memory)`
///
/// "i have a greyhound called sid" then "hes 6": the figure is that animal's age. It is only
/// read when the line before it named an animal or the one the store knows by name, and when the
/// chat holds one animal - with two, "hes 6" says nothing about which one.
pub fn _age_said_of_the_only_animal(text_: &str, memory: &mut Memory) -> Option<Value> {
    let said = fold(text_);
    if said.is_empty() || text::split(&said).len() > 5 {
        return None;
    }
    let previous = fold(&last_user(memory));
    let pets: Vec<(usize, Value)> = vget(user(memory), "pets").items().iter().cloned().enumerate().filter(|(_, pet)| pet.is_dict()).collect();
    let (at, pet) = if pets.len() == 1 {
        pets[0].clone()
    } else {
        // With more than one animal, the one the line before named is the one the pronoun is:
        // "rufus is a terrier" then "hes two" is Rufus's age, and Nell's is not known. Any other
        // line before it leaves the pronoun pointing at nobody in particular.
        let named: Vec<&(usize, Value)> = pets
            .iter()
            .filter(|(_, one)| {
                let name = sget(one, "name");
                let name = text::strip(&name);
                text::len(name) > 2 && says_word(&text::strip(&py_str(vget(one, "name"))).to_lowercase(), &previous)
            })
            .collect();
        if named.len() != 1 {
            return None;
        }
        named[0].clone()
    };
    if pet_count(&pet) > 1 {
        // "i have two cats" is one record for two animals, so a pronoun and a figure do not say
        // which of them it is.
        return None;
    }
    let name = text::strip(&sget(&pet, "name")).to_lowercase();
    let called: Vec<String> = ["animal", "breed"].iter().map(|word| text::strip(&sget(&pet, word)).to_lowercase()).collect();
    if !(_PET_WORD.is_match(&previous)
        || (text::len(&name) > 2 && previous.contains(&name))
        || called.iter().any(|word| text::len(word) > 2 && previous.contains(word.as_str())))
    {
        return None;
    }
    if !A_PRONOUN_WITH_ITS_VERB.is_match(&said) {
        // A figure with no pronoun is someone else's unless the bot asked for it: the line has to
        // say whose age it is.
        return None;
    }
    let m = PET_AGE_ANSWER.match_(&said)?;
    let age = figure(_the_figure_said(&m));
    if !(0 < age && age <= 30) {
        return None;
    }
    if !_person_from_pronoun(&said, memory, &[]).is_empty() {
        // "hes 8" after "my son has a dog" is the son, and a person's age is not the animal's.
        return None;
    }
    put_a_pets_age_at(memory, at, age, age_unit(&m), text_)
}

/// Python: bot/modules/memory_store.py:2228 `_age_they_answered(text, memory)`
///
/// The bot asked how old their animal was, so the figure in this line is that animal's age. A
/// bare age names nobody, so it is read only when the reply before it asked.
pub fn _age_they_answered(text_: &str, memory: &mut Memory) -> Option<Value> {
    let asked = memory.turns.last()?.bot.clone();
    if !asked.to_lowercase().contains("how old") {
        return None;
    }
    let pets: Vec<(usize, Value)> = vget(user(memory), "pets").items().iter().cloned().enumerate().filter(|(_, pet)| pet.is_dict()).collect();
    if pets.is_empty() {
        return None;
    }
    // Only an animal was asked about: a figure given after "how old is your daughter" is not the
    // dog's age, so the question has to name an animal, or a name the store has for one.
    let names: Vec<String> = pets.iter().map(|(_, pet)| text::strip(&sget(pet, "name")).to_lowercase()).collect();
    if !(_PET_WORD.is_match(&asked) || names.iter().any(|name| !name.is_empty() && says_word(name, &asked.to_lowercase()))) {
        return None;
    }
    let m = PET_AGE_ANSWER.match_(text_)?;
    let age = figure(_the_figure_said(&m));
    if !(0 < age && age <= 30) {
        return None;
    }
    let unit = age_unit(&m);
    // `for pet in reversed(pets)`: the loop returns on its first pass either way.
    let (at, mut pet) = pets[pets.len() - 1].clone();
    if vget(&pet, "age") == &Value::Int(age) && sget(&pet, "unit") == unit {
        return None;
    }
    pet.set_key("age", age);
    pet.del_key("unit");
    if !unit.is_empty() {
        pet.set_key("unit", unit);
    }
    put_pet(memory, at, pet.clone());
    // The store took a figure the person gave to the bot's own question, so the reply says it
    // back ("Noted: your cat is 4.") rather than a line that would fit any message at all.
    memory.set("pet_age", pet.clone());
    Some(dict_of(vec![("key", Value::from("pet")), ("value", pet), ("text", Value::from(text_))]))
}

/// Python: bot/modules/memory_store.py:2273 `_their_name_answered(text, memory)`
///
/// The person's own name, given as the answer to the bot's own question. A word on its own says
/// nothing, so it is read as a name only when the reply right before it asked for one, and only
/// when no name is on record yet.
pub fn _their_name_answered(text_: &str, memory: &mut Memory) -> Option<String> {
    let asked = memory.turns.last()?.bot.clone();
    if has(user(memory), "name") {
        return None;
    }
    if mget(memory, "asked_for").as_str() != Some("name") || !fx("USER_NAME_ASKED").as_re().unwrap().is_match(&asked) {
        return None;
    }
    // The hedge comes off first: "its sam by the way" and "sam, btw" are the one word the store
    // waits for with three more around it, and read whole they were nothing at all.
    let answered = ret_str(extract::without_the_hedges(text_));
    if let Some(m) = fx("NAME_ANSWER_ANNOUNCED").as_re().unwrap().match_(&answered) {
        return ret_some(extract::reads_as_a_name(m.str(1)));
    }
    if let Some(last) = fx("NAME_ANSWER_ANNOUNCED_LAST").as_re().unwrap().search(&answered) {
        // "i havent told you, its sam": the name is still the last thing said, after the apology
        // for not having given it, and the store was waiting for exactly this.
        return ret_some(extract::reads_as_a_name(last.str(1)));
    }
    ret_some(extract::reads_as_a_name(&answered))
}

/// Python: bot/modules/memory_store.py:2440 `_fix_a_persons_age(text, memory)`
///
/// Put a corrected age on the person the pronoun points at. The line names no relation, so who
/// it is about comes from the pronoun, and the age it replaces has to be one the store already
/// has.
pub fn _fix_a_persons_age(text_: &str, memory: &mut Memory) -> Option<Value> {
    let low = fold(text_);
    if low.is_empty() || text::split(&low).len() > *AGE_FIX_WORDS || !CORRECTING.is_match(&low) {
        return None;
    }
    let m = AGE_FIX.search(&low)?;
    let age = figure(_the_figure_said(&m));
    if !(0 <= age && age <= 120) {
        return None;
    }
    let who = _person_from_pronoun(&low, memory, &[]);
    if who.is_empty() {
        return None;
    }
    let found = others(memory).items().iter().position(|known| vget(known, "who").as_str() == Some(who.as_str()) && has_key(known, "age"))?;
    let mut known = others(memory).items()[found].clone();
    known.set_key("age", age);
    known.del_key("turns");
    put_other(memory, found, known.clone());
    log_fact(memory, "other", known.clone(), text_);
    Some(known)
}

rx!(WEEKS_FIX, PREGNANCY_SAID_AGAIN);
inline_re!(NOW_OR_TODAY, r"\b(?:now|today)\b", 0);
inline_re!(SAID_MINE, r"\b(?:im|i'm|i am|i)\b", 0);
inline_re!(SAID_THEIRS, r"\b(?:my|our|she|he|they|her|his|their|hers)\b", 0);

/// Python: bot/modules/memory_store.py:2478 `_fix_the_weeks_of_a_pregnancy(text, memory)`
///
/// Move the length the store holds to the one this line gives, for the person it points at.
pub fn _fix_the_weeks_of_a_pregnancy(text_: &str, memory: &mut Memory) -> Option<Value> {
    let low = fold(text_);
    if low.is_empty() || text::split(&low).len() > 8 {
        return None;
    }
    let m = WEEKS_FIX.search(&low)?;
    // A line that states the pregnancy again ("my wife is 12 weeks pregnant") is read by the
    // extractor, which files the new length itself; this is for the line that only moves it.
    if PREGNANCY_SAID_AGAIN.is_match(&low) {
        return None;
    }
    let weeks = py_int(m.name_str("weeks")).unwrap();
    if !(1 <= weeks && weeks <= 45) {
        return None;
    }
    if !(CORRECTING.is_match(&low) || NOW_OR_TODAY.is_match(&low)) {
        return None;
    }
    // The subject of the line decides whose length it moves, exactly as it does for a line that
    // states the pregnancy: "im 14 weeks now" is theirs, "she's 10 weeks now" is hers, and a chat
    // that has both on file must not move the wrong one.
    let said_mine = SAID_MINE.is_match(&low);
    let said_theirs = SAID_THEIRS.is_match(&low);
    let own = user(memory).clone();
    if said_mine && !said_theirs && has(&own, "pregnant") {
        // A line about the person themselves: "im 14 weeks now", "actually im 16 weeks". Their own
        // pregnancy is kept on them, not in `others`, so nothing moved it: the bot agreed ("got it
        // - you're 14 weeks now") while "how far along am i" still answered 12.
        remember(memory, "weeks", &Value::Int(weeks), text_);
        let mut moved = own;
        moved.set_key("weeks", weeks);
        return Some(moved);
    }
    let who = _person_from_pronoun(&low, memory, &[]);
    if who.is_empty() {
        return None;
    }
    let found =
        others(memory).items().iter().position(|known| vget(known, "who").as_str() == Some(who.as_str()) && has(known, "pregnant") && has_key(known, "weeks"))?;
    let mut known = others(memory).items()[found].clone();
    known.set_key("weeks", weeks);
    known.del_key("turns");
    put_other(memory, found, known.clone());
    log_fact(memory, "other", known.clone(), text_);
    Some(known)
}

rx!(NAME_FIX, A_NAME_AFTER_A_PRONOUN, A_NAME_WITH_ITS_VERB, ANOTHER_ANIMAL);
strset!(NAME_NOT_A_NAME);

/// The pet correction note `memory["correction"]` holds.
fn pet_correction(value: &Value, wrong: &str, animal: &str, low: &str, text_: &str) -> Value {
    dict_of(vec![
        ("key", Value::from("pet")),
        ("value", value.clone()),
        ("wrong", Value::from(cap1(wrong))),
        ("animal", Value::from(animal)),
        ("said", Value::from(low)),
        ("text", Value::from(text_)),
    ])
}

/// `isinstance(memory.get("correction"), dict) and memory["correction"].get("said") == low`.
fn correction_said(memory: &Memory, low: &str) -> bool {
    let kept = mget(memory, "correction");
    kept.is_dict() && vget(kept, "said").as_str() == Some(low)
}

/// Python: bot/modules/memory_store.py:2550 `_fix_a_pets_name(text, memory)`
///
/// A pet's name put right in the sentence moves the name on the pet. The pet is the animal the
/// line names, or else the pet the turn before was about.
///
/// Python works on the store's own dicts (identity, in-place writes, `list.remove` by equality);
/// here each record carries its position in the store's list, and every write goes back there.
pub fn _fix_a_pets_name(text_: &str, memory: &mut Memory) -> Option<Value> {
    let low = fold(text_);
    let moves = ret_true(crate::memory::rename::moves(&low));
    if low.is_empty() || text::split(&low).len() > *NAME_FIX_WORDS || !(CORRECTING.is_match(&low) || moves) {
        return None;
    }
    // A correction says the name with its verb ("no hes called max"); a rename says it at the end
    // of the line, with the verb about the animal in front ("we renamed him max").
    let mut raw = String::new();
    if let Some(m) = A_NAME_WITH_ITS_VERB.search(&low) {
        raw = m.name_str("name").to_string();
    } else if moves {
        let held = vget(user(memory), "pets").clone();
        let animal_named = fx("PETS").as_re().unwrap().is_match(&low);
        raw = ret_str(crate::memory::rename::the_name_a_line_moved_on(&low, &held, &Value::Bool(animal_named)));
    }
    if raw.is_empty() {
        return None;
    }
    let said = ret_str(extract::_finish(&raw));
    if said.is_empty() || ret_true(extract::_not_a_name(&raw, false)) {
        return None;
    }
    // The store's list as (identity, record): the identity is the place it had when read.
    let mut store: Vec<(usize, Value)> = vget(user(memory), "pets").items().iter().cloned().enumerate().collect();
    let dict_ids: Vec<usize> = store.iter().filter(|(_, pet)| pet.is_dict()).map(|(id, _)| *id).collect();
    // The line that names the animal it is about ("no the cat is called tom") has already been
    // carried out by the facts reader: the store keeps one pet an animal, so filing "a cat called
    // tom" replaced the cat called milo. Nothing is left to move, and going on to look for a pet
    // anyway found the pet said last - the dog, in a house with one of each - and renamed that.
    if _names_a_pet_here(text_, memory) {
        if _another_animal_is_named(text_) {
            // "no my other cat is called max", "no we also have a cat called max": the word says
            // the name belongs beside the pet already there, so nothing is put right.
            return None;
        }
        let write_back = |memory: &mut Memory, store: &Vec<(usize, Value)>| {
            user(memory).set_key("pets", list_of(store.iter().map(|(_, pet)| pet.clone()).collect()));
        };
        for animal in _the_animals_named(text_, memory) {
            let kept: Vec<usize> =
                dict_ids.iter().copied().filter(|id| text::strip(&sget(&store.iter().find(|(i, _)| i == id).unwrap().1, "animal")).to_lowercase() == animal).collect();
            if kept.is_empty() {
                continue;
            }
            let record = |store: &Vec<(usize, Value)>, id: usize| store.iter().find(|(i, _)| *i == id).unwrap().1.clone();
            let filed: Vec<usize> = kept.iter().copied().filter(|id| _a_pet_new_this_line(&record(&store, *id), memory)).collect();
            let older: Vec<usize> = kept.iter().copied().filter(|id| !_a_pet_new_this_line(&record(&store, *id), memory)).collect();
            if let Some(&target) = older.last() {
                // The line that names the animal files the name it says ("no the cat is called
                // max" filed a cat called max), so that entry is the line's own echo. In a house
                // of one the facts reader files it over the cat already there and nothing is
                // left to move; in a house of two the cat it was filed beside is still there and
                // the name belongs on that one - the cat said last. Either way the entry the
                // line added is not a pet of its own.
                let at = store.iter().position(|(i, _)| *i == target).unwrap();
                if sget(&store[at].1, "name").to_lowercase() != said.to_lowercase() {
                    store[at].1.set_key("name", cap1(&said));
                    store[at].1.del_key("turns");
                    write_back(memory, &store);
                    let was = _the_name_it_had(&animal, &said, memory);
                    if !was.is_empty() && !correction_said(memory, &low) {
                        let value = vget(&store[at].1, "name").clone();
                        memory.set("correction", pet_correction(&value, &was, &animal, &low, text_));
                    }
                }
                for id in filed {
                    // `pet is not target and pet in pets`, then `pets.remove(pet)`: the first
                    // record equal to it goes.
                    let pet = record(&store, id);
                    if let Some(first) = store.iter().position(|(_, one)| *one == pet) {
                        store.remove(first);
                    }
                }
                write_back(memory, &store);
                return None;
            }
            for id in filed {
                // A pet given its first name: the facts reader filed it as the line said it
                // ("max"), and a name on a pet is read back the way a name is written.
                let at = store.iter().position(|(i, _)| *i == id).unwrap();
                store[at].1.set_key("name", cap1(&said));
                write_back(memory, &store);
                let was = _the_name_it_had(&animal, &said, memory);
                if !was.is_empty() && !correction_said(memory, &low) {
                    let value = vget(&store[at].1, "name").clone();
                    memory.set("correction", pet_correction(&value, &was, &animal, &low, text_));
                }
            }
            return None;
        }
    }
    if _another_animal_is_named(text_) {
        // "no my other cat is called max" names a second cat, not a name taken back: the store
        // keeps one pet an animal, so the facts reader files the new one and this line adds it.
        return None;
    }
    // `pets`: the records of the store's list (the dicts), by their place in this list from here.
    let mut pets: Vec<Value> = dict_ids.iter().map(|id| store.iter().find(|(i, _)| i == id).unwrap().1.clone()).collect();
    let named: Vec<usize> = (0..pets.len()).filter(|at| has(&pets[*at], "name")).collect();
    // The line that renames also names the animal ("no the cat is called tom"), so the facts
    // reader has already filed a second cat of the new name: that entry is this line's echo, not
    // a pet of its own, and counting it would leave two cats matching the word "cat" and send the
    // correction on to the pet said last - the dog, in a house with one of each.
    let here: Vec<usize> = (0..pets.len())
        .filter(|at| has(&pets[*at], "animal") && says_with_s(&py_str(vget(&pets[*at], "animal")).to_lowercase(), &low))
        .filter(|at| sget(&pets[*at], "name").to_lowercase() != said.to_lowercase())
        .collect();
    let mut target: Option<usize> = None;
    if here.len() == 1 {
        target = Some(here[0]);
    } else {
        for key in ["name", "animal"] {
            let found: Vec<usize> = named.iter().copied().filter(|at| _the_turn_before_said(vget(&pets[*at], key), memory)).collect();
            if let Some(&last) = found.last() {
                // The line says "he", and in "i have two cats called milo and tom" the he is
                // the one said last, so the pet named last in the same line is the one the
                // correction is about. A house of one pet is unaffected.
                target = Some(last);
                break;
            }
        }
    }
    let target = target?;
    let before = sget(&pets[target], "name");
    if before.to_lowercase() == said.to_lowercase() {
        // The name said again is the name already on file: nothing was put right.
        return None;
    }
    pets[target].set_key("name", cap1(&said));
    pets[target].del_key("turns");
    // The in-place write shows in the store before anything below reads it.
    put_pet(memory, dict_ids[target], pets[target].clone());
    // The correction is the turn's news, said with the store's own correction note ("Max, not
    // Milo — got it."), unless the line already left one of its own.
    if !correction_said(memory, &low) {
        let value = vget(&pets[target], "name").clone();
        let animal = sget(&pets[target], "animal");
        memory.set("correction", pet_correction(&value, &before, if animal.is_empty() { "pet" } else { &animal }, &low, text_));
    }
    // The line that renames also names a pet ("no, my cat is called max"), so the facts reader
    // has just filed a second cat of that name beside the one being renamed. One cat a person
    // names once: the renamed entry is the whole of what that line says.
    let animal = sget(&pets[target], "animal").to_lowercase();
    let mut live: Vec<usize> = (0..pets.len()).collect();
    for other in 0..pets.len() {
        if other == target {
            continue;
        }
        if sget(&pets[other], "animal").to_lowercase() == animal && sget(&pets[other], "name").to_lowercase() == said.to_lowercase() {
            // `pets.remove(other)`: the first record equal to it goes.
            if let Some(first) = live.iter().position(|at| pets[*at] == pets[other]) {
                live.remove(first);
            }
        }
    }
    user(memory).set_key("pets", list_of(live.iter().map(|at| pets[*at].clone()).collect()));
    // The store keeps every pet it had: a rename is one animal under a new name, not the whole
    // household. Writing the renamed pet back on its own dropped the others - "i have two cats
    // called milo and tom" then "no hes called max" left one cat, and the count answer was
    // "You've got one cat" against two lines of what the person told us.
    log_fact(memory, "pet", pets[target].clone(), text_);
    Some(pets[target].clone())
}

/// Python: bot/modules/memory_store.py:2692 `_pets_of(animal, pets)`
///
/// The pets of one animal in the store's list, in the order the store keeps them.
pub fn _pets_of(animal: &str, pets: &[Value]) -> Vec<Value> {
    pets.iter().filter(|pet| pet.is_dict() && text::strip(&sget(pet, "animal")).to_lowercase() == animal).cloned().collect()
}

/// Python: bot/modules/memory_store.py:2698 `_a_pet_new_this_line(pet, memory)`
///
/// Whether this pet entry was filed by the line being read. The store before the line was read
/// is carried in the memory for the length of the turn (`_pets_before`).
pub fn _a_pet_new_this_line(pet: &Value, memory: &Memory) -> bool {
    let held = mget(memory, "_pets_before");
    if !matches!(held, Value::Set(_)) || !pet.is_dict() {
        return false;
    }
    let pair = Value::Tuple(Arc::new(vec![
        Value::from(text::strip(&sget(pet, "animal")).to_lowercase()),
        Value::from(text::strip(&sget(pet, "name")).to_lowercase()),
    ]));
    !held.items().contains(&pair)
}

/// Python: bot/modules/memory_store.py:2712 `_another_animal_is_named(text)`
///
/// Whether the line says the name belongs to another animal, not the one on record.
pub fn _another_animal_is_named(text_: &str) -> bool {
    ANOTHER_ANIMAL.is_match(&fold(text_))
}

/// Python: bot/modules/memory_store.py:2717 `_two_of_the_animal_are_kept(pets, animal)`
///
/// Whether the store holds more than one pet of this animal.
pub fn _two_of_the_animal_are_kept(pets: &[Value], animal: &str) -> bool {
    pets.iter().filter(|pet| pet.is_dict() && sget(pet, "animal").to_lowercase() == animal).count() > 1
}

/// Python: bot/modules/memory_store.py:2729 `_the_turn_before_said(word, memory)`
///
/// Whether the line before this one said this word about the pet in question.
pub fn _the_turn_before_said(word: &Value, memory: &Memory) -> bool {
    if !word.truthy() || memory.turns.is_empty() {
        return false;
    }
    let said = fold(&memory.turns[memory.turns.len() - 1].user);
    if said.is_empty() {
        return false;
    }
    says_with_s(&py_str(word).to_lowercase(), &said)
}

rx!(CORRECTS_A_MAKE);

/// Python: bot/modules/memory_store.py:2750 `_fix_the_make_of_a_thing(text, memory)`
///
/// A correction that names only the make goes on the last thing the store was told about.
pub fn _fix_the_make_of_a_thing(text_: &str, memory: &mut Memory) {
    let line = text::squash(text::strip(text_));
    if line.is_empty() || line.ends_with('?') || text::split(&line).len() > 6 {
        return;
    }
    let lowered = line.to_lowercase();
    let Some(m) = CORRECTS_A_MAKE.search(&lowered) else { return };
    let held = vget(user(memory), "owns").clone();
    if !held.is_dict() || !held.truthy() {
        return;
    }
    // Which thing the correction is about is decided by what the make is a make of, not by the
    // order things were said: after "my car is a ford fiesta" and "my phone is a pixel", "sorry
    // it's a focus" is the car, and putting it on the phone invents a phone nobody has. A make
    // that fits more than one thing ("sorry, it's a bosch") goes on the one mentioned last.
    let mut fits: Vec<(String, Value)> = Vec::new();
    for (thing, _) in held.pairs() {
        let name = extract::_the_model(&Value::from(m.name_str("value")), thing.str()).val();
        if name.truthy() {
            fits.push((thing.str().to_string(), name));
        }
    }
    if fits.is_empty() {
        return;
    }
    let mut last_said: Vec<Value> = Vec::new();
    for entry in log(memory).items() {
        if vget(entry, "key").as_str() == Some("owns") {
            last_said.extend(py_list(vget(entry, "value")));
        }
    }
    let thing = last_said
        .iter()
        .rev()
        .filter_map(|one| one.as_str())
        .find(|one| fits.iter().any(|(t, _)| t == one))
        .map(str::to_string)
        .unwrap_or_else(|| fits[0].0.clone());
    let name = fits.iter().find(|(t, _)| *t == thing).map(|(_, n)| n.clone()).unwrap_or_else(|| Value::from(""));
    if !name.truthy() || Some(&name) == held.get(&thing) {
        return;
    }
    remember(memory, "owns", &dict_of(vec![(thing.as_str(), name)]), text_);
}

inline_re!(HIS_NAME, r"\bhis\b(\s+name)", 0);
inline_re!(HER_NAME, r"\bher\b(\s+name)", 0);

/// Python: bot/modules/memory_store.py:2787 `_fix_a_persons_name(text, memory)`
///
/// Put a corrected name on the person the pronoun points at. The line names no relation, so who
/// it is about comes from the pronoun, and the name it replaces has to be one the store already
/// has: a correction replaces a fact, it is not the first one said.
pub fn _fix_a_persons_name(text_: &str, memory: &mut Memory) -> Value {
    let low = fold(text_);
    if low.is_empty() || text::split(&low).len() > *NAME_FIX_WORDS || !CORRECTING.is_match(&low) {
        return Value::None;
    }
    let mut m = NAME_FIX.search(&low);
    let mut bare = false;
    if m.is_none() {
        // A bare rename: "no sorry, she is rosie". Both halves are needed - the correcting word
        // in the line, which the check above has already asked for, and a word that is not a
        // word everybody uses.
        let Some(found) = A_NAME_AFTER_A_PRONOUN.search(&low) else { return Value::None };
        let word = text::strip_chars(found.name_str("name"), "'\u{2019}");
        if fx("COMMON_WORDS").contains_str(word) || fx("JOB_WORDS").contains_str(word) || fx("KIN").contains_str(word) || ret_true(extract::_not_a_name(word, false)) {
            return Value::None;
        }
        bare = true;
        m = Some(found);
    }
    let m = m.unwrap();
    let said = text::strip_chars(m.name_str("name"), "'\u{2019}");
    if said.is_empty() || NAME_NOT_A_NAME.contains(said) || ret_true(extract::_not_a_name(said, false)) {
        return Value::None;
    }
    let name = ret_str(extract::_tidy(&ret_str(extract::_finish(said))));
    if name.is_empty() {
        return Value::None;
    }
    // "her name is maggie" points the same way "she's called maggie" does, and the pronoun has to
    // be the one `_person_from_pronoun` reads, so the possessive is turned into it here.
    let about = HIS_NAME.sub(r"he\1", &low);
    let about = HER_NAME.sub(r"she\1", &about);
    let mut who = _person_from_pronoun(&about, memory, &[]);
    if who.is_empty() && bare {
        // "my daughter is called amy" then "no sorry, he is jake": a bare "he is jake" names no
        // relation, so the pronoun is all the line gives, and a "he" about a daughter is not a
        // correction of her but the person starting on somebody else. Renaming her Jake is a
        // wrong fact, worse than the store keeping Amy. The possessive shape below keeps the
        // fallback, because "sorry, her name is tomas" says which fact is being put right.
        return Value::None;
    }
    if who.is_empty() {
        // "my brother is called tom" then "sorry, her name is tomas": the pronoun they picked is
        // the one they type, and with one person named in the chat the correction is about that
        // person - reading the "her" literally left the store on the wrong name and the next ask
        // said the name they had just put right.
        let named: Vec<&Value> = others(memory).items().iter().filter(|entry| has(entry, "name")).collect();
        if named.len() != 1 {
            return Value::None;
        }
        who = sget(named[0], "who");
    }
    let tidy = ret_str(extract::_tidy(&name));
    let Some(found) = others(memory).items().iter().position(|known| vget(known, "who").as_str() == Some(who.as_str()) && has(known, "name")) else {
        return Value::None;
    };
    let mut known = others(memory).items()[found].clone();
    let before = sget(&known, "name");
    if bare && before.to_lowercase() == tidy.to_lowercase() {
        // The word is the name already on record, so there is nothing to put right.
        return Value::None;
    }
    known.set_key("name", tidy.as_str());
    known.del_key("turns");
    put_other(memory, found, known.clone());
    log_fact(memory, "other", known.clone(), text_);
    dict_of(vec![("who", Value::from(who)), ("name", Value::from(py_str(vget(&known, "name")))), ("from", Value::from(before))])
}

rx!(NAME_GIVEN);
strset!(NOT_A_NAME_AFTER_CALLED);
inline_re!(SHE_OR_HER, r"\b(?:she|her)\b", 0);
inline_re!(HE_OR_HIS, r"\b(?:he|his)\b", 0);

/// `_pronoun_fits(who, fits)` against one of `extract.SHE` / `extract.HE`.
fn _pronoun_fits(who: &str, fits: &str) -> bool {
    ret_true(relations::_pronoun_fits(who, &fx(fits).strs()))
}

/// Python: bot/modules/memory_store.py:2870 `_the_name_they_gave_to_the_person_they_mean(text, memory)`
///
/// Put a name given with a pronoun on the person the store already holds. A name the store
/// already holds is replaced. A person it holds and never named gets the name as their first one,
/// and only when exactly one unnamed person fits the pronoun.
pub fn _the_name_they_gave_to_the_person_they_mean(text_: &str, memory: &mut Memory) -> Option<Value> {
    let low = fold(text_);
    if low.is_empty() || text::split(&low).len() > *NAME_GIVEN_WORDS || low.contains('?') {
        // "hes called jake" on its own is a name for the person already meant, but the same words
        // inside a longer line introduce somebody ("my nephew? hes called tom"), and that is a
        // fact about the nephew that the relation readers handle.
        return None;
    }
    let m = NAME_GIVEN.search(&low)?;
    let raw = m.name("name").filter(|s| !s.is_empty()).or_else(|| m.name("name2").filter(|s| !s.is_empty())).unwrap_or("");
    let name = text::strip_chars(raw, " .,!");
    if name.is_empty() || !ret_true(extract::_may_be_a_name(name)) || NOT_A_NAME_AFTER_CALLED.contains(&name.to_lowercase()) {
        return None;
    }
    if !text::strip_chars(&low[m.end()..], " .,!;:").is_empty() {
        // Words after the name mean the line is not only the name: "hes called in sick today".
        return None;
    }
    let who = _person_from_pronoun(&low, memory, &[]);
    if who.is_empty() {
        return None;
    }
    let tidy = extract::_tidy(name).val();
    let count = others(memory).len();
    for at in 0..count {
        let mut known = others(memory).items()[at].clone();
        if vget(&known, "who").as_str() != Some(who.as_str()) || !has(&known, "name") {
            continue;
        }
        if py_str(vget(&known, "name")).to_lowercase() == name {
            return None;
        }
        known.set_key("name", tidy.clone());
        known.del_key("turns");
        put_other(memory, at, known.clone());
        log_fact(memory, "other", known.clone(), text_);
        return Some(known);
    }
    // A first name for a person the chat was told about and never named: "im taking my daughter to
    // london for her birthday" then "her name is maggie". The reader above only replaces a name the
    // store already holds, so the reply said "Her name is maggie - I'll remember that" and filed
    // nothing, and eight turns later the same chat answered "You haven't told me your daughter's
    // name." One person on record has to be the one the pronoun points at, unnamed, and no second
    // unnamed person may fit it: with two daughters in the chat "her name is maggie" does not say
    // which of them, and a guess is worse than the honest "you haven't told me".
    if fx("PETS").as_re().unwrap().is_match(&low) {
        // The line names an animal, so the name is that animal's: "her name is biscuit" after "we've
        // got a cat" belongs to the cat, and `_name_the_last_pet` has it.
        return None;
    }
    let fits: Option<&str> = if SHE_OR_HER.is_match(&low) {
        Some("SHE")
    } else if HE_OR_HIS.is_match(&low) {
        Some("HE")
    } else {
        None // "theyre called" says no gender, so any unnamed person is a candidate
    };
    let mut unnamed: Vec<String> = others(memory).items().iter().filter(|one| has(one, "who") && !has(one, "name")).map(|one| py_str(vget(one, "who"))).collect();
    // A person the chat named only by their relation ("my daughter") is in the list of people and
    // has no entry of its own until a fact is filed about them, so the word itself is a candidate.
    let kept: HashSet<String> = others(memory).items().iter().map(|one| sget(one, "who")).collect();
    for word in vget(user(memory), "people").items() {
        if let Some(word) = word.as_str() {
            if !kept.contains(&word.to_lowercase()) {
                unnamed.push(word.to_lowercase());
            }
        }
    }
    let mut seen: Vec<String> = Vec::new();
    for word in unnamed {
        if !seen.contains(&word) {
            seen.push(word);
        }
    }
    let unnamed: Vec<String> = seen.into_iter().filter(|word| fits.map_or(true, |fits| _pronoun_fits(word, fits))).collect();
    if unnamed.len() != 1 {
        return None;
    }
    for at in 0..others(memory).len() {
        let mut known = others(memory).items()[at].clone();
        if sget(&known, "who") == unnamed[0] && !has(&known, "name") {
            known.set_key("name", tidy.clone());
            known.del_key("turns");
            put_other(memory, at, known.clone());
            log_fact(memory, "other", known.clone(), text_);
            return Some(known);
        }
    }
    let entry = dict_of(vec![("who", Value::from(unnamed[0].as_str())), ("name", tidy.clone())]);
    remember_other(memory, &entry, text_);
    Some(entry)
}

/// Python: bot/modules/memory_store.py:3111 `_a_person(value)`
///
/// True when the corrected value names a person ("sorry, i meant my wife").
pub fn _a_person(value: &Value) -> bool {
    let lowered = s_or(Some(value)).to_lowercase();
    let words: HashSet<String> = LOWER_WORDS.findall_str(&lowered).iter().map(|word| word.trim_end_matches('s').to_string()).collect();
    fx("RELATIONS").items().iter().any(|relation| words.contains(relation.str().trim_end_matches('s')))
}

val!(PAIR_WITHOUT_A_MARKER);
strset!(NOT_A_NEW_VALUE);

/// Python: bot/modules/memory_store.py:3153 `_a_correction_without_a_marker(text, memory)`
///
/// The {to, from} of a pair correction said with no "no"/"sorry" in front, or None. Only a value
/// the store already holds can be taken back this way.
pub fn _a_correction_without_a_marker(text_: &str, memory: &mut Memory) -> Value {
    if text::split(text_).len() > 6 {
        return Value::None;
    }
    for shape in PAIR_WITHOUT_A_MARKER.items() {
        let Some(found) = shape.as_re().unwrap().match_(text_) else { continue };
        let right = ret_str(extract::_tidy(found.name_str("to")));
        let wrong = ret_str(extract::_tidy(found.name_str("wrong")));
        if right.is_empty() || wrong.is_empty() || right.to_lowercase() == wrong.to_lowercase() {
            continue;
        }
        if NOT_A_NEW_VALUE.contains(&right.to_lowercase()) || NOT_A_NEW_VALUE.contains(&wrong.to_lowercase()) {
            continue;
        }
        if !ret_true(crate::memory::corrected::_fact_naming(memory, &Value::from(wrong.as_str()))) && !_a_person_called(memory, &wrong) {
            continue; // nothing on record says that, so this is not a correction of it
        }
        return dict_of(vec![("to", Value::from(right)), ("from", Value::from(wrong))]);
    }
    Value::None
}

/// Python: bot/modules/memory_store.py:3181 `_a_person_called(memory, name)`
///
/// Whether the store holds a person under that name, or under a name they had before.
pub fn _a_person_called(memory: &mut Memory, name: &str) -> bool {
    let low = fold(name);
    if low.is_empty() {
        return false;
    }
    for entry in others(memory).items() {
        if !entry.is_dict() {
            continue;
        }
        if text::strip(&sget(entry, "name")).to_lowercase() == low {
            return true;
        }
        if vget(entry, "was").items().iter().any(|old| text::strip(&py_str(old)).to_lowercase() == low) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/memory_store.py:3221 `_stated_value(memory, key)`
///
/// The value of `key` the line before this one stated, or None.
pub fn _stated_value(memory: &Memory, key: &str) -> Value {
    let said = fold(&last_user(memory));
    let entries = mget(memory, "facts_log");
    if said.is_empty() || !entries.is_list() {
        return Value::None;
    }
    for entry in entries.items().iter().rev() {
        if !entry.is_dict() {
            continue;
        }
        if fold(&sget(entry, "text")) != said {
            return Value::None;
        }
        if vget(entry, "key").as_str() == Some(key) {
            return vget(entry, "value").clone();
        }
    }
    Value::None
}

rx!(TOO_AS_WELL);

/// Python: bot/modules/memory_store.py:3262 `_a_second_persons_limit(text, memory)`
///
/// The limit the line before gave, carried onto the person this line names. Only the limit is
/// carried, and only when the line before stated one.
pub fn _a_second_persons_limit(text_: &str, memory: &mut Memory) -> Value {
    let low = fold(text_);
    let Some(m) = TOO_AS_WELL.match_(&low) else { return Value::None };
    let who = ret_str(relations::_canon_relation(&ret_str(extract::a_person_word(m.name_str("who")))));
    if !fx("_A_WORD_FOR_A_PERSON").contains_str(&who) {
        return Value::None; // "my budget is too" is not a person with an allergy
    }
    let theirs = sr::_the_limit_the_last_line_gave(memory).val();
    if !theirs.truthy() {
        return Value::None;
    }
    let mut giving = dict_of(vec![("who", Value::from(who))]);
    let allergies = vget(&theirs, "allergies");
    if matches!(allergies, Value::List(_) | Value::Tuple(_)) {
        let foods: Vec<String> = allergies.items().iter().map(py_str).filter(|one| !text::strip(one).is_empty()).collect();
        if !foods.is_empty() {
            giving.set_key("allergies", str_list(&foods));
        }
    }
    if let Some(diet) = vget(&theirs, "diet").as_str() {
        if !text::strip(diet).is_empty() {
            giving.set_key("diet", diet);
        }
    }
    if giving.len() < 2 {
        return Value::None;
    }
    giving
}

// ---------------------------------------------------------------------------------------------
// A value taken back.

strs!(DROP_LISTS);
strset!(MONTHS);

/// Python: bot/modules/memory_store.py:3476 `_drop_value(memory, value, text='')`
///
/// 'Sorry, not peanuts' takes that one back: it comes off the list of things they named. Only a
/// value on the list can go, so a "not" about anything else leaves memory alone.
pub fn _drop_value(memory: &mut Memory, value: &Value, text_: &str) -> Value {
    user(memory);
    let low = fold(&s_or(Some(value)));
    if low.is_empty() {
        return Value::None;
    }
    for key in DROP_LISTS.iter() {
        let held = vget(user(memory), key).clone();
        if !held.is_list() {
            continue;
        }
        let left: Vec<Value> = held.items().iter().filter(|v| v.as_str().map_or(false, |v| !_names_this(v, &low))).cloned().collect();
        if left.len() == held.len() {
            continue;
        }
        user(memory).set_key(key.as_str(), list_of(left));
        let correction = dict_of(vec![
            ("key", Value::from(key.as_str())),
            ("value", Value::from(py_str(value))),
            ("dropped", Value::Bool(true)),
            ("said", Value::from(fold(text_))),
        ]);
        memory.set("correction", correction.clone());
        return correction;
    }
    _drop_from_plan(memory, value, text_)
}

inline_re!(PLAN_WORDS, r"[a-z0-9']+", 0);

/// Python: bot/modules/memory_store.py:3501 `_plan_names(plan, low)`
///
/// Does the plan say this value? A plan is kept as the sentence it arrived in, so the thing
/// taken back is a word inside it ("Lisbon" in "Lisbon in may") and not an item on a list.
pub fn _plan_names(plan: &str, low: &str) -> bool {
    let lowered = plan.to_lowercase();
    for word in PLAN_WORDS.findall_str(&lowered) {
        if word == low || (low.ends_with('s') && word == sl(low, 0, Some(-1))) || (word.ends_with('s') && low == sl(&word, 0, Some(-1))) {
            return true;
        }
    }
    false
}

/// Python: bot/modules/memory_store.py:3512 `_drop_from_plan(memory, value, text='')`
///
/// 'Sorry, not Lisbon' after 'im going to Lisbon in May' takes the trip back with the place. Only
/// a plan that names the value goes, and the place goes with it. A city they also stated on its
/// own ("i live in lisbon") stays.
pub fn _drop_from_plan(memory: &mut Memory, value: &Value, text_: &str) -> Value {
    let known = user(memory).clone();
    let low = fold(&s_or(Some(value)));
    if low.is_empty() {
        return Value::None;
    }
    // A month or a day inside a plan says when it happens, not what it is: "my exam is in june" /
    // "sorry, not june" took the exam with it, so the next "when is my exam" had nothing to answer
    // from. A date they want moved is a correction ("actually june not may"), and that path already
    // rewrites the plan; a take-back of a date leaves the plan where it was.
    if MONTHS.contains(&low) || fx("DAYS").contains_str(&low) {
        return Value::None;
    }
    let plans = vget(&known, "plans");
    let mut held: Vec<String> = if plans.is_list() { plans.items().iter().filter_map(|plan| plan.as_str().map(str::to_string)).collect() } else { Vec::new() };
    if let Some(current) = vget(&known, "plan").as_str() {
        if !held.iter().any(|plan| plan == current) {
            held.push(current.to_string());
        }
    }
    let left: Vec<String> = held.iter().filter(|plan| !_plan_names(plan, &low)).cloned().collect();
    if left.len() == held.len() {
        return Value::None;
    }
    let known = user(memory);
    if !left.is_empty() {
        known.set_key("plans", str_list(&left[left.len().saturating_sub(6)..]));
        known.set_key("plan", left[left.len() - 1].as_str());
    } else {
        known.del_key("plans");
        known.del_key("plan");
    }
    let home = vget(known, "city").as_str().map_or(false, |city| _plan_names(city, &low));
    let places = vget(known, "places").clone();
    if places.is_list() && !home {
        let left_places: Vec<Value> = places.items().iter().filter(|place| !place.as_str().map_or(false, |place| _plan_names(place, &low))).cloned().collect();
        if left_places.len() != places.len() {
            if !left_places.is_empty() {
                known.set_key("places", list_of(left_places));
            } else {
                known.del_key("places");
            }
        }
    }
    known.set_key("plan_dropped", low.as_str());
    let correction =
        dict_of(vec![("key", Value::from("plan")), ("value", Value::from(py_str(value))), ("dropped", Value::Bool(true)), ("said", Value::from(fold(text_)))]);
    memory.set("correction", correction.clone());
    correction
}

/// Python: bot/modules/memory_store.py:3563 `_names_this(held, low)`
///
/// Does the value the person named name the thing on the list? Only the word itself and its
/// singular or plural: "peanut" is "peanuts", and "nuts" is not something else that went.
pub fn _names_this(held: &str, low: &str) -> bool {
    let held = fold(held);
    if held == low {
        return true;
    }
    for (a, b) in [(held.as_str(), low), (low, held.as_str())] {
        if b.ends_with('s') && a == sl(b, 0, Some(-1)) {
            return true;
        }
    }
    false
}

inline_re!(NOT_A_FIGURE_CHAR, r"[^\d.,]", 0);
inline_re!(A_SIGNED_AMOUNT, "([\u{a3}$\u{20ac}])\\s?([\\d][\\d,.]*)", 0);

/// Python: bot/modules/memory_store.py:3679 `_budget_sign(text, budget)`
///
/// The currency sign before the same amount in the line they typed, or "".
pub fn _budget_sign(text_: &str, budget: &str) -> String {
    let number = NOT_A_FIGURE_CHAR.sub("", budget);
    if number.is_empty() {
        return String::new();
    }
    for m in A_SIGNED_AMOUNT.finditer(text_) {
        if NOT_A_FIGURE_CHAR.sub("", m.str(2)) == number {
            return m.str(1).to_string();
        }
    }
    String::new()
}

/// Python: bot/modules/memory_store.py:3786 `_a_feeling_about_their_own_news(text, memory)`
///
/// This line is a feeling about the thing they just said, and the store answers it. It is one of
/// the lines the arbiter's single hook asks about, not a hook of its own.
pub fn _a_feeling_about_their_own_news(text_: &str, memory: &Memory) -> bool {
    // a reading that fails just leaves the line to the others
    py_try(|| !sr::_a_feeling_about_what_they_said(memory, &_bare(&_folded(text_, memory))).val().is_none()).unwrap_or(false)
}

/// Python: bot/modules/memory_store.py:3810 `takes_as_it_comes(text, memory=None)`
///
/// True when the line is an ask for the bot's own last answer, so the store answers it.
pub fn takes_as_it_comes(text_: &str, memory: &Memory) -> bool {
    ret_true(sr::_repeat_ask(text_, &_folded(text_, memory), memory))
}

/// Python: bot/modules/memory_store.py:3828 `_the_ask_about_how_it_went(text, memory)`
///
/// This line asks how the thing they told us about went, and only they can say
/// (bot/memory/thread.went).
pub fn _the_ask_about_how_it_went(text_: &str, memory: &Memory) -> bool {
    // a reading that fails leaves the line to the others
    py_try(|| !crate::memory::thread::went(text_, memory).val().is_none()).unwrap_or(false)
}

/// Python: bot/modules/memory_store.py:3847 `_a_count_from_the_date(text, memory)`
///
/// Whether the ask is a count from a date already on file: "how many weeks till that".
pub fn _a_count_from_the_date(text_: &str, memory: &Memory) -> bool {
    // a reading that fails claims nothing
    py_try(|| ret_true(crate::memory::when::until_answer(text_, memory))).unwrap_or(false)
}

// ---------------------------------------------------------------------------------------------
// A figure the answer on the screen named.

rx!(FIGURE_ASK, ACTION_DURATION, ACTION_AMOUNT, A_MONEY_FIGURE, A_PERCENT_FIGURE, LISTED_FIGURE, A_RATE_ASK, A_RATE_STATED);
strset!(FIGURE_FILLER, FIGURE_AFTER, ANSWER_MODULES);
strs!(FIGURE_TIME, FIGURE_MONEY);
/// `FIGURE_UNITS`: each unit family and its words, in the order the dict keeps them.
static FIGURE_UNITS: LazyLock<Vec<(String, Vec<String>)>> =
    LazyLock::new(|| g("FIGURE_UNITS").pairs().iter().map(|(k, v)| (k.str().to_string(), v.strs())).collect());

/// `FIGURE_UNITS[family]` (a KeyError for a family the table does not have).
fn figure_words(family: &str) -> &'static [String] {
    FIGURE_UNITS.iter().find(|(name, _)| name == family).map(|(_, words)| words.as_slice()).unwrap_or_else(|| panic!("KeyError: {family:?}"))
}

/// The words of these families, longest first, escaped for an alternation.
fn unit_alternation(units: &[String]) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for unit in units {
        for word in figure_words(unit) {
            if !words.contains(word) {
                words.push(word.clone());
            }
        }
    }
    // `sorted(set, key=len, reverse=True)`: longest first; words of one length never match at
    // the same place, so their order among themselves does not change what the pattern reads.
    words.sort_by(|a, b| text::len(b).cmp(&text::len(a)).then(a.cmp(b)));
    words.iter().map(|word| pyre::escape(word)).collect()
}

static PER_WEEK: LazyLock<Re> = LazyLock::new(|| Re::new(&format!(r"^\W*(?:a|an|per|each)\s+(?:{})\b", unit_alternation(&FIGURE_TIME).join("|")), pyre::I));

/// Python: bot/modules/memory_store.py:3888 `_the_unit_asked(line)`
///
/// The unit families this line asks a figure in, or None when it names a thing of its own.
pub fn _the_unit_asked(line: &str) -> Option<Vec<String>> {
    let folded = fold(line);
    let found = FIGURE_ASK.match_(&folded)?;
    let mut units: Vec<String> = Vec::new();
    for word in LOWER_WORDS.findall_str(found.name_str("rest")) {
        if FIGURE_FILLER.contains(&word) {
            continue;
        }
        // a thing, not a unit: the thing has an answer of its own
        let family = FIGURE_UNITS.iter().find(|(_, words)| words.contains(&word)).map(|(name, _)| name.clone())?;
        if !units.contains(&family) {
            units.push(family);
        }
    }
    let per_week = PER_WEEK.match_(found.name_str("rest")).is_some();
    let asked = found.name_str("asked").to_lowercase();
    if per_week && asked == "much" && units.iter().all(|unit| FIGURE_TIME.contains(unit)) {
        // "how much a month" is a rate of money and not a length: the period is what the amount is
        // per, so "three to six months" - the length of the buffer the answer names - is not what
        // was asked. Only the article makes it a rate: "how long a month" is not a line, and a
        // line with a thing of its own in it ("how much a month do i need") never gets here.
        return Some(FIGURE_MONEY.clone());
    }
    if units.is_empty() && asked == "much" {
        // An amount with no unit named is money, and a bare "how much" is a price rather than a
        // length. After "should i get a new phone or fix mine" / "its cracked", the answer on the
        // screen said "Repair it if the phone is under four years old ... £50 to £120 at a local
        // shop", and "how much" was handed "Under four years." - the default families here are
        // the time ones, so a price question picked out a time figure. Only a bare "how much" is
        // money: "how much water" names its unit and keeps it.
        return Some(FIGURE_MONEY.clone());
    }
    if units.is_empty() && asked == "many" {
        // "how many was it" names no unit at all, so it asks for a count and not for a length.
        // The default family here is a length, and the count ask was handed the last answer's
        // figure: eight turns after "theres going to be 8 of us", "how many was it" answered
        // "4 to 5 hours." A bare "how long" is still a length and keeps the default.
        return None;
    }
    if units.is_empty() {
        Some(FIGURE_TIME.clone())
    } else {
        Some(units)
    }
}

/// Python: bot/modules/memory_store.py:3956 `_the_clause(said, start, end)`
///
/// The clause of `said` a figure sits in, between the punctuation either side of it. (`start` and
/// `end` are byte offsets here; the marks are ASCII, so the cut is the same.)
pub fn _the_clause(said: &str, start: usize, end: usize) -> &str {
    let left = said[..start].rfind(['.', ';', ':']).map(|at| at + 1).unwrap_or(0);
    let right = said[end..].find(['.', ';', ':']).map(|at| end + at).unwrap_or(said.len());
    &said[left..right]
}

/// Python: bot/modules/memory_store.py:3981 `_the_money_figures(said)`
///
/// Every amount of money the answer states, in order, as (words, definite).
pub fn _the_money_figures(said: &str) -> Vec<(String, bool)> {
    // The amount as it was said, without the punctuation that ended it: the caller puts the full
    // stop on, and "£60 to £120," came back as "£60 to £120,.".
    let mut out = Vec::new();
    for found in A_MONEY_FIGURE.finditer(said) {
        if ACTION_AMOUNT.is_match(_the_clause(said, found.start(), found.end())) {
            // The clause says when or if it happens, so the money is the size of a step and not
            // the amount the person asked about.
            continue;
        }
        out.push((text::rstrip_chars(text::strip(found.as_str()), ",;:").to_string(), true));
    }
    if !out.is_empty() {
        return out;
    }
    for found in A_PERCENT_FIGURE.finditer(said) {
        if ACTION_AMOUNT.is_match(_the_clause(said, found.start(), found.end())) {
            continue;
        }
        out.push((text::rstrip_chars(text::strip(found.as_str()), ",;:").to_string(), true));
    }
    out
}

inline_re!(A_RATE_AFTER, r"(?:a|an|per)\s+", 0);

/// Python: bot/modules/memory_store.py:4001 `_the_figures_in(said, units)`
///
/// Every figure of these units the answer states, in order, as (words, definite).
pub fn _the_figures_in(said: &str, units: &[String]) -> Vec<(String, bool)> {
    if units.iter().any(|unit| unit == "money") {
        return _the_money_figures(said);
    }
    let unit = format!("(?-i:{})", unit_alternation(units).join("|"));
    let qualifier = g("FIGURE_QUALIFIER").str();
    let number = g("FIGURE_NUMBER").str();
    let pattern = Re::new(
        &format!(
            r"(?<![\d,])\b(?P<qual>{qualifier}\s+)?(?P<num>{number})(?:\s*(?:-|to|or)\s*(?P<more>{number}))?\s*(?P<unit>{unit})\b(?P<extra>\s+\d{{1,3}})?(?P<after>[^.;:!?,]*)"
        ),
        pyre::I,
    );
    let unit_words: HashSet<&String> = units.iter().flat_map(|family| figure_words(family).iter()).collect();
    let mut out = Vec::new();
    for found in pattern.finditer(said) {
        let after = text::strip(found.name_str("after")).to_lowercase();
        let next_word = LOWER_WORDS.findall_str(&after).into_iter().next().unwrap_or_default();
        let unit_word = found.name_str("unit").to_lowercase();
        let one = !unit_word.ends_with('s') && !["km", "kg", "lb", "lbs"].contains(&unit_word.as_str());
        if one && !next_word.is_empty() && !FIGURE_AFTER.contains(&next_word) && !unit_words.contains(&next_word) {
            // "the 5 to 6 minute egg": a singular unit with a word after it is an adjective, not
            // the noun of a figure, and a card that says it is not answering "how many minutes".
            continue;
        }
        // The figure alone, not the rest of the sentence it is the subject of: "eight to ten weeks
        // is a realistic build" is handed back as "Eight to ten weeks."
        let more = found.name("more").filter(|more| !more.is_empty()).map(|more| format!("to {more}")).unwrap_or_default();
        let pieces = [found.name_str("qual"), found.name_str("num"), more.as_str(), found.name_str("unit"), found.name_str("extra")];
        let said_figure = pieces.iter().map(|piece| text::strip(piece)).filter(|piece| !piece.is_empty()).collect::<Vec<_>>().join(" ");
        // A rate is not a figure the person can plan by: "three runs a week" and "20 minutes
        // three times a week" belong to the plan, not to the ask about the length of it.
        if A_RATE_AFTER.match_(&after).is_some() || next_word == "per" {
            continue;
        }
        if ACTION_DURATION.is_match(_the_clause(said, found.start(), found.end())) {
            // A length of the doing, not of the thing: "for the first two days", "20 minutes at a
            // time". The clause says so, and the number is not the one that was asked for.
            continue;
        }
        let definite = !["a", "an"].contains(&found.name_str("num").to_lowercase().as_str());
        if !definite {
            continue;
        }
        out.push((said_figure, definite));
    }
    out
}

/// Python: bot/modules/memory_store.py:4043 `_the_figure_the_answer_names(text, memory)`
///
/// The figure the answer on the screen named, on a line that asks only for a figure, else None.
/// Narrow on purpose: the line asks for one kind of figure and names nothing else, the answer is
/// a knowledge answer from a turn or two back, the figure is definite and of the kind asked for,
/// the unit is the noun of it, a rate is not a figure, and a length the answer gives for doing
/// something is a duration of the action, not the length of the thing the person asked about.
pub fn _the_figure_the_answer_names(text_: &str, memory: &Memory) -> Option<String> {
    let units = _the_unit_asked(text_)?;
    if units.is_empty() {
        return None;
    }
    // A figure the person gave about their own life is theirs, not the answer's. "my back has
    // been bad for a week now" then "how long have i had it now" was handed "Six weeks." - the
    // follow-up figure at the end of the back answer - and the week they had just said was in the
    // chat all along. The same line after "i started running two weeks ago" was "20 minutes.", the
    // length of the run-walk in the answer. Only where the person's own figure is there to give:
    // with nothing of their own, the answer on the screen is still the best thing the store knows.
    if let Some(theirs) = ret_some(crate::memory::repeat::the_figure_they_gave_text(text_, memory)) {
        return Some(theirs);
    }
    // An amount per month is not a length in months: an answer that states rates is a table of
    // them, and the saving answer is the one that belongs on the line.
    let rate_ask = A_RATE_ASK.match_(&fold(text_)).is_some();
    let look_back = g("LOOK_BACK_FIGURE").as_int().unwrap() as usize;
    let answer_len = g("FIGURE_ANSWER_LEN").as_int().unwrap() as usize;
    for turn in recent(memory, look_back).iter().rev() {
        if !turn.module.as_deref().map_or(false, |module| ANSWER_MODULES.contains(module)) {
            continue;
        }
        let said = text::squash(&turn.bot);
        if said.is_empty() || text::len(&said) < answer_len || text::rstrip(&said).ends_with('?') {
            continue;
        }
        if rate_ask && A_RATE_STATED.is_match(&said) {
            continue;
        }
        if LISTED_FIGURE.is_match(&said) {
            // A list of figures on one unit ("6 minutes for a runny yolk, 7 for jammy, 9 to 10 for
            // hard boiled") is the answer the person needs, and picking one of its numbers is not:
            // "how long do i boil an egg" / "and how long?" came back as "6 minutes." where the
            // answer had already said the whole range.
            continue;
        }
        let found = _the_figures_in(&said, &units);
        if found.is_empty() {
            if _a_figure_of_those_units(&said, &units) {
                // The answer does state a figure of the kind asked and this reader cannot hand
                // back exactly one of them, so the store says nothing rather than reading an
                // older answer: the figure the person wants is the one on the newest answer, and
                // a number out of an answer two turns back is about something else.
                //
                // "im saving for a car" / "i need about 4000" / "i put away 300 a month" /
                // "so how long" was answered "About 3 years." - the 40 months of the table two
                // turns back - while the answer just above it said "£4,000 at £300 a month is 14
                // months". The scan "a month" out of "300 a month is 14 months" as an
                // indefinite figure and moved past the 14, so nothing was read off the newest
                // answer and the reader fell through to the one behind it. The module that works
                // the figure out (`bot/modules/saving_followup.py`) answers this line with the
                // 14 months.
                return None;
            }
            continue;
        }
        let stated: HashSet<String> = found.iter().map(|(figure, _)| figure.to_lowercase()).collect();
        if stated.len() > 1 {
            // Two figures of the same kind: the answer is not one figure, and the one behind it
            // is about an older thing, so there is nothing to hand back here.
            return None;
        }
        return Some(format!("{}.", cap1(&found[0].0)));
    }
    None
}

/// Python: bot/modules/memory_store.py:4123 `_a_figure_of_those_units(said, units)`
///
/// True when the answer states a number with a unit of one of these families after it. The loose
/// reading, used only to tell "this answer says a length and I cannot read which" from "this
/// answer says no length at all".
pub fn _a_figure_of_those_units(said: &str, units: &[String]) -> bool {
    if units.iter().any(|unit| unit == "money") {
        // Money is its own family and has no entry in FIGURE_UNITS: an amount is written with a sign
        // or with a word for it, so the loose reading is the two money patterns themselves.
        return A_MONEY_FIGURE.is_match(said) || A_PERCENT_FIGURE.is_match(said);
    }
    let words = unit_alternation(units);
    if words.is_empty() {
        return false;
    }
    let unit = format!("(?-i:{})", words.join("|"));
    let number = g("FIGURE_NUMBER").str();
    Re::new(&format!(r"(?<![\d,])(?:{number}|\d[\d,.]*)\s*(?:-|to|or)?\s*{unit}\b"), pyre::I).is_match(said)
}

/// Python: bot/modules/memory_store.py:4162 `_the_time_they_put_right(memory)`
///
/// The reply to a line that does nothing but move a routine time, or None: the time is said back
/// in the words the store holds it in, and the one it replaced is named.
pub fn _the_time_they_put_right(memory: &Memory) -> Option<String> {
    let fixed = mget(memory, "fixed_time");
    if !fixed.is_dict() {
        return None;
    }
    // The time is moved while the line is being read, so its turn is this one: `turns` holds the
    // turns before it. Any other turn is another line, and this reply is not about that line.
    let turn = match vget(fixed, "turn") {
        v if !v.truthy() => -1,
        Value::Int(n) => *n,
        Value::Bool(b) => *b as i64,
        Value::Float(f) => f.trunc() as i64,
        Value::Str(s) => py_int(s).unwrap_or_else(|| panic!("ValueError: invalid literal for int(): {s:?}")),
        other => panic!("TypeError: int() argument: {}", py_repr(other)),
    };
    if turn != memory.turns.len() as i64 {
        return None;
    }
    let at = text::squash(&sget(fixed, "at"));
    if at.is_empty() {
        return None;
    }
    let what = text::squash(&sget(fixed, "what"));
    let template = match g("ROUTINE_SAY_BACK").get(&what) {
        Some(v) if v.truthy() => v.str().to_string(),
        _ => "You {what} at {at}.".to_string(),
    };
    let said_back = fmt_named(&template, &[("at", at.as_str()), ("what", what.as_str())]);
    let mut said_back = text::rstrip_chars(&said_back, " .").to_string();
    let was = text::squash(&sget(fixed, "from"));
    if !was.is_empty() && was != at {
        said_back.push_str(", not ");
        said_back.push_str(&was);
    }
    if said_back.is_empty() {
        panic!("IndexError: string index out of range");
    }
    Some(format!("Got it - {}.", low1(&said_back)))
}

// ---------------------------------------------------------------------------------------------
// The reply.

/// Python: bot/modules/memory_store.py:4191 `respond(text, memory)` - a runtime hook; its signature is the runtime's.
///
/// The store's reply, with the names in it written as the store holds them. Every route out of
/// the store ends here, so the names are written once, at the end. Then the floor under it: never
/// the same words twice in a row - whatever route the reply came by, `bot/memory/again.instead`
/// frames it as the repeat it is, after the names are written, so the words it compares are the
/// words on the screen.
pub fn respond(text_: &str, memory: &mut Memory) -> Option<String> {
    let text_owned = _without_the_shrug_at_the_end(text_);
    let text_: &str = &text_owned;
    // A line that only moves a time the store already holds ("i meant seven") is answered by the
    // store: nothing else has anything to say about it, and what did say something handed the
    // person a shrug or the search hedge over a correction the store had just taken.
    if let Some(put_right) = _the_time_they_put_right(memory).and_then(nz) {
        return Some(put_right);
    }
    // A line that points back at what was said before, in a chat with nothing in it yet ("same as
    // before", "and the other one"): nothing below reads it, and the small-talk pools answered it
    // with a receipt of a line nobody had given. There is no before to go on, so the store says so
    // and asks for the thing itself - read here, above every reader, because on a chat that has
    // said something these lines are theirs and go on being read exactly as they were.
    if let Some(pointed_at_nothing) = _points_back_at_nothing(text_, memory).and_then(nz) {
        return Some(pointed_at_nothing);
    }
    // A recall frame with nothing behind it: "what did i say about it", "what did i say i lost".
    // Read above the pointer asks, because the pointer reader read the "it" in the recall and
    // answered a request for their own words as a question about a subject.
    if let Some(recall_frame) = _a_recall_frame_about_nothing(text_, memory).and_then(nz) {
        return Some(recall_frame);
    }
    // An ask whose only subject is a pointing word, in a chat with nothing behind it: "how do i
    // do it", "how long will it take", "do that again". The "it" has no thing to be yet, and the
    // turn was answered with a search hedge ("I'm not sure about that one, sorry - I'd rather not
    // guess") or a shrug ("Fair enough."), neither of which asks the one question that would let
    // the person get an answer.
    if let Some(ask_about_nothing) = _an_ask_about_nothing(text_, memory).and_then(nz) {
        return Some(ask_about_nothing);
    }
    // "how am i doing" with a figure for the plan in the chat is answered from that figure
    // (`bot/memory/goals.py`). Without it the line is a question about a diet in general and the
    // knowledge base's paragraph stands - and on a chat about a half stone that paragraph was an
    // arithmetic card ("Divide it by 2: half of 46 is 23..."), which answers a question nobody
    // asked. Both halves are needed: a line that is not this ask, or a chat with no figure, gets
    // Nothing from here and goes on exactly as it did.
    if let Some(going) = ret_some(crate::memory::goals::a_progress_asked(text_, memory)) {
        return Some(going);
    }
    // An ask about what they said or were worried about, in a chat that has said nothing: "what was
    // it i was worried about". Read as a line about a worry it got advice about worrying ("Write
    // down what's actually at stake"), which invents a feeling nobody named.
    if let Some(recall_about_nothing) = _a_recall_about_nothing(text_, memory).and_then(nz) {
        return Some(recall_about_nothing);
    }
    // "what did i say her birthday was" is the direct ask with a wrapper round it, and the reader
    // that keeps that slot answers it; read as a topic instead, the last line they said about
    // anything came back as if it were the answer.
    if let Some((direct, attribute, possessive)) = _the_ask_behind_the_recall(text_) {
        if let Some(answer) = respond(&direct, memory).and_then(nz) {
            return Some(answer);
        }
        if _a_word_for_the_slot(memory, &attribute) {
            // They said it in a shape the slot reader does not read ("hes a labrador, 9 weeks
            // old" is an age the store never filed), so a line saying they never told me would
            // deny what they said. The store's own reply to the line as asked quotes it instead,
            // and that is the honest answer.
        } else {
            // Nothing is filed for that slot: the honest line is that, since the reader for it
            // has just said it holds nothing, and a line about something else is not an answer.
            let theirs = if possessive == "my" || possessive == "our" { "your" } else { possessive.as_str() };
            return Some(pick(
                &[
                    format!("You haven't told me {theirs} {attribute}."),
                    format!("I don't have {theirs} {attribute} from you yet."),
                    format!("You haven't mentioned {theirs} {attribute} to me."),
                ],
                memory,
                "",
            ));
        }
    }
    // The ages of the children the store counts, given one at a time ("ones 7 and ones 4" after
    // "i have two kids"): the line names no group, so nothing else reads it and the store's own
    // echo said the line back while keeping nothing.
    let ages_of_the_children = _the_ages_of_the_children(text_, memory);
    if !ages_of_the_children.is_empty() {
        return Some(ages_of_the_children);
    }
    // A line that only rules a place out is not a line about that place: it is answered first,
    // because every reader below would answer about the place they had just taken back.
    if let Some(ruled_out) = ret_some(crate::memory::not_this::reply_for(text_, memory)) {
        return Some(ruled_out);
    }
    if let Some(told_about_the_trip) = ret_some(crate::memory::trips::reply_for(text_, memory, true)) {
        // A trip said plainly, or a detail added to one the conversation is on, is answered
        // about the trip: the month it is in, the people going, and how the journey goes. Read
        // here rather than inside the store's own reply so it comes before every reader below
        // that would answer the trip's subject instead.
        return Some(told_about_the_trip);
    }
    // A length the person gave about their own life, asked back with nothing named ("how long
    // has it been" three turns after "my back has been bad for a week now", "how long have i had
    // it now" after "i started running two weeks ago"). The unit reader below reads only a line
    // that names its unit ("how many weeks"), so this shape fell past it and the answer on the
    // screen was quoted instead - a length of something else entirely. Their own words are the
    // answer while the length is in the chat.
    let folded_line = fold(text_);
    if data::re("bot.memory.repeat.A_BARE_DURATION_ASK").is_match(&folded_line) {
        if let Some(theirs) = ret_some(crate::memory::repeat::the_figure_they_gave_text(text_, memory)) {
            return Some(theirs);
        }
    }
    if data::re("bot.memory.repeat.A_PAY_WORD").is_match(&folded_line) {
        // An ask for pay names its figure in a word the unit reader has no family for: "how much
        // is my wage" names no unit at all, and "how much do i make" names its pay in a verb the
        // keywords drop. So the wage the chat holds was read back only when the ask happened to
        // carry a unit word, and the person asking for their own pay got the budgeting card. The
        // figure they gave is the answer to any ask for it, and the reader below is where the
        // judgement words that make it a different ask are refused.
        if let Some(theirs) = ret_some(crate::memory::repeat::the_figure_they_gave_text(text_, memory)) {
            return Some(theirs);
        }
    }
    if ret_true(crate::memory::repeat::how_long_its_been(text_)) {
        // The same ask with the thing named instead of "it" ("how long has my back been like this
        // now"): the unit reader names the back and stands down, so the store says the figure the
        // person gave, which is what the line asked for.
        if let Some(theirs) = ret_some(crate::memory::repeat::the_figure_they_gave_text(text_, memory)) {
            return Some(theirs);
        }
    }
    if let Some(figure) = _the_figure_the_answer_names(text_, memory).and_then(nz) {
        // The figure the answer on the screen named, on a line that asks for nothing else: said
        // back as the figure, because the sentence it is in is one the person has just read.
        return Some(figure);
    }
    // A length ask about a pregnancy the chat holds is answered here, before the figure they gave
    // is looked for: "how many weeks am i" is a count of weeks in a line of theirs, so the reader
    // for a figure said back took it ("You said 12 weeks.") where the number they gave is the
    // length of their pregnancy and is said as one ("Twelve weeks.").
    let bare_line = _bare(&_folded(text_, memory));
    if let Some(along) = ret_some(sr::_how_far_along(memory, &bare_line)) {
        return Some(along);
    }
    if let Some(theirs) = ret_some(crate::memory::repeat::asked_back_text(text_, memory)) {
        // A recall ask about a figure of their own is answered from the line they gave it on,
        // before any entry is looked up for the thing it names. "how long am i going for" after
        // "we fly out on the 12th for 10 days" reached the departure card and answered about
        // getting to the airport two hours early, and "how many nights did i say" answered with
        // when to book flights - both while the nights and the days were in their own line. The
        // reader is narrow (bot/memory/repeat.py, `asked_back`): a recall cue, a short line, the
        // thing named in a line they said, and one figure of the kind asked for.
        return Some(theirs);
    }
    let reply = ret_opt(sr::_the_store_reply(text_, memory))?;
    let reply = _the_places_are_written_as_names(&reply, memory);
    // Two lines are the ones whose repeat is the point. A line that asked for the whole of the
    // answer: the reply opens "All of it:" because the person asked for that, so framing it
    // again as "Same answer, and it covers this:" takes the lead-in off and says the bot decided
    // to repeat itself - "my name is sam" / "all of it" came back as that, about a greeting.
    // And a request to hear the answer again, which is handed the words and not a note that they
    // were said before: the never-twice rule is about answering twice, not about being asked
    // twice.
    if WHOLE_ASK.match_(&text::strip(text_).to_lowercase()).is_none() && !_asks_to_hear_it_again(text_, memory) {
        if let Some(framed) = ret_some(crate::memory::again::instead(&reply, text_, memory)) {
            return Some(framed);
        }
    }
    Some(reply)
}

/// Python: bot/modules/memory_store.py:4361 `_asks_to_hear_it_again(text, memory)`
///
/// True for the one turn where the same words are the answer: "what did you say earlier".
pub fn _asks_to_hear_it_again(text_: &str, memory: &Memory) -> bool {
    // an unreadable line keeps the floor under the reply
    py_try(|| ret_true(sr::_repeat_ask_m(text_, &_folded(text_, memory), None))).unwrap_or(false)
}

rx!(YES_NO_ASK);

/// Python: bot/modules/memory_store.py:4503 `_asks_about_someone_else(text)`
///
/// True for a yes/no question about a person ("is he allergic to peanuts"). The question mark is
/// not asked for: what makes the line an ask is the auxiliary at the front and a person after it.
pub fn _asks_about_someone_else(text_: &str) -> bool {
    YES_NO_ASK.match_(&text::squash(text_)).is_some()
}

rx!(SAID_AGAIN, SORRY_OPENS_A_CORRECTION);

/// `value in ("", None)`.
fn blank(value: &Value) -> bool {
    value.is_none() || value.as_str() == Some("")
}

/// Python: bot/modules/memory_store.py:4607 `_note_a_corrected_fact(memory, text, entry, was)`
///
/// Remember that this turn put a fact about someone right, so the reply can say which one. A
/// correction is answered with the fact, so this one is.
pub fn _note_a_corrected_fact(memory: &mut Memory, text_: &str, entry: &Value, was: Option<&Value>) {
    let Some(was) = was.filter(|was| was.is_dict()) else { return };
    if !entry.is_dict() {
        return;
    }
    let said = fold(text_);
    if !(SAID_AGAIN.is_match(&said) || SORRY_OPENS_A_CORRECTION.is_match(&said)) {
        return;
    }
    for (kind, value) in entry.pairs() {
        let kind_s = kind.as_str().unwrap_or("");
        if ["who", "pronoun", "turns"].contains(&kind_s) || blank(value) {
            continue;
        }
        let before = vget(was, kind_s);
        if blank(before) || before == value {
            continue;
        }
        memory.set(
            "fact_fixed",
            dict_of(vec![
                ("who", Value::from(sget(entry, "who"))),
                ("kind", kind.clone()),
                ("value", value.clone()),
                ("was", before.clone()),
                ("said", Value::from(said.as_str())),
            ]),
        );
        return;
    }
}

/// Python: bot/modules/memory_store.py:4634 `_note_a_fact_moved(memory, text, entry, was)`
///
/// Remember a correction that moves a fact from one person to another. Only the shape a
/// correction is said in ("i meant", "sorry") counts, and only when a different person already
/// holds the same value.
pub fn _note_a_fact_moved(memory: &mut Memory, text_: &str, entry: &Value, was: Option<&Value>) {
    if !entry.is_dict() {
        return;
    }
    let said = fold(text_);
    if !(SAID_AGAIN.is_match(&said) || SORRY_OPENS_A_CORRECTION.is_match(&said)) {
        return;
    }
    let who = text::strip(&sget(entry, "who")).to_string();
    if who.is_empty() {
        return;
    }
    // The person has to be named by this line: "sorry i meant my daughter is a nurse" is read for
    // the daughter, and the store also holds the nurse the son had, which is the fact being moved
    // away from and not the one moving - taken as the mover it sent the job the wrong way.
    if !says_word(&who, &said) {
        return;
    }
    for (kind, value) in entry.pairs() {
        let kind_s = kind.as_str().unwrap_or("");
        if ["who", "pronoun", "turns"].contains(&kind_s) || blank(value) {
            continue;
        }
        if value.is_list() || value.is_dict() {
            continue;
        }
        if let Some(was) = was.filter(|was| was.is_dict()) {
            let held = vget(was, kind_s);
            if !blank(held) && held != value {
                continue; // the same person's own fact being put right: `fact_fixed` has that one
            }
        }
        let who_v = Value::from(who.as_str());
        let Some(at) = others(memory).items().iter().position(|one| one.is_dict() && has(one, "who") && vget(one, "who") != &who_v && vget(one, kind_s) == value)
        else {
            continue;
        };
        let was_who = py_str(vget(&others(memory).items()[at], "who"));
        memory.set(
            "fact_moved",
            dict_of(vec![
                ("who", who_v),
                ("was_who", Value::from(was_who)),
                ("kind", kind.clone()),
                ("value", value.clone()),
                ("said", Value::from(said.as_str())),
            ]),
        );
        // The claim the arbiter asks for (`TAKES_AS_IT_COMES`) is put on the chat's own memory,
        // because each module is handed a copy of it: a mark made in the store's copy would be
        // thrown away. This turn is the store's answer even when an entry reads the word "sorry"
        // in the line, and the apology card took the turn off "sorry i meant my daughter is a
        // nurse" while the store's line said which child the nurse was.
        memory.set("moved_to_who", said.as_str());
        // The first line said the fact about the wrong person, so it comes off that one: leaving
        // it made "how old is my son" answer nine when the person had already said the nine was
        // their daughter's.
        others(memory).items_mut().unwrap()[at].del_key(kind_s);
        return;
    }
}

rx!(RECALL_WRAPPER, RECALL_POSSESSIVE, RECALL_SLOTS_BY_POSSESSIVE, A_RECALL_IN_ANOTHER_SHAPE);
val!(RECALL_SLOTS, SHE_OR_HE, SLOT_WORDS);

/// Python: bot/modules/memory_store.py:4736 `_a_word_for_the_slot(memory, attribute)`
///
/// Whether a line of theirs in this chat holds a word for this slot.
pub fn _a_word_for_the_slot(memory: &Memory, attribute: &str) -> bool {
    let Some(pattern) = SLOT_WORDS.get(&attribute.to_lowercase()).and_then(Value::as_re) else { return false };
    recent(memory, 8).iter().any(|turn| pattern.is_match(&turn.user))
}

inline_re!(A_COPULA_AT_THE_END, r"\s+(?:was|is|were|are)\s*$", 0);
inline_re!(A_COPULA, r"\b(?:was|is|were|are)\b", 0);

/// Python: bot/modules/memory_store.py:4744 `_the_ask_behind_the_recall(low)`
///
/// The direct ask and the slot a recall ask is about - (ask, attribute, possessive) - or None.
/// Only a slot the person could have given in words is read this way, and only with a possessive
/// in front of it ("his name", "my budget").
pub fn _the_ask_behind_the_recall(low: &str) -> Option<(String, String, String)> {
    let wrapped = RECALL_WRAPPER.match_(low)?;
    let rest = A_COPULA_AT_THE_END.sub("", text::strip(wrapped.name_str("rest")));
    let said = wrapped.name_str("word").to_lowercase().replace("  ", " ");
    let slot = RECALL_SLOTS.items().iter().find(|slot| slot.items()[0].as_re().unwrap().is_match(&rest))?;
    let attribute: Option<&str> = slot.items()[2].as_str();
    let possessive = RECALL_POSSESSIVE.match_(&rest);
    if said.starts_with("how old") {
        // "how old did i say she was": the frame asks the age itself, so the person is the whole
        // of what is left, and it has to be a pronoun the store can point at.
        let person = text::strip(&rest).to_lowercase();
        let pair = SHE_OR_HE.get(&person)?;
        let (who, verb) = (pair.items()[0].str(), pair.items()[1].str());
        let ask = format!("how old {}{}", verb, if who == "i" { " i".to_string() } else { format!(" {who}") });
        return Some((ask, "age".to_string(), person));
    }
    let possessive = possessive?;
    let person = possessive.name_str("poss").to_lowercase();
    // The ask is rebuilt from the words the person used, with the verb taken out and the question
    // word that slot is asked with: "what did i say my dog was called" is "what is my dog called"
    // (not "what is my name"), and "what did i say her birthday was" is "when is her birthday".
    let plain = text::squash(&A_COPULA.sub(" ", &rest));
    if plain.is_empty() {
        return None;
    }
    let word = if attribute == Some("birthday") { "when is " } else { "what is " };
    // The miss line names the slot only when the ask was the slot on its own ("his name", "her
    // birthday"): with a noun in between ("my dog called") the thing is the dog, and what it is
    // called is a read the store's own readers answer.
    let only_slot = RECALL_SLOTS_BY_POSSESSIVE.match_(&plain).map(|m| m.name_str("attribute").to_string()).filter(|s| !s.is_empty());
    let named = only_slot.unwrap_or_else(|| attribute.filter(|a| !a.is_empty()).unwrap_or("answer").to_string());
    Some((format!("{word}{plain}"), named, person))
}

rx!(BARE_WHAT_I_DO);

/// Python: bot/modules/memory_store.py:4816 `TAKES_AS_IT_COMES(text, memory=None)`
///
/// Claim every line the store can answer - the bare "what do i do" included. The arbiter reads
/// one `TAKES_AS_IT_COMES` per module, so every reading has a private name and is asked from
/// here. The job ask is the last reading: the bare "what do i do" said right after the person
/// gave their job is the fact, whatever else read. False everywhere else.
pub fn TAKES_AS_IT_COMES(text_: &str, memory: &mut Memory) -> bool {
    if mget(memory, "moved_to_who").as_str() == Some(fold(text_).as_str()) {
        // "sorry i meant my daughter is a nurse": the fact moved from one person to another, and
        // the store said which. An entry that reads the word "sorry" answers it as a request for
        // the apology script, so the store's line takes the turn on this line alone.
        return true;
    }
    if _the_store_readings(text_, memory) {
        // Every reading the store had before this one lives in `_the_store_readings`, and it is
        // asked from here because this is the only `def` of the exported name: a second one above
        // would replace this hook at import and every reading under it would stop answering.
        return true;
    }
    if ret_true(crate::memory::told::A_CALLBACK(text_, Some(&*memory))) {
        // "remember when i said my knee hurt": the store is the part that can say what was
        // actually said, and an entry that reads the word in it ("knee") would serve the advice
        // again. This hook is the one the arbiter reads - the store has an older definition
        // above, which no longer runs - so the claim is made here.
        return true;
    }
    if _the_ask_about_how_it_went(text_, memory) {
        // The thing they did today and the ask about how it went: only they can say how it went,
        // so the store answers by asking them. An entry about the thing would answer the question
        // nobody asked - "interview" is a word in the line, not what the line is about.
        return true;
    }
    if _a_callback_that_names_its_thing(text_, memory) {
        // "my boiler keeps losing pressure", then "what have i been saying about the boiler": the
        // line is about the conversation and the store's line answers it. The reading still gives
        // it a topic - a boiler is home-and-diy - so the specialist rule lifted the boiler entry
        // over the recall band, and the person who asked what they had said was told how to set
        // the flow temperature on the dial. "my kitchen tap drips" then "what have i been saying
        // about the kitchen" got a traybake the same way.
        //
        // Only the store's own gate for "this line is about the conversation", and only with the
        // thing named after "about": "what were we on about" and "what was i saying" name nothing,
        // and "when should i get the boiler serviced" names a boiler without being about the chat.
        return true;
    }
    if ret_true(crate::memory::reminded::about_a_thing_they_said(text_, memory)) {
        // "my boiler keeps losing pressure", then "what was wrong with my boiler again": the
        // past tense and the "again" make it a reminder about the line they said, and read as a
        // question about the world it was a fault to diagnose - the boiler entry answered it
        // with the flow temperature, a turn after the person had told us what was wrong.
        // The reading is `bot/memory/reminded.py`, and it is the same one the store's gate for
        // "this line is about the chat" uses, so the two cannot disagree.
        return true;
    }
    let folded = _folded(text_, memory);
    if BARE_WHAT_I_DO.match_(&_bare(if folded.is_empty() { text_ } else { &folded })).is_none() {
        return false;
    }
    has(user(memory), "job") && ret_true(sr::_the_job_was_just_said(memory))
}

/// Python: bot/modules/memory_store.py:5019 `_about_the_verb(plans, low, found)`
///
/// The plan a when or what-time ask is about: the one that says its verb, and its name too. When
/// no plan names the words after the verb the verb-only reading still stands.
pub fn _about_the_verb(plans: &[String], low: &str, found: &Match) -> Option<String> {
    let named = crate::memory::topics::content(&low[found.end()..]);
    if !named.is_empty() {
        for plan in plans.iter().rev() {
            if ret_true(sr::_says_the_verb(plan, found.str(1))) && named.iter().any(|word| ret_true(sr::_names_the_thing_m(plan, word, None))) {
                return Some(plan.clone());
            }
        }
    }
    plans.iter().rev().find(|plan| ret_true(sr::_says_the_verb(plan, found.str(1)))).cloned()
}

/// Python: bot/modules/memory_store.py:5120 `_remember_dislikes(text, memory, named_here=())`
///
/// Keep a food someone else will not eat against that person: "she hates mushrooms".
pub fn _remember_dislikes(text_: &str, memory: &mut Memory, named_here: &[String]) {
    for pair in crate::memory::guest::found(text_).val().items() {
        // `for food, at in guest.found(text)`
        let (food, at) = match pair.items() {
            [food, at] => (py_str(food), int_of(at).unwrap_or(0)),
            _ => panic!("ValueError: not enough values to unpack (expected 2)"),
        };
        let read = extract::subject(text_, at).val();
        let kind = s_or(read.items().first());
        let who = s_or(read.items().get(1));
        if kind == "user" {
            if let Some(food) = ret_some(crate::memory::guest::plain(&food, false)) {
                remember(memory, "dislikes", &str_list(&[food]), text_);
            }
            continue;
        }
        if kind != "other" {
            continue;
        }
        let who = if who.is_empty() { Value::from(_person_from_pronoun(text_, memory, named_here)) } else { Value::from(who) };
        remember_other(memory, &dict_of(vec![("who", who), ("dislikes", str_list(&[food]))]), text_);
    }
}

/// Python: bot/modules/memory_store.py:5194 `_diet_reply(memory, low='')`
///
/// How they eat, said back when they ask. This is the copy the move to
/// `bot/memory/store_replies.py` left behind: nothing in PyBot calls it (every caller reads the
/// one in `store_replies`), and past its first branch it reads names this module no longer has
/// (`DIET_FACT_ASK`, `_food_facts`, `MEAL_WANTED`), so there Python raises NameError.
pub fn _diet_reply(memory: &mut Memory, low: &str) -> Value {
    // A line that reports a change to the diet, or asks for one, is not a request for the fact
    // and not a gap in it: "i want to lose a bit of weight" then "ive changed my diet" was
    // answered "You haven't told me about your diet", a denial of what they had just said.
    let said = crate::memory::change::reported(low, "diet").val();
    if said.truthy() {
        let was = vget(user(memory), "diet").clone();
        if said.as_str() == Some("changed") && was.truthy() {
            // The reply says what it is doing with the old value ("You were vegetarian, so I'll
            // drop that"), so the store has to do it: "im vegetarian" / "i changed my diet" /
            // "am i vegetarian" said "Yes - you're vegetarian" and "what can i cook" still picked
            // the vegetarian dish, so the bot promised a drop and went on quoting the value.
            forget(memory, "diet", &list_of(vec![was.clone()]), None);
            user(memory).set_key("diet_changed", was.clone());
        }
        return crate::memory::change::reply(&py_str(&said), "diet", &s_or(Some(&was))).val();
    }
    let diet = vget(user(memory), "diet").clone();
    if !diet.truthy() {
        if !low.is_empty() {
            panic!("NameError: name 'DIET_FACT_ASK' is not defined");
        }
        panic!("NameError: name '_food_facts' is not defined");
    }
    panic!("NameError: name 'MEAL_WANTED' is not defined");
}

strset!(FACT_WORDS, RECALL_NAMES_NOTHING);
inline_re!(AFTER_ABOUT, r"\babout\b\s+(.+)$", 0);
inline_re!(LETTERS, r"[a-z]+", 0);

/// Python: bot/modules/memory_store.py:5722 `_a_fact_word(low)`
///
/// True when the thing a callback names is one of the facts the store keeps.
pub fn _a_fact_word(low: &str) -> bool {
    let after = AFTER_ABOUT.search(low).map(|m| m.str(1).to_lowercase()).unwrap_or_default();
    LETTERS.findall_str(&after).iter().any(|word| FACT_WORDS.contains(word))
}

/// Python: bot/modules/memory_store.py:5729 `_a_callback_that_names_its_thing(text, memory)`
///
/// True when the line asks about the conversation and names the thing it asks about. Both,
/// because either on its own is too wide for a claim.
pub fn _a_callback_that_names_its_thing(text_: &str, memory: &mut Memory) -> bool {
    // a reading that fails claims nothing
    py_try(|| {
        let low = fold(text_);
        ret_true(sr::_names_its_thing(&low)) && ret_true(sr::_about_this_chat(text_, memory))
    })
    .unwrap_or(false)
}

strs!(POINTS_BACK_NOTHING, WHAT_I_SAID_NOTHING, POINTING_WORD_NOTHING);

/// `_vary(options, memory)`.
fn _vary(options: &[String], memory: &Memory) -> String {
    ret_str(sr::_vary(options, memory))
}

/// Python: bot/modules/memory_store.py:5908 `_points_back_at_nothing(text, memory)`
///
/// The store's answer to a pointer with nothing behind it, or None when it is not that line.
pub fn _points_back_at_nothing(text_: &str, memory: &Memory) -> Option<String> {
    if !ret_true(crate::memory::thread::points_back(text_)) || !ret_true(crate::memory::thread::nothing_said(memory)) {
        return None;
    }
    Some(_vary(&POINTS_BACK_NOTHING, memory))
}

/// Python: bot/modules/memory_store.py:5922 `_a_recall_about_nothing(text, memory)`
///
/// The store's answer to "what was it i was worried about" with nothing said, or None.
pub fn _a_recall_about_nothing(text_: &str, memory: &Memory) -> Option<String> {
    if !ret_true(crate::memory::thread::asks_what_i_said(text_)) || !ret_true(crate::memory::thread::nothing_said(memory)) {
        return None;
    }
    Some(_vary(&WHAT_I_SAID_NOTHING, memory))
}

/// Python: bot/modules/memory_store.py:5958 `_names_nothing_in_a_recall(rest)`
///
/// True when what follows a recall ask names no thing of its own.
pub fn _names_nothing_in_a_recall(rest: &str) -> bool {
    LETTERS.findall_str(&rest.to_lowercase().replace('\'', "")).iter().all(|word| RECALL_NAMES_NOTHING.contains(word))
}

/// Python: bot/modules/memory_store.py:5964 `_a_recall_frame_about_nothing(text, memory)`
///
/// The store's answer to "what did i say about it" with nothing said, or None. A rule-out, not a
/// new pool: it hands back the same WHAT_I_SAID_NOTHING line the feelings recall gets.
pub fn _a_recall_frame_about_nothing(text_: &str, memory: &Memory) -> Option<String> {
    if !ret_true(crate::memory::thread::nothing_said(memory)) {
        return None;
    }
    let folded = fold(text_);
    let wrapped_low = text::strip_chars(&folded, " .!?,");
    // The shapes of the ask that no slot reader reaches, kept to the ones that name no thing: a
    // line that does name one ("did i say anything about the boiler") has a reader below that says
    // the thing back, which is the better answer.
    if let Some(another) = A_RECALL_IN_ANOTHER_SHAPE.match_(wrapped_low) {
        if _names_nothing_in_a_recall(&wrapped_low[another.end()..]) {
            return Some(_vary(&WHAT_I_SAID_NOTHING, memory));
        }
    }
    let wrapped = RECALL_WRAPPER.match_(wrapped_low)?;
    // Only where what the frame is filled with is a pointer and nothing else: "about it", "about
    // that". A person or a thing named inside the frame ("what did i say her birthday was", "how
    // old did i say he was") is a slot one of the readers below keeps, and those answer with the
    // slot that is empty, which is a better line than this one.
    let rest = wrapped.name_str("rest");
    if !data::re("bot.memory.thread.POINTING_WORD").is_match(rest) || LETTERS.findall_str(rest).len() > 2 {
        return None;
    }
    Some(_vary(&WHAT_I_SAID_NOTHING, memory))
}

/// Python: bot/modules/memory_store.py:6000 `_an_ask_about_nothing(text, memory)`
///
/// The store's answer to an ask about nothing at all, or None when it is not that line.
pub fn _an_ask_about_nothing(text_: &str, memory: &Memory) -> Option<String> {
    if !ret_true(crate::memory::thread::points_at_a_word_alone(text_)) || !ret_true(crate::memory::thread::nothing_said(memory)) {
        return None;
    }
    // A line that is nothing but pointing words ("what does that mean", "how does that work") is
    // already read by the composer, which asks for the thing itself in its own words. Two pools
    // answering one shape is two voices for the same turn, so the composer keeps that one and
    // this reading is only for the pointer asks its word list does not reach.
    if ret_true(crate::compose::fallback::_points_at_nothing(text_, memory, "")) {
        return None;
    }
    Some(_vary(&POINTING_WORD_NOTHING, memory))
}

/// Python: bot/modules/memory_store.py:6021 `_the_last_ask_about_the_bot(memory)`
///
/// The last thing they asked about the bot, as a sentence, when that is all they have said.
pub fn _the_last_ask_about_the_bot(memory: &Memory) -> Option<String> {
    let lines = crate::memory::thread::user_lines(memory).val();
    for pair in lines.items().iter().rev() {
        let (said, reading) = match pair.items() {
            [said, reading] => (said, reading),
            _ => panic!("ValueError: not enough values to unpack (expected 2)"),
        };
        // `reading` is the understanding of the line: a dict in Python, an object here.
        let about_bot = reading.get("about_bot").or_else(|| reading.attr("about_bot")).map_or(false, Value::truthy);
        if !about_bot {
            return None;
        }
        if let Some(asked) = ret_some(crate::memory::thread::restate(&py_str(said))) {
            return Some(asked);
        }
    }
    None
}

/// Python: bot/modules/memory_store.py:6227 `_fits(answer, room)`
///
/// The move of an answer whose own length is the largest one that still fits `room`. The largest
/// that fits, not the shortest.
pub fn _fits(answer: &str, room: i64) -> Option<String> {
    let mut best: Option<String> = None;
    let mut best_seconds: i64 = -1;
    let names = ret_true(sr::_names_minutes(answer));
    for sentence in sr::_sentences(answer).val().items() {
        for part in sr::_halves(sentence.str()).val().items() {
            let part = part.str();
            let Some(took) = int_of(&sr::_longest(part, &Value::Bool(names)).val()) else { continue };
            if took > room || !ret_true(sr::_says_something(part)) {
                continue;
            }
            if took > best_seconds {
                best = Some(part.to_string());
                best_seconds = took;
            }
        }
    }
    best
}

rx!(CHECK_FOLLOWED);

/// Python: bot/modules/memory_store.py:6348 `_asks_whether_we_followed(text, memory)`
///
/// True when the whole line asks whether the last thing they said was taken in.
pub fn _asks_whether_we_followed(text_: &str, memory: &Memory) -> bool {
    let mut low = ret_str(sr::_plain_line(&_folded(text_, memory)));
    if low.is_empty() {
        return false;
    }
    if low.ends_with('?') {
        low = text::strip(&sl(&low, 0, Some(-1))).to_string();
    }
    // The same words asked about the world: "did you see it" after "we watched the final
    // together" asks whether the bot was at the match, not whether it took the line in, and
    // reading their own evening back to them ("Yes — you watched the final together") is a true
    // sentence and no answer at all. The composer already reads that line as the invitation it
    // is, so the store stands down exactly where the composer answers (bot/compose/fallback.py,
    // `reads_as_if_i_saw`) and every other check keeps this reading.
    if ret_true(sr::_reads_as_the_invitation(&low, memory)) {
        return false;
    }
    CHECK_FOLLOWED.match_(&low).is_some()
}

/// Python: bot/modules/memory_store.py:6391 `_the_group_size_ask(text, memory)`
///
/// Whether this is a question about how many the person's own group is, and the number is kept.
/// Nothing is claimed unless the store actually holds a size.
pub fn _the_group_size_ask(text_: &str, memory: &Memory) -> bool {
    let low = text_.to_lowercase();
    // The ask may name the group ("how many of us") or point back at the number with nothing else
    // said ("how many was it", "how many again"): both are the size this store is holding, and the
    // bare one was otherwise handed to the answers module, which read "how many was it" as a
    // figure off the last card.
    // "how many people is it for" is not in here: the roast card answers it with the rate a
    // table of that size needs, and the number said back is a poorer answer to the same ask.
    if !(data::re("bot.memory.counts.GROUP_ASK").is_match(&low) || data::re("bot.memory.counts.BARE_COUNT_ASK").is_match(&low)) {
        return false;
    }
    ret_true(crate::memory::counts::answer(text_, memory))
}
