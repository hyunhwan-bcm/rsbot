//! Port of `bot/memory/meant.py`: they ask what the bot meant, and the answer is in the line it
//! just gave.
//!
//! A bare "what do you mean" is pointed at the sentence the bot meant, with a lead-in that hands
//! the bit over ("Just this: ..."); "i dont follow" gets it again in plainer framing. Narrow: the
//! line is only that question, there has to be a bot turn to point at, a hedge is never cited
//! back at itself, and one sentence is handed over, never the paragraph again.
#![allow(non_snake_case, clippy::all)]

use crate::data::{self, Value};
use crate::memory::mem7::{self, inline_re, lists, res, sets, texts, values};
use crate::memory::{again, repeat, topics};
use crate::py::re::{self, Re};
use crate::py::text as pt;
use crate::state::Memory;
use std::collections::HashSet;
use std::sync::LazyLock;

res!("meant"; MEAN, ASKS_SOMETHING, REPEAT, PERSONAL, FOR_MINE, NOT_LETTERS, TELLS, WHAT_YOU_ARE, ABOUT_MINE, SOMEONE_ELSE, RECEIPT, CLAUSE, DEFINES, CONDOLENCE, CONDOLENCE_SENTENCE);
sets!("meant"; FOLLOW_UP_WORDS, PUT_IT_PLAINER, ASKING_ABOUT_DOING, NOT_TO_REPEAT);
lists!("meant"; REPEAT_LEADS, LEADS, PERSONAL_LEADS, NOTHING_WRITTEN, NO_ANSWER_OF_MINE, SAID_TWICE, _PERSONAL_HEADS, _HERE);
texts!("meant"; SOMEONE_LEADS);
values!("meant"; MOST, LEAST, ANSWERED, LOOK_BACK, LOOK_BACK_FAR);
sets!("topics"; STOPWORDS);
static HEDGE: LazyLock<&'static Re> = LazyLock::new(|| data::re("bot.memory.repeat.HEDGE"));
inline_re! {
    WORDS = r"[a-z']+", 0;
    A_DIGIT = r"\d", 0;
}

fn int_of(v: &Value) -> usize {
    v.as_int().expect("an int global") as usize
}

/// Python: bot/memory/meant.py:243 `_asked_plainly(clean)`
///
/// True when the whole line is a pointer back at the last line or a word for plainer words.
pub fn _asked_plainly(clean: &str) -> bool {
    let said = pt::strip_chars(&pt::lower(clean).replace('\u{2019}', "'"), " ?.!,").to_string();
    if NOT_LETTERS.is_match(&said) {
        return false;
    }
    let words = WORDS.findall_str(&said);
    if words.is_empty() || words.len() > 9 {
        return false;
    }
    if !words.iter().all(|w| FOLLOW_UP_WORDS.contains(w)) || !words.iter().any(|w| PUT_IT_PLAINER.contains(w)) {
        return false;
    }
    if words.iter().any(|w| w == "tell" || w == "telling") && !TELLS.is_match(&said) {
        return false;
    }
    if WHAT_YOU_ARE.is_match(&said) {
        return false;
    }
    !ASKING_ABOUT_DOING.contains(words.last().expect("not empty"))
}

/// Python: bot/memory/meant.py:284 `_personal(said)`
///
/// True when this line already hands the answer over as it applies to the person.
pub fn _personal(said: &str) -> bool {
    let line = pt::strip(said);
    _PERSONAL_HEADS.iter().any(|head| line.starts_with(head.as_str()))
}

/// Python: bot/memory/meant.py:289 `_their_subject(said)`
///
/// The thing a line names for itself in "what does that mean for my pay", or "".
pub fn _their_subject(said: &str) -> String {
    FOR_MINE.search(said).map_or(String::new(), |found| pt::squash(&pt::lower(found.str(1))))
}

/// Python: bot/memory/meant.py:295 `_on_the_table(noun, memory)`
///
/// True when `noun` is already what the last turn or two is about.
pub fn _on_the_table(noun: &str, memory: &Memory) -> bool {
    let Some(head) = pt::split_ref(noun).first().map(|h| h.to_string()) else {
        return false;
    };
    let mut said = String::new();
    for turn in mem7::tail(&memory.turns, 2) {
        said.push_str(&format!(" {} {}", turn.user, turn.bot));
    }
    mem7::cached_re(&format!(r"\b{}", re::escape(&head)), 0).is_match(&pt::lower(&said))
}

/// Python: bot/memory/meant.py:317 `_under_the_lead(said)`
///
/// The answer a hand-back is carrying, whichever module wrote the lead-in.
pub fn _under_the_lead(said: &str) -> String {
    let text = pt::strip(said);
    let mut heads: Vec<&String> = _HERE.iter().collect();
    // `sorted(_HERE, key=len, reverse=True)`: longest first, ties in their order.
    heads.sort_by(|a, b| pt::len(b).cmp(&pt::len(a)));
    let low = pt::lower(text);
    for head in heads {
        if low.starts_with(&pt::lower(head)) {
            return pt::strip(mem7::from_char(text, pt::len(head))).to_string();
        }
    }
    _carried(text)
}

/// Python: bot/memory/meant.py:401 `_sentences(said)`
///
/// The sentences of an answer, with a quotation kept in one piece.
pub fn _sentences(said: &str) -> Vec<String> {
    crate::memory::quotes::sentences(said, data::get("bot.memory.meant.SENTENCE"))
}

/// Python: bot/memory/meant.py:424 `_gist(sentence)`
///
/// The first clause of a sentence: the bit that carries what the bot meant.
pub fn _gist(sentence: &str) -> String {
    let mut sentence = pt::strip(sentence).to_string();
    let least = int_of(&LEAST);
    let most = int_of(&MOST);
    let mut cut: Option<(usize, bool)> = None;
    for m in CLAUSE.finditer(&sentence) {
        if pt::char_index(&sentence, m.start()) >= least {
            cut = Some((m.start(), m.as_str().starts_with(':')));
            break;
        }
    }
    if let Some((cut, after_a_colon)) = cut {
        let explained = pt::strip(pt::lstrip_chars(&sentence[cut..], ":;- ")).to_string();
        if after_a_colon && pt::len(&explained) >= least && DEFINES.is_match(&explained) {
            sentence = pt::upper_first(&explained);
        } else {
            sentence = sentence[..cut].to_string();
        }
    }
    let mut sentence = pt::rstrip_chars(pt::strip(&sentence), ",;:- ").to_string();
    if pt::len(&sentence) > most {
        let head = mem7::first_chars(&sentence, most);
        let kept = pt::rsplitn_on(head, " ", 1).into_iter().next().unwrap_or_default();
        sentence = pt::rstrip_chars(&kept, ",;:- ").to_string();
    }
    if sentence.chars().last().is_some_and(|c| !".!?".contains(c)) {
        sentence.push('.');
    }
    sentence
}

/// Python: bot/memory/meant.py:478 `_answers(memory, back=LOOK_BACK)`
///
/// The bot's own lines in the last turn or two, newest first, each with the line it answered.
pub fn _answers(memory: &Memory, back: usize) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for turn in mem7::tail(&memory.turns, back).iter().rev() {
        let said = pt::strip(&turn.bot).to_string();
        if said.is_empty() {
            continue;
        }
        // An emergency reply or a condolence is not an explanation to point back at.
        if NOT_TO_REPEAT.contains(turn.module.as_deref().unwrap_or("")) {
            break;
        }
        out.push((said, pt::strip(&turn.user).to_string()));
    }
    out
}

/// Content words: `[w for w in topics.words(text) if w not in STOPWORDS and len(w) > 2]`.
fn content(text: &str) -> Vec<String> {
    topics::words(text).into_iter().filter(|w| !STOPWORDS.contains(w) && pt::len(w) > 2).collect()
}

/// Python: bot/memory/meant.py:497 `_thick_enough(said, asked)`
///
/// True when the bot's line could be explaining something, rather than saying how the news lands.
pub fn _thick_enough(said: &str, asked: &str) -> bool {
    let words = content(said);
    if words.len() >= 5 {
        return true;
    }
    let theirs: HashSet<String> = content(asked).into_iter().collect();
    words.iter().any(|w| theirs.contains(w))
}

/// Python: bot/memory/meant.py:518 `_framed(said)`
///
/// True when this line already has a hand-back lead on it, from this module or from repeat.
pub fn _framed(said: &str) -> bool {
    repeat::framed(said) || _HERE.iter().any(|head| pt::strip(said).starts_with(head.as_str()))
}

/// Python: bot/memory/meant.py:523 `_a_figure(said)`
///
/// True when the line states a figure, which is an answer however short it is.
pub fn _a_figure(said: &str) -> bool {
    A_DIGIT.is_match(said)
}

/// Python: bot/memory/meant.py:536 `_a_question(sentence)`
pub fn _a_question(sentence: &str) -> bool {
    pt::strip(sentence).ends_with('?')
}

/// Python: bot/memory/meant.py:541 `_worth_handing_over(said, sentence, asked)`
///
/// Whether what would come back is an answer rather than a way of saying how the news lands.
pub fn _worth_handing_over(said: &str, sentence: &str, asked: &str) -> bool {
    if !(_thick_enough(said, asked) || _a_figure(said)) {
        return false;
    }
    _a_figure(said) || _a_question(sentence) || _thick_enough(sentence, asked)
}

/// Python: bot/memory/meant.py:556 `_cite(said, sentence, asked="", memory=None)`
///
/// The sentence to hand over, or None when there is nothing to cite.
pub fn _cite(said: &str, sentence: &str, asked: &str, memory: Option<&Memory>) -> Option<String> {
    if HEDGE.is_match(said) {
        return None;
    }
    let mut said = said.to_string();
    let mut sentence = sentence.to_string();
    if CONDOLENCE.match_(&said).is_some() {
        // It opened by saying how the news lands: the rest is read as the line.
        said = _after_the_condolence(&said);
        if said.is_empty() {
            return None;
        }
        if !sentence.is_empty() && !said.contains(&sentence) {
            sentence = _meant(&said, pt::rstrip(&said).ends_with('?'));
        }
    }
    if pt::len(&sentence) < int_of(&ANSWERED) && !_a_figure(&said) && !_thick_enough(&said, asked) {
        return None;
    }
    if RECEIPT.is_match(&said) || RECEIPT.is_match(&sentence) {
        return None;
    }
    if _framed(&said) {
        return None;
    }
    if let Some(memory) = memory {
        if mem7::iterate_or_empty(memory.get("repeated")).iter().any(|line| line.as_str() == Some(said.as_str())) {
            return None;
        }
    }
    if !_worth_handing_over(&said, &sentence, asked) {
        return None;
    }
    Some(_gist(&sentence))
}

/// Python: bot/memory/meant.py:592 `_meant(said, asked_what_it_meant=False)`
///
/// The sentence the bot meant: its question when its own line ended on one, otherwise the first
/// thing it said that is not a hedge, or the sentence further on that says what the thing means.
pub fn _meant(said: &str, asked_what_it_meant: bool) -> String {
    let parts = _sentences(said);
    if parts.is_empty() {
        return String::new();
    }
    if pt::rstrip(said).ends_with('?') && (asked_what_it_meant || parts.len() == 1) {
        return parts.last().expect("not empty").clone();
    }
    let answered = int_of(&ANSWERED);
    let Some(first) = parts.iter().find(|s| !HEDGE.is_match(s) && pt::len(s) >= answered) else {
        return parts[0].clone();
    };
    if !DEFINES.is_match(first) {
        for sentence in &parts {
            if sentence == first || HEDGE.is_match(sentence) {
                continue;
            }
            if pt::len(sentence) >= answered && DEFINES.is_match(sentence) {
                return sentence.clone();
            }
        }
    }
    first.clone()
}

/// Python: bot/memory/meant.py:635 `_after_the_condolence(said)`
///
/// What the bot said after it said how the news lands, or "" when it only said that.
pub fn _after_the_condolence(said: &str) -> String {
    match CONDOLENCE_SENTENCE.match_(said) {
        Some(cut) => pt::squash(&said[cut.end()..]),
        None => String::new(),
    }
}

/// Python: bot/memory/meant.py:650 `_same_bit(one, other)`
///
/// Whether two lines hand over the same bit, even when one is cut shorter than the other.
pub fn _same_bit(one: &str, other: &str) -> bool {
    let one = pt::lower(pt::rstrip_chars(pt::strip(one), ".?! "));
    let mut other = pt::lower(pt::rstrip_chars(pt::strip(other), ".?! "));
    for lead in LEADS.iter().chain(REPEAT_LEADS.iter()) {
        let head = pt::lower(pt::partition(lead, "{").0);
        if other.starts_with(&head) {
            other = pt::strip(&other[head.len()..]).to_string();
        }
    }
    if one.is_empty() || other.is_empty() {
        return false;
    }
    let (short, long) = if pt::len(&one) <= pt::len(&other) { (&one, &other) } else { (&other, &one) };
    long.starts_with(short.as_str()) || long.contains(short.as_str())
}

/// Python: bot/memory/meant.py:666 `_already(cited, said_since)`
pub fn _already(cited: &str, said_since: &[String]) -> bool {
    said_since.iter().any(|line| _same_bit(cited, line))
}

/// Python: bot/memory/meant.py:672 `_carried(said)`
///
/// The answer a hand-back is carrying, or "" when the line above is not one.
pub fn _carried(said: &str) -> String {
    again::unframed(said).unwrap_or_default()
}

/// Python: bot/memory/meant.py:684 `_applied_to_the_person(clean, said, asked, memory, what_it_meant)`
///
/// The last answer handed back as it applies to the person asked about, or None.
pub fn _applied_to_the_person(clean: &str, said: &str, asked: &str, memory: &mut Memory, what_it_meant: bool) -> Option<String> {
    if !(what_it_meant && PERSONAL.is_match(clean)) {
        return None;
    }
    let carried = _under_the_lead(said);
    if carried.is_empty() || _personal(said) {
        return None;
    }
    let noun = _their_subject(clean);
    if !noun.is_empty() && !_on_the_table(&noun, memory) {
        return None;
    }
    // The sentence under the last hand-back is the answer, whatever `_cite` makes of it.
    let cited = _cite(&carried, &_meant(&carried, what_it_meant), asked, Some(memory)).filter(|c| !c.is_empty()).unwrap_or(carried);
    if cited.is_empty() {
        return None;
    }
    let lead = match SOMEONE_ELSE.search(clean) {
        Some(someone) => mem7::fmt(&SOMEONE_LEADS, &[("who", &pt::lower(someone.str(1))), ("said", &cited)]),
        None => mem7::fmt(&mem7::pick(&PERSONAL_LEADS, memory, clean), &[("said", &cited)]),
    };
    mem7::some_text(repeat::handed_back(memory, &lead))
}

/// `repeat.handed_back(memory, pick(pool, memory, clean))`.
fn hand_back_from(pool: &[String], memory: &mut Memory, clean: &str) -> Option<String> {
    let picked = mem7::pick(pool, memory, clean);
    mem7::some_text(repeat::handed_back(memory, &picked))
}

/// Python: bot/memory/meant.py:716 `answer_for(text, memory)`
///
/// The bit the bot meant, or None when this is not that turn.
///
/// CONTRACT: the skeleton gave this no return type; PyBot returns the reply or None.
pub fn answer_for(text: &str, memory: &mut Memory) -> Option<String> {
    // The line as the reading has it, so an abbreviation is met as the phrase it stands for.
    let understood_clean = memory.understood.as_ref().map(|u| u.clean.clone()).unwrap_or_default();
    let mut clean = pt::strip(if understood_clean.is_empty() { text } else { &understood_clean }).to_string();
    // A slip in the asking is still the asking: the reading's repair is read too.
    let repaired = pt::strip(&memory.understood.as_ref().map(|u| u.plain.clone()).unwrap_or_default()).to_string();
    if !repaired.is_empty() && repaired != clean && MEAN.match_(&clean).is_none() && REPEAT.match_(&clean).is_none() {
        if MEAN.match_(&repaired).is_some() || REPEAT.match_(&repaired).is_some() {
            clean = repaired;
        }
    }
    // A question hung on the end of a meaning ask is the question.
    if MEAN.match_(&clean).is_some() && ASKS_SOMETHING.is_match(&clean) {
        return None;
    }
    let (leads, what_it_meant): (&Vec<String>, bool) = if MEAN.match_(&clean).is_some() {
        let subject = _their_subject(&clean);
        if !subject.is_empty() && !_on_the_table(&subject, memory) {
            return None;
        }
        if let Some(about) = ABOUT_MINE.search(&clean) {
            if !_on_the_table(about.str(1), memory) {
                return None;
            }
        }
        (&LEADS, true)
    } else if REPEAT.match_(&clean).is_some() || _asked_plainly(&clean) {
        (&REPEAT_LEADS, false)
    } else {
        return None;
    };
    let personal_ask = what_it_meant && PERSONAL.is_match(&clean);
    let mut said_since: Vec<String> = Vec::new();
    let mut lines: Vec<(String, String)> = Vec::new();
    let mut applied_once = false;
    for back in [int_of(&LOOK_BACK), int_of(&LOOK_BACK_FAR)] {
        for (said, asked) in _answers(memory, back) {
            if said_since.contains(&said) {
                continue;
            }
            if !applied_once && personal_ask && _personal(&said) {
                applied_once = true;
            }
            if !applied_once {
                if let Some(applied) = _applied_to_the_person(&clean, &said, &asked, memory, what_it_meant) {
                    return Some(applied);
                }
            }
            let carried = _carried(&said);
            let (line, base) = if carried.is_empty() { (&said, said.clone()) } else { (&said, carried) };
            // A line that is already a hand-back is read through its frame.
            let cited = _cite(&base, &_meant(&base, what_it_meant), &asked, Some(memory));
            if let Some(cited) = cited.filter(|c| !c.is_empty()) {
                if !_already(&cited, &said_since) {
                    return Some(mem7::fmt(&mem7::pick(leads, memory, &clean), &[("said", &cited)]));
                }
            }
            said_since.push(line.clone());
            lines.push((said.clone(), asked.clone()));
        }
    }
    let Some((first, first_asked)) = lines.first().cloned() else {
        return None;
    };
    if _framed(&first) {
        // The answer behind it has been read out twice already.
        let carried = _under_the_lead(&first);
        let noun = _their_subject(&clean);
        if what_it_meant && !carried.is_empty() && PERSONAL.is_match(&clean) && (noun.is_empty() || _on_the_table(&noun, memory)) && !_personal(&first) {
            if let Some(applied) = _applied_to_the_person(&clean, &first, &first_asked, memory, what_it_meant) {
                return Some(applied);
            }
        }
        return hand_back_from(&SAID_TWICE, memory, &clean);
    }
    if CONDOLENCE.match_(&first).is_some() {
        return hand_back_from(&NOTHING_WRITTEN, memory, &clean);
    }
    if HEDGE.is_match(&first) {
        return hand_back_from(&NO_ANSWER_OF_MINE, memory, &clean);
    }
    if RECEIPT.is_match(&first)
        || (!_a_figure(&first) && (!_thick_enough(&first, &first_asked) || pt::len(&_meant(&first, false)) < int_of(&ANSWERED)))
    {
        return hand_back_from(&NOTHING_WRITTEN, memory, &clean);
    }
    None
}

/// Python: bot/memory/meant.py:838 `reads_it(text)`
///
/// Whether the line asks for the last answer again - the one ask that repeats on purpose.
pub fn reads_it(text: &str) -> bool {
    let clean = pt::squash(&pt::lower(text));
    MEAN.match_(&clean).is_some() || REPEAT.match_(&clean).is_some() || _asked_plainly(&clean)
}
