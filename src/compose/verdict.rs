//! Port of `bot/compose/verdict.py`: a bare yes/no question about the thing the bot just
//! explained ("is that bad", "what should i do", "should i see someone"), answered from that
//! answer.
//!
//! Python indexes strings by code point, and several readers here do arithmetic on positions
//! (`said[at - 24:at + 24]`, `text.find(word, start)`); those positions are character indices
//! here too (`cfind`, `cslice`), and a regex match's byte offset is turned into one before it is
//! used that way.
#![allow(non_snake_case, clippy::all)]

use crate::compose::fallback_chain::{caught, data_re, data_set, data_strs, NO_MEMORY};
use crate::data::{self, Value};
use crate::py::re::Re;
use crate::py::text;
use crate::state::{Memory, Turn};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.compose.verdict.";

// ---------------------------------------------------------------------------------------------
// Strings by code point, as Python indexes them.
// ---------------------------------------------------------------------------------------------

fn clen(s: &str) -> usize {
    s.chars().count()
}

/// `s[a:b]` with character indices (clamped like Python's slice).
fn cslice(s: &str, a: usize, b: usize) -> &str {
    let n = clen(s);
    let (a, b) = (a.min(n), b.min(n));
    if a >= b {
        return "";
    }
    &s[text::byte_at(s, a)..text::byte_at(s, b)]
}

/// `s[a:]` with a character index.
fn cfrom(s: &str, a: usize) -> &str {
    &s[text::byte_at(s, a)..]
}

/// `s.find(sub, start)` as a character index.
fn cfind(s: &str, sub: &str, start: usize) -> Option<usize> {
    let n = clen(s);
    if start > n {
        return None;
    }
    let b = text::byte_at(s, start);
    s[b..].find(sub).map(|i| text::char_index(s, b + i))
}

/// `s.rfind(sub, 0, end)` as a character index, or -1.
fn crfind_before(s: &str, sub: &str, end: usize) -> isize {
    let b = text::byte_at(s, end.min(clen(s)));
    match s[..b].rfind(sub) {
        Some(i) => text::char_index(s, i) as isize,
        None => -1,
    }
}

/// A regex match's start as a character index.
fn cstart(s: &str, byte: usize) -> usize {
    text::char_index(s, byte)
}

/// `s[:1].upper() + s[1:]`.
fn upper_first(s: &str) -> String {
    text::upper_first(s)
}

/// `s[0].lower() + s[1:]`.
fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}

/// Python's `str(list_of_str)`, for the one reader that hands a list where a line is read.
fn py_list_repr(words: &[String]) -> String {
    let parts: Vec<String> = words.iter().map(|w| crate::understood::py_str_repr(w)).collect();
    format!("[{}]", parts.join(", "))
}

/// The text before the first match (`re.split(pattern, s, 1)[0]`).
fn before_first(rx: &Re, s: &str) -> String {
    match rx.search(s) {
        Some(m) => s[..m.start()].to_string(),
        None => s.to_string(),
    }
}

// ---------------------------------------------------------------------------------------------
// Data.
// ---------------------------------------------------------------------------------------------

data_re!(M; A_STEP_ASK, BARE, CONDITIONAL, CONDITIONAL_AFTER, HARD_NEWS, SYMPATHY, COMPARATIVE, URGENT_TIME,
    A_SYMPTOM_WORD, A_PERSON_THEY_KNOW, A_RELATION_TROUBLE, A_WAIT_IN_THE_STEP, _ASKS_ABOUT_A_STEP,
    _ASKS_FOR_THE_STEP, _A_FIX_WORD, NOT_A_STEP_TO_WATCH_WORDS, WHEN_ASK, WORRYING_ASK, NEEDS_A_CALL, _LIST_JOIN,
    CALLS_FOR, IS_A_NUMBER, A_PLAN, A_HOW_TO_ASK, A_DECISION_ASK, SAID_YES, METHOD_DOUBT, A_CHECK_ASK,
    A_THINKING_ABOUT, A_LENGTH_IN_THE_ANSWER, ASKS_FINE, ASKS_WRONG, MONEY_NOT_MET, NOT_MONEY_PAY,
    A_BODY_IN_THE_LINE, _A_NUMBER, A_FIGURE_ASK, A_FIGURE, JUST_AGE, ASKS_NORMAL, ASKS_OK, CARE_NOW,
    A_THING_THEY_OWN, A_BODY_WORD, WORRIED_ASK, HELPED, THROW_AWAY, NOT_TOLD_TO, ONE_OPTION, TESTING, STATED_NO,
    ENOUGH_ASK, STATED_ENOUGH, NEGATED, STEP, SPEAK_UP, EMERGENCY_CALL, SEE, HOW_TO, TIPS_AGAIN, A_HOW_ASK,
    ONE_THING, AFTER, SPLIT, THEN_OPEN, REFER, NAMES_A_CLINICIAN, DECLARATIVE_REFERRAL, FIRST_ITSELF,
    SUBJECT_OPEN, ASKS_THEM, OWN_PATTERN, _WORD, PUNT_START, PUNT_OFFER, PUNT_CHAT, URGENT_CALL,
    EMERGENCY_ORDER, EMERGENCY_CALL_OPEN, A_CHECK_ON_A_PLAN, CONSIDERING, OWN_DOING, NAMES_A_TRADE,
    A_KEEP_OR_STOP, OVER_VERB, STILL_GOING, SIGN, ASKING_HOW_TO_TELL, LATER_CONDITION, TRADE, NEGATED_BEFORE,
    A_FREQUENCY_SAID, A_FREQUENCY_ASKED, TIME_NOW, CONDITION_CLAUSE, FENCED_IN_SENTENCE, A_STATEMENT,
    TARGET_SHAPE, CONTRAST, COPULA, DETAIL_STATEMENT, CONDITIONAL_OPEN, ABOUT_THE_TALK, NEGATIVE_OPEN, HELPS,
    HEAD_REFUSES, DESCRIBES, CONTINUES, ORDER_CONTINUES, FRAME_AND, QUESTION_FRAME, OFFER_QUESTION,
    _INVITATION, THE_PLEA, AN_OFFER, _A_DASH, _AN_OPENER, REFER_VERB, RESTATED, RELATIVE, JUDGED_REFERRAL,
    STATES_A_REFERRAL, LONG_SPAN, HEDGE_SPAN, LOSS_WORDS, MOOD, ASKED_FOR_A_CALL, A_REFUSAL, OWED,
    A_DOUBT_ABOUT_THE_THING, _A_RANGE_OF_ITS_OWN, _A_REFERRAL_OUT, _A_STOP, A_WORRY_IN_THE_LINE, _AN_OFFER);
data_strs!(M; HOW_MANY_PEOPLE, DENIES, SAYS_NORMAL, REASSURE, CARE, HARD_NEWS_LINES, PERSON_ORDINARY, PERSON_CALL,
    NO_LINES, BOTH_LINES, DO_LINES, FIX_ASKED_LINES, FIX_ASKED_NOW_LINES, CONDITION_LINES, GRIEF_FLOOR_LINES,
    MET_CONDITION_LINES, WHEN_LINES, WHEN_CALL_LINES, METHOD_OK_LINES, CHECK_WORKS_LINES, CHECK_ONCE_LINES,
    CHECK_WATCH_LINES, CHECK_JOB_LINES, CHOICE_OK_LINES, NOTHING_WARNED_LINES, MONEY_TROUBLE_LINES, MET_LINES,
    FIGURE_LINES, MONEY_LINES, A_COMPARISON, AGE_LINES, OK_LINES, ORDINARY_NORMAL, CARE_NORMAL,
    CARE_NORMAL_UNKNOWABLE, CARE_NOW_NORMAL, DO_NORMAL, CONDITION_NORMAL, NOTHING_WARNED_NORMAL, YES_LINES,
    THING_ORDINARY, THING_CALL, BACKSTOP_LINES, PEOPLE_CARE, FLOOR_LINES, NEUTRAL_ASK_LINES,
    A_PLAN_DOUBT_LINES, SOON_LINES, HELPED_LINES, YES_SAID_LINES, THROW_AWAY_LINES, TEST_LINES,
    NO_STATED_LINES, ENOUGH_STATED_LINES, STOP_DOING, DANGER, AGAIN, LIST_STEP_LINES, STEP_LINES,
    STEP_DETAIL_LINES, STEP_ALREADY_FIRST_LINES, STEP_ONE_LINES, STEP_KEPT_LINES, ALSO_LINES, ONE_TURN_LINES,
    ONE_TURN_PLAIN_LINES, ALL_OF_IT_LINES, QUESTION_BACK_LINES, ASKED_FIRST_LINES, OFFER_TAKEN_LINES,
    SEE_NOW_LINES, SEE_ANYWAY_LINES, SEE_STATED_LINES, NOTHING_LINES, SEE_NONE_LINES, A_CHECK_ON_A_PLAN_LINES,
    GO_FROM, TRIM, TRIM_INSTRUMENT, ADVICE_OK_LINES, DIRECTED_LINES, OWED_LINES, HANDED_BACK_LINES,
    ASKED_THEM_LINES);
data_set!(M; NOT_A_STEP_TO_WATCH, _NOT_A_SIGN_WORD, _A_BODY_WORD, MONEY_NOUN_WORDS, MONEY_WORD, BODY_WORD,
    SOMEONE_ELSE, DOING_WITH_NO_OBJECT, KNOWING_DOING, WHEN, HANGING, IMPERATIVES, READABLE, UNREADABLE,
    COMPOSED, _AFTER_A_NOUN, ECHO_FILLER, RECAP_FILLER, THING_WORDS, CONDITION_VERBS, CONDITION_HANGING,
    LOOK_AND_SEE, NAMES_NOTHING, DOING, FENCE_FILLER, _STEP_OPENERS, FRONT_LEAD, FRONT_THIS, FRONT_WHEN);

fn int_(name: &str) -> usize {
    data::get(&format!("{M}{name}")).as_int().unwrap() as usize
}
static EXPLANATION_WORDS: LazyLock<usize> = LazyLock::new(|| int_("EXPLANATION_WORDS"));
static ECHO_WORDS: LazyLock<usize> = LazyLock::new(|| int_("ECHO_WORDS"));
static RUN: LazyLock<usize> = LazyLock::new(|| int_("RUN"));
static STEP_WORDS: LazyLock<usize> = LazyLock::new(|| int_("STEP_WORDS"));
static LONG_STEP_WORDS: LazyLock<usize> = LazyLock::new(|| int_("LONG_STEP_WORDS"));
static STEP_LONG: LazyLock<usize> = LazyLock::new(|| int_("STEP_LONG"));
static STATED_LONGEST: LazyLock<usize> = LazyLock::new(|| int_("STATED_LONGEST"));
static HAND_OVER_WORDS: LazyLock<usize> = LazyLock::new(|| int_("HAND_OVER_WORDS"));
static FRONT_SUBJECT_WORDS: LazyLock<usize> = LazyLock::new(|| int_("FRONT_SUBJECT_WORDS"));

/// `RULED_BY_THE_COMPOSER`: the pools whose lines rule on a doubt.
static RULED_BY_THE_COMPOSER: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
    data::get(&format!("{M}RULED_BY_THE_COMPOSER")).items().iter().map(|pool| pool.strs()).collect()
});
/// `FEELING_FAMILY`: a feeling word -> its family.
static FEELING_FAMILY: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| data::str_map(&format!("{M}FEELING_FAMILY")));
/// `FEELING_DOUBT_LINES`: family -> its pool.
static FEELING_DOUBT_LINES: LazyLock<HashMap<String, Vec<String>>> =
    LazyLock::new(|| data::get(&format!("{M}FEELING_DOUBT_LINES")).str_list_map());
/// `THE_PLAN_CAN_HURT_LINES`: family -> its pool.
static THE_PLAN_CAN_HURT_LINES: LazyLock<HashMap<String, Vec<String>>> =
    LazyLock::new(|| data::get(&format!("{M}THE_PLAN_CAN_HURT_LINES")).str_list_map());
/// `CAN_HURT`: (family, the act, the guard that makes it dangerous or None), in order.
static CAN_HURT: LazyLock<Vec<(String, &'static Re, Option<&'static Re>)>> = LazyLock::new(|| {
    data::get(&format!("{M}CAN_HURT"))
        .items()
        .iter()
        .map(|row| {
            let row = row.items();
            (row[0].str().to_string(), row[1].as_re().unwrap(), row[2].as_re())
        })
        .collect()
});
static RED_FLAG: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.RED_FLAG"));
static A_RULING_ALREADY: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.safetyask.A_RULING_ALREADY"));
static A_SAFETY_ALREADY_RULED: LazyLock<Vec<String>> =
    LazyLock::new(|| data::strs("bot.compose.safetyask.A_SAFETY_ALREADY_RULED"));
static COMFORT_SAID: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.compose.fallback.COMFORT_SAID"));
static FEELING_SAID: LazyLock<Vec<String>> = LazyLock::new(|| {
    data::try_get("bot.modules.chat.FEELING_SAID").map(|v| v.strs()).unwrap_or_default()
});

// Patterns written inline in the Python functions.
fn rx(p: &str) -> Re {
    Re::new(p, 0)
}
fn rx_i(p: &str) -> Re {
    Re::new(p, crate::py::re::I)
}
static SENTENCES: LazyLock<Re> = LazyLock::new(|| rx(r"(?<=[.!?]) "));
static SENTENCES_WS: LazyLock<Re> = LazyLock::new(|| rx(r"(?<=[.!?])\s+"));
static SENTENCES_SPACES: LazyLock<Re> = LazyLock::new(|| rx(r"(?<=[.!?]) +"));
static CLAUSE_BREAKS: LazyLock<Re> = LazyLock::new(|| rx(r"[:;.!?]|,?\s+and\s+"));
static HEAD_BREAK: LazyLock<Re> = LazyLock::new(|| rx(r"[:;.!?]"));
static A_PAREN: LazyLock<Re> = LazyLock::new(|| rx(r"\([^)]*\)"));
static LETTERS: LazyLock<Re> = LazyLock::new(|| rx(r"[a-z']+"));
static LETTERS_4: LazyLock<Re> = LazyLock::new(|| rx(r"[a-z']{4,}"));
static WS: LazyLock<Re> = LazyLock::new(|| rx(r"\s+"));
static BODY_LETTERS: LazyLock<Re> = LazyLock::new(|| rx(r"[a-z'-]+"));
static MONEY_LETTERS: LazyLock<Re> = LazyLock::new(|| rx("[A-Za-z\u{00a3}$\u{20ac}']+"));
static EXPLANATION_HEAD: LazyLock<Re> = LazyLock::new(|| rx(":|,|;| - | \u{2014} "));
static A_GAP: LazyLock<Re> = LazyLock::new(|| rx(r"\{[A-Za-z]+\}"));
static WORDS_09: LazyLock<Re> = LazyLock::new(|| rx(r"[a-z0-9']+"));
static I_OPEN: LazyLock<Re> = LazyLock::new(|| rx_i("(?:i\\s|ive\\s|i've\\s|i\u{2019}ve\\s)"));
static RELATIVE_MARKER: LazyLock<Re> = LazyLock::new(|| rx_i(r"\b(?:that|which|who)\s+"));
static A_FENCE_WORD: LazyLock<Re> = LazyLock::new(|| rx_i(r"\b(?:if|unless|once|when) \w"));
static A_SENTENCE_END: LazyLock<Re> = LazyLock::new(|| rx(r"[.!?](?:\s|$)"));
static AGO: LazyLock<Re> = LazyLock::new(|| rx_i(r"\b(?:days?|weeks?)\s+ago\b"));
static STEP_PIECES: LazyLock<Re> = LazyLock::new(|| rx("[.!?;:]\\s+| \u{2014} | \u{2013} | - "));
static HOW_MUCH: LazyLock<Re> = LazyLock::new(|| rx(r"\bhow (?:much|many)\b|\bamount\b|\benough\b"));
static AND_OR_THEN: LazyLock<Re> = LazyLock::new(|| rx(" and | then "));
static IF_OPENS: LazyLock<Re> = LazyLock::new(|| rx_i(r"(?:so )?if "));
static AFTER_IF_OPENS: LazyLock<Re> = LazyLock::new(|| {
    rx(r"(?:^|, |; | and | then | so )(?:and )?(?:get|see|ring|call|take|go|book|treat|watch|keep|check|apply|cover|use|rest|ice|put|give|try|test|monitor|stop|unplug|switch|leave|turn|have|ask|phone|contact|speak|mention)\b")
});
static AFTER_IF_INSIDE: LazyLock<Re> = LazyLock::new(|| {
    rx(r"(?:^|, | and | then )(?:and )?(?:get|see|ring|call|take|go|book|treat|watch|keep|check|apply|cover|use|rest|ice|put|give|try|test|monitor|have|ask|phone|contact|speak|mention)\b")
});
static A_JUDGEMENT_LIST: LazyLock<Re> =
    LazyLock::new(|| rx(r"(?i)\b(?:anything|any [a-z-]+)\b(.*?)\b(?:needs?|should be|is worth|wants)\b"));
static HANGING_JOIN: LazyLock<Re> = LazyLock::new(|| rx(r"(?:^|[ ,]+)(?:and|or|but|then|so|also|plus)$"));
static OR_AND_IF: LazyLock<Re> = LazyLock::new(|| rx(r"\b(or|and) if\b"));
static AN_ASIDE: LazyLock<Re> = LazyLock::new(|| rx(r"\s+-\s+|\s+--\s+"));
static IF_UNLESS: LazyLock<Re> = LazyLock::new(|| rx(r"\b(?:if|unless|when|once)\b"));
static CLAUSE_SPLIT: LazyLock<Re> = LazyLock::new(|| rx(",|;| and | then | so | but "));
static CLAUSE_SPLIT_COLON: LazyLock<Re> = LazyLock::new(|| rx(",|;| and | then | so | but |:"));
static A_LINK_OR_ASIDE: LazyLock<Re> = LazyLock::new(|| rx(r"\([^)]*\)|\S*\.(?:com|co\.uk|org|net|gov)\S*"));
static A_RANGE_AND: LazyLock<Re> = LazyLock::new(|| rx(r"(?<=\d) and (?=[\d.])"));
static THEN_CUT: LazyLock<Re> = LazyLock::new(|| rx(r",? then "));
static ACTION_SPLIT: LazyLock<Re> =
    LazyLock::new(|| rx(",| and | then |\\. |; |: | - | \u{2014} | \u{2013} "));
static LIST_ITEMS: LazyLock<Re> = LazyLock::new(|| rx(",| and "));
static ENGLISH_WORDS: LazyLock<Re> = LazyLock::new(|| rx("[A-Za-z'\u{2019}-]+"));
static LOWER_WORDS: LazyLock<Re> = LazyLock::new(|| rx("[a-z'\u{2019}-]+"));
static A_WORD_OF_LETTERS: LazyLock<Re> = LazyLock::new(|| rx(r"[A-Za-z']+"));
static A_BREAK: LazyLock<Re> = LazyLock::new(|| rx("[,;:]|\\s[-\u{2013}\u{2014}]\\s"));
static FENCE_IN_FRONT: LazyLock<Re> = LazyLock::new(|| rx_i(r"^(?:but )?(?:if|once|when|whenever) [^,]{0,70}, "));
static A_STOP_IN_FRONT: LazyLock<Re> = LazyLock::new(|| rx_i(r"^(?:stop|don't|dont|do not) [^,]{0,40}?,? (?:and|but) "));

/// `pick(options, memory, seed_text)` from `bot/compose/__init__.py`.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, memory, seed_text)
}

/// `str(x or "")` for an optional turn field.
fn module_of(turn: &Turn) -> &str {
    turn.module.as_deref().unwrap_or("")
}
fn in_set(set: &HashSet<String>, module: &Option<String>) -> bool {
    module.as_ref().is_some_and(|m| set.contains(m))
}

// ---------------------------------------------------------------------------------------------
// A step asked about, a call and its signs.
// ---------------------------------------------------------------------------------------------

/// Python: bot/compose/verdict.py:412 `_a_job_somewhere(reply)`
pub fn _a_job_somewhere(reply: &str) -> bool {
    let flat = _flat(reply);
    if flat.is_empty() {
        return false;
    }
    for clause in CLAUSE_BREAKS.split(&flat) {
        let words = _WORD.findall_str(text::strip_chars(&clause, " ,-"));
        if words.is_empty() || !IMPERATIVES.contains(&words[0]) {
            continue;
        }
        if words.len() > 1 && (words[1] == "me" || words[1] == "us") {
            continue;
        }
        if words.len() > 1 && _AFTER_A_NOUN.contains(&words[1]) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/compose/verdict.py:438 `the_fix_asked(clean, memory)`
pub fn the_fix_asked(clean: &str, memory: &Memory) -> String {
    let flat = _flat(clean);
    if flat.is_empty() {
        return String::new();
    }
    if text::split(&flat).len() > 10 {
        return String::new();
    }
    if _ASKS_ABOUT_A_STEP.search(&flat).is_none() || _A_FIX_WORD.search(&flat).is_none() {
        return String::new();
    }
    if _ASKS_FOR_THE_STEP.match_(&flat).is_some() {
        return String::new();
    }
    let last = _last_reply(memory, false, false);
    if last.is_empty() || !(opens_on_a_job(&last) || _a_job_somewhere(&last)) {
        return String::new();
    }
    let domain = memory
        .get("last_answer")
        .filter(|v| v.truthy())
        .and_then(|v| v.get("domain"))
        .filter(|v| v.truthy())
        .map(|v| crate::compose::fallback_chain::py_str(Some(v)))
        .unwrap_or_default();
    if NOT_A_STEP_TO_WATCH.contains(&text::lower(&domain)) {
        return String::new();
    }
    if NOT_A_STEP_TO_WATCH_WORDS.search(&last).is_some() {
        return String::new();
    }
    if A_WAIT_IN_THE_STEP.search(&last).is_some() {
        return _say(&FIX_ASKED_LINES, memory, &flat);
    }
    _say(&FIX_ASKED_NOW_LINES, memory, &flat)
}

/// Python: bot/compose/verdict.py:565 `_the_list_before_the_call(said)`
pub fn _the_list_before_the_call(said: &str) -> String {
    let mut best = String::new();
    for m in NEEDS_A_CALL.finditer(said) {
        let part = _LIST_JOIN.sub("", text::strip_chars(m.name_str("list"), " ,;:-"));
        let part = text::strip_chars(&part, " ,;:-").to_string();
        let words = text::split(&part).len();
        if (3..=24).contains(&words) && clen(&part) > clen(&best) {
            best = part;
        }
    }
    if !best.is_empty() && ["A", "An", "The"].contains(&text::split_ref(&best)[0]) {
        best = lower_first(&best);
    }
    best
}

/// Python: bot/compose/verdict.py:602 `_the_call_and_its_reason(said)`
pub fn _the_call_and_its_reason(said: &str) -> String {
    let mut found: Vec<(usize, String, String)> = Vec::new();
    for m in CALLS_FOR.finditer(said) {
        let sign = _no_hanging_end(text::strip_chars(m.name_str("list"), " ,;:-"));
        let n = text::split(&sign).len();
        if !(2..=20).contains(&n) {
            continue;
        }
        found.push((m.start(), sign, _the_person_for(m.name("who"), m.name("quick"))));
    }
    for m in IS_A_NUMBER.finditer(said) {
        let sign = _no_hanging_end(text::strip_chars(m.name_str("when"), " ,;:-"));
        let n = text::split(&sign).len();
        if !(2..=20).contains(&n) {
            continue;
        }
        found.push((m.start(), sign, _the_person_for(m.name("who"), m.name("quick"))));
    }
    found.sort();
    let mut said_signs: Vec<String> = Vec::new();
    for (_, sign, who) in found {
        if said_signs.iter().any(|other| other.contains(sign.as_str())) {
            continue;
        }
        said_signs.retain(|s| !sign.contains(s.as_str()));
        said_signs.push(if who.is_empty() { sign } else { format!("{sign} ({who})") });
        if said_signs.len() == 2 {
            break;
        }
    }
    if said_signs.is_empty() {
        return String::new();
    }
    let clause = said_signs.join("; ");
    if clause.chars().next().is_some_and(|c| c.is_uppercase()) { lower_first(&clause) } else { clause }
}

/// Python: bot/compose/verdict.py:651 `_the_sign_words(call)`
pub fn _the_sign_words(call: &str) -> HashSet<String> {
    let signs = A_PAREN.sub(" ", call);
    LETTERS
        .findall_str(&text::lower(&signs))
        .into_iter()
        .filter(|w| clen(w) >= 3 && !_NOT_A_SIGN_WORD.contains(w))
        .collect()
}

/// Python: bot/compose/verdict.py:658 `_they_named_the_sign_themselves(call, memory)`
pub fn _they_named_the_sign_themselves(call: &str, memory: &Memory) -> bool {
    let words = _the_sign_words(call);
    if words.is_empty() {
        return false;
    }
    let said = _flat(&_the_line_behind(memory));
    LETTERS.findall_str(&said).iter().any(|w| words.contains(w))
}

/// Python: bot/compose/verdict.py:667 `_the_person_for(who, quick)`
pub fn _the_person_for(who: Option<&str>, quick: Option<&str>) -> String {
    let mut trade = text::lower(text::strip(who.unwrap_or("")));
    if trade.is_empty() {
        return String::new();
    }
    trade = match trade.as_str() {
        "gp" => "GP".to_string(),
        "a&e" => "A&E".to_string(),
        "nhs" => "NHS".to_string(),
        _ => trade,
    };
    let first_digit = trade.chars().next().is_some_and(|c| text::is_digit(&c.to_string()));
    let piece = if first_digit || trade == "A&E" || trade == "NHS" {
        trade.clone()
    } else if ["GP", "doctor", "practice nurse", "nurse", "surgery", "vet", "dentist", "optician", "midwife", "physio", "physiotherapist"]
        .contains(&trade.as_str())
    {
        format!("your {trade}")
    } else {
        format!("a {trade}")
    };
    let hurry = text::strip_chars(&WS.sub(" ", quick.unwrap_or("")), " ,").to_string();
    if hurry.is_empty() { piece } else { text::strip(&format!("{piece} {hurry}")).to_string() }
}

/// Python: bot/compose/verdict.py:868 `_about_a_body(*texts)`
pub fn _about_a_body(texts: &[&str]) -> bool {
    texts.iter().any(|t| BODY_LETTERS.findall_str(&text::lower(t)).iter().any(|w| _A_BODY_WORD.contains(w)))
}

/// Python: bot/compose/verdict.py:974 `_a_doubt_about_money(memory)`: the line, or "".
pub fn _a_doubt_about_money(memory: &Memory) -> String {
    let said = _flat(&_last_person_line(memory));
    if said.is_empty() || text::split(&said).len() > 16 {
        return String::new();
    }
    if !_words(&said).iter().any(|w| MONEY_NOUN_WORDS.contains(w)) {
        return String::new();
    }
    if MONEY_NOT_MET.search(&said).is_none() {
        return String::new();
    }
    if A_BODY_IN_THE_LINE.search(&said).is_some() {
        return String::new();
    }
    if NOT_MONEY_PAY.search(&said).is_some() {
        return String::new();
    }
    said
}

/// Python: bot/compose/verdict.py:1072 `_is_a_comparison(said)`
pub fn _is_a_comparison(said: &str) -> bool {
    if A_FIGURE.search(said).is_some() {
        return true;
    }
    _first(said, &A_COMPARISON).is_some()
}

/// Python: bot/compose/verdict.py:1096 `_a_figure_owed(line)`
pub fn _a_figure_owed(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    let words: Vec<String> = MONEY_LETTERS.findall_str(line).iter().map(|w| text::lower(&w.replace('\'', ""))).collect();
    if words.iter().any(|w| BODY_WORD.contains(w)) {
        return false;
    }
    if words.iter().any(|w| MONEY_WORD.contains(w)) {
        return true;
    }
    A_FIGURE.search(line).is_some()
}

/// Python: bot/compose/verdict.py:1895 `_is_an_explanation(sentence)`
pub fn _is_an_explanation(sentence: &str) -> bool {
    let t = text::strip(sentence);
    if SUBJECT_OPEN.match_(t).is_none() {
        return false;
    }
    let head = before_first(&EXPLANATION_HEAD, t);
    text::split(&head).len() > *EXPLANATION_WORDS
}

/// Python 3.12's `re.escape`: only the characters it lists get a backslash.
fn py_escape(part: &str) -> String {
    let mut out = String::with_capacity(part.len() * 2);
    for c in part.chars() {
        if "()[]{}?*+-|^$\\.&~# \t\n\r\u{b}\u{c}".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Python: bot/compose/verdict.py:2047 `_own_pattern(line)`
pub fn _own_pattern(line: &str) -> String {
    A_GAP.split(line).iter().map(|part| py_escape(part)).collect::<Vec<_>>().join(".+?")
}

/// `_ACK_PATTERN`, built the first time it is needed from the acknowledgement frames.
static _ACK_PATTERN: LazyLock<Re> = LazyLock::new(|| {
    let mut frames = Vec::new();
    for name in ["FRAMES", "HABIT_TAIL", "GOING_TAIL", "EXPLAIN_TAIL"] {
        frames.extend(data::strs(&format!("bot.memory.ack.{name}")));
    }
    let joined: Vec<String> = frames.iter().map(|line| _own_pattern(line)).collect();
    Re::new(&joined.join("|"), crate::py::re::I)
});

/// Python: bot/compose/verdict.py:2075 `takes_a_fact_back(said)`
pub fn takes_a_fact_back(said: &str) -> bool {
    let t = text::squash(said);
    if t.is_empty() || _ACK_PATTERN.match_(&t).is_none() {
        return false;
    }
    let words = LETTERS.findall_str(&text::lower(&t));
    words.len() <= *ECHO_WORDS && !words.iter().any(|w| IMPERATIVES.contains(w))
}

/// Python: bot/compose/verdict.py:2097 `_say(pool, memory, flat, **gaps)` with no gaps.
pub fn _say(pool: &[String], memory: &Memory, flat: &str) -> String {
    _say_with(pool, memory, flat, &[])
}

/// `_say(pool, memory, flat, name=value, ...)`: the gaps filled in before the pick.
pub fn _say_with(pool: &[String], memory: &Memory, flat: &str, gaps: &[(&str, &str)]) -> String {
    let mut rendered: Vec<String> = pool.to_vec();
    for (name, value) in gaps {
        let upper_name = upper_first(name);
        for (template, gap) in [(name.to_string(), format!("{{{name}}}")), (upper_name.clone(), format!("{{{upper_name}}}"))] {
            let _ = template;
            let filled = if gap == format!("{{{name}}}") { value.to_string() } else { upper_first(value) };
            rendered = rendered.iter().map(|line| line.replace(&gap, &filled)).collect();
        }
    }
    pick(&rendered, memory, flat)
}

/// Python: bot/compose/verdict.py:2114 `_a_report_line(said)`
pub fn _a_report_line(said: &str) -> bool {
    // Only the import is under Python's `try`; the reading itself raises through.
    crate::compose::fallback::is_a_report_reply(said)
}

/// Python: bot/compose/verdict.py:2129 `_flat(text)`
pub fn _flat(text_: &str) -> String {
    text::squash(&text::lower(text_).replace('\u{2019}', "'"))
}

/// Python: bot/compose/verdict.py:2165 `rules_on_the_doubt(said)`
pub fn rules_on_the_doubt(said: &str) -> bool {
    let said = text::strip(said);
    if said.is_empty() {
        return false;
    }
    let head = text::lower(&text::slice(said, 0, Some(40)));
    RULED_BY_THE_COMPOSER.iter().any(|pool| pool.iter().any(|line| head == text::lower(&text::slice(line, 0, Some(40)))))
}

/// Python: bot/compose/verdict.py:2192 `opens_on_a_job(reply)`
pub fn opens_on_a_job(reply: &str) -> bool {
    let flat = _flat(reply);
    if flat.is_empty() {
        return false;
    }
    let head = before_first(&HEAD_BREAK, &flat);
    let words = _WORD.findall_str(text::strip_chars(&head, " ,-"));
    if words.is_empty() || !IMPERATIVES.contains(&words[0]) {
        return false;
    }
    if words.len() > 1 && (words[1] == "me" || words[1] == "us") {
        return false;
    }
    words.len() == 1 || !_AFTER_A_NOUN.contains(&words[1])
}

/// Python: bot/compose/verdict.py:2220 `carries_the_answer(reply, memory)`
pub fn carries_the_answer(reply: &str, memory: &Memory) -> bool {
    if _kept_step(reply) {
        return true;
    }
    let said = _WORD.findall_str(&_flat(&_last_reply(memory, false, false)));
    let words = _WORD.findall_str(&_flat(reply));
    let run = *RUN;
    if said.len() < run || words.len() < run {
        return false;
    }
    let runs: HashSet<String> = (0..said.len() - run + 1).map(|i| said[i..i + run].join(" ")).collect();
    (0..words.len() - run + 1).any(|i| runs.contains(&words[i..i + run].join(" ")))
}

/// Python: bot/compose/verdict.py:2261 `_a_punt(last)`
pub fn _a_punt(last: &Turn) -> bool {
    let module = module_of(last);
    if !["greeting", "fallback", "chat"].contains(&module) {
        return false;
    }
    let said = _flat(&last.bot);
    if said.is_empty() || PUNT_START.match_(&said).is_none() {
        return false;
    }
    if module == "chat" && PUNT_CHAT.search(&said).is_none() {
        return false;
    }
    if PUNT_OFFER.search(&said).is_some() {
        return true;
    }
    if said.contains('?') {
        return false;
    }
    text::split(&said).len() <= 8
}

/// Python: bot/compose/verdict.py:2283 `_skip_the_derived(turns, memory)`
pub fn _skip_the_derived<'a>(turns: &'a [Turn], memory: &Memory) -> &'a [Turn] {
    if turns.len() <= 1 {
        return turns;
    }
    if memory.get("frequency_reply").is_some_and(|v| v.truthy()) || crate::memory::healing::derived(memory) {
        return &turns[..turns.len() - 1];
    }
    turns
}

/// Python: bot/compose/verdict.py:2298 `answers_the_ask(clean, memory)`
pub fn answers_the_ask(clean: &str, memory: &Memory) -> bool {
    if _last_module(memory, true) != "answers" {
        return false;
    }
    reply_for(clean, memory).is_some()
}

/// Python: bot/compose/verdict.py:2316 `_the_news_just_before(memory)`
pub fn _the_news_just_before(memory: &Memory) -> bool {
    let said = memory.turns.last().map(|t| text::squash(&text::lower(&t.user))).unwrap_or_default();
    !said.is_empty() && crate::compose::badnews::reads_as_bad_news(&said)
}

/// Python: bot/compose/verdict.py:2337 `_last_reply(memory, past_the_nod=False,
/// past_the_verdict=False)`
pub fn _last_reply(memory: &Memory, past_the_nod: bool, past_the_verdict: bool) -> String {
    let mut punt_line = String::new();
    if memory.turns.is_empty() {
        return String::new();
    }
    let turns = _skip_the_derived(&memory.turns, memory);
    for index in (0..turns.len()).rev() {
        let last = &turns[index];
        let before = if index > 0 { Some(&turns[index - 1]) } else { None };
        let module = &last.module;
        let said = last.bot.as_str();
        if in_set(&UNREADABLE, module) {
            return punt_line;
        }
        if module.as_deref() == Some("chat") && OWN_PATTERN.search(&_flat(said)).is_some() {
            continue;
        }
        if _a_report_line(said) {
            continue;
        }
        if in_set(&COMPOSED, module) && crate::compose::why::is_a_reason_line(said) {
            continue;
        }
        if in_set(&READABLE, module) && takes_a_fact_back(said) {
            continue;
        }
        if A_STEP_ASK.match_(&_flat(&last.user)).is_some() && module.as_deref() != Some("answers") {
            continue;
        }
        if _a_punt(last) {
            if !past_the_nod || !punt_line.is_empty() {
                return if !past_the_nod { said.to_string() } else { punt_line };
            }
            punt_line = said.to_string();
            continue;
        }
        if past_the_verdict && _only_a_verdict(&last.user) {
            continue;
        }
        if _is_an_echo(last) {
            continue;
        }
        if let Some(before) = before {
            if _is_a_recap(last, before) {
                continue;
            }
        }
        if in_set(&READABLE, module) {
            if !punt_line.is_empty() && module.as_deref() != Some("answers") {
                return punt_line;
            }
            return said.to_string();
        }
        return punt_line;
    }
    punt_line
}

/// Python: bot/compose/verdict.py:2448 `_words(text)`
pub fn _words(text_: &str) -> Vec<String> {
    WORDS_09.findall_str(&text::lower(text_)).into_iter().map(|w| w.replace('\'', "")).collect()
}

/// Python: bot/compose/verdict.py:2456 `_is_an_echo(turn)`
pub fn _is_an_echo(turn: &Turn) -> bool {
    let said = _words(&turn.bot);
    let theirs: HashSet<String> = _words(&turn.user).into_iter().collect();
    if said.is_empty() || theirs.is_empty() || said.len() > 12 {
        return false;
    }
    said.iter().all(|w| ECHO_FILLER.contains(w) || theirs.contains(w))
}

/// Python: bot/compose/verdict.py:2479 `_is_a_recap(turn, before)`
pub fn _is_a_recap(turn: &Turn, before: &Turn) -> bool {
    let said = _words(&turn.bot);
    let had: HashSet<String> = _words(&before.bot).into_iter().collect();
    if said.is_empty() || had.is_empty() || said.len() >= had.len() {
        return false;
    }
    if before.module.as_deref() != Some("answers") {
        return false;
    }
    said.iter().all(|w| had.contains(w) || RECAP_FILLER.contains(w) || ECHO_FILLER.contains(w))
}

/// Python: bot/compose/verdict.py:2495 `_last_line_that_named_a_call(memory)`
pub fn _last_line_that_named_a_call(memory: &Memory) -> String {
    let Some(last) = memory.turns.last() else {
        return String::new();
    };
    if last.module.as_deref() == Some("answers") {
        return String::new();
    }
    let said = _flat(&last.bot);
    if said.is_empty() || (_first(&said, &CARE).is_none() && REFER.search(&said).is_none()) {
        return String::new();
    }
    said
}

/// Python: bot/compose/verdict.py:2528 `_a_call_for_now(said)`
pub fn _a_call_for_now(said: &str) -> bool {
    if _care_for_now(said) {
        return true;
    }
    for sentence in SENTENCES.split(said) {
        let Some(found) = REFER.search(&sentence) else {
            let Some(trade) = TRADE.search(&sentence) else { continue };
            if LATER_CONDITION.search(&sentence[..trade.start()]).is_some() {
                continue;
            }
            if TIME_NOW.search(&sentence).is_some() {
                return true;
            }
            continue;
        };
        if LATER_CONDITION.search(&sentence[found.start()..]).is_none() {
            return true;
        }
        if TIME_NOW.search(&sentence[found.start()..]).is_some() {
            return true;
        }
    }
    false
}

/// Python: bot/compose/verdict.py:2582 `_the_call_sentence(said)`
pub fn _the_call_sentence(said: &str) -> String {
    for sentence in SENTENCES.split(said) {
        if EMERGENCY_ORDER.match_(&sentence).is_some() {
            return sentence;
        }
        if let Some(opening) = EMERGENCY_CALL_OPEN.match_(&sentence) {
            if LATER_CONDITION.search(&sentence[opening.end()..]).is_none() {
                return sentence;
            }
        }
        if TIME_NOW.search(&sentence).is_none() {
            continue;
        }
        if let Some(found) = REFER.search(&sentence) {
            let tail = &sentence[found.start()..];
            if LATER_CONDITION.search(tail).is_none() || TIME_NOW.search(tail).is_some() {
                return sentence;
            }
            continue;
        }
        let Some(trade) = TRADE.search(&sentence) else { continue };
        if LATER_CONDITION.search(&sentence[..trade.start()]).is_some() {
            continue;
        }
        return sentence;
    }
    String::new()
}

/// Python: bot/compose/verdict.py:2624 `_last_person_line(memory)`
pub fn _last_person_line(memory: &Memory) -> String {
    for last in memory.turns.iter().rev() {
        let said = _flat(&last.user);
        if !said.is_empty() {
            return said;
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:2639 `_the_worst_thing_they_said(memory)`: the line, or "".
pub fn _the_worst_thing_they_said(memory: &Memory) -> String {
    for last in memory.turns.iter().rev() {
        let said = _flat(&last.user);
        if !said.is_empty() {
            return if HARD_NEWS.search(&said).is_some() { said } else { String::new() };
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:2658 `_as_written(line)`: (spelled right, said out).
pub fn _as_written(line: &str) -> (String, String) {
    let t = _flat(line);
    if t.is_empty() {
        return (String::new(), String::new());
    }
    match caught(|| crate::understanding::normalize::normalise(&t)) {
        Some(read) => (_flat(&read.plain), _flat(&read.norm_text)),
        None => (String::new(), String::new()),
    }
}

/// Python: bot/compose/verdict.py:2679 `_a_thing_they_own(memory)`
pub fn _a_thing_they_own(memory: &Memory) -> String {
    let line = _the_line_behind(memory);
    if line.is_empty() {
        return String::new();
    }
    let low = _flat(&line);
    if A_BODY_WORD.search(&low).is_some() {
        return String::new();
    }
    A_THING_THEY_OWN.search(&low).map(|m| text::lower(m.as_str())).unwrap_or_default()
}

/// Python: bot/compose/verdict.py:2696 `_a_person_they_know(memory, said)`
pub fn _a_person_they_know(memory: &Memory, _said: &str) -> String {
    let line = _the_line_behind(memory);
    if line.is_empty() {
        return String::new();
    }
    let low = _flat(&line);
    if A_SYMPTOM_WORD.search(&low).is_some() {
        return String::new();
    }
    if A_PERSON_THEY_KNOW.search(&low).is_none() || A_RELATION_TROUBLE.search(&low).is_none() {
        return String::new();
    }
    "person".into()
}

/// Python: bot/compose/verdict.py:2715 `_the_line_behind(memory)`
pub fn _the_line_behind(memory: &Memory) -> String {
    if memory.turns.is_empty() {
        return String::new();
    }
    let turns = _skip_the_derived(&memory.turns, memory);
    for index in (0..turns.len()).rev() {
        let last = &turns[index];
        let module = &last.module;
        if in_set(&UNREADABLE, module) {
            return String::new();
        }
        if module.as_deref() == Some("chat") && OWN_PATTERN.search(&_flat(&last.bot)).is_some() {
            continue;
        }
        if _is_an_echo(last) {
            continue;
        }
        if A_STEP_ASK.match_(&_flat(&last.user)).is_some() && module.as_deref() != Some("answers") {
            continue;
        }
        if index > 0 && _is_a_recap(last, &turns[index - 1]) {
            continue;
        }
        if in_set(&READABLE, module) {
            return _flat(&last.user);
        }
        return String::new();
    }
    String::new()
}

/// Python: bot/compose/verdict.py:2756 `_the_plan_they_brought(memory, back=2)`
pub fn _the_plan_they_brought(memory: &Memory, back: usize) -> String {
    let mut seen = 0;
    for turn in memory.turns.iter().rev() {
        let said = _flat(&turn.user);
        if said.is_empty() {
            continue;
        }
        seen += 1;
        if seen > back {
            break;
        }
        if A_PLAN.search(&said).is_some() || CONSIDERING.search(&said).is_some() {
            return said;
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:2822 `_considering(memory)`
pub fn _considering(memory: &Memory) -> bool {
    for turn in memory.turns.iter().rev() {
        let said = _flat(&turn.user);
        if !said.is_empty() {
            return CONSIDERING.search(&said).is_some();
        }
    }
    false
}

/// Python: bot/compose/verdict.py:2836 `_the_last_line_was_mine(memory)`
pub fn _the_last_line_was_mine(memory: &Memory) -> bool {
    for last in memory.turns.iter().rev() {
        let module = module_of(last);
        if module.is_empty() {
            continue;
        }
        return module == "chat" || COMPOSED.contains(module);
    }
    false
}

/// Python: bot/compose/verdict.py:2858 `_last_module(memory, past_the_nod=False)`
pub fn _last_module(memory: &Memory, past_the_nod: bool) -> String {
    let mut punt_module = String::new();
    if memory.turns.is_empty() {
        return String::new();
    }
    let turns = _skip_the_derived(&memory.turns, memory);
    for index in (0..turns.len()).rev() {
        let last = &turns[index];
        let module = &last.module;
        let name = module_of(last).to_string();
        if in_set(&UNREADABLE, module) {
            return punt_module;
        }
        if module.as_deref() == Some("chat") && OWN_PATTERN.search(&_flat(&last.bot)).is_some() {
            continue;
        }
        if _a_report_line(&last.bot) {
            continue;
        }
        if in_set(&COMPOSED, module) && crate::compose::why::is_a_reason_line(&_flat(&last.bot)) {
            continue;
        }
        if in_set(&READABLE, module) && takes_a_fact_back(&last.bot) {
            continue;
        }
        if A_STEP_ASK.match_(&_flat(&last.user)).is_some() && module.as_deref() != Some("answers") {
            continue;
        }
        if _a_punt(last) {
            if !past_the_nod || !punt_module.is_empty() {
                return if !past_the_nod { name } else { punt_module };
            }
            punt_module = name;
            continue;
        }
        if _is_an_echo(last) {
            continue;
        }
        if index > 0 && _is_a_recap(last, &turns[index - 1]) {
            continue;
        }
        if in_set(&READABLE, module) {
            if !punt_module.is_empty() && module.as_deref() != Some("answers") {
                return punt_module;
            }
            return name;
        }
        return punt_module;
    }
    punt_module
}

// ---------------------------------------------------------------------------------------------
// Plans, and what the person brought.
// ---------------------------------------------------------------------------------------------

/// Python: bot/compose/verdict.py:3021 `_the_plan_can_hurt(said)`: the family, or None.
pub fn _the_plan_can_hurt(said: &str) -> Option<String> {
    if said.is_empty() || NAMES_A_TRADE.search(said).is_some() {
        return None;
    }
    for (family, act, guard) in CAN_HURT.iter() {
        if act.search(said).is_none() {
            continue;
        }
        if let Some(guard) = guard {
            if guard.search(said).is_none() {
                continue;
            }
        }
        return Some(family.clone());
    }
    None
}

/// Python: bot/compose/verdict.py:3040 `_the_answer_already_warned(said)`
pub fn _the_answer_already_warned(said: &str) -> String {
    if said.is_empty() || NAMES_A_CLINICIAN.search(said).is_none() {
        return String::new();
    }
    for sentence in SENTENCES_WS.split(said) {
        let sentence = text::strip(&sentence);
        let n = text::split(sentence).len();
        if !(6..=40).contains(&n) {
            continue;
        }
        if NAMES_A_CLINICIAN.search(sentence).is_none() {
            continue;
        }
        if A_KEEP_OR_STOP.search(sentence).is_none() {
            continue;
        }
        return format!("{}.", text::rstrip_chars(sentence, " ."));
    }
    String::new()
}

/// Python: bot/compose/verdict.py:3061 `_a_plan_doubt(memory)`
pub fn _a_plan_doubt(memory: &Memory) -> Option<String> {
    let mut said = _flat(&_the_line_behind(memory));
    if said.is_empty() || A_PLAN.search(&said).is_none() {
        said = _the_plan_they_brought(memory, 2);
    }
    if said.is_empty() {
        return None;
    }
    Some(pick(&_round_robin(&A_PLAN_DOUBT_LINES, &said), memory, &said))
}

/// The person's lines from the last back, skipping the echoes, as the readers below walk them.
fn the_line_past_the_echoes(memory: &Memory) -> Option<String> {
    for last in memory.turns.iter().rev() {
        if _is_an_echo(last) {
            continue;
        }
        return Some(_flat(&last.user));
    }
    None
}

/// Python: bot/compose/verdict.py:3082 `_about_the_person(memory)`
pub fn _about_the_person(memory: &Memory) -> bool {
    let Some(said) = the_line_past_the_echoes(memory) else { return false };
    if said.is_empty() || text::split(&said).len() < 3 {
        return false;
    }
    I_OPEN.match_(&said).is_some()
}

/// Python: bot/compose/verdict.py:3105 `_own_doing(memory)`
pub fn _own_doing(memory: &Memory) -> bool {
    let Some(said) = the_line_past_the_echoes(memory) else { return false };
    !said.is_empty() && text::split(&said).len() >= 3 && OWN_DOING.match_(&said).is_some()
}

/// Python: bot/compose/verdict.py:3120 `_a_plan_or_a_choice(memory)`
pub fn _a_plan_or_a_choice(memory: &Memory) -> bool {
    let said = _flat(&_the_line_behind(memory));
    if said.is_empty() || !(A_PLAN.search(&said).is_some() || A_DECISION_ASK.search(&said).is_some()) {
        return false;
    }
    let words = _words(&said);
    !words.iter().any(|w| BODY_WORD.contains(w)) && !words.iter().any(|w| SOMEONE_ELSE.contains(w))
}

/// Python: bot/compose/verdict.py:3137 `_good_news(memory)`
pub fn _good_news(memory: &Memory) -> bool {
    let Some(said) = the_line_past_the_echoes(memory) else { return false };
    !said.is_empty() && crate::compose::goodnews::reads_as_good_news(&said)
}

/// Python: bot/compose/verdict.py:3174 `_an_event_that_is_over(memory)`
pub fn _an_event_that_is_over(memory: &Memory) -> bool {
    let Some(said) = the_line_past_the_echoes(memory) else { return false };
    if said.is_empty() || STILL_GOING.search(&said).is_some() {
        return false;
    }
    OVER_VERB.search(&said).is_some()
}

/// Python: bot/compose/verdict.py:3197 `_nothing_to_be_long_about(memory)`
pub fn _nothing_to_be_long_about(memory: &Memory) -> bool {
    _good_news(memory) || _an_event_that_is_over(memory)
}

/// Python: bot/compose/verdict.py:3207 `_first(text, words)`: where the first of these words sits
/// in the lowered text (a character index), or None.
pub fn _first(text_: &str, words: &[String]) -> Option<usize> {
    let t = text::lower(text_);
    let mut best: Option<usize> = None;
    for word in words {
        if let Some(at) = cfind(&t, word, 0) {
            if best.map_or(true, |b| at < b) {
                best = Some(at);
            }
        }
    }
    best
}

/// Python: bot/compose/verdict.py:3222 `_a_throw_away(said)`: (the action, written as a test).
pub fn _a_throw_away(said: &str) -> (Option<String>, bool) {
    for m in THROW_AWAY.finditer(said) {
        let before = &said[..m.start()];
        if NOT_TOLD_TO.search(before).is_some() || ONE_OPTION.search(before).is_some() {
            continue;
        }
        let sentence = before.split('.').last().unwrap_or("");
        let action = upper_first(m.as_str());
        if TESTING.search(sentence).is_some() || CONDITIONAL.search(sentence.split(',').last().unwrap_or("")).is_some() {
            return (Some(action), true);
        }
        return (Some(action), false);
    }
    (None, false)
}

/// Python: bot/compose/verdict.py:3258 `_the_person_line(memory, said)`
pub fn _the_person_line(memory: &Memory, said: &str) -> String {
    let said = text::strip(said);
    for turn in memory.turns.iter().rev() {
        let line = text::strip(&turn.user);
        if !line.is_empty() && line != said {
            return line.to_string();
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:3268 `_said_no(said)`
pub fn _said_no(said: &str) -> Option<String> {
    for sentence in SENTENCES_WS.split(said) {
        if STATED_NO.search(&sentence).is_some() {
            let s = text::strip(text::strip_chars(text::strip(&sentence), "\"'\u{201c}\u{201d}"));
            if !s.is_empty() {
                return Some(format!("{}.", text::rstrip_chars(s, ".")));
            }
        }
    }
    None
}

/// Python: bot/compose/verdict.py:3286 `_said_enough(said)`
pub fn _said_enough(said: &str) -> Option<String> {
    for sentence in SENTENCES_WS.split(said) {
        if STATED_ENOUGH.search(&sentence).is_none() {
            continue;
        }
        let mut s = text::rstrip_chars(text::strip(text::strip_chars(text::strip(&sentence), "\"'\u{201c}\u{201d}")), ".").to_string();
        if clen(&s) > 90 && s.contains(',') {
            let head = text::strip(s.split(',').next().unwrap_or("")).to_string();
            if clen(&head) > 20 && STATED_ENOUGH.search(&head).is_some() {
                s = head;
            }
        }
        if !s.is_empty() {
            return Some(format!("{s}."));
        }
    }
    None
}

/// Python: bot/compose/verdict.py:3307 `_the_person_gave_the_sign(memory, said)`
pub fn _the_person_gave_the_sign(memory: &Memory, said: &str) -> bool {
    let line = _the_person_line(memory, said);
    if line.is_empty() || ASKING_HOW_TO_TELL.search(&line).is_some() {
        return false;
    }
    SIGN.search(&line).is_some()
}

/// Python: bot/compose/verdict.py:3327 `_care_for_now(said)`
pub fn _care_for_now(said: &str) -> bool {
    for sentence in SENTENCES.split(said) {
        if _first(&sentence, &CARE).is_none() {
            continue;
        }
        let low = text::lower(&sentence);
        let cue: isize = CARE
            .iter()
            .filter(|word| low.contains(word.as_str()))
            .map(|word| cfind(&low, word, 0).map(|i| i as isize).unwrap_or(-1))
            .min()
            .unwrap_or(-1);
        if cue >= 0 {
            let cue = cue as usize;
            if LATER_CONDITION.search(cfrom(&sentence, cue)).is_none()
                && LATER_CONDITION.search(cslice(&sentence, 0, cue)).is_none()
            {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/verdict.py:3345 `_sentence_at(text, at)`
pub fn _sentence_at(text_: &str, at: usize) -> String {
    let mut begin = 0usize;
    for found in SENTENCES.finditer(text_) {
        let start = cstart(text_, found.start());
        if start >= at {
            return cslice(text_, begin, start).to_string();
        }
        begin = cstart(text_, found.end());
    }
    cfrom(text_, begin).to_string()
}

/// Python: bot/compose/verdict.py:3355 `_behind_a_description(said, at)`
pub fn _behind_a_description(said: &str, at: usize) -> bool {
    let mut sentence = _sentence_at(said, at);
    let n = clen(said) as isize;
    if clen(&sentence) as isize <= n - at as isize - 1 && !text::ends_with_any(&sentence, &[".", "!", "?"]) {
        sentence.push('.');
    }
    let described = _a_described_condition(&sentence);
    if described.is_empty() {
        return false;
    }
    let mut marker: Option<(usize, usize)> = None;
    for found in RELATIVE_MARKER.finditer(&sentence) {
        marker = Some((found.start(), found.end()));
    }
    let Some((mstart, _)) = marker else { return false };
    if text::strip(&sentence[..mstart]).is_empty() {
        return false;
    }
    let cue: isize = if (at as isize) < n {
        let piece = text::lower(cslice(said, at, at + 12));
        match cfind(&text::lower(&sentence), &piece, 0) {
            Some(i) => i as isize,
            None => -1,
        }
    } else {
        -1
    };
    cue > cstart(&sentence, mstart) as isize
}

/// Python: bot/compose/verdict.py:3376 `_not_fenced(said, start, end=None)` (character indices).
pub fn _not_fenced(said: &str, start: usize, end: Option<usize>) -> bool {
    let before = cslice(said, start.saturating_sub(90), start);
    if CONDITIONAL.search(before.split('.').last().unwrap_or("")).is_some() {
        return false;
    }
    if let Some(end) = end {
        let after_all = cslice(said, end, end + 90);
        let after = after_all.split('.').next().unwrap_or("");
        if let Some(fence) = CONDITIONAL_AFTER.search(after) {
            if URGENT_TIME.search(cslice(said, start, end + 40)).is_none() {
                let rider = &after[..fence.start()];
                if COMPARATIVE.search(rider).is_none() {
                    return false;
                }
            }
        }
    }
    true
}

/// Python: bot/compose/verdict.py:3408 `_said_without_a_not(text, at, length)`
pub fn _said_without_a_not(text_: &str, at: usize, length: usize) -> bool {
    if cslice(text_, at.saturating_sub(2), at) == "un" {
        return false;
    }
    let before = if at > 0 { text_.chars().nth(at - 1) } else { Some(' ') };
    let after = text_.chars().nth(at + length);
    if before.is_some_and(|c| c.is_alphanumeric()) || after.is_some_and(|c| c.is_alphanumeric()) {
        return false;
    }
    NEGATED_BEFORE.search(cslice(text_, at.saturating_sub(26), at)).is_none()
}

/// Python: bot/compose/verdict.py:3419 `_first_positive(text, words)`
pub fn _first_positive(text_: &str, words: &[String]) -> Option<usize> {
    let t = text::lower(text_);
    let mut best: Option<usize> = None;
    for word in words {
        let mut start = 0;
        while let Some(at) = cfind(&t, word, start) {
            if _said_without_a_not(&t, at, clen(word)) {
                if best.map_or(true, |b| at < b) {
                    best = Some(at);
                }
                break;
            }
            start = at + 1;
        }
    }
    best
}

/// Python: bot/compose/verdict.py:3443 `_unconditional_care(said)`
pub fn _unconditional_care(said: &str) -> Option<usize> {
    let said = text::lower(said);
    let mut best: Option<usize> = None;
    for word in CARE.iter() {
        let mut start = cfind(&said, word, 0);
        while let Some(at) = start {
            if _not_fenced(&said, at, Some(at + clen(word))) && !_behind_a_description(&said, at) {
                if best.map_or(true, |b| at < b) {
                    best = Some(at);
                }
                break;
            }
            start = cfind(&said, word, at + 1);
        }
    }
    for m in REFER.finditer(&said) {
        let (s, e) = (cstart(&said, m.start()), cstart(&said, m.end()));
        if !_not_fenced(&said, s, Some(e)) {
            continue;
        }
        if best.map_or(true, |b| s < b) {
            best = Some(s);
        }
        break;
    }
    best
}

/// `{word.rstrip("s") for word in re.findall(r"[a-z']{4,}", line)}`
fn long_words_unplural(line: &str) -> HashSet<String> {
    LETTERS_4.findall_str(line).iter().map(|w| text::rstrip_chars(w, "s").to_string()).collect()
}

/// Python: bot/compose/verdict.py:3474 `_the_fence_is_met(fence, said, memory)`
pub fn _the_fence_is_met(fence: &str, said: &str, memory: &Memory) -> bool {
    let mut line = text::lower(&_the_line_it_answered(said, memory));
    if line.is_empty() {
        line = memory.turns.last().map(|t| text::lower(&t.user)).unwrap_or_default();
    }
    if line.is_empty() {
        return false;
    }
    let said_words = long_words_unplural(&line);
    LETTERS_4.findall_str(&text::lower(fence)).iter().any(|w| said_words.contains(text::rstrip_chars(w, "s")))
}

/// Python: bot/compose/verdict.py:3506 `_a_frequency_already_said(fence, said, memory)`
pub fn _a_frequency_already_said(fence: &str, said: &str, memory: &Memory) -> bool {
    if A_FREQUENCY_ASKED.search(fence).is_none() {
        return false;
    }
    let answered = _the_line_it_answered(said, memory);
    let subject = long_words_unplural(&answered);
    for turn in &memory.turns {
        let line = text::lower(&turn.user);
        if A_FREQUENCY_SAID.search(&line).is_none() {
            continue;
        }
        if subject.is_empty() {
            return true;
        }
        if long_words_unplural(&line).iter().any(|w| subject.contains(w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/verdict.py:3538 `_condition_after_care(said, care_at, memory=None)`
pub fn _condition_after_care(said: &str, care_at: Option<usize>, memory: Option<&Memory>) -> String {
    let Some(care_at) = care_at else { return String::new() };
    let sentence = SENTENCES.split(cfrom(said, care_at)).into_iter().next().unwrap_or_default();
    let Some(fence) = A_FENCE_WORD.search(&sentence) else {
        if !_the_call_sentence(said).is_empty() {
            return String::new();
        }
        if let Some(memory) = memory {
            if _past_the_timescale(said, memory) {
                return String::new();
            }
        }
        let later = _when(said);
        // `_a_frequency_already_said(later, said, memory)` and `_the_fence_is_met(...)` both read
        // a memory that is not a dict as "no": the None memory says no to both.
        if !later.is_empty() && memory.is_some_and(|m| _a_frequency_already_said(&later, said, m)) {
            return String::new();
        }
        if !later.is_empty() && memory.is_some_and(|m| _the_fence_is_met(&later, said, m)) {
            return String::new();
        }
        return if later.is_empty() { String::new() } else { _cut_condition(&later, 22) };
    };
    if TIME_NOW.search(&sentence).is_some() {
        return String::new();
    }
    if COMPARATIVE.search(&sentence[..fence.start()]).is_some() {
        return String::new();
    }
    _cut_condition(&_when(said), 22)
}

/// Python: bot/compose/verdict.py:3615 `_cut_condition(when, most=22)`
pub fn _cut_condition(when: &str, most: usize) -> String {
    let when = text::strip_chars(when, " ,.");
    if text::split(when).len() <= most {
        return when.to_string();
    }
    let first = CONDITION_CLAUSE.split(when).into_iter().next().unwrap_or_default();
    text::strip_chars(&first, " ,.").to_string()
}

/// Python: bot/compose/verdict.py:3622 `_first_sentence_end(said)` (a character index).
pub fn _first_sentence_end(said: &str) -> usize {
    match A_SENTENCE_END.search(said) {
        Some(m) => cstart(said, m.end()),
        None => clen(said),
    }
}

/// Python: bot/compose/verdict.py:3628 `_told_to_act(said, at)`
pub fn _told_to_act(said: &str, at: usize) -> bool {
    // `DECLARATIVE_REFERRAL.match(said, at)`: a match that starts at `at`.
    let b = text::byte_at(said, at);
    let here = DECLARATIVE_REFERRAL.search_at(said, b).filter(|m| m.start() == b);
    if here.is_none() {
        return true;
    }
    at <= _first_sentence_end(said)
}

/// Python: bot/compose/verdict.py:3645 `_referral(said)`, as the contract has it: the character
/// index as an int, or None.
pub fn _referral(said: &str) -> Value {
    match _referral_at(said) {
        Some(at) => Value::Int(at as i64),
        None => Value::None,
    }
}

/// `_referral(said)`: where the answer's own referral sits (a character index), or None.
pub fn _referral_at(said: &str) -> Option<usize> {
    let Some(care) = _unconditional_care(said) else {
        if let Some(stated) = DECLARATIVE_REFERRAL.search(said) {
            let (s, e) = (cstart(said, stated.start()), cstart(said, stated.end()));
            if _not_fenced(said, s, Some(e)) {
                return Some(s);
            }
        }
        return None;
    };
    if care <= _first_sentence_end(said) {
        return Some(care);
    }
    if _told_to_act(said, care) { Some(care) } else { None }
}

/// Python: bot/compose/verdict.py:3674 `_a_plain_go(at, said)`
pub fn _a_plain_go(at: usize, said: &str) -> bool {
    let said = text::lower(said);
    let window = cslice(&said, at.saturating_sub(24), at + 24);
    GO_FROM.iter().any(|word| window.contains(word.as_str()))
}

/// Python: bot/compose/verdict.py:3688 `_opening(said)`
pub fn _opening(said: &str) -> String {
    let parts = SENTENCES.split(said);
    parts[..parts.len().min(2)].join(" ")
}

/// Python: bot/compose/verdict.py:3723 `_the_thing(line)`
pub fn _the_thing(line: &str) -> HashSet<String> {
    LETTERS_4.findall_str(&_flat(line)).into_iter().filter(|w| !THING_WORDS.contains(w)).collect()
}

/// Python: bot/compose/verdict.py:3728 `_referral_about_the_thing(said, line)`
pub fn _referral_about_the_thing(said: &str, line: &str) -> bool {
    let thing = _the_thing(line);
    if thing.is_empty() {
        return false;
    }
    for sentence in SENTENCES.split(said) {
        if NAMES_A_CLINICIAN.search(&sentence).is_none() {
            continue;
        }
        if FENCED_IN_SENTENCE.search(&sentence).is_some() {
            continue;
        }
        if LETTERS_4.findall_str(&text::lower(&sentence)).iter().any(|w| thing.contains(w)) {
            return true;
        }
    }
    false
}

fn content_set(t: &str) -> HashSet<String> {
    crate::memory::topics::content(t).into_iter().collect()
}

/// Python: bot/compose/verdict.py:3743 `_echoes_the_person(step, memory)`
pub fn _echoes_the_person(step: &str, memory: &Memory) -> bool {
    if memory.turns.is_empty() {
        return false;
    }
    let mut said = "";
    for turn in memory.turns.iter().rev() {
        if !turn.user.is_empty() {
            said = &turn.user;
            break;
        }
    }
    if said.is_empty() {
        return false;
    }
    let want = content_set(step);
    let have = content_set(said);
    if want.is_empty() || want.len() > 3 {
        return false;
    }
    want.iter().all(|w| have.contains(w))
}

/// `_own_pattern(line)` for each kept-step line, compiled once.
static STEP_KEPT_PATTERNS: LazyLock<Vec<Re>> =
    LazyLock::new(|| STEP_KEPT_LINES.iter().map(|line| Re::new(&_own_pattern(line), crate::py::re::I)).collect());

/// Python: bot/compose/verdict.py:3768 `_kept_step(reply)`
pub fn _kept_step(reply: &str) -> bool {
    let plain = text::strip(&_flat(reply)).to_string();
    STEP_KEPT_PATTERNS.iter().any(|p| p.fullmatch(&plain).is_some())
}

/// Python: bot/compose/verdict.py:3775 `_only_a_verdict(line)`
pub fn _only_a_verdict(line: &str) -> bool {
    let t = text::strip(&_flat(line)).to_string();
    if t.is_empty() {
        return false;
    }
    caught(|| crate::compose::fallback::reads_as_a_verdict(&t, &NO_MEMORY)).unwrap_or(false)
}

/// Python: bot/compose/verdict.py:3790 `_the_line_just_given(memory)`
pub fn _the_line_just_given(memory: &Memory) -> String {
    memory.turns.iter().rev().find(|t| !t.bot.is_empty()).map(|t| t.bot.clone()).unwrap_or_default()
}

/// Python: bot/compose/verdict.py:3801 `_the_answer_before_that(memory)`
pub fn _the_answer_before_that(memory: &Memory) -> String {
    memory.turns.iter().rev().filter(|t| !t.bot.is_empty()).nth(1).map(|t| t.bot.clone()).unwrap_or_default()
}

/// Python: bot/compose/verdict.py:3819 `_word_runs(text, size=5)`
pub fn _word_runs(text_: &str, size: usize) -> HashSet<Vec<String>> {
    let words = text::split(&_flat(text_));
    let n = (words.len() + 1).saturating_sub(size);
    (0..n).map(|at| words[at..(at + size).min(words.len())].to_vec()).collect()
}

/// Python: bot/compose/verdict.py:3824 `_says_it_again(line, said)`
pub fn _says_it_again(line: &str, said: &str) -> bool {
    let theirs = _word_runs(said, 5);
    _word_runs(line, 5).iter().any(|run| theirs.contains(run))
}

/// Python: bot/compose/verdict.py:3843 `_already_asked_for_a_step(memory)`
pub fn _already_asked_for_a_step(memory: &Memory) -> bool {
    for turn in memory.turns.iter().rev() {
        if !turn.user.is_empty() {
            return STEP.match_(&_flat(&turn.user)).is_some();
        }
    }
    false
}

/// Python: bot/compose/verdict.py:3854 `_the_referral_sentence(said)`, as the contract has it.
pub fn _the_referral_sentence(said: &str) -> Value {
    Value::from(the_referral_sentence(said))
}

/// `_the_referral_sentence(said)`: the one sentence of the answer that holds its judgement.
pub fn the_referral_sentence(said: &str) -> String {
    let at = _first(&text::lower(said), &CARE).unwrap_or(0);
    let start = crfind_before(said, ". ", at).max(crfind_before(said, "? ", at)).max(crfind_before(said, "! ", at));
    let rest = text::strip(cfrom(said, (start + 1) as usize)).to_string();
    SENTENCES.split(&rest).into_iter().next().unwrap_or_default()
}

/// Python: bot/compose/verdict.py:3864 `_the_line_is_the_threshold(stated, line)`
pub fn _the_line_is_the_threshold(stated: &str, line: &str) -> bool {
    let want = content_set(stated);
    let have = content_set(line);
    want.intersection(&have).count() >= 2
}

/// Python: bot/compose/verdict.py:3878 `_most_of_the_line(step, heard)`
pub fn _most_of_the_line(step: &str, heard: &str) -> bool {
    let want = content_set(step);
    let have = content_set(heard);
    if want.is_empty() || have.is_empty() {
        return false;
    }
    want.intersection(&have).count() as f64 / have.len() as f64 >= 0.8
}

// ---------------------------------------------------------------------------------------------
// Steps: reading the answer's own instructions out of it.
// ---------------------------------------------------------------------------------------------

/// Python: bot/compose/verdict.py:3893 `_step(said)`, as the contract has it: the step, or None.
pub fn _step(said: &str) -> Value {
    match the_step(said) {
        Some(s) => Value::from(s),
        None => Value::None,
    }
}

/// `_step(said)`: the first thing the answer told them to do, in its own words.
pub fn the_step(said: &str) -> Option<String> {
    the_steps(said)
        .into_iter()
        .filter(|one| NEGATIVE_OPEN.match_(one).is_none())
        .find(|one| A_STATEMENT.match_(one).is_none())
}

/// Python: bot/compose/verdict.py:3927 `_listed(said)`
pub fn _listed(said: &str) -> Option<String> {
    for sentence in SENTENCES.split(said) {
        if CONDITIONAL_OPEN.match_(text::strip(&sentence)).is_some() {
            continue;
        }
        let (head, colon, tail) = text::partition(&sentence, ": ");
        if colon.is_empty() || tail.matches(',').count() < 2 {
            continue;
        }
        if HEAD_REFUSES.search(head).is_some() || HELPS.search(head).is_none() {
            continue;
        }
        for item in LIST_ITEMS.split(tail) {
            let item = text::squash(text::strip_chars(&item, " .:;,-"));
            let n = text::split(&item).len();
            if !(3..=*STEP_WORDS).contains(&n) {
                continue;
            }
            if NEGATIVE_OPEN.match_(&item).is_some() || CONDITIONAL_OPEN.match_(&item).is_some() {
                continue;
            }
            if DESCRIBES.search(&item).is_some() {
                continue;
            }
            return Some(item);
        }
    }
    None
}

/// Python: bot/compose/verdict.py:4022 `_cut_on(head, joins)`
pub fn _cut_on(head: &str, joins: &[String]) -> String {
    let mut best = String::new();
    for join in joins {
        let mut at = cfind(head, join, 0);
        while let Some(a) = at {
            if a == 0 {
                break;
            }
            let cut = text::strip_chars(cslice(head, 0, a), " ,;:-").to_string();
            let cut_words = text::split(&cut);
            if (4..=*STEP_WORDS).contains(&cut_words.len())
                && !HANGING.contains(&text::lower(cut_words.last().unwrap()))
                && clen(&cut) > clen(&best)
            {
                best = cut;
            }
            at = cfind(head, join, a + 1);
        }
    }
    best
}

/// Python: bot/compose/verdict.py:4062 `_shorten(head, words)`
pub fn _shorten(head: &str, words: &[String]) -> String {
    if words.len() <= *STEP_WORDS {
        return head.to_string();
    }
    let best = _cut_on(head, &TRIM);
    if !best.is_empty() {
        return best;
    }
    if words.len() <= *LONG_STEP_WORDS && !HANGING.contains(&text::lower(words.last().unwrap())) {
        return head.to_string();
    }
    _cut_on(head, &TRIM_INSTRUMENT)
}

/// Python: bot/compose/verdict.py:4098 `_cut_at_a_join(part)`
pub fn _cut_at_a_join(part: &str) -> String {
    let mut keep_until = 0usize;
    for frame in FRAME_AND.finditer(part) {
        if frame.end() > keep_until {
            keep_until = frame.end();
        }
    }
    for at in AND_OR_THEN.finditer(part) {
        if at.start() < keep_until {
            continue;
        }
        return part[..at.start()].to_string();
    }
    part.to_string()
}

/// Python: bot/compose/verdict.py:4111 `_first_clause(part, and_break=True)`
pub fn _first_clause(part: &str, and_break: bool) -> String {
    for (at, c) in part.char_indices() {
        if c != ',' {
            continue;
        }
        let rest = &part[at + 1..];
        if !and_break && ORDER_CONTINUES.match_(rest).is_some() {
            continue;
        }
        if CONTINUES.match_(rest).is_none() {
            return part[..at].to_string();
        }
    }
    if !and_break {
        return part.to_string();
    }
    _cut_at_a_join(part)
}

/// Python: bot/compose/verdict.py:4172 `_an_offer_is_waiting(said)`
pub fn _an_offer_is_waiting(said: &str) -> bool {
    let t = text::strip(said);
    if !t.ends_with('?') {
        return false;
    }
    let last = SENTENCES.split(t).pop().unwrap_or_default();
    OFFER_QUESTION.search(&last).is_some()
}

/// Python: bot/compose/verdict.py:4204 `_after_a_comfort_line(memory)`
pub fn _after_a_comfort_line(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said = text::lower(&last.bot);
    if said.is_empty() {
        return false;
    }
    COMFORT_SAID.iter().chain(FEELING_SAID.iter()).any(|line| !line.is_empty() && said.contains(&text::lower(line)))
}

/// Python: bot/compose/verdict.py:4239 `_the_question_asked(said)`
pub fn _the_question_asked(said: &str) -> String {
    let t = text::strip(said);
    if !t.ends_with('?') {
        return String::new();
    }
    let last = text::strip(&SENTENCES.split(t).pop().unwrap_or_default()).to_string();
    if !_is_a_question(&last) {
        return String::new();
    }
    let mut asked = text::strip_chars(&last[..last.len() - 1], " ,;:-").to_string();
    if let Some(split) = _A_DASH.match_(&asked) {
        let mut front = text::strip_chars(split.str(1), " ,;:-").to_string();
        let rest = text::strip_chars(split.str(2), " ,;:-").to_string();
        if _AN_OPENER.match_(&front).is_some() && _is_a_question(&format!("{rest}?")) {
            front = rest;
        }
        asked = front;
    }
    if _INVITATION.match_(&asked).is_some() {
        return String::new();
    }
    if asked.is_empty() || text::split(&asked).len() > 14 {
        return String::new();
    }
    let mut words = text::split(&asked);
    if !["I", "I'm", "I've", "I'd", "I'll"].contains(&words[0].as_str()) {
        words[0] = lower_first(&words[0]);
    }
    words.join(" ")
}

/// `text.rsplit(", ", 1)[-1]`
fn after_the_last_comma(t: &str) -> &str {
    match t.rfind(", ") {
        Some(i) => &t[i + 2..],
        None => t,
    }
}

/// Python: bot/compose/verdict.py:4273 `_ends_on_a_question(said)`
pub fn _ends_on_a_question(said: &str) -> bool {
    let t = text::strip(said);
    if !t.ends_with('?') {
        return false;
    }
    let last = SENTENCES.split(t).pop().unwrap_or_default();
    QUESTION_FRAME.match_(&last).is_some() || QUESTION_FRAME.match_(after_the_last_comma(&last)).is_some()
}

/// Python: bot/compose/verdict.py:4282 `_is_a_question(sentence)`
pub fn _is_a_question(sentence: &str) -> bool {
    let t = text::strip(sentence);
    if !t.ends_with('?') {
        return false;
    }
    if QUESTION_FRAME.match_(t).is_some() {
        return true;
    }
    QUESTION_FRAME.match_(after_the_last_comma(t)).is_some()
}

/// Python: bot/compose/verdict.py:4292 `_steps(said)`, as the contract has it: a list.
pub fn _steps(said: &str) -> Value {
    Value::from(the_steps(said))
}

/// `_steps(said)`: every step the answer gave, in the answer's own words and order.
pub fn the_steps(said: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut parts: Vec<(String, bool)> = Vec::new();
    for (at_sentence, sentence) in SENTENCES.split(said).into_iter().enumerate() {
        let stripped = text::strip(&sentence);
        if CONDITIONAL_OPEN.match_(stripped).is_some()
            || RED_FLAG.match_(stripped).is_some()
            || _is_a_question(&sentence)
            || _is_an_explanation(&sentence)
        {
            continue;
        }
        let mut sentence = sentence.clone();
        if at_sentence == 0 {
            // `opened is not sentence`: a new string only when the strip or the sub changed it.
            let stripped = text::strip(&sentence).to_string();
            let changed = stripped != sentence || THEN_OPEN.search(&stripped).is_some();
            if changed {
                let opened = THEN_OPEN.sub("", &stripped);
                sentence = upper_first(&opened);
                if sentence.is_empty() {
                    continue;
                }
            }
        }
        let raw = SPLIT.split(&sentence);
        let mut kept: Vec<(String, String)> = Vec::new();
        for (at, chunk) in raw.iter().enumerate() {
            if at % 2 == 1 || text::strip(chunk).is_empty() {
                continue;
            }
            let sep = if kept.is_empty() { String::new() } else { raw[at - 1].clone() };
            kept.push((text::strip(chunk).to_string(), sep));
        }
        if kept.len() > 1
            && CONTRAST.search(&kept[0].0).is_some()
            && IMPERATIVES.contains(&_doing_word(&text::split(&kept[1].0)))
        {
            kept.remove(0);
        }
        if kept.len() > 1 && !IMPERATIVES.contains(&_doing_word(&text::split(&kept[0].0))) {
            kept.remove(0);
        }
        if kept.len() > 1 && kept[1..].iter().all(|(_, sep)| sep == ": ") {
            parts.push((kept.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>().join(": "), true));
        }
        parts.extend(kept.into_iter().map(|(t, _)| (t, false)));
    }
    for (part, spread) in parts {
        if RED_FLAG.match_(&part).is_some() {
            continue;
        }
        let part = A_LINK_OR_ASIDE.sub(" ", &part);
        let part = text::squash(text::strip_chars(&part, " .:;,-"));
        if part.is_empty() {
            continue;
        }
        let words = text::split(&part);
        if !IMPERATIVES.contains(&_doing_word(&words)) || COPULA.match_(&part).is_some() {
            continue;
        }
        if TARGET_SHAPE.match_(&part).is_some() {
            continue;
        }
        if ABOUT_THE_TALK.search(&part).is_some() {
            continue;
        }
        let protected = A_RANGE_AND.sub(" \u{0} ", &part);
        let head: String;
        if spread {
            let (opening, _, detail) = text::partition(&protected, ": ");
            let opening = _first_clause(opening, false).replace('\u{0}', "and");
            let opening = text::strip_chars(&opening, " .:;,-").to_string();
            let mut detail = text::strip_chars(&detail.replace('\u{0}', "and"), " .:;,-").to_string();
            if DETAIL_STATEMENT.search(&detail).is_some() {
                detail = String::new();
            } else {
                detail = text::strip_chars(&_first_clause(&detail, true), " .:;,-").to_string();
            }
            let mut h = if detail.is_empty() { opening.clone() } else { format!("{opening}: {detail}") };
            if text::split(&h).len() > *STEP_WORDS && detail.contains(',') {
                detail = text::strip_chars(detail.split(',').next().unwrap_or(""), " .:;,-").to_string();
                h = if detail.is_empty() { opening.clone() } else { format!("{opening}: {detail}") };
            }
            if text::split(&h).len() > *STEP_WORDS {
                let short = _shorten(&opening, &text::split(&opening));
                h = if short.is_empty() { opening.clone() } else { short };
            }
            head = h;
        } else {
            let clause = text::strip_chars(&_first_clause(&protected, true).replace('\u{0}', "and"), " .:;,-").to_string();
            let clause_words = text::split(&clause);
            let mut h = if clause_words.len() >= 3 { _shorten(&clause, &clause_words) } else { String::new() };
            if h.is_empty() {
                let trimmed = THEN_CUT.split(&part).into_iter().next().unwrap_or_default();
                let trimmed_words = text::split(&trimmed);
                h = if trimmed_words.len() >= 2 { _shorten(&trimmed, &trimmed_words) } else { String::new() };
            }
            if h.is_empty() {
                h = _shorten(&part, &words);
            }
            head = h;
        }
        if head.is_empty() {
            return found;
        }
        let words = text::split(&head);
        if words.is_empty() || HANGING.contains(words.last().unwrap()) || found.contains(&head) {
            continue;
        }
        if !IMPERATIVES.contains(&_doing_word(&words)) {
            continue;
        }
        found.push(head);
    }
    found
}

/// Python: bot/compose/verdict.py:4425 `_doing_word(words)`
pub fn _doing_word(words: &[String]) -> String {
    if let Some(first) = words.first() {
        if ["a", "an", "the"].contains(&text::lower(text::strip_chars(first, "*_`(\"'[]")).as_str()) {
            return String::new();
        }
    }
    for word in words.iter().take(5) {
        let core = text::lower(text::strip_chars(word, ",;:-\u{2014}\u{2013}"));
        if WHEN.contains(&core) {
            continue;
        }
        return core;
    }
    String::new()
}

/// Python: bot/compose/verdict.py:4440 `_action_count(said)`
pub fn _action_count(said: &str) -> usize {
    let said: Vec<String> = SENTENCES.split(said).into_iter().filter(|s| !_is_a_question(s)).collect();
    let said = said.join(" ");
    let mut total = 0;
    for part in ACTION_SPLIT.split(&said) {
        let words = text::split(text::strip_chars(&part, " .:;,-()"));
        if !words.is_empty() && IMPERATIVES.contains(&_doing_word(&words)) {
            total += 1;
        }
    }
    total
}

/// Python: bot/compose/verdict.py:4456 `_no_hanging_end(when)`
pub fn _no_hanging_end(when: &str) -> String {
    let mut t = text::strip_chars(when, " ,;:").to_string();
    loop {
        let shorter = text::strip_chars(&HANGING_JOIN.sub("", &t), " ,;:").to_string();
        if shorter == t || shorter.is_empty() {
            break;
        }
        t = shorter;
    }
    t
}

/// Python: bot/compose/verdict.py:4505 `_described_clause(sentence)`: (from, to, its words), the
/// positions as character indices into the sentence.
pub fn _described_clause(sentence: &str) -> Option<(usize, usize, String)> {
    let referral = REFER.search(sentence)?;
    let head = &sentence[..referral.start()];
    let mut marker: Option<(usize, usize)> = None;
    for found in RELATIVE.finditer(head) {
        marker = Some((found.start(), found.end()));
    }
    let (mstart, mend) = marker?;
    if text::strip(&head[..mstart]).is_empty() {
        return None;
    }
    let clause = &head[mend..];
    let mut cut: Option<usize> = None;
    for found in REFER_VERB.finditer(clause) {
        let rest = text::strip_chars(&clause[found.end()..], " ,");
        if text::split(rest).len() <= 4 && !rest.contains(',') {
            cut = Some(found.start());
        }
    }
    let cut = cut?;
    let words = text::split(text::strip_chars(&clause[..cut], " ,;:-"));
    if !(2..=26).contains(&words.len()) {
        return None;
    }
    if CONDITION_HANGING.contains(&text::lower(text::strip_chars(words.last().unwrap(), ",;:"))) {
        return None;
    }
    if !CONDITION_VERBS.contains(&text::lower(text::strip_chars(&words[0], ",;:"))) {
        return None;
    }
    Some((cstart(sentence, mend), cstart(sentence, referral.start()), words.join(" ")))
}

/// Python: bot/compose/verdict.py:4546 `_in_a_described_clause(sentence, at)`
pub fn _in_a_described_clause(sentence: &str, at: usize) -> bool {
    match _described_clause(sentence) {
        Some((from, to, _)) => from <= at && at < to,
        None => false,
    }
}

/// Python: bot/compose/verdict.py:4552 `_a_described_condition(sentence)`
pub fn _a_described_condition(sentence: &str) -> String {
    match _described_clause(sentence) {
        Some((_, _, words)) => format!("it {words}"),
        None => String::new(),
    }
}

/// Python: bot/compose/verdict.py:4563 `_an_if_that_is_a_condition(sentence)`: a character index,
/// or -1.
pub fn _an_if_that_is_a_condition(sentence: &str) -> isize {
    let mut start = 0;
    loop {
        let Some(at) = cfind(sentence, " if ", start) else { return -1 };
        let before = LETTERS.findall_str(&text::lower(cslice(sentence, 0, at)));
        if before.is_empty() || !LOOK_AND_SEE.contains(before.last().unwrap()) {
            return at as isize;
        }
        start = at + 4;
    }
}

/// Python: bot/compose/verdict.py:4592 `_a_judgement_fence(said)`
pub fn _a_judgement_fence(said: &str) -> String {
    let sentences: Vec<String> = SENTENCES.split(text::strip(said)).into_iter().filter(|s| !text::strip(s).is_empty()).collect();
    let Some(last) = sentences.last() else { return String::new() };
    if NAMES_A_CLINICIAN.search(last).is_none() {
        return String::new();
    }
    let Some(found) = A_JUDGEMENT_LIST.search(last) else { return String::new() };
    let clause = text::strip_chars(found.str(1), " ,;:-").to_string();
    if !clause.contains(',') && !clause.contains(" or ") {
        return String::new();
    }
    let n = text::split(&clause).len();
    if !(2..=26).contains(&n) {
        return String::new();
    }
    clause
}

/// Python: bot/compose/verdict.py:4618 `_after_the_if(sentence)`
pub fn _after_the_if(sentence: &str) -> String {
    let mut when;
    if let Some(opens) = IF_OPENS.match_(sentence) {
        when = sentence[opens.end()..].to_string();
        when = before_first(&AFTER_IF_OPENS, &when);
    } else {
        let Some(at) = sentence.find(" if ") else { return String::new() };
        when = text::rstrip_chars(&sentence[at + 4..], " .").to_string();
        when = before_first(&AFTER_IF_INSIDE, &when);
    }
    when = before_first(&RESTATED, &when);
    when = _no_hanging_end(text::strip_chars(&when, " ,."));
    if text::split(&when).len() >= 2 { when } else { String::new() }
}

/// Python: bot/compose/verdict.py:4652 `_when(said)`
pub fn _when(said: &str) -> String {
    for sentence in SENTENCES.split(said) {
        if REFER.search(&sentence).is_none() {
            continue;
        }
        let opens = IF_OPENS.match_(&sentence).is_some();
        if opens || _an_if_that_is_a_condition(&sentence) >= 0 {
            let when = _after_the_if(&sentence);
            if !when.is_empty() {
                return when;
            }
            continue;
        }
        let when = _a_described_condition(&sentence);
        if !when.is_empty() {
            return when;
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:4710 `_stated_before_the_judgement(sentence)`
pub fn _stated_before_the_judgement(sentence: &str) -> String {
    let Some(frame) = STATES_A_REFERRAL.match_(sentence) else { return String::new() };
    let mut when = _no_hanging_end(text::strip_chars(frame.name_str("when"), " ,;:"));
    let words = text::split(&when);
    if !(2..=*STATED_LONGEST).contains(&words.len()) {
        return String::new();
    }
    if NAMES_NOTHING.contains(&text::lower(text::strip_chars(&when, " .,"))) {
        return String::new();
    }
    if NAMES_A_CLINICIAN.search(&when).is_some() || ["only", "just"].contains(&text::lower(&words[0]).as_str()) {
        return String::new();
    }
    let head = &words[0];
    let first = text::slice(head, 0, Some(1));
    let rest = text::slice(head, 1, None);
    if text::is_upper(&first)
        && (text::is_lower(&rest) || clen(head) == 1)
        && !["i", "a&e", "nhs"].contains(&text::lower(head).as_str())
    {
        when = lower_first(&when);
    }
    when
}

/// Python: bot/compose/verdict.py:4734 `_stated_referral(said)`: (the threshold, "clause" or
/// "noun"), or ("", "").
pub fn _stated_referral(said: &str) -> (String, String) {
    for sentence in SENTENCES.split(said) {
        if !(JUDGED_REFERRAL.search(&sentence).is_some()
            || DECLARATIVE_REFERRAL.search(&sentence).is_some()
            || REFER.search(&sentence).is_some())
        {
            continue;
        }
        let opens = IF_OPENS.match_(&sentence).is_some();
        if opens || sentence.contains(" if ") {
            let mut when = _after_the_if(&sentence);
            if !when.is_empty() {
                when = OR_AND_IF.sub("\\1", &when);
                let first = AN_ASIDE.split(&when).into_iter().next().unwrap_or_default();
                when = _no_hanging_end(text::strip_chars(&first, " ,.;:"));
                when = text::strip_chars(when.split(", ask ").next().unwrap_or(""), " ,.;:").to_string();
                if text::split(&when).len() >= 2 {
                    return (when, "clause".into());
                }
            }
            continue;
        }
        let when = _stated_before_the_judgement(&sentence);
        if !when.is_empty() {
            return (when, "noun".into());
        }
    }
    (String::new(), String::new())
}

/// Python: bot/compose/verdict.py:4790 `_danger(said)`: a character index, or None.
pub fn _danger(said: &str) -> Option<usize> {
    let mut best: Option<usize> = None;
    for word in DANGER.iter() {
        let mut start = cfind(said, word, 0);
        while let Some(at) = start {
            let before = cslice(said, at.saturating_sub(14), at);
            let from = crfind_before(said, ". ", at).max(0) as usize;
            let sentence = cslice(said, from, at);
            let condition = IF_UNLESS.search(sentence).is_some();
            if NEGATED.search(before).is_none() && !condition {
                if best.map_or(true, |b| at < b) {
                    best = Some(at);
                }
                break;
            }
            start = cfind(said, word, at + clen(word));
        }
    }
    best
}

/// `AFTER.split(said)[1:] + re.split(r"(?<=[.!?]) ", said)`
fn the_parts_of(said: &str) -> Vec<String> {
    let mut parts: Vec<String> = AFTER.split(said).into_iter().skip(1).collect();
    parts.extend(SENTENCES.split(said));
    parts
}

/// Python: bot/compose/verdict.py:4812 `_a_how_to(said)`
pub fn _a_how_to(said: &str) -> bool {
    for part in the_parts_of(said) {
        for clause in CLAUSE_SPLIT.split(text::strip(&part)) {
            let words = text::split(text::strip_chars(&clause, " .:;,-"));
            if !words.is_empty() && DOING.contains(&text::lower(&words[0])) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/verdict.py:4838 `_round_robin(pool, said)`
pub fn _round_robin(pool: &[String], said: &str) -> Vec<String> {
    if pool.is_empty() || said.is_empty() {
        return pool.to_vec();
    }
    let total: u128 = said.chars().enumerate().map(|(i, c)| (i as u128 + 1) * c as u128).sum();
    let start = (total % pool.len() as u128) as usize;
    let mut out = pool[start..].to_vec();
    out.extend_from_slice(&pool[..start]);
    out
}

/// Python: bot/compose/verdict.py:4853 `_says_what_it_is(said)`
pub fn _says_what_it_is(said: &str) -> bool {
    for part in the_parts_of(said) {
        for clause in CLAUSE_SPLIT_COLON.split(text::strip(&part)) {
            let words = text::split(text::strip_chars(&clause, " .:;,-"));
            let leads: Vec<String> = words.iter().take(2).map(|w| text::lower(w)).collect();
            if words.len() >= 4 && !leads.iter().any(|w| DOING.contains(w)) {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------------------------
// The line the answer was answering, and how long the thing has gone on.
// ---------------------------------------------------------------------------------------------

/// Python: bot/compose/verdict.py:4911 `_the_line_it_answered(said, memory)`
pub fn _the_line_it_answered(said: &str, memory: &Memory) -> String {
    if said.is_empty() {
        return String::new();
    }
    let flat = _flat(said);
    for last in memory.turns.iter().rev() {
        if _flat(&last.bot) == flat {
            return _flat(&last.user);
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:4931 `_past_the_timescale(said, memory)`
pub fn _past_the_timescale(said: &str, memory: &Memory) -> bool {
    if said.is_empty() || HEDGE_SPAN.search(said).is_none() {
        return false;
    }
    let mut line = _the_line_it_answered(said, memory);
    if LONG_SPAN.search(&line).is_none() {
        let further = _the_span_further_up(said, memory);
        if !further.is_empty() {
            line = further;
        }
    }
    let mut ours = LONG_SPAN.search(&line).is_some();
    if ours && AGO.search(&line).is_some() {
        ours = false;
    }
    ours
}

/// Python: bot/compose/verdict.py:4954 `_subject_words(text)`
pub fn _subject_words(text_: &str) -> HashSet<String> {
    _words(text_).into_iter().filter(|w| clen(w) >= 5 && !FENCE_FILLER.contains(w)).map(|w| text::slice(&w, 0, Some(5))).collect()
}

/// Python: bot/compose/verdict.py:4960 `_the_span_further_up(said, memory)`
pub fn _the_span_further_up(said: &str, memory: &Memory) -> String {
    let turns = &memory.turns;
    if turns.len() < 2 {
        return String::new();
    }
    let here = _subject_words(said);
    if here.is_empty() {
        return String::new();
    }
    for turn in turns[..turns.len() - 1].iter().rev() {
        let line = _flat(&turn.user);
        if LONG_SPAN.search(&line).is_none() {
            continue;
        }
        if _subject_words(&turn.bot).iter().any(|w| here.contains(w)) {
            return line;
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:5004 `_stems(text)`
pub fn _stems(text_: &str) -> HashSet<String> {
    let mut words = HashSet::new();
    for word in LETTERS.findall_str(&text::lower(&_flat(text_))) {
        let mut word = text::strip_chars(&word, "'").to_string();
        if clen(&word) < 4 || FENCE_FILLER.contains(&word) {
            continue;
        }
        for suffix in ["ing", "ed", "es", "s"] {
            if word.ends_with(suffix) && clen(&word) - suffix.len() >= 4 {
                word.truncate(word.len() - suffix.len());
                break;
            }
        }
        words.insert(word);
    }
    words
}

/// Python: bot/compose/verdict.py:5019 `_the_fence_is_their_own_line(said, memory)`
pub fn _the_fence_is_their_own_line(said: &str, memory: &Memory) -> bool {
    let when = _when(said);
    if when.is_empty() {
        return false;
    }
    let fence = _stems(&when);
    let theirs = _stems(&_the_line_it_answered(said, memory));
    if fence.is_empty() || theirs.is_empty() {
        return false;
    }
    let shared = fence.intersection(&theirs).count();
    shared > 0 && shared * 2 >= fence.len()
}

/// Python: bot/compose/verdict.py:5060 `_a_mood(memory)`
pub fn _a_mood(memory: &Memory) -> bool {
    _the_last_person_lines(memory, 2).iter().any(|line| MOOD.search(line).is_some())
}

/// Python: bot/compose/verdict.py:5065 `_about_a_death(memory)`
pub fn _about_a_death(memory: &Memory) -> bool {
    for line in _the_last_person_lines(memory, 3) {
        if LOSS_WORDS.search(&line).is_none() {
            continue;
        }
        let words = crate::compose::loss::words_of(&line);
        if crate::compose::loss::a_thing_stopped(&line, &words) {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/compose/verdict.py:5085 `_the_last_person_lines(memory, count)`
pub fn _the_last_person_lines(memory: &Memory, count: usize) -> Vec<String> {
    let lines: Vec<String> = memory.turns.iter().map(|t| _flat(&t.user)).filter(|l| !l.is_empty()).collect();
    lines[lines.len().saturating_sub(count)..].to_vec()
}

/// Python: bot/compose/verdict.py:5113 `_a_job_to_do(text)`
pub fn _a_job_to_do(text_: &str) -> bool {
    for piece in STEP_PIECES.split(&_flat(text_)) {
        let words = _WORD.findall_str(text::strip_chars(&piece, " ,-"));
        if !words.is_empty() && _STEP_OPENERS.contains(&words[0]) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/verdict.py:5130 `_the_advice_itself(said)`
pub fn _the_advice_itself(said: &str) -> String {
    for sentence in SENTENCES.split(&text::squash(said)) {
        let mut clean = text::strip(&sentence).to_string();
        if clean.ends_with(':') {
            clean = text::strip(&clean[..clean.len() - 1]).to_string();
        }
        if text::split(&clean).len() >= 4 {
            if !_a_job_to_do(&clean) {
                return String::new();
            }
            return if text::ends_with_any(&clean, &[".", "!", "?"]) { clean } else { format!("{clean}.") };
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:5165 `_an_ask_for_a_call(memory)`
pub fn _an_ask_for_a_call(memory: &Memory) -> bool {
    let said = text::strip(&_flat(&_last_person_line(memory))).to_string();
    !said.is_empty() && ASKED_FOR_A_CALL.match_(&said).is_some()
}

/// Python: bot/compose/verdict.py:5171 `_a_fix_with_no_warning(said)`
pub fn _a_fix_with_no_warning(said: &str) -> bool {
    if said.is_empty() || clen(said) < 80 {
        return false;
    }
    if !_says_what_it_is(said) {
        return false;
    }
    if _first(said, &CARE).is_some() || REFER.search(said).is_some() {
        return false;
    }
    if _first(said, &REASSURE).is_some() || _first(said, &STOP_DOING).is_some() {
        return false;
    }
    _a_how_to(said)
}

/// Python: bot/compose/verdict.py:5224 `_somebody_owes_them(memory, back=3)`
pub fn _somebody_owes_them(memory: &Memory, back: usize) -> bool {
    let turns = &memory.turns;
    turns[turns.len().saturating_sub(back)..].iter().any(|last| OWED.search(&last.user).is_some())
}

/// Python: bot/compose/verdict.py:5238 `_a_reading_of_the_feeling(memory, flat)`
pub fn _a_reading_of_the_feeling(memory: &Memory, flat: &str) -> Option<String> {
    let behind = _the_line_behind(memory);
    let family = _a_feeling_of_their_own(&behind);
    if family.is_empty() {
        return None;
    }
    let seed = format!("{behind} {flat} {}", memory.turns.len());
    let pool = FEELING_DOUBT_LINES.get(&family).unwrap_or_else(|| panic!("KeyError: {family}"));
    _round_robin(pool, &seed).into_iter().next()
}

/// Python: bot/compose/verdict.py:5257 `_care_pool(said)`
pub fn _care_pool(said: &str) -> Vec<String> {
    let flat = _flat(said);
    if CARE_NOW.search(&flat).is_some() {
        return CARE_NOW_NORMAL.clone();
    }
    if _first_positive(&flat, &SAYS_NORMAL).is_some() {
        return ORDINARY_NORMAL.clone();
    }
    if _first_positive(&flat, &HOW_MANY_PEOPLE).is_some() {
        return CARE_NORMAL[..1].to_vec();
    }
    if _first_positive(&flat, &REASSURE).is_some() {
        return CARE_NORMAL.clone();
    }
    let mut pool = CARE_NORMAL.clone();
    pool.extend(CARE_NORMAL_UNKNOWABLE.iter().cloned());
    pool
}

/// Python: bot/compose/verdict.py:5291 `_the_knowledge_has_this(clean, memory, above)`
pub fn _the_knowledge_has_this(clean: &str, memory: &Memory, above: &str) -> Option<String> {
    let card = crate::compose::fallback_chain::text_of(crate::compose::safetyask::_the_answers_have_this(clean, memory))?;
    let said = _flat(&card);
    let opening = text::slice(&_flat(above), 0, Some(60));
    if !opening.is_empty() && said.contains(&opening) {
        return None;
    }
    Some(card)
}

/// Python: bot/compose/verdict.py:5317 `a_check_on_the_reply(reply)`
pub fn a_check_on_the_reply(reply: &str) -> bool {
    if reply.is_empty() {
        return false;
    }
    let t = text::strip(reply);
    [&*CHECK_WORKS_LINES, &*CHECK_ONCE_LINES, &*CHECK_WATCH_LINES, &*CHECK_JOB_LINES]
        .iter()
        .any(|pool| pool.iter().any(|line| text::strip(line) == t))
}

/// Python: bot/compose/verdict.py:5332 `_an_amount_asked(flat)`
pub fn _an_amount_asked(flat: &str) -> bool {
    ENOUGH_ASK.match_(flat).is_some() || HOW_MUCH.search(flat).is_some()
}

/// Python: bot/compose/verdict.py:5338 `_the_last_thing_said_is_a_nod(memory)`
pub fn _the_last_thing_said_is_a_nod(memory: &Memory) -> bool {
    memory.turns.last().is_some_and(_a_punt)
}

/// Python: bot/compose/verdict.py:5347 `_a_check_on_the_method(flat, memory)`
pub fn _a_check_on_the_method(flat: &str, memory: &Memory) -> Option<String> {
    let said = _last_reply(memory, true, false);
    if said.is_empty() || !_a_how_to(&said) {
        return None;
    }
    let behind = _flat(&_last_person_line(memory));
    if A_THINKING_ABOUT.search(&behind).is_some() || A_DECISION_ASK.search(&behind).is_some() {
        return None;
    }
    if A_PLAN.search(&behind).is_some() {
        return None;
    }
    if _last_module(memory, false) != "answers" {
        return None;
    }
    if _the_last_thing_said_is_a_nod(memory) {
        return None;
    }
    if _an_amount_asked(flat) && _the_knowledge_has_this(flat, memory, &said).is_some() {
        return None;
    }
    let pool: &Vec<String> = if !_about_a_body(&[&behind]) {
        &CHECK_JOB_LINES
    } else {
        // `_first(said.split(), STOP_DOING)` hands the list itself to `_first`, which reads its
        // repr: Python's `str(['word', ...]).lower()`.
        let warns = REFER.search(&said).is_some()
            || NAMES_A_CLINICIAN.search(&said).is_some()
            || _first(&py_list_repr(&text::split(&said)), &STOP_DOING).is_some();
        if warns {
            &CHECK_WATCH_LINES
        } else if A_LENGTH_IN_THE_ANSWER.search(&_flat(&said)).is_some() {
            &CHECK_WORKS_LINES
        } else {
            &CHECK_ONCE_LINES
        }
    };
    Some(pick(&_round_robin(pool, &format!("{flat} {said}")), memory, &said))
}

// ---------------------------------------------------------------------------------------------
// The doubt about the answer above.
// ---------------------------------------------------------------------------------------------

fn either_spelling(rx: &Re, line: &str, written: &str, spoken: &str) -> bool {
    rx.search(line).is_some() || rx.search(written).is_some() || rx.search(spoken).is_some()
}

fn concat(pools: &[&Vec<String>]) -> Vec<String> {
    pools.iter().flat_map(|p| p.iter().cloned()).collect()
}

/// Python: bot/compose/verdict.py:5422 `reply_for(clean, memory)`: the answer to a bare "is
/// that bad / what should i do / should i see someone", or None. (The skeleton had this
/// returning nothing; Python returns the line or None.)
pub fn reply_for(clean: &str, memory: &Memory) -> Option<String> {
    let flat = text::strip_chars(&_flat(clean), " .!?").to_string();
    if crate::compose::safetyask::the_bot_already_ruled(memory)
        && A_DOUBT_ABOUT_THE_THING.search(&flat).is_some()
        && !A_RULING_ALREADY.iter().any(|word| flat.contains(word.as_str()))
    {
        return Some(pick(&A_SAFETY_ALREADY_RULED, memory, &flat));
    }
    if A_CHECK_ON_A_PLAN.match_(&flat).is_some() {
        let plan = _the_plan_they_brought(memory, 2);
        if !plan.is_empty() {
            return Some(pick(&_round_robin(&A_CHECK_ON_A_PLAN_LINES, &plan), memory, &plan));
        }
    }
    if METHOD_DOUBT.match_(&flat).is_some() {
        let behind = _last_reply(memory, true, false);
        let line = _the_line_behind(memory);
        let (written, spoken) = _as_written(&line);
        if behind.is_empty() || !_a_how_to(&behind) || !either_spelling(&A_HOW_TO_ASK, &line, &written, &spoken) {
            return _a_check_on_the_method(&flat, memory);
        }
        if _danger(&behind).is_some() {
            return None;
        }
        if REFER.search(&behind).is_some() || NAMES_A_CLINICIAN.search(&behind).is_some() {
            return _a_check_on_the_method(&flat, memory);
        }
        if let Some(checked) = _a_check_on_the_method(&flat, memory) {
            if !checked.is_empty() {
                return Some(checked);
            }
        }
        return Some(pick(&_round_robin(&METHOD_OK_LINES, &behind), memory, &behind));
    }
    if A_FIGURE_ASK.match_(&flat).is_some() {
        let figure_line = _the_line_behind(memory);
        let figure_said = _last_reply(memory, true, false);
        if !figure_line.is_empty() && _a_figure_owed(&figure_line) && !figure_said.is_empty() {
            if _first(&figure_said, &A_COMPARISON).is_some() {
                return Some(pick(&MONEY_LINES, memory, &figure_said));
            }
            if A_FIGURE.search(&figure_said).is_some() {
                return Some(pick(&FIGURE_LINES, memory, &figure_said));
            }
        }
        return None;
    }
    let enough_ask = ENOUGH_ASK.match_(&flat).is_some();
    if BARE.match_(&flat).is_none() && !enough_ask {
        if A_CHECK_ASK.search(&flat).is_some() {
            return _a_check_on_the_method(&flat, memory);
        }
        return None;
    }
    let normal_ask = ASKS_NORMAL.search(&flat).is_some();
    let ok_ask = !normal_ask && ASKS_OK.search(&flat).is_some();
    let said = _last_reply(memory, true, false);
    let plan = _the_line_behind(memory);
    if let Some(family) = _the_plan_can_hurt(&_flat(&plan)) {
        let warned = _the_answer_already_warned(&_flat(&said));
        if !warned.is_empty() {
            return Some(format!("Go with the answer above rather than around it: {warned}"));
        }
        let pool = THE_PLAN_CAN_HURT_LINES.get(&family).unwrap_or_else(|| panic!("KeyError: {family}"));
        let seed = _flat(&plan);
        return Some(pick(&_round_robin(pool, &seed), memory, &seed));
    }
    if _about_a_death(memory) && !_last_module(memory, false).is_empty() {
        let mut seed = _the_line_it_answered(&said, memory);
        if seed.is_empty() {
            seed = flat.clone();
        }
        return Some(pick(&_round_robin(&GRIEF_FLOOR_LINES, &seed), memory, &flat));
    }
    if enough_ask {
        let Some(stated) = _said_enough(&said) else {
            if _last_module(memory, false) == "answers" {
                if let Some(checked) = _a_check_on_the_method(&flat, memory) {
                    if !checked.is_empty() {
                        return Some(checked);
                    }
                }
            }
            return None;
        };
        return Some(pick(&_round_robin(&ENOUGH_STATED_LINES, &format!("{said} {flat}")), memory, &said).replace("{yes}", &stated));
    }
    if _considering(memory) || A_PLAN.search(&_flat(&_the_line_behind(memory))).is_some() {
        return _a_plan_doubt(memory);
    }
    let worst = _the_worst_thing_they_said(memory);
    if !worst.is_empty() && SYMPATHY.search(&_flat(&said)).is_some() {
        let seed = format!("{worst} {flat} {}", memory.turns.len());
        return _round_robin(&HARD_NEWS_LINES, &seed).into_iter().next();
    }
    let money = _a_doubt_about_money(memory);
    if !money.is_empty() && ["answers", "chat", "greeting", "trouble"].contains(&_last_module(memory, true).as_str()) {
        if !_about_a_death(memory)
            && _the_worst_thing_they_said(memory).is_empty()
            && !(_A_NUMBER.search(&money).is_some() && _is_a_comparison(&said))
        {
            let seed = if said.is_empty() { money.clone() } else { said.clone() };
            return Some(pick(&_round_robin(&MONEY_TROUBLE_LINES, &seed), memory, &seed));
        }
    }
    if said.is_empty() || _last_module(memory, true) != "answers" {
        if let Some(reading) = _a_reading_of_the_feeling(memory, &flat) {
            if !reading.is_empty() {
                return Some(reading);
            }
        }
        let person = _flat(&_last_person_line(memory));
        if HELPED.search(&person).is_some() {
            return Some(pick(&_round_robin(&HELPED_LINES, &person), memory, &person));
        }
        let named = _last_line_that_named_a_call(memory);
        if !named.is_empty() {
            if _a_call_for_now(&named) {
                let call = _the_call_sentence(&named);
                let seed = format!("{flat} {call}");
                if !call.is_empty() && URGENT_CALL.search(&call).is_some() {
                    return Some(pick(&_round_robin(&YES_SAID_LINES, &call), memory, &seed));
                }
                if TRADE.search(&named).is_none() {
                    return Some(pick(&YES_LINES[..YES_LINES.len().min(2)], memory, &flat));
                }
                return Some(pick(&SOON_LINES, memory, &flat));
            }
            let when = _when(&named);
            if !when.is_empty() {
                return Some(pick(&CONDITION_LINES, memory, &flat).replace("{when}", &when));
            }
        }
        if WORRIED_ASK.match_(&flat).is_some() {
            return None;
        }
        if ["chat", "greeting", "trouble"].contains(&_last_module(memory, false).as_str()) {
            if let Some(reading) = _a_reading_of_the_feeling(memory, &flat) {
                if !reading.is_empty() {
                    return Some(reading);
                }
            }
            if _own_doing(memory) && _a_mood(memory) {
                return Some(pick(&_round_robin(&BACKSTOP_LINES, &said), memory, &said));
            }
            let step = _the_advice_itself(&said);
            if !step.is_empty() && _an_ask_for_a_call(memory) && A_REFUSAL.search(&said).is_none() {
                return Some(pick(&_round_robin(&ADVICE_OK_LINES, &said), memory, &said).replace("{step}", &step));
            }
            if _own_doing(memory) && _a_fix_with_no_warning(&said) {
                return Some(pick(&_round_robin(&NOTHING_WARNED_LINES, &said), memory, &said));
            }
            if _own_doing(memory) && !_nothing_to_be_long_about(memory) {
                return Some(pick(&_round_robin(&FLOOR_LINES, &said), memory, &said));
            }
            if _about_the_person(memory) && !_nothing_to_be_long_about(memory) {
                return Some(pick(&_round_robin(&NEUTRAL_ASK_LINES, &said), memory, &said));
            }
            if _the_floor_asked_them_something(memory) {
                let seed = format!("{said} {flat}");
                return Some(pick(&_round_robin(&ASKED_THEM_LINES, &seed), memory, &seed));
            }
        }
        return _a_plan_doubt(memory);
    }
    if WHEN_ASK.match_(&flat).is_some() || WORRYING_ASK.match_(&flat).is_some() {
        let mut when = _when(&said);
        if when.is_empty() {
            when = _the_list_before_the_call(&said);
        }
        if when.is_empty() {
            when = _stated_referral(&said).0;
        }
        if !when.is_empty() {
            return Some(pick(&WHEN_LINES, memory, &flat).replace("{when}", &when));
        }
        let mut call = _the_call_and_its_reason(&said);
        if !call.is_empty() && _they_named_the_sign_themselves(&call, memory) {
            call = String::new();
        }
        if !call.is_empty() {
            return Some(pick(&WHEN_CALL_LINES, memory, &flat).replace("{when}", &call));
        }
        if WORRYING_ASK.match_(&flat).is_some() {
            return None;
        }
    }
    if _the_last_line_was_mine(memory) && OWN_PATTERN.search(&_flat(&said)).is_some() {
        return Some(pick(&AGAIN, memory, &flat));
    }
    let flat_said = _flat(&said);
    let mine = concat(&[
        &NO_LINES, &DO_LINES, &YES_LINES, &ORDINARY_NORMAL, &CARE_NORMAL, &CARE_NORMAL_UNKNOWABLE, &CARE_NOW_NORMAL, &DO_NORMAL,
    ]);
    if mine.iter().any(|line| flat_said.contains(&text::lower(line))) {
        return Some(pick(&AGAIN, memory, &flat));
    }
    if _first(&said, &AGAIN).is_some() {
        return None;
    }
    if !_a_person_they_know(memory, &said).is_empty() {
        let seed = format!("{flat} {}", _the_line_behind(memory));
        if !_when(&said).is_empty() || REFER.search(&said).is_some() {
            return Some(pick(&PERSON_CALL, memory, &seed));
        }
        return Some(pick(&PERSON_ORDINARY, memory, &seed));
    }
    let opening = _opening(&said);
    let thing = _a_thing_they_own(memory);
    let yes_pool: &Vec<String> = if !thing.is_empty() { &THING_CALL } else { &YES_LINES };
    let (action, tested) = _a_throw_away(&opening);
    if let Some(action) = action {
        let pool: &Vec<String> =
            if tested && !_the_person_gave_the_sign(memory, clean) { &TEST_LINES } else { &THROW_AWAY_LINES };
        let seed = format!("{opening} {}", _the_person_line(memory, clean));
        return Some(pick(pool, memory, &seed).replace("{Action}", &action));
    }
    if let Some(no_stated) = _said_no(&said) {
        return Some(pick(&_round_robin(&NO_STATED_LINES, &format!("{said} {flat}")), memory, &said).replace("{no}", &no_stated));
    }
    let care = _referral_at(&said);
    let ordinary = _first_positive(&opening, &REASSURE);
    let anywhere = care;
    let a_yes = |memory: &Memory| -> String {
        if normal_ask {
            pick(&_care_pool(&said), memory, &flat)
        } else if !thing.is_empty() {
            pick(&THING_CALL, memory, &flat)
        } else {
            pick(&YES_LINES[..YES_LINES.len().min(2)], memory, &flat)
        }
    };
    if _first(&opening, &STOP_DOING).is_some() {
        return Some(a_yes(memory));
    }
    let when = _when(&said);
    if when.is_empty() && _danger(&said).is_some() {
        return Some(a_yes(memory));
    }
    let line = _the_line_behind(memory);
    let (written, spoken) = _as_written(&line);
    let warns = REFER.search(&said).is_some()
        || NAMES_A_CLINICIAN.search(&said).is_some()
        || _first(&opening, &STOP_DOING).is_some();
    if SAID_YES.search(&_flat(&said)).is_some() && _a_how_to(&said) && !warns {
        return Some(pick(&_round_robin(&METHOD_OK_LINES, &said), memory, &said));
    }
    if either_spelling(&A_HOW_TO_ASK, &line, &written, &spoken) && _a_how_to(&said) && !warns {
        return Some(pick(&_round_robin(&METHOD_OK_LINES, &said), memory, &said));
    }
    if either_spelling(&A_PLAN, &line, &written, &spoken) {
        return _a_plan_doubt(memory);
    }
    if either_spelling(&A_DECISION_ASK, &line, &written, &spoken) {
        if ASKS_FINE.search(&flat).is_some() && ASKS_WRONG.search(&flat).is_none() {
            let seed = format!("{line} {said}");
            return Some(pick(&_round_robin(&CHOICE_OK_LINES, &seed), memory, &seed));
        }
        let first_said = SENTENCES.split(text::strip(&said)).into_iter().next().unwrap_or_default();
        if !warns
            || (_referral_at(&first_said).is_none()
                && NAMES_A_CLINICIAN.search(&first_said).is_none()
                && REFER.search(&first_said).is_none())
        {
            let fenced = _condition_after_care(&said, _first(&text::lower(&said), &CARE), None);
            if !fenced.is_empty() {
                let pool: &Vec<String> = if normal_ask { &CONDITION_NORMAL } else { &CONDITION_LINES };
                return Some(pick(pool, memory, &flat).replace("{when}", &fenced));
            }
            let (stated, kind) = _stated_referral(&said);
            let sentence = the_referral_sentence(&said);
            if !stated.is_empty()
                && kind == "noun"
                && TIME_NOW.search(&sentence).is_none()
                && COMPARATIVE.search(&sentence).is_none()
                && !_the_line_is_the_threshold(&stated, &_the_line_it_answered(&said, memory))
            {
                if _past_the_timescale(&said, memory) {
                    return Some(pick(&_round_robin(&MET_LINES, &said), memory, &said));
                }
                return Some(_say_with(&SEE_STATED_LINES, memory, &flat, &[("when", &stated)]));
            }
            return None;
        }
        if _care_for_now(&first_said) {
            let pool: &Vec<String> = if URGENT_CALL.search(&first_said).is_some() { &YES_SAID_LINES } else { &SOON_LINES };
            return Some(pick(pool, memory, &flat));
        }
        return None;
    }
    if JUST_AGE.search(&flat).is_some() {
        return Some(pick(&AGE_LINES, memory, &said));
    }
    if _a_figure_owed(&line) && _is_a_comparison(&said) {
        return Some(pick(&MONEY_LINES, memory, &said));
    }
    let mut care_at = _first(&text::lower(&said), &CARE);
    if let Some(c) = care {
        if care_at.map_or(true, |at| c < at) {
            care_at = Some(c);
        }
    }
    if _past_the_timescale(&said, memory) {
        return Some(pick(&_round_robin(&MET_LINES, &said), memory, &said));
    }
    if care_at.is_some() {
        let fenced = _condition_after_care(&said, care_at, Some(memory));
        if !fenced.is_empty() {
            let pool: &Vec<String> = if normal_ask { &CONDITION_NORMAL } else { &CONDITION_LINES };
            return Some(pick(pool, memory, &flat).replace("{when}", &fenced));
        }
    }
    let denies = _first(&opening, &DENIES);
    if let Some(c) = care {
        if ordinary.map_or(true, |o| c < o) || _a_plain_go(c, &said) {
            let pool = if normal_ask { _care_pool(&said) } else { yes_pool.clone() };
            return Some(pick(&pool, memory, &flat));
        }
        if ordinary.map_or(true, |o| c < o) {
            return Some(pick(&YES_LINES, memory, &flat));
        }
    }
    if denies.is_none() && anywhere.is_some() && ordinary.is_some() {
        let pool = if normal_ask { _care_pool(&said) } else { yes_pool.clone() };
        return Some(pick(&pool, memory, &flat));
    }
    if ordinary.is_some() {
        if !when.is_empty() && normal_ask {
            return Some(pick(&CONDITION_NORMAL, memory, &flat).replace("{when}", &when));
        }
        if normal_ask && _first_positive(&opening, &SAYS_NORMAL).is_none() && _a_how_to(&said) {
            return Some(pick(&_round_robin(&NOTHING_WARNED_NORMAL, &said), memory, &said));
        }
        if _care_for_now(&said) {
            return Some(pick(&BOTH_LINES, memory, &flat));
        }
        let fence = _a_judgement_fence(&said);
        if !fence.is_empty() {
            return Some(pick(&CONDITION_LINES, memory, &flat).replace("{when}", &fence));
        }
        if warns && _when(&said).is_empty() && _referral_about_the_thing(&said, &_the_line_behind(memory)) {
            return Some(pick(&SEE_ANYWAY_LINES, memory, &flat));
        }
        if !thing.is_empty() && !normal_ask {
            return Some(pick(&THING_ORDINARY, memory, &format!("{flat} {thing}")));
        }
        let pool: &Vec<String> = if normal_ask {
            &ORDINARY_NORMAL
        } else if ok_ask {
            &OK_LINES
        } else {
            &NO_LINES
        };
        return Some(pick(pool, memory, &flat));
    }
    let when = _when(&said);
    if !when.is_empty() {
        if _the_fence_is_their_own_line(&said, memory) {
            if _a_call_for_now(&said) {
                return Some(pick(&YES_LINES, memory, &flat));
            }
            return Some(pick(&_round_robin(&MET_CONDITION_LINES, &said), memory, &said));
        }
        let pool: &Vec<String> = if normal_ask { &CONDITION_NORMAL } else { &CONDITION_LINES };
        return Some(pick(pool, memory, &flat).replace("{when}", &when));
    }
    if care.is_some() || anywhere.is_some() {
        let pool: &Vec<String> = if normal_ask { &DO_NORMAL } else { &DO_LINES };
        return Some(pick(pool, memory, &flat));
    }
    if _first(&opening, &CARE).is_some() {
        if _first(&opening, &PEOPLE_CARE).is_some() {
            return Some(pick(&YES_LINES, memory, &flat));
        }
        let pool: &Vec<String> = if normal_ask { &DO_NORMAL } else { &DO_LINES };
        return Some(pick(pool, memory, &flat));
    }
    if _a_fix_with_no_warning(&said) {
        if _somebody_owes_them(memory, 3) {
            return Some(pick(&_round_robin(&OWED_LINES, &said), memory, &said));
        }
        let pool: &Vec<String> = if normal_ask { &NOTHING_WARNED_NORMAL } else { &NOTHING_WARNED_LINES };
        return Some(pick(&_round_robin(pool, &said), memory, &said));
    }
    if _an_instruction_and_nothing_else(&said, memory) {
        let referral = _the_referral_it_closed_on(&said);
        let seed = format!("{said} {flat}");
        if !referral.is_empty() {
            let kept = _the_range_it_gave(&said);
            let said_back = if kept.is_empty() { referral } else { format!("{referral} {kept}") };
            return Some(pick(&_round_robin(&HANDED_BACK_LINES, &seed), memory, &seed).replace("{referral}", &said_back));
        }
        return Some(pick(&_round_robin(&DIRECTED_LINES, &seed), memory, &seed));
    }
    None
}

/// Python: bot/compose/verdict.py:6161 `_the_referral_it_closed_on(said)`
pub fn _the_referral_it_closed_on(said: &str) -> String {
    let sentences: Vec<String> =
        SENTENCES_SPACES.split(&_flat(said)).iter().map(|s| text::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    let Some(last) = sentences.last() else { return String::new() };
    if text::split(last).len() < 5 {
        return String::new();
    }
    if !(REFER.search(last).is_some()
        || _A_REFERRAL_OUT.search(last).is_some()
        || _A_STOP.search(last).is_some()
        || _first(last, &STOP_DOING).is_some())
    {
        return String::new();
    }
    let unfenced = FENCE_IN_FRONT.sub("", last);
    let act = A_STOP_IN_FRONT.sub("", &unfenced);
    if text::split(&act).len() >= 4 {
        return act;
    }
    if text::split(&unfenced).len() >= 4 { unfenced } else { last.clone() }
}

/// Python: bot/compose/verdict.py:6193 `_the_range_it_gave(said)`
pub fn _the_range_it_gave(said: &str) -> String {
    let flat = _flat(said);
    let Some(found) = _A_RANGE_OF_ITS_OWN.search(&flat) else { return String::new() };
    let (low, high, unit) = (found.str(1), found.str(2), found.str(3));
    if ["and", "the", "for", "with"].contains(&text::lower(unit).as_str()) {
        return String::new();
    }
    format!("Keep it between {low} and {high} {unit} in the meantime.")
}

/// Python: bot/compose/verdict.py:6218 `_an_instruction_and_nothing_else(said, memory)`
pub fn _an_instruction_and_nothing_else(said: &str, memory: &Memory) -> bool {
    let flat = _flat(said);
    if text::split(&flat).len() < 12 {
        return false;
    }
    let head = before_first(&HEAD_BREAK, &flat);
    let words = _WORD.findall_str(text::strip_chars(&head, " ,-"));
    if words.is_empty() || !_STEP_OPENERS.contains(&text::lower(&words[0])) {
        return false;
    }
    let line = _flat(&_last_person_line(memory));
    if line.is_empty() || A_HOW_TO_ASK.search(&line).is_some() {
        return false;
    }
    true
}

/// Python: bot/compose/verdict.py:6277 `_the_floor_asked_them_something(memory)`
pub fn _the_floor_asked_them_something(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said = text::squash(&last.bot);
    if !(said.ends_with('?') || _AN_OFFER.search(&said).is_some()) {
        return false;
    }
    let line = _flat(&_the_line_behind(memory));
    if line.is_empty() || A_HOW_TO_ASK.search(&line).is_some() {
        return false;
    }
    A_WORRY_IN_THE_LINE.search(&line).is_some()
}

/// Python: bot/compose/verdict.py:6300 `_a_feeling_of_their_own(line)`
pub fn _a_feeling_of_their_own(line: &str) -> String {
    let words: Vec<String> = A_WORD_OF_LETTERS.findall_str(line).iter().map(|w| text::lower(&w.replace('\'', ""))).collect();
    if words.is_empty() || words.len() > 14 {
        return String::new();
    }
    if !["i", "im", "ive", "id"].contains(&words[0].as_str()) {
        return String::new();
    }
    if words.iter().any(|w| BODY_WORD.contains(w)) || words.iter().any(|w| SOMEONE_ELSE.contains(w)) {
        return String::new();
    }
    for word in &words {
        if let Some(family) = FEELING_FAMILY.get(word) {
            return family.clone();
        }
    }
    String::new()
}

/// `" ".join(str(step or "").split()).lower().strip(" .!?,")`
fn needle_of(step: Option<&str>) -> String {
    text::strip_chars(&text::lower(&text::squash(step.unwrap_or(""))), " .!?,").to_string()
}

/// Python: bot/compose/verdict.py:6322 `_the_other_thing_in_the_answer(said, step, memory)`
pub fn _the_other_thing_in_the_answer(said: &str, step: Option<&str>, memory: &Memory) -> String {
    let needle = needle_of(step);
    let mine = _flat(said);
    let seen: Vec<String> = memory.turns.iter().map(|t| _flat(&t.bot)).collect();
    let earlier: Vec<&String> =
        seen[..seen.len().saturating_sub(1)].iter().filter(|heard| !heard.is_empty() && **heard != mine).collect();
    let sentences = SENTENCES.split(said);
    let starts: Vec<usize> = sentences
        .iter()
        .enumerate()
        .filter(|(_, one)| !needle.is_empty() && text::strip_chars(&text::lower(one), " .!?,").contains(needle.as_str()))
        .map(|(i, _)| i)
        .collect();
    let from = if let Some(first) = starts.first() { first + 1 } else { 1 };
    for sentence in sentences.iter().skip(from) {
        let t = text::strip(&text::squash(sentence)).to_string();
        if text::split(&t).len() < 6 || _is_a_question(&t) {
            continue;
        }
        if !needle.is_empty() && text::lower(&t).contains(needle.as_str()) {
            continue;
        }
        if RED_FLAG.match_(&t).is_some() {
            continue;
        }
        if ASKS_THEM.match_(&t).is_some() {
            continue;
        }
        let low = text::lower(&t);
        if earlier.iter().any(|heard| heard.contains(low.as_str())) {
            continue;
        }
        return t;
    }
    String::new()
}

/// Python: bot/compose/verdict.py:6363 `_step_sentence(said, step)`
pub fn _step_sentence(said: &str, step: &str) -> String {
    let needle = needle_of(Some(step));
    if needle.is_empty() {
        return String::new();
    }
    for sentence in SENTENCES.split(said) {
        let t = text::squash(&sentence);
        if text::lower(&t).contains(needle.as_str()) {
            return text::strip_chars(&t, " .!?").to_string();
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:6383 `_up_to_a_break(said, step, within)`
pub fn _up_to_a_break(said: &str, step: &str, within: usize) -> String {
    let mut said = said.to_string();
    let mut words = text::split(&said);
    if words.len() <= within {
        return said;
    }
    let joined = words.join(" ");
    let opening = cfind(&text::lower(&joined), &text::lower(&text::squash(step)), 0).map(|i| i as isize).unwrap_or(-1);
    let start = if opening > 0 { text::split(cslice(&joined, 0, opening as usize)).len() } else { 0 };
    if start > 0 && start < words.len() - 1 {
        said = words[start..].join(" ");
        words = text::split(&said);
        if words.len() <= within {
            return said;
        }
    }
    let mut best = String::new();
    for found in A_BREAK.finditer(&said) {
        let head = text::strip_chars(&said[..found.start()], " ,;:-\u{2013}\u{2014}").to_string();
        let n = text::split(&head).len();
        if n <= within && n > text::split(&best).len() {
            best = head;
        }
    }
    if best.is_empty() { words[..within.min(words.len())].join(" ") } else { best }
}

/// Python: bot/compose/verdict.py:6415 `_an_instruction_sentence(said)`
pub fn _an_instruction_sentence(said: &str) -> String {
    for sentence in SENTENCES.split(said) {
        let t = text::squash(&sentence);
        if t.is_empty() || CONDITIONAL_OPEN.match_(&t).is_some() || RED_FLAG.match_(&t).is_some() || _is_a_question(&t) {
            continue;
        }
        if A_STATEMENT.match_(&t).is_some() {
            continue;
        }
        let words: Vec<String> = ENGLISH_WORDS.findall_str(&t).into_iter().take(5).collect();
        if IMPERATIVES.contains(&_doing_word(&words)) {
            return text::strip_chars(&t, " .!?").to_string();
        }
        let (_, colon, tail) = text::partition(&t, ": ");
        if !colon.is_empty() && !tail.is_empty() {
            let words: Vec<String> = ENGLISH_WORDS.findall_str(tail).into_iter().take(5).collect();
            if IMPERATIVES.contains(&_doing_word(&words)) {
                return text::strip_chars(&t, " .!?").to_string();
            }
        }
        if FIRST_ITSELF.search(&t).is_some() && COPULA.match_(&t).is_none() && text::split(&t).len() >= 5 {
            return text::strip_chars(&t, " .!?").to_string();
        }
    }
    String::new()
}

/// Python: bot/compose/verdict.py:6446 `_the_answer_under_a_frame(memory)`
pub fn _the_answer_under_a_frame(memory: &Memory) -> String {
    let Some(last) = memory.turns.last() else { return String::new() };
    if in_set(&UNREADABLE, &last.module) {
        return String::new();
    }
    crate::memory::again::unframed(&last.bot).unwrap_or_default()
}

/// Python: bot/compose/verdict.py:6482 `_the_front_names_what_is_in_hand(head, line, memory)`
pub fn _the_front_names_what_is_in_hand(head: &str, line: &str, memory: &Memory) -> bool {
    let words = ENGLISH_WORDS.findall_str(&text::lower(head));
    if words.iter().any(|w| FRONT_WHEN.contains(w)) {
        return false;
    }
    if words.first().is_some_and(|w| FRONT_THIS.contains(w)) {
        return true;
    }
    let subject: Vec<&String> = words.iter().filter(|w| !FRONT_LEAD.contains(*w)).take(*FRONT_SUBJECT_WORDS).collect();
    if subject.is_empty() {
        return false;
    }
    let mut in_hand: HashSet<String> = LOWER_WORDS.findall_str(&text::lower(line)).into_iter().collect();
    let mut topics: Vec<Value> = vec![memory.get("topic").cloned().unwrap_or(Value::None)];
    if let Some(more) = memory.get("topics") {
        topics.extend(more.items().iter().cloned());
    }
    for topic in topics {
        if topic.is_dict() {
            if let Some(words) = topic.get("words") {
                for word in words.items() {
                    in_hand.insert(crate::compose::fallback_chain::py_str(Some(word)));
                }
            }
        }
    }
    for turn in &memory.turns {
        in_hand.extend(LOWER_WORDS.findall_str(&text::lower(&turn.user)));
    }
    subject.iter().any(|w| in_hand.contains(*w))
}

/// Python: bot/compose/verdict.py:6511 `next_step(clean, memory)`: the step asked for, from the
/// advice given, or None. (The skeleton had this returning nothing; Python returns the line or
/// None.)
pub fn next_step(clean: &str, memory: &Memory) -> Option<String> {
    let flat = text::strip_chars(&_flat(clean), " .!?").to_string();
    let bare = STEP.match_(&flat).is_some();
    let someone_new = memory.understood.as_ref().is_some_and(|u| u.someone_new);
    let mut asks_step = bare || (SPEAK_UP.match_(&flat).is_some() && !someone_new);
    let asks_who = SEE.match_(&flat).is_some();
    let plea = TIPS_AGAIN.match_(&flat).is_some() || ONE_THING.match_(&flat).is_some();
    let tips = TIPS_AGAIN.match_(&flat).is_some();
    let how_to_asked = match HOW_TO.match_(&flat) {
        Some(how_to) => {
            let doing = how_to.name("doing");
            doing.is_some_and(|d| DOING_WITH_NO_OBJECT.contains(d))
                || (how_to.name("obj").is_some_and(|o| !o.is_empty()) && !doing.is_some_and(|d| KNOWING_DOING.contains(d)))
        }
        None => false,
    };
    if how_to_asked {
        asks_step = true;
    }
    if plea && _last_module(memory, false) == "answers" {
        asks_step = true;
    }
    if A_HOW_ASK.match_(&flat).is_some() && _last_module(memory, false) == "answers" {
        asks_step = true;
    }
    if !(asks_step || asks_who) {
        return None;
    }
    let mut said = _last_reply(memory, false, true);
    if said.is_empty() {
        said = _the_answer_under_a_frame(memory);
    }
    if said.is_empty() {
        return None;
    }
    if _first(&said, &AGAIN).is_some() {
        said = crate::memory::again::unframed(&said).unwrap_or_default();
        if said.is_empty() {
            return None;
        }
    }
    if EMERGENCY_CALL.match_(text::strip(&said)).is_some() {
        return None;
    }
    let when = _when(&said);
    if asks_who {
        if !when.is_empty() {
            return Some(_say_with(&SEE_NOW_LINES, memory, &flat, &[("when", &when)]));
        }
        let (stated, kind) = _stated_referral(&said);
        if !stated.is_empty() {
            let pool: &Vec<String> = if kind == "clause" { &SEE_NOW_LINES } else { &SEE_STATED_LINES };
            return Some(_say_with(pool, memory, &flat, &[("when", &stated)]));
        }
        if REFER.search(&said).is_some() {
            return Some(pick(&SEE_ANYWAY_LINES, memory, &flat));
        }
        if _first(&said, &CARE).is_some() {
            return Some(pick(&SEE_NONE_LINES, memory, &flat));
        }
        return None;
    }
    let mut step = the_step(&said);
    if step.is_none() && _ends_on_a_question(&said) {
        step = the_step(&_the_answer_before_that(memory));
    }
    if step.as_deref().is_some_and(|s| _echoes_the_person(s, memory)) {
        step = None;
    }
    if tips {
        let other = _the_other_thing_in_the_answer(&said, step.as_deref(), memory);
        if !other.is_empty() {
            return Some(_say_with(&ALSO_LINES, memory, &flat, &[("other", &other)]));
        }
        if step.is_none() {
            return Some(pick(&STEP_KEPT_LINES, memory, &flat));
        }
    }
    if let Some(s) = &step {
        if _already_asked_for_a_step(memory) || _most_of_the_line(s, &_the_line_just_given(memory)) || tips {
            return Some(pick(&STEP_KEPT_LINES, memory, &flat));
        }
    }
    let Some(mut step) = step else {
        if _an_offer_is_waiting(&said) {
            return None;
        }
        if bare && _ends_on_a_question(&said) {
            if _the_news_just_before(memory) {
                return None;
            }
            let asked = _the_question_asked(&said);
            if !asked.is_empty() && THE_PLEA.match_(&flat).is_some() {
                return Some(pick(&QUESTION_BACK_LINES, memory, &flat));
            }
            if !asked.is_empty() && _after_a_comfort_line(memory) {
                return Some(pick(&QUESTION_BACK_LINES, memory, &flat));
            }
            if !asked.is_empty() && AN_OFFER.search(&asked).is_some() {
                return Some(pick(&OFFER_TAKEN_LINES, memory, &flat));
            }
            if !asked.is_empty() {
                return Some(_say_with(&ASKED_FIRST_LINES, memory, &flat, &[("q", &asked)]));
            }
            return Some(pick(&QUESTION_BACK_LINES, memory, &flat));
        }
        if let Some(listed) = _listed(&said) {
            if !listed.is_empty() {
                return Some(_say_with(&LIST_STEP_LINES, memory, &flat, &[("step", &listed)]));
            }
        }
        if !when.is_empty() && !plea && _action_count(&said) < 2 {
            return Some(pick(&NOTHING_LINES, memory, &flat).replace("{when}", &when));
        }
        if ONE_THING.match_(&flat).is_some() && _last_module(memory, false) == "answers" {
            let mut moved = _an_instruction_sentence(&said);
            if !moved.is_empty() {
                let head = text::strip_chars(text::partition(&moved, ": ").0, " .;:,-").to_string();
                if moved.contains(": ") && text::split(&head).len() >= 8 {
                    moved = head;
                } else if text::split(&moved).len() > *HAND_OVER_WORDS {
                    let cut = text::strip_chars(&_first_clause(&moved, true), " .;:,-").to_string();
                    if !cut.is_empty() {
                        moved = cut;
                    }
                }
                return Some(_say_with(&ONE_TURN_PLAIN_LINES, memory, &flat, &[("step", &moved)]));
            }
            return Some(_say(&ALL_OF_IT_LINES, memory, &flat));
        }
        return None;
    };
    if text::lower(&step).ends_with(" first") {
        let n = clen(&step);
        step = text::rstrip(cslice(&step, 0, n.saturating_sub(6))).to_string();
    }
    if ONE_THING.match_(&flat).is_some() {
        let whole = _step_sentence(&said, &step);
        let piece = if whole.is_empty() { String::new() } else { _up_to_a_break(&whole, &step, *HAND_OVER_WORDS) };
        if text::split(&piece).len() > text::split(&step).len() {
            return Some(_say_with(&ONE_TURN_PLAIN_LINES, memory, &flat, &[("step", &piece)]));
        }
        let pool: &Vec<String> =
            if step.contains(": ") || FIRST_ITSELF.search(&step).is_some() { &ONE_TURN_PLAIN_LINES } else { &ONE_TURN_LINES };
        return Some(_say_with(pool, memory, &flat, &[("step", &step)]));
    }
    if how_to_asked {
        let whole = _step_sentence(&said, &step);
        if !whole.is_empty() && text::split(&whole).len() > text::split(&step).len() {
            step = whole;
        }
        let (head, colon, tail) = text::partition(&step, ": ");
        let (head, colon, tail) = (head.to_string(), colon.to_string(), tail.to_string());
        if !colon.is_empty()
            && !tail.is_empty()
            && !IMPERATIVES.contains(&_doing_word(&ENGLISH_WORDS.findall_str(&head)))
            && _the_front_names_what_is_in_hand(&head, &flat, memory)
        {
            step = text::strip(&tail).to_string();
        }
    }
    if FIRST_ITSELF.search(&step).is_some() {
        return Some(_say_with(&STEP_ALREADY_FIRST_LINES, memory, &flat, &[("step", &step)]));
    }
    if step.contains(": ") || text::split(&step).len() >= *STEP_LONG {
        return Some(_say_with(&STEP_DETAIL_LINES, memory, &flat, &[("step", &step)]));
    }
    if _action_count(&said) > 1 {
        let mut pool: &Vec<String> =
            if FIRST_ITSELF.search(&step).is_some() { &STEP_ALREADY_FIRST_LINES } else { &STEP_LINES };
        if how_to_asked {
            pool = &STEP_DETAIL_LINES;
        }
        return Some(_say_with(pool, memory, &flat, &[("step", &step)]));
    }
    Some(_say_with(&STEP_ONE_LINES, memory, &flat, &[("step", &step)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    // Expected values from PyBot (bot.compose.verdict); positions are Python's character indices.
    #[test]
    fn readers_like_python() {
        assert_eq!(_round_robin(&s(&["x", "y", "z", "w"]), "is that bad"), s(&["y", "z", "w", "x"]));
        assert_eq!(_the_person_for(Some("gp"), Some("the same day")), "your GP the same day");
        assert_eq!(_the_person_for(Some("111"), None), "111");
        assert_eq!(_the_person_for(Some("physio"), Some(" this  week ,")), "your physio this week");
        assert_eq!(_the_person_for(Some("dentist"), Some("")), "your dentist");
        assert_eq!(_the_person_for(Some("A&E"), None), "A&E");
        assert_eq!(
            _stated_before_the_judgement("Blood in it, breathlessness or three weeks of it is a reason to ring the surgery or 111."),
            "blood in it, breathlessness or three weeks of it"
        );
        assert!(!_said_without_a_not("it is unusually common", 9, 7));
        assert!(!_said_without_a_not("it is not usual", 10, 5));
        assert!(_said_without_a_not("usual", 0, 5));
        assert_eq!(_first_positive("It is common at four, but less usual past ten", &s(&["usual", "common"])), Some(6));
        assert_eq!(_unconditional_care("See a GP if the arm is weak or numb. Ring 111 today."), Some(37));
        assert_eq!(_danger("It is not dangerous. If it bleeds a lot, that is an emergency."), Some(49));
        assert_eq!(_danger("That could be dangerous - stop using it."), Some(14));
        assert_eq!(
            _up_to_a_break(
                "Fill a freezer bag with white vinegar - half vinegar, half water if the chrome worries you - tie it over the shower head so the nozzles soak for an hour, then rub them with a sponge and poke any blocked holes with a pin",
                "Fill a freezer bag with white vinegar",
                12
            ),
            "Fill a freezer bag with white vinegar - half vinegar"
        );
        // Python 3.12's re.escape leaves "," and ":" alone.
        assert_eq!(_own_pattern("Still, {Step}: first."), "Still,\\ .+?:\\ first\\.");
    }
    #[test]
    fn inline_patterns_compile() {
        SENTENCES.check().unwrap();
        SENTENCES_WS.check().unwrap();
        SENTENCES_SPACES.check().unwrap();
        CLAUSE_BREAKS.check().unwrap();
        HEAD_BREAK.check().unwrap();
        A_PAREN.check().unwrap();
        LETTERS.check().unwrap();
        LETTERS_4.check().unwrap();
        WS.check().unwrap();
        BODY_LETTERS.check().unwrap();
        MONEY_LETTERS.check().unwrap();
        EXPLANATION_HEAD.check().unwrap();
        A_GAP.check().unwrap();
        WORDS_09.check().unwrap();
        I_OPEN.check().unwrap();
        RELATIVE_MARKER.check().unwrap();
        A_FENCE_WORD.check().unwrap();
        A_SENTENCE_END.check().unwrap();
        AGO.check().unwrap();
        STEP_PIECES.check().unwrap();
        HOW_MUCH.check().unwrap();
        AND_OR_THEN.check().unwrap();
        IF_OPENS.check().unwrap();
        AFTER_IF_OPENS.check().unwrap();
        AFTER_IF_INSIDE.check().unwrap();
        A_JUDGEMENT_LIST.check().unwrap();
        HANGING_JOIN.check().unwrap();
        OR_AND_IF.check().unwrap();
        AN_ASIDE.check().unwrap();
        IF_UNLESS.check().unwrap();
        CLAUSE_SPLIT.check().unwrap();
        CLAUSE_SPLIT_COLON.check().unwrap();
        A_LINK_OR_ASIDE.check().unwrap();
        A_RANGE_AND.check().unwrap();
        THEN_CUT.check().unwrap();
        ACTION_SPLIT.check().unwrap();
        LIST_ITEMS.check().unwrap();
        ENGLISH_WORDS.check().unwrap();
        LOWER_WORDS.check().unwrap();
        A_WORD_OF_LETTERS.check().unwrap();
        A_BREAK.check().unwrap();
        FENCE_IN_FRONT.check().unwrap();
        A_STOP_IN_FRONT.check().unwrap();
        _ACK_PATTERN.check().unwrap();
    }

}
