//! Port of `bot/compose/badnews.py`: the readings of news a friend would answer gently - a
//! death, a leaving, a sacking, an illness of theirs or of someone they look after.
#![allow(non_upper_case_globals)]

use crate::compose::com4::{self, any_search, bundle_re_lists, bundle_res, bundle_sets, bundle_str, bundle_str_res, bundle_vals, inline_re};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;

bundle_res!("bot.compose.badnews"; _NEARLY, _AFTER_IT, DEATH_SAID, _WORD, LEFT_ME_RE, PARTNER_LEFT_RE, PARTNER_LEAVING_RE,
    REPORTED_LEAVING_RE, MARRIAGE_ENDED_RE, NOT_THEIRS, PARTNER, _LEAVE_THE_NOUN, CLOSED_DOOR_WORK, CLOSED_DOOR_WORK_OWN,
    CLOSED_DOOR_ELSEWHERE, JOKED, BARE_ACHE, BARE_ILL, BARE_BODY, BARE_HURTS, BARE_WORD_ONLY, TROUBLE_OVER, SUICIDE,
    _NAMES_ITS_SUBJECT);
bundle_re_lists!("bot.compose.badnews"; AWARD_LOST, BAD_NEWS, BAD_NEWS_ITSELF, NEWS_WITH_A_NAME, CLOSED_DOOR, OWN_PLAN,
    CARER_SYMPTOM, CARER_BODY, PET_SYMPTOM, PET_LOW_SYMPTOM, PET_CONDITION, SYMPTOM, SUDDEN_LOSS, PREGNANCY_BLEEDING);
bundle_str_res!("bot.compose.badnews"; STILL_HERE, LEFT_GONE, STILL_COMING, SACKED_FIRED);
bundle_sets!("bot.compose.badnews"; NEARLY, NOT_A_TYPO, LEXICON, VOWELS, BARE_FILLER, _CARRIES, _PLACE, _PERSON);
bundle_str!("bot.compose.badnews"; CARED_SICK, BODY, HURT);
bundle_vals!("bot.compose.badnews"; CLOSED_DOOR_ELSE_OWN);
inline_re!(A_WORD, r"[a-z']+");

/// Python: bot/compose/badnews.py:289 `_without_the_noun_leave(said)`
///
/// The line with "his leave" and "annual leave" taken out: time off is not a leaving.
pub fn _without_the_noun_leave(said: &str) -> String {
    text::squash(&_LEAVE_THE_NOUN.sub(" ", said))
}

/// Python: bot/compose/badnews.py:294 `_a_leaving_at_all(said)`
///
/// The leaving in any of its wordings: left me, is leaving, or the marriage ending said plain.
pub fn _a_leaving_at_all(said: &str) -> bool {
    let said = _without_the_noun_leave(said);
    let has = |rx: &Re| rx.search(&said).is_some();
    if !(has(&LEFT_ME_RE) || has(&PARTNER_LEAVING_RE) || has(&PARTNER_LEFT_RE) || has(&MARRIAGE_ENDED_RE)) {
        // A leaving reported rather than announced is the person's own when nobody else is
        // named in it: "my sister said shes leaving her husband" is a question about the sister.
        return has(&REPORTED_LEAVING_RE) && !has(&NOT_THEIRS);
    }
    !(has(&NOT_THEIRS) && !has(&PARTNER))
}

/// Python: bot/compose/badnews.py:305 `reads_as_a_leaving_already_gone(clean)`
///
/// Whether the leaving has happened, so the pool's question about the house does not fit.
pub fn reads_as_a_leaving_already_gone(clean: &str) -> bool {
    let said = com4::flat(clean);
    if !_a_leaving_at_all(&said) {
        return false;
    }
    if STILL_HERE.search(&said).is_some() {
        return false;
    }
    LEFT_GONE.search(&said).is_some() && STILL_COMING.search(&said).is_none()
}

/// Python: bot/compose/badnews.py:315 `reads_as_a_reported_leaving(clean)`
///
/// Whether the leaving is reported in the third person rather than said of the partner.
pub fn reads_as_a_reported_leaving(clean: &str) -> bool {
    let said = _without_the_noun_leave(&com4::flat(clean));
    if REPORTED_LEAVING_RE.search(&said).is_none() || NOT_THEIRS.search(&said).is_some() {
        return false;
    }
    !(LEFT_ME_RE.search(&said).is_some()
        || PARTNER_LEAVING_RE.search(&said).is_some()
        || MARRIAGE_ENDED_RE.search(&said).is_some())
}

/// Python: bot/compose/badnews.py:333 `reads_as_the_leaving(clean)`
///
/// Whether the line is the person being left by the one they live with.
pub fn reads_as_the_leaving(clean: &str) -> bool {
    _a_leaving_at_all(&com4::flat(clean))
}

/// Python: bot/compose/badnews.py:659 `reads_as_an_award_lost(clean)`
///
/// Whether the line is a benefit, a pension or a wage stopped, refused or cut.
pub fn reads_as_an_award_lost(clean: &str) -> bool {
    any_search(&AWARD_LOST, clean)
}

/// Python: bot/compose/badnews.py:668 `reads_as_a_sacking(clean)`
///
/// Whether the line is somebody losing their job, and nothing else.
pub fn reads_as_a_sacking(clean: &str) -> bool {
    SACKED_FIRED.search(clean).is_some()
}

/// Python: bot/compose/badnews.py:681 `reads_as_news_with_a_name(clean)`
///
/// Whether the line is the news that has a name in it, rather than the medical list.
pub fn reads_as_news_with_a_name(clean: &str) -> bool {
    any_search(&NEWS_WITH_A_NAME, clean)
}

/// Python: bot/compose/badnews.py:692 `reads_as_heavy_news(clean)`
///
/// Whether a line is news that a bare ask after it would be asking about (the medical list only).
pub fn reads_as_heavy_news(clean: &str) -> bool {
    any_search(&BAD_NEWS_ITSELF, clean)
}

/// Python: bot/compose/badnews.py:756 `reads_as_a_closed_door(clean)`
///
/// Whether a cleaned, lowercased line is the person being turned away from work.
pub fn reads_as_a_closed_door(clean: &str) -> bool {
    if CLOSED_DOOR_WORK.search(clean).is_none() && CLOSED_DOOR_WORK_OWN.search(clean).is_none() {
        return false;
    }
    for (pattern, else_owns_it) in CLOSED_DOOR.iter().zip(CLOSED_DOOR_ELSE_OWN.items()) {
        if pattern.search(clean).is_none() {
            continue;
        }
        if else_owns_it.truthy() && CLOSED_DOOR_ELSEWHERE.search(clean).is_some() {
            continue;
        }
        return true;
    }
    false
}

/// Python: bot/compose/badnews.py:785 `_nearly(word)`
///
/// Whether `word` is an emergency word with one letter typed wrong.
pub fn _nearly(word: &str) -> bool {
    let n = text::len(word) as i64;
    if n < 6 || NOT_A_TYPO.contains(word) {
        return false;
    }
    for target in NEARLY.iter() {
        if word == target || (n - text::len(target) as i64).abs() > 1 {
            continue; // the word itself is the patterns' job, not the typos'
        }
        if LEXICON.contains(word) {
            continue; // an everyday word is never a misspelling of one of these
        }
        if _one_edit(word, target) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/badnews.py:924 `reads_as_a_bare_ailment(clean)`
///
/// Whether the whole line is just the name of something that hurts: "toothache", "sore throat".
pub fn reads_as_a_bare_ailment(clean: &str) -> bool {
    if text::strip(clean).is_empty() || clean.contains('?') {
        return false;
    }
    if BARE_WORD_ONLY.search(clean).is_some() {
        return false;
    }
    let words = A_WORD.findall_str(&clean.to_lowercase());
    if words.is_empty() || words.len() > 5 {
        return false;
    }
    let named = BARE_ACHE.search(clean).is_some() || BARE_ILL.search(clean).is_some();
    if !named && !(BARE_BODY.search(clean).is_some() && BARE_HURTS.search(clean).is_some()) {
        return false;
    }
    for word in &words {
        if BARE_FILLER.contains(word) {
            continue;
        }
        if BARE_ACHE.fullmatch(word).is_some()
            || BARE_ILL.fullmatch(word).is_some()
            || BARE_BODY.fullmatch(word).is_some()
            || BARE_HURTS.fullmatch(word).is_some()
        {
            continue;
        }
        return false;
    }
    true
}

/// Python: bot/compose/badnews.py:1018 `_someone_elses_symptoms(who, sick=CARED_SICK, owner=r"my|our", named=None, generic=True, body_only=False, part=None)`
///
/// The shapes a line about somebody else being unwell takes, for the names in `who`. Python
/// builds CARER_BODY, PET_SYMPTOM, PET_LOW_SYMPTOM and PET_CONDITION with it at import time
/// (those come from the bundle); this is the builder itself, with the defaults spelled out by
/// the caller (`sick` = CARED_SICK, `owner` = "my|our").
pub fn _someone_elses_symptoms(
    who: &str,
    sick: Option<&str>,
    owner: Option<&str>,
    named: Option<&str>,
    generic: bool,
    body_only: bool,
    part: Option<&str>,
) -> Vec<Re> {
    let sick = sick.unwrap_or(&CARED_SICK);
    let owner = owner.unwrap_or("my|our");
    let i = crate::py::re::I;
    let mut shapes = Vec::new();
    if generic && !body_only {
        // "my son has a temperature", "our daughter's got a cough", "my mum came down with a cold"
        shapes.push(Re::new(
            &format!(
                r"\b(?:{owner})\s+(?:poor\s+|little\s+|eldest\s+|youngest\s+|old\s+)?{who}(?:'s|s)?\s+(?:has|have|had|got|has got|keeps getting|keeps|kept|keep|came down with|come down with|is|was|been|looks|seems)\b[^.!?]{{0,30}}?\b(?:{sick})\b"
            ),
            i,
        ));
        // "my dad vomits every morning": the verb itself is the connector.
        shapes.push(Re::new(&format!(r"\b(?:{owner})\s+{who}\s+(?:{sick})\b"), i));
    }
    if generic {
        // "my mum's knee is swollen", "my sons ear hurts", "my daughters back is sore"
        shapes.push(Re::new(
            &format!(r"\b(?:my|our)\s+{who}(?:'s|s)\s+{}\b[^.!?]{{0,18}}?\b(?:{}|infected|playing up)\b", *BODY, *HURT),
            i,
        ));
        // "my wife has a bad back", "my dad has a gammy knee"
        let part = part.filter(|p| !p.is_empty()).unwrap_or(&BODY);
        shapes.push(Re::new(
            &format!(
                r"\b(?:{owner})\s+{who}(?:'s|s)?\s+(?:has|have|had|got)(?:\s+got)?\s+(?:a|an)?\s*(?:bad|sore|dodgy|gammy|weak|dicky|knackered|broken|cracked|twisted|swollen|stiff|aching)\s+{part}\b"
            ),
            i,
        ));
    }
    if let Some(named) = named.filter(|n| !n.is_empty()) {
        // The animal is named or called something, or the condition is said with a possessive.
        shapes.push(Re::new(
            &format!(
                r"\b(?:{owner})\s+(?:poor\s+|little\s+|eldest\s+|youngest\s+|old\s+)?{who}(?:'s|s)?\b[^.!?]{{0,24}}?\b(?:{named})\b"
            ),
            i,
        ));
    }
    shapes
}

/// Python: bot/compose/badnews.py:1132 `reads_for_a_carer(clean, memory=None)`
///
/// Whether a cleaned, lowercased line is about someone the person looks after being ill.
pub fn reads_for_a_carer(clean: &str, memory: &Memory) -> bool {
    let _ = memory;
    if TROUBLE_OVER.search(clean).is_some() {
        return false;
    }
    any_search(&CARER_SYMPTOM, clean)
}

/// Python: bot/compose/badnews.py:1144 `reads_as_a_carer_body(clean, memory=None)`
///
/// Whether a cleaned, lowercased line is somebody else's body, rather than their illness.
/// (Every caller passes the memory, so the `memory is not None` arm is the one taken.)
pub fn reads_as_a_carer_body(clean: &str, memory: &Memory) -> bool {
    if TROUBLE_OVER.search(clean).is_some() {
        return false;
    }
    if any_search(&CARER_BODY, clean) {
        return true;
    }
    // "my dad is 70" then "hes got a bad hip": the "he" is the dad the chat has just named.
    crate::memory::whose_body::somebody_elses_body(clean, memory)
}

/// Python: bot/compose/badnews.py:1159 `reads_for_a_pet(clean)`
///
/// Whether a cleaned, lowercased line is about an animal of theirs being ill or hurt.
pub fn reads_for_a_pet(clean: &str) -> bool {
    if TROUBLE_OVER.search(clean).is_some() {
        return false;
    }
    any_search(&PET_LOW_SYMPTOM, clean) || any_search(&PET_SYMPTOM, clean)
}

/// Python: bot/compose/badnews.py:1168 `reads_as_a_pet_low(clean)`
pub fn reads_as_a_pet_low(clean: &str) -> bool {
    TROUBLE_OVER.search(clean).is_none() && any_search(&PET_LOW_SYMPTOM, clean)
}

/// Python: bot/compose/badnews.py:1175 `reads_as_a_pet_condition(clean)`
pub fn reads_as_a_pet_condition(clean: &str) -> bool {
    TROUBLE_OVER.search(clean).is_none() && any_search(&PET_CONDITION, clean)
}

/// Python: bot/compose/badnews.py:1182 `reads_like_a_symptom(clean)`
///
/// Whether a cleaned, lowercased line is the person's own body complaining.
pub fn reads_like_a_symptom(clean: &str) -> bool {
    let clean = crate::understanding::normalize::without_the_figure_of_speech(clean);
    any_search(&SYMPTOM, &clean) || reads_as_a_bare_ailment(&clean)
}

/// Python: bot/compose/badnews.py:1190 `_one_edit(left, right)`
///
/// Whether `left` is `right` with one letter typed wrong: a letter dropped or added, or a vowel
/// written for another vowel.
pub fn _one_edit(left: &str, right: &str) -> bool {
    if left == right {
        return false;
    }
    let l: Vec<char> = left.chars().collect();
    let r: Vec<char> = right.chars().collect();
    if l.len() == r.len() {
        let different: Vec<(char, char)> = l.iter().zip(r.iter()).filter(|(a, b)| a != b).map(|(a, b)| (*a, *b)).collect();
        return different.len() == 1
            && VOWELS.contains(&different[0].0.to_string())
            && VOWELS.contains(&different[0].1.to_string());
    }
    let (short, long) = if l.len() < r.len() { (&l, &r) } else { (&r, &l) };
    for index in 0..long.len() {
        let without: Vec<char> = long[..index].iter().chain(long[index + 1..].iter()).copied().collect();
        if without == **short {
            return true;
        }
    }
    false
}

/// Python: bot/compose/badnews.py:1244 `reads_as_sudden_loss(clean)`
///
/// Whether a line is a pregnancy that has ended or someone who took their own life.
pub fn reads_as_sudden_loss(clean: &str) -> bool {
    let text_ = com4::flat(clean);
    // A dream of a death is not a death; a death said before the dream keeps this reading.
    if crate::memory::dream::a_dream_is_not_happening(&text_) && !crate::memory::dream::a_death_came_first(&text_) {
        return false;
    }
    any_search(&SUDDEN_LOSS, &text_) || SUICIDE.search(&text_).is_some() || reads_as_someones_death(&text_)
}

/// Python: bot/compose/badnews.py:1258 `reads_as_someones_death(clean)`
///
/// Whether the line reports that a person or an animal has died.
pub fn reads_as_someones_death(clean: &str) -> bool {
    let text_ = com4::flat(clean);
    if matches!(text_.as_str(), "died" | "dies" | "she died" | "he died") {
        // The line is only the verb, answering a question about someone who was ill.
        return true;
    }
    for found in DEATH_SAID.finditer(&text_) {
        if _NEARLY.search(&text_[..found.end()]).is_some() {
            continue;
        }
        if _AFTER_IT.match_(&text_[found.end()..]).is_some() {
            continue;
        }
        if _subject_before(&text_, found.start(), found.as_str()) {
            return true;
        }
    }
    false
}

/// Python: bot/compose/badnews.py:1287 `_subject_before(text, start, said='')`
///
/// Whether the word the death belongs to is a person or an animal. `start` is a byte offset
/// (Python's is a character index into the same text).
pub fn _subject_before(text_: &str, start: usize, said: &str) -> bool {
    if !said.is_empty() && _NAMES_ITS_SUBJECT.match_(said).is_some() {
        return true;
    }
    let mut clause = &text_[..start];
    for mark in [".", "!", "?"] {
        if let Some(cut) = clause.rfind(mark) {
            clause = &clause[cut + 1..];
        }
    }
    let words = _WORD.findall_str(clause);
    for word in words.iter().rev() {
        if _CARRIES.contains(word) || _PLACE.contains(word) {
            continue;
        }
        return _PERSON.contains(word);
    }
    false
}

/// Python: bot/compose/badnews.py:1316 `reads_as_pregnancy_bleeding(clean)`
pub fn reads_as_pregnancy_bleeding(clean: &str) -> bool {
    any_search(&PREGNANCY_BLEEDING, clean)
}

/// Python: bot/compose/badnews.py:1321 `reads_as_the_speakers_own_plan(clean)`
pub fn reads_as_the_speakers_own_plan(clean: &str) -> bool {
    any_search(&OWN_PLAN, clean)
}

/// Python: bot/compose/badnews.py:1326 `_the_result_came_back_well(clean)`
///
/// Whether the line is a clear result, which is news of the other kind (`goodnews`).
pub fn _the_result_came_back_well(clean: &str) -> bool {
    crate::compose::goodnews::reads_as_a_clear_result(clean)
}

/// Python: bot/compose/badnews.py:1339 `reads_as_bad_news(clean)`
///
/// Whether a cleaned, lowercased line is news a friend would answer gently.
pub fn reads_as_bad_news(clean: &str) -> bool {
    if _the_result_came_back_well(clean) {
        return false;
    }
    if any_search(&BAD_NEWS, &JOKED.sub(" ", clean)) {
        return true;
    }
    A_WORD.findall_str(clean).iter().any(|w| _nearly(w))
}

