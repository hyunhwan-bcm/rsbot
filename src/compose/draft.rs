//! Port of `bot/compose/draft.py`: after the bot offered to write a message, the person's next
//! line turned into the message itself ("saying im off sick tomorrow" -> "Hi, I'm off sick
//! tomorrow. Thanks."), and the readings that keep news, a mood or a complaint out of it.
#![allow(non_upper_case_globals)]

use crate::compose::badnews::{reads_as_bad_news, reads_as_pregnancy_bleeding, reads_as_sudden_loss, reads_like_a_symptom};
use crate::compose::com4::{bundle_res, bundle_sets, bundle_strs, bundle_vals, inline_re};
use crate::compose::loss::names_a_feeling;
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

bundle_res!("bot.compose.draft"; OFFER, ABSENT, THIRD_PERSON, APOLOGY, APOLOGY_ACT, APOLOGY_SELF, SORRY, SUBJECT, COPULA,
    QUESTION, THING_GONE_WRONG, NOT_A_CHANGE, WANT_FROM, NO_MESSAGE, A_WISH_FOR_SOMEONE, HURTS_SOMEONE, WANTS_THEM_DEAD,
    REPORTS_ON_SOMEONE, QUOTED, MESSAGE_WORD, SHAPE_CUES, WRITING_ASK, THE_CONTENT, WANTS_MORE_OF_IT, A_SPAN,
    REPORTED_SPEECH, TELL_FRAME, BARE_LEAD);
bundle_sets!("bot.compose.draft"; FORMAL, CREATIVE, AGREEMENT, CAPITALS, VENT, NEWS, FEELING, AFTER_ILL);
bundle_strs!("bot.compose.draft"; VENT_PHRASES, NEWS_PHRASES, FEELING_PHRASES, LEAD_INS);
bundle_vals!("bot.compose.draft"; LEADS, TIDY, MESSAGE_SHAPES, SUBJECT_EXAMPLES);
inline_re!(A_WORD, r"[a-z']+");
inline_re!(OPENS_ON_A_METHOD, r"^\s*(?:how|what|where|when|why)\b");
inline_re!(OPENS_ON_I, r"i(?:'|\b)", crate::py::re::I);

/// LEADS as (pattern, kind) pairs.
static LEAD_PAIRS: LazyLock<Vec<(&'static Re, &'static str)>> = LazyLock::new(|| {
    LEADS.items().iter().map(|pair| (pair.items()[0].as_re().expect("a lead pattern"), pair.items()[1].str())).collect()
});

fn words_of(text: &str) -> HashSet<String> {
    A_WORD.findall_str(text).into_iter().collect()
}

/// `words & (NEWS | FEELING)` or a news or feeling phrase in the line.
fn news_or_feeling(low: &str) -> bool {
    words_of(low).iter().any(|w| NEWS.contains(w) || FEELING.contains(w))
        || NEWS_PHRASES.iter().chain(FEELING_PHRASES.iter()).any(|p| low.contains(p.as_str()))
}

/// Python: bot/compose/draft.py:261 `_wants_to_hurt(low)`
///
/// True when the line is a wish to hurt somebody, which is never a message to send.
pub fn _wants_to_hurt(low: &str) -> bool {
    HURTS_SOMEONE.search(low).is_some() || WANTS_THEM_DEAD.search(low).is_some()
}

/// Python: bot/compose/draft.py:266 `_is_a_message_shape(low)`
///
/// True when the line reads as the content of a message rather than a fragment or a state.
pub fn _is_a_message_shape(low: &str) -> bool {
    if NO_MESSAGE.search(low).is_some() || NOT_A_CHANGE.search(low).is_some() || WANT_FROM.search(low).is_some() {
        return false;
    }
    if A_WORD.findall_str(low).len() < 2 {
        return false;
    }
    if MESSAGE_SHAPES.items().iter().any(|shape| shape.as_re().expect("a shape pattern").search(low).is_some()) {
        return true;
    }
    THING_GONE_WRONG.search(low).is_some() && says_it_is_the_message(low)
}

/// Python: bot/compose/draft.py:298 `names_a_draft(reply)`
///
/// True when a reply is the composer's drafted message rather than a line about it.
pub fn names_a_draft(reply: &str) -> bool {
    text::starts_with_any(text::lstrip(reply), &LEAD_INS)
}

/// Python: bot/compose/draft.py:303 `offers_a_draft(reply)`
///
/// True when a reply is the bot's promise to write the message, rather than the message.
pub fn offers_a_draft(reply: &str) -> bool {
    OFFER.search(&reply.to_lowercase()).is_some()
}

/// Python: bot/compose/draft.py:317 `hands_over_words(reply)`
///
/// True when a reply already hands the person words to send, in quotation marks: four words or
/// more between quotes is a message to send.
pub fn hands_over_words(reply: &str) -> bool {
    QUOTED.findall(reply).iter().any(|body| text::split_ref(body.one()).len() >= 4)
}

/// Python: bot/compose/draft.py:350 `hands_over_the_shape(reply)`
///
/// True when a reply already gives the parts of the message, without quoting it.
pub fn hands_over_the_shape(reply: &str) -> bool {
    let low = reply.to_lowercase();
    let cues: HashSet<&str> = SHAPE_CUES.finditer(&low).iter().map(|m| m.as_str()).collect();
    cues.len() >= 3 || (cues.len() >= 2 && MESSAGE_WORD.search(&low).is_some())
}

/// Python: bot/compose/draft.py:387 `reads_as_a_writing_ask(line)`
///
/// True when the line asks the bot to write a message and has not said what it should say.
pub fn reads_as_a_writing_ask(line: &str) -> bool {
    let clean = text::squash(&line.to_lowercase());
    if clean.is_empty() {
        return false;
    }
    if words_of(&clean).iter().any(|w| CREATIVE.contains(w)) {
        return false;
    }
    if OPENS_ON_A_METHOD.match_(&clean).is_some() {
        return false; // "how do i write an email to my boss" asks for the method
    }
    let Some(found) = WRITING_ASK.search(&clean) else { return false };
    THE_CONTENT.search(&clean[found.end()..]).is_none()
}

/// Python: bot/compose/draft.py:408 `_tidy(text)`
///
/// The person's own sentence, with chat-speak put right and a capital at the front.
pub fn _tidy(text_: &str) -> String {
    let words = text::split_ref(text_);
    let mut line_in = text_.to_string();
    if words.len() > 1
        && text::strip_chars(words[0], ".,!?;:").to_lowercase() == "ill"
        && AFTER_ILL.contains(&text::strip_chars(words[1], ".,!?;:").to_lowercase())
    {
        line_in = format!("i'll {}", words[1..].join(" "));
    }
    let mut out: Vec<String> = Vec::new();
    for word in text::split_ref(&line_in) {
        let bare = text::strip_chars(word, ".,!?;:");
        let tail = if !bare.is_empty() && word.starts_with(bare) { &word[bare.len()..] } else { "" };
        if CAPITALS.contains(&bare.to_lowercase()) {
            out.push(format!("{}{tail}", text::upper_first(bare)));
            continue;
        }
        match TIDY.get(&bare.to_lowercase()).map(|v| v.str()) {
            Some(fixed) if !fixed.is_empty() && text::is_lower(bare) => out.push(format!("{fixed}{tail}")),
            _ => out.push(word.to_string()),
        }
    }
    let joined = out.join(" ");
    let line = text::strip_chars(&joined, " .");
    if line.is_empty() {
        return String::new();
    }
    text::upper_first(line)
}

/// Python: bot/compose/draft.py:432 `_small(text)`
///
/// "A recycling bin" -> "a recycling bin", but "I need my stuff back" keeps its capital.
pub fn _small(text_: &str) -> String {
    let first: String = text_.chars().take(1).collect();
    if text_.is_empty() || !text::is_upper(&first) || text::starts_with_any(text_, &["I ", "I'", "I,"]) {
        return text_.to_string();
    }
    let head = text::split_ref(text_)[0];
    if CAPITALS.contains(&text::strip_chars(head, ".,!?;:").to_lowercase()) {
        return text_.to_string(); // a day or a month is still a name
    }
    lower_first(text_)
}

/// `text[0].lower() + text[1:]`.
fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().chain(c).collect(),
        None => String::new(),
    }
}

/// Python: bot/compose/draft.py:451 `_content(text)`
///
/// (the sentence to put in the message, shape), or None for Python's `(None, None)`.
pub fn _content(text_: &str) -> Option<(String, String)> {
    let mut line = text::squash(text_);
    if line.is_empty() || line.contains('?') {
        return None;
    }
    let low = line.to_lowercase();
    if ["ok", "okay", "yes", "yeah", "no", "thanks", "thank you", "hi", "hello", "hey"].contains(&low.as_str()) {
        return None;
    }
    if text::split_ref(&line).iter().all(|w| AGREEMENT.contains(text::strip_chars(w, ".,!?;:'"))) {
        return None; // "yes please", "go on then" - nothing to send
    }
    if QUESTION.match_(&low).is_some() {
        return None;
    }
    let mut shape = "said";
    for (pattern, kind) in LEAD_PAIRS.iter() {
        if let Some(found) = pattern.match_(&low) {
            shape = kind;
            // The end is read off the lowered line and cut from the line as typed, by character.
            let at = text::char_index(&low, found.end());
            line = text::strip(&text::slice(&line, at as isize, None)).to_string();
            break;
        }
    }
    let low = line.to_lowercase();
    if WANTS_MORE_OF_IT.match_(&low).is_some() {
        // "i want to hit the gym more" is a want about the person's own life, not the wording
        // of a message.
        return None;
    }
    let words = text::split_ref(&line);
    if words.len() < 2 || words.len() > 30 {
        return None;
    }
    if words.iter().any(|w| CREATIVE.contains(text::strip_chars(&w.to_lowercase(), ".,!?;:"))) {
        return None;
    }
    let body = _tidy(&line);
    if body.is_empty() {
        return None;
    }
    Some((body, shape.to_string()))
}

/// Python: bot/compose/draft.py:488 `_formal(memory, words)`
///
/// True when the message is going to someone a person writes to formally.
pub fn _formal<S: AsRef<str>>(memory: &Memory, words: &[S]) -> bool {
    if words.iter().any(|w| FORMAL.contains(w.as_ref())) {
        return true;
    }
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(4)..] {
        if words_of(&turn.user.to_lowercase()).iter().any(|w| FORMAL.contains(w)) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/draft.py:499 `_said_before(memory, line)`
///
/// The line's words, when the bot's last reply offered to draft; otherwise None.
pub fn _said_before(memory: &Memory, line: &str) -> Option<String> {
    if OFFER.search(&crate::memory::last_bot(memory).to_lowercase()).is_none() {
        return None;
    }
    let low = text::squash(&line.to_lowercase());
    if low.is_empty() || low.contains('?') {
        return None;
    }
    Some(low)
}

/// Python: bot/compose/draft.py:522 `_a_fault_they_were_asked_for(low)`
///
/// How long the thing has been wrong, on the turn after the bot asked what to say.
pub fn _a_fault_they_were_asked_for(low: &str) -> bool {
    if A_SPAN.search(low).is_none() {
        return false;
    }
    if THIRD_PERSON.search(low).is_some() || REPORTED_SPEECH.search(low).is_some() {
        return false;
    }
    !news_or_feeling(low)
}

/// Python: bot/compose/draft.py:547 `is_theirs(memory, line)`
///
/// True when the offer is open and the line is news about the person, or a mood.
pub fn is_theirs(memory: &Memory, line: &str) -> bool {
    let Some(low) = _said_before(memory, line) else { return false };
    if A_WISH_FOR_SOMEONE.search(&low).is_some() {
        return true;
    }
    if _wants_to_hurt(&low) {
        // A threat must never come back signed. The anger has its own answer.
        return true;
    }
    if let Some(lead) = _names_the_message(&low) {
        // The bare "it's ..." prefix is not enough on its own - "its about the deposit" is a
        // fragment - so that one still has to be shaped like a message.
        if is_bare_lead(lead) && !_is_a_message_shape(&low) {
            return true;
        }
        // What follows the lead is read for news and mood like any other line: "saying my mum
        // died" is not a message to draft.
        let end = lead.match_(&low).expect("the lead matched").end();
        return news_or_feeling(&low[end..]);
    }
    // How the person is feeling in their own words comes before the complaint guard.
    if APOLOGY.match_(&low).is_some() && THIRD_PERSON.search(&low).is_none() {
        // "im sorry for shouting" is the message itself, not how the person feels.
        return false;
    }
    if ABSENT.search(&low).is_some() {
        // "im off sick tomorrow" is the message they were asked for.
        return false;
    }
    if text::starts_with_any(&low, &["i feel", "im ", "i'm ", "i am ", "ive been ", "i've been "])
        && (words_of(&low).iter().any(|w| FEELING.contains(w)) || FEELING_PHRASES.iter().any(|p| low.contains(p.as_str())))
    {
        return true;
    }
    if words_of(&low).iter().any(|w| VENT.contains(w)) {
        return false; // a complaint about someone is the mood, not their news
    }
    if VENT_PHRASES.iter().any(|p| low.contains(p.as_str())) || REPORTS_ON_SOMEONE.search(&low).is_some() {
        return false;
    }
    // Nothing else drafts unless the line is shaped like a message.
    if !(_is_a_message_shape(&low) || _a_fault_they_were_asked_for(&low)) {
        return true;
    }
    if news_or_feeling(&low) {
        return true;
    }
    // The composer's own readings know the classes a word list does not.
    reads_as_pregnancy_bleeding(&low)
        || reads_as_sudden_loss(&low)
        || reads_as_bad_news(&low)
        || reads_like_a_symptom(&low)
        || names_a_feeling(&low)
}

/// `lead is BARE_LEAD`: the bundle holds BARE_LEAD and LEADS apart, so the two are compared by
/// what they are (pattern and flags).
fn is_bare_lead(lead: &Re) -> bool {
    lead.pattern() == BARE_LEAD.pattern() && lead.flags() == BARE_LEAD.flags()
}

/// Python: bot/compose/draft.py:624 `_names_the_message(low)`
///
/// The lead that opens the line, or None: "saying ...", "asking for ...", "tell them ...".
pub fn _names_the_message(low: &str) -> Option<&'static Re> {
    LEAD_PAIRS.iter().find(|(pattern, _)| pattern.match_(low).is_some()).map(|(pattern, _)| *pattern)
}

/// Python: bot/compose/draft.py:632 `says_it_is_the_message(line)`
///
/// True when the line opens by naming the message. The bare "that/it's" lead is left out.
pub fn says_it_is_the_message(line: &str) -> bool {
    match _names_the_message(&text::squash(&line.to_lowercase())) {
        Some(lead) => !is_bare_lead(lead),
        None => false,
    }
}

/// Python: bot/compose/draft.py:646 `not_the_message(memory, line)`
///
/// True when the offer is open but this line is a complaint about somebody, not the message.
pub fn not_the_message(memory: &Memory, line: &str) -> bool {
    let Some(low) = _said_before(memory, line) else { return false };
    if is_theirs(memory, line) {
        return false;
    }
    if words_of(&low).iter().any(|w| VENT.contains(w)) {
        return true;
    }
    if VENT_PHRASES.iter().any(|p| low.contains(p.as_str())) {
        return true;
    }
    REPORTS_ON_SOMEONE.search(&low).is_some()
}

/// Python: bot/compose/draft.py:665 `drafter(memory, line, words=None)`
///
/// The draft for this line, or None when the bot did not offer one last turn. (`words` empty is
/// Python's None: the words are read off the line.)
pub fn drafter<S: AsRef<str>>(memory: &Memory, line: &str, words: &[S]) -> Option<String> {
    if OFFER.search(&crate::memory::last_bot(memory).to_lowercase()).is_none() {
        return None;
    }
    if is_theirs(memory, line) || not_the_message(memory, line) {
        return None;
    }
    let flat = text::squash(&line.to_lowercase());
    let mut line_in = line.to_string();
    if let Some(frame) = TELL_FRAME.match_(&flat) {
        let rest = text::strip_chars(&flat[frame.end()..], " ,.").to_string();
        if rest.is_empty() || THIRD_PERSON.search(&rest).is_some() {
            return None;
        }
        line_in = rest;
    }
    let (mut body, shape) = _content(&line_in)?;
    if shape == "told" {
        // "telling him i quit" is the person describing the message in the third person.
        return None;
    }
    if shape == "asked" {
        body = format!("I wanted to ask about {}", _small(&body));
    }
    if body.ends_with(['?', '!', '.']) {
        body.pop();
    }
    // "The boiler is broken" after "Hi, " is one sentence, so the capital goes.
    body = _small(&body);
    if COPULA.match_(&body.to_lowercase()).is_some() {
        body = format!("I'm {}", lower_first(&body));
    } else if SUBJECT.match_(&body.to_lowercase()).is_some() {
        body = format!("I {}", lower_first(&body));
    }
    let apology = APOLOGY.match_(&body.to_lowercase()).is_some();
    if apology {
        // The message is itself an apology, so it needs the part that carries one - taking the
        // blame for what was done; an apology that names no doing is left to the shape answer.
        if APOLOGY_ACT.search(&body.to_lowercase()).is_none() || APOLOGY_SELF.search(&body.to_lowercase()).is_some() {
            return None;
        }
        body.push_str(". I shouldn't have done that");
    } else if SORRY.search(&body.to_lowercase()).is_some() {
        // The apology opens the message, so the sentence after it starts in lower case - unless
        // it starts with "I".
        let keeps_cap = OPENS_ON_I.match_(&body).is_some();
        body = format!("sorry for the short notice - {}", if keeps_cap { body.clone() } else { lower_first(&body) });
    }
    let given: Vec<String> = words.iter().map(|w| w.as_ref().to_string()).collect();
    let words: Vec<String> = if given.is_empty() { A_WORD.findall_str(&line_in.to_lowercase()) } else { given };
    let friendly = !_formal(memory, &words);
    let hello = if friendly { "Hey" } else { "Hi" };
    let sign = if friendly { "Cheers" } else { "Thanks" };
    if apology || A_WORD.findall_str(&body.to_lowercase()).len() <= 4 {
        // A four-word message signed "Cheers" reads like a joke. The short line stands on its own.
        return Some(format!("{hello}, {body}."));
    }
    Some(format!("{hello}, {body}. {sign}."))
}

/// Python: bot/compose/draft.py:754 `example_subject(understood)`
///
/// The sample subject line for the email-shape card, in the world the line names.
pub fn example_subject(understood: &Understood) -> String {
    let line = if !understood.clean.is_empty() { &understood.clean } else { &understood.text };
    for pair in SUBJECT_EXAMPLES.items() {
        let (pattern, example) = (pair.items()[0].as_re().expect("a subject pattern"), pair.items()[1].str());
        if pattern.search(line).is_some() {
            return example.to_string();
        }
    }
    "Invoice 2144 - payment due".to_string()
}

