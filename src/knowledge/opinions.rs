//! Port of `bot/knowledge/opinions.py`: the bot's own taste on small choices, given briefly with
//! a reason, and the trade that settles a decision. Tables, replies and entries are data from the
//! bundle.
//!
//! Build-time only: `_either`'s outer body (it builds the entry); its inner `answer` is ported.

use crate::data;
use crate::data::{FnRef, Value};
use crate::knowledge::d2_shared::asks_where_a_shop_answers;
use crate::knowledge::d2dup_g9::the_book_is_a_reservation;
use crate::knowledge::registry::{GuardFn, ReplyFn};
use crate::py::re::{escape, Re};
use crate::py::text;
use crate::state::Memory;
use crate::understood::Understood;
use std::collections::HashSet;
use std::sync::LazyLock;

fn strs(name: &str) -> Vec<String> {
    data::strs(&format!("bot.knowledge.opinions.{name}"))
}
fn string(name: &str) -> String {
    data::string(&format!("bot.knowledge.opinions.{name}"))
}
fn re(name: &str) -> &'static Re {
    data::re(&format!("bot.knowledge.opinions.{name}"))
}
fn set(name: &str) -> HashSet<String> {
    data::set(&format!("bot.knowledge.opinions.{name}"))
}
/// `\b<word>\b` for each word: `_says_one_of`'s whole-word test.
fn whole_words<S: AsRef<str>>(words: &[S]) -> Vec<Re> {
    words.iter().map(|w| Re::new(&format!(r"\b{}\b", escape(w.as_ref())), 0)).collect()
}
fn word_rows(name: &str) -> Vec<(Vec<String>, String)> {
    data::get(&format!("bot.knowledge.opinions.{name}"))
        .items()
        .iter()
        .map(|row| (row.items()[0].strs(), row.items()[1].str().to_string()))
        .collect()
}

static ASK_MARKERS: LazyLock<HashSet<String>> = LazyLock::new(|| set("ASK_MARKERS"));
static OWN_OPINION: LazyLock<&'static Re> = LazyLock::new(|| re("OWN_OPINION"));
static FIRST_PERSON: LazyLock<HashSet<String>> = LazyLock::new(|| set("FIRST_PERSON"));
static INTENT: LazyLock<Vec<String>> = LazyLock::new(|| strs("INTENT"));
static EXPLAINS: LazyLock<&'static Re> = LazyLock::new(|| re("EXPLAINS"));
static _A_PART: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_PART"));
static _A_JOB: LazyLock<Vec<String>> = LazyLock::new(|| strs("_A_JOB"));
static _BUYING: LazyLock<Vec<String>> = LazyLock::new(|| strs("_BUYING"));
static _GADGET_TIPS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_GADGET_TIPS"));
static _VERDICT_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("_VERDICT_ASK"));
static _THE_VERDICT: LazyLock<String> = LazyLock::new(|| string("_THE_VERDICT"));
static CHOICE: LazyLock<HashSet<String>> = LazyLock::new(|| set("CHOICE"));
static CHOICE_ASKS: LazyLock<Vec<String>> = LazyLock::new(|| strs("CHOICE_ASKS"));
static CHOICE_IN_WORDS: LazyLock<&'static Re> = LazyLock::new(|| re("CHOICE_IN_WORDS"));
static CHOICE_ASK_WORDS: LazyLock<HashSet<String>> = LazyLock::new(|| set("CHOICE_ASK_WORDS"));
static TOO_LATE: LazyLock<Vec<String>> = LazyLock::new(|| strs("TOO_LATE"));
static DRUMS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DRUMS"));
static OTHER_INSTRUMENTS: LazyLock<Vec<String>> = LazyLock::new(|| strs("OTHER_INSTRUMENTS"));
static HEALTH_ASK: LazyLock<Vec<String>> = LazyLock::new(|| strs("HEALTH_ASK"));
static INSTRUMENT_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("INSTRUMENT_WORDS"));
static INSTRUMENT_LOOKBACK: LazyLock<usize> =
    LazyLock::new(|| data::get("bot.knowledge.opinions.INSTRUMENT_LOOKBACK").as_int().unwrap_or(6) as usize);
static CANT_DECIDE: LazyLock<Vec<String>> = LazyLock::new(|| strs("CANT_DECIDE"));
static OUT_ALL_DAY: LazyLock<Vec<String>> = LazyLock::new(|| strs("OUT_ALL_DAY"));
static CAT_WINS: LazyLock<Vec<String>> = LazyLock::new(|| strs("CAT_WINS"));
static DOG_WINS: LazyLock<Vec<String>> = LazyLock::new(|| strs("DOG_WINS"));
static TRAIN_WORDS: LazyLock<Vec<String>> = LazyLock::new(|| strs("TRAIN_WORDS"));
static TRAIN_COMPARE: LazyLock<String> = LazyLock::new(|| string("TRAIN_COMPARE"));
static CAT_ON_A_DIMENSION: LazyLock<String> = LazyLock::new(|| string("CAT_ON_A_DIMENSION"));
static DOG_ON_A_DIMENSION: LazyLock<String> = LazyLock::new(|| string("DOG_ON_A_DIMENSION"));
static CAT_TRADE: LazyLock<String> = LazyLock::new(|| string("CAT_TRADE"));
static CAT_WORDS: LazyLock<Vec<Re>> = LazyLock::new(|| whole_words(&strs("CAT_WORDS")));
static CAT_EQUIPMENT: LazyLock<Vec<Re>> = LazyLock::new(|| whole_words(&strs("CAT_EQUIPMENT")));
static QUALIFIED: LazyLock<Vec<Re>> = LazyLock::new(|| whole_words(&strs("QUALIFIED")));
static TWO_WAYS: LazyLock<Vec<Vec<Re>>> = LazyLock::new(|| {
    data::get("bot.knowledge.opinions.TWO_WAYS").items().iter().map(|w| whole_words(&w.strs())).collect()
});
static DECISION_SUBJECTS: LazyLock<Vec<(Vec<Re>, String)>> = LazyLock::new(|| {
    word_rows("DECISION_SUBJECTS").into_iter().map(|(words, trade)| (whole_words(&words), trade)).collect()
});
static MADE_UP: LazyLock<&'static Re> = LazyLock::new(|| re("MADE_UP"));
static A_NEW_ONE: LazyLock<&'static Re> = LazyLock::new(|| re("A_NEW_ONE"));
static _WHAT_TO_BUY: LazyLock<&'static Re> = LazyLock::new(|| re("_WHAT_TO_BUY"));
static _TWO_OF_THEM: LazyLock<&'static Re> = LazyLock::new(|| re("_TWO_OF_THEM"));
static _ANOTHER_CATS_QUESTION: LazyLock<&'static Re> = LazyLock::new(|| re("_ANOTHER_CATS_QUESTION"));
static _NORMAL_SUBJECTS: LazyLock<Vec<(Vec<String>, String)>> = LazyLock::new(|| word_rows("_NORMAL_SUBJECTS"));
static _A_MANNERS_WORD: LazyLock<&'static Re> = LazyLock::new(|| re("_A_MANNERS_WORD"));
static _A_TABLE_BILL: LazyLock<&'static Re> = LazyLock::new(|| re("_A_TABLE_BILL"));
static _A_SHARED_HOME: LazyLock<&'static Re> = LazyLock::new(|| re("_A_SHARED_HOME"));

static CAT_OR_KITTEN: LazyLock<Vec<Re>> = LazyLock::new(|| whole_words(&["cat", "cats", "kitten", "kittens"]));
static DOG_OR_PUPPY: LazyLock<Vec<Re>> = LazyLock::new(|| whole_words(&["dog", "dogs", "puppy", "puppies"]));
static BETWEEN: LazyLock<Re> = LazyLock::new(|| Re::new(r"\bbetween\b", 0));
static STILL_DECIDING: LazyLock<Re> = LazyLock::new(|| Re::new(r"\b(cant|can't|cannot|whether|which|torn|unsure)\b", 0));

/// One instrument's answers (`INSTRUMENTS[key]`, or `INSTRUMENT_GENERIC` with no names).
pub struct Instrument {
    pub names: Vec<String>,
    pub hint: String,
    pub start: String,
    pub practise: String,
    pub how_long: String,
    pub how_often: String,
    pub generic: bool,
}

fn instrument(v: &Value, generic: bool) -> Instrument {
    let s = |k: &str| v.get(k).map(|x| x.str().to_string()).unwrap_or_default();
    Instrument {
        names: v.get("names").map(|x| x.strs()).unwrap_or_default(),
        hint: s("hint"),
        start: s("start"),
        practise: s("practise"),
        how_long: s("how_long"),
        how_often: s("how_often"),
        generic,
    }
}

static INSTRUMENTS: LazyLock<Vec<Instrument>> = LazyLock::new(|| {
    data::get("bot.knowledge.opinions.INSTRUMENTS").pairs().iter().map(|(_, v)| instrument(v, false)).collect()
});
static INSTRUMENT_GENERIC: LazyLock<Instrument> =
    LazyLock::new(|| instrument(data::get("bot.knowledge.opinions.INSTRUMENT_GENERIC"), true));

fn any_in<S: AsRef<str>>(text: &str, words: &[S]) -> bool {
    words.iter().any(|w| text.contains(w.as_ref()))
}

fn word_set(understood: &Understood) -> HashSet<&str> {
    understood.words.iter().map(String::as_str).collect()
}

fn names_you(words: &HashSet<&str>, with_yours: bool) -> bool {
    ["you", "your", "yourself"].iter().any(|w| words.contains(w)) || (with_yours && words.contains("yours"))
}

/// The buy-it-or-not rule for a machine the table has no row for, worded about its name.
pub fn _worth_an_appliance(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    const NAMES: [&[&str]; 12] = [
        &["robot vacuum", "robot vacuums", "robot hoover"],
        &["dishwasher", "dishwashers"],
        &["tumble dryer", "tumble dryers", "dryer"],
        &["washing machine", "washing machines"],
        &["slow cooker", "slow cookers"],
        &["bread maker", "bread makers"],
        &["blender", "blenders"],
        &["food processor", "food processors"],
        &["pressure cooker", "pressure cookers"],
        &["vacuum", "vacuum cleaner"],
        &["hoover", "hoovers"],
        &["freezer", "freezers"],
    ];
    let line = if understood.text.is_empty() { &understood.clean } else { &understood.text };
    let mut said = line.to_lowercase();
    if any_in(&said, &_A_PART) && any_in(&said, &_A_JOB) && !any_in(&said, &_BUYING) {
        return Some(String::new());
    }
    if said.contains("heat pump")
        && (said.contains("dryer") || said.contains("drier"))
        && (any_in(&said, &_BUYING) || any_in(&said, &["any good", "good", "worth", "worth it", "should i get", "should i buy", "get one"]))
    {
        return None;
    }
    for other in ["hair dryer", "hair drier", "hairdryer", "nail dryer", "vacuum flask", "flask", "thermos"] {
        said = said.replace(other, " ");
    }
    let mut thing: Option<String> = None;
    for forms in NAMES {
        if forms.iter().any(|f| said.contains(f)) {
            let mut t = forms[0].to_string();
            if forms[0] == "vacuum" || forms[0] == "hoover" {
                t = if said.contains("vacuum") { "vacuum cleaner".to_string() } else { "hoover".to_string() };
            }
            thing = Some(t);
            break;
        }
    }
    let subject = match &thing {
        Some(t) => {
            let article = if t.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
            text::capitalize(&format!("{article} {t}"))
        }
        None => "It".to_string(),
    };
    Some(format!(
        "{subject} is worth it if you would use it three \
         times a week or more and it saves a job you dislike; skip it otherwise, because it \
         takes up a cupboard. Buy the cheap one first, because most of these are replaced by \
         a better model you only learn you wanted after a year of using the first."
    ))
}

/// A tip for the machine they already have, or a verdict when it is the machine being judged.
pub fn _gadget_view(_memory: &mut Memory, understood: &Understood) -> String {
    let mut said = understood.words.join(" ").to_lowercase();
    if said.is_empty() {
        said = understood.text.to_lowercase();
    }
    if any_in(&said, &_VERDICT_ASK) {
        return _THE_VERDICT.clone();
    }
    for (forms, tip) in _GADGET_TIPS.iter() {
        if any_in(&said, forms) {
            return tip.clone();
        }
    }
    _THE_VERDICT.clone()
}

/// A reply value: a string, or one of this module's functions (or a closure of `_asked`).
fn call_reply_value(reply: &Value, memory: &mut Memory, understood: &Understood) -> Option<String> {
    match reply {
        Value::Str(s) => Some(s.to_string()),
        Value::Fn(f) => match f.name() {
            "bot.knowledge.opinions:_asked.<locals>.answer" => {
                _asked_answer(f.cell("reply").unwrap_or(&Value::None), memory, understood)
            }
            "bot.knowledge.opinions:_best_season" => Some(_best_season(memory, understood)),
            "bot.knowledge.opinions:_gadget_view" => Some(_gadget_view(memory, understood)),
            "bot.knowledge.opinions:_start_instrument" => Some(_start_instrument(memory, understood)),
            "bot.knowledge.opinions:_cats_or_dogs" => Some(_cats_or_dogs(memory, understood)),
            "bot.knowledge.opinions:_tea_or_coffee" => Some(_tea_or_coffee(memory, understood)),
            _ => {
                let got = crate::knowledge::registry::reply(f, memory, understood);
                if got.is_empty() { None } else { Some(got) }
            }
        },
        Value::None => None,
        // Python returns any other value as it is; the registry treats a non-str as declining.
        _ => None,
    }
}

/// `_asked(reply).<locals>.answer`: a reply, but only to a line that asks; a statement about
/// the same subject passes.
pub fn _asked_answer(reply: &Value, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let words = word_set(understood);
    let text = understood.text.to_lowercase();
    if EXPLAINS.match_(&text).is_some() && !names_you(&words, false) {
        return None;
    }
    if understood.kind == "question" {
        return call_reply_value(reply, memory, understood);
    }
    let line = understood.words.join(" ");
    let full = &text;
    if any_in(&line, &INTENT) || any_in(full, &INTENT) {
        return call_reply_value(reply, memory, understood);
    }
    if OWN_OPINION.is_match(full) && !names_you(&words, true) {
        return None;
    }
    if words.iter().any(|w| ASK_MARKERS.contains(*w)) {
        return call_reply_value(reply, memory, understood);
    }
    if understood.words.len() <= 6 && !words.iter().any(|w| FIRST_PERSON.contains(*w)) {
        return call_reply_value(reply, memory, understood);
    }
    None
}

/// True when the line compares two things: a choice word, or a question that picks one.
pub fn _is_a_choice(understood: &Understood) -> bool {
    if understood.words.iter().any(|w| CHOICE.contains(w)) {
        return true;
    }
    let text = format!(" {} ", understood.words.join(" ").to_lowercase());
    any_in(&text, &CHOICE_ASKS)
}

/// True when the line names one of these things, in any of the ways people write it.
pub fn _names<S: AsRef<str>>(understood: &Understood, options: &[S]) -> bool {
    let joined = understood.words.join(" ");
    _names_in(if joined.is_empty() { &understood.text } else { &joined }, options)
}

/// `_names({"text": text}, options)`: the same test on a bare line.
pub fn _names_in<S: AsRef<str>>(line: &str, options: &[S]) -> bool {
    let text = format!(" {} ", line.to_lowercase());
    any_in(&text, options)
}

/// `_either(id, a, b, reply).<locals>.answer`: the pick between two everyday things, in either
/// order, only when the line is a choice.
pub fn _either_answer(a: &[String], b: &[String], reply: &Value, memory: &mut Memory, understood: &Understood) -> Option<String> {
    if !_is_a_choice(understood) {
        return None;
    }
    if !(_names(understood, a) && _names(understood, b)) {
        return None;
    }
    if let Some(said) = _asked_answer(reply, memory, understood) {
        return Some(said);
    }
    let words = word_set(understood);
    let text = understood.text.to_lowercase();
    if !CHOICE_IN_WORDS.is_match(&text) {
        return None;
    }
    if words.iter().any(|w| FIRST_PERSON.contains(*w)) && !words.iter().any(|w| CHOICE_ASK_WORDS.contains(*w)) {
        return None;
    }
    if OWN_OPINION.is_match(&text) && !names_you(&words, true) {
        return None;
    }
    call_reply_value(reply, memory, understood)
}

fn either_reply(f: &FnRef, memory: &mut Memory, understood: &Understood) -> Option<String> {
    let a = f.cell("a").map(|v| v.strs()).unwrap_or_default();
    let b = f.cell("b").map(|v| v.strs()).unwrap_or_default();
    _either_answer(&a, &b, f.cell("reply").unwrap_or(&Value::None), memory, understood)
}

fn asked_reply(f: &FnRef, memory: &mut Memory, understood: &Understood) -> Option<String> {
    _asked_answer(f.cell("reply").unwrap_or(&Value::None), memory, understood)
}

/// True when the bot's last turn was already about learning an instrument.
pub fn _already_on_instruments(memory: &Memory) -> bool {
    let Some(last) = memory.turns.last() else { return false };
    let said = last.bot.to_lowercase();
    said.contains("15–20 minutes") || said.contains("learning an instrument") || said.contains("four chords")
}

fn lookback(memory: &Memory) -> &[crate::state::Turn] {
    let turns = &memory.turns;
    &turns[turns.len().saturating_sub(*INSTRUMENT_LOOKBACK)..]
}

/// True when this line, or a line or two back, is about drums rather than a guitar.
pub fn _on_drums(memory: &Memory, understood: &Understood) -> bool {
    if _names(understood, &DRUMS) {
        return true;
    }
    if _names(understood, &OTHER_INSTRUMENTS) {
        return false;
    }
    for turn in lookback(memory).iter().rev() {
        let said = turn.bot.to_lowercase();
        if said.contains("practice pad") {
            return true;
        }
        if said.contains("basic beat") {
            return true;
        }
        if _names_in(&turn.user, &DRUMS) {
            return true;
        }
        if _names_in(&turn.user, &OTHER_INSTRUMENTS) {
            return false;
        }
    }
    false
}

/// Tea or coffee: a taste, unless the line asks which one is better for you.
pub fn _tea_or_coffee(_memory: &mut Memory, understood: &Understood) -> String {
    if _names(understood, &HEALTH_ASK) {
        return "Neither is bad for you, and the health difference is small: tea has about half \
                the caffeine of coffee, both count towards your fluids, and green tea and filter \
                coffee have the most evidence behind them. What matters far more is how much \
                sugar and milk goes in."
            .to_string();
    }
    "Tea, most of the time: it's a better excuse to stop for five minutes. Coffee when \
     you actually need to be awake in a hurry."
        .to_string()
}

/// Which of the instrument answers this line is about, or None for one we do not cover.
pub fn _named_instrument(understood: &Understood) -> Option<&'static Instrument> {
    INSTRUMENTS.iter().find(|table| _names(understood, &table.names))
}

/// The instrument this conversation is about, the named one first, then the last answer's.
pub fn _instrument_table(memory: &Memory, understood: &Understood) -> &'static Instrument {
    if let Some(named) = _named_instrument(understood) {
        return named;
    }
    if _names(understood, &OTHER_INSTRUMENTS) {
        return &INSTRUMENT_GENERIC;
    }
    for turn in lookback(memory).iter().rev() {
        let said = turn.bot.to_lowercase();
        for table in INSTRUMENTS.iter() {
            if said.contains(table.hint.as_str()) {
                return table;
            }
        }
        if !turn.user.is_empty() {
            for table in INSTRUMENTS.iter() {
                if _names_in(&turn.user, &table.names) {
                    return table;
                }
            }
        }
    }
    &INSTRUMENT_GENERIC
}

/// True when this line, or something said a line or two back, names an instrument.
pub fn _an_instrument_is_in_play(memory: &Memory, understood: &Understood) -> bool {
    if _names(understood, &INSTRUMENT_WORDS) {
        return true;
    }
    // Python: a memory with no "turns" list is False here; an empty list reads " ".
    let mut said = " ".to_string();
    for turn in lookback(memory) {
        said += &format!("{} {} ", turn.user, turn.bot);
    }
    let said = said.to_lowercase();
    any_in(&said, &INSTRUMENT_WORDS)
}

/// What to practise in the first week, for the instrument this line (or the last one) named.
pub fn _practise_first(memory: &mut Memory, understood: &Understood) -> Option<String> {
    if _on_drums(memory, understood) {
        return Some(
            "Single strokes first, slow and even, ten minutes a day on a practice pad or a \
             cushion: full strokes up to about a foot, then back down. Get that steady, then \
             a basic beat — hi-hat on the eighths, snare on two and four, kick on one and \
             three — and play along to a song you like. Speed comes last."
                .to_string(),
        );
    }
    let table = _instrument_table(memory, understood);
    if table.generic && !_an_instrument_is_in_play(memory, understood) {
        return None;
    }
    Some(table.practise.clone())
}

/// The drums version of the adult-beginner take: a beat in weeks, no kit to start.
pub fn _drums_start(too_late: bool) -> String {
    let first = if too_late { "Not too late at all. " } else { "" };
    format!(
        "{first}Ten minutes a day on a practice pad gets you a steady basic beat in about \
         six weeks: single strokes slow and even, then hi-hat on the eighths, snare on two \
         and four, kick on one and three. Play along to songs you like before you buy a kit, \
         and take one lesson at the start so your grip is right — a pad, or a cushion and a \
         pair of sticks, is enough to begin."
    )
}

/// `s[0].upper() + s[1:]`.
fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        // Python: an IndexError on an empty string
        None => panic!("string index out of range"),
    }
}

/// An adult who always wanted to learn: the first month, and the reassurance when it is asked.
pub fn _start_instrument(memory: &mut Memory, understood: &Understood) -> String {
    if _on_drums(memory, understood) {
        if _already_on_instruments(memory) {
            return "Then start this week: pad and sticks, ten minutes a day, one lesson at the \
                    start, and one song you like to play along to. Speed comes last — slow and \
                    even beats fast and ragged every time."
                .to_string();
        }
        return _drums_start(_names(understood, &TOO_LATE));
    }
    let table = _instrument_table(memory, understood);
    let start = &table.start;
    if _already_on_instruments(memory) || !_names(understood, &TOO_LATE) {
        return upper_first(start);
    }
    format!(
        "Not too late at all: adults pick up the theory faster than children, they just need \
         shorter and more regular practice. {}",
        upper_first(start)
    )
}

/// The follow-up after the instrument: how long until something recognisable.
pub fn _how_long_song(memory: &mut Memory, understood: &Understood) -> String {
    if _on_drums(memory, understood) {
        return "About six weeks of ten minutes a day for a steady basic beat, and three or four \
                months before it holds together with a song at full speed. Play along to songs you \
                like before you buy a kit."
            .to_string();
    }
    _instrument_table(memory, understood).how_long.clone()
}

/// How often to practise, for the instrument this line (or the last one) named.
pub fn _how_often_at_it(memory: &mut Memory, understood: &Understood) -> String {
    if _on_drums(memory, understood) {
        return "Ten minutes a day, five or six days a week, on a practice pad: single strokes \
                slow and even first, then the basic beat played along to a song. The hands want \
                the day off more than they want the extra twenty minutes."
            .to_string();
    }
    let table = _instrument_table(memory, understood);
    if table.generic && !_an_instrument_is_in_play(memory, understood) {
        return INSTRUMENT_GENERIC.how_often.clone();
    }
    table.how_often.clone()
}

/// A taste on the seasons, plus the way through it when the line says it cannot decide.
pub fn _best_season(_memory: &mut Memory, understood: &Understood) -> String {
    let taste = "Late spring for me: long evenings, everything green, and not hot enough to make \
                 excuses.";
    if _names(understood, &CANT_DECIDE) {
        return format!(
            "{taste} Since you cannot settle on one, pick by what you would do in it: \
             autumn for walking, summer for eating outside, winter for staying in."
        );
    }
    taste.to_string()
}

/// Cats or dogs: by the reason the line gives, or a whisker for the dog.
pub fn _cats_or_dogs(_memory: &mut Memory, understood: &Understood) -> String {
    if _names(understood, &OUT_ALL_DAY) {
        return "A cat, if you're out all day: it sleeps through your shift and does not need a \
                walk at seven in the morning. Get an adult cat rather than a kitten, and two if \
                you are away a lot, so they have company."
            .to_string();
    }
    if _names(understood, &TRAIN_WORDS) {
        return TRAIN_COMPARE.clone();
    }
    if _names(understood, &CAT_WINS) {
        return CAT_ON_A_DIMENSION.clone();
    }
    if _names(understood, &DOG_WINS) {
        return DOG_ON_A_DIMENSION.clone();
    }
    "Dogs, by a whisker: daft, straightforward and pleased to see you. A cat is the kinder \
     choice if you're out all day, though."
        .to_string()
}

/// True when the line holds one of these as a whole word.
pub fn _says_one_of<S: AsRef<str>>(text: &str, options: &[S]) -> bool {
    options.iter().any(|o| Re::new(&format!(r"\b{}\b", escape(o.as_ref())), 0).is_match(text))
}

fn says_one_of(text: &str, options: &[Re]) -> bool {
    options.iter().any(|rx| rx.is_match(text))
}

/// True when the line does not name another row's cat question.
pub fn _not_another_cats_question(understood: &Understood) -> bool {
    let line = if understood.text.is_empty() { &understood.clean } else { &understood.text };
    let said = if line.is_empty() { understood.words.join(" ") } else { line.clone() };
    !_ANOTHER_CATS_QUESTION.is_match(&said)
}

/// The trade that settles the choice the line names, or None when it names none.
pub fn _deciding(memory: &mut Memory, understood: &Understood) -> Option<String> {
    let joined = understood.words.join(" ");
    let line = if joined.is_empty() { understood.text.clone() } else { joined };
    let text = format!(" {} ", text::squash(&line.to_lowercase()));
    if A_NEW_ONE.is_match(&text) {
        return None;
    }
    if _WHAT_TO_BUY.is_match(&text) && says_one_of(&text, &CAT_WORDS) {
        return None;
    }
    if _TWO_OF_THEM.is_match(&text) && !BETWEEN.is_match(&text) {
        return None;
    }
    if MADE_UP.is_match(&text) && !STILL_DECIDING.is_match(&text) {
        return None;
    }
    if says_one_of(&text, &CAT_OR_KITTEN) && says_one_of(&text, &DOG_OR_PUPPY) {
        return Some(_cats_or_dogs(memory, understood));
    }
    if says_one_of(&text, &TWO_WAYS[0]) && says_one_of(&text, &TWO_WAYS[1]) {
        return None;
    }
    if says_one_of(&text, &CAT_WORDS) {
        if says_one_of(&text, &CAT_EQUIPMENT) {
            return None;
        }
        return Some(CAT_TRADE.clone());
    }
    if says_one_of(&text, &QUALIFIED) {
        return None;
    }
    for (options, trade) in DECISION_SUBJECTS.iter() {
        if says_one_of(&text, options) {
            return Some(trade.clone());
        }
    }
    None
}

/// The sentence for the thing the line asked about, and nothing when it names no subject.
pub fn _the_normal_answer(_memory: &mut Memory, understood: &Understood) -> Option<String> {
    let line = if understood.text.is_empty() { &understood.clean } else { &understood.text };
    let text = format!(" {} ", line.to_lowercase());
    for (words, reply) in _NORMAL_SUBJECTS.iter() {
        if any_in(&text, words) {
            return Some(reply.clone());
        }
    }
    None
}

/// True when the split in the line is a bill at a table, or asks whether it is rude.
pub fn _a_table_bills_line(understood: &Understood) -> bool {
    let mut text = understood.words.join(" ").to_lowercase();
    if text.is_empty() {
        let line = if understood.clean.is_empty() { &understood.text } else { &understood.clean };
        text = line.to_lowercase();
    }
    if _A_SHARED_HOME.is_match(&text) {
        return false;
    }
    _A_MANNERS_WORD.is_match(&text) || _A_TABLE_BILL.is_match(&text)
}

pub const GUARDS: &[(&str, GuardFn)] = &[
    ("bot.knowledge.opinions:<lambda>:1280", |_: &FnRef, u: &Understood| !asks_where_a_shop_answers(u)),
    ("bot.knowledge.opinions:<lambda>:1395", |_: &FnRef, u: &Understood| !the_book_is_a_reservation(u)),
    ("bot.knowledge.opinions:_a_table_bills_line:1119", |_: &FnRef, u: &Understood| _a_table_bills_line(u)),
    ("bot.knowledge.opinions:_not_another_cats_question:986", |_: &FnRef, u: &Understood| {
        _not_another_cats_question(u)
    }),
];

pub const REPLIES: &[(&str, ReplyFn)] = &[
    ("bot.knowledge.opinions:_asked.<locals>.answer:207", asked_reply),
    ("bot.knowledge.opinions:_deciding:996", |_: &FnRef, m: &mut Memory, u: &Understood| _deciding(m, u)),
    ("bot.knowledge.opinions:_either.<locals>.answer:358", either_reply),
    ("bot.knowledge.opinions:_how_long_song:699", |_: &FnRef, m: &mut Memory, u: &Understood| Some(_how_long_song(m, u))),
    ("bot.knowledge.opinions:_how_often_at_it:708", |_: &FnRef, m: &mut Memory, u: &Understood| {
        Some(_how_often_at_it(m, u))
    }),
    ("bot.knowledge.opinions:_practise_first:657", |_: &FnRef, m: &mut Memory, u: &Understood| _practise_first(m, u)),
    ("bot.knowledge.opinions:_the_normal_answer:1109", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _the_normal_answer(m, u)
    }),
    ("bot.knowledge.opinions:_worth_an_appliance:64", |_: &FnRef, m: &mut Memory, u: &Understood| {
        _worth_an_appliance(m, u)
    }),
];
