//! Port of `bot/memory/limit.py`: a follow-up that puts a limit on the answer just given ("i
//! dont have a plunger", "i ran out of oven cleaner", "ive got a cat"). The answer is cut into
//! steps, the ones the limit rules out are dropped, and the rest is said back with the limit as
//! the reason. Only a short line that names a limit, and only the turn after an answer.

use super::pyv;
use super::topics;
use super::tried;
use crate::data::{self, Value};
use crate::py::re::{self, Match, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understanding::understand;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

const M: &str = "bot.memory.limit.";
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn words_set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}

static OUT_OF: LazyLock<&'static Re> = LazyLock::new(|| rx("OUT_OF"));
static NOT_HAVE: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_HAVE"));
static A_DETAIL_OF_THE_FAULT: LazyLock<&'static Re> = LazyLock::new(|| rx("A_DETAIL_OF_THE_FAULT"));
static A_HEATING_FAULT: LazyLock<&'static Re> = LazyLock::new(|| rx("A_HEATING_FAULT"));
static A_LEAKING_FAULT: LazyLock<&'static Re> = LazyLock::new(|| rx("A_LEAKING_FAULT"));
static NOT_A_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_A_THING"));
static IN_THE_HOUSE: LazyLock<&'static Re> = LazyLock::new(|| rx("IN_THE_HOUSE"));
static ALSO_THERE: LazyLock<&'static Re> = LazyLock::new(|| rx("ALSO_THERE"));
static NOT_WITH_A_PET: LazyLock<&'static Re> = LazyLock::new(|| rx("NOT_WITH_A_PET"));
static ASKS: LazyLock<&'static Re> = LazyLock::new(|| rx("ASKS"));
static ASKS_A_CHOICE: LazyLock<&'static Re> = LazyLock::new(|| rx("ASKS_A_CHOICE"));
static CLAUSE_STARTS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("CLAUSE_STARTS"));
static STEP: LazyLock<&'static Re> = LazyLock::new(|| rx("STEP"));
static AND_STEP: LazyLock<&'static Re> = LazyLock::new(|| rx("AND_STEP"));
static LEAD_PET: LazyLock<String> = LazyLock::new(|| data::get("bot.memory.limit.LEADS").get("pet").unwrap().str().to_string());
static LONGEST: LazyLock<usize> = LazyLock::new(|| data::get("bot.memory.limit.LONGEST").as_int().unwrap() as usize);
static A_FAULT_IN_A_NAMED_THING: LazyLock<&'static Re> = LazyLock::new(|| rx("A_FAULT_IN_A_NAMED_THING"));
static A_REASON: LazyLock<&'static Re> = LazyLock::new(|| rx("A_REASON"));
static WHERE_IT_IS: LazyLock<&'static Re> = LazyLock::new(|| rx("WHERE_IT_IS"));
static A_PET_IN_THE_REASON: LazyLock<&'static Re> = LazyLock::new(|| rx("A_PET_IN_THE_REASON"));
static _MOVE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("_MOVE_WORDS"));
static _LIMIT_FRAME: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("_LIMIT_FRAME"));
static _LEAVES_IT_OUT: LazyLock<&'static Re> = LazyLock::new(|| rx("_LEAVES_IT_OUT"));
static JOINING: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("JOINING"));
static SAID_OF_IT: LazyLock<&'static Re> = LazyLock::new(|| rx("SAID_OF_IT"));
static MEASURED: LazyLock<&'static Re> = LazyLock::new(|| rx("MEASURED"));
static MEASURED_PARTS: LazyLock<&'static Re> = LazyLock::new(|| rx("MEASURED_PARTS"));
static AN_AMOUNT: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("AN_AMOUNT"));
static MOVE: LazyLock<&'static Re> = LazyLock::new(|| rx("MOVE"));
static MOVE_ASKS: LazyLock<HashSet<String>> = LazyLock::new(|| words_set("MOVE_ASKS"));
static APPLIANCES: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.limit.APPLIANCES"));
static NO_APPLIANCE: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.limit.NO_APPLIANCE"));
static HOLDS_ON: LazyLock<&'static Re> = LazyLock::new(|| rx("HOLDS_ON"));
static WORDS_IN_A_REPLY: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.memory.limit.WORDS_IN_A_REPLY").as_int().unwrap() as usize);
static WHO_TO_CALL: LazyLock<&'static Re> = LazyLock::new(|| rx("WHO_TO_CALL"));
static TIME_TAIL: LazyLock<&'static Re> = LazyLock::new(|| rx("TIME_TAIL"));
static A_TIME: LazyLock<&'static Re> = LazyLock::new(|| rx("A_TIME"));
static TRIED_FAULT: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.tried.FAULT"));
static STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.STOPWORDS"));

static LETTERS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", re::U));
static A_TRAILING_WORD: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\s+(?:left|anyway|though|tho|either|now|anymore|for that|for it)$", re::I | re::U));
static NEXT_CLAUSE: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\s+(?:but|and|though|although|because|so|unless|whereas)\s+(?=(?:i|we|you|they|it|there|he|she)\b)", re::I | re::U)
});
static A_COPULA: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:is|are|was|were|isn'?t|aren'?t)\b", re::I | re::U));
static SENTENCE_END: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<=[.!?])\s+", re::U));
static A_NEW_STEP_COMMA: LazyLock<Re> = LazyLock::new(|| Re::new(r",\s+(?=[a-z])", re::U));
static A_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\d", re::U));
static JOINING_WORD: LazyLock<Re> = LazyLock::new(|| Re::new(r"^(?:and|then|so|but|also)\s+", re::I | re::U));

const THING_EDGE: &str = " .,!?;:'\"";

/// Python: bot/memory/limit.py:159 `words_of(text)`
pub fn words_of(text_: &str) -> Vec<String> {
    LETTERS.findall_str(&text_.to_lowercase())
}

/// Python: bot/memory/limit.py:176 `_a_fault_in_the_thing(said, found)`
/// True when the words before the lack (which starts at byte `start`) name the thing that
/// lacks it: "my shower has no water pressure" is a shower doing something wrong.
pub fn _a_fault_in_the_thing(said: &str, start: usize) -> bool {
    A_FAULT_IN_A_NAMED_THING.is_match(&said[..start])
}

/// Python: bot/memory/limit.py:187 `_shown(match)`
/// The thing a limit names, as said, without a trailing word, bare, and as the pattern found it.
pub fn _shown(found: &Match) -> (String, String, String) {
    for group in ["thing", "thing2", "thing3"] {
        let Some(said_raw) = found.name(group).filter(|g| !g.is_empty()) else { continue };
        let raw = text::squash(said_raw);
        let said = A_TRAILING_WORD.sub("", &raw);
        let said = text::strip_chars(&said, THING_EDGE).to_string();
        // A limit is often said with the next thing after it: "no oven cleaner but i have vinegar".
        let said = NEXT_CLAUSE.split(&said).swap_remove(0);
        let said = _without_a_reason(&said);
        let said = text::strip_chars(&said, THING_EDGE).to_string();
        // A thing is followed by where it was: "no yeast in the house".
        let said = text::strip_chars(&WHERE_IT_IS.sub("", &said), THING_EDGE).to_string();
        let bare = _bare(&said);
        return (said, bare, raw);
    }
    (String::new(), String::new(), String::new())
}

/// Python: bot/memory/limit.py:228 `_without_a_reason(said)`
/// The thing without the reason they said after it: "fan" from "fan because the dog chews".
pub fn _without_a_reason(said: &str) -> String {
    text::strip_chars(&A_REASON.sub("", said), THING_EDGE).to_string()
}

/// Python: bot/memory/limit.py:242 `_a_pet_in_the_reason(said, found)`
/// The animal or child the limit was explained by, or "" when the reason names none. (Python's
/// `found` is not read.)
pub fn _a_pet_in_the_reason(said: &str) -> String {
    A_PET_IN_THE_REASON.search(said).map(|hit| hit.name_str("who").to_string()).unwrap_or_default()
}

/// Python: bot/memory/limit.py:249 `_bare(said)`
/// "oven cleaner" from "the oven cleaner".
pub fn _bare(said: &str) -> String {
    let words: Vec<&str> = text::split_ref(said)
        .into_iter()
        .filter(|w| !matches!(w.to_lowercase().as_str(), "the" | "a" | "an" | "any" | "some" | "my" | "our" | "of" | "it"))
        .collect();
    if words.is_empty() { said.to_string() } else { words.join(" ") }
}

/// Python: bot/memory/limit.py:256 `_the_read(said, memory)`
/// The reading of the line, from the turn that already made one, or a fresh one.
pub fn _the_read(said: &str, memory: Option<&Memory>) -> Understood {
    if let Some(read) = memory.and_then(|memory| memory.understood.as_ref()) {
        if text::strip_chars(&text::squash(&read.text), " .,!;:") == said {
            return read.clone();
        }
    }
    understand(said, Some(&Memory::new()))
}

/// Python: bot/memory/limit.py:264 `reads_as_a_limit(text, memory=None)`
/// (kind, the thing as said, the thing as named) for a line that limits the last answer: kind
/// is "ran_out", "not_have" or "pet".
pub fn reads_as_a_limit(text_: &str, memory: Option<&Memory>) -> Option<(String, String, String)> {
    let flat = text::squash(text_);
    let said = text::strip_chars(&flat, " .,!;:");
    if said.is_empty() || words_of(said).len() > *LONGEST {
        return None;
    }
    // A line that asks for a name is an ask, not a limit on the answer above.
    if _the_read(said, memory).asks.iter().any(|ask| ask == "recommend") {
        return None;
    }
    for (kind, pattern) in [("ran_out", *OUT_OF), ("not_have", *NOT_HAVE), ("pet", *IN_THE_HOUSE)] {
        let Some(found) = pattern.search(said) else { continue };
        if kind == "pet" {
            let who = _everyone_in_the_house(said, &found);
            return Some(("pet".into(), who.clone(), who));
        }
        let rest = said.replace(found.as_str(), " ");
        if ASKS.is_match(&rest) {
            return None; // the line has a question of its own, so it is an ask
        }
        if ASKS_A_CHOICE.is_match(&rest) {
            return None; // the line asks for a meal, which is a choice and not a limit
        }
        if _a_fault_in_the_thing(said, found.start()) {
            // The thing they name is the subject of the lack: a fault its own card answers.
            return None;
        }
        let (shown, bare, raw) = _shown(&found);
        if !raw.is_empty() && CLAUSE_STARTS.contains(text::strip_chars(&text::split_ref(&raw)[0].to_lowercase(), ".,!")) {
            return None;
        }
        if A_COPULA.is_match(&bare) {
            return None; // "no my car is fine" is a sentence about a thing, not a limit on one
        }
        if bare.is_empty() || NOT_A_THING.is_match(&bare) || NOT_A_THING.is_match(&raw) {
            return None;
        }
        if ASKS.is_match(&bare) {
            return None;
        }
        let who = _a_pet_in_the_reason(said);
        if !who.is_empty() {
            // A pet named as the reason is one in the house, and only a step a pet rules out
            // makes the reply the pet one.
            let answer = memory.map(|memory| _last_answer(memory).0).unwrap_or_default();
            if !answer.is_empty() && NOT_WITH_A_PET.is_match(&answer) {
                let said_as = if who.ends_with('s') || who == "children" { who } else { format!("a {who}") };
                return Some(("pet".into(), said_as.clone(), said_as));
            }
        }
        return Some((kind.into(), shown, bare));
    }
    None
}

/// Python: bot/memory/limit.py:345 `only_a_limit(text, memory=None)`
/// True when every word of the line is the limit and none of it is a subject of its own.
pub fn only_a_limit(text_: &str, _memory: Option<&Memory>) -> bool {
    let flat = text::squash(text_);
    let said = text::strip_chars(&flat, " .,!;:");
    if said.is_empty() || words_of(said).len() > *LONGEST {
        return false;
    }
    if _LEAVES_IT_OUT.match_(said).is_none() {
        return false;
    }
    // A limit on the money is about what kind of answer to give; only a limit on the feet names
    // a thing to leave out of the answer above.
    match pyv::attempt(|| super::facts::a_limit_on_the_plan(said)) {
        Some(Some(kind)) if kind == "move" => {}
        _ => return false,
    }
    let words: Vec<String> =
        LETTERS.findall_str(&said.to_lowercase()).iter().map(|w| text::strip_chars(w, "'").to_string()).collect();
    !words.is_empty() && words.iter().all(|w| _MOVE_WORDS.contains(w) || _LIMIT_FRAME.contains(w))
}

/// Python: bot/memory/limit.py:369 `_everyone_in_the_house(line, found)`
/// "a dog and a toddler" from a line that names both of them.
pub fn _everyone_in_the_house(line: &str, found: &Match) -> String {
    let mut names: Vec<String> = vec![found.name("who").or(found.name("who2")).unwrap_or("").to_string()];
    let mut rest = &line[found.end()..];
    while let Some(more) = ALSO_THERE.match_(rest) {
        names.push(more.name_str("who3").to_string());
        rest = &rest[more.end()..];
    }
    let mut seen = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for name in names {
        let name = text::squash(&name);
        if name.is_empty() || !seen.insert(name.to_lowercase()) {
            continue;
        }
        // Said as a person says it: "with a dog and a toddler in the house".
        out.push(if name.ends_with('s') || name == "children" { name } else { format!("a {name}") });
    }
    out.join(" and ")
}

/// Python: bot/memory/limit.py:390 `_pieces(answer)`
/// The answer as sentences, each split at the commas that open a new step.
pub fn _pieces(answer: &str) -> Vec<(String, Vec<String>)> {
    let mut pieces = Vec::new();
    for sentence in SENTENCE_END.split(answer) {
        let sentence = text::strip(&sentence).to_string();
        if sentence.is_empty() {
            continue;
        }
        let clauses: Vec<String> = A_NEW_STEP_COMMA
            .split(&sentence)
            .iter()
            .map(|clause| text::strip(clause).to_string())
            .filter(|clause| !clause.is_empty())
            .collect();
        pieces.push((sentence, clauses));
    }
    pieces
}

/// The pieces of a clause split at an "and" that joins two steps, trimmed, empties dropped.
fn steps_of_clause(clause: &str) -> Vec<String> {
    AND_STEP
        .split(clause)
        .iter()
        .map(|step| text::strip_chars(step, " .,;:").to_string())
        .filter(|step| !step.is_empty())
        .collect()
}

/// Python: bot/memory/limit.py:403 `_steps(answer)`
/// `[(clause, sentence)]`: the steps of an answer, in the order they were given.
pub fn _steps(answer: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (sentence, clauses) in _pieces(answer) {
        for clause in &clauses {
            for step in steps_of_clause(clause) {
                out.push((step, sentence.clone()));
            }
        }
    }
    out
}

/// Python: bot/memory/limit.py:422 `_names(clause, thing)`
/// Whether a clause names the thing, by that thing's own forms, with the joining words left
/// out and either way round.
pub fn _names(clause: &str, thing: &str) -> bool {
    let forms = topics::forms(thing);
    let words: HashSet<String> = words_of(clause).into_iter().collect();
    if forms.iter().any(|form| words.contains(form)) {
        return true;
    }
    let want: Vec<String> = words_of(thing).into_iter().filter(|w| !JOINING.contains(w)).collect();
    if want.len() < 2 {
        return false;
    }
    let got: Vec<String> = words_of(clause).into_iter().filter(|w| !JOINING.contains(w)).collect();
    if got.len() < want.len() {
        return false;
    }
    (0..=got.len() - want.len()).any(|start| {
        got[start..start + want.len()].iter().zip(&want).all(|(one, other)| one == other || topics::forms(other).contains(one))
    })
}

/// Python: bot/memory/limit.py:445 `_first_with(answer, thing)`
/// The first step of the answer that names the thing, or "".
pub fn _first_with(answer: &str, thing: &str) -> String {
    _steps(answer).into_iter().map(|(clause, _)| clause).find(|clause| _names(clause, thing)).unwrap_or_default()
}

/// Python: bot/memory/limit.py:487 `_the_amount_of_it(clause, thing)`
/// True when the thing is said as an amount: "the same weight of soft butter", "325ml water".
pub fn _the_amount_of_it(clause: &str, thing: &str) -> bool {
    let words = words_of(clause);
    let wanted: HashSet<String> = words_of(thing).into_iter().filter(|w| !JOINING.contains(w)).collect();
    for (place, word) in words.iter().enumerate() {
        if !wanted.contains(word) {
            continue;
        }
        let before = &words[place.saturating_sub(4)..place];
        if before.iter().any(|one| AN_AMOUNT.contains(one)) || A_DIGIT.is_match(&before.join(" ")) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/limit.py:500 `_among_measured_things(clause, thing)`
/// True when the thing is one item of a measured list: "500g flour, 7g yeast, 10g salt".
pub fn _among_measured_things(clause: &str, thing: &str) -> bool {
    let words = clause.to_lowercase();
    if thing.is_empty() || !MEASURED.is_match(&words) {
        return false;
    }
    let parts: Vec<String> =
        MEASURED_PARTS.split(&words).iter().map(|part| text::strip(part).to_string()).filter(|p| !p.is_empty()).collect();
    if parts.len() < 3 {
        return false;
    }
    parts.iter().any(|part| _names(part, thing))
}

/// Python: bot/memory/limit.py:521 `is_a_thing_on_a_list(clause, thing='')`
/// True for a step of an answer that only names a thing ("7g dried yeast"): a list of what a
/// job is made of is not a set of steps.
pub fn is_a_thing_on_a_list(clause: &str, thing: &str) -> bool {
    let mut said = text::strip(clause).to_string();
    let tidied = _tidy(&said);
    if !tidied.is_empty() && !MEASURED.is_match(&tidied) {
        said = tidied;
    }
    if said.is_empty() {
        return false;
    }
    if _among_measured_things(&said, thing) || _the_amount_of_it(&said, thing) {
        return true;
    }
    STEP.match_(&said).is_none() && !SAID_OF_IT.is_match(&said)
}

/// Python: bot/memory/limit.py:546 `after_an_answer(memory)`
/// True when the turn before this one was an answer to a how-to, not the bot's own chatter.
pub fn after_an_answer(memory: &Memory) -> bool {
    tried::after_an_answer(memory)
}

/// Python: bot/memory/limit.py:551 `_last_answer(memory)`
/// (the bot's last line, the line it answered).
pub fn _last_answer(memory: &Memory) -> (String, String) {
    let turns = &memory.turns;
    let Some(last) = turns.last() else { return (String::new(), String::new()) };
    let asked = if turns.len() > 1 {
        turns[turns.len() - 2].user.clone()
    } else {
        last_answer_question(memory)
    };
    (last.bot.clone(), asked)
}

/// `str((memory.get("last_answer") or {}).get("question") or "")`.
fn last_answer_question(memory: &Memory) -> String {
    match memory.get("last_answer") {
        Some(record) if record.truthy() => pyv::str_or_empty(pyv::get(record, "question")),
        _ => String::new(),
    }
}

/// Python: bot/memory/limit.py:561 `reads_the_answer(text, memory)`
/// True when this limit is answered from the last answer's own steps: a thing the answer asks
/// for, or a product missing after a how-to.
pub fn reads_the_answer(text_: &str, memory: &Memory) -> bool {
    if !after_an_answer(memory) {
        return false;
    }
    let Some((kind, _thing, bare)) = reads_as_a_limit(text_, Some(memory)) else { return false };
    let (answer, _asked) = _last_answer(memory);
    if answer.is_empty() {
        return false;
    }
    if kind == "pet" {
        return _with_a_pet(&answer, &bare).is_some();
    }
    let named = _first_with(&answer, &bare);
    if !named.is_empty() {
        return !is_a_thing_on_a_list(&named, &bare);
    }
    if _without_the_appliance(&bare, &answer, memory).is_some() {
        // The answer never says the appliance; this module has the answer for it.
        return true;
    }
    tried::asks_for_a_fix(memory)
        && (_about_the_job(&bare, memory) || _a_fault_in_the_line_answered(memory))
        && !a_detail_of_a_fault(memory, &bare)
}

/// Python: bot/memory/limit.py:591 `_the_line_answered(memory)`
/// The person's line that the answer above was written for.
pub fn _the_line_answered(memory: &Memory) -> String {
    match memory.turns.iter().rev().find(|turn| !turn.bot.is_empty()) {
        Some(turn) => turn.user.clone(),
        None => last_answer_question(memory),
    }
}

/// Python: bot/memory/limit.py:600 `_a_fault_in_the_line_answered(memory)`
/// True when the answer above was written for a thing that has gone wrong.
pub fn _a_fault_in_the_line_answered(memory: &Memory) -> bool {
    let reported = _the_line_answered(memory);
    !reported.is_empty() && TRIED_FAULT.is_match(&reported.to_lowercase())
}

/// The forms of the content words (longer than three letters) of a line.
fn content_forms(said: &str) -> Vec<HashSet<String>> {
    words_of(said).into_iter().filter(|w| text::len(w) > 3 && !STOPWORDS.contains(w)).map(|w| topics::forms(&w)).collect()
}

/// Python: bot/memory/limit.py:606 `_about_the_job(bare, memory)`
/// True when the thing they lack is part of the job the answer above was for.
pub fn _about_the_job(bare: &str, memory: &Memory) -> bool {
    let named_words: HashSet<String> = content_forms(&_the_line_answered(memory)).into_iter().flatten().collect();
    if named_words.is_empty() {
        return false;
    }
    content_forms(bare).iter().any(|forms| forms.iter().any(|form| named_words.contains(form)))
}

/// Python: bot/memory/limit.py:613 `reply_for(text, memory)`
/// The reply a limit on the last answer deserves, or None to leave the turn alone. The steps
/// the limit left are kept in `memory["limit"]` for the next short ask.
pub fn reply_for(text_: &str, memory: &mut Memory) -> Option<String> {
    let reply = _limit_reply(text_, memory)?;
    let steps = _steps_the_limit_left(text_, memory);
    let mut record = Value::dict();
    record.set_key("said", reply.as_str());
    record.set_key("steps", Value::from(steps));
    memory.set("limit", record);
    Some(reply)
}

/// Python: bot/memory/limit.py:628 `move_after_a_limit(text, memory)`
/// The first step the limit left, when the line right after it asks for the move.
pub fn move_after_a_limit(text_: &str, memory: &mut Memory) -> Option<String> {
    let record = memory.get("limit").filter(|record| record.is_dict())?.clone();
    let fresh = memory
        .turns
        .last()
        .is_some_and(|turn| text::strip(&turn.bot) == text::strip(&pyv::str_or_empty(pyv::get(&record, "said"))));
    if !fresh {
        memory.remove("limit"); // a limit reply two turns back is not what this line is about
        return None;
    }
    let line = text::strip(text_);
    if !MOVE.is_match(line) || words_of(line).len() > 8 {
        return None;
    }
    if topics::pairing(line) || !_names_no_thing(line) {
        return None;
    }
    let steps: Vec<Value> = {
        let held = pyv::get(&record, "steps");
        if held.truthy() { pyv::iter(held).into_iter().filter(Value::truthy).collect() } else { Vec::new() }
    };
    let first = steps.first()?;
    let step = text::rstrip_chars(text::strip(first.str()), ". ").to_string();
    if step.is_empty() {
        return None;
    }
    Some(format!("I would start here: {}.", lower_first(&step)))
}

/// `s[0].lower() + s[1:]`.
fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => panic!("IndexError: string index out of range"),
    }
}

/// Python: bot/memory/limit.py:658 `_names_no_thing(line)`
/// True when the line names nothing of its own: "whats the first thing to change".
pub fn _names_no_thing(line: &str) -> bool {
    !words_of(line).iter().any(|w| !STOPWORDS.contains(w) && !MOVE_ASKS.contains(w) && text::len(w) > 1)
}

/// Python: bot/memory/limit.py:671 `_steps_the_limit_left(text, memory)`
/// The steps of the last answer that the limit does not rule out.
pub fn _steps_the_limit_left(text_: &str, memory: &Memory) -> Vec<String> {
    let limit = reads_as_a_limit(text_, Some(memory));
    let (answer, _asked) = _last_answer(memory);
    let Some((kind, _thing, bare)) = limit.filter(|_| !answer.is_empty()) else { return Vec::new() };
    let steps = _steps(&answer).into_iter().map(|(step, _)| step);
    if kind == "pet" {
        return steps.filter(|step| !NOT_WITH_A_PET.is_match(step)).collect();
    }
    steps.filter(|step| !_names(step, &bare)).collect()
}

/// Python: bot/memory/limit.py:683 `a_detail_of_a_fault(memory, bare)`
/// True when the line says the fault again ("no hot water" after "my boiler is broken").
pub fn a_detail_of_a_fault(memory: &Memory, bare: &str) -> bool {
    if A_DETAIL_OF_THE_FAULT.match_(&text::squash(bare)).is_none() {
        return false;
    }
    // The fault being answered is the person's line of the last turn with a reply of its own.
    let reported = memory.turns.iter().rev().find(|turn| !turn.bot.is_empty()).map(|turn| turn.user.clone()).unwrap_or_default();
    if reported.is_empty() {
        return false;
    }
    let line = reported.to_lowercase();
    A_HEATING_FAULT.is_match(&line) && TRIED_FAULT.is_match(&line) && !A_LEAKING_FAULT.is_match(&line) && !tried::buying(memory)
}

/// Python: bot/memory/limit.py:722 `_without_the_appliance(thing, answer, memory)`
/// What to cook without it, when the answer in front needs the appliance that has gone.
pub fn _without_the_appliance(thing: &str, answer: &str, memory: &Memory) -> Option<String> {
    let name = text::strip(thing).to_lowercase();
    if !APPLIANCES.contains(&name) {
        return None;
    }
    let asked = format!("what can i cook with no {name}");
    // Python hands the kitchen a dict with only these four keys.
    let understood = Understood {
        clean: asked.clone(),
        words: asked.split(' ').map(str::to_string).collect(),
        excludes: vec![name.clone()],
        constraint: true,
        ..Understood::default()
    };
    if crate::knowledge::cooking_b::missing(&understood).is_empty() {
        return None;
    }
    if !crate::knowledge::cooking_b::unusable(answer, &understood) {
        return None;
    }
    let alternative = crate::knowledge::cooking_b::without(&understood).filter(|a| !a.is_empty())?;
    let alternative = text::upper_first(&alternative);
    Some(pyv::pick(&NO_APPLIANCE, memory, &name).replace("{thing}", &name).replace("{answer}", &alternative))
}

/// Python: bot/memory/limit.py:738 `_limit_reply(text, memory)`
/// The reply a limit on the last answer deserves, or None to leave the turn alone.
pub fn _limit_reply(text_: &str, memory: &Memory) -> Option<String> {
    if !after_an_answer(memory) {
        return None;
    }
    let (kind, thing, bare) = reads_as_a_limit(text_, Some(memory))?;
    let (answer, _asked) = _last_answer(memory);
    if answer.is_empty() {
        return None;
    }
    if kind == "pet" {
        return _with_a_pet(&answer, &bare);
    }
    if a_detail_of_a_fault(memory, &bare) {
        return None;
    }
    let steps = _steps(&answer);
    if steps.is_empty() {
        return None;
    }
    let named = _first_with(&answer, &bare);
    if named.is_empty() {
        // An appliance the answer's instructions need is not a thing inside it.
        if let Some(alternative) = _without_the_appliance(&bare, &answer, memory) {
            return Some(alternative);
        }
        // The answer never asks for the thing: only a shortage after a fault gets the outright line.
        if !_a_fault_in_the_line_answered(memory) || !tried::asks_for_a_fix(memory) {
            return None;
        }
        return Some(_say(&format!("You don't need {thing} for that: "), &_up_to_words(&_kept(&answer, &bare))));
    }
    if is_a_thing_on_a_list(&named, &bare) {
        // Bread without the yeast is not bread with one step fewer.
        return None;
    }
    if _names(&steps[0].0, &bare) && _holds_on(&answer, &bare) {
        // What is left of the answer is done to what the missing step made.
        return Some(_needs_it(&thing, &answer));
    }
    let kept = _without_a_repeated_time(&_kept(&answer, &bare));
    if kept.is_empty() {
        return None;
    }
    // Said the way they said it: "bicarbonate of soda", not the bare form it is filed under.
    let verbatim = if _names(&named, &thing) { thing } else { _bare_phrase(&named, &bare) };
    Some(_say(&format!("Skip the {verbatim}: "), &_up_to_words(&kept)))
}

/// Python: bot/memory/limit.py:810 `_kept(answer, bare)`
/// The answer with the steps that name the thing cut out, one entry for each sentence left.
pub fn _kept(answer: &str, bare: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (_sentence, clauses) in _pieces(answer) {
        let kept: Vec<String> = clauses.iter().flat_map(|clause| steps_of_clause(clause)).filter(|step| !_names(step, bare)).collect();
        if !kept.is_empty() {
            out.push(kept.join(", "));
        }
    }
    out
}

/// Python: bot/memory/limit.py:840 `_without_a_repeated_time(sentences)`
/// Drop a closing time estimate when the steps left already say how long something takes.
pub fn _without_a_repeated_time(sentences: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for sentence in sentences {
        if !out.is_empty() && TIME_TAIL.match_(text::strip(sentence)).is_some() && A_TIME.is_match(&out.join(" ")) {
            continue;
        }
        out.push(sentence.clone());
    }
    out
}

/// Python: bot/memory/limit.py:850 `_up_to_words(sentences)`
/// The first of them, and as many more as a reply can carry.
pub fn _up_to_words(sentences: &[String]) -> Vec<String> {
    let (mut out, mut count) = (Vec::new(), 0);
    for sentence in sentences {
        let words = words_of(sentence).len();
        if !out.is_empty() && count + words > *WORDS_IN_A_REPLY {
            break;
        }
        out.push(sentence.clone());
        count += words;
    }
    out
}

/// Python: bot/memory/limit.py:862 `_holds_on(answer, bare)`
/// True when what the answer has left is done to what the missing step makes.
pub fn _holds_on(answer: &str, bare: &str) -> bool {
    _kept(answer, bare).first().is_some_and(|first| HOLDS_ON.is_match(first))
}

/// Python: bot/memory/limit.py:868 `_needs_it(thing, answer)`
/// The reply when the step that is out is the one the rest of the answer is done to.
pub fn _needs_it(thing: &str, answer: &str) -> String {
    let lead = format!("You can't do the first step without the {thing}, and the rest of that answer is done after it. ");
    for (sentence, _clauses) in _pieces(answer).into_iter().rev() {
        if WHO_TO_CALL.is_match(&sentence) && !_names(&sentence, thing) {
            return lead + text::strip(&sentence);
        }
    }
    text::rstrip(&lead).to_string()
}

/// Python: bot/memory/limit.py:878 `_bare_phrase(named, bare)`
/// "plunger" from "Start with a plunger", so the step can be named in six words.
pub fn _bare_phrase(named: &str, bare: &str) -> String {
    let forms = topics::forms(bare);
    words_of(named).into_iter().find(|word| forms.contains(word)).unwrap_or_else(|| bare.to_string())
}

/// Python: bot/memory/limit.py:887 `_with_a_pet(answer, who)`
/// The steps a cat or a child does not rule out, said with the reason they are missing.
pub fn _with_a_pet(answer: &str, who: &str) -> Option<String> {
    let steps: Vec<String> = _steps(answer).into_iter().map(|(step, _)| step).collect();
    let hazard: Vec<&String> = steps.iter().filter(|step| NOT_WITH_A_PET.is_match(step)).collect();
    let safe: Vec<String> = steps.iter().filter(|step| !NOT_WITH_A_PET.is_match(step)).cloned().collect();
    if hazard.is_empty() || safe.is_empty() {
        return None;
    }
    let named = NOT_WITH_A_PET.search(hazard[0]).unwrap().as_str().to_lowercase();
    let lead = format!("{}skip the {named}: ", LEAD_PET.replace("{who}", who));
    Some(_say(&lead, &safe))
}

/// Python: bot/memory/limit.py:899 `_say(lead, steps)`
/// The lead and up to two steps, said as one sentence somebody would want to read.
pub fn _say(lead: &str, steps: &[String]) -> String {
    let mut said = _tidy(&steps[0]);
    if steps.len() > 1 && !said.ends_with(['.', '!', '?']) {
        let second = _tidy(&steps[1]);
        // A capital is the answer starting a new sentence, so it goes on as one.
        let opens_upper = second.chars().next().is_some_and(|c| text::is_upper(c.encode_utf8(&mut [0; 4])));
        said += &if opens_upper { format!(". {second}") } else { format!(", {second}") };
    }
    let mut said = text::rstrip_chars(&said, " ,;:").to_string();
    if !said.ends_with(['.', '!', '?']) {
        said.push('.');
    }
    if lead.is_empty() { text::upper_first(&said) } else { format!("{lead}{}", lower_first(&said)) }
}

/// Python: bot/memory/limit.py:913 `_tidy(step)`
/// The step without the joining word it was cut with.
pub fn _tidy(step: &str) -> String {
    text::strip(&JOINING_WORD.sub("", text::strip(step))).to_string()
}
