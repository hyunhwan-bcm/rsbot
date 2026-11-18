//! Port of `bot/memory/again.py`: the same paragraph is about to be said twice: say the short of
//! it, framed as the repeat it is.
//!
//! Every frame (lead-in) the codebase puts in front of an answer is here, with the readers that
//! recognise a framed reply, and `instead` - the floor every module's reply goes through, which
//! frames a paragraph the chat has already had. Module data (the frames, their patterns, the lead
//! words) comes from the exported bundle.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::topics;
use crate::py::re::Re;
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

const I: u32 = crate::py::re::I;

fn g(name: &str) -> &'static Value {
    data::get(&format!("bot.memory.again.{name}"))
}
fn strs(name: &str) -> Vec<String> {
    g(name).strs()
}
fn re(name: &str) -> &'static Re {
    g(name).as_re().unwrap_or_else(|| panic!("bot.memory.again.{name} is not a regex"))
}

/// `SHORT`: a reply shorter than this many words is already the short of itself.
static SHORT: LazyLock<usize> = LazyLock::new(|| g("SHORT").as_int().unwrap_or(14) as usize);
/// `A_PHRASE`: a phrase longer than this is a sentence about something.
static A_PHRASE: LazyLock<usize> = LazyLock::new(|| g("A_PHRASE").as_int().unwrap_or(8) as usize);

static AGAIN: LazyLock<Vec<String>> = LazyLock::new(|| strs("AGAIN"));
static SHORT_AGAIN: LazyLock<Vec<String>> = LazyLock::new(|| strs("SHORT_AGAIN"));
static RETURN: LazyLock<Vec<String>> = LazyLock::new(|| strs("RETURN"));
static UNDERNEATH_SAID: LazyLock<Vec<String>> = LazyLock::new(|| strs("UNDERNEATH_SAID"));
static UNDERNEATH_CLAUSE: LazyLock<Vec<String>> = LazyLock::new(|| strs("UNDERNEATH_CLAUSE"));
pub static FRAMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("FRAMES"));
static FOR_THE_PERSON: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    g("FOR_THE_PERSON").pairs().iter().map(|(k, v)| (k.str().to_string(), v.str().to_string())).collect()
});
static _NOT_THE_END: LazyLock<HashSet<String>> = LazyLock::new(|| g("_NOT_THE_END").str_set());
static FRAME_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| g("FRAME_WORDS").str_set());
/// `PATTERNS`: the lead-in of every frame, compiled.
static PATTERNS: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| g("PATTERNS").items().iter().map(|v| v.as_re().unwrap()).collect());
/// `LEAD_WORDS`: the lead-ins of more than one word, as word tuples.
static LEAD_WORDS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| g("LEAD_WORDS").items().iter().map(Value::strs).collect());
/// `ONE_WORD_LEAD`: a one-word lead with its colon or comma (None when there is none).
static ONE_WORD_LEAD: LazyLock<Option<&'static Re>> = LazyLock::new(|| g("ONE_WORD_LEAD").as_re());

static SENTENCE: LazyLock<&'static Value> = LazyLock::new(|| g("SENTENCE"));
static SLOT: LazyLock<&'static Re> = LazyLock::new(|| re("SLOT"));
static FEELING: LazyLock<&'static Re> = LazyLock::new(|| re("FEELING"));
static CANNOT: LazyLock<&'static Re> = LazyLock::new(|| re("CANNOT"));
static BEING: LazyLock<&'static Re> = LazyLock::new(|| re("BEING"));
static NOTHING: LazyLock<&'static Re> = LazyLock::new(|| re("NOTHING"));
static CLAUSE_ONLY: LazyLock<&'static Re> = LazyLock::new(|| re("CLAUSE_ONLY"));
static OPENING_FILLER: LazyLock<&'static Re> = LazyLock::new(|| re("OPENING_FILLER"));
static SEPARATOR: LazyLock<&'static Re> = LazyLock::new(|| re("SEPARATOR"));
static A_DETAIL: LazyLock<&'static Re> = LazyLock::new(|| re("A_DETAIL"));
static ANOTHER_ONE: LazyLock<&'static Re> = LazyLock::new(|| re("ANOTHER_ONE"));
static A_MONEY_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("A_MONEY_ASK"));
static MONEY_SAID: LazyLock<&'static Re> = LazyLock::new(|| re("MONEY_SAID"));
static AN_AMOUNT: LazyLock<&'static Re> = LazyLock::new(|| re("AN_AMOUNT"));
static A_REPORT: LazyLock<&'static Re> = LazyLock::new(|| re("A_REPORT"));
static AN_ASK: LazyLock<&'static Re> = LazyLock::new(|| re("AN_ASK"));
static A_STILL_GOING_ON: LazyLock<&'static Re> = LazyLock::new(|| re("A_STILL_GOING_ON"));
static A_SCRIPT: LazyLock<&'static Re> = LazyLock::new(|| re("A_SCRIPT"));
pub static FACT_RECALL: LazyLock<&'static Re> = LazyLock::new(|| re("FACT_RECALL"));
static SAID_BACK: LazyLock<&'static Re> = LazyLock::new(|| re("SAID_BACK"));
static _RULED_OUT: LazyLock<&'static Re> = LazyLock::new(|| re("_RULED_OUT"));

// The other modules' data this file reads.
static TOPICS_REPEAT_LEADS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| data::get("bot.memory.topics.REPEAT_LEADS").items().iter().map(Value::strs).collect());
static TOPICS_STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.STOPWORDS"));
static TOPICS_ASK_ONLY: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.ASK_ONLY"));
static NAMED_CAUSE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.why.NAMED_CAUSE"));
static PLACES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.topics.PLACES"));
static CORE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.understanding.lexicon.CORE_WORDS"));
static UNKNOWN: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.UNKNOWN"));
static WORDING_ASKS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback.WORDING_ASKS"));
static WORDING_SOMEONE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback.WORDING_SOMEONE"));
static WORDING_SAYS: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.compose.fallback.WORDING_SAYS"));

/// Python's `string.punctuation`.
const PUNCTUATION: &str = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

/// `template.format(name=value, ...)` for the `{name}` fields the frames use, read in one pass
/// so a value is never itself read as a field.
fn fmt(template: &str, fields: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 64);
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

/// `seq[i]` with Python's negative index.
fn py_at<T: Clone>(seq: &[T], i: i64) -> T {
    let n = seq.len() as i64;
    let k = if i < 0 { n + i } else { i };
    seq[k as usize].clone()
}

/// `s[:1].upper() + s[1:]`.
fn up1(s: &str) -> String {
    pt::upper_first(s)
}

/// The bot lines of the chat, newest last (`memory["turns"]`).
fn bots(memory: &Memory) -> impl DoubleEndedIterator<Item = &String> {
    memory.turns.iter().map(|t| &t.bot)
}

/// `topics.resumed(memory)` as the record Python returns.
///
/// CONTRACT NOTE: the skeleton gives `topics::resumed` no return type, but PyBot returns the
/// return record (a dict) or None. This adapter compiles against either, so the call keeps working
/// when the owner gives it the type that holds the record.
fn resumed(memory: &Memory) -> Value {
    trait IntoRecord {
        fn into_record(self) -> Value;
    }
    impl IntoRecord for () {
        fn into_record(self) -> Value {
            Value::None
        }
    }
    impl IntoRecord for Value {
        fn into_record(self) -> Value {
            self
        }
    }
    impl IntoRecord for Option<Value> {
        fn into_record(self) -> Value {
            self.unwrap_or(Value::None)
        }
    }
    #[allow(clippy::unit_arg)]
    topics::resumed(memory).into_record()
}

/// Python: bot/memory/again.py:166 `the_name_the_answer_says(answer, detail, card="")`: the longest
/// run of `detail`'s own words that `answer` says, or "" for none.
pub fn the_name_the_answer_says(answer: &str, detail: &str, card: &str) -> String {
    let said = topics::words(answer);
    if said.is_empty() {
        return String::new();
    }
    let mut words = pt::split(detail);
    while let Some(last) = words.last() {
        let last = pt::lower(last);
        if !_NOT_THE_END.contains(&last) && (said.contains(&last) || (pt::len(&last) > 3 && said.iter().any(|w| w.starts_with(last.as_str())))) {
            return words.join(" ");
        }
        words.pop();
    }
    let whole = topics::words(card);
    let phrase = pt::split(detail);
    let last = phrase.last().map(|w| pt::lower(w)).unwrap_or_default();
    let led = phrase.len() > 1 && phrase[..phrase.len() - 1].iter().any(|w| _NOT_THE_END.contains(&pt::lower(w)));
    if led && !_NOT_THE_END.contains(&last) && whole.contains(&last) {
        return pt::strip(detail).to_string();
    }
    String::new()
}

static CLAUSE_PARTS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[,;]|\band\b|\bbut\b|\bso\b", 0));

/// Python: bot/memory/again.py:212 `as_a_phrase(text)`: the person's own words for it as a phrase,
/// and whether it can stand as a subject: (kind, phrase) with kind "said" or "clause", or None.
pub fn as_a_phrase(text: &str) -> Option<(String, String)> {
    if let Some(whole) = _a_phrase_of(text) {
        return Some(whole);
    }
    let clauses: Vec<String> = CLAUSE_PARTS.split(text).into_iter().filter(|p| !pt::strip(p).is_empty()).collect();
    if clauses.len() > 1 {
        for clause in clauses[1..].iter().rev() {
            if let Some(made) = _a_phrase_of(clause) {
                return Some(made);
            }
        }
    }
    None
}

/// Python: bot/memory/again.py:238 `_a_phrase_of(text)`: `as_a_phrase` for one clause of the line.
pub fn _a_phrase_of(text: &str) -> Option<(String, String)> {
    let line = pt::strip(&OPENING_FILLER.sub("", pt::rstrip_chars(pt::strip(text), ".!,"))).to_string();
    if line.is_empty() || pt::split(&line).len() > *A_PHRASE || line.contains('?') {
        return None;
    }
    if let Some(found) = NOTHING.match_(&line) {
        let verb = pt::lower(found.str(1));
        if verb.ends_with('s') && !["is", "was", "has"].contains(&verb.as_str()) {
            return Some(("clause".into(), format!("Nothing {verb} you")));
        }
        return None;
    }
    for (k, pattern) in [&*FEELING, &*CANNOT].into_iter().enumerate() {
        if let Some(found) = pattern.match_(&line) {
            let rest = _for_the_person(found.str(1));
            let words = pt::split(&rest);
            if rest.is_empty() || words.len() > *A_PHRASE - 1 {
                return None;
            }
            if rest.contains(',') || rest.contains(';') || _says_i(found.str(1)) {
                return None;
            }
            if ["to", "that", "like", "as", "so"].contains(&pt::lower(&words[0]).as_str()) {
                return None;
            }
            if k == 1 {
                return Some(("said".into(), format!("Not being able to {rest}")));
            }
            return Some(("said".into(), format!("Feeling {rest}")));
        }
    }
    if CLAUSE_ONLY.match_(&line).is_some() {
        let turned = _for_the_person(&line);
        return Some(("clause".into(), up1(&turned)));
    }
    if let Some(found) = BEING.match_(&line) {
        let rest = _for_the_person(found.str(1));
        if rest.contains(',') {
            return None;
        }
        let words = pt::split(&rest);
        if words.is_empty() || words.len() > 3 {
            return None;
        }
        if pt::lower(words.last().unwrap()).ends_with("ing") || ["going", "gonna", "about", "not"].contains(&pt::lower(&words[0]).as_str()) {
            return None;
        }
        if words.iter().any(|w| ["to", "the", "a", "an", "my", "your"].contains(&pt::lower(w).as_str())) {
            return None;
        }
        return Some(("said".into(), format!("Being {rest}")));
    }
    None
}

fn turned_word(bare: &str) -> Option<&'static str> {
    FOR_THE_PERSON.iter().find(|(k, _)| k == bare).map(|(_, v)| v.as_str())
}

/// Python: bot/memory/again.py:286 `_says_i(text)`: true when the words hold a first person still
/// to turn, so they are more than one clause.
pub fn _says_i(text: &str) -> bool {
    pt::split(text).iter().any(|w| turned_word(&pt::lower(pt::strip_chars(w, PUNCTUATION))).is_some())
}

/// Python: bot/memory/again.py:292 `_for_the_person(text)`: `text` with the first person turned
/// round: "my head is banging" -> "your head is banging".
pub fn _for_the_person(text: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    for word in pt::split(text) {
        let bare = pt::lower(pt::strip_chars(&word, PUNCTUATION));
        let kept = pt::rstrip_chars(&word, PUNCTUATION);
        let ending = &word[kept.len()..];
        match turned_word(&bare) {
            Some(turned) => {
                let turned = if word.chars().next().is_some_and(char::is_uppercase) { pt::capitalize(turned) } else { turned.to_string() };
                words.push(format!("{turned}{ending}"));
            }
            None => words.push(word.clone()),
        }
    }
    words.join(" ")
}

/// Python: bot/memory/again.py:309 `the_same_thing_underneath(text, reply, times, memory=None)`:
/// the answer behind the person's own word for it, once the frame has been said twice, or "".
pub fn the_same_thing_underneath(text: &str, reply: &str, times: i64, memory: &Memory) -> String {
    if pt::strip(reply).is_empty() || times == 0 || times < 2 {
        return String::new();
    }
    if _carries_a_lead_in(&_flat(reply)) || unframed(reply).is_some() {
        return String::new();
    }
    let Some((kind, said)) = as_a_phrase(text) else {
        return String::new();
    };
    let pool: &Vec<String> = if kind == "said" { &UNDERNEATH_SAID } else { &UNDERNEATH_CLAUSE };
    let turns = memory.turns.len();
    fmt(&pool[turns % pool.len()], &[("said", &said), ("reply", pt::strip(reply))])
}

static ANY_SEPARATOR: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.again.ANY_SEPARATOR"));

/// Python's `re.escape` (3.7+): only the characters that mean something in a pattern.
fn py_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        if "()[]{}?*+-|^$\\.&~# \t\n\r\x0b\x0c".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Python: bot/memory/again.py:342 `_lead(frame)`: a pattern for the lead-in `frame` puts in front
/// of its answer.
pub fn _lead(frame: &str) -> Re {
    let mut head = frame.split("{reply}").next().unwrap_or("").to_string();
    let mut tail = String::new();
    let trimmed = pt::rstrip(&head).to_string();
    let stripped = SEPARATOR.sub("", &trimmed);
    if stripped != trimmed {
        head = stripped;
        tail = ANY_SEPARATOR.clone();
    }
    let parts: Vec<String> = SLOT.split(&head).iter().map(|p| py_escape(p)).collect();
    Re::new(&format!("^{}{}", parts.join(".+?"), tail), 0)
}

/// Python: bot/memory/again.py:382 `unframed(reply)`: the answer under a repeat's lead-in, or None
/// when the reply carries none.
pub fn unframed(reply: &str) -> Option<String> {
    for pattern in PATTERNS.iter() {
        if let Some(m) = pattern.match_(reply) {
            let rest = pt::strip(&reply[m.end()..]);
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

/// Python: bot/memory/again.py:392 `_flat(text)`: a reply reduced to its words.
pub fn _flat(text: &str) -> String {
    topics::words(text).join(" ")
}

/// `quoted.sentences(said, SENTENCE)`.
fn sentences(said: &str) -> Vec<String> {
    crate::memory::quotes::sentences(said, *SENTENCE)
}

/// Python: bot/memory/again.py:397 `the_cause_said_again(reply, memory=None)`: the sentence that
/// names the cause, when the line asked why and the answer has one; None otherwise.
pub fn the_cause_said_again(reply: &str, memory: &Memory) -> Option<String> {
    let understood = memory.understood.as_ref()?;
    if understood.wants.as_deref() != Some("reason") {
        return None;
    }
    let named = &**NAMED_CAUSE;
    let candidates = [reply.to_string(), _the_answer_in_hand(reply, memory), _a_card_behind(memory)];
    for said in candidates.iter() {
        let parts: Vec<String> = sentences(said).iter().map(|p| pt::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
        for (at, part) in parts.iter().enumerate() {
            let Some(found) = named.search(part) else {
                continue;
            };
            if pt::char_index(part, found.start()) > 2 || pt::split(part).len() < 6 {
                continue;
            }
            if at == 0 {
                if parts.len() > 1 && _led_an_earlier_answer(part, memory) {
                    return Some(parts[1..].join(" "));
                }
                return None;
            }
            let moves: Vec<&String> = parts[..at].iter().filter(|o| !o.is_empty()).collect();
            if _led_an_earlier_answer(part, memory) {
                return if parts.len() > 1 { Some(parts[1..].join(" ")) } else { None };
            }
            return Some(match moves.first() {
                Some(first) => format!("{part} {first}"),
                None => part.clone(),
            });
        }
    }
    None
}

/// Python: bot/memory/again.py:444 `_a_card_behind(memory)`: the newest reply in this chat that
/// carries a cause sentence of its own, or "".
pub fn _a_card_behind(memory: &Memory) -> String {
    let named = &**NAMED_CAUSE;
    let mut seen = 0;
    for said in bots(memory).rev() {
        if said.is_empty() {
            continue;
        }
        for part in sentences(said) {
            if let Some(found) = named.search(pt::strip(&part)) {
                if pt::char_index(pt::strip(&part), found.start()) <= 2 && pt::split(&part).len() >= 6 {
                    return said.clone();
                }
            }
        }
        seen += 1;
        if seen >= 3 {
            break;
        }
    }
    String::new()
}

/// Python: bot/memory/again.py:470 `_led_an_earlier_answer(sentence, memory)`: whether a reply in
/// this chat opened on this sentence, so it is not news a second time.
pub fn _led_an_earlier_answer(sentence: &str, memory: &Memory) -> bool {
    let said = pt::squash(&pt::lower(sentence));
    if said.is_empty() {
        return false;
    }
    bots(memory).rev().any(|bot| !bot.is_empty() && pt::squash(&pt::lower(bot)).starts_with(&said))
}

/// Python: bot/memory/again.py:488 `_the_answer_in_hand(reply, memory)`: the answer on the screen,
/// when what this turn is building is the opening of it.
pub fn _the_answer_in_hand(reply: &str, memory: &Memory) -> String {
    let mine = pt::split(&pt::lower(reply)).into_iter().take(8).collect::<Vec<_>>().join(" ");
    if pt::len(&mine) < 25 {
        return String::new();
    }
    let mut seen = 0;
    for said in bots(memory).rev() {
        if said.is_empty() {
            continue;
        }
        if pt::squash(&pt::lower(said)).contains(&mine) {
            return said.clone();
        }
        seen += 1;
        if seen >= 3 {
            break;
        }
    }
    String::new()
}

/// The first one or two sentences, never cut inside a quotation (`quotes.closed`).
fn opening(reply: &str, two_when_short_below: usize, by_first_length: bool) -> String {
    let parts = sentences(reply);
    if parts.is_empty() {
        return reply.to_string();
    }
    let first_len = pt::split(&parts[0]).len();
    let take = if by_first_length {
        if first_len >= *SHORT || parts.len() == 1 { 1 } else { 2 }
    } else if first_len < two_when_short_below && parts.len() > 1 {
        2
    } else {
        1
    };
    let kept: Vec<String> = parts[..take].iter().map(|p| pt::strip(p).to_string()).collect();
    crate::memory::quotes::closed(&kept.join(" "), &parts[take..], reply)
}

/// Python: bot/memory/again.py:506 `_short(reply, memory=None)`: the first sentence of an answer,
/// or the first two when the first is only a lead-in.
pub fn _short(reply: &str, memory: Option<&Memory>) -> String {
    if let Some(memory) = memory {
        if let Some(cause) = the_cause_said_again(reply, memory) {
            if !cause.is_empty() {
                return cause;
            }
        }
    }
    opening(reply, 0, true)
}

/// Python: bot/memory/again.py:525 `_the_first_sentence(reply, memory=None)`: the sentence that
/// carries the answer, so a return gives the move rather than the lot.
pub fn _the_first_sentence(reply: &str, memory: &Memory) -> String {
    if let Some(cause) = the_cause_said_again(reply, memory) {
        if !cause.is_empty() {
            return cause;
        }
    }
    opening(reply, 5, false)
}

/// Python: bot/memory/again.py:539 `_carries_a_lead_in(mine)`: true when the reply opens with a
/// repeat's lead-in ("as i said: ...", "same as before: ...").
pub fn _carries_a_lead_in(mine: &str) -> bool {
    let said = topics::words(mine).join(" ");
    if said.is_empty() {
        return false;
    }
    for lead in TOPICS_REPEAT_LEADS.iter().chain(LEAD_WORDS.iter()) {
        if said.starts_with(&format!("{} ", lead.join(" "))) {
            return true;
        }
    }
    if let Some(one) = *ONE_WORD_LEAD {
        if one.match_(pt::lstrip(mine)).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/memory/again.py:552 `carries_a_lead_in(reply)`: true when the reply already opens
/// with a lead-in, so a second one may not go in front.
pub fn carries_a_lead_in(reply: &str) -> bool {
    _carries_a_lead_in(reply)
}

/// Python: bot/memory/again.py:561 `_times_on_the_screen(mine, memory)`: how many turns in a row
/// already said this reply, word for word or under a lead-in.
pub fn _times_on_the_screen(mine: &str, memory: &Memory) -> i64 {
    let mut times = 0;
    for said in bots(memory).rev() {
        if _flat(said) != mine {
            let body = _flat(&unframed(said).unwrap_or_default());
            if body.is_empty() || !mine.starts_with(body.as_str()) {
                break;
            }
        }
        times += 1;
    }
    times
}

/// Python: bot/memory/again.py:588 `_said_earlier(mine, memory)`: true when this answer was
/// already given somewhere earlier in the conversation.
pub fn _said_earlier(mine: &str, memory: &Memory) -> bool {
    !_said_earlier_text(mine, memory).is_empty()
}

/// Python: bot/memory/again.py:600 `_said_earlier_text(mine, memory)`: the bot turn that already
/// says this answer, or "".
pub fn _said_earlier_text(mine: &str, memory: &Memory) -> String {
    if pt::split(mine).len() <= *SHORT {
        return String::new();
    }
    for bot in bots(memory) {
        let said = _flat(bot);
        if said == mine || said.starts_with(&format!("{mine} ")) {
            return said;
        }
    }
    String::new()
}

/// Python: bot/memory/again.py:621 `_names_something_the_answer_says(text, mine)`: true when the
/// line names a thing this answer is about.
pub fn _names_something_the_answer_says(text: &str, mine: &str) -> bool {
    let said: HashSet<String> = topics::words(text)
        .into_iter()
        .filter(|w| pt::len(w) > 2 && !TOPICS_STOPWORDS.contains(w) && !TOPICS_ASK_ONLY.contains(w))
        .collect();
    if said.is_empty() {
        return false;
    }
    let answer: HashSet<String> = pt::split(mine).into_iter().collect();
    said.iter().any(|w| answer.contains(w))
}

/// Python: bot/memory/again.py:643 `_returned_thing(memory, text)`: the thing the person went back
/// to on this turn, when the line names it: "the tap".
pub fn _returned_thing(memory: &Memory, text: &str) -> String {
    let record = resumed(memory);
    if !record.is_dict() {
        return String::new();
    }
    let asked = record.get("asked").filter(|v| v.truthy()).map(value_text).unwrap_or_default();
    let thing = pt::squash(&pt::lower(&asked));
    if thing.is_empty() || pt::split(&thing).len() > 4 {
        return String::new();
    }
    if !_flat(text).contains(&thing) {
        return String::new();
    }
    thing
}

/// `str(value)` for a recorded scalar.
fn value_text(v: &Value) -> String {
    match v {
        Value::Str(s) => s.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => pt::float_repr(*f),
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
        Value::None => "None".into(),
        _ => String::new(),
    }
}

/// Python: bot/memory/again.py:663 `_the_line_asks(memory, text)`: whether the line asks something,
/// read the way the store read the return.
pub fn _the_line_asks(memory: &Memory, text: &str) -> bool {
    let record = resumed(memory);
    if record.is_dict() {
        if let Some(asks) = record.get("asks") {
            if !asks.is_none() {
                return asks.truthy();
            }
        }
    }
    if let Some(said) = &memory.understood {
        return said.kind == "question" || !said.asks.is_empty();
    }
    AN_ASK.is_match(&_flat(text))
}

/// Python: bot/memory/again.py:681 `_adds_nothing_the_answer_did_not_say(thing, text, said,
/// memory)`: whether the return asks again for the answer, rather than asking something it covers.
pub fn _adds_nothing_the_answer_did_not_say(thing: &str, text: &str, said: &str, memory: &Memory) -> bool {
    if !_the_line_asks(memory, text) {
        return false;
    }
    let typed = memory.understood.as_ref().map(|u| u.text.as_str());
    let asked = _flat(match typed {
        Some(t) if !pt::strip(t).is_empty() => t,
        _ => text,
    });
    let thing: HashSet<String> = pt::split(&pt::lower(thing)).into_iter().collect();
    let answer: HashSet<String> = pt::split(&_flat(said)).into_iter().collect();
    let added: Vec<String> = pt::split(&asked)
        .into_iter()
        .filter(|w| pt::is_alpha(w) && pt::len(w) > 2 && !FRAME_WORDS.contains(w) && !thing.contains(w) && !CORE_WORDS.contains(w))
        .collect();
    !added.iter().any(|w| answer.contains(w))
}

/// Python: bot/memory/again.py:718 `_places()`: the place names the vocabulary knows.
pub fn _places() -> Value {
    data::get("bot.understanding.topics.PLACES").clone()
}

/// Python: bot/memory/again.py:727 `_as_a_name(thing)`: the thing with a place in it written as a
/// name: "lisbon" -> "Lisbon", "new york" -> "New York".
pub fn _as_a_name(thing: &str) -> String {
    pt::split(thing).iter().map(|w| if PLACES.contains(w) { up1(w) } else { w.clone() }).collect::<Vec<_>>().join(" ")
}

/// Python: bot/memory/again.py:740 `_without_the_thing_in_front(short, thing)`: the short of the
/// answer with a phrase naming the thing taken off the front.
pub fn _without_the_thing_in_front(short: &str, thing: &str) -> String {
    let said = pt::strip(short);
    let escaped = py_escape(&pt::lower(thing));
    for pattern in [format!(r"^(?:in|at|on|for|about|with|around)\s+{escaped}\s*[:,]?\s+"), format!(r"^{escaped}\s*[:,]\s+")] {
        if let Some(found) = Re::new(&pattern, I).match_(said) {
            let rest = &said[found.end()..];
            if !rest.is_empty() {
                return up1(rest);
            }
        }
    }
    said.to_string()
}

/// Python: bot/memory/again.py:757 `_said_recently(mine, memory, look=3)`: 1 when this reply is
/// among the last few bot turns word for word, turns between or not.
pub fn _said_recently(mine: &str, memory: &Memory, look: i64) -> i64 {
    let said: Vec<String> = bots(memory).map(|b| _flat(b)).filter(|s| !s.is_empty()).collect();
    let from = said.len().saturating_sub(look.max(0) as usize);
    if said[from..].iter().any(|s| s == mine) {
        1
    } else {
        0
    }
}

/// Python: bot/memory/again.py:782 `_says_it_came_back(text)`: whether the line reports the thing
/// the last answer was about coming back (`bot/memory/returned`).
pub fn _says_it_came_back(text: &str) -> bool {
    crate::memory::returned::says_it_happened_again(text)
}

/// Python: bot/memory/again.py:822 `_a_figure_asked_that_the_answer_does_not_hold(text, reply)`:
/// whether the line asks for an amount of money and the reply holds no amount.
pub fn _a_figure_asked_that_the_answer_does_not_hold(text: &str, reply: &str) -> bool {
    let line = pt::squash(text);
    if line.is_empty() || !A_MONEY_ASK.is_match(&line) || !MONEY_SAID.is_match(&line) {
        return false;
    }
    !AN_AMOUNT.is_match(reply)
}

/// Python: bot/memory/again.py:837 `_asks_for_a_detail(text)`: true when the line asks for a time,
/// an amount or a count.
pub fn _asks_for_a_detail(text: &str) -> bool {
    A_DETAIL.is_match(text)
}

/// Python: bot/memory/again.py:870 `_a_report(text)`: true when the line tells the bot something
/// about the person rather than asking for a part.
pub fn _a_report(text: &str) -> bool {
    if text.is_empty() || !A_REPORT.is_match(text) || AN_ASK.is_match(text) {
        return false;
    }
    !A_STILL_GOING_ON.is_match(text)
}

/// Python: bot/memory/again.py:892 `_has_a_script(reply)`: whether the answer holds a sentence in
/// quotes, which is the thing a wording ask wants.
pub fn _has_a_script(reply: &str) -> bool {
    A_SCRIPT.is_match(reply)
}

/// Python: bot/memory/again.py:897 `_the_line_that_got_it(mine, memory, look=3)`: the line the
/// answer `mine` was given to, among the last few turns, or "".
pub fn _the_line_that_got_it(mine: &str, memory: &Memory, look: i64) -> String {
    let n = memory.turns.len();
    let from = n.saturating_sub(look.max(0) as usize);
    for turn in memory.turns[from..].iter().rev() {
        if _flat(&turn.bot) == mine {
            return pt::squash(&turn.user);
        }
    }
    String::new()
}

/// Python: bot/memory/again.py:908 `_said_and_more(mine, memory)`: true when the bot's last line was
/// this reply and then a question of its own.
pub fn _said_and_more(mine: &str, memory: &Memory) -> bool {
    if mine.is_empty() {
        return false;
    }
    let Some(last) = memory.turns.last() else {
        return false;
    };
    let raw = pt::strip(&last.bot);
    let said = _flat(raw);
    if said.is_empty() || !said.starts_with(&format!("{mine} ")) || !raw.ends_with('?') {
        return false;
    }
    true
}

/// Python: bot/memory/again.py:949 `already_had(reply, memory)`: the reply behind a lead-in that
/// says the chat has had this answer, or None.
pub fn already_had(reply: &str, memory: &Memory) -> Option<String> {
    let mine = _flat(reply);
    if mine.is_empty() {
        return None;
    }
    if _carries_a_lead_in(&mine) || pt::split(&mine).len() <= *SHORT {
        return None;
    }
    if !_said_earlier(&mine, memory) {
        return None;
    }
    Some(fmt(&AGAIN[0], &[("reply", pt::strip(reply))]))
}

/// Python: bot/memory/again.py:970 `_as_said(text)`: the content words of a line in the person's
/// own spelling, in the order they said them.
pub fn _as_said(text: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for word in topics::words(text) {
        if pt::len(pt::rstrip_chars(&word, "s")) > 2 && !TOPICS_STOPWORDS.contains(&word) && !TOPICS_ASK_ONLY.contains(&word) {
            if !seen.contains(&word) {
                seen.push(word);
            }
        }
    }
    seen
}

/// `word.rstrip("s") or word`.
fn stem(word: &str) -> String {
    let s = pt::rstrip_chars(word, "s");
    if s.is_empty() { word.to_string() } else { s.to_string() }
}

/// Python: bot/memory/again.py:981 `_content_words_in_order(text)`: the content words of a line as
/// the matcher compares them, in the order they were said.
pub fn _content_words_in_order(text: &str) -> Vec<String> {
    _as_said(text).iter().map(|w| stem(w)).collect()
}

/// Python: bot/memory/again.py:997 `_the_run_they_said_both_times(text, earlier)`: the longest run
/// of the line's own words that the earlier line used too, as they said it.
pub fn _the_run_they_said_both_times(text: &str, earlier: &HashSet<String>) -> String {
    let said = match _RULED_OUT.search(text) {
        Some(stop) => &text[..stop.start()],
        None => text,
    };
    let mut best: Vec<String> = Vec::new();
    let mut run: Vec<String> = Vec::new();
    for word in topics::words(said) {
        if earlier.contains(&stem(&word)) {
            if !run.contains(&word) {
                run.push(word.clone());
            }
            if run.len() > best.len() {
                best = run.clone();
            }
        } else if pt::len(pt::rstrip_chars(&word, "s")) > 2 && !TOPICS_STOPWORDS.contains(&word) && !TOPICS_ASK_ONLY.contains(&word) {
            run.clear();
        }
    }
    if best.len() > 1 {
        best.join(" ")
    } else {
        String::new()
    }
}

/// Python: bot/memory/again.py:1024 `_the_same_question_asked_again(mine, text, memory)`: the thing
/// the line asks about again, when an earlier turn asked the same question.
pub fn _the_same_question_asked_again(mine: &str, text: &str, memory: &Memory) -> String {
    let asked: HashSet<String> = _content_words_in_order(text).into_iter().collect();
    if asked.len() < 2 {
        return String::new();
    }
    for turn in &memory.turns {
        let said = _flat(&turn.bot);
        if said != mine && !said.starts_with(&format!("{mine} ")) {
            continue;
        }
        let earlier: HashSet<String> = _content_words_in_order(&turn.user).into_iter().collect();
        if earlier.len() < 2 {
            continue;
        }
        let shared = asked.intersection(&earlier).count();
        let all = asked.union(&earlier).count();
        if shared * 4 < all * 3 {
            continue;
        }
        return _the_run_they_said_both_times(text, &earlier);
    }
    String::new()
}

/// Python: bot/memory/again.py:1057 `_the_subject_on_the_screen(memory)`: the subject the
/// conversation is on, in the person's words ("my back"), or "".
pub fn _the_subject_on_the_screen(memory: &Memory) -> String {
    if let Some(returned) = memory.get("returned").filter(|r| r.is_dict()) {
        if let Some(Value::Int(turn)) = returned.get("turn") {
            if *turn >= memory.turns.len() as i64 - 2 {
                let asked = returned.get("asked").filter(|v| v.truthy()).map(value_text).unwrap_or_default();
                let subject = pt::squash(&asked);
                if !subject.is_empty() {
                    return _as_your(&subject);
                }
            }
        }
    }
    let subject = match memory.get("topic") {
        Some(record) if record.is_dict() => {
            pt::strip(&record.get("subject").filter(|v| v.truthy()).map(value_text).unwrap_or_default()).to_string()
        }
        _ => String::new(),
    };
    _as_your(&subject)
}

/// Python: bot/memory/again.py:1078 `_as_your(subject)`: the person's own words for a thing, said
/// back to them: "my back" is "your back".
pub fn _as_your(subject: &str) -> String {
    if pt::lower(subject).starts_with("my ") {
        return format!("your {}", pt::slice(subject, 3, None));
    }
    subject.to_string()
}

/// Python: bot/memory/again.py:1085 `replied(respond)`: a module's respond, with the earlier-turn
/// check applied once at its exit.
///
/// CONTRACT NOTE: Python's `replied` wraps a function and returns the wrapper; a `Value` can
/// carry neither. This keeps the contract shape (the value comes back unchanged) and the work is
/// `replied_reply`, which a module's respond applies to its own reply as its last step - the
/// same as the wrapper: `again::replied_reply(text, memory, |t, m| respond_inner(t, m))`.
pub fn replied(respond: &Value) -> Value {
    respond.clone()
}

/// `replied(respond)(text, memory)`: the module's reply, framed by `instead` when the chat has had
/// it; the reply as written when it is new, empty, or when the check fails.
pub fn replied_reply(text: &str, memory: &mut Memory, respond: impl FnOnce(&str, &mut Memory) -> Option<String>) -> Option<String> {
    let reply = respond(text, memory);
    let Some(said) = reply.as_ref().filter(|r| !pt::strip(r).is_empty()) else {
        return reply;
    };
    let framed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| instead(said, text, memory)));
    match framed {
        Ok(Some(f)) if !f.is_empty() => Some(f),
        _ => reply,
    }
}

/// Python: bot/memory/again.py:1122 `instead(reply, text, memory)`: the reply framed as the repeat
/// it is, or None when it is new to this conversation.
pub fn instead(reply: &str, text: &str, memory: &Memory) -> Option<String> {
    let mine = _flat(reply);
    if mine.is_empty() {
        return None;
    }
    if _carries_a_lead_in(&mine) {
        return None;
    }
    let said_back = pt::strip(reply);
    if SAID_BACK.match_(said_back).is_some() && pt::split(&mine).len() <= *SHORT {
        return None;
    }
    let mut times = _times_on_the_screen(&mine, memory);
    if times == 0 {
        times = _said_recently(&mine, memory, 3);
    }
    if times == 0 && _said_and_more(&mine, memory) {
        times = 1;
    }
    if times == 0 && _said_earlier(&mine, memory) {
        let thing = _returned_thing(memory, text);
        if thing.is_empty() {
            let asked = _the_same_question_asked_again(&mine, text, memory);
            if !asked.is_empty() {
                let mut detail = _the_subject_on_the_screen(memory);
                if detail.is_empty() {
                    detail = asked;
                }
                let frame = &RETURN[memory.turns.len() % RETURN.len()];
                return Some(fmt(frame, &[("detail", &_as_a_name(&detail)), ("reply", &_the_first_sentence(reply, memory))]));
            }
            return None;
        }
        let said = _said_earlier_text(&mine, memory);
        if !said.is_empty() && !_adds_nothing_the_answer_did_not_say(&thing, text, &said, memory) {
            return None;
        }
        let thing = _as_a_name(&thing);
        let frame = &RETURN[memory.turns.len() % RETURN.len()];
        let body = _without_the_thing_in_front(pt::strip(reply), &thing);
        return Some(fmt(frame, &[("detail", &thing), ("reply", &body)]));
    }
    if times == 0 {
        return None;
    }
    _framed_as_a_repeat(reply, &mine, memory, times, text)
}

/// Python: bot/memory/again.py:1184 `a_thing_of_the_answer(reply, text, memory)`: the reply framed
/// as the repeat it is, when the line names a thing the reply is about (a str, or None).
pub fn a_thing_of_the_answer(reply: &str, text: &str, memory: &Memory) -> Value {
    let mine = _flat(reply);
    if mine.is_empty() {
        return Value::None;
    }
    if _carries_a_lead_in(&mine) {
        return Value::None;
    }
    if !_said_earlier(&mine, memory) {
        return Value::None;
    }
    if !_names_something_the_answer_says(text, &mine) {
        return Value::None;
    }
    if crate::memory::returned::the_next_step(reply).truthy() {
        return Value::None;
    }
    Value::from(_framed_as_a_repeat(reply, &mine, memory, 1, text))
}

/// Python: bot/memory/again.py:1221 `_another_stand_down(reply, memory)`: a stand-down said a second
/// time, in the words of another one, or None.
pub fn _another_stand_down(reply: &str, memory: &Memory) -> Option<String> {
    let mine = _flat(reply);
    if mine.is_empty() {
        return None;
    }
    for line in UNKNOWN.iter() {
        if _flat(line) != mine {
            continue;
        }
        let others: Vec<&String> = UNKNOWN.iter().filter(|o| _flat(o) != mine).collect();
        if others.is_empty() {
            return None;
        }
        return Some(others[memory.turns.len() % others.len()].clone());
    }
    None
}

/// Python: bot/memory/again.py:1245 `_framed_as_a_repeat(reply, mine, memory, times, text="")`: the
/// reply under a lead-in saying it is the same answer, cut to the short of it.
pub fn _framed_as_a_repeat(reply: &str, mine: &str, memory: &Memory, times: i64, text: &str) -> Option<String> {
    if _says_it_came_back(text) {
        return None;
    }
    if pt::split(mine).len() <= *SHORT {
        let lead = py_at(&SHORT_AGAIN, times.min(SHORT_AGAIN.len() as i64) - 1);
        return Some(fmt(&lead, &[("reply", pt::strip(reply))]));
    }
    if _a_report(text) {
        return None;
    }
    if (WORDING_ASKS.is_match(text) || WORDING_SOMEONE.is_match(text) || WORDING_SAYS.is_match(text)) && !_has_a_script(reply) {
        return None;
    }
    if crate::memory::repeat::asks_for_the_move(text) {
        return None;
    }
    if _a_figure_asked_that_the_answer_does_not_hold(text, reply) {
        return None;
    }
    let underneath = the_same_thing_underneath(text, &_short(reply, Some(memory)), times, memory);
    if !underneath.is_empty() {
        return Some(underneath);
    }
    let mut short = _short(reply, Some(memory));
    if _flat(&short) != mine && _times_on_the_screen(&_flat(&short), memory) != 0 {
        short = reply.to_string();
    }
    if (_asks_for_a_detail(text) || ANOTHER_ONE.is_match(text)) && !_asks_for_a_detail(&_the_line_that_got_it(mine, memory, 3)) {
        short = reply.to_string();
    }
    if let Some(said_twice) = _another_stand_down(reply, memory) {
        return Some(said_twice);
    }
    let mut lead = py_at(&AGAIN, times.min(AGAIN.len() as i64) - 1);
    if short.contains(':') {
        lead = lead.replace(": {reply}", " \u{2014} {reply}");
    }
    Some(fmt(&lead, &[("reply", &short)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_and_phrases() {
        assert_eq!(fmt("Back to {detail}: {reply}", &[("detail", "the {reply}"), ("reply", "x")]), "Back to the {reply}: x");
        assert_eq!(_for_the_person("My head is banging, I'm tired."), "Your head is banging, You're tired.");
        assert_eq!(_as_your("my back"), "your back");
        assert_eq!(py_escape("Same answer, and it"), "Same\\ answer,\\ and\\ it");
    }

    #[test]
    fn leads_rebuilt_as_the_bundle_has_them() {
        // `PATTERNS` is `_lead(frame)` for every frame at import: the port of `_lead` builds the
        // same pattern text for each.
        for (frame, pattern) in FRAMES.iter().zip(PATTERNS.iter()) {
            assert_eq!(_lead(frame).pattern(), pattern.pattern(), "{frame}");
        }
    }
}

#[cfg(test)]
mod pybot_values {
    use super::*;

    /// Expected values from PyBot (`bot.memory.again`), called directly.
    #[test]
    fn the_readers_say_what_pybot_says() {
        let pair = |k: &str, p: &str| Some((k.to_string(), p.to_string()));
        assert_eq!(_for_the_person("My head is banging, I am tired."), "Your head is banging, You are tired.");
        assert_eq!(as_a_phrase("i feel empty"), pair("said", "Feeling empty"));
        assert_eq!(as_a_phrase("nothing interests me"), pair("clause", "Nothing interests you"));
        assert_eq!(as_a_phrase("i feel low, i feel empty"), pair("said", "Feeling empty"));
        assert_eq!(as_a_phrase("im so tired"), pair("said", "Being tired"));
        assert_eq!(unframed("Same answer, and it covers this: Do the thing.").as_deref(), Some("Do the thing."));
        assert_eq!(unframed("Back to the tap \u{2014} Turn it off.").as_deref(), Some("Turn it off."));
        assert!(_carries_a_lead_in("still: do it"));
        assert!(_carries_a_lead_in("same as before do it"));
        assert_eq!(the_name_the_answer_says("Chewing is normal while teeth change", "the chewing", ""), "the chewing");
        assert_eq!(the_name_the_answer_says("A scratching post helps", "a scratching post help", ""), "a scratching post help");
        assert_eq!(_as_a_name("back to new york"), "back to New York");
        assert_eq!(_without_the_thing_in_front("In Rome: the Colosseum with a ticket", "rome"), "The Colosseum with a ticket");
        assert!(!_a_report("he keeps coming into our bed"));
        assert!(!_a_report("my son is 6"));
        assert_eq!(_as_said("packed lunch ideas no sandwiches"), vec!["packed", "lunch", "ideas", "sandwiches"]);
        let earlier: HashSet<String> = ["heat", "ice", "better"].iter().map(|s| s.to_string()).collect();
        assert_eq!(_the_run_they_said_both_times("so is heat or ice better then", &earlier), "heat ice better");
    }
}
