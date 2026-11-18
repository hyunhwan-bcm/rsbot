//! Port of PyBot's `bot/knowledge/language.py`: spelling, the words people mix up, punctuation,
//! plurals and past tenses, meanings, and how to word the things people write.

use crate::data::{self, FnRef, Value};
use crate::knowledge::d1_dup_g5 as dup;
use crate::knowledge::d1_shared::{self as sh, ShapeFn};
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::Re;
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

const M: &str = "bot.knowledge.language.";

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
/// A `{str: str}` dict global, in insertion order.
fn str_pairs(name: &str) -> Vec<(String, String)> {
    get(name)
        .pairs()
        .iter()
        .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
        .collect()
}
/// A `((words, reply), ...)` table global.
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
/// `table[key]` for an ordered `{str: str}` table.
fn lookup<'a>(table: &'a [(String, String)], key: &str) -> Option<&'a String> {
    table.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// `" ".join(str(reading.get(k) or "") for k in keys).lower()` (a key the reading lacks is "").
fn joined_lower(u: &Understood, keys: &[&str]) -> String {
    keys.iter().map(|k| sh::field(u, k)).collect::<Vec<_>>().join(" ").to_lowercase()
}

/// The first of plain, clean and text that is not empty ("" when none is).
fn plain_clean_or_text(u: &Understood) -> &str {
    for name in ["plain", "clean", "text"] {
        let said = sh::str_field(u, name);
        if !said.is_empty() {
            return said;
        }
    }
    ""
}

static WORDS_RX: LazyLock<Re> = LazyLock::new(|| Re::new(r"[a-z']+", 0));

// --- apologies, fewer and less, figures of speech --------------------------------------------------

/// An apology in words: the work one (what you did and what you'll fix) or the personal one.
pub fn _apology(_memory: &Memory, understood: &Understood) -> String {
    let words = understood.words.join(" ");
    const WORK: &[&str] = &[
        "customer", "client", "company", "order", "refund", "grovelling", "professional", "boss", "manager",
        "colleague", "work", "team", "staff", "late",
    ];
    if sh::any_in(&words, WORK) {
        return "Say what you did and what you'll fix, in that order, with no 'if': \"I'm sorry the \
                order arrived late \u{2014} I posted it on Tuesday and it should have been there on \
                Thursday. I've refunded the delivery charge.\" Then one line on what changes, and \
                leave out the 'but'."
            .to_string();
    }
    "Say what you did, not 'if you were upset': \"I'm sorry I forgot your birthday \u{2014} I had \
     it in my head as next week and that's no excuse.\" Then say what you'll do about it, \
     and leave out the 'but'."
        .to_string()
}

/// Fewer or less: the uncountable example first when the line names money, time or water.
pub fn _fewer(_memory: &Memory, understood: &Understood) -> String {
    let words = understood.words.join(" ");
    if words.contains("money") || words.contains("time") || words.contains("water") {
        return "Use 'less' for things you can't count: less time, less money, less water. \
                Use 'fewer' for things you can: fewer people, fewer cars, fewer mistakes. \
                'Less people' is the one people notice."
            .to_string();
    }
    "Use 'fewer' for things you can count and 'less' for things you can't: fewer people, \
     fewer mistakes, but less time and less money. 'Fewer people' is right; 'less people' \
     is the mistake everyone notices."
        .to_string()
}

static FIGURES: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_FIGURES"));

/// The figure of speech the line names, defined with one example; all of them when it names none.
pub fn _figure_of_speech(_memory: &Memory, understood: &Understood) -> String {
    let said = joined_lower(understood, &["clean", "text", "line"]);
    for (words, reply) in FIGURES.iter() {
        if sh::any_in(&said, words) {
            return reply.clone();
        }
    }
    "Irony says the opposite of what is meant (a fire station burning down); onomatopoeia \
     is a word that sounds like the thing (splash, buzz); alliteration repeats the opening \
     sound (slippery slope); hyperbole exaggerates on purpose (a million times). Ask me \
     about one and I'll give you its example."
        .to_string()
}

// --- dashes, commas, end marks, speaking ------------------------------------------------------------

static DASH_LOOKS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("DASH_LOOKS"));
static A_DASH_OF_SOMETHING: LazyLock<&'static Re> = LazyLock::new(|| rx("A_DASH_OF_SOMETHING"));

/// True when the line means the punctuation mark, not a dash of a liquid.
pub fn _dash_is_the_mark(reading: &Understood) -> bool {
    !A_DASH_OF_SOMETHING.is_match(&plain_clean_or_text(reading).to_lowercase())
}

/// Whether the text names one of the looks (a word, with its endings, or a two-word phrase).
fn names_a_look(text: &str, looks: &[String]) -> bool {
    _mentions(text, looks) || looks.iter().any(|look| look.contains(' ') && text.contains(look.as_str()))
}

/// The dash the line names, or the rule that tells a dash and a hyphen apart.
pub fn _dash_or_hyphen(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    for (looks, reply) in DASH_LOOKS.iter() {
        if names_a_look(&text, looks) {
            return reply.clone();
        }
    }
    "A hyphen joins words into one (well-known, two-year-old) and has no spaces. A dash \
     separates a phrase for emphasis \u{2014} like this \u{2014} and British style usually spaces an en \
     dash. Word processors can turn two hyphens into a dash for you."
        .to_string()
}

/// The general comma rule, or the however one when that is the word the line names.
pub fn _comma_rules(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    if text.contains("however") {
        return "Put the comma after however when it opens a sentence or a clause: 'However, the \
                train was late.' It goes before it in the middle of one: 'the plan was fine, \
                however late.' With two full sentences, a semicolon before and a comma after is \
                tidier: 'I wanted to go; however, I was ill.'"
            .to_string();
    }
    "A comma marks a pause the sentence needs: between the items in a list (bread, milk, \
     eggs), before and, but or which when a whole new clause follows, and around a name or \
     an aside (my brother, who lives in Leeds, is coming). Read it aloud - if you would \
     take a breath there, a comma usually belongs; if both halves could stand alone as \
     sentences, use a full stop or a semicolon instead."
        .to_string()
}

/// Which mark ends the sentence the line asks about.
pub fn _end_mark(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    if text.contains("question mark") || text.contains("questions mark") {
        return "A question mark closes a direct question: 'where are you going?' Not an indirect \
                one - 'she asked where I was going' keeps a full stop. One mark is enough, so \
                never '?!' more than once in a piece of writing."
            .to_string();
    }
    if text.contains("exclamation") {
        return "An exclamation mark is for a real exclamation - surprise, shouting, a warning - \
                and one per page is plenty. Use it instead of a full stop at the end of the \
                sentence, never in the middle, and never more than one in a row."
            .to_string();
    }
    "A full stop ends a statement, and every sentence needs one even when the next word \
     carries on the thought. Use one rather than a comma where both halves could stand \
     alone, and never put a space before it. A question takes a question mark and a real \
     exclamation an exclamation mark."
        .to_string()
}

/// The filler, the nerves or the mumble: the speaking advice for whichever is in the line.
pub fn _speak_clearly(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    if _mentions(&text, &["um", "ums", "erm", "er", "filler", "fillers", "like", "y'know"]) {
        return "At the moment you feel an 'um' coming, close your mouth and breathe instead: a \
                half-second of silence sounds calm and no one notices it, where a filler sounds \
                like doubt. Record thirty seconds of yourself on your phone and count the fillers; \
                most people halve them in a week just by hearing them once."
            .to_string();
    }
    if _mentions(
        &text,
        &[
            "confident", "confidence", "confidently", "nervous", "nerves", "shaking", "shaky", "wobble", "wobbly",
            "wavers",
        ],
    ) {
        return "Slow down and pause at the full stops rather than at the commas, and look at one \
                person for a whole sentence instead of sweeping the room - both read as \
                confidence even when you are nervous. Write the first thirty seconds word for \
                word and say it aloud five times, because getting started is the part nerves \
                spoil."
            .to_string();
    }
    "Slow to about two-thirds of your usual speed and finish the ends of words, which is \
     most of what 'clear' means. Pause at the full stops instead of running the sentences \
     together, drop your voice a little at the end of a sentence rather than letting it \
     rise, and if you are on a call, stand up - it makes the voice carry."
        .to_string()
}

/// A word, with its plural and its -ing allowed: the line says the thing, not the spelling.
pub fn _mentions<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    let said = WORDS_RX.findall_str(text);
    for word in words {
        let word = word.as_ref();
        for end in ["", "s", "es", "ing", "ed"] {
            let form = format!("{word}{end}");
            if said.iter().any(|w| *w == form) {
                return true;
            }
        }
    }
    false
}

// --- meanings ----------------------------------------------------------------------------------------

static WORD_GLOSSES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("WORD_GLOSSES"));
static EVERYDAY_WORDS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("EVERYDAY_WORDS"));
static EVERYDAY_KEYS: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| EVERYDAY_WORDS.iter().map(|(k, _)| k.as_str()).collect());
static A_MEANING_IS_ASKED: LazyLock<&'static Re> = LazyLock::new(|| rx("_A_MEANING_IS_ASKED"));
static WHAT_IS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_WHAT_IS"));
static FILLER: LazyLock<HashSet<String>> = LazyLock::new(|| set("_FILLER"));
static ASK_FOR_THE_WORD: LazyLock<String> = LazyLock::new(|| string("ASK_FOR_THE_WORD"));

/// True when the line is asking what a word or saying means, not using it.
pub fn _asks_what_a_word_means(understood: &Understood) -> bool {
    let clean = understood.clean.as_str();
    if !clean.is_empty() && A_MEANING_IS_ASKED.is_match(clean) {
        return true;
    }
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if words.is_empty() || words.len() > 6 {
        return false;
    }
    let mut rest: &[String] = &words;
    while let Some(first) = rest.first() {
        if WHAT_IS.contains(first) || FILLER.contains(first) {
            rest = &rest[1..];
        } else {
            break;
        }
    }
    rest.len() == 1 || EVERYDAY_KEYS.contains(rest.join(" ").as_str())
}

/// The table's word and the word the line said are one word, allowing a regular ending.
pub fn _same_word(phrase: &str, said: &str) -> bool {
    if text::len(phrase) < 4 || text::len(said) < 4 {
        return false;
    }
    for ending in ["s", "d", "ed", "es", "ing", "ings"] {
        if said == format!("{phrase}{ending}") || phrase == format!("{said}{ending}") {
            return true;
        }
    }
    if let Some(stem) = phrase.strip_suffix('y') {
        if said == format!("{stem}ies") {
            return true;
        }
    }
    if let Some(stem) = said.strip_suffix('y') {
        if phrase == format!("{stem}ies") {
            return true;
        }
    }
    false
}

/// The meaning of the word the line names, from the tables (WORD_GLOSSES and EVERYDAY_WORDS by
/// default), or the ask for the word when no table holds it.
pub fn _word_meaning(_memory: &Memory, understood: &Understood, tables: &[&[(String, String)]]) -> String {
    let said: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let flat = said.join(" ");
    for table in tables {
        for (phrase, reply) in table.iter() {
            if said.contains(phrase) || (phrase.contains(' ') && flat.contains(phrase.as_str())) {
                return reply.clone();
            }
        }
    }
    for table in tables {
        for (phrase, reply) in table.iter() {
            if !phrase.contains(' ') && said.iter().any(|w| _same_word(phrase, w)) {
                return reply.clone();
            }
        }
    }
    ASK_FOR_THE_WORD.clone()
}

/// `_word_meaning` with its default tables (WORD_GLOSSES, EVERYDAY_WORDS).
pub fn _word_meaning_default(memory: &Memory, understood: &Understood) -> String {
    _word_meaning(memory, understood, &[&WORD_GLOSSES, &EVERYDAY_WORDS])
}

/// The everyday table's definition, or None when the table has not got the word.
pub fn _an_everyday_word(memory: &Memory, understood: &Understood) -> Option<String> {
    let reply = _word_meaning(memory, understood, &[&EVERYDAY_WORDS]);
    if reply == *ASK_FOR_THE_WORD { None } else { Some(reply) }
}

/// What a paragraph is, or how long one is: the length ask wants the number first.
pub fn _paragraph(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    if text.contains("how long") || text.contains("how many words") || text.contains("how much") {
        return "Three or four sentences, about 60 to 100 words, is a good paragraph: long enough \
                to make one point and short enough to take in at a glance. Start a new one when \
                you move to a new idea, and if one runs past a screenful, split it where the \
                subject changes."
            .to_string();
    }
    "A paragraph is one idea in three or four sentences, about 60 to 100 words: long \
     enough to make the point and short enough to take in at a glance. Start a new one \
     when you move to a new idea, and if one runs past a screenful, split it where the \
     subject changes."
        .to_string()
}

// --- deadlines and extensions -------------------------------------------------------------------

static EXT_SUBJECTS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EXT_SUBJECTS"));
static EXT_ILL: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EXT_ILL"));
static EXT_LATE: LazyLock<&'static Re> = LazyLock::new(|| rx("_EXT_LATE"));
static EXT_LATE_WORK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_EXT_LATE_WORK"));
static TOO_LATE_TO_GET_GOOD: LazyLock<Re> = LazyLock::new(|| {
    Re::new(r"\b(?:too late|too old) to (?:get|be|become) (?:any )?(?:good|better|decent|anywhere)\b", 0)
});

/// True unless the line's only sign of a deadline is the bare word "late".
pub fn _late_names_the_work(understood: &Understood) -> bool {
    let text = sh::clean_or_text(understood).to_lowercase();
    if !EXT_LATE.is_match(&text) {
        return true;
    }
    sh::any_in(&text, &EXT_LATE_WORK)
}

/// Asking for more time on a piece of work, in the shape the line takes: refused, missed,
/// exam access, the extenuating-circumstances form, illness, keeping track, or the plain ask.
/// None for a doubt about getting good at something late in life.
pub fn _a_deadline_extension(_memory: &Memory, understood: &Understood) -> Option<String> {
    let line = understood.text.to_lowercase();
    if TOO_LATE_TO_GET_GOOD.is_match(&line) && !sh::any_in(&line, &EXT_SUBJECTS) {
        return None;
    }
    let has = |words: &[&str]| sh::any_in(&line, words);
    if has(&["said no", "says no", "refused", "refuses", "turned me down", "said i cant", "said i can't", "wouldnt give", "wouldn't give"]) {
        return Some(
            "Ask again in writing, once, with the date and the evidence attached - a refusal \
             at the desk is often the person, not the rule. Then go up one step: the module \
             leader, then the extenuating-circumstances route, which is a form the university \
             has to consider separately from the tutor's answer. Hand in the best version you \
             have by the original date whatever happens, because a late mark beats a zero."
                .to_string(),
        );
    }
    if has(&[
        "missed", "miss", "was due", "is late", "in late", "already late", "past the deadline", "overdue",
        "didnt hand in", "didn't hand in", "nowhere near done", "not done", "ran out of time", "havent done",
        "haven't done",
    ]) {
        return Some(
            "Email the tutor today, before they mark it: the date you can deliver, what you \
             have done so far, and the reason in one sentence. Universities cap late work \
             rather than refusing it, so a complete essay handed in now usually beats a \
             perfect one next month - and if the reason is illness or a bereavement, ask for \
             the extenuating-circumstances form as well, because that can lift the cap."
                .to_string(),
        );
    }
    if has(&["extra time", "access arrangements", "special arrangements", "dyslexia", "adhd", "extra 25", "rest breaks"]) {
        return Some(
            "Extra time in an exam is arranged months in advance, not on the day: the exams \
             office needs evidence - a diagnosis, or a history of extra time at school - and \
             the deadline is usually early in the year. Go to the disability or study-support \
             service with any report you have and ask what you qualify for; 25 per cent extra \
             time and rest breaks are the common ones, and the application is theirs to make."
                .to_string(),
        );
    }
    if has(&["extenuating", "mitigating", "ec form", "special circumstances"]) {
        return Some(
            "Ask the school or module office for the extenuating-circumstances form the same \
             week, and fill it in yourself: what happened, the dates it covered, and the \
             evidence - a doctor's note, a hospital letter, a death notice. It goes to a \
             panel rather than to your tutor, and it is the route that protects the mark or \
             gives you another attempt; an extension only moves the date. Keep a copy, and \
             note the deadline for the form, which is usually days after the assessment."
                .to_string(),
        );
    }
    if sh::any_in(&line, &EXT_ILL) {
        return Some(
            "Ask for the extension in writing now, name what happened in one line, and attach \
             the evidence you have - a doctor's note, a hospital letter. Then ask the office \
             for the extenuating-circumstances form: that is the route that protects the mark \
             itself, while an extension only moves the date. Do it the same week, because \
             most deadlines for that form are days, not weeks, after the assessment."
                .to_string(),
        );
    }
    if has(&["keep track", "track of", "track my", "organise", "organize", "planner", "diary", "reminder"]) {
        return Some(
            "Put every deadline in one place the day you get it - a paper week-to-view diary \
             or the calendar on your phone - and write each one a week early as well as on \
             the day. Then work backwards: the day you must start, not the day it is due. \
             Five minutes every Monday moving the next fortnight into view is what stops the \
             surprise, and check the same page for the exam dates."
                .to_string(),
        );
    }
    Some(
        "Ask before the deadline, say how much more time you need and what you'll deliver: \
         \"I'm not going to do this justice by Friday. Could I have until Tuesday? I'd then \
         send the finished draft rather than a thin one.\" A specific new date does the work \
         \u{2014} no long explanation of why you're behind."
            .to_string(),
    )
}

// --- brackets, ellipses, speech marks, lists ------------------------------------------------------

static NOT_MATHS_BRACKETS: LazyLock<&'static Re> = LazyLock::new(|| rx("_NOT_MATHS_BRACKETS"));
static NOT_A_PUNCTUATION_BRACKET: LazyLock<&'static Re> = LazyLock::new(|| rx("_NOT_A_PUNCTUATION_BRACKET"));
static BUYING_BRACKETS: LazyLock<&'static Re> = LazyLock::new(|| rx("_BUYING_BRACKETS"));
static WRITING_BRACKETS: LazyLock<&'static Re> = LazyLock::new(|| rx("_WRITING_BRACKETS"));

/// True when the line means the mark, not a sum, a band of money or a piece of metal.
pub fn _brackets_are_punctuation(reading: &Understood) -> bool {
    let low = plain_clean_or_text(reading).to_lowercase();
    if WRITING_BRACKETS.is_match(&low) {
        return true;
    }
    if NOT_A_PUNCTUATION_BRACKET.is_match(&low) || BUYING_BRACKETS.is_match(&low) {
        return false;
    }
    !NOT_MATHS_BRACKETS.is_match(&low)
}

/// What an ellipsis is and how to use it.
pub fn _ellipsis_reply(_memory: &Memory, _understood: &Understood) -> String {
    "An ellipsis is three dots (...) and it means something has been left out or left \
     hanging: a sentence that trails off, a pause, or words cut from a quote. Use it once \
     in a while, not as a habit, and in British style it closes up to the word before it: \
     'I was going to...'"
        .to_string()
}

/// How speech marks work.
pub fn _speech_marks_reply(_memory: &Memory, _understood: &Understood) -> String {
    "Speech goes in double quotation marks, and a quote inside a quote takes single ones: \
     'She said \"I am coming\" and left.' The full stop or comma goes inside the closing \
     mark when it belongs to the speech, and every new speaker starts a new line. Do not \
     close the marks and carry the same speech on after a comma."
        .to_string()
}

/// How to punctuate a list: the bulleted one, or the list inside a sentence.
pub fn _list_reply(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    if text.contains("bullet") {
        return "A bulleted list needs the same lead-in for every line and no full stops if the \
                lines are fragments: 'Bring:' then 'milk', 'eggs', 'bread'. Capitalise each line \
                or none, and keep the grammar parallel - all nouns or all verbs, not a mix."
            .to_string();
    }
    "A list inside a sentence takes commas, and semicolons when the items have commas of \
     their own: 'we need flour, which is in the cupboard; eggs, which are in the fridge; \
     and milk.' A colon introduces the list, and never after a verb: 'the things I need \
     are milk, eggs and bread', not 'are: milk'."
        .to_string()
}

// --- octopuses and octopi; simile and metaphor ----------------------------------------------------

static PLUS_FORMS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLUS_FORMS"));

/// A shape: the line holds two of the -us family's spellings ("is it octopuses or octopi").
pub fn _offers_two_of_the_family(line: &str) -> bool {
    let found: HashSet<&str> = text::split_ref(line).into_iter().filter(|w| PLUS_FORMS.iter().any(|f| f == w)).collect();
    found.len() > 1
}

/// True when the line offers two forms and asks which: "octopuses or octopi".
pub fn _asks_which_of_the_two(understood: &Understood) -> bool {
    _the_words_of(understood).iter().any(|w| w == "or")
}

static THE_TWO_FIGURES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_THE_TWO_FIGURES"));

/// The simile or the metaphor, whichever the line named; both when it named both or neither.
pub fn _the_figure_that_was_named(_memory: &Memory, understood: &Understood) -> String {
    let said = joined_lower(understood, &["clean", "text", "line", "norm_text"]);
    let named_metaphor = said.contains("metaphor");
    let named_simile = said.contains("simile");
    if named_metaphor && !named_simile {
        return lookup(&THE_TWO_FIGURES, "metaphor").cloned().unwrap_or_default();
    }
    if named_simile && !named_metaphor {
        return lookup(&THE_TWO_FIGURES, "simile").cloned().unwrap_or_default();
    }
    "A simile compares with like or as (her laugh was like a kettle), and a metaphor says \
     it is the thing (her laugh was a kettle). Both make a picture; the metaphor just \
     does it without the 'like'."
        .to_string()
}

// --- synonyms -------------------------------------------------------------------------------------

static WORD_SETS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("WORD_SETS"));
static WORD_SET_OF: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| WORD_SETS.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect());
static ASK_SPANS: LazyLock<Vec<Vec<String>>> =
    LazyLock::new(|| get("_ASK_SPANS").items().iter().map(|v| v.strs()).collect());
static CONNECTORS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_CONNECTORS"));
static PADDING: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PADDING"));
static PHRASE_KEYS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PHRASE_KEYS"));

/// The sort key `_PHRASE_KEYS` is built with (import time): longer phrases first.
pub fn lambda_2373(one: &str) -> i64 {
    -(text::split_ref(one).len() as i64)
}

/// The words the line is asking about: everything after "another word for" and its kin.
pub fn _span_after_ask(said: &[String]) -> Vec<String> {
    let mut end = 0;
    for parts in ASK_SPANS.iter() {
        let size = parts.len();
        if said.len() + 1 > size {
            for start in 0..(said.len() + 1 - size) {
                if said[start..start + size] == parts[..] && start + size > end {
                    end = start + size;
                }
            }
        }
    }
    if end == 0 {
        for (index, word) in said.iter().enumerate() {
            if CONNECTORS.contains(word) {
                end = index + 1;
            }
        }
    }
    said[end.min(said.len())..].to_vec()
}

/// The set of other words for the word the line asked about, or None so another module answers.
pub fn _synonyms(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let span = _span_after_ask(&said);
    for start in 0..span.len() {
        for key in PHRASE_KEYS.iter() {
            let parts = text::split(key);
            let stop = (start + parts.len()).min(span.len());
            if span[start..stop] == parts[..] {
                let rest = &span[stop..];
                if rest.iter().all(|w| PADDING.contains(w)) {
                    return WORD_SET_OF.get(key.as_str()).map(|s| s.to_string());
                }
            }
        }
        let mut word: &str = &span[start];
        if !WORD_SET_OF.contains_key(word) {
            if let Some(stem) = word.strip_suffix('s') {
                if WORD_SET_OF.contains_key(stem) {
                    word = stem;
                }
            }
        }
        if let Some(reply) = WORD_SET_OF.get(word) {
            if !word.contains(' ') {
                let rest = &span[start + 1..];
                if rest.iter().all(|w| PADDING.contains(w)) {
                    return Some(reply.to_string());
                }
            }
        }
    }
    None
}

// --- plurals --------------------------------------------------------------------------------------

/// One plural row: the plural and the sentence for its pattern.
#[derive(Clone, Debug)]
pub struct Plural {
    pub more: String,
    pub pattern: String,
}

static PLURAL_GROUPS: LazyLock<Vec<(String, Vec<(String, String)>)>> = LazyLock::new(|| {
    get("_PLURAL_GROUPS")
        .items()
        .iter()
        .filter_map(|row| {
            let row = row.items();
            let pattern = row.first()?.as_str()?.to_string();
            let words = row
                .get(1)?
                .pairs()
                .iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
                .collect();
            Some((pattern, words))
        })
        .collect()
});
static PLURALS: LazyLock<HashMap<String, Plural>> = LazyLock::new(|| {
    get("PLURALS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let row = v.items();
            Some((
                k.as_str()?.to_string(),
                Plural { more: row.first()?.as_str()?.to_string(), pattern: row.get(1)?.as_str()?.to_string() },
            ))
        })
        .collect()
});
static PLURAL_SINGULARS: LazyLock<HashMap<String, String>> =
    LazyLock::new(|| data::str_map("bot.knowledge.language.PLURAL_SINGULARS"));
static UNCOUNTABLE: LazyLock<HashSet<String>> = LazyLock::new(|| set("_UNCOUNTABLE"));
static PLAIN_NOUNS: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLAIN_NOUNS"));
static PLURAL_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLURAL_ASK"));
static PLURAL_ASK_FRAME: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLURAL_ASK_FRAME"));
static PLURAL_GENERAL: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLURAL_GENERAL"));
static PLURAL_RULE: LazyLock<String> = LazyLock::new(|| string("_PLURAL_RULE"));
static PLURAL_RULE_SENTENCE: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_PLURAL_RULE_SENTENCE"));
static PLURAL_LATIN_TWIN: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_PLURAL_LATIN_TWIN"));
static PLURAL_OES: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PLURAL_OES"));
static PLURAL_SAME_WORD: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PLURAL_SAME_WORD"));
static PLURAL_NO_COUNT: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PLURAL_NO_COUNT"));
static PLURAL_NONE_SENTENCE: LazyLock<String> = LazyLock::new(|| string("_PLURAL_NONE_SENTENCE"));
static PLURAL_NOT_A_NOUN_AT_ALL: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PLURAL_NOT_A_NOUN_AT_ALL"));
static PLURAL_US_SENTENCE: LazyLock<String> = LazyLock::new(|| string("_PLURAL_US_SENTENCE"));
static PLURAL_NOT_THE_NOUN: LazyLock<HashSet<String>> = LazyLock::new(|| set("_PLURAL_NOT_THE_NOUN"));

/// `{noun: noun + "s"}` for the everyday nouns no earlier plural group answers (import time: its
/// result is the last group of the bundle's `_PLURAL_GROUPS`, so "taken" is the groups before it).
pub fn _plain_plurals() -> Vec<(String, String)> {
    let mut taken: HashSet<String> = UNCOUNTABLE.clone();
    let groups = &PLURAL_GROUPS[..PLURAL_GROUPS.len().saturating_sub(1)];
    for (_pattern, words) in groups {
        taken.extend(words.iter().map(|(k, _)| k.clone()));
    }
    let mut out: Vec<(String, String)> = Vec::new();
    for one in PLAIN_NOUNS.iter() {
        if !taken.contains(one) {
            match out.iter_mut().find(|(k, _)| k == one) {
                Some(row) => row.1 = format!("{one}s"),
                None => out.push((one.clone(), format!("{one}s"))),
            }
        }
    }
    out
}

/// Every word the plural table holds, with its plural and the pattern of its group (import time:
/// the bundle's `PLURALS`).
pub fn _plurals_by_word() -> HashMap<String, Plural> {
    let mut table = HashMap::new();
    for (pattern, words) in PLURAL_GROUPS.iter() {
        for (one, more) in words {
            table.insert(one.clone(), Plural { more: more.clone(), pattern: pattern.clone() });
        }
    }
    table
}

/// The line's own words, lowercased and with the punctuation taken off the ends.
pub fn _the_words_of(understood: &Understood) -> Vec<String> {
    understood.words.iter().map(|w| sh::strip_punct(&w.to_lowercase()).to_string()).collect()
}

/// True when the line names a word the plural table has, asks about plurals in general, or
/// names a noun after the ask whose ending gives the rule.
pub fn _names_a_word_to_change(understood: &Understood) -> bool {
    let said = _the_words_of(understood);
    for word in &said {
        if PLURALS.contains_key(word) || PLURAL_SINGULARS.contains_key(word) || UNCOUNTABLE.contains(word) {
            return true;
        }
    }
    if said.iter().any(|w| PLURAL_GENERAL.contains(w)) {
        return true;
    }
    !_the_words_the_ask_names(understood).is_empty()
}

/// The words the line asks the plural of, read after the ask; empty when it names none.
pub fn _the_words_the_ask_names(understood: &Understood) -> Vec<String> {
    let said = _the_words_of(understood);
    let Some(at) = said.iter().position(|w| PLURAL_ASK.contains(w) || PLURAL_ASK_FRAME.contains(w)) else {
        return Vec::new();
    };
    said[at + 1..]
        .iter()
        .filter(|w| {
            !PLURAL_NOT_THE_NOUN.contains(*w)
                && !PLURAL_ASK.contains(w)
                && !text::is_digit(w)
                && text::len(w) > 2
                && text::is_alpha(w)
                && !PLURALS.contains_key(*w)
                && !PLURAL_SINGULARS.contains_key(*w)
                && !UNCOUNTABLE.contains(*w)
                && !PLUS_FORMS.contains(w)
        })
        .cloned()
        .collect()
}

/// The plural of a noun no group holds, and the sentence for its ending.
pub fn _the_plural_by_its_ending(word: &str) -> String {
    if PLURAL_SAME_WORD.contains(word) {
        return format!(
            "{} is its own plural - the same word for one and for many: one {word}, two {word}.",
            text::capitalize(word)
        );
    }
    if text::ends_with_any(word, &PLURAL_NO_COUNT) {
        return PLURAL_NONE_SENTENCE.replace("{Word}", &text::capitalize(word)).replace("{word}", word);
    }
    if PLURAL_NOT_A_NOUN_AT_ALL.contains(word) {
        return PLURAL_RULE.clone();
    }
    if let Some(stem) = word.strip_suffix("us") {
        let more = format!("{stem}uses");
        if let Some(twin) = lookup(&PLURAL_LATIN_TWIN, word) {
            if !twin.is_empty() {
                return format!(
                    "{} is the plural of {word}, and {twin} is the Latin form plenty of people still use. {}",
                    text::capitalize(&more),
                    *PLURAL_US_SENTENCE
                );
            }
        }
        return format!("{} is the plural of {word}. {}", text::capitalize(&more), *PLURAL_US_SENTENCE);
    }
    let chars: Vec<char> = word.chars().collect();
    let (more, rule) = if text::ends_with_any(word, &["s", "x", "z", "ch", "sh"]) {
        let end = if word.ends_with('z') && !word.ends_with("zz") { "zes" } else { "es" };
        (format!("{word}{end}"), "es")
    } else if word.ends_with('y') && chars.len() > 2 && !"aeiou".contains(chars[chars.len() - 2]) {
        (format!("{}ies", &word[..word.len() - 1]), "ies")
    } else if word.ends_with('o') {
        if PLURAL_OES.contains(word) { (format!("{word}es"), "oes") } else { (format!("{word}s"), "os") }
    } else {
        (format!("{word}s"), "plain")
    };
    let sentence = lookup(&PLURAL_RULE_SENTENCE, rule).cloned().unwrap_or_default();
    format!("{} is the plural of {word}. {sentence}", text::capitalize(&more))
}

/// The plural of the word the line asked about, with its pattern, or None to pass.
pub fn _plural_of(_memory: &Memory, understood: &Understood) -> Option<String> {
    let said = _the_words_of(understood);
    for word in &said {
        if PLURAL_GENERAL.contains(word) {
            continue;
        }
        if let Some(found) = PLURALS.get(word) {
            if found.more == *word {
                return Some(format!(
                    "{} is its own plural - the same word for one and for many: one {word}, two {word}. {}",
                    text::capitalize(word),
                    found.pattern
                ));
            }
            return Some(format!("{} is the plural of {word}. {}", text::capitalize(&found.more), found.pattern));
        }
        if let Some(one) = PLURAL_SINGULARS.get(word) {
            let pattern = PLURALS.get(one).map(|p| p.pattern.clone())?;
            return Some(format!("{} is already plural - one {one}, two {word}. {pattern}", text::capitalize(word)));
        }
        if UNCOUNTABLE.contains(word) {
            return Some(format!(
                "{} has no plural in the everyday sense: you would say some {word}, or a bit of {word}, not {word}s.",
                text::capitalize(word)
            ));
        }
    }
    if said.iter().any(|w| PLURAL_GENERAL.contains(w)) {
        return Some(PLURAL_RULE.clone());
    }
    let asked = _the_words_the_ask_names(understood);
    let distinct: HashSet<&String> = asked.iter().collect();
    if distinct.len() == 1 {
        return Some(_the_plural_by_its_ending(&asked[0]));
    }
    if !asked.is_empty() {
        return Some(PLURAL_RULE.clone());
    }
    None
}

// --- past tenses ------------------------------------------------------------------------------------

static IRREGULAR_VERBS: LazyLock<HashMap<String, (String, String)>> = LazyLock::new(|| {
    get("IRREGULAR_VERBS")
        .pairs()
        .iter()
        .filter_map(|(k, v)| {
            let row = v.items();
            Some((k.as_str()?.to_string(), (row.first()?.as_str()?.to_string(), row.get(1)?.as_str()?.to_string())))
        })
        .collect()
});
static DOUBLED_L: LazyLock<HashSet<String>> = LazyLock::new(|| set("_DOUBLED_L"));
static PAST_OF: LazyLock<&'static Re> = LazyLock::new(|| rx("_PAST_OF"));
static TWO_VERBS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_TWO_VERBS"));

/// The line in lower case (clean, else text).
pub fn _line_text(understood: &Understood) -> String {
    sh::clean_or_text(understood).to_lowercase()
}

/// The past tense a regular verb takes, by the spelling rules people are taught.
pub fn _regular_form(verb: &str) -> String {
    const VOWELS: &str = "aeiou";
    if verb.is_empty() {
        return format!("{verb}ed");
    }
    if verb.ends_with('e') {
        return format!("{verb}d");
    }
    let chars: Vec<char> = verb.chars().collect();
    let n = chars.len();
    if verb.ends_with('y') && n > 1 && !VOWELS.contains(chars[n - 2]) {
        return format!("{}ied", &verb[..verb.len() - 1]);
    }
    if DOUBLED_L.contains(verb) {
        return format!("{verb}{}ed", chars[n - 1]);
    }
    if n > 2 && !VOWELS.contains(chars[n - 1]) && !"wxy".contains(chars[n - 1]) {
        if VOWELS.contains(chars[n - 2]) && !VOWELS.contains(chars[n - 3]) {
            let (mut groups, mut previous) = (0, false);
            for &letter in &chars {
                let vowel = VOWELS.contains(letter);
                if vowel && !previous {
                    groups += 1;
                }
                previous = vowel;
            }
            if groups == 1 {
                return format!("{verb}{}ed", chars[n - 1]);
            }
        }
    }
    format!("{verb}ed")
}

/// The two forms of the irregular verb the line names after "past ... of"; None for any other.
pub fn _irregular_past(_memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    let found = PAST_OF.search(&text)?;
    let verb = found.str(1);
    let (past, participle) = IRREGULAR_VERBS.get(verb)?;
    if participle == past {
        return Some(format!(
            "The past tense of {verb} is {past}, and the past participle is the same: 'I {past} it yesterday, \
             I have {past} it before.' Most verbs just add -ed (walk, walked), so the irregular ones are the \
             ones worth learning."
        ));
    }
    Some(format!(
        "The past tense of {verb} is {past} - 'I {past} yesterday' - and the past participle is {participle}, \
         the one that goes with have: 'I have {participle}'. Most verbs just add -ed (walk, walked), so the \
         irregular ones are the ones worth learning."
    ))
}

/// The irregular row's own question: "past ... of" and a verb of the table after it.
pub fn _names_a_past_table_verb(understood: &Understood) -> bool {
    let text = _line_text(understood);
    PAST_OF.search(&text).is_some_and(|m| IRREGULAR_VERBS.contains_key(m.str(1)))
}

/// The regular row's own question: "past ... of" and a verb the irregular table does not hold.
pub fn _names_a_past_verb(understood: &Understood) -> bool {
    let text = _line_text(understood);
    match PAST_OF.search(&text) {
        None => false,
        Some(m) => !IRREGULAR_VERBS.contains_key(m.str(1)),
    }
}

/// The -ed rule with the verb the line names (the irregular answer for a table verb, the two
/// verbs for lie and lay, a doubt for a word too long to be a verb).
pub fn _regular_past(memory: &Memory, understood: &Understood) -> Option<String> {
    let text = _line_text(understood);
    let found = PAST_OF.search(&text)?;
    let verb = found.str(1);
    if IRREGULAR_VERBS.contains_key(verb) {
        return _irregular_past(memory, understood);
    }
    if let Some(reply) = lookup(&TWO_VERBS, verb) {
        return Some(reply.clone());
    }
    if text::len(verb) > 12 {
        return Some(format!(
            "I don't know a verb '{verb}'. If it is a name or a made-up word the -ed rule still gives {verb}ed, \
             and if it is a typo the nearest real verb is the one to take."
        ));
    }
    Some(format!(
        "{} -> {}. Regular verbs add -ed; after an e it is just -d (hope -> hoped), a consonant and a y \
         becomes -ied (carry -> carried), and a one-syllable verb doubles its last letter (stop -> stopped). \
         If none of that fits, it is irregular: swim -> swam, buy -> bought.",
        text::capitalize(verb),
        _regular_form(verb)
    ))
}

// --- grammar terms, poems, emails -------------------------------------------------------------------

static GRAMMAR_TERMS: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("GRAMMAR_TERMS"));
static SOMEBODY_FORGETTING: LazyLock<&'static Re> = LazyLock::new(|| rx("_SOMEBODY_FORGETTING"));
static POEM_FORMS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_POEM_FORMS"));

/// The part of speech (or other grammar term) the line names, as a word; the overview when none.
pub fn _grammar_term(_memory: &Memory, understood: &Understood) -> String {
    let said: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let flat = said.join(" ");
    for (phrase, reply) in GRAMMAR_TERMS.iter() {
        let plural = format!("{}s", phrase.trim_end_matches('s'));
        if said.contains(phrase) || said.contains(&plural) {
            return reply.clone();
        }
        if phrase.contains(' ') && flat.contains(phrase.as_str()) {
            return reply.clone();
        }
    }
    "The parts of a sentence, in the order they turn up: a noun names something, a verb is \
     what it does or is, an adjective describes it, an adverb says how the doing happened, \
     and a pronoun stands in for the noun. 'The tired dog slept badly' - dog is the noun, \
     tired the adjective, slept the verb, badly the adverb."
        .to_string()
}

/// True unless the line is about a person losing their words, not about the words.
pub fn _nobody_in_the_line_is_forgetting(understood: &Understood) -> bool {
    let said = text::squash(sh::clean_or_text(understood)).to_lowercase();
    !SOMEBODY_FORGETTING.is_match(&said)
}

/// The form of poem the line names (sonnet, stanza, ...), or the three common ones.
pub fn _poem_form(_memory: &Memory, understood: &Understood) -> String {
    let said: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    let flat = said.join(" ");
    for (words, reply) in POEM_FORMS.iter() {
        for one in words {
            if said.contains(one) || (one.contains(' ') && flat.contains(one.as_str())) {
                return reply.clone();
            }
        }
    }
    "A haiku is three lines of five, seven and five sounds; a limerick is five lines \
     rhyming aabba with a joke in it; a sonnet is fourteen lines in iambic pentameter. Ask \
     about one and I'll give you its shape."
        .to_string()
}

/// The shape of an email, with the sample subject line in the world the line names.
pub fn _write_an_email(_memory: &Memory, understood: &Understood) -> String {
    format!(
        "Put the ask in the subject line and in the first sentence: \"{}\" tells them what it \
         is before they open it. Then three or four sentences - what you want, the one or two \
         facts they need, and by when - and a plain sign-off with your name. If you want an \
         answer, say what happens next: \"If I do not hear back by Friday I will call.\"",
        dup::example_subject(understood)
    )
}

// --- verse asked for, openings, minutes -------------------------------------------------------------

static VERSE_INSTANCE_LEAD: LazyLock<&'static Re> = LazyLock::new(|| rx("_VERSE_INSTANCE_LEAD"));
static VERSE_METHOD_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("_VERSE_METHOD_WORDS"));
static AN_OPENING_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| rx("_AN_OPENING_QUESTION"));
static AN_OPENING_IS_WRITTEN: LazyLock<&'static Re> = LazyLock::new(|| rx("_AN_OPENING_IS_WRITTEN"));
static A_COUNTED_MINUTE: LazyLock<&'static Re> = LazyLock::new(|| rx("A_COUNTED_MINUTE"));
static EVERY_MINUTE: LazyLock<&'static Re> = LazyLock::new(|| rx("EVERY_MINUTE"));

/// True when the line asks for a poem of this form rather than for its rules.
pub fn _asks_for_the_verse(understood: &Understood) -> bool {
    let words: Vec<String> = understood.words.iter().map(|w| w.to_lowercase()).collect();
    if words.is_empty() {
        return false;
    }
    if words.iter().any(|w| VERSE_METHOD_WORDS.contains(w) && w != "write" && w != "writing") {
        return false;
    }
    VERSE_INSTANCE_LEAD.match_(&words.join(" ")).is_some()
}

/// write-a-limerick's guard: the line asks how the form works, not for a limerick.
pub fn lambda_3376(understood: &Understood) -> bool {
    !_asks_for_the_verse(understood)
}

/// True unless the line asks for a question to ask: an interview question is not a first line.
pub fn _an_opening_is_written_not_asked(understood: &Understood) -> bool {
    let text = sh::clean_or_text(understood).to_lowercase();
    if !AN_OPENING_QUESTION.is_match(&text) {
        return true;
    }
    AN_OPENING_IS_WRITTEN.is_match(&text)
}

/// True when the line has a "minutes" that is not a count of them (the minutes of a meeting).
pub fn _minutes_are_a_length_of_time(understood: &Understood) -> bool {
    let text = text::squash(&understood.clean.to_lowercase());
    EVERY_MINUTE.finditer(&text).len() > A_COUNTED_MINUTE.finditer(&text).len()
}

// --- captions ---------------------------------------------------------------------------------------

static PHOTO_OF: LazyLock<Re> =
    LazyLock::new(|| Re::new(r"(?:photo|picture|pic|snap|shot|image)\s+(?:of|showing)\s+(.+)$", 0));
static ME_US_WE: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:me|us|we)\b", 0));
static STANDALONE_I: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bi\b", 0));
static PEOPLE_PHOTO: LazyLock<Vec<String>> = LazyLock::new(|| strs("_PEOPLE_PHOTO"));
static PHOTO_DETAIL: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_PHOTO_DETAIL"));
static PHOTO_DETAIL_PEOPLE: LazyLock<String> = LazyLock::new(|| string("_PHOTO_DETAIL_PEOPLE"));
static PHOTO_DETAIL_ANY: LazyLock<String> = LazyLock::new(|| string("_PHOTO_DETAIL_ANY"));

/// What is in the picture, when the line says ("a photo of my dog in the snow"); "" otherwise.
pub fn _the_photo_of(clean: &str) -> String {
    let Some(found) = PHOTO_OF.search(clean) else {
        return String::new();
    };
    let words = WORDS_RX.findall_str(found.str(1));
    let mut kept: Vec<String> = Vec::new();
    for word in words {
        if ["and", "but", "that", "which", "please", "for", "from", "on", "at"].contains(&word.as_str()) {
            break;
        }
        kept.push(word);
    }
    while kept.last().is_some_and(|w| ["i", "me", "my", "the", "a", "an", "just", "that"].contains(&w.as_str())) {
        kept.pop();
    }
    if kept.is_empty() || kept.len() > 6 {
        return String::new();
    }
    if kept.iter().all(|w| ["my", "the", "a", "an", "our", "some"].contains(&w.as_str())) {
        return String::new();
    }
    kept.join(" ")
}

/// The half of the caption the picture cannot show, chosen for the kind of picture.
pub fn _the_caption_detail(subject: &str) -> String {
    let said = format!(" {subject} ");
    if PEOPLE_PHOTO.iter().any(|w| said.contains(&format!(" {w} "))) || ME_US_WE.is_match(subject) {
        return PHOTO_DETAIL_PEOPLE.clone();
    }
    for (words, detail) in PHOTO_DETAIL.iter() {
        if words.iter().any(|w| said.contains(&format!(" {w} "))) {
            return detail.clone();
        }
    }
    PHOTO_DETAIL_ANY.clone()
}

/// The one-line caption rule, with an example built from the picture the person named.
pub fn _a_caption(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    let subject = _the_photo_of(&clean);
    if subject.is_empty() {
        return "Say what is in the picture and one thing the picture cannot show, then stop: \
                \"Third attempt at the sourdough - this one actually rose.\" One line beats \
                three, a joke only works if it is about the picture, and leave the hashtags to a \
                line of their own."
            .to_string();
    }
    let said = text::upper_first(&subject);
    let typed = STANDALONE_I.sub("I", &said);
    let example = format!("{typed}, {}", _the_caption_detail(&subject));
    format!(
        "Say what is in the picture and one thing the picture cannot show, then stop: \"{example}.\" \
         One line beats three, a joke only works if it is about the picture, and leave the \
         hashtags to a line of their own."
    )
}

// --- a kind word -------------------------------------------------------------------------------------

static AFTER_ABOUT_FOR_TO: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(?:about|for|to)\s+(.+)$", 0));
static AFTER_ABOUT: LazyLock<Re> = LazyLock::new(|| Re::new(r"\babout\s+(.+)$", 0));
static KIND_GROUPS: LazyLock<Vec<(&'static str, Vec<String>)>> = LazyLock::new(|| {
    vec![
        ("stranger", strs("_STRANGER_KIND")),
        ("animal", strs("_ANIMAL_KIND")),
        ("person", strs("_PEOPLE_KIND")),
        ("bake", strs("_BAKE_KIND")),
        ("looks", strs("_LOOKS_KIND")),
        ("place", strs("_PLACE_KIND")),
    ]
});
static A_KIND_WORD: LazyLock<Vec<(String, String)>> = LazyLock::new(|| str_pairs("_A_KIND_WORD"));
static A_MATTER_TO_RAISE: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_MATTER_TO_RAISE"));

/// Who the line is about, from the words after about/for/to, or the words anywhere in it.
pub fn _who_the_kind_word_is_for(clean: &str) -> String {
    let after = AFTER_ABOUT_FOR_TO.search(clean);
    let said = format!(" {} ", after.as_ref().map(|m| m.str(1)).unwrap_or(clean));
    let whole = format!(" {clean} ");
    for (place, words) in KIND_GROUPS.iter() {
        if words.iter().any(|w| said.contains(&format!(" {w} "))) {
            return place.to_string();
        }
        if words.iter().any(|w| whole.contains(&format!(" {w} "))) {
            return place.to_string();
        }
    }
    "someone".to_string()
}

/// A compliment the person can copy, chosen for what the line says it is about.
pub fn _a_kind_word(_memory: &Memory, understood: &Understood) -> String {
    let clean = understood.clean.to_lowercase();
    let who = _who_the_kind_word_is_for(&clean);
    lookup(&A_KIND_WORD, &who).cloned().unwrap_or_default()
}

/// True unless the line names a thing to raise with somebody, rather than a thing to praise.
pub fn _the_matter_is_worth_praise(understood: &Understood) -> bool {
    let clean = understood.clean.to_lowercase();
    let Some(after) = AFTER_ABOUT.search(&clean) else {
        return true;
    };
    let matter = format!(" {} ", text::squash(after.str(1)));
    !A_MATTER_TO_RAISE.iter().any(|w| matter.contains(&format!(" {w} ")))
}

// --- a mark with no name ------------------------------------------------------------------------------

static MARK_BY_LOOK: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_MARK_BY_LOOK"));

/// The mark from the way it looks or the key it sits on, else how to describe it.
pub fn _mark_unnamed(_memory: &Memory, understood: &Understood) -> String {
    let text = understood.clean.to_lowercase();
    for (looks, reply) in MARK_BY_LOOK.iter() {
        if names_a_look(&text, looks) {
            return reply.clone();
        }
    }
    for (sign, name) in [
        ("#", "hashtag"),
        ("@", "at sign"),
        ("|", "pipe"),
        ("~", "tilde"),
        ("*", "asterisk"),
        ("^", "caret"),
        ("_", "underscore"),
    ] {
        if text.contains(sign) {
            // `GRAMMAR_TERMS[name]`: a missing name would raise KeyError (the reply declines).
            return lookup(&GRAMMAR_TERMS, name).cloned().unwrap_or_else(|| panic!("KeyError: {name}"));
        }
    }
    for (name, reply) in GRAMMAR_TERMS.iter() {
        if text.contains(name.as_str()) {
            return reply.clone();
        }
    }
    "Tell me which key it is on and I will name it: the tall line on the left of the \
     keyboard is the pipe (|), the mark over 2 is the at sign (@), over 3 the hash (#), \
     and the wavy one on the left is a tilde (~). If you can paste it into a line, I can \
     read it too."
        .to_string()
}

// --- registry wrappers ----------------------------------------------------------------------------

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

guard!(g_lambda_3376, lambda_3376);
guard!(g_an_opening_is_written_not_asked, _an_opening_is_written_not_asked);
guard!(g_asks_what_a_word_means, _asks_what_a_word_means);
guard!(g_asks_which_of_the_two, _asks_which_of_the_two);
guard!(g_brackets_are_punctuation, _brackets_are_punctuation);
guard!(g_dash_is_the_mark, _dash_is_the_mark);
guard!(g_late_names_the_work, _late_names_the_work);
guard!(g_minutes_are_a_length_of_time, _minutes_are_a_length_of_time);
guard!(g_names_a_past_table_verb, _names_a_past_table_verb);
guard!(g_names_a_past_verb, _names_a_past_verb);
guard!(g_names_a_word_to_change, _names_a_word_to_change);
guard!(g_nobody_in_the_line_is_forgetting, _nobody_in_the_line_is_forgetting);
guard!(g_the_matter_is_worth_praise, _the_matter_is_worth_praise);

reply!(r_a_caption, _a_caption);
reply!(r_a_deadline_extension, _a_deadline_extension, opt);
reply!(r_a_kind_word, _a_kind_word);
reply!(r_an_everyday_word, _an_everyday_word, opt);
reply!(r_apology, _apology);
reply!(r_comma_rules, _comma_rules);
reply!(r_dash_or_hyphen, _dash_or_hyphen);
reply!(r_ellipsis_reply, _ellipsis_reply);
reply!(r_end_mark, _end_mark);
reply!(r_fewer, _fewer);
reply!(r_figure_of_speech, _figure_of_speech);
reply!(r_grammar_term, _grammar_term);
reply!(r_irregular_past, _irregular_past, opt);
reply!(r_list_reply, _list_reply);
reply!(r_mark_unnamed, _mark_unnamed);
reply!(r_paragraph, _paragraph);
reply!(r_plural_of, _plural_of, opt);
reply!(r_poem_form, _poem_form);
reply!(r_regular_past, _regular_past, opt);
reply!(r_speak_clearly, _speak_clearly);
reply!(r_speech_marks_reply, _speech_marks_reply);
reply!(r_synonyms, _synonyms, opt);
reply!(r_the_figure_that_was_named, _the_figure_that_was_named);
reply!(r_word_meaning, _word_meaning_default);
reply!(r_write_an_email, _write_an_email);

fn s_offers_two_of_the_family(_f: &FnRef, line: &str) -> bool {
    _offers_two_of_the_family(line)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.language:<lambda>:3376", g_lambda_3376),
    ("bot.knowledge.language:_an_opening_is_written_not_asked:3477", g_an_opening_is_written_not_asked),
    ("bot.knowledge.language:_asks_what_a_word_means:548", g_asks_what_a_word_means),
    ("bot.knowledge.language:_asks_which_of_the_two:846", g_asks_which_of_the_two),
    ("bot.knowledge.language:_brackets_are_punctuation:784", g_brackets_are_punctuation),
    ("bot.knowledge.language:_dash_is_the_mark:125", g_dash_is_the_mark),
    ("bot.knowledge.language:_late_names_the_work:674", g_late_names_the_work),
    ("bot.knowledge.language:_minutes_are_a_length_of_time:3544", g_minutes_are_a_length_of_time),
    ("bot.knowledge.language:_names_a_past_table_verb:2883", g_names_a_past_table_verb),
    ("bot.knowledge.language:_names_a_past_verb:2893", g_names_a_past_verb),
    ("bot.knowledge.language:_names_a_word_to_change:2572", g_names_a_word_to_change),
    ("bot.knowledge.language:_nobody_in_the_line_is_forgetting:3062", g_nobody_in_the_line_is_forgetting),
    ("bot.knowledge.language:_the_matter_is_worth_praise:3806", g_the_matter_is_worth_praise),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.language:_a_caption:3673", r_a_caption),
    ("bot.knowledge.language:_a_deadline_extension:682", r_a_deadline_extension),
    ("bot.knowledge.language:_a_kind_word:3781", r_a_kind_word),
    ("bot.knowledge.language:_an_everyday_word:612", r_an_everyday_word),
    ("bot.knowledge.language:_apology:28", r_apology),
    ("bot.knowledge.language:_comma_rules:146", r_comma_rules),
    ("bot.knowledge.language:_dash_or_hyphen:135", r_dash_or_hyphen),
    ("bot.knowledge.language:_ellipsis_reply:799", r_ellipsis_reply),
    ("bot.knowledge.language:_end_mark:161", r_end_mark),
    ("bot.knowledge.language:_fewer:42", r_fewer),
    ("bot.knowledge.language:_figure_of_speech:80", r_figure_of_speech),
    ("bot.knowledge.language:_grammar_term:3033", r_grammar_term),
    ("bot.knowledge.language:_irregular_past:2854", r_irregular_past),
    ("bot.knowledge.language:_list_reply:813", r_list_reply),
    ("bot.knowledge.language:_mark_unnamed:3957", r_mark_unnamed),
    ("bot.knowledge.language:_paragraph:625", r_paragraph),
    ("bot.knowledge.language:_plural_of:2728", r_plural_of),
    ("bot.knowledge.language:_poem_form:3111", r_poem_form),
    ("bot.knowledge.language:_regular_past:2914", r_regular_past),
    ("bot.knowledge.language:_speak_clearly:178", r_speak_clearly),
    ("bot.knowledge.language:_speech_marks_reply:806", r_speech_marks_reply),
    ("bot.knowledge.language:_synonyms:2393", r_synonyms),
    ("bot.knowledge.language:_the_figure_that_was_named:867", r_the_figure_that_was_named),
    ("bot.knowledge.language:_word_meaning:588", r_word_meaning),
    ("bot.knowledge.language:_write_an_email:3151", r_write_an_email),
];

pub const SHAPES: &[(&str, ShapeFn)] = &[(
    "bot.knowledge.language:_offers_two_of_the_family:836",
    s_offers_two_of_the_family,
)];
