//! Port of `bot/memory/repeat.py`: they ask for what to do next, and the bot has just said it.
//!
//! The commonest follow-up: the bot answers "my washing machine is leaking" with the steps to
//! take, and the next line is "what do I do?" or "and now?". This finds that turn - a question
//! with no thing of its own, the answer the bot gave a turn or two ago, and nothing said in
//! between that moved the subject on - and the reply is that answer again in short form (or the
//! part of it the line asks for), not a hedge.
//!
//! CONTRACT notes: several generated signatures cannot carry the Python value (a function that
//! returns a string is typed `()`, the generator `_the_steps` is typed `Option<String>`, the
//! (back, question, answer) triples of `_recent_answers` are typed `Vec<String>`). Each keeps its
//! contract signature and has a second function that carries the value: `answer_for_text`,
//! `the_steps`, `recent_answers`, and so on.
#![allow(non_snake_case)]

use crate::data::{self, Value};
use crate::memory::narrow::{self, py_format, py_str, py_str_or_empty, py_try, re_cached};
use crate::memory::topics;
use crate::py::re::{self as pyre, Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understanding::pyuni;
use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, OnceLock};

macro_rules! data_re {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.memory.repeat.", stringify!($name)))); )*
    };
}
macro_rules! data_set {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.memory.repeat.", stringify!($name)))); )*
    };
}
macro_rules! data_strs {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.memory.repeat.", stringify!($name)))); )*
    };
}
macro_rules! data_int {
    ($($name:ident),* $(,)?) => {
        $( static $name: LazyLock<usize> = LazyLock::new(|| {
            data::get(concat!("bot.memory.repeat.", stringify!($name))).as_int().expect("an int") as usize
        }); )*
    };
}
macro_rules! inline_re {
    ($name:ident, $pattern:expr, $flags:expr) => {
        static $name: LazyLock<Re> = LazyLock::new(|| Re::new($pattern, $flags));
    };
}

data_re!(
    QUESTION, RESET_LEAD, AT_A_LOSS, HEDGE, BARE_ASK, NEXT_STEP_EMERGENCY, NEXT_STEP, MOVE_ASK, PUT_RIGHT,
    PUT_RIGHT_ASK, AND_NOW, USE_WITH, NEXT_ASK, STEP_SAYS_FIRST, LEAD_SAYS_FIRST, AGREES_IN_A_PHRASE,
    CHANGE_ASK, QUALIFIER, PIECE_END, NOT_A_STEP, A_MENU, SENTENCE, ASKS, ASKING_THEM, REFER_BACK,
    ABOUT_THE_BOT, WORDS_WANTED, STALL, JUST_A_NOD, HELP_SHAPE, HELP_ASK_QUESTION_FIRST, NAMED_AFTER_IT,
    CONDITION_CLAUSE, NOT_A_MOVE, ORDER, FIRST_THING, _WEIGHS_ONE_AGAINST, _SUBJECT_OPEN, _NEGATES,
    _SO_COMES_THE_STEP, _SO_THE_STEP_FOLLOWS, _SO_DOES_NOT_ASK_SOMETHING_OF_THEM, _A_JUDGEMENT, LESS,
    OWN_LEAD, KEEP_CLAUSE, FIGURES, DIGIT, HOW_TO, SENTENCE_END, CLAUSE_SPLIT, A_LENGTH_ASK, A_BARE_TIME,
    A_PRICE_ASK, A_COUNT_OF_MORE_THAN_ONE, TROUBLE, OWNED, SOMEONE_ELSES_LIMIT, ACK_OPENS, ACK_AND_REST,
    BROODING, TRIED, NOT_TRIED, QUESTION_MORE, DONE, REJOINDER, ASKING_TO_START, START_ASK, STARTING_WHAT,
    STARTING_OUT, MORE_ASK, IDEAS_ONLY, WORDING, HOW_IT, LIST_LEAD, NOT_A_TAIL, NOT_A_DOING, A_QUESTION,
    DECLARATIVE, QUESTIONS_THAT_ARE_NOT_STEPS, WHAT_IT_IS, ATTEMPT, STILL_ASKS, GAVE_ASKS, A_FIGURE,
    A_PAY_WORD, A_PAY_VERB_AT_THE_END, A_TAKE_HOME, A_IM_ON_A_FIGURE, A_PAY_JUDGEMENT, A_GOAL_LINE,
    A_RATE_LINE, A_WERE_ASKS, A_RATE_OF_TIME, A_SCHEDULED_AFTER, A_HAS_THE_LENGTH, NOT_A_FIGURE_ASK,
    A_PLAN_ASK, A_THE_PLAN_ASK, A_THING_NAMED_AFTER_THE_CUE, A_BARE_AMOUNT_ASK, A_BARE_DURATION_ASK,
    A_HOW_LONG_ITS_BEEN, _OWNS_THE_THING, A_LINE_THAT_ASKS, A_CORRECTION, A_MARKER_AT_THE_END,
    A_NUMBER_WORD, A_ONE_MORE, A_GAIN, A_GROUP_WORD, A_LOSS, A_LIST_OF_AGES, COUNT_ASK,
    A_RATE_IN_THE_FIGURE, _MONEY_PAID_FOR, _AN_AMOUNT_WANTED, A_TOLD_BACK, A_NARROWING_ONLY,
    A_FIGURE_WORD, A_POINTS_BACK_HARD, CHANGED_IT, NOT_ALONE,
);

data_set!(
    ASK, ASK_VERB, WHOLE_OF_IT, FILLER, TIME, SELF, LOSS_WORDS, BARE_ASK_NEEDS_A_QUESTION, PUT_RIGHT_WORDS,
    BE_FORM, POINTING_AT, AGREEING, AGREEMENT_TAIL, AGREES_ON_ITS_OWN, NEEDS_COMPLEMENT, LEADING_PHRASE,
    AN_ARTICLE, OPENERS, DANGLING, ECHO_FRAME, NOT_A_THING, STEP_VERBS, _AFTER_A_NOUN, _HANGS, LESS_WORDS,
    BREVITY_WORDS, CUT_MID_SENTENCE, POINTING_BACK, PRICE_WORDS, CLOSES_THE_TURN, NO_LONGER_TO_DO,
    STANDS_IN, OBJECT_END, STARTING_BACK, STARTING_AT, STARTING_INTO, FIGURE_WORDS, FIGURE_SAYING,
    PART_STAND_INS, A_MOVE_WORDS, NOT_A_METHOD, A_QUALITY, NOT_A_SUBJECT, ASK_FRAME, PAY_FAMILY,
    PAY_WHEN_WORDS, A_MEASURE_WORD, A_DOING_WORD, FIGURE_NOUNS, GOAL_NOUNS, FIGURE_TAIL, PERIOD_WORD,
    A_PLURAL_ONLY_NOUN, NOT_A_THING_ASKED,
);

data_strs!(
    ANSWER_MODULES, LEADS, SAY_AGAIN_LEADS, TRIED_LEADS, PUT_RIGHT_LEADS, NEXT_LEADS, ALSO_LEADS, MOVE_LEADS,
    NOW_LEADS, LOST_LEADS, CHANGE_LEADS, MORE_LEADS, MOVE_VERBS, CLAIMS_IT_WAS_SAID, LESS_LEADS, KEEPING,
    ALREADY_SHORT, NO_SHORTER_STILL, NOTHING_TO_CUT, JUST_THIS,
);

data_int!(
    A_STORE_ANSWER, STEP_WORDS, STEP_WITH_ITS_METHOD, CUT_WORDS, SHORT_STEP, LEADING_PHRASE_MAX, MOST,
    LEAD_IN, LOOK_BACK, LOOK_BACK_FOR_LESS, _EXPLANATION_WORDS, OWN_LEAD_WORDS, SHORT_ENOUGH, LENGTH_BACK,
    GAVE_LOOK_BACK, A_NARROWING_WORDS,
);

static WELCOMES: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.repeat.WELCOMES"));
static WHERE_TO_LOOK_DEFAULT: LazyLock<String> = LazyLock::new(|| data::string("bot.memory.repeat.WHERE_TO_LOOK_DEFAULT"));

/// `DOINGS`: the verb families a line that names nothing can be written with.
static DOINGS: LazyLock<Vec<HashSet<String>>> =
    LazyLock::new(|| data::get("bot.memory.repeat.DOINGS").items().iter().map(|family| family.str_set()).collect());
/// `QUOTE_PAIRS`: (opening mark, closing mark).
static QUOTE_PAIRS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    data::get("bot.memory.repeat.QUOTE_PAIRS")
        .items()
        .iter()
        .map(|pair| (pair.items()[0].str().to_string(), pair.items()[1].str().to_string()))
        .collect()
});
/// `FIGURE_ASKS`: (name, the ask, the figure that answers it).
static FIGURE_ASKS: LazyLock<Vec<(String, &'static Re, &'static Re)>> = LazyLock::new(|| {
    data::get("bot.memory.repeat.FIGURE_ASKS")
        .items()
        .iter()
        .map(|row| {
            let row = row.items();
            (row[0].str().to_string(), row[1].as_re().unwrap(), row[2].as_re().unwrap())
        })
        .collect()
});
/// `PART_ASKS`: (name, the ask, the mark the card's sentence has, or None).
static PART_ASKS: LazyLock<Vec<(String, &'static Re, Option<&'static Re>)>> = LazyLock::new(|| {
    data::get("bot.memory.repeat.PART_ASKS")
        .items()
        .iter()
        .map(|row| {
            let row = row.items();
            (row[0].str().to_string(), row[1].as_re().unwrap(), row[2].as_re())
        })
        .collect()
});
static WHERE_TO_LOOK: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.repeat.WHERE_TO_LOOK"));
static WHERE_TO_LOOK_WORDS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| {
    data::get("bot.memory.repeat.WHERE_TO_LOOK_WORDS")
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
});
static WHETHER_ASKS: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| data::get("bot.memory.repeat.WHETHER_ASKS").items().iter().map(|r| r.as_re().unwrap()).collect());
static METHOD_ASKS: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| data::get("bot.memory.repeat.METHOD_ASKS").items().iter().map(|r| r.as_re().unwrap()).collect());
static IRREGULAR_PAST: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.repeat.IRREGULAR_PAST"));
static THE_SAME_NUMBER: LazyLock<HashMap<String, String>> = LazyLock::new(|| data::str_map("bot.memory.repeat.THE_SAME_NUMBER"));

// Data of other modules this one reads.
static STOPWORDS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.topics.STOPWORDS"));
static RED_FLAG: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.topics.RED_FLAG"));
static REPEAT: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.memory.topics.REPEAT"));
static IMPERATIVES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.verdict.IMPERATIVES"));
static NEGATIVE_CUES: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.compose.fallback.NEGATIVE_CUES"));
static FREE_TIME: LazyLock<Vec<String>> = LazyLock::new(|| data::strs("bot.knowledge.plans.FREE_TIME"));
static COUNTS_THEIRS: LazyLock<HashSet<String>> = LazyLock::new(|| data::set("bot.memory.counts.THEIRS"));
/// `IMPERATIVES | MOVE_VERBS`: every verb a step can open on.
static VERBS: LazyLock<HashSet<String>> = LazyLock::new(|| IMPERATIVES.iter().chain(MOVE_VERBS.iter()).cloned().collect());

inline_re!(RE_SPACED_DASH_LEAD, r"\s+[-—]\s+", 0);
inline_re!(RE_STARTS_WITH, r"(?:start|begin) with\b", pyre::I);
inline_re!(RE_RATHER_THAN, r"\s+rather than\b", 0);
inline_re!(RE_EXPLAIN_HEAD, r":|,|;| - | — ", 0);
inline_re!(RE_CLAUSES, r"(?<!\d),(?!\d)| - |;| — |\. ", 0);
inline_re!(RE_DASH_OR_BRACKET, r"\s+[-—]\s+|\s+\(", 0);
inline_re!(RE_WORDS, r"[a-z']+", 0);
inline_re!(RE_PART_LEAD, r"^(?:and\s+|but\s+|so\s+|then\s+|after\s+that,?\s+|next,?\s+)", pyre::I);
inline_re!(RE_THEN_LEAD, r"^then\s+", pyre::I);
inline_re!(RE_COMMA_SPACES, r",\s+", 0);
inline_re!(RE_DO_NOT, r"\bdo not\b|\bdon'?t\b", pyre::I);
inline_re!(RE_QUOTE_END, "[.!?][\"\u{201d}]?", 0);
inline_re!(RE_TRAILING_DOTS, r"\.{2,}$", 0);
inline_re!(RE_DOTS_SPACE, r"\.\.\s+", 0);
inline_re!(RE_NAME_LEAD, r"^(?:mate|love|pet|hun|hunny|duck|sunshine|chief|fella|sweetheart)\b[,!.]*\s*", pyre::I);
inline_re!(RE_FOR_LEAD, r"for\b", pyre::I);
inline_re!(RE_CONNECTOR_LEAD, r"^(?:and|then|but|also|so|next|plus)\s+", pyre::I);
inline_re!(RE_FIRST_OR_NEXT_END, r"\s+(?:first|next)$", pyre::I);
inline_re!(RE_PLACEHOLDER, r"\{[a-z_]+\}", 0);
inline_re!(RE_HOW_A_FIGURE, r"\bhow (?:long|much|many|often|far|hot|soon)\b", pyre::I);
inline_re!(RE_WERE_OPENS, r"\A\s*were\s+\w", pyre::I);
inline_re!(RE_WERE_LEAD, r"\A(\s*)were\b", pyre::I);
inline_re!(RE_DIGITS_LEAD, r"\s*([\d][\d,]*(?:\.\d+)?)", 0);
inline_re!(RE_FIGURE_COMMA, r"(?<!\d),|,(?!\d)", 0);
inline_re!(RE_NUMBERS, r"\d[\d,]*", 0);
inline_re!(RE_NOT_ALNUM, r"[^a-z0-9]+", 0);

// ---------------------------------------------------------------------------------------------
// Small Python-shaped helpers.

/// `len(s.split())`.
fn n_words(s: &str) -> usize {
    text::split_ref(s).len()
}

/// `s[0].lower() + s[1:]` (raises IndexError on "", as Python does).
fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    let first = chars.next().expect("IndexError: string index out of range");
    format!("{}{}", first.to_lowercase(), chars.as_str())
}

/// `s[:1].upper() + s[1:]`.
fn upper_first(s: &str) -> String {
    text::upper_first(s)
}

/// `s[:n]` in characters.
fn first_chars(s: &str, n: usize) -> &str {
    &s[..text::byte_at(s, n)]
}

/// `s.strip()[:60].lower()`.
fn opening60(s: &str) -> String {
    text::lower(first_chars(text::strip(s), 60))
}

/// `set(topics.words(text))`.
fn word_set(text_: &str) -> HashSet<String> {
    topics::words(text_).into_iter().collect()
}

/// `topics.keywords(text)` as a set.
fn keyword_set(text_: &str) -> HashSet<String> {
    topics::keywords(text_).into_iter().collect()
}

/// `re.findall(r"[a-z']+", text)` (text already lower-cased by the caller where Python does).
fn az_words(text_: &str) -> Vec<String> {
    RE_WORDS.findall_str(text_)
}

/// `word.isdigit()`.
fn is_digit(word: &str) -> bool {
    pyuni::is_digit(word)
}

/// The turns before this one: Python's `[t for t in memory.get("turns") or [] if isinstance(t, dict)]`.
fn turns(memory: &Memory) -> &[Turn] {
    &memory.turns
}

/// `turns[-n:]`.
fn last_turns(memory: &Memory, n: usize) -> &[Turn] {
    let all = turns(memory);
    &all[all.len().saturating_sub(n)..]
}

/// `turns[len(turns) - back + 1:]` (with Python's negative-index wrap when `back` passes the end).
fn turns_since(memory: &Memory, back: usize) -> &[Turn] {
    let all = turns(memory);
    let from = all.len() as isize - back as isize + 1;
    let from = if from < 0 { (all.len() as isize + from).max(0) as usize } else { (from as usize).min(all.len()) };
    &all[from..]
}

fn module_of(turn: &Turn) -> &str {
    turn.module.as_deref().unwrap_or("")
}

/// `memory.get("repeated") or []`: the lines handed back so far.
fn repeated_lines(memory: &Memory) -> Vec<String> {
    match memory.get("repeated") {
        Some(v) if v.truthy() => v.items().iter().map(|line| py_str_or_empty(Some(line))).collect(),
        _ => Vec::new(),
    }
}

/// `pick(options, memory, seed_text)` from the composer.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, memory, seed_text)
}

/// `limits.names_a_limit(clean)`: whether the line names something a person cannot do.
fn names_a_limit(text_: &str) -> bool {
    crate::memory::limits::names_a_limit(text_).is_some()
}

/// CONTRACT: `topics::said_again` is typed `()`; Python returns the framed repeat or None. Read
/// as None until the contract carries the value.
fn topics_said_again(said: &str, memory: &Memory) -> Option<String> {
    topics::said_again(said, memory, "");
    None
}

/// CONTRACT: `counts::answer` is typed `()`; Python returns the store's answer or None. Read as
/// None until the contract carries the value.
fn counts_answer(text_: &str, memory: &Memory) -> Option<String> {
    crate::memory::counts::answer(text_, memory);
    None
}

// ---------------------------------------------------------------------------------------------

/// Python: bot/memory/repeat.py:84 `_doings(text)`
///
/// The verb families a line asks its work with.
pub fn _doings(text_: &str) -> HashSet<usize> {
    let words = word_set(text_);
    DOINGS.iter().enumerate().filter(|(_, family)| family.iter().any(|w| words.contains(w))).map(|(i, _)| i).collect()
}

/// Python: bot/memory/repeat.py:90 `asks_for_the_same_work(text, memory)`
///
/// True when a line that names nothing asks for the answer the question before it got.
pub fn asks_for_the_same_work(text_: &str, memory: &Memory) -> bool {
    if !names_nothing(text_) {
        return false;
    }
    if crate::memory::history::about_the_chat(text_) {
        // A line that asks what was said before is never a request for the answer again.
        return false;
    }
    let asked = _doings(text_);
    let words = word_set(text_);
    if asked.is_empty() && !words.iter().any(|w| WHOLE_OF_IT.contains(w)) {
        return false;
    }
    let whole = words.iter().any(|w| WHOLE_OF_IT.contains(w));
    for (back, question, said) in recent_answers(memory) {
        if back > 1 && !_about_the_same_thing(&question, &said, memory) {
            continue;
        }
        return whole || asked.intersection(&_doings(&question)).next().is_some();
    }
    false
}

/// Python: bot/memory/repeat.py:153 `without_a_reset(text)`
///
/// The line with a "forget that" lead taken off, so the new ask is read as the whole turn.
pub fn without_a_reset(text_: &str) -> String {
    RESET_LEAD.subn("", text::strip(text_), 1)
}

/// Python: bot/memory/repeat.py:492 `_the_first_instruction_in_a_long_piece(head, verbs)`
///
/// The first instruction out of a piece too long to hand over whole, else None.
pub fn _the_first_instruction_in_a_long_piece(head: &str, verbs: &HashSet<String>) -> Option<String> {
    let first = head.split(',').next().unwrap_or("");
    let piece = text::strip_chars(first, " ,;:.");
    let words = text::split_ref(piece);
    if words.len() < *SHORT_STEP {
        return None;
    }
    if verbs.contains(&text::lower(words[0])) || _a_verb_after_a_leading_phrase(piece, verbs) {
        return Some(piece.to_string());
    }
    None
}

/// Python: bot/memory/repeat.py:509 `_a_verb_after_a_leading_phrase(head, verbs)`
///
/// True when the instruction's verb sits just after a phrase like "each week" or "half".
pub fn _a_verb_after_a_leading_phrase(head: &str, verbs: &HashSet<String>) -> bool {
    let tokens: Vec<String> = text::split_ref(head).iter().map(|w| text::lower(text::strip_chars(w, " ,-"))).collect();
    let top = (*LEADING_PHRASE_MAX + 1).min(tokens.len());
    for at in 1..top {
        if !verbs.contains(&tokens[at]) {
            continue;
        }
        let phrase = &tokens[..at];
        if AN_ARTICLE.contains(&phrase[0]) {
            continue;
        }
        if phrase.iter().all(|w| LEADING_PHRASE.contains(w) || AN_ARTICLE.contains(w)) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/repeat.py:631 `only_said_it_back(line, said)`
///
/// Whether the reply only says the person's own line back, so nothing has been answered.
pub fn only_said_it_back(line: &str, said: Option<&str>) -> bool {
    let said = text::strip(said.unwrap_or(""));
    if said.is_empty() {
        return false;
    }
    let low = text::lower(said);
    let spoken: Vec<String> = topics::words(line)
        .into_iter()
        .filter(|w| !STOPWORDS.contains(w) && !ECHO_FRAME.contains(w) && text::len(w) > 1)
        .collect();
    if spoken.is_empty() {
        return false;
    }
    let kept = spoken.iter().filter(|w| low.contains(w.as_str())).count();
    spoken.len() >= 2 && kept >= 2.max((spoken.len() * 3) / 4)
}

/// Python: bot/memory/repeat.py:655 `_an_echo(turn)`
///
/// The bot only said the person's own line back, or stalled, so nothing has been answered.
pub fn _an_echo(turn: &Turn) -> bool {
    let said = text::strip(&turn.bot);
    if only_said_it_back(&turn.user, Some(said)) {
        return true;
    }
    !said.is_empty() && STALL.is_match(&text::lower(said))
}

/// Python: bot/memory/repeat.py:663 `_names_no_topic(turn)`
///
/// The line was a detail, not a subject of its own.
pub fn _names_no_topic(turn: &Turn) -> bool {
    let line = turn.user.as_str();
    if text::strip(line).is_empty() {
        return false;
    }
    py_try(|| crate::understanding::understand(line, None).topics.is_empty()).unwrap_or(false)
}

/// Python: bot/memory/repeat.py:675 `_only_echoes_between(memory, back)`
///
/// Whether every turn after an answer `back` turns ago was a detail said back, and no more.
pub fn _only_echoes_between(memory: &Memory, back: usize) -> bool {
    let between: &[Turn] = if back > 1 { turns_since(memory, back) } else { &[] };
    !between.is_empty() && between.iter().all(|turn| _an_echo(turn) && _names_no_topic(turn))
}

/// Python: bot/memory/repeat.py:721 `_a_help_ask(text)`
///
/// True when the whole line asks for a way to help and names nothing of its own.
pub fn _a_help_ask(text_: &str) -> bool {
    if !HELP_SHAPE.is_match(text_) {
        return false;
    }
    if HELP_ASK_QUESTION_FIRST.is_match(text_) && !NAMED_AFTER_IT.is_match(text_) {
        return true;
    }
    if a_more_ask(text_) {
        // one more thing out of the answer already given is `MORE_ASK`'s turn
        return false;
    }
    crate::understanding::understand(text_, None).help_ask
}

/// Python: bot/memory/repeat.py:751 `asks_for_the_move(text)`
///
/// True when the line asks which part of the answer in hand to do.
pub fn asks_for_the_move(text_: &str) -> bool {
    says_where_to_start(text_) || HOW_IT.is_match(text_) || _a_help_ask(text_)
}

/// Python: bot/memory/repeat.py:762 `names_nothing(text)`
///
/// True when the line asks for something without naming anything of its own.
pub fn names_nothing(text_: &str) -> bool {
    if JUST_A_NOD.match_(text_).is_some() {
        return false;
    }
    let lost = AT_A_LOSS.is_match(text_);
    let help_ask = _a_help_ask(text_);
    if !(QUESTION.match_(text::strip(text_)).is_some() || lost || help_ask) {
        return false;
    }
    if ABOUT_THE_BOT.is_match(text_) {
        return false;
    }
    if WORDS_WANTED.is_match(text_) {
        return false;
    }
    if topics::pairing(text_) {
        return false;
    }
    if help_ask {
        return true;
    }
    !topics::words(text_).is_empty() && _names_a_thing_of_its_own(text_).is_empty()
}

/// Python: bot/memory/repeat.py:793 `_names_a_thing_of_its_own(text)`
///
/// The content words of the line that name something, in the ask's own frame left out.
pub fn _names_a_thing_of_its_own(text_: &str) -> Vec<String> {
    let lost = AT_A_LOSS.is_match(text_);
    let said = |w: &str| {
        ASK.contains(w)
            || ASK_VERB.contains(w)
            || TIME.contains(w)
            || FILLER.contains(w)
            || NOT_A_THING.contains(w)
            || (lost && LOSS_WORDS.contains(w))
    };
    topics::words(text_)
        .into_iter()
        .filter(|w| text::len(w) > 2 && !said(w) && !SELF.contains(w) && !is_digit(w) && !STOPWORDS.contains(w))
        .collect()
}

/// Python: bot/memory/repeat.py:830 `asks_to_put_it_right(text)`
///
/// True when the line asks whether the thing on the table can still be put right.
pub fn asks_to_put_it_right(text_: &str) -> bool {
    let line = text::strip(text_);
    if !PUT_RIGHT.is_match(line) || PUT_RIGHT_ASK.match_(line).is_none() {
        return false;
    }
    if topics::pairing(line) || ABOUT_THE_BOT.is_match(line) {
        return false;
    }
    topics::words(line).iter().all(|w| {
        ASK.contains(w) || FILLER.contains(w) || is_digit(w) || SELF.contains(w) || PUT_RIGHT_WORDS.contains(w) || STOPWORDS.contains(w)
    })
}

/// Python: bot/memory/repeat.py:849 `a_step_of_the_same_answer(text)`
///
/// True when the line asks which of the answer's own steps to do first.
pub fn a_step_of_the_same_answer(text_: &str) -> bool {
    if !ORDER.is_match(text_) || !MOVE_ASK.is_match(text_) {
        return false;
    }
    if topics::pairing(text_) || ABOUT_THE_BOT.is_match(text_) {
        return false;
    }
    let words: Vec<String> = topics::words(text_)
        .into_iter()
        .filter(|w| !ASK.contains(w) && !FILLER.contains(w) && !is_digit(w) && !STOPWORDS.contains(w))
        .collect();
    !words.is_empty() && words.iter().all(|w| STEP_VERBS.contains(w))
}

/// Python: bot/memory/repeat.py:866 `_recent_answers(memory)`
///
/// CONTRACT: typed `Vec<String>`; Python gives (back, question, answer) triples, which is
/// [`recent_answers`]. This keeps the contract and gives the answers, newest first.
pub fn _recent_answers(memory: &Memory) -> Vec<String> {
    recent_answers(memory).into_iter().map(|(_, _, said)| said).collect()
}

/// The bot's own answers in the recent past, newest first, each with the line it answered and
/// how many turns ago it was (1 is the turn just before this one). (`repeat.recent_answers`,
/// Python's own alias of `_recent_answers`.)
pub fn recent_answers(memory: &Memory) -> Vec<(usize, String, String)> {
    let mut out = Vec::new();
    for (back, turn) in last_turns(memory, *LOOK_BACK).iter().rev().enumerate() {
        let back = back + 1;
        // Only an answer is worth saying again.
        if !ANSWER_MODULES.iter().any(|m| turn.module.as_deref() == Some(m.as_str())) {
            continue;
        }
        let said = text::strip(&turn.bot);
        if said.is_empty() || HEDGE.is_match(said) || text::len(said) < 50 {
            continue;
        }
        // The store's own short lines are not answers; its entry answers are.
        if text::len(said) < *A_STORE_ANSWER && module_of(turn) != "answers" {
            continue;
        }
        // The bot asking a question is not an answer to hand back.
        if text::rstrip(said).ends_with('?') || QUESTION.match_(said).is_some() {
            continue;
        }
        // A trip they have changed moves the answer with it.
        if topics::about_a_place_they_left_behind(said, memory) {
            continue;
        }
        out.push((back, turn.user.clone(), said.to_string()));
    }
    out
}

/// Python: bot/memory/repeat.py:903 `_the_answer_about_what_we_are_on(memory, recent)`
///
/// The recent answers that can be handed back, newest first, with the topic in hand first.
pub fn _the_answer_about_what_we_are_on(memory: &Memory, recent: Vec<(usize, String, String)>) -> Vec<(usize, String, String)> {
    let here = topics::current(memory);
    let mut mine: HashSet<String> =
        topics::keywords(&py_str_or_empty(here.and_then(|h| h.get("text")))).into_iter().collect();
    if let Some(words) = here.and_then(|h| h.get("words")) {
        if words.truthy() {
            mine.extend(words.items().iter().map(|w| text::lower(&py_str(w))));
        }
    }
    if recent.is_empty() || mine.is_empty() {
        return recent;
    }
    let touches = |s: &str| keyword_set(s).iter().any(|w| mine.contains(w));
    if touches(&recent[0].1) || touches(&recent[0].2) {
        return recent;
    }
    let about_it: Vec<(usize, String, String)> = recent.iter().filter(|item| touches(&item.1)).cloned().collect();
    if about_it.is_empty() { recent } else { about_it }
}

/// Python: bot/memory/repeat.py:949 `_nothing_but_acknowledgements(back, memory)`
///
/// Nothing has answered since that answer: only acknowledgements of the person's own line.
pub fn _nothing_but_acknowledgements(back: usize, memory: &Memory) -> bool {
    let all = turns(memory);
    if back <= 1 || back > all.len() {
        return true;
    }
    for turn in turns_since(memory, back) {
        if module_of(turn) == "answers" {
            return false;
        }
        if n_words(&turn.user) > 6 && !ATTEMPT.is_match(&turn.user) {
            return false;
        }
    }
    true
}

/// Python: bot/memory/repeat.py:969 `_about_the_same_thing(asked, said, memory)`
///
/// The answer belongs to what the chat is still on, not to a subject that has moved.
pub fn _about_the_same_thing(asked: &str, said: &str, memory: &Memory) -> bool {
    let all = turns(memory);
    // A line the bot had no answer to does not move the chat on.
    if let Some(last) = all.last() {
        if HEDGE.is_match(&last.bot) {
            return true;
        }
    }
    let subject: Vec<String> = topics::words(&topics::subject(memory)).into_iter().filter(|w| !STOPWORDS.contains(w)).collect();
    if subject.is_empty() {
        return true;
    }
    // An answer that does not use one word of the line it answered is not about that line.
    let content = |s: &str| -> HashSet<String> {
        topics::words(s).into_iter().filter(|w| text::len(w) > 2 && !STOPWORDS.contains(w)).collect()
    };
    let theirs = content(asked);
    let mut mine = content(said);
    mine.extend(_the_keys_of(said, memory).into_iter().filter(|w| text::len(w) > 2));
    if !theirs.is_empty() && mine.intersection(&theirs).next().is_none() {
        return false;
    }
    let mut broken = word_set(asked);
    broken.extend(word_set(said));
    if subject.iter().any(|w| broken.contains(w)) {
        return true;
    }
    // A line that reports having a go at the answer is a line about that answer.
    let mine = content(said);
    for turn in last_turns(memory, 3) {
        let line = &turn.user;
        if !ATTEMPT.is_match(line) {
            continue;
        }
        let theirs: HashSet<String> = topics::words(line).into_iter().filter(|w| text::len(w) > 2).collect();
        if mine.intersection(&theirs).count() >= 2 {
            return true;
        }
    }
    false
}

/// Python: bot/memory/repeat.py:1018 `_the_chat_is_still_on(said_before, memory)` - the first of
/// the two definitions in the file. Python's second definition (line 4603) replaces it when the
/// module loads, so every call goes to [`_the_chat_is_still_on`]; this one is kept for the record
/// and is never called, as in Python.
#[allow(dead_code)]
fn _the_chat_is_still_on_replaced(said_before: &str, memory: &Memory) -> bool {
    let topic = memory.get("topic");
    let now: HashSet<String> = topics::words(&py_str_or_empty(topic.and_then(|t| t.get("text"))))
        .into_iter()
        .filter(|w| text::len(w) > 2 && !STOPWORDS.contains(w))
        .collect();
    if now.is_empty() {
        return true;
    }
    let then: HashSet<String> =
        topics::words(said_before).into_iter().filter(|w| text::len(w) > 2 && !STOPWORDS.contains(w)).collect();
    then.intersection(&now).next().is_some()
}

/// Python: bot/memory/repeat.py:1038 `_short(said)`
///
/// The answer the way a friend repeats it: its first sentence, or two when the first is only a
/// lead-in, and never more than MOST characters.
pub fn _short(said: &str) -> String {
    let parts: Vec<String> =
        SENTENCE.split(text::strip(said)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return String::new();
    }
    let mut short = parts[0].clone();
    if text::len(&short) < *LEAD_IN && parts.len() > 1 {
        short = format!("{} {}", short, parts[1]);
    }
    if text::len(&short) > *MOST {
        let cut = first_chars(&short, *MOST);
        let head = match cut.rfind(' ') {
            Some(at) => &cut[..at],
            None => cut,
        };
        short = format!("{}...", text::rstrip_chars(head, ",;:"));
    }
    short
}

/// Python: bot/memory/repeat.py:1084 `_explains(sentence)`
///
/// True for a sentence that says how something is, so holds no step of its own.
pub fn _explains(sentence: &str) -> bool {
    let text_ = text::strip(sentence);
    if _SUBJECT_OPEN.match_(text_).is_none() {
        return false;
    }
    let head = match RE_EXPLAIN_HEAD.search(text_) {
        Some(m) => &text_[..m.start()],
        None => text_,
    };
    n_words(head) > *_EXPLANATION_WORDS
}

/// Python: bot/memory/repeat.py:1093 `_a_step_comes_after(part)`
///
/// Whether the sentence that explains a fault ends on the step ("..., so clear the vents").
pub fn _a_step_comes_after(part: &str) -> bool {
    _SO_THE_STEP_FOLLOWS.finditer(part).iter().any(|m| _opens_an_instruction(&part[m.end()..]))
}

/// Python: bot/memory/repeat.py:1108 `_opens_an_instruction(text)`
///
/// Whether a piece of an answer opens on a verb an instruction can open on.
pub fn _opens_an_instruction(text_: &str) -> bool {
    let words = text::split_ref(text::strip_chars(text_, " ,.:;"));
    match words.first() {
        Some(first) => IMPERATIVES.contains(text::strip_chars(&text::lower(first), " ,.;:")),
        None => false,
    }
}

/// Python: bot/memory/repeat.py:1145 `_opening_move(part)`
///
/// The instruction a sentence opens with, in the answer's own words, or None.
pub fn _opening_move(part: &str) -> Option<String> {
    let stripped = text::strip(part);
    // The piece before the first dash is the step when it is itself a short sentence.
    let lead = RE_SPACED_DASH_LEAD.split(stripped).into_iter().next().unwrap_or_default();
    if !text::strip(&lead).is_empty() && text::strip(&lead) != stripped && n_words(&lead) <= 12 {
        if let Some(found) = _opening_move(&lead) {
            return Some(found);
        }
    }
    let leads_with_the_plan = RE_STARTS_WITH.match_(stripped).is_some();
    let mut pieces: Vec<String> = vec![part.to_string()];
    pieces.extend(part.split(':').map(str::to_string));
    // A sentence that opens on a subject pronoun and runs on is explaining something.
    let explaining = _explains(part);
    for (index, piece) in pieces.iter().enumerate() {
        if index > 0 && explaining {
            continue;
        }
        let mut head = _SO_COMES_THE_STEP.sub("", text::strip_chars(piece, " ,.:;"));
        if RED_FLAG.match_(&head).is_some() {
            continue; // a warning is not a move
        }
        let words = text::split_ref(&head);
        if !(3..=40).contains(&words.len()) {
            continue;
        }
        let first = text::strip_chars(&text::lower(words[0]), " ,.;:").to_string();
        let rest = words[1..].join(" ");
        if first != text::lower(words[0]) && !_opens_an_instruction(&rest) && _NEGATES.match_(&rest).is_none() {
            // The comma comes off the verb only when the answer goes on with another instruction.
            continue;
        }
        if !IMPERATIVES.contains(&first) {
            // The steps after a "start with X:" lead open on a measure rather than a verb.
            if !(leads_with_the_plan && index > 1) {
                continue;
            }
        } else if _AFTER_A_NOUN.contains(text::strip_chars(&text::lower(words[1]), " ,.;:")) {
            continue; // "water at the front": a noun the imperative list happens to hold
        } else if RE_STARTS_WITH.match_(&head).is_some() && part.contains(':') {
            continue; // the lead itself; what it leads to is the next piece
        } else if _WEIGHS_ONE_AGAINST.is_match(&head) && index + 1 < pieces.len() && _opens_an_instruction(&pieces[index + 1]) {
            continue; // a heading that weighs one way against another
        }
        let cut = RE_RATHER_THAN.split(&head).into_iter().next().unwrap_or_default();
        head = text::strip_chars(&cut, " ,;:-").to_string();
        if n_words(&head) < 2 || RED_FLAG.match_(&head).is_some() {
            continue;
        }
        // The first step on its own when the sentence runs on with ", then ".
        if !head.contains(':') && head.contains(", then ") {
            let shorter = text::strip_chars(head.split(", then ").next().unwrap_or(""), " ,;:-").to_string();
            let shorter_words = text::split_ref(&shorter);
            if (3..=8).contains(&shorter_words.len())
                && _opens_an_instruction(&shorter)
                && !_HANGS.contains(&text::lower(shorter_words[shorter_words.len() - 1]))
            {
                head = shorter;
            }
        }
        return Some(lower_first(&head));
    }
    None
}

/// Python: bot/memory/repeat.py:1254 `_a_question(said)`
///
/// Whether the sentence asks rather than tells.
pub fn _a_question(said: &str) -> bool {
    let text_ = text::strip(said);
    if text_.ends_with('?') {
        return true;
    }
    QUESTIONS_THAT_ARE_NOT_STEPS.match_(text_).is_some()
}

/// Python: bot/memory/repeat.py:1269 `_a_move(said, questions_are_steps=False)`
///
/// CONTRACT: typed `()`; Python returns the move or None, which is [`a_move`].
pub fn _a_move(said: &str, questions_are_steps: bool) {
    let _ = a_move(said, questions_are_steps);
}

/// The move in the answer, when it is a move and not the answer with a lead on it
/// (`_a_move` with its value).
pub fn a_move(said: &str, questions_are_steps: bool) -> Option<String> {
    let found = _a_move_in(said, questions_are_steps);
    if let Some(found) = &found {
        if n_words(found) > *STEP_WITH_ITS_METHOD {
            return None;
        }
    }
    found
}

/// Python: bot/memory/repeat.py:1287 `_a_move_in(said, questions_are_steps=False)`
///
/// The sentence of the answer that tells them what to do, or None.
pub fn _a_move_in(said: &str, questions_are_steps: bool) -> Option<String> {
    let parts: Vec<String> =
        SENTENCE.split(text::strip(said)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    if let Some(first) = parts.first() {
        if let Some(opening) = _opening_move(first) {
            if !opening.is_empty() && (questions_are_steps || !_a_question(&opening)) {
                return Some(opening);
            }
        }
    }
    for part in parts.iter().rev() {
        // A red flag is not a move.
        if RED_FLAG.match_(part).is_some() {
            continue;
        }
        if let Some(named) = FIRST_THING.match_(part) {
            let found = text::strip_chars(named.name_str("move"), " ,;:");
            if (2..=20).contains(&n_words(found))
                && RED_FLAG.match_(found).is_none()
                && (questions_are_steps || !_a_question(found))
            {
                return Some(lower_first(found));
            }
        }
        if let Some(head) = _opening_move(part) {
            if !head.is_empty()
                && CONDITION_CLAUSE.match_(&head).is_none()
                && NOT_A_MOVE.match_(&head).is_none()
                && (questions_are_steps || !_a_question(&head))
            {
                return Some(head);
            }
        }
        if _explains(part) && !_a_step_comes_after(part) {
            continue;
        }
        let mut pieces: Vec<String> = vec![part.clone()];
        pieces.extend(part.split(':').map(str::to_string));
        'pieces: for piece in &pieces {
            let clauses = RE_CLAUSES.split(piece);
            let mut conditional = false;
            let mut in_a_bracket: i64 = 0;
            let mut chosen: Option<String> = None;
            for clause in &clauses {
                let mut head = text::strip_chars(clause, " .:;").to_string();
                if head.is_empty() {
                    continue;
                }
                let opened_inside = in_a_bracket > 0;
                in_a_bracket = 0.max(in_a_bracket + text::count(&head, "(") as i64 - text::count(&head, ")") as i64);
                if opened_inside {
                    continue;
                }
                let a_conclusion = _SO_COMES_THE_STEP.match_(&head).is_some();
                head = _SO_COMES_THE_STEP.sub("", &head);
                if head.is_empty() {
                    continue;
                }
                if text::count(&head, "(") != text::count(&head, ")") {
                    continue;
                }
                if a_conclusion && _SO_DOES_NOT_ASK_SOMETHING_OF_THEM.match_(&head).is_some() {
                    continue;
                }
                if CONDITION_CLAUSE.match_(&head).is_some() {
                    conditional = true; // the rest of this sentence is what to do in that case
                    continue;
                }
                if conditional {
                    continue;
                }
                if RED_FLAG.match_(&head).is_some() {
                    continue;
                }
                if NOT_A_MOVE.match_(&head).is_some() {
                    continue; // a referral ends a health answer
                }
                let words = text::split_ref(&head);
                let most = if a_conclusion { *STEP_WITH_ITS_METHOD } else { 14 };
                if !(2..=most).contains(&words.len()) {
                    continue;
                }
                if !IMPERATIVES.contains(&text::lower(words[0])) {
                    continue;
                }
                if !questions_are_steps && _a_question(&head) {
                    continue;
                }
                chosen = Some(head);
                break;
            }
            let Some(head) = chosen else { continue 'pieces };
            // "Start with run-walk: ..." already leads; a lead in front of it says start twice.
            if RE_STARTS_WITH.match_(&head).is_some() && part.contains(':') {
                continue;
            }
            return Some(lower_first(&head));
        }
    }
    None
}

/// Python: bot/memory/repeat.py:1411 `_a_conditional_move(said)`
///
/// The answer's step that comes wrapped in its own condition, kept whole, or None.
pub fn _a_conditional_move(said: &str) -> Option<String> {
    let parts: Vec<String> =
        SENTENCE.split(text::strip(said)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    for part in &parts {
        if RED_FLAG.match_(part).is_some() || NOT_A_MOVE.match_(part).is_some() {
            continue; // a warning or a referral is not a step
        }
        let Some(found) = CONDITION_CLAUSE.match_(part) else { continue };
        let head = RE_DASH_OR_BRACKET.split(text::strip(part)).into_iter().next().unwrap_or_default();
        let step = text::strip_chars(&head, " ,.;:").to_string();
        // The condition runs to its comma, and the step is what follows it.
        let rest = match step.split_once(',') {
            Some((_, after)) => after.to_string(),
            None => text::slice(&step, text::char_index(part, found.end()) as isize, None),
        };
        let rest = text::strip_chars(&rest, " ,");
        if !(3..=14).contains(&n_words(rest)) {
            continue; // a two-word step is a shrug, not the thing to do
        }
        if !_opens_an_instruction(rest) {
            continue; // the condition leads into a fact, not an instruction
        }
        if _A_JUDGEMENT.match_(&step).is_some() {
            continue; // the decision itself is not a step
        }
        if !(4..=24).contains(&n_words(&step)) {
            continue;
        }
        if _explains(&step) {
            continue;
        }
        return Some(lower_first(&step));
    }
    None
}

/// Python: bot/memory/repeat.py:1457 `asks_for_more(text)`
///
/// Whether the line asks for the next part of the answer in hand, and says nothing else.
pub fn asks_for_more(text_: &str) -> bool {
    if az_words(&text::lower(text_)).len() > 4 {
        return false;
    }
    NEXT_ASK.is_match(text_)
}

/// Python: bot/memory/repeat.py:1470 `a_hand_back(said, memory)`
///
/// Whether this bot line was handed back out of an earlier answer rather than being one.
pub fn a_hand_back(said: &str, memory: &Memory) -> bool {
    let wanted = first_chars(&text::lower(text::strip(said)), 60).to_string();
    if wanted.is_empty() {
        return false;
    }
    repeated_lines(memory).iter().any(|line| text::lower(line).contains(&wanted))
}

/// Python: bot/memory/repeat.py:1484 `_handed_back_last(memory)`
///
/// Whether the bot's last turn was a short form handed back out of an earlier answer.
pub fn _handed_back_last(memory: &Memory) -> bool {
    match turns(memory).last() {
        None => false,
        Some(last) => a_hand_back(&last.bot, memory),
    }
}

/// Python: bot/memory/repeat.py:1492 `_another_part(said, memory)`
///
/// The next part of the answer in hand and whether it says what to do, else ("", False).
pub fn _another_part(said: &str, memory: &Memory) -> (String, bool) {
    let mut spare = String::new();
    for part in SENTENCE.split(text::strip(said)) {
        let part = text::rstrip_chars(text::strip(&part), " .").to_string();
        if n_words(&part) < 4 || text::len(&part) > *MOST {
            continue;
        }
        if RED_FLAG.match_(&part).is_some() || ASKING_THEM.is_match(&part) {
            continue;
        }
        if _already_repeated(&part, memory) {
            continue;
        }
        // The lead already says where the part comes from.
        let part = RE_PART_LEAD.sub("", &part);
        let mut part = RE_THEN_LEAD.sub("", &part);
        if text::len(&part) > 110 {
            // A sentence that carries its examples after a colon says what to do before it.
            let head = text::strip_chars(part.split(": ").next().unwrap_or(""), " ,;").to_string();
            if n_words(&head) >= 4 {
                let tail = part.split_once(": ").map(|(_, t)| t).expect("IndexError: list index out of range");
                let under: Vec<String> = RE_COMMA_SPACES
                    .split(tail)
                    .iter()
                    .map(|one| text::strip_chars(one, " ,;:").to_string())
                    .filter(|one| !one.is_empty())
                    .collect();
                if !under.is_empty() && _already_repeated(&under[0], memory) {
                    for clause in &under[1..] {
                        if _says_what_to_do_bool(clause) && !_already_repeated(clause, memory) {
                            return (clause.clone(), true);
                        }
                    }
                    continue;
                }
                part = head;
                // The head on its own is the part that was handed back a turn ago.
                if _already_repeated(&part, memory) {
                    continue;
                }
            }
        }
        if _says_what_to_do_bool(&part) {
            return (part, true);
        }
        if spare.is_empty() {
            spare = part;
        }
    }
    // An explanatory part is offered only when it was not on the screen already.
    if !spare.is_empty() && !_said_in_a_turn(&spare, memory) {
        return (spare, false);
    }
    (String::new(), false)
}

/// Python: bot/memory/repeat.py:1557 `_said_in_a_turn(short, memory)`
///
/// Whether these words are already in a reply the bot gave, whichever turn it was.
pub fn _said_in_a_turn(short: &str, memory: &Memory) -> bool {
    let said = text::lower(first_chars(text::strip(short), 60));
    if said.is_empty() {
        return false;
    }
    turns(memory).iter().any(|turn| text::lower(&turn.bot).contains(&said))
}

/// Python: bot/memory/repeat.py:1568 `_says_what_to_do(part)`
///
/// True when the sentence tells them to do something rather than saying how things are.
/// (The contract types it `Value`; it is a bool.)
pub fn _says_what_to_do(part: &str) -> Value {
    Value::Bool(_says_what_to_do_bool(part))
}

fn _says_what_to_do_bool(part: &str) -> bool {
    let low = text::lower(part);
    let words: Vec<String> = text::split_ref(&low).iter().map(|w| text::strip_chars(w, " ,;:.").to_string()).collect();
    let orders = RE_DO_NOT.is_match(part);
    let Some(at) = words.iter().position(|w| IMPERATIVES.contains(w)) else { return orders };
    // A verb that comes after the sentence has said how things are is not the doing asked for.
    if words[..at].iter().any(|w| ["is", "are", "was", "were", "means", "sounds"].contains(&w.as_str())) {
        return orders;
    }
    true
}

/// Python: bot/memory/repeat.py:1607 `_said_in_the_chat(said, memory)`
///
/// True when the chat has already read this wording.
pub fn _said_in_the_chat(said: &str, memory: &Memory) -> bool {
    let flat = text::squash(&text::lower(said).replace("...", " "));
    let opening = text::split_ref(&flat).iter().take(10).cloned().collect::<Vec<_>>().join(" ");
    if opening.is_empty() {
        return false;
    }
    turns(memory).iter().any(|turn| text::squash(&text::lower(&turn.bot)).contains(&opening))
}

/// Python: bot/memory/repeat.py:1623 `_claims_what_it_never_said(reply, memory)`
///
/// The line, with its claim to have said this before taken off when the claim does not hold.
pub fn _claims_what_it_never_said(reply: &str, memory: &Memory) -> String {
    let line = text::strip(reply);
    let low = text::lower(line);
    for head in CLAIMS_IT_WAS_SAID.iter() {
        if !low.starts_with(head.as_str()) {
            continue;
        }
        let after = text::slice(line, text::len(head) as isize, None);
        let rest = text::strip(text::lstrip_chars(&after, " :-,\u{2014}")).to_string();
        if !rest.is_empty() && !_said_in_the_chat(&rest, memory) {
            return upper_first(&rest);
        }
        return line.to_string();
    }
    line.to_string()
}

/// Python: bot/memory/repeat.py:1637 `_handed_back(memory, reply)`
///
/// Keep the line handed back, so the next bare ask sees it as already said.
pub fn _handed_back(memory: &mut Memory, reply: &str) -> String {
    let reply = _claims_what_it_never_said(reply, memory);
    let kept = memory.get("repeated");
    if !matches!(kept, Some(Value::List(_))) {
        memory.set("repeated", Value::list());
    }
    let kept = memory.get_mut("repeated").unwrap();
    kept.push(reply.clone());
    if let Some(items) = kept.items_mut() {
        let extra = items.len().saturating_sub(8);
        items.drain(..extra);
    }
    reply
}

/// Python: bot/memory/repeat.py:1656 `_frame_heads()`
///
/// Every lead-in this module can put in front of a line, as the text it opens on: read off the
/// module's own tuples and lists of templates, so a pool added later is framed like the rest.
pub fn _frame_heads() -> &'static [String] {
    static HEADS: OnceLock<Vec<String>> = OnceLock::new();
    HEADS.get_or_init(|| {
        let mut heads: HashSet<String> = HashSet::new();
        for name in data::names_with_prefix("bot.memory.repeat.") {
            let value = data::get(name);
            if !matches!(value, Value::Tuple(_) | Value::List(_)) {
                continue;
            }
            for item in value.items() {
                let Some(item) = item.as_str() else { continue };
                let head = text::strip(item.split('{').next().unwrap_or(""));
                if item.contains('{') && !head.is_empty() {
                    heads.insert(text::lower(head));
                }
            }
        }
        let mut heads: Vec<String> = heads.into_iter().collect();
        heads.sort_by(|a, b| text::len(b).cmp(&text::len(a)).then_with(|| a.cmp(b)));
        heads
    })
}

/// Python: bot/memory/repeat.py:1681 `framed(said)`
///
/// True when this line already has one of this module's hand-back frames on it.
pub fn framed(said: &str) -> bool {
    let line = text::lower(text::strip(said));
    !line.is_empty() && _frame_heads().iter().any(|head| line.starts_with(head.as_str()))
}

/// Python: bot/memory/repeat.py:1693 `handed_back(memory, reply)`
///
/// The public form of `_handed_back` (the contract types its value `Value`).
pub fn handed_back(memory: &mut Memory, reply: &str) -> Value {
    Value::from(_handed_back(memory, reply))
}

/// Python: bot/memory/repeat.py:1702 `_already_repeated(short, memory)`
///
/// Once the answer has come back once, another "what now" wants the next step.
pub fn _already_repeated(short: &str, memory: &Memory) -> bool {
    let said = opening60(short);
    if said.is_empty() {
        return false;
    }
    let mut seen = turns(memory).iter().filter(|turn| text::lower(&turn.bot).contains(&said)).count();
    seen += repeated_lines(memory).iter().filter(|line| text::lower(line).contains(&said)).count();
    seen >= 2
}

/// Python: bot/memory/repeat.py:1724 `_handed_back_already(step, memory)`
///
/// True when this very step has already been handed back as the reply, not merely said.
pub fn _handed_back_already(step: &str, memory: &Memory) -> bool {
    let said = opening60(step);
    if said.is_empty() {
        return false;
    }
    for turn in turns(memory) {
        let reply = text::lower(text::strip(&turn.bot));
        if reply.is_empty() || !reply.contains(&said) {
            continue;
        }
        // The reply has to be the step rather than a paragraph that carries it.
        if text::len(&said) * 3 >= text::len(&reply) {
            return true;
        }
    }
    repeated_lines(memory).iter().any(|line| text::lower(line).contains(&said))
}

/// Python: bot/memory/repeat.py:1806 `_a_lead_of_its_own(short)`
///
/// True when the short form already opens with a label of its own ("Quick: ...").
pub fn _a_lead_of_its_own(short: &str) -> bool {
    match OWN_LEAD.match_(short) {
        Some(found) => n_words(found.as_str()) <= *OWN_LEAD_WORDS,
        None => false,
    }
}

/// Python: bot/memory/repeat.py:1826 `_kept_clause(text)`
///
/// The words a shorter version has to keep, or "" when the line names none.
pub fn _kept_clause(text_: &str) -> String {
    let said = text::squash(text_);
    match KEEP_CLAUSE.search(&said) {
        Some(found) => text::strip_chars(found.name_str("what"), " .!?,").to_string(),
        None => String::new(),
    }
}

/// Python: bot/memory/repeat.py:1849 `asks_for_less(text)`
///
/// True when the whole line, bar what it says to keep, is a request for a shorter answer.
pub fn asks_for_less(text_: &str) -> bool {
    let clean = text::squash(text_);
    if clean.is_empty() {
        return false;
    }
    if HOW_TO.is_match(&clean) {
        return false;
    }
    let asked = if KEEP_CLAUSE.is_match(&clean) { KEEP_CLAUSE.sub("", &clean) } else { clean.clone() };
    let said = topics::words(&asked);
    let left = said.iter().any(|w| {
        !LESS_WORDS.contains(w) && !FILLER.contains(w) && !ASK.contains(w) && !ASK_VERB.contains(w) && !STOPWORDS.contains(w)
    });
    if left {
        return false;
    }
    if !(LESS.is_match(&clean) || said.iter().any(|w| BREVITY_WORDS.contains(w))) {
        return false;
    }
    // The last word of the line that is not filler decides it.
    let source = if said.is_empty() { topics::words(&clean) } else { said };
    let settled: Vec<String> = source.into_iter().filter(|w| !FILLER.contains(w) && !STOPWORDS.contains(w)).collect();
    match settled.last() {
        Some(last) => LESS_WORDS.contains(last) || BREVITY_WORDS.contains(last),
        None => false,
    }
}

/// Python: bot/memory/repeat.py:1920 `_recent_lines(memory)`
///
/// The bot's own recent lines that said something, newest first, with how many lines ago.
pub fn _recent_lines(memory: &Memory) -> Vec<(usize, String)> {
    let repeated: Vec<Value> = match memory.get("repeated") {
        Some(v) if v.truthy() => v.items().to_vec(),
        _ => Vec::new(),
    };
    let mut out: Vec<(usize, String)> = Vec::new();
    for turn in last_turns(memory, *LOOK_BACK_FOR_LESS).iter().rev() {
        let said = text::strip(&turn.bot);
        if said.is_empty() || HEDGE.is_match(said) {
            continue;
        }
        if module_of(turn) == WELCOMES.as_str() && _the_quotation(said).is_empty() {
            continue; // a hello has nothing to shorten; a draft it handed over does
        }
        if repeated.iter().any(|line| line.as_str() == Some(said)) {
            continue; // our own hand-back
        }
        if n_words(said) < 6 || text::rstrip(said).ends_with('?') {
            continue;
        }
        out.push((out.len() + 1, said.to_string()));
    }
    out
}

/// Python: bot/memory/repeat.py:1955 `_sentences(said)`
///
/// The sentences of a line, split only where the line is not inside a quotation.
pub fn _sentences(said: &str) -> Vec<String> {
    let text_ = text::strip(said);
    let mut out = Vec::new();
    let mut last = 0;
    for end in SENTENCE_END.finditer(text_) {
        let piece = text::rstrip(&text_[last..end.end()]);
        if _unbalanced(piece) {
            continue;
        }
        if !text::strip(piece).is_empty() {
            out.push(text::strip(piece).to_string());
        }
        last = end.end();
    }
    let tail = text::strip(&text_[last..]);
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

/// Python: bot/memory/repeat.py:1979 `_most_condensed(said, memory=None)`
///
/// The one sentence of the answer that carries the most, for a second request for less.
pub fn _most_condensed(said: &str, memory: Option<&Memory>) -> String {
    let empty = Memory::new();
    let memory = memory.unwrap_or(&empty);
    let sentences: Vec<String> =
        _sentences(text::strip(said)).iter().map(|p| text::strip_chars(text::strip(p), ",;:").to_string()).collect();
    let parts: Vec<String> = sentences
        .iter()
        .filter(|p| (4..=30).contains(&n_words(p)) && RED_FLAG.match_(p).is_none())
        .filter(|p| !_already_repeated(p, memory))
        .cloned()
        .collect();
    let shortest = |items: &[String]| -> Option<String> {
        let mut best: Option<&String> = None;
        for item in items {
            if best.is_none_or(|b| n_words(item) < n_words(b)) {
                best = Some(item);
            }
        }
        best.cloned()
    };
    if parts.is_empty() {
        // An answer written in long sentences gets its shortest one anyway.
        let long_parts: Vec<String> = sentences
            .iter()
            .filter(|p| (4..=45).contains(&n_words(p)) && RED_FLAG.match_(p).is_none())
            .filter(|p| !_already_repeated(p, memory))
            .cloned()
            .collect();
        if let Some(best) = shortest(&long_parts) {
            return best;
        }
    }
    shortest(&parts).unwrap_or_default()
}

/// Python: bot/memory/repeat.py:2017 `_short_enough(cut)`
///
/// The answer's shorter read, trimmed to something a person would call short.
pub fn _short_enough(cut: &str) -> String {
    let cut = text::strip(cut);
    if n_words(cut) <= *SHORT_ENOUGH {
        return _ended(cut);
    }
    let head = _read_up_to_the_colon(cut);
    if n_words(&head) <= *SHORT_ENOUGH {
        return _ended(&head);
    }
    let parts: Vec<String> =
        CLAUSE_SPLIT.split(&head).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    let mut out = String::new();
    for part in &parts {
        let together = text::strip(&format!("{} {}", out, part)).to_string();
        if !out.is_empty() && n_words(&together) > *SHORT_ENOUGH {
            break;
        }
        out = together;
    }
    let trimmed = text::rstrip_chars(&out, ",;").to_string();
    let mut words: Vec<&str> = text::split_ref(&trimmed);
    while let Some(last) = words.last() {
        if CUT_MID_SENTENCE.contains(text::strip_chars(&text::lower(last), ",;")) {
            words.pop();
        } else {
            break;
        }
    }
    let out = text::rstrip_chars(&words.join(" "), ",;").to_string();
    if n_words(&out) < 6 {
        return _ended(&head);
    }
    _ended(&out)
}

/// Python: bot/memory/repeat.py:2047 `_ended(cut)`
///
/// A short answer cut out of a sentence still ends like a sentence.
pub fn _ended(cut: &str) -> String {
    let cut = text::rstrip_chars(text::strip(cut), ",;:");
    let mut chars = cut.chars().rev();
    let Some(last) = chars.next() else { return String::new() };
    if ".!?".contains(last) {
        return cut.to_string();
    }
    if "\"\u{201d}".contains(last) {
        match chars.next() {
            None => return cut.to_string(),
            Some(before) if ".!?".contains(before) => return cut.to_string(),
            _ => {}
        }
    }
    format!("{}.", cut)
}

/// Python: bot/memory/repeat.py:2062 `_unbalanced(piece)`
///
/// True when a piece opens a quotation it does not close, so it is the middle of one.
pub fn _unbalanced(piece: &str) -> bool {
    for (opens, closes) in QUOTE_PAIRS.iter() {
        if opens == closes {
            if text::count(piece, opens) % 2 == 1 {
                return true;
            }
        } else if text::count(piece, opens) != text::count(piece, closes) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/repeat.py:2078 `_the_quotation(said)`
///
/// The draft the answer handed over inside its own quotation marks, or "".
pub fn _the_quotation(said: &str) -> String {
    for (opens, closes) in QUOTE_PAIRS.iter() {
        let mut at = said.find(opens.as_str());
        while let Some(start) = at {
            let from = start + opens.len();
            if let Some(end) = said[from..].find(closes.as_str()).map(|e| from + e) {
                let quote = text::strip(&said[from..end]);
                // The quote has to be most of the answer.
                if n_words(quote) >= 4 && n_words(quote) * 2 >= n_words(said) {
                    return quote.to_string();
                }
            }
            at = said[from..].find(opens.as_str()).map(|e| from + e);
        }
    }
    String::new()
}

/// Python: bot/memory/repeat.py:2103 `_closing_the_quote(cut, said)`
///
/// The cut, carried on to the end of the quotation it stops inside.
pub fn _closing_the_quote(cut: &str, said: &str) -> String {
    let cut = text::strip(cut);
    let whole = text::strip(said);
    if cut.is_empty() || !_unbalanced(cut) || !whole.contains(cut) {
        return cut.to_string();
    }
    let rest = &whole[whole.find(cut).unwrap() + cut.len()..];
    for end in RE_QUOTE_END.finditer(rest) {
        let longer = text::strip(&format!("{}{}", cut, &rest[..end.end()])).to_string();
        if !_unbalanced(&longer) {
            return longer;
        }
    }
    cut.to_string()
}

/// Python: bot/memory/repeat.py:2123 `_read_up_to_the_colon(said)`
///
/// The sentence, or the part of it before its own colon when that is a shorter read of it.
pub fn _read_up_to_the_colon(said: &str) -> String {
    let head = text::partition(said, ":").0;
    if head != said && 8 <= n_words(head) && n_words(head) < n_words(said) {
        return text::strip(head).to_string();
    }
    said.to_string()
}

/// Python: bot/memory/repeat.py:2131 `_keeps(said, what)`
///
/// The sentence of the answer that carries what the person said the short version has to keep.
pub fn _keeps(said: &str, what: &str) -> String {
    let parts: Vec<String> =
        SENTENCE.split(text::strip(said)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    let words: HashSet<String> = word_set(what)
        .into_iter()
        .filter(|w| !STOPWORDS.contains(w) && !["timings", "timing", "numbers", "figures", "details"].contains(&w.as_str()))
        .collect();
    let named: Vec<&String> = parts.iter().filter(|p| word_set(p).iter().any(|w| words.contains(w))).collect();
    if !named.is_empty() {
        let mut best = named[0];
        for one in &named[1..] {
            if text::len(one) > text::len(best) {
                best = one;
            }
        }
        return best.clone();
    }
    if FIGURES.is_match(what) {
        if let Some(counted) = parts.iter().find(|p| DIGIT.is_match(p)) {
            return counted.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/repeat.py:2152 `shorter_for(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`shorter_for_text`].
pub fn shorter_for(text_: &str, memory: &mut Memory) {
    let _ = shorter_for_text(text_, memory);
}

/// The last answer's own lead, or None when this is not a request for less (`shorter_for`).
pub fn shorter_for_text(text_: &str, memory: &mut Memory) -> Option<String> {
    if !asks_for_less(text_) {
        return None;
    }
    let mut said_any = String::new();
    for (back, said) in _recent_lines(memory) {
        if back > 2 {
            break;
        }
        // A semicolon is where the answer already offered a shorter read of itself.
        let parts: Vec<String> = _sentences(&said).iter().map(|p| text::strip_chars(text::strip(p), ";").to_string()).collect();
        if said_any.is_empty() {
            said_any = said.clone();
        }
        let mut lead = if parts.len() > 1 { parts[0].clone() } else { String::new() };
        if !lead.is_empty() && n_words(&lead) < 8 && parts.len() > 1 {
            lead = parts[1].clone(); // "Sorry to hear that." on its own is not the answer
        }
        if lead.is_empty() {
            lead = text::strip(&_read_up_to_the_colon(&said)).to_string();
        }
        let kept = _kept_clause(text_);
        if !kept.is_empty() {
            // What they said to keep is the short version they asked for.
            let carry = _keeps(&said, &kept);
            if !carry.is_empty() {
                let line = py_format(&pick(&KEEPING, memory, text_), &[("keep", &kept), ("short", &carry)]);
                return Some(_handed_back(memory, &line));
            }
        }
        // A draft handed over in quotation marks is the thing to give back shorter.
        let quote = _the_quotation(&said);
        if !quote.is_empty() && text::strip(&quote) != text::strip(&said) && !_already_repeated(&quote, memory) {
            let line = py_format(&pick(&JUST_THIS, memory, text_), &[("short", &quote)]);
            return Some(_handed_back(memory, &line));
        }
        let condensed = _most_condensed(&said, Some(memory));
        for (pool, cut) in [(&*LESS_LEADS, lead.clone()), (&*JUST_THIS, condensed)] {
            let cut = _closing_the_quote(&_short_enough(&cut), &said);
            if cut.is_empty() || _unbalanced(&cut) {
                continue;
            }
            if text::lower(text::strip(&cut)) == text::lower(text::strip(&said)) {
                continue;
            }
            if _already_repeated(&cut, memory) {
                continue;
            }
            let mut line = pick(pool, memory, text_);
            if line != "{short}" && _a_lead_of_its_own(&cut) {
                line = "{short}".to_string();
            }
            let line = py_format(&line, &[("short", &cut)]);
            return Some(_handed_back(memory, &line));
        }
    }
    if said_any.is_empty() {
        let line = pick(&NOTHING_TO_CUT, memory, text_);
        return Some(_handed_back(memory, &line));
    }
    // It is already one sentence: there is nothing to take out, and saying so is the answer.
    let one = _most_condensed(&said_any, Some(memory));
    if !one.is_empty() {
        let line = py_format(&pick(&ALREADY_SHORT, memory, text_), &[("short", &_ended(&one))]);
        return Some(_handed_back(memory, &line));
    }
    // Every sentence of it has now been read out as the short version.
    let line = pick(&NO_SHORTER_STILL, memory, text_);
    Some(_handed_back(memory, &line))
}

/// Python: bot/memory/repeat.py:2233 `_a_bare_time(part)`
///
/// The part of an answer when it is nothing but the length asked for, else None.
pub fn _a_bare_time(part: Option<&str>) -> Option<String> {
    let part = text::squash(part?);
    if part.is_empty() || n_words(&part) > 6 || A_BARE_TIME.match_(&part).is_none() {
        return None;
    }
    Some(upper_first(&part))
}

/// Python: bot/memory/repeat.py:2268 `_names_no_thing_of_its_own(text)`
///
/// Whether the line asks about a price without naming what the price is for.
pub fn _names_no_thing_of_its_own(text_: &str) -> bool {
    let words = topics::words(text_);
    if words.is_empty() || !words.iter().any(|w| POINTING_BACK.contains(w)) {
        return false;
    }
    words.iter().all(|w| POINTING_BACK.contains(w) || PRICE_WORDS.contains(w) || STOPWORDS.contains(w))
}

/// Python: bot/memory/repeat.py:2277 `_the_price_the_answer_gave(text, memory)`
///
/// The price the answer in hand stated, when the line asks what that costs, else None.
pub fn _the_price_the_answer_gave(text_: &str, memory: &mut Memory) -> Option<String> {
    let part = _the_price_in_hand(text_, memory)?;
    if _a_length_already_handed(&part, memory) {
        return None;
    }
    Some(_handed_back(memory, &part))
}

/// Python: bot/memory/repeat.py:2298 `asks_what_that_costs(text, memory)`
///
/// Whether the line asks the price of the thing the answer in hand has just priced.
pub fn asks_what_that_costs(text_: &str, memory: &Memory) -> bool {
    _the_price_in_hand(text_, memory).is_some()
}

fn money_span() -> &'static Re {
    data::get("bot.memory.narrow.SPANS").get("money").and_then(Value::as_re).expect("narrow.SPANS money")
}

fn time_span() -> &'static Re {
    data::get("bot.memory.narrow.SPANS").get("time").and_then(Value::as_re).expect("narrow.SPANS time")
}

/// Python: bot/memory/repeat.py:2308 `_the_price_in_hand(text, memory)`
///
/// The bare price the answer in hand gave, when the line asks what that costs, else None.
pub fn _the_price_in_hand(text_: &str, memory: &Memory) -> Option<String> {
    if !A_PRICE_ASK.is_match(text_) || narrow::kind_of(text_).as_deref() != Some("money") {
        return None;
    }
    if !_names_no_thing_of_its_own(text_) {
        return None;
    }
    let last = turns(memory).last()?.bot.clone();
    if last.is_empty() {
        return None;
    }
    let found: HashSet<String> = money_span().finditer(&last).iter().map(|hit| text::squash(&text::lower(hit.as_str()))).collect();
    if found.len() != 1 {
        return None;
    }
    let part = narrow::narrow(&last, text_, "")?;
    if part.is_empty() || !money_span().is_match(&part) {
        return None;
    }
    let mut part = text::squash(&part);
    if n_words(&part) > 8 {
        return None;
    }
    if !part.ends_with(['.', '!', '?']) {
        part.push('.');
    }
    Some(upper_first(&part))
}

/// Python: bot/memory/repeat.py:2336 `_the_time_an_earlier_answer_gave(text, memory)`
///
/// The length an answer further back gave, when the line asks for one and the last one does not
/// hold it.
pub fn _the_time_an_earlier_answer_gave(text_: &str, memory: &mut Memory) -> Option<String> {
    if !A_LENGTH_ASK.is_match(text_) || narrow::kind_of(text_).as_deref() != Some("time") {
        return None;
    }
    if turns(memory).len() < 2 {
        return None;
    }
    // The answer in hand is the normal place for this.
    let last = turns(memory).last().map(|t| t.bot.clone()).unwrap_or_default();
    if !last.is_empty() && _a_bare_time(narrow::narrow(&last, text_, "").as_deref()).is_some() {
        return None;
    }
    for (back, asked, said) in recent_answers(memory) {
        if back < 2 || back > *LENGTH_BACK || !time_span().is_match(&said) {
            continue;
        }
        if !_about_the_same_thing(&asked, &said, memory) {
            continue;
        }
        // Only while the chat is still on the thing that answer was about.
        if !_the_chat_is_still_on(&format!("{} {}", asked, said), memory) {
            continue;
        }
        if let Some(part) = _a_bare_time(narrow::narrow(&said, text_, "").as_deref()) {
            if !_a_length_already_handed(&part, memory) {
                return Some(_handed_back(memory, &part));
            }
        }
    }
    None
}

/// Python: bot/memory/repeat.py:2377 `_a_length_already_handed(part, memory)`
///
/// True when this length has been handed back already in this chat.
pub fn _a_length_already_handed(part: &str, memory: &Memory) -> bool {
    let Some(Value::List(kept)) = memory.get("repeated") else { return false };
    let flat = text::strip_chars(&text::squash(&text::lower(part)), " .").to_string();
    kept.iter().any(|said| text::strip_chars(&text::squash(&text::lower(&py_str(said))), " .") == flat)
}

/// Python: bot/memory/repeat.py:2392 `_a_what_to_do_answer(memory)`
///
/// True when the answer in hand is itself the answer to "what should i do": a plan, or the free
/// time to fill.
pub fn _a_what_to_do_answer(memory: &Memory) -> bool {
    let Some(answer) = memory.get("entry_answer").filter(|a| a.is_dict()) else { return false };
    let mut blocks: HashSet<String> =
        ["plan", "plans", "planning", "weekend", "weekends", "bored", "dayoff"].iter().map(|s| s.to_string()).collect();
    for one in FREE_TIME.iter() {
        let low = text::lower(one);
        let words: Vec<&str> = text::split_ref(&low).into_iter().filter(|w| text::len(w) > 3).collect();
        if !words.is_empty() && words.iter().all(|w| blocks.contains(*w)) {
            blocks.extend(words.iter().map(|w| w.to_string()));
        }
    }
    match answer.get("keys") {
        Some(keys) if keys.truthy() => keys.items().iter().any(|key| blocks.contains(&text::lower(&py_str(key)))),
        _ => false,
    }
}

/// Python: bot/memory/repeat.py:2421 `_a_city_guide_is_the_answer_in_front(memory)`
///
/// True when the reply on the screen is a city's list of places to go.
pub fn _a_city_guide_is_the_answer_in_front(memory: &Memory) -> bool {
    match turns(memory).last() {
        None => false,
        Some(last) => crate::knowledge::travel::a_city_card_is_in_front(&last.bot),
    }
}

/// Python: bot/memory/repeat.py:2438 `answer_for(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`answer_for_text`].
pub fn answer_for(text_: &str, memory: &mut Memory) {
    let _ = answer_for_text(text_, memory);
}

/// The answer they just asked for again, or None when this is not that turn (`answer_for`).
pub fn answer_for_text(text_: &str, memory: &mut Memory) -> Option<String> {
    // The card in front is a city's places, and which of them comes first is the city row's own.
    if asks_for_the_move(text_) && _a_city_guide_is_the_answer_in_front(memory) {
        if crate::knowledge::travel::asks_which_one_first(text_) {
            return None;
        }
    }
    let more = asks_for_more(text_) && _handed_back_last(memory);
    if USE_WITH.is_match(text_) && !_a_what_to_do_answer(memory) {
        return None;
    }
    if WORDING.is_match(text_) {
        return None; // a line about how to say it is not a line about the last answer
    }
    if let Some(time_said) = _the_time_an_earlier_answer_gave(text_, memory) {
        if !time_said.is_empty() {
            return Some(time_said);
        }
    }
    if let Some(price_said) = _the_price_the_answer_gave(text_, memory) {
        if !price_said.is_empty() {
            return Some(price_said);
        }
    }
    let put_right = asks_to_put_it_right(text_);
    if !(names_nothing(text_) || a_step_of_the_same_answer(text_))
        && !(more || says_where_to_start(text_) || HOW_IT.is_match(text_) || a_more_ask(text_) || put_right)
    {
        return None;
    }
    // An ask for an idea is answered by the entries when an entry has an idea of its own.
    if IDEAS_ONLY.match_(text_).is_some() && _an_entry_answers_it(text_, memory) {
        return None;
    }
    let which_part = asks_for_the_move(text_);
    if let Some(last) = turns(memory).last() {
        if !which_part && REFER_BACK.is_match(&last.bot) {
            return None;
        }
        // A turn that said the answer again with something taken out of it is not a new answer.
        if crate::compose::limit::restates_the_answer(&last.bot) {
            return None;
        }
    }
    let tried = turns(memory).last().map(|t| t.user.clone()).unwrap_or_default();
    let wanted = narrow::kind_of(text_);
    let recent = _the_answer_about_what_we_are_on(memory, recent_answers(memory));
    for (back, asked, said) in recent {
        if back > 1
            && !_about_the_same_thing(&asked, &said, memory)
            && !_only_echoes_between(memory, back)
            && !_nothing_but_acknowledgements(back, memory)
        {
            continue;
        }
        if _a_new_work(text_, &asked) {
            return None; // a new question about the same subject
        }
        if let Some(kind) = wanted.as_deref() {
            if !narrow::carries(&said, kind) {
                continue; // a figure the answer does not carry is not the answer again
            }
        }
        let mut nothing_more = false;
        if more {
            let (part, doing) = _another_part(&said, memory);
            if !part.is_empty() {
                let leads = if doing { &*NEXT_LEADS } else { &*ALSO_LEADS };
                let line = py_format(&pick(leads, memory, text_), &[("step", &upper_first(&part))]);
                return Some(_handed_back(memory, &line));
            }
        }
        if !tried.is_empty() && !QUESTION_MORE.is_match(&tried) && (_tried(&tried) || _a_step_done(&tried, &said)) {
            let later = later_part(&tried, &said);
            if !later.is_empty() {
                let short = _short(&later);
                let lead = _a_lead_for(&_the_lead_for(text_, &short), &short, memory, text_);
                return Some(_handed_back(memory, &_a_lead_and_a_step(&lead, &short)));
            }
            if !a_more_ask(text_) {
                continue;
            }
        }
        if a_more_ask(text_) {
            if let Some(step) = _the_next_step(&said, memory) {
                let line = py_format(&pick(&MORE_LEADS, memory, text_), &[("step", &upper_first(&step))]);
                return Some(_handed_back(memory, &line));
            }
            nothing_more = true;
        }
        if asks_for_the_move(text_) {
            // "Where do i start" is read forwards and the first instruction given.
            if let Some(step) = _the_first_step(&said) {
                if !_handed_back_already(&step, memory) && !_they_say_they_tried(&step, memory) {
                    let wording = upper_first(&step);
                    let lead = _a_lead_for(&_leads_for(text_), &wording, memory, text_);
                    return Some(_handed_back(memory, &_a_lead_and_a_step(&lead, &wording)));
                }
            }
        }
        if MOVE_ASK.is_match(text_) || put_right || AND_NOW.match_(text_).is_some() {
            let mut step = a_move(&said, false);
            if step.is_none() && AT_A_LOSS.is_match(text_) {
                // Every step waits on a moment: the person at a loss gets it with its moment.
                step = _a_conditional_move(&said);
            }
            if let Some(step) = step.filter(|s| !s.is_empty()) {
                if !_already_repeated(&step, memory) && !_they_say_they_tried(&step, memory) {
                    let wording = upper_first(&step);
                    let leads: Vec<String> = if put_right {
                        PUT_RIGHT_LEADS.clone()
                    } else if AT_A_LOSS.is_match(text_) {
                        LOST_LEADS.clone()
                    } else {
                        _leads_for(text_)
                    };
                    let lead = _a_lead_for(&leads, &wording, memory, text_);
                    return Some(_handed_back(memory, &_a_lead_and_a_step(&lead, &wording)));
                }
                // The step has already been handed back once: offered again rather than nothing.
                let wording = upper_first(&step);
                let lead = _a_lead_for(&_the_lead_for(text_, &step), &wording, memory, text_);
                return Some(_handed_back(memory, &_a_lead_and_a_step(&lead, &wording)));
            }
        }
        if more && !a_more_ask(text_) {
            return None; // nothing left of the answer to hand over
        }
        if nothing_more {
            return None;
        }
        let short = _short(&said);
        if !short.is_empty() && !_already_repeated(&short, memory) {
            let leads = if _a_bare_ask(text_) { SAY_AGAIN_LEADS.clone() } else { _the_lead_for(text_, &short) };
            let lead = _a_lead_for(&leads, &short, memory, text_);
            return Some(_handed_back(memory, &_a_lead_and_a_step(&lead, &short)));
        }
    }
    None
}

thread_local! {
    /// `_AN_ENTRY_IS_LOOKING`: the lookup is guarded against coming back into itself.
    static _AN_ENTRY_IS_LOOKING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Python: bot/memory/repeat.py:2625 `_an_entry_answers_it(text, memory)`
///
/// Whether the knowledge base has an idea of its own for this ask, rather than the answer again.
/// The entries are asked on a copy of the memory, so a probe claims nothing. (Python's copy is
/// `dict(memory)`, a shallow one, so a list the probe appends to in place is shared with the real
/// memory; the copy here is a whole one.)
pub fn _an_entry_answers_it(text_: &str, memory: &Memory) -> bool {
    if _AN_ENTRY_IS_LOOKING.with(|f| f.get()) {
        return false;
    }
    _AN_ENTRY_IS_LOOKING.with(|f| f.set(true));
    let mut copy = memory.clone();
    let reply = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::modules::answers::respond(text_, &mut copy)));
    _AN_ENTRY_IS_LOOKING.with(|f| f.set(false));
    let reply = match reply {
        Ok(reply) => reply,
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            if message.starts_with("UNPORTED") {
                std::panic::resume_unwind(payload);
            }
            return false; // a lookup that fails claims no turn
        }
    };
    let said = _words(reply.as_deref().unwrap_or(""));
    if said.is_empty() {
        return false;
    }
    let last = turns(memory).last().map(|t| t.bot.clone()).unwrap_or_default();
    let opening = _words(&last).into_iter().take(8).collect::<Vec<_>>().join(" ");
    if !opening.is_empty() && said.join(" ").contains(&opening) {
        return false; // the entry is the answer in hand, said again
    }
    true
}

/// Python: bot/memory/repeat.py:2658 `_words(text)`
///
/// The words of a line, lowercased, in order.
pub fn _words(text_: &str) -> Vec<String> {
    az_words(&text::lower(text_))
}

/// Python: bot/memory/repeat.py:2663 `_a_bare_ask(text)`
///
/// True when the whole line is one word asking to hear the last answer again.
pub fn _a_bare_ask(text_: &str) -> bool {
    let Some(found) = BARE_ASK.match_(text::strip(text_)) else { return false };
    // "sorry" and "pardon" are the ask only as a question.
    if BARE_ASK_NEEDS_A_QUESTION.contains(&text::lower(found.name_str("word"))) && !text_.contains('?') {
        return false;
    }
    true
}

/// Python: bot/memory/repeat.py:2675 `_a_lead_and_a_step(lead, step)`
///
/// A lead with the step in it, and one full stop at the end.
pub fn _a_lead_and_a_step(lead: &str, step: &str) -> String {
    let said = text::rstrip(step);
    let filled = py_format(lead, &[("step", said), ("said", said)]);
    RE_TRAILING_DOTS.sub(".", &RE_DOTS_SPACE.sub(". ", &filled))
}

/// Python: bot/memory/repeat.py:2694 `_the_lead_for(text, step='')`
///
/// Which set of leads this line gets.
pub fn _the_lead_for(text_: &str, step: &str) -> Vec<String> {
    let leads = if AT_A_LOSS.is_match(text_) { LOST_LEADS.clone() } else { LEADS.clone() };
    if A_COUNT_OF_MORE_THAN_ONE.match_(step).is_some() {
        let kept: Vec<String> = leads.iter().filter(|lead| !text::lower(lead).contains("one thing")).cloned().collect();
        if !kept.is_empty() {
            return kept;
        }
    }
    leads
}

/// Python: bot/memory/repeat.py:2709 `_content(text)`
///
/// The words of a line that name something.
pub fn _content(text_: &str) -> HashSet<String> {
    topics::words(text_).into_iter().filter(|w| !STOPWORDS.contains(w) && text::len(w) > 2).collect()
}

/// Python: bot/memory/repeat.py:2714 `_trouble(text)`
///
/// True when the line is a complaint about the thing still in play, not a fact about it.
pub fn _trouble(text_: &str) -> bool {
    TROUBLE.is_match(text_)
}

/// Python: bot/memory/repeat.py:2758 `_a_thing_of_its_own(line, said)`
///
/// True when the line names a thing as its own that the answer does not mention.
pub fn _a_thing_of_its_own(line: &str, said: &str) -> bool {
    let named: HashSet<String> = OWNED.findall_str(&text::lower(line)).into_iter().collect();
    if named.is_empty() {
        return false;
    }
    let words: HashSet<String> = az_words(&text::lower(said)).into_iter().collect();
    let stems: HashSet<String> = words.iter().map(|w| text::rstrip_chars(w, "s").to_string()).collect();
    !named.iter().any(|word| words.contains(word) || stems.contains(text::rstrip_chars(word, "s")))
}

/// Python: bot/memory/repeat.py:2789 `after_an_ack(text)`
///
/// The rest of a line that opens with a thank-you, or "" when it opens with anything else.
pub fn after_an_ack(text_: &str) -> String {
    let Some(found) = ACK_AND_REST.match_(text::strip(text_)) else { return String::new() };
    let rest = text::strip_chars(found.name_str("rest"), " ,.!?;:-\u{2014}");
    // The name is said to the bot, not part of the line.
    let rest = RE_NAME_LEAD.sub("", rest);
    // A rest that opens with "for" is what the thanks is for.
    if RE_FOR_LEAD.match_(&rest).is_some() {
        return String::new();
    }
    let named = topics::words(&rest).into_iter().filter(|w| !CLOSES_THE_TURN.contains(w)).count();
    if named >= 2 { rest } else { String::new() }
}

/// Python: bot/memory/repeat.py:2842 `_a_feeling(text)`
///
/// True when the line is about how they are, not about the thing the answer was about.
pub fn _a_feeling(text_: &str) -> bool {
    if BROODING.is_match(text_) {
        return true;
    }
    word_set(text_).iter().any(|w| NEGATIVE_CUES.contains(w))
}

/// Python: bot/memory/repeat.py:2860 `_the_doing(sentence)`
///
/// The doing part of a sentence: what follows a short label, cut to the length of a reply.
pub fn _the_doing(sentence: &str) -> String {
    let stripped = text::strip(sentence);
    let (head, sep, rest) = text::partition(stripped, ":");
    let mut whole = if !sep.is_empty() && !_opens_a_doing(head) && n_words(head) <= 12 && text::len(text::strip(rest)) >= 20 {
        upper_first(text::strip(rest))
    } else {
        stripped.to_string()
    };
    whole = crate::compose::whole::a_whole_cut(&whole, *MOST as i64);
    let last = text::rstrip_chars(&whole, "'\"\u{2019}\u{201d}");
    if !whole.is_empty() && !last.ends_with(['.', '!', '?']) {
        return format!("{}.", whole);
    }
    whole
}

/// Python: bot/memory/repeat.py:2886 `_the_advice(said)`
///
/// The sentence of an answer that tells them what to do, or "" when it tells them nothing.
pub fn _the_advice(said: &str) -> String {
    let mut best = String::new();
    let mut most = 1usize;
    for part in SENTENCE.split(text::strip(said)) {
        let part = text::strip(&part);
        if part.is_empty() || RED_FLAG.match_(part).is_some() || ASKING_THEM.is_match(part) {
            continue;
        }
        let low = text::lower(part);
        let mut count = text::split_ref(&low).iter().filter(|w| IMPERATIVES.contains(text::strip_chars(w, " ,;:."))).count();
        if RE_DO_NOT.is_match(part) {
            count += 1;
        }
        if count > most {
            best = part.to_string();
            most = count;
        }
    }
    best
}

/// Python: bot/memory/repeat.py:2908 `the_next_move(memory, text='')`
///
/// The step after advice the person says did not work, in the answer's own words, or "".
pub fn the_next_move(memory: &Memory, text_: &str) -> String {
    for (back, asked, said) in recent_answers(memory) {
        if back > 1 && !_about_the_same_thing(&asked, &said, memory) {
            continue;
        }
        if NEXT_STEP_EMERGENCY.is_match(&said) {
            // Nothing is handed back from an emergency answer, and nothing older is reached past it.
            return String::new();
        }
        let step = _next_step(&said);
        if !step.is_empty() && !_already_repeated(&step, memory) {
            return py_format(&pick(&TRIED_LEADS, memory, text_), &[("step", &upper_first(&step))]);
        }
        return String::new();
    }
    String::new()
}

/// Python: bot/memory/repeat.py:2933 `_next_step(said)`
///
/// The sentence an answer ends on about who to go to, when the advice alone is not enough.
pub fn _next_step(said: &str) -> String {
    let sentences: Vec<String> =
        SENTENCE.split(said).iter().map(|s| text::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    for sentence in sentences.iter().skip(1).rev() {
        if NEXT_STEP.is_match(sentence) {
            return sentence.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/repeat.py:2942 `_later_part(tried, said)`
///
/// The part of one answer that comes after the step the line says it has tried. (The contract
/// types it on `Value`s; [`later_part`] is the same on strings.)
pub fn _later_part(tried: &Value, said: &str) -> Value {
    Value::from(later_part(&py_str_or_empty(Some(tried)), said))
}

/// `_later_part(tried, said)` on strings.
pub fn later_part(tried: &str, said: &str) -> String {
    fn shared(part: &str, words: &HashSet<String>) -> usize {
        _content(part)
            .iter()
            .filter(|w| {
                words.iter().any(|v| {
                    *w == v || (text::len(w) >= 5 && text::len(v) >= 5 && first_chars(w, 5) == first_chars(v, 5))
                })
            })
            .count()
    }
    let parts: Vec<String> =
        SENTENCE.split(text::strip(said)).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    let tried_words = _content(tried);
    let mut best_i: Option<usize> = None;
    let mut most = 0;
    for (i, part) in parts.iter().enumerate() {
        let count = shared(part, &tried_words);
        if count > most {
            best_i = Some(i);
            most = count;
        }
    }
    if let Some(best_i) = best_i {
        // The tried step is in the answer: the sentence right after it is the next step.
        let mut later: Vec<String> = Vec::new();
        for part in &parts[best_i + 1..] {
            if RED_FLAG.match_(part).is_some() || ASKING_THEM.is_match(part) {
                continue;
            }
            if shared(part, &tried_words) >= most {
                continue;
            }
            let doing = _the_doing(part);
            if text::len(&doing) >= 25 {
                later.push(doing);
            }
        }
        // The sentence that opens on a doing is the step; a timing between steps is skipped.
        for doing in &later {
            if _opens_a_doing(doing) {
                return doing.clone();
            }
        }
        return later.into_iter().next().unwrap_or_default();
    }
    // The step is not named in the answer: the last sentence is the part they have not heard.
    let mut best = String::new();
    for part in parts.iter().skip(1) {
        if RED_FLAG.match_(part).is_some() || ASKING_THEM.is_match(part) {
            continue;
        }
        let doing = _the_doing(part);
        if text::len(&doing) >= 25 {
            best = doing;
        }
    }
    best
}

/// Python: bot/memory/repeat.py:2992 `_opens_a_doing(sentence)`
///
/// Whether a sentence opens on a doing rather than on its subject.
pub fn _opens_a_doing(sentence: &str) -> bool {
    let words = text::split_ref(text::strip_chars(sentence, " .,:;"));
    let Some(first) = words.first() else { return false };
    let first = text::strip_chars(&text::lower(first), ".,;:").to_string();
    if IMPERATIVES.contains(&first) || MOVE_VERBS.contains(&first) {
        return true;
    }
    if first.ends_with("ing") && text::len(&first) > 4 {
        let stem = &first[..first.len() - 3];
        let mut bare = vec![stem.to_string(), format!("{stem}e")];
        let chars: Vec<char> = stem.chars().collect();
        if chars.len() > 2 && chars[chars.len() - 1] == chars[chars.len() - 2] {
            bare.push(chars[..chars.len() - 1].iter().collect());
        }
        return bare.iter().any(|candidate| IMPERATIVES.contains(candidate));
    }
    false
}

/// Python: bot/memory/repeat.py:3018 `_the_untried(tried, memory)`
///
/// The later part of the last answer that is not the step they have just tried.
pub fn _the_untried(tried: &str, memory: &mut Memory) -> Option<String> {
    for (back, _question, said) in recent_answers(memory) {
        if back != 1 {
            return None;
        }
        let best = later_part(tried, &said);
        if !best.is_empty() && !_already_repeated(&best, memory) {
            return Some(_handed_back(memory, &best));
        }
    }
    None
}

/// Python: bot/memory/repeat.py:3055 `_tried(text)`
///
/// True when the line says the advice has already been carried out, not that it failed.
pub fn _tried(text_: &str) -> bool {
    TRIED.is_match(text_) && !NOT_TRIED.is_match(text_)
}

/// Python: bot/memory/repeat.py:3059 `_says_they_agree(clean)`
///
/// Whether the line carries a word that agrees: "ok so low heat then", "low heat, got it".
pub fn _says_they_agree(clean: &str) -> bool {
    let words: Vec<String> = text::split_ref(clean).iter().map(|w| text::lower(text::strip_chars(w, ".,!;:\"'"))).collect();
    if words.iter().any(|w| AGREES_ON_ITS_OWN.contains(w)) {
        return true;
    }
    if AGREES_IN_A_PHRASE.is_match(clean) {
        return true;
    }
    words.len() <= 8 && !words.is_empty() && AGREEMENT_TAIL.contains(&words[words.len() - 1])
}

/// Python: bot/memory/repeat.py:3079 `_the_thing_they_named(clean, part)`
///
/// The words of the line that are the answer's own, in the person's order: "low heat".
pub fn _the_thing_they_named(clean: &str, part: &str) -> String {
    let _ = part;
    let mut kept: Vec<String> = Vec::new();
    for word in text::split_ref(clean) {
        let bare = text::lower(text::strip_chars(word, ".,!;:\"'"));
        if bare.is_empty() || AGREEING.contains(&bare) {
            continue;
        }
        kept.push(text::strip_chars(word, ".,!;:\"'").to_string());
    }
    let scaffold = |w: &str| {
        let low = text::lower(w);
        BE_FORM.contains(&low) || POINTING_AT.contains(&low)
    };
    while kept.len() > 1 && scaffold(&kept[0]) {
        kept.remove(0);
    }
    if kept.is_empty() || kept.len() > 4 || scaffold(&kept[0]) {
        return String::new();
    }
    // A line that opens on a preposition is a detail of the answer rather than a thing it named.
    if OPENERS.contains(&text::lower(&kept[0])) {
        return String::new();
    }
    kept.join(" ")
}

/// Python: bot/memory/repeat.py:3112 `agreement_with_the_answer(text, memory)`
///
/// The confirmation of a thing the person has just said back, or "" for every other line.
pub fn agreement_with_the_answer(text_: &str, memory: &Memory) -> String {
    let clean = text::strip(text_);
    if clean.is_empty()
        || clean.ends_with('?')
        || QUESTION.match_(clean).is_some()
        || ASKS.match_(clean).is_some()
        || n_words(clean) > 20
    {
        return String::new();
    }
    if ABOUT_THE_BOT.is_match(clean) || WORDS_WANTED.is_match(clean) {
        return String::new();
    }
    let asked = _content(clean);
    if asked.len() < 2 {
        return String::new();
    }
    for (back, question, said) in recent_answers(memory) {
        if back > 1 && !_about_the_same_thing(&question, &said, memory) {
            continue;
        }
        let mut best = String::new();
        let mut shared = 0;
        for part in SENTENCE.split(&said) {
            let part = text::strip(&part);
            if part.is_empty() || RED_FLAG.match_(part).is_some() || ASKING_THEM.is_match(part) {
                continue;
            }
            let count = asked.intersection(&_content(part)).count();
            if count > shared {
                best = part.to_string();
                shared = count;
            }
        }
        if shared < asked.len() || !_says_they_agree(clean) {
            continue;
        }
        let named_thing = _the_thing_they_named(clean, &best);
        if !named_thing.is_empty() {
            return _agreed(&named_thing, memory);
        }
    }
    String::new()
}

/// Python: bot/memory/repeat.py:3148 `_agreed(named, memory)`
///
/// The confirmation of a thing the person said back: "Low heat, yes."
pub fn _agreed(named: &str, memory: &Memory) -> String {
    let mut chars = named.chars();
    let low = match chars.next() {
        Some(first) => format!("{}{}", first.to_lowercase(), chars.as_str()),
        None => String::new(),
    };
    let up = upper_first(named);
    let options = vec![
        format!("{up}, yes."),
        format!("Yes \u{2014} {low}."),
        format!("{up} it is."),
        format!("That's it: {low}."),
        format!("{up}, that's the main thing."),
    ];
    pick(&options, memory, named)
}

/// Python: bot/memory/repeat.py:3186 `_the_stems(word)`
///
/// The base verbs an -ed form could have come from ("topped" -> top, "dried" -> dry). Python
/// keeps them in a set; they are kept here in the order Python adds them.
pub fn _the_stems(word: &str) -> Vec<String> {
    let word = text::lower(word);
    let mut out: Vec<String> = Vec::new();
    let mut add = |s: String| {
        if !out.contains(&s) {
            out.push(s);
        }
    };
    let n = text::len(&word);
    if word.ends_with("ied") && n > 4 {
        add(format!("{}y", &word[..word.len() - 3]));
    }
    if word.ends_with("ed") {
        let base = &word[..word.len() - 2];
        add(base.to_string());
        add(format!("{base}e"));
        let chars: Vec<char> = base.chars().collect();
        if chars.len() > 2 && chars[chars.len() - 1] == chars[chars.len() - 2] {
            add(chars[..chars.len() - 1].iter().collect());
        }
    } else if word.ends_with('d') && n > 3 {
        add(word[..word.len() - 1].to_string());
    }
    out
}

/// Python: bot/memory/repeat.py:3203 `_the_act(word)`
///
/// The step a word names in the past tense, if it is one an instruction can open on.
pub fn _the_act(word: &str) -> Option<String> {
    if let Some(base) = IRREGULAR_PAST.get(word).filter(|b| !b.is_empty()) {
        if base == "do" || IMPERATIVES.contains(base) {
            return Some(base.clone());
        }
    }
    _the_stems(word).into_iter().find(|stem| IMPERATIVES.contains(stem))
}

/// Python: bot/memory/repeat.py:3227 `_the_object(clean, at)`
///
/// The thing a past tense was done to, as words to find in the answer; None if it says none.
pub fn _the_object(clean: &str, at: usize) -> Option<HashSet<String>> {
    let tail_text = text::lower(&clean[at..]).replace(',', " ");
    let mut tail: Vec<String> = Vec::new();
    for word in text::split_ref(&tail_text) {
        let bare = text::strip_chars(word, " .;:!?()[]\"");
        if bare.is_empty() || OBJECT_END.contains(bare) {
            break;
        }
        if STANDS_IN.contains(bare) {
            return None;
        }
        tail.push(bare.to_string());
    }
    let said: HashSet<String> = tail.into_iter().filter(|w| text::len(w) > 2 && !STOPWORDS.contains(w)).collect();
    if said.is_empty() { None } else { Some(said) }
}

/// Python: bot/memory/repeat.py:3247 `_the_steps_done(text)`
///
/// The steps a line reports having done: the act, and the thing it was done to if it names one.
pub fn _the_steps_done(text_: &str) -> Vec<(String, Option<HashSet<String>>)> {
    let mut acts: Vec<(String, Option<HashSet<String>>)> = Vec::new();
    for found in DONE.finditer(text_) {
        let low = text::lower(&text_[..found.start()]);
        let before = text::split_ref(&low);
        let before = &before[before.len().saturating_sub(3)..];
        if before.iter().any(|w| NO_LONGER_TO_DO.contains(text::strip_chars(w, " ,.;:!?'"))) {
            continue; // "i havent cleaned the filter" is the step still to do
        }
        if let Some(act) = _the_act(&text::lower(found.name_str("verb"))) {
            let entry = (act, _the_object(text_, found.end()));
            if !acts.contains(&entry) {
                acts.push(entry);
            }
        }
    }
    acts
}

/// Python: bot/memory/repeat.py:3261 `_the_answer_gives(said, act)`
///
/// Whether the answer just given is the one that named the step.
pub fn _the_answer_gives(said: &str, act: &str) -> bool {
    let low = text::lower(said).replace('\'', "");
    for word in text::split_ref(&low) {
        let word = text::strip_chars(word, " ,.;:!?()[]\"");
        if word == act || _the_stems(word).iter().any(|s| s == act) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/repeat.py:3270 `_a_step_done(text, said)`
///
/// True when the line reports doing a step of the answer just given.
pub fn _a_step_done(text_: &str, said: &str) -> bool {
    let acts = _the_steps_done(text_);
    if acts.is_empty() || said.is_empty() {
        return false;
    }
    let given = _content(said);
    for (act, thing) in acts {
        // "i did the hot wash" names its step by the thing done.
        let by_object = act == "do";
        if !by_object && !_the_answer_gives(said, &act) {
            continue;
        }
        match thing {
            None => {
                if !by_object {
                    return true;
                }
            }
            Some(thing) => {
                if thing.intersection(&given).next().is_some() {
                    return true;
                }
            }
        }
    }
    false
}

/// Python: bot/memory/repeat.py:3303 `_the_step_just_given(text, memory)`
///
/// True when the line reports doing a step of the answer one turn back.
pub fn _the_step_just_given(text_: &str, memory: &Memory) -> bool {
    if let Some((back, _question, said)) = recent_answers(memory).into_iter().next() {
        return back == 1 && _a_step_done(text_, &said);
    }
    false
}

/// Python: bot/memory/repeat.py:3317 `carried_on(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`carried_on_text`].
pub fn carried_on(text_: &str, memory: &mut Memory) {
    let _ = carried_on_text(text_, memory);
}

/// The reply to a line that picks up a detail of the answer just given, or None (`carried_on`).
pub fn carried_on_text(text_: &str, memory: &mut Memory) -> Option<String> {
    // A reset lead is taken off only when what follows it is a question of its own.
    let mut clean = text::strip(text_).to_string();
    let asked_after = without_a_reset(&clean);
    if QUESTION.match_(&asked_after).is_some() || ASKS.match_(&asked_after).is_some() {
        clean = asked_after;
    }
    if clean.is_empty()
        || clean.ends_with('?')
        || QUESTION.match_(&clean).is_some()
        || ASKS.match_(&clean).is_some()
        || n_words(&clean) > 20
    {
        return None;
    }
    if ABOUT_THE_BOT.is_match(&clean) || WORDS_WANTED.is_match(&clean) {
        return None;
    }
    if ASKING_TO_START.is_match(&clean) {
        return None; // a stall about somebody else asks for the first move
    }
    if ACK_OPENS.match_(&clean).is_some() {
        return None; // a thank-you ends the turn
    }
    if names_a_limit(&clean) {
        return None; // a limit is a fact about a person, not a detail of the answer
    }
    if !QUESTION_MORE.is_match(&clean) && (_tried(&clean) || _the_step_just_given(&clean, memory)) {
        return _the_untried(&clean, memory);
    }
    let routine = memory.understood.as_ref().is_some_and(|u| u.routine);
    if routine && !REJOINDER.is_match(&clean) {
        // A routine the person reports names its own subject: the habit tail answers it.
        return None;
    }
    let asked = _content(&clean);
    if asked.len() < 2 {
        return None;
    }
    let which_part = asks_for_the_move(text_);
    if let Some(last) = turns(memory).last() {
        if !which_part && REFER_BACK.is_match(&last.bot) {
            return None;
        }
    }
    for (back, question, said) in recent_answers(memory) {
        if back > 1 && !_about_the_same_thing(&question, &said, memory) {
            continue;
        }
        let mut best = String::new();
        let mut shared = 0;
        for part in SENTENCE.split(&said) {
            let part = text::strip(&part);
            if part.is_empty() || RED_FLAG.match_(part).is_some() || ASKING_THEM.is_match(part) {
                continue;
            }
            let count = asked.intersection(&_content(part)).count();
            if count > shared {
                best = part.to_string();
                shared = count;
            }
        }
        if shared < 2 {
            // The line names something the answer does not put in words: the doing part again.
            best = if back == 1
                && _trouble(&clean)
                && !_a_feeling(&clean)
                && !_a_thing_of_its_own(&clean, &said)
                && !SOMEONE_ELSES_LIMIT.is_match(&clean)
            {
                _the_advice(&said)
            } else {
                String::new()
            };
            if best.is_empty() {
                continue;
            }
        } else if shared >= asked.len() && _says_they_agree(&clean) {
            // The person has just said the answer back, as people agree: confirm it.
            let named = _the_thing_they_named(&clean, &best);
            if !named.is_empty() {
                return Some(_agreed(&named, memory));
            }
        }
        let doing = _the_doing(&best);
        if text::len(&doing) < 25 || _already_repeated(&doing, memory) {
            continue;
        }
        return Some(_handed_back(memory, &doing));
    }
    None
}

/// Python: bot/memory/repeat.py:3495 `_starting_out_names_a_thing(line)`
///
/// True when what is started out *into* is in the line: "start out in gardening".
pub fn _starting_out_names_a_thing(line: &str) -> bool {
    let Some(found) = STARTING_OUT.search(line) else { return false };
    let first = found.str(1);
    let second = found.str(2);
    if STARTING_INTO.contains(first) || STARTING_AT.contains(first) {
        return !second.is_empty() && !STARTING_BACK.contains(second);
    }
    !STARTING_BACK.contains(first)
}

/// Python: bot/memory/repeat.py:3506 `says_where_to_start(text)`
///
/// START_ASK for a line that asks which part of the answer to do first, and names no starting.
pub fn says_where_to_start(text_: &str) -> bool {
    if !START_ASK.is_match(text_) {
        return false;
    }
    if _starting_out_names_a_thing(text_) {
        return false;
    }
    let Some(found) = STARTING_WHAT.search(text_) else {
        // A start ask leans on the answer in hand only when the line names nothing of its own.
        return _names_a_thing_of_its_own(text_).len() < 2;
    };
    let first = found.str(1);
    let second = found.str(2);
    if STARTING_AT.contains(first) {
        return second.is_empty() || STARTING_BACK.contains(second);
    }
    STARTING_BACK.contains(first)
}

/// Python: bot/memory/repeat.py:3563 `a_more_ask(text)`
///
/// Whether the line asks for the next part of the answer in hand, said either way.
pub fn a_more_ask(text_: &str) -> bool {
    MORE_ASK.is_match(text_) || IDEAS_ONLY.match_(text_).is_some()
}

/// Python: bot/memory/repeat.py:3588 `_places_to_cut(head)`
///
/// Where a long instruction could end (byte offsets): at a qualifier, or at a bracket that
/// explains it.
pub fn _places_to_cut(head: &str) -> Vec<usize> {
    let mut spots: Vec<usize> = Vec::new();
    for found in QUALIFIER.finditer(head) {
        let after = text::split_ref(text::strip(&head[found.end()..]));
        let Some(first) = after.first() else { continue };
        if OPENERS.contains(&text::lower(first)) {
            continue;
        }
        spots.push(found.start());
    }
    for (index, c) in head.char_indices() {
        if c == '(' && head[index..].contains(')') {
            spots.push(index);
        }
    }
    spots.sort();
    spots.dedup();
    spots
}

/// Python: bot/memory/repeat.py:3609 `_inside_a_bracket(text, at)`
///
/// Whether the character at `at` (a byte offset) sits inside a bracket not yet closed.
pub fn _inside_a_bracket(text_: &str, at: usize) -> bool {
    let before = &text_[..at.min(text_.len())];
    text::count(before, "(") > text::count(before, ")")
}

/// Python: bot/memory/repeat.py:3633 `_a_list_item(part)`
///
/// The first item of a list the answer gives after a colon, when no item opens on a verb.
pub fn _a_list_item(part: &str) -> String {
    if !part.contains(':') {
        return String::new();
    }
    let lead = part.split(':').next().unwrap_or("");
    if !LIST_LEAD.is_match(lead) {
        return String::new(); // a set of things the answer names, not the things to do
    }
    let items: Vec<String> = _pieces(part)
        .iter()
        .skip(1)
        .map(|p| text::strip_chars(p, " .:;").to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if items.len() < 2 {
        return String::new();
    }
    let first = &items[0];
    let words = text::split_ref(first);
    if !(3..=8).contains(&words.len()) {
        return String::new();
    }
    if NOT_A_DOING.is_match(first) || RED_FLAG.match_(first).is_some() || NOT_A_STEP.match_(first).is_some() {
        return String::new();
    }
    if words.iter().any(|w| VERBS.contains(&text::lower(text::strip_chars(w, ".,!?;:'\"")))) {
        return String::new();
    }
    first.clone()
}

/// Slice `part[start:end]` as Python does (empty when start passes end).
fn py_slice(part: &str, start: usize, end: usize) -> &str {
    let end = end.min(part.len());
    if start >= end { "" } else { &part[start..end] }
}

/// Python: bot/memory/repeat.py:3670 `_pieces(part)`
///
/// The instructions of one sentence, split at its colons and commas (never inside a bracket).
pub fn _pieces(part: &str) -> Vec<String> {
    let marks: Vec<Match> = PIECE_END.finditer(part).into_iter().filter(|f| !_inside_a_bracket(part, f.start())).collect();
    let mut out: Vec<String> = Vec::new();
    let mut start = 0usize;
    let mut listed = false;
    for (index, found) in marks.iter().enumerate() {
        let mark = text::strip(found.as_str());
        if mark == "," && !listed {
            // A comma only splits an instruction when what follows it is one of its own.
            let end = if index + 1 < marks.len() { marks[index + 1].start() } else { part.len() };
            let after = text::strip_chars(py_slice(part, found.end(), end), " ,;:.");
            let after = RE_CONNECTOR_LEAD.sub("", after);
            if !_opens_on_a_verb(&after) {
                continue;
            }
        }
        if mark == ":" && !listed {
            let (keeps, at) = _the_colon_tail_kept(part, &marks, index, start, found);
            match keeps {
                None => {
                    // The head only says what the tail is: a lead-in and its list.
                    start = part.len();
                    listed = true;
                    continue;
                }
                Some(keeps) if !keeps.is_empty() => {
                    // The colon does not end the instruction: the step is the whole of it.
                    let Some(at) = at else { continue };
                    out.push(keeps);
                    start = at;
                    listed = true;
                    continue;
                }
                Some(_) => {}
            }
        }
        out.push(py_slice(part, start, found.start()).to_string());
        start = found.end();
        listed = listed || mark == ":";
    }
    out.push(py_slice(part, start, part.len()).to_string());
    out
}

/// Python: bot/memory/repeat.py:3722 `_the_tail_asks_a_question(part, marks, index, found)`
///
/// True when what follows the colon is the questions to put, not a list of things to try.
pub fn _the_tail_asks_a_question(part: &str, marks: &[Match], index: usize, found: &Match) -> bool {
    let end = if index + 1 < marks.len() { marks[index + 1].start() } else { part.len() };
    A_QUESTION.match_(text::strip_chars(py_slice(part, found.end(), end), " ,;.")).is_some()
}

/// Python: bot/memory/repeat.py:3728 `_the_colon_tail_kept(part, marks, index, start, found)`
///
/// The instruction with its colon tail, when the tail belongs to it rather than to a list:
/// (the piece, where the next piece starts). The piece is Some("") when the colon splits as
/// before and None when the head only announces a list of things.
pub fn _the_colon_tail_kept(part: &str, marks: &[Match], index: usize, start: usize, found: &Match) -> (Option<String>, Option<usize>) {
    let head = text::strip_chars(py_slice(part, start, found.start()), " ,;").to_string();
    if !_opens_on_a_verb(&head) {
        return (Some(String::new()), None);
    }
    if NOT_A_TAIL.is_match(&head) && !_the_tail_asks_a_question(part, marks, index, found) {
        return (None, None);
    }
    let tail = &part[found.end()..];
    if n_words(&format!("{}: {}", head, tail)) <= *STEP_WITH_ITS_METHOD {
        return (Some(format!("{}: {}", head, text::rstrip_chars(tail, " ,;"))), None);
    }
    // The step is the instruction and the first item of the tail.
    for later in &marks[index + 1..] {
        let mark = text::strip(later.as_str());
        if mark != "," && mark != ";" {
            continue;
        }
        let first = text::strip_chars(py_slice(part, found.end(), later.start()), " ,;");
        if first.is_empty() {
            continue;
        }
        let piece = format!("{}: {}", head, first);
        if n_words(&piece) <= *STEP_WITH_ITS_METHOD {
            return (Some(piece), Some(later.end()));
        }
        return (Some(String::new()), None);
    }
    (Some(String::new()), None)
}

/// Python: bot/memory/repeat.py:3773 `_opens_on_a_verb(text)`
///
/// Whether this clause starts on a verb, which is what makes it an instruction of its own.
pub fn _opens_on_a_verb(text_: &str) -> bool {
    let words = text::split_ref(text::strip_chars(text_, " .,:;"));
    match words.first() {
        Some(first) => {
            let low = text::lower(first);
            IMPERATIVES.contains(&low) || MOVE_VERBS.contains(&low)
        }
        None => false,
    }
}

/// Python: bot/memory/repeat.py:3799 `_the_steps(said)`
///
/// CONTRACT: Python's `_the_steps` is a generator, typed `Option<String>` in the contract; this
/// gives its first step, and [`the_steps`] gives them all.
pub fn _the_steps(said: &str) -> Option<String> {
    the_steps(said).into_iter().next()
}

/// Every instruction in the answer, in the order the answer gives them (`_the_steps`).
pub fn the_steps(said: &str) -> Vec<String> {
    let verbs = &*VERBS;
    let mut steps: Vec<String> = Vec::new();
    for part in SENTENCE.split(text::strip(said)) {
        let part = text::strip(&part);
        if part.is_empty() || RED_FLAG.match_(part).is_some() {
            continue;
        }
        // What to do about a pattern, not the first thing to do now.
        if CONDITION_CLAUSE.match_(part).is_some() {
            continue;
        }
        let mut found: Vec<String> = Vec::new();
        for piece in _pieces(part) {
            let head = text::strip_chars(&piece, " .:;");
            let mut head = RE_CONNECTOR_LEAD.sub("", head);
            if head.is_empty() || RED_FLAG.match_(&head).is_some() || NOT_A_STEP.match_(&head).is_some() {
                continue;
            }
            if A_MENU.is_match(&head) {
                continue;
            }
            if DECLARATIVE.match_(&head).is_some() {
                continue; // a statement about the thing, not something to do
            }
            if head.ends_with('?') || QUESTIONS_THAT_ARE_NOT_STEPS.match_(&head).is_some() {
                continue;
            }
            let mut words: Vec<String> = text::split(&head);
            let (limit, cut_words) =
                if head.contains(':') { (*STEP_WITH_ITS_METHOD, *STEP_WITH_ITS_METHOD) } else { (*STEP_WORDS, *CUT_WORDS) };
            if words.len() > cut_words && !NEEDS_COMPLEMENT.contains(&text::lower(&words[0])) {
                // The verb phrase is the step and the qualifier after it is detail.
                let mut cut: Option<usize> = None;
                for at in _places_to_cut(&head) {
                    if _inside_a_bracket(&head, at) {
                        continue;
                    }
                    let left = text::split(text::strip_chars(&head[..at], " ,;:."));
                    if left.len() < *SHORT_STEP {
                        continue;
                    }
                    if DANGLING.contains(&text::lower(&left[left.len() - 1])) {
                        continue;
                    }
                    cut = Some(at);
                    break;
                }
                if let Some(cut) = cut.filter(|&c| c > 0) {
                    head = text::strip_chars(&head[..cut], " ,;:.").to_string();
                    words = text::split(&head);
                }
            }
            if !(3..=limit).contains(&words.len()) {
                // A piece too long to hand over whole still opens on one instruction.
                let Some(first_half) = _the_first_instruction_in_a_long_piece(&head, verbs) else { continue };
                head = first_half;
                words = text::split(&head);
            }
            if !verbs.contains(&text::lower(&words[0])) && !_a_verb_after_a_leading_phrase(&head, verbs) {
                continue;
            }
            // "The thing to do first: fix the wake-up time first" says first twice.
            let trimmed = RE_FIRST_OR_NEXT_END.sub("", &head);
            if let Some(last) = text::split_ref(&trimmed).last() {
                if !DANGLING.contains(&text::lower(last)) {
                    head = trimmed.clone();
                }
            }
            found.push(lower_first(&head));
        }
        let none_found = found.is_empty();
        steps.extend(found);
        if none_found {
            let item = _a_list_item(part);
            if !item.is_empty() {
                steps.push(lower_first(&item));
            }
        }
    }
    steps
}

/// Python: bot/memory/repeat.py:3903 `_the_next_step(said, memory)`
///
/// One thing out of the answer past the first, which they have not already had or tried.
pub fn _the_next_step(said: &str, memory: &Memory) -> Option<String> {
    let steps = the_steps(said);
    for step in steps.iter().skip(1) {
        if RED_FLAG.match_(step).is_some() {
            continue; // a warning is not one more thing to do
        }
        if _already_repeated(step, memory) || _they_say_they_tried(step, memory) {
            continue;
        }
        return Some(step.clone());
    }
    None
}

/// Python: bot/memory/repeat.py:3927 `_the_first_step(said)`
///
/// The first thing to do in the answer, read more generously than `_a_move`.
pub fn _the_first_step(said: &str) -> Option<String> {
    the_steps(said).into_iter().next()
}

/// Python: bot/memory/repeat.py:3951 `_a_new_work(text, asked)`
///
/// True when a how-to question follows a definition of the same thing.
pub fn _a_new_work(text_: &str, asked: &str) -> bool {
    if asks_to_put_it_right(text_) || HOW_IT.is_match(text_) {
        return false;
    }
    if WHAT_IT_IS.match_(asked).is_none() {
        return false;
    }
    !_doings(text_).is_empty()
}

/// Python: bot/memory/repeat.py:3970 `asks_which_part(text)`
///
/// Whether the line asks which part of the answer to do first, and names no doing of its own.
pub fn asks_which_part(text_: &str) -> bool {
    if text::strip(text_).is_empty() || WORDING.is_match(text_) {
        return false;
    }
    asks_for_the_move(text_) && !HOW_IT.is_match(text_)
}

/// Python: bot/memory/repeat.py:3985 `_leads_for(text)`
///
/// The lead-ins that answer this ask: the thing to do, or the thing to change.
pub fn _leads_for(text_: &str) -> Vec<String> {
    if CHANGE_ASK.is_match(text_) { CHANGE_LEADS.clone() } else { MOVE_LEADS.clone() }
}

/// Python: bot/memory/repeat.py:3990 `_a_lead_for(leads, step, memory, text)`
///
/// One lead out of `leads`, and never one that says what the step's first word says.
pub fn _a_lead_for(leads: &[String], step: &str, memory: &Memory, text_: &str) -> String {
    let mut leads: Vec<String> = leads.to_vec();
    if STEP_SAYS_FIRST.is_match(text::strip(step)) {
        leads.retain(|lead| !LEAD_SAYS_FIRST.is_match(&RE_PLACEHOLDER.sub(" ", lead)));
        if leads.is_empty() {
            leads = NOW_LEADS.clone();
        }
    }
    pick(&leads, memory, text_)
}

/// Python: bot/memory/repeat.py:4007 `the_move(text, said, memory)`
///
/// The first step of the answer just given, when the line is asking which part to do.
pub fn the_move(text_: &str, said: &str, memory: &mut Memory) -> Option<String> {
    if text::strip(said).is_empty() {
        return None;
    }
    if !asks_for_the_move(text_) {
        return None;
    }
    if WORDING.is_match(text_) {
        return None;
    }
    if RE_HOW_A_FIGURE.is_match(text_) {
        return None; // a figure is asked for, not the first step
    }
    let step = _the_first_step(said)?;
    if step.is_empty() || _already_repeated(&step, memory) {
        return None;
    }
    if _they_say_they_tried(&step, memory) {
        return None; // the tried pools have the next step
    }
    let wording = upper_first(&step);
    let lead = _a_lead_for(&_leads_for(text_), &wording, memory, text_);
    Some(_handed_back(memory, &_a_lead_and_a_step(&lead, &wording)))
}

/// Python: bot/memory/repeat.py:4038 `_they_say_they_tried(step, memory)`
///
/// True when a line just before this one reports having done this very thing.
pub fn _they_say_they_tried(step: &str, memory: &Memory) -> bool {
    let mine: HashSet<String> = az_words(&text::lower(step)).into_iter().filter(|w| text::len(w) > 2).collect();
    if mine.is_empty() {
        return false;
    }
    for turn in last_turns(memory, 3) {
        let line = text::lower(&turn.user);
        if !ATTEMPT.is_match(&line) {
            continue;
        }
        let said: HashSet<String> = az_words(&line).into_iter().filter(|w| text::len(w) > 2).collect();
        let shared: Vec<&String> = mine.intersection(&said).collect();
        if shared.len() >= 2 || shared.iter().any(|w| text::len(w) >= 5) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/repeat.py:4120 `the_figure_asked(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`the_figure_asked_text`].
pub fn the_figure_asked(text_: &str, memory: &Memory) {
    let _ = the_figure_asked_text(text_, memory);
}

/// The sentence of the last answer that carries the figure this line asks for, or None.
pub fn the_figure_asked_text(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text::lower(text_));
    if clean.is_empty() {
        return None;
    }
    let (ask, figure) = FIGURE_ASKS.iter().find(|(_, ask, _)| ask.is_match(&clean)).map(|(_, a, f)| (*a, *f))?;
    let rest = az_words(&ask.sub(" ", &clean)).into_iter().any(|w| !FIGURE_WORDS.contains(&w) && !FIGURE_SAYING.contains(&w));
    if rest {
        return None; // the line names a thing of its own: that thing has an answer
    }
    for (back, turn) in last_turns(memory, *LOOK_BACK).iter().rev().enumerate() {
        let back = back + 1;
        if module_of(turn) != "answers" {
            continue;
        }
        let said = text::strip(&turn.bot);
        if said.is_empty() || HEDGE.is_match(said) || text::rstrip(said).ends_with('?') {
            continue;
        }
        if text::len(said) < 50 && !figure.is_match(said) {
            continue; // a stock line is not an answer with a figure in it
        }
        if back > 1 && !_about_the_same_thing(&turn.user, said, memory) {
            continue; // the chat has moved on
        }
        for part in _sentences(said) {
            let part = text::rstrip_chars(text::strip(&part), " ;").to_string();
            if !part.is_empty() && figure.is_match(&part) && !_already_repeated(&part, memory) {
                let end = if part.ends_with(['.', '!', '?']) { "" } else { "." };
                return Some(format!("{}{}", upper_first(&part), end));
            }
        }
    }
    None
}

/// Python: bot/memory/repeat.py:4228 `_the_keys_of(said, memory)`
///
/// The words the entry behind an answer was keyed on, or an empty set.
pub fn _the_keys_of(said: &str, memory: &Memory) -> HashSet<String> {
    let head = first_chars(&text::squash(said), 60).to_string();
    let Some(records) = memory.get("entry_answers") else { return HashSet::new() };
    for record in records.items() {
        if !record.is_dict() {
            continue;
        }
        let said_there = text::squash(&py_str_or_empty(record.get("text")));
        if !said_there.is_empty() && first_chars(&said_there, 60) == head {
            return match record.get("keys") {
                Some(keys) if keys.truthy() => keys.items().iter().map(|key| text::lower(&py_str(key))).collect(),
                _ => HashSet::new(),
            };
        }
    }
    HashSet::new()
}

/// Python: bot/memory/repeat.py:4255 `answers_nothing(reply)`
///
/// True when the reply is one of the bot's own ways of not answering this turn.
pub fn answers_nothing(reply: &str) -> bool {
    if reply.is_empty() {
        return false;
    }
    use crate::compose::fallback;
    if fallback::_is_a_hedge(reply) || fallback::_nothing_to_stand_behind(reply) {
        return true;
    }
    static POOLS: LazyLock<Value> = LazyLock::new(|| {
        Value::Tuple(std::sync::Arc::new(
            [
                "bot.compose.fallback.SAME_AGAIN",
                "bot.compose.fallback.HELP_ALREADY",
                "bot.compose.fallback.TIPS_LINES",
                "bot.compose.fallback.TIPS_THING",
                "bot.compose.fallback.TRIED_NO_CHANGE_ELSEWHERE",
                "bot.compose.verdict.ALL_OF_IT_LINES",
            ]
            .iter()
            .map(|name| data::get(name).clone())
            .collect(),
        ))
    });
    fallback::_in_a_pool(reply, &POOLS)
}

/// Python: bot/memory/repeat.py:4280 `the_advice_was_to_get_seen(memory)`
///
/// True when the bot has already told them to be seen today - 111, A&E, a GP, a vet.
pub fn the_advice_was_to_get_seen(memory: &Memory) -> bool {
    crate::memory::stuck::the_advice_was_to_get_seen(memory)
}

/// Python: bot/memory/repeat.py:4290 `the_part_asked(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`the_part_asked_text`].
pub fn the_part_asked(text_: &str, memory: &Memory) {
    let _ = the_part_asked_text(text_, memory);
}

/// The sentence of a card in hand that answers this follow-up, or None (`the_part_asked`).
pub fn the_part_asked_text(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text::lower(text_));
    if clean.is_empty() {
        return None;
    }
    let (name, mark) =
        PART_ASKS.iter().find(|(_, ask, _)| ask.is_match(&clean)).map(|(name, _, mark)| (name.as_str(), *mark))?;
    if name == "move" && _whether_to(&clean).is_empty() {
        return None; // a decision about a person is the composer's to answer
    }
    let mut asked_words = topics::words(&clean);
    if name == "move" {
        // The verb the ask is made with names what is being asked about.
        for (_name2, ask2, mark2) in PART_ASKS.iter() {
            if mark2.is_none() {
                continue;
            }
            let Some(found) = ask2.search(&clean) else { continue };
            let frame = az_words(found.as_str());
            asked_words.retain(|w| !frame.contains(w));
            break;
        }
    }
    if name == "move" {
        let step = _whether_to(&clean);
        let after: Vec<&str> = text::split_ref(&step).into_iter().skip(2).filter(|w| !PART_STAND_INS.contains(*w)).collect();
        if after.len() <= 1 {
            return None; // the step names no thing: it is about the person
        }
    }
    let asked: Vec<String> = asked_words
        .into_iter()
        .filter(|w| {
            text::len(w) > 2
                && !is_digit(w)
                && !STOPWORDS.contains(w)
                && !ASK.contains(w)
                && !ASK_VERB.contains(w)
                && !TIME.contains(w)
                && !FILLER.contains(w)
                && !SELF.contains(w)
                && !PART_STAND_INS.contains(w)
        })
        .collect();
    // A yes/no about still being able to do something is settled only when the card says that doing.
    let mut doing = String::new();
    if let Some(still) = STILL_ASKS.match_(&clean) {
        if let Some(first) = az_words(still.name_str("rest")).first() {
            doing = _a_stem(first);
        }
    }
    let mut seen_a_card = false;
    for (back, turn) in last_turns(memory, *LOOK_BACK).iter().rev().enumerate() {
        let back = back + 1;
        if module_of(turn) != "answers" {
            continue;
        }
        let said = text::strip(&turn.bot);
        if said.is_empty() || HEDGE.is_match(said) || text::rstrip(said).ends_with('?') {
            continue;
        }
        if text::len(said) < 50 {
            continue; // a stock line, not a card
        }
        let keys = _the_keys_of(said, memory);
        let on_the_card = !asked.is_empty() && asked.iter().any(|w| keys.contains(w));
        if asked.is_empty() && seen_a_card {
            continue; // the newest card is the one a line that names nothing leans on
        }
        seen_a_card = true;
        if !asked.is_empty() && back > 1 && !on_the_card && !_about_the_same_thing(&turn.user, said, memory) {
            continue;
        }
        if asked.is_empty() && back > 1 && !_the_chat_is_still_on(said, memory) {
            continue;
        }
        for (index, part) in _sentences(said).iter().enumerate() {
            let part = text::rstrip_chars(text::strip(part), " ;").to_string();
            if n_words(&part) < 4 {
                continue;
            }
            if text::lower(&part).ends_with('?') {
                continue;
            }
            if NOT_ALONE.match_(&part).is_some() {
                continue; // a sentence that only follows the one before it is not a reply
            }
            if name == "start" {
                if index > 0 {
                    break;
                }
            } else {
                let mark = mark.expect("a part ask other than the start has a mark");
                if !mark.is_match(&part) {
                    continue;
                }
                if CONDITION_CLAUSE.match_(&part).is_some() || NOT_A_MOVE.match_(&part).is_some() {
                    continue;
                }
            }
            let low = text::lower(&part);
            if !asked.is_empty() && !on_the_card && !asked.iter().any(|w| low.contains(w.as_str())) {
                continue;
            }
            if !doing.is_empty() && !az_words(&low).iter().any(|w| _a_stem(w) == doing) {
                continue; // the card never says this doing
            }
            if _already_repeated(&part, memory) {
                continue;
            }
            let end = if part.ends_with(['.', '!', '?']) { "" } else { "." };
            return Some(format!("{}{}", upper_first(&part), end));
        }
    }
    None
}

/// Python: bot/memory/repeat.py:4534 `_whether_to(clean)`
///
/// The step the line asks about, as "whether to <it>", or "".
pub fn _whether_to(clean: &str) -> String {
    for shape in WHETHER_ASKS.iter() {
        let Some(found) = shape.search(clean) else { continue };
        let rest = text::strip_chars(&text::squash(found.name_str("rest")), " .,").to_string();
        let words = text::split_ref(&rest);
        if words.is_empty() || words.len() > 5 {
            continue;
        }
        if !A_MOVE_WORDS.contains(words[0]) {
            continue;
        }
        return format!("whether to {rest}");
    }
    for shape in METHOD_ASKS.iter() {
        let Some(found) = shape.search(clean) else { continue };
        let rest = text::strip_chars(&text::squash(found.name_str("rest")), " .,").to_string();
        let words = text::split_ref(&rest);
        if words.is_empty() || words.len() > 5 {
            continue;
        }
        if NOT_A_METHOD.contains(words[0]) {
            continue;
        }
        return format!("whether to use {rest}");
    }
    // A yes/no about still being able to use the thing on the screen.
    if let Some(found) = STILL_ASKS.match_(clean) {
        return format!(
            "whether {} {} still {}",
            text::lower(found.name_str("who")),
            text::lower(found.name_str("aux")),
            text::strip_chars(&text::squash(found.name_str("rest")), " .,")
        );
    }
    String::new()
}

/// Python: bot/memory/repeat.py:4576 `_a_stem(word)`
///
/// "preheat" and "preheating" are one word said in two forms.
pub fn _a_stem(word: &str) -> String {
    let mut word = text::lower(word);
    for suffix in ["ing", "ings", "ed", "es", "s"] {
        if word.ends_with(suffix) && text::len(&word) >= suffix.len() + 4 {
            word.truncate(word.len() - suffix.len());
            break;
        }
    }
    if word.ends_with('e') && text::len(&word) > 4 {
        word.pop();
    }
    word
}

/// Python: bot/memory/repeat.py:4603 `_the_chat_is_still_on(said, memory)`
///
/// True when the card is the one the chat is still on, not just one it passed through.
pub fn _the_chat_is_still_on(said: &str, memory: &Memory) -> bool {
    let mut words: HashSet<String> = az_words(&text::lower(said)).into_iter().collect();
    words.extend(_the_keys_of(said, memory).into_iter().filter(|w| text::len(w) > 2));
    let keep = |w: &String| text::len(w) > 2 && !STOPWORDS.contains(w) && !NOT_A_SUBJECT.contains(w);
    let words: HashSet<String> = words.into_iter().filter(keep).collect();
    let Some(last) = turns(memory).last() else { return false };
    for side in [&last.bot, &last.user] {
        let here: HashSet<String> = az_words(&text::lower(side)).into_iter().filter(keep).collect();
        if words.intersection(&here).next().is_some() {
            return true;
        }
    }
    false
}

/// The topic the understanding engine reads for a line, or "" when the reading fails.
fn topic_of(line: &str) -> String {
    py_try(|| crate::understanding::understand(line, None).topic.unwrap_or_default()).unwrap_or_default()
}

/// Python: bot/memory/repeat.py:4632 `_where_the_rest_is(said, memory, turn, ask='')`
///
/// The one place that settles what an answer in hand does not cover, read off the answer.
pub fn _where_the_rest_is(said: &str, memory: &Memory, turn: &Turn, ask: &str) -> String {
    let card = az_words(&text::lower(said)).join(" ");
    let mut keys: Vec<String> = _the_keys_of(said, memory).into_iter().collect();
    keys.sort();
    let card_words = text::strip(&format!("{} {}", card, keys.join(" "))).to_string();
    for (needles, place) in WHERE_TO_LOOK_WORDS.iter() {
        if needles.iter().any(|word| re_cached(&format!(r"\b{word}\b"), 0).is_match(&card_words)) {
            return place.clone();
        }
    }
    // The topic of the turn that fetched the card is the last resort, and only on the same topic.
    let card_topic = topic_of(&turn.user);
    if !ask.is_empty() {
        let ask_topic = topic_of(ask);
        if !card_topic.is_empty() && !ask_topic.is_empty() && card_topic != ask_topic {
            return WHERE_TO_LOOK_DEFAULT.clone();
        }
    }
    WHERE_TO_LOOK.get(&card_topic).cloned().unwrap_or_else(|| WHERE_TO_LOOK_DEFAULT.clone())
}

/// Python: bot/memory/repeat.py:4673 `what_it_knows(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`what_it_knows_text`].
pub fn what_it_knows(text_: &str, memory: &Memory) {
    let _ = what_it_knows_text(text_, memory);
}

/// What the answer in hand does know, when the follow-up asks something it never covered.
pub fn what_it_knows_text(text_: &str, memory: &Memory) -> Option<String> {
    let squashed = text::squash(&text::lower(text_));
    let clean = text::strip_chars(&squashed, " ?!.").to_string();
    if clean.is_empty() {
        return None;
    }
    let whether = _whether_to(&clean);
    if whether.is_empty() {
        return None;
    }
    // The step, not the frame of the ask.
    let step_words = text::split_ref(&whether).into_iter().skip(2).collect::<Vec<_>>().join(" ");
    let asked: Vec<String> = topics::words(&step_words)
        .into_iter()
        .filter(|w| {
            text::len(w) > 2
                && !is_digit(w)
                && !STOPWORDS.contains(w)
                && !ASK.contains(w)
                && !ASK_VERB.contains(w)
                && !TIME.contains(w)
                && !FILLER.contains(w)
                && !SELF.contains(w)
                && !PART_STAND_INS.contains(w)
        })
        .collect();
    if asked.is_empty() {
        return None;
    }
    let asked_stems: HashSet<String> = asked.iter().map(|w| _a_stem(w)).collect();
    let mut gap = String::new();
    let window = last_turns(memory, *LOOK_BACK);
    for (back, turn) in window.iter().rev().enumerate() {
        let back = back + 1;
        if module_of(turn) != "answers" {
            continue;
        }
        let said = text::strip(&turn.bot);
        if said.is_empty() || HEDGE.is_match(said) || text::rstrip(said).ends_with('?') {
            continue;
        }
        if text::len(said) < 50 {
            continue; // a stock line, not a card
        }
        let keys = _the_keys_of(said, memory);
        if asked.iter().any(|w| keys.contains(w)) {
            continue; // the card does answer this: not a gap
        }
        let on_the_card: HashSet<String> = az_words(&text::lower(said)).iter().map(|w| _a_stem(w)).collect();
        let named = asked.iter().any(|w| on_the_card.contains(&_a_stem(w)));
        let missing = asked.iter().any(|w| !on_the_card.contains(&_a_stem(w)) && !A_QUALITY.contains(w));
        if named && !missing {
            // The card says the step: the sentence of it that carries the step is the answer.
            for sentence in _sentences(said) {
                let sentence = text::rstrip_chars(text::strip(&sentence), " ;").to_string();
                if n_words(&sentence) < 4 || text::lower(&sentence).ends_with('?') {
                    continue;
                }
                let stems: HashSet<String> = az_words(&text::lower(&sentence)).iter().map(|w| _a_stem(w)).collect();
                if asked_stems.intersection(&stems).next().is_some() {
                    let mut head = if sentence.contains(':') { sentence.split(':').next().unwrap().to_string() } else { sentence.clone() };
                    if n_words(&head) < 4 {
                        head = sentence.clone();
                    }
                    let end = if head.ends_with(['.', '!', '?', '"']) { "" } else { "." };
                    return Some(format!("{}{}", upper_first(&head), end));
                }
            }
            continue; // nothing of the card says it in a sentence of its own
        }
        // The newest card comes first, and an older one only when the newer cards do not name it.
        let first_card = back <= 1 || !last_turns(memory, back - 1).iter().any(|t| module_of(t) == "answers");
        if back > 1 && !_the_chat_is_still_on(said, memory) {
            continue; // the chat has moved on
        }
        if back > 1 && !first_card && !_about_the_same_thing(&turn.user, said, memory) {
            continue; // the chat has moved on
        }
        // The step has to be a step on the thing the card is about.
        let move_words: Vec<&str> = text::split_ref(&whether).into_iter().skip(2).filter(|w| !PART_STAND_INS.contains(*w)).collect();
        if move_words.len() <= 1 {
            continue; // the move has no object: it is about the person
        }
        let mut part = String::new();
        let mut lead = String::new();
        for sentence in _sentences(said) {
            let sentence = text::rstrip_chars(text::strip(&sentence), " ;").to_string();
            if n_words(&sentence) < 4 || text::lower(&sentence).ends_with('?') {
                // A short opening ("Bin it.") is kept and put in front of the detail.
                if lead.is_empty()
                    && !sentence.is_empty()
                    && !text::lower(&sentence).ends_with('?')
                    && n_words(&sentence) >= 2
                    && NOT_ALONE.match_(&sentence).is_none()
                {
                    lead = sentence;
                }
                continue;
            }
            if _already_repeated(&sentence, memory) {
                continue;
            }
            let stems: HashSet<String> = az_words(&text::lower(&sentence)).iter().map(|w| _a_stem(w)).collect();
            if asked_stems.intersection(&stems).next().is_some() {
                // The card's own sentence says the step, so there is no gap to name.
                return None;
            }
            part = sentence;
            break;
        }
        let look = _where_the_rest_is(said, memory, turn, &clean);
        if look.is_empty() {
            return None;
        }
        if part.is_empty() {
            // Held, not returned: a card further back may name the thing this one does not.
            gap = format!("On {}, {} will know more than I can - nothing I have covers it.", whether, look);
            continue;
        }
        let end = if part.ends_with(['.', '!', '?', '"']) { "" } else { "." };
        let mut head = format!("{}{}", upper_first(&part), end);
        if !lead.is_empty() {
            head = format!("{} {}", upper_first(&lead), head);
        }
        if text::len(&clean) % 2 == 1 {
            return Some(format!("{} On {}, {} will know more than I can.", head, whether, look));
        }
        return Some(format!("{} That is what I can tell you; on {}, {} will say.", head, whether, look));
    }
    if gap.is_empty() { None } else { Some(gap) }
}

/// Python: bot/memory/repeat.py:4976 `the_plain_line(line)`
///
/// The line with "were" read as "we are" where that is what it says, and otherwise as it is.
pub fn the_plain_line(line: &str) -> String {
    if A_WERE_ASKS.match_(line).is_some() || RE_WERE_OPENS.match_(line).is_none() {
        return line.to_string();
    }
    RE_WERE_LEAD.subn(r"\1we are", line, 1)
}

/// Python: bot/memory/repeat.py:5078 `how_long_its_been(text)`
///
/// True when the line asks how long the person's own thing has been like this.
pub fn how_long_its_been(text_: &str) -> bool {
    let said = text::squash(&text::lower(text_));
    match A_HOW_LONG_ITS_BEEN.search(&said) {
        Some(found) => _OWNS_THE_THING.is_match(found.as_str()),
        None => false,
    }
}

/// Python: bot/memory/repeat.py:5107 `_a_line_that_corrects(line)`
///
/// Whether a line takes a figure back, whether it opens with the word or ends with it.
pub fn _a_line_that_corrects(line: &str) -> bool {
    if A_CORRECTION.is_match(line) {
        return true;
    }
    let Some(found) = A_MARKER_AT_THE_END.search(line) else { return false };
    let head = text::strip(&line[..found.start()]);
    if head.is_empty() || n_words(head) > 8 {
        return false;
    }
    !crate::memory::facts::extract(head, false).truthy()
}

/// Python: bot/memory/repeat.py:5165 `_a_list_of_ages(line)`
///
/// Whether the line lists ages, which names the members of a group rather than its size.
pub fn _a_list_of_ages(line: &str) -> bool {
    A_LIST_OF_AGES.is_match(line)
}

/// Python: bot/memory/repeat.py:5170 `_something_gone(lines, at, words)`
///
/// Whether a later line says one of the counted things is gone, so the count is out of date.
pub fn _something_gone(lines: &[String], at: usize, words: &[String]) -> bool {
    for later in lines.iter().skip(at + 1) {
        if later.is_empty() {
            continue;
        }
        if A_GAIN.is_match(later) && A_GROUP_WORD.is_match(later) {
            return true; // the group grew
        }
        let Some(loss) = A_LOSS.search(later) else { continue };
        let said = topics::keywords(later);
        if words.iter().any(|word| said.iter().any(|other| _the_same_word(word, other))) {
            return true;
        }
        if text::lower(loss.as_str()).contains("one") {
            return true; // a count that went down
        }
    }
    false
}

/// Python: bot/memory/repeat.py:5217 `_the_ask_gave_that_figure(number, clean)`
///
/// Whether the ask itself names the figure the reading found.
pub fn _the_ask_gave_that_figure(number: &str, clean: &str) -> bool {
    let number = text::lower(text::strip(number));
    let Some(first) = text::split_ref(&number).first().map(|s| s.to_string()) else { return false };
    let same = |w: &str| THE_SAME_NUMBER.get(w).cloned().unwrap_or_else(|| w.to_string());
    let looked: HashSet<String> = [first.clone(), same(&first)].into_iter().collect();
    let alpha = |w: &str| w.chars().next().is_some_and(pyuni::ch_alpha);
    for other in A_NUMBER_WORD.findall_str(clean) {
        if other == first || looked.contains(&same(&other)) {
            return true;
        }
        if alpha(&other) && alpha(&first) && _the_same_word(&first, &other) {
            return true;
        }
    }
    false
}

/// Python: bot/memory/repeat.py:5236 `_a_count_of_one_with_a_plural(number, noun)`
///
/// Whether a count of one is about to be said with a noun in the plural: "one people".
pub fn _a_count_of_one_with_a_plural(number: &str, noun: &str) -> bool {
    if !["1", "one", "a", "single"].contains(&text::lower(text::strip(number)).as_str()) {
        return false;
    }
    let word = text::lower(text::split_ref(noun).last().copied().unwrap_or(""));
    if word.is_empty() {
        return false;
    }
    A_PLURAL_ONLY_NOUN.contains(&word) || word.ends_with('s')
}

/// Python: bot/memory/repeat.py:5246 `_the_thing_counted(clean, said)`
///
/// The ask's word for the thing counted, with the figure's own modifier when it has one.
pub fn _the_thing_counted(clean: &str, said: &str) -> String {
    let Some(found) = COUNT_ASK.search(clean) else { return String::new() };
    let phrase: Vec<&str> = text::split_ref(found.str(1));
    let words: Vec<&str> =
        phrase.iter().copied().filter(|w| !STOPWORDS.contains(*w) && !ASK_FRAME.contains(*w) && !FIGURE_NOUNS.contains(*w)).collect();
    if words.is_empty() {
        return String::new();
    }
    let tail = text::lower(text::split_ref(said).last().copied().expect("IndexError: list index out of range"));
    // "two flat tyres": the whole pair is the thing.
    for pair in phrase.windows(2) {
        if words.contains(&pair[1]) && _the_same_word(pair[0], &tail) {
            return format!("{} {}", pair[0], pair[1]);
        }
    }
    for word in &words {
        if _the_same_word(word, &tail) {
            return word.to_string();
        }
    }
    words[words.len() - 1].to_string()
}

/// One figure a line gives: (kind, the words used, the number, whether it counted something).
pub type Figure = (String, String, String, bool);

/// Python: bot/memory/repeat.py:5269 `_figures_in(line)`
///
/// The figures a line gives, as (kind, the words used, the number, whether it counted).
pub fn _figures_in(line: &str) -> Vec<Figure> {
    if line.is_empty() || line.contains('?') || A_QUESTION.match_(line).is_some() {
        return Vec::new();
    }
    let mut out: Vec<Figure> = Vec::new();
    for found in A_FIGURE.finditer(line) {
        let is_time = !found.name_str("time").is_empty();
        if is_time && A_RATE_OF_TIME.is_match(&line[..found.start()]) {
            continue; // a rate, not a length
        }
        if is_time && A_SCHEDULED_AFTER.match_(&line[found.end()..]).is_some() && !A_HAS_THE_LENGTH.is_match(&line[..found.start()]) {
            continue; // when to take them, not a length they have had
        }
        let mut said = text::rstrip_chars(&text::squash(found.as_str()), " ,.;:").to_string();
        if said.is_empty() {
            continue;
        }
        let counted = found.name("count").is_some();
        if counted {
            let after = text::split_ref(&said).into_iter().skip(1).collect::<Vec<_>>().join(" ");
            if A_ONE_MORE.match_(&after).is_some() {
                continue; // how many were added, not how many there are
            }
        }
        let pos = text::char_index(line, found.start()) as i64;
        let whose = py_try(|| crate::memory::facts::subject(line, pos)).and_then(|v| v.into_iter().next()).unwrap_or_default();
        if whose == "other" {
            continue;
        }
        let kind = if is_time { "time" } else { "amount" }.to_string();
        if !found.name_str("share").is_empty() {
            // Said back whole and never as a count: "You said 10 per cent."
            let number = match RE_DIGITS_LEAD.match_(found.as_str()) {
                Some(digits) => digits.str(1).to_string(),
                None => said.clone(),
            };
            out.push((kind, format!("{number} per cent"), number, false));
            continue;
        }
        if counted && text::lower(&said).ends_with(" per") {
            said = text::rstrip_chars(&said[..said.len() - 4], " ,.;:").to_string();
        }
        // The words a figure is joined to the rest of the sentence by are not part of it.
        let mut words: Vec<&str> = text::split_ref(&said);
        while words.len() > 1 && FIGURE_TAIL.contains(&text::lower(words[words.len() - 1])) {
            words.pop();
        }
        let joined = words.join(" ");
        let first = RE_FIGURE_COMMA.split(&joined).into_iter().next().unwrap_or_default();
        let said = text::rstrip_chars(&first, " ,.;:").to_string();
        let number = if counted {
            text::rstrip_chars(text::split_ref(&said).first().copied().expect("IndexError: list index out of range"), " ,.;:")
                .to_string()
        } else {
            text::rstrip_chars(&said, " ,.;:").to_string()
        };
        if number.is_empty() {
            continue;
        }
        out.push((kind, said, number, counted));
    }
    out
}

/// Python: bot/memory/repeat.py:5356 `_money_paid_for_a_thing(clean, line)`
///
/// True when the line's figure is money spent on a thing, and the ask wants the thing itself.
pub fn _money_paid_for_a_thing(clean: &str, line: &str) -> bool {
    if _AN_AMOUNT_WANTED.is_match(clean) {
        return false;
    }
    _MONEY_PAID_FOR.is_match(line)
}

/// Python: bot/memory/repeat.py:5363 `_one_figure(found, kind)`
///
/// The figure of the kind asked for, or None when the line holds none or holds two kinds.
pub fn _one_figure(found: &[Figure], kind: Option<&str>) -> Option<Figure> {
    if found.is_empty() {
        return None;
    }
    match kind.filter(|k| !k.is_empty()) {
        Some(kind) => found.iter().find(|one| one.0 == kind).cloned(),
        None => {
            let kinds: HashSet<&String> = found.iter().map(|one| &one.0).collect();
            if kinds.len() > 1 {
                return None; // a length of time and an amount on one line: which one is not known
            }
            found.first().cloned()
        }
    }
}

/// Python: bot/memory/repeat.py:5374 `_a_bare_count(figure)`
///
/// Whether a figure is the number on its own, with nothing saying what it counts.
pub fn _a_bare_count(figure: &str) -> bool {
    let words = text::split_ref(figure);
    words.len() == 1 && A_NUMBER_WORD.fullmatch(words[0]).is_some()
}

/// Python: bot/memory/repeat.py:5385 `_the_same_word(one, other)`
///
/// Whether two words are the same thing said twice: "move" and "moving", "bed" and "bedrooms".
pub fn _the_same_word(one: &str, other: &str) -> bool {
    let first = topics::stem(one);
    let second = topics::stem(other);
    if first == second {
        return true;
    }
    if topics::forms(&first).intersection(&topics::forms(&second)).next().is_some() {
        return true;
    }
    for (shorter, longer) in [(&first, &second), (&second, &first)] {
        if longer.ends_with("room") && &longer[..longer.len() - 4] == shorter.as_str() {
            return true;
        }
    }
    if PAY_FAMILY.contains(&first) && PAY_FAMILY.contains(&second) {
        return true;
    }
    text::len(&first) >= 4 && text::len(&second) >= 4 && (first.starts_with(second.as_str()) || second.starts_with(first.as_str()))
}

/// Python: bot/memory/repeat.py:5409 `_the_corrected(lines, at, kind, found)`
///
/// The figure a later line takes back to, or the one it was given, or None when it is unsure.
pub fn _the_corrected(lines: &[String], at: usize, kind: Option<&str>, found: Figure) -> Option<Figure> {
    let mut found = Some(found);
    for later in lines.iter().skip(at + 1) {
        if later.is_empty() || !_a_line_that_corrects(later) || A_LINE_THAT_ASKS.is_match(later) {
            continue;
        }
        if let Some(got) = _one_figure(&_figures_in(later), kind) {
            found = Some(got);
        } else if A_NUMBER_WORD.is_match(later) && kind != Some("time") {
            return None; // a number was taken back and the new one cannot be read
        }
    }
    found
}

/// Python: bot/memory/repeat.py:5443 `_the_length_they_mean(clean, lines, spoken=())`
///
/// The length a recall ask points at, in the person's own words, or "".
pub fn _the_length_they_mean(clean: &str, lines: &[String], spoken: &[String]) -> String {
    let mut wanted: Vec<String> =
        topics::keywords(clean).into_iter().filter(|w| !ASK_FRAME.contains(w) && !NOT_A_THING_ASKED.contains(w)).collect();
    if !wanted.is_empty() && !spoken.is_empty() {
        let theirs: Vec<String> = lines.iter().flat_map(|line| topics::keywords(line)).collect();
        let heard: Vec<String> = spoken.iter().flat_map(|line| topics::keywords(line)).collect();
        let new: Vec<&String> = wanted.iter().filter(|w| !theirs.iter().any(|o| _the_same_word(w, o))).collect();
        if !new.is_empty() && new.iter().all(|w| heard.iter().any(|o| _the_same_word(w, o))) {
            wanted = Vec::new();
        }
    }
    let mut named: Vec<(usize, Vec<Figure>)> = Vec::new();
    let mut anywhere: Vec<(usize, Vec<Figure>)> = Vec::new();
    for (back, line) in lines.iter().enumerate() {
        if A_LINE_THAT_ASKS.is_match(line) {
            continue; // a question the person asked is not a length the person gave
        }
        let figures: Vec<Figure> = _figures_in(line).into_iter().filter(|one| one.0 == "time").collect();
        if figures.is_empty() {
            continue;
        }
        anywhere.push((back, figures.clone()));
        let words = topics::keywords(line);
        if wanted.iter().any(|w| words.iter().any(|o| _the_same_word(w, o))) {
            named.push((back, figures));
        }
    }
    let pools: Vec<&Vec<(usize, Vec<Figure>)>> = if wanted.is_empty() { vec![&named, &anywhere] } else { vec![&named] };
    for pool in pools {
        if pool.is_empty() {
            continue;
        }
        let distinct: HashSet<String> = pool.iter().flat_map(|(_, figures)| figures.iter().map(|one| text::lower(&one.1))).collect();
        if distinct.len() == 1 {
            let (back, figures) = &pool[pool.len() - 1];
            let last = figures[figures.len() - 1].clone();
            let later = _the_corrected(lines, *back, Some("time"), last.clone());
            return later.unwrap_or(last).1;
        }
        let (back, figures) = &pool[pool.len() - 1];
        if A_CORRECTION.is_match(&lines[*back]) {
            return figures[figures.len() - 1].1.clone();
        }
    }
    String::new()
}

/// Python: bot/memory/repeat.py:5498 `_the_figure_of_a_goal(lines, kind, wanted=())`
///
/// The figure on the line where the person set a goal, or None when there is none.
pub fn _the_figure_of_a_goal(lines: &[String], kind: Option<&str>, wanted: &[String]) -> Option<Figure> {
    for (at, line) in lines.iter().enumerate() {
        if line.is_empty() || A_LINE_THAT_ASKS.is_match(line) || !A_GOAL_LINE.is_match(line) {
            continue;
        }
        if A_RATE_LINE.is_match(line) {
            continue;
        }
        let words = topics::keywords(line);
        let narrowing: Vec<&String> = wanted.iter().filter(|w| !GOAL_NOUNS.contains(*w)).collect();
        if !narrowing.iter().all(|w| words.iter().any(|o| _the_same_word(w, o))) {
            continue;
        }
        let figures = _figures_in(line);
        let mut found = _one_figure(&figures, kind);
        if found.is_none() && kind.is_none() {
            found = _one_figure(&figures, Some("amount"));
        }
        if let Some(found) = found {
            // A later line that puts the goal right is the goal.
            return _the_corrected(lines, at, kind, found);
        }
    }
    None
}

/// Python: bot/memory/repeat.py:5525 `_the_figure_counts_the_thing(text, figure)`
///
/// True when a figure is an answer to an ask that names a thing, or the ask names nothing.
pub fn _the_figure_counts_the_thing(text_: &str, figure: &str) -> bool {
    let thing = crate::memory::history::the_thing_they_named(text_);
    if !thing.truthy() {
        return true;
    }
    let named: HashSet<String> = topics::content(&py_str(&thing)).into_iter().collect();
    topics::content(figure).iter().any(|w| named.contains(w))
}

/// Python: bot/memory/repeat.py:5570 `a_narrowing_and_its_line(line, memory)`
///
/// The line `line` narrows, with `line` after it (a tuple), or None.
pub fn a_narrowing_and_its_line(line: &str, memory: &Memory) -> Value {
    match narrowing_and_its_line(line, memory) {
        Some((before, said)) => Value::Tuple(std::sync::Arc::new(vec![Value::from(before), Value::from(said)])),
        None => Value::None,
    }
}

fn narrowing_and_its_line(line: &str, memory: &Memory) -> Option<(String, String)> {
    let said = text::squash(line);
    if said.is_empty() || said.contains('?') || n_words(&said) > *A_NARROWING_WORDS {
        return None;
    }
    A_NARROWING_ONLY.match_(&said)?;
    let lines: Vec<String> = turns(memory).iter().map(|turn| text::squash(&turn.user)).collect();
    let places: Vec<usize> = lines.iter().enumerate().filter(|(_, t)| **t == said).map(|(at, _)| at).collect();
    let last = *places.last()?;
    if last == 0 {
        return None;
    }
    let before = &lines[last - 1];
    if before.is_empty()
        || before.contains('?')
        || n_words(before) < 2
        || topics::content(before).is_empty()
        || A_NARROWING_ONLY.match_(before).is_some()
    {
        return None;
    }
    Some((before.clone(), said))
}

/// Python: bot/memory/repeat.py:5604 `_the_turn_that_names_it(text, memory)`
///
/// The line they said about the thing the ask names, quoted back, or None.
pub fn _the_turn_that_names_it(text_: &str, memory: &Memory) -> Option<String> {
    let row = crate::memory::history::the_earlier_turn(memory, text_, &Value::from(""));
    let line = text::rstrip_chars(&text::squash(&py_str_or_empty(row.get("user"))), " .!?").to_string();
    if line.is_empty() {
        return None;
    }
    if let Some((before, after)) = narrowing_and_its_line(&line, memory) {
        // The thing asked about is in the line it narrows, so both are said.
        let options = vec![
            format!("You said: \"{before}\" \u{2014} then \"{after}\"."),
            format!("Your own words were: \"{before}\", then \"{after}\"."),
            format!("That was: \"{before}\" \u{2014} then \"{after}\"."),
        ];
        return Some(pick(&options, memory, ""));
    }
    let options = vec![
        format!("You said: \"{line}\""),
        format!("Your own words were: \"{line}\""),
        format!("That was: \"{line}\""),
    ];
    Some(pick(&options, memory, ""))
}

/// "You said <figure>." in one of its two shapes.
fn said_back(figure: &str, memory: &Memory) -> String {
    pick(&[format!("You said {figure}."), format!("{} \u{2014} that's what you told me.", upper_first(figure))], memory, "")
}

/// Python: bot/memory/repeat.py:5623 `the_figure_they_gave(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`the_figure_they_gave_text`].
pub fn the_figure_they_gave(text_: &str, memory: &Memory) {
    let _ = the_figure_they_gave_text(text_, memory);
}

/// The figure a person gave earlier and asks for again, or None (`the_figure_they_gave`).
pub fn the_figure_they_gave_text(text_: &str, memory: &Memory) -> Option<String> {
    let clean = text::squash(&text::lower(text_));
    if clean.is_empty() || n_words(&clean) > 16 || !GAVE_ASKS.is_match(&clean) {
        return None;
    }
    if NOT_A_FIGURE_ASK.is_match(&clean) {
        return None;
    }
    // Where the count store holds the count, the store's answer is the whole one.
    if let Some(counting) = COUNT_ASK.search(&clean) {
        if let Some(counted) = counts_answer(&clean, memory).filter(|c| !c.is_empty()) {
            return Some(counted);
        }
        if text::split_ref(counting.str(1)).iter().any(|w| COUNTS_THEIRS.contains(*w)) {
            return None;
        }
    }
    let mut wanted: Vec<String> = topics::keywords(&clean).into_iter().filter(|w| !ASK_FRAME.contains(w)).collect();
    if A_PAY_WORD.is_match(&clean) {
        wanted.retain(|w| !PAY_WHEN_WORDS.contains(w));
    }
    if wanted.is_empty() && A_PAY_WORD.is_match(&clean) && A_PAY_VERB_AT_THE_END.is_match(&clean) {
        wanted = vec!["pay".to_string()];
    }
    if wanted.len() > 4 {
        return None; // too much to be about one thing
    }
    let goal_named = wanted.iter().any(|w| GOAL_NOUNS.contains(w));
    // The kind is read from the most particular frame first.
    let mut kind: Option<String> = None;
    for name in ["often", "time", "heat", "amount"] {
        if FIGURE_ASKS.iter().any(|(one, ask, _)| one == name && ask.is_match(&clean)) {
            kind = Some(name.to_string());
            break;
        }
    }
    if matches!(kind.as_deref(), None | Some("amount") | Some("heat"))
        && (A_PLAN_ASK.is_match(&clean) || A_THE_PLAN_ASK.is_match(&clean))
    {
        return None;
    }
    let kind = kind.as_deref();
    // The lines are read as the person typed them.
    let all_lines: Vec<String> = turns(memory).iter().map(|turn| the_plain_line(&turn.user)).collect();
    let lines: Vec<String> = all_lines[all_lines.len().saturating_sub(*GAVE_LOOK_BACK)..].to_vec();
    let all_spoken: Vec<String> = turns(memory).iter().map(|turn| turn.bot.clone()).collect();
    let spoken: Vec<String> = all_spoken[all_spoken.len().saturating_sub(*GAVE_LOOK_BACK)..].to_vec();
    if kind == Some("time") && A_TOLD_BACK.is_match(&clean) {
        let said = _the_length_they_mean(&clean, &lines, &spoken);
        if !said.is_empty() {
            if !_the_figure_counts_the_thing(&clean, &said) {
                return _the_turn_that_names_it(text_, memory);
            }
            return Some(said_back(&said, memory));
        }
    }
    if goal_named {
        // A goal is read from the whole chat, not the last few lines.
        if let Some(found) = _the_figure_of_a_goal(&all_lines, kind, &wanted) {
            return Some(said_back(&found.1, memory));
        }
    }
    if !wanted.is_empty() {
        let mut said: Vec<String> = lines.iter().flat_map(|line| topics::keywords(line)).collect();
        said.extend(lines.iter().filter(|line| A_TAKE_HOME.is_match(line)).map(|_| "takehome".to_string()));
        said.extend(lines.iter().filter(|line| A_IM_ON_A_FIGURE.is_match(line)).map(|_| "pay".to_string()));
        let kept: Vec<String> = wanted.iter().filter(|w| said.iter().any(|o| _the_same_word(w, o))).cloned().collect();
        let named: Vec<&String> = wanted.iter().filter(|w| !A_MEASURE_WORD.contains(*w) && !A_DOING_WORD.contains(*w)).collect();
        if !named.is_empty() && !named.iter().any(|w| kept.contains(w)) {
            return None; // the ask named a thing of its own and no line holds it
        }
        if A_PAY_WORD.is_match(&clean) && (A_PAY_JUDGEMENT.is_match(&clean) || kept.len() < wanted.len()) {
            return None; // pay with a qualifier on it is a question of its own
        }
        if !A_THING_NAMED_AFTER_THE_CUE.is_match(&clean) {
            wanted = kept;
        }
    }
    if _the_store_keeps_this_figure(&clean, memory) {
        return None; // the store answers it from its own record
    }
    if wanted.is_empty() {
        // Names nothing at all: one figure of that kind in the chat is the one they mean.
        let bare = match kind {
            Some("amount") => A_BARE_AMOUNT_ASK.is_match(&clean),
            Some("time") => A_BARE_DURATION_ASK.is_match(&clean),
            _ => false,
        };
        if !bare {
            return None;
        }
        let found: Vec<(usize, String)> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| !A_LINE_THAT_ASKS.is_match(line))
            .flat_map(|(back, line)| {
                _figures_in(line).into_iter().filter(|one| Some(one.0.as_str()) == kind).map(move |one| (back, one.1))
            })
            .collect();
        let distinct: HashSet<String> = found.iter().map(|(_, one)| text::lower(one)).collect();
        let figure = if distinct.len() != 1 {
            // A line that opens as a correction is the one they mean, when it is the last said.
            let corrected: Vec<&String> = found.iter().filter(|(back, _)| A_CORRECTION.is_match(&lines[*back])).map(|(_, one)| one).collect();
            match (corrected.last(), found.last()) {
                (Some(last_corrected), Some(last)) if &last.1 == *last_corrected => (*last_corrected).clone(),
                _ => return None,
            }
        } else {
            found[0].1.clone()
        };
        if !_the_figure_counts_the_thing(&clean, &figure) {
            return _the_turn_that_names_it(text_, memory);
        }
        return Some(said_back(&figure, memory));
    }
    let mut best: Option<((usize, usize), Figure, Vec<String>, String)> = None;
    for back in (0..lines.len()).rev() {
        let line = &lines[back];
        if line.is_empty() || A_LINE_THAT_ASKS.is_match(line) {
            continue; // a question the person asked is not a line that gave a figure
        }
        let mut said = topics::keywords(line);
        if A_TAKE_HOME.is_match(line) {
            said.push("takehome".to_string());
        }
        if A_IM_ON_A_FIGURE.is_match(line) {
            said.push("pay".to_string());
        }
        let core: Vec<String> = {
            let c: Vec<String> = wanted.iter().filter(|w| !FIGURE_NOUNS.contains(*w)).cloned().collect();
            if c.is_empty() { wanted.clone() } else { c }
        };
        if !core.iter().all(|w| said.iter().any(|o| _the_same_word(w, o))) {
            continue; // the line does not say the thing the figure is for
        }
        let shared: Vec<String> = wanted.iter().filter(|w| said.iter().any(|o| _the_same_word(w, o))).cloned().collect();
        let figures = _figures_in(line);
        let Some(mut found) = _one_figure(&figures, kind) else { continue };
        if let Some(beside) = _the_figure_beside_the_thing(&figures, &core) {
            found = beside; // the figure the line puts next to the thing the ask names
        }
        if COUNT_ASK.is_match(&clean) && _a_list_of_ages(line) {
            continue; // a list of ages names the children one at a time
        }
        if _money_paid_for_a_thing(&clean, line) {
            continue; // what the thing cost, not the thing
        }
        let Some(found) = _the_corrected(&lines, back, kind, found) else { continue };
        let many = _every_figure_the_ask_names(&wanted, line, kind);
        let last = text::split_ref(&found.1).last().map(|s| s.to_string()).unwrap_or_default();
        let mut gone_words: Vec<String> = said.iter().filter(|w| shared.contains(w)).cloned().collect();
        gone_words.push(last);
        if _something_gone(&lines, back, &gone_words) {
            continue; // the count they gave is not the count they have any more
        }
        let score = (shared.len(), back);
        if best.as_ref().is_none_or(|b| score > b.0) {
            best = Some((score, found, shared, many));
        }
    }
    let (_score, (_kind, said, number, counted), _shared, many) = best?;
    if !many.is_empty() {
        // Every figure the ask named, in the person's own words.
        return Some(said_back(&many, memory));
    }
    let mut noun = if counted { _the_thing_counted(&clean, &said) } else { String::new() };
    if counted
        && !noun.is_empty()
        && PERIOD_WORD.contains(&text::lower(text::split_ref(&noun).last().copied().unwrap_or("")))
        && A_RATE_IN_THE_FIGURE.is_match(&said)
    {
        return None; // the thing they gave is a rate: said back in the doing's own words elsewhere
    }
    if !noun.is_empty() && ["1", "one"].contains(&text::lower(&number).as_str()) && noun.ends_with('s') && !noun.ends_with("ss") {
        noun.pop(); // "one kid", not "one kids"
    }
    let figure = if noun.is_empty() { said.clone() } else { text::strip(&format!("{number} {noun}")).to_string() };
    if _the_ask_gave_that_figure(&number, &clean) {
        return None; // a figure in the ask is the ask's own
    }
    if !noun.is_empty() && _a_count_of_one_with_a_plural(&number, &noun) {
        return None;
    }
    if !_the_figure_counts_the_thing(&clean, &figure) {
        return _the_turn_that_names_it(text_, memory);
    }
    if _a_bare_count(&figure) {
        return Some(pick(&[format!("You said {figure}."), format!("{figure}, as you said.")], memory, ""));
    }
    Some(said_back(&figure, memory))
}

/// Python: bot/memory/repeat.py:5863 `_the_store_keeps_this_figure(clean, memory)`
///
/// Whether the store keeps a figure under the thing this ask names.
pub fn _the_store_keeps_this_figure(clean: &str, memory: &Memory) -> bool {
    let asked = keyword_set(clean);
    if asked.is_empty() {
        return false;
    }
    let Some(log) = memory.get("facts_log") else { return false };
    for entry in log.items().iter().rev() {
        if !entry.is_dict() {
            continue;
        }
        let value = text::squash(&py_str_or_empty(entry.get("value")));
        if value.is_empty() || !A_FIGURE_WORD.is_match(&value) {
            continue;
        }
        let key_words = topics::keywords(&py_str_or_empty(entry.get("key")).replace('_', " "));
        if key_words.is_empty() {
            continue;
        }
        if !key_words.iter().any(|w| asked.iter().any(|o| _the_same_word(w, o))) {
            continue;
        }
        let said_on = py_str_or_empty(entry.get("text"));
        if !said_on.is_empty() && A_LINE_THAT_ASKS.is_match(&said_on) {
            continue; // a question is not a line that gave a figure
        }
        return true;
    }
    false
}

/// Python: bot/memory/repeat.py:5926 `_the_figure_beside_the_thing(figures, words)`
///
/// The figure the line says next to the thing the ask names, or None.
pub fn _the_figure_beside_the_thing(figures: &[Figure], words: &[String]) -> Option<Figure> {
    for figure in figures {
        let beside = topics::keywords(&figure.1);
        if words.iter().any(|w| beside.iter().any(|o| _the_same_word(w, o))) {
            return Some(figure.clone());
        }
    }
    None
}

/// Python: bot/memory/repeat.py:5942 `_every_figure_the_ask_names(wanted, line, kind)`
///
/// Every figure the line gives for the things the ask names, said as one phrase, or "".
pub fn _every_figure_the_ask_names(wanted: &[String], line: &str, kind: Option<&str>) -> String {
    let named: Vec<&String> = wanted.iter().filter(|w| !ASK_FRAME.contains(*w) && !FIGURE_NOUNS.contains(*w)).collect();
    if named.len() < 2 {
        return String::new();
    }
    let figures: Vec<Figure> =
        _figures_in(line).into_iter().filter(|one| kind.is_none_or(|k| k.is_empty() || one.0 == k)).collect();
    if figures.len() < 2 {
        return String::new();
    }
    let mut taken: Vec<Figure> = Vec::new();
    for word in &named {
        for figure in &figures {
            if taken.contains(figure) {
                continue;
            }
            if topics::keywords(&figure.1).iter().any(|o| _the_same_word(word, o)) {
                taken.push(figure.clone());
                break;
            }
        }
    }
    if taken.len() < 2 {
        return String::new();
    }
    let parts: Vec<&str> = taken.iter().map(|f| f.1.as_str()).collect();
    format!("{} and {}", parts[..parts.len() - 1].join(", "), parts[parts.len() - 1])
}

/// Python: bot/memory/repeat.py:5972 `asked_back(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`asked_back_text`].
pub fn asked_back(text_: &str, memory: &Memory) {
    let _ = asked_back_text(text_, memory);
}

/// The figure a recall ask wants back, or None when the line is not one (`asked_back`).
pub fn asked_back_text(text_: &str, memory: &Memory) -> Option<String> {
    if crate::memory::recall::looks_back(text_).is_empty() {
        return None;
    }
    if !A_POINTS_BACK_HARD.is_match(text_) && crate::memory::recall::the_earlier_turn(text_, memory).is_none() {
        return None; // the line has to name something a line already carries
    }
    if let Some(figure) = the_figure_they_gave_text(text_, memory).filter(|f| !f.is_empty()) {
        return Some(figure);
    }
    // A count asked about the job rather than about the people is read where every count is read.
    let said = py_try(|| counts_answer(text_, memory))?;
    said.filter(|s| !s.is_empty())
}

/// Python: bot/memory/repeat.py:6009 `carries_the_figure(figure, reply)`
///
/// Whether a reply already says the figure the person asked back.
pub fn carries_the_figure(figure: &Value, reply: &str) -> bool {
    let figure = py_str_or_empty(Some(figure));
    let numbers = RE_NUMBERS.findall_str(&figure);
    if !numbers.is_empty() {
        for number in &numbers {
            let pattern = format!(r"(?<![\d,.]){}(?!\d)", pyre::escape(number));
            if !re_cached(&pattern, 0).is_match(reply) {
                return false;
            }
        }
        return true;
    }
    let words: Vec<String> = topics::keywords(&figure).into_iter().filter(|w| !is_digit(w)).collect();
    let low = text::lower(reply);
    !words.is_empty() && words.iter().all(|w| low.contains(w.as_str()))
}

/// Python: bot/memory/repeat.py:6054 `_the_same_words(one, other)`
///
/// Whether two lines are the same question in the same words.
pub fn _the_same_words(one: &str, other: &str) -> bool {
    let bare = |line: &str| text::squash(&RE_NOT_ALNUM.sub(" ", &text::lower(line)));
    let first = bare(one);
    first == bare(other) && n_words(&first) >= 4
}

/// Python: bot/memory/repeat.py:6061 `_shortened(said)`
///
/// The answer, kept whole up to a paragraph, so a repeat is not a wall of text again.
pub fn _shortened(said: &str) -> String {
    let said = text::squash(said);
    if text::len(&said) <= 320 {
        return said;
    }
    let mut kept = String::new();
    for part in _sentences(&said) {
        if !kept.is_empty() && text::len(&kept) + text::len(&part) > 320 {
            break;
        }
        kept = text::strip(&format!("{} {}", kept, part)).to_string();
    }
    if kept.is_empty() { said } else { kept }
}

/// Python: bot/memory/repeat.py:6074 `_the_answer_again(said, memory)`
///
/// The earlier answer, framed as the repeat it is, in the frames the chat already knows.
pub fn _the_answer_again(said: &str, memory: &Memory) -> String {
    if let Some(framed) = topics_said_again(said, memory).filter(|f| !f.is_empty()) {
        return framed;
    }
    py_format(&REPEAT[0], &[("reply", &_shortened(said))])
}

/// Python: bot/memory/repeat.py:6090 `the_same_question_again(text, memory)`
///
/// CONTRACT: typed `()`; the value is [`the_same_question_again_text`].
pub fn the_same_question_again(text_: &str, memory: &Memory) {
    let _ = the_same_question_again_text(text_, memory);
}

/// The answer this same question got earlier in the chat, or None (`the_same_question_again`).
pub fn the_same_question_again_text(text_: &str, memory: &Memory) -> Option<String> {
    let all = turns(memory);
    if all.len() < 3 {
        return None;
    }
    let clean = text::strip(text_);
    if n_words(clean) < 4 {
        return None;
    }
    let earlier = &all[..all.len() - 2];
    for (back, turn) in earlier.iter().rev().enumerate() {
        if !_the_same_words(clean, &turn.user) {
            continue;
        }
        let said = text::strip(&turn.bot);
        if said.is_empty() || text::len(said) < 40 {
            continue;
        }
        if HEDGE.is_match(said) || text::rstrip(said).ends_with('?') {
            continue; // a hedge or a question is not an answer to hand back
        }
        let index = all.len() - 2 - back;
        let between = if index + 1 < all.len() - 1 { &all[index + 1..all.len() - 1] } else { &[][..] };
        if between.iter().any(|t| CHANGED_IT.is_match(&t.user)) {
            return None; // the answer of then is about a plan that has since changed
        }
        return Some(_the_answer_again(said, memory));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chat(lines: &[(&str, &str, &str)]) -> Memory {
        let mut memory = Memory::new();
        memory.turns = lines
            .iter()
            .map(|(user, bot, module)| Turn { user: user.to_string(), bot: bot.to_string(), module: Some(module.to_string()) })
            .collect();
        memory
    }

    /// Expected values from PyBot (`harness/pycall.py`).
    #[test]
    fn readers_of_an_answer() {
        let washer = "Stop the cycle, unplug it and turn the water off. Check the door seal for a tear, then run a hot 60C service wash with a washing-machine cleaner because the smell comes from the drum. If it still leaks, call an engineer.";
        assert_eq!(_the_first_step(washer), Some("stop the cycle".into()));
        assert_eq!(_the_next_step(washer, &Memory::new()), Some("unplug it and turn the water off".into()));
        assert_eq!(
            a_move("Most bad posture is the chair and the sitting still: set the screen so the top of it is at eye level, and get up every half hour.", false),
            Some("set the screen so the top of it is at eye level, and get up every half hour".into())
        );
        assert_eq!(
            _short_enough("Most back pain is muscular and settles in two to six weeks, and staying active is what helps most: keep walking, take paracetamol or ibuprofen if you need it, and use heat for the stiffness in the morning and evening for a week or two."),
            "Most back pain is muscular and settles in two to six weeks, and staying active is what helps most."
        );
        assert_eq!(
            _most_condensed("Most back pain is muscular and settles in two to six weeks. Staying active is what helps most. Keep walking and take paracetamol if you need it.", Some(&Memory::new())),
            "Staying active is what helps most."
        );
        assert_eq!(
            _closing_the_quote("Ask early, name the dates and say who covers: \"Could I take 12-16 May off?", "Ask early, name the dates and say who covers: \"Could I take 12-16 May off? I've arranged cover with Priya.\" Then send it."),
            "Ask early, name the dates and say who covers: \"Could I take 12-16 May off? I've arranged cover with Priya.\""
        );
        assert_eq!(
            _a_conditional_move("That's disappointing, especially after an interview. When you're ready, ask for one line of feedback (was there anything I could strengthen?) and keep applying."),
            Some("when you're ready, ask for one line of feedback".into())
        );
        assert_eq!(_opening_move("Blot, don't rub, then dab a little hand sanitiser into the mark with a cotton bud and blot again."), Some("blot, don't rub".into()));
        assert_eq!(
            _opening_move("Start with run-walk: one minute of easy jogging, two minutes of walking, repeated for 20 minutes, three times a week."),
            Some("one minute of easy jogging, two minutes of walking, repeated for 20 minutes, three times a week".into())
        );
        assert_eq!(after_an_ack("cheers. also im skint till friday and theres nothing in the fridge"), "also im skint till friday and theres nothing in the fridge");
        assert!(asks_for_less("make it shorter please"));
        assert!(carries_the_figure(&Value::from("£1,500"), "It costs about £1,500 a year."));
        assert_eq!(_whether_to("can i still eat the ones that look alright"), "whether i can still eat the ones that look alright");
    }

    #[test]
    fn readers_of_a_card_in_hand() {
        let freezer = chat(&[(
            "my freezer door was left open overnight",
            "Food that still has ice crystals in it is safe to refreeze. Anything fully thawed should be cooked and eaten within a day or binned.",
            "answers",
        )]);
        assert_eq!(the_part_asked_text("is the food still ok", &freezer), Some("Food that still has ice crystals in it is safe to refreeze.".into()));
        let phone = chat(&[(
            "i dropped my phone in water",
            "Wipe it, shake out the port, and leave it in a warm dry place for 24 hours. Rice does nothing for it, so do not bother.",
            "answers",
        )]);
        assert_eq!(the_figure_asked_text("how long should i leave it", &phone), Some("Wipe it, shake out the port, and leave it in a warm dry place for 24 hours.".into()));
        let eggs = chat(&[(
            "how do i soft boil an egg",
            "Lower the eggs into boiling water and cook for 6 minutes for a runny yolk, then put them in cold water for a minute so they peel.",
            "answers",
        )]);
        assert_eq!(
            what_it_knows_text("do i need to prick them", &eggs),
            Some("Lower the eggs into boiling water and cook for 6 minutes for a runny yolk, then put them in cold water for a minute so they peel. On whether to prick them, the packet or a cookery site will know more than I can.".into())
        );
        let mut washer = chat(&[
            ("my washing machine is leaking", "Stop the cycle, unplug it and turn the water off. Check the door seal for a tear. Run a hot service wash with a cleaner while you are out.", "answers"),
            ("what do i do", "Start with this: Stop the cycle, unplug it and turn the water off.", "answers"),
        ]);
        washer.set("repeated", Value::from(vec!["Start with this: Stop the cycle, unplug it and turn the water off.".to_string()]));
        assert_eq!(
            _another_part("Stop the cycle, unplug it and turn the water off. Check the door seal for a tear. Run a hot service wash with a cleaner while you are out.", &washer),
            ("Check the door seal for a tear".into(), true)
        );
    }
}
