//! Port of `bot/compose/lead.py`: the answer leads. A signpost sentence ("Three tests, in this
//! order.") is dropped, and the sentence that answers the wanted kind of question (the rate,
//! the verdict, the cause, the time) is moved to the front. Nothing is reworded, only moved.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, bundle_res, bundle_sets, inline_re};
use crate::compose::fit;
use crate::data::Value;
use crate::py::re::Re;
use crate::py::text;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

bundle_res!("bot.compose.lead"; _TOTAL, _OPENING, _SIGNPOST, _SENTENCE_END, _ABBREVIATION_END, _ASKS_HOW_OFTEN, _A_RATE,
    _VERDICT_OPENS, _THINKING_IT_OVER, _CAUSE_SENTENCE, _SAID_WORD, _POINTS_BACKWARDS, _OPENS_WITH_SYMPATHY,
    _OPENS_WITH_A_WARNING);
bundle_sets!("bot.compose.lead"; _METHOD, _HARD, _LIFTABLE, _SHARED_WORDS);
inline_re!(A_LONG_WORD, r"[a-z]{4,}");
inline_re!(A_DIGIT, r"\d");
inline_re!(WHEN, r"\bwhen\b", crate::py::re::I);

static YESNO: LazyLock<String> = LazyLock::new(|| crate::data::string("bot.compose.fit.YESNO"));
static REASON: LazyLock<String> = LazyLock::new(|| crate::data::string("bot.compose.fit.REASON"));
static TIME: LazyLock<String> = LazyLock::new(|| crate::data::string("bot.compose.fit.TIME"));
static AMOUNT: LazyLock<String> = LazyLock::new(|| crate::data::string("bot.compose.fit.AMOUNT"));
static CLOCK: LazyLock<&'static Re> = LazyLock::new(|| crate::data::re("bot.compose.fit.CLOCK"));

/// `understood.get("wants")`, stripped and lowered when it is a str.
fn wants(understood: Option<&Understood>) -> Option<String> {
    understood?.wants.as_ref().map(|w| text::strip(w).to_lowercase())
}

/// Python: bot/compose/lead.py:80 `_sentences(reply)`
///
/// The reply's sentences, in order, with their own punctuation kept.
pub fn _sentences(reply: &str) -> Vec<String> {
    let t = text::squash(reply);
    if t.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    let mut start = 0;
    for end in _SENTENCE_END.finditer(&t) {
        let before = &t[start..end.start()];
        if _ABBREVIATION_END.search(before).is_some() {
            continue;
        }
        out.push(text::strip(before).to_string());
        start = end.end();
    }
    let rest = text::strip(&t[start..]);
    if !rest.is_empty() {
        out.push(rest.to_string());
    }
    out.into_iter().filter(|s| !s.is_empty()).collect()
}

/// Python: bot/compose/lead.py:103 `_wants_the_method(understood)`
pub fn _wants_the_method(understood: Option<&Understood>) -> bool {
    wants(understood).is_some_and(|w| _METHOD.contains(&w))
}

/// Python: bot/compose/lead.py:127 `rate_first(reply, understood, line=None)`
///
/// The rate sentence leads when the line asked how often and the row puts it second. Only a
/// sentence that opens on the rate is moved.
pub fn rate_first(reply: &str, understood: Option<&Understood>, line: &str) -> String {
    let _ = understood;
    if text::strip(reply).is_empty() || _ASKS_HOW_OFTEN.search(line).is_none() {
        return reply.to_string();
    }
    let parts = _sentences(reply);
    if parts.len() < 2 || _A_RATE.search(&parts[0]).is_some() {
        return reply.to_string();
    }
    for at in 1..parts.len() {
        let sentence = &parts[at];
        if _A_RATE.match_(sentence).is_some() && text::split_ref(sentence).len() <= 22 {
            let mut moved = vec![sentence.clone()];
            moved.extend(parts[..at].iter().cloned());
            moved.extend(parts[at + 1..].iter().cloned());
            return moved.join(" ");
        }
    }
    reply.to_string()
}

/// Python: bot/compose/lead.py:157 `answer_first(reply, understood, line=None)`
///
/// The answer leads: a sentence that only announces the answer is not the answer.
pub fn answer_first(reply: &str, understood: Option<&Understood>, line: &str) -> String {
    if text::strip(reply).is_empty() {
        return reply.to_string();
    }
    // A line that asks how often gets the sentence that says how often, in front.
    let mut reply = rate_first(reply, understood, line);
    let parts = _sentences(&reply);
    if parts.len() > 1 {
        let first = text::strip(text::rstrip_chars(&parts[0], ".!? "));
        if !first.is_empty() && _SIGNPOST.match_(first).is_some() {
            reply = parts[1..].join(" ");
        }
    }
    // A yes-or-no ask read out gets the verdict first.
    reply = verdict_first(&reply, understood, line);
    if _wants_the_method(understood) {
        return method_first(&reply, understood);
    }
    cause_first(&reply, understood)
}

/// Python: bot/compose/lead.py:213 `verdict_first(reply, understood, line=None)`
///
/// The verdict leads when the line asked yes or no and the card gave it later; the move is made
/// only when the shape check can see the opening sentence is not a yes or no.
pub fn verdict_first(reply: &str, understood: Option<&Understood>, line: &str) -> String {
    if text::strip(reply).is_empty() || _the_want(understood) != *YESNO {
        return reply.to_string();
    }
    let parts = _sentences(reply);
    if parts.len() < 2 {
        return reply.to_string();
    }
    let bare_opening = text::strip(text::rstrip_chars(&parts[0], ".!? "));
    if _VERDICT_OPENS.match_(bare_opening).is_some() {
        return reply.to_string();
    }
    for place in 1..parts.len() {
        let sentence = &parts[place];
        let bare = text::strip(text::rstrip_chars(sentence, ".!? "));
        if bare.is_empty() || _VERDICT_OPENS.match_(bare).is_none() {
            continue;
        }
        if _THINKING_IT_OVER.search(bare).is_some() {
            // The reply turning the question back is not the verdict on it.
            continue;
        }
        match com4::attempt(|| fit::gives(bare, &YESNO, Some(line))) {
            Some(Some(false)) | None => continue,
            _ => {}
        }
        let rest: Vec<String> = parts[..place].iter().chain(parts[place + 1..].iter()).cloned().collect();
        return format!("{sentence} {}", rest.join(" "));
    }
    reply.to_string()
}

/// Python: bot/compose/lead.py:256 `method_first(reply, understood)`
///
/// The reply with its total time moved to the end when the line asked for the method.
pub fn method_first(reply: &str, understood: Option<&Understood>) -> String {
    if text::strip(reply).is_empty() || !_wants_the_method(understood) {
        return reply.to_string();
    }
    let Some(found) = _OPENING.match_(reply) else { return reply.to_string() };
    let (opening, rest) = (text::strip(found.str(1)), text::strip(found.str(2)));
    if _TOTAL.match_(opening).is_none() || rest.is_empty() {
        return reply.to_string();
    }
    format!("{rest} {opening}")
}

/// Python: bot/compose/lead.py:290 `_wants_the_cause(understood)`
pub fn _wants_the_cause(understood: Option<&Understood>) -> bool {
    wants(understood).is_some_and(|w| w == *REASON)
}

/// Python: bot/compose/lead.py:299 `cause_first(reply, understood)`
///
/// The cause sentence leads when the line asked why and the card puts it last.
pub fn cause_first(reply: &str, understood: Option<&Understood>) -> String {
    if text::strip(reply).is_empty() || !_wants_the_cause(understood) {
        return reply.to_string();
    }
    let parts = _sentences(reply);
    if parts.len() < 2 {
        return reply.to_string();
    }
    for index in 1..parts.len() {
        if _CAUSE_SENTENCE.match_(&parts[index]).is_some() {
            let mut moved = vec![parts[index].clone()];
            moved.extend(parts[..index].iter().cloned());
            moved.extend(parts[index + 1..].iter().cloned());
            return moved.join(" ");
        }
    }
    reply.to_string()
}

/// Python: bot/compose/lead.py:339 `_the_want(understood)`
pub fn _the_want(understood: Option<&Understood>) -> String {
    wants(understood).unwrap_or_default()
}

/// Python: bot/compose/lead.py:348 `_the_opening(reply)`
pub fn _the_opening(reply: &str) -> String {
    fit::first_sentence(reply)
}

/// Python: bot/compose/lead.py:352 `_already_said(lead_said, opening)`
///
/// True when the opening sentence already says the declared one, in its own words.
pub fn _already_said(lead_said: &str, opening: &str) -> bool {
    let mine: Vec<String> = _SAID_WORD.findall_str(&lead_said.to_lowercase()).into_iter().filter(|w| text::len(w) > 2).collect();
    if mine.is_empty() {
        return text::strip_chars(&lead_said.to_lowercase(), " .,!?") == _SAID_WORD.findall_str(&opening.to_lowercase()).join(" ");
    }
    let there: HashSet<String> = _SAID_WORD.findall_str(&opening.to_lowercase()).into_iter().collect();
    mine.iter().all(|w| there.contains(w))
}

/// Python: bot/compose/lead.py:361 `_the_own_sentence_holding(parts, bare)`
///
/// The place of the reply's own sentence that says the declared words, or None.
pub fn _the_own_sentence_holding(parts: &[String], bare: &str) -> Option<usize> {
    let mine: Vec<String> = _SAID_WORD.findall_str(&bare.to_lowercase()).into_iter().filter(|w| text::len(w) > 2).collect();
    if mine.is_empty() {
        return None;
    }
    parts.iter().position(|part| {
        let there: HashSet<String> = _SAID_WORD.findall_str(&part.to_lowercase()).into_iter().collect();
        mine.iter().all(|w| there.contains(w))
    })
}

/// Run an entry's `lead` callable (`said(understood)`) by its Python id. The knowledge modules
/// that have one export a `LEADS` table; an id none of them holds is a lead not ported yet.
fn run_lead(f: &crate::data::FnRef, understood: &Understood) -> String {
    for table in [crate::knowledge::garden::LEADS, crate::knowledge::science::LEADS] {
        if let Some((_, run)) = table.iter().find(|(id, _)| *id == f.id) {
            return run(f, understood);
        }
    }
    // The leads of the other domains, which keep no `LEADS` table of their own.
    match f.id.as_str() {
        "bot.knowledge.consumer:_change_of_mind_verdict:1464" => return crate::knowledge::consumer::_change_of_mind_verdict(understood),
        "bot.knowledge.health:_the_pregnancy_verdict:329" => return crate::knowledge::health::_the_pregnancy_verdict(understood),
        "bot.knowledge.money:_the_assessment_sentence:11454" => return crate::knowledge::money::_the_assessment_sentence(understood),
        "bot.knowledge.travel:_the_passport_verdict:7878" => return crate::knowledge::travel::_the_passport_verdict(understood),
        _ => {}
    }
    // An id none of the tables holds is a lead nobody has ported: the runtime counts it so.
    panic!("UNPORTED lead {}", f.id)
}

/// Python: bot/compose/lead.py:379 `the_sentence_owed(reply, understood, declared=None)`
///
/// The sentence the line asked for, put in front, when the row declares one for its shape
/// (`declared` is the row's `lead`: `(shape, sentence)` or `(shape, callable)`).
pub fn the_sentence_owed(reply: &str, understood: Option<&Understood>, declared: &Value) -> String {
    if text::strip(reply).is_empty() {
        return reply.to_string();
    }
    let pair = match declared {
        Value::Tuple(v) | Value::List(v) if v.len() == 2 => v,
        _ => return reply.to_string(),
    };
    let shape = pair[0].as_str().map(|s| text::strip(s).to_lowercase()).unwrap_or_default();
    if !_HARD.contains(&shape) || _the_want(understood) != shape {
        return reply.to_string();
    }
    let said: String = match &pair[1] {
        Value::Fn(f) => {
            // A row whose key is asked from both sides reads the line and says which sentence it
            // owes. Anything it cannot answer with is no sentence at all.
            let reading = understood.cloned().unwrap_or_default();
            match com4::attempt(|| run_lead(f, &reading)) {
                Some(s) => s,
                None => return reply.to_string(),
            }
        }
        Value::Str(s) => s.to_string(),
        _ => return reply.to_string(),
    };
    if text::strip(&said).is_empty() {
        return reply.to_string();
    }
    let bare = text::rstrip_chars(&text::squash(&said), " .!?").to_string();
    if bare.is_empty() {
        return reply.to_string();
    }
    let opening = _the_opening(reply);
    if text::lstrip(reply).to_lowercase().starts_with(&bare.to_lowercase()) {
        return reply.to_string();
    }
    if _already_said(&bare, &opening) {
        // the card opens on the same answer in its own words
        return reply.to_string();
    }
    let line = understood.map(|u| if u.text.is_empty() { u.clean.clone() } else { u.text.clone() }).unwrap_or_default();
    match com4::attempt(|| fit::gives(&bare, &shape, Some(&line))) {
        Some(Some(false)) | None => return reply.to_string(),
        _ => {}
    }
    // The row already says this sentence, later on: that sentence moves to the front, whole.
    let parts = _sentences(reply);
    if let Some(at) = _the_own_sentence_holding(&parts, &bare) {
        if at > 0 {
            return parts[at..].iter().chain(parts[..at].iter()).cloned().collect::<Vec<_>>().join(" ");
        }
    }
    format!("{bare}. {}", text::lstrip(reply))
}

/// Python: bot/compose/lead.py:493 `_words(text)`
pub fn _words(text_: &str) -> HashSet<String> {
    A_LONG_WORD.findall_str(&text_.to_lowercase()).into_iter().filter(|w| !_SHARED_WORDS.contains(w)).collect()
}

/// Python: bot/compose/lead.py:498 `_names_the_line(sentence, line)`
///
/// True when the sentence shares a word with the line that asked.
pub fn _names_the_line(sentence: &str, line: &str) -> bool {
    if line.is_empty() {
        return true;
    }
    let asked = _words(line);
    _words(sentence).iter().any(|w| asked.contains(w))
}

/// Python: bot/compose/lead.py:506 `_sentence_stands_alone(sentence)`
///
/// True when a sentence reads as an answer with nothing before it.
pub fn _sentence_stands_alone(sentence: &str) -> bool {
    let t = text::squash(sentence);
    if text::split_ref(&t).len() < 4 || _POINTS_BACKWARDS.match_(&t).is_some() {
        return false;
    }
    !t.ends_with('?')
}

/// Python: bot/compose/lead.py:516 `shape(reply, understood, line=None)`
///
/// The reply with the sentence that answers the wanted kind of question moved to the front.
pub fn shape(reply: &str, understood: Option<&Understood>, line: &str) -> String {
    if text::strip(reply).is_empty() {
        return reply.to_string();
    }
    let Some(want) = wants(understood) else { return reply.to_string() };
    if _METHOD.contains(&want) {
        return method_first(reply, understood);
    }
    if !_LIFTABLE.contains(&want) {
        return reply.to_string();
    }
    let parts = _sentences(reply);
    if parts.len() < 2 {
        return reply.to_string();
    }
    // The opening already answers the question, or opens on a shrug: nothing to lift.
    if fit::gives(&parts[0], &want, Some(line)) == Some(true) || fit::is_a_punt(&parts[0]) {
        return reply.to_string();
    }
    // An opening that names the thing asked about and carries a figure is an answer the check
    // may have no unit for.
    if A_DIGIT.search(&parts[0]).is_some() && _names_the_line(&parts[0], line) {
        return reply.to_string();
    }
    if _OPENS_WITH_SYMPATHY.search(&parts[0]).is_some() || _OPENS_WITH_A_WARNING.search(&parts[0]).is_some() {
        return reply.to_string();
    }
    // A "when" is answered by a moment, not by a length of time.
    let when_asked = !line.is_empty() && WHEN.search(line).is_some();
    for index in 1..parts.len() {
        let sentence = &parts[index];
        if when_asked && CLOCK.search(sentence).is_none() {
            continue;
        }
        // A sentence that warns is never lifted to the head either.
        if _OPENS_WITH_A_WARNING.search(sentence).is_some() {
            continue;
        }
        if !_sentence_stands_alone(sentence) {
            continue;
        }
        // A duration or an amount answers only the clock or the amount it was asked about.
        if want != *TIME && want != *AMOUNT && (fit::_a_time(sentence) || fit::_an_amount(sentence)) {
            continue;
        }
        if fit::gives(sentence, &want, Some(line)) != Some(true) {
            continue;
        }
        if !_names_the_line(sentence, line) {
            continue;
        }
        let mut moved = vec![sentence.clone()];
        moved.extend(parts[..index].iter().cloned());
        moved.extend(parts[index + 1..].iter().cloned());
        return moved.join(" ");
    }
    reply.to_string()
}
