//! Port of PyBot's `bot/knowledge/study.py`: school and study - revising, remembering, exams and
//! grades, degrees, careers, and phrases in other languages.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g5 as dup;
use crate::knowledge::d1_shared as sh;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::knowledge::wording;
use crate::py::re::{self as re, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};

const M: &str = "bot.knowledge.study.";

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("{M}{name}"))
}
fn set(name: &str) -> HashSet<String> {
    data::set(&format!("{M}{name}"))
}
fn rx(name: &str) -> &'static Re {
    data::re(&format!("{M}{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("{M}{name}"))
}
fn get(name: &str) -> &'static Value {
    data::get(&format!("{M}{name}"))
}
fn str_pairs(name: &str) -> Vec<(String, String)> {
    get(name)
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
        .collect()
}
fn word_rows(name: &str) -> Vec<(Vec<String>, String)> {
    get(name)
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.strs(), row.get(1)?.as_str()?.to_string()))
        })
        .collect()
}
fn lookup<'a>(table: &'a [(String, String)], key: &str) -> Option<&'a String> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}
/// `(?<![a-z])word(?![a-z])` (the word escaped when `escape`), compiled once.
fn edged(word: &str, escape: bool) -> Re {
    let w = if escape { re::escape(word) } else { word.to_string() };
    Re::new(&format!("(?<![a-z]){w}(?![a-z])"), 0)
}

static WORDS_RX: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

// --- past papers and fractions ------------------------------------------------------------------

static PAST_PAPERS: LazyLock<String> = LazyLock::new(|| string("_PAST_PAPERS"));
static PAST_PAPERS_WHEN: LazyLock<String> = LazyLock::new(|| string("_PAST_PAPERS_WHEN"));
static PAST_PAPERS_ASK_WHEN: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PAST_PAPERS_ASK_WHEN"));
static FRACTIONS_FIRST: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("FRACTIONS_FIRST"));
static FRACTIONS_MULTIPLY: LazyLock<String> = LazyLock::new(|| string("FRACTIONS_MULTIPLY"));

/// `str(reading.get("text") or reading.get("norm") or "").lower()`: the norm is a list, so a line
/// with no text reads as the list's repr.
fn text_or_norm_lower(reading: &Understood) -> String {
    if !reading.text.is_empty() {
        return reading.text.to_lowercase();
    }
    if !reading.norm.is_empty() {
        return dup::py_repr_str_list(&reading.norm).to_lowercase();
    }
    String::new()
}

/// The past-papers answer, opening with the timing when the line asks when to start.
pub fn _past_papers_reply(_memory: &Memory, reading: &Understood) -> String {
    let line = text_or_norm_lower(reading);
    let words: HashSet<String> = WORDS_RX.findall_str(&line).into_iter().collect();
    let has = |w: &str| words.contains(w);
    if PAST_PAPERS_ASK_WHEN.iter().any(|w| words.contains(w))
        && (has("when")
            || has("should")
            || has("start")
            || has("begin")
            || line.contains("how long")
            || line.contains("how many")
            || has("soon"))
    {
        return PAST_PAPERS_WHEN.clone();
    }
    PAST_PAPERS.clone()
}

/// The fractions answer, opening with the operation the line asks about.
pub fn _fractions_reply(_memory: &Memory, reading: &Understood) -> String {
    let line = text_or_norm_lower(reading);
    let words: HashSet<String> = WORDS_RX.findall_str(&line).into_iter().collect();
    for (asking, said) in FRACTIONS_FIRST.iter() {
        if asking.iter().any(|w| words.contains(w)) {
            return said.clone();
        }
    }
    FRACTIONS_MULTIPLY.clone()
}

// --- the end of an essay ---------------------------------------------------------------------------

static ENDING_WORDS: LazyLock<Vec<Re>> = LazyLock::new(|| {
    let mut words = strs("_A_CONCLUSION_SAID");
    words.extend(strs("_SOMETHING_WRITTEN"));
    words.iter().map(|w| Re::new(&format!("(?<![a-z0-9]){}(?![a-z0-9])", re::escape(w)), 0)).collect()
});

/// True when the line is about the end of something written, not the end of anything.
pub fn _the_ending_is_written(understood: &Understood) -> bool {
    let clean = text::squash(sh::clean_or_text(understood)).to_lowercase();
    if clean.is_empty() {
        return false;
    }
    ENDING_WORDS.iter().any(|r| r.is_match(&clean))
}

// --- phrases in other languages ----------------------------------------------------------------------

/// A phrase and how to say it.
#[derive(Clone, Debug)]
pub struct Phrase {
    pub phrase: String,
    pub say: String,
}

static PHRASES: LazyLock<Vec<(String, Vec<(String, Phrase)>)>> = LazyLock::new(|| {
    get("PHRASES")
        .pairs()
        .iter()
        .filter_map(|(concept, langs)| {
            let rows = langs
                .pairs()
                .iter()
                .filter_map(|(lang, pair)| {
                    let p = pair.items();
                    Some((
                        lang.as_str()?.to_string(),
                        Phrase { phrase: p.first()?.as_str()?.to_string(), say: p.get(1)?.as_str()?.to_string() },
                    ))
                })
                .collect();
            Some((concept.as_str()?.to_string(), rows))
        })
        .collect()
});
static LANGUAGES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("LANGUAGES"));
static LANGUAGE_KEYS: LazyLock<Vec<String>> = LazyLock::new(|| LANGUAGES.iter().map(|(k, _)| k.clone()).collect());
static CONCEPTS: LazyLock<Vec<(Vec<(String, Re)>, String)>> = LazyLock::new(|| {
    word_rows("CONCEPTS")
        .into_iter()
        .map(|(alts, name)| (alts.iter().map(|a| (a.clone(), edged(a, true))).collect(), name))
        .collect()
});
/// NATIVE: foreign word -> (language, concept), in insertion order, each with its edged regex.
static NATIVE: LazyLock<Vec<(String, (String, String), Re)>> = LazyLock::new(|| {
    get("NATIVE")
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let word = k.as_str()?.to_string();
            let row = v.items();
            let re = edged(&word, true);
            Some((word, (row.first()?.as_str()?.to_string(), row.get(1)?.as_str()?.to_string()), re))
        })
        .collect()
});
static NATIVE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| NATIVE.iter().map(|(w, _, _)| w.clone()).collect());
/// NATIVE's indices longest word first (ties in insertion order).
static NATIVE_LONGEST_FIRST: LazyLock<Vec<usize>> = LazyLock::new(|| {
    let mut idx: Vec<usize> = (0..NATIVE.len()).collect();
    idx.sort_by(|&a, &b| text::len(&NATIVE[b].0).cmp(&text::len(&NATIVE[a].0)));
    idx
});
static A_MEANING_IS_ASKED: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_MEANING_IS_ASKED"));
static LANGUAGE_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| strs("LANGUAGE_NAMES"));
static LANGUAGE_NAME_RES: LazyLock<Vec<Re>> = LazyLock::new(|| LANGUAGE_NAMES.iter().map(|n| edged(n, false)).collect());
static ENGLISH_FURNITURE: LazyLock<HashSet<String>> = LazyLock::new(|| set("_ENGLISH_FURNITURE"));
static SAY_HOW_DO: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bsay\b|\bhow do you\b|\bhow do i\b", 0));
static MEANING_FRAME: LazyLock<&'static Re> = LazyLock::new(|| rx("_MEANING_FRAME"));
static ENGLISH_TOO: LazyLock<HashSet<String>> = LazyLock::new(|| set("_ENGLISH_TOO"));
static ABOUT_THE_PLATE: LazyLock<&'static Re> = LazyLock::new(|| rx("_ABOUT_THE_PLATE"));
static SAY_OR_TRANSLATE: LazyLock<&'static Re> = LazyLock::new(|| rx("_SAY_OR_TRANSLATE"));
static PHRASE_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_PHRASE_ASK"));
static CORRECTION_FURNITURE: LazyLock<HashSet<String>> = LazyLock::new(|| set("_CORRECTION_FURNITURE"));

/// A table's keys longest first, each with its `\bname\b` regex.
fn find_table(keys: &[String]) -> Vec<(String, Re)> {
    dup::longest_first(keys)
        .into_iter()
        .map(|k| {
            let r = Re::new(&format!(r"\b{}\b", re::escape(&k)), 0);
            (k, r)
        })
        .collect()
}
static FIND_LANGUAGES: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| find_table(&LANGUAGE_KEYS));

/// The longest key of the table (given as its keys, in order) the text names as a word, or None.
pub fn _find(text: &str, table: &[String]) -> Option<String> {
    for name in dup::longest_first(table) {
        if sh::re_search(&format!(r"\b{}\b", re::escape(&name)), text, 0) {
            return Some(name);
        }
    }
    None
}

/// `_find` over a table prepared once.
fn find_in(text: &str, table: &[(String, Re)]) -> Option<String> {
    table.iter().find(|(_, r)| r.is_match(text)).map(|(k, _)| k.clone())
}

/// True when the line asks for the meaning of a foreign word, not when it uses an English one.
pub fn _a_translation_is_asked(understood: &Understood) -> bool {
    let clean = sh::clean_or_text(understood).to_lowercase();
    if A_MEANING_IS_ASKED.is_match(&clean) {
        return true;
    }
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let content: Vec<&String> = words.iter().filter(|w| !ENGLISH_FURNITURE.contains(*w)).collect();
    let named = LANGUAGE_NAME_RES.iter().any(|r| r.is_match(&clean));
    if named
        && (content.iter().any(|w| NATIVE_WORDS.contains(*w))
            || A_MEANING_IS_ASKED.is_match(&clean)
            || SAY_HOW_DO.is_match(&clean))
    {
        return true;
    }
    !content.is_empty() && content.iter().all(|w| NATIVE_WORDS.contains(*w))
}

/// "What does <foreign word> mean", answered from the phrases table backwards; None otherwise.
pub fn _what_does_it_mean(_memory: &Memory, understood: &Understood) -> Option<String> {
    if !_a_translation_is_asked(understood) {
        return None;
    }
    let clean = understood.clean.to_lowercase();
    for &i in NATIVE_LONGEST_FIRST.iter() {
        let (_word, (language, concept), r) = &NATIVE[i];
        if r.is_match(&clean) {
            let english = if concept == "i don't understand" { "I don't understand" } else { concept.as_str() };
            // `LANGUAGES[language]`: a KeyError there declines the reply.
            let name = lookup(&LANGUAGES, language)?;
            return Some(format!(
                "That is {name} for '{english}'. It is a phrase people say a \
                 lot, so learn it whole and out loud rather than word by word, and the reply \
                 you get back will tell you whether it landed."
            ));
        }
    }
    None
}

/// True when the line asks what a word means, and the word in it is the thing asked about.
pub fn _a_word_is_being_asked_about(understood: &Understood) -> bool {
    let said = sh::clean_or_text(understood).to_lowercase();
    if !MEANING_FRAME.is_match(&said) {
        return false;
    }
    let named: Vec<&String> = NATIVE.iter().filter(|(_, _, r)| r.is_match(&said)).map(|(w, _, _)| w).collect();
    if named.is_empty() {
        return false;
    }
    if named.iter().any(|w| !ENGLISH_TOO.contains(*w)) {
        return true;
    }
    find_in(&said, &FIND_LANGUAGES).is_some() || said.contains('"') || said.contains('\u{201c}')
}

/// The PHRASES key the line is asking about, with word edges respected either side.
pub fn _concept(text: &str) -> Option<String> {
    for (alternatives, name) in CONCEPTS.iter() {
        if alternatives.iter().any(|(_, r)| r.is_match(text)) {
            return Some(name.clone());
        }
    }
    None
}

/// True when the line is a language and nothing else: `actually french`, `what about spanish`.
pub fn _a_language_alone(text: &str) -> bool {
    let words: Vec<String> =
        WORDS_RX.findall_str(text).into_iter().filter(|w| !CORRECTION_FURNITURE.contains(w)).collect();
    !words.is_empty() && words.iter().all(|w| LANGUAGE_NAMES.contains(w))
}

/// True when the line asks for a phrase in another language rather than mentioning one.
pub fn _a_phrase_is_asked(understood: &Understood) -> bool {
    let text = text::squash(&sh::clean_or_text(understood).to_lowercase());
    if PHRASE_ASK.is_match(&text) {
        return true;
    }
    if _concept(&text).is_some() {
        return true;
    }
    _a_language_alone(&text)
}

/// "How do you say <phrase> in <language>", from the PHRASES table; the whole-phrases advice
/// when the phrase is not in it; None when no language is named or the line is about food.
pub fn _say_in_language(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if ABOUT_THE_PLATE.is_match(&clean) && !SAY_OR_TRANSLATE.is_match(&clean) {
        return None;
    }
    let language = find_in(&clean, &FIND_LANGUAGES)?;
    let name = lookup(&LANGUAGES, &language)?;
    let whole_phrases = || {
        format!(
            "Learn {name} as a set of whole phrases rather than word by word, \
             and learn them out loud: 'hello', 'please', 'thank you', 'sorry, I don't understand', \
             'do you speak English?' and 'how much is that?' cover most of a short trip. Ask me any \
             of those in one language and I will give you the phrase and how to say it."
        )
    };
    let Some(concept) = _concept(&clean) else {
        return Some(whole_phrases());
    };
    let pair = PHRASES
        .iter()
        .find(|(c, _)| *c == concept)
        .and_then(|(_, rows)| rows.iter().find(|(l, _)| *l == language).map(|(_, p)| p.clone()));
    let Some(Phrase { phrase, say }) = pair else {
        return Some(whole_phrases());
    };
    let question = if concept != "i don't understand" { concept.as_str() } else { "I don't understand" };
    Some(format!(
        "'{}' in {name} is {phrase} \u{2014} say it roughly \
         \"{say}\". Learn it out loud a few times rather than reading it, and people will \
         understand you even if the pronunciation is not perfect.",
        text::capitalize(question)
    ))
}

// --- degrees of heat, failing, testing ------------------------------------------------------------

static TEMPERATURE_DEGREES: LazyLock<&'static Re> = LazyLock::new(|| rx("_TEMPERATURE_DEGREES"));
static NUMBER_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_NUMBER_WORDS"));
static FAILING: LazyLock<HashSet<String>> = LazyLock::new(|| set("_FAILING"));
static EXAM_IN_THE_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_EXAM_IN_THE_WORDS"));
static DASH_OR_SPACE: LazyLock<Re> = LazyLock::new(|| Re::new(r"[-\s]", 0));
static MY_DEGREE: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:my|the|a|his|her|their|your)\s+degree\b|\bdegree\s+(?:in|course|exam)\b", 0));
static TEST_AS_A_VERB: LazyLock<&'static Re> = LazyLock::new(|| rx("TEST_AS_A_VERB"));
static TIME_AFTER_TEST: LazyLock<&'static Re> = LazyLock::new(|| rx("TIME_AFTER_TEST"));
static LEARNING_OBJECTS: LazyLock<HashSet<String>> = LazyLock::new(|| set("LEARNING_OBJECTS"));

/// Whether any of the words is (or has a part that is) a number said in words.
pub fn _a_number_word<S: AsRef<str>>(words: &[S]) -> bool {
    words.iter().any(|w| DASH_OR_SPACE.split(w.as_ref()).iter().any(|part| NUMBER_WORDS.contains(part)))
}

/// False when "N degrees" is a temperature rather than a qualification.
pub fn _the_degrees_are_heat(understood: &Understood) -> bool {
    let text = sh::clean_or_text(understood).to_lowercase();
    let words: HashSet<&str> = understood.words.iter().map(String::as_str).collect();
    if !words.contains("degree") && !words.contains("degrees") {
        return true;
    }
    let listed: Vec<&str> = words.iter().copied().collect();
    if !(TEMPERATURE_DEGREES.is_match(&text) || _a_number_word(&listed)) {
        return true;
    }
    if words.iter().any(|w| *w != "degree" && *w != "degrees" && EXAM_IN_THE_WORDS.contains(*w)) {
        return true;
    }
    if MY_DEGREE.is_match(&text) {
        return true;
    }
    false
}

/// True when the line fails at something that is not named as an exam.
pub fn _a_failure_of_something_else(understood: &Understood) -> bool {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if !words.iter().any(|w| FAILING.contains(w)) {
        return false;
    }
    !words.iter().any(|w| EXAM_IN_THE_WORDS.contains(w))
}

/// True when "test" in this line is the verb and the thing after it is the subject.
pub fn _a_test_of_a_thing(understood: &Understood) -> bool {
    let text = sh::or(&understood.norm_text, &understood.clean);
    if text.is_empty() || !TEST_AS_A_VERB.is_match(text) {
        return false;
    }
    if TIME_AFTER_TEST.is_match(text) {
        return false;
    }
    !understood.words.iter().any(|w| LEARNING_OBJECTS.contains(&w.to_lowercase()))
}

/// The reply, unless the line is testing a thing rather than revising for a test (import time:
/// the entries hold the closure it makes, `_unless_testing_a_thing__answer`).
pub fn _unless_testing_a_thing(reply: &str) -> impl Fn(&Memory, &Understood) -> String + use<> {
    let reply = reply.to_string();
    move |memory: &Memory, understood: &Understood| _unless_testing_a_thing__answer(&reply, memory, understood)
}

/// The closure `_unless_testing_a_thing(reply)` makes: the reply, unless the line is testing a
/// thing rather than revising for a test ("" then).
#[allow(non_snake_case)]
pub fn _unless_testing_a_thing__answer(reply: &str, _memory: &Memory, understood: &Understood) -> String {
    if _a_test_of_a_thing(understood) {
        return String::new();
    }
    reply.to_string()
}

// --- nerves and forgetting ------------------------------------------------------------------------

static EXAM_NERVES: LazyLock<String> = LazyLock::new(|| string("EXAM_NERVES"));
static NERVES_BODY: LazyLock<Vec<String>> = LazyLock::new(|| strs("NERVES_BODY"));
static OTHER_PEOPLE: LazyLock<HashSet<String>> = LazyLock::new(|| set("_OTHER_PEOPLE"));
static FORGETTING: LazyLock<HashSet<String>> = LazyLock::new(|| set("_FORGETTING"));

/// The exam nerves answer, with the person's own sign named first when the line is a symptom.
pub fn _exam_nerves(_memory: &Memory, understood: &Understood) -> String {
    let said = sh::clean_or_text(understood).to_lowercase();
    if sh::any_in(&said, &NERVES_BODY) {
        return format!(
            "{}: eat something plain, take water in with you, and breathe out slowly for a \
             minute in the queue. {}",
            wording::nerves_opening(&said, "before an exam", "adrenaline, not a bug"),
            *EXAM_NERVES
        );
    }
    EXAM_NERVES.clone()
}

/// True when the forgetting in the line belongs to another person, not the speaker.
pub fn _somebody_else_forgets(understood: &Understood) -> bool {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if !words.iter().any(|w| FORGETTING.contains(w)) {
        return false;
    }
    words.iter().any(|w| OTHER_PEOPLE.contains(w))
}

// --- grades and scores --------------------------------------------------------------------------------

static A_MARK_NOT_A_GRADE: LazyLock<Vec<String>> = LazyLock::new(|| strs("A_MARK_NOT_A_GRADE"));
static COUNTED_AFTER: LazyLock<Vec<String>> = LazyLock::new(|| strs("COUNTED_AFTER"));
static GRADE_NAME: LazyLock<&'static Re> = LazyLock::new(|| rx("GRADE_NAME"));
static LETTER_GRADES: LazyLock<&'static Re> = LazyLock::new(|| rx("LETTER_GRADES"));
static TOP_GRADES: LazyLock<&'static Re> = LazyLock::new(|| rx("TOP_GRADES"));
static SCORE: LazyLock<&'static Re> = LazyLock::new(|| rx("SCORE"));
static GRADE_DIGIT: LazyLock<Re> = LazyLock::new(|| Re::new(r"([1-9])%?", 0));
static GRADE_TALK: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"\b(?:gcses?|grades?|marks|score|scores|pass|passed|mocks?|exams?)\b", 0));
static UNI_OR_COLLEGE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\buni\b|\buniversit(y|ies)\b|\bcollege\b|sixth form", 0));

/// The grade number the line is about, or None when the number is a count or a score.
pub fn _grade_in(clean: &str) -> Option<String> {
    let replaced = clean.replace(',', " ");
    let words = text::split_ref(&replaced);
    for (n, word) in words.iter().enumerate() {
        let Some(found) = GRADE_DIGIT.fullmatch(word) else {
            continue;
        };
        let before = if n > 0 { words[n - 1] } else { "" };
        let after = words.get(n + 1).copied().unwrap_or("");
        let next_two = words[(n + 1).min(words.len())..(n + 3).min(words.len())].join(" ");
        if A_MARK_NOT_A_GRADE.iter().any(|w| w == before) || word.ends_with('%') {
            continue;
        }
        if COUNTED_AFTER.iter().any(|w| w == after)
            || next_two.starts_with("a level")
            || GRADE_NAME.match_(&next_two).is_some()
            || LETTER_GRADES.match_(&next_two).is_some()
        {
            continue;
        }
        return Some(found.str(1).to_string());
    }
    None
}

/// What one GCSE grade is worth, in the words a person wants to hear.
pub fn _grade_sentence(grade: &str) -> String {
    match grade {
        "9" => "A 9 is the top grade, above the old A*.".to_string(),
        "7" | "8" => {
            let a = if grade == "8" { "An" } else { "A" };
            format!("{a} {grade} is a top grade, the old A and above.")
        }
        "6" => "A 6 is a good pass, about the old B.".to_string(),
        "5" => "A 5 is a strong pass, the old high C, and over the line most colleges ask for.".to_string(),
        "4" => "A 4 is a standard pass, the old C: you are over the line most colleges ask for, \
                and a 5 is the strong pass above it."
            .to_string(),
        _ => format!(
            "A {grade} is below a pass at GCSE. Most sixth forms and colleges take you on other \
             subjects and have you retake English or maths alongside, and that retake is free \
             until you are 18, so it does not close anything on its own."
        ),
    }
}

/// A score and the total it is out of ("4 out of 9"), with its percentage, or None.
pub fn _score_out_of(clean: &str) -> Option<(i64, i64, i64)> {
    let found = SCORE.search(clean)?;
    let got = dup::py_int(found.str(1))?;
    let total = dup::py_int(found.str(2))?;
    if !(1 <= got && got <= total && total <= 200) || total < 5 {
        return None;
    }
    Some((got, total, text::round_half_even(100.0 * got as f64 / total as f64)))
}

/// How much of the paper that score is, in the words a person uses about their own work.
pub fn _score_sentence(score: (i64, i64, i64)) -> String {
    let (got, total, percent) = score;
    let part = if (40..=60).contains(&percent) {
        "about half"
    } else if percent > 60 {
        "most"
    } else {
        "under half"
    };
    format!("That is {got} out of {total}, {part} the marks.")
}

/// "Is a grade 4 bad?": what the number is worth, and what it opens up; None for a proud count
/// of top grades or a number that is not about grades.
pub fn _grade_answer(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    if TOP_GRADES.is_match(&clean) {
        return None;
    }
    if let Some(score) = _score_out_of(&clean) {
        return Some(format!(
            "{} Treat that as a score rather than a grade, because a \
             GCSE grade is where you stand across a whole subject and one piece of work \
             cannot tell you that. Go through it and write down the questions that lost the \
             marks, then ask your teacher for the mark scheme on the worst two, since the \
             same kind of question is usually where the marks are next time.",
            _score_sentence(score)
        ));
    }
    let Some(grade) = _grade_in(&clean) else {
        if !GRADE_TALK.is_match(&clean) {
            return None;
        }
        return Some(
            "GCSE grades run from 9 down to 1: 4 is a standard pass, 5 a strong one, and 7 \
             to 9 are the top grades. Below 4 is not a pass yet, and English and maths can \
             be retaken at college until you are 18. Tell me the grade and the subject and \
             I will say what it opens up."
                .to_string(),
        );
    };
    if grade.as_str() >= "4" {
        return Some(format!(
            "{} English and maths are the two that colleges and \
             employers look at first, whatever else you take.",
            _grade_sentence(&grade)
        ));
    }
    Some(_grade_sentence(&grade))
}

/// A mock grade is a map of what to fix, not a verdict on the real exam.
pub fn _mock_answer(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    let grade = _grade_in(&clean);
    let score = _score_out_of(&clean);
    let mut answer = format!(
        "{}A mock or a predicted grade is a signal, not a verdict: find out which topics cost \
         you the marks \u{2014} the mark scheme for a mock, your teacher for a predicted grade \u{2014} \
         and give the worst two a 25-minute slot each week. Moving a grade or two in a term \
         is normal.",
        score.map(|s| format!("{} ", _score_sentence(s))).unwrap_or_default()
    );
    if let Some(grade) = grade {
        answer = format!("{} {answer}", _grade_sentence(&grade));
    }
    if UNI_OR_COLLEGE.is_match(&clean) {
        answer.push_str(
            " For the university question, what counts is what the course asks for: check \
             two or three you like and their entry requirements, because a 6 at GCSE rarely \
             closes a door on its own.",
        );
    }
    answer
}

// --- a strict teacher --------------------------------------------------------------------------------

static FEELS_WRONGED: LazyLock<&'static Re> = LazyLock::new(|| rx("_FEELS_WRONGED"));
static IS_CHILD: LazyLock<&'static Re> = LazyLock::new(|| rx("_IS_CHILD"));
static SCHOOL_THINGS: LazyLock<&'static Re> = LazyLock::new(|| rx("_SCHOOL_THINGS"));

/// False when nothing in the line is about school, so the bare hate is somebody else's.
pub fn _a_school_line(understood: &Understood) -> bool {
    let said = text::squash(&sh::clean_or_text(understood).to_lowercase());
    SCHOOL_THINGS.is_match(&said)
}

/// The strict-teacher advice, with a first sentence that takes the feeling when there is one.
pub fn _strict_teacher(_memory: &Memory, understood: &Understood) -> String {
    let text = sh::or(&understood.norm_text, &understood.clean).to_lowercase();
    let plan = "Do the boring things first and most strict teachers stop being a problem: hand work \
                in on time, sit where you are seen, and ask one question in the lesson so they know \
                your name for the right reason. If it is personal - sarcasm, singled out, marks that \
                do not match the work - ask for the marked work back in writing and take it to your \
                form tutor or head of year with dates and one example, not a general complaint. Keep \
                the tone flat and factual, because a complaint that is a feeling gets deflected and \
                one that is a date, a thing said and a mark does not. Ask a parent to come to the \
                meeting if you are at school.";
    if !FEELS_WRONGED.is_match(&text) {
        return plan.to_string();
    }
    if IS_CHILD.is_match(&text) {
        return "Take it seriously, then test it before you act: ask what happens in that lesson \
                and whether other children say the same, because most of these turn out to be \
                about one subject or one seating plan. Then ask the school for a short meeting \
                with the class teacher, and go to the head of year if nothing changes in a term. \
                Ask for the marked work so the conversation is about the marks and not about \
                whether the teacher likes him."
            .to_string();
    }
    format!(
        "That is a miserable way to spend a lesson, and it is worth testing before you believe \
         it: most teachers who seem to hate one person are strict with the whole class. {plan}"
    )
}

// --- how long a language takes --------------------------------------------------------------------

static LANGUAGE_HOURS: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    get("LANGUAGE_HOURS").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?))).collect()
});
static LANGUAGE_HOURS_KEYS: LazyLock<Vec<String>> =
    LazyLock::new(|| LANGUAGE_HOURS.iter().map(|(k, _)| k.clone()).collect());
static FIND_LANGUAGE_HOURS: LazyLock<Vec<(String, Re)>> = LazyLock::new(|| find_table(&LANGUAGE_HOURS_KEYS));

/// True when the line names a language, or says what it is about without naming one.
pub fn _names_a_language(understood: &Understood) -> bool {
    let clean = sh::clean_or_text(understood).to_lowercase();
    let (leaned_text, leaned_words) = match &understood.leans_on {
        Some(l) => (l.text.clone(), l.words.join(" ")),
        None => (String::new(), String::new()),
    };
    let said = [clean.as_str(), &understood.words.join(" "), &leaned_text, &leaned_words].join(" ").to_lowercase();
    if find_in(&said, &FIND_LANGUAGE_HOURS).is_some() {
        return true;
    }
    const ABOUT_A_LANGUAGE: &[&str] = &[
        "language", "languages", "fluent", "fluency", "fluently", "bilingual", "conversation", "conversations", "speak",
        "speaking", "spoken",
    ];
    WORDS_RX.findall_str(&said).iter().any(|w| ABOUT_A_LANGUAGE.contains(&w.as_str()))
}

/// The language the person has already named in this chat (the last six turns), or "".
pub fn _the_language_the_chat_named(memory: &Memory) -> String {
    let turns = &memory.turns;
    let recent = &turns[turns.len().saturating_sub(6)..];
    for turn in recent.iter().rev() {
        let said = turn.user.to_lowercase();
        if said.is_empty() {
            continue;
        }
        if let Some(named) = find_in(&said, &FIND_LANGUAGE_HOURS) {
            return named;
        }
    }
    String::new()
}

/// How many hours a language takes, said in the language the person named.
pub fn _language_hours(memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    let named = find_in(&clean, &FIND_LANGUAGE_HOURS).unwrap_or_else(|| _the_language_the_chat_named(memory));
    let hours = LANGUAGE_HOURS.iter().find(|(k, _)| *k == named).map(|(_, h)| *h).unwrap_or(600);
    let place = if named.is_empty() { "Spanish, French or Italian".to_string() } else { text::capitalize(&named) };
    let years = text::round_half_even(hours as f64 / 365.0).max(2);
    format!(
        "About {hours} hours of real study to hold a proper conversation in {place} - around \
         {years} years at an hour a day, or three times that at twenty minutes a day, which \
         is why a little every day beats a long weekend. A holiday and small talk come far \
         sooner: about 100 hours, which a twice-weekly evening class reaches in a year."
    )
}

// --- what was forgotten ------------------------------------------------------------------------------

static FORGET_TO_DO: LazyLock<&'static Re> = LazyLock::new(|| rx("_FORGET_TO_DO"));
static THE_PAGE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_THE_PAGE"));
static LAST_TIME: LazyLock<&'static Re> = LazyLock::new(|| rx("_LAST_TIME"));
static HOW_TO_REMEMBER: LazyLock<&'static Re> = LazyLock::new(|| rx("_HOW_TO_REMEMBER"));

/// True only when the line is about a page (or asks how to remember): a thing to do, or a life,
/// is not a page.
pub fn _the_page_is_what_was_forgotten(understood: &Understood) -> bool {
    let text = text::squash(&sh::clean_or_text(understood).to_lowercase());
    let split = text::split_ref(&text);
    if THE_PAGE.iter().any(|w| split.contains(&w.as_str())) {
        return true;
    }
    if HOW_TO_REMEMBER.is_match(&text) {
        return true;
    }
    if LAST_TIME.is_match(&text) {
        return false;
    }
    if FORGET_TO_DO.is_match(&text) {
        return false;
    }
    false
}

// --- degree classes --------------------------------------------------------------------------------

static CLASS_BAND: LazyLock<Vec<(String, i64)>> = LazyLock::new(|| {
    get("CLASS_BAND").pairs().iter().filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_int()?))).collect()
});
static CLASS_SAYS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("CLASS_SAYS"));
static CLASS_NAMES: LazyLock<Vec<(String, Vec<String>)>> = LazyLock::new(|| {
    get("CLASS_NAMES")
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.as_str()?.to_string(), row.get(1)?.strs()))
        })
        .collect()
});
static CLASS_ALONE: LazyLock<Vec<(String, &'static Re)>> = LazyLock::new(|| {
    get("CLASS_ALONE")
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            Some((row.first()?.as_str()?.to_string(), row.get(1)?.as_re()?))
        })
        .collect()
});
static NOT_THE_CLASS: LazyLock<Vec<String>> = LazyLock::new(|| strs("NOT_THE_CLASS"));
static HALF_OF_CLASS: LazyLock<Vec<(String, String, String, i64, i64)>> = LazyLock::new(|| {
    get("HALF_OF_CLASS")
        .items()
        .iter()
        .filter_map(|row| {
            let r = row.items();
            Some((
                r.first()?.as_str()?.to_string(),
                r.get(1)?.as_str()?.to_string(),
                r.get(2)?.as_str()?.to_string(),
                r.get(3)?.as_int()?,
                r.get(4)?.as_int()?,
            ))
        })
        .collect()
});
static PASS_MARK: LazyLock<String> = LazyLock::new(|| string("PASS_MARK"));

/// The classes the line names, in the order they are written (a class matched by two of its
/// patterns is listed twice, as PyBot's list does).
pub fn _classes_named(clean: &str) -> Vec<String> {
    let mut found: Vec<(usize, String)> = Vec::new();
    for (name, patterns) in CLASS_NAMES.iter() {
        for pattern in patterns {
            // `name not in found` compares a name with (start, name) pairs, so it always holds.
            if let Some(m) = sh::re_cached(pattern, 0).search(clean) {
                found.push((text::char_index(clean, m.start()), name.clone()));
            }
        }
    }
    if found.is_empty() && !sh::any_in(clean, &NOT_THE_CLASS) {
        for (name, pattern) in CLASS_ALONE.iter() {
            if let Some(m) = pattern.search(clean) {
                found.push((text::char_index(clean, m.start()), name.clone()));
            }
        }
    }
    found.sort();
    found.into_iter().map(|(_, name)| name).collect()
}

/// The bottom or top half of a band, when the line asks for it by name ("" when it does not).
pub fn _half_of_class_said(clean: &str) -> String {
    for (word, pattern, name, low, high) in HALF_OF_CLASS.iter() {
        if sh::re_search(pattern, clean, 0) {
            let side = if word == "low" { "bottom" } else { "top" };
            return format!(
                "A {word} {name} is the {side} of that band: a weighted average of \
                 {low} to {high}%, and it counts as a {name} on a certificate."
            );
        }
    }
    String::new()
}

/// `s[0].lower() + s[1:]` / `s[0].upper() + s[1:]`.
fn first_char(s: &str, upper: bool) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => {
            let head: String = if upper { f.to_uppercase().collect() } else { f.to_lowercase().collect() };
            head + c.as_str()
        }
    }
}

/// What the class the line names is (and how it sits against a second one), or what a degree
/// passes at when it names none or asks how to get one.
pub fn _university_pass_mark(_memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = understood.clean.to_lowercase();
    let asks = &understood.asks;
    let classes = _classes_named(&clean);
    if classes.is_empty() {
        return Some(PASS_MARK.clone());
    }
    if asks.iter().any(|a| a == "how" || a == "steps") {
        return Some(PASS_MARK.clone());
    }
    let defines = asks.iter().any(|a| a == "definition" || a == "compare")
        || sh::any_in(&clean, &["mean", "means", "stands for", "stand for", "difference between"])
        || sh::any_in(&clean, &["percentage", "percent", "what grade"]);
    if !defines {
        return Some(PASS_MARK.clone());
    }
    // `CLASS_SAYS[...]`: a KeyError would decline the reply.
    let says = |name: &str| lookup(&CLASS_SAYS, name).cloned();
    let half = _half_of_class_said(&clean);
    if !half.is_empty() {
        return Some(format!("In UK universities, {} In full, {}", first_char(&half, false), says(&classes[0])?));
    }
    let mut said = format!("In UK universities, {}", says(&classes[0])?);
    if classes.len() > 1 {
        said.push(' ');
        said.push_str(&first_char(&says(&classes[1])?, true));
        let band = |n: &str| CLASS_BAND.iter().find(|(k, _)| k == n).map(|(_, v)| *v);
        if let (Some(a), Some(b)) = (band(&classes[0]), band(&classes[1])) {
            let gap = (a - b).abs();
            let bands = if gap == 10 { "one band".to_string() } else { format!("{} bands", gap.div_euclid(10)) };
            said.push_str(&format!(
                " So the line between them is {gap} points of average, {bands}, and the \
                 higher one is the class a course or a job asks for by name."
            ));
        }
    }
    Some(said)
}

// --- remembering, studying later, course hours ---------------------------------------------------------

static REMEMBERS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_REMEMBERS"));
static MEMORISE: LazyLock<String> = LazyLock::new(|| string("_MEMORISE"));
static LATER_STUDY: LazyLock<&'static Re> = LazyLock::new(|| rx("_LATER_STUDY"));
static CAREER_CHANGE: LazyLock<&'static Re> = LazyLock::new(|| rx("_CAREER_CHANGE"));
static NOT_AGE_STUDY: LazyLock<&'static Re> = LazyLock::new(|| rx("_NOT_AGE_STUDY"));
static AGE_ASK: LazyLock<&'static Re> = LazyLock::new(|| rx("_AGE_ASK"));
static AGE_NUMBER: LazyLock<&'static Re> = LazyLock::new(|| rx("_AGE_NUMBER"));
static NOT_LINE_CHARS: LazyLock<Re> = LazyLock::new(|| Re::new(r"[^a-z0-9' ]+", 0));
static COURSE_HOURS: LazyLock<String> = LazyLock::new(|| string("COURSE_HOURS"));

/// The recall method, for a line that is about remembering and not about another subject.
pub fn _memorise(_memory: &Memory, understood: &Understood) -> Option<String> {
    // `" ".join(norm or plain or "")`: a plain string joins its characters.
    let joined = if !understood.norm.is_empty() {
        understood.norm.join(" ")
    } else {
        understood.plain.chars().map(String::from).collect::<Vec<_>>().join(" ")
    };
    let line = format!(" {} ", joined.to_lowercase());
    if !sh::any_in(&line, &REMEMBERS) {
        return None;
    }
    Some(MEMORISE.clone())
}

/// The age question about a course (or a change of career), which may name the course only in
/// the turn before; None for any other line.
pub fn _study_later(memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = NOT_LINE_CHARS.sub(" ", &format!(" {} ", sh::clean_or_text(understood).to_lowercase()));
    if NOT_AGE_STUDY.is_match(&clean) {
        return None;
    }
    let career = CAREER_CHANGE.is_match(&clean);
    if !(AGE_ASK.is_match(&clean) || career) {
        return None;
    }
    if !(LATER_STUDY.is_match(&clean) || career) {
        let turns = &memory.turns;
        let seen = turns[turns.len().saturating_sub(2)..].iter().any(|t| LATER_STUDY.is_match(&t.user.to_lowercase()));
        if !seen {
            return None;
        }
    }
    let age = AGE_NUMBER.search(&clean).and_then(|m| dup::py_int(m.str(1))).unwrap_or(0);
    let opener = if (18..=90).contains(&age) {
        if age > 60 {
            return Some(format!(
                "At {age} you would be older than most of the class, and that is normal on \
                 these courses: what decides it is not the age but the routine - one fixed \
                 evening in class and one more for reading, kept for the length of the \
                 course - and a subject you will still open on a bad week. Ask the college \
                 what share of last year's class finished, and whether any of it can be done \
                 in the daytime or online, because that is what makes it stick."
            ));
        }
        format!("At {age} you would not be late: ")
    } else {
        "You would not be late: ".to_string()
    };
    Some(format!(
        "{opener}evening classes, access courses, apprenticeships and part-time degrees are \
         full of people in their thirties, forties and fifties, and the ones who finish are \
         the ones with a \
         protected evening rather than the fastest readers. What decides it is the routine \u{2014} \
         one fixed evening in class and one more for reading, kept for the length of the \
         course \u{2014} and a subject you will still open on a bad week. Ask the college what share \
         of last year's class finished, because these courses are lost to work and life far \
         more than to age."
    ))
}

/// The hours a course takes, for the follow-up that names no course itself; None otherwise.
pub fn _course_hours(memory: &Memory, understood: &Understood) -> Option<String> {
    let clean = sh::clean_or_text(understood).to_lowercase();
    if LATER_STUDY.is_match(&clean) {
        return None;
    }
    let turns = &memory.turns;
    for turn in &turns[turns.len().saturating_sub(2)..] {
        let before = turn.user.to_lowercase();
        if LATER_STUDY.is_match(&before) || CAREER_CHANGE.is_match(&before) {
            return Some(COURSE_HOURS.clone());
        }
    }
    None
}

// --- jobs, grades and lengths --------------------------------------------------------------------------

/// One JOB_LENGTHS row: the job's names, the years it takes, and the route back in.
#[derive(Clone, Debug)]
pub struct JobRow {
    pub names: Vec<String>,
    pub length: String,
    pub returning: String,
}

static JOB_LENGTHS: LazyLock<Vec<(JobRow, Vec<Re>)>> = LazyLock::new(|| {
    get("JOB_LENGTHS")
        .items()
        .iter()
        .filter_map(|row| {
            let r = row.items();
            let names = r.first()?.strs();
            let res = names.iter().map(|n| edged(n, true)).collect();
            Some((
                JobRow { names, length: r.get(1)?.str().to_string(), returning: r.get(2)?.str().to_string() },
                res,
            ))
        })
        .collect()
});
static GRADES_ROUTE: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_GRADES_ROUTE"));
static CAREER_SUBJECTS: LazyLock<Vec<(Vec<Re>, String)>> = LazyLock::new(|| {
    word_rows("CAREER_SUBJECTS").into_iter().map(|(names, reply)| (names.iter().map(|n| edged(n, true)).collect(), reply)).collect()
});
static THE_GENERAL_CAREER: LazyLock<String> = LazyLock::new(|| string("THE_GENERAL_CAREER"));
static ASKS_THE_LENGTH: LazyLock<&'static Re> = LazyLock::new(|| rx("_ASKS_THE_LENGTH"));
static DONE_IT_BEFORE: LazyLock<&'static Re> = LazyLock::new(|| rx("_DONE_IT_BEFORE"));
static EXAM_WHEN: LazyLock<String> = LazyLock::new(|| string("EXAM_WHEN"));
static MOCKS: LazyLock<Re> = LazyLock::new(|| Re::new(r"(?<![a-z])mocks?(?![a-z])", 0));

/// The JOB_LENGTHS row a text names, or None.
pub fn _the_job_row(text: &str) -> Option<JobRow> {
    let words = format!(" {} ", text.to_lowercase());
    JOB_LENGTHS.iter().find(|(_, res)| res.iter().any(|r| r.is_match(&words))).map(|(row, _)| row.clone())
}

/// `memory["career"]` when it is a string.
fn career_of(memory: &Memory) -> Option<String> {
    memory.extra.get("career").and_then(|v| v.as_str()).map(str::to_string)
}

/// The years a job takes, when the line asks how long and names one (or the chat named one
/// before, kept in `memory["career"]`), or None. The job named is remembered in the memory.
pub fn _the_length_answer(clean: &str, memory: Option<&mut Memory>) -> Option<String> {
    let said = clean.to_lowercase();
    let mut row = _the_job_row(&said);
    if let Some(memory) = memory {
        match &row {
            Some(r) => {
                memory.extra.insert("career".to_string(), Value::Str(Arc::from(r.names[0].as_str())));
            }
            None => {
                if let Some(career) = career_of(memory) {
                    row = _the_job_row(&career);
                }
            }
        }
    }
    let row = row?;
    if !ASKS_THE_LENGTH.is_match(&said) {
        return None;
    }
    if !row.returning.is_empty() && DONE_IT_BEFORE.is_match(&said) {
        return Some(row.returning);
    }
    Some(row.length)
}

/// The grades a named job asks for (or the years, when the line asks how long), then the same
/// advice for all.
pub fn _job_grades(memory: &mut Memory, understood: &Understood) -> String {
    if let Some(length) = _the_length_answer(&understood.clean, Some(memory)) {
        if !length.is_empty() {
            return length;
        }
    }
    let text = format!(" {} ", understood.words.join(" "));
    let mut first = String::new();
    for (words, sentence) in GRADES_ROUTE.iter() {
        if words.iter().any(|w| text.contains(&format!(" {w} "))) {
            first = sentence.clone();
            break;
        }
    }
    if first.is_empty() {
        first = "The route sets the grades, so there is no single number: the job names the course, \
                 and the course names the subjects."
            .to_string();
    }
    format!(
        "{first} Check the entry requirements on two or three course pages before you apply, \
         because the subjects named there are what decide it - a grade below them is the thing \
         to fix first, and a course that asks for less is the fallback."
    )
}

/// The date of the summer exams, and of the mocks, which the school sets itself.
pub fn _exam_when(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    if MOCKS.is_match(&clean) {
        return "Your school sets its own mock dates, and they are usually in November or December \
                of the year before, so ask your head of year for the week rather than a \
                national one. The summer exams themselves run from the middle of May to the \
                middle of June, and the board's timetable for your subjects is on AQA, Edexcel, \
                OCR or WJEC's site about a year ahead."
            .to_string();
    }
    EXAM_WHEN.clone()
}

/// The subjects a named job asks for (or the years, when the line asks how long), or the
/// general advice. The job named is remembered in the memory.
pub fn _career_subjects(memory: &mut Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    if let Some(length) = _the_length_answer(&clean, Some(memory)) {
        if !length.is_empty() {
            return length;
        }
    }
    if let Some(row) = _the_job_row(&clean) {
        memory.extra.insert("career".to_string(), Value::Str(Arc::from(row.names[0].as_str())));
    }
    for (names, reply) in CAREER_SUBJECTS.iter() {
        if names.iter().any(|r| r.is_match(&clean)) {
            return reply.clone();
        }
    }
    THE_GENERAL_CAREER.clone()
}

// --- a child who cannot write their name yet ------------------------------------------------------------

static AGE_SAID: LazyLock<Vec<&'static Re>> =
    LazyLock::new(|| get("_AGE_SAID").items().iter().filter_map(|v| v.as_re()).collect());
static WRITING_HAND: LazyLock<String> = LazyLock::new(|| string("_WRITING_HAND"));

/// The age the line gives a child (1 to 18), or None.
pub fn _the_age_of_a_child(reading: &Understood) -> Option<i64> {
    let said = text::squash(&reading.clean.to_lowercase());
    for pattern in AGE_SAID.iter() {
        if let Some(found) = pattern.search(&said) {
            let Some(age) = dup::py_int(found.str(1)) else {
                continue;
            };
            if 0 < age && age <= 18 {
                return Some(age);
            }
        }
    }
    None
}

/// The handwriting answer at the age the line names.
pub fn the_writing_card(_memory: &Memory, reading: &Understood) -> String {
    let age = _the_age_of_a_child(reading);
    let hand = WRITING_HAND.as_str();
    match age {
        Some(a) if a <= 4 => format!(
            "At three or four that is not late at all: the shoulder and hand are still \
             building, and a child who cannot write their name yet is exactly on time. \
             {hand} Nobody should be sitting a four year old down to drill it, \
             and if a five year old cannot hold a pencil or copy a circle, that is worth a \
             word with the health visitor."
        ),
        Some(a) if a >= 7 => "At seven this is past the usual age, so get it looked at rather than practised \
                              harder: most children write a recognisable name by five or six. Ask the teacher \
                              what it looks like in class, have hearing and vision checked, and if it is just \
                              the same at school ask about handwriting practice with an occupational therapist \
                              or the SENCO. Keep the practice at home short and keep the worry off it."
            .to_string(),
        Some(_) => format!(
            "Late but still inside normal, and worth a light touch rather than a push: most \
             children write a recognisable name between four and six. {hand} \
             Ask the teacher whether the grip looks right, and if it has not moved by the \
             end of the year get hearing and vision checked."
        ),
        None => format!(
            "Five is early for a name and nothing to worry about: the shoulder and hand are still \
             building, so most children get a recognisable name somewhere between four and six. \
             {} Worth asking nursery or \
             the reception teacher whether the grip is right, and worth a word with the health \
             visitor if a child of six cannot hold a pencil or copy a circle at all.",
            hand.replace("Give the hand", "Give the hand")
        ),
    }
}

// --- changing subject or course ----------------------------------------------------------------------------

static ASKED_TO_MOVE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_ASKED_TO_MOVE"));
static THE_ANNOUNCEMENT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_THE_ANNOUNCEMENT"));
static LEAD_IN: LazyLock<Vec<String>> = LazyLock::new(|| strs("_LEAD_IN"));

/// True when the line itself asks something, rather than only announcing a change of subject.
pub fn _asks_to_move(understood: &Understood) -> bool {
    let said = format!(" {} ", text::squash(&sh::or(&understood.plain, &understood.text).to_lowercase()));
    let mut bare: String = text::strip_chars(&said, " .,!?;:-\u{2026}").to_string();
    let mut changed = true;
    while changed {
        changed = false;
        for lead in LEAD_IN.iter() {
            if bare == *lead || bare.starts_with(&format!("{lead} ")) {
                bare = text::strip_chars(&bare[lead.len()..], " ,").to_string();
                changed = true;
            }
        }
    }
    if THE_ANNOUNCEMENT.contains(&bare) {
        return false;
    }
    ASKED_TO_MOVE.iter().any(|word| {
        said.contains(&format!(" {word} "))
            || said.starts_with(&format!(" {word} "))
            || text::rstrip(&said).ends_with(&format!(" {word}"))
    })
}

// --- entry lambdas ---------------------------------------------------------------------------------------

/// remember-names' guard: the forgetting is the speaker's own.
pub fn lambda_1452(understood: &Understood) -> bool {
    !_somebody_else_forgets(understood)
}

/// The vocabulary drill's guard: the forgetting is the speaker's own.
pub fn lambda_1945(understood: &Understood) -> bool {
    !_somebody_else_forgets(understood)
}

/// The exam-resit row's guard: a bare failure names no exam.
pub fn lambda_3055(understood: &Understood) -> bool {
    !_a_failure_of_something_else(understood)
}

/// The pass-mark row's guard: no unit on the figure, and the degrees are not a temperature.
pub fn lambda_3244(understood: &Understood) -> bool {
    !dup::a_figure_with_a_unit(understood, "") && _the_degrees_are_heat(understood)
}

/// The strict-teacher row's guard: something in the line is about school.
pub fn lambda_3458(understood: &Understood) -> bool {
    _a_school_line(understood)
}

// --- registry wrappers ----------------------------------------------------------------------------------

macro_rules! guard {
    ($name:ident, $f:path) => {
        fn $name(_f: &FnRef, u: &Understood) -> bool {
            $f(u)
        }
    };
}
macro_rules! reply {
    ($name:ident, $f:path) => {
        fn $name(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
            Some($f(m, u))
        }
    };
    ($name:ident, $f:path, opt) => {
        fn $name(_f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
            $f(m, u)
        }
    };
}

guard!(g_lambda_1452, lambda_1452);
guard!(g_lambda_1945, lambda_1945);
guard!(g_lambda_3055, lambda_3055);
guard!(g_lambda_3244, lambda_3244);
guard!(g_lambda_3458, lambda_3458);
guard!(g_a_phrase_is_asked, _a_phrase_is_asked);
guard!(g_a_word_is_being_asked_about, _a_word_is_being_asked_about);
guard!(g_asks_to_move, _asks_to_move);
guard!(g_names_a_language, _names_a_language);
guard!(g_the_ending_is_written, _the_ending_is_written);
guard!(g_the_page_is_what_was_forgotten, _the_page_is_what_was_forgotten);

reply!(r_career_subjects, _career_subjects);
reply!(r_course_hours, _course_hours, opt);
reply!(r_exam_nerves, _exam_nerves);
reply!(r_exam_when, _exam_when);
reply!(r_fractions_reply, _fractions_reply);
reply!(r_grade_answer, _grade_answer, opt);
reply!(r_job_grades, _job_grades);
reply!(r_language_hours, _language_hours);
reply!(r_memorise, _memorise, opt);
reply!(r_mock_answer, _mock_answer);
reply!(r_past_papers_reply, _past_papers_reply);
reply!(r_say_in_language, _say_in_language, opt);
reply!(r_strict_teacher, _strict_teacher);
reply!(r_study_later, _study_later, opt);
reply!(r_university_pass_mark, _university_pass_mark, opt);
reply!(r_what_does_it_mean, _what_does_it_mean, opt);
reply!(r_the_writing_card, the_writing_card);

fn r_unless_testing_a_thing_answer(f: &FnRef, m: &mut Memory, u: &Understood) -> Option<String> {
    let reply = f.cell("reply")?.as_str()?;
    Some(_unless_testing_a_thing__answer(reply, m, u))
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.study:<lambda>:1452", g_lambda_1452),
    ("bot.knowledge.study:<lambda>:1945", g_lambda_1945),
    ("bot.knowledge.study:<lambda>:3055", g_lambda_3055),
    ("bot.knowledge.study:<lambda>:3244", g_lambda_3244),
    ("bot.knowledge.study:<lambda>:3458", g_lambda_3458),
    ("bot.knowledge.study:_a_phrase_is_asked:571", g_a_phrase_is_asked),
    ("bot.knowledge.study:_a_word_is_being_asked_about:487", g_a_word_is_being_asked_about),
    ("bot.knowledge.study:_asks_to_move:4827", g_asks_to_move),
    ("bot.knowledge.study:_names_a_language:1121", g_names_a_language),
    ("bot.knowledge.study:_the_ending_is_written:421", g_the_ending_is_written),
    ("bot.knowledge.study:_the_page_is_what_was_forgotten:1213", g_the_page_is_what_was_forgotten),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.study:_career_subjects:4259", r_career_subjects),
    ("bot.knowledge.study:_course_hours:3703", r_course_hours),
    ("bot.knowledge.study:_exam_nerves:831", r_exam_nerves),
    ("bot.knowledge.study:_exam_when:4009", r_exam_when),
    ("bot.knowledge.study:_fractions_reply:89", r_fractions_reply),
    ("bot.knowledge.study:_grade_answer:995", r_grade_answer),
    ("bot.knowledge.study:_job_grades:3890", r_job_grades),
    ("bot.knowledge.study:_language_hours:1171", r_language_hours),
    ("bot.knowledge.study:_memorise:3488", r_memorise),
    ("bot.knowledge.study:_mock_answer:1024", r_mock_answer),
    ("bot.knowledge.study:_past_papers_reply:78", r_past_papers_reply),
    ("bot.knowledge.study:_say_in_language:585", r_say_in_language),
    ("bot.knowledge.study:_strict_teacher:1079", r_strict_teacher),
    ("bot.knowledge.study:_study_later:3644", r_study_later),
    ("bot.knowledge.study:_university_pass_mark:1350", r_university_pass_mark),
    ("bot.knowledge.study:_unless_testing_a_thing.<locals>.answer:795", r_unless_testing_a_thing_answer),
    ("bot.knowledge.study:_what_does_it_mean:453", r_what_does_it_mean),
    ("bot.knowledge.study:the_writing_card:4711", r_the_writing_card),
];
