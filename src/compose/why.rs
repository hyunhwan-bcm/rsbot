//! Port of `bot/compose/why.py`: the reply to a bare "why" after an answer, read out of the
//! answer itself, or an honest "I can't see the cause from here"; and the answer above said
//! shorter for a bare "say that again".
//!
//! Every table and compiled regex comes from the data bundle under its Python name, which is the
//! binding the running module sees (`CONDITIONAL` and `CLAUSE_END` are defined twice in the file;
//! the bundle holds the later one, as the functions do at run time).
#![allow(non_upper_case_globals, clippy::all)]

use crate::data;
use crate::py::re::{Match, Re};
use crate::py::text;
use crate::state::{Memory, Turn};
use crate::understanding::rx;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

macro_rules! regexes {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<&'static Re> =
            LazyLock::new(|| data::re(concat!("bot.compose.why.", stringify!($name))));)*
    };
}
macro_rules! word_sets {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<HashSet<String>> =
            LazyLock::new(|| data::set(concat!("bot.compose.why.", stringify!($name))));)*
    };
}
macro_rules! pools {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<Vec<String>> =
            LazyLock::new(|| data::strs(concat!("bot.compose.why.", stringify!($name))));)*
    };
}
macro_rules! ints {
    ($($name:ident),* $(,)?) => {
        $(static $name: LazyLock<usize> =
            LazyLock::new(|| data::get(concat!("bot.compose.why.", stringify!($name))).as_int().unwrap() as usize);)*
    };
}

regexes!(BARE_WHY, CAUSING_IT, WRONG, LIFE, WHY_NOT, THEIR_DECISION, SAID_NO, THEIR_SUGGESTION, HEDGED, CAUSE, SO_ADVICE, DOSE, AND_STEP, AND_ADVICE);
regexes!(NEGATIVE, CONDITIONAL, CONDITIONAL_SENTENCE, INSTRUCTION, SO_REASON, LEADING_MARKER, CLAIM_BEFORE_COLON, CLAIM_SUBJECT, IMPERATIVE, THE_FIX, NAMED_CAUSE);
regexes!(EXPLAINS, DANGLING, RESULT_NUMBER, CHANGE, WHEN_CAUSE, SO_FROM_HERE, THINKS_BACK, A_NOD_OPENS, THE_REASON_POINTS_AT_THE_ADVICE, EXAMPLE_IN_QUOTES);
regexes!(WHAT_IT_TAKES, A_THING_NAMED_BEFORE_A_COLON, A_LEAD_IN, SHORT_OF_A_CAUSE, AFTER_THE_MARK, A_GENERAL_CLAIM, ENDS_THE_CLASS, SAYS_WHAT_IT_DOES);
regexes!(ACT_NOW, ACT_NOW_SOON, SENTENCE, SAID_AGAIN, CLAUSE_END, NO_CUT, END_OF_SENTENCE, OPENS_ON_CARE, _A_DOING, _A_FIGURE, _SAYS_HOW, _ANNOUNCES);
regexes!(_NEEDS_A_SUBJECT, _VERB_SHAPE, _A_CAUSE_OPENS, _LEANS_BACK, _EXPLAINS, _OPENS_ON_A_FIGURE, _A_CATEGORY, _POINTS_FORWARD, _POINTS_FORWARD_ANY);
regexes!(WHY_YOU_SAID_IT, THE_BOT_DESCRIBED_ITSELF, CARE_ANSWER, ABOUT_THE_BOT_LINE);
word_sets!(SUGGESTED_VERBS, ANSWERED, KEEP_CAPITAL, WHY_FRONT, A_PLACEHOLDER, _A_LEAD_IN, _A_PARTICIPLE, _A_VERB);
pools!(WHY_ANSWERED, WHY_BODY, PARTS, WHY_NAMED, WHY_NAMED_BECAUSE, WHY_NAMED_SELF, WHY_AGAIN, WRONG_ANSWERED, WRONG_NOT_A_FAULT, WHY_NOT_ANSWERED);
pools!(WHY_NOT_ITS_OWN, WHY_METHOD, AGAIN_SHORTER, REPEAT_LEADS, WHY_CARE, WHY_THE_BOT);
ints!(SUBSTANCE, A_NOD_IS_SHORT, FIRST_SENTENCES, ANSWER_LONG, _A_SAVE_LEAST, _A_CLAUSE_LEAST, _A_LONG_SENTENCE, _A_STRIDE, AGAIN_WORDS);
ints!(_A_DOING_WINDOW, _LEANS_BACK_WINDOW, _A_REASON_LEAST, _POINTS_FORWARD_LEAST);
static WHY_END: LazyLock<String> = LazyLock::new(|| data::string("bot.compose.why.WHY_END"));
static ANOTHER_FRAME: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.another.FRAME"));

/// `from . import pick`: one of the options, never the reply just given.
fn pick(options: &[String], memory: &Memory, seed_text: &str) -> String {
    crate::compose::pick(options, memory, seed_text)
}

/// `re.split(r"(?<=[.!?])\s+", s)`.
fn sentences_of(s: &str) -> Vec<String> {
    rx(r"(?<=[.!?])\s+").split(s)
}

/// `str.split()` word count.
fn n_words(s: &str) -> usize {
    text::split_ref(s).len()
}

/// The bot's line of a turn, or "" for Python's `{}`.
fn bot_of(last: Option<&Turn>) -> &str {
    last.map(|t| t.bot.as_str()).unwrap_or("")
}

/// `s.strip().rstrip(".,;:")`.
fn tidy(s: &str) -> String {
    text::strip(s).trim_end_matches(['.', ',', ';', ':']).to_string()
}

/// Python: bot/compose/why.py:94 `_named_as_a_sentence(last, reason)`
/// True when the answer named this very clause as its cause, as "the cause is ...".
pub fn _named_as_a_sentence(last: Option<&Turn>, reason: &str) -> bool {
    let want = text::squash(&text::lower(reason)).trim_matches([' ', '.', ',', ';', ':']).to_string();
    if want.is_empty() {
        return false;
    }
    for sentence in sentences_of(bot_of(last)) {
        let Some(found) = NAMED_CAUSE.search(&sentence) else { continue };
        let clause = text::squash(&text::lower(found.str(1))).trim_matches([' ', '.', ',', ';', ':']).to_string();
        if clause == want {
            return true;
        }
    }
    false
}

/// Python: bot/compose/why.py:111 `_reads_a_fault(last)`
/// True when the turn above was about a thing that can go wrong, not about the person's life.
pub fn _reads_a_fault(last: Option<&Turn>) -> bool {
    let said = text::squash(&text::lower(bot_of(last)));
    let them = text::squash(&text::lower(last.map(|t| t.user.as_str()).unwrap_or("")));
    if LIFE.is_match(&them) {
        return false;
    }
    !said.is_empty()
}

/// Python: bot/compose/why.py:149 `_about_someone_elses_decision(memory)`
/// True when the person's own last line was about somebody else deciding or refusing.
pub fn _about_someone_elses_decision(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(3);
    for last in turns[start..].iter().rev() {
        if last.user.is_empty() {
            continue;
        }
        let low = text::squash(&text::lower(&last.user));
        if THEIR_DECISION.is_match(&low) && SAID_NO.is_match(&low) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/why.py:193 `_the_suggestion_was_theirs(memory)`
/// True when the person's own line handed on the advice the "why not" is asking about.
pub fn _the_suggestion_was_theirs(memory: &Memory) -> bool {
    let turns = &memory.turns;
    let start = turns.len().saturating_sub(3);
    for last in turns[start..].iter().rev() {
        if last.user.is_empty() {
            continue;
        }
        let low = text::squash(&text::lower(&last.user));
        if THEIR_SUGGESTION.is_match(&low) {
            return true;
        }
        let words = text::split_ref(low.trim_matches([' ', '.', ',', '!', '?', ':', ';']));
        if let Some(first) = words.first() {
            if SUGGESTED_VERBS.contains(first.trim_matches(['.', ',', '!', '?', ':', ';'])) {
                return true;
            }
        }
    }
    false
}

/// Python: bot/compose/why.py:460 `_in_frame(frame, clause)`
/// The clause put into a frame, with its first letter following the frame's sentence.
pub fn _in_frame(frame: &str, clause: &str) -> String {
    let word = clause.split(' ').next().unwrap_or("").trim_matches(['"', '\'', '(', '*', '[']);
    let second = text::slice(word, 1, Some(2));
    let keep = KEEP_CAPITAL.contains(text::lower(word).trim_matches(['.', ',', '!', '?', ':', ';'])) || (!second.is_empty() && text::is_upper(&second));
    if frame.starts_with("{c}") {
        return frame.replace("{c}", clause);
    }
    if keep || clause.is_empty() {
        return frame.replace("{c}", clause);
    }
    let lowered = format!("{}{}", text::lower(&text::slice(clause, 0, Some(1))), text::slice(clause, 1, None));
    frame.replace("{c}", &lowered)
}

/// Python: bot/compose/why.py:499 `is_a_reason_line(text)`
/// True when this is a cause this module handed back, not an answer of its own.
pub fn is_a_reason_line(text_: &str) -> bool {
    let line = text::lower(text::strip(text_)).replace('\u{2019}', "'");
    if line.is_empty() {
        return false;
    }
    if line.trim_end_matches([' ', '.']).ends_with(WHY_END.as_str()) {
        return true;
    }
    WHY_FRONT.iter().any(|front| line.starts_with(front.as_str()))
}

/// Python: bot/compose/why.py:569 `_said_the_reason_cannot_be_seen(memory)`
/// True when the bot's last line was this module's own answer to the same question.
pub fn _said_the_reason_cannot_be_seen(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said = &last.bot;
    [&*WHY_ANSWERED, &*WHY_BODY, &*WRONG_ANSWERED, &*WHY_METHOD, &*WRONG_NOT_A_FAULT].iter().any(|pool| pool.contains(said))
}

/// Python: bot/compose/why.py:596 `_is_bare_why(clean)`
/// True when the whole line asks for the reason and nothing else.
pub fn _is_bare_why(clean: &str) -> bool {
    BARE_WHY.match_(text::strip(clean)).is_some()
}

/// Python: bot/compose/why.py:601 `_last_turn(memory)`
/// The turn above this one, or None (Python's `{}`) when there is nothing to explain.
pub fn _last_turn(memory: &Memory) -> Option<&Turn> {
    let turns = &memory.turns;
    if turns.is_empty() {
        return None;
    }
    let mut skipped_a_nod = false;
    let start = turns.len().saturating_sub(3);
    for last in turns[start..].iter().rev() {
        let said = &last.bot;
        if _answered_nothing(said) {
            skipped_a_nod = true;
            continue;
        }
        if last.module.as_deref().is_some_and(|m| ANSWERED.contains(m)) {
            if n_words(said) < *SUBSTANCE {
                return None;
            }
            if skipped_a_nod {
                let reason = _reason_in(Some(last), false);
                if reason.is_empty() || THE_REASON_POINTS_AT_THE_ADVICE.match_(&reason).is_some() {
                    return None;
                }
            }
            return Some(last);
        }
        if last.module.as_deref() == Some("greeting") && HEDGED.is_match(said) {
            continue;
        }
        return None;
    }
    None
}

/// Python: bot/compose/why.py:664 `_answered_nothing(said)`
/// True when a bot line is a nod rather than an answer: it gives no step and names no cause.
pub fn _answered_nothing(said: &str) -> bool {
    let said = text::strip(said);
    if said.is_empty() || n_words(said) > *A_NOD_IS_SHORT {
        return false;
    }
    if IMPERATIVE.match_(said).is_some() || INSTRUCTION.is_match(said) || CAUSE.is_match(said) {
        return false;
    }
    if EXPLAINS.is_match(said) || NAMED_CAUSE.is_match(said) {
        return false;
    }
    A_NOD_OPENS.match_(said).is_some()
}

/// Python: bot/compose/why.py:693 `_gave_a_method(last)`
/// True when the answer above was the how-to and said nothing about why it goes wrong.
pub fn _gave_a_method(last: Option<&Turn>) -> bool {
    let said = bot_of(last);
    if said.is_empty() {
        return false;
    }
    if CAUSE.is_match(said) || EXPLAINS.is_match(said) || NAMED_CAUSE.is_match(said) {
        return false;
    }
    let sentences: Vec<String> = sentences_of(said).iter().map(|s| text::strip(s).to_string()).filter(|s| !s.is_empty()).collect();
    let steps = sentences.iter().filter(|s| IMPERATIVE.match_(s).is_some() || INSTRUCTION.is_match(s)).count();
    steps > 0 && steps * 2 >= sentences.len()
}

/// Python: bot/compose/why.py:705 `_a_cause_named_outright(last)`
/// The cause the answer above names as a cause, or "" when it only has a "because" clause.
pub fn _a_cause_named_outright(last: Option<&Turn>) -> String {
    for sentence in sentences_of(bot_of(last)).iter().take(*FIRST_SENTENCES) {
        let t = text::strip(sentence);
        if IMPERATIVE.match_(t).is_some() {
            continue;
        }
        if let Some(named) = NAMED_CAUSE.search(t) {
            if !THE_FIX.is_match(t) {
                let cause = tidy(named.str(1));
                if n_words(&cause) >= 2 {
                    return cause;
                }
            }
        }
        if let Some(thing) = A_THING_NAMED_BEFORE_A_COLON.match_(t) {
            return format!("the {}", text::strip(thing.name_str("thing")));
        }
    }
    String::new()
}

/// Python: bot/compose/why.py:729 `_reason_in(last, strict=False)`
/// The cause the answer above already named, as one clause, or "" when it named none.
pub fn _reason_in(last: Option<&Turn>, strict: bool) -> String {
    let said = bot_of(last);
    let reason = _a_reason_in(said, strict);
    if !reason.is_empty() && _inside_a_quotation(said, &reason) {
        return _a_reason_in(&_without_examples(said), strict);
    }
    reason
}

/// Python: bot/compose/why.py:754 `_without_examples(said)`
/// The answer with its quoted examples taken out, so their words cannot be handed back.
pub fn _without_examples(said: &str) -> String {
    EXAMPLE_IN_QUOTES.sub(" ", said)
}

/// Python: bot/compose/why.py:759 `_inside_a_quotation(said, clause)`
/// True when the clause sits inside a quotation in the answer, so it is an example's words.
pub fn _inside_a_quotation(said: &str, clause: &str) -> bool {
    let Some(at) = said.find(clause) else { return false };
    if at == 0 {
        return false;
    }
    let before = &said[..at];
    let marks = before.matches('"').count() + before.matches('\u{201c}').count() + before.matches('\u{201d}').count();
    marks % 2 == 1
}

/// Python: bot/compose/why.py:770 `_a_reason_in(said, strict=False)`
/// The cause the answer above already named, as one clause, or "" when it named none.
pub fn _a_reason_in(said: &str, strict: bool) -> String {
    let whole = sentences_of(said);
    let sentences: Vec<String> = whole.iter().take(*FIRST_SENTENCES).cloned().collect();
    for sentence in &sentences {
        let t = text::strip(sentence);
        if IMPERATIVE.match_(t).is_some() {
            continue;
        }
        let stripped_lead = A_LEAD_IN.sub("", t);
        let named = A_THING_NAMED_BEFORE_A_COLON.match_(t).map(|m| m.name_str("thing").to_string()).or_else(|| A_THING_NAMED_BEFORE_A_COLON.match_(&stripped_lead).map(|m| m.name_str("thing").to_string()));
        if let Some(thing) = named {
            return format!("the {}", text::strip(&thing));
        }
        if let Some(short) = SHORT_OF_A_CAUSE.match_(t) {
            let cause = text::squash(short.name_str("cause")).trim_end_matches(['.', ',', ';', ':']).to_string();
            if n_words(&cause) >= 2 {
                return cause;
            }
        }
    }
    for sentence in &whole {
        let Some(found) = NAMED_CAUSE.search(sentence) else { continue };
        if INSTRUCTION.is_match(sentence) || RESULT_NUMBER.is_match(sentence) || THE_FIX.is_match(sentence) {
            continue;
        }
        let clause = tidy(found.str(1));
        if n_words(&clause) >= 2 {
            return clause;
        }
    }
    for sentence in &sentences {
        if let Some(named) = WHAT_IT_TAKES.search(sentence) {
            if IMPERATIVE.match_(text::strip(sentence)).is_none() {
                let clause = tidy(named.str(1));
                if n_words(&clause) >= 3 {
                    return clause;
                }
            }
        }
    }
    let passes: &[bool] = if strict { &[false] } else { &[false, true] };
    for &instruction_ok in passes {
        for (number, sentence) in sentences.iter().enumerate() {
            let st = text::strip(sentence);
            if RESULT_NUMBER.is_match(sentence) {
                continue;
            }
            if number > 0 && DANGLING.match_(st).is_some() {
                continue;
            }
            if number > 0 && !instruction_ok && INSTRUCTION.is_match(sentence) {
                continue;
            }
            if CONDITIONAL_SENTENCE.match_(st).is_some() {
                continue;
            }
            let trigger = _trigger_in(sentence);
            if !trigger.is_empty() && !THE_FIX.is_match(&trigger) && !(number > 0 && INSTRUCTION.is_match(sentence)) {
                return trigger;
            }
            if !CAUSE.is_match(sentence)
                && CLAIM_BEFORE_COLON.match_(st).is_none()
                && EXPLAINS.is_match(sentence)
                && IMPERATIVE.match_(sentence).is_none()
                && !THE_FIX.is_match(sentence)
                && CONDITIONAL_SENTENCE.match_(st).is_none()
                && !DOSE.is_match(sentence)
                && !_is_the_answer_itself(said, sentence)
            {
                let clause = tidy(sentence);
                if n_words(&clause) >= 3 {
                    return clause;
                }
            }
            if CONDITIONAL_SENTENCE.match_(st).is_some() {
                continue;
            }
            let mut parts: Vec<String> = vec![sentence.clone()];
            for mark in PARTS.iter() {
                parts = parts.iter().flat_map(|p| p.split(mark.as_str()).map(str::to_string).collect::<Vec<_>>()).collect();
            }
            for part in &parts {
                let clause = _cause_clause(part, number, instruction_ok);
                if !clause.is_empty() && !THE_FIX.is_match(&clause) {
                    if strict && _carries_the_fix(sentence, &clause) {
                        return tidy(sentence);
                    }
                    return clause;
                }
                let trigger = _trigger_in(part);
                if !trigger.is_empty() && !THE_FIX.is_match(&trigger) && !(number > 0 && INSTRUCTION.is_match(part)) {
                    return trigger;
                }
            }
        }
    }
    let named_class = _named_by_definition(said);
    if !named_class.is_empty() {
        return named_class;
    }
    _the_class_it_belongs_to(said)
}

/// Python: bot/compose/why.py:946 `_is_the_answer_itself(said, sentence)`
/// True when this sentence is the whole answer, so handing it back explains nothing.
pub fn _is_the_answer_itself(said: &str, sentence: &str) -> bool {
    let whole = _words(said);
    let part = _words(sentence);
    if whole.is_empty() || part.is_empty() {
        return false;
    }
    part.len() >= 3usize.max((whole.len() as f64 * 0.9) as usize)
}

/// Python: bot/compose/why.py:968 `_words(text)`
/// The words of a line, folded, so punctuation and case cannot hide a repeat.
pub fn _words(text_: &str) -> Vec<String> {
    rx(r"[^a-z0-9']+").split(&text::lower(text_)).into_iter().filter(|w| !w.is_empty()).collect()
}

/// Python: bot/compose/why.py:973 `_carries_the_fix(sentence, clause)`
/// True when the sentence says the cause and then the fix after its colon or dash.
pub fn _carries_the_fix(sentence: &str, clause: &str) -> bool {
    let rest = text::strip(sentence);
    let Some(after) = rest.strip_prefix(clause) else { return false };
    match AFTER_THE_MARK.match_(after) {
        Some(found) => IMPERATIVE.match_(text::strip(found.name_str("rest"))).is_some(),
        None => false,
    }
}

/// Python: bot/compose/why.py:982 `_trigger_in(part)`
/// The reason when the answer explains a fault by its trigger, else "".
pub fn _trigger_in(part: &str) -> String {
    let t = text::strip(part);
    if t.is_empty() || IMPERATIVE.match_(t).is_some() || CONDITIONAL.match_(t).is_some() || NEGATIVE.is_match(t) {
        return String::new();
    }
    for m in WHEN_CAUSE.finditer(t) {
        let head = text::strip(&t[..m.start()]);
        if n_words(head) < 2 || !CHANGE.is_match(head) {
            continue;
        }
        let clause = SO_FROM_HERE.split(&t[m.start()..]).into_iter().next().unwrap_or_default();
        let clause = clause.split(": ").next().unwrap_or("");
        let joined = format!("{head} {clause}");
        let clause = text::strip(text::strip(&joined).trim_end_matches(['.', ',', ';', ':'])).to_string();
        let clause = _cut_long(&clause);
        if n_words(&clause) < 4 {
            continue;
        }
        return clause;
    }
    String::new()
}

/// Python: bot/compose/why.py:1013 `_cause_clause(part, number, instruction_ok)`
/// The cause in one part of a sentence, as a clause, or "" when it names none.
pub fn _cause_clause(part: &str, number: usize, instruction_ok: bool) -> String {
    let (found, so_reason): (Match, bool) = match CAUSE.search(part) {
        Some(m) => (m, false),
        None => match SO_REASON.search(part) {
            Some(m) => (m, true),
            None => return String::new(),
        },
    };
    if number > 0 && INSTRUCTION.is_match(part) {
        return String::new();
    }
    if !instruction_ok && IMPERATIVE.match_(text::strip(part)).is_some() {
        return String::new();
    }
    let g0 = text::lower(found.as_str());
    // part[:part.lower().find(found.group(0).lower())], in code points
    let at = text::find(&text::lower(part), &g0);
    if g0 == "because" || g0 == "since" {
        let prefix = text::slice(part, 0, Some(at));
        if NEGATIVE.is_match(&prefix) || CONDITIONAL.is_match(&prefix) {
            return String::new();
        }
    }
    let from_the_marker = g0 == "because" || g0 == "since" || so_reason;
    let clause = if from_the_marker { text::slice(part, at, None) } else { part.to_string() };
    let clause = SO_ADVICE.split(&clause).into_iter().next().unwrap_or_default();
    let clause = AND_STEP.split(&clause).into_iter().next().unwrap_or_default();
    let clause = AND_ADVICE.split(&clause).into_iter().next().unwrap_or_default();
    let clause = LEADING_MARKER.subn("because ", text::strip(&clause), 1);
    let clause = text::strip(&clause).trim_end_matches(['.', ',', ';', ':']);
    let clause = text::strip(clause).to_string();
    let clause = _cut_long(&clause);
    if n_words(&clause) < 3 {
        return String::new();
    }
    let low = text::lower(&clause);
    if !(low.starts_with("because ") || low.starts_with("since ")) && IMPERATIVE.match_(&clause).is_some() {
        return String::new();
    }
    clause
}

/// Python: bot/compose/why.py:1094 `_the_class_it_belongs_to(said)`
/// The class the answer said the thing belongs to, as the reason for it, or "".
pub fn _the_class_it_belongs_to(said: &str) -> String {
    let first_owned = sentences_of(text::strip(said)).into_iter().next().unwrap_or_default();
    let first = text::strip(&first_owned);
    if IMPERATIVE.match_(first).is_some() || CONDITIONAL_SENTENCE.match_(first).is_some() {
        return String::new();
    }
    let Some(found) = A_GENERAL_CLAIM.match_(first) else { return String::new() };
    let what = found.name_str("what");
    let claim = match ENDS_THE_CLASS.search(what) {
        Some(m) => &what[..m.start()],
        None => what,
    };
    let claim = tidy(claim);
    let n = n_words(&claim);
    if n < 1 || n > 14 {
        return String::new();
    }
    if SAYS_WHAT_IT_DOES.is_match(&claim) || rx(r"\d").is_match(&claim) {
        return String::new();
    }
    if A_PLACEHOLDER.contains(&text::lower(&claim)) || THE_FIX.is_match(&claim) {
        return String::new();
    }
    if INSTRUCTION.is_match(&claim) || DOSE.is_match(&claim) {
        return String::new();
    }
    let subject = text::strip(found.name_str("subject"));
    let (start, _) = found.name_span("what").unwrap();
    let lead = text::strip(&first[..start]);
    let lead = if lead.is_empty() { format!("Most {subject}") } else { lead.to_string() };
    format!("{lead} {claim}")
}

/// Python: bot/compose/why.py:1124 `_named_by_definition(said)`
/// The reason when the answer gives it as a definition: "Most headaches are tension: ...".
pub fn _named_by_definition(said: &str) -> String {
    for sentence in sentences_of(said).iter().take(1) {
        let Some(found) = CLAIM_BEFORE_COLON.match_(text::strip(sentence)) else { continue };
        if CLAIM_SUBJECT.match_(sentence).is_some() || IMPERATIVE.match_(text::strip(sentence)).is_some() {
            continue;
        }
        let why = tidy(found.name_str("why"));
        let first = text::slice(&why, 0, Some(1));
        // `why[:1] in "..."` is also true for an empty `why`
        if first.is_empty() || "\"'\u{2018}\u{201c}".contains(first.as_str()) || IMPERATIVE.match_(&why).is_some() || INSTRUCTION.match_(&why).is_some() {
            continue;
        }
        let why = tidy(&SO_ADVICE.split(&why).into_iter().next().unwrap_or_default());
        let why = _cut_long(&why);
        if n_words(&why) < 4 {
            continue;
        }
        return format!("because {}{}", text::lower(&text::slice(&why, 0, Some(1))), text::slice(&why, 1, None));
    }
    String::new()
}

/// Python: bot/compose/why.py:1155 `_cut_long(clause)`
/// The clause at its own comma, or "" when it only ends where the character limit is.
pub fn _cut_long(clause: &str) -> String {
    let words = text::split_ref(clause);
    if words.len() <= 24 {
        return clause.to_string();
    }
    let head = words[..24].join(" ");
    let at = text::rfind(&head, ", ").max(text::rfind(&head, " - "));
    if at < (text::len(&head) / 3) as isize {
        return String::new();
    }
    text::slice(&head, 0, Some(at))
}

/// Python: bot/compose/why.py:1204 `_an_answer_to_act_on(last)`
/// True when the answer above says to act now, so the reason is not read out of it.
pub fn _an_answer_to_act_on(last: Option<&Turn>) -> bool {
    let said = bot_of(last);
    if said.is_empty() {
        return false;
    }
    if ACT_NOW.is_match(said) {
        return true;
    }
    SENTENCE.findall_str(said).iter().any(|s| ACT_NOW_SOON.is_match(s) && !CONDITIONAL.is_match(s))
}

/// Python: bot/compose/why.py:1318 `_first_sentence(said)`
/// The answer's own opening sentence, or "" when it has only one.
pub fn _first_sentence(said: &str) -> String {
    let said = text::squash(said);
    if said.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = END_OF_SENTENCE.split(&said).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return String::new();
    }
    parts[0].trim_end_matches([',', ';', ':', '-', ' ']).to_string()
}

/// Python: bot/compose/why.py:1329 `_a_shorter_way(said)`
/// The answer above said shorter, or "" when there is nothing in it to cut down to.
pub fn _a_shorter_way(said: &str) -> String {
    let said = text::squash(said);
    if said.is_empty() {
        return String::new();
    }
    let lead = _past_a_lead_in(&said);
    let first = _first_sentence(&lead);
    if !first.is_empty() {
        let doing = _the_doing_of(&first);
        if !doing.is_empty() {
            return doing;
        }
        let clause = _a_clause_of(&first);
        if n_words(&first) > *_A_LONG_SENTENCE {
            if !clause.is_empty() && n_words(&clause) * *_A_STRIDE <= n_words(&first) {
                return clause;
            }
            if lead != text::squash(&said) {
                return _end_a_sentence(&first);
            }
            return String::new();
        } else if !clause.is_empty() && n_words(&clause) * 2 >= n_words(&first) {
            return clause;
        }
        if n_words(&first) <= *AGAIN_WORDS {
            return _end_a_sentence(&first);
        }
        return clause;
    }
    _a_clause_of(&_past_a_lead_in(&said))
}

/// Python: bot/compose/why.py:1389 `_past_a_lead_in(said)`
/// The answer without a lead-in sentence in front of it, which is a label and not the answer.
pub fn _past_a_lead_in(said: &str) -> String {
    let parts: Vec<String> = END_OF_SENTENCE.split(said).iter().map(|p| text::strip(p).to_string()).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return said.to_string();
    }
    if n_words(&parts[0]) >= *_A_CLAUSE_LEAST + 2 {
        return said.to_string();
    }
    text::strip(&text::slice(said, text::len(&parts[0]) as isize, None)).to_string()
}

/// Python: bot/compose/why.py:1405 `_end_a_sentence(text)`
/// The sentence with its full stop, so the frame in front of it reads as a sentence.
pub fn _end_a_sentence(text_: &str) -> String {
    let t = text::strip(text_).trim_end_matches([',', ';', ':', '-', ' ']);
    if t.is_empty() {
        return String::new();
    }
    if t.ends_with(['.', '!', '?']) { t.to_string() } else { format!("{t}.") }
}

/// `cut.strip().rstrip(",;:-— ")` for a cut at a clause end.
fn cut_at(sentence: &str, m: &Match) -> String {
    text::strip(&sentence[..m.start()]).trim_end_matches([',', ';', ':', '-', '\u{2014}', ' ']).to_string()
}

/// Python: bot/compose/why.py:1413 `_a_clause_of(sentence)`
/// The first real clause of one long sentence, or "" when it has none worth handing over.
pub fn _a_clause_of(sentence: &str) -> String {
    let sentence = text::strip(sentence);
    for m in CLAUSE_END.finditer(sentence) {
        let cut = cut_at(sentence, &m);
        if n_words(&cut) < *_A_CLAUSE_LEAST + 2 {
            continue;
        }
        if !_brackets_closed(&cut) {
            continue;
        }
        if !_carries_a_verb(&cut) || _leans_back(&cut) {
            continue;
        }
        if _points_at_what_follows(&cut) {
            continue;
        }
        return _end_a_sentence(&cut);
    }
    String::new()
}

/// Python: bot/compose/why.py:1441 `_the_doing_of(sentence)`
/// The half after the sentence's colon when that half is what to do, or "" otherwise.
pub fn _the_doing_of(sentence: &str) -> String {
    let sentence = text::strip(sentence);
    for m in CLAUSE_END.finditer(sentence) {
        let cut = cut_at(sentence, &m);
        if n_words(&cut) < *_A_CLAUSE_LEAST + 2 {
            continue;
        }
        if !_brackets_closed(&cut) {
            continue;
        }
        if _points_at_what_follows(&cut) {
            continue;
        }
        return _end_a_sentence(&_the_half_that_does_it(&cut, &sentence[m.end()..]));
    }
    String::new()
}

/// Python: bot/compose/why.py:1497 `_opens_a_doing(half)`
/// True when the half opens on a thing to do rather than on a thing to know.
pub fn _opens_a_doing(half: &str) -> bool {
    let words = rx("[a-z']+").findall_str(&text::lower(half));
    let Some(first) = words.first() else { return false };
    if _A_DOING.fullmatch(first).is_some() {
        return true;
    }
    if _A_LEAD_IN.contains(first) {
        let end = (*_A_DOING_WINDOW + 2).min(words.len());
        return words[1.min(words.len())..end].iter().any(|w| _A_DOING.fullmatch(w).is_some());
    }
    false
}

/// Python: bot/compose/why.py:1526 `_says_how_it_works(half)`
/// True when a colon half is a clause about the thing rather than a list of the things.
pub fn _says_how_it_works(half: &str) -> bool {
    let first = rx("[,;]").split(half).into_iter().next().unwrap_or_default();
    _SAYS_HOW.is_match(&first) || _VERB_SHAPE.is_match(&first)
}

/// Python: bot/compose/why.py:1540 `_announces_the_list(half)`
/// True when a half is a heading for the list in the other half rather than the doing itself.
pub fn _announces_the_list(half: &str) -> bool {
    _ANNOUNCES.is_match(half)
}

/// Python: bot/compose/why.py:1553 `_needs_its_subject(after)`
/// True when a half cannot stand alone as a short form: it opens on a pronoun or a participle.
pub fn _needs_its_subject(after: &str) -> bool {
    let words = text::split_ref(after);
    let Some(first) = words.first() else { return true };
    if _NEEDS_A_SUBJECT.match_(first).is_some() {
        return true;
    }
    let first = rx("[^a-z]").sub("", &text::lower(first));
    _A_PARTICIPLE.contains(&first) && !["set", "left", "put", "lost"].contains(&first.as_str())
}

/// Python: bot/compose/why.py:1581 `_carries_a_verb(half)`
/// True when a half is a sentence rather than a list of places or things.
pub fn _carries_a_verb(half: &str) -> bool {
    if text::strip(half).is_empty() {
        return false;
    }
    if _A_DOING.is_match(half) || _VERB_SHAPE.is_match(half) {
        return true;
    }
    if rx("[a-z']+").findall_str(&text::lower(half)).iter().any(|w| _A_VERB.contains(w)) {
        return true;
    }
    if _A_FIGURE.is_match(half) {
        return true;
    }
    _A_CAUSE_OPENS.match_(half).is_some()
}

/// Python: bot/compose/why.py:1613 `_leans_back(half)`
/// True when a half needs the half before it to mean anything.
pub fn _leans_back(half: &str) -> bool {
    let words: Vec<&str> = text::split_ref(half).into_iter().take(*_LEANS_BACK_WINDOW).collect();
    _LEANS_BACK.is_match(&words.join(" "))
}

/// Python: bot/compose/why.py:1637 `_the_reason(after)`
/// True when the half after the colon justifies the half before it rather than completing it.
pub fn _the_reason(after: &str) -> bool {
    if n_words(after) > *_A_REASON_LEAST || after.contains(',') || after.contains(';') {
        return false;
    }
    _carries_a_verb(after)
}

/// Python: bot/compose/why.py:1652 `_opens_on_a_figure(after)`
/// True when the half after the colon opens on an amount or a count.
pub fn _opens_on_a_figure(after: &str) -> bool {
    _OPENS_ON_A_FIGURE.match_(after).is_some()
}

/// Python: bot/compose/why.py:1668 `_names_no_particular(before)`
/// True when the opening is general enough that only the colon half says what to do.
pub fn _names_no_particular(before: &str) -> bool {
    _A_CATEGORY.is_match(before)
}

/// `after[:1].upper() + after[1:]`.
fn capital(s: &str) -> String {
    format!("{}{}", text::upper(&text::slice(s, 0, Some(1))), text::slice(s, 1, None))
}

/// Python: bot/compose/why.py:1673 `_the_half_that_does_it(before, after)`
/// The half of a colon sentence a person who did not follow it needs, or "" for the opening.
pub fn _the_half_that_does_it(before: &str, after: &str) -> String {
    let before = text::strip(before);
    let after = text::strip(after).trim_end_matches([',', ';', ':', '-', ' ']).to_string();
    if n_words(&after) < *_A_CLAUSE_LEAST / 2 {
        return String::new();
    }
    if !_carries_a_verb(&after) || _leans_back(&after) {
        return String::new();
    }
    if _needs_its_subject(&after) {
        return String::new();
    }
    if _opens_a_doing(before) && !_announces_the_list(before) {
        if _opens_a_doing(&after) {
            if n_words(&after) >= n_words(before) {
                return capital(&after);
            }
            return String::new();
        }
        if _the_reason(&after) || _opens_on_a_figure(&after) {
            return String::new();
        }
        if _A_FIGURE.is_match(before) && !_opens_on_a_figure(&after) {
            return String::new();
        }
        if !_names_no_particular(before) {
            return String::new();
        }
        if _says_how_it_works(&after) {
            return String::new();
        }
        return capital(&after);
    }
    if _opens_a_doing(&after) {
        if _A_FIGURE.is_match(before) && !_opens_on_a_figure(before) && !_names_no_particular(before) {
            return String::new();
        }
        return capital(&after);
    }
    if _EXPLAINS.match_(&after).is_some() {
        if _A_FIGURE.is_match(before) && !_opens_on_a_figure(&after) {
            return String::new();
        }
        if !_names_no_particular(before) {
            return String::new();
        }
        if _the_reason(&after) {
            return String::new();
        }
        if _carries_a_verb(before) {
            return capital(&after);
        }
        return String::new();
    }
    if !_A_FIGURE.is_match(before) && _A_FIGURE.is_match(&after) {
        return capital(&after);
    }
    String::new()
}

/// Python: bot/compose/why.py:1769 `_points_at_what_follows(cut)`
/// True when a cut is too short and too vague to be anything but a heading.
pub fn _points_at_what_follows(cut: &str) -> bool {
    let cut = text::strip(cut);
    if _POINTS_FORWARD_ANY.is_match(cut) {
        return true;
    }
    _POINTS_FORWARD.is_match(cut) && n_words(cut) < *_POINTS_FORWARD_LEAST
}

/// Python: bot/compose/why.py:1777 `_brackets_closed(text)`
/// True when every bracket in `text` is opened and closed in it.
pub fn _brackets_closed(text_: &str) -> bool {
    text_.matches('(').count() == text_.matches(')').count() && text_.matches('[').count() == text_.matches(']').count()
}

/// Python: bot/compose/why.py:1783 `_the_route_is_in_it(said)`
/// True when the answer names the way out of it, so no part of it can be handed back alone.
pub fn _the_route_is_in_it(said: &str) -> bool {
    NO_CUT.is_match(said)
}

/// Python: bot/compose/why.py:1788 `said_again(clean, memory)`
/// The reply to a bare "say that again", or None when the line is not one this reads. (The
/// skeleton's `()` return could not carry the reply.)
pub fn said_again(clean: &str, memory: &Memory) -> Option<String> {
    if SAID_AGAIN.match_(clean).is_none() {
        return None;
    }
    if _an_answer_to_act_on(_last_turn(memory)) {
        return None;
    }
    let short = _not_shortened(&_said_above(memory));
    if short.is_empty() {
        return None;
    }
    Some(pick(&AGAIN_SHORTER, memory, clean).replace("{short}", &short))
}

/// Python: bot/compose/why.py:1809 `is_the_answer_shorter(reply, last)`
/// True when `reply` is the last answer's own words, shorter, in this module's frame.
pub fn is_the_answer_shorter(reply: &str, last: &str) -> bool {
    let t = text::strip(reply);
    let mut short = "";
    for lead in REPEAT_LEADS.iter() {
        if let Some(rest) = t.strip_prefix(lead.as_str()) {
            short = rest;
            break;
        }
    }
    if short.is_empty() {
        return false;
    }
    let words = rx("[a-z0-9']+").findall_str(&text::lower(short));
    let previous = rx("[a-z0-9']+").findall_str(&text::lower(last));
    if words.len() < 3 || previous.len() < words.len() + 4 {
        return false;
    }
    let size = words.len();
    (0..=previous.len() - size).any(|start| previous[start..start + size] == words[..])
}

/// Python: bot/compose/why.py:1836 `_said_above(memory)`
/// The answer the person is asking about, or "" when there is not one.
pub fn _said_above(memory: &Memory) -> String {
    _unframed(bot_of(_last_turn(memory)))
}

/// Python: bot/compose/why.py:1847 `_unframed(reply)`
/// The answer under a repeat's lead-in ("From just now: ...", "You've got it: ...").
pub fn _unframed(reply: &str) -> String {
    for lead in REPEAT_LEADS.iter() {
        if let Some(rest) = reply.strip_prefix(lead.as_str()) {
            if !text::strip(rest).is_empty() {
                return text::strip(rest).to_string();
            }
        }
    }
    if let Some(found) = ANOTHER_FRAME.match_(reply) {
        let rest = text::strip(&reply[found.end()..]);
        if !rest.is_empty() {
            return rest.to_string();
        }
    }
    crate::memory::again::unframed(reply).filter(|s| !s.is_empty()).unwrap_or_else(|| reply.to_string())
}

/// Python: bot/compose/why.py:1862 `_not_shortened(said)`
/// The answer said shorter, or "" when the answer is not this module's to shorten.
pub fn _not_shortened(said: &str) -> String {
    if n_words(said) < *ANSWER_LONG {
        return String::new();
    }
    let first = _first_sentence(said);
    if CARE_ANSWER.is_match(said) || OPENS_ON_CARE.match_(if first.is_empty() { said } else { &first }).is_some() {
        return String::new();
    }
    if _the_route_is_in_it(said) {
        return String::new();
    }
    let short = _a_shorter_way(said);
    let mut echo = _first_sentence(&_past_a_lead_in(said));
    if echo.is_empty() {
        echo = said.to_string();
    }
    let doing = _end_a_sentence(&_the_doing_of(&echo));
    if !doing.is_empty() && short == doing {
        echo = said.to_string();
    }
    let saved: i64 = if short.is_empty() { 0 } else { n_words(&echo) as i64 - n_words(&short) as i64 };
    if _past_a_lead_in(said) == text::squash(said) && saved < (*_A_SAVE_LEAST as i64).max(n_words(&echo) as i64 / 4) {
        return String::new();
    }
    short
}

/// Python: bot/compose/why.py:1885 `takes_as_it_comes(text, memory)`
/// Read by the arbiter: on a bare "why" after an answer, the reason is this module's turn.
pub fn takes_as_it_comes(text_: &str, memory: &Memory) -> bool {
    let clean = text::squash(&text::lower(text_));
    if SAID_AGAIN.match_(&clean).is_some() {
        if _an_answer_to_act_on(_last_turn(memory)) {
            return false;
        }
        return !_not_shortened(&_said_above(memory)).is_empty();
    }
    if !_is_bare_why(&clean) {
        return false;
    }
    let last = _last_turn(memory);
    if _an_answer_to_act_on(last) {
        return false;
    }
    if _said_the_reason_cannot_be_seen(memory) {
        return true;
    }
    n_words(bot_of(last)) >= *ANSWER_LONG
}

/// Python: bot/compose/why.py:2030 `asks_about_the_last_answer(clean)`
/// True when the line asks about the words the bot just used ("why do you say that").
pub fn asks_about_the_last_answer(clean: &str) -> bool {
    WHY_YOU_SAID_IT.match_(text::strip(clean)).is_some()
}

/// Python: bot/compose/why.py:2035 `respond_to_a_saying_ask(clean, memory, understood=None, body=False)`
/// The reply to "why do you say that", or None when the line is not one of those. (The
/// skeleton's `()` return could not carry the reply.)
pub fn respond_to_a_saying_ask(clean: &str, memory: &Memory, understood: &Understood, body: bool) -> Option<String> {
    if THINKS_BACK.match_(text::strip(clean)).is_some() {
        if _the_turn_above(memory).is_none_or(|t| t.bot.is_empty()) {
            return None;
        }
        return respond(clean, memory, understood, body);
    }
    if !asks_about_the_last_answer(clean) {
        return None;
    }
    let above = _the_turn_above(memory);
    let said = above.map(|t| t.user.as_str()).unwrap_or("");
    if ABOUT_THE_BOT_LINE.match_(said).is_some() {
        return Some(pick(&WHY_THE_BOT, memory, clean));
    }
    let above_bot = above.map(|t| t.bot.as_str()).unwrap_or("");
    if CARE_ANSWER.is_match(above_bot) {
        return Some(pick(&WHY_CARE, memory, clean));
    }
    if THE_BOT_DESCRIBED_ITSELF.is_match(above_bot) {
        return Some(pick(&WHY_THE_BOT, memory, clean));
    }
    let last = _last_turn(memory)?;
    if _reason_in(Some(last), false).is_empty() {
        use crate::compose::fallback::PyOptStr;
        return crate::memory::topics::said_again(above_bot, memory, "").py_opt_str();
    }
    respond("why", memory, understood, body)
}

/// Python: bot/compose/why.py:2091 `_the_turn_above(memory)`
/// The last turn, whatever module answered it, or None (Python's `{}`) when there is none.
pub fn _the_turn_above(memory: &Memory) -> Option<&Turn> {
    memory.turns.last()
}

/// Python: bot/compose/why.py:2103 `respond(clean, memory, understood=None, body=False)`
/// The reply to a bare "why" follow-up, or None when the line is not one. (The skeleton's `()`
/// return could not carry the reply.)
pub fn respond(clean: &str, memory: &Memory, understood: &Understood, body: bool) -> Option<String> {
    let _ = understood; // passed on by Python, read by nothing here
    if let Some(again) = said_again(clean, memory) {
        if !again.is_empty() {
            return Some(again);
        }
    }
    let thinks_back = THINKS_BACK.match_(text::strip(clean)).is_some();
    if !(_is_bare_why(clean) || thinks_back) {
        return None;
    }
    let above_bot = _the_turn_above(memory).map(|t| t.bot.clone()).unwrap_or_default();
    let Some(last) = _last_turn(memory) else {
        if THE_BOT_DESCRIBED_ITSELF.is_match(&above_bot) {
            return Some(pick(&WHY_THE_BOT, memory, clean));
        }
        if _said_the_reason_cannot_be_seen(memory) {
            return Some(pick(&WHY_AGAIN, memory, clean));
        }
        return None;
    };
    let last = Some(last);
    let act_now = _an_answer_to_act_on(last);
    let whole_thing = WRONG.is_match(clean) || CAUSING_IT.is_match(clean);
    let reason = if WHY_NOT.is_match(clean) {
        let reason = if act_now { String::new() } else { _a_cause_named_outright(last) };
        if reason.is_empty() {
            if _about_someone_elses_decision(memory) {
                return Some(pick(&WHY_NOT_ANSWERED, memory, clean));
            }
            if _the_suggestion_was_theirs(memory) {
                return Some(pick(&WHY_NOT_ITS_OWN, memory, clean));
            }
            return Some(pick(&WHY_NOT_ANSWERED, memory, clean));
        }
        reason
    } else if act_now {
        String::new()
    } else {
        _reason_in(last, whole_thing)
    };
    if !reason.is_empty() {
        let sliced = text::lower(&reason).starts_with("because ");
        let said = if sliced { text::slice(&reason, "because ".len() as isize, None) } else { reason.clone() };
        let pool: &[String] = if _named_as_a_sentence(last, &said) {
            &WHY_NAMED_SELF
        } else if sliced {
            &WHY_NAMED_BECAUSE
        } else {
            &WHY_NAMED
        };
        let mut reply = _in_frame(&pick(pool, memory, clean), &said);
        if !reply.ends_with(['.', '!', '?']) {
            reply.push('.');
        }
        return Some(capital(&reply));
    }
    if thinks_back && !act_now {
        use crate::compose::fallback::PyOptStr;
        if let Some(handed_back) = crate::memory::topics::said_again(&above_bot, memory, "").py_opt_str() {
            if !handed_back.is_empty() {
                return Some(handed_back);
            }
        }
    }
    if body {
        return Some(pick(&WHY_BODY, memory, clean));
    }
    if !act_now && whole_thing && _gave_a_method(last) && _reads_a_fault(last) {
        return Some(pick(&WHY_METHOD, memory, clean));
    }
    if whole_thing {
        let pool: &[String] = if _reads_a_fault(last) { &WRONG_ANSWERED } else { &WRONG_NOT_A_FAULT };
        return Some(pick(pool, memory, clean));
    }
    Some(pick(&WHY_ANSWERED, memory, clean))
}
